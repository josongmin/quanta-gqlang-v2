#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed indices/variants and panic on unexpected shapes"
)]
//! Hellgate (fail-closed) rails for FIND graph-query SOURCE semantics (ticket 05,
//! D31). Proves that each FIND subkind carried by
//! [`quanta_gqlang_v2_hir::GraphQueryHirV1`] is classified at the FRONTEND
//! contract: legality of endpoint presence + bounded-witness contract, never
//! traversal execution.
//!
//! - A bounded taint FIND (source -> sink, depth bound) ADMITS with a witness
//!   contract.
//! - An unbounded reachability FIND (no depth, no waypoints over a cyclic
//!   relation) REJECTS with a typed `logic.unbounded_witness.*` diagnostic.
//! - A points_to FIND ADMITS (alias resolution is acyclic; no path bound
//!   required).
//! - A subkind missing a REQUIRED endpoint REJECTS with a typed
//!   `logic.graph_endpoint_missing` diagnostic.
//!
//! Each rail asserts the ACTUAL typed code + message, not merely that an error
//! occurred. These rails fail RED before the graph-query impl exists.

use quanta_gqlang_v2_hir::{
    GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, GraphRefV1,
    HirNodeIdV1, SpanV1,
};
use quanta_gqlang_v2_logic::{
    GraphEndpointRoleV1, GraphQueryContractV1, GraphQuerySemanticsV1, LogicDiagnosticV1,
    UnboundedWitnessReasonV1,
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

// Hellgate: a bounded taint FIND (source -> sink with an explicit depth bound)
// ADMITS with a witness contract.
#[test]
fn bounded_taint_admits_with_witness_contract() {
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("user_input", 5, 15)),
        sink: Some(gref("exec", 19, 23)),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        depth: Some(8),
        ..GraphConstraintsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(1),
        sp(0, 40),
        GraphQueryKindV1::Taint,
        endpoints,
        constraints,
    );

    let contract = GraphQuerySemanticsV1::classify(&query).expect("bounded taint admits");
    assert_eq!(contract.kind, GraphQueryKindV1::Taint);
    // taint flows over a cyclic call/flow graph: a path bound is mandatory and
    // present here.
    assert_eq!(contract.witness.max_path_length, Some(8));
    assert!(contract.witness.max_witnesses > 0);
}

// Hellgate: an unbounded reachability FIND (no depth, no waypoints over a cyclic
// relation) REJECTS with a typed diagnostic.
#[test]
fn unbounded_reachability_rejects_with_typed_code() {
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("a", 5, 6)),
        sink: Some(gref("b", 10, 11)),
        ..GraphEndpointsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(1),
        sp(0, 30),
        GraphQueryKindV1::Reachability,
        endpoints,
        GraphConstraintsV1::default(),
    );

    let err = GraphQuerySemanticsV1::classify(&query).expect_err("unbounded reachability rejects");
    assert_eq!(
        err.code(),
        "logic.unbounded_witness.missing_path_length_bound"
    );
    assert_eq!(
        err.message(),
        "unbounded witness materialization: a cyclic relation requires a path-length bound"
    );
    assert_eq!(err.span(), sp(0, 30));
    match err {
        LogicDiagnosticV1::UnboundedWitness { reason, .. } => {
            assert_eq!(reason, UnboundedWitnessReasonV1::MissingPathLengthBound);
        }
        other => panic!("expected UnboundedWitness, got {other:?}"),
    }
}

// Hellgate: a points_to FIND ADMITS. Alias resolution is acyclic by treatment,
// so no path-length bound is required.
#[test]
fn points_to_admits_without_path_bound() {
    let endpoints = GraphEndpointsV1 {
        reference: Some(gref("p", 5, 6)),
        pointee: Some(gref("q", 10, 11)),
        ..GraphEndpointsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(1),
        sp(0, 20),
        GraphQueryKindV1::PointsTo,
        endpoints,
        GraphConstraintsV1::default(),
    );

    let contract = GraphQuerySemanticsV1::classify(&query).expect("points_to admits");
    assert_eq!(contract.kind, GraphQueryKindV1::PointsTo);
    assert_eq!(contract.witness.max_path_length, None);
}

// A subkind missing a REQUIRED endpoint rejects with a typed diagnostic. Taint
// requires a sink; omitting it must fail closed.
#[test]
fn taint_missing_sink_rejects_with_typed_code() {
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("user_input", 5, 15)),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        depth: Some(8),
        ..GraphConstraintsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(1),
        sp(0, 40),
        GraphQueryKindV1::Taint,
        endpoints,
        constraints,
    );

    let err = GraphQuerySemanticsV1::classify(&query).expect_err("missing sink rejects");
    assert_eq!(err.code(), "logic.graph_endpoint_missing");
    assert_eq!(err.message(), "FIND `taint` requires a `sink` endpoint");
    assert_eq!(err.span(), sp(0, 40));
    match err {
        LogicDiagnosticV1::GraphEndpointMissing { kind, role, .. } => {
            assert_eq!(kind, GraphQueryKindV1::Taint);
            assert_eq!(role, GraphEndpointRoleV1::Sink);
        }
        other => panic!("expected GraphEndpointMissing, got {other:?}"),
    }
}

// A cyclic reachability with a `via` waypoint but no depth still ADMITS: the
// waypoint set bounds enumeration. (Frontend legality, not execution.)
#[test]
fn reachability_with_via_waypoint_admits() {
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("a", 5, 6)),
        sink: Some(gref("b", 10, 11)),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        via: vec![gref("w", 14, 15)],
        ..GraphConstraintsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(1),
        sp(0, 30),
        GraphQueryKindV1::Reachability,
        endpoints,
        constraints,
    );

    let contract =
        GraphQuerySemanticsV1::classify(&query).expect("reachability with waypoint admits");
    assert_eq!(contract.kind, GraphQueryKindV1::Reachability);
}

// A bounded path FIND ADMITS and surfaces the NodeSequence dedup identity:
// distinct routes between endpoints are distinct path witnesses.
#[test]
fn bounded_path_admits_with_node_sequence_dedup() {
    use quanta_gqlang_v2_logic::DedupIdentityV1;
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("a", 5, 6)),
        sink: Some(gref("b", 10, 11)),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        depth: Some(4),
        ..GraphConstraintsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(1),
        sp(0, 30),
        GraphQueryKindV1::Path,
        endpoints,
        constraints,
    );

    let contract: GraphQueryContractV1 =
        GraphQuerySemanticsV1::classify(&query).expect("bounded path admits");
    assert_eq!(contract.witness.dedup, DedupIdentityV1::NodeSequence);
}
