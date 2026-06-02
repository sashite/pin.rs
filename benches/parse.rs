//! Criterion micro-benchmarks for the hot paths: parsing, encoding, and the
//! parse → encode round trip.

// `criterion_group!` expands to an undocumented public item; a benchmark crate
// is not a public API, so the crate-wide `missing_docs` lint does not apply.
#![allow(missing_docs)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use sashite_pin::Identifier;

/// Valid tokens covering every structural shape (1–3 bytes, with/without a
/// modifier and/or terminal marker).
const VALID: &[&str] = &["K", "+R", "K^", "+K^"];

/// Rejected inputs spanning the failure modes (empty, bad shape, too long).
const INVALID: &[&str] = &["", "KK", "++K", "KKKK"];

fn parse_valid(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_valid");
    for &token in VALID {
        group.bench_with_input(BenchmarkId::from_parameter(token), token, |b, t| {
            b.iter(|| Identifier::parse(black_box(t)));
        });
    }
    group.finish();
}

fn parse_invalid(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_invalid");
    for &token in INVALID {
        let label = if token.is_empty() { "<empty>" } else { token };
        group.bench_with_input(BenchmarkId::from_parameter(label), token, |b, t| {
            b.iter(|| Identifier::parse(black_box(t)));
        });
    }
    group.finish();
}

fn encode(c: &mut Criterion) {
    let id = Identifier::parse("+K^").expect("valid token");
    c.bench_function("encode", |b| {
        b.iter(|| black_box(id).encode());
    });
}

fn round_trip(c: &mut Criterion) {
    c.bench_function("round_trip", |b| {
        b.iter(|| Identifier::parse(black_box("+K^")).map(Identifier::encode));
    });
}

criterion_group!(benches, parse_valid, parse_invalid, encode, round_trip);
criterion_main!(benches);
