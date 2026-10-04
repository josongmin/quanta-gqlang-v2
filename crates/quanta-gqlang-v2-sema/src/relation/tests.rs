use super::*;
use crate::names::ResolvedCompositionSourceIdV2;
use crate::ResolvedBindingKindV2;
use quanta_gqlang_v2_hir::{
    HirNodeIdV1, LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_types::RecordFieldTypeV1;

#[test]
#[expect(
    clippy::expect_used,
    reason = "exact sealed projection constructor controls"
)]
fn ordinary_projection_rejects_misaligned_expression_origins() {
    use crate::test_support::{reference_relation_document_v2, ReferenceRelationFixtureV2};
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::ProjectMultipleExpressions)
            .expect("real projection fixture");
    let Some(crate::SemanticItemV2::Relation(relation)) =
        fixture.document().item(fixture.item_id())
    else {
        unreachable!("fixture contains its relation");
    };
    let root = relation.root();
    let input = root.inputs().next().expect("project input");
    let keys = root.expressions().to_vec();
    assert_eq!(keys.len(), 2);
    assert!(TypedRelationExprV2::try_project(
        root.key(),
        root.location().clone(),
        input.clone(),
        keys.clone(),
        root.output().clone()
    )
    .is_ok());
    let mut short = keys.clone();
    short.pop();
    let mut long = keys.clone();
    long.push(*keys.first().expect("first expression"));
    let mut reversed = keys.clone();
    reversed.reverse();
    let mut foreign = keys.clone();
    foreign.first_mut().expect("first expression").source_id = SourceIdV2::from_raw(999);
    for changed in [Vec::new(), short, long, reversed, foreign] {
        assert_eq!(
            TypedRelationExprV2::try_project(
                root.key(),
                root.location().clone(),
                input.clone(),
                changed,
                root.output().clone()
            )
            .err(),
            Some("invalid typed relation stage arity or payload")
        );
    }
    let mut generated = root.output().clone();
    for column in &mut generated.columns {
        column.origin = OutputColumnOriginV2::Generated(fixture.item_id());
    }
    assert_eq!(
        TypedRelationExprV2::try_project(
            root.key(),
            root.location().clone(),
            input.clone(),
            keys.clone(),
            generated
        )
        .err(),
        Some("invalid typed relation stage arity or payload")
    );
    assert_eq!(input.output().columns().len(), keys.len());
    assert_eq!(
        TypedRelationExprV2::try_project(
            root.key(),
            root.location().clone(),
            input.clone(),
            keys,
            input.output().clone()
        )
        .err(),
        Some("invalid typed relation stage arity or payload")
    );
}

#[test]
#[expect(clippy::expect_used, reason = "sealed join constructor controls")]
fn join_requires_exact_operand_frame_and_bag_cardinality() {
    use crate::test_support::{
        forged_output_contract_v2, reference_relation_document_v2,
        OutputContractMutationV2 as Mutation, ReferenceRelationFixtureV2 as Fixture,
    };
    for fixture in [
        Fixture::CrossJoin,
        Fixture::InnerJoinOn,
        Fixture::SelfJoinInner,
    ] {
        let document = reference_relation_document_v2(fixture).expect("real join");
        let Some(crate::SemanticItemV2::Relation(relation)) =
            document.document().item(document.item_id())
        else {
            unreachable!("real relation");
        };
        let root = relation.root();
        let build = |output| {
            TypedRelationExprV2::try_from_stage(TypedRelationStageInputV2 {
                key: root.key(),
                location: root.location().clone(),
                kind: TypedRelationStageKindV2::Join,
                inputs: root.inputs().cloned().collect(),
                expressions: Vec::new(),
                predicate: root.predicate().cloned(),
                source: None,
                join_mode: root.join_mode(),
                implicit_partition: false,
                partition_input: Vec::new(),
                output,
            })
        };
        assert!(build(root.output().clone()).is_ok());
        for mutation in [
            Mutation::SetCardinality,
            Mutation::SequenceCardinality,
            Mutation::FirstFieldType,
            Mutation::FirstFieldOptionality,
            Mutation::Partition,
            Mutation::Ordering,
        ] {
            let output = forged_output_contract_v2(root.output(), root.key(), mutation)
                .expect("generic output is row-aligned");
            assert_ne!(&output, root.output(), "{fixture:?} {mutation:?}");
            assert_eq!(
                build(output).err(),
                Some("invalid typed relation stage arity or payload"),
                "{fixture:?} {mutation:?}"
            );
        }
    }
}

fn empty_output() -> Result<RelationOutputContractV2, String> {
    RelationOutputContractV2::try_new(
        RecordTypeV1::try_new(Vec::new()).map_err(|error| format!("{error:?}"))?,
        OutputProvenanceV2::default(),
        Vec::new(),
        CardinalityV1::Bag,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(str::to_owned)
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "owner test assertions must fail on a contract mismatch"
)]
fn output_columns_require_exact_row_alignment_and_expression_source() -> Result<(), String> {
    let source_id = SourceIdV2::from_raw(8);
    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id,
        source_len: 2,
    }])
    .map_err(|error| format!("{error:?}"))?;
    let location = registry
        .try_check_location(LocatedSpanV2 {
            source_id,
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| format!("{error:?}"))?;
    let field = RecordFieldTypeV1::try_new("value", ValueTypeV1::int(), location.clone())
        .map_err(|error| format!("{error:?}"))?;
    let row = RecordTypeV1::try_new(vec![field]).map_err(|error| format!("{error:?}"))?;
    let key = SourceQualifiedNodeIdV2 {
        source_id,
        node_id: HirNodeIdV1::from_raw(9),
    };
    let column = OutputColumnContractV2::new(
        "value".into(),
        OutputColumnOriginV2::Expression(key),
        location.clone(),
    );
    let build = |columns| {
        RelationOutputContractV2::try_new(
            row.clone(),
            OutputProvenanceV2::default(),
            columns,
            CardinalityV1::Bag,
            Vec::new(),
            Vec::new(),
            ExpressionEffectsV2::pure_scalar(),
        )
    };
    let output = build(vec![column.clone()]).map_err(str::to_owned)?;
    assert_eq!(
        output.columns().first().map(OutputColumnContractV2::origin),
        Some(&OutputColumnOriginV2::Expression(key))
    );
    assert_eq!(
        build(Vec::new()).err(),
        Some("output columns differ from the typed row or authoritative origins")
    );
    let wrong_name = OutputColumnContractV2::new(
        "other".into(),
        OutputColumnOriginV2::Expression(key),
        location.clone(),
    );
    assert_eq!(
        build(vec![wrong_name]).err(),
        Some("output columns differ from the typed row or authoritative origins")
    );
    let foreign = OutputColumnContractV2::new(
        "value".into(),
        OutputColumnOriginV2::Expression(SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(10),
            node_id: key.node_id,
        }),
        location,
    );
    assert_eq!(
        build(vec![foreign]).err(),
        Some("output columns differ from the typed row or authoritative origins")
    );
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "owner test assertions must fail on a contract mismatch"
)]
fn join_mode_cannot_disagree_with_predicate_and_six_stage_vocabulary() -> Result<(), String> {
    let source_id = SourceIdV2::from_raw(8);
    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id,
        source_len: 2,
    }])
    .map_err(|error| format!("{error:?}"))?;
    let location = registry
        .try_check_location(LocatedSpanV2 {
            source_id,
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| format!("{error:?}"))?;
    let forged = TypedRelationStageInputV2 {
        key: SourceQualifiedNodeIdV2 {
            source_id,
            node_id: HirNodeIdV1::from_raw(3),
        },
        location,
        kind: TypedRelationStageKindV2::Join,
        inputs: Vec::new(),
        expressions: Vec::new(),
        predicate: None,
        source: None,
        join_mode: Some(JoinModeV2::Inner),
        implicit_partition: false,
        partition_input: Vec::new(),
        output: empty_output()?,
    };
    assert_eq!(
        TypedRelationExprV2::try_from_stage(forged).err(),
        Some("invalid typed relation stage arity or payload")
    );
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    reason = "owner test assertions must fail on a contract mismatch"
)]
fn duplicate_output_keys_are_rejected_by_the_only_constructor() -> Result<(), String> {
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(8),
        node_id: HirNodeIdV1::from_raw(11),
    };
    assert_eq!(
        RelationOutputContractV2::try_new(
            RecordTypeV1::try_new(Vec::new()).map_err(|error| format!("{error:?}"))?,
            OutputProvenanceV2::default(),
            Vec::new(),
            CardinalityV1::Bag,
            vec![key, key],
            Vec::new(),
            ExpressionEffectsV2::pure_scalar(),
        )
        .err(),
        Some("duplicate partition or order key")
    );
    let order = OrderKeyContractV2::new(
        key,
        SortDirectionV2::Desc,
        NullOrderV2::Last,
        ValueTypeV1::int(),
    );
    assert_eq!(
        RelationOutputContractV2::try_new(
            RecordTypeV1::try_new(Vec::new()).map_err(|error| format!("{error:?}"))?,
            OutputProvenanceV2::default(),
            Vec::new(),
            CardinalityV1::Bag,
            Vec::new(),
            vec![order.clone(), order],
            ExpressionEffectsV2::pure_scalar(),
        )
        .err(),
        Some("duplicate partition or order key")
    );
    Ok(())
}

#[test]
#[expect(
    clippy::panic_in_result_fn,
    clippy::too_many_lines,
    reason = "forged aggregate shapes must fail the sealed constructors"
)]
fn sealed_aggregate_constructors_reject_every_forged_shape() -> Result<(), String> {
    let source_id = SourceIdV2::from_raw(8);
    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id,
        source_len: 64,
    }])
    .map_err(|error| format!("{error:?}"))?;
    let location = registry
        .try_check_location(LocatedSpanV2 {
            source_id,
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| format!("{error:?}"))?;
    let other_registry = SourceRegistryViewV2::try_new(vec![
        SourceRegistrationV2 {
            source_id,
            source_len: 64,
        },
        SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 8,
        },
    ])
    .map_err(|error| format!("{error:?}"))?;
    let other_location = other_registry
        .try_check_location(LocatedSpanV2 {
            source_id,
            span: SpanV1::new(0, 1),
        })
        .map_err(|error| format!("{error:?}"))?;
    let key = |node: u32| SourceQualifiedNodeIdV2 {
        source_id,
        node_id: HirNodeIdV1::from_raw(node),
    };
    let binding = ResolvedValueBindingV2::try_new(
        source_id,
        HirNodeIdV1::from_raw(2),
        0,
        ResolvedBindingKindV2::Composition,
        "q".to_owned(),
    )
    .map_err(|error| error.to_string())?;
    let scan_source = ResolvedScanSourceV2::Composition(
        ResolvedCompositionSourceIdV2::try_from_binding(binding)
            .map_err(|error| error.to_string())?,
    );
    let scan_output = RelationOutputContractV2::try_new(
        RecordTypeV1::try_new(Vec::new()).map_err(|error| format!("{error:?}"))?,
        OutputProvenanceV2::new([scan_source.clone()]),
        Vec::new(),
        CardinalityV1::Bag,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(str::to_owned)?;
    let scan = TypedRelationExprV2::scan(key(3), location.clone(), scan_source, scan_output)
        .map_err(str::to_owned)?;
    let aggregate_effects = ExpressionEffectsV2 {
        aggregation: AggregationClassV1::Aggregate,
        ..ExpressionEffectsV2::pure_scalar()
    };
    // Implicit empty-partition group: sealed from the input stage alone.
    let group = TypedRelationExprV2::implicit_group(key(10), location.clone(), scan.clone())
        .map_err(str::to_owned)?;
    assert!(group.expressions().is_empty());
    assert!(group.partition_input().is_empty());
    assert_eq!(group.output().cardinality(), CardinalityV1::Set);
    assert!(group.output().partition_keys().is_empty());
    assert!(group.output().columns().is_empty());
    assert_eq!(
        group.output().provenance(),
        scan.output().provenance(),
        "pre-fold provenance is inherited exactly"
    );
    // Foreign source key and cross-registry location both reject.
    assert!(TypedRelationExprV2::implicit_group(
        SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(99),
            node_id: HirNodeIdV1::from_raw(10),
        },
        location.clone(),
        scan.clone(),
    )
    .is_err());
    assert!(
        TypedRelationExprV2::implicit_group(key(10), other_location.clone(), scan.clone(),)
            .is_err()
    );
    // A forged implicit Group with authored-shape keys cannot be built:
    // the stage constructor rejects nonempty keys under the implicit flag
    // and nonempty partition input.
    let forged_implicit = TypedRelationStageInputV2 {
        key: key(10),
        location: location.clone(),
        kind: TypedRelationStageKindV2::Group,
        inputs: vec![scan.clone()],
        expressions: vec![key(11)],
        predicate: None,
        source: None,
        join_mode: None,
        implicit_partition: true,
        partition_input: Vec::new(),
        output: group.output().clone(),
    };
    assert!(TypedRelationExprV2::try_from_stage(forged_implicit).is_err());
    // Scalar aggregate output: one expression column, empty partition.
    let total_field = RecordFieldTypeV1::try_new("total", ValueTypeV1::int(), location.clone())
        .map_err(|error| format!("{error:?}"))?;
    let scalar_row =
        RecordTypeV1::try_new(vec![total_field]).map_err(|error| format!("{error:?}"))?;
    let scalar_columns = vec![OutputColumnContractV2::new(
        "total".to_owned(),
        OutputColumnOriginV2::Expression(key(20)),
        location.clone(),
    )];
    let scalar_output = RelationOutputContractV2::try_aggregate(
        scalar_row.clone(),
        group.output().provenance().clone(),
        scalar_columns.clone(),
        Vec::new(),
        aggregate_effects,
        &[],
    )
    .map_err(str::to_owned)?;
    let aggregate = TypedRelationExprV2::try_aggregate_project(
        key(30),
        location.clone(),
        group.clone(),
        vec![key(20)],
        scalar_output.clone(),
    )
    .map_err(str::to_owned)?;
    assert_eq!(aggregate.output().cardinality(), CardinalityV1::Set);
    // Wrong Group child: the aggregate projection rejects a non-Group.
    assert_eq!(
        TypedRelationExprV2::try_aggregate_project(
            key(30),
            location.clone(),
            scan.clone(),
            vec![key(20)],
            scalar_output.clone(),
        )
        .err(),
        Some("aggregate projection must consume exactly one Group stage")
    );
    // Transplanted expression ID: column origin is a foreign expression.
    let transplanted_columns = vec![OutputColumnContractV2::new(
        "total".to_owned(),
        OutputColumnOriginV2::Expression(key(77)),
        location.clone(),
    )];
    let transplanted = RelationOutputContractV2::try_aggregate(
        scalar_row.clone(),
        group.output().provenance().clone(),
        transplanted_columns,
        Vec::new(),
        aggregate_effects,
        &[],
    )
    .map_err(str::to_owned)?;
    assert_eq!(
        TypedRelationExprV2::try_aggregate_project(
            key(30),
            location.clone(),
            group.clone(),
            vec![key(20)],
            transplanted,
        )
        .err(),
        Some("aggregate projection origins are not row-aligned with its expressions")
    );
    // Cardinality downgrade: a Bag aggregate output rejects.
    let downgraded = RelationOutputContractV2::try_new(
        scalar_row.clone(),
        group.output().provenance().clone(),
        scalar_columns.clone(),
        CardinalityV1::Bag,
        Vec::new(),
        Vec::new(),
        aggregate_effects,
    )
    .map_err(str::to_owned)?;
    assert_eq!(
        TypedRelationExprV2::try_aggregate_project(
            key(30),
            location.clone(),
            group.clone(),
            vec![key(20)],
            downgraded,
        )
        .err(),
        Some("aggregate projection output loses its Group partition identity")
    );
    // Authored group for the grouped-aggregate forgeries.
    let id_field = RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), location.clone())
        .map_err(|error| format!("{error:?}"))?;
    let key_row = RecordTypeV1::try_new(vec![id_field]).map_err(|error| format!("{error:?}"))?;
    let key_columns = vec![OutputColumnContractV2::new(
        "id".to_owned(),
        OutputColumnOriginV2::Expression(key(11)),
        location.clone(),
    )];
    let key_output = RelationOutputContractV2::try_new(
        key_row,
        scan.output().provenance().clone(),
        key_columns,
        CardinalityV1::Set,
        vec![key(11)],
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(str::to_owned)?;
    let authored = TypedRelationExprV2::try_group(
        key(40),
        location.clone(),
        scan.clone(),
        vec![key(11)],
        Vec::new(),
        key_output,
    )
    .map_err(str::to_owned)?;
    // Grouped aggregate output retains the authored partition identity.
    let grouped_output = RelationOutputContractV2::try_aggregate(
        scalar_row.clone(),
        authored.output().provenance().clone(),
        scalar_columns.clone(),
        vec![key(11)],
        aggregate_effects,
        authored.output().partition_keys(),
    )
    .map_err(str::to_owned)?;
    TypedRelationExprV2::try_aggregate_project(
        key(50),
        location.clone(),
        authored.clone(),
        vec![key(20)],
        grouped_output.clone(),
    )
    .map_err(str::to_owned)?;
    // Missing partition key.
    assert_eq!(
        RelationOutputContractV2::try_aggregate(
            scalar_row.clone(),
            authored.output().provenance().clone(),
            scalar_columns.clone(),
            Vec::new(),
            aggregate_effects,
            authored.output().partition_keys(),
        )
        .err(),
        Some("aggregate output partition identity differs from its Group authority")
    );
    // Extra partition key.
    assert_eq!(
        RelationOutputContractV2::try_aggregate(
            scalar_row.clone(),
            authored.output().provenance().clone(),
            scalar_columns.clone(),
            vec![key(11), key(12)],
            aggregate_effects,
            authored.output().partition_keys(),
        )
        .err(),
        Some("aggregate output partition identity differs from its Group authority")
    );
    // Swapped partition keys (two authored keys, swapped order).
    let second_field = RecordFieldTypeV1::try_new("id2", ValueTypeV1::int(), location.clone())
        .map_err(|error| format!("{error:?}"))?;
    let two_key_row = RecordTypeV1::try_new(vec![
        RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), location.clone())
            .map_err(|error| format!("{error:?}"))?,
        second_field,
    ])
    .map_err(|error| format!("{error:?}"))?;
    let two_key_columns = vec![
        OutputColumnContractV2::new(
            "id".to_owned(),
            OutputColumnOriginV2::Expression(key(11)),
            location.clone(),
        ),
        OutputColumnContractV2::new(
            "id2".to_owned(),
            OutputColumnOriginV2::Expression(key(12)),
            location.clone(),
        ),
    ];
    let two_key_output = RelationOutputContractV2::try_new(
        two_key_row,
        scan.output().provenance().clone(),
        two_key_columns,
        CardinalityV1::Set,
        vec![key(11), key(12)],
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(str::to_owned)?;
    let two_key_group = TypedRelationExprV2::try_group(
        key(41),
        location.clone(),
        scan.clone(),
        vec![key(11), key(12)],
        Vec::new(),
        two_key_output,
    )
    .map_err(str::to_owned)?;
    assert_eq!(
        RelationOutputContractV2::try_aggregate(
            scalar_row.clone(),
            two_key_group.output().provenance().clone(),
            scalar_columns.clone(),
            vec![key(12), key(11)],
            aggregate_effects,
            two_key_group.output().partition_keys(),
        )
        .err(),
        Some("aggregate output partition identity differs from its Group authority")
    );
    // Wrong scan occurrence: a SourceField origin cannot enter an
    // aggregate output column.
    let scan_field_columns = vec![OutputColumnContractV2::new(
        "total".to_owned(),
        OutputColumnOriginV2::SourceField {
            source: match scan.source() {
                Some(source) => source.clone(),
                None => return Err("scan source expected".to_owned()),
            },
            scan_stage: key(3),
            field_name: "total".to_owned(),
        },
        location.clone(),
    )];
    assert!(RelationOutputContractV2::try_aggregate(
        scalar_row,
        two_key_group.output().provenance().clone(),
        scan_field_columns,
        Vec::new(),
        aggregate_effects,
        &[],
    )
    .is_err());
    // Scalar effects cannot masquerade as an aggregate output.
    assert_eq!(
        RelationOutputContractV2::try_aggregate(
            RecordTypeV1::try_new(Vec::new()).map_err(|error| format!("{error:?}"))?,
            two_key_group.output().provenance().clone(),
            Vec::new(),
            Vec::new(),
            ExpressionEffectsV2::pure_scalar(),
            &[],
        )
        .err(),
        Some("aggregate output effects must retain the aggregate class")
    );
    Ok(())
}
