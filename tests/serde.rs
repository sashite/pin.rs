//! Tests for the `serde` feature: an `Identifier` (de)serializes as its
//! canonical token string.
#![cfg(feature = "serde")]

use sashite_pin::Identifier;
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
fn invalid_strings_fail_to_deserialize() {
    assert_de_tokens_error::<Identifier>(&[Token::Str("")], "empty PIN token");
    assert_de_tokens_error::<Identifier>(
        &[Token::Str("K+")],
        "invalid PIN terminal marker (expected '^')",
    );
    assert_de_tokens_error::<Identifier>(
        &[Token::Str("++K")],
        "PIN token must contain exactly one ASCII letter",
    );
}
