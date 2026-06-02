//! Conformance tests for PIN v1.0.0.
//!
//! Two guarantees are checked here:
//!
//! 1. every one of the 312 canonical tokens round-trips and exposes the
//!    attributes its spelling implies, and
//! 2. the hand-written parser agrees with the specification's anchored regular
//!    expression on an exhaustive sweep of short inputs.

use regex::Regex;
use sashite_pin::{Identifier, Side, State};

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
