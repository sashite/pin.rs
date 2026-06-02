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
