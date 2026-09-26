#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "owner test builds a sealed semantic document and pins exact forged-contract rejections"
)]

use quanta_gqlang_v2_algebra::{lower_relation_item, AlgebraFailureKindV2, AlgebraFamilyV2};
use quanta_gqlang_v2_sema::test_support::{
    forged_capture_mutation_document_v2, forged_mixed_scalar_aggregate_projection_document_v2,
    forged_wrong_nested_root_document_v2, reference_relation_document_v2,
    ForgedCaptureEdgeMutationV2, ReferenceRelationDocumentV2, ReferenceRelationFixtureV2,
    REFERENCE_SOURCE_V2,
};
use quanta_gqlang_v2_sema::{
    ResolvedScanSourceV2, SemanticItemV2, TypedCorrelationEdgeV2, TypedExprShapeV2,
};

#[test]
fn scan_preserves_the_sema_output_contract_without_reconstruction() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).unwrap();
    let item_id = reference.item_id();

    let lowered = lower_relation_item(reference.document(), item_id).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Scan);
    assert_eq!(root.item_id(), item_id);
    assert_eq!(root.stage_key().source_id, REFERENCE_SOURCE_V2);
    assert_eq!(root.output().row().fields().len(), 1);
    assert_eq!(root.output().row().fields().first().unwrap().name(), "id");
    assert!(
        matches!(
            root.scan_source(),
            Some(ResolvedScanSourceV2::Relation(source)) if source.qualified_name() == "app::records"
        ),
        "the scan preserves the sealed host relation identity"
    );
    assert!(root.children().is_empty());
}

#[test]
fn mixed_scalar_aggregate_projection_is_an_impossible_contract() {
    let forged = forged_mixed_scalar_aggregate_projection_document_v2().unwrap();
    let Some(SemanticItemV2::Relation(relation)) = forged.document().item(forged.item_id()) else {
        panic!("forged mixed projection must hold a relation item");
    };

    let diagnostic = match lower_relation_item(forged.document(), forged.item_id()) {
        Ok(_) => panic!("a mixed scalar/aggregate projection must not lower"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::InvalidStageContract
    );
    assert_eq!(diagnostic.code(), "algebra.invalid_stage_contract");
    assert_eq!(
        diagnostic.message(),
        "projection stage cannot form a checked algebra node"
    );
    assert_eq!(
        diagnostic.location().map(|location| location.location()),
        Some(relation.root().location().location()),
        "the rejection is pinned at the forged projection stage's checked location"
    );
}

#[test]
fn wrong_nested_root_tree_with_swapped_inner_scan_rejects() {
    let forged = forged_wrong_nested_root_document_v2().unwrap();

    let diagnostic = match lower_relation_item(forged.document(), forged.item_id()) {
        Ok(_) => panic!("a nested tree with a swapped inner scan must not lower"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::InvalidStageContract
    );
    assert_eq!(diagnostic.code(), "algebra.invalid_stage_contract");
    assert_eq!(
        diagnostic.message(),
        "filter stage cannot form a checked algebra node"
    );
    assert!(
        diagnostic.location().is_some_and(|location| {
            location.location().source_id == REFERENCE_SOURCE_V2
                && location.location().span.start == 25
                && location.location().span.end == 49
        }),
        "the rejection is pinned at the nested filter's checked span (25, 49)"
    );
}

// ---- correlation capture-mutation rails -------------------------------------
//
// `forged_capture_mutation_document_v2` reshapes the sealed two-edge capture
// vector of a real two-capture correlated EXISTS (`f.id = r.id AND f.value =
// r.value` over records2). The finalizer assembles Dropped and Swapped (it
// checks membership, uniqueness, and registry identity — not authored count
// or order) and rejects Duplicate and Foreign outright; the rails below pin
// exactly what each mutation proves at the algebra boundary.

/// The SemiJoin root's capture edges, cloned for exact vector comparison.
fn lowered_capture_edges(reference: &ReferenceRelationDocumentV2) -> Vec<TypedCorrelationEdgeV2> {
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    assert_eq!(lowered.root().family(), AlgebraFamilyV2::SemiJoin);
    // The nested right input keeps its own ordinary Filter (the AND of the
    // two capture comparisons) over the inner scan.
    let root_children = lowered.root().children();
    let nested = root_children.get(1).unwrap();
    assert_eq!(nested.family(), AlgebraFamilyV2::Filter);
    assert_eq!(nested.filter_conjuncts().len(), 2);
    lowered
        .root()
        .correlations()
        .unwrap()
        .iter()
        .cloned()
        .collect()
}

#[test]
fn dropped_capture_edge_lowers_with_exactly_the_surviving_edge() {
    let dropped =
        forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Dropped).unwrap();

    // The assembled document lowers: the SemiJoin retains EXACTLY the
    // surviving single edge (a dropped capture cannot be re-derived).
    let edges = lowered_capture_edges(&dropped);
    assert_eq!(edges.len(), 1);
    let edge = edges.first().unwrap();
    assert_eq!(edge.binding().canonical_name(), "r");

    // The surviving edge is the pristine vector's FIRST authored edge: the
    // same edge the Swapped document retains in second position.
    let swapped =
        forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Swapped).unwrap();
    let swapped_edges = lowered_capture_edges(&swapped);
    assert_eq!(swapped_edges.len(), 2);
    assert_eq!(
        swapped_edges.get(1),
        edges.first(),
        "the dropped document keeps the first authored edge, which the swapped document retains second"
    );
    assert_ne!(
        swapped_edges.first(),
        edges.first(),
        "the swapped document leads with the OTHER authored edge"
    );
}

#[test]
fn swapped_capture_edges_lower_in_the_mutated_order() {
    let swapped =
        forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Swapped).unwrap();
    let edges = lowered_capture_edges(&swapped);

    // The SemiJoin retains the vector in its MUTATED order: the second
    // authored edge first, the first authored edge second.
    assert_eq!(edges.len(), 2);
    assert_ne!(
        edges.first().unwrap().reference(),
        edges.get(1).unwrap().reference()
    );
    for edge in &edges {
        assert_eq!(edge.binding().canonical_name(), "r");
        assert!(
            swapped.document().expression(edge.reference()).is_some(),
            "every retained capture reference is a fact of this document"
        );
    }

    // The doc-side RelationExists fact carries the same mutated vector, so
    // the algebra retains exactly the reshaped payload — map and lowered
    // node agree.
    let Some(SemanticItemV2::Relation(relation)) = swapped.document().item(swapped.item_id())
    else {
        panic!("capture-mutation fixture must hold a relation item");
    };
    let TypedExprShapeV2::RelationExists { correlations, .. } = swapped
        .document()
        .expression(relation.root().predicate().unwrap().expression().key())
        .unwrap()
        .shape()
    else {
        panic!("the outer predicate is the relational EXISTS fact");
    };
    assert_eq!(correlations.len(), 2);
    assert_eq!(
        correlations.first().unwrap().reference(),
        edges.first().unwrap().reference()
    );
    assert_eq!(
        correlations.get(1).unwrap().reference(),
        edges.get(1).unwrap().reference()
    );
}

#[test]
fn duplicate_capture_reference_cannot_become_a_document() {
    let duplicate =
        forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Duplicate).unwrap_err();
    assert!(
        duplicate.contains("duplicate correlation reference in one nested relation"),
        "the real finalizer rejects a duplicate capture reference: {duplicate}"
    );
}

#[test]
fn foreign_capture_reference_cannot_become_a_document() {
    let foreign =
        forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Foreign).unwrap_err();
    assert!(
        foreign.contains("correlation reference is foreign to this document's expressions"),
        "the real finalizer rejects a foreign capture reference: {foreign}"
    );
}
