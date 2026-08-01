//! Value-semantics and real-world usage tests.
//!
//! Boolean queries are covered in `transformations.rs`. This file checks the
//! attributes decoded from the documented piece-set mappings of the spec's
//! examples page, the behaviour of the derived `Eq` / `Hash` / `Ord`
//! implementations when identifiers are used in collections, and the same
//! properties for [`sashite_pin::EncodedPin`], whose comparisons are written by
//! hand rather than derived.

use sashite_pin::{EncodedPin, Identifier, Letter, Side, State};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};

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
    assert_eq!(
        set.len(),
        312,
        "all 312 identifiers must be distinct under Hash + Eq"
    );

    // Re-inserting existing values does not grow the set.
    let mut reinserted = set;
    reinserted.extend(ids.iter().copied());
    assert_eq!(reinserted.len(), 312);
}

#[test]
fn usable_as_map_keys() {
    let mut map: HashMap<Identifier, &str> = HashMap::new();
    map.insert(
        Identifier::parse("+K^").unwrap(),
        "enhanced terminal first king",
    );
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
    assert_eq!(
        scrambled, canonical,
        "sorting must reproduce canonical order"
    );
}

/// Hashes anything hashable with the standard library's default hasher.
fn hash_of<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn encodings_compare_to_each_other() {
    // The reflexive comparison: two independently produced encodings of the
    // same token are equal, and equal to the token text on both sides.
    let a = Identifier::parse("+K^").unwrap().encode();
    let b = Identifier::new(
        Letter::try_from_char('K').unwrap(),
        Side::First,
        State::Enhanced,
        true,
    )
    .encode();
    assert_eq!(a, b);
    assert_eq!(a, "+K^");
    assert_eq!("+K^", b);
    assert!(a <= b);
    assert!(a >= b);
    assert_eq!(a.cmp(&b), std::cmp::Ordering::Equal);
    assert_eq!(hash_of(&a), hash_of(&b));
}

#[test]
fn a_shorter_encoding_never_equals_a_longer_one() {
    // `EncodedPin` stores its bytes in a fixed three-byte buffer with a length,
    // so a naive whole-buffer comparison would risk letting tokens of different
    // lengths alias. Each pair below shares a prefix and differs only in what
    // follows, which is exactly the case such a comparison could get wrong.
    let enc = |text: &str| Identifier::parse(text).unwrap().encode();

    for (short, long) in [
        ("K", "K^"),   // 1 vs 2 bytes, common prefix
        ("K", "+K"),   // 1 vs 2 bytes, common suffix
        ("K", "+K^"),  // 1 vs 3 bytes
        ("+K", "+K^"), // 2 vs 3 bytes, common prefix
        ("K^", "+K^"), // 2 vs 3 bytes, common suffix
        ("k", "k^"),
        ("-z", "-z^"),
    ] {
        let (s, l) = (enc(short), enc(long));
        assert_ne!(s, l, "{short:?} must not equal {long:?}");
        assert_ne!(l, s, "{long:?} must not equal {short:?}");
        assert_ne!(s.len(), l.len());
        // Ordering follows the text, and a proper prefix sorts first.
        assert_eq!(s.cmp(&l), short.cmp(long), "{short:?} vs {long:?}");
        // Cross-type comparison agrees with the self-comparison.
        assert_ne!(s, long);
        assert_ne!(l, short);
    }
}

#[test]
fn all_encodings_are_distinct_and_hash_like_their_text() {
    let ids = all_ids();
    let encodings: Vec<EncodedPin> = ids.iter().map(|id| id.encode()).collect();

    // Distinct under Eq + Hash, and under Ord.
    let hashed: HashSet<EncodedPin> = encodings.iter().copied().collect();
    assert_eq!(hashed.len(), 312, "all 312 encodings must be distinct");
    let sorted: BTreeSet<EncodedPin> = encodings.iter().copied().collect();
    assert_eq!(sorted.len(), 312);

    for (i, left) in encodings.iter().enumerate() {
        // Hash matches the `str` it compares equal to, so an `EncodedPin` can
        // be looked up interchangeably with its token text.
        assert_eq!(hash_of(left), hash_of(&left.as_str()));

        for (j, right) in encodings.iter().enumerate() {
            let text_ord = left.as_str().cmp(right.as_str());
            // Eq, Ord and Hash agree with each other and with the text.
            assert_eq!(left == right, i == j, "{left:?} vs {right:?}");
            assert_eq!(left.cmp(right), text_ord, "{left:?} vs {right:?}");
            assert_eq!(left.partial_cmp(right), Some(text_ord));
            if left == right {
                assert_eq!(hash_of(left), hash_of(right));
            }
        }
    }

    // Sorting encodings reproduces lexicographic order on the token text.
    let mut by_value: Vec<EncodedPin> = encodings.clone();
    by_value.sort_unstable();
    let mut by_text: Vec<&str> = encodings.iter().map(EncodedPin::as_str).collect();
    by_text.sort_unstable();
    let sorted_text: Vec<&str> = by_value.iter().map(EncodedPin::as_str).collect();
    assert_eq!(sorted_text, by_text);
}

#[test]
fn encodings_are_usable_as_collection_keys() {
    let mut map: HashMap<EncodedPin, u32> = HashMap::new();
    for id in all_ids() {
        *map.entry(id.encode()).or_default() += 1;
    }
    assert_eq!(map.len(), 312);
    assert!(map.values().all(|&count| count == 1));
    assert_eq!(
        map.get(&Identifier::parse("+K^").unwrap().encode()),
        Some(&1)
    );
}

#[test]
fn an_identifier_is_the_four_byte_copy_value_the_documentation_promises() {
    // The crate documentation states the size outright, and callers pack
    // identifiers into arrays and structs on the strength of it. Nothing in the
    // language guarantees the layout of a `repr(Rust)` type, so the claim rests
    // on there being four one-byte fields at one-byte alignment, leaving no
    // room for padding. That is true today; this fails loudly if it stops being.
    assert_eq!(size_of::<Identifier>(), 4);
    assert_eq!(align_of::<Identifier>(), 1);

    // `Copy` is the other half of the same sentence, and the reason every
    // transformation can take `self` by value: reading a value after it has
    // been passed on compiles only for a `Copy` type.
    let id = Identifier::parse("+K^").unwrap();
    let passed_on = id;
    assert_eq!(passed_on, id);
}

#[test]
fn encoding_order_is_genuinely_not_identifier_order() {
    // `EncodedPin` documents that it orders lexicographically by token text and
    // that this is *not* the order `Identifier` derives. A claim that two
    // orderings differ is worth nothing without a witness, so here is one.
    //
    // `-A` and `+A` share a letter and a side, so `Identifier` lets `State`
    // decide and `Diminished` sorts below `Enhanced`. The text inverts it: '+'
    // is 0x2B and '-' is 0x2D, so `"+A"` sorts first.
    let diminished = Identifier::parse("-A").unwrap();
    let enhanced = Identifier::parse("+A").unwrap();
    assert!(diminished < enhanced);
    assert!(diminished.encode() > enhanced.encode());

    // The divergence is structural rather than a single quirk: sorting the
    // whole closed domain by each ordering yields different sequences, and each
    // one starts where its own rule says it should.
    let mut by_identifier = all_ids();
    by_identifier.sort_unstable();
    let mut by_text = all_ids();
    by_text.sort_unstable_by_key(|id| id.encode());
    assert_ne!(by_identifier, by_text);
    assert_eq!(by_identifier[0].encode(), "-A");
    assert_eq!(by_text[0].encode(), "+A");
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
