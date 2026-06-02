//! Piece-name abbreviation: a single ASCII letter, side-agnostic.

use crate::error::ParseError;
use crate::side::Side;

/// The single-letter abbreviation of a piece name.
///
/// A `Letter` is the *identity* part of a PIN token, independent of side. Per
/// the specification the abbreviation is case-insensitive (`K` and `k` denote
/// the same piece name), so a `Letter` is always stored uppercase; the case of
/// the original token is carried separately by [`Side`].
///
/// # Invariant
///
/// The wrapped byte is always an uppercase ASCII letter (`b'A'..=b'Z'`). Every
/// constructor enforces this, so the invariant cannot be violated from outside
/// the crate.
///
/// Ordering is alphabetical (`A < B < … < Z`).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Letter(u8);

impl Letter {
    /// Every abbreviation, in alphabetical order (`A` through `Z`).
    pub const ALL: [Self; 26] = [
        Self(b'A'),
        Self(b'B'),
        Self(b'C'),
        Self(b'D'),
        Self(b'E'),
        Self(b'F'),
        Self(b'G'),
        Self(b'H'),
        Self(b'I'),
        Self(b'J'),
        Self(b'K'),
        Self(b'L'),
        Self(b'M'),
        Self(b'N'),
        Self(b'O'),
        Self(b'P'),
        Self(b'Q'),
        Self(b'R'),
        Self(b'S'),
        Self(b'T'),
        Self(b'U'),
        Self(b'V'),
        Self(b'W'),
        Self(b'X'),
        Self(b'Y'),
        Self(b'Z'),
    ];

    /// Decodes a raw ASCII byte into a [`Letter`] and the [`Side`] its case
    /// implies.
    ///
    /// Returns `None` for any byte that is not an ASCII letter. This is the
    /// lossless decoder used by the token parser.
    ///
    /// # Examples
    ///
    /// ```
    /// use sashite_pin::{Letter, Side};
    ///
    /// let (letter, side) = Letter::from_ascii(b'k').unwrap();
    /// assert_eq!(letter.as_char(), 'K');
    /// assert_eq!(side, Side::Second);
    ///
    /// assert!(Letter::from_ascii(b'1').is_none());
    /// ```
    #[must_use]
    pub const fn from_ascii(byte: u8) -> Option<(Self, Side)> {
        match byte {
            b'A'..=b'Z' => Some((Self(byte), Side::First)),
            b'a'..=b'z' => Some((Self(byte - 32), Side::Second)),
            _ => None,
        }
    }

    /// Builds a [`Letter`] from a `char`, folding case.
    ///
    /// Both `'K'` and `'k'` yield the same `Letter`; the case (which encodes
    /// side) is not retained.
    ///
    /// # Errors
    ///
    /// Returns [`ParseError::InvalidLetter`] if `c` is not an ASCII letter.
    ///
    /// # Examples
    ///
    /// ```
    /// use sashite_pin::Letter;
    ///
    /// assert_eq!(Letter::try_from_char('q').unwrap().as_char(), 'Q');
    /// assert!(Letter::try_from_char('+').is_err());
    /// ```
    #[allow(clippy::cast_possible_truncation)] // guarded: `c` is ASCII here
    pub const fn try_from_char(c: char) -> Result<Self, ParseError> {
        match c {
            'A'..='Z' => Ok(Self(c as u8)),
            'a'..='z' => Ok(Self(c as u8 - 32)),
            _ => Err(ParseError::InvalidLetter),
        }
    }

    /// Returns the abbreviation as an uppercase `char`.
    #[must_use]
    pub const fn as_char(self) -> char {
        self.0 as char
    }

    /// Returns the abbreviation as its raw uppercase ASCII byte.
    #[must_use]
    pub const fn as_ascii(self) -> u8 {
        self.0
    }

    /// Returns the ASCII byte as it appears in a token for the given side:
    /// uppercase for [`Side::First`], lowercase for [`Side::Second`].
    #[must_use]
    pub(crate) const fn to_ascii(self, side: Side) -> u8 {
        match side {
            Side::First => self.0,
            Side::Second => self.0 + 32,
        }
    }
}

impl TryFrom<char> for Letter {
    type Error = ParseError;

    fn try_from(c: char) -> Result<Self, Self::Error> {
        Self::try_from_char(c)
    }
}

impl core::fmt::Debug for Letter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Letter({:?})", self.as_char())
    }
}
