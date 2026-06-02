//! Errors produced when parsing a PIN token.

/// The reason a string could not be parsed as a PIN token.
///
/// Returned by the parsing entry points ([`crate::Identifier::parse`],
/// [`core::str::FromStr`], [`TryFrom`]) and by [`crate::Letter::try_from_char`].
///
/// This enum is `#[non_exhaustive]`: future revisions may add variants without
/// a breaking change, so downstream `match` expressions should include a
/// wildcard arm.
///
/// # Examples
///
/// ```
/// use sashite_pin::{Identifier, ParseError};
///
/// assert_eq!("".parse::<Identifier>(), Err(ParseError::Empty));
/// assert_eq!("++K".parse::<Identifier>(), Err(ParseError::InvalidLetter));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ParseError {
    /// The input was empty.
    Empty,
    /// The input was longer than the three characters a token can occupy.
    TooLong,
    /// The input did not contain exactly one ASCII letter.
    InvalidLetter,
    /// The state-modifier position held something other than `+` or `-`.
    InvalidStateModifier,
    /// The terminal-marker position held something other than `^`.
    InvalidTerminalMarker,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::Empty => "empty PIN token",
            Self::TooLong => "PIN token longer than three characters",
            Self::InvalidLetter => "PIN token must contain exactly one ASCII letter",
            Self::InvalidStateModifier => "invalid PIN state modifier (expected '+' or '-')",
            Self::InvalidTerminalMarker => "invalid PIN terminal marker (expected '^')",
        };
        f.write_str(message)
    }
}

impl core::error::Error for ParseError {}
