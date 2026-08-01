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
/// Every comparison — `Eq`, `Ord`, `Hash`, and the `PartialEq<str>` family — is
/// defined on the token *text* rather than on the padded buffer, so two
/// encodings relate to each other exactly as their `&str` forms do.
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
///
/// // Two encodings compare to each other, and sort like their text.
/// assert_eq!(enc, Identifier::parse("+K^")?.encode());
/// assert!(Identifier::parse("K")?.encode() < Identifier::parse("K^")?.encode());
///
/// // Display honours the format spec, exactly as `str` does.
/// assert_eq!(format!("{enc:>5}"), "  +K^");
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
    ///
    /// Every arm below writes the buffer whole, so the encoder performs no
    /// index arithmetic at all: it cannot panic, in debug or release, and it is
    /// usable in `const` context. The four arms are the four token shapes, and
    /// each `len` counts exactly the bytes that arm wrote.
    #[must_use]
    pub(crate) const fn from_identifier(id: Identifier) -> Self {
        // The cased abbreviation letter is the one mandatory byte.
        let letter = id.letter().to_ascii(id.side());

        match (state_modifier(id.state()), id.is_terminal()) {
            (None, false) => Self {
                buf: [letter, 0, 0],
                len: 1,
            },
            (None, true) => Self {
                buf: [letter, b'^', 0],
                len: 2,
            },
            (Some(modifier), false) => Self {
                buf: [modifier, letter, 0],
                len: 2,
            },
            (Some(modifier), true) => Self {
                buf: [modifier, letter, b'^'],
                len: 3,
            },
        }
    }

    /// Returns the encoded token as a string slice.
    #[must_use]
    pub const fn as_str(&self) -> &str {
        // `split_at` rather than a range index, and a `match` rather than
        // `unwrap_or`: both slice `Index` and `Result::unwrap_or` are still
        // non-const at the 1.81 MSRV, whereas `str::from_utf8` is already
        // const there. Going through them is what lets the whole path from a
        // typed identifier to its token text run at compile time — the claim
        // the crate docs make.
        let (bytes, _) = self.buf.split_at(self.len as usize);
        // ASCII is always valid UTF-8, so this conversion cannot fail; the
        // empty fallback is unreachable and exists only to avoid `unsafe`.
        match core::str::from_utf8(bytes) {
            Ok(text) => text,
            Err(_) => "",
        }
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

/// Writes the canonical token.
///
/// Formatting goes through [`core::fmt::Formatter::pad`], so the token honours
/// width, fill, alignment and precision exactly as a [`str`] would:
/// `format!("{enc:>4}")` right-aligns it in four columns. With an empty format
/// spec the output is the bare token.
impl core::fmt::Display for EncodedPin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.pad(self.as_str())
    }
}

impl core::fmt::Debug for EncodedPin {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "EncodedPin({:?})", self.as_str())
    }
}

/// Compares two encodings by their token text.
///
/// The inline buffer is a fixed three bytes but a token occupies one to three,
/// so the tail past `len` is padding that carries no meaning. Comparing
/// [`EncodedPin::as_str`] rather than the raw buffer keeps the relation defined
/// by what the value *denotes*, and makes it agree with the `PartialEq<str>`
/// impls below: `a == b` holds exactly when `a.as_str() == b.as_str()`.
impl PartialEq for EncodedPin {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for EncodedPin {}

/// Hashes the token text, so an `EncodedPin` hashes identically to the [`str`]
/// it compares equal to, and `Hash` stays consistent with `Eq`.
impl core::hash::Hash for EncodedPin {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl PartialOrd for EncodedPin {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// Orders encodings lexicographically by token text, matching [`str`]'s own
/// ordering. This is *not* the ordering [`crate::Identifier`] derives, which
/// compares letter → side → state → terminal.
impl Ord for EncodedPin {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_str().cmp(other.as_str())
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
