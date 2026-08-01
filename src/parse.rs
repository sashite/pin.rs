//! Internal byte-level parser for PIN tokens.
//!
//! This is the only place untrusted input is inspected. The parser is
//! allocation-free, uses no regex engine, and rejects over-long input on a
//! structural length check before any byte is examined. It is reached from
//! outside the crate only through [`crate::Identifier`]: its `parse` and
//! `is_valid` inherent methods, and its [`core::str::FromStr`] and [`TryFrom`]
//! implementations (for both `&str` and `&[u8]`).

use crate::error::ParseError;
use crate::identifier::Identifier;
use crate::letter::Letter;
use crate::state::State;

/// Parses a string slice into an [`Identifier`].
pub(crate) const fn parse(input: &str) -> Result<Identifier, ParseError> {
    parse_bytes(input.as_bytes())
}

/// Parses a raw byte slice into an [`Identifier`].
///
/// No UTF-8 validation is needed: only ASCII bytes can form a valid token, and
/// any other byte falls through to an error arm. Dispatches on the byte length
/// first; input longer than three bytes is rejected as [`ParseError::TooLong`]
/// without inspecting its contents.
pub(crate) const fn parse_bytes(bytes: &[u8]) -> Result<Identifier, ParseError> {
    match bytes {
        [] => Err(ParseError::Empty),
        [b0] => parse_bare(*b0),
        [b0, b1] => parse_pair(*b0, *b1),
        [b0, b1, b2] => parse_triple(*b0, *b1, *b2),
        _ => Err(ParseError::TooLong),
    }
}

/// `<abbr>`: a lone abbreviation letter.
const fn parse_bare(byte: u8) -> Result<Identifier, ParseError> {
    match Letter::from_ascii(byte) {
        Some((letter, side)) => Ok(Identifier::new(letter, side, State::Normal, false)),
        None => Err(ParseError::InvalidLetter),
    }
}

/// Two bytes: either `<abbr><terminal>` or `<modifier><abbr>`.
const fn parse_pair(first: u8, second: u8) -> Result<Identifier, ParseError> {
    // `<abbr><terminal>`: a letter must be followed by the terminal marker.
    if let Some((letter, side)) = Letter::from_ascii(first) {
        return match second {
            b'^' => Ok(Identifier::new(letter, side, State::Normal, true)),
            _ => Err(ParseError::InvalidTerminalMarker),
        };
    }

    // `<modifier><abbr>`: a modifier must be followed by a letter.
    if let Some(state) = modifier_state(first) {
        return match Letter::from_ascii(second) {
            Some((letter, side)) => Ok(Identifier::new(letter, side, state, false)),
            None => Err(ParseError::InvalidLetter),
        };
    }

    // The leading byte is neither a letter nor a state modifier.
    Err(ParseError::InvalidStateModifier)
}

/// Three bytes: the only valid shape is `<modifier><abbr><terminal>`.
const fn parse_triple(first: u8, second: u8, third: u8) -> Result<Identifier, ParseError> {
    let Some(state) = modifier_state(first) else {
        return Err(ParseError::InvalidStateModifier);
    };
    let Some((letter, side)) = Letter::from_ascii(second) else {
        return Err(ParseError::InvalidLetter);
    };
    match third {
        b'^' => Ok(Identifier::new(letter, side, state, true)),
        _ => Err(ParseError::InvalidTerminalMarker),
    }
}

/// Decodes a state-modifier byte. Inverse of the encoder in `encode.rs`.
const fn modifier_state(byte: u8) -> Option<State> {
    match byte {
        b'+' => Some(State::Enhanced),
        b'-' => Some(State::Diminished),
        _ => None,
    }
}
