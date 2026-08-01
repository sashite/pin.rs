# Changelog

All notable changes to this crate are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this
crate adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **The README is now compiled**, and fixing it up to compile found seven broken
  examples. A `#[cfg(doctest)]` `include_str!` runs it as doctests (9 → 16);
  nothing built it before, so its examples used `?` at what would be a
  doctest's top level, and several blocks named `Identifier` without importing
  it — neither could have compiled. They now use `.expect(...)` and carry their
  own `use`, since a doctest is a whole crate rather than a continuation of the
  block above it.
- **"Parsing borrows the input bytes" was wrong.** An `Identifier` borrows
  *nothing*: it is a self-contained 4-byte `Copy` value, and no output outlives
  its input.
- The README now documents what 1.1.0 made true: `Display` renders through
  `Formatter::pad`; `EncodedPin` compares, orders and hashes by its token text;
  `encode` is `const`; and the whole-input anchoring requirement, with the
  trailing-newline trap it exists to avoid, is spelled out beside the grammar.

Documentation only — no code change, so no release is required. Note that the
README ships inside the published tarball, so crates.io and docs.rs keep showing
the 1.1.0 text until the next publish.

## [1.1.0] — 2026-07-31

A reliability review of the whole crate against the normative
[PIN v1.0.0 specification](https://sashite.dev/specs/pin/1.0.0/). **The parser
was found correct** — the accepted set is exactly the 312 valid tokens, proven
exhaustively — so nothing about which inputs are accepted or rejected has
changed. What was wrong was the reporting around it: two error messages that
said something false, formatting that dropped the format spec, a documented
guarantee the code did not deliver, and a type that could not be compared to
itself.

### Fixed

- **`Display` silently discarded the format spec**, on all three types that
  implement it — `Identifier`, `EncodedPin` and `ParseError`. Each wrote its
  bytes straight out with `write_str`, so a width, fill, alignment or precision
  was dropped: `format!("{id:>6}")` produced `"+K^"` where every other
  string-like type in Rust produces `"   +K^"`, and `{:.2}` did not truncate.
  All three now route through `Formatter::pad`, as `str` and `char` do. A plain
  `{}` and `to_string()` are byte-identical to before, so only formats that were
  already broken change.

- **`ParseError::InvalidLetter`'s message asserted something its own variant
  documentation contradicted.** It read "PIN token must contain exactly one
  ASCII letter" — a claim about the whole token — while the variant is chosen
  *positionally*, for the abbreviation slot alone. **21 216** inputs of three
  bytes or fewer yield this variant *while containing exactly one ASCII
  letter*: every `<modifier><non-letter><letter>` shape. The clearest case is
  the specification's own invalid example, which is also this crate's doctest:
  `"++K"` does contain exactly one ASCII letter, yet was told it must. The
  message is now positional: **"PIN abbreviation is not an ASCII letter"**.

- **`ParseError::InvalidStateModifier`'s parenthetical was incomplete.** It read
  "expected '+' or '-'", but at byte 0 of a two-byte token an ASCII letter is
  equally legal — that is the `X^` shape. **51 712** inputs received the
  misleading hint, the specification's `"^K"` example among them. Now: **"PIN
  token starts with neither a state modifier nor a letter"**.

- **The crate's `const` guarantee was overstated.** The docs promised a token
  could be "built, checked and spelled out at compile time", but
  `EncodedPin::as_str` was an ordinary method and `const _: &str = …` failed
  with `E0015`. Rather than weaken the claim, the method is now `const fn`: it
  reaches that by going through `split_at` and a `match` instead of a range
  index and `unwrap_or`, both of which are still non-const at the 1.81 MSRV,
  whereas `str::from_utf8` is already const there. The whole path from typed
  components to token text now runs in a `const` context, and a test pins it.

### Added

- **`EncodedPin` can be compared to itself.** It implemented `PartialEq<str>`
  and `PartialEq<&str>` but nothing reflexive, so `a == b` on two encodings did
  not compile and `assert_eq!(x.encode(), y.encode())` was a type error —
  awkward in the type whose whole purpose is to carry the canonical form.
  `PartialEq`, `Eq`, `Hash`, `PartialOrd` and `Ord` are now implemented, all
  additive. Its ordering is the token *text's*, which is deliberately **not**
  `Identifier`'s attribute ordering: `-A < +A` by identifier (Diminished before
  Enhanced) but `-A > +A` by text (`'+'` is 0x2B, `'-'` is 0x2D). 40 456 of the
  97 344 pairs diverge, and a test pins the divergence with that witness so the
  two orderings are never assumed interchangeable.

- **Exhaustive conformance evidence**, replacing sampled checks. An independent
  decision procedure written from the EBNF — not from the regex and not from the
  code — was run over every raw byte slice of length 0 to 3 (16 843 009), every
  slice of length exactly 4 (4 294 967 296, all `TooLong`), every valid UTF-8
  string up to 4 bytes (385 939 457) and every Unicode scalar (1 112 064). The
  accepted set is exactly the 312 tokens in each sweep, and all five public
  entry points return the identical `Result` — error variant included. The error
  census matches a branch-by-branch hand count exactly, which independently
  confirms the parser's control flow is the one `error.rs` documents.

- Tests closing three real coverage gaps, each proven sensitive by mutation:
  every rejection is checked to blame a position whose byte is genuinely wrong
  *and* whose already-accepted prefix was genuinely acceptable (a mutation
  making a two-byte `<letter><non-ASCII>` blame byte 0 was caught by this test
  and by nothing else); `size_of::<Identifier>() == 4` and `align == 1`, stated
  in the docs but never pinned, and guaranteed by nothing under `repr(Rust)`;
  and the `EncodedPin`-versus-`Identifier` ordering divergence above.

### Notes

- **The parser needed no change.** Its structural length check is in *bytes*, so
  the specification's whole-string anchoring requirement is met by construction
  — `"K\n"` and `"\nK"` are two bytes and never reach a valid shape. `encode ∘
  parse` is byte-for-byte identity on every accepted input, so the "MUST NOT
  normalize beyond strict validation" requirement holds.
- **Panic-freedom re-verified**, with the sites enumerated by clippy rather than
  by eye: one slicing site and three arithmetic sites, each guarded by a match
  arm or by `Letter`'s private-field invariant, which holds because the type has
  exactly three construction sites and no `Default` or `serde` back door.
- Incidental: `State`'s ordering comes from its explicit discriminants, not its
  declaration order, so reversing the declaration is a no-op. That is a
  robustness property, and the documentation is correct either way.

## [1.0.1] — 2026-06-03

### Fixed

- Dropped the removed `doc_auto_cfg` gate so docs.rs builds.

## [1.0.0] — 2026-06-02

### Added

- Initial release implementing the PIN v1.0.0 specification: `Identifier`,
  `Letter`, `Side`, `State`, `EncodedPin` and `ParseError`, with an
  allocation-free byte-level parser, `no_std` and no required dependencies, and
  optional `serde` support.

[1.1.0]: https://github.com/sashite/pin.rs/compare/v1.0.1...v1.1.0
[1.0.1]: https://github.com/sashite/pin.rs/compare/v1.0.0...v1.0.1
[1.0.0]: https://github.com/sashite/pin.rs/releases/tag/v1.0.0
