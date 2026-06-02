//! Value-semantics and real-world usage tests.
//!
//! Boolean queries are covered in `transformations.rs`. This file checks the
//! attributes decoded from the documented piece-set mappings of the spec's
//! examples page, and the behaviour of the derived `Eq` / `Hash` / `Ord`
//! implementations when identifiers are used in collections.

use sashite_pin::{Identifier, Letter, Side, State};
use std::collections::{HashMap, HashSet};

/// Builds all 312 identifiers in canonical (ascending) order.
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

/// Tokens drawn from the spec's examples page, with the attributes they must
/// decode to. The conventions (e.g. `-` for "in check", `+` for "promoted" or
/// "river crossed") are rule-system meanings; PIN only encodes the markers.
const PIECE_SET_CASES: &[(&str, char, Side, State, bool)] = &[
    // Western chess.
    ("K^", 'K', Side::First, State::Normal, true), // king (terminal)
    ("k^", 'K', Side::Second, State::Normal, true),
    ("-K^", 'K', Side::First, State::Diminished, true), // king in check
    ("Q", 'Q', Side::First, State::Normal, false),
    ("-R", 'R', Side::First, State::Diminished, false), // castling temporarily blocked
    ("+R", 'R', Side::First, State::Enhanced, false),   // castling available
    ("+P", 'P', Side::First, State::Enhanced, false),   // double-step eligible
    ("-P", 'P', Side::First, State::Diminished, false), // en-passant vulnerable
    // Japanese shogi (promotion as the enhanced state).
    ("+B", 'B', Side::First, State::Enhanced, false),
    ("+S", 'S', Side::First, State::Enhanced, false),
    ("+N", 'N', Side::First, State::Enhanced, false),
    ("+L", 'L', Side::First, State::Enhanced, false),
    ("+r", 'R', Side::Second, State::Enhanced, false),
    ("g", 'G', Side::Second, State::Normal, false),
    // Chinese xiangqi.
    ("G^", 'G', Side::First, State::Normal, true), // general (terminal)
    ("-g^", 'G', Side::Second, State::Diminished, true), // general in check
    ("+G^", 'G', Side::First, State::Enhanced, true), // flying-general available (prefix + suffix)
    ("E", 'E', Side::First, State::Normal, false),
    ("C", 'C', Side::First, State::Normal, false),
    ("+s", 'S', Side::Second, State::Enhanced, false), // soldier crossed the river
    // Thai makruk.
    ("M", 'M', Side::First, State::Normal, false), // met
    ("m", 'M', Side::Second, State::Normal, false),
    ("+p", 'P', Side::Second, State::Enhanced, false), // promoted pawn (bia ngai)
];

#[test]
fn real_world_piece_sets_decode_as_documented() {
    for &(token, letter, side, state, terminal) in PIECE_SET_CASES {
        let id = Identifier::parse(token).unwrap_or_else(|e| panic!("{token:?}: {e:?}"));
        assert_eq!(id.letter().as_char(), letter, "{token:?}");
        assert_eq!(id.side(), side, "{token:?}");
        assert_eq!(id.state(), state, "{token:?}");
        assert_eq!(id.is_terminal(), terminal, "{token:?}");
        assert_eq!(id.encode().as_str(), token, "{token:?} round-trip");
    }
}

#[test]
fn equality_distinguishes_every_attribute() {
    let base = Identifier::parse("K").unwrap();
    assert_eq!(base, Identifier::parse("K").unwrap()); // reflexive across parses
    assert_ne!(base, Identifier::parse("k").unwrap()); // side differs
    assert_ne!(base, Identifier::parse("+K").unwrap()); // state differs
    assert_ne!(base, Identifier::parse("K^").unwrap()); // terminal differs
    assert_ne!(base, Identifier::parse("Q").unwrap()); // letter differs
}

#[test]
fn parses_from_bytes() {
    // `TryFrom<&[u8]>` parses raw bytes and agrees with the string path.
    assert_eq!(
        Identifier::try_from(&b"+K^"[..]).unwrap(),
        Identifier::parse("+K^").unwrap(),
    );
    assert!(Identifier::try_from(&b"K+"[..]).is_err());
    // Non-UTF-8 bytes are rejected gracefully, not via a panic.
    assert!(Identifier::try_from(&[0xFF_u8, 0xFE][..]).is_err());
}

#[test]
fn all_identifiers_are_distinct_and_hashable() {
    let ids = all_ids();
    let set: HashSet<Identifier> = ids.iter().copied().collect();
    assert_eq!(set.len(), 312, "all 312 identifiers must be distinct under Hash + Eq");

    // Re-inserting existing values does not grow the set.
    let mut reinserted = set;
    reinserted.extend(ids.iter().copied());
    assert_eq!(reinserted.len(), 312);
}

#[test]
fn usable_as_map_keys() {
    let mut map: HashMap<Identifier, &str> = HashMap::new();
    map.insert(Identifier::parse("+K^").unwrap(), "enhanced terminal first king");
    map.insert(Identifier::parse("p").unwrap(), "second pawn");

    assert_eq!(
        map.get(&Identifier::parse("+K^").unwrap()),
        Some(&"enhanced terminal first king"),
    );
    // A different side is a different key.
    assert_eq!(map.get(&Identifier::parse("P").unwrap()), None);
}

#[test]
fn sorting_yields_canonical_order() {
    let canonical = all_ids(); // generated ascending: letter, side, state, terminal
    assert!(
        canonical.windows(2).all(|w| w[0] < w[1]),
        "generation order must be strictly ascending and duplicate-free",
    );

    let mut scrambled = canonical.clone();
    scrambled.reverse();
    scrambled.sort();
    assert_eq!(scrambled, canonical, "sorting must reproduce canonical order");
}

#[test]
fn debug_output_is_informative() {
    let id = Identifier::parse("+K^").unwrap();
    let rendered = format!("{id:?}");
    assert!(rendered.contains("Letter('K')"), "{rendered}");
    assert!(rendered.contains("Enhanced"), "{rendered}");
    assert!(rendered.contains("First"), "{rendered}");

    // EncodedPin's Debug shows the token text.
    assert_eq!(format!("{:?}", id.encode()), "EncodedPin(\"+K^\")");
}
