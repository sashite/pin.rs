//! The central PIN identifier type.

use crate::encode::EncodedPin;
use crate::error::ParseError;
use crate::letter::Letter;
use crate::side::Side;
use crate::state::State;

/// A parsed PIN token: the identity of a piece at the level of notation.
///
/// An `Identifier` bundles the four attributes a token encodes — a
/// [`Letter`] abbreviation, a [`Side`], a [`State`], and a terminal flag — into
/// a single 4-byte `Copy` value. Construction from typed components via
/// [`Identifier::new`] is total: every combination is a valid token, so it
/// cannot fail.
///
/// The derived total ordering compares attributes in the order
/// letter → side → state → terminal.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), sashite_pin::ParseError> {
/// use sashite_pin::{Identifier, Side, State};
///
/// let rook: Identifier = "+r".parse()?;
/// assert_eq!(rook.letter().as_char(), 'R');
/// assert_eq!(rook.side(), Side::Second);
/// assert_eq!(rook.state(), State::Enhanced);
/// assert!(!rook.is_terminal());
///
/// // Transformations are cheap and infallible; the value is `Copy`.
/// assert_eq!(rook.flipped().normalized().encode().as_str(), "R");
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identifier {
    letter: Letter,
    side: Side,
    state: State,
    terminal: bool,
}

impl Identifier {
    /// Builds an identifier from its four typed components.
    ///
    /// This is infallible: because each component type is valid by
    /// construction, every combination denotes a valid PIN token.
    ///
    /// # Examples
    ///
    /// ```
    /// use sashite_pin::{Identifier, Letter, Side, State};
    ///
    /// let p = Identifier::new(
    ///     Letter::try_from_char('R').unwrap(),
    ///     Side::Second,
    ///     State::Enhanced,
    ///     false,
    /// );
    /// assert_eq!(p.encode().as_str(), "+r");
    /// ```
    #[must_use]
    pub const fn new(letter: Letter, side: Side, state: State, terminal: bool) -> Self {
        Self { letter, side, state, terminal }
    }

    /// Parses a string slice into an identifier.
    ///
    /// # Errors
    ///
    /// Returns a [`ParseError`] if `input` is not a valid PIN token (empty,
    /// too long, or malformed).
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), sashite_pin::ParseError> {
    /// use sashite_pin::Identifier;
    ///
    /// let king = Identifier::parse("K^")?;
    /// assert!(king.is_first());
    /// assert!(king.is_terminal());
    /// # Ok(())
    /// # }
    /// ```
    pub const fn parse(input: &str) -> Result<Self, ParseError> {
        crate::parse::parse(input)
    }

    /// Reports whether `input` is a valid PIN token, without allocating or
    /// constructing an identifier on the caller's side.
    #[must_use]
    pub const fn is_valid(input: &str) -> bool {
        Self::parse(input).is_ok()
    }

    /// Returns the canonical, allocation-free string encoding of this token.
    #[must_use]
    pub fn encode(self) -> EncodedPin {
        EncodedPin::from_identifier(self)
    }

    // --- Accessors ---

    /// Returns the piece-name abbreviation (always uppercase).
    #[must_use]
    pub const fn letter(self) -> Letter {
        self.letter
    }

    /// Returns the side the piece belongs to.
    #[must_use]
    pub const fn side(self) -> Side {
        self.side
    }

    /// Returns the piece state.
    #[must_use]
    pub const fn state(self) -> State {
        self.state
    }

    /// Reports whether the piece is terminal (the `^` marker is present).
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        self.terminal
    }

    // --- State queries ---

    /// Reports whether the state is [`State::Normal`].
    #[must_use]
    pub const fn is_normal(self) -> bool {
        matches!(self.state, State::Normal)
    }

    /// Reports whether the state is [`State::Enhanced`].
    #[must_use]
    pub const fn is_enhanced(self) -> bool {
        matches!(self.state, State::Enhanced)
    }

    /// Reports whether the state is [`State::Diminished`].
    #[must_use]
    pub const fn is_diminished(self) -> bool {
        matches!(self.state, State::Diminished)
    }

    // --- Side queries ---

    /// Reports whether the side is [`Side::First`].
    #[must_use]
    pub const fn is_first(self) -> bool {
        matches!(self.side, Side::First)
    }

    /// Reports whether the side is [`Side::Second`].
    #[must_use]
    pub const fn is_second(self) -> bool {
        matches!(self.side, Side::Second)
    }

    // --- Transformations (return a new value; the type is `Copy`) ---

    /// Returns a copy with the abbreviation replaced.
    #[must_use]
    pub const fn with_letter(self, letter: Letter) -> Self {
        Self::new(letter, self.side, self.state, self.terminal)
    }

    /// Returns a copy with the side replaced.
    #[must_use]
    pub const fn with_side(self, side: Side) -> Self {
        Self::new(self.letter, side, self.state, self.terminal)
    }

    /// Returns a copy with the state replaced.
    #[must_use]
    pub const fn with_state(self, state: State) -> Self {
        Self::new(self.letter, self.side, state, self.terminal)
    }

    /// Returns a copy with the terminal flag replaced.
    #[must_use]
    pub const fn with_terminal(self, terminal: bool) -> Self {
        Self::new(self.letter, self.side, self.state, terminal)
    }

    /// Returns a copy in the [`State::Enhanced`] state.
    #[must_use]
    pub const fn enhanced(self) -> Self {
        self.with_state(State::Enhanced)
    }

    /// Returns a copy in the [`State::Diminished`] state.
    #[must_use]
    pub const fn diminished(self) -> Self {
        self.with_state(State::Diminished)
    }

    /// Returns a copy in the baseline [`State::Normal`] state.
    #[must_use]
    pub const fn normalized(self) -> Self {
        self.with_state(State::Normal)
    }

    /// Returns a copy belonging to the opposite [`Side`].
    #[must_use]
    pub const fn flipped(self) -> Self {
        self.with_side(self.side.flip())
    }
}

impl core::fmt::Display for Identifier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(self.encode().as_str())
    }
}

impl core::str::FromStr for Identifier {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl TryFrom<&str> for Identifier {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::parse(s)
    }
}

impl TryFrom<&[u8]> for Identifier {
    type Error = ParseError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        crate::parse::parse_bytes(bytes)
    }
}
