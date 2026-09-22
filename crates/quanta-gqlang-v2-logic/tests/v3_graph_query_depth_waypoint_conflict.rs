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

//! POLICY FLIP rails: `depth` plus `via`/`through` waypoints is now VALID
//! (ticket 06, lane W3-B; supersedes the V1 `GraphConflictingPathBound` policy).
//!
//! V1 treated an explicit `depth` and bounding waypoints as two COMPETING
//! binding strategies and rejected the combination. This packet separates the
//! two roles instead: `depth` is the only termination bound, while `via` and
//! `through` are ordered PATH PREDICATES that restrict which paths count as
//! witnesses. Accepting both is therefore the lossless behavior — neither
//! constraint is dropped, so there is no silent downgrade (`D10`) and no seed
//! that means two different things (`D26`).
//!
//! `logic.graph_conflicting_path_bound` and its variant no longer exist; the
//! rails below assert the POSITIVE contract and the exact retained order of the
//! waypoint constraints.

use quanta_gqlang_v2_logic::{DedupIdentityV1, GraphQueryContractV2};
use quanta_gqlang_v2_sema::{GraphQueryKindV1, GraphReferenceIdV2, ResolvedGraphReferenceV2};

mod support;

use support::{bounded_query, graph_document, reference, GraphReferenceRoleV2};

/// The exact `(role, selector, use-span)` triple of one waypoint.
fn waypoint_facts(
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

// `find taint a -> b | depth=5 | via=x,y | through=z`: depth AND waypoints are
// both retained. The order of the waypoint constraints is part of the contract.
#[test]
fn taint_depth_and_waypoints_admit_with_ordered_retention() {
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
    let document = graph_document(request).expect("taint fixture resolves");

    let contract = GraphQueryContractV2::from_resolved(document.graph())
        .expect("depth plus waypoints must be VALID");
    assert_eq!(contract.kind(), GraphQueryKindV1::Taint);
    // The length bound is the explicit depth, never a waypoint count.
    assert_eq!(contract.depth(), 5);
    assert_eq!(contract.dedup(), DedupIdentityV1::NodeSequence);
    assert_eq!(
        contract.max_witnesses(),
        document.document().budget_receipt().max_witnesses()
    );

    let waypoints: Vec<(GraphReferenceRoleV2, Option<&str>, (usize, usize))> =
        contract.waypoints().map(waypoint_facts).collect();
    assert_eq!(
        waypoints,
        vec![
            (GraphReferenceRoleV2::Via, Some("x"), (14, 15)),
            (GraphReferenceRoleV2::Via, Some("y"), (18, 19)),
            (GraphReferenceRoleV2::Through, Some("z"), (22, 23)),
        ]
    );
    // Both endpoint anchors survive alongside the waypoints.
    assert_eq!(contract.endpoints().count(), 2);
}

// The same flip holds when the waypoint set is `through`-only: the depth is the
// bound and the ordered `through` constraint is retained.
#[test]
fn reachability_depth_and_through_waypoints_admit() {
    let request = bounded_query(
        GraphQueryKindV1::Reachability,
        vec![
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
            reference(GraphReferenceRoleV2::Through, "mid", 14, 17),
            reference(GraphReferenceRoleV2::Through, "late", 20, 24),
        ],
        7,
        0,
        30,
    );
    let document = graph_document(request).expect("reachability fixture resolves");

    let contract = GraphQueryContractV2::from_resolved(document.graph())
        .expect("depth plus through waypoints must be VALID");
    assert_eq!(contract.kind(), GraphQueryKindV1::Reachability);
    assert_eq!(contract.depth(), 7);
    assert_eq!(contract.dedup(), DedupIdentityV1::Endpoints);

    let waypoints: Vec<(GraphReferenceRoleV2, Option<&str>, (usize, usize))> =
        contract.waypoints().map(waypoint_facts).collect();
    assert_eq!(
        waypoints,
        vec![
            (GraphReferenceRoleV2::Through, Some("mid"), (14, 17)),
            (GraphReferenceRoleV2::Through, Some("late"), (20, 24)),
        ]
    );
}
