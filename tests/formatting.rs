//! Formatting tests.
//!
//! Every string-like type in this crate renders through
//! [`core::fmt::Formatter::pad`], which is what makes width, fill, alignment and
//! precision work. A `Display` implementation that calls `write_str` instead
//! silently swallows the whole format spec — `format!("{id:>6}")` would yield
//! the bare token rather than padding it — so these tests pin the padding
//! behaviour for all three types, and pin just as firmly that an *empty* format
//! spec still produces the bare canonical text.

use sashite_pin::{Identifier, ParseError};

/// The three shapes that must all agree: a bare `{}`, `to_string`, and the
/// canonical encoding.
fn assert_plain_form_is_canonical(id: Identifier) {
    let canonical = id.encode();
    assert_eq!(format!("{id}"), canonical.as_str());
    assert_eq!(id.to_string(), canonical.as_str());
    assert_eq!(format!("{canonical}"), canonical.as_str());
    assert_eq!(canonical.to_string(), canonical.as_str());
}

/// Builds all 312 canonical tokens by spelling them out, independently of the
/// crate's own encoder.
fn every_token_text() -> Vec<String> {
    let mut out = Vec::with_capacity(312);
    for upper in b'A'..=b'Z' {
        for cased in [upper, upper + 32] {
            for prefix in ["", "+", "-"] {
                for suffix in ["", "^"] {
                    out.push(format!("{prefix}{}{suffix}", cased as char));
                }
            }
        }
    }
    out
}

#[test]
fn plain_display_is_the_bare_canonical_token_for_every_identifier() {
    for text in every_token_text() {
        let id = Identifier::parse(&text).expect("canonical token parses");
        assert_eq!(format!("{id}"), text);
        assert_plain_form_is_canonical(id);
    }
}

#[test]
fn identifier_display_honours_width_fill_and_alignment() {
    let id = Identifier::parse("+K^").expect("valid token");

    // Width with the three alignments, then a custom fill character.
    assert_eq!(format!("{id:>6}"), "   +K^");
    assert_eq!(format!("{id:<6}"), "+K^   ");
    assert_eq!(format!("{id:^7}"), "  +K^  ");
    assert_eq!(format!("{id:*>6}"), "***+K^");
    assert_eq!(format!("{id:.>6}"), "...+K^");

    // A width no wider than the token leaves it untouched.
    assert_eq!(format!("{id:>3}"), "+K^");
    assert_eq!(format!("{id:>1}"), "+K^");

    // Padding is applied to the token's own width, which varies with its shape.
    let short = Identifier::parse("K").expect("valid token");
    assert_eq!(format!("{short:>6}"), "     K");
    assert_eq!(format!("{short:<6}|"), "K     |");
}

#[test]
fn identifier_display_honours_precision() {
    // Precision truncates, exactly as it does for `str`.
    let id = Identifier::parse("+K^").expect("valid token");
    assert_eq!(format!("{id:.2}"), "+K");
    assert_eq!(format!("{id:.0}"), "");
    assert_eq!(format!("{id:.9}"), "+K^");
    // Precision and width compose.
    assert_eq!(format!("{id:>5.2}"), "   +K");
}

#[test]
fn encoded_pin_display_honours_the_format_spec() {
    let enc = Identifier::parse("-p").expect("valid token").encode();

    assert_eq!(format!("{enc}"), "-p");
    assert_eq!(format!("{enc:>5}"), "   -p");
    assert_eq!(format!("{enc:<5}|"), "-p   |");
    assert_eq!(format!("{enc:^6}"), "  -p  ");
    assert_eq!(format!("{enc:0>5}"), "000-p");
    assert_eq!(format!("{enc:.1}"), "-");

    // An `EncodedPin` and the `Identifier` it came from format identically
    // under any spec.
    for text in every_token_text() {
        let id = Identifier::parse(&text).expect("canonical token parses");
        let enc = id.encode();
        assert_eq!(format!("{id:>8}"), format!("{enc:>8}"), "{text:?}");
        assert_eq!(format!("{id:_^9}"), format!("{enc:_^9}"), "{text:?}");
        assert_eq!(format!("{id:.2}"), format!("{enc:.2}"), "{text:?}");
    }
}

#[test]
fn parse_error_display_honours_the_format_spec() {
    let err = ParseError::Empty;
    assert_eq!(err.to_string(), "empty PIN token");
    assert_eq!(format!("{err}"), "empty PIN token");

    // "empty PIN token" is 15 bytes wide.
    assert_eq!(format!("{err:>20}"), "     empty PIN token");
    assert_eq!(format!("{err:<20}|"), "empty PIN token     |");
    assert_eq!(format!("{err:.5}"), "empty");

    // Every variant pads, and padding never alters the message itself.
    for variant in [
        ParseError::Empty,
        ParseError::TooLong,
        ParseError::InvalidLetter,
        ParseError::InvalidStateModifier,
        ParseError::InvalidTerminalMarker,
    ] {
        let plain = variant.to_string();
        let width = plain.chars().count() + 4;
        let padded = format!("{variant:>width$}");
        assert_eq!(padded.chars().count(), width, "{plain:?}");
        assert_eq!(padded.trim_start(), plain, "{plain:?}");
        assert_eq!(format!("{variant}"), plain, "{plain:?}");
    }
}

#[test]
fn error_messages_are_measured_in_bytes_not_characters() {
    // The `TooLong` message must not claim "characters": a single non-ASCII
    // character can exceed the three-byte budget on its own. U+1FA00 is one
    // character and four bytes.
    assert_eq!("\u{1FA00}".chars().count(), 1);
    assert_eq!("\u{1FA00}".len(), 4);
    assert_eq!(
        Identifier::parse("\u{1FA00}"),
        Err(ParseError::TooLong),
        "a one-character, four-byte input is TooLong",
    );
    let message = ParseError::TooLong.to_string();
    assert!(
        message.contains("bytes"),
        "TooLong must be described in bytes, got {message:?}",
    );
    assert!(
        !message.contains("character"),
        "TooLong must not claim characters, got {message:?}",
    );

    // Conversely, a two-byte character fits the length budget and is rejected
    // on its content, not its length.
    assert_eq!("é".len(), 2);
    assert_eq!(
        Identifier::parse("é"),
        Err(ParseError::InvalidStateModifier)
    );
}

/// The whole path from typed components to token text runs at compile time.
///
/// `EncodedPin::as_str` was an ordinary method, which made the crate docs'
/// "spelled out at compile time" claim false. It is `const` now — reached by
/// going through `split_at` and a `match` rather than a range index and
/// `unwrap_or`, both of which are still non-const at the 1.81 MSRV, whereas
/// `str::from_utf8` is already const there.
#[test]
fn the_token_text_is_available_in_a_const_context() {
    const KING: Identifier = match Identifier::parse("+K^") {
        Ok(id) => id,
        Err(_) => unreachable!(),
    };
    const TEXT: &str = KING.encode().as_str();
    assert_eq!(TEXT, "+K^");
}
