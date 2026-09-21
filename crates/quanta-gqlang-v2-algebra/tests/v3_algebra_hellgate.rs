#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "owner test inspects exact lowerer diagnostics and sealed fixture rejections"
)]

use quanta_gqlang_v2_algebra::{lower_relation_item, AlgebraFailureKindV2, AlgebraFamilyV2};
use quanta_gqlang_v2_sema::test_support::{
    deep_bool_tree_document_v2, deep_relation_chain_document_v2,
    forged_aggregate_without_group_document_v2, forged_cross_join_with_on_seal_v2,
    forged_inner_join_without_on_seal_v2, forged_small_receipt_bool_tree_document_v2,
    forged_small_receipt_relation_chain_document_v2, reference_absent_item_id_v2,
    reference_relation_document_v2, second_source_relation_document_v2, ReferenceBudgetV2,
    ReferenceRelationFixtureV2, SECONDARY_REFERENCE_SOURCE_V2, SMALL_BUDGET_BOOL_TREE_EXACT_DEPTH,
    SMALL_BUDGET_BOOL_TREE_OVER_DEPTH, SMALL_BUDGET_MAX_EXPRESSION_NODES,
    SMALL_BUDGET_MAX_SEMANTIC_STEPS, SMALL_BUDGET_RELATION_CHAIN_EXACT_DEPTH,
    SMALL_BUDGET_RELATION_CHAIN_OVER_DEPTH,
};
use quanta_gqlang_v2_sema::{ResolvedScanSourceV2, SemanticItemV2};

#[test]
fn absent_item_cannot_enter_the_algebra_boundary() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).unwrap();

    let diagnostic = match lower_relation_item(reference.document(), reference_absent_item_id_v2())
    {
        Ok(_) => panic!("absent items are not algebra relation items"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(diagnostic.kind(), AlgebraFailureKindV2::NonRelationItem);
    assert_eq!(diagnostic.code(), "algebra.non_relation_item");
    assert_eq!(
        diagnostic.message(),
        "requested semantic item is absent from the document"
    );
}

#[test]
fn inner_join_without_on_cannot_be_sealed() {
    assert_eq!(
        forged_inner_join_without_on_seal_v2().err(),
        Some("invalid typed relation stage arity or payload"),
        "an Inner join missing its ON predicate can never become a sealed stage"
    );
}

#[test]
fn cross_join_with_on_cannot_be_sealed() {
    assert_eq!(
        forged_cross_join_with_on_seal_v2().err(),
        Some("invalid typed relation stage arity or payload"),
        "a Cross join carrying an ON predicate can never become a sealed stage"
    );
}

#[test]
fn aggregate_without_group_rejects_with_the_exact_diagnostic() {
    let forged = forged_aggregate_without_group_document_v2().unwrap();
    let Some(SemanticItemV2::Relation(relation)) = forged.document().item(forged.item_id()) else {
        panic!("forged aggregate-without-group must hold a relation item");
    };

    let diagnostic = match lower_relation_item(forged.document(), forged.item_id()) {
        Ok(_) => panic!("an aggregate projection over a non-Group input must not lower"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::AggregateWithoutGroup
    );
    assert_eq!(diagnostic.code(), "algebra.aggregate_without_group");
    assert_eq!(
        diagnostic.message(),
        "aggregate projection must directly follow a checked Group stage"
    );
    assert_eq!(
        diagnostic.location().map(|location| location.location()),
        Some(relation.root().location().location()),
        "the rejection is pinned at the aggregate projection stage's checked location"
    );
}

#[test]
fn second_source_item_lowers_and_cross_source_lookup_stays_absent() {
    // The boundary is host-agnostic: an item sealed under a second source
    // identity lowers through exactly the same public cut.
    let secondary = second_source_relation_document_v2().unwrap();
    let lowered = lower_relation_item(secondary.document(), secondary.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Scan);
    assert_eq!(root.stage_key().source_id, SECONDARY_REFERENCE_SOURCE_V2);
    assert!(matches!(
        root.scan_source(),
        Some(ResolvedScanSourceV2::Relation(source)) if source.qualified_name() == "app::records"
    ));

    // The same boundary stays closed to another document's item identity:
    // the primary document holds no such item, so the lookup is absent.
    let primary = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).unwrap();
    assert!(primary.document().item(secondary.item_id()).is_none());
    let diagnostic = match lower_relation_item(primary.document(), secondary.item_id()) {
        Ok(_) => panic!("an item of another source document must not lower"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(diagnostic.kind(), AlgebraFailureKindV2::NonRelationItem);
    assert_eq!(diagnostic.code(), "algebra.non_relation_item");
    assert_eq!(
        diagnostic.message(),
        "requested semantic item is absent from the document"
    );
}

#[test]
fn relation_chain_small_budget_resolves_exact_and_rejects_one_over() {
    // Sema-side exact fit: depth SMALL_BUDGET_RELATION_CHAIN_EXACT_DEPTH
    // consumes 2 * depth + 3 of the small receipt's semantic steps.
    let exact = deep_relation_chain_document_v2(
        SMALL_BUDGET_RELATION_CHAIN_EXACT_DEPTH,
        ReferenceBudgetV2::Small,
    )
    .unwrap();
    assert_eq!(
        exact.document().budget_receipt().max_semantic_steps(),
        SMALL_BUDGET_MAX_SEMANTIC_STEPS
    );

    let over = deep_relation_chain_document_v2(
        SMALL_BUDGET_RELATION_CHAIN_OVER_DEPTH,
        ReferenceBudgetV2::Small,
    );
    assert!(over
        .err()
        .is_some_and(|error| error.contains("semantic_steps exceeds frontend limit")));
}

#[test]
fn relation_walk_small_receipt_is_exact_and_rejects_one_stage_over() {
    // The forged document carries the small receipt while its tree was
    // resolved under the standard budget, so the algebra relation walk's
    // own charging (one step per stage visit) is the only limit. A chain
    // of depth d lowers d + 1 stages.
    let exact_depth = SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;
    let exact = forged_small_receipt_relation_chain_document_v2(exact_depth).unwrap();
    let lowered = lower_relation_item(exact.document(), exact.item_id()).unwrap();
    assert_eq!(lowered.root().family(), AlgebraFamilyV2::Filter);

    let over = forged_small_receipt_relation_chain_document_v2(exact_depth + 1).unwrap();
    let diagnostic = match lower_relation_item(over.document(), over.item_id()) {
        Ok(_) => panic!("one relation stage over the receipt must not lower"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::TraversalBudgetExceeded
    );
    assert_eq!(diagnostic.code(), "algebra.traversal_budget_exceeded");
    assert_eq!(
        diagnostic.message(),
        "relation lowering exceeded the document's max_semantic_steps budget"
    );
    // The refused visit is the chain's deepest stage: the scan.
    assert!(
        diagnostic.location().is_some_and(|location| {
            location.location().span.start == 5 && location.location().span.end == 12
        }),
        "the over-budget rejection names the refused stage's checked location"
    );
}

#[test]
fn bool_tree_small_budget_resolves_exact_and_rejects_one_over() {
    // Sema-side exact fit: depth SMALL_BUDGET_BOOL_TREE_EXACT_DEPTH
    // consumes 3 * depth + 6 semantic steps, inside the small receipt.
    let exact =
        deep_bool_tree_document_v2(SMALL_BUDGET_BOOL_TREE_EXACT_DEPTH, ReferenceBudgetV2::Small)
            .unwrap();
    assert_eq!(
        exact.document().budget_receipt().max_expression_nodes(),
        SMALL_BUDGET_MAX_EXPRESSION_NODES
    );

    let over =
        deep_bool_tree_document_v2(SMALL_BUDGET_BOOL_TREE_OVER_DEPTH, ReferenceBudgetV2::Small);
    assert!(over
        .err()
        .is_some_and(|error| error.contains("semantic_steps exceeds frontend limit")));
}

#[test]
fn expression_walk_small_receipt_is_exact_and_rejects_one_node_over() {
    // Each authored leaf is a BoolValue wrapping one Bool literal: the
    // decomposition walk charges 2 * d + 1 nodes and the per-leaf
    // descendant search charges two more per leaf, 4 * d + 3 in total.
    // Depth 1 charges seven of the eight small-receipt nodes; depth 2
    // charges eleven and rejects one over.
    let exact = forged_small_receipt_bool_tree_document_v2(1).unwrap();
    let lowered = lower_relation_item(exact.document(), exact.item_id()).unwrap();
    assert_eq!(lowered.root().family(), AlgebraFamilyV2::Filter);
    assert_eq!(lowered.root().filter_conjuncts().len(), 2);

    let over = forged_small_receipt_bool_tree_document_v2(2).unwrap();
    let diagnostic = match lower_relation_item(over.document(), over.item_id()) {
        Ok(_) => panic!("one expression node over the receipt must not lower"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::TraversalBudgetExceeded
    );
    assert_eq!(diagnostic.code(), "algebra.traversal_budget_exceeded");
    assert_eq!(
        diagnostic.message(),
        "predicate traversal exceeded the document's max_expression_nodes budget"
    );
    assert!(
        diagnostic.location().is_some(),
        "the over-budget rejection names the refused expression's checked location"
    );
}
