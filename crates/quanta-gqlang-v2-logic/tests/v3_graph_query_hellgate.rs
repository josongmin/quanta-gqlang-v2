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

//! Hellgate (fail-closed) rails for the FIND graph contract over the sealed
//! resolved query (ticket 06, lane W3-B).
//!
//! These rails build real sealed Sema documents and seal them through
//! [`GraphQueryContractV2::from_resolved`]. They prove, at the public surface:
//!
//! - a bounded taint FIND ADMITS and retains the exact sealed witness bounds;
//! - an unbounded reachability FIND (no depth) REJECTS with
//!   `logic.unbounded_witness.missing_path_length_bound`;
//! - a `points_to` FIND without a depth REJECTS: the retired acyclic exemption
//!   is dead policy (POLICY FLIP from the V1 suite);
//! - a cyclic FIND whose only bound is a `via` waypoint REJECTS: waypoint count
//!   is not a termination bound (POLICY FLIP from the V1 suite);
//! - a subkind missing a REQUIRED endpoint REJECTS with the exact role;
//! - every subkind admits at the sealed depth limit with the sealed cap;
//! - a nested CHECK/FIND retains the same sealed contract (G8).
//!
//! Each rail asserts the ACTUAL typed code + message + checked location, not
//! merely that an error occurred.

use quanta_gqlang_v2_logic::{
    DedupIdentityV1, GraphEndpointRoleV1, GraphQueryContractV2, LogicDiagnosticV1, RelationShapeV1,
    UnboundedWitnessReasonV1,
};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, GraphQueryKindV1, GraphReferenceIdV2, ResolvedGraphReferenceV2,
    SemanticItemV2,
};

mod support;

use support::{
    bounded_query, graph_document, nested_check_document, reference, unbounded_query,
    GraphReferenceRoleV2, ReferenceGraphReferenceV2,
};

/// The exact `(role, selector, use-span)` triple of one resolved reference.
type ReferenceFacts<'a> = (GraphReferenceRoleV2, Option<&'a str>, (usize, usize));

fn facts(reference: &ResolvedGraphReferenceV2) -> ReferenceFacts<'_> {
    let span = reference.use_location().location().span;
    (
        reference.role(),
        reference
            .target()
            .host_selector()
            .map(GraphReferenceIdV2::spelling),
        (span.start, span.end),
    )
}

/// The exact byte range of one checked location, without naming a HIR type.
fn span_of(location: &CheckedLocatedSpanV2) -> (usize, usize) {
    let span = location.location().span;
    (span.start, span.end)
}

// A bounded taint FIND (source -> sink with an explicit depth) ADMITS with the
// exact sealed witness contract: depth retained, cap == the document's sealed
// max_witnesses (never an independent default), NodeSequence dedup, and the
// ordered endpoint role/target/location facts retained verbatim.
#[test]
fn bounded_taint_admits_with_exact_sealed_witness_contract() {
    let request = bounded_query(
        GraphQueryKindV1::Taint,
        vec![
            reference(GraphReferenceRoleV2::Source, "user_input", 5, 15),
            reference(GraphReferenceRoleV2::Sink, "exec", 19, 23),
        ],
        8,
        0,
        40,
    );
    let document = graph_document(request).expect("taint fixture resolves");
    let sealed_cap = document.document().budget_receipt().max_witnesses();
    let sealed_depth = document.document().budget_receipt().max_witness_depth();

    let contract =
        GraphQueryContractV2::from_resolved(document.graph()).expect("bounded taint admits");
    assert_eq!(contract.kind(), GraphQueryKindV1::Taint);
    assert_eq!(contract.item_id(), document.graph_item_id());
    assert_eq!(contract.depth(), 8);
    assert!(contract.depth() <= sealed_depth);
    assert_eq!(contract.max_witnesses(), sealed_cap);
    assert_eq!(contract.shape(), RelationShapeV1::Cyclic);
    assert_eq!(contract.dedup(), DedupIdentityV1::NodeSequence);
    assert_eq!(span_of(contract.location()), (0, 40));

    let endpoints: Vec<ReferenceFacts<'_>> = contract.endpoints().map(facts).collect();
    assert_eq!(
        endpoints,
        vec![
            (GraphReferenceRoleV2::Source, Some("user_input"), (5, 15)),
            (GraphReferenceRoleV2::Sink, Some("exec"), (19, 23)),
        ]
    );
    assert_eq!(contract.waypoints().count(), 0);
}

// An unbounded reachability FIND (no depth) REJECTS with the exact typed code,
// message, and checked location of the FIND item.
#[test]
fn unbounded_reachability_rejects_with_typed_code() {
    let request = unbounded_query(
        GraphQueryKindV1::Reachability,
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
        ],
        0,
        30,
    );
    let document = graph_document(request).expect("reachability fixture resolves");
    let location = document.graph().location().clone();

    let err = GraphQueryContractV2::from_resolved(document.graph())
        .expect_err("depth-free reachability rejects");
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
        LogicDiagnosticV1::UnboundedWitness { reason, .. } => {
            assert_eq!(reason, UnboundedWitnessReasonV1::MissingPathLengthBound);
        }
        other => panic!("expected UnboundedWitness, got {other:?}"),
    }
}

// POLICY FLIP: `points_to` no longer has an acyclic shortcut. Without an
// explicit depth it is an unbounded witness family and rejects exactly like
// every other subkind.
#[test]
fn points_to_without_depth_rejects() {
    let request = unbounded_query(
        GraphQueryKindV1::PointsTo,
        vec![
            reference(GraphReferenceRoleV2::Reference, "p", 5, 6),
            reference(GraphReferenceRoleV2::Pointee, "q", 10, 11),
        ],
        0,
        20,
    );
    let document = graph_document(request).expect("points_to fixture resolves");
    let location = document.graph().location().clone();

    let err = GraphQueryContractV2::from_resolved(document.graph())
        .expect_err("points_to without depth must reject");
    assert_eq!(
        err.code(),
        "logic.unbounded_witness.missing_path_length_bound"
    );
    assert_eq!(err.span(), &location);
    match err {
        LogicDiagnosticV1::UnboundedWitness { reason, .. } => {
            assert_eq!(reason, UnboundedWitnessReasonV1::MissingPathLengthBound);
        }
        other => panic!("expected UnboundedWitness, got {other:?}"),
    }
}

// POLICY FLIP: waypoints are path predicates, never a termination bound. A
// reachability FIND carrying only a `via` waypoint still rejects, because
// waypoint count cannot bound path length.
#[test]
fn reachability_with_via_waypoint_but_no_depth_rejects() {
    let request = unbounded_query(
        GraphQueryKindV1::Reachability,
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
            reference(GraphReferenceRoleV2::Via, "w", 14, 15),
        ],
        0,
        30,
    );
    let document = graph_document(request).expect("reachability fixture resolves");

    let err = GraphQueryContractV2::from_resolved(document.graph())
        .expect_err("waypoint-only reachability still rejects");
    assert_eq!(
        err.code(),
        "logic.unbounded_witness.missing_path_length_bound"
    );
    match err {
        LogicDiagnosticV1::UnboundedWitness { reason, .. } => {
            assert_eq!(reason, UnboundedWitnessReasonV1::MissingPathLengthBound);
        }
        other => panic!("expected UnboundedWitness, got {other:?}"),
    }
}

// A subkind missing a REQUIRED endpoint rejects with the exact role and the
// FIND item's checked location.
#[test]
fn taint_missing_sink_rejects_with_typed_code() {
    let request = bounded_query(
        GraphQueryKindV1::Taint,
        vec![reference(GraphReferenceRoleV2::Source, "user_input", 5, 15)],
        8,
        0,
        40,
    );
    let document = graph_document(request).expect("taint fixture resolves");
    let location = document.graph().location().clone();

    let err = GraphQueryContractV2::from_resolved(document.graph()).expect_err("missing sink");
    assert_eq!(err.code(), "logic.graph_endpoint_missing");
    assert_eq!(err.message(), "FIND `taint` requires a `sink` endpoint");
    assert_eq!(err.span(), &location);
    assert_eq!(span_of(err.span()), (0, 40));
    match err {
        LogicDiagnosticV1::GraphEndpointMissing { kind, role, span } => {
            assert_eq!(kind, GraphQueryKindV1::Taint);
            assert_eq!(role, GraphEndpointRoleV1::Sink);
            assert_eq!(&span, &location);
        }
        other => panic!("expected GraphEndpointMissing, got {other:?}"),
    }
}

// A bounded path FIND ADMITS and surfaces the NodeSequence dedup identity:
// distinct routes between endpoints are distinct path witnesses.
#[test]
fn bounded_path_admits_with_node_sequence_dedup() {
    let request = bounded_query(
        GraphQueryKindV1::Path,
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
        ],
        4,
        0,
        30,
    );
    let document = graph_document(request).expect("path fixture resolves");

    let contract = GraphQueryContractV2::from_resolved(document.graph()).expect("bounded path");
    assert_eq!(contract.kind(), GraphQueryKindV1::Path);
    assert_eq!(contract.dedup(), DedupIdentityV1::NodeSequence);
    assert_eq!(contract.depth(), 4);
}

// Every FIND subkind admits at the document's sealed depth limit and retains
// exactly that depth plus the sealed witness cap (the exact-limit rail; the +1
// over-budget rails are transaction-unreachable and live in the crate-private
// `graph.rs`/`witness.rs` owner-unit rails).
#[test]
fn every_subkind_admits_at_the_sealed_depth_limit() {
    let probe = graph_document(bounded_query(
        GraphQueryKindV1::Reachability,
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
        ],
        1,
        0,
        30,
    ))
    .expect("probe fixture resolves");
    let sealed_depth = probe.document().budget_receipt().max_witness_depth();
    let sealed_cap = probe.document().budget_receipt().max_witnesses();

    let source_sink = || {
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
        ]
    };
    let cases: Vec<(GraphQueryKindV1, Vec<ReferenceGraphReferenceV2>)> = vec![
        (GraphQueryKindV1::Reachability, source_sink()),
        (GraphQueryKindV1::Flow, source_sink()),
        (GraphQueryKindV1::Taint, source_sink()),
        (GraphQueryKindV1::Path, source_sink()),
        (
            GraphQueryKindV1::Slice,
            vec![reference(GraphReferenceRoleV2::Anchor, "anchor", 5, 6)],
        ),
        (
            GraphQueryKindV1::PointsTo,
            vec![
                reference(GraphReferenceRoleV2::Reference, "p", 5, 6),
                reference(GraphReferenceRoleV2::Pointee, "q", 10, 11),
            ],
        ),
        (
            GraphQueryKindV1::Summary,
            vec![reference(GraphReferenceRoleV2::Callable, "callable", 5, 6)],
        ),
    ];

    for (kind, references) in cases {
        let document =
            graph_document(bounded_query(kind, references, sealed_depth, 0, 30)).expect("fixture");
        let contract = GraphQueryContractV2::from_resolved(document.graph())
            .unwrap_or_else(|error| panic!("{kind:?} must admit at the sealed limit: {error:?}"));
        assert_eq!(contract.kind(), kind);
        assert_eq!(
            contract.depth(),
            sealed_depth,
            "{kind:?} retains exact depth"
        );
        assert_eq!(
            contract.max_witnesses(),
            sealed_cap,
            "{kind:?} retains the sealed cap"
        );
    }
}

// G8: a FIND nested inside a CHECK keeps the exact nested graph identity and the
// same sealed contract facts as a top-level FIND.
#[test]
fn nested_check_find_retains_the_sealed_contract() {
    let request = bounded_query(
        GraphQueryKindV1::Taint,
        vec![
            reference(GraphReferenceRoleV2::Source, "user_input", 5, 15),
            reference(GraphReferenceRoleV2::Sink, "exec", 19, 23),
            reference(GraphReferenceRoleV2::Via, "hop", 27, 30),
            reference(GraphReferenceRoleV2::Through, "mid", 33, 36),
        ],
        6,
        0,
        40,
    );
    let document = nested_check_document(request).expect("nested CHECK fixture resolves");

    let Some(SemanticItemV2::Check(check)) = document.document().item(document.check_item_id())
    else {
        panic!("fixture CHECK item must resolve to a resolved CHECK");
    };
    let nested = check.nested_graph().expect("CHECK keeps its nested FIND");
    assert_eq!(nested.item_id(), document.nested_graph_item_id());

    let contract = GraphQueryContractV2::from_resolved(nested).expect("nested FIND admits");
    assert_eq!(contract.item_id(), document.nested_graph_item_id());
    assert_eq!(contract.kind(), GraphQueryKindV1::Taint);
    assert_eq!(contract.depth(), 6);
    assert_eq!(
        contract.max_witnesses(),
        document.document().budget_receipt().max_witnesses()
    );
    let waypoints: Vec<GraphReferenceRoleV2> = contract
        .waypoints()
        .map(|waypoint| waypoint.role())
        .collect();
    assert_eq!(
        waypoints,
        vec![GraphReferenceRoleV2::Via, GraphReferenceRoleV2::Through]
    );
}
