//! Transformation, query, and ordering tests.
//!
//! These exercise the algebra of the builder-style methods on [`Identifier`]
//! (each returns a new value; the type is `Copy`) over the whole closed domain,
//! plus the [`Letter`] helpers and the documented total orderings.

use sashite_pin::{Identifier, Letter, Side, State};

/// Builds all 312 identifiers directly from their typed components.
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
fn flip_is_an_involution_and_changes_only_side() {
    for id in all_ids() {
        let flipped = id.flipped();
        assert_ne!(flipped.side(), id.side());
        assert_eq!(flipped.flipped(), id, "double flip must restore {id:?}");
        // Every other attribute is preserved.
        assert_eq!(flipped.letter(), id.letter());
        assert_eq!(flipped.state(), id.state());
        assert_eq!(flipped.is_terminal(), id.is_terminal());
    }
    // Side's own involution.
    assert_eq!(Side::First.flip().flip(), Side::First);
}

#[test]
fn state_transformations_change_only_state() {
    for id in all_ids() {
        assert_eq!(id.enhanced().state(), State::Enhanced);
        assert_eq!(id.diminished().state(), State::Diminished);
        assert_eq!(id.normalized().state(), State::Normal);

        for state in [State::Diminished, State::Normal, State::Enhanced] {
            let changed = id.with_state(state);
            assert_eq!(changed.state(), state);
            assert_eq!(changed.letter(), id.letter());
            assert_eq!(changed.side(), id.side());
            assert_eq!(changed.is_terminal(), id.is_terminal());
        }
    }
}

#[test]
fn with_setters_change_only_their_target() {
    let queen = Letter::try_from_char('Q').unwrap();

    for id in all_ids() {
        for terminal in [false, true] {
            let changed = id.with_terminal(terminal);
            assert_eq!(changed.is_terminal(), terminal);
            assert_eq!(changed.letter(), id.letter());
            assert_eq!(changed.side(), id.side());
            assert_eq!(changed.state(), id.state());
        }

        for side in [Side::First, Side::Second] {
            let changed = id.with_side(side);
            assert_eq!(changed.side(), side);
            assert_eq!(changed.letter(), id.letter());
            assert_eq!(changed.state(), id.state());
            assert_eq!(changed.is_terminal(), id.is_terminal());
        }

        let changed = id.with_letter(queen);
        assert_eq!(changed.letter(), queen);
        assert_eq!(changed.side(), id.side());
        assert_eq!(changed.state(), id.state());
        assert_eq!(changed.is_terminal(), id.is_terminal());
    }
}

#[test]
fn queries_agree_with_accessors() {
    for id in all_ids() {
        assert_eq!(id.is_first(), id.side() == Side::First);
        assert_eq!(id.is_second(), id.side() == Side::Second);
        assert_eq!(id.is_normal(), id.state() == State::Normal);
        assert_eq!(id.is_enhanced(), id.state() == State::Enhanced);
        assert_eq!(id.is_diminished(), id.state() == State::Diminished);

        // Exactly one side-query and exactly one state-query hold.
        assert!(id.is_first() ^ id.is_second());
        let state_hits =
            u8::from(id.is_normal()) + u8::from(id.is_enhanced()) + u8::from(id.is_diminished());
        assert_eq!(state_hits, 1);
    }
}

// The const fns are usable in const context: these are evaluated at compile time.
const KING: Identifier = Identifier::new(Letter::ALL[10], Side::First, State::Normal, false);
const PROMOTED_TERMINAL: Identifier = KING.enhanced().with_terminal(true);
const BLACK_KING: Identifier = KING.flipped();

#[test]
fn transforms_work_in_const_context() {
    assert_eq!(KING.letter().as_char(), 'K');
    assert_eq!(KING.encode().as_str(), "K");
    assert_eq!(PROMOTED_TERMINAL.encode().as_str(), "+K^");
    assert_eq!(BLACK_KING.encode().as_str(), "k");
}

// Parsing and validation are const fns too: usable at compile time.
const PARSED: Result<Identifier, sashite_pin::ParseError> = Identifier::parse("+K^");
const KING_IS_VALID: bool = Identifier::is_valid("+K^");

#[test]
fn parsing_works_in_const_context() {
    // Each const is evaluated at compile time; comparing against a runtime call
    // both consumes it and confirms const and runtime parsing agree.
    assert_eq!(KING_IS_VALID, Identifier::is_valid("+K^"));
    assert_eq!(PARSED, Identifier::parse("+K^"));
}

#[test]
fn letter_helpers() {
    // Case folding: both cases yield the same uppercase letter.
    let upper = Letter::try_from_char('K').unwrap();
    let lower = Letter::try_from_char('k').unwrap();
    assert_eq!(upper, lower);
    assert_eq!(upper.as_char(), 'K');
    assert_eq!(upper.as_ascii(), b'K');

    // TryFrom<char>.
    assert_eq!(Letter::try_from('Z').unwrap().as_char(), 'Z');
    assert!(Letter::try_from('!').is_err());

    // from_ascii reports the side implied by case.
    let letter_a = Letter::try_from_char('A').unwrap();
    assert_eq!(Letter::from_ascii(b'A'), Some((letter_a, Side::First)));
    assert_eq!(
        Letter::from_ascii(b'a').map(|(letter, side)| (letter.as_char(), side)),
        Some(('A', Side::Second)),
    );
    assert_eq!(Letter::from_ascii(b'0'), None);

    // ALL spans A..=Z in order.
    let spelled: String = Letter::ALL.iter().map(|letter| letter.as_char()).collect();
    assert_eq!(spelled, "ABCDEFGHIJKLMNOPQRSTUVWXYZ");
}

#[test]
fn setters_are_idempotent_and_compose() {
    let queen = Letter::try_from_char('Q').unwrap();
    let rook = Letter::try_from_char('R').unwrap();

    for id in all_ids() {
        // Applying the same setter twice is the same as applying it once: each
        // one replaces an attribute outright rather than accumulating.
        assert_eq!(id.enhanced().enhanced(), id.enhanced());
        assert_eq!(id.diminished().diminished(), id.diminished());
        assert_eq!(id.normalized().normalized(), id.normalized());
        assert_eq!(
            id.with_letter(queen).with_letter(queen),
            id.with_letter(queen)
        );

        for state in [State::Diminished, State::Normal, State::Enhanced] {
            assert_eq!(id.with_state(state).with_state(state), id.with_state(state));
        }
        for side in [Side::First, Side::Second] {
            assert_eq!(id.with_side(side).with_side(side), id.with_side(side));
        }
        for terminal in [false, true] {
            assert_eq!(
                id.with_terminal(terminal).with_terminal(terminal),
                id.with_terminal(terminal),
            );
        }

        // The last write wins, whatever the order of the writes before it.
        assert_eq!(
            id.with_letter(queen).with_letter(rook),
            id.with_letter(rook)
        );
        assert_eq!(id.enhanced().diminished(), id.diminished());
        assert_eq!(id.diminished().normalized(), id.normalized());

        // Setters on distinct attributes commute, so a full rebuild by any
        // route lands on the same value as `new`.
        let rebuilt = id
            .with_letter(queen)
            .with_side(Side::Second)
            .with_state(State::Enhanced)
            .with_terminal(true);
        let other_route = id
            .with_terminal(true)
            .with_state(State::Enhanced)
            .with_side(Side::Second)
            .with_letter(queen);
        assert_eq!(rebuilt, other_route);
        assert_eq!(
            rebuilt,
            Identifier::new(queen, Side::Second, State::Enhanced, true),
        );

        // Setting every attribute back to what it already was is the identity.
        assert_eq!(
            id.with_letter(id.letter())
                .with_side(id.side())
                .with_state(id.state())
                .with_terminal(id.is_terminal()),
            id,
        );
    }
}

#[test]
fn parse_and_encode_are_mutually_inverse_and_transformations_preserve_that() {
    for id in all_ids() {
        // `parse ∘ encode = id` on the value itself, and on the value after
        // each transformation — the algebra must not be able to produce an
        // identifier that fails to round-trip.
        for derived in [
            id,
            id.flipped(),
            id.enhanced(),
            id.diminished(),
            id.normalized(),
            id.with_terminal(!id.is_terminal()),
            id.flipped().enhanced().with_terminal(true),
        ] {
            let text = derived.encode();
            assert_eq!(Identifier::parse(text.as_str()), Ok(derived), "{text:?}");
            assert_eq!(derived.to_string(), text.as_str());
            assert!(Identifier::is_valid(text.as_str()));
        }
    }
}

#[test]
fn ordering_equality_and_hashing_agree_with_one_another() {
    use std::cmp::Ordering;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn hash_of(id: Identifier) -> u64 {
        let mut hasher = DefaultHasher::new();
        id.hash(&mut hasher);
        hasher.finish()
    }

    let ids = all_ids();
    for (i, &left) in ids.iter().enumerate() {
        for (j, &right) in ids.iter().enumerate() {
            let ordering = left.cmp(&right);

            // `Ord` is consistent with `PartialOrd` and with `Eq`.
            assert_eq!(left.partial_cmp(&right), Some(ordering));
            assert_eq!(ordering == Ordering::Equal, left == right);
            assert_eq!(ordering == Ordering::Less, left < right);
            assert_eq!(ordering == Ordering::Greater, left > right);

            // The order is total and antisymmetric.
            assert_eq!(ordering.reverse(), right.cmp(&left));
            assert_eq!(ordering, i.cmp(&j), "generation order must be the order");

            // Equal values hash equally (the other direction is not required).
            if left == right {
                assert_eq!(hash_of(left), hash_of(right));
            }
        }
    }
}

#[test]
fn derived_orderings_are_canonical() {
    assert!(Side::First < Side::Second);
    assert!(State::Diminished < State::Normal);
    assert!(State::Normal < State::Enhanced);
    assert_eq!(State::default(), State::Normal);

    // Identifier compares by letter, then side, then state, then terminal.
    let less = |left: &str, right: &str| {
        Identifier::parse(left).unwrap() < Identifier::parse(right).unwrap()
    };
    assert!(less("A", "B")); // letter
    assert!(less("A", "a")); // same letter: First < Second
    assert!(less("-A", "A")); // Diminished < Normal
    assert!(less("A", "+A")); // Normal < Enhanced
    assert!(less("A", "A^")); // non-terminal < terminal
}
