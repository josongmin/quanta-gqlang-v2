#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Hellgate (fail-closed) rails for the WITNESS / CAP / DEPTH / ORDERING
//! families (ticket 06, lane W3-B).
//!
//! Row ledger for the V1 twelve-row `v3_logic_hellgate.rs`:
//!
//! | V1 row | Disposition |
//! |---|---|
//! | 1 negation cycle, 2 mutual negation, 3 recursive aggregate, 11 aggregated self-recursion, 12 precedence | MOVED OUT to the W3-A rule/SCC owner file |
//! | 4 unbounded witness (missing cap) | positive counter-rail `every_admitted_find_is_bounded_on_both_dimensions` here + unit rail `missing_and_zero_witness_cap_reject` |
//! | 5 cyclic witness without path bound | `unbounded_find_without_depth_rejects` here |
//! | 6 zero witness cap | unit rail `missing_and_zero_witness_cap_reject` |
//! | 7 cyclic zero path-length bound | unit rail `missing_and_zero_path_length_reject_over_a_cyclic_relation` |
//! | 8 acyclic zero path-length bound | unit rail `acyclic_relation_may_omit_the_path_bound` + the deterministic reject order |
//! | 9/10 reject-order precedence | unit rail `rejection_order_is_deterministic` |
//!
//! Rows 4/6-10 are unreachable through the sealed transaction (Sema never emits
//! a missing/zero cap, `depth = 0`, or an over-budget depth), and the V2 witness
//! binding is crate-private precisely so no caller can supply one. Their exact
//! `logic.unbounded_witness.*` codes, messages, and located spans are therefore
//! proven by the owner-unit rails in `src/witness.rs`/`src/graph.rs`; this file
//! proves everything the public sealed surface can still reach.

use quanta_gqlang_v2_logic::{GraphQueryContractV2, LogicDiagnosticV1, UnboundedWitnessReasonV1};
use quanta_gqlang_v2_sema::{CheckedLocatedSpanV2, GraphQueryKindV1};

mod support;

use support::{
    bounded_query, graph_document, reference, unbounded_query, GraphReferenceRoleV2,
    ReferenceGraphReferenceV2,
};

/// The legal, bounded reference set for one subkind.
fn references(kind: GraphQueryKindV1) -> Vec<ReferenceGraphReferenceV2> {
    match kind {
        GraphQueryKindV1::Reachability
        | GraphQueryKindV1::Flow
        | GraphQueryKindV1::Taint
        | GraphQueryKindV1::Path => vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
        ],
        GraphQueryKindV1::Slice => vec![reference(GraphReferenceRoleV2::Anchor, "anchor", 5, 6)],
        GraphQueryKindV1::PointsTo => vec![
            reference(GraphReferenceRoleV2::Reference, "p", 5, 6),
            reference(GraphReferenceRoleV2::Pointee, "q", 10, 11),
        ],
        GraphQueryKindV1::Summary => {
            vec![reference(GraphReferenceRoleV2::Callable, "callable", 5, 6)]
        }
    }
}

const ALL_KINDS: [GraphQueryKindV1; 7] = [
    GraphQueryKindV1::Reachability,
    GraphQueryKindV1::Flow,
    GraphQueryKindV1::Slice,
    GraphQueryKindV1::Taint,
    GraphQueryKindV1::Path,
    GraphQueryKindV1::PointsTo,
    GraphQueryKindV1::Summary,
];

/// The exact byte range of one checked location, without naming a HIR type.
fn span_of(location: &CheckedLocatedSpanV2) -> (usize, usize) {
    let span = location.location().span;
    (span.start, span.end)
}

// V1 row 5: a FIND over a relation that can cycle with no explicit depth is an
// unbounded witness family and rejects with the exact code, message, and the
// FIND item's checked location.
#[test]
fn unbounded_find_without_depth_rejects() {
    let request = unbounded_query(
        GraphQueryKindV1::Flow,
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
        ],
        0,
        30,
    );
    let document = graph_document(request).expect("flow fixture resolves");
    let location = document.graph().location().clone();

    let err =
        GraphQueryContractV2::from_resolved(document.graph()).expect_err("depth-free flow rejects");
    assert_eq!(
        err.code(),
        "logic.unbounded_witness.missing_path_length_bound"
    );
    assert_eq!(
        err.message(),
        "unbounded witness materialization: a cyclic relation requires a path-length bound"
    );
    assert_eq!(err.span(), &location);
    assert_eq!(span_of(err.span()), (0, 30));
    match err {
        LogicDiagnosticV1::UnboundedWitness { reason, span } => {
            assert_eq!(reason, UnboundedWitnessReasonV1::MissingPathLengthBound);
            assert_eq!(&span, &location);
        }
        other => panic!("expected UnboundedWitness, got {other:?}"),
    }
}

// V1 row 5 + policy flip: bounding waypoints do NOT substitute for a depth, so a
// waypoint-only FIND is still unbounded.
#[test]
fn waypoint_count_is_not_a_path_bound() {
    let request = unbounded_query(
        GraphQueryKindV1::Taint,
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
            reference(GraphReferenceRoleV2::Via, "x", 14, 15),
            reference(GraphReferenceRoleV2::Through, "z", 18, 19),
        ],
        0,
        30,
    );
    let document = graph_document(request).expect("taint fixture resolves");

    let err = GraphQueryContractV2::from_resolved(document.graph())
        .expect_err("waypoints are not a path bound");
    assert_eq!(
        err.code(),
        "logic.unbounded_witness.missing_path_length_bound"
    );
}

// V1 row 4 counter-rail: the sealed transaction never emits a missing or zero
// cap, so every admitted FIND carries a genuinely finite, non-zero,
// document-owned cap on BOTH witness dimensions.
#[test]
fn every_admitted_find_is_bounded_on_both_dimensions() {
    let probe = graph_document(bounded_query(
        GraphQueryKindV1::Reachability,
        references(GraphQueryKindV1::Reachability),
        1,
        0,
        30,
    ))
    .expect("probe fixture");
    let sealed_cap = probe.document().budget_receipt().max_witnesses();

    for kind in ALL_KINDS {
        let document =
            graph_document(bounded_query(kind, references(kind), 4, 0, 30)).expect("fixture");
        let contract = GraphQueryContractV2::from_resolved(document.graph())
            .unwrap_or_else(|error| panic!("{kind:?} must admit: {error:?}"));
        assert_eq!(contract.max_witnesses(), sealed_cap, "{kind:?} cap");
        assert!(contract.max_witnesses() > 0, "{kind:?} cap is non-zero");
        assert_eq!(
            contract.witness().max_path_length(),
            Some(4),
            "{kind:?} depth"
        );
    }
}

// The reject is anchored at the offending FIND item's OWN checked location, not
// at a synthetic or neighbouring span.
#[test]
fn rejection_is_anchored_at_the_find_item() {
    let request = unbounded_query(
        GraphQueryKindV1::PointsTo,
        vec![
            reference(GraphReferenceRoleV2::Reference, "p", 20, 21),
            reference(GraphReferenceRoleV2::Pointee, "q", 25, 26),
        ],
        12,
        40,
    );
    let document = graph_document(request).expect("points_to fixture resolves");
    let item_location = document.graph().location().clone();

    let err = GraphQueryContractV2::from_resolved(document.graph()).expect_err("missing depth");
    assert_eq!(err.span(), &item_location);
    assert_eq!(span_of(err.span()), (12, 40));
}
