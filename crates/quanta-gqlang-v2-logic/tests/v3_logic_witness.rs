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
