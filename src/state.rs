//! Piece state: the optional enhancement marker.

/// The state of a piece, encoded by the optional `+` / `-` prefix.
///
/// PIN standardizes only the *encoding*; what an enhanced or diminished state
/// means in play is defined by the rule system (e.g. promotion, double-step
/// eligibility, check status). The mapping is:
///
/// - no prefix → [`State::Normal`] (the baseline),
/// - `+` prefix → [`State::Enhanced`],
/// - `-` prefix → [`State::Diminished`].
///
/// The total ordering runs along the natural spectrum
/// `Diminished < Normal < Enhanced`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[repr(u8)]
pub enum State {
    /// Diminished state, encoded by the `-` prefix.
    Diminished = 0,
    /// Baseline state, encoded by the absence of any prefix.
    #[default]
    Normal = 1,
    /// Enhanced state, encoded by the `+` prefix.
    Enhanced = 2,
}
