#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed indices/variants and panic on unexpected shapes"
)]
//! Bounded witness/path materialization rails (ticket 05, D31).
//!
//! Proves the BOUNDED side of the witness contract: a complete request (dedup
//! identity, finite cap, path-length bound where cyclic, canonical tie-break)
//! binds into a [`quanta_gqlang_v2_logic::WitnessContractV1`]. The reject side
//! lives in `v3_logic_hellgate.rs`.

use quanta_gqlang_v2_hir::SpanV1;
use quanta_gqlang_v2_logic::{
    DedupIdentityV1, RelationShapeV1, WitnessContractV1, WitnessRequestV1, WitnessTieBreakV1,
};

fn sp(a: usize, b: usize) -> SpanV1 {
    SpanV1::new(a, b)
}

#[test]
fn cyclic_witness_with_full_bounds_binds() {
    let request = WitnessRequestV1 {
        shape: RelationShapeV1::Cyclic,
        dedup: DedupIdentityV1::NodeSequence,
        max_witnesses: Some(16),
        max_path_length: Some(8),
        tie_break: WitnessTieBreakV1::ShortestThenLexical,
        span: sp(0, 40),
    };
    let contract = WitnessContractV1::bind(&request).expect("fully bounded request binds");
    assert_eq!(contract.max_witnesses, 16);
    assert_eq!(contract.max_path_length, Some(8));
    assert_eq!(contract.dedup, DedupIdentityV1::NodeSequence);
    assert_eq!(contract.tie_break, WitnessTieBreakV1::ShortestThenLexical);
    assert_eq!(contract.dedup.keyword(), "node_sequence");
}

#[test]
fn acyclic_witness_without_path_bound_binds() {
    // An acyclic relation does not require a path-length bound (every path is
    // finite by structure).
    let request = WitnessRequestV1 {
        shape: RelationShapeV1::Acyclic,
        dedup: DedupIdentityV1::Endpoints,
        max_witnesses: Some(1),
        max_path_length: None,
        tie_break: WitnessTieBreakV1::Lexical,
        span: sp(0, 12),
    };
    let contract = WitnessContractV1::bind(&request).expect("acyclic without path bound binds");
    assert_eq!(contract.max_witnesses, 1);
    assert_eq!(contract.max_path_length, None);
    assert_eq!(contract.shape, RelationShapeV1::Acyclic);
}

// Boundary: a path-length bound of exactly ONE is the smallest *non-degenerate*
// bound (one edge can connect two distinct endpoints) and binds. This pins the
// `Some(0)`-rejects / `Some(1)`-admits boundary directly above the reject line.
#[test]
fn cyclic_path_length_bound_of_one_binds() {
    let request = WitnessRequestV1 {
        shape: RelationShapeV1::Cyclic,
        dedup: DedupIdentityV1::NodeSet,
        max_witnesses: Some(1),
        max_path_length: Some(1),
        tie_break: WitnessTieBreakV1::Lexical,
        span: sp(0, 4),
    };
    let contract = WitnessContractV1::bind(&request).expect("path-length bound of one binds");
    assert_eq!(contract.max_path_length, Some(1));
    assert_eq!(contract.max_witnesses, 1);
    assert_eq!(contract.shape, RelationShapeV1::Cyclic);
    assert_eq!(contract.dedup, DedupIdentityV1::NodeSet);
    assert_eq!(contract.tie_break, WitnessTieBreakV1::Lexical);
}

// An ACYCLIC relation may ALSO carry an explicit (non-zero) path-length bound;
// the optional bound is preserved verbatim on the bound contract.
#[test]
fn acyclic_with_explicit_path_bound_preserves_it() {
    let request = WitnessRequestV1 {
        shape: RelationShapeV1::Acyclic,
        dedup: DedupIdentityV1::NodeSequence,
        max_witnesses: Some(3),
        max_path_length: Some(2),
        tie_break: WitnessTieBreakV1::ShortestThenLexical,
        span: sp(5, 10),
    };
    let contract = WitnessContractV1::bind(&request).expect("acyclic with path bound binds");
    assert_eq!(contract.max_path_length, Some(2));
    assert_eq!(contract.shape, RelationShapeV1::Acyclic);
}

// `bind` is pure/lossless on the surviving fields: the dedup identity and
// tie-break round-trip through every variant, and the equality/copy derives
// hold (a re-bound identical request yields an equal contract).
#[test]
fn bind_round_trips_every_dedup_and_tie_break_variant() {
    for dedup in [
        DedupIdentityV1::Endpoints,
        DedupIdentityV1::NodeSequence,
        DedupIdentityV1::NodeSet,
    ] {
        for tie_break in [
            WitnessTieBreakV1::ShortestThenLexical,
            WitnessTieBreakV1::Lexical,
        ] {
            let request = WitnessRequestV1 {
                shape: RelationShapeV1::Cyclic,
                dedup,
                max_witnesses: Some(2),
                max_path_length: Some(4),
                tie_break,
                span: sp(0, 1),
            };
            let contract = WitnessContractV1::bind(&request).expect("variant binds");
            assert_eq!(contract.dedup, dedup);
            assert_eq!(contract.tie_break, tie_break);
            // Copy + structural equality: a second bind of the same request is
            // byte-for-byte equal.
            let again = WitnessContractV1::bind(&request).expect("re-bind");
            assert_eq!(contract, again);
        }
    }
}

// Stable keywords are part of the explain/diagnostics contract: assert every
// `DedupIdentityV1` and `WitnessTieBreakV1` keyword verbatim. These keywords
// must never change spelling.
#[test]
fn dedup_identity_keywords_are_stable() {
    assert_eq!(DedupIdentityV1::Endpoints.keyword(), "endpoints");
    assert_eq!(DedupIdentityV1::NodeSequence.keyword(), "node_sequence");
    assert_eq!(DedupIdentityV1::NodeSet.keyword(), "node_set");
}

#[test]
fn tie_break_keywords_are_stable() {
    assert_eq!(
        WitnessTieBreakV1::ShortestThenLexical.keyword(),
        "shortest_then_lexical"
    );
    assert_eq!(WitnessTieBreakV1::Lexical.keyword(), "lexical");
}
