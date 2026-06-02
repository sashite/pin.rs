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
