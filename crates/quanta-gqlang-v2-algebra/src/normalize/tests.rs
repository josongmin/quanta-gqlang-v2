#![expect(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "owner tests build sealed semantic documents and assert exact trees at fixed positions"
)]

use super::*;
use crate::node::AlgebraFamilyV2;
use quanta_gqlang_v2_sema::test_support::{
    forged_mixed_scalar_aggregate_projection_document_v2,
    forged_small_receipt_bool_tree_document_v2, forged_small_receipt_relation_chain_document_v2,
    forged_wrong_nested_root_document_v2, reference_relation_document_v2,
    ReferenceRelationFixtureV2, SMALL_BUDGET_MAX_EXPRESSION_NODES, SMALL_BUDGET_MAX_SEMANTIC_STEPS,
};
use quanta_gqlang_v2_sema::{
    DownstreamBudgetReceiptV2, FrontendBudgetV2, NullOrderV2, OutputColumnOriginV2,
    SortDirectionV2, TypedCallArgumentV2,
};

#[test]
fn nested_ownership_discovery_is_charged_once_per_session() {
    use quanta_gqlang_v2_sema::test_support::{
        reference_composition_document_v2, ReferenceCompositionBodyV2,
    };
    let fixture =
        reference_composition_document_v2(ReferenceCompositionBodyV2::Relational).unwrap();
    let ResolvedCompositionBodyV2::Query(body) = fixture.composition().body() else {
        panic!("query body");
    };
    let mut session = AlgebraSessionV2::new(fixture.document());
    // The fixture owns one Project(Scan) with Path(Binding): discovery
    // charges exactly 2 relation and 2 expression nodes. Each lowering
    // separately charges the 2 relation nodes, without rediscovery.
    session.budget.semantic_steps = 6;
    session.budget.expression_nodes = 2;
    session
        .lower_nested_relation(fixture.composition_item_id(), body.root())
        .unwrap();
    assert_eq!(session.budget.semantic_steps, 2);
    assert_eq!(session.budget.expression_nodes, 0);
    session
        .lower_nested_relation(fixture.composition_item_id(), body.root())
        .unwrap();
    assert_eq!(session.budget.semantic_steps, 0);
    assert_eq!(session.budget.expression_nodes, 0);
    let error = session
        .lower_nested_relation(fixture.composition_item_id(), body.root())
        .unwrap_err();
    assert_eq!(error.kind(), AlgebraFailureKindV2::TraversalBudgetExceeded);
    assert_eq!(
        error.message(),
        "relation lowering exceeded the document's max_semantic_steps budget"
    );
}

#[test]
fn ownership_index_scales_with_distinct_producer_nodes() {
    use quanta_gqlang_v2_sema::test_support::reference_query_producer_composition_v2;
    for producer_count in [1, 4, 16] {
        let fixture = reference_query_producer_composition_v2(producer_count).unwrap();
        let composition = fixture.composition();
        let mut roots = composition
            .bindings()
            .iter()
            .filter_map(|binding| match binding.producer() {
                ResolvedCompositionProducerV2::Query(query) => Some(query.root()),
                ResolvedCompositionProducerV2::Sample { .. }
                | ResolvedCompositionProducerV2::Graph { .. } => None,
            })
            .collect::<Vec<_>>();
        let ResolvedCompositionBodyV2::Query(body) = composition.body() else {
            panic!("query body");
        };
        roots.push(body.root());
        assert_eq!(roots.len(), producer_count + 1);
        let visits = u32::try_from(producer_count + 1).unwrap();
        let mut session = AlgebraSessionV2::new(fixture.document());
        session.budget.semantic_steps = 2 * visits;
        session.budget.expression_nodes = 0;
        for root in roots {
            assert_eq!(root.inputs().count(), 0, "one Scan per root");
            session
                .lower_nested_relation(fixture.composition_item_id(), root)
                .unwrap();
        }
        assert_eq!(session.budget.semantic_steps, 0);
        assert_eq!(session.budget.expression_nodes, 0);
        assert_eq!(session.owned_relations.len(), 1);
        assert_eq!(
            session
                .owned_relations
                .get(&fixture.composition_item_id())
                .unwrap()
                .len(),
            producer_count + 1
        );
    }
}

#[test]
fn ownership_index_budget_failures_never_publish_partial_authority() {
    use quanta_gqlang_v2_sema::test_support::{
        reference_composition_document_v2, ReferenceCompositionBodyV2,
    };
    let fixture =
        reference_composition_document_v2(ReferenceCompositionBodyV2::Relational).unwrap();
    let ResolvedCompositionBodyV2::Query(body) = fixture.composition().body() else {
        panic!("query body");
    };
    let mut absent = AlgebraSessionV2::new(fixture.document());
    for owner in fixture
        .producer_item_ids()
        .iter()
        .copied()
        .chain(std::iter::once(fixture.body_item_id()))
    {
        assert!(fixture.document().item(owner).is_none());
        let error = absent
            .lower_nested_relation(owner, body.root())
            .unwrap_err();
        assert_eq!(error.kind(), AlgebraFailureKindV2::ForeignSemanticFact);
        assert!(absent.owned_relations.is_empty());
    }
    for (steps, expressions, message, span) in [
        (
            1,
            2,
            "ownership discovery exceeded the document's max_semantic_steps budget",
            (125, 134),
        ),
        (
            4,
            1,
            "ownership discovery exceeded the document's max_expression_nodes budget",
            (152, 153),
        ),
    ] {
        let mut session = AlgebraSessionV2::new(fixture.document());
        session.budget.semantic_steps = steps;
        session.budget.expression_nodes = expressions;
        let error = session
            .lower_nested_relation(fixture.composition_item_id(), body.root())
            .unwrap_err();
        assert_eq!(error.code(), "algebra.traversal_budget_exceeded");
        assert_eq!(error.message(), message);
        let location = error.location().unwrap().location();
        assert_eq!((location.span.start, location.span.end), span);
        assert_eq!(
            location.source_id,
            fixture.composition_item_id().source_id()
        );
        assert!(session.owned_relations.is_empty());
    }
}

#[test]
fn cached_ownership_preserves_exact_tree_and_transitive_root_checks() {
    use quanta_gqlang_v2_sema::test_support::{
        reference_composition_document_v2, ReferenceCompositionBodyV2,
    };
    let good =
        reference_composition_document_v2(ReferenceCompositionBodyV2::RelationalFilter).unwrap();
    let changed =
        reference_composition_document_v2(ReferenceCompositionBodyV2::RelationalFilterFalse)
            .unwrap();
    let ResolvedCompositionBodyV2::Query(body) = good.composition().body() else {
        panic!("query body");
    };
    let ResolvedCompositionBodyV2::Query(other) = changed.composition().body() else {
        panic!("query body");
    };
    assert_eq!(body.root().key(), other.root().key());
    assert_eq!(body.root().output(), other.root().output());
    let mut session = AlgebraSessionV2::new(good.document());
    session
        .lower_nested_relation(good.composition_item_id(), body.root())
        .unwrap();
    for invalid in [other.root(), body.root().inputs().next().unwrap()] {
        let error = session
            .lower_nested_relation(good.composition_item_id(), invalid)
            .unwrap_err();
        assert_eq!(error.kind(), AlgebraFailureKindV2::ForeignSemanticFact);
        assert_eq!(
            error.message(),
            "nested relation root is not owned by the requested semantic item"
        );
    }
    let nested =
        reference_relation_document_v2(ReferenceRelationFixtureV2::NestedExistsTwoLevels).unwrap();
    let mut session = AlgebraSessionV2::new(nested.document());
    let roots = nested
        .document()
        .nested_relations()
        .map(|(_, root)| root)
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 2);
    for root in roots {
        session
            .lower_nested_relation(nested.item_id(), root)
            .unwrap();
    }
    assert_eq!(session.owned_relations.len(), 1);
    assert_eq!(
        session
            .owned_relations
            .get(&nested.item_id())
            .unwrap()
            .len(),
        2
    );
}

/// Smallest legal budget: every dimension 1 except the two under test,
/// with frame bytes covering source bytes. The public entry always reads
/// the document receipt; this helper reaches the private frame machinery
/// only to prove exact-limit and one-over behavior.
fn tiny_receipt(max_semantic_steps: u32, max_expression_nodes: u32) -> DownstreamBudgetReceiptV2 {
    FrontendBudgetV2::try_new(
        1,
        2,
        1,
        1,
        1,
        1,
        max_expression_nodes,
        1,
        1,
        max_semantic_steps,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        1,
        2,
    )
    .unwrap()
    .downstream_receipt()
}

fn families_in_authoring_order(root: &CheckedAlgebraNodeV2) -> Vec<AlgebraFamilyV2> {
    let mut stack = vec![root];
    let mut families = Vec::new();
    while let Some(node) = stack.pop() {
        families.push(node.family());
        let mut children = node.children();
        stack.extend(children.drain(..).rev());
    }
    families
}

#[test]
fn pure_exists_predicate_leaves_no_residual_filter_node() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Exists).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::SemiJoin);
    assert_eq!(
        families_in_authoring_order(root),
        vec![
            AlgebraFamilyV2::SemiJoin,
            AlgebraFamilyV2::Scan,
            AlgebraFamilyV2::Scan
        ],
        "a pure relational EXISTS predicate must not retain a Filter node"
    );
    assert!(root.filter_conjuncts().is_empty());
    assert!(root.correlations().is_some_and(|set| set.is_empty()));
    assert_eq!(root.output(), lowered.output());
    assert_eq!(root.children()[1].family(), AlgebraFamilyV2::Scan);
}

#[test]
fn pure_not_exists_predicate_leaves_no_residual_filter_node() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::NotExists).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::AntiJoin);
    assert_eq!(
        families_in_authoring_order(root),
        vec![
            AlgebraFamilyV2::AntiJoin,
            AlgebraFamilyV2::Scan,
            AlgebraFamilyV2::Scan
        ],
        "a pure relational NOT EXISTS predicate must not retain a Filter node"
    );
    assert!(root.filter_conjuncts().is_empty());
    assert!(root.correlations().is_some_and(|set| set.is_empty()));
}

#[test]
fn ordinary_filter_keeps_one_filter_with_the_sealed_bool_fact() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Filter).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Filter);
    let conjuncts = root.filter_conjuncts();
    assert_eq!(
        conjuncts.len(),
        1,
        "an ordinary predicate is retained as one full conjunction"
    );
    // The fixture authors `bool_value(bool_lit(true))`: the retained
    // conjunct is the BoolValue fact, and its forwarded child is the
    // document-owned Bool literal.
    let TypedExprShapeV2::BoolValue { value } = conjuncts[0].expression().shape() else {
        panic!("the retained conjunct must be the sealed typed Bool fact");
    };
    let child = reference
        .document()
        .expression(*value)
        .expect("the BoolValue child is a document expression");
    assert!(matches!(child.shape(), TypedExprShapeV2::Literal(_)));
    let child = &root.children()[0];
    assert_eq!(child.family(), AlgebraFamilyV2::Scan);
    assert_eq!(root.output().row(), child.output().row());
    assert_eq!(root.output().columns(), child.output().columns());
    assert_eq!(root.output().cardinality(), child.output().cardinality());
    assert_eq!(
        root.output().partition_keys(),
        child.output().partition_keys()
    );
    assert_eq!(root.output().ordering(), child.output().ordering());
}

#[test]
fn relation_walk_exact_budget_succeeds_and_one_over_rejects() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Filter).unwrap();
    let document = reference.document();
    let relation = match document.item(reference.item_id()) {
        Some(quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation)) => relation,
        other => panic!("filter fixture must hold a relation item: {other:?}"),
    };
    // The fixture lowers exactly two relation stages: Filter and Scan.
    let exact = lower_root(
        document,
        reference.item_id(),
        relation.root(),
        tiny_receipt(2, 8),
    );
    assert!(exact.is_ok(), "exact max_semantic_steps must succeed");

    let over = lower_root(
        document,
        reference.item_id(),
        relation.root(),
        tiny_receipt(1, 8),
    )
    .unwrap_err();
    assert_eq!(over.kind(), AlgebraFailureKindV2::TraversalBudgetExceeded);
    assert_eq!(over.code(), "algebra.traversal_budget_exceeded");
    assert!(
        over.location().is_some(),
        "the over-budget rejection names a checked stage location"
    );
}

#[test]
fn document_lowering_session_does_not_reset_between_items() {
    let reference =
        forged_small_receipt_relation_chain_document_v2(SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1)
            .expect("exact small-receipt chain");
    let mut session = AlgebraSessionV2::new(reference.document());
    session
        .lower_relation_item(reference.item_id())
        .expect("first traversal consumes the exact document allowance");
    let error = session
        .lower_relation_item(reference.item_id())
        .expect_err("a second traversal cannot reuse the consumed allowance");
    assert_eq!(error.kind(), AlgebraFailureKindV2::TraversalBudgetExceeded);
}

#[test]
fn expression_walk_exact_budget_succeeds_and_one_over_rejects() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Filter).unwrap();
    let document = reference.document();
    let relation = match document.item(reference.item_id()) {
        Some(quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation)) => relation,
        other => panic!("filter fixture must hold a relation item: {other:?}"),
    };
    // The predicate is one BoolValue leaf forwarding one Bool literal:
    // the decomposition walk visits the leaf, then the descendant search
    // visits the leaf and its literal child — three charged nodes.
    let exact = lower_root(
        document,
        reference.item_id(),
        relation.root(),
        tiny_receipt(4, 3),
    );
    assert!(exact.is_ok(), "exact max_expression_nodes must succeed");

    let over = lower_root(
        document,
        reference.item_id(),
        relation.root(),
        tiny_receipt(4, 2),
    )
    .unwrap_err();
    assert_eq!(over.kind(), AlgebraFailureKindV2::TraversalBudgetExceeded);
    assert_eq!(over.code(), "algebra.traversal_budget_exceeded");
    assert!(over.location().is_some());
}

#[test]
fn public_entry_reads_only_the_document_budget_receipt() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::Exists).unwrap();
    let document = reference.document();
    // Three relation stages (filter, input scan, nested scan) and one
    // EXISTS leaf sit far below the standard receipt's caps; the public
    // entry accepts no budget argument of its own.
    assert!(document.budget_receipt().max_semantic_steps() >= 3);
    let lowered = lower_relation_item(document, reference.item_id()).unwrap();
    assert_eq!(lowered.root().family(), AlgebraFamilyV2::SemiJoin);
}

fn scan_qualified_name(node: &CheckedAlgebraNodeV2) -> &str {
    match node.scan_source().expect("node must be a scan") {
        quanta_gqlang_v2_sema::ResolvedScanSourceV2::Relation(id) => id.qualified_name(),
        quanta_gqlang_v2_sema::ResolvedScanSourceV2::View(_)
        | quanta_gqlang_v2_sema::ResolvedScanSourceV2::Composition(_) => {
            panic!("reference fixtures scan host relations only")
        }
    }
}

#[test]
fn grouped_aggregate_preserves_the_sealed_partition_identity() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::GroupedAggregateSum).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Aggregate);
    let group = &root.children()[0];
    assert_eq!(group.family(), AlgebraFamilyV2::Group);
    assert_eq!(group.children()[0].family(), AlgebraFamilyV2::Scan);
    // Group: exactly one authored key, key-only row, partition proof
    // aligned with the pre-group input width, Set output.
    assert_eq!(group.expressions().len(), 1);
    assert_eq!(group.output().row().fields().len(), 1);
    assert_eq!(group.output().partition_keys().len(), 1);
    assert_eq!(
        group.partition_input().len(),
        group.children()[0].output().row().fields().len()
    );
    assert_eq!(
        group.output().cardinality(),
        quanta_gqlang_v2_types::CardinalityV1::Set
    );
    // Aggregate: consumes exactly the one Group, retains its partition
    // identity verbatim, and keeps the fold expression row-aligned with
    // the output column origins (the ordering property that generalizes
    // to multiple aggregates).
    assert_eq!(
        root.output().partition_keys(),
        group.output().partition_keys(),
        "aggregate output must retain its Group partition identity"
    );
    assert_eq!(root.expressions().len(), 1);
    assert_eq!(
        root.expressions()[0].aggregate_context().aggregation(),
        AggregationClassV1::Aggregate
    );
    assert_eq!(root.output().columns().len(), 1);
    assert!(matches!(
        root.output().columns()[0].origin(),
        OutputColumnOriginV2::Expression(origin)
            if *origin == root.expressions()[0].key()
    ));
}

#[test]
fn scalar_count_star_consumes_the_producer_sealed_empty_partition() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ScalarCountStar).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Aggregate);
    let group = &root.children()[0];
    assert_eq!(group.family(), AlgebraFamilyV2::Group);
    // The implicit Group(empty) is key-only: no keys, no row, no columns,
    // no partition, Set cardinality.
    assert!(group.expressions().is_empty());
    assert!(group.partition_input().is_empty());
    assert!(group.output().row().fields().is_empty());
    assert!(group.output().columns().is_empty());
    assert!(group.output().partition_keys().is_empty());
    assert_eq!(
        group.output().cardinality(),
        quanta_gqlang_v2_types::CardinalityV1::Set
    );
    assert_eq!(group.children()[0].family(), AlgebraFamilyV2::Scan);
    // count(*): the retained fold uses the authored star, never a
    // zero-argument call spelling.
    assert_eq!(root.expressions().len(), 1);
    let fold = &root.expressions()[0];
    assert!(fold.aggregate_context().uses_star());
    assert_eq!(
        fold.aggregate_context().aggregation(),
        AggregationClassV1::Aggregate
    );
    assert!(matches!(
        fold.shape(),
        TypedExprShapeV2::Call { arguments, .. }
            if matches!(arguments.as_slice(), [TypedCallArgumentV2::Star])
    ));
    assert!(root.output().partition_keys().is_empty());
}

#[test]
fn inner_join_retains_the_typed_on_and_ordered_child_schema() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::InnerJoinOn).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Join);
    assert_eq!(
        root.join_mode(),
        Some(quanta_gqlang_v2_sema::JoinModeV2::Inner)
    );
    assert!(root.predicate().is_some());
    let children = root.children();
    assert_eq!(children.len(), 2);
    // Ordered concatenation of the child columns and the union of the
    // child provenance are checked at construction.
    assert_eq!(
        root.output().columns(),
        children[0]
            .output()
            .columns()
            .iter()
            .chain(children[1].output().columns().iter())
            .cloned()
            .collect::<Vec<_>>()
    );
    assert_eq!(root.output().provenance().iter().count(), 2);
}

#[test]
fn project_retains_exact_expressions_and_row_aligned_origins() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ProjectMultipleExpressions)
            .unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Project);
    assert_eq!(root.expressions().len(), 2);
    assert_eq!(root.output().columns().len(), 2);
    for (expression, column) in root.expressions().iter().zip(root.output().columns()) {
        assert!(matches!(
            column.origin(),
            OutputColumnOriginV2::Expression(origin) if *origin == expression.key()
        ));
    }
    assert_eq!(root.children()[0].family(), AlgebraFamilyV2::Scan);
}

#[test]
fn sort_retains_direction_null_order_and_sealed_key_identity() {
    let reference = reference_relation_document_v2(ReferenceRelationFixtureV2::SortDesc).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::Sort);
    let keys = root.order_keys();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].direction(), SortDirectionV2::Desc);
    assert_eq!(keys[0].null_order(), NullOrderV2::Last);
    // The key's value type and identity stay sealed: the ordering equals
    // the output ordering contract and the input frame is preserved.
    let input = &root.children()[0];
    assert_eq!(root.output().ordering(), keys);
    assert_eq!(
        keys[0].value_type(),
        input.output().row().fields()[0].value_type()
    );
    assert_eq!(root.output().row(), input.output().row());
}

#[test]
fn correlated_exists_retains_the_authored_capture_edge_set() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedExists).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    // The outer filter stage vanishes (pure EXISTS); the NESTED relation's
    // own ordinary predicate keeps exactly its own Filter stage.
    assert_eq!(
        families_in_authoring_order(root),
        vec![
            AlgebraFamilyV2::SemiJoin,
            AlgebraFamilyV2::Scan,
            AlgebraFamilyV2::Filter,
            AlgebraFamilyV2::Scan,
        ]
    );
    assert_eq!(root.children()[0].family(), AlgebraFamilyV2::Scan);
    assert!(root.children()[1].filter_conjuncts().len() == 1);
    let correlations = root.correlations().expect("semi join retains correlations");
    assert_eq!(correlations.iter().count(), 1);
    let edge = &correlations.iter().collect::<Vec<_>>()[0];
    // The correlation is a resolved outer binding identity plus the inner
    // reference location, in authored stable order.
    assert_eq!(edge.binding().canonical_name(), "r");
    assert!(reference.document().expression(edge.reference()).is_some());
}

#[test]
fn correlated_not_exists_retains_the_authored_capture_edge_set() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedNotExists).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    assert_eq!(root.family(), AlgebraFamilyV2::AntiJoin);
    assert!(root
        .correlations()
        .is_some_and(|set| set.iter().count() == 1));
}

#[test]
fn conjunctive_exists_pair_lowers_each_leaf_exactly_once_in_stable_order() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ConjunctiveExistsPair).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    let families = families_in_authoring_order(root);
    assert_eq!(
        families,
        vec![
            AlgebraFamilyV2::ExistsFilter,
            AlgebraFamilyV2::Scan,
            AlgebraFamilyV2::Scan,
            AlgebraFamilyV2::Scan,
        ],
        "two conjunctive EXISTS lower to one flat operator over three scans"
    );
    let children = root.children();
    assert_eq!(scan_qualified_name(children[0]), "app::records");
    assert_eq!(scan_qualified_name(children[1]), "app::other");
    assert_eq!(scan_qualified_name(children[2]), "app::records2");
    assert_eq!(root.existential_terms().len(), 2);
    assert!(root
        .existential_terms()
        .iter()
        .all(|term| term.correlations().is_empty()));
}

#[test]
fn mixed_conjunction_keeps_exactly_one_residual_filter() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ExistsWithBoolConjunct).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    // The residual Filter is the stage root and holds the FULL ordinary
    // conjunction; the EXISTS lowered exactly once below it. The ordinary
    // conjunct is the authored BoolValue fact wrapping its Bool literal.
    assert_eq!(root.family(), AlgebraFamilyV2::Filter);
    let conjuncts = root.filter_conjuncts();
    assert_eq!(conjuncts.len(), 1);
    assert!(matches!(
        conjuncts[0].expression().shape(),
        TypedExprShapeV2::BoolValue { .. }
    ));
    assert_eq!(
        families_in_authoring_order(root),
        vec![
            AlgebraFamilyV2::Filter,
            AlgebraFamilyV2::SemiJoin,
            AlgebraFamilyV2::Scan,
            AlgebraFamilyV2::Scan,
        ]
    );
    let semi = &root.children()[0];
    assert_eq!(scan_qualified_name(semi.children()[1]), "app::other");
    assert_eq!(scan_qualified_name(semi.children()[0]), "app::records");
    // Filter and SemiJoin both retain the filter stage's exact output.
    assert_eq!(root.output(), semi.output());
    assert_eq!(root.output(), lowered.output());
}

#[test]
fn nested_exists_lowers_each_level_once_with_separate_capture_sets() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::NestedExistsTwoLevels).unwrap();
    let lowered = lower_relation_item(reference.document(), reference.item_id()).unwrap();
    let root = lowered.root();
    let families = families_in_authoring_order(root);
    assert_eq!(
            families,
            vec![
                AlgebraFamilyV2::SemiJoin,
                AlgebraFamilyV2::Scan,
                AlgebraFamilyV2::Filter,
                AlgebraFamilyV2::SemiJoin,
                AlgebraFamilyV2::Scan,
                AlgebraFamilyV2::Filter,
                AlgebraFamilyV2::Scan,
            ],
            "each EXISTS level lowers to exactly one SemiJoin; ordinary conjuncts stay in exactly one residual Filter per stage"
        );
    // The outer EXISTS captures the outer binding at both reference
    // sites; the nested EXISTS carries its own single-edge capture set.
    assert!(root
        .correlations()
        .is_some_and(|set| set.iter().count() == 2));
    let middle_filter = &root.children()[1];
    assert_eq!(middle_filter.family(), AlgebraFamilyV2::Filter);
    assert_eq!(middle_filter.filter_conjuncts().len(), 1);
    let nested_semi = &middle_filter.children()[0];
    assert!(nested_semi
        .correlations()
        .is_some_and(|set| set.iter().count() == 1));
}

#[test]
fn mixed_scalar_aggregate_projection_is_an_impossible_contract() {
    // Forged at the document boundary: one scalar group key and one
    // aggregate fold in the same projection over a real Group. Sema
    // seals aggregate projections uniformly, so this shape can only be
    // minted through the test seam and must die at the checked
    // Aggregate constructor, not lower as a mixed Aggregate node.
    let forged = forged_mixed_scalar_aggregate_projection_document_v2().unwrap();
    let diagnostic = lower_relation_item(forged.document(), forged.item_id())
        .expect_err("a mixed scalar/aggregate projection must not lower");
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::InvalidStageContract
    );
    assert_eq!(diagnostic.code(), "algebra.invalid_stage_contract");
    assert!(
        diagnostic.location().is_some(),
        "the rejection names the forged projection stage"
    );
}

#[test]
fn nested_registry_tree_with_a_swapped_inner_scan_rejects() {
    // Forged nested registry entry: the EXISTS root key still resolves,
    // but the registered tree's inner scan carries a minted stage
    // identity while the nested Filter keeps its sealed output columns.
    // The row-aligned frame check at the nested Filter constructor must
    // reject the disagreement; the right tree is consumed exactly by
    // registry key and never re-derived from the outer predicate.
    let forged = forged_wrong_nested_root_document_v2().unwrap();
    let diagnostic = lower_relation_item(forged.document(), forged.item_id())
        .expect_err("a nested tree whose frame disagrees with its stages must not lower");
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::InvalidStageContract
    );
    assert_eq!(diagnostic.code(), "algebra.invalid_stage_contract");
    assert!(
        diagnostic.location().is_some(),
        "the rejection names the inconsistent nested stage"
    );
}

#[test]
fn public_relation_walk_small_receipt_exact_and_one_over() {
    // A forged small-receipt chain: sema resolves it under the standard
    // budget and the document carries the small receipt, so the relation
    // walk's own charging is the only limit. A chain of depth d lowers
    // d + 1 relation stages.
    let exact_depth = SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;
    let exact = forged_small_receipt_relation_chain_document_v2(exact_depth).unwrap();
    let lowered =
        lower_relation_item(exact.document(), exact.item_id()).expect("exact budget lowers");
    assert_eq!(lowered.root().family(), AlgebraFamilyV2::Filter);

    let over = forged_small_receipt_relation_chain_document_v2(exact_depth + 1).unwrap();
    let diagnostic = lower_relation_item(over.document(), over.item_id())
        .expect_err("one stage over the receipt must reject");
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::TraversalBudgetExceeded
    );
    assert_eq!(diagnostic.code(), "algebra.traversal_budget_exceeded");
    assert!(diagnostic.location().is_some());
}

#[test]
fn public_expression_walk_small_receipt_exact_and_one_over() {
    // A left-leaning conjunction of depth d has d And nodes and d + 1
    // BoolValue leaves, each forwarding one Bool literal: the
    // decomposition walk charges 2 * d + 1 nodes and the per-leaf
    // descendant search charges two more per leaf, 4 * d + 3 charged
    // nodes in total. Depth 1 charges seven of the eight small-receipt
    // nodes; depth 2 charges eleven and rejects one over.
    let exact = forged_small_receipt_bool_tree_document_v2(1).unwrap();
    let lowered =
        lower_relation_item(exact.document(), exact.item_id()).expect("exact budget lowers");
    assert_eq!(lowered.root().family(), AlgebraFamilyV2::Filter);
    assert_eq!(lowered.root().filter_conjuncts().len(), 2);

    let over = forged_small_receipt_bool_tree_document_v2(2).unwrap();
    assert_eq!(
        over.document().budget_receipt().max_expression_nodes(),
        SMALL_BUDGET_MAX_EXPRESSION_NODES
    );
    let diagnostic = lower_relation_item(over.document(), over.item_id())
        .expect_err("one node over the receipt must reject");
    assert_eq!(
        diagnostic.kind(),
        AlgebraFailureKindV2::TraversalBudgetExceeded
    );
    assert_eq!(diagnostic.code(), "algebra.traversal_budget_exceeded");
    assert!(diagnostic.location().is_some());
}
