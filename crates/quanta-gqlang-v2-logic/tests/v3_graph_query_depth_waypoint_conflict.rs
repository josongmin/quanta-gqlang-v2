#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed variants and panic on unexpected shapes"
)]
//! Fail-closed rail for FIND_DEPTH_VIA_SILENT_OVERRIDE (ticket 05; D10, D26).
//!
//! When a FIND query over a CYCLIC relation supplies BOTH an explicit `depth`
//! bound AND bounding waypoints (`via` / `through`), the two are competing
//! path-length binding strategies. Silently letting `depth` win drops the
//! waypoint constraint without telling the user (a silent downgrade that
//! violates D10) and accepts a seed that means two different things at once
//! (violating D26's explicit-multiplicity contract). The query must REJECT and
//! force the user to choose one binding strategy.

use quanta_gqlang_v2_hir::{
    GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, GraphRefV1,
    HirNodeIdV1, SpanV1,
};
use quanta_gqlang_v2_logic::{GraphQuerySemanticsV1, LogicDiagnosticV1};

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

// `find taint a -> b | depth=5 | via=x,y`: depth AND via both supplied over a
// cyclic relation. The waypoints would be silently ignored; the query must
// instead reject with a typed conflicting-path-bound diagnostic.
#[test]
fn taint_depth_and_via_conflict_rejects_with_typed_code() {
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("a", 5, 6)),
        sink: Some(gref("b", 10, 11)),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        depth: Some(5),
        via: vec![gref("x", 14, 15), gref("y", 16, 17)],
        ..GraphConstraintsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(1),
        sp(0, 30),
        GraphQueryKindV1::Taint,
        endpoints,
        constraints,
    );

    let err = GraphQuerySemanticsV1::classify(&query)
        .expect_err("depth + via must reject, not silently override");
    assert_eq!(err.code(), "logic.graph_conflicting_path_bound");
    assert_eq!(
        err.message(),
        "FIND `taint` supplies both an explicit `depth` bound and bounding \
         waypoints (`via`/`through`); choose one path-length binding strategy"
    );
    assert_eq!(err.span(), sp(0, 30));
    match err {
        LogicDiagnosticV1::GraphConflictingPathBound { kind, .. } => {
            assert_eq!(kind, GraphQueryKindV1::Taint);
        }
        other => panic!("expected GraphConflictingPathBound, got {other:?}"),
    }
}

// `through` is the other waypoint flavor: depth + through must reject too.
#[test]
fn reachability_depth_and_through_conflict_rejects() {
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("a", 5, 6)),
        sink: Some(gref("b", 10, 11)),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        depth: Some(3),
        through: vec![gref("m", 14, 15)],
        ..GraphConstraintsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(2),
        sp(0, 30),
        GraphQueryKindV1::Reachability,
        endpoints,
        constraints,
    );

    let err = GraphQuerySemanticsV1::classify(&query).expect_err("depth + through must reject");
    assert_eq!(err.code(), "logic.graph_conflicting_path_bound");
}

// REGRESSION GUARD: depth WITHOUT waypoints still admits (the common case must
// not be broken by the new conflict check).
#[test]
fn taint_depth_only_still_admits() {
    let endpoints = GraphEndpointsV1 {
        source: Some(gref("a", 5, 6)),
        sink: Some(gref("b", 10, 11)),
        ..GraphEndpointsV1::default()
    };
    let constraints = GraphConstraintsV1 {
        depth: Some(5),
        ..GraphConstraintsV1::default()
    };
    let query = GraphQueryHirV1::new(
        id(3),
        sp(0, 30),
        GraphQueryKindV1::Taint,
        endpoints,
        constraints,
    );

    let contract = GraphQuerySemanticsV1::classify(&query).expect("depth-only taint still admits");
    assert_eq!(contract.witness.max_path_length, Some(5));
}
