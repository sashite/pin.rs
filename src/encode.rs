//! Allocation-free string encoding of a PIN token.

use crate::identifier::Identifier;
use crate::state::State;

/// The canonical string form of an [`Identifier`], stored inline.
///
/// A token occupies at most three bytes, so `EncodedPin` keeps them in a fixed
/// buffer with no heap allocation. It is produced by [`Identifier::encode`] and
/// dereferences to [`str`], so it can be used wherever a string slice is
/// expected.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), sashite_pin::ParseError> {
/// use sashite_pin::Identifier;
///
/// let enc = Identifier::parse("+K^")?.encode();
/// assert_eq!(enc.as_str(), "+K^");
/// assert_eq!(&*enc, "+K^"); // via Deref<Target = str>
/// assert_eq!(enc.len(), 3); // str method reached through Deref
/// assert_eq!(enc, "+K^"); // direct comparison via PartialEq<&str>
/// assert_eq!("+K^", enc); // and the reverse direction
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Copy)]
pub struct EncodedPin {
    buf: [u8; 3],
    len: u8,
}

impl EncodedPin {
    /// Encodes an identifier into its canonical token form.
    #[must_use]
    pub(crate) fn from_identifier(id: Identifier) -> Self {
        let mut buf = [0u8; 3];
        let mut len: u8 = 0;

        // Optional state-modifier prefix.
        if let Some(modifier) = state_modifier(id.state()) {
            buf[usize::from(len)] = modifier;
            len += 1;
        }

        // The cased abbreviation letter is always present.
        buf[usize::from(len)] = id.letter().to_ascii(id.side());
        len += 1;

        // Optional terminal marker suffix.
        if id.is_terminal() {
            buf[usize::from(len)] = b'^';
            len += 1;
        }

        Self { buf, len }
    }

    /// Returns the encoded token as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        let bytes = &self.buf[..usize::from(self.len)];
        debug_assert!(bytes.is_ascii(), "EncodedPin must contain only ASCII bytes");
        // ASCII is always valid UTF-8, so this conversion cannot fail; the empty
        // fallback is unreachable and exists only to avoid `unsafe`.
        core::str::from_utf8(bytes).unwrap_or("")
    }
}

impl core::ops::Deref for EncodedPin {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for EncodedPin {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl core::fmt::Display for EncodedPin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl core::fmt::Debug for EncodedPin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EncodedPin({:?})", self.as_str())
    }
}

impl PartialEq<str> for EncodedPin {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for EncodedPin {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<EncodedPin> for str {
    fn eq(&self, other: &EncodedPin) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<EncodedPin> for &str {
    fn eq(&self, other: &EncodedPin) -> bool {
        *self == other.as_str()
    }
}

/// Encodes a state into its modifier byte. Inverse of the decoder in
/// `parse.rs`.
const fn state_modifier(state: State) -> Option<u8> {
    match state {
        State::Normal => None,
        State::Enhanced => Some(b'+'),
        State::Diminished => Some(b'-'),
    }
}
