#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "document-assembly rails assert exact sealed payloads with unwrap/expect/indexing/panic; allowed for test targets only"
)]

//! Compiled-document exact-key assembly (ticket 07, lane W4-B).
//!
//! Every rail drives the PUBLIC pipeline entry points over sealed semantic
//! documents (Sema `test_support` fixtures) or over source text plus a real
//! closed authority, then asserts the exact assembled payload:
//!
//! - one plan per applicable authored identity, one checked non-plan reason per
//!   non-executable declaration, one rule-program plan for the exact member set;
//! - missing / extra / duplicate / foreign / wrong-kind membership is rejected
//!   with the closed `pipeline.document.*` code vocabulary;
//! - the traversal budget is the SEALED document receipt, never a caller
//!   argument (exact-limit admits, limit+1 rejects with the algebra code);
//! - group/partition/order identity, CHECK nested-graph identity, rule
//!   membership, and FIND witness retention all change the assembled payload,
//!   so a hardcoded or dropped contract cannot stay green.

use std::collections::BTreeSet;

use quanta_gqlang_v2_algebra::{lower_relation_item, AlgebraFamilyV2};
use quanta_gqlang_v2_logic::RelationShapeV1;
use quanta_gqlang_v2_modules::{
    CurrentModuleSelectionInputV2, ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2,
    VersionV2,
};
use quanta_gqlang_v2_pipeline::{
    compile_resolved_document_v2, compile_source_document_v3, format_source_document_v2,
    AnalysisItemV2, CompiledItemV2, DocumentAssemblyCodeV2, ExecutionAuthorityV3,
    ExecutionCallTargetV3, ExecutionCardinalityV3, ExecutionCompositionBodyV3,
    ExecutionCompositionProducerV3, ExecutionExpressionShapeV3, ExecutionFamilyV3,
    ExecutionGraphCapabilityV3, ExecutionGraphDedupIdentityV3, ExecutionGraphDirectionV3,
    ExecutionGraphKindV3, ExecutionGraphReferenceKindV3, ExecutionGraphReferenceRoleV3,
    ExecutionGraphRelationShapeV3, ExecutionGraphTargetV3, ExecutionGraphWitnessTieBreakV3,
    ExecutionLiteralV3, ExecutionRelationAuthorityV3, ExecutionRelationV3,
    ExecutionRelationalOperatorV3, ExecutionSampleModeV3, ExecutionScalarTypeV3, ExecutionSchemaV3,
    ExecutionSourceKindV3, ExecutionValueTypeV3, FrontendAuthorityLocationV2,
    FrontendAuthoritySourceV2, FrontendCatalogAssetInputV2, FrontendCatalogPayloadV2,
    FrontendGraphReferenceInputV2, FrontendLanguageVersionV2, FrontendRelationAuthorityV2,
    FrontendRelationInputV2, FrontendScalarTypeV2, FrontendStageV2, FrontendTransactionErrorV2,
    PlannedItemV2,
};
use quanta_gqlang_v2_planning::{
    AuthorityLaneV2, CapabilityV2, CardinalityClassV2, CheckedCompositionPlanInputV2,
    CompositionBodyFamilyV2, CompositionBodyPlanInputV2, CompositionInvalidReasonV2,
    CompositionRelationalBodyPlanInputV2, EffectClassV2, EvaluationDeterminismV2,
    EvidenceRequirementV2, NoPlanReasonV2, OrderingContractV2, PlanPolicyV2, PlanningDiagnosticV1,
    RecursionRiskClassV2, RenderRequirementV2, ResultKindV2, SemanticCostV2,
    WitnessExplosionRiskClassV2, WitnessSelectionPolicyV2,
};
use quanta_gqlang_v2_sema::test_support::{
    deep_relation_chain_document_v2, forged_rule_membership_document_v2,
    forged_small_receipt_bool_tree_document_v2, forged_small_receipt_relation_chain_document_v2,
    reference_graph_document_v2, reference_nested_check_document_v2,
    reference_relation_document_v2, reference_rule_document_v2, ForgedRuleMembershipMutationV2,
    ReferenceBudgetV2, ReferenceGraphDocumentV2, ReferenceGraphReferenceV2,
    ReferenceGraphRequestV2, ReferenceRelationFixtureV2, ReferenceRuleFixtureV2,
    SMALL_BUDGET_MAX_SEMANTIC_STEPS,
};
use quanta_gqlang_v2_sema::{
    CardinalityV1, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2, CompareMetricV1,
    ConfirmModeV1, DetectionModeV1, EmitModeV1, EvidenceModeV1, ExportFormatV1, FrontendBudgetV2,
    GraphDirectionV2, GraphQueryKindV1, GraphReferenceCatalogV2, GraphReferenceInputV2,
    GraphReferenceKindV2, GraphReferenceRoleV2, LocatedSpanV2, ProvenanceNamespaceV2,
    RecordFieldTypeV1, RecordTypeV1, RelationCatalogV2, RelationSourceInputV2,
    ResolvedExplainTargetV2, ResolvedPresentationKindV2, SampleModeV1, SemanticItemIdV2,
    SemanticItemV2, ShowCategoryV1, SourceIdV2, SourceQualifiedNodeIdV2, SourceRegistrationV2,
    SourceRegistryViewV2, SpanV1, TruthModeV1, ValueTypeV1,
};

/// The host registry source identity used by the minimal authority.
const HOST_SOURCE: SourceIdV2 = SourceIdV2::from_raw(1);

/// The document source identity used by the source-text rails.
const DOC_SOURCE: SourceIdV2 = SourceIdV2::from_raw(9);

/// A source document that needs no relation or graph authority: one value
/// declaration, one rule, and one closed-category presentation.
const MINIMAL_SOURCE: &str = "let answer = 1; rule local_rule : x if x = 1; show capabilities;";

/// One sequential composition whose producer is a bounded SAMPLE and whose
/// body consumes the working set, so no host relation authority is needed.
fn composition_source(limit: u32) -> String {
    format!(
        "let sampled = sample query=\"auth\" | mode=lexical | limit={limit} \
         from sampled s | project s.path as path with sampled"
    )
}

/// One typed relation query over the host relation `records`.
const RECORD_SOURCE: &str = "from records r | project r.id as id";
const WORKSPACE_FILES_SOURCE: &str = "from files f | project f.path as path";

/// The stable code of one transaction failure.
fn failure_code(error: &FrontendTransactionErrorV2) -> String {
    match error {
        FrontendTransactionErrorV2::Document(failure) => failure
            .diagnostics()
            .first()
            .expect("a document failure carries at least one diagnostic")
            .code()
            .to_owned(),
        FrontendTransactionErrorV2::Authority(authority) => authority.code().to_owned(),
    }
}

/// Build one closed authority admitting a single host relation `records` with
/// one `id` field, solely from sema/modules re-exports.
fn authority_with_records() -> ClosedFrontendAuthorityV2 {
    authority_with_records_type(ValueTypeV1::int())
}

fn authority_with_records_type(id_type: ValueTypeV1) -> ClosedFrontendAuthorityV2 {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: HOST_SOURCE,
        source_len: 64,
    }])
    .expect("host registry admits one source");
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: HOST_SOURCE,
            span: SpanV1::new(0, 1),
        })
        .expect("checked host location");
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let profile = budget.authority_profile().expect("budget profile");
    let module = ModuleIdV2::try_new("app", location.clone()).expect("module identity");
    let mut builder =
        ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).expect("module builder");
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .expect("module node admits");
    let modules = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location: location.clone(),
            },
            VersionV2::new(1, 0, 0),
        )
        .expect("module authority seals");
    let relations = RelationCatalogV2::try_new(
        vec![RelationSourceInputV2 {
            authority: quanta_gqlang_v2_sema::RelationAuthorityV2::Host,
            provenance_namespace: ProvenanceNamespaceV2::try_new("host", location.clone())
                .expect("provenance namespace"),
            qualified_name: "app::records".to_owned(),
            lookup_name: "records".to_owned(),
            row: RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
                "id",
                id_type,
                location.clone(),
            )
            .expect("id field")])
            .expect("records row"),
            cardinality: CardinalityV1::Bag,
            provenance: "host".to_owned(),
            location: location.clone(),
        }],
        &host,
        profile.clone(),
    )
    .expect("relation catalog");
    let graph_references = GraphReferenceCatalogV2::try_new(
        vec![
            GraphReferenceInputV2 {
                spelling: "a".to_owned(),
                kind: GraphReferenceKindV2::Entity,
                entity_subtype: None,
                location: location.clone(),
            },
            GraphReferenceInputV2 {
                spelling: "b".to_owned(),
                kind: GraphReferenceKindV2::Entity,
                entity_subtype: None,
                location: location.clone(),
            },
            GraphReferenceInputV2 {
                spelling: "scope".to_owned(),
                kind: GraphReferenceKindV2::Scope,
                entity_subtype: None,
                location: location.clone(),
            },
        ],
        &host,
        profile,
    )
    .expect("graph catalog");
    ClosedFrontendAuthorityV2::try_new(modules, relations, graph_references, budget)
        .expect("closed authority seals")
}

fn authority_with_workspace_files(
    authority: FrontendRelationAuthorityV2,
) -> ClosedFrontendAuthorityV2 {
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
    .expect("workspace authority builder starts");
    let mut relation = FrontendRelationInputV2::canonical_workspace_files(location, location);
    relation.authority = authority;
    builder.add_relation(relation);
    builder.finish().expect("workspace authority seals")
}

/// Build one closed authority with a module graph and no relation/graph
/// authority, solely from sema/modules re-exports.
fn minimal_authority() -> ClosedFrontendAuthorityV2 {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: HOST_SOURCE,
        source_len: 64,
    }])
    .expect("host registry admits one source");
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: HOST_SOURCE,
            span: SpanV1::new(0, 1),
        })
        .expect("checked host location");
    let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
    let profile = budget.authority_profile().expect("budget profile");
    let module = ModuleIdV2::try_new("app", location.clone()).expect("module identity");
    let mut builder =
        ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).expect("module builder");
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .expect("module node admits");
    let modules = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location,
            },
            VersionV2::new(1, 0, 0),
        )
        .expect("module authority seals");
    let relations =
        RelationCatalogV2::try_new(Vec::new(), &host, profile.clone()).expect("relation catalog");
    let graph_references =
        GraphReferenceCatalogV2::try_new(Vec::new(), &host, profile).expect("graph catalog");
    ClosedFrontendAuthorityV2::try_new(modules, relations, graph_references, budget)
        .expect("closed authority seals")
}

fn source_policy_authority() -> ClosedFrontendAuthorityV2 {
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
    .expect("presentation authority builder starts");
    for (spelling, kind) in [
        ("api.entry", GraphReferenceKindV2::Entity),
        ("db.write", GraphReferenceKindV2::Entity),
        ("authz.clean", GraphReferenceKindV2::Sanitizer),
        ("repo", GraphReferenceKindV2::Scope),
        ("mid.one", GraphReferenceKindV2::Entity),
        ("mid.two", GraphReferenceKindV2::Entity),
        ("must.pass", GraphReferenceKindV2::Entity),
        ("recent", GraphReferenceKindV2::Window),
        ("edge", GraphReferenceKindV2::Frontier),
        ("svc.handler", GraphReferenceKindV2::Callable),
        ("pattern.hybrid", GraphReferenceKindV2::Pattern),
        ("src.entry", GraphReferenceKindV2::Entity),
        ("sink.write", GraphReferenceKindV2::Entity),
        ("run.a", GraphReferenceKindV2::Snapshot),
        ("run.b", GraphReferenceKindV2::Snapshot),
        ("result.latest", GraphReferenceKindV2::ExportSource),
    ] {
        builder.add_graph_reference(FrontendGraphReferenceInputV2 {
            spelling: spelling.to_owned(),
            kind,
            entity_subtype: None,
            location,
        });
    }
    builder.add_graph_reference(FrontendGraphReferenceInputV2 {
        spelling: "rule.sqli".to_owned(),
        kind: GraphReferenceKindV2::Entity,
        entity_subtype: Some("rule".to_owned()),
        location,
    });
    builder.add_graph_reference(FrontendGraphReferenceInputV2 {
        spelling: "owasp.sqli".to_owned(),
        kind: GraphReferenceKindV2::Entity,
        entity_subtype: Some("rule".to_owned()),
        location,
    });
    builder.add_catalog_asset(FrontendCatalogAssetInputV2 {
        module_name: "app".to_owned(),
        local_name: "rule.hybrid".to_owned(),
        payload: FrontendCatalogPayloadV2::Rule {
            parameters: Vec::new(),
            result: FrontendScalarTypeV2::Bool,
        },
        provenance: "host".to_owned(),
        location,
    });
    builder.finish().expect("source policy authority seals")
}

/// One bounded FIND request with the exact authored depth.
fn bounded_query(kind: GraphQueryKindV1, depth: u32) -> ReferenceGraphRequestV2 {
    ReferenceGraphRequestV2 {
        kind,
        references: vec![
            ReferenceGraphReferenceV2 {
                role: GraphReferenceRoleV2::Source,
                selector: "user_input",
                span: (5, 15),
            },
            ReferenceGraphReferenceV2 {
                role: GraphReferenceRoleV2::Sink,
                selector: "exec",
                span: (19, 23),
            },
        ],
        depth: Some(depth),
        direction: None,
        summary_facets: Vec::new(),
        item_span: (0, 40),
    }
}

/// Compile one sealed FIND fixture.
fn compile_graph(request: ReferenceGraphRequestV2) -> ReferenceGraphDocumentV2 {
    reference_graph_document_v2(request).expect("bounded FIND fixture resolves")
}

// ---------------------------------------------------------------------------
// Positive assembly
// ---------------------------------------------------------------------------

#[test]
fn relation_scan_compiles_to_one_exact_planned_item() {
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let compiled = compile_resolved_document_v2(fixture.document()).expect("scan compiles");
    assert_eq!(compiled.source_id(), fixture.document().source_id());
    assert_eq!(compiled.items().len(), 1);
    assert!(compiled.diagnostics().is_empty());
    assert!(compiled.rule_program().is_none());
    let item = compiled
        .item(fixture.item_id())
        .expect("the authored relation identity is compiled");
    assert_eq!(item.item_id(), fixture.item_id());
    let CompiledItemV2::Planned(planned) = item else {
        panic!("a relational item must compile to a planned relation item");
    };
    let PlannedItemV2::Relation { plan, .. } = planned.as_ref() else {
        panic!("a relational item retains its relation plan");
    };
    assert!(item.plan().is_some());
    assert!(item.no_plan_reason().is_none());
    assert_eq!(
        plan,
        item.plan().expect("plan accessor agrees with the variant")
    );
}

#[test]
fn compiled_item_order_is_the_authored_source_order() {
    let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::CrossJoin)
        .expect("join resolves");
    let compiled = compile_resolved_document_v2(fixture.document()).expect("join compiles");
    let order: Vec<SemanticItemIdV2> = fixture.document().order().to_vec();
    let assembled: Vec<SemanticItemIdV2> = compiled
        .items()
        .iter()
        .map(CompiledItemV2::item_id)
        .collect();
    assert_eq!(assembled, order);

    // A multi-item source discriminates ORDER, not just membership: with three
    // authored items the compiled sequence must match the document order
    // element-by-element.
    let multi = compile_source_document_v3(DOC_SOURCE, MINIMAL_SOURCE, &minimal_authority())
        .expect("the minimal three-item document compiles");
    assert_eq!(multi.items().len(), 3);
    let authored: Vec<SemanticItemIdV2> = resolve_minimal_document(&minimal_authority())
        .order()
        .to_vec();
    let assembled: Vec<SemanticItemIdV2> =
        multi.items().iter().map(CompiledItemV2::item_id).collect();
    assert_eq!(assembled, authored);
}

#[test]
fn group_partition_identity_and_sort_policy_change_the_assembled_plan() {
    let one_key = reference_relation_document_v2(ReferenceRelationFixtureV2::GroupedAggregateSum)
        .expect("one-key group resolves");
    let two_keys = reference_relation_document_v2(
        ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys,
    )
    .expect("two-key group resolves");
    let one = compile_resolved_document_v2(one_key.document()).expect("one-key group compiles");
    let two = compile_resolved_document_v2(two_keys.document()).expect("two-key group compiles");
    assert_ne!(
        one.items(),
        two.items(),
        "a swapped Group partition identity must change the assembled plan"
    );

    let ascending =
        reference_relation_document_v2(ReferenceRelationFixtureV2::SortAsc).expect("sort resolves");
    let nulls_last =
        reference_relation_document_v2(ReferenceRelationFixtureV2::SortNullsLastNullableField)
            .expect("nulls-last sort resolves");
    let asc = compile_resolved_document_v2(ascending.document()).expect("ascending sort compiles");
    let last = compile_resolved_document_v2(nulls_last.document()).expect("nulls-last compiles");
    assert_ne!(
        asc.items(),
        last.items(),
        "a dropped sort direction / null policy must change the assembled plan"
    );
}

#[test]
fn find_witness_retention_changes_the_assembled_plan() {
    let shallow = compile_graph(bounded_query(GraphQueryKindV1::Taint, 1));
    let deep = compile_graph(bounded_query(GraphQueryKindV1::Taint, 4));
    let shallow_plan =
        compile_resolved_document_v2(shallow.document()).expect("shallow FIND compiles");
    let deep_plan = compile_resolved_document_v2(deep.document()).expect("deep FIND compiles");
    assert_ne!(
        shallow_plan.items(),
        deep_plan.items(),
        "a lost FIND witness cap/path-length bound must change the assembled plan"
    );
    let CompiledItemV2::Planned(planned) = &shallow_plan.items()[0] else {
        panic!("a FIND item must compile to a planned graph item");
    };
    let PlannedItemV2::Graph { item_id, .. } = planned.as_ref() else {
        panic!("a FIND item retains its graph plan");
    };
    assert_eq!(*item_id, shallow.graph_item_id());
}

#[test]
fn nested_check_retains_its_own_graph_identity() {
    let fixture = reference_nested_check_document_v2(bounded_query(GraphQueryKindV1::Path, 2))
        .expect("nested CHECK fixture resolves");
    let compiled = compile_resolved_document_v2(fixture.document()).expect("CHECK compiles");
    assert_eq!(compiled.items().len(), 1);
    let CompiledItemV2::Planned(planned) = &compiled.items()[0] else {
        panic!("a CHECK item must compile to a planned CHECK item");
    };
    let PlannedItemV2::Check { item_id, plan } = planned.as_ref() else {
        panic!("a CHECK item retains its CHECK plan");
    };
    assert_eq!(*item_id, fixture.check_item_id());
    assert_ne!(*item_id, fixture.nested_graph_item_id());

    // The document's OWN sealed CHECK and the nested FIND it retains. Every
    // assertion below reads one of those facts (or an exact class derived from
    // them), never the plan against a payload destructured out of that same
    // plan.
    let Some(SemanticItemV2::Check(check)) = fixture.document().item(*item_id) else {
        panic!("the CHECK identity is the document's own item");
    };
    let nested = check
        .nested_graph()
        .expect("the sealed CHECK retains its nested confirm FIND");
    assert_eq!(nested.item_id(), fixture.nested_graph_item_id());

    // Derived lanes: `confirm=skip` keeps the source-location evidence and
    // `emit=findings` keeps the read-only effect and the alert render.
    assert_eq!(plan.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(
        plan.evaluation_determinism(),
        EvaluationDeterminismV2::Deterministic
    );
    assert_eq!(plan.evidence(), EvidenceRequirementV2::SourceLocation);
    assert_eq!(plan.effect(), EffectClassV2::ReadOnly);
    assert_eq!(plan.result().result_kind(), ResultKindV2::Alert);
    assert_eq!(
        plan.result().render_requirement(),
        RenderRequirementV2::SourceLocation
    );
    assert_eq!(
        plan.result().witness_selection(),
        Some(WitnessSelectionPolicyV2::AllWitness),
        "the standard witness cap is wider than one, so every witness is materialized"
    );
    assert!(plan.result().multiplicity_order().eliminates_duplicates());
    assert!(!plan.result().multiplicity_order().is_ordered());

    // The visited nested FIND is a cyclic witness contract: witness
    // materialization and recursive closure, and the bound-dependent cost.
    assert_eq!(
        plan.capabilities(),
        &BTreeSet::from([
            CapabilityV2::WitnessMaterialization,
            CapabilityV2::RecursiveClosure,
        ])
    );
    assert_eq!(
        plan.semantic_cost(),
        &SemanticCostV2 {
            estimated_cardinality_class: CardinalityClassV2::Closure,
            recursion_risk_class: RecursionRiskClassV2::BoundDependent,
            witness_explosion_risk_class: WitnessExplosionRiskClassV2::CapBounded,
            ordering_contract: OrderingContractV2::Unordered,
        }
    );

    // Attribution is the CHECK item itself; a CHECK plan carries no algebra
    // family, but it IS recursive through its nested confirm FIND.
    assert_eq!(plan.explain().root_location(), check.location());
    assert!(plan.explain().families().is_empty());
    assert!(plan.explain().recursive());

    // The retained CHECK policy is the sealed CHECK's own payload, field for
    // field, plus the nested witness provenance of the sealed FIND.
    let PlanPolicyV2::Check(policy) = plan.policy() else {
        panic!("a CHECK plan must retain the CHECK policy");
    };
    assert_eq!(policy.mode(), check.mode());
    assert_eq!(policy.confirm(), check.confirm());
    assert_eq!(policy.emit(), check.emit());
    assert_eq!(policy.rule(), check.rule());
    assert_eq!(policy.scope(), check.scope());
    assert_eq!(policy.pattern(), check.pattern());
    assert_eq!(
        policy.rule().location().location(),
        LocatedSpanV2 {
            source_id: fixture.check_item_id().source_id(),
            span: SpanV1::new(100, 105),
        },
        "the retained rule reference keeps its authored `rule1` span"
    );
    let retention = policy
        .nested_graph()
        .expect("the nested semantic-confirm FIND is retained");
    assert_eq!(retention.item_id(), fixture.nested_graph_item_id());
    assert_ne!(retention.item_id(), *item_id);
    assert_eq!(retention.location(), nested.location());
    assert_eq!(retention.kind(), nested.kind());
    assert_eq!(retention.depth(), 2);
    assert_eq!(retention.shape(), RelationShapeV1::Cyclic);
    assert_eq!(
        retention.max_witnesses(),
        nested.witness_bounds().max_witnesses()
    );
    assert_eq!(retention.max_path_length(), Some(2));
}

#[test]
fn rule_program_receives_one_plan_and_exact_members() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)
        .expect("rule fixture resolves");
    let compiled = compile_resolved_document_v2(fixture.document()).expect("rules compile");
    let program = compiled
        .rule_program()
        .expect("a rule document has one program");
    let members: Vec<SemanticItemIdV2> = program
        .members()
        .iter()
        .map(|member| member.member_item_id())
        .collect();
    assert_eq!(members, fixture.item_ids());
    assert_eq!(program.program_id(), fixture.item_ids()[0]);
    assert!(program.stratification().stratum_count() > 0);
    assert_eq!(
        compiled.items().len(),
        fixture.item_ids().len(),
        "every authored rule item compiles to one item result"
    );
    for member in program.members() {
        assert_eq!(member.program_id(), program.program_id());
    }
    for item in compiled.items() {
        let CompiledItemV2::Planned(planned) = item else {
            panic!("every source rule compiles to a rule-program member reference");
        };
        let PlannedItemV2::RuleMember(member) = planned.as_ref() else {
            panic!("every source rule retains its member reference");
        };
        assert!(item.plan().is_none());
        assert_eq!(member.program_id(), program.program_id());
    }
}

/// F-06: the document rule graph is stratified exactly ONCE, at the program's
/// sole ingress, and every consumer reads THAT proof.
///
/// Before this rail's subject change the pipeline stratified the program itself
/// (charging the document's rule-graph allowance) and planning stratified it
/// again from the same receipt with a fresh counter, producing two independent
/// proofs of one member set. The assertions below pin the seam: the compiled
/// program's proof, the plan's retained rule policy, and the ingress contract's
/// proof are one and the same admission of the FULL projected program under the
/// document's own receipt (a re-derived proof over a subset, or under a
/// different budget, would not compare equal).
#[test]
fn the_rule_program_proof_is_the_single_ingress_stratification() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)
        .expect("rule fixture resolves");
    let document = fixture.document();
    let compiled = compile_resolved_document_v2(document).expect("rules compile");
    let program = compiled.rule_program().expect("one program");

    // The ONE admitted proof: `from_document` charged this document's own
    // rule-graph receipt, and the mutual pair is one recursive SCC.
    let contract = quanta_gqlang_v2_logic::RuleProgramContractV2::from_document(document)
        .expect("the mutual pair is stratifiable");
    assert_eq!(contract.budget(), document.budget_receipt());
    assert_eq!(contract.budget().max_rule_graph_nodes(), 50_000);
    assert_eq!(contract.stratification().stratum_count(), 1);
    assert_eq!(contract.stratification().recursive().len(), 2);

    // The compiled DTO carries THAT proof, byte for byte.
    assert_eq!(program.stratification(), contract.stratification());

    // The plan's rule policy retains the same proof summary: the ordered member
    // identities, the stratum count, and the recursive member set of the ONE
    // ingress proof — not a second classifier's.
    let PlanPolicyV2::Rule(policy) = program.plan().policy() else {
        panic!("a rule program must retain the rule-program policy");
    };
    assert_eq!(policy.item_ids(), fixture.item_ids());
    assert_eq!(
        policy.stratum_count(),
        contract.stratification().stratum_count()
    );
    assert_eq!(
        policy.recursive_members(),
        contract
            .stratification()
            .recursive()
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .as_slice()
    );

    // Non-vacuity: recomputing the stratification over the program the contract
    // projects, under the DOCUMENT's own receipt, reproduces the retained proof
    // exactly. The retained proof is therefore the full-program proof under the
    // document's single allowance.
    let recomputed = quanta_gqlang_v2_logic::stratify(
        &quanta_gqlang_v2_logic::RecursiveProgramV1::from_rules(contract.to_program()),
        &document.budget_receipt(),
    )
    .expect("the admitted member set re-stratifies identically");
    assert_eq!(contract.stratification(), &recomputed);
    assert_eq!(program.stratification(), &recomputed);
}

// ---------------------------------------------------------------------------
// Exact-key membership kills
// ---------------------------------------------------------------------------

#[test]
fn forged_rule_membership_rejects_with_the_logic_code() {
    for mutation in [
        ForgedRuleMembershipMutationV2::DroppedItem,
        ForgedRuleMembershipMutationV2::OrphanItem,
        ForgedRuleMembershipMutationV2::DuplicateOrderId,
    ] {
        let forged =
            forged_rule_membership_document_v2(mutation).expect("forged rule document builds");
        let error = compile_resolved_document_v2(forged.document())
            .expect_err("a forged rule membership must reject");
        let code = failure_code(&error);
        assert!(
            [
                DocumentAssemblyCodeV2::MissingItemResult.as_str(),
                DocumentAssemblyCodeV2::ExtraItemResult.as_str(),
                DocumentAssemblyCodeV2::DuplicateItemResult.as_str(),
                DocumentAssemblyCodeV2::RuleMembershipMismatch.as_str(),
                "logic.rule_program.missing_member",
                "logic.rule_program.duplicate_member",
                "logic.rule_program.membership_mismatch",
            ]
            .contains(&code.as_str()),
            "expected a stable membership code, got {code}"
        );
    }
}

#[test]
fn duplicate_authored_identity_rejects() {
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, _items| {
            let first = order[0];
            order.push(first);
        });
    let error = compile_resolved_document_v2(&forged).expect_err("duplicate order rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::DuplicateItemResult.as_str()
    );
}

#[test]
fn missing_produced_item_rejects() {
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, items| {
            let first = order[0];
            items.remove(&first);
        });
    let error = compile_resolved_document_v2(&forged).expect_err("missing item rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::MissingItemResult.as_str()
    );
}

#[test]
fn extra_produced_item_rejects() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)
        .expect("rule fixture resolves");
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, _items| {
            order.remove(0);
        });
    let error = compile_resolved_document_v2(&forged).expect_err("extra item rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::ExtraItemResult.as_str()
    );
}

#[test]
fn foreign_authored_identity_rejects() {
    let fixture =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, _items| {
            let first = order[0];
            order[0] = SemanticItemIdV2::new(SourceIdV2::from_raw(77), first.hir_item_id());
        });
    let error = compile_resolved_document_v2(&forged).expect_err("foreign identity rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::ForeignItemResult.as_str()
    );
}

#[test]
fn same_name_wrong_id_substitution_rejects() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)
        .expect("rule fixture resolves");
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, items| {
            let first = order[0];
            let second = order[1];
            let first_item = items.remove(&first).expect("first item present");
            let second_item = items.remove(&second).expect("second item present");
            items.insert(first, second_item);
            items.insert(second, first_item);
        });
    let error = compile_resolved_document_v2(&forged).expect_err("wrong-ID substitution rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::WrongKindItemResult.as_str()
    );
}

#[test]
fn check_graph_identity_substitution_rejects() {
    let fixture = reference_nested_check_document_v2(bounded_query(GraphQueryKindV1::Path, 2))
        .expect("nested CHECK fixture resolves");
    let check = fixture.check_item_id();
    let nested = fixture.nested_graph_item_id();
    let forged = fixture
        .document()
        .clone()
        .forged_membership(|order, items| {
            let item = items.remove(&check).expect("check item present");
            items.insert(nested, item);
            order[0] = nested;
        });
    let error = compile_resolved_document_v2(&forged).expect_err("CHECK id substitution rejects");
    assert_eq!(
        failure_code(&error),
        DocumentAssemblyCodeV2::WrongKindItemResult.as_str()
    );
}

// ---------------------------------------------------------------------------
// Budget: the sealed document receipt is the only authority
// ---------------------------------------------------------------------------

#[test]
fn sealed_receipt_admits_the_exact_limit_and_rejects_limit_plus_one() {
    // The forged document carries the small receipt while its chain was
    // resolved under the standard budget, so the algebra relation walk's own
    // charging is the only limit: a chain of depth d lowers d + 1 stages.
    let exact_depth = SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;
    let exact = forged_small_receipt_relation_chain_document_v2(exact_depth)
        .expect("exact-limit chain builds");
    compile_resolved_document_v2(exact.document()).expect("the exact sealed limit admits");

    let over = forged_small_receipt_relation_chain_document_v2(exact_depth + 1)
        .expect("over-limit builds");
    let error = compile_resolved_document_v2(over.document())
        .expect_err("one visit past the sealed receipt rejects");
    assert_eq!(failure_code(&error), "algebra.traversal_budget_exceeded");

    // The same depth resolved under the standard budget is NOT a limit: the
    // pipeline reads the document's receipt, never a caller-selected budget.
    let standard = deep_relation_chain_document_v2(exact_depth + 1, ReferenceBudgetV2::Standard)
        .expect("the standard budget admits the deeper chain");
    compile_resolved_document_v2(standard.document())
        .expect("the standard receipt admits what the small receipt refuses");
}

#[test]
fn sealed_expression_node_receipt_governs_the_expression_walk() {
    // Depth 1 charges seven of the eight small-receipt expression nodes; depth
    // 2 charges eleven and must reject one over.
    let exact =
        forged_small_receipt_bool_tree_document_v2(1).expect("exact-limit bool tree builds");
    compile_resolved_document_v2(exact.document()).expect("the exact node limit admits");

    let over = forged_small_receipt_bool_tree_document_v2(2).expect("over-limit bool tree builds");
    let error = compile_resolved_document_v2(over.document())
        .expect_err("the sealed expression-node receipt rejects the oversized walk");
    assert_eq!(failure_code(&error), "algebra.traversal_budget_exceeded");
}

// ---------------------------------------------------------------------------
// Public source entries
// ---------------------------------------------------------------------------

#[test]
fn closed_source_entry_assembles_the_exact_item_dispositions() {
    let authority = minimal_authority();
    let compiled = compile_source_document_v3(DOC_SOURCE, MINIMAL_SOURCE, &authority)
        .expect("the minimal document compiles");
    assert_eq!(compiled.source_id(), DOC_SOURCE);
    assert_eq!(compiled.items().len(), 3);
    assert!(compiled.rule_program().is_some());

    let dispositions: Vec<&'static str> = compiled
        .items()
        .iter()
        .map(|item| match item {
            CompiledItemV2::NoPlan { .. } => "no_plan",
            CompiledItemV2::Planned(planned) => match planned.as_ref() {
                PlannedItemV2::RuleMember(_) => "rule_member",
                PlannedItemV2::Presentation { .. } => "presentation",
                PlannedItemV2::Relation { .. }
                | PlannedItemV2::Graph { .. }
                | PlannedItemV2::Check { .. }
                | PlannedItemV2::Composition { .. }
                | PlannedItemV2::MetaRequireCertified { .. } => "planned",
            },
        })
        .collect();
    assert_eq!(dispositions, ["no_plan", "rule_member", "presentation"]);
    let first = &compiled.items()[0];
    assert_eq!(
        first.no_plan_reason(),
        Some(&NoPlanReasonV2::ValueDeclaration)
    );
    assert!(first.plan().is_none());
}

#[test]
fn authored_presentation_options_reach_the_exact_planned_policy() {
    let source = concat!(
        "sample query=\"callgraph\" | mode=semantic | limit=5 | scope=repo; ",
        "compare baseline=run.a | candidate=run.b | metric=semantic; ",
        "export source=result.latest | kind=sarif; ",
        "show capabilities; ",
        "describe rule::owasp.sqli;",
    );
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &source_policy_authority())
        .expect("all five authored presentation requests compile");
    assert_eq!(compiled.items().len(), 5);
    assert_eq!(
        compiled
            .execution()
            .items()
            .iter()
            .map(|item| item.item_id())
            .collect::<Vec<_>>(),
        compiled
            .items()
            .iter()
            .map(CompiledItemV2::item_id)
            .collect::<Vec<_>>(),
        "V3 items keep the authored multi-item order",
    );
    assert!(compiled.execution().items().iter().all(|item| {
        item.family() == ExecutionFamilyV3::Presentation
            && item.relation().is_none()
            && item.composition().is_none()
    }));

    let policies: Vec<&ResolvedPresentationKindV2> = compiled
        .items()
        .iter()
        .map(|item| {
            let CompiledItemV2::Planned(planned) = item else {
                panic!("presentation source became a no-plan item: {item:?}");
            };
            let PlannedItemV2::Presentation { plan, .. } = planned.as_ref() else {
                panic!("presentation source became a different plan: {planned:?}");
            };
            let PlanPolicyV2::Presentation(kind) = plan.policy() else {
                panic!("presentation plan lost its policy: {:?}", plan.policy());
            };
            kind
        })
        .collect();

    let ResolvedPresentationKindV2::Sample {
        query_text,
        mode,
        limit,
        scope: Some(scope),
    } = policies[0]
    else {
        panic!(
            "authored SAMPLE fields were not retained: {:?}",
            policies[0]
        );
    };
    assert_eq!(query_text, "callgraph");
    assert_eq!(*mode, SampleModeV1::Semantic);
    assert_eq!(*limit, 5);
    assert_eq!(
        scope
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("repo")
    );
    let ResolvedPresentationKindV2::Compare {
        baseline,
        candidate,
        metric,
    } = policies[1]
    else {
        panic!(
            "authored COMPARE fields were not retained: {:?}",
            policies[1]
        );
    };
    assert_eq!(
        baseline
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("run.a")
    );
    assert_eq!(
        candidate
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("run.b")
    );
    assert_eq!(*metric, CompareMetricV1::Semantic);
    let ResolvedPresentationKindV2::Export {
        source,
        format,
        profile,
    } = policies[2]
    else {
        panic!(
            "authored EXPORT fields were not retained: {:?}",
            policies[2]
        );
    };
    assert_eq!(
        source
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("result.latest")
    );
    assert_eq!(*format, ExportFormatV1::Sarif);
    assert!(profile.is_none());
    assert!(matches!(
        policies[3],
        ResolvedPresentationKindV2::Show {
            category: ShowCategoryV1::Capabilities
        }
    ));
    let ResolvedPresentationKindV2::Describe { target } = policies[4] else {
        panic!("authored DESCRIBE target was not retained");
    };
    assert_eq!(
        target
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("owasp.sqli")
    );
}

#[test]
fn authored_nondefault_execution_plan_facts_survive_projection() {
    use quanta_gqlang_v2_pipeline::{
        ExecutionEffectV3, ExecutionEvidenceV3, ExecutionPolicyFamilyV3,
    };

    let source = concat!(
        "export source=result.latest | kind=sarif; ",
        "explain rule.sqli | evidence=trace | truth=confirm; ",
        "find taint api.entry -> db.write | sanitizers=authz.clean | direction=backward | depth=3; ",
        "check rule.hybrid | scope=repo | pattern=pattern.hybrid | confirm=require ",
        "| emit=sarif | mode=hybrid using find reachability src.entry -> sink.write | depth=8;",
    );
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &source_policy_authority())
        .expect("existing source policy fixtures compile together");
    let expected = [
        (
            ExecutionFamilyV3::Presentation,
            ExecutionEffectV3::ExportOnly,
            ExecutionEvidenceV3::None,
            ExecutionPolicyFamilyV3::Presentation,
        ),
        (
            ExecutionFamilyV3::Presentation,
            ExecutionEffectV3::ReadOnly,
            ExecutionEvidenceV3::ProofTree,
            ExecutionPolicyFamilyV3::Presentation,
        ),
        (
            ExecutionFamilyV3::Graph,
            ExecutionEffectV3::ProofEvidenceOnly,
            ExecutionEvidenceV3::WitnessPayload,
            ExecutionPolicyFamilyV3::Graph,
        ),
        (
            ExecutionFamilyV3::Check,
            ExecutionEffectV3::ExportOnly,
            ExecutionEvidenceV3::ProofTree,
            ExecutionPolicyFamilyV3::Check,
        ),
    ];
    assert_eq!(compiled.items().len(), expected.len());
    assert_eq!(compiled.execution().items().len(), expected.len());
    for (index, (family, effect, evidence, policy_family)) in expected.into_iter().enumerate() {
        let item = &compiled.execution().items()[index];
        assert_eq!(item.item_id(), compiled.items()[index].item_id());
        assert_eq!(item.family(), family, "authored item {index}");
        assert_eq!(item.plan().effect(), effect, "authored item {index}");
        assert_eq!(item.plan().evidence(), evidence, "authored item {index}");
        assert_eq!(
            item.plan().policy_family(),
            policy_family,
            "authored item {index}"
        );
    }
}

#[test]
fn authored_find_check_and_explain_options_reach_the_exact_planned_policy() {
    let source = concat!(
        "find taint api.entry -> db.write | sanitizers=authz.clean | direction=backward | depth=3; ",
        "check rule.hybrid | scope=repo | pattern=pattern.hybrid | confirm=require ",
        "| emit=sarif | mode=hybrid using find reachability src.entry -> sink.write | depth=8; ",
        "explain rule.sqli | evidence=trace | truth=confirm;",
    );
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &source_policy_authority())
        .expect("authored graph, check, and explain requests compile");
    assert_eq!(compiled.items().len(), 3);

    let CompiledItemV2::Planned(find) = &compiled.items()[0] else {
        panic!("FIND must be planned");
    };
    let PlannedItemV2::Graph { plan, .. } = find.as_ref() else {
        panic!("FIND must keep its graph plan");
    };
    let PlanPolicyV2::Graph(witness) = plan.policy() else {
        panic!("FIND must keep its graph witness policy");
    };
    assert_eq!(witness.kind(), GraphQueryKindV1::Taint);
    assert_eq!(witness.depth(), 3);
    assert_eq!(witness.direction(), Some(GraphDirectionV2::Backward));
    assert_eq!(witness.references().len(), 3);
    assert!(witness
        .references()
        .iter()
        .any(|reference| reference.role() == GraphReferenceRoleV2::Sanitizer));

    let CompiledItemV2::Planned(check) = &compiled.items()[1] else {
        panic!("CHECK must be planned");
    };
    let PlannedItemV2::Check { plan, .. } = check.as_ref() else {
        panic!("CHECK must keep its check plan");
    };
    let PlanPolicyV2::Check(policy) = plan.policy() else {
        panic!("CHECK must keep its option policy");
    };
    assert_eq!(policy.mode(), DetectionModeV1::Hybrid);
    assert_eq!(policy.confirm(), ConfirmModeV1::Require);
    assert_eq!(policy.emit(), EmitModeV1::Sarif);
    assert_eq!(policy.rule().id().local_name(), "rule.hybrid");
    assert_eq!(
        policy
            .scope()
            .and_then(|use_site| use_site.target().host_selector())
            .map(|target| target.spelling()),
        Some("repo")
    );
    assert_eq!(
        policy
            .pattern()
            .and_then(|use_site| use_site.target().host_selector())
            .map(|target| target.spelling()),
        Some("pattern.hybrid")
    );
    let nested = policy
        .nested_graph()
        .expect("semantic confirm FIND retained");
    assert_eq!(nested.kind(), GraphQueryKindV1::Reachability);
    assert_eq!(nested.depth(), 8);

    let CompiledItemV2::Planned(explain) = &compiled.items()[2] else {
        panic!("EXPLAIN must be planned");
    };
    let PlannedItemV2::Presentation { plan, .. } = explain.as_ref() else {
        panic!("EXPLAIN must keep its presentation plan");
    };
    let PlanPolicyV2::Presentation(ResolvedPresentationKindV2::Explain {
        target,
        evidence,
        truth,
    }) = plan.policy()
    else {
        panic!("EXPLAIN must keep its evidence/truth policy");
    };
    assert_eq!(*evidence, EvidenceModeV1::Trace);
    assert_eq!(*truth, TruthModeV1::Confirm);
    let ResolvedExplainTargetV2::Entity(target) = target else {
        panic!("EXPLAIN must keep its resolved entity target");
    };
    assert_eq!(
        target
            .target()
            .host_selector()
            .map(|target| target.spelling()),
        Some("rule.sqli")
    );
}

#[test]
fn find_selectors_and_summary_facets_survive_source_to_final_plan() {
    let taint = compile_source_document_v3(
        DOC_SOURCE,
        concat!(
            "find taint api.entry -> db.write | via=mid.one,mid.two ",
            "| through=must.pass | sanitizers=authz.clean | scope=repo ",
            "| window=recent | direction=backward | frontier=edge | depth=3",
        ),
        &source_policy_authority(),
    )
    .expect("fully selected FIND compiles");
    let CompiledItemV2::Planned(item) = &taint.items()[0] else {
        panic!("FIND must be planned");
    };
    let PlanPolicyV2::Graph(retention) = item.plan().expect("FIND plan").policy() else {
        panic!("FIND must retain graph policy");
    };
    assert_eq!(retention.direction(), Some(GraphDirectionV2::Backward));
    assert!(retention.summary_facets().is_empty());
    let references = retention
        .references()
        .iter()
        .map(|reference| {
            (
                reference.role(),
                reference
                    .target()
                    .host_selector()
                    .expect("source fixture uses host selectors")
                    .spelling(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        references,
        [
            (GraphReferenceRoleV2::Source, "api.entry"),
            (GraphReferenceRoleV2::Sink, "db.write"),
            (GraphReferenceRoleV2::Scope, "repo"),
            (GraphReferenceRoleV2::Window, "recent"),
            (GraphReferenceRoleV2::Frontier, "edge"),
            (GraphReferenceRoleV2::Via, "mid.one"),
            (GraphReferenceRoleV2::Via, "mid.two"),
            (GraphReferenceRoleV2::Through, "must.pass"),
            (GraphReferenceRoleV2::Sanitizer, "authz.clean"),
        ]
    );

    let summary = compile_source_document_v3(
        DOC_SOURCE,
        "find summary | callable=svc.handler | facets=calls,reads,writes | direction=forward | depth=4",
        &source_policy_authority(),
    )
    .expect("summary FIND compiles");
    let CompiledItemV2::Planned(item) = &summary.items()[0] else {
        panic!("summary FIND must be planned");
    };
    let PlanPolicyV2::Graph(retention) = item.plan().expect("summary plan").policy() else {
        panic!("summary FIND must retain graph policy");
    };
    assert_eq!(retention.direction(), Some(GraphDirectionV2::Forward));
    assert_eq!(retention.summary_facets(), ["calls", "reads", "writes"]);
    assert_eq!(retention.references().len(), 1);
    assert_eq!(
        retention.references()[0].role(),
        GraphReferenceRoleV2::Callable
    );
}

#[test]
fn inline_view_is_algebra_checked_before_its_catalog_no_plan_outcome() {
    let source = "view active | select=from records r | project r.id as id";
    let compiled = compile_source_document_v3(DOC_SOURCE, source, &authority_with_records())
        .expect("inline View traverses Sema and Algebra before catalog disposition");
    assert!(matches!(
        compiled.items(),
        [CompiledItemV2::NoPlan {
            reason: NoPlanReasonV2::CatalogDeclaration { .. },
            ..
        }]
    ));

    // Negative control: an inline View whose `select` body names an unknown
    // relation must be rejected by Sema before any catalog non-plan outcome, so
    // the positive case cannot stay green if the body traversal is dropped.
    let unresolved = "view active | select=from missing_relation r | project r.id as id";
    let error = compile_source_document_v3(DOC_SOURCE, unresolved, &authority_with_records())
        .expect_err("an unresolvable view body must not reach catalog disposition");
    assert_eq!(failure_code(&error), "sema.unresolved_reference");
}

#[test]
fn duplicate_authored_projection_alias_rejects_before_a_plan_is_issued() {
    let source = "from records r | project r.id as id, r.id as id";
    let error = compile_source_document_v3(DOC_SOURCE, source, &authority_with_records())
        .expect_err("duplicate authored projection names cannot produce a checked plan");
    assert_eq!(failure_code(&error), "binder.duplicate_declaration");
}

#[test]
fn analyze_and_compile_share_the_same_stage_payloads() {
    let authority = minimal_authority();
    let document = resolve_minimal_document(&authority);
    let compiled = compile_resolved_document_v2(&document).expect("the minimal document compiles");
    let analyzed = quanta_gqlang_v2_pipeline::analyze_resolved_document_v2(&document)
        .expect("sealed analysis succeeds");
    assert_eq!(analyzed.source_id(), compiled.source_id());
    assert_eq!(analyzed.order(), document.order());
    assert_eq!(analyzed.items().len(), compiled.items().len());
    assert!(
        !compiled.items().is_empty(),
        "the minimal document must compile at least one item to compare"
    );
    for index in 0..compiled.items().len() {
        match (&compiled.items()[index], &analyzed.items()[index]) {
            (CompiledItemV2::Planned(compiled_item), AnalysisItemV2::Complete(analyzed_item)) => {
                assert_eq!(compiled_item, analyzed_item);
            }
            (
                CompiledItemV2::NoPlan { reason, .. },
                AnalysisItemV2::NotApplicable(analyzed_reason),
            ) => {
                assert_eq!(reason, analyzed_reason);
            }
            _ => panic!("compile and analysis item dispositions must agree"),
        }
        assert!(
            analyzed.item(analyzed.order()[index]).is_some(),
            "every authored identity is addressable in the analysis document"
        );
    }
}

/// Resolve one source document through the public sema transaction.
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

/// Resolve one source document through the public sema transaction.
fn resolve_minimal_document(
    authority: &ClosedFrontendAuthorityV2,
) -> quanta_gqlang_v2_sema::SemanticDocumentV2 {
    let parsed = quanta_gqlang_v2_surface::parse_source_document_v2(
        DOC_SOURCE,
        MINIMAL_SOURCE,
        &quanta_gqlang_v2_surface::ParseBudgetV2::standard_v2(),
    )
    .expect("minimal source parses");
    let checked = CheckedSyntaxDocumentV2::try_from_hir(parsed.hir).expect("HIR integrity holds");
    quanta_gqlang_v2_sema::resolve_checked_document_v2(checked, authority)
        .expect("semantic transaction succeeds")
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
    for row in include_str!("fixtures/execution-v3-vectors.tsv")
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
    assert_eq!(selected, 15, "the shared V3 corpus must not be vacuous");
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
                _ => None,
            }).collect::<Vec<_>>();
        assert_eq!(symbols, [("length", None), ("+", Some(101))], "{source}");
    }
}
