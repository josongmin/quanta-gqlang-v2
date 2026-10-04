#![expect(
    clippy::indexing_slicing,
    clippy::panic_in_result_fn,
    reason = "owner assertions pin exact fixture shapes and index pinned capture vectors"
)]

use super::*;
use crate::{NullOrderV2, RelationExistsPolarityV2, SortDirectionV2};

#[test]
fn every_positive_fixture_variant_resolves() -> Result<(), String> {
    let variants = [
        ReferenceRelationFixtureV2::Scan,
        ReferenceRelationFixtureV2::Filter,
        ReferenceRelationFixtureV2::CrossJoin,
        ReferenceRelationFixtureV2::Exists,
        ReferenceRelationFixtureV2::NotExists,
        ReferenceRelationFixtureV2::ProjectMultipleExpressions,
        ReferenceRelationFixtureV2::InnerJoinOn,
        ReferenceRelationFixtureV2::SelfJoinInner,
        ReferenceRelationFixtureV2::GroupedAggregateSum,
        ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys,
        ReferenceRelationFixtureV2::ScalarCountStar,
        ReferenceRelationFixtureV2::SortAsc,
        ReferenceRelationFixtureV2::SortDesc,
        ReferenceRelationFixtureV2::SortNullsLastNullableField,
        ReferenceRelationFixtureV2::SortMultiKeyMixedDirections,
        ReferenceRelationFixtureV2::SortAliasKeyed,
        ReferenceRelationFixtureV2::SortByProjectedExpression,
        ReferenceRelationFixtureV2::CorrelatedExists,
        ReferenceRelationFixtureV2::CorrelatedNotExists,
        ReferenceRelationFixtureV2::ConjunctiveExistsPair,
        ReferenceRelationFixtureV2::DisjunctiveExistsPair,
        ReferenceRelationFixtureV2::NegatedExists,
        ReferenceRelationFixtureV2::ExistsWithBoolConjunct,
        ReferenceRelationFixtureV2::NestedExistsTwoLevels,
        ReferenceRelationFixtureV2::ExistsWithMultiplicativeRight,
        ReferenceRelationFixtureV2::AsymmetricJoinPreorder,
    ];
    for fixture in variants {
        let reference = reference_relation_document_v2(fixture)?;
        assert!(relation_item(reference.document(), reference.item_id()).is_ok());
    }
    Ok(())
}

#[test]
fn projection_fixture_retains_expression_origins_and_types() -> Result<(), String> {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ProjectMultipleExpressions)?;
    let relation = relation_item(reference.document(), reference.item_id())?;
    let columns = relation.output().columns();
    assert_eq!(columns.len(), 2);
    let first = columns.first().ok_or("first column expected")?;
    let second = columns.get(1).ok_or("second column expected")?;
    assert_eq!(first.name(), "num");
    assert_eq!(second.name(), "label");
    assert!(matches!(
        first.origin(),
        OutputColumnOriginV2::Expression(_)
    ));
    assert!(matches!(
        second.origin(),
        OutputColumnOriginV2::Expression(_)
    ));
    assert_eq!(
        relation
            .output()
            .row()
            .fields()
            .get(1)
            .ok_or("second field expected")?
            .value_type(),
        &ValueTypeV1::string(),
    );
    Ok(())
}

#[test]
fn join_fixtures_retain_mode_and_distinct_scan_occurrences() -> Result<(), String> {
    let join = reference_relation_document_v2(ReferenceRelationFixtureV2::InnerJoinOn)?;
    let relation = relation_item(join.document(), join.item_id())?;
    assert_eq!(relation.root().join_mode(), Some(JoinModeV2::Inner));
    assert!(relation.root().predicate().is_some());
    assert_eq!(relation.root().inputs().count(), 2);
    let mut join_inputs = relation.root().inputs();
    let left = join_inputs.next().ok_or("left join input expected")?;
    let right = join_inputs.next().ok_or("right join input expected")?;
    let left_source = left
        .output()
        .provenance()
        .iter()
        .next()
        .ok_or("left source provenance expected")?;
    let right_source = right
        .output()
        .provenance()
        .iter()
        .next()
        .ok_or("right source provenance expected")?;
    assert_ne!(left_source, right_source);
    let sources = relation.output().provenance().iter().collect::<Vec<_>>();
    assert_eq!(
        sources.len(),
        2,
        "the join must retain both distinct origins"
    );
    assert!(sources.contains(&left_source));
    assert!(sources.contains(&right_source));

    let self_join = reference_relation_document_v2(ReferenceRelationFixtureV2::SelfJoinInner)?;
    let relation = relation_item(self_join.document(), self_join.item_id())?;
    let mut inputs = relation.root().inputs();
    let left = inputs.next().ok_or("left input expected")?;
    let right = inputs.next().ok_or("right input expected")?;
    let left_scan = left.inputs().next().ok_or("left scan expected")?;
    let right_scan = right.inputs().next().ok_or("right scan expected")?;
    assert_eq!(left_scan.source(), right_scan.source());
    assert_ne!(left_scan.key(), right_scan.key());
    Ok(())
}

#[test]
fn aggregate_fixtures_retain_group_shapes() -> Result<(), String> {
    let grouped = reference_relation_document_v2(ReferenceRelationFixtureV2::GroupedAggregateSum)?;
    let relation = relation_item(grouped.document(), grouped.item_id())?;
    assert!(matches!(relation.root(), TypedRelationExprV2::Project(_)));
    let group = relation
        .root()
        .inputs()
        .next()
        .ok_or("projection input expected")?;
    assert!(matches!(group, TypedRelationExprV2::Group(_)));
    assert_eq!(group.expressions().len(), 1);
    assert_eq!(relation.output().cardinality(), CardinalityV1::Set);
    assert_eq!(
        relation.output().partition_keys(),
        group.output().partition_keys()
    );

    let scalar = reference_relation_document_v2(ReferenceRelationFixtureV2::ScalarCountStar)?;
    let relation = relation_item(scalar.document(), scalar.item_id())?;
    let group = relation
        .root()
        .inputs()
        .next()
        .ok_or("implicit group expected")?;
    assert!(matches!(group, TypedRelationExprV2::Group(_)));
    assert!(group.expressions().is_empty());
    assert!(group.partition_input().is_empty());
    assert!(relation.output().partition_keys().is_empty());
    Ok(())
}

#[test]
fn sort_fixtures_retain_directions_and_null_policy() -> Result<(), String> {
    let cases = [
        (
            ReferenceRelationFixtureV2::SortAsc,
            vec![(SortDirectionV2::Asc, NullOrderV2::Last)],
        ),
        (
            ReferenceRelationFixtureV2::SortDesc,
            vec![(SortDirectionV2::Desc, NullOrderV2::Last)],
        ),
        (
            ReferenceRelationFixtureV2::SortNullsLastNullableField,
            vec![(SortDirectionV2::Desc, NullOrderV2::Last)],
        ),
        (
            ReferenceRelationFixtureV2::SortMultiKeyMixedDirections,
            vec![
                (SortDirectionV2::Asc, NullOrderV2::Last),
                (SortDirectionV2::Desc, NullOrderV2::Last),
            ],
        ),
        (
            ReferenceRelationFixtureV2::SortAliasKeyed,
            vec![(SortDirectionV2::Desc, NullOrderV2::Last)],
        ),
        (
            ReferenceRelationFixtureV2::SortByProjectedExpression,
            vec![(SortDirectionV2::Desc, NullOrderV2::Last)],
        ),
    ];
    for (fixture, expected) in cases {
        let reference = reference_relation_document_v2(fixture)?;
        let relation = relation_item(reference.document(), reference.item_id())?;
        assert!(matches!(relation.root(), TypedRelationExprV2::Sort(_)));
        let actual: Vec<_> = relation
            .output()
            .ordering()
            .iter()
            .map(|key| (key.direction(), key.null_order()))
            .collect();
        assert_eq!(actual, expected);
    }
    Ok(())
}

#[test]
fn exists_fixtures_retain_correlations_and_conjuncts() -> Result<(), String> {
    let correlated = reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedExists)?;
    let mut correlated_shapes =
        correlated
            .document()
            .expressions()
            .filter_map(|(_key, expression)| {
                if let TypedExprShapeV2::RelationExists {
                    polarity,
                    correlations,
                    ..
                } = expression.shape()
                {
                    Some((*polarity, correlations.len()))
                } else {
                    None
                }
            });
    assert_eq!(
        correlated_shapes.next(),
        Some((RelationExistsPolarityV2::Exists, 1))
    );

    let not_exists =
        reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedNotExists)?;
    assert!(not_exists.document().expressions().any(
        |(_key, expression)| matches!(expression.shape(),
            TypedExprShapeV2::RelationExists {
                polarity: RelationExistsPolarityV2::NotExists,
                correlations,
                ..
            } if !correlations.is_empty())
    ));

    let nested = reference_relation_document_v2(ReferenceRelationFixtureV2::NestedExistsTwoLevels)?;
    assert_eq!(nested.document().nested_relations().count(), 2);
    let pair = reference_relation_document_v2(ReferenceRelationFixtureV2::ConjunctiveExistsPair)?;
    assert_eq!(pair.document().nested_relations().count(), 2);
    let mixed = reference_relation_document_v2(ReferenceRelationFixtureV2::ExistsWithBoolConjunct)?;
    assert_eq!(mixed.document().nested_relations().count(), 1);

    let disjunctive =
        reference_relation_document_v2(ReferenceRelationFixtureV2::DisjunctiveExistsPair)?;
    assert_eq!(disjunctive.document().nested_relations().count(), 2);
    let negated = reference_relation_document_v2(ReferenceRelationFixtureV2::NegatedExists)?;
    assert_eq!(negated.document().nested_relations().count(), 1);
    assert!(negated
        .document()
        .expressions()
        .any(|(_key, expression)| matches!(expression.shape(), TypedExprShapeV2::Not { .. })));
    assert!(disjunctive
        .document()
        .expressions()
        .any(|(_key, expression)| matches!(expression.shape(), TypedExprShapeV2::Or { .. })));
    Ok(())
}

#[test]
fn second_source_document_is_foreign_to_the_primary_source() -> Result<(), String> {
    let secondary = second_source_relation_document_v2()?;
    assert_eq!(
        secondary.document().source_id(),
        SECONDARY_REFERENCE_SOURCE_V2
    );
    assert!(relation_item(secondary.document(), secondary.item_id()).is_ok());
    let primary = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)?;
    assert_ne!(
        primary.document().source_id(),
        secondary.document().source_id()
    );
    assert!(primary.document().item(secondary.item_id()).is_none());
    Ok(())
}

#[test]
fn deep_chain_bounds_are_exact_at_the_small_budget() -> Result<(), String> {
    let exact = deep_relation_chain_document_v2(
        SMALL_BUDGET_RELATION_CHAIN_EXACT_DEPTH,
        ReferenceBudgetV2::Small,
    )?;
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
    let deep = deep_relation_chain_document_v2(24, ReferenceBudgetV2::Standard)?;
    assert!(relation_item(deep.document(), deep.item_id()).is_ok());
    Ok(())
}

#[test]
fn deep_bool_tree_bounds_follow_the_documented_formulas() -> Result<(), String> {
    let exact =
        deep_bool_tree_document_v2(SMALL_BUDGET_BOOL_TREE_EXACT_DEPTH, ReferenceBudgetV2::Small)?;
    assert_eq!(
        exact.document().budget_receipt().max_expression_nodes(),
        SMALL_BUDGET_MAX_EXPRESSION_NODES
    );
    assert_eq!(
        u32::try_from(exact.document().expressions().count())
            .map_err(|_error| "expression fact count exceeds u32".to_owned())?,
        3 * SMALL_BUDGET_BOOL_TREE_EXACT_DEPTH + 2
    );
    let over =
        deep_bool_tree_document_v2(SMALL_BUDGET_BOOL_TREE_OVER_DEPTH, ReferenceBudgetV2::Small);
    assert!(over
        .err()
        .is_some_and(|error| error.contains("semantic_steps exceeds frontend limit")));
    Ok(())
}

#[test]
fn two_key_group_fixture_retains_both_partition_keys_in_authored_order() -> Result<(), String> {
    let reference = reference_relation_document_v2(
        ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys,
    )?;
    let relation = relation_item(reference.document(), reference.item_id())?;
    assert!(matches!(relation.root(), TypedRelationExprV2::Project(_)));
    let group = relation
        .root()
        .inputs()
        .next()
        .ok_or("projection input expected")?;
    assert!(matches!(group, TypedRelationExprV2::Group(_)));
    assert_eq!(group.expressions().len(), 2);
    assert_eq!(group.output().partition_keys().len(), 2);
    assert_eq!(relation.output().partition_keys().len(), 2);
    assert_eq!(
        relation.output().partition_keys(),
        group.output().partition_keys()
    );
    let first_key = group
        .expressions()
        .first()
        .copied()
        .ok_or("first key expected")?;
    let second_key = group
        .expressions()
        .get(1)
        .copied()
        .ok_or("second key expected")?;
    assert_ne!(first_key, second_key);
    let first_name = group
        .output()
        .columns()
        .first()
        .ok_or("first key column expected")?
        .name()
        .to_owned();
    let second_name = group
        .output()
        .columns()
        .get(1)
        .ok_or("second key column expected")?
        .name()
        .to_owned();
    assert_eq!(first_name, "id");
    assert_eq!(second_name, "value");
    Ok(())
}

#[test]
fn forged_aggregate_without_group_removes_the_implicit_group() -> Result<(), String> {
    let forged = forged_aggregate_without_group_document_v2()?;
    let relation = relation_item(forged.document(), forged.item_id())?;
    assert!(matches!(relation.root(), TypedRelationExprV2::Project(_)));
    let input = relation
        .root()
        .inputs()
        .next()
        .ok_or("projection input expected")?;
    assert!(matches!(input, TypedRelationExprV2::Scan(_)));

    let real = reference_relation_document_v2(ReferenceRelationFixtureV2::ScalarCountStar)?;
    let real_relation = relation_item(real.document(), real.item_id())?;
    let real_group = real_relation
        .root()
        .inputs()
        .next()
        .ok_or("real group expected")?;
    assert!(matches!(real_group, TypedRelationExprV2::Group(_)));
    Ok(())
}

#[test]
fn forged_mixed_projection_carries_scalar_and_aggregate_columns() -> Result<(), String> {
    let forged = forged_mixed_scalar_aggregate_projection_document_v2()?;
    let relation = relation_item(forged.document(), forged.item_id())?;
    assert_eq!(relation.root().expressions().len(), 2);
    assert_eq!(relation.output().columns().len(), 2);
    let aggregate_classes: Vec<_> = relation
        .root()
        .expressions()
        .iter()
        .filter_map(|key| {
            forged
                .document()
                .expression(*key)
                .map(|expression| expression.aggregate_context().aggregation())
        })
        .collect();
    assert_eq!(aggregate_classes.len(), 2);
    assert!(aggregate_classes.contains(&AggregationClassV1::Aggregate));
    assert!(aggregate_classes.contains(&AggregationClassV1::Scalar));
    Ok(())
}

#[test]
fn forged_cardinality_downgrade_pins_bag_on_the_aggregate_stage() -> Result<(), String> {
    let forged = forged_cardinality_downgrade_document_v2()?;
    let relation = relation_item(forged.document(), forged.item_id())?;
    assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
    let real = reference_relation_document_v2(ReferenceRelationFixtureV2::GroupedAggregateSum)?;
    let real_relation = relation_item(real.document(), real.item_id())?;
    assert_eq!(real_relation.output().cardinality(), CardinalityV1::Set);
    Ok(())
}

#[test]
fn wrong_scan_occurrence_projection_rejects_at_sealing() {
    assert_eq!(
        try_wrong_scan_occurrence_projection_v2(),
        Err("invalid typed relation stage arity or payload".to_owned())
    );
}

#[test]
fn forged_wrong_nested_root_swaps_the_nested_tree() -> Result<(), String> {
    let forged = forged_wrong_nested_root_document_v2()?;
    let real = reference_relation_document_v2(ReferenceRelationFixtureV2::CorrelatedExists)?;
    let forged_key = exists_root(forged.document())?;
    let real_key = exists_root(real.document())?;
    assert_eq!(forged_key, real_key);
    let forged_nested = forged
        .document()
        .nested_relation(forged_key)
        .ok_or("forged nested relation expected")?;
    let real_nested = real
        .document()
        .nested_relation(real_key)
        .ok_or("real nested relation expected")?;
    assert_eq!(forged_nested.key(), real_nested.key());
    let forged_child = forged_nested
        .inputs()
        .next()
        .ok_or("forged nested scan expected")?;
    let real_child = real_nested
        .inputs()
        .next()
        .ok_or("real nested scan expected")?;
    assert_ne!(forged_child.key(), real_child.key());
    Ok(())
}

#[test]
fn forged_capture_mutations_assemble_or_reject_exactly() -> Result<(), String> {
    let capture_vector =
        |document: &ReferenceRelationDocumentV2| -> Result<Vec<SourceQualifiedNodeIdV2>, String> {
            let relation = relation_item(document.document(), document.item_id())?;
            if let TypedExprShapeV2::RelationExists { correlations, .. } = relation
                .root()
                .predicate()
                .ok_or("outer predicate expected")?
                .expression()
                .shape()
            {
                Ok(correlations.iter().map(|edge| edge.reference()).collect())
            } else {
                Err("EXISTS predicate expected".to_owned())
            }
        };
    let (_checked, real, item_id) = two_capture_correlated_exists_parts()?;
    assert!(relation_item(&real, item_id).is_ok());
    let pristine_document = ReferenceRelationDocumentV2 {
        document: real.clone(),
        item_id,
    };
    let pristine = capture_vector(&pristine_document)?;
    assert_eq!(pristine.len(), 2);
    assert_ne!(pristine[0], pristine[1]);

    let dropped = forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Dropped)?;
    assert_eq!(capture_vector(&dropped)?, vec![pristine[0]]);
    let swapped = forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Swapped)?;
    assert_eq!(capture_vector(&swapped)?, vec![pristine[1], pristine[0]]);

    let duplicate = forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Duplicate);
    assert!(duplicate.err().is_some_and(
        |error| error.contains("duplicate correlation reference in one nested relation")
    ));
    let foreign = forged_capture_mutation_document_v2(ForgedCaptureEdgeMutationV2::Foreign);
    assert!(foreign
        .err()
        .is_some_and(|error| error
            .contains("correlation reference is foreign to this document's expressions")));

    // The map fact and the Filter predicate carry the same mutated vector.
    let swapped_fact = swapped
        .document()
        .expressions()
        .find_map(|(_key, expression)| {
            if let TypedExprShapeV2::RelationExists { correlations, .. } = expression.shape() {
                Some(
                    correlations
                        .iter()
                        .map(|edge| edge.reference())
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .ok_or("swapped document must retain its EXISTS fact")?;
    assert_eq!(swapped_fact, vec![pristine[1], pristine[0]]);
    assert!(relation_item(swapped.document(), swapped.item_id()).is_ok());
    assert_eq!(swapped.document().nested_relations().count(), 1);
    Ok(())
}

#[test]
fn forged_join_seal_rejections_are_exact() -> Result<(), String> {
    assert_eq!(
        forged_inner_join_without_on_seal_v2().err(),
        Some("invalid typed relation stage arity or payload")
    );
    assert_eq!(
        forged_cross_join_with_on_seal_v2().err(),
        Some("invalid typed relation stage arity or payload")
    );
    Ok(())
}

#[test]
fn capture_edge_mutations_match_their_exact_shapes() -> Result<(), String> {
    let pristine = pristine_capture_edge_set_v2()?;
    assert_eq!(pristine.len(), 2);
    assert_ne!(
        pristine.first().map(|edge| edge.reference().node_id),
        pristine.get(1).map(|edge| edge.reference().node_id)
    );
    let dropped = forged_capture_edge_set_v2(ForgedCaptureEdgeMutationV2::Dropped)?;
    assert_eq!(dropped.len(), 1);
    let duplicate = forged_capture_edge_set_v2(ForgedCaptureEdgeMutationV2::Duplicate)?;
    assert_eq!(duplicate.len(), 2);
    assert_eq!(
        duplicate.first().map(|edge| edge.reference().node_id),
        duplicate.get(1).map(|edge| edge.reference().node_id)
    );
    let swapped = forged_capture_edge_set_v2(ForgedCaptureEdgeMutationV2::Swapped)?;
    assert_eq!(
        swapped.first().map(|edge| edge.reference().node_id),
        pristine.get(1).map(|edge| edge.reference().node_id)
    );
    let foreign = forged_capture_edge_set_v2(ForgedCaptureEdgeMutationV2::Foreign)?;
    assert_eq!(
        foreign.get(1).map(|edge| edge.reference().node_id),
        Some(HirNodeIdV1::from_raw(9_999))
    );
    Ok(())
}

#[test]
fn forged_small_receipt_documents_carry_oversized_trees() -> Result<(), String> {
    let chain =
        forged_small_receipt_relation_chain_document_v2(SMALL_BUDGET_RELATION_CHAIN_OVER_DEPTH)?;
    assert_eq!(
        chain.document().budget_receipt().max_semantic_steps(),
        SMALL_BUDGET_MAX_SEMANTIC_STEPS
    );
    assert!(relation_item(chain.document(), chain.item_id()).is_ok());

    let tree = forged_small_receipt_bool_tree_document_v2(4)?;
    assert_eq!(
        tree.document().budget_receipt().max_expression_nodes(),
        SMALL_BUDGET_MAX_EXPRESSION_NODES
    );
    assert_eq!(tree.document().expressions().count(), 14);
    Ok(())
}
