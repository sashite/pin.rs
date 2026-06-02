//! Piece side: the camp a piece belongs to.

/// The camp a piece belongs to.
///
/// In PIN, the *case* of the abbreviation letter encodes the side: an
/// uppercase letter (`A`–`Z`) denotes [`Side::First`], a lowercase letter
/// (`a`–`z`) denotes [`Side::Second`]. This mirrors the two-side model of the
/// [glossary](https://sashite.dev/glossary/): a side is a pure label
/// (`first` / `second`) that acquires meaning only once a rule system assigns
/// it to a player or piece.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Side {
    /// The `first` side, encoded by an uppercase letter (`A`–`Z`).
    First = 0,
    /// The `second` side, encoded by a lowercase letter (`a`–`z`).
    Second = 1,
}

impl Side {
    /// Returns the opposite side.
    ///
    /// # Examples
    ///
    /// ```
    /// use sashite_pin::Side;
    ///
    /// assert_eq!(Side::First.flip(), Side::Second);
    /// assert_eq!(Side::Second.flip(), Side::First);
    /// ```
    #[must_use]
    pub const fn flip(self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::First,
        }
    }
}
