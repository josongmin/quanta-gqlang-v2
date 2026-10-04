#![expect(
    clippy::panic_in_result_fn,
    reason = "these rails assert exact plan fields and return a typed fixture error, so every assertion is a deliberate crash on drift"
)]

use quanta_gqlang_v2_algebra::lower_relation_item;
use quanta_gqlang_v2_sema::test_support::{
    deep_relation_chain_document_v2, forged_cardinality_downgrade_document_v2,
    forged_small_receipt_relation_chain_document_v2, reference_graph_document_v2,
    reference_relation_document_v2, reference_rule_document_v2, second_source_relation_document_v2,
    ReferenceBudgetV2, ReferenceGraphDocumentV2, ReferenceGraphReferenceV2,
    ReferenceGraphRequestV2, ReferenceRelationDocumentV2, ReferenceRelationFixtureV2,
    ReferenceRuleFixtureV2, SMALL_BUDGET_MAX_SEMANTIC_STEPS,
};
use quanta_gqlang_v2_sema::{GraphQueryKindV1, GraphReferenceRoleV2};

use super::*;

/// The checked location a document-level outcome is attributed to.
fn expected_anchor(
    document: &SemanticDocumentV2,
    id: SemanticItemIdV2,
) -> Option<CheckedLocatedSpanV2> {
    document.item(id).and_then(item_location)
}

/// The visited checked algebra nodes and folded effects of a plan that must
/// carry the relational policy. Every other arm is a named failure, so the
/// match stays exhaustive over the closed policy vocabulary.
fn relational_policy(plan: &SemanticPlanV2) -> Result<(u32, EffectSummaryV2), String> {
    match plan.policy() {
        PlanPolicyV2::Relational { nodes, effects } => Ok((*nodes, *effects)),
        PlanPolicyV2::Graph(_)
        | PlanPolicyV2::Check(_)
        | PlanPolicyV2::Presentation(_)
        | PlanPolicyV2::Rule(_)
        | PlanPolicyV2::MetaRequireCertified
        | PlanPolicyV2::Composition(_)
        | PlanPolicyV2::Command(_) => Err("expected a relational policy".to_owned()),
    }
}

/// One reference relation document plus the sealed planning input lowered
/// from it.
fn relational_fixture(
    fixture: ReferenceRelationFixtureV2,
) -> Result<(ReferenceRelationDocumentV2, SemanticPlanInputV2), String> {
    let document = reference_relation_document_v2(fixture)?;
    let input = lower_relation_item(document.document(), document.item_id())
        .map(|item| SemanticPlanInputV2::Relational(Box::new(item)))
        .map_err(|error| format!("{error:?}"))?;
    Ok((document, input))
}

/// A taint FIND document plus the sealed graph planning input.
fn graph_fixture() -> Result<(ReferenceGraphDocumentV2, SemanticPlanInputV2), String> {
    let document = reference_graph_document_v2(ReferenceGraphRequestV2 {
        kind: GraphQueryKindV1::Taint,
        references: vec![
            ReferenceGraphReferenceV2 {
                role: GraphReferenceRoleV2::Source,
                selector: "input_a",
                span: (20, 27),
            },
            ReferenceGraphReferenceV2 {
                role: GraphReferenceRoleV2::Sink,
                selector: "exec_b",
                span: (30, 36),
            },
        ],
        depth: Some(4),
        direction: None,
        summary_facets: Vec::new(),
        item_span: (0, 8),
    })?;
    let contract = GraphQueryContractV2::from_resolved(document.graph())
        .map_err(|error| format!("{error:?}"))?;
    Ok((document, SemanticPlanInputV2::Graph(contract)))
}

/// An ordinary relation plans a read-only table whose explain note carries
/// the checked algebra families in root-first order.
#[test]
fn a_relational_scan_plans_a_read_only_table() -> Result<(), String> {
    let (fixture, input) = relational_fixture(ReferenceRelationFixtureV2::Scan)?;
    let document = fixture.document();
    let plan = plan(&input, document).map_err(|error| format!("{error:?}"))?;

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
    assert!(!plan.result().multiplicity_order().is_ordered());
    assert!(plan.capabilities().contains(&CapabilityV2::RelationScan));
    assert_eq!(
        plan.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::LinearInInput
    );
    assert_eq!(
        plan.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::None
    );
    assert_eq!(
        plan.semantic_cost().witness_explosion_risk_class,
        WitnessExplosionRiskClassV2::None
    );
    assert_eq!(
        plan.semantic_cost().ordering_contract,
        OrderingContractV2::Unordered
    );
    assert_eq!(plan.explain().families(), &[AlgebraFamilyV2::Scan]);
    assert!(!plan.explain().recursive());
    assert_eq!(
        Some(plan.explain().root_location()),
        expected_anchor(document, fixture.item_id()).as_ref()
    );
    let (nodes, effects) = relational_policy(&plan)?;
    assert_eq!(nodes, 1);
    assert_eq!(effects.determinism, DeterminismV1::Deterministic);
    Ok(())
}

/// The exact sealed semantic-step limit plans; one more visited stage is
/// refused at that stage, charging before the visit.
#[test]
fn the_exact_step_limit_plans_and_one_more_is_refused() -> Result<(), String> {
    let exact_depth = SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;
    let exact_carrier = deep_relation_chain_document_v2(exact_depth, ReferenceBudgetV2::Standard)?;
    let exact_input = lower_relation_item(exact_carrier.document(), exact_carrier.item_id())
        .map(|item| SemanticPlanInputV2::Relational(Box::new(item)))
        .map_err(|error| format!("{error:?}"))?;
    let exact_document = forged_small_receipt_relation_chain_document_v2(exact_depth)?;
    let mut session = PlanningSessionV2::new(exact_document.document());
    session
        .plan_outcome(&exact_input)
        .map_err(|error| format!("{error:?}"))?;
    assert!(matches!(
        session.plan_outcome(&exact_input),
        Err(PlanningDiagnosticV1::SemanticStepBudgetExceeded { .. })
    ));

    let over_carrier = deep_relation_chain_document_v2(
        SMALL_BUDGET_MAX_SEMANTIC_STEPS,
        ReferenceBudgetV2::Standard,
    )?;
    let over_input = lower_relation_item(over_carrier.document(), over_carrier.item_id())
        .map(|item| SemanticPlanInputV2::Relational(Box::new(item)))
        .map_err(|error| format!("{error:?}"))?;
    let over_document =
        forged_small_receipt_relation_chain_document_v2(SMALL_BUDGET_MAX_SEMANTIC_STEPS)?;
    match plan(&over_input, over_document.document()) {
        Err(PlanningDiagnosticV1::SemanticStepBudgetExceeded { limit, .. }) => {
            assert_eq!(limit, SMALL_BUDGET_MAX_SEMANTIC_STEPS);
        }
        other => return Err(format!("expected the exact budget refusal, got {other:?}")),
    }
    Ok(())
}

/// A carrier from another document is rejected before any classification, and
/// a same-identity carrier with different sealed facts is an identity
/// mismatch.
#[test]
fn a_carrier_must_belong_to_the_document() -> Result<(), String> {
    let (_foreign, foreign_input) = relational_fixture(ReferenceRelationFixtureV2::Scan)?;
    let other = second_source_relation_document_v2()?;
    match plan(&foreign_input, other.document()) {
        Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, .. }) => {
            assert_eq!(reason, InputDocumentMismatchReasonV2::Absent);
        }
        error => {
            return Err(format!(
                "expected an absent-identity rejection, got {error:?}"
            ))
        }
    }

    let (pristine, forged_input) =
        relational_fixture(ReferenceRelationFixtureV2::GroupedAggregateSum)?;
    let forged = forged_cardinality_downgrade_document_v2()?;
    match plan(&forged_input, forged.document()) {
        Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, .. }) => {
            assert_eq!(reason, InputDocumentMismatchReasonV2::IdentityMismatch);
        }
        error => {
            return Err(format!(
                "expected a sealed-identity rejection, got {error:?}"
            ))
        }
    }
    // The untouched carrier still plans against its own document.
    plan(&forged_input, pristine.document()).map_err(|error| format!("{error:?}"))?;
    Ok(())
}

/// A document-wide rule program with no member is a typed non-plan outcome,
/// not an error and not a silent skip.
#[test]
fn an_empty_rule_program_is_a_typed_non_plan() -> Result<(), String> {
    let (fixture, _input) = relational_fixture(ReferenceRelationFixtureV2::Scan)?;
    let document = fixture.document();
    let contract =
        RuleProgramContractV2::from_document(document).map_err(|error| format!("{error:?}"))?;
    let input = SemanticPlanInputV2::Rule(contract);

    match plan_outcome(&input, document).map_err(|error| format!("{error:?}"))? {
        PlanOutcomeV2::NoPlan(reason) => assert_eq!(reason, NoPlanReasonV2::EmptyRuleProgram),
        PlanOutcomeV2::Plan(plan) => {
            return Err(format!("an empty rule program must not plan: {plan:?}"))
        }
    }
    match plan(&input, document) {
        Err(PlanningDiagnosticV1::NoPlan { reason, span }) => {
            assert_eq!(reason, NoPlanReasonV2::EmptyRuleProgram);
            assert_eq!(
                Some(&span),
                expected_anchor(document, fixture.item_id()).as_ref()
            );
        }
        other => {
            return Err(format!(
                "expected the typed non-plan refusal, got {other:?}"
            ))
        }
    }
    Ok(())
}

/// A FIND plans a proof/evidence-only path result that retains the exact
/// sealed witness provenance.
#[test]
fn a_find_plan_retains_the_sealed_witness_provenance() -> Result<(), String> {
    let (document, input) = graph_fixture()?;
    let plan = plan(&input, document.document()).map_err(|error| format!("{error:?}"))?;

    assert_eq!(plan.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(plan.evidence(), EvidenceRequirementV2::WitnessPayload);
    assert_eq!(plan.effect(), EffectClassV2::ProofEvidenceOnly);
    assert_eq!(plan.result().result_kind(), ResultKindV2::PathWitness);
    assert_eq!(
        plan.result().render_requirement(),
        RenderRequirementV2::WitnessPayload
    );
    assert_eq!(
        plan.result().witness_selection(),
        Some(WitnessSelectionPolicyV2::AllWitness)
    );
    assert_eq!(
        plan.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::Closure
    );
    assert_eq!(
        plan.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::BoundDependent
    );
    assert_eq!(
        plan.semantic_cost().witness_explosion_risk_class,
        WitnessExplosionRiskClassV2::CapBounded
    );
    assert!(plan
        .capabilities()
        .contains(&CapabilityV2::WitnessMaterialization));
    assert!(plan
        .capabilities()
        .contains(&CapabilityV2::RecursiveClosure));
    assert!(plan.explain().recursive());
    assert!(plan.explain().families().is_empty());

    let SemanticPlanInputV2::Graph(contract) = &input else {
        return Err("the fixture seals a graph input".to_owned());
    };
    match plan.policy() {
        PlanPolicyV2::Graph(retention) => {
            assert_eq!(retention.item_id(), contract.item_id());
            assert_eq!(retention.depth(), 4);
            assert_eq!(retention.max_witnesses(), contract.max_witnesses());
            assert_eq!(retention.dedup(), contract.dedup());
            assert_eq!(retention.tie_break(), contract.tie_break());
            assert_eq!(
                retention.max_path_length(),
                contract.witness().max_path_length()
            );
        }
        PlanPolicyV2::Relational { .. }
        | PlanPolicyV2::Check(_)
        | PlanPolicyV2::Presentation(_)
        | PlanPolicyV2::Rule(_)
        | PlanPolicyV2::MetaRequireCertified
        | PlanPolicyV2::Composition(_)
        | PlanPolicyV2::Command(_) => {
            return Err("expected a graph policy".to_owned());
        }
    }
    Ok(())
}

/// A recursive document rule program plans a bounded fixpoint whose policy
/// retains the stratified member summary.
#[test]
fn a_recursive_rule_program_plans_a_bounded_fixpoint() -> Result<(), String> {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::PositiveSelfRecursion)?;
    let document = fixture.document();
    let contract =
        RuleProgramContractV2::from_document(document).map_err(|error| format!("{error:?}"))?;
    let input = SemanticPlanInputV2::Rule(contract);
    let plan = plan(&input, document).map_err(|error| format!("{error:?}"))?;

    assert_eq!(plan.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(plan.effect(), EffectClassV2::ReadOnly);
    assert_eq!(
        plan.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::Closure
    );
    assert_eq!(
        plan.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::BoundedFixpoint
    );
    assert!(plan
        .capabilities()
        .contains(&CapabilityV2::RecursiveClosure));
    assert!(plan.explain().recursive());
    match plan.policy() {
        PlanPolicyV2::Rule(policy) => {
            assert_eq!(policy.item_ids().len(), fixture.item_ids().len());
            assert_eq!(policy.recursive_members().len(), 1);
            assert!(policy.stratum_count() >= 1);
        }
        PlanPolicyV2::Relational { .. }
        | PlanPolicyV2::Graph(_)
        | PlanPolicyV2::Check(_)
        | PlanPolicyV2::Presentation(_)
        | PlanPolicyV2::MetaRequireCertified
        | PlanPolicyV2::Composition(_)
        | PlanPolicyV2::Command(_) => {
            return Err("expected a rule-program policy".to_owned());
        }
    }
    Ok(())
}

/// The traversal is root-first and charges exactly one step per visited
/// node, and a checked sort makes the result order semantic (`D26`).
#[test]
fn the_traversal_is_root_first_and_prices_one_step_per_node() -> Result<(), String> {
    let (fixture, input) = relational_fixture(ReferenceRelationFixtureV2::SortAsc)?;
    let document = fixture.document();
    let plan = plan(&input, document).map_err(|error| format!("{error:?}"))?;

    assert_eq!(
        plan.explain().families(),
        &[AlgebraFamilyV2::Sort, AlgebraFamilyV2::Scan]
    );
    assert_eq!(
        plan.semantic_cost().ordering_contract,
        OrderingContractV2::Ordered
    );
    assert!(plan.result().multiplicity_order().is_ordered());
    assert_eq!(
        plan.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::LinearInInput
    );
    let (nodes, _effects) = relational_policy(&plan)?;
    assert_eq!(nodes, 2);
    Ok(())
}

/// The composition policy survives plan assembly unchanged: the assembled
/// plan's `policy()` IS the retained composition policy, so no stage can
/// substitute or drop the composition identity, its ordered producers, its
/// `WITH` binding, its body identity, or the body's own policy.
///
/// This private unit rail isolates the exact assembly hop
/// ([`SemanticPlanV2::from_parts`]) and checked policy landing
/// ([`crate::CompositionPolicyV2`]). Resolved composition fixtures exercise
/// public construction and [`classify`] in the owner integration suite.
#[test]
fn a_composition_policy_survives_plan_assembly_exactly() -> Result<(), String> {
    let policy = crate::composition::tests::fixture_policy()?;
    let parts = PlanPartsV2 {
        authority_lane: AuthorityLaneV2::FrontendChecked,
        evaluation_determinism: EvaluationDeterminismV2::Deterministic,
        evidence: EvidenceRequirementV2::None,
        effect: EffectClassV2::ReadOnly,
        result: ResultContractV2::new(
            ResultKindV2::Table,
            RenderRequirementV2::RowsOnly,
            None,
            MultiplicityOrderContractV2::from_cardinality(CardinalityV1::Set),
        ),
        capabilities: BTreeSet::new(),
        semantic_cost: SemanticCostV2 {
            estimated_cardinality_class: CardinalityClassV2::LinearInInput,
            recursion_risk_class: RecursionRiskClassV2::None,
            witness_explosion_risk_class: WitnessExplosionRiskClassV2::None,
            ordering_contract: OrderingContractV2::Unordered,
        },
        explain: ExplainNoteV2::new(policy.location().clone(), Vec::new(), false),
        policy: PlanPolicyV2::Composition(policy.clone()),
    };
    let plan = SemanticPlanV2::from_parts(parts);

    match plan.policy() {
        PlanPolicyV2::Composition(retained) => {
            assert_eq!(retained, &policy);
            assert_eq!(retained.root_item_id(), policy.root_item_id());
            assert_eq!(retained.location(), policy.location());
            assert_eq!(retained.producers(), policy.producers());
            assert_eq!(retained.with_binding(), policy.with_binding());
            assert_eq!(retained.body_item_id(), policy.body_item_id());
            assert_eq!(retained.body_family(), policy.body_family());
            assert_eq!(retained.body(), policy.body());
        }
        PlanPolicyV2::Relational { .. }
        | PlanPolicyV2::Graph(_)
        | PlanPolicyV2::Check(_)
        | PlanPolicyV2::Presentation(_)
        | PlanPolicyV2::Rule(_)
        | PlanPolicyV2::MetaRequireCertified
        | PlanPolicyV2::Command(_) => {
            return Err("expected a composition policy".to_owned());
        }
    }
    assert_eq!(plan.policy().keyword(), "composition");
    Ok(())
}
