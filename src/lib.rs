//! # Piece Identifier Notation (PIN)
//!
//! A `no_std`, `unsafe`-free implementation of the
//! [PIN v1.0.0 specification](https://sashite.dev/specs/pin/1.0.0/).
//!
//! PIN is a compact, ASCII-only token that encodes a piece's identity at the
//! level of notation. A token has the shape:
//!
//! ```text
//! [<state-modifier>]<abbr>[<terminal-marker>]
//! ```
//!
//! matching the anchored regular expression `^[+-]?[A-Za-z]\^?$`, and maps to
//! exactly four attributes:
//!
//! | Component        | Encodes          | Values                                    |
//! |------------------|------------------|-------------------------------------------|
//! | letter case      | [`Side`]         | uppercase → `First`, lowercase → `Second` |
//! | letter           | [`Letter`]       | the piece-name abbreviation (`A`–`Z`)     |
//! | `+` / `-` prefix | [`State`]        | `Enhanced` / `Diminished` (else `Normal`) |
//! | `^` suffix       | terminal status  | present → terminal piece                  |
//!
//! PIN standardizes only the *encoding*; the meaning of each attribute is left
//! to the rule system. See the [glossary](https://sashite.dev/glossary/).
//!
//! ## Example
//!
//! ```
//! # fn main() -> Result<(), sashite_pin::ParseError> {
//! use sashite_pin::{Identifier, Side, State};
//!
//! let king: Identifier = "+K^".parse()?;
//! assert_eq!(king.letter().as_char(), 'K');
//! assert_eq!(king.side(), Side::First);
//! assert_eq!(king.state(), State::Enhanced);
//! assert!(king.is_terminal());
//!
//! // Round-trips back to its canonical, zero-allocation string form.
//! assert_eq!(king.encode().as_str(), "+K^");
//!
//! // Cheap, infallible transformations (the type is `Copy`).
//! assert_eq!(king.flipped().encode().as_str(), "+k^");
//!
//! // Formatting behaves like any other string-like type.
//! assert_eq!(king.to_string(), "+K^");
//! assert_eq!(format!("{king:>6}"), "   +K^");
//! # Ok(())
//! # }
//! ```
//!
//! ## Guarantees
//!
//! - **`no_std` and allocation-free:** parsing borrows the input bytes and an
//!   [`Identifier`] is a 4-byte `Copy` value; nothing is heap-allocated.
//! - **No `unsafe`:** the crate is built under a forbid-`unsafe` lint policy.
//! - **Bounded, panic-free parsing:** inputs longer than three bytes are
//!   rejected before inspection, and the public parsing API never panics.
//! - **`const` throughout:** construction, parsing, validation, the accessors,
//!   the transformations, [`Identifier::encode`] and [`EncodedPin::as_str`] are
//!   all `const fn`, so a token can be built, checked, encoded *and spelled
//!   out* at compile time — the whole path from typed components to token text
//!   runs in a `const` context.
//! - **String-like formatting:** [`Identifier`], [`EncodedPin`] and
//!   [`ParseError`] all render through [`core::fmt::Formatter::pad`], so width,
//!   fill, alignment and precision work as they do for [`str`].
//! - **No required dependencies:** the default build has an empty dependency
//!   graph; the optional `serde` feature adds `serde` (kept `no_std`).

#![no_std]

/// Compiles every example in `README.md` as a doctest.
///
/// Nothing built the README before, and its examples used `?` at what would be
/// a doctest's top level — so they could not have compiled. Wiring it in costs
/// nothing in a normal build and keeps the front page honest.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

mod encode;
mod error;
mod identifier;
mod letter;
mod parse;
#[cfg(feature = "serde")]
mod serde_impl;
mod side;
mod state;

pub use crate::encode::EncodedPin;
pub use crate::error::ParseError;
pub use crate::identifier::Identifier;
pub use crate::letter::Letter;
pub use crate::side::Side;
pub use crate::state::State;
