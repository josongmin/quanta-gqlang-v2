#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed indices/variants and panic on unexpected shapes"
)]
//! Punch-list rails for the logic crate (ticket 05).
//!
//! Three suspected defects from the 3-round self-audit:
//!
//! 1. `pathwitness_missing_witness`: every witness-producing FIND subkind that
//!    `classify` ADMITS must carry a bound (non-empty) witness contract. A
//!    classified contract whose witness cap is zero / unbounded would be a
//!    missing witness contract; prove the contract is always genuinely bound.
//! 2. `find_depth_zero_boundary`: a `depth = 0` constraint that reaches
//!    `classify` must NOT produce a degenerate path bound of zero. A zero
//!    path-length bound can never connect two distinct endpoints, so it is an
//!    ill-formed bound and must reject fail-closed (parallel to a zero
//!    max-witness cap), not silently admit a degenerate contract.

use quanta_gqlang_v2_hir::{
    GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, GraphRefV1,
    HirNodeIdV1, SpanV1,
};
use quanta_gqlang_v2_logic::{
    GraphQuerySemanticsV1, LogicDiagnosticV1, RelationShapeV1, UnboundedWitnessReasonV1,
    WitnessContractV1, WitnessRequestV1, WitnessTieBreakV1,
};

fn sp(a: usize, b: usize) -> SpanV1 {
    SpanV1::new(a, b)
}

fn id(n: u32) -> HirNodeIdV1 {
    HirNodeIdV1::from_raw(n)
}

fn gref(name: &str, a: usize, b: usize) -> GraphRefV1 {
    GraphRefV1 {
        name: name.to_owned(),
        span: sp(a, b),
    }
}

// Finding 1: every FIND subkind that ADMITS carries a genuinely bound witness
// contract (a finite, non-zero max-witness cap). A "missing witness contract"
// would be a zero / absent cap. Sweep every subkind with a legal, bounded shape
// and assert the witness cap is present and non-zero.
#[test]
fn every_admitted_subkind_carries_a_bound_witness_contract() {
    // (kind, endpoints, depth) for a legal, bounded shape of each subkind.
    let cases: &[(GraphQueryKindV1, GraphEndpointsV1, Option<u32>)] = &[
        (
            GraphQueryKindV1::Reachability,
            GraphEndpointsV1 {
                source: Some(gref("a", 0, 1)),
                sink: Some(gref("b", 2, 3)),
                ..GraphEndpointsV1::default()
            },
            Some(4),
        ),
        (
            GraphQueryKindV1::Flow,
            GraphEndpointsV1 {
                source: Some(gref("a", 0, 1)),
                sink: Some(gref("b", 2, 3)),
                ..GraphEndpointsV1::default()
            },
            Some(4),
        ),
        (
            GraphQueryKindV1::Taint,
            GraphEndpointsV1 {
                source: Some(gref("a", 0, 1)),
                sink: Some(gref("b", 2, 3)),
                ..GraphEndpointsV1::default()
            },
            Some(4),
        ),
        (
            GraphQueryKindV1::Path,
            GraphEndpointsV1 {
                source: Some(gref("a", 0, 1)),
                sink: Some(gref("b", 2, 3)),
                ..GraphEndpointsV1::default()
            },
            Some(4),
        ),
        (
            GraphQueryKindV1::Slice,
            GraphEndpointsV1 {
                anchor: Some(gref("a", 0, 1)),
                ..GraphEndpointsV1::default()
            },
            Some(4),
        ),
        (
            GraphQueryKindV1::Summary,
            GraphEndpointsV1 {
                callable: Some(gref("f", 0, 1)),
                ..GraphEndpointsV1::default()
            },
            Some(4),
        ),
        (
            GraphQueryKindV1::PointsTo,
            GraphEndpointsV1 {
                reference: Some(gref("p", 0, 1)),
                pointee: Some(gref("q", 2, 3)),
                ..GraphEndpointsV1::default()
            },
            None,
        ),
    ];

    for (kind, endpoints, depth) in cases {
        let constraints = GraphConstraintsV1 {
            depth: *depth,
            ..GraphConstraintsV1::default()
        };
        let query = GraphQueryHirV1::new(id(1), sp(0, 40), *kind, endpoints.clone(), constraints);
        let contract = GraphQuerySemanticsV1::classify(&query)
            .unwrap_or_else(|e| panic!("{kind:?} should admit, got {}", e.code()));
        // The witness contract must be genuinely bound: a non-zero cap is the
        // proof the witness shape is admissible, not missing.
        assert!(
            contract.witness.max_witnesses > 0,
            "{kind:?} admitted with an empty/missing witness cap"
        );
    }
}

// Finding 2: a `depth = 0` on a cyclic-relation subkind must NOT produce a
// degenerate path bound of zero. It must reject fail-closed with a typed
// zero-path-length-bound diagnostic.
#[test]
fn find_depth_zero_rejects_as_degenerate_bound() {
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("a", 5, 6)),
        sink: Some(gref("b", 10, 11)),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        depth: Some(0),
        ..GraphConstraintsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(1),
        sp(0, 30),
        GraphQueryKindV1::Reachability,
        endpoints,
        constraints,
    );

    let err = GraphQuerySemanticsV1::classify(&query).expect_err("depth=0 must reject");
    assert_eq!(err.code(), "logic.unbounded_witness.zero_path_length_bound");
    assert_eq!(
        err.message(),
        "ill-formed witness bound: a path-length bound of zero can never traverse an edge"
    );
    assert_eq!(err.span(), sp(0, 30));
    match err {
        LogicDiagnosticV1::UnboundedWitness { reason, .. } => {
            assert_eq!(reason, UnboundedWitnessReasonV1::ZeroPathLengthBound);
        }
        other => panic!("expected UnboundedWitness, got {other:?}"),
    }
}

// Finding 2 (witness-contract owner): the boundedness rule lives in the witness
// contract, so a direct witness request with a zero path-length bound rejects
// too, regardless of relation shape.
#[test]
fn witness_bind_rejects_zero_path_length_bound() {
    let request = WitnessRequestV1 {
        shape: RelationShapeV1::Cyclic,
        dedup: quanta_gqlang_v2_logic::DedupIdentityV1::NodeSequence,
        max_witnesses: Some(8),
        max_path_length: Some(0),
        tie_break: WitnessTieBreakV1::ShortestThenLexical,
        span: sp(0, 12),
    };
    let err = WitnessContractV1::bind(&request).expect_err("zero path bound rejects");
    assert_eq!(err.code(), "logic.unbounded_witness.zero_path_length_bound");
    match err {
        LogicDiagnosticV1::UnboundedWitness { reason, .. } => {
            assert_eq!(reason, UnboundedWitnessReasonV1::ZeroPathLengthBound);
        }
        other => panic!("expected UnboundedWitness, got {other:?}"),
    }
}
