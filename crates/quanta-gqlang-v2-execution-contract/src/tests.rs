use super::*;
use quanta_gqlang_v2_sema::{
    test_support::{reference_composition_document_v2, ReferenceCompositionBodyV2},
    HirNodeIdV1,
};

#[test]
fn sha256_matches_published_empty_vector() {
    assert_eq!(
        ExecutionDigestV3(sha256(b"")).to_hex(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn typed_literal_bytes_match_the_v3_contract() {
    let mut encoded = Vec::new();
    put_literal(&mut encoded, &ExecutionLiteralV3::Int(i128::MIN));
    assert_eq!(encoded, [vec![0, 0x80], vec![0; 15]].concat());

    encoded.clear();
    put_literal(
        &mut encoded,
        &ExecutionLiteralV3::Float(ExecutionFiniteFloatV3(1.5f64.to_bits())),
    );
    assert_eq!(
        encoded,
        [vec![1], 1.5f64.to_bits().to_be_bytes().to_vec()].concat()
    );

    encoded.clear();
    put_literal(&mut encoded, &ExecutionLiteralV3::Bool(false));
    assert_eq!(encoded, [2, 0]);
    encoded.clear();
    put_literal(&mut encoded, &ExecutionLiteralV3::Bool(true));
    assert_eq!(encoded, [2, 1]);

    encoded.clear();
    put_literal(&mut encoded, &ExecutionLiteralV3::String("é".to_owned()));
    assert_eq!(
        encoded,
        [vec![3], 2u64.to_be_bytes().to_vec(), vec![0xc3, 0xa9]].concat()
    );

    encoded.clear();
    put_literal(&mut encoded, &ExecutionLiteralV3::Null);
    assert_eq!(encoded, [4]);
}

#[test]
fn canonical_digests_match_the_pinned_golden_vectors() {
    let document = ExecutionDocumentV3::new(
        SourceIdV2::from_raw(0),
        b"show models",
        vec![relation_item()],
    );
    assert_eq!(document.source_digest_schema_version(), 3);
    assert_eq!(document.contract_digest_schema_version(), 8);
    assert_eq!(document.qualification_digest_schema_version(), 8);
    // Golden pins: any canonical-encoding change (tag swap, field reorder,
    // domain edit) must update these deliberately, never silently.
    assert_eq!(
        document.source_digest().to_hex(),
        "a8b69361d9c0d24e3022914bf1fba6386643164dc53911d34f1115dbc6102480"
    );
    assert_eq!(
        document.contract_digest().to_hex(),
        "c076e609614a771112e3d36e697daefc789d930c4005e66d165dafafe4250444"
    );
    assert_eq!(
        document.qualification_digest().to_hex(),
        "6d0e03a1a037aa4fb29c281c5e1e037ec3f06e274f26179f230345f93978d521"
    );
}

#[test]
fn independent_whole_stream_oracle_pins_document_header_items_and_order() {
    // Handwritten schema-8 stream for two payload-free certified items.
    // This does not call any production tag or put_* helper. The SHA values
    // were computed independently from these bytes with Python hashlib.
    let plan = ExecutionPlanV3 {
        authority: ExecutionAuthorityV3::CertifiedRequirementValidated,
        determinism: ExecutionDeterminismV3::Deterministic,
        evidence: ExecutionEvidenceV3::None,
        effect: ExecutionEffectV3::ReadOnly,
        policy_family: ExecutionPolicyFamilyV3::MetaRequireCertified,
    };
    let first = ExecutionItemV3 {
        item_id: item(9, 7),
        family: ExecutionFamilyV3::MetaRequireCertified,
        relation: None,
        graph: None,
        check: None,
        presentation: None,
        composition: None,
        command: None,
        plan: plan.clone(),
    };
    let second = ExecutionItemV3 {
        item_id: item(9, 8),
        family: ExecutionFamilyV3::MetaRequireCertified,
        relation: None,
        graph: None,
        check: None,
        presentation: None,
        composition: None,
        command: None,
        plan,
    };
    let source = b"require certified";
    let document = ExecutionDocumentV3::new(
        SourceIdV2::from_raw(9),
        source,
        vec![first.clone(), second.clone()],
    );

    let source_stream = [
        &[0, 0, 0, 0, 0, 0, 0, 32][..],
        b"qglang.execution.source-bytes.v3",
        &[0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 17],
        source,
    ]
    .concat();
    let contract_stream = [
        &[0, 0, 0, 0, 0, 0, 0, 28][..],
        b"qglang.execution.contract.v8",
        &[0, 0, 0, 8, 0, 0, 0, 9, 0, 0, 0, 0, 0, 0, 0, 2],
        // item 7: source, node, family, authority, determinism,
        // evidence, effect, policy, six absent payloads.
        &[0, 0, 0, 9, 0, 0, 0, 7, 5, 1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0],
        &[0, 0, 0, 9, 0, 0, 0, 8, 5, 1, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0],
    ]
    .concat();
    let qualification_stream = [
        &[0, 0, 0, 0, 0, 0, 0, 33][..],
        b"qglang.execution.qualification.v8",
        &[0, 0, 0, 8, 0, 0, 0, 9],
        Sha256::digest(&source_stream).as_slice(),
        Sha256::digest(&contract_stream).as_slice(),
    ]
    .concat();
    let expected = [
        (
            &source_stream,
            document.source_digest(),
            "f086dc8812ecb6fc73c5cbc01d655e1743695c83629f577cd096043fab1ae1eb",
        ),
        (
            &contract_stream,
            document.contract_digest(),
            "728b35c8e47e9c04f18fa2738f120bd630b8b945f89f5343fc0dc603dd967366",
        ),
        (
            &qualification_stream,
            document.qualification_digest(),
            "e147b89a6e6a8912dfb4a455b268a96b7bbf1da6f62a0e2ea70dde7dd5d35dcc",
        ),
    ];
    for (independent_stream, production_digest, pinned_sha) in expected {
        assert_eq!(
            format!("{:x}", Sha256::digest(independent_stream)),
            pinned_sha
        );
        assert_eq!(production_digest.to_hex(), pinned_sha);
    }
    let reversed = ExecutionDocumentV3::new(SourceIdV2::from_raw(9), source, vec![second, first]);
    assert_ne!(reversed.contract_digest(), document.contract_digest());
}

#[test]
fn root_stage_and_child_order_each_change_the_contract_digest() {
    let mut relation_item = relation_item();
    let relation = relation_item.relation.as_mut().expect("relation payload");
    relation.root.family = AlgebraFamilyV2::Join;
    relation.root.join_mode = Some(ExecutionJoinModeV3::Inner);
    let mut left = root();
    left.stage = key(0, 3);
    let mut right = root();
    right.stage = key(0, 4);
    relation.root.children = Box::new([left, right]);
    let baseline = contract_of(vec![relation_item.clone()]);

    let mut changed_stage = relation_item.clone();
    changed_stage
        .relation
        .as_mut()
        .expect("relation payload")
        .root
        .stage = key(0, 5);
    assert_ne!(contract_of(vec![changed_stage]), baseline);

    let mut reversed_children = relation_item;
    reversed_children
        .relation
        .as_mut()
        .expect("relation payload")
        .root
        .children
        .reverse();
    assert_ne!(contract_of(vec![reversed_children]), baseline);
}

fn assert_distinct_tags<const N: usize>(tags: [u8; N]) {
    let mut sorted = tags.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), N, "duplicate canonical tag in {tags:?}");
}

#[test]
fn every_canonical_tag_set_is_duplicate_free() {
    use ExecutionAuthorityV3 as Authority;
    use ExecutionDeterminismV3 as Determinism;
    use ExecutionEffectV3 as Effect;
    use ExecutionEvidenceV3 as Evidence;
    assert_distinct_tags([
        family_tag(ExecutionFamilyV3::Relation),
        family_tag(ExecutionFamilyV3::Graph),
        family_tag(ExecutionFamilyV3::Check),
        family_tag(ExecutionFamilyV3::Presentation),
        family_tag(ExecutionFamilyV3::Composition),
        family_tag(ExecutionFamilyV3::MetaRequireCertified),
        family_tag(ExecutionFamilyV3::RuleProgram),
    ]);
    assert_distinct_tags([
        authority_tag(Authority::FrontendChecked),
        authority_tag(Authority::CertifiedRequirementValidated),
    ]);
    assert_distinct_tags([
        determinism_tag(Determinism::Deterministic),
        determinism_tag(Determinism::NonDeterministic),
    ]);
    assert_distinct_tags([
        evidence_tag(Evidence::None),
        evidence_tag(Evidence::SourceLocation),
        evidence_tag(Evidence::WitnessPayload),
        evidence_tag(Evidence::ProofTree),
    ]);
    assert_distinct_tags([
        effect_tag(Effect::ReadOnly),
        effect_tag(Effect::ProofEvidenceOnly),
        effect_tag(Effect::ExportOnly),
        effect_tag(Effect::MutationForbidden),
    ]);
    assert_distinct_tags([
        policy_family_tag(ExecutionPolicyFamilyV3::Relational),
        policy_family_tag(ExecutionPolicyFamilyV3::Graph),
        policy_family_tag(ExecutionPolicyFamilyV3::Check),
        policy_family_tag(ExecutionPolicyFamilyV3::Presentation),
        policy_family_tag(ExecutionPolicyFamilyV3::RuleProgram),
        policy_family_tag(ExecutionPolicyFamilyV3::MetaRequireCertified),
        policy_family_tag(ExecutionPolicyFamilyV3::Composition),
    ]);
    assert_distinct_tags([
        cardinality_tag(ExecutionCardinalityV3::Set),
        cardinality_tag(ExecutionCardinalityV3::Bag),
        cardinality_tag(ExecutionCardinalityV3::Sequence),
    ]);
    assert_distinct_tags([
        sort_direction_tag(ExecutionSortDirectionV3::Asc),
        sort_direction_tag(ExecutionSortDirectionV3::Desc),
    ]);
    assert_distinct_tags([
        null_order_tag(ExecutionNullOrderV3::First),
        null_order_tag(ExecutionNullOrderV3::Last),
    ]);
    assert_distinct_tags([
        algebra_family_tag(AlgebraFamilyV2::Scan),
        algebra_family_tag(AlgebraFamilyV2::Filter),
        algebra_family_tag(AlgebraFamilyV2::Project),
        algebra_family_tag(AlgebraFamilyV2::Join),
        algebra_family_tag(AlgebraFamilyV2::Group),
        algebra_family_tag(AlgebraFamilyV2::Aggregate),
        algebra_family_tag(AlgebraFamilyV2::Sort),
        algebra_family_tag(AlgebraFamilyV2::SemiJoin),
        algebra_family_tag(AlgebraFamilyV2::AntiJoin),
        algebra_family_tag(AlgebraFamilyV2::ExistsFilter),
        algebra_family_tag(AlgebraFamilyV2::Limit),
    ]);
    assert_distinct_tags([
        join_mode_tag(ExecutionJoinModeV3::Inner),
        join_mode_tag(ExecutionJoinModeV3::Cross),
    ]);
    assert_distinct_tags([
        source_kind_tag(ExecutionSourceKindV3::Relation),
        source_kind_tag(ExecutionSourceKindV3::View),
        source_kind_tag(ExecutionSourceKindV3::Composition),
    ]);
    assert_distinct_tags([
        scalar_tag(ExecutionScalarTypeV3::Bool),
        scalar_tag(ExecutionScalarTypeV3::Int),
        scalar_tag(ExecutionScalarTypeV3::Float),
        scalar_tag(ExecutionScalarTypeV3::String),
        scalar_tag(ExecutionScalarTypeV3::Duration),
        scalar_tag(ExecutionScalarTypeV3::Location),
        scalar_tag(ExecutionScalarTypeV3::Path),
        scalar_tag(ExecutionScalarTypeV3::RepoPath),
        scalar_tag(ExecutionScalarTypeV3::EntityRef),
        scalar_tag(ExecutionScalarTypeV3::SymbolRef),
        scalar_tag(ExecutionScalarTypeV3::RuleRef),
        scalar_tag(ExecutionScalarTypeV3::SourceScope),
        scalar_tag(ExecutionScalarTypeV3::ProofTree),
    ]);
    assert_distinct_tags([
        logical_quantifier_tag(ExecutionLogicalQuantifierV3::Exists),
        logical_quantifier_tag(ExecutionLogicalQuantifierV3::Forall),
    ]);
    assert_distinct_tags([
        exists_polarity_tag(ExecutionExistsPolarityV3::Exists),
        exists_polarity_tag(ExecutionExistsPolarityV3::NotExists),
    ]);
    assert_distinct_tags([
        graph_kind_tag(ExecutionGraphKindV3::Reachability),
        graph_kind_tag(ExecutionGraphKindV3::Flow),
        graph_kind_tag(ExecutionGraphKindV3::Slice),
        graph_kind_tag(ExecutionGraphKindV3::Taint),
        graph_kind_tag(ExecutionGraphKindV3::Path),
        graph_kind_tag(ExecutionGraphKindV3::PointsTo),
        graph_kind_tag(ExecutionGraphKindV3::Summary),
    ]);
    assert_distinct_tags([
        graph_direction_tag(ExecutionGraphDirectionV3::Forward),
        graph_direction_tag(ExecutionGraphDirectionV3::Backward),
        graph_direction_tag(ExecutionGraphDirectionV3::Both),
        graph_direction_tag(ExecutionGraphDirectionV3::Undirected),
    ]);
    assert_distinct_tags([
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Source),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Sink),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Anchor),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Reference),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Pointee),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Callable),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Via),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Through),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Sanitizer),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Scope),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Window),
        graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Frontier),
    ]);
    assert_distinct_tags([
        graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Entity),
        graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Callable),
        graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Scope),
        graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Window),
        graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Frontier),
        graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Sanitizer),
        graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Pattern),
        graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Snapshot),
        graph_reference_kind_tag(ExecutionGraphReferenceKindV3::ExportSource),
    ]);
    assert_distinct_tags([
        graph_relation_shape_tag(ExecutionGraphRelationShapeV3::Acyclic),
        graph_relation_shape_tag(ExecutionGraphRelationShapeV3::Cyclic),
    ]);
    assert_distinct_tags([
        graph_dedup_identity_tag(ExecutionGraphDedupIdentityV3::Endpoints),
        graph_dedup_identity_tag(ExecutionGraphDedupIdentityV3::NodeSequence),
        graph_dedup_identity_tag(ExecutionGraphDedupIdentityV3::NodeSet),
    ]);
    assert_distinct_tags([
        graph_witness_tie_break_tag(ExecutionGraphWitnessTieBreakV3::ShortestThenLexical),
        graph_witness_tie_break_tag(ExecutionGraphWitnessTieBreakV3::Lexical),
    ]);
}

#[test]
fn registered_symbols_preserve_exact_spelling_and_reject_absent_authority() {
    for spelling in ["sum", "starts_with", "+", "!=", "not"] {
        assert_eq!(
            registered_execution_symbol(Some(spelling)),
            Ok(spelling.to_owned())
        );
    }
    assert_eq!(
        registered_execution_symbol(None),
        Err(ExecutionClosureFailureV3::UnregisteredSymbol)
    );
}

fn key(source: u32, node: u32) -> SourceQualifiedNodeIdV2 {
    SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(source),
        node_id: HirNodeIdV1::from_raw(node),
    }
}

fn item(source: u32, node: u32) -> SemanticItemIdV2 {
    SemanticItemIdV2::new(SourceIdV2::from_raw(source), HirNodeIdV1::from_raw(node))
}

fn binding(name: &str) -> ExecutionBindingV3 {
    ExecutionBindingV3 {
        source_id: SourceIdV2::from_raw(0),
        owner_node: 7,
        ordinal: 0,
        name: name.to_owned(),
    }
}

fn schema() -> ExecutionSchemaV3 {
    ExecutionSchemaV3 {
        fields: Box::new([ExecutionFieldV3 {
            name: "a".to_owned(),
            value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int),
        }]),
        cardinality: ExecutionCardinalityV3::Set,
        partition_keys: Box::new([]),
        order: Box::new([]),
    }
}

fn root() -> ExecutionRelationalOperatorV3 {
    ExecutionRelationalOperatorV3 {
        family: AlgebraFamilyV2::Scan,
        join_mode: None,
        limit: None,
        stage: key(0, 1),
        source: None,
        schema: schema(),
        predicates: Box::new([]),
        expressions: Box::new([]),
        correlations: Box::new([]),
        existential_terms: Box::new([]),
        partition_input: Box::new([]),
        children: Box::new([]),
    }
}

fn literal_expression(id: SourceQualifiedNodeIdV2, value: i128) -> ExecutionExpressionV3 {
    ExecutionExpressionV3 {
        id,
        value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int),
        shape: ExecutionExpressionShapeV3::Literal(ExecutionLiteralV3::Int(value)),
    }
}

fn operator(symbol: &str, overload_id: u32) -> ExecutionOperatorV3 {
    ExecutionOperatorV3 {
        symbol: symbol.to_owned(),
        overload_id,
    }
}

fn contract_with_shape(shape: ExecutionExpressionShapeV3) -> ExecutionDigestV3 {
    let mut item = relation_item();
    item.relation.as_mut().expect("relation").expressions = Box::new([ExecutionExpressionV3 {
        id: key(0, 2),
        value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int),
        shape,
    }]);
    contract_of(vec![item])
}

fn assert_all_digests_distinct(digests: &[ExecutionDigestV3]) {
    assert!(
        !digests.is_empty(),
        "an empty digest set cannot witness distinct encodings"
    );
    let mut deduped = digests.to_vec();
    deduped.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    deduped.dedup();
    assert_eq!(
        deduped.len(),
        digests.len(),
        "two DTO arms share one encoding"
    );
}

fn relation() -> ExecutionRelationV3 {
    ExecutionRelationV3 {
        item_id: item(0, 1),
        root: root(),
        expressions: Box::new([literal_expression(key(0, 2), 1)]),
    }
}

fn plan() -> ExecutionPlanV3 {
    ExecutionPlanV3 {
        authority: ExecutionAuthorityV3::FrontendChecked,
        determinism: ExecutionDeterminismV3::Deterministic,
        evidence: ExecutionEvidenceV3::None,
        effect: ExecutionEffectV3::ReadOnly,
        policy_family: ExecutionPolicyFamilyV3::Relational,
    }
}

fn relation_item() -> ExecutionItemV3 {
    ExecutionItemV3 {
        item_id: item(0, 1),
        family: ExecutionFamilyV3::Relation,
        relation: Some(relation()),
        graph: None,
        check: None,
        presentation: None,
        composition: None,
        command: None,
        plan: plan(),
    }
}

fn composition_item(binding_schema: Option<ExecutionSchemaV3>) -> ExecutionItemV3 {
    ExecutionItemV3 {
        item_id: item(0, 9),
        family: ExecutionFamilyV3::Composition,
        relation: None,
        graph: None,
        check: None,
        presentation: None,
        composition: Some(ExecutionCompositionV3 {
            bindings: Box::new([ExecutionCompositionBindingV3 {
                binding: binding("q"),
                producer_item_id: item(0, 1),
                producer_family: ExecutionFamilyV3::Relation,
                schema: binding_schema,
                relation: None,
                producer: ExecutionCompositionProducerV3::Relation,
            }]),
            with_binding: binding("q"),
            body_item_id: item(0, 1),
            body_family: ExecutionFamilyV3::Relation,
            body_relation: None,
            body: ExecutionCompositionBodyV3::Relation,
        }),
        command: None,
        plan: plan(),
    }
}

fn graph_query(kind: ExecutionGraphKindV3) -> ExecutionGraphQueryV3 {
    ExecutionGraphQueryV3 {
        kind,
        references: Box::new([]),
        depth: 1,
        direction: None,
        summary_facets: Box::new([]),
        relation_shape: ExecutionGraphRelationShapeV3::Cyclic,
        dedup_identity: ExecutionGraphDedupIdentityV3::NodeSequence,
        max_witnesses: 1,
        max_path_length: Some(1),
        witness_tie_break: ExecutionGraphWitnessTieBreakV3::ShortestThenLexical,
    }
}

#[test]
fn composition_rejects_a_same_variant_body_projection_from_another_item() {
    let fixture = reference_composition_document_v2(ReferenceCompositionBodyV2::Relational)
        .expect("reference composition resolves");
    let composition = fixture.composition();
    let producer_projections = composition
        .bindings()
        .iter()
        .map(|_| ExecutionCompositionProducerProjectionV3::Sample)
        .collect();

    assert_eq!(
        project_composition(
            composition,
            producer_projections,
            ExecutionCompositionBodyProjectionV3::Relation(relation()),
        ),
        Err(ExecutionCompositionProjectionFailureV3::Body),
    );
}

#[test]
fn composition_rejects_same_variant_graph_projections_from_other_items() {
    let fixture = reference_composition_document_v2(ReferenceCompositionBodyV2::Graph)
        .expect("reference graph composition resolves");
    let composition = fixture.composition();

    assert_eq!(
        project_composition(
            composition,
            vec![ExecutionCompositionProducerProjectionV3::Graph {
                item_id: fixture.body_item_id(),
                query: graph_query(ExecutionGraphKindV3::Path),
            }],
            ExecutionCompositionBodyProjectionV3::Graph {
                item_id: fixture.body_item_id(),
                query: graph_query(ExecutionGraphKindV3::Path),
            },
        ),
        Err(ExecutionCompositionProjectionFailureV3::Producer),
    );

    assert_eq!(
        project_composition(
            composition,
            vec![ExecutionCompositionProducerProjectionV3::Graph {
                item_id: fixture.producer_item_ids()[0],
                query: graph_query(ExecutionGraphKindV3::Path),
            }],
            ExecutionCompositionBodyProjectionV3::Graph {
                item_id: fixture.producer_item_ids()[0],
                query: graph_query(ExecutionGraphKindV3::Path),
            },
        ),
        Err(ExecutionCompositionProjectionFailureV3::Body),
    );
}

#[test]
fn independent_graph_encoder_matches_the_canonical_substream() {
    fn write_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_be_bytes());
    }
    fn write_len(out: &mut Vec<u8>, value: usize) {
        out.extend_from_slice(
            &u64::try_from(value)
                .expect("test length fits")
                .to_be_bytes(),
        );
    }
    fn write_text(out: &mut Vec<u8>, value: &str) {
        write_len(out, value.len());
        out.extend_from_slice(value.as_bytes());
    }
    fn independent_target(out: &mut Vec<u8>, target: &ExecutionGraphTargetV3) {
        match target {
            ExecutionGraphTargetV3::Host {
                kind,
                spelling,
                entity_subtype,
            } => {
                out.push(0);
                out.push(match kind {
                    ExecutionGraphReferenceKindV3::Entity => 0,
                    ExecutionGraphReferenceKindV3::Callable => 1,
                    ExecutionGraphReferenceKindV3::Scope => 2,
                    ExecutionGraphReferenceKindV3::Window => 3,
                    ExecutionGraphReferenceKindV3::Frontier => 4,
                    ExecutionGraphReferenceKindV3::Sanitizer => 5,
                    ExecutionGraphReferenceKindV3::Pattern => 6,
                    ExecutionGraphReferenceKindV3::Snapshot => 7,
                    ExecutionGraphReferenceKindV3::ExportSource => 8,
                });
                write_text(out, spelling);
                if let Some(subtype) = entity_subtype {
                    out.push(1);
                    write_text(out, subtype);
                } else {
                    out.push(0);
                }
            }
            ExecutionGraphTargetV3::Composition {
                binding,
                capability,
            } => {
                out.push(1);
                write_u32(out, binding.source_id().raw());
                write_u32(out, binding.owner_node());
                write_len(out, binding.ordinal());
                write_text(out, binding.name());
                out.push(match capability {
                    ExecutionGraphCapabilityV3::Entity => 0,
                    ExecutionGraphCapabilityV3::Scope => 1,
                });
            }
        }
    }
    fn independent_encode(query: &ExecutionGraphQueryV3) -> Vec<u8> {
        let mut out = Vec::new();
        out.push(match query.kind() {
            ExecutionGraphKindV3::Reachability => 0,
            ExecutionGraphKindV3::Flow => 1,
            ExecutionGraphKindV3::Slice => 2,
            ExecutionGraphKindV3::Taint => 3,
            ExecutionGraphKindV3::Path => 4,
            ExecutionGraphKindV3::PointsTo => 5,
            ExecutionGraphKindV3::Summary => 6,
        });
        write_len(&mut out, query.references().len());
        for reference in query.references() {
            out.push(match reference.role() {
                ExecutionGraphReferenceRoleV3::Source => 0,
                ExecutionGraphReferenceRoleV3::Sink => 1,
                ExecutionGraphReferenceRoleV3::Anchor => 2,
                ExecutionGraphReferenceRoleV3::Reference => 3,
                ExecutionGraphReferenceRoleV3::Pointee => 4,
                ExecutionGraphReferenceRoleV3::Callable => 5,
                ExecutionGraphReferenceRoleV3::Via => 6,
                ExecutionGraphReferenceRoleV3::Through => 7,
                ExecutionGraphReferenceRoleV3::Sanitizer => 8,
                ExecutionGraphReferenceRoleV3::Scope => 9,
                ExecutionGraphReferenceRoleV3::Window => 10,
                ExecutionGraphReferenceRoleV3::Frontier => 11,
            });
            independent_target(&mut out, reference.target());
        }
        write_u32(&mut out, query.depth());
        if let Some(direction) = query.direction() {
            out.push(1);
            out.push(match direction {
                ExecutionGraphDirectionV3::Forward => 0,
                ExecutionGraphDirectionV3::Backward => 1,
                ExecutionGraphDirectionV3::Both => 2,
                ExecutionGraphDirectionV3::Undirected => 3,
            });
        } else {
            out.push(0);
        }
        write_len(&mut out, query.summary_facets().len());
        for facet in query.summary_facets() {
            write_text(&mut out, facet);
        }
        out.push(match query.relation_shape() {
            ExecutionGraphRelationShapeV3::Acyclic => 0,
            ExecutionGraphRelationShapeV3::Cyclic => 1,
        });
        out.push(match query.dedup_identity() {
            ExecutionGraphDedupIdentityV3::Endpoints => 0,
            ExecutionGraphDedupIdentityV3::NodeSequence => 1,
            ExecutionGraphDedupIdentityV3::NodeSet => 2,
        });
        write_u32(&mut out, query.max_witnesses());
        if let Some(max_path_length) = query.max_path_length() {
            out.push(1);
            write_u32(&mut out, max_path_length);
        } else {
            out.push(0);
        }
        out.push(match query.witness_tie_break() {
            ExecutionGraphWitnessTieBreakV3::ShortestThenLexical => 0,
            ExecutionGraphWitnessTieBreakV3::Lexical => 1,
        });
        out
    }

    let query = ExecutionGraphQueryV3 {
        kind: ExecutionGraphKindV3::Summary,
        references: Box::new([
            ExecutionGraphReferenceV3 {
                role: ExecutionGraphReferenceRoleV3::Source,
                target: ExecutionGraphTargetV3::Host {
                    kind: ExecutionGraphReferenceKindV3::Snapshot,
                    spelling: "repo@head".to_owned(),
                    entity_subtype: Some("function".to_owned()),
                },
            },
            ExecutionGraphReferenceV3 {
                role: ExecutionGraphReferenceRoleV3::Scope,
                target: ExecutionGraphTargetV3::Composition {
                    binding: binding("prior"),
                    capability: ExecutionGraphCapabilityV3::Scope,
                },
            },
        ]),
        depth: 7,
        direction: Some(ExecutionGraphDirectionV3::Undirected),
        summary_facets: Box::new(["calls".to_owned(), "types".to_owned()]),
        relation_shape: ExecutionGraphRelationShapeV3::Acyclic,
        dedup_identity: ExecutionGraphDedupIdentityV3::NodeSet,
        max_witnesses: 11,
        max_path_length: Some(13),
        witness_tie_break: ExecutionGraphWitnessTieBreakV3::Lexical,
    };
    let mut canonical = Vec::new();
    put_graph_query(&mut canonical, &query);
    assert_eq!(canonical, independent_encode(&query));
}

fn host_graph_target(spelling: &str) -> ExecutionGraphTargetV3 {
    ExecutionGraphTargetV3::Host {
        kind: ExecutionGraphReferenceKindV3::Entity,
        spelling: spelling.to_owned(),
        entity_subtype: None,
    }
}

fn contract_of(items: Vec<ExecutionItemV3>) -> ExecutionDigestV3 {
    ExecutionDocumentV3::new(SourceIdV2::from_raw(0), b"show models", items).contract_digest()
}

fn qualification_of(items: Vec<ExecutionItemV3>) -> ExecutionDigestV3 {
    ExecutionDocumentV3::new(SourceIdV2::from_raw(0), b"show models", items).qualification_digest()
}

#[test]
fn predicate_mutation_moves_contract_and_qualification() {
    let base = contract_of(vec![relation_item()]);
    let mut mutated = relation_item();
    mutated.relation.as_mut().expect("relation").root.predicates = Box::new([key(0, 2)]);
    assert_ne!(contract_of(vec![mutated.clone()]), base);
    assert_ne!(
        qualification_of(vec![mutated]),
        qualification_of(vec![relation_item()])
    );
}

#[test]
fn partition_input_mutation_moves_contract_and_qualification() {
    let base = contract_of(vec![relation_item()]);
    let mut mutated = relation_item();
    mutated
        .relation
        .as_mut()
        .expect("relation")
        .root
        .partition_input = Box::new([binding("p")]);
    assert_ne!(contract_of(vec![mutated.clone()]), base);
    assert_ne!(
        qualification_of(vec![mutated]),
        qualification_of(vec![relation_item()])
    );
}

#[test]
fn correlation_mutation_moves_contract_and_qualification() {
    let base = contract_of(vec![relation_item()]);
    let mut mutated = relation_item();
    mutated
        .relation
        .as_mut()
        .expect("relation")
        .root
        .correlations = Box::new([ExecutionCorrelationV3 {
        binding: binding("c"),
        reference: key(0, 2),
    }]);
    assert_ne!(contract_of(vec![mutated.clone()]), base);
    assert_ne!(
        qualification_of(vec![mutated]),
        qualification_of(vec![relation_item()])
    );
}

#[test]
fn schema_mutation_moves_contract_and_qualification() {
    let base = contract_of(vec![relation_item()]);
    let mut mutated = relation_item();
    mutated.relation.as_mut().expect("relation").root.schema = ExecutionSchemaV3 {
        fields: Box::new([ExecutionFieldV3 {
            name: "b".to_owned(),
            value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::String),
        }]),
        cardinality: ExecutionCardinalityV3::Bag,
        partition_keys: Box::new([key(0, 2)]),
        order: Box::new([ExecutionOrderKeyV3 {
            expression: key(0, 2),
            direction: ExecutionSortDirectionV3::Desc,
            null_order: ExecutionNullOrderV3::Last,
            value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::String),
        }]),
    };
    assert_ne!(contract_of(vec![mutated.clone()]), base);
    assert_ne!(
        qualification_of(vec![mutated]),
        qualification_of(vec![relation_item()])
    );
}

#[test]
fn expression_shape_mutation_moves_contract_and_qualification() {
    let base = contract_of(vec![relation_item()]);
    let mut mutated = relation_item();
    mutated.relation.as_mut().expect("relation").expressions =
        Box::new([literal_expression(key(0, 2), 2)]);
    assert_ne!(contract_of(vec![mutated.clone()]), base);
    assert_ne!(
        qualification_of(vec![mutated]),
        qualification_of(vec![relation_item()])
    );
}

#[test]
fn composition_binding_schema_mutation_moves_contract_and_qualification() {
    let base = contract_of(vec![composition_item(None)]);
    assert_ne!(contract_of(vec![composition_item(Some(schema()))]), base);
    assert_ne!(
        qualification_of(vec![composition_item(Some(schema()))]),
        qualification_of(vec![composition_item(None)])
    );
}

#[test]
fn composition_sample_scope_and_every_graph_query_field_move_contract() {
    let mut unscoped = composition_item(None);
    unscoped.composition.as_mut().expect("composition").bindings[0].producer =
        ExecutionCompositionProducerV3::Sample {
            query_text: "auth".to_owned(),
            mode: ExecutionSampleModeV3::Lexical,
            limit: 1,
            scope: None,
        };
    let mut scoped = unscoped.clone();
    let ExecutionCompositionProducerV3::Sample { scope, .. } =
        &mut scoped.composition.as_mut().expect("composition").bindings[0].producer
    else {
        panic!("sample")
    };
    *scope = Some(host_graph_target("repo"));
    assert_ne!(contract_of(vec![unscoped]), contract_of(vec![scoped]));

    let mut base = composition_item(None);
    base.composition.as_mut().expect("composition").bindings[0].producer =
        ExecutionCompositionProducerV3::Graph {
            query: graph_query(ExecutionGraphKindV3::Path),
        };
    let base_digest = contract_of(vec![base.clone()]);
    let mut mutations = Vec::new();

    let mut kind = base.clone();
    let ExecutionCompositionProducerV3::Graph { query } =
        &mut kind.composition.as_mut().expect("composition").bindings[0].producer
    else {
        panic!("graph")
    };
    query.kind = ExecutionGraphKindV3::Flow;
    mutations.push(kind);

    let mut references = base.clone();
    let ExecutionCompositionProducerV3::Graph { query } = &mut references
        .composition
        .as_mut()
        .expect("composition")
        .bindings[0]
        .producer
    else {
        panic!("graph")
    };
    query.references = Box::new([ExecutionGraphReferenceV3 {
        role: ExecutionGraphReferenceRoleV3::Source,
        target: host_graph_target("a"),
    }]);
    mutations.push(references);

    let mut depth = base.clone();
    let ExecutionCompositionProducerV3::Graph { query } =
        &mut depth.composition.as_mut().expect("composition").bindings[0].producer
    else {
        panic!("graph")
    };
    query.depth = 2;
    mutations.push(depth);

    let mut direction = base.clone();
    let ExecutionCompositionProducerV3::Graph { query } = &mut direction
        .composition
        .as_mut()
        .expect("composition")
        .bindings[0]
        .producer
    else {
        panic!("graph")
    };
    query.direction = Some(ExecutionGraphDirectionV3::Backward);
    mutations.push(direction);

    let mut facets = base.clone();
    let ExecutionCompositionProducerV3::Graph { query } =
        &mut facets.composition.as_mut().expect("composition").bindings[0].producer
    else {
        panic!("graph")
    };
    query.summary_facets = Box::new(["calls".to_owned()]);
    mutations.push(facets);

    let mut relation_shape = base.clone();
    let ExecutionCompositionProducerV3::Graph { query } = &mut relation_shape
        .composition
        .as_mut()
        .expect("composition")
        .bindings[0]
        .producer
    else {
        panic!("graph")
    };
    query.relation_shape = ExecutionGraphRelationShapeV3::Acyclic;
    mutations.push(relation_shape);

    let mut dedup_identity = base.clone();
    let ExecutionCompositionProducerV3::Graph { query } = &mut dedup_identity
        .composition
        .as_mut()
        .expect("composition")
        .bindings[0]
        .producer
    else {
        panic!("graph")
    };
    query.dedup_identity = ExecutionGraphDedupIdentityV3::Endpoints;
    mutations.push(dedup_identity);

    let mut witnesses = base.clone();
    let ExecutionCompositionProducerV3::Graph { query } = &mut witnesses
        .composition
        .as_mut()
        .expect("composition")
        .bindings[0]
        .producer
    else {
        panic!("graph")
    };
    query.max_witnesses = 2;
    mutations.push(witnesses);

    let mut max_path_length = base.clone();
    let ExecutionCompositionProducerV3::Graph { query } = &mut max_path_length
        .composition
        .as_mut()
        .expect("composition")
        .bindings[0]
        .producer
    else {
        panic!("graph")
    };
    query.max_path_length = Some(2);
    mutations.push(max_path_length);

    let mut witness_tie_break = base;
    let ExecutionCompositionProducerV3::Graph { query } = &mut witness_tie_break
        .composition
        .as_mut()
        .expect("composition")
        .bindings[0]
        .producer
    else {
        panic!("graph")
    };
    query.witness_tie_break = ExecutionGraphWitnessTieBreakV3::Lexical;
    mutations.push(witness_tie_break);

    for mutation in mutations {
        assert_ne!(contract_of(vec![mutation]), base_digest);
    }
}

#[test]
fn composition_binding_identity_and_order_move_contract() {
    let mut renamed = composition_item(None);
    let composition = renamed.composition.as_mut().expect("composition");
    composition
        .bindings
        .first_mut()
        .expect("one binding")
        .binding
        .name = "renamed".to_owned();
    composition.with_binding.name = "renamed".to_owned();
    assert_ne!(
        contract_of(vec![renamed]),
        contract_of(vec![composition_item(None)])
    );

    let mut ordered = composition_item(None);
    let composition = ordered.composition.as_mut().expect("composition");
    let first = composition.bindings.first().expect("one binding").clone();
    let mut second = first.clone();
    second.binding.ordinal = 1;
    second.binding.name = "second".to_owned();
    second.producer_item_id = item(0, 3);
    composition.bindings = vec![first, second].into_boxed_slice();
    let ordered_digest = contract_of(vec![ordered.clone()]);
    ordered
        .composition
        .as_mut()
        .expect("composition")
        .bindings
        .swap(0, 1);
    assert_ne!(ordered_digest, contract_of(vec![ordered]));
}

#[test]
fn body_relation_mutation_moves_contract_and_qualification() {
    let base = contract_of(vec![composition_item(None)]);
    let mut mutated = composition_item(None);
    mutated
        .composition
        .as_mut()
        .expect("composition")
        .body_relation = Some(relation());
    assert_ne!(contract_of(vec![mutated.clone()]), base);
    assert_ne!(
        qualification_of(vec![mutated]),
        qualification_of(vec![composition_item(None)])
    );
}

#[test]
fn plan_authority_effect_evidence_mutations_move_contract_and_qualification() {
    let base_contract = contract_of(vec![relation_item()]);
    let base_qualification = qualification_of(vec![relation_item()]);
    let mut authority = relation_item();
    authority.plan.authority = ExecutionAuthorityV3::CertifiedRequirementValidated;
    assert_ne!(contract_of(vec![authority.clone()]), base_contract);
    assert_ne!(qualification_of(vec![authority]), base_qualification);
    let mut effect = relation_item();
    effect.plan.effect = ExecutionEffectV3::ExportOnly;
    assert_ne!(contract_of(vec![effect.clone()]), base_contract);
    assert_ne!(qualification_of(vec![effect]), base_qualification);
    let mut evidence = relation_item();
    evidence.plan.evidence = ExecutionEvidenceV3::ProofTree;
    assert_ne!(contract_of(vec![evidence.clone()]), base_contract);
    assert_ne!(qualification_of(vec![evidence]), base_qualification);
}

#[test]
fn source_byte_mutation_moves_qualification_but_not_contract() {
    let base = ExecutionDocumentV3::new(
        SourceIdV2::from_raw(0),
        b"show models",
        vec![relation_item()],
    );
    let mutated = ExecutionDocumentV3::new(
        SourceIdV2::from_raw(0),
        b"show models ",
        vec![relation_item()],
    );
    assert_eq!(base.contract_digest(), mutated.contract_digest());
    assert_ne!(base.source_digest(), mutated.source_digest());
    assert_ne!(base.qualification_digest(), mutated.qualification_digest());
}

#[test]
fn closure_collects_transitively_in_stable_identity_order() {
    let root_key = key(0, 9);
    let left = key(0, 3);
    let right = key(0, 5);
    let leaf = key(0, 1);
    let children = |id: SourceQualifiedNodeIdV2| {
        if id == root_key {
            Some(vec![right, left])
        } else if id == left || id == right {
            Some(vec![leaf])
        } else if id == leaf {
            Some(Vec::new())
        } else {
            None
        }
    };
    let closed =
        close_expression_ids(SourceIdV2::from_raw(0), &[root_key], &children).expect("closed");
    assert_eq!(closed, vec![leaf, left, right, root_key]);
}

#[test]
fn closure_dangling_reference_fails_closed() {
    let children = |_: SourceQualifiedNodeIdV2| -> Option<Vec<SourceQualifiedNodeIdV2>> { None };
    assert_eq!(
        close_expression_ids(SourceIdV2::from_raw(0), &[key(0, 1)], &children),
        Err(ExecutionClosureFailureV3::Dangling)
    );
}

#[test]
fn closure_cross_source_reference_fails_closed() {
    let children = |_: SourceQualifiedNodeIdV2| Some(Vec::new());
    assert_eq!(
        close_expression_ids(SourceIdV2::from_raw(0), &[key(1, 1)], &children),
        Err(ExecutionClosureFailureV3::Foreign)
    );
}

#[test]
fn value_type_projection_covers_every_constructible_arm() {
    use quanta_gqlang_v2_sema::{
        LocatedSpanV2, RecordFieldTypeV1, RecordTypeV1, SourceRegistrationV2, SourceRegistryViewV2,
        SpanV1,
    };
    // Every constructible scalar arm projects one-to-one; a swapped
    // `project_scalar` pair must fail here rather than stay green.
    for (scalar, expected) in [
        (ScalarTyV1::Bool, ExecutionScalarTypeV3::Bool),
        (ScalarTyV1::Int, ExecutionScalarTypeV3::Int),
        (ScalarTyV1::Float, ExecutionScalarTypeV3::Float),
        (ScalarTyV1::String, ExecutionScalarTypeV3::String),
        (ScalarTyV1::Duration, ExecutionScalarTypeV3::Duration),
        (ScalarTyV1::Location, ExecutionScalarTypeV3::Location),
        (ScalarTyV1::Path, ExecutionScalarTypeV3::Path),
        (ScalarTyV1::RepoPath, ExecutionScalarTypeV3::RepoPath),
        (ScalarTyV1::EntityRef, ExecutionScalarTypeV3::EntityRef),
        (ScalarTyV1::SymbolRef, ExecutionScalarTypeV3::SymbolRef),
        (ScalarTyV1::RuleRef, ExecutionScalarTypeV3::RuleRef),
        (ScalarTyV1::SourceScope, ExecutionScalarTypeV3::SourceScope),
        (ScalarTyV1::ProofTree, ExecutionScalarTypeV3::ProofTree),
    ] {
        assert_eq!(
            project_value_type(&ValueTypeV1::scalar(scalar.clone())),
            ExecutionValueTypeV3::Scalar(expected),
            "{scalar:?}"
        );
    }
    assert_eq!(
        project_value_type(&ValueTypeV1::null()),
        ExecutionValueTypeV3::Null
    );
    let tuple = ValueTypeV1::try_tuple(vec![ValueTypeV1::int(), ValueTypeV1::string()])
        .expect("tuple of scalars");
    assert_eq!(
        project_value_type(&tuple),
        ExecutionValueTypeV3::Tuple(Box::new([
            ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int),
            ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::String),
        ]))
    );
    let optional = ValueTypeV1::try_optional(ValueTypeV1::bool()).expect("optional scalar");
    assert_eq!(
        project_value_type(&optional),
        ExecutionValueTypeV3::Optional(Box::new(ExecutionValueTypeV3::Scalar(
            ExecutionScalarTypeV3::Bool
        )))
    );
    let collection = ValueTypeV1::try_collection(ValueTypeV1::int(), CardinalityV1::Sequence)
        .expect("sequence collection");
    assert_eq!(
        project_value_type(&collection),
        ExecutionValueTypeV3::Collection {
            element: Box::new(ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int)),
            cardinality: ExecutionCardinalityV3::Sequence,
        }
    );
    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(0),
        source_len: 64,
    }])
    .expect("one registered source");
    let location = registry
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(0),
            span: SpanV1::new(0, 1),
        })
        .expect("checked location");
    let field =
        RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), location).expect("record field");
    let record = ValueTypeV1::try_record(RecordTypeV1::try_new(vec![field]).expect("record row"))
        .expect("record value");
    assert_eq!(
        project_value_type(&record),
        ExecutionValueTypeV3::Record(Box::new([ExecutionRecordFieldV3 {
            name: "id".to_owned(),
            value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int),
        }]))
    );
    // Union projection mirrors record arm-for-arm; its constructors are not
    // re-exported for downstream (no producer path builds unions), so the
    // union ENCODING is pinned by `composite_value_type_arms_encode_distinctly`
    // instead of round-tripping through the owner here.
}

#[test]
fn composite_value_type_arms_encode_distinctly() {
    let scalar = ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int);
    let variants = [
        ExecutionValueTypeV3::Null,
        ExecutionValueTypeV3::Tuple(Box::new([scalar.clone()])),
        ExecutionValueTypeV3::Record(Box::new([ExecutionRecordFieldV3 {
            name: "id".to_owned(),
            value_type: scalar.clone(),
        }])),
        ExecutionValueTypeV3::Optional(Box::new(scalar.clone())),
        ExecutionValueTypeV3::Collection {
            element: Box::new(scalar.clone()),
            cardinality: ExecutionCardinalityV3::Sequence,
        },
        ExecutionValueTypeV3::Union(Box::new([ExecutionUnionVariantV3 {
            tag: "ok".to_owned(),
            value_type: scalar.clone(),
        }])),
        ExecutionValueTypeV3::Unknown,
    ];
    let mut digests = Vec::with_capacity(variants.len());
    for value_type in &variants {
        let mut item = relation_item();
        item.relation.as_mut().expect("relation").root.schema.fields =
            Box::new([ExecutionFieldV3 {
                name: "a".to_owned(),
                value_type: value_type.clone(),
            }]);
        digests.push(contract_of(vec![item]));
    }
    let mut deduped = digests.clone();
    deduped.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    deduped.dedup();
    assert_eq!(
        deduped.len(),
        variants.len(),
        "two value-type arms share one encoding"
    );
}

#[test]
fn plan_determinism_and_policy_family_mutations_move_contract() {
    let base = contract_of(vec![relation_item()]);
    let mut determinism = relation_item();
    determinism.plan.determinism = ExecutionDeterminismV3::NonDeterministic;
    assert_ne!(contract_of(vec![determinism]), base);
    let mut policy = relation_item();
    policy.plan.policy_family = ExecutionPolicyFamilyV3::Graph;
    assert_ne!(contract_of(vec![policy]), base);
}

#[test]
fn every_expression_shape_and_call_target_has_a_distinct_encoding() {
    let lhs = key(0, 3);
    let rhs = key(0, 4);
    let op = operator("+", 7);
    let shapes = vec![
        ExecutionExpressionShapeV3::Literal(ExecutionLiteralV3::Int(1)),
        ExecutionExpressionShapeV3::Binding(binding("value")),
        ExecutionExpressionShapeV3::Path {
            base: lhs,
            segments: Box::new(["field".to_owned()]),
        },
        ExecutionExpressionShapeV3::Unary {
            operator: op.clone(),
            operand: lhs,
        },
        ExecutionExpressionShapeV3::Binary {
            operator: op.clone(),
            lhs,
            rhs,
        },
        ExecutionExpressionShapeV3::Call {
            target: ExecutionCallTargetV3::Builtin {
                name: "len".to_owned(),
            },
            arguments: Box::new([Some(lhs), None]),
        },
        ExecutionExpressionShapeV3::Call {
            target: ExecutionCallTargetV3::Predicate {
                name: "p".to_owned(),
            },
            arguments: Box::new([Some(lhs)]),
        },
        ExecutionExpressionShapeV3::Call {
            target: ExecutionCallTargetV3::HostPredicate {
                fingerprint: "00ff".to_owned(),
            },
            arguments: Box::new([Some(lhs)]),
        },
        ExecutionExpressionShapeV3::Paren(lhs),
        ExecutionExpressionShapeV3::Record(Box::new([("field".to_owned(), lhs)])),
        ExecutionExpressionShapeV3::List(Box::new([lhs, rhs])),
        ExecutionExpressionShapeV3::BoolValue(lhs),
        ExecutionExpressionShapeV3::Compare {
            operator: op.clone(),
            lhs,
            rhs,
        },
        ExecutionExpressionShapeV3::And {
            operator: op.clone(),
            lhs,
            rhs,
        },
        ExecutionExpressionShapeV3::Or {
            operator: op.clone(),
            lhs,
            rhs,
        },
        ExecutionExpressionShapeV3::Not {
            operator: op,
            operand: lhs,
        },
        ExecutionExpressionShapeV3::LogicalQuantifier {
            kind: ExecutionLogicalQuantifierV3::Exists,
            binders: Box::new([binding("x")]),
            body: lhs,
        },
        ExecutionExpressionShapeV3::RelationExists {
            polarity: ExecutionExistsPolarityV3::Exists,
            relation_root: lhs,
            correlations: Box::new([ExecutionCorrelationV3 {
                binding: binding("outer"),
                reference: rhs,
            }]),
        },
    ];
    let digests: Vec<_> = shapes.into_iter().map(contract_with_shape).collect();
    assert_all_digests_distinct(&digests);
}

#[test]
fn expression_payload_fields_move_the_contract_digest() {
    let lhs = key(0, 3);
    let rhs = key(0, 4);
    let changed = key(0, 5);
    let base_call = contract_with_shape(ExecutionExpressionShapeV3::Call {
        target: ExecutionCallTargetV3::Predicate {
            name: "p".to_owned(),
        },
        arguments: Box::new([Some(lhs), None]),
    });
    for shape in [
        ExecutionExpressionShapeV3::Call {
            target: ExecutionCallTargetV3::Predicate {
                name: "q".to_owned(),
            },
            arguments: Box::new([Some(lhs), None]),
        },
        ExecutionExpressionShapeV3::Call {
            target: ExecutionCallTargetV3::Predicate {
                name: "p".to_owned(),
            },
            arguments: Box::new([Some(changed), None]),
        },
        ExecutionExpressionShapeV3::Call {
            target: ExecutionCallTargetV3::Predicate {
                name: "p".to_owned(),
            },
            arguments: Box::new([Some(lhs), Some(rhs)]),
        },
    ] {
        assert_ne!(contract_with_shape(shape), base_call);
    }

    let base_quantifier = contract_with_shape(ExecutionExpressionShapeV3::LogicalQuantifier {
        kind: ExecutionLogicalQuantifierV3::Exists,
        binders: Box::new([binding("x")]),
        body: lhs,
    });
    for shape in [
        ExecutionExpressionShapeV3::LogicalQuantifier {
            kind: ExecutionLogicalQuantifierV3::Forall,
            binders: Box::new([binding("x")]),
            body: lhs,
        },
        ExecutionExpressionShapeV3::LogicalQuantifier {
            kind: ExecutionLogicalQuantifierV3::Exists,
            binders: Box::new([binding("y")]),
            body: lhs,
        },
        ExecutionExpressionShapeV3::LogicalQuantifier {
            kind: ExecutionLogicalQuantifierV3::Exists,
            binders: Box::new([binding("x")]),
            body: changed,
        },
    ] {
        assert_ne!(contract_with_shape(shape), base_quantifier);
    }

    let base_exists = contract_with_shape(ExecutionExpressionShapeV3::RelationExists {
        polarity: ExecutionExistsPolarityV3::Exists,
        relation_root: lhs,
        correlations: Box::new([ExecutionCorrelationV3 {
            binding: binding("outer"),
            reference: rhs,
        }]),
    });
    for shape in [
        ExecutionExpressionShapeV3::RelationExists {
            polarity: ExecutionExistsPolarityV3::NotExists,
            relation_root: lhs,
            correlations: Box::new([ExecutionCorrelationV3 {
                binding: binding("outer"),
                reference: rhs,
            }]),
        },
        ExecutionExpressionShapeV3::RelationExists {
            polarity: ExecutionExistsPolarityV3::Exists,
            relation_root: changed,
            correlations: Box::new([ExecutionCorrelationV3 {
                binding: binding("outer"),
                reference: rhs,
            }]),
        },
        ExecutionExpressionShapeV3::RelationExists {
            polarity: ExecutionExistsPolarityV3::Exists,
            relation_root: lhs,
            correlations: Box::new([ExecutionCorrelationV3 {
                binding: binding("changed"),
                reference: rhs,
            }]),
        },
    ] {
        assert_ne!(contract_with_shape(shape), base_exists);
    }
}

#[test]
fn join_source_composition_body_and_sample_mode_arms_are_digest_bound() {
    let mut join_digests = Vec::new();
    for mode in [
        None,
        Some(ExecutionJoinModeV3::Inner),
        Some(ExecutionJoinModeV3::Cross),
    ] {
        let mut candidate = relation_item();
        let root = &mut candidate.relation.as_mut().expect("relation").root;
        root.family = AlgebraFamilyV2::Join;
        root.join_mode = mode;
        join_digests.push(contract_of(vec![candidate]));
    }
    assert_all_digests_distinct(&join_digests);

    let mut source_digests = Vec::new();
    for authority in [
        ExecutionRelationAuthorityV3::Host,
        ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles,
        ExecutionRelationAuthorityV3::CatalogView,
        ExecutionRelationAuthorityV3::CompositionBinding,
    ] {
        let mut candidate = relation_item();
        candidate.relation.as_mut().expect("relation").root.source = Some(ExecutionSourceV3 {
            authority,
            kind: ExecutionSourceKindV3::Relation,
            namespace: Some("ns".to_owned()),
            name: "items".to_owned(),
            composition_binding: None,
        });
        source_digests.push(contract_of(vec![candidate]));
    }
    assert_all_digests_distinct(&source_digests);

    let mut body_digests = Vec::new();
    for body in [
        ExecutionCompositionBodyV3::Relation,
        ExecutionCompositionBodyV3::Graph(graph_query(ExecutionGraphKindV3::Taint)),
        ExecutionCompositionBodyV3::Check,
        ExecutionCompositionBodyV3::Explain,
    ] {
        let mut candidate = composition_item(None);
        candidate.composition.as_mut().expect("composition").body = body;
        body_digests.push(contract_of(vec![candidate]));
    }
    assert_all_digests_distinct(&body_digests);

    let mut sample_digests = Vec::new();
    for mode in [
        ExecutionSampleModeV3::Lexical,
        ExecutionSampleModeV3::Semantic,
        ExecutionSampleModeV3::Hybrid,
    ] {
        let mut candidate = composition_item(None);
        let binding = candidate
            .composition
            .as_mut()
            .expect("composition")
            .bindings
            .first_mut()
            .expect("one binding");
        binding.producer = ExecutionCompositionProducerV3::Sample {
            query_text: "auth".to_owned(),
            mode,
            limit: 7,
            scope: None,
        };
        sample_digests.push(contract_of(vec![candidate]));
    }
    assert_all_digests_distinct(&sample_digests);
}
fn authored_projector_facts(source: &str) -> SemanticDocumentV2 {
    use quanta_gqlang_v2_sema::{
        resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityBuilderV2,
        FrontendAuthorityLocationV2, FrontendAuthoritySourceV2, FrontendBudgetV2,
        FrontendCallableParameterInputV2, FrontendCatalogAssetInputV2, FrontendCatalogPayloadV2,
        FrontendLanguageVersionV2, FrontendRelationAuthorityV2, FrontendRelationCardinalityV2,
        FrontendRelationFieldInputV2, FrontendRelationInputV2, FrontendScalarTypeV2,
    };
    let location = FrontendAuthorityLocationV2 {
        source_id: SourceIdV2::from_raw(1),
        start: 0,
        end: 1,
    };
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let mut builder = ClosedFrontendAuthorityBuilderV2::try_new(
        budget,
        vec![FrontendAuthoritySourceV2 {
            source_id: location.source_id,
            source_len: 64,
        }],
        "app".to_owned(),
        location,
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("closed builder");
    builder.add_relation(FrontendRelationInputV2 {
        authority: FrontendRelationAuthorityV2::Host,
        provenance_namespace: "host".to_owned(),
        qualified_name: "app::records".to_owned(),
        lookup_name: "records".to_owned(),
        fields: vec![FrontendRelationFieldInputV2 {
            name: "id".to_owned(),
            scalar_type: FrontendScalarTypeV2::Int,
            sequence: false,
            optional: false,
            location,
        }],
        cardinality: FrontendRelationCardinalityV2::Bag,
        provenance: "projector test".to_owned(),
        location,
    });
    builder.add_catalog_asset(FrontendCatalogAssetInputV2 {
        module_name: "app".to_owned(),
        local_name: "host_pred".to_owned(),
        payload: FrontendCatalogPayloadV2::Rule {
            parameters: vec![FrontendCallableParameterInputV2 {
                name: "value".to_owned(),
                scalar_type: FrontendScalarTypeV2::Int,
                optional: false,
                location,
            }],
            result: FrontendScalarTypeV2::Bool,
        },
        provenance: "projector test".to_owned(),
        location,
    });
    let seed = builder.finish().expect("closed fixture seed");
    // A Rule catalog payload cannot satisfy a source predicate call. The
    // curated builder has no Predicate input, so seal that test authority
    // through the modules owner with the SAME registry and budget profile.
    let authority = {
        use quanta_gqlang_v2_modules::{
            CallableKindV2, CurrentModuleSelectionInputV2, ExternalCatalogAssetInputV2,
            ExternalCatalogPayloadInputV2, LocatedParameterInputV2, ModuleAuthorityBuilderV2,
            ModuleIdV2, ModuleNodeInputV2, SignatureInputV2, VersionV2,
        };
        use quanta_gqlang_v2_sema::{ClosedFrontendAuthorityV2, LocatedSpanV2, SpanV1};
        let host = seed.modules().source_registry().clone();
        let checked_location = host
            .try_check_location(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(1),
                span: SpanV1::new(0, 1),
            })
            .expect("checked fixture location");
        let effects = seed
            .modules()
            .external_catalog()
            .iter()
            .find_map(|asset| {
                if let quanta_gqlang_v2_modules::ExternalCatalogPayloadV2::Rule(signature) =
                    asset.payload()
                {
                    Some(signature.effects())
                } else {
                    None
                }
            })
            .expect("seed callable effects");
        let module = ModuleIdV2::try_new("app", checked_location.clone()).expect("app module");
        let mut modules = ModuleAuthorityBuilderV2::try_new(
            seed.budget()
                .authority_profile()
                .expect("authority profile"),
            host,
        )
        .expect("modules builder");
        modules
            .try_add_module(ModuleNodeInputV2 {
                module_id: module.clone(),
                location: checked_location.clone(),
            })
            .expect("app module added");
        modules
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: "app".to_owned(),
                local_name: "host_pred".to_owned(),
                kind_token: "predicate".to_owned(),
                typed_payload: ExternalCatalogPayloadInputV2::Predicate(SignatureInputV2 {
                    parameters: vec![LocatedParameterInputV2 {
                        name: "value".to_owned(),
                        value_type: ValueTypeV1::int(),
                        location: checked_location.clone(),
                    }],
                    result: ValueTypeV1::bool(),
                    callable_kind: CallableKindV2::Predicate,
                    effects,
                    location: checked_location.clone(),
                }),
                provenance: "projector test".to_owned(),
                location: checked_location.clone(),
                dependencies: Vec::new(),
            })
            .expect("host predicate added");
        let modules = modules
            .finish(
                CurrentModuleSelectionInputV2 {
                    module_id: module,
                    location: checked_location,
                },
                VersionV2::new(1, 0, 0),
            )
            .expect("predicate module authority seals");
        ClosedFrontendAuthorityV2::try_new(
            modules,
            seed.relations().clone(),
            seed.graph_references().clone(),
            seed.budget().clone(),
        )
        .expect("shared registry and profile authority seals")
    };
    let parsed = quanta_gqlang_v2_surface::parse_source_document_v2(
        SourceIdV2::from_raw(9),
        source,
        &quanta_gqlang_v2_surface::ParseBudgetV2::standard_v2(),
    )
    .expect("authored projector source parses");
    let checked = CheckedSyntaxDocumentV2::try_from_hir(parsed.hir).expect("checked HIR");
    resolve_checked_document_v2(checked, &authority).expect("authored source resolves")
}

#[test]
fn quantified_shape_retains_kind_binder_identities_and_body() {
    use quanta_gqlang_v2_sema::{ResolvedBindingKindV2, ResolvedValueBindingV2};
    // Logical quantifiers are currently rule-only, and source rule members
    // issue no Execution item. Pin this private projector's full owner-fact
    // payload without claiming an active source execution path.
    let first = ResolvedValueBindingV2::try_new(
        SourceIdV2::from_raw(9),
        HirNodeIdV1::from_raw(71),
        2,
        ResolvedBindingKindV2::Quantified,
        "left",
    )
    .expect("first quantified identity");
    let second = ResolvedValueBindingV2::try_new(
        SourceIdV2::from_raw(9),
        HirNodeIdV1::from_raw(72),
        3,
        ResolvedBindingKindV2::Quantified,
        "right",
    )
    .expect("second quantified identity");
    for (kind, expected_kind) in [
        (
            LogicalQuantifierKindV2::Exists,
            ExecutionLogicalQuantifierV3::Exists,
        ),
        (
            LogicalQuantifierKindV2::Forall,
            ExecutionLogicalQuantifierV3::Forall,
        ),
    ] {
        assert_eq!(
            project_expression_shape(&TypedExprShapeV2::LogicalQuantifier {
                kind,
                binders: vec![first.clone(), second.clone()],
                body: key(9, 83),
            })
            .expect("closed quantified shape"),
            ExecutionExpressionShapeV3::LogicalQuantifier {
                kind: expected_kind,
                binders: Box::new([
                    ExecutionBindingV3 {
                        source_id: SourceIdV2::from_raw(9),
                        owner_node: 71,
                        ordinal: 2,
                        name: "left".to_owned(),
                    },
                    ExecutionBindingV3 {
                        source_id: SourceIdV2::from_raw(9),
                        owner_node: 72,
                        ordinal: 3,
                        name: "right".to_owned(),
                    },
                ]),
                body: key(9, 83),
            },
        );
    }
}

#[test]
fn authored_exists_shape_retains_polarity_nested_identity_and_capture() {
    for (source, expected_polarity) in [
        (
            "from records r | where exists (from records s | where s.id = r.id)",
            ExecutionExistsPolarityV3::Exists,
        ),
        (
            "from records r | where not exists (from records s | where s.id = r.id)",
            ExecutionExistsPolarityV3::NotExists,
        ),
    ] {
        let document = authored_projector_facts(source);
        let shapes = document
            .expressions()
            .filter_map(|(_, expression)| {
                if let TypedExprShapeV2::RelationExists {
                    relation_root,
                    correlations,
                    ..
                } = expression.shape()
                {
                    Some((expression.shape(), *relation_root, correlations))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        let [(shape, relation_root, captures)] = shapes.as_slice() else {
            panic!("one authored EXISTS semantic fact expected");
        };
        let [capture] = captures.as_slice() else {
            panic!("one authored outer-r capture expected");
        };
        assert_eq!(capture.binding().canonical_name(), "r");
        assert_eq!(relation_root.source_id, SourceIdV2::from_raw(9));
        assert!(document.nested_relation(*relation_root).is_some());
        // EXISTS is lowered to SemiJoin/AntiJoin on the active source path;
        // its typed expression shape currently has no retained DTO root.
        // This checks the dormant shape projector against sealed source facts.
        assert_eq!(
            project_expression_shape(shape).expect("closed EXISTS shape"),
            ExecutionExpressionShapeV3::RelationExists {
                polarity: expected_polarity,
                relation_root: *relation_root,
                correlations: Box::new([ExecutionCorrelationV3 {
                    binding: ExecutionBindingV3 {
                        source_id: SourceIdV2::from_raw(9),
                        owner_node: capture.binding().target_node().raw(),
                        ordinal: capture.binding().ordinal(),
                        name: "r".to_owned(),
                    },
                    reference: capture.reference(),
                }]),
            },
        );
    }
}

#[test]
fn authored_local_and_host_calls_retain_target_identity_and_argument_keys() {
    let source =
        "rule local_pred : x if x = 1; rule caller : x if x = 1 and local_pred(x) and host_pred(x)";
    let document = authored_projector_facts(source);
    let mut local_calls = 0;
    let mut host_calls = 0;
    for (_, expression) in document.expressions() {
        let TypedExprShapeV2::Call { target, arguments } = expression.shape() else {
            continue;
        };
        let [TypedCallArgumentV2::Expression(argument)] = arguments.as_slice() else {
            panic!("each authored predicate takes exactly one expression argument");
        };
        assert_eq!(argument.source_id, SourceIdV2::from_raw(9));
        let argument_fact = document
            .expression(*argument)
            .expect("authored argument fact");
        let TypedExprShapeV2::Binding(binding) = argument_fact.shape() else {
            panic!("the authored argument is the caller's x binding");
        };
        assert_eq!(binding.canonical_name(), "x");
        let expected_target = match target {
            ResolvedCallTargetV2::Predicate(_) => {
                local_calls += 1;
                ExecutionCallTargetV3::Predicate {
                    name: "local_pred".to_owned(),
                }
            }
            ResolvedCallTargetV2::HostPredicate(_) => {
                host_calls += 1;
                // Independent literal for prefix + len(app) + app + Predicate(1)
                // + len(host_pred) + host_pred. Do not derive via hex_lower
                // or canonical_bytes: both are part of the production seam.
                ExecutionCallTargetV3::HostPredicate {
                        fingerprint: "67716c616e672d686f73742d7072656469636174652d7632000000000000000003617070010000000000000009686f73745f70726564".to_owned(),
                    }
            }
            ResolvedCallTargetV2::Builtin(_) => panic!("no authored builtin"),
        };
        assert_eq!(
            project_expression_shape(expression.shape()).expect("closed call shape"),
            ExecutionExpressionShapeV3::Call {
                target: expected_target,
                arguments: Box::new([Some(*argument)]),
            },
        );
    }
    assert_eq!((local_calls, host_calls), (1, 1));
}

#[test]
fn source_match_binds_exact_utf8_bytes_without_recompilation_v3() {
    let source = "show capabilities // 한글";
    let document = ExecutionDocumentV3::new(SourceIdV2::from_raw(2), source.as_bytes(), Vec::new());
    assert!(document.matches_source_bytes_v3(source.as_bytes()));
    for foreign in [
        "show capabilities",
        "show capabilities // 한글 ",
        "show models // 한글",
        "",
    ] {
        assert!(!document.matches_source_bytes_v3(foreign.as_bytes()));
    }
}
