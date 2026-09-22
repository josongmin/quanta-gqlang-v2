#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Mutation rails for the FIND graph/witness owner (ticket 06, lane W3-B;
//! moved from `v3_logic_punchlist.rs`).
//!
//! Each rail is written to KILL one deliberate source mutation of the graph or
//! witness contract. The mutation is named in the rail's doc comment; the
//! assertions pin the exact semantic payload a mutant would change.
//!
//! Transaction-reachable mutations (killed here):
//!
//! 1. `waypoint_count_as_depth` — reinstate V1's "waypoint count bounds the
//!    path" inference.
//! 2. `points_to_acyclic_exemption` — restore the retired acyclic bypass.
//! 3. `depth_waypoint_conflict` — restore the retired conflict policy that
//!    rejected depth plus waypoints.
//! 4. `default_max_witnesses` — hardcode an independent cap (the retired
//!    `DEFAULT_MAX_WITNESSES = 64`).
//! 5. `discard_tie_break` — drop or change the canonical tie-break.
//! 6. `erase_waypoint_order` — sort, dedup, or drop resolved waypoints.
//! 7. `any_endpoint_role` — report an arbitrary endpoint role/subkind.
//! 8. `admit_missing_depth` — treat a depth-free FIND as admissible.
//! 9. `missing_witness_contract` — admit a FIND whose witness cap is absent or
//!    degenerate.
//!
//! Transaction-UNREACHABLE mutations (zero/over-budget bounds are rejected by
//! Sema before Logic sees them) are killed by the crate-private owner-unit rails
//! `one_over_sealed_witness_cap_rejects_over_budget`,
//! `one_over_sealed_witness_depth_rejects_over_budget`,
//! `missing_and_zero_witness_cap_reject`,
//! `missing_and_zero_path_length_reject_over_a_cyclic_relation`, and
//! `rejection_order_is_deterministic` in `src/witness.rs`.

use quanta_gqlang_v2_logic::{
    DedupIdentityV1, GraphEndpointRoleV1, GraphQueryContractV2, LogicDiagnosticV1,
    WitnessTieBreakV1,
};
use quanta_gqlang_v2_sema::{GraphQueryKindV1, GraphReferenceIdV2, ResolvedGraphReferenceV2};

mod support;

use support::{
    bounded_query, graph_document, reference, unbounded_query, GraphReferenceRoleV2,
    ReferenceGraphReferenceV2,
};

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

/// The exact `(role, selector, use-span)` triple of one resolved reference.
fn facts(
    reference: &ResolvedGraphReferenceV2,
) -> (GraphReferenceRoleV2, Option<&str>, (usize, usize)) {
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

// Mutation 1 `waypoint_count_as_depth`: a mutant that treats the waypoint count
// as a path-length bound would ADMIT this shape. It must reject.
#[test]
fn waypoints_never_bound_path_length() {
    let request = unbounded_query(
        GraphQueryKindV1::Reachability,
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
            reference(GraphReferenceRoleV2::Via, "x", 14, 15),
            reference(GraphReferenceRoleV2::Through, "z", 18, 19),
        ],
        0,
        30,
    );
    let document = graph_document(request).expect("fixture");
    let err = GraphQueryContractV2::from_resolved(document.graph())
        .expect_err("waypoint count is not a depth");
    assert_eq!(
        err.code(),
        "logic.unbounded_witness.missing_path_length_bound"
    );
}

// Mutation 2 `points_to_acyclic_exemption`: a mutant that restores the V1
// `points_to` acyclic shortcut would ADMIT this depth-free shape.
#[test]
fn points_to_requires_a_depth() {
    let request = unbounded_query(
        GraphQueryKindV1::PointsTo,
        references(GraphQueryKindV1::PointsTo),
        0,
        20,
    );
    let document = graph_document(request).expect("fixture");
    let err = GraphQueryContractV2::from_resolved(document.graph())
        .expect_err("points_to has no acyclic bypass");
    assert_eq!(
        err.code(),
        "logic.unbounded_witness.missing_path_length_bound"
    );
}

// Mutation 3 `depth_waypoint_conflict`: a mutant that restores the retired
// conflict policy would REJECT this shape instead of retaining both constraints.
#[test]
fn depth_and_waypoints_admit_with_both_retained() {
    let request = bounded_query(
        GraphQueryKindV1::Reachability,
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
            reference(GraphReferenceRoleV2::Via, "x", 14, 15),
        ],
        5,
        0,
        30,
    );
    let document = graph_document(request).expect("fixture");
    let contract = GraphQueryContractV2::from_resolved(document.graph())
        .expect("depth plus waypoints is valid");
    assert_eq!(contract.depth(), 5);
    assert_eq!(contract.waypoints().count(), 1);
}

// Mutation 4 `default_max_witnesses`: a mutant that returned an independent cap
// (`DEFAULT_MAX_WITNESSES = 64`) would disagree with the sealed receipt.
#[test]
fn witness_cap_is_the_sealed_receipt_value() {
    for kind in ALL_KINDS {
        let document =
            graph_document(bounded_query(kind, references(kind), 3, 0, 30)).expect("fixture");
        let sealed_cap = document.document().budget_receipt().max_witnesses();
        let contract =
            GraphQueryContractV2::from_resolved(document.graph()).expect("bounded FIND admits");
        assert_eq!(contract.max_witnesses(), sealed_cap, "{kind:?}");
        assert_ne!(
            contract.max_witnesses(),
            64,
            "{kind:?} must not carry the retired default cap"
        );
    }
}

// Mutation 5 `discard_tie_break`: a mutant that dropped or changed the canonical
// tie-break fails here for every subkind.
#[test]
fn tie_break_is_retained_for_every_subkind() {
    for kind in ALL_KINDS {
        let document =
            graph_document(bounded_query(kind, references(kind), 3, 0, 30)).expect("fixture");
        let contract =
            GraphQueryContractV2::from_resolved(document.graph()).expect("bounded FIND admits");
        assert_eq!(
            contract.tie_break(),
            WitnessTieBreakV1::ShortestThenLexical,
            "{kind:?}"
        );
    }
}

// Mutation 6 `erase_waypoint_order`: a mutant that sorted, deduped, or dropped
// resolved waypoints fails the exact ordered identity assertion.
#[test]
fn waypoint_order_and_identity_are_retained() {
    let request = bounded_query(
        GraphQueryKindV1::Taint,
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
            reference(GraphReferenceRoleV2::Via, "x", 14, 15),
            reference(GraphReferenceRoleV2::Via, "y", 18, 19),
            reference(GraphReferenceRoleV2::Through, "z", 22, 23),
        ],
        5,
        0,
        30,
    );
    let document = graph_document(request).expect("fixture");
    let contract = GraphQueryContractV2::from_resolved(document.graph()).expect("admits");
    let waypoints: Vec<(GraphReferenceRoleV2, Option<&str>, (usize, usize))> =
        contract.waypoints().map(facts).collect();
    assert_eq!(
        waypoints,
        vec![
            (GraphReferenceRoleV2::Via, Some("x"), (14, 15)),
            (GraphReferenceRoleV2::Via, Some("y"), (18, 19)),
            (GraphReferenceRoleV2::Through, Some("z"), (22, 23)),
        ]
    );
}

// Mutation 7 `any_endpoint_role`: a mutant that reported an arbitrary role or
// subkind fails the exact payload assertion.
#[test]
fn first_offending_role_is_reported() {
    let request = bounded_query(
        GraphQueryKindV1::Slice,
        vec![
            reference(GraphReferenceRoleV2::Anchor, "p.foo", 20, 25),
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
        ],
        5,
        0,
        40,
    );
    let document = graph_document(request).expect("fixture");
    let err = GraphQueryContractV2::from_resolved(document.graph()).expect_err("extraneous");
    match err {
        LogicDiagnosticV1::GraphEndpointUnexpected { kind, role, .. } => {
            assert_eq!(kind, GraphQueryKindV1::Slice);
            assert_eq!(role, GraphEndpointRoleV1::Source);
        }
        other => panic!("expected GraphEndpointUnexpected, got {other:?}"),
    }
}

// Mutation 8 `admit_missing_depth`: a mutant that treated a depth-free FIND as
// bounded would return Ok here.
#[test]
fn missing_depth_is_never_admitted() {
    for kind in ALL_KINDS {
        let document =
            graph_document(unbounded_query(kind, references(kind), 0, 30)).expect("fixture");
        let err = GraphQueryContractV2::from_resolved(document.graph())
            .expect_err("depth-free FIND is unbounded");
        assert_eq!(
            err.code(),
            "logic.unbounded_witness.missing_path_length_bound",
            "{kind:?}"
        );
    }
}

// Mutation 9 / punchlist finding 1 `missing_witness_contract`: every admitted
// subkind carries a genuinely bounded witness contract (finite, non-zero cap and
// an explicit depth), never an empty or absent one.
#[test]
fn every_admitted_subkind_carries_a_bound_witness_contract() {
    for kind in ALL_KINDS {
        let document =
            graph_document(bounded_query(kind, references(kind), 4, 0, 30)).expect("fixture");
        let contract = GraphQueryContractV2::from_resolved(document.graph())
            .unwrap_or_else(|error| panic!("{kind:?} must admit: {error:?}"));
        assert!(contract.max_witnesses() > 0, "{kind:?} cap");
        assert_eq!(
            contract.witness().max_path_length(),
            Some(4),
            "{kind:?} depth"
        );
        assert!(
            matches!(
                contract.dedup(),
                DedupIdentityV1::Endpoints
                    | DedupIdentityV1::NodeSequence
                    | DedupIdentityV1::NodeSet
            ),
            "{kind:?} dedup identity is one of the three contracts"
        );
    }
}
