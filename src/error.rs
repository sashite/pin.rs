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
/// # Which variant you get
///
/// The parser first dispatches on the input's **byte** length, then checks each
/// position against the shape that length permits. The variant therefore names
/// the *position* that failed, not the most striking oddity of the input:
///
/// | Input length | Shape the parser expects        | Failing position → variant                          |
/// |--------------|---------------------------------|-----------------------------------------------------|
/// | 0 bytes      | —                               | [`Empty`](Self::Empty)                              |
/// | 1 byte       | `<abbr>`                        | byte 0 → [`InvalidLetter`](Self::InvalidLetter)     |
/// | 2 bytes      | `<abbr><terminal>` …            | byte 1 → [`InvalidTerminalMarker`](Self::InvalidTerminalMarker) |
/// |              | … or `<modifier><abbr>`         | byte 1 → [`InvalidLetter`](Self::InvalidLetter)     |
/// |              | … neither shape can start       | byte 0 → [`InvalidStateModifier`](Self::InvalidStateModifier) |
/// | 3 bytes      | `<modifier><abbr><terminal>`    | byte 0 → [`InvalidStateModifier`](Self::InvalidStateModifier), byte 1 → [`InvalidLetter`](Self::InvalidLetter), byte 2 → [`InvalidTerminalMarker`](Self::InvalidTerminalMarker) |
/// | ≥ 4 bytes    | —                               | [`TooLong`](Self::TooLong), before any byte is read  |
///
/// Two consequences are worth stating outright, because the variant can read as
/// surprising until the table above is in mind:
///
/// - `"K^^"` yields [`InvalidStateModifier`](Self::InvalidStateModifier), not a
///   complaint about the doubled marker: at three bytes the only legal shape
///   starts with a modifier, and `K` is not one.
/// - Length is counted in **bytes**, never in characters. `"é"` is one
///   character but two bytes, so it is parsed as a two-byte input; `"🨀"` is one
///   character but four bytes, so it is rejected as
///   [`TooLong`](Self::TooLong).
///
/// The distinction between variants is a diagnostic convenience. The *set* of
/// accepted inputs is what the specification fixes, and no variant is ever
/// returned for a valid token.
///
/// # Examples
///
/// ```
/// use sashite_pin::{Identifier, ParseError};
///
/// assert_eq!("".parse::<Identifier>(), Err(ParseError::Empty));
/// assert_eq!("++K".parse::<Identifier>(), Err(ParseError::InvalidLetter));
/// // Byte length, not character length: one character, four bytes.
/// assert_eq!("🨀".parse::<Identifier>(), Err(ParseError::TooLong));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ParseError {
    /// The input was empty.
    Empty,
    /// The input was longer than the three **bytes** a token can occupy. A
    /// single non-ASCII character can exceed that on its own.
    TooLong,
    /// The abbreviation position held a byte that is not an ASCII letter.
    InvalidLetter,
    /// The state-modifier position held something other than `+` or `-`.
    InvalidStateModifier,
    /// The terminal-marker position held something other than `^`.
    InvalidTerminalMarker,
}

/// Writes a short, lower-case reason, suitable for chaining into a larger
/// message.
///
/// Formatting goes through [`core::fmt::Formatter::pad`], so the message
/// honours width, fill, alignment and precision exactly as a [`str`] would.
/// With an empty format spec the output is the bare message.
impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::Empty => "empty PIN token",
            // "bytes", not "characters": a lone non-ASCII character such as
            // '🨀' is four bytes and lands here.
            Self::TooLong => "PIN token longer than three bytes",
            Self::InvalidLetter => "PIN abbreviation is not an ASCII letter",
            Self::InvalidStateModifier => {
                "PIN token starts with neither a state modifier nor a letter"
            }
            Self::InvalidTerminalMarker => "invalid PIN terminal marker (expected '^')",
        };
        f.pad(message)
    }
}

impl core::error::Error for ParseError {}
