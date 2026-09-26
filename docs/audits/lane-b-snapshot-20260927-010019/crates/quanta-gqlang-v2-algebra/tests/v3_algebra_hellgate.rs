#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "owner test inspects exact lowerer diagnostics and sealed fixture rejections"
)]

use quanta_gqlang_v2_algebra::{
    lower_nested_relation_v2, lower_relation_item, AlgebraFailureKindV2, AlgebraFamilyV2,
};
use quanta_gqlang_v2_sema::test_support::{
    deep_bool_tree_document_v2, deep_relation_chain_document_v2,
    forged_aggregate_without_group_document_v2, forged_cross_join_with_on_seal_v2,
    forged_inner_join_without_on_seal_v2, forged_small_receipt_bool_tree_document_v2,
    forged_small_receipt_relation_chain_document_v2, reference_absent_item_id_v2,
    reference_composition_document_v2, reference_relation_document_v2,
    second_source_relation_document_v2, ReferenceBudgetV2, ReferenceCompositionBodyV2,
    ReferenceRelationFixtureV2, SECONDARY_REFERENCE_SOURCE_V2, SMALL_BUDGET_BOOL_TREE_EXACT_DEPTH,
    SMALL_BUDGET_BOOL_TREE_OVER_DEPTH, SMALL_BUDGET_MAX_EXPRESSION_NODES,
    SMALL_BUDGET_MAX_SEMANTIC_STEPS, SMALL_BUDGET_RELATION_CHAIN_EXACT_DEPTH,
    SMALL_BUDGET_RELATION_CHAIN_OVER_DEPTH,
};
use quanta_gqlang_v2_sema::{
    ResolvedCompositionBodyV2, ResolvedScanSourceV2, SemanticItemV2, TypedExprShapeV2,
};

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
fn nested_relation_rejects_a_same_source_non_owner_identity() {
    let reference =
        reference_composition_document_v2(ReferenceCompositionBodyV2::Relational).unwrap();
    let ResolvedCompositionBodyV2::Query(body) = reference.composition().body() else {
        panic!("the relational composition fixture must retain a query body");
    };

    let diagnostic =
        match lower_nested_relation_v2(reference.document(), body.item_id(), body.root()) {
            Ok(_) => panic!("a nested body identity is not the composition item that owns it"),
            Err(diagnostic) => diagnostic,
        };

    assert_eq!(diagnostic.kind(), AlgebraFailureKindV2::ForeignSemanticFact);
    assert_eq!(diagnostic.code(), "algebra.foreign_semantic_fact");
    assert_eq!(
        diagnostic.message(),
        "nested relation root is not owned by the requested semantic item"
    );
    assert_eq!(
        diagnostic.location().map(|location| location.location()),
        Some(body.root().location().location())
    );

    let transplanted = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).unwrap();
    let Some(SemanticItemV2::Relation(transplanted_relation)) =
        transplanted.document().item(transplanted.item_id())
    else {
        panic!("the scan fixture must retain a relation item");
    };
    let diagnostic = match lower_nested_relation_v2(
        reference.document(),
        reference.composition_item_id(),
        transplanted_relation.root(),
    ) {
        Ok(_) => panic!("another same-source document's root is not owned by this composition"),
        Err(diagnostic) => diagnostic,
    };
    assert_eq!(diagnostic.kind(), AlgebraFailureKindV2::ForeignSemanticFact);
    assert_eq!(diagnostic.code(), "algebra.foreign_semantic_fact");
    assert_eq!(
        diagnostic.message(),
        "nested relation root is not owned by the requested semantic item"
    );
    assert_eq!(
        diagnostic.location().map(|location| location.location()),
        Some(transplanted_relation.root().location().location())
    );
}

#[test]
fn nested_relation_accepts_only_the_exact_sealed_owner_paths() {
    let composition =
        reference_composition_document_v2(ReferenceCompositionBodyV2::Relational).unwrap();
    let ResolvedCompositionBodyV2::Query(body) = composition.composition().body() else {
        panic!("the relational composition fixture must retain a query body");
    };
    let lowered_body = lower_nested_relation_v2(
        composition.document(),
        composition.composition_item_id(),
        body.root(),
    )
    .unwrap();
    assert_eq!(lowered_body.item_id(), composition.composition_item_id());
    assert_eq!(lowered_body.stage_key(), body.root().key());

    let relation = reference_relation_document_v2(ReferenceRelationFixtureV2::Exists).unwrap();
    let Some(SemanticItemV2::Relation(outer)) = relation.document().item(relation.item_id()) else {
        panic!("the EXISTS fixture must retain a relation item");
    };
    let predicate = outer.root().predicate().unwrap().expression();
    let TypedExprShapeV2::RelationExists { relation_root, .. } = predicate.shape() else {
        panic!("the EXISTS fixture predicate must retain a nested relation identity");
    };
    let nested = relation.document().nested_relation(*relation_root).unwrap();
    let lowered_nested =
        lower_nested_relation_v2(relation.document(), relation.item_id(), nested).unwrap();
    assert_eq!(lowered_nested.item_id(), relation.item_id());
    assert_eq!(lowered_nested.stage_key(), nested.key());

    let nested_two_levels =
        reference_relation_document_v2(ReferenceRelationFixtureV2::NestedExistsTwoLevels).unwrap();
    let owned = nested_two_levels
        .document()
        .nested_relations()
        .map(|(_, relation)| relation)
        .collect::<Vec<_>>();
    assert_eq!(owned.len(), 2, "the fixture seals outer and inner roots");
    for root in owned {
        let lowered = lower_nested_relation_v2(
            nested_two_levels.document(),
            nested_two_levels.item_id(),
            root,
        )
        .unwrap();
        assert_eq!(lowered.item_id(), nested_two_levels.item_id());
        assert_eq!(lowered.stage_key(), root.key());
    }
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
fn expression_walk_small_receipt_admits_below_limit_and_rejects_next_tree() {
    // Each authored leaf is a BoolValue wrapping one Bool literal: the
    // decomposition walk charges 2 * d + 1 nodes and the per-leaf
    // descendant search charges two more per leaf, 4 * d + 3 in total.
    // Depth 1 charges seven of the eight small-receipt nodes. The next
    // representable tree has depth 2, charges eleven, and is rejected; this
    // fixture therefore does not claim an exact-limit or limit-plus-one case.
    let exact = forged_small_receipt_bool_tree_document_v2(1).unwrap();
    let lowered = lower_relation_item(exact.document(), exact.item_id()).unwrap();
    assert_eq!(lowered.root().family(), AlgebraFamilyV2::Filter);
    let conjuncts = lowered.root().filter_conjuncts();
    assert_eq!(conjuncts.len(), 2);
    let Some(SemanticItemV2::Relation(relation)) = exact.document().item(exact.item_id()) else {
        panic!("forged Bool tree is a relation item");
    };
    let predicate = relation.root().predicate().unwrap();
    let TypedExprShapeV2::And { lhs, rhs, .. } = predicate.expression().shape() else {
        panic!("depth-one Bool tree is one authored AND");
    };
    assert_eq!(
        conjuncts
            .iter()
            .map(|conjunct| conjunct.expression().key())
            .collect::<Vec<_>>(),
        [*lhs, *rhs],
        "ordinary conjuncts retain exact authored left-to-right order"
    );

    let over = forged_small_receipt_bool_tree_document_v2(2).unwrap();
    let diagnostic = match lower_relation_item(over.document(), over.item_id()) {
        Ok(_) => panic!("the next representable tree over the receipt must not lower"),
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
