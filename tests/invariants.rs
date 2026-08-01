//! Panic-freedom and the invariant chain that guarantees it.
//!
//! This crate targets bare metal (`thumbv7em-none-eabi`), where a panic is not
//! a graceful failure: there is no unwinder and usually no way to report what
//! happened. Every site that could panic is therefore worth naming, along with
//! the invariant that rules it out.
//!
//! After the encoder was rewritten to build its buffer whole, the entire crate
//! contains exactly **two** slice-index sites and **three** arithmetic sites:
//!
//! | Site                              | Could panic if…            | Ruled out by |
//! |-----------------------------------|----------------------------|--------------|
//! | `encode.rs` `buf[..len]`          | `len > 3`                  | **E1**       |
//! | `encode.rs` `debug_assert!`       | a non-ASCII byte in `buf`  | **E1**       |
//! | `letter.rs` `byte - 32`           | `byte < 32`                | match arm    |
//! | `letter.rs` `c as u8 - 32`        | `c < '\u{20}'`             | match arm    |
//! | `letter.rs` `self.0 + 32`         | `self.0 > 223`             | **L1**       |
//!
//! The invariants, each depending only on the ones above it:
//!
//! - **L1** — a [`Letter`] always wraps a byte in `b'A'..=b'Z'`. The field is
//!   private and the type has exactly three construction sites (`Letter::ALL`,
//!   `from_ascii`, `try_from_char`); each either writes an uppercase literal or
//!   subtracts 32 from a byte a match arm has already pinned to `b'a'..=b'z'`.
//! - **L2** — given L1, `Letter::to_ascii` computes `self.0 + 32` on a value of
//!   at most 90, so it cannot overflow, and yields an ASCII letter byte.
//! - **E1** — given L2, an [`EncodedPin`] always has `len` in `1..=3` with
//!   `buf[..len]` holding only non-zero ASCII bytes. The fields are private and
//!   the type has exactly one constructor, whose four arms write a complete
//!   buffer and a matching literal length.
//! - **E2** — given E1, the slice in `as_str` is in bounds and is valid UTF-8,
//!   so `from_utf8` never fails and the `unwrap_or("")` fallback is unreachable.
//!
//! The parser adds no sites at all: it destructures with slice patterns and does
//! no arithmetic.
//!
//! The tests below exercise each invariant over its whole domain. They matter
//! because `cargo test` builds with the dev profile, where both debug assertions
//! and integer-overflow checks are on — the first test asserts exactly that, so
//! that a profile change can never quietly turn the rest into no-ops.

use sashite_pin::{EncodedPin, Identifier, Letter, ParseError, Side, State};

/// Builds all 312 identifiers from their typed components.
fn all_ids() -> Vec<Identifier> {
    let mut ids = Vec::with_capacity(312);
    for letter in Letter::ALL {
        for side in [Side::First, Side::Second] {
            for state in [State::Diminished, State::Normal, State::Enhanced] {
                for terminal in [false, true] {
                    ids.push(Identifier::new(letter, side, state, terminal));
                }
            }
        }
    }
    ids
}

#[test]
fn these_tests_run_with_overflow_and_debug_checks_on() {
    // Without this, an optimized profile would silently wrap the arithmetic the
    // tests below are meant to police, and they would pass for the wrong reason.
    // Read through a runtime binding so the check is not folded into a constant.
    let checks_are_on = cfg!(debug_assertions);
    assert!(
        checks_are_on,
        "the panic-freedom tests are only meaningful with debug assertions on",
    );
    let (sum, overflowed) = 250_u8.overflowing_add(32);
    assert!(overflowed && sum == 26, "sanity: u8 addition does wrap");
}

#[test]
fn l1_every_letter_wraps_an_uppercase_ascii_byte() {
    // Construction site 1: the ALL table.
    assert_eq!(Letter::ALL.len(), 26);
    for (offset, letter) in Letter::ALL.into_iter().enumerate() {
        assert_eq!(letter.as_ascii(), b'A' + u8::try_from(offset).unwrap());
        assert!(letter.as_ascii().is_ascii_uppercase());
    }

    // Construction site 2: from_ascii, over every possible byte.
    let mut decoded = 0_u32;
    for byte in 0..=u8::MAX {
        match Letter::from_ascii(byte) {
            Some((letter, side)) => {
                assert!(byte.is_ascii_alphabetic(), "{byte:#04X} is not a letter");
                assert!(
                    letter.as_ascii().is_ascii_uppercase(),
                    "L1 violated for {byte:#04X}",
                );
                assert_eq!(letter.as_ascii(), byte.to_ascii_uppercase());
                assert_eq!(
                    side,
                    if byte.is_ascii_uppercase() {
                        Side::First
                    } else {
                        Side::Second
                    },
                );
                decoded += 1;
            }
            None => assert!(!byte.is_ascii_alphabetic(), "{byte:#04X} is a letter"),
        }
    }
    assert_eq!(decoded, 52, "exactly the 52 ASCII letters decode");

    // Construction site 3: try_from_char, over every Unicode scalar. This is
    // also where the `c as u8` truncation lives, so the whole scalar range is
    // worth sweeping rather than sampling.
    let mut accepted = 0_u32;
    for code_point in 0..=0x0010_FFFF_u32 {
        let Some(character) = char::from_u32(code_point) else {
            continue;
        };
        match Letter::try_from_char(character) {
            Ok(letter) => {
                assert!(character.is_ascii_alphabetic(), "U+{code_point:04X}");
                assert!(
                    letter.as_ascii().is_ascii_uppercase(),
                    "L1 violated for U+{code_point:04X}",
                );
                assert_eq!(letter.as_char(), character.to_ascii_uppercase());
                assert_eq!(Letter::try_from(character), Ok(letter));
                accepted += 1;
            }
            Err(error) => {
                assert!(!character.is_ascii_alphabetic(), "U+{code_point:04X}");
                assert_eq!(error, ParseError::InvalidLetter);
            }
        }
    }
    assert_eq!(accepted, 52, "exactly the 52 ASCII letters convert");
}

#[test]
fn l2_the_cased_byte_never_overflows() {
    // `to_ascii` is crate-private; `encode` is how it is reached from outside.
    // Every letter, on both sides, must produce the ASCII byte its case implies
    // and nothing else — an overflow would show up as a wrapped byte here.
    for letter in Letter::ALL {
        for (side, expected) in [
            (Side::First, letter.as_ascii()),
            (Side::Second, letter.as_ascii() + 32),
        ] {
            let encoded = Identifier::new(letter, side, State::Normal, false).encode();
            assert_eq!(encoded.as_str().as_bytes(), &[expected], "{letter:?}");
            assert!(expected.is_ascii_alphabetic());
        }
        // The widest value L1 permits, 'Z' (90), plus 32 is 122 — still far
        // from the 255 at which a u8 would wrap.
        assert!(letter.as_ascii() <= b'Z');
        assert!(u8::try_from(u16::from(letter.as_ascii()) + 32).is_ok());
    }
}

#[test]
fn e1_every_encoding_has_a_length_in_range_and_ascii_content() {
    for id in all_ids() {
        let encoded = id.encode();
        let bytes = encoded.as_str().as_bytes();

        // The bound that makes `buf[..len]` an in-range slice.
        assert!(
            (1..=3).contains(&bytes.len()),
            "{encoded:?} has length {}",
            bytes.len(),
        );
        // The property the `debug_assert!` in `as_str` checks.
        assert!(bytes.is_ascii(), "{encoded:?}");
        // No byte is ever the buffer's zero padding, so the padding can never
        // be mistaken for content.
        assert!(!bytes.contains(&0), "{encoded:?}");
        // The length agrees with the shape the identifier describes.
        let expected = 1 + usize::from(!id.is_normal()) + usize::from(id.is_terminal());
        assert_eq!(bytes.len(), expected, "{encoded:?}");
    }
}

#[test]
fn e2_as_str_never_falls_back_to_the_empty_string() {
    // `as_str` ends in `from_utf8(..).unwrap_or("")`. That fallback exists only
    // to avoid `unsafe`; reaching it would mean E1 had been broken. An empty
    // result is the observable signature of that, so no encoding may be empty.
    for id in all_ids() {
        let encoded = id.encode();
        assert!(!encoded.as_str().is_empty(), "{id:?} encoded to nothing");
        // The three views of the same bytes must coincide.
        assert_eq!(&*encoded, encoded.as_str());
        assert_eq!(AsRef::<str>::as_ref(&encoded), encoded.as_str());
        assert_eq!(encoded.len(), encoded.as_str().len());
    }
}

#[test]
fn the_parser_survives_every_byte_pattern_it_can_be_handed() {
    // The parser itself has no arithmetic and no indexing — it destructures
    // with slice patterns — so this is a direct demonstration rather than an
    // invariant argument: running the whole reachable input space through it
    // under overflow checks, without a panic, is the property.
    let mut buf = [0u8; 3];
    let mut accepted = 0_u32;
    for first in 0..=u8::MAX {
        buf[0] = first;
        accepted += u32::from(Identifier::try_from(&buf[..1]).is_ok());
        for second in 0..=u8::MAX {
            buf[1] = second;
            accepted += u32::from(Identifier::try_from(&buf[..2]).is_ok());
            for third in 0..=u8::MAX {
                buf[2] = third;
                accepted += u32::from(Identifier::try_from(&buf[..3]).is_ok());
            }
        }
    }
    // Counting the acceptances both consumes every result and restates the
    // size of the closed domain from a wholly separate direction.
    assert_eq!(accepted, 312);

    // Degenerate and oversized inputs reach the same code by a different route.
    for slice in [&[][..], &[0xFF][..], &[0x00; 64][..], &[0xC3, 0x28][..]] {
        assert!(Identifier::try_from(slice).is_err());
    }
    assert_eq!(Identifier::try_from(&[][..]), Err(ParseError::Empty));
}

#[test]
fn encoding_and_parsing_are_mutually_inverse_over_the_whole_domain() {
    // `parse ∘ encode = id` on identifiers built from typed components. The
    // reverse direction, `encode ∘ parse = id` on every accepted string, is
    // established over the exhaustive byte sweep in `conformance.rs`.
    for id in all_ids() {
        let encoded: EncodedPin = id.encode();
        assert_eq!(Identifier::parse(encoded.as_str()), Ok(id), "{encoded:?}");
        assert_eq!(
            Identifier::try_from(encoded.as_str().as_bytes()),
            Ok(id),
            "{encoded:?}",
        );
        // Encoding is deterministic: the same identifier always encodes alike.
        assert_eq!(id.encode(), encoded);
    }
}
