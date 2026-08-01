//! Conformance tests for PIN v1.0.0.
//!
//! Three guarantees are checked here:
//!
//! 1. every one of the 312 canonical tokens round-trips and exposes the
//!    attributes its spelling implies,
//! 2. the hand-written parser agrees with the specification's anchored regular
//!    expression on an exhaustive sweep of short inputs, and
//! 3. every public entry point — `parse`, `FromStr`, `TryFrom<&str>`,
//!    `TryFrom<&[u8]>` and `is_valid` — agrees with the others on that same
//!    sweep, down to the error variant.
//!
//! The regular expression is written with whole-string anchors (`\A`…`\z`)
//! rather than `^`…`$`, because in a multi-line-capable engine `$` also matches
//! before a trailing newline. Anchoring with `^`…`$` would make the oracle
//! itself accept `"K\n"`, and the sweeps below would then have "confirmed" a
//! parser bug rather than caught one.

use std::collections::BTreeSet;

use regex::bytes::Regex as ByteRegex;
use regex::Regex;
use sashite_pin::{Identifier, ParseError, Side, State};

/// A canonical token paired with the attributes it must decode to.
struct Token {
    text: String,
    letter: char,
    side: Side,
    state: State,
    terminal: bool,
}

/// Generates all 312 canonical tokens: 26 letters × 2 sides × 3 states ×
/// 2 terminal flags.
fn every_token() -> Vec<Token> {
    let mut tokens = Vec::with_capacity(312);
    for upper in b'A'..=b'Z' {
        for (side, byte) in [(Side::First, upper), (Side::Second, upper + 32)] {
            for (state, prefix) in [
                (State::Normal, ""),
                (State::Enhanced, "+"),
                (State::Diminished, "-"),
            ] {
                for (terminal, suffix) in [(false, ""), (true, "^")] {
                    tokens.push(Token {
                        text: format!("{prefix}{}{suffix}", byte as char),
                        letter: upper as char,
                        side,
                        state,
                        terminal,
                    });
                }
            }
        }
    }
    tokens
}

#[test]
fn the_closed_domain_has_312_tokens() {
    let tokens = every_token();
    assert_eq!(tokens.len(), 312);

    // The generated set must be free of duplicates.
    let mut texts: Vec<&str> = tokens.iter().map(|t| t.text.as_str()).collect();
    texts.sort_unstable();
    texts.dedup();
    assert_eq!(texts.len(), 312);
}

#[test]
fn every_token_round_trips_and_decodes_correctly() {
    for token in every_token() {
        let id = Identifier::parse(&token.text)
            .unwrap_or_else(|e| panic!("parse {:?} failed: {e:?}", token.text));

        // Attributes match the spelling.
        assert_eq!(id.letter().as_char(), token.letter, "{:?}", token.text);
        assert_eq!(id.side(), token.side, "{:?}", token.text);
        assert_eq!(id.state(), token.state, "{:?}", token.text);
        assert_eq!(id.is_terminal(), token.terminal, "{:?}", token.text);

        // Round-trips through both encode() and Display.
        assert_eq!(id.encode().as_str(), token.text);
        assert_eq!(id.to_string(), token.text);
        assert!(Identifier::is_valid(&token.text));

        // Rebuilding from components yields the identical value.
        let rebuilt = Identifier::new(id.letter(), id.side(), id.state(), id.is_terminal());
        assert_eq!(rebuilt, id);
        assert_eq!(rebuilt.encode().as_str(), token.text);
    }
}

/// The specification regex, written with whole-string anchors (`\A`…`\z`) so it
/// cannot match across a trailing newline, matching PIN's normative anchoring
/// requirement.
fn spec_regex() -> Regex {
    Regex::new(r"\A[+-]?[A-Za-z]\^?\z").expect("valid regex")
}

fn assert_agreement(s: &str, re: &Regex) {
    assert_eq!(
        Identifier::is_valid(s),
        re.is_match(s),
        "parser and spec regex disagree on {s:?}",
    );
}

#[test]
fn parser_matches_spec_regex_over_all_ascii_inputs() {
    let re = spec_regex();
    let mut buf = [0u8; 3];

    assert_agreement("", &re);

    for a in 0u8..=127 {
        buf[0] = a;
        assert_agreement(std::str::from_utf8(&buf[..1]).unwrap(), &re);
        for b in 0u8..=127 {
            buf[1] = b;
            assert_agreement(std::str::from_utf8(&buf[..2]).unwrap(), &re);
            for c in 0u8..=127 {
                buf[2] = c;
                assert_agreement(std::str::from_utf8(&buf[..3]).unwrap(), &re);
            }
        }
    }
}

#[test]
fn parser_matches_spec_regex_on_non_ascii_and_long_inputs() {
    let re = spec_regex();
    for s in [
        "é",    // 2-byte char, single grapheme
        "K♔",   // letter followed by a multi-byte char
        "♔",    // lone multi-byte char
        "KKKK", // four ASCII letters
        "+K^x", // valid token plus trailing junk
        "  ",   // whitespace only
        "\n",   // bare newline
        "K\n",  // letter then newline (must not match via `$`)
    ] {
        assert_agreement(s, &re);
    }
}

/// The same specification regex, compiled to match raw bytes. Unicode mode is
/// switched off (`(?-u)`) so the class `[A-Za-z]` denotes exactly the 52 ASCII
/// letter *bytes*, which is what the byte-slice entry point sees.
fn spec_byte_regex() -> ByteRegex {
    ByteRegex::new(r"(?-u)\A[+-]?[A-Za-z]\^?\z").expect("valid regex")
}

/// Bytes worth combining exhaustively: every byte that may legally appear in a
/// token, one representative of each rejected ASCII family, both ends of the
/// ASCII range, and continuation/leading bytes of multi-byte UTF-8 sequences.
const INTERESTING_BYTES: &[u8] = &[
    0x00, b'\n', b' ', b'+', b'-', b'0', b'A', b'K', b'Z', b'^', b'a', b'k', b'z', 0x7F, 0x80,
    0xC3, 0xE2, 0xF0, 0xFF,
];

/// Checks one raw byte slice against the specification regex through every
/// public entry point, and reports whether it was accepted.
///
/// The byte entry point is the reference here because it is the only one that
/// sees inputs that are not valid UTF-8. Whenever the slice *is* valid UTF-8,
/// all four string entry points must return the identical `Result` — the same
/// `Ok` value or the same `ParseError` variant, not merely the same verdict.
fn agrees_on_bytes(bytes: &[u8], re: &ByteRegex) -> bool {
    let from_bytes = Identifier::try_from(bytes);
    assert_eq!(
        from_bytes.is_ok(),
        re.is_match(bytes),
        "parser and spec regex disagree on {bytes:?}",
    );

    if let Ok(text) = std::str::from_utf8(bytes) {
        assert_eq!(Identifier::parse(text), from_bytes, "parse {text:?}");
        assert_eq!(text.parse::<Identifier>(), from_bytes, "FromStr {text:?}");
        assert_eq!(Identifier::try_from(text), from_bytes, "TryFrom {text:?}");
        assert_eq!(
            Identifier::is_valid(text),
            from_bytes.is_ok(),
            "is_valid {text:?}",
        );
    } else {
        // A slice that is not valid UTF-8 can never be a token, since every
        // token byte is ASCII.
        assert!(from_bytes.is_err(), "non-UTF-8 {bytes:?} must be rejected");
    }

    // An accepted input must re-encode to itself, byte for byte.
    if let Ok(id) = from_bytes {
        assert_eq!(id.encode().as_str().as_bytes(), bytes, "round trip");
    }
    from_bytes.is_ok()
}

/// Exhaustive sweep of the entire input space the parser can accept from.
///
/// Every one of the 16 843 009 byte slices of length 0, 1, 2 and 3 is checked —
/// not just the valid UTF-8 ones, because `TryFrom<&[u8]>` performs no UTF-8
/// validation and so must be correct on arbitrary bytes. Since a token is at
/// most three bytes, this covers the whole space in which acceptance is even
/// possible; everything longer is handled by the test below.
///
/// The sweep ends by asserting that the accepted set is *exactly* the 312
/// canonical tokens. That is the strongest statement available about the closed
/// domain, and it also guards the oracle: a regex that had been mis-transcribed
/// into something over- or under-permissive would change this count.
#[test]
fn every_byte_slice_up_to_three_bytes_agrees_with_the_spec() {
    let re = spec_byte_regex();
    let mut accepted: BTreeSet<String> = BTreeSet::new();
    let mut buf = [0u8; 3];

    let record = |bytes: &[u8], accepted: &mut BTreeSet<String>| {
        if agrees_on_bytes(bytes, &re) {
            accepted.insert(String::from_utf8(bytes.to_vec()).expect("a token is ASCII"));
        }
    };

    record(&[], &mut accepted);
    for first in 0..=u8::MAX {
        buf[0] = first;
        record(&buf[..1], &mut accepted);
        for second in 0..=u8::MAX {
            buf[1] = second;
            record(&buf[..2], &mut accepted);
            for third in 0..=u8::MAX {
                buf[2] = third;
                record(&buf[..3], &mut accepted);
            }
        }
    }

    let canonical: BTreeSet<String> = every_token().into_iter().map(|t| t.text).collect();
    assert_eq!(canonical.len(), 312);
    assert_eq!(
        accepted, canonical,
        "the accepted set must be exactly the 312 canonical tokens",
    );
}

/// Nothing four bytes or longer is ever accepted, whatever it holds.
///
/// The parser dispatches on length before looking at content: a slice of four
/// or more bytes reaches the catch-all arm and returns `TooLong` without a
/// single byte being inspected. That makes the property content-independent, so
/// an exhaustive 2^32 sweep would add no information. What is swept instead is
/// every combination over the interesting-byte alphabet, plus every one of the
/// 256 byte values in each of the four positions, plus longer inputs.
#[test]
fn nothing_four_bytes_or_longer_is_accepted() {
    let re = spec_byte_regex();

    for &a in INTERESTING_BYTES {
        for &b in INTERESTING_BYTES {
            for &c in INTERESTING_BYTES {
                for &d in INTERESTING_BYTES {
                    let buf = [a, b, c, d];
                    assert!(!re.is_match(&buf), "spec regex accepted {buf:?}");
                    assert_eq!(Identifier::try_from(&buf[..]), Err(ParseError::TooLong));
                }
            }
        }
    }

    // Every byte value in every position, against a background that is a valid
    // token with one byte appended.
    for position in 0..4 {
        for value in 0..=u8::MAX {
            let mut buf = *b"+K^K";
            buf[position] = value;
            assert!(!re.is_match(&buf), "spec regex accepted {buf:?}");
            assert_eq!(Identifier::try_from(&buf[..]), Err(ParseError::TooLong));
        }
    }

    // Longer inputs, including a valid token padded on either side.
    for text in [
        "+K^^",
        "+K^x",
        "KKKK",
        "abcd",
        "    ",
        "hello world",
        "\n\n\n\n",
        "  +K^",
        "+K^  ",
    ] {
        assert!(
            !re.is_match(text.as_bytes()),
            "spec regex accepted {text:?}"
        );
        assert_eq!(
            Identifier::parse(text),
            Err(ParseError::TooLong),
            "{text:?}"
        );
        assert_eq!(
            Identifier::try_from(text.as_bytes()),
            Err(ParseError::TooLong),
            "{text:?}",
        );
    }
}

/// Every Unicode scalar value, as a one-character string.
///
/// This is the exhaustive form of the "a character is not a byte" check: 1 112
/// 064 scalars, spanning all four UTF-8 encoded lengths. Only the 52 ASCII
/// letters may be accepted; everything else — including characters that *look*
/// like token syntax, such as U+FF2B FULLWIDTH LATIN CAPITAL LETTER K or
/// U+2212 MINUS SIGN — must be rejected.
#[test]
fn every_unicode_scalar_agrees_with_the_spec() {
    let re = spec_regex();
    let byte_re = spec_byte_regex();
    let mut buf = [0u8; 4];
    let mut accepted = 0_u32;

    for code_point in 0..=0x0010_FFFF_u32 {
        let Some(character) = char::from_u32(code_point) else {
            continue; // surrogate halves are not scalar values
        };
        let text: &str = character.encode_utf8(&mut buf);
        let matches = re.is_match(text);
        assert_eq!(Identifier::is_valid(text), matches, "U+{code_point:04X}");
        assert_eq!(
            Identifier::try_from(text.as_bytes()).is_ok(),
            matches,
            "U+{code_point:04X}",
        );
        // The byte-mode and Unicode-mode oracles must not disagree either.
        assert_eq!(byte_re.is_match(text.as_bytes()), matches);
        accepted += u32::from(matches);
    }

    assert_eq!(
        accepted, 52,
        "only the 26 uppercase and 26 lowercase ASCII letters stand alone as tokens",
    );
}

/// Characters that resemble token syntax but are not it, in every position.
///
/// The single-scalar sweep above proves none of these is a token on its own.
/// This one places them where a token's modifier, abbreviation and marker go,
/// which is where a parser that decoded characters instead of bytes, or that
/// applied a Unicode-aware case or symbol rule, would go wrong.
#[test]
fn confusable_characters_are_rejected_in_every_position() {
    let re = spec_regex();
    let confusables = [
        '\u{FF2B}',  // FULLWIDTH LATIN CAPITAL LETTER K
        '\u{212A}',  // KELVIN SIGN (uppercases/lowercases to ASCII 'k')
        '\u{FF0B}',  // FULLWIDTH PLUS SIGN
        '\u{FE62}',  // SMALL PLUS SIGN
        '\u{2212}',  // MINUS SIGN
        '\u{2010}',  // HYPHEN
        '\u{02C4}',  // MODIFIER LETTER UP ARROWHEAD
        '\u{2038}',  // CARET
        '\u{0301}',  // COMBINING ACUTE ACCENT
        '\u{00E9}',  // é
        '\u{2654}',  // WHITE CHESS KING
        '\u{1FA00}', // NEUTRAL CHESS KING
        '\u{FEFF}',  // ZERO WIDTH NO-BREAK SPACE (a BOM, if leading)
        '\u{0000}',  // NUL
    ];

    for confusable in confusables {
        for template in ["{}", "+{}", "{}^", "K{}", "{}K", "+{}^", "+K{}", "{}K^"] {
            let text = template.replace("{}", &confusable.to_string());
            assert!(
                !re.is_match(&text),
                "spec regex accepted {text:?} (U+{:04X})",
                confusable as u32,
            );
            assert!(
                !Identifier::is_valid(&text),
                "parser accepted {text:?} (U+{:04X})",
                confusable as u32,
            );
            assert!(Identifier::try_from(text.as_bytes()).is_err());
        }
    }
}
