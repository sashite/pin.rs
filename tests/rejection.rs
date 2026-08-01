//! Rejection tests for PIN v1.0.0.
//!
//! `conformance.rs` proves the parser accepts exactly the right *set* of
//! strings. This file pins down *which* [`ParseError`] each class of malformed
//! input yields, so the documented (deliberately coarse) error mapping cannot
//! drift, and verifies every public entry point rejects identically.

use sashite_pin::{Identifier, ParseError};

/// Malformed inputs paired with the exact error they must produce.
const REJECTED: &[(&str, ParseError)] = &[
    // Empty.
    ("", ParseError::Empty),
    // Length 1 — a lone non-letter cannot be an abbreviation.
    ("+", ParseError::InvalidLetter),
    ("-", ParseError::InvalidLetter),
    ("^", ParseError::InvalidLetter),
    ("1", ParseError::InvalidLetter),
    (" ", ParseError::InvalidLetter),
    ("\n", ParseError::InvalidLetter),
    // Length 2 — `<letter>` must be followed by the terminal marker.
    ("K+", ParseError::InvalidTerminalMarker),
    ("Kk", ParseError::InvalidTerminalMarker),
    ("K ", ParseError::InvalidTerminalMarker),
    ("K\n", ParseError::InvalidTerminalMarker),
    // Length 2 — `<modifier>` must be followed by a letter.
    ("+^", ParseError::InvalidLetter),
    ("-^", ParseError::InvalidLetter),
    ("++", ParseError::InvalidLetter),
    ("+1", ParseError::InvalidLetter),
    // Length 2 — leading byte is neither a letter nor a modifier.
    ("^K", ParseError::InvalidStateModifier),
    ("^^", ParseError::InvalidStateModifier),
    (" K", ParseError::InvalidStateModifier),
    ("1K", ParseError::InvalidStateModifier),
    ("\nK", ParseError::InvalidStateModifier),
    // Length 3 — the only valid shape is `<modifier><letter><terminal>`.
    ("+KK", ParseError::InvalidTerminalMarker),
    ("+K+", ParseError::InvalidTerminalMarker),
    ("+K ", ParseError::InvalidTerminalMarker),
    ("++^", ParseError::InvalidLetter),
    ("+^K", ParseError::InvalidLetter),
    ("K^^", ParseError::InvalidStateModifier),
    ("K^K", ParseError::InvalidStateModifier),
    ("KKK", ParseError::InvalidStateModifier),
    (" K^", ParseError::InvalidStateModifier),
    ("^K^", ParseError::InvalidStateModifier),
    // Length ≥ 4 — rejected on the structural length check.
    ("KKKK", ParseError::TooLong),
    ("+K^^", ParseError::TooLong),
    ("+K^x", ParseError::TooLong),
    ("abcd", ParseError::TooLong),
    ("    ", ParseError::TooLong),
    ("hello world", ParseError::TooLong),
    // Non-ASCII: multi-byte characters never form a valid token.
    ("é", ParseError::InvalidStateModifier),  // 2 bytes
    ("♔", ParseError::InvalidStateModifier),  // 3 bytes
    ("Ké", ParseError::InvalidStateModifier), // letter + 2-byte char = 3 bytes
    ("🨀", ParseError::TooLong),               // 4 bytes
];

#[test]
fn every_entry_point_agrees_on_rejection() {
    for &(input, expected) in REJECTED {
        assert_eq!(Identifier::parse(input), Err(expected), "parse {input:?}");
        // `str::parse` exercises the `FromStr` implementation.
        assert_eq!(
            input.parse::<Identifier>(),
            Err(expected),
            "FromStr {input:?}"
        );
        assert_eq!(
            Identifier::try_from(input),
            Err(expected),
            "TryFrom {input:?}"
        );
        assert!(!Identifier::is_valid(input), "is_valid {input:?}");
    }
}

#[test]
fn spec_cited_non_canonical_forms_are_rejected() {
    // The four shapes the specification explicitly calls out as invalid in its
    // canonical-form section.
    assert_eq!(
        Identifier::parse("K+"),
        Err(ParseError::InvalidTerminalMarker)
    );
    assert_eq!(
        Identifier::parse("^K"),
        Err(ParseError::InvalidStateModifier)
    );
    assert_eq!(Identifier::parse("++K"), Err(ParseError::InvalidLetter));
    assert_eq!(
        Identifier::parse("K^^"),
        Err(ParseError::InvalidStateModifier)
    );
}

/// The one byte class the grammar recognises outside the abbreviation and the
/// terminal marker, named so the property below reads like the specification.
fn is_state_modifier(byte: u8) -> bool {
    byte == b'+' || byte == b'-'
}

/// Checks that a rejection blames a position whose byte is genuinely wrong
/// there, and reports whether the slice was rejected at all.
fn blames_a_position_that_is_genuinely_wrong(bytes: &[u8]) -> bool {
    let Err(error) = Identifier::try_from(bytes) else {
        return false;
    };
    match error {
        ParseError::Empty => assert!(bytes.is_empty(), "{bytes:?}"),
        ParseError::TooLong => assert!(bytes.len() > 3, "{bytes:?}"),
        ParseError::InvalidLetter => {
            // The abbreviation is the whole one-byte token, and byte 1 once a
            // state modifier has been consumed ahead of it.
            let abbr = usize::from(bytes.len() > 1);
            assert!(!bytes[abbr].is_ascii_alphabetic(), "{bytes:?}");
            assert!(abbr == 0 || is_state_modifier(bytes[0]), "{bytes:?}");
        }
        ParseError::InvalidStateModifier => {
            // Blamed only when byte 0 can open no shape of this length: never a
            // modifier, and never a letter either while a two-byte
            // `<abbr><terminal>` was still on the table.
            assert!(!is_state_modifier(bytes[0]), "{bytes:?}");
            assert!(
                bytes.len() != 2 || !bytes[0].is_ascii_alphabetic(),
                "{bytes:?}"
            );
        }
        ParseError::InvalidTerminalMarker => {
            let (last, head) = bytes.split_last().expect("a blamed position exists");
            assert_ne!(*last, b'^', "{bytes:?}");
            // What the parser had already accepted really was a well-formed
            // `[<modifier>]<abbr>` prefix, so the marker is the only complaint.
            match head {
                [letter] => assert!(letter.is_ascii_alphabetic(), "{bytes:?}"),
                [modifier, letter] => {
                    assert!(is_state_modifier(*modifier), "{bytes:?}");
                    assert!(letter.is_ascii_alphabetic(), "{bytes:?}");
                }
                other => panic!("unexpected accepted prefix {other:?} in {bytes:?}"),
            }
        }
        // `ParseError` is `#[non_exhaustive]`. A variant added later must be
        // given its own position rule here rather than slip through unexamined.
        other => panic!("unclassified variant {other:?} for {bytes:?}"),
    }
    true
}

/// Every rejection blames a position whose byte is genuinely wrong there.
///
/// `ParseError`'s documentation says a variant names the *position* that
/// failed, not the most striking oddity of the input. That is a checkable
/// claim, not merely a description: whenever the parser blames a position, the
/// byte there must really be unusable, and the bytes the parser had already
/// accepted must really have been acceptable. The table above fixes that
/// mapping on 40 hand-picked inputs; this sweeps every byte slice of length 0
/// to 3 — the whole space in which acceptance is even possible — so a future
/// rearrangement of the parser's branches cannot start blaming the wrong byte
/// on some input nobody thought to list.
#[test]
fn every_rejection_blames_a_position_that_is_genuinely_wrong() {
    let mut buf = [0u8; 3];
    let mut rejected = 0_u32;

    rejected += u32::from(blames_a_position_that_is_genuinely_wrong(&[]));
    for first in 0..=u8::MAX {
        buf[0] = first;
        rejected += u32::from(blames_a_position_that_is_genuinely_wrong(&buf[..1]));
        for second in 0..=u8::MAX {
            buf[1] = second;
            rejected += u32::from(blames_a_position_that_is_genuinely_wrong(&buf[..2]));
            for third in 0..=u8::MAX {
                buf[2] = third;
                rejected += u32::from(blames_a_position_that_is_genuinely_wrong(&buf[..3]));
            }
        }
    }

    // 1 + 256 + 256² + 256³ slices, of which exactly the 312 tokens survive.
    assert_eq!(rejected, 16_843_009 - 312);
}

#[test]
fn error_messages_are_nonempty_distinct_and_usable_as_std_error() {
    let variants = [
        ParseError::Empty,
        ParseError::TooLong,
        ParseError::InvalidLetter,
        ParseError::InvalidStateModifier,
        ParseError::InvalidTerminalMarker,
    ];

    let mut messages: Vec<String> = variants.iter().map(ToString::to_string).collect();
    assert!(messages.iter().all(|m| !m.is_empty()));

    messages.sort();
    messages.dedup();
    assert_eq!(
        messages.len(),
        variants.len(),
        "Display messages must be distinct"
    );

    // The error type integrates with the standard error trait.
    let as_error: &dyn std::error::Error = &ParseError::Empty;
    assert!(!as_error.to_string().is_empty());
}
