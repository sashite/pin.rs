# sashite-pin

[![Crates.io](https://img.shields.io/crates/v/sashite-pin.svg)](https://crates.io/crates/sashite-pin)
[![Docs.rs](https://docs.rs/sashite-pin/badge.svg)](https://docs.rs/sashite-pin)
[![CI](https://github.com/sashite/pin.rs/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/sashite/pin.rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/sashite-pin.svg)](https://github.com/sashite/pin.rs/blob/main/LICENSE)

> **PIN** (Piece Identifier Notation) implementation for Rust.

## Overview

This crate implements the [PIN Specification v1.0.0](https://sashite.dev/specs/pin/1.0.0/).

PIN is a compact, ASCII-only token format that encodes a **piece identity** at the
level of notation: the tuple (piece name, side, state, terminal status). The case
of a single letter encodes the side, an optional `+`/`-` prefix encodes the state,
and an optional `^` suffix marks a terminal piece.

```text
[<state-modifier>]<abbr>[<terminal-marker>]      e.g.  K   +R   k^   -K^   +G^
```

PIN standardizes only the *encoding*. What a state means, which pieces are
terminal, and how players are named are all left to the rule system — see the
[Game Protocol](https://sashite.dev/game-protocol/) and [Glossary](https://sashite.dev/glossary/).

### Implementation constraints

| Property            | Value                | Rationale                                                  |
|---------------------|----------------------|------------------------------------------------------------|
| Token length        | 1–3 bytes            | `^[+-]?[A-Za-z]\^?$` per the specification                  |
| Closed domain       | 312 tokens           | 26 letters × 2 sides × 3 states × 2 terminal flags         |
| `Identifier` size   | 4 bytes, `Copy`      | stored inline; parsing and encoding never allocate         |
| Dependencies        | none required        | zero by default; `serde` is an optional, `no_std` add-on   |
| `unsafe`            | forbidden            | the crate is built under a forbid-`unsafe` lint policy     |
| MSRV                | 1.81                 | for `core::error::Error` without a `std` feature           |

## Installation

```sh
cargo add sashite-pin
```

Or add it manually to `Cargo.toml`:

```toml
[dependencies]
sashite-pin = "1"
```

## Cargo features

- **`serde`** *(off by default)* — implements `Serialize` / `Deserialize` for
  `Identifier`, (de)serializing it as its canonical token string (e.g.
  `"+K^"`). Enabling it keeps the crate `no_std`.

```toml
[dependencies]
sashite-pin = { version = "1", features = ["serde"] }
```

## Usage

### Parsing

```rust
use sashite_pin::{Identifier, Side, State};

let king: Identifier = "+K^".parse().expect("a valid PIN token"); // via FromStr
let rook = Identifier::parse("r").expect("a valid PIN token");    // inherent

assert_eq!(king.letter().as_char(), 'K');
assert_eq!(king.side(), Side::First);
assert_eq!(king.state(), State::Enhanced);
assert!(king.is_terminal());

assert_eq!(rook.side(), Side::Second);
```

### Building from typed components

Construction is **infallible**: because each component type is valid by
construction, every combination denotes a valid token.

```rust
use sashite_pin::{Identifier, Letter, Side, State};

let pawn = Identifier::new(
    Letter::try_from_char('P').expect("an ASCII letter"),
    Side::Second,
    State::Enhanced,
    false,
);
assert_eq!(pawn.encode().as_str(), "+p");
```

### Encoding and formatting

`encode` returns an allocation-free, fixed-buffer string view that dereferences
to `str`; `Display` writes the same canonical form.

```rust
use sashite_pin::Identifier;

let id = Identifier::parse("+K^").expect("a valid PIN token");
assert_eq!(id.encode().as_str(), "+K^");
assert_eq!(id.to_string(), "+K^");           // requires `alloc`/`std`
```

`Display` routes through `Formatter::pad`, so a token honours the format spec
exactly as a `str` does — width, fill, alignment and precision all apply. The
same holds for `EncodedPin` and for `ParseError`'s message:

```rust
use sashite_pin::Identifier;
let id = Identifier::parse("+K^").expect("a valid PIN token");
assert_eq!(format!("{id:>6}"), "   +K^");
assert_eq!(format!("{id:*<6}"), "+K^***");
```

An `EncodedPin` compares, orders and hashes by its token text, so it behaves
like the `&str` it stands for — against another encoding or against a string:

```rust
use sashite_pin::Identifier;
let a = Identifier::parse("K").expect("a valid PIN token").encode();
let b = Identifier::parse("K^").expect("a valid PIN token").encode();
assert_ne!(a, b);
assert!(a < b);                              // lexicographic, like `str`
assert_eq!(a, "K");                          // and directly against a string
```

### Validation

```rust
use sashite_pin::Identifier;

assert!(Identifier::is_valid("+K^"));
assert!(!Identifier::is_valid("K+"));         // a modifier must be a prefix
```

### Transformations and queries

Every transformation returns a new value (the type is `Copy`, so this is cheap):

```rust
use sashite_pin::Identifier;

let white = Identifier::parse("+P").expect("a valid PIN token");
assert_eq!(white.flipped().encode().as_str(), "+p");
assert_eq!(white.normalized().encode().as_str(), "P");
assert_eq!(white.diminished().with_terminal(true).encode().as_str(), "-P^");

assert!(white.is_first());
assert!(white.is_enhanced());
```

## Token format

The grammar (EBNF) is:

```ebnf
pin              ::= [ state-modifier ] abbr [ terminal-marker ] ;
state-modifier   ::= "+" | "-" ;
abbr             ::= "A"…"Z" | "a"…"z" ;
terminal-marker  ::= "^" ;
```

A token maps to exactly four attributes:

| Component        | Encodes          | Values                                    |
|------------------|------------------|-------------------------------------------|
| letter case      | side             | uppercase → `First`, lowercase → `Second` |
| letter           | piece name       | a single-letter abbreviation (`A`–`Z`)    |
| `+` / `-` prefix | state            | `Enhanced` / `Diminished` (else `Normal`) |
| `^` suffix       | terminal status  | present → terminal piece                  |

The regex must be anchored to the **whole** input. In an engine where `$` also
matches before a trailing newline, `^[+-]?[A-Za-z]\^?$` would accept `"K\n"`;
this crate rejects it, and its conformance tests use `\A`…`\z` so the oracle
cannot drift into that trap either.

Letters are not reserved: the mapping from abbreviation to full piece name is
defined entirely by the rule system. See the
[examples page](https://sashite.dev/specs/pin/1.0.0/examples/) for sample
mappings (chess, shogi, xiangqi, makruk).

## Design and guarantees

- **`no_std` and allocation-free.** An `Identifier` borrows *nothing* — it is a
  self-contained 4-byte `Copy` value — and `EncodedPin` keeps the ≤ 3 output
  bytes in a fixed inline buffer. Nothing touches the heap, and no output
  outlives its input.
- **No `unsafe`, no regex engine.** The parser matches raw bytes directly,
  eliminating ReDoS as an attack vector.
- **Bounded, panic-free parsing.** Inputs longer than three bytes are rejected on
  a structural length check before any byte is inspected, and the public parsing
  API returns a `Result` rather than panicking.
- **`const`-friendly.** Construction, parsing, validation, the accessors, the
  transformations and `encode` are all `const fn`, so identifiers can be built,
  checked and spelled out at compile time.
- **Total component construction.** With valid-by-construction component types,
  building an identifier from its parts cannot fail.

### Performance

The hot paths run in single-digit nanoseconds per call. Indicative figures from
`benches/parse.rs`:

| Operation                 | Time    |
|---------------------------|---------|
| parse `+K^` (3 bytes)     | ~3.2 ns |
| parse `K` (1 byte)        | ~3.6 ns |
| reject over-long input    | ~1.8 ns |

Run them with `cargo bench`.

## Related specifications

- [Game Protocol](https://sashite.dev/game-protocol/) — the conceptual foundation
- [PIN Specification v1.0.0](https://sashite.dev/specs/pin/1.0.0/) — the normative document
- [PIN Examples](https://sashite.dev/specs/pin/1.0.0/examples/) — sample piece-set mappings

Reference implementations in other languages are maintained by Sashité:
[Elixir](https://github.com/sashite/pin.ex),
[Go](https://github.com/sashite/pin.go),
[Ruby](https://github.com/sashite/pin.rb).

If a library's behavior appears to conflict with the specification, the
specification is normative.

## License

Available as open source under the terms of the [Apache License 2.0](https://opensource.org/licenses/Apache-2.0).
