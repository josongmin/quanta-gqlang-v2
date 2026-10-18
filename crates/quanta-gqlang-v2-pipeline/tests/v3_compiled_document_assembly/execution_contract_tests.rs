use super::*;

fn resolve_source(
    authority: &ClosedFrontendAuthorityV2,
    source: &str,
) -> quanta_gqlang_v2_sema::SemanticDocumentV2 {
    let parsed = quanta_gqlang_v2_surface::parse_source_document_v2(
        DOC_SOURCE,
        source,
        &quanta_gqlang_v2_surface::ParseBudgetV2::standard_v2(),
    )
    .expect("source parses");
    let checked = CheckedSyntaxDocumentV2::try_from_hir(parsed.hir).expect("HIR integrity holds");
    quanta_gqlang_v2_sema::resolve_checked_document_v2(checked, authority)
        .expect("semantic transaction succeeds")
}

#[test]
fn composition_exists_source_uses_compile_without_false_correlations() {
    use quanta_gqlang_v2_pipeline::ExecutionExistsPolarityV3::{Exists, NotExists};

    for (predicate, family, polarities) in [
        ("exists (from sampled t)", AlgebraFamilyV2::SemiJoin, vec![]),
        (
            "not exists (from sampled t)",
            AlgebraFamilyV2::AntiJoin,
            vec![],
        ),
        (
            "not (exists (from sampled t))",
            AlgebraFamilyV2::AntiJoin,
            vec![],
        ),
        (
            "exists (from sampled t) and exists (from sampled u)",
            AlgebraFamilyV2::ExistsFilter,
            vec![Exists, Exists],
        ),
        (
            "exists (from sampled t) and not (exists (from sampled u))",
            AlgebraFamilyV2::ExistsFilter,
            vec![Exists, NotExists],
        ),
    ] {
        let source = format!(
            "let sampled = sample query=\"auth\" | mode=lexical | limit=2 \
             from sampled s | where {predicate} | project s.path as path with sampled"
        );
        let compiled = compile_source_document_v3(DOC_SOURCE, &source, &minimal_authority())
            .expect("LET relation sources inside EXISTS compile");
        let again = compile_source_document_v3(DOC_SOURCE, &source, &minimal_authority())
            .expect("repeat compilation succeeds");
        assert_eq!(compiled.execution(), again.execution());
        let composition = compiled.execution().items()[0]
            .composition()
            .expect("composition");
        let relation = composition.body_relation().expect("relational body");
        assert_eq!(relation.root().family(), AlgebraFamilyV2::Project);
        let exists = &relation.root().children()[0];
        assert_eq!(exists.family(), family);
        assert!(exists.correlations().is_empty());
        assert_eq!(
            exists
                .existential_terms()
                .iter()
                .map(|term| term.polarity())
                .collect::<Vec<_>>(),
            polarities
        );
        assert!(exists
            .existential_terms()
            .iter()
            .all(|term| term.correlations().is_empty()));
        for child in exists.children() {
            assert_eq!(child.family(), AlgebraFamilyV2::Scan);
            let source = child.source().expect("each child is a LET scan");
            assert_eq!(
                source.authority(),
                ExecutionRelationAuthorityV3::CompositionBinding
            );
            assert_eq!(source.name(), "sampled");
            assert_eq!(
                source.composition_binding(),
                Some(composition.bindings()[0].binding()),
                "scan retains the exact selected LET slot independently of row capture"
            );
        }
    }
}

#[test]
fn composition_exists_retains_only_actual_outer_row_captures() {
    for polarity in ["exists", "not exists"] {
        let source = format!(
            "let sampled = sample query=\"auth\" | mode=lexical | limit=2 \
             from sampled s | where {polarity} (from sampled t | where t.path = s.path) \
             | project s.path as path with sampled"
        );
        let compiled = compile_source_document_v3(DOC_SOURCE, &source, &minimal_authority())
            .expect("actual outer row capture compiles");
        let relation = compiled.execution().items()[0]
            .composition()
            .expect("composition")
            .body_relation()
            .expect("body");
        let exists = &relation.root().children()[0];
        assert_eq!(exists.correlations().len(), 1);
        let capture = &exists.correlations()[0];
        assert_eq!(capture.binding().name(), "s");
        let reference = relation
            .expressions()
            .iter()
            .find(|expression| expression.id() == capture.reference())
            .expect("capture references a real expression");
        assert!(
            matches!(reference.shape(), ExecutionExpressionShapeV3::Binding(binding)
            if binding == capture.binding())
        );
        assert_eq!(
            capture.binding().owner_node(),
            exists.children()[0].stage().node_id.raw()
        );
    }
}

#[test]
fn composition_flat_mixed_exists_retains_each_capture_and_residual_filter() {
    use quanta_gqlang_v2_pipeline::ExecutionExistsPolarityV3::{Exists, NotExists};

    let source = concat!(
        "let sampled = sample query=\"auth\" | mode=lexical | limit=2 ",
        "from sampled s | where false and exists ",
        "(from sampled t | where t.path = s.path) and not ",
        "(exists (from sampled u | where u.path = s.path)) ",
        "| project s.path as path with sampled"
    );
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &minimal_authority())
        .expect("two correlated LET scans and the residual predicate compile");
    let composition = compiled.execution().items()[0]
        .composition()
        .expect("composition");
    let relation = composition.body_relation().expect("relational body");
    assert_eq!(relation.root().family(), AlgebraFamilyV2::Project);
    let [residual] = relation.root().children() else {
        panic!("project has one residual filter child");
    };
    assert_eq!(residual.family(), AlgebraFamilyV2::Filter);
    let [predicate] = residual.predicates() else {
        panic!("the residual false predicate is retained exactly once");
    };
    let residual_expression = relation
        .expressions()
        .iter()
        .find(|expression| expression.id() == *predicate)
        .expect("residual expression is closed");
    let ExecutionExpressionShapeV3::BoolValue(literal) = residual_expression.shape() else {
        panic!("residual preserves the authored Bool wrapper");
    };
    assert!(relation.expressions().iter().any(|expression| {
        expression.id() == *literal
            && expression.shape()
                == &ExecutionExpressionShapeV3::Literal(ExecutionLiteralV3::Bool(false))
    }));

    let [flat] = residual.children() else {
        panic!("one flat existential operator is below the residual filter");
    };
    assert_eq!(flat.family(), AlgebraFamilyV2::ExistsFilter);
    assert_eq!(flat.stage(), residual.stage());
    assert_eq!(flat.children().len(), 3);
    assert_eq!(flat.existential_terms().len(), 2);
    assert_eq!(
        flat.existential_terms()
            .iter()
            .map(|term| term.polarity())
            .collect::<Vec<_>>(),
        vec![Exists, NotExists]
    );
    let outer_scan = &flat.children()[0];
    assert_eq!(outer_scan.family(), AlgebraFamilyV2::Scan);
    let expected_binding = composition.bindings()[0].binding();
    for scan in std::iter::once(outer_scan).chain(flat.children()[1..].iter().map(|right| {
        assert_eq!(right.family(), AlgebraFamilyV2::Filter);
        let [scan] = right.children() else {
            panic!("each right predicate filters one LET scan");
        };
        scan
    })) {
        assert_eq!(scan.family(), AlgebraFamilyV2::Scan);
        let source = scan.source().expect("LET scan source");
        assert_eq!(
            source.authority(),
            ExecutionRelationAuthorityV3::CompositionBinding
        );
        assert_eq!(source.name(), "sampled");
        assert_eq!(source.composition_binding(), Some(expected_binding));
    }
    let mut capture_references = BTreeSet::new();
    for (term, right) in flat.existential_terms().iter().zip(&flat.children()[1..]) {
        assert_eq!(term.correlations().len(), 1);
        let capture = &term.correlations()[0];
        assert_eq!(capture.binding().name(), "s");
        assert_eq!(
            capture.binding().owner_node(),
            outer_scan.stage().node_id.raw()
        );
        assert!(capture_references.insert(capture.reference()));
        assert!(relation.expressions().iter().any(|expression| {
            expression.id() == capture.reference()
                && matches!(expression.shape(), ExecutionExpressionShapeV3::Binding(binding)
                    if binding == capture.binding())
        }));
        let expression = relation
            .expressions()
            .iter()
            .find(|expression| expression.id() == term.expression())
            .expect("each term has a closed relation-exists expression");
        assert!(
            matches!(expression.shape(), ExecutionExpressionShapeV3::RelationExists {
            relation_root, correlations, ..
        } if *relation_root == right.stage() && correlations.as_ref() == term.correlations())
        );
    }
    assert_eq!(capture_references.len(), 2);
}

#[test]
fn composition_nested_exists_retains_each_crossed_row_boundary() {
    let source = concat!(
        "let sampled = sample query=\"auth\" | mode=lexical | limit=2 ",
        "from sampled s | where exists (from sampled t | where not exists ",
        "(from sampled u | where u.path = s.path and u.path = t.path)) ",
        "| project s.path as path with sampled"
    );
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &minimal_authority())
        .expect("nested LET scans retain only expression correlations");
    let relation = compiled.execution().items()[0]
        .composition()
        .expect("composition")
        .body_relation()
        .expect("body");
    let outer = &relation.root().children()[0];
    assert_eq!(outer.family(), AlgebraFamilyV2::SemiJoin);
    assert_eq!(
        outer
            .correlations()
            .iter()
            .map(|edge| edge.binding().name())
            .collect::<Vec<_>>(),
        vec!["s"]
    );
    let inner = &outer.children()[1];
    assert_eq!(inner.family(), AlgebraFamilyV2::AntiJoin);
    assert_eq!(
        inner
            .correlations()
            .iter()
            .map(|edge| edge.binding().name())
            .collect::<Vec<_>>(),
        vec!["s", "t"]
    );
    for edge in outer.correlations().iter().chain(inner.correlations()) {
        let expression = relation
            .expressions()
            .iter()
            .find(|expression| expression.id() == edge.reference())
            .expect("nested capture has a real expression");
        assert!(
            matches!(expression.shape(), ExecutionExpressionShapeV3::Binding(binding) if binding == edge.binding())
        );
    }
}

#[test]
fn query_producer_composition_exists_uses_have_no_source_captures() {
    for predicate in [
        "exists (from rows t)",
        "exists (from rows t) and not exists (from rows u)",
    ] {
        let source = format!(
            "let rows = from records r from rows x | where {predicate} | project x.id as id with rows"
        );
        let compiled = compile_source_document_v3(DOC_SOURCE, &source, &authority_with_records())
            .expect("Query-produced LET scans inside EXISTS compile");
        let relation = compiled.execution().items()[0]
            .composition()
            .expect("composition")
            .body_relation()
            .expect("body");
        let exists = &relation.root().children()[0];
        assert!(exists.correlations().is_empty());
        assert!(exists
            .existential_terms()
            .iter()
            .all(|term| term.correlations().is_empty()));
    }
}

#[test]
fn composition_producer_and_with_identity_are_assembled_exactly() {
    let authority = minimal_authority();
    let compiled = compile_source_document_v3(DOC_SOURCE, &composition_source(2), &authority)
        .expect("the composition compiles");
    assert_eq!(compiled.items().len(), 1);
    let execution = compiled.execution().items()[0]
        .composition()
        .expect("composition carries ordered execution DTO");
    assert_eq!(execution.bindings().len(), 1);
    assert_eq!(execution.bindings()[0].binding().ordinal(), 0);
    assert_eq!(execution.with_binding(), execution.bindings()[0].binding());
    assert!(execution.body_relation().is_some());
    assert!(matches!(
        execution.bindings()[0].producer(),
        ExecutionCompositionProducerV3::Sample {
            mode: ExecutionSampleModeV3::Lexical,
            limit: 2,
            ..
        }
    ));
    assert_eq!(execution.body(), &ExecutionCompositionBodyV3::Relation);
    let semantic_source = composition_source(2).replace("mode=lexical", "mode=semantic");
    let semantic = compile_source_document_v3(DOC_SOURCE, &semantic_source, &authority)
        .expect("semantic SAMPLE composition compiles");
    let semantic_execution = semantic.execution().items()[0]
        .composition()
        .expect("semantic SAMPLE carries composition DTO");
    assert!(matches!(
        semantic_execution.bindings()[0].producer(),
        ExecutionCompositionProducerV3::Sample {
            mode: ExecutionSampleModeV3::Semantic,
            ..
        }
    ));
    assert_ne!(
        compiled.execution().contract_digest(),
        semantic.execution().contract_digest()
    );
    let CompiledItemV2::Planned(planned) = &compiled.items()[0] else {
        panic!("a composition item must compile to a planned composition item");
    };
    let PlannedItemV2::Composition { item_id, plan } = planned.as_ref() else {
        panic!("a composition item retains its composition plan");
    };
    let document = resolve_source(&authority, &composition_source(2));
    let SemanticItemV2::Composition(composition) = document
        .item(*item_id)
        .expect("the composition identity is the document's own item")
    else {
        panic!("the authored item is a composition");
    };
    // The plan's retained composition policy is the sealed composition's own
    // payload: root identity, ordered producer facts, exact WITH binding, and
    // the sealed body identity — every one asserted against the DOCUMENT, never
    // against the plan's own accessor.
    let PlanPolicyV2::Composition(retained) = plan.policy() else {
        panic!("a composition plan must retain the composition policy");
    };
    assert_eq!(composition.bindings().len(), 1);
    let binding = composition.bindings().first().expect("one sealed binding");
    assert_eq!(binding.binding().ordinal(), 0);
    assert_eq!(binding.name(), "sampled");
    assert_eq!(
        composition.with_target(),
        binding.binding(),
        "the WITH target must be exactly one ordered producer"
    );
    assert_ne!(
        composition.body().item_id(),
        composition.item_id(),
        "the body keeps its own identity and is never a second top-level plan key"
    );
    assert_eq!(retained.root_item_id(), *item_id);
    assert_eq!(retained.location(), composition.location());
    assert_eq!(retained.producers().len(), 1);
    let producer = retained.producers().first().expect("one ordered producer");
    assert_eq!(producer.ordinal(), 0);
    assert_eq!(producer.item_id(), binding.producer().item_id());
    assert_eq!(producer.name_location(), binding.name_location());
    assert_eq!(producer.relation_output(), binding.relation_output());
    assert_eq!(retained.with_binding(), composition.with_target());
    assert_eq!(retained.body_item_id(), composition.body().item_id());
    assert_eq!(retained.body_family(), CompositionBodyFamilyV2::Query);
    assert_ne!(retained.body_item_id(), retained.root_item_id());

    // The plan is attributed to the COMPOSITION root and carries the body's own
    // relational policy (a working-set scan plus one projection), never a plan
    // per `LET` producer.
    assert_eq!(plan.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(plan.evidence(), EvidenceRequirementV2::None);
    assert_eq!(plan.effect(), EffectClassV2::ReadOnly);
    assert_eq!(plan.result().result_kind(), ResultKindV2::Table);
    assert_eq!(
        plan.result().render_requirement(),
        RenderRequirementV2::RowsOnly
    );
    assert_eq!(plan.result().witness_selection(), None);
    assert_eq!(
        plan.capabilities(),
        &BTreeSet::from([CapabilityV2::RelationScan])
    );
    assert_eq!(
        plan.semantic_cost(),
        &SemanticCostV2 {
            estimated_cardinality_class: CardinalityClassV2::LinearInInput,
            recursion_risk_class: RecursionRiskClassV2::None,
            witness_explosion_risk_class: WitnessExplosionRiskClassV2::None,
            ordering_contract: OrderingContractV2::Unordered,
        }
    );
    assert_eq!(plan.explain().root_location(), composition.location());
    assert_eq!(
        plan.explain().families(),
        [AlgebraFamilyV2::Project, AlgebraFamilyV2::Scan],
        "root-first visit order: the projection over its `records` scan"
    );
    assert!(!plan.explain().recursive());
    let PlanPolicyV2::Relational { nodes, .. } = retained.body().policy() else {
        panic!("a relational composition body keeps the body's own relational policy");
    };
    assert_eq!(*nodes, 2, "the working-set scan and its projection");
    assert_eq!(retained.body().family(), CompositionBodyFamilyV2::Query);
    assert_eq!(document.items().count(), 1);

    // The body contract reaches the plan: a composition whose body is a FIND
    // EXPLAIN dispatches through a different sealed arm, so the assembled
    // payload cannot stay identical to a relational body's.
    let explain_body = "let paths = find path a -> b | depth=1 \
                        explain paths | evidence=auto | truth=inspect with paths";
    let other = compile_source_document_v3(DOC_SOURCE, explain_body, &authority_with_records())
        .expect("the EXPLAIN-body composition compiles");
    assert_ne!(
        compiled.items(),
        other.items(),
        "a swapped composition body arm must change the assembled plan"
    );
}

#[test]
fn graph_composition_producer_is_logic_checked_and_fully_projected() {
    let authority = source_policy_authority();
    let unbounded = "let paths = find path api.entry -> db.write \
                     find reachability paths -> db.write | depth=1 with paths";
    let error = compile_source_document_v3(DOC_SOURCE, unbounded, &authority)
        .expect_err("a graph producer cannot bypass the logic boundedness gate");
    assert_eq!(
        failure_code(&error),
        "logic.unbounded_witness.missing_path_length_bound"
    );

    let source = "let paths = find path api.entry -> db.write \
                  | via=mid.one | direction=backward | depth=2 \
                  find reachability paths -> db.write | direction=both | depth=3 with paths";
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &authority)
        .expect("bounded graph producer and body compile");
    let composition = compiled.execution().items()[0]
        .composition()
        .expect("composition execution payload");
    let ExecutionCompositionProducerV3::Graph { query: producer } =
        composition.bindings()[0].producer()
    else {
        panic!("graph producer payload expected")
    };
    assert_eq!(producer.kind(), ExecutionGraphKindV3::Path);
    assert_eq!(producer.depth(), 2);
    assert_eq!(
        producer.direction(),
        Some(ExecutionGraphDirectionV3::Backward)
    );
    assert_eq!(producer.max_witnesses(), 10_000);
    assert_eq!(
        producer.relation_shape(),
        ExecutionGraphRelationShapeV3::Cyclic
    );
    assert_eq!(
        producer.dedup_identity(),
        ExecutionGraphDedupIdentityV3::NodeSequence
    );
    assert_eq!(producer.max_path_length(), Some(2));
    assert_eq!(
        producer.witness_tie_break(),
        ExecutionGraphWitnessTieBreakV3::ShortestThenLexical
    );
    assert_eq!(
        producer
            .references()
            .iter()
            .map(|reference| reference.role())
            .collect::<Vec<_>>(),
        [
            ExecutionGraphReferenceRoleV3::Source,
            ExecutionGraphReferenceRoleV3::Sink,
            ExecutionGraphReferenceRoleV3::Via,
        ]
    );
    let ExecutionCompositionBodyV3::Graph(body) = composition.body() else {
        panic!("graph body payload expected")
    };
    assert_eq!(body.kind(), ExecutionGraphKindV3::Reachability);
    assert_eq!(body.depth(), 3);
    assert_eq!(body.direction(), Some(ExecutionGraphDirectionV3::Both));
    assert_eq!(
        body.dedup_identity(),
        ExecutionGraphDedupIdentityV3::Endpoints
    );
    assert_eq!(body.max_path_length(), Some(3));
    // Full role/target retention: the composition source and the host sink,
    // not merely the first target's shape.
    assert_eq!(
        body.references().len(),
        2,
        "body carries source and sink only"
    );
    assert_eq!(
        body.references()[0].role(),
        ExecutionGraphReferenceRoleV3::Source
    );
    assert_eq!(
        body.references()[0].target(),
        &ExecutionGraphTargetV3::Composition {
            binding: composition.bindings()[0].binding().clone(),
            capability: ExecutionGraphCapabilityV3::Entity,
        }
    );
    assert_eq!(
        body.references()[1].role(),
        ExecutionGraphReferenceRoleV3::Sink
    );
    assert_eq!(
        body.references()[1].target(),
        &ExecutionGraphTargetV3::Host {
            kind: ExecutionGraphReferenceKindV3::Entity,
            spelling: "db.write".to_owned(),
            entity_subtype: None,
        }
    );
    assert_eq!(
        composition.with_binding(),
        composition.bindings()[0].binding()
    );
}

#[test]
fn typed_relation_source_requires_and_retains_host_authority() {
    let compiled = compile_source_document_v3(DOC_SOURCE, RECORD_SOURCE, &authority_with_records())
        .expect("an admitted host relation compiles");
    assert_eq!(compiled.items().len(), 1);
    let CompiledItemV2::Planned(planned) = &compiled.items()[0] else {
        panic!("a typed relation compiles to a planned relation item");
    };
    let PlannedItemV2::Relation { plan, .. } = planned.as_ref() else {
        panic!("a typed relation retains its relation plan");
    };

    // FIELD assertions over the host `records` scan + one projection: the
    // retained policy names the two visited algebra nodes, and every derived
    // lane is the exact class that shape implies.
    assert_eq!(plan.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(
        plan.evaluation_determinism(),
        EvaluationDeterminismV2::Deterministic
    );
    assert_eq!(plan.evidence(), EvidenceRequirementV2::None);
    assert_eq!(plan.effect(), EffectClassV2::ReadOnly);
    assert_eq!(plan.result().result_kind(), ResultKindV2::Table);
    assert_eq!(
        plan.result().render_requirement(),
        RenderRequirementV2::RowsOnly
    );
    assert_eq!(plan.result().witness_selection(), None);
    assert_eq!(
        plan.capabilities(),
        &BTreeSet::from([CapabilityV2::RelationScan])
    );
    assert_eq!(
        plan.semantic_cost(),
        &SemanticCostV2 {
            estimated_cardinality_class: CardinalityClassV2::LinearInInput,
            recursion_risk_class: RecursionRiskClassV2::None,
            witness_explosion_risk_class: WitnessExplosionRiskClassV2::None,
            ordering_contract: OrderingContractV2::Unordered,
        }
    );
    assert_eq!(
        plan.explain().families(),
        [AlgebraFamilyV2::Project, AlgebraFamilyV2::Scan],
        "root-first visit order: the projection over its `records` scan"
    );
    assert!(!plan.explain().recursive());
    let PlanPolicyV2::Relational { nodes, .. } = plan.policy() else {
        panic!("a relational item must retain the relational policy");
    };
    assert_eq!(*nodes, 2, "the `records` scan and its projection");

    let error = compile_source_document_v3(DOC_SOURCE, RECORD_SOURCE, &minimal_authority())
        .expect_err("an absent host relation must reject");
    assert_eq!(failure_code(&error), "sema.unresolved_reference");
}

#[test]
fn execution_contract_is_complete_deterministic_and_mutation_sensitive() {
    let authority = authority_with_records();
    let compiled = compile_source_document_v3(DOC_SOURCE, RECORD_SOURCE, &authority)
        .expect("relation execution contract compiles");
    let repeated = compile_source_document_v3(DOC_SOURCE, RECORD_SOURCE, &authority)
        .expect("same source compiles deterministically");
    assert_eq!(compiled.execution(), repeated.execution());
    assert_eq!(compiled.execution().items().len(), 1);
    let item = &compiled.execution().items()[0];
    assert_eq!(item.family(), ExecutionFamilyV3::Relation);
    assert_eq!(
        item.plan().authority(),
        ExecutionAuthorityV3::FrontendChecked
    );
    let relation = item
        .relation()
        .expect("relation item carries operator tree");
    assert_eq!(relation.root().family(), AlgebraFamilyV2::Project);
    assert_eq!(relation.root().join_mode(), None);
    assert_eq!(
        relation.root().children()[0].family(),
        AlgebraFamilyV2::Scan
    );
    assert_eq!(relation.root().schema().fields()[0].name(), "id");
    assert_eq!(
        relation.root().schema().fields()[0].value_type(),
        &ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int)
    );
    assert_eq!(
        relation.root().schema().cardinality(),
        ExecutionCardinalityV3::Bag
    );
    assert!(relation.expressions().iter().any(|expression| matches!(
        expression.shape(),
        ExecutionExpressionShapeV3::Path { .. } | ExecutionExpressionShapeV3::Binding(_)
    )));
    assert_ne!(
        compiled.execution().source_digest().to_hex(),
        "0".repeat(64)
    );

    let mutated = compile_source_document_v3(
        DOC_SOURCE,
        "from records r | project r.id as renamed",
        &authority,
    )
    .expect("schema mutation remains valid source");
    assert_ne!(
        compiled.execution().contract_digest(),
        mutated.execution().contract_digest()
    );
    assert_ne!(
        compiled.execution().qualification_digest(),
        mutated.execution().qualification_digest()
    );
    assert_ne!(
        compiled.execution().source_digest(),
        mutated.execution().source_digest()
    );

    let byte_mutated = compile_source_document_v3(
        DOC_SOURCE,
        "from records r  | project r.id as id",
        &authority,
    )
    .expect("whitespace-only byte mutation remains valid source");
    assert_ne!(
        compiled.execution().source_digest(),
        byte_mutated.execution().source_digest()
    );
    assert_ne!(
        compiled.execution().qualification_digest(),
        byte_mutated.execution().qualification_digest()
    );
    // The DTO is whitespace-insensitive: identical contracts share one digest
    // (the source and qualification legs still move, asserted above).
    assert_eq!(
        compiled.execution().contract_digest(),
        byte_mutated.execution().contract_digest()
    );
    assert_eq!(
        compiled.execution().items(),
        byte_mutated.execution().items()
    );
}

fn projected_literal(source: &str) -> ExecutionLiteralV3 {
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &authority_with_records())
        .expect("checked literal source compiles");
    let relation = compiled.execution().items()[0]
        .relation()
        .expect("one top-level relation");
    relation
        .expressions()
        .iter()
        .find_map(|expression| {
            if let ExecutionExpressionShapeV3::Literal(literal) = expression.shape() {
                Some(literal.clone())
            } else {
                None
            }
        })
        .expect("the checked literal is projected into the execution table")
}

#[test]
fn source_literals_reach_the_v3_contract_as_checked_typed_values() {
    assert_eq!(
        projected_literal("from records r | project 42 as x"),
        ExecutionLiteralV3::Int(42)
    );
    assert_eq!(
        projected_literal("from records r | project -170141183460469231731687303715884105728 as x"),
        ExecutionLiteralV3::Int(i128::MIN)
    );
    let float = projected_literal("from records r | project 1.5 as x");
    assert!(matches!(
        float,
        ExecutionLiteralV3::Float(value) if value.bits() == 1.5f64.to_bits()
    ));
    let negative_zero = projected_literal("from records r | project -0.0 as x");
    assert!(matches!(
        negative_zero,
        ExecutionLiteralV3::Float(value) if value.bits() == 0.0f64.to_bits()
    ));
    assert_eq!(
        projected_literal("from records r | project true as x"),
        ExecutionLiteralV3::Bool(true)
    );
    assert_eq!(
        projected_literal("from records r | project \"é\" as x"),
        ExecutionLiteralV3::String("é".to_owned())
    );
    assert_eq!(
        projected_literal("from records r | project null as x"),
        ExecutionLiteralV3::Null
    );
}

#[test]
fn i128_min_projects_as_the_root_literal_without_a_second_negation() {
    let source = "from records r | project -170141183460469231731687303715884105728 as x";
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &authority_with_records())
        .expect("checked i128::MIN source compiles");
    let relation = compiled.execution().items()[0]
        .relation()
        .expect("one top-level relation");
    let root_ids = relation.root().expressions();
    assert_eq!(root_ids.len(), 1, "one authored projection expression");
    let root = relation
        .expressions()
        .iter()
        .find(|expression| expression.id() == root_ids[0])
        .expect("the projection root is present in the closed expression table");
    assert_eq!(
        root.shape(),
        &ExecutionExpressionShapeV3::Literal(ExecutionLiteralV3::Int(i128::MIN)),
        "the checked minimum value must not remain Unary('-', Int(MIN))"
    );
    assert_eq!(
        relation.expressions().len(),
        1,
        "the out-of-range positive magnitude is not an executable child"
    );
}

#[test]
fn source_id_is_transaction_local_and_foreign_item_lookup_is_empty() {
    let authority = authority_with_records();
    let first = compile_source_document_v3(DOC_SOURCE, RECORD_SOURCE, &authority)
        .expect("first source identity compiles");
    let second = compile_source_document_v3(SourceIdV2::from_raw(10), RECORD_SOURCE, &authority)
        .expect("second source identity compiles");
    assert_eq!(
        first.execution().source_digest(),
        second.execution().source_digest()
    );
    assert_ne!(
        first.execution().contract_digest(),
        second.execution().contract_digest()
    );
    assert_ne!(
        first.execution().qualification_digest(),
        second.execution().qualification_digest()
    );
    let foreign_item = second.execution().items()[0].item_id();
    assert!(first.execution().item(foreign_item).is_none());
    assert!(first
        .execution()
        .item(first.execution().items()[0].item_id())
        .is_some());
}

#[test]
fn same_source_under_changed_host_field_type_changes_the_v3_contract() {
    let int_document =
        compile_source_document_v3(DOC_SOURCE, RECORD_SOURCE, &authority_with_records())
            .expect("integer host field compiles");
    let string_document = compile_source_document_v3(
        DOC_SOURCE,
        RECORD_SOURCE,
        &authority_with_records_type(ValueTypeV1::string()),
    )
    .expect("string host field compiles");
    let int_execution = int_document.execution();
    let string_execution = string_document.execution();
    assert_eq!(
        int_execution.source_digest(),
        string_execution.source_digest()
    );
    assert_eq!(
        int_execution.items()[0].item_id(),
        string_execution.items()[0].item_id()
    );
    assert_ne!(
        int_execution.contract_digest(),
        string_execution.contract_digest()
    );
    assert_ne!(
        int_execution.qualification_digest(),
        string_execution.qualification_digest()
    );
    let string_relation = string_execution.items()[0]
        .relation()
        .expect("relation payload");
    assert_eq!(
        string_relation.root().schema().fields()[0].value_type(),
        &ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::String),
    );
    assert!(
        int_execution
            .item(string_execution.items()[0].item_id())
            .is_some(),
        "item lookup compares a document-local raw key, not transaction provenance",
    );
}

#[test]
fn source_bound_relation_depth_admits_exact_limit_and_rejects_without_abort() {
    let authority = authority_with_records();
    let exact = format!(
        "from records r{} | project r.id as id",
        " | where true".repeat(126)
    );
    let compiled = compile_source_document_v3(DOC_SOURCE, &exact, &authority)
        .expect("the exact relation-depth limit admits");
    assert_eq!(compiled.execution().items().len(), 1);

    for depth in [127, 4_096] {
        let over = format!(
            "from records r{} | project r.id as id",
            " | where true".repeat(depth)
        );
        let error = compile_source_document_v3(DOC_SOURCE, &over, &authority)
            .expect_err("relation depth past the checked limit must reject before recursion");
        assert_eq!(failure_code(&error), "surface.resource_exhausted");
        let FrontendTransactionErrorV2::Document(failure) = error else {
            panic!("source resource failure must be source-anchored");
        };
        assert_eq!(failure.diagnostics()[0].stage(), FrontendStageV2::Resource);
    }

    for (nested_filters, admitted) in [(125, true), (126, false)] {
        let nested = format!(
            "from records r | where exists (from records s{}) | project r.id as id",
            " | where true".repeat(nested_filters)
        );
        let result = compile_source_document_v3(DOC_SOURCE, &nested, &authority);
        if admitted {
            assert!(
                result.is_ok(),
                "nested relation at the exact limit must admit"
            );
        } else {
            assert_eq!(
                failure_code(&result.expect_err("nested relation past the limit must reject")),
                "surface.resource_exhausted"
            );
        }
    }

    let colliding = format!(
        "from records r{} | project r.id as id",
        " | where true".repeat(126)
    );
    let error = compile_source_document_v3(HOST_SOURCE, &colliding, &authority)
        .expect_err("a valid-depth source must reject a registry identity collision");
    assert_eq!(
        failure_code(&error),
        "hir.source_registry.duplicate_source_id"
    );
}

#[test]
fn formatting_relation_pipeline_respects_source_depth_admission() {
    let source = format!("from records r{}", " | where true".repeat(127));
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let formatted = format_source_document_v2(DOC_SOURCE, &source, &budget)
        .expect("the exact admitted relation depth must format");
    assert_eq!(formatted, source);

    let over = format!("from records r{}", " | where true".repeat(4_096));
    let error = format_source_document_v2(DOC_SOURCE, &over, &budget)
        .expect_err("overdeep relations must reject before formatter traversal");
    assert_eq!(error.diagnostics()[0].code(), "surface.resource_exhausted");
}

#[test]
fn formatting_long_flat_boolean_chain_rejects_at_structural_depth_budget() {
    let source = format!("from records r | where true{}", " and true".repeat(4_096));
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let failure = format_source_document_v2(DOC_SOURCE, &source, &budget)
        .expect_err("overdeep boolean HIR must reject before formatting");
    assert_eq!(
        failure.diagnostics()[0].code(),
        "surface.resource_exhausted"
    );
}

#[test]
fn compiling_long_flat_boolean_chain_rejects_at_structural_depth_budget() {
    let source = format!(
        "from records r | where true{} | project r.id as id",
        " and true".repeat(4_096)
    );
    let result = compile_source_document_v3(DOC_SOURCE, &source, &authority_with_records());
    assert_eq!(
        failure_code(&result.expect_err("overdeep boolean HIR must reject before compilation")),
        "surface.resource_exhausted"
    );
}

#[test]
fn source_bound_v3_digests_match_the_shared_golden_vectors() {
    let mut selected = 0;
    for row in include_str!("../fixtures/execution-v3-vectors.tsv")
        .lines()
        .skip(1)
    {
        let fields = row.split('\t').collect::<Vec<_>>();
        assert_eq!(fields.len(), 7, "malformed vector row: {row}");
        let [case, profile, raw_source_id, source, source_hex, contract_hex, qualification_hex] =
            fields.as_slice()
        else {
            panic!("malformed vector row: {row}");
        };
        let authority = match *profile {
            "records_host" => authority_with_records(),
            "records_host_string_id" => authority_with_records_type(ValueTypeV1::string()),
            "minimal" => minimal_authority(),
            "source_policy" => source_policy_authority(),
            "workspace_files_canonical" => {
                authority_with_workspace_files(FrontendRelationAuthorityV2::CanonicalWorkspaceFiles)
            }
            "workspace_files_host" => {
                authority_with_workspace_files(FrontendRelationAuthorityV2::Host)
            }
            other => panic!("unknown vector authority profile: {other}"),
        };
        let source_id = SourceIdV2::from_raw(raw_source_id.parse().expect("u32 source id"));
        let compiled = compile_source_document_v3(source_id, source, &authority)
            .unwrap_or_else(|error| panic!("{case}: {error:?}"));
        let execution = compiled.execution();
        let actual = [
            execution.source_digest().to_hex(),
            execution.contract_digest().to_hex(),
            execution.qualification_digest().to_hex(),
        ];
        for digest in [source_hex, contract_hex, qualification_hex] {
            assert_eq!(digest.len(), 64, "{case}: malformed hex digest");
            assert!(digest
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()));
        }
        assert_eq!(
            actual,
            [*source_hex, *contract_hex, *qualification_hex],
            "{case}"
        );
        selected += 1;
    }
    assert_eq!(
        selected, 24,
        "the shared V3 corpus must cover every standalone family"
    );
}

#[test]
fn workspace_files_authority_is_closed_and_digest_bound() {
    let canonical = compile_source_document_v3(
        DOC_SOURCE,
        WORKSPACE_FILES_SOURCE,
        &authority_with_workspace_files(FrontendRelationAuthorityV2::CanonicalWorkspaceFiles),
    )
    .expect("canonical workspace-files source compiles");
    let repeated = compile_source_document_v3(
        DOC_SOURCE,
        WORKSPACE_FILES_SOURCE,
        &authority_with_workspace_files(FrontendRelationAuthorityV2::CanonicalWorkspaceFiles),
    )
    .expect("same workspace-files source compiles deterministically");
    assert_eq!(canonical.execution(), repeated.execution());

    let source = canonical.execution().items()[0]
        .relation()
        .expect("relation execution contract")
        .root()
        .children()[0]
        .source()
        .expect("scan carries source identity");
    assert_eq!(
        source.authority(),
        ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles
    );
    assert_eq!(source.kind(), ExecutionSourceKindV3::Relation);
    assert_eq!(source.namespace(), Some("builtin"));
    assert_eq!(source.name(), "workspace_files");

    let host = compile_source_document_v3(
        DOC_SOURCE,
        WORKSPACE_FILES_SOURCE,
        &authority_with_workspace_files(FrontendRelationAuthorityV2::Host),
    )
    .expect("same shape remains a valid host relation");
    let host_source = host.execution().items()[0]
        .relation()
        .expect("relation execution contract")
        .root()
        .children()[0]
        .source()
        .expect("scan carries source identity");
    assert_eq!(host_source.authority(), ExecutionRelationAuthorityV3::Host);
    assert_ne!(
        canonical.execution().contract_digest(),
        host.execution().contract_digest()
    );
    assert_ne!(
        canonical.execution().qualification_digest(),
        host.execution().qualification_digest()
    );
}

#[test]
fn execution_expressions_expose_operator_and_builtin_semantics_without_registries() {
    // The `let` noise item must NOT leak into the relation's closed table: the
    // census below observes only the relation item's own expressions.
    let source = "let noise = 6 * 7; \
                  from records r | where r.id + 1 > 2 and r.id - 1 != 0 | project count(*) as n";
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &authority_with_records())
        .expect("mixed expression document compiles");
    let expressions = compiled
        .execution()
        .items()
        .last()
        .and_then(|item| item.relation())
        .expect("relation carries its closed expression table")
        .expressions();
    let mut operators = BTreeSet::new();
    let mut builtins = Vec::new();
    for expression in expressions {
        match expression.shape() {
            ExecutionExpressionShapeV3::Unary { operator, .. }
            | ExecutionExpressionShapeV3::Binary { operator, .. }
            | ExecutionExpressionShapeV3::Compare { operator, .. }
            | ExecutionExpressionShapeV3::And { operator, .. }
            | ExecutionExpressionShapeV3::Or { operator, .. }
            | ExecutionExpressionShapeV3::Not { operator, .. } => {
                operators.insert(operator.symbol().to_owned());
            }
            ExecutionExpressionShapeV3::Call { target, .. } => {
                let ExecutionCallTargetV3::Builtin { name } = target else {
                    panic!("expected a builtin call target, got {target:?}");
                };
                builtins.push(name.clone());
            }
            // Exhaustive by construction: a future shape variant fails compilation
            // here instead of silently evading the operator/call census.
            ExecutionExpressionShapeV3::Literal(_)
            | ExecutionExpressionShapeV3::Binding(_)
            | ExecutionExpressionShapeV3::Path { .. }
            | ExecutionExpressionShapeV3::Paren(_)
            | ExecutionExpressionShapeV3::Record(_)
            | ExecutionExpressionShapeV3::List(_)
            | ExecutionExpressionShapeV3::BoolValue(_)
            | ExecutionExpressionShapeV3::LogicalQuantifier { .. }
            | ExecutionExpressionShapeV3::RelationExists { .. } => {}
        }
    }
    assert!(operators.contains("+"));
    assert!(operators.contains("-"));
    assert!(operators.contains(">"));
    assert!(operators.contains("!="));
    // The `and` combines the two conjuncts, which the operator tree references
    // directly: the filter carries both conjunct keys, so the unreferenced
    // `And` parent is correctly absent from the closed table.
    assert!(!operators.contains("and"));
    let relation = compiled
        .execution()
        .items()
        .last()
        .and_then(|item| item.relation())
        .expect("relation");
    let mut stack = vec![relation.root()];
    let mut filter_predicates = None;
    while let Some(node) = stack.pop() {
        if node.family() == AlgebraFamilyV2::Filter {
            filter_predicates = Some(
                node.predicates()
                    .iter()
                    .map(|key| {
                        let expression = expressions
                            .iter()
                            .find(|entry| entry.id() == *key)
                            .expect("each filter key names its closed expression");
                        let ExecutionExpressionShapeV3::Compare { operator, .. } =
                            expression.shape()
                        else {
                            panic!("the authored filter conjunct is a comparison");
                        };
                        operator.symbol()
                    })
                    .collect::<Vec<_>>(),
            );
        }
        stack.extend(node.children().iter());
    }
    assert_eq!(filter_predicates, Some(vec![">", "!="]));
    assert!(
        !operators.contains("*"),
        "the noise item's `*` must not leak into the relation table: {operators:?}"
    );
    assert!(builtins.iter().any(|name| name == "count"));
    assert!(!builtins
        .iter()
        .any(|name| name.chars().all(|c| c.is_ascii_digit())));
}

/// Adding unrelated expressions to a sibling relation leaves this item's
/// contract byte-identical: each relation carries only its own transitive
/// expression closure, never the document-wide table.
#[test]
fn unrelated_sibling_expressions_do_not_move_an_item_contract() {
    let authority = authority_with_records();
    let head = "from records r | project r.id as id";
    let base = compile_source_document_v3(
        DOC_SOURCE,
        &format!("{head} ;\nfrom records s | project s.id as sid"),
        &authority,
    )
    .expect("two-relation document compiles");
    let grown = compile_source_document_v3(
        DOC_SOURCE,
        &format!(
            "{head} ;\nfrom records s | where s.id + 1 > 2 and s.id - 1 != 0 | project count(*) as n"
        ),
        &authority,
    )
    .expect("sibling with more expressions compiles");
    let head_base = base.execution().items()[0]
        .relation()
        .expect("head relation");
    let head_grown = grown.execution().items()[0]
        .relation()
        .expect("head relation");
    assert_eq!(head_base, head_grown);
    // Sanity: the sibling really did grow, so the equality above is not vacuous.
    assert_ne!(
        base.execution().items()[1]
            .relation()
            .expect("sibling")
            .expressions()
            .len(),
        grown.execution().items()[1]
            .relation()
            .expect("sibling")
            .expressions()
            .len()
    );
}

/// Every expression identity referenced anywhere in a projected relation —
/// operator roots and transitive shape children alike — resolves to a row of
/// that relation's own closed table. Binding schemas whose producer carries no
/// table (SAMPLE/Graph) must contribute no dangling order/partition references.
#[test]
fn every_projected_expression_reference_resolves_within_its_own_table() {
    fn schema_keys(schema: &ExecutionSchemaV3, out: &mut Vec<SourceQualifiedNodeIdV2>) {
        out.extend_from_slice(schema.partition_keys());
        out.extend(schema.order().iter().map(|key| key.expression()));
    }
    fn operator_keys(node: &ExecutionRelationalOperatorV3, out: &mut Vec<SourceQualifiedNodeIdV2>) {
        out.extend_from_slice(node.predicates());
        out.extend_from_slice(node.expressions());
        out.extend(node.correlations().iter().map(|edge| edge.reference()));
        for term in node.existential_terms() {
            out.push(term.expression());
            out.extend(term.correlations().iter().map(|edge| edge.reference()));
        }
        schema_keys(node.schema(), out);
        for child in node.children() {
            operator_keys(child, out);
        }
    }
    fn shape_keys(shape: &ExecutionExpressionShapeV3) -> Vec<SourceQualifiedNodeIdV2> {
        match shape {
            ExecutionExpressionShapeV3::Literal(_) | ExecutionExpressionShapeV3::Binding(_) => {
                Vec::new()
            }
            ExecutionExpressionShapeV3::Path { base, .. } => vec![*base],
            ExecutionExpressionShapeV3::Unary { operand, .. }
            | ExecutionExpressionShapeV3::Not { operand, .. } => vec![*operand],
            ExecutionExpressionShapeV3::Binary { lhs, rhs, .. }
            | ExecutionExpressionShapeV3::Compare { lhs, rhs, .. }
            | ExecutionExpressionShapeV3::And { lhs, rhs, .. }
            | ExecutionExpressionShapeV3::Or { lhs, rhs, .. } => vec![*lhs, *rhs],
            ExecutionExpressionShapeV3::Call { arguments, .. } => {
                arguments.iter().filter_map(|argument| *argument).collect()
            }
            ExecutionExpressionShapeV3::Paren(inner)
            | ExecutionExpressionShapeV3::BoolValue(inner) => vec![*inner],
            ExecutionExpressionShapeV3::Record(fields) => {
                fields.iter().map(|(_, id)| *id).collect()
            }
            ExecutionExpressionShapeV3::List(items) => items.to_vec(),
            ExecutionExpressionShapeV3::LogicalQuantifier { body, .. } => vec![*body],
            ExecutionExpressionShapeV3::RelationExists { correlations, .. } => {
                correlations.iter().map(|edge| edge.reference()).collect()
            }
        }
    }
    fn assert_relation_closed(
        label: &str,
        relation: &ExecutionRelationV3,
        expect_references: bool,
    ) {
        let table: BTreeSet<SourceQualifiedNodeIdV2> = relation
            .expressions()
            .iter()
            .map(|entry| entry.id())
            .collect();
        let mut referenced = Vec::new();
        operator_keys(relation.root(), &mut referenced);
        for entry in relation.expressions() {
            referenced.extend(shape_keys(entry.shape()));
        }
        if expect_references {
            assert!(
                !referenced.is_empty(),
                "{label}: the positive control must reference expressions"
            );
        }
        for key in referenced {
            assert!(
                table.contains(&key),
                "{label}: {key:?} is referenced but absent from the closed table"
            );
        }
    }
    let where_source =
        "from records r | where r.id + 1 > 2 and r.id - 1 != 0 | project count(*) as n";
    let where_compiled =
        compile_source_document_v3(DOC_SOURCE, where_source, &authority_with_records())
            .expect("where/count relation compiles");
    let mut closed_relations = 0_usize;
    for item in where_compiled.execution().items() {
        if let Some(relation) = item.relation() {
            assert_relation_closed("where/count relation", relation, true);
            closed_relations += 1;
        }
    }
    assert_eq!(
        closed_relations, 1,
        "the where/count document must expose exactly one relation to close"
    );
    let sample =
        compile_source_document_v3(DOC_SOURCE, &composition_source(2), &minimal_authority())
            .expect("SAMPLE composition compiles");
    let query_composition =
        "let rows = from records r | where r.id + 1 > 2 from rows x | project x.id as id with rows";
    let query =
        compile_source_document_v3(DOC_SOURCE, query_composition, &authority_with_records())
            .expect("Query composition compiles");
    for (label, compiled, expected_family, expected_name) in [
        ("SAMPLE", sample, ExecutionFamilyV3::Presentation, "sampled"),
        ("Query", query, ExecutionFamilyV3::Relation, "rows"),
    ] {
        assert_eq!(compiled.execution().items().len(), 1, "{label}");
        let item = compiled.execution().items().first().expect("one item");
        let composition = item.composition().expect("composition payload");
        assert_eq!(composition.bindings().len(), 1, "{label}");
        for binding in composition.bindings() {
            assert_eq!(binding.binding().name(), expected_name, "{label}");
            assert_eq!(binding.binding().ordinal(), 0, "{label}");
            assert_eq!(binding.producer_family(), expected_family, "{label}");
            let schema = binding
                .schema()
                .expect("each authored producer exposes a schema");
            let mut keys = Vec::new();
            schema_keys(schema, &mut keys);
            match expected_family {
                ExecutionFamilyV3::Relation => {
                    assert!(matches!(
                        binding.producer(),
                        ExecutionCompositionProducerV3::Relation
                    ));
                    let producer = binding
                        .relation()
                        .expect("the Query producer retains its relation table");
                    assert_eq!(producer.item_id(), binding.producer_item_id());
                    assert_eq!(schema, producer.root().schema());
                    let table: BTreeSet<SourceQualifiedNodeIdV2> = producer
                        .expressions()
                        .iter()
                        .map(|entry| entry.id())
                        .collect();
                    for key in keys {
                        assert!(
                            table.contains(&key),
                            "{label}: binding schema {key:?} is absent from the producer table"
                        );
                    }
                    assert_relation_closed(label, producer, true);
                }
                ExecutionFamilyV3::Presentation => {
                    assert!(matches!(
                        binding.producer(),
                        ExecutionCompositionProducerV3::Sample { .. }
                    ));
                    assert!(binding.relation().is_none(), "SAMPLE has no relation table");
                    assert!(
                        keys.is_empty(),
                        "{label}: table-less producer schema must not reference expressions: {keys:?}"
                    );
                }
                other @ (ExecutionFamilyV3::Graph
                | ExecutionFamilyV3::Command
                | ExecutionFamilyV3::Check
                | ExecutionFamilyV3::Composition
                | ExecutionFamilyV3::MetaRequireCertified
                | ExecutionFamilyV3::RuleProgram) => {
                    panic!("unexpected authored producer family {other:?}")
                }
            }
        }
        let body = composition
            .body_relation()
            .expect("the SAMPLE/Query composition body is relational");
        assert_relation_closed(label, body, true);
    }
}

/// The resolved entry shares checked semantic items with the source entry,
/// but cannot issue an execution document because it has no source bytes.
#[test]
fn resolved_entry_preserves_semantics_without_execution_identity() {
    let authority = authority_with_records();
    let via_source = compile_source_document_v3(DOC_SOURCE, RECORD_SOURCE, &authority)
        .expect("source entry compiles");
    let document = resolve_source(&authority, RECORD_SOURCE);
    let via_resolved = compile_resolved_document_v2(&document).expect("resolved entry compiles");
    assert_eq!(via_source.source_id(), via_resolved.source_id());
    assert_eq!(via_source.items(), via_resolved.items());
    assert_eq!(via_source.rule_program(), via_resolved.rule_program());
    assert_eq!(via_source.execution().items().len(), 1);
}

/// A composition body whose lowered root belongs to ANOTHER relation is
/// rejected with the exact planning reason: the sealed body identity is checked
/// against the canonical lowering of the SEALED body, so a transplanted subtree
/// that reuses the body identity cannot be planned.
#[test]
fn a_composition_body_from_another_relation_is_rejected() {
    let authority = minimal_authority();
    let document = resolve_source(&authority, &composition_source(2));
    let composition_id = *document.order().first().expect("one composition item");
    let Some(SemanticItemV2::Composition(composition)) = document.item(composition_id) else {
        panic!("the authored item is a composition");
    };
    let sealed_body_id = composition.body().item_id();
    assert_ne!(sealed_body_id, composition.item_id());

    // Another relation's canonical lowering: same source, different owner.
    let record_document = resolve_source(&authority_with_records(), RECORD_SOURCE);
    let relation_id = *record_document
        .order()
        .first()
        .expect("one relation item in the record document");
    let lowered = lower_relation_item(&record_document, relation_id)
        .expect("the host relation lowering admits");
    assert_eq!(lowered.item_id(), relation_id);
    assert_ne!(lowered.root().item_id(), composition.item_id());

    let body =
        CompositionRelationalBodyPlanInputV2::try_new(sealed_body_id, lowered.root().clone())
            .expect("both the sealed body and the transplanted root are this source's");
    let PlanningDiagnosticV1::InvalidComposition { reason, span } =
        CheckedCompositionPlanInputV2::new(
            composition,
            CompositionBodyPlanInputV2::Relational(body),
        )
        .expect_err("a body root from another relation must reject")
    else {
        panic!("a transplanted body root is the InvalidComposition rejection");
    };
    assert_eq!(reason, CompositionInvalidReasonV2::BodyIdentityMismatch);
    assert_eq!(span, *composition.location());
}

#[test]
fn format_entry_emits_canonical_text_and_enforces_the_source_budget() {
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let formatted = format_source_document_v2(DOC_SOURCE, MINIMAL_SOURCE, &budget)
        .expect("the minimal source formats");
    assert_eq!(
        formatted, "let answer = 1;\nrule local_rule : x if x = 1;\nshow capabilities",
        "canonical formatting must emit the exact canonical text"
    );

    let tiny = FrontendBudgetV2::try_new(
        8, 4_194_304, 131_072, 4_096, 64, 128, 262_144, 4_096, 4_096, 1_000_000, 50_000, 200_000,
        1_000_000, 16_777_216, 65_536, 50_000, 200_000, 1_000_000, 1_024, 10_000, 1_024, 8_388_608,
    )
    .expect("tiny source budget admits");
    let failure = format_source_document_v2(DOC_SOURCE, MINIMAL_SOURCE, &tiny)
        .expect_err("a source past max_source_bytes rejects");
    assert_eq!(
        failure
            .diagnostics()
            .first()
            .expect("one diagnostic")
            .code(),
        "frontend.resource_limit_exceeded"
    );

    let syntax = format_source_document_v2(DOC_SOURCE, "from records r | bogus", &budget)
        .expect_err("unknown stage must retain syntax/integrity classification");
    assert_eq!(syntax.diagnostics()[0].stage(), FrontendStageV2::Integrity);
}

#[test]
fn existence_cardinality_uses_the_left_subtree_after_both_children_fold() {
    use quanta_gqlang_v2_sema::{FrontendRelationCardinalityV2, FrontendRelationFieldInputV2};
    let location = FrontendAuthorityLocationV2 {
        source_id: HOST_SOURCE,
        start: 0,
        end: 1,
    };
    let mut builder = quanta_gqlang_v2_pipeline::ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST_SOURCE,
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
    .expect("source authority builder");
    // Distinct field names make both joins valid under the closed row schema.
    for (name, field) in [("left_rows", "id"), ("right_rows", "other_id")] {
        builder.add_relation(FrontendRelationInputV2 {
            authority: FrontendRelationAuthorityV2::Host,
            provenance_namespace: "host".to_owned(),
            qualified_name: format!("app::{name}"),
            lookup_name: name.to_owned(),
            fields: vec![FrontendRelationFieldInputV2 {
                name: field.to_owned(),
                scalar_type: FrontendScalarTypeV2::Int,
                sequence: false,
                optional: false,
                location,
            }],
            cardinality: FrontendRelationCardinalityV2::Bag,
            provenance: "test".to_owned(),
            location,
        });
    }
    let authority = builder.finish().expect("join authority");
    for polarity in ["exists", "not exists"] {
        for left_join in [false, true] {
            for right_join in [false, true] {
                // Self-joins must avoid duplicate output field names so this
                // rail reaches planning instead of failing in record typing.
                let left = if left_join {
                    "from left_rows a | project a.id as left_id | join right_rows b on left_id = b.other_id"
                } else {
                    "from left_rows a"
                };
                let right = if right_join {
                    "from left_rows c | project c.id as right_id | join right_rows d on right_id = d.other_id"
                } else {
                    "from left_rows c"
                };
                let source = format!("{left} | where {polarity} ({right})");
                let compiled = compile_source_document_v3(DOC_SOURCE, &source, &authority)
                    .unwrap_or_else(|error| panic!("{source}: {error:?}"));
                assert_eq!(compiled.items().len(), 1);
                let plan = compiled.items()[0].plan().expect("relation plan");
                let expected = if left_join {
                    CardinalityClassV2::Multiplicative
                } else {
                    CardinalityClassV2::LinearInInput
                };
                assert_eq!(
                    plan.semantic_cost().estimated_cardinality_class,
                    expected,
                    "{source}"
                );
            }
        }
    }
}

#[test]
fn false_value_and_predicate_literals_reach_the_execution_contract_unchanged() {
    // Both value projection and predicate use must retain the source literal.
    // Raw Bool HIR literals have a separate Types owner rail.
    for source in [
        "from records r | project false as x",
        "from records r | where false",
    ] {
        assert_eq!(
            projected_literal(source),
            ExecutionLiteralV3::Bool(false),
            "{source}"
        );
    }
}

#[test]
fn public_optional_relation_field_reaches_the_execution_schema_as_optional() {
    use quanta_gqlang_v2_sema::{
        ClosedFrontendAuthorityBuilderV2, FrontendRelationCardinalityV2,
        FrontendRelationFieldInputV2,
    };
    let location = FrontendAuthorityLocationV2 {
        source_id: HOST_SOURCE,
        start: 0,
        end: 1,
    };
    let mut builder = ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST_SOURCE,
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
    .expect("public builder starts");
    builder.add_relation(FrontendRelationInputV2 {
        authority: FrontendRelationAuthorityV2::Host,
        provenance_namespace: "host".to_owned(),
        qualified_name: "app::optional_records".to_owned(),
        lookup_name: "optional_records".to_owned(),
        fields: vec![FrontendRelationFieldInputV2 {
            name: "id".to_owned(),
            scalar_type: FrontendScalarTypeV2::Int,
            sequence: false,
            optional: true,
            location,
        }],
        cardinality: FrontendRelationCardinalityV2::Bag,
        provenance: "test".to_owned(),
        location,
    });
    let authority = builder.finish().expect("optional relation authority seals");
    let compiled = compile_source_document_v3(
        DOC_SOURCE,
        "from optional_records r | project r.id as id",
        &authority,
    )
    .expect("optional field projection compiles");
    let relation = compiled.execution().items()[0]
        .relation()
        .expect("one authored relation");
    let fields = relation.root().schema().fields();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].name(), "id");
    assert_eq!(
        fields[0].value_type(),
        &ExecutionValueTypeV3::Optional(Box::new(ExecutionValueTypeV3::Scalar(
            ExecutionScalarTypeV3::Int
        )))
    );
}

#[test]
fn source_sort_direction_and_null_placement_reach_execution_unchanged() {
    use quanta_gqlang_v2_pipeline::{ExecutionNullOrderV3, ExecutionSortDirectionV3};
    let nullable = ValueTypeV1::try_optional(ValueTypeV1::int()).expect("nullable sort key");
    let sorted = compile_source_document_v3(
        DOC_SOURCE,
        "from records r | order by r.id asc, r.id desc",
        &authority_with_records_type(nullable),
    )
    .expect("mixed-direction nullable sort source");
    let relation = sorted.execution().items()[0]
        .relation()
        .expect("sort relation");
    let schema = relation.root().schema();
    assert_eq!(schema.cardinality(), ExecutionCardinalityV3::Sequence);
    assert_eq!(
        schema
            .order()
            .iter()
            .map(|key| (key.direction(), key.null_order()))
            .collect::<Vec<_>>(),
        [
            (ExecutionSortDirectionV3::Asc, ExecutionNullOrderV3::Last),
            (ExecutionSortDirectionV3::Desc, ExecutionNullOrderV3::Last)
        ]
    );
    for key in schema.order() {
        assert_eq!(
            key.value_type(),
            &ExecutionValueTypeV3::Optional(Box::new(ExecutionValueTypeV3::Scalar(
                ExecutionScalarTypeV3::Int
            )))
        );
        let expression = relation
            .expressions()
            .iter()
            .find(|entry| entry.id() == key.expression())
            .expect("sort key retains its expression identity");
        let ExecutionExpressionShapeV3::Path { segments, .. } = expression.shape() else {
            panic!("source sort key is a path");
        };
        // Row aliases resolve to binding identity; the canonical field path retains id.
        assert_eq!(segments.as_ref(), &["id".to_owned()]);
    }
}

#[test]
fn composition_flow_and_taint_project_the_exact_graph_kinds_roles_and_targets() {
    let source = "let paths = find flow api.entry -> db.write | via=mid.two,mid.one | through=must.pass | depth=2 \
                  find taint api.entry -> db.write | sanitizers=authz.clean | scope=repo | window=recent | frontier=edge | depth=3 with paths";
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &source_policy_authority())
        .expect("bounded flow producer and taint body compile");
    let composition = compiled.execution().items()[0]
        .composition()
        .expect("composition DTO");
    let ExecutionCompositionProducerV3::Graph { query: producer } =
        composition.bindings()[0].producer()
    else {
        panic!("graph producer");
    };
    assert_eq!(producer.kind(), ExecutionGraphKindV3::Flow);
    let expected = [
        (ExecutionGraphReferenceRoleV3::Source, "api.entry"),
        (ExecutionGraphReferenceRoleV3::Sink, "db.write"),
        (ExecutionGraphReferenceRoleV3::Via, "mid.two"),
        (ExecutionGraphReferenceRoleV3::Via, "mid.one"),
        (ExecutionGraphReferenceRoleV3::Through, "must.pass"),
    ];
    assert_eq!(producer.references().len(), expected.len());
    for (reference, (role, spelling)) in producer.references().iter().zip(expected) {
        assert_eq!(reference.role(), role);
        assert_eq!(
            reference.target(),
            &ExecutionGraphTargetV3::Host {
                kind: ExecutionGraphReferenceKindV3::Entity,
                spelling: spelling.to_owned(),
                entity_subtype: None
            }
        );
    }
    let ExecutionCompositionBodyV3::Graph(body) = composition.body() else {
        panic!("graph body");
    };
    assert_eq!(body.kind(), ExecutionGraphKindV3::Taint);
    assert_eq!(body.references().len(), 6);
    assert_eq!(
        body.references()[0].role(),
        ExecutionGraphReferenceRoleV3::Source
    );
    assert_eq!(
        body.references()[0].target(),
        &ExecutionGraphTargetV3::Host {
            kind: ExecutionGraphReferenceKindV3::Entity,
            spelling: "api.entry".to_owned(),
            entity_subtype: None
        }
    );
    let expected = [
        (
            ExecutionGraphReferenceRoleV3::Sink,
            ExecutionGraphReferenceKindV3::Entity,
            "db.write",
        ),
        (
            ExecutionGraphReferenceRoleV3::Scope,
            ExecutionGraphReferenceKindV3::Scope,
            "repo",
        ),
        (
            ExecutionGraphReferenceRoleV3::Window,
            ExecutionGraphReferenceKindV3::Window,
            "recent",
        ),
        (
            ExecutionGraphReferenceRoleV3::Frontier,
            ExecutionGraphReferenceKindV3::Frontier,
            "edge",
        ),
        (
            ExecutionGraphReferenceRoleV3::Sanitizer,
            ExecutionGraphReferenceKindV3::Sanitizer,
            "authz.clean",
        ),
    ];
    for (reference, (role, kind, spelling)) in body.references().iter().skip(1).zip(expected) {
        assert_eq!(reference.role(), role);
        assert_eq!(
            reference.target(),
            &ExecutionGraphTargetV3::Host {
                kind,
                spelling: spelling.to_owned(),
                entity_subtype: None
            }
        );
    }
}

#[test]
fn source_registered_builtin_and_operator_retain_exact_execution_identity() {
    for source in [
        "from records r | project length(\"ab\") + 1 as n",
        "let rows = from records r | project length(\"ab\") + 1 as n from rows x | project x.n as n with rows",
        "let rows = from records r from rows x | project length(\"ab\") + 1 as n with rows",
    ] {
        let compiled = compile_source_document_v3(DOC_SOURCE, source, &authority_with_records())
            .expect("registered expression compiles");
        let item = &compiled.execution().items()[0];
        let mut relations = item.relation().into_iter().collect::<Vec<_>>();
        if let Some(composition) = item.composition() {
            relations.extend(composition.bindings().iter().filter_map(|binding| binding.relation()));
            relations.extend(composition.body_relation());
        }
        let symbols = relations.iter().flat_map(|relation| relation.expressions())
            .filter_map(|expression| match expression.shape() {
                ExecutionExpressionShapeV3::Call { target: ExecutionCallTargetV3::Builtin { name }, .. } =>
                    Some((name.as_str(), None)),
                ExecutionExpressionShapeV3::Binary { operator, .. } =>
                    Some((operator.symbol(), Some(operator.overload_id()))),
                ExecutionExpressionShapeV3::Literal(_)
                | ExecutionExpressionShapeV3::Binding(_)
                | ExecutionExpressionShapeV3::Path { .. }
                | ExecutionExpressionShapeV3::Unary { .. }
                | ExecutionExpressionShapeV3::Call {
                    target: ExecutionCallTargetV3::Predicate { .. } | ExecutionCallTargetV3::HostPredicate { .. }, ..
                }
                | ExecutionExpressionShapeV3::Paren(_)
                | ExecutionExpressionShapeV3::Record(_)
                | ExecutionExpressionShapeV3::List(_)
                | ExecutionExpressionShapeV3::BoolValue(_)
                | ExecutionExpressionShapeV3::Compare { .. }
                | ExecutionExpressionShapeV3::And { .. }
                | ExecutionExpressionShapeV3::Or { .. }
                | ExecutionExpressionShapeV3::Not { .. }
                | ExecutionExpressionShapeV3::LogicalQuantifier { .. }
                | ExecutionExpressionShapeV3::RelationExists { .. } => None,
            }).collect::<Vec<_>>();
        assert_eq!(symbols, [("length", None), ("+", Some(101))], "{source}");
    }
}
