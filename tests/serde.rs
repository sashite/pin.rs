//! Tests for the `serde` feature: an `Identifier` (de)serializes as its
//! canonical token string.
#![cfg(feature = "serde")]

use sashite_pin::{Identifier, Letter, Side, State};
use serde_test::{assert_de_tokens_error, assert_tokens, Token};

#[test]
fn serializes_and_deserializes_as_token_string() {
    // One token per structural shape: both sides, all three states, and the
    // terminal marker present or absent (including the prefix+suffix case).
    for token in ["K", "k", "+R", "-p", "K^", "+G^", "-k^"] {
        let id = Identifier::parse(token).unwrap();
        assert_tokens(&id, &[Token::Str(token)]);
    }
}

#[test]
fn every_identifier_survives_a_serde_round_trip() {
    // The seven shapes above cover the grammar, but the mapping from an
    // identifier to its string runs through `encode`, so it is worth walking
    // the whole closed domain rather than a sample of it.
    let mut count = 0_u32;
    for letter in Letter::ALL {
        for side in [Side::First, Side::Second] {
            for state in [State::Diminished, State::Normal, State::Enhanced] {
                for terminal in [false, true] {
                    let id = Identifier::new(letter, side, state, terminal);
                    // `Token::Str` holds a `&'static str`; leaking 312 tokens
                    // of at most three bytes inside a test is the cheapest way
                    // to satisfy that.
                    let encoded: &'static str = id.encode().as_str().to_owned().leak();
                    assert_tokens(&id, &[Token::Str(encoded)]);
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 312);
}

#[test]
fn invalid_strings_fail_to_deserialize() {
    // These strings are the `Display` text of the corresponding `ParseError`,
    // so they also pin that text against accidental rewording.
    assert_de_tokens_error::<Identifier>(&[Token::Str("")], "empty PIN token");
    assert_de_tokens_error::<Identifier>(
        &[Token::Str("K+")],
        "invalid PIN terminal marker (expected '^')",
    );
    assert_de_tokens_error::<Identifier>(
        &[Token::Str("++K")],
        "PIN abbreviation is not an ASCII letter",
    );
    assert_de_tokens_error::<Identifier>(
        &[Token::Str("^K")],
        "PIN token starts with neither a state modifier nor a letter",
    );
    // Measured in bytes: "🨀" is a single character but four bytes long.
    assert_de_tokens_error::<Identifier>(&[Token::Str("🨀")], "PIN token longer than three bytes");
}
