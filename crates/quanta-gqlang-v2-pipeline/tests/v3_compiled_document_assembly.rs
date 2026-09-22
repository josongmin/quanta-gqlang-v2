#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
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
    compile_resolved_document_v2, compile_source_document_v2, format_source_document_v2,
    AnalysisItemV2, CompiledItemV2, DocumentAssemblyCodeV2, FrontendTransactionErrorV2,
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
    CardinalityV1, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2, FrontendBudgetV2,
    GraphQueryKindV1, GraphReferenceCatalogV2, GraphReferenceInputV2, GraphReferenceKindV2,
    GraphReferenceRoleV2, LocatedSpanV2, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1,
    RelationCatalogV2, RelationSourceInputV2, SemanticItemIdV2, SemanticItemV2, SourceIdV2,
    SourceRegistrationV2, SourceRegistryViewV2, SpanV1, ValueTypeV1,
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
            provenance_namespace: ProvenanceNamespaceV2::try_new("host", location.clone())
                .expect("provenance namespace"),
            qualified_name: "app::records".to_owned(),
            lookup_name: "records".to_owned(),
            row: RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
                "id",
                ValueTypeV1::int(),
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
    let CompiledItemV2::Planned(PlannedItemV2::Relation { plan, .. }) = item else {
        panic!("a relational item must compile to a planned relation item");
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
    let CompiledItemV2::Planned(PlannedItemV2::Graph { item_id, .. }) = &shallow_plan.items()[0]
    else {
        panic!("a FIND item must compile to a planned graph item");
    };
    assert_eq!(*item_id, shallow.graph_item_id());
}

#[test]
fn nested_check_retains_its_own_graph_identity() {
    let fixture = reference_nested_check_document_v2(bounded_query(GraphQueryKindV1::Path, 2))
        .expect("nested CHECK fixture resolves");
    let compiled = compile_resolved_document_v2(fixture.document()).expect("CHECK compiles");
    assert_eq!(compiled.items().len(), 1);
    let CompiledItemV2::Planned(PlannedItemV2::Check { item_id, plan }) = &compiled.items()[0]
    else {
        panic!("a CHECK item must compile to a planned CHECK item");
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
    for member in program.members() {
        assert_eq!(member.program_id(), program.program_id());
    }
    for item in compiled.items() {
        let CompiledItemV2::Planned(PlannedItemV2::RuleMember(member)) = item else {
            panic!("every source rule compiles to a rule-program member reference");
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
    let compiled = compile_source_document_v2(DOC_SOURCE, MINIMAL_SOURCE, &authority)
        .expect("the minimal document compiles");
    assert_eq!(compiled.source_id(), DOC_SOURCE);
    assert_eq!(compiled.items().len(), 3);
    assert!(compiled.rule_program().is_some());

    let dispositions: Vec<&'static str> = compiled
        .items()
        .iter()
        .map(|item| match item {
            CompiledItemV2::NoPlan { .. } => "no_plan",
            CompiledItemV2::Planned(PlannedItemV2::RuleMember(_)) => "rule_member",
            CompiledItemV2::Planned(PlannedItemV2::Presentation { .. }) => "presentation",
            CompiledItemV2::Planned(_) => "planned",
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
fn analyze_and_compile_share_the_same_stage_payloads() {
    let authority = minimal_authority();
    let document = resolve_minimal_document(&authority);
    let compiled = compile_resolved_document_v2(&document).expect("the minimal document compiles");
    let analyzed = quanta_gqlang_v2_pipeline::analyze_resolved_document_v2(&document)
        .expect("sealed analysis succeeds");
    assert_eq!(analyzed.source_id(), compiled.source_id());
    assert_eq!(analyzed.order(), document.order());
    assert_eq!(analyzed.items().len(), compiled.items().len());
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
    let compiled = compile_source_document_v2(DOC_SOURCE, &composition_source(2), &authority)
        .expect("the composition compiles");
    assert_eq!(compiled.items().len(), 1);
    let CompiledItemV2::Planned(PlannedItemV2::Composition { item_id, plan }) =
        &compiled.items()[0]
    else {
        panic!("a composition item must compile to a planned composition item");
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
    let other = compile_source_document_v2(DOC_SOURCE, explain_body, &authority_with_records())
        .expect("the EXPLAIN-body composition compiles");
    assert_ne!(
        compiled.items(),
        other.items(),
        "a swapped composition body arm must change the assembled plan"
    );
}

#[test]
fn typed_relation_source_requires_and_retains_host_authority() {
    let compiled = compile_source_document_v2(DOC_SOURCE, RECORD_SOURCE, &authority_with_records())
        .expect("an admitted host relation compiles");
    assert_eq!(compiled.items().len(), 1);
    let CompiledItemV2::Planned(PlannedItemV2::Relation { plan, .. }) = &compiled.items()[0] else {
        panic!("a typed relation compiles to a planned relation item");
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

    let error = compile_source_document_v2(DOC_SOURCE, RECORD_SOURCE, &minimal_authority())
        .expect_err("an absent host relation must reject");
    assert_eq!(failure_code(&error), "sema.unresolved_reference");
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
    assert!(formatted.contains("let answer = 1"));
    assert!(formatted.contains("show capabilities"));

    let tiny = FrontendBudgetV2::try_new(
        8, 4_194_304, 131_072, 4_096, 64, 262_144, 4_096, 4_096, 1_000_000, 50_000, 200_000,
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
}
