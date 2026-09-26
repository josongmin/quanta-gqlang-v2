#![expect(
    clippy::expect_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Witness-contract RETENTION rails over the sealed resolved FIND (ticket 06,
//! lane W3-B; `D31`).
//!
//! Every admitted FIND must carry a genuinely bounded witness contract whose
//! facts survive the seal: the document's exact max-witness cap, the exact
//! resolved depth, the subkind's dedup identity, the canonical tie-break, and a
//! cyclic relation shape (no subkind keeps an acyclic bypass). None of these may
//! be defaulted, dropped, or invented downstream.
//!
//! The DEGENERATE and OVER-BUDGET rejects (`missing`/`zero` cap,
//! `missing`/`zero` path bound, cap or depth one over the sealed receipt) are
//! unreachable through the sealed transaction — Sema already refuses them — so
//! those rails are the crate-private owner-unit tests in
//! `src/witness.rs`/`src/graph.rs`, which pin the full
//! `logic.unbounded_witness.*` codes, messages, and located spans.

use quanta_gqlang_v2_logic::{
    DedupIdentityV1, GraphQueryContractV2, RelationShapeV1, WitnessTieBreakV1,
};
use quanta_gqlang_v2_sema::GraphQueryKindV1;

mod support;

use support::{
    bounded_query, graph_document, reference, GraphReferenceRoleV2, ReferenceGraphReferenceV2,
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

/// Seal one bounded FIND of `kind` at `depth` and return the contract.
fn contract(kind: GraphQueryKindV1, depth: u32) -> GraphQueryContractV2 {
    let document =
        graph_document(bounded_query(kind, references(kind), depth, 0, 30)).expect("fixture");
    GraphQueryContractV2::from_resolved(document.graph()).expect("bounded FIND admits")
}

// Every subkind retains the document's EXACT sealed max-witness cap. A contract
// that carried an independent default (the retired `DEFAULT_MAX_WITNESSES = 64`)
// or dropped the cap entirely fails here.
#[test]
fn every_subkind_retains_the_sealed_witness_cap() {
    let probe = graph_document(bounded_query(
        GraphQueryKindV1::Reachability,
        references(GraphQueryKindV1::Reachability),
        1,
        0,
        30,
    ))
    .expect("probe fixture");
    let sealed_cap = probe.document().budget_receipt().max_witnesses();
    assert!(sealed_cap > 0, "the sealed cap is a real finite bound");

    for kind in ALL_KINDS {
        let contract = contract(kind, 3);
        assert_eq!(contract.max_witnesses(), sealed_cap, "{kind:?}");
        assert_eq!(contract.witness().max_witnesses(), sealed_cap, "{kind:?}");
    }
}

// Every subkind retains the exact resolved depth as its path-length bound.
#[test]
fn every_subkind_retains_the_exact_resolved_depth() {
    for kind in ALL_KINDS {
        let contract = contract(kind, 9);
        assert_eq!(contract.depth(), 9, "{kind:?}");
        assert_eq!(contract.witness().max_path_length(), Some(9), "{kind:?}");
    }
}

// No FIND family keeps an acyclic bypass: every admitted contract is bound over
// a relation that can revisit nodes.
#[test]
fn every_subkind_binds_a_cyclic_relation_shape() {
    for kind in ALL_KINDS {
        let contract = contract(kind, 2);
        assert_eq!(contract.shape(), RelationShapeV1::Cyclic, "{kind:?}");
        assert_eq!(
            contract.witness().shape(),
            RelationShapeV1::Cyclic,
            "{kind:?}"
        );
    }
}

// Dedup identity is retained per subkind: route-distinguishing families keep the
// full node sequence, reachability collapses to its endpoint pair, and
// node-set families dedup by the nodes they touch.
#[test]
fn dedup_identity_is_retained_per_subkind() {
    let expected = [
        (GraphQueryKindV1::Reachability, DedupIdentityV1::Endpoints),
        (GraphQueryKindV1::Flow, DedupIdentityV1::NodeSequence),
        (GraphQueryKindV1::Slice, DedupIdentityV1::NodeSet),
        (GraphQueryKindV1::Taint, DedupIdentityV1::NodeSequence),
        (GraphQueryKindV1::Path, DedupIdentityV1::NodeSequence),
        (GraphQueryKindV1::PointsTo, DedupIdentityV1::NodeSet),
        (GraphQueryKindV1::Summary, DedupIdentityV1::NodeSet),
    ];
    assert_eq!(
        expected.len(),
        ALL_KINDS.len(),
        "the dedup table covers every subkind"
    );
    for (kind, dedup) in expected {
        let contract = contract(kind, 2);
        assert_eq!(contract.dedup(), dedup, "{kind:?}");
        assert_eq!(contract.witness().dedup(), dedup, "{kind:?}");
    }
}

// The canonical tie-break is part of every admitted contract; a contract that
// discarded it would not be reproducible under a cap.
#[test]
fn tie_break_is_retained_for_every_subkind() {
    for kind in ALL_KINDS {
        let contract = contract(kind, 2);
        assert_eq!(
            contract.tie_break(),
            WitnessTieBreakV1::ShortestThenLexical,
            "{kind:?}"
        );
        assert_eq!(
            contract.witness().tie_break(),
            WitnessTieBreakV1::ShortestThenLexical,
            "{kind:?}"
        );
    }
}

// The sealed depth is the exact-limit bound: a FIND authored at the document's
// max_witness_depth admits and keeps that exact value (the +1 over-budget rail
// is the crate-private unit test in `src/witness.rs`).
#[test]
fn exact_sealed_depth_admits_and_is_retained() {
    let probe = graph_document(bounded_query(
        GraphQueryKindV1::Reachability,
        references(GraphQueryKindV1::Reachability),
        1,
        0,
        30,
    ))
    .expect("probe fixture");
    let sealed_depth = probe.document().budget_receipt().max_witness_depth();

    for kind in ALL_KINDS {
        let contract = contract(kind, sealed_depth);
        assert_eq!(contract.depth(), sealed_depth, "{kind:?}");
        assert_eq!(
            contract.witness().max_path_length(),
            Some(sealed_depth),
            "{kind:?}"
        );
    }
}
