//! Independent schema-8 fixtures from docs/contracts/frontend-execution-v3.md.
//!
//! Expected bytes use only fixed-width primitives and literal tags. They never
//! read DTO fields or call production tag/writer helpers. These synthetic DTOs
//! exercise encoding, not source acceptance or downstream runtime admission.

use super::*;
use quanta_gqlang_v2_sema::HirNodeIdV1;

fn n(value: u32) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}
fn count(value: u64) -> Vec<u8> {
    value.to_be_bytes().to_vec()
}
#[expect(
    clippy::expect_used,
    reason = "bounded fixture lengths must fit the wire width"
)]
fn length(value: usize) -> Vec<u8> {
    count(u64::try_from(value).expect("fixture length fits u64"))
}
fn text(value: &str) -> Vec<u8> {
    [length(value.len()), value.as_bytes().to_vec()].concat()
}
fn seq(values: &[Vec<u8>]) -> Vec<u8> {
    [length(values.len()), values.concat()].concat()
}
fn key(node: u32) -> SourceQualifiedNodeIdV2 {
    SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(17),
        node_id: HirNodeIdV1::from_raw(node),
    }
}
fn id(node: u32) -> SemanticItemIdV2 {
    SemanticItemIdV2::new(SourceIdV2::from_raw(17), HirNodeIdV1::from_raw(node))
}
fn wire_key(node: u32) -> Vec<u8> {
    [n(17), n(node)].concat()
}
fn binding() -> ExecutionBindingV3 {
    ExecutionBindingV3 {
        source_id: SourceIdV2::from_raw(17),
        owner_node: 23,
        ordinal: 2,
        name: "résult".to_owned(),
    }
}
fn wire_binding() -> Vec<u8> {
    [n(17), n(23), count(2), text("résult")].concat()
}
fn operator() -> ExecutionOperatorV3 {
    ExecutionOperatorV3 {
        symbol: "!=".to_owned(),
        overload_id: 321,
    }
}
fn wire_operator() -> Vec<u8> {
    [text("!="), n(321)].concat()
}

fn type_cases() -> Vec<(ExecutionValueTypeV3, Vec<u8>)> {
    use ExecutionScalarTypeV3 as S;
    use ExecutionValueTypeV3 as T;
    let mut cases = vec![
        (T::Null, vec![1]),
        (
            T::Tuple(Box::new([T::Null, T::Unknown])),
            [vec![2], seq(&[vec![1], vec![7]])].concat(),
        ),
        (
            T::Record(Box::new([ExecutionRecordFieldV3 {
                name: "é".to_owned(),
                value_type: T::Null,
            }])),
            [vec![3], seq(&[[text("é"), vec![1]].concat()])].concat(),
        ),
        (T::Optional(Box::new(T::Scalar(S::Int))), vec![4, 0, 1]),
        (
            T::Collection {
                element: Box::new(T::Null),
                cardinality: ExecutionCardinalityV3::Bag,
            },
            vec![5, 1, 1],
        ),
        (
            T::Union(Box::new([ExecutionUnionVariantV3 {
                tag: "none".to_owned(),
                value_type: T::Null,
            }])),
            [vec![6], seq(&[[text("none"), vec![1]].concat()])].concat(),
        ),
        (T::Unknown, vec![7]),
    ];
    for (scalar, tag) in [
        (S::Bool, 0),
        (S::Int, 1),
        (S::Float, 2),
        (S::String, 3),
        (S::Duration, 4),
        (S::Location, 5),
        (S::Path, 6),
        (S::RepoPath, 7),
        (S::EntityRef, 8),
        (S::SymbolRef, 9),
        (S::RuleRef, 10),
        (S::SourceScope, 11),
        (S::ProofTree, 12),
    ] {
        cases.push((T::Scalar(scalar), vec![0, tag]));
    }
    cases
}

fn expression_cases() -> Vec<(ExecutionExpressionShapeV3, Vec<u8>)> {
    use ExecutionExpressionShapeV3 as E;
    use ExecutionLiteralV3 as L;
    let mut cases = vec![
        (
            E::Literal(L::Int(-257)),
            [vec![0, 0], (-257i128).to_be_bytes().to_vec()].concat(),
        ),
        (
            E::Literal(L::Float(ExecutionFiniteFloatV3(1.5f64.to_bits()))),
            vec![0, 1, 0x3f, 0xf8, 0, 0, 0, 0, 0, 0],
        ),
        (E::Literal(L::Bool(false)), vec![0, 2, 0]),
        (E::Literal(L::Bool(true)), vec![0, 2, 1]),
        (
            E::Literal(L::String("é\0".to_owned())),
            [vec![0, 3], text("é\0")].concat(),
        ),
        (E::Literal(L::Null), vec![0, 4]),
        (E::Binding(binding()), [vec![1], wire_binding()].concat()),
        (
            E::Path {
                base: key(41),
                segments: Box::new(["a".to_owned(), "é".to_owned()]),
            },
            [vec![2], wire_key(41), seq(&[text("a"), text("é")])].concat(),
        ),
        (
            E::Unary {
                operator: operator(),
                operand: key(41),
            },
            [vec![3], wire_operator(), wire_key(41)].concat(),
        ),
        (
            E::Binary {
                operator: operator(),
                lhs: key(41),
                rhs: key(42),
            },
            [vec![4], wire_operator(), wire_key(41), wire_key(42)].concat(),
        ),
        (E::Paren(key(41)), [vec![6], wire_key(41)].concat()),
        (
            E::Record(Box::new([
                ("z".to_owned(), key(42)),
                ("a".to_owned(), key(41)),
            ])),
            [
                vec![7],
                seq(&[
                    [text("z"), wire_key(42)].concat(),
                    [text("a"), wire_key(41)].concat(),
                ]),
            ]
            .concat(),
        ),
        (
            E::List(Box::new([key(42), key(41)])),
            [vec![8], seq(&[wire_key(42), wire_key(41)])].concat(),
        ),
        (E::BoolValue(key(41)), [vec![9], wire_key(41)].concat()),
        (
            E::Compare {
                operator: operator(),
                lhs: key(41),
                rhs: key(42),
            },
            [vec![10], wire_operator(), wire_key(41), wire_key(42)].concat(),
        ),
        (
            E::And {
                operator: operator(),
                lhs: key(41),
                rhs: key(42),
            },
            [vec![11], wire_operator(), wire_key(41), wire_key(42)].concat(),
        ),
        (
            E::Or {
                operator: operator(),
                lhs: key(41),
                rhs: key(42),
            },
            [vec![12], wire_operator(), wire_key(41), wire_key(42)].concat(),
        ),
        (
            E::Not {
                operator: operator(),
                operand: key(41),
            },
            [vec![13], wire_operator(), wire_key(41)].concat(),
        ),
    ];
    for (target, target_bytes) in [
        (
            ExecutionCallTargetV3::Builtin {
                name: "sum".to_owned(),
            },
            [vec![0], text("sum")].concat(),
        ),
        (
            ExecutionCallTargetV3::Predicate {
                name: "p".to_owned(),
            },
            [vec![1], text("p")].concat(),
        ),
        (
            ExecutionCallTargetV3::HostPredicate {
                fingerprint: "host:v1".to_owned(),
            },
            [vec![2], text("host:v1")].concat(),
        ),
    ] {
        cases.push((
            E::Call {
                target,
                arguments: Box::new([Some(key(41)), None, Some(key(42))]),
            },
            [
                vec![5],
                target_bytes,
                seq(&[
                    [vec![1], wire_key(41)].concat(),
                    vec![0],
                    [vec![1], wire_key(42)].concat(),
                ]),
            ]
            .concat(),
        ));
    }
    for (kind, tag) in [
        (ExecutionLogicalQuantifierV3::Exists, 0),
        (ExecutionLogicalQuantifierV3::Forall, 1),
    ] {
        cases.push((
            E::LogicalQuantifier {
                kind,
                binders: Box::new([binding()]),
                body: key(42),
            },
            [vec![14, tag], seq(&[wire_binding()]), wire_key(42)].concat(),
        ));
    }
    for (polarity, tag) in [
        (ExecutionExistsPolarityV3::Exists, 0),
        (ExecutionExistsPolarityV3::NotExists, 1),
    ] {
        cases.push((
            E::RelationExists {
                polarity,
                relation_root: key(43),
                correlations: Box::new([ExecutionCorrelationV3 {
                    binding: binding(),
                    reference: key(42),
                }]),
            },
            [
                vec![15, tag],
                wire_key(43),
                seq(&[[wire_binding(), wire_key(42)].concat()]),
            ]
            .concat(),
        ));
    }
    cases
}

fn schema() -> (ExecutionSchemaV3, Vec<u8>) {
    let (fields, field_bytes): (Vec<_>, Vec<_>) = type_cases()
        .into_iter()
        .enumerate()
        .map(|(i, (value_type, bytes))| {
            let name = format!("field{i}");
            (
                ExecutionFieldV3 {
                    name: name.clone(),
                    value_type,
                },
                [text(&name), bytes].concat(),
            )
        })
        .unzip();
    let value_type = ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int);
    (
        ExecutionSchemaV3 {
            fields: fields.into_boxed_slice(),
            cardinality: ExecutionCardinalityV3::Sequence,
            partition_keys: Box::new([key(42), key(41)]),
            order: Box::new([
                ExecutionOrderKeyV3 {
                    expression: key(42),
                    direction: ExecutionSortDirectionV3::Desc,
                    null_order: ExecutionNullOrderV3::First,
                    value_type: value_type.clone(),
                },
                ExecutionOrderKeyV3 {
                    expression: key(41),
                    direction: ExecutionSortDirectionV3::Asc,
                    null_order: ExecutionNullOrderV3::Last,
                    value_type,
                },
            ]),
        },
        [
            seq(&field_bytes),
            vec![2],
            seq(&[wire_key(42), wire_key(41)]),
            seq(&[
                [wire_key(42), vec![1, 0, 0, 1]].concat(),
                [wire_key(41), vec![0, 1, 0, 1]].concat(),
            ]),
        ]
        .concat(),
    )
}

fn relation() -> (ExecutionRelationV3, Vec<u8>) {
    let (schema, schema_bytes) = schema();
    let empty_schema = ExecutionSchemaV3 {
        fields: Box::new([]),
        cardinality: ExecutionCardinalityV3::Set,
        partition_keys: Box::new([]),
        order: Box::new([]),
    };
    let children = [61, 62].map(|node| ExecutionRelationalOperatorV3 {
        family: AlgebraFamilyV2::Scan,
        join_mode: None,
        stage: key(node),
        source: None,
        schema: empty_schema.clone(),
        predicates: Box::new([]),
        expressions: Box::new([]),
        correlations: Box::new([]),
        existential_terms: Box::new([]),
        partition_input: Box::new([]),
        children: Box::new([]),
    });
    let child_bytes = [61, 62].map(|node| {
        [
            vec![0, 0],
            wire_key(node),
            vec![0],
            count(0),
            vec![0],
            count(0),
            count(0),
            count(0),
            count(0),
            count(0),
            count(0),
            count(0),
        ]
        .concat()
    });
    let root = ExecutionRelationalOperatorV3 {
        family: AlgebraFamilyV2::Join,
        join_mode: Some(ExecutionJoinModeV3::Cross),
        stage: key(60),
        source: Some(ExecutionSourceV3 {
            composition_binding: None,
            authority: ExecutionRelationAuthorityV3::CatalogView,
            kind: ExecutionSourceKindV3::View,
            namespace: Some("库".to_owned()),
            name: "rows".to_owned(),
        }),
        schema,
        predicates: Box::new([key(42), key(41)]),
        expressions: Box::new([key(43)]),
        correlations: Box::new([ExecutionCorrelationV3 {
            binding: binding(),
            reference: key(42),
        }]),
        existential_terms: Box::new([]),
        partition_input: Box::new([binding()]),
        children: Box::new(children),
    };
    let root_bytes = [
        vec![3, 1, 1],
        wire_key(60),
        vec![1, 2, 1, 1],
        text("库"),
        text("rows"),
        schema_bytes,
        seq(&[wire_key(42), wire_key(41)]),
        seq(&[wire_key(43)]),
        seq(&[[wire_binding(), wire_key(42)].concat()]),
        seq(&[wire_binding()]),
        seq(&child_bytes),
    ]
    .concat();
    let (expressions, expression_bytes): (Vec<_>, Vec<_>) = expression_cases()
        .into_iter()
        .zip(100u32..)
        .map(|((shape, bytes), node)| {
            (
                ExecutionExpressionV3 {
                    id: key(node),
                    value_type: ExecutionValueTypeV3::Unknown,
                    shape,
                },
                [wire_key(node), vec![7], bytes].concat(),
            )
        })
        .unzip();
    (
        ExecutionRelationV3 {
            item_id: id(31),
            root,
            expressions: expressions.into_boxed_slice(),
        },
        [vec![1], wire_key(31), root_bytes, seq(&expression_bytes)].concat(),
    )
}

fn graph() -> (ExecutionGraphQueryV3, Vec<u8>) {
    let target = ExecutionGraphTargetV3::Host {
        kind: ExecutionGraphReferenceKindV3::Snapshot,
        spelling: "repo@é".to_owned(),
        entity_subtype: Some("function".to_owned()),
    };
    let host_bytes = [vec![0, 7], text("repo@é"), vec![1], text("function")].concat();
    let local = ExecutionGraphTargetV3::Composition {
        binding: binding(),
        capability: ExecutionGraphCapabilityV3::Scope,
    };
    let local_bytes = [vec![1], wire_binding(), vec![1]].concat();
    (
        ExecutionGraphQueryV3 {
            kind: ExecutionGraphKindV3::Summary,
            references: Box::new([
                ExecutionGraphReferenceV3 {
                    role: ExecutionGraphReferenceRoleV3::Source,
                    target,
                },
                ExecutionGraphReferenceV3 {
                    role: ExecutionGraphReferenceRoleV3::Scope,
                    target: local,
                },
            ]),
            depth: 7,
            direction: Some(ExecutionGraphDirectionV3::Undirected),
            summary_facets: Box::new(["types".to_owned(), "calls".to_owned()]),
            relation_shape: ExecutionGraphRelationShapeV3::Acyclic,
            dedup_identity: ExecutionGraphDedupIdentityV3::NodeSet,
            max_witnesses: 11,
            max_path_length: Some(13),
            witness_tie_break: ExecutionGraphWitnessTieBreakV3::Lexical,
        },
        [
            vec![6],
            seq(&[
                [vec![0], host_bytes].concat(),
                [vec![9], local_bytes].concat(),
            ]),
            n(7),
            vec![1, 3],
            seq(&[text("types"), text("calls")]),
            vec![0, 2],
            n(11),
            vec![1],
            n(13),
            vec![1],
        ]
        .concat(),
    )
}

fn plan() -> ExecutionPlanV3 {
    ExecutionPlanV3 {
        authority: ExecutionAuthorityV3::FrontendChecked,
        determinism: ExecutionDeterminismV3::NonDeterministic,
        evidence: ExecutionEvidenceV3::WitnessPayload,
        effect: ExecutionEffectV3::ProofEvidenceOnly,
        policy_family: ExecutionPolicyFamilyV3::Composition,
    }
}

fn assert_document(items: Vec<ExecutionItemV3>, expected_items: &[Vec<u8>]) {
    let source = "digest fixture é";
    let source_bytes = [text("qglang.execution.source-bytes.v3"), n(3), text(source)].concat();
    // Every non-command fixture explicitly has the V8 command-absence tag.
    let expected_items: Vec<Vec<u8>> = expected_items
        .iter()
        .map(|item| [item.as_slice(), &[0]].concat())
        .collect();
    let contract_bytes = [
        text("qglang.execution.contract.v8"),
        n(8),
        n(17),
        seq(&expected_items),
    ]
    .concat();
    let source_hash = Sha256::digest(&source_bytes);
    let contract_hash = Sha256::digest(&contract_bytes);
    let qualification_bytes = [
        text("qglang.execution.qualification.v8"),
        n(8),
        n(17),
        source_hash.to_vec(),
        contract_hash.to_vec(),
    ]
    .concat();
    let document = ExecutionDocumentV3::new(SourceIdV2::from_raw(17), source.as_bytes(), items);
    assert_eq!(
        document.source_digest().as_bytes().as_slice(),
        source_hash.as_slice()
    );
    assert_eq!(
        document.contract_digest().as_bytes().as_slice(),
        contract_hash.as_slice()
    );
    assert_eq!(
        document.qualification_digest().as_bytes().as_slice(),
        Sha256::digest(&qualification_bytes).as_slice()
    );
}

#[test]
fn rename_apply_has_independent_exact_target_and_token_bytes() {
    let command = ExecutionRenameCommandV3 {
        target: ExecutionSymbolTargetV3 {
            selector: "s".to_owned(),
            repo_path: "src/a.py".to_owned(),
            symbol_id: "def:17".to_owned(),
            start_byte: 17,
            end_byte: 20,
        },
        new_name: "new".to_owned(),
        phase: ExecutionCommandPhaseV3::Apply,
        expected_snapshot: "snap-7".to_owned(),
        preview_token: Some("issued-1".to_owned()),
    };
    let item = ExecutionItemV3 {
        item_id: id(71),
        family: ExecutionFamilyV3::Command,
        relation: None,
        composition: None,
        graph: None,
        check: None,
        presentation: None,
        command: Some(command),
        plan: ExecutionPlanV3 {
            authority: ExecutionAuthorityV3::FrontendChecked,
            determinism: ExecutionDeterminismV3::Deterministic,
            evidence: ExecutionEvidenceV3::None,
            effect: ExecutionEffectV3::MutationRequiresGrant,
            policy_family: ExecutionPolicyFamilyV3::Command,
        },
    };
    let expected_command = [
        vec![1],
        text("s"),
        text("src/a.py"),
        text("def:17"),
        count(17),
        count(20),
        text("new"),
        text("snap-7"),
        vec![1],
        text("issued-1"),
    ]
    .concat();
    let expected_item = [
        wire_key(71),
        // family, authority, determinism, evidence, effect, policy;
        // five preceding payloads absent, command present.
        vec![7, 0, 0, 0, 4, 7, 0, 0, 0, 0, 0, 1],
        expected_command,
    ]
    .concat();
    let expected_contract = [
        text("qglang.execution.contract.v8"),
        n(8),
        n(17),
        seq(&[expected_item]),
    ]
    .concat();
    let document = ExecutionDocumentV3::new(SourceIdV2::from_raw(17), b"rename", vec![item]);
    assert_eq!(
        document.contract_digest().as_bytes().as_slice(),
        Sha256::digest(expected_contract).as_slice()
    );
}

#[test]
fn whole_relation_stream_has_independent_expected_bytes() {
    let (relation, bytes) = relation();
    let item = ExecutionItemV3 {
        item_id: id(31),
        family: ExecutionFamilyV3::Relation,
        relation: Some(relation),
        graph: None,
        check: None,
        presentation: None,
        composition: None,
        command: None,
        plan: plan(),
    };
    assert_document(
        vec![item],
        &[[
            wire_key(31),
            vec![0, 0, 1, 2, 1, 6],
            bytes,
            vec![0, 0, 0, 0],
        ]
        .concat()],
    );
}

#[test]
fn flat_exists_node_encodes_each_condition_before_its_ordered_children() {
    let empty_schema = ExecutionSchemaV3 {
        fields: Box::new([]),
        cardinality: ExecutionCardinalityV3::Set,
        partition_keys: Box::new([]),
        order: Box::new([]),
    };
    let scans = [60, 61, 62].map(|stage| ExecutionRelationalOperatorV3 {
        family: AlgebraFamilyV2::Scan,
        join_mode: None,
        stage: key(stage),
        source: None,
        schema: empty_schema.clone(),
        predicates: Box::new([]),
        expressions: Box::new([]),
        correlations: Box::new([]),
        existential_terms: Box::new([]),
        partition_input: Box::new([]),
        children: Box::new([]),
    });
    let node = ExecutionRelationalOperatorV3 {
        family: AlgebraFamilyV2::ExistsFilter,
        join_mode: None,
        stage: key(70),
        source: None,
        schema: empty_schema,
        predicates: Box::new([]),
        expressions: Box::new([]),
        correlations: Box::new([]),
        existential_terms: Box::new([
            ExecutionExistentialTermV3 {
                expression: key(71),
                polarity: ExecutionExistsPolarityV3::Exists,
                correlations: Box::new([]),
            },
            ExecutionExistentialTermV3 {
                expression: key(72),
                polarity: ExecutionExistsPolarityV3::NotExists,
                correlations: Box::new([ExecutionCorrelationV3 {
                    binding: binding(),
                    reference: key(73),
                }]),
            },
        ]),
        partition_input: Box::new([]),
        children: Box::new(scans),
    };
    let schema_bytes = [count(0), vec![0], count(0), count(0)].concat();
    let scan_bytes = [60, 61, 62].map(|stage| {
        [
            vec![0, 0],
            wire_key(stage),
            vec![0],
            schema_bytes.clone(),
            count(0),
            count(0),
            count(0),
            count(0),
            count(0),
        ]
        .concat()
    });
    let terms = [
        [wire_key(71), vec![0], count(0)].concat(),
        [
            wire_key(72),
            vec![1],
            seq(&[[wire_binding(), wire_key(73)].concat()]),
        ]
        .concat(),
    ];
    let expected = [
        vec![9, 0],
        wire_key(70),
        vec![0],
        schema_bytes,
        count(0),
        count(0),
        count(0),
        seq(&terms),
        count(0),
        seq(&scan_bytes),
    ]
    .concat();
    let mut actual = Vec::new();
    put_node(&mut actual, &node);
    assert_eq!(actual, expected);

    let expressions = [
        ExecutionExpressionV3 {
            id: key(71),
            value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Bool),
            shape: ExecutionExpressionShapeV3::RelationExists {
                polarity: ExecutionExistsPolarityV3::Exists,
                relation_root: key(61),
                correlations: Box::new([]),
            },
        },
        ExecutionExpressionV3 {
            id: key(72),
            value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Bool),
            shape: ExecutionExpressionShapeV3::RelationExists {
                polarity: ExecutionExistsPolarityV3::NotExists,
                relation_root: key(62),
                correlations: Box::new([ExecutionCorrelationV3 {
                    binding: binding(),
                    reference: key(73),
                }]),
            },
        },
        ExecutionExpressionV3 {
            id: key(73),
            value_type: ExecutionValueTypeV3::Unknown,
            shape: ExecutionExpressionShapeV3::Binding(binding()),
        },
    ];
    let expression_bytes = [
        [wire_key(71), vec![0, 0, 15, 0], wire_key(61), count(0)].concat(),
        [
            wire_key(72),
            vec![0, 0, 15, 1],
            wire_key(62),
            seq(&[[wire_binding(), wire_key(73)].concat()]),
        ]
        .concat(),
        [wire_key(73), vec![7, 1], wire_binding()].concat(),
    ];
    let relation_bytes = [vec![1], wire_key(31), expected, seq(&expression_bytes)].concat();
    assert_document(
        vec![ExecutionItemV3 {
            item_id: id(31),
            family: ExecutionFamilyV3::Relation,
            relation: Some(ExecutionRelationV3 {
                item_id: id(31),
                root: node,
                expressions: Box::new(expressions),
            }),
            graph: None,
            check: None,
            presentation: None,
            composition: None,
            command: None,
            plan: plan(),
        }],
        &[[
            wire_key(31),
            vec![0, 0, 1, 2, 1, 6],
            relation_bytes,
            vec![0, 0, 0, 0],
        ]
        .concat()],
    );
}

#[test]
fn whole_composition_stream_has_independent_expected_bytes() {
    let (relation, relation_bytes) = relation();
    let (schema, schema_bytes) = schema();
    let (graph, graph_bytes) = graph();
    let mut producers = vec![
        (ExecutionCompositionProducerV3::Relation, vec![0]),
        (
            ExecutionCompositionProducerV3::Graph {
                query: graph.clone(),
            },
            [vec![2], graph_bytes.clone()].concat(),
        ),
    ];
    for (mode, tag, scope, scope_bytes) in [
        (ExecutionSampleModeV3::Lexical, 0, None, vec![0]),
        (
            ExecutionSampleModeV3::Semantic,
            1,
            Some(ExecutionGraphTargetV3::Host {
                kind: ExecutionGraphReferenceKindV3::Scope,
                spelling: "workspace".to_owned(),
                entity_subtype: None,
            }),
            [vec![1, 0, 2], text("workspace"), vec![0]].concat(),
        ),
        (
            ExecutionSampleModeV3::Hybrid,
            2,
            Some(ExecutionGraphTargetV3::Composition {
                binding: binding(),
                capability: ExecutionGraphCapabilityV3::Entity,
            }),
            [vec![1, 1], wire_binding(), vec![0]].concat(),
        ),
    ] {
        producers.push((
            ExecutionCompositionProducerV3::Sample {
                query_text: "é".to_owned(),
                mode,
                limit: 257,
                scope,
            },
            [vec![1], text("é"), vec![tag], n(257), scope_bytes].concat(),
        ));
    }
    let (mut bindings, mut bindings_bytes): (Vec<_>, Vec<_>) = producers
        .into_iter()
        .zip(70u32..)
        .map(|((producer, bytes), node)| {
            (
                ExecutionCompositionBindingV3 {
                    binding: binding(),
                    producer_item_id: id(node),
                    producer_family: ExecutionFamilyV3::Relation,
                    schema: Some(schema.clone()),
                    relation: Some(relation.clone()),
                    producer,
                },
                [
                    wire_binding(),
                    wire_key(node),
                    text("relation"),
                    vec![1],
                    schema_bytes.clone(),
                    relation_bytes.clone(),
                    bytes,
                ]
                .concat(),
            )
        })
        .unzip();
    // Pin the absent nested options beside the populated bindings.
    bindings.push(ExecutionCompositionBindingV3 {
        binding: binding(),
        producer_item_id: id(75),
        producer_family: ExecutionFamilyV3::Graph,
        schema: None,
        relation: None,
        producer: ExecutionCompositionProducerV3::Relation,
    });
    bindings_bytes.push([wire_binding(), wire_key(75), text("graph"), vec![0, 0, 0]].concat());
    let mut items = Vec::new();
    let mut expected = Vec::new();
    for ((body, body_bytes), node) in [
        (ExecutionCompositionBodyV3::Relation, vec![0]),
        (
            ExecutionCompositionBodyV3::Graph(graph),
            [vec![1], graph_bytes].concat(),
        ),
        (ExecutionCompositionBodyV3::Check, vec![2]),
        (ExecutionCompositionBodyV3::Explain, vec![3]),
    ]
    .into_iter()
    .zip(80u32..)
    {
        let composition = ExecutionCompositionV3 {
            bindings: bindings.clone().into_boxed_slice(),
            with_binding: binding(),
            body_item_id: id(31),
            body_family: ExecutionFamilyV3::Relation,
            body_relation: Some(relation.clone()),
            body,
        };
        items.push(ExecutionItemV3 {
            item_id: id(node),
            family: ExecutionFamilyV3::Composition,
            relation: None,
            graph: None,
            check: None,
            presentation: None,
            composition: Some(composition),
            command: None,
            plan: plan(),
        });
        expected.push(
            [
                wire_key(node),
                vec![4, 0, 1, 2, 1, 6, 0, 1],
                seq(&bindings_bytes),
                wire_binding(),
                wire_key(31),
                text("relation"),
                relation_bytes.clone(),
                body_bytes,
                vec![0, 0, 0],
            ]
            .concat(),
        );
    }
    // Empty bindings and an absent body relation must each keep their own marker.
    items.push(ExecutionItemV3 {
        item_id: id(84),
        family: ExecutionFamilyV3::Composition,
        relation: None,
        graph: None,
        check: None,
        presentation: None,
        composition: Some(ExecutionCompositionV3 {
            bindings: Box::new([]),
            with_binding: binding(),
            body_item_id: id(32),
            body_family: ExecutionFamilyV3::Check,
            body_relation: None,
            body: ExecutionCompositionBodyV3::Check,
        }),
        command: None,
        plan: plan(),
    });
    expected.push(
        [
            wire_key(84),
            vec![4, 0, 1, 2, 1, 6, 0, 1],
            count(0),
            wire_binding(),
            wire_key(32),
            text("check"),
            vec![0, 2, 0, 0, 0],
        ]
        .concat(),
    );
    assert_document(items, &expected);
}

#[test]
fn standalone_graph_check_and_all_presentation_verbs_have_independent_bytes() {
    let (graph, graph_bytes) = graph();
    let host = |kind, spelling: &str| ExecutionGraphTargetV3::Host {
        kind,
        spelling: spelling.to_owned(),
        entity_subtype: None,
    };
    let host_bytes = |kind: u8, spelling: &str| [vec![0, kind], text(spelling), vec![0]].concat();
    let mut items = Vec::new();
    let mut expected = Vec::new();
    items.push(ExecutionItemV3 {
        item_id: id(40),
        family: ExecutionFamilyV3::Graph,
        relation: None,
        composition: None,
        graph: Some(graph.clone()),
        check: None,
        presentation: None,
        command: None,
        plan: plan(),
    });
    expected.push(
        [
            wire_key(40),
            vec![1, 0, 1, 2, 1, 6, 0, 0, 1],
            graph_bytes.clone(),
            vec![0, 0],
        ]
        .concat(),
    );

    items.push(ExecutionItemV3 {
        item_id: id(41),
        family: ExecutionFamilyV3::Check,
        relation: None,
        composition: None,
        graph: None,
        check: Some(ExecutionCheckV3 {
            rule: ExecutionCatalogIdentityV3 {
                module: "app".to_owned(),
                name: "rule".to_owned(),
            },
            scope: Some(host(ExecutionGraphReferenceKindV3::Scope, "repo")),
            pattern: None,
            nested_graph: Some(ExecutionNestedGraphV3 {
                item_id: id(31),
                query: graph,
            }),
            confirm: ExecutionConfirmModeV3::Require,
            emit: ExecutionEmitModeV3::Sarif,
            mode: ExecutionDetectionModeV3::SemanticOnly,
        }),
        presentation: None,
        command: None,
        plan: plan(),
    });
    expected.push(
        [
            wire_key(41),
            vec![2, 0, 1, 2, 1, 6, 0, 0, 0, 1],
            text("app"),
            text("rule"),
            vec![1],
            host_bytes(2, "repo"),
            vec![0, 1],
            wire_key(31),
            graph_bytes.clone(),
            vec![2, 1, 1, 0],
        ]
        .concat(),
    );

    let presentations = vec![
        (
            ExecutionPresentationKindV3::Explain {
                target: ExecutionExplainTargetV3::Rule(ExecutionCatalogIdentityV3 {
                    module: "app".to_owned(),
                    name: "rule".to_owned(),
                }),
                evidence: ExecutionEvidenceModeV3::Trace,
                truth: ExecutionTruthModeV3::Confirm,
            },
            [vec![0, 0], text("app"), text("rule"), vec![1, 1]].concat(),
        ),
        (
            ExecutionPresentationKindV3::Show {
                category: ExecutionShowCategoryV3::Capabilities,
            },
            vec![1, 0],
        ),
        (
            ExecutionPresentationKindV3::Describe {
                target: host(ExecutionGraphReferenceKindV3::Entity, "node"),
            },
            [vec![2], host_bytes(0, "node")].concat(),
        ),
        (
            ExecutionPresentationKindV3::Sample {
                query_text: "q".to_owned(),
                mode: ExecutionSampleModeV3::Semantic,
                limit: 2,
                scope: None,
            },
            [vec![3], text("q"), vec![1], n(2), vec![0]].concat(),
        ),
        (
            ExecutionPresentationKindV3::Compare {
                baseline: host(ExecutionGraphReferenceKindV3::Snapshot, "a"),
                candidate: host(ExecutionGraphReferenceKindV3::Snapshot, "b"),
                metric: ExecutionCompareMetricV3::Structural,
            },
            [vec![4], host_bytes(7, "a"), host_bytes(7, "b"), vec![1]].concat(),
        ),
        (
            ExecutionPresentationKindV3::Export {
                source: host(ExecutionGraphReferenceKindV3::ExportSource, "result"),
                format: ExecutionExportFormatV3::Sarif,
                profile: Some(ExecutionCatalogIdentityV3 {
                    module: "app".to_owned(),
                    name: "profile".to_owned(),
                }),
            },
            [
                vec![5],
                host_bytes(8, "result"),
                vec![2, 1],
                text("app"),
                text("profile"),
            ]
            .concat(),
        ),
    ];
    for (node, (presentation, bytes)) in (42u32..).zip(presentations) {
        items.push(ExecutionItemV3 {
            item_id: id(node),
            family: ExecutionFamilyV3::Presentation,
            relation: None,
            composition: None,
            graph: None,
            check: None,
            presentation: Some(presentation),
            command: None,
            plan: plan(),
        });
        expected.push([wire_key(node), vec![3, 0, 1, 2, 1, 6, 0, 0, 0, 0, 1], bytes].concat());
    }
    assert_document(items, &expected);
}
