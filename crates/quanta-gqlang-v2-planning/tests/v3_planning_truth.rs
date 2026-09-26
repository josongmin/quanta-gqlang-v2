#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! v3 planning truth / evidence / effect / result-kind / cost rails (ticket 07).
//!
//! Pins the syntax->sema->algebra/logic->PLANNING seam for every assembled plan
//! field: authority lane, evaluation-determinism lane, evidence requirement,
//! effect class, result contract (kind, render payload, witness selection,
//! multiplicity/order), the fixed semantic-cost schema, and the retained family
//! policy.
//!
//! V2 cut: no field is caller-selected any more, so the V1 rails that requested a
//! truth lane / result kind / effect are replaced by rails that assert the
//! DERIVED value of each sealed arm, plus the determinism rails (two identical
//! plans compare equal and every field is stable under repeated construction).
//!
//! Meta and presentation fixtures are routed through the same sealed owner
//! entry as relations, graphs, checks, and rule programs.

use std::collections::BTreeSet;

use quanta_gqlang_v2_planning::{
    plan, plan_outcome, AuthorityLaneV2, CapabilityV2, CardinalityClassV2, EffectClassV2,
    EvaluationDeterminismV2, EvidenceRequirementV2, NoPlanReasonV2, OrderingContractV2,
    PlanOutcomeV2, RecursionRiskClassV2, RenderRequirementV2, ResultKindV2, SemanticPlanInputV2,
    SemanticPlanV2, WitnessExplosionRiskClassV2,
};
use quanta_gqlang_v2_sema::test_support::{
    ReferenceMetaFixtureV2, ReferenceRelationFixtureV2, ReferenceRuleFixtureV2,
};
use quanta_gqlang_v2_types::{
    AggregationClassV1, CardinalityV1, DeterminismV1, MonotonicityV1, PurityV1,
};

mod support;

use support::{check_input, graph_input, planned_relation, relational_policy, rule_input};

// ---- derived lanes on a relation -------------------------------------------

/// group + aggregate: every derived plan field is populated with its exact
/// sealed-derived value — authority lane, determinism lane, evidence, effect,
/// result contract, capability set, cost schema, explain note, and the retained
/// relational policy.
#[test]
fn a_group_aggregate_plan_populates_every_derived_field() {
    let (document, input) =
        support::relation_input(ReferenceRelationFixtureV2::GroupedAggregateSum)
            .expect("the grouped-aggregate fixture");
    let planned = plan(&input, document.document()).expect("a grouped aggregate plans");

    assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(
        planned.evaluation_determinism(),
        EvaluationDeterminismV2::Deterministic
    );
    assert_eq!(planned.evidence(), EvidenceRequirementV2::None);
    assert_eq!(planned.effect(), EffectClassV2::ReadOnly);

    assert_eq!(planned.result().result_kind(), ResultKindV2::Table);
    assert_eq!(
        planned.result().render_requirement(),
        RenderRequirementV2::RowsOnly
    );
    assert_eq!(planned.result().witness_selection(), None);
    assert_eq!(
        planned.result().multiplicity_order().cardinality,
        CardinalityV1::Set
    );

    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([
            CapabilityV2::RelationScan,
            CapabilityV2::Grouping,
            CapabilityV2::Aggregation,
        ])
    );
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::LinearInInput
    );
    assert_eq!(
        planned.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::None
    );
    assert_eq!(
        planned.semantic_cost().witness_explosion_risk_class,
        WitnessExplosionRiskClassV2::None
    );
    assert_eq!(
        planned.semantic_cost().ordering_contract,
        OrderingContractV2::Unordered
    );

    assert_eq!(
        planned.explain().families(),
        &[
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Aggregate,
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Group,
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Scan,
        ]
    );
    assert!(!planned.explain().recursive());

    let (nodes, effects) = relational_policy(&planned).expect("a relational policy");
    assert_eq!(nodes, 3);
    assert_eq!(effects.purity, PurityV1::Pure);
    assert_eq!(effects.determinism, DeterminismV1::Deterministic);
    assert_eq!(effects.monotonicity, MonotonicityV1::Monotone);
    assert_eq!(effects.aggregation, AggregationClassV1::Aggregate);
}

/// A checked sort makes order semantic, and its folded effects stay monotone and
/// scalar: ordering is a result contract fact, not an effect class change.
#[test]
fn a_sort_plan_marks_order_semantic() {
    let (_document, planned) =
        planned_relation(ReferenceRelationFixtureV2::SortAsc).expect("the SortAsc fixture");
    assert_eq!(
        planned.semantic_cost().ordering_contract,
        OrderingContractV2::Ordered
    );
    assert!(planned.result().multiplicity_order().is_ordered());
    assert!(planned.capabilities().contains(&CapabilityV2::Ordering));
    let (nodes, effects) = relational_policy(&planned).expect("a relational policy");
    assert_eq!(nodes, 2);
    assert_eq!(effects.monotonicity, MonotonicityV1::Monotone);
    assert_eq!(effects.aggregation, AggregationClassV1::Scalar);
}

/// A SCALAR aggregate collapses the whole input to one row: the class is
/// `ScalarOrSingleton`, and the fold is visible in the retained effect summary.
#[test]
fn a_scalar_aggregate_collapses_to_one_row() {
    let (_document, planned) =
        planned_relation(ReferenceRelationFixtureV2::ScalarCountStar).expect("scalar count");
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::ScalarOrSingleton
    );
    let (_nodes, effects) = relational_policy(&planned).expect("a relational policy");
    assert_eq!(effects.aggregation, AggregationClassV1::Aggregate);
}

/// The determinism LANE is folded from the sealed expression effects, never
/// hardcoded: an ordinary relation is deterministic, while a relation whose
/// predicate proves a relational EXISTS carries impure, non-deterministic,
/// non-monotone effects.
#[test]
fn the_determinism_lane_is_folded_from_sealed_expression_effects() {
    let (_document, plain) =
        planned_relation(ReferenceRelationFixtureV2::Scan).expect("the Scan fixture");
    assert_eq!(
        plain.evaluation_determinism(),
        EvaluationDeterminismV2::Deterministic
    );
    let (_nodes, effects) = relational_policy(&plain).expect("a relational policy");
    assert_eq!(effects.purity, PurityV1::Pure);
    assert_eq!(effects.monotonicity, MonotonicityV1::Monotone);

    let (_document, exists) = planned_relation(ReferenceRelationFixtureV2::ExistsWithBoolConjunct)
        .expect("the existence-conjunct fixture");
    assert_eq!(
        exists.evaluation_determinism(),
        EvaluationDeterminismV2::NonDeterministic
    );
    let (_nodes, effects) = relational_policy(&exists).expect("a relational policy");
    assert_eq!(effects.purity, PurityV1::Impure);
    assert_eq!(effects.determinism, DeterminismV1::NonDeterministic);
    assert_eq!(effects.monotonicity, MonotonicityV1::NonMonotone);
    // Determinism is an independent lane: the join predicate makes the plan
    // non-monotone without making evaluation non-deterministic.
    let (_document, joined) =
        planned_relation(ReferenceRelationFixtureV2::InnerJoinOn).expect("the ON-join fixture");
    assert_eq!(
        joined.evaluation_determinism(),
        EvaluationDeterminismV2::Deterministic
    );
    let (_nodes, effects) = relational_policy(&joined).expect("a relational policy");
    assert_eq!(effects.monotonicity, MonotonicityV1::NonMonotone);
}

/// Every positional algebra arm plans a read-only TABLE with rows-only rendering
/// and NO standalone evidence requirement. V2 replacement for the V1
/// caller-selected `result_kind = Table` / `effect = ReadOnly` rail.
#[test]
fn every_relational_arm_plans_a_table_read_only_result() {
    for fixture in [
        ReferenceRelationFixtureV2::Scan,
        ReferenceRelationFixtureV2::Filter,
        ReferenceRelationFixtureV2::CrossJoin,
        ReferenceRelationFixtureV2::InnerJoinOn,
        ReferenceRelationFixtureV2::SelfJoinInner,
        ReferenceRelationFixtureV2::ProjectMultipleExpressions,
        ReferenceRelationFixtureV2::GroupedAggregateSum,
        ReferenceRelationFixtureV2::ScalarCountStar,
        ReferenceRelationFixtureV2::SortAsc,
        ReferenceRelationFixtureV2::SortMultiKeyMixedDirections,
        ReferenceRelationFixtureV2::SortNullsLastNullableField,
        ReferenceRelationFixtureV2::ExistsWithBoolConjunct,
    ] {
        let (_document, planned) = planned_relation(fixture).expect("the fixture plans");
        assert_eq!(
            planned.authority_lane(),
            AuthorityLaneV2::FrontendChecked,
            "{fixture:?}"
        );
        assert_eq!(
            planned.evidence(),
            EvidenceRequirementV2::None,
            "{fixture:?}"
        );
        assert_eq!(planned.effect(), EffectClassV2::ReadOnly, "{fixture:?}");
        assert_eq!(
            planned.result().result_kind(),
            ResultKindV2::Table,
            "{fixture:?}"
        );
        assert_eq!(
            planned.result().render_requirement(),
            RenderRequirementV2::RowsOnly,
            "{fixture:?}"
        );
        assert_eq!(planned.result().witness_selection(), None, "{fixture:?}");
    }
}

// ---- derived lanes on the non-relational arms ------------------------------

/// The graph arm is a proof/evidence-ONLY path result: V2 replacement for the V1
/// "proof-confirmed lane requires a proof tree" rail. Planning can no longer be
/// asked for a proof lane over an ordinary result; the graph arm's own sealed
/// witness contract is what carries the evidence.
#[test]
fn a_graph_arm_carries_proof_evidence_only() {
    let (document, input) = graph_input(quanta_gqlang_v2_sema::GraphQueryKindV1::Taint, 4)
        .expect("the reference taint FIND");
    let planned = plan(&input, document.document()).expect("a bounded FIND plans");
    assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(planned.evidence(), EvidenceRequirementV2::WitnessPayload);
    assert_eq!(planned.effect(), EffectClassV2::ProofEvidenceOnly);
    assert_eq!(planned.result().result_kind(), ResultKindV2::PathWitness);
    assert_eq!(
        planned.result().render_requirement(),
        RenderRequirementV2::WitnessPayload
    );
}

/// The CHECK arm is an alert with a source-location evidence requirement under
/// the sealed `Skip` confirmation: the V1 caller-selected alert rail is replaced
/// by the sealed CHECK policy the plan retains.
#[test]
fn a_check_arm_carries_source_location_evidence() {
    let (document, input) = check_input().expect("the nested CHECK fixture");
    let planned = plan(&input, document.document()).expect("the CHECK plans");
    assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(planned.evidence(), EvidenceRequirementV2::SourceLocation);
    assert_eq!(planned.effect(), EffectClassV2::ReadOnly);
    assert_eq!(planned.result().result_kind(), ResultKindV2::Alert);
    assert_eq!(
        planned.result().render_requirement(),
        RenderRequirementV2::SourceLocation
    );
    let policy = support::check_policy(&planned).expect("a CHECK policy");
    assert_eq!(
        policy.mode(),
        quanta_gqlang_v2_sema::DetectionModeV1::SemanticOnly
    );
    assert_eq!(policy.confirm(), quanta_gqlang_v2_sema::ConfirmModeV1::Skip);
    assert_eq!(policy.emit(), quanta_gqlang_v2_sema::EmitModeV1::Findings);
    assert_eq!(policy.nested_graph().map(|nested| nested.depth()), Some(4));
}

/// A document rule program plans a read-only TABLE whose multiplicity/order
/// contract is a SET: a least-fixpoint closure eliminates duplicates and carries
/// no semantic order.
#[test]
fn a_rule_program_plans_a_table_read_only_set_result() {
    for fixture in [
        ReferenceRuleFixtureV2::SingleRule,
        ReferenceRuleFixtureV2::PositiveSelfRecursion,
    ] {
        let (document, input) = rule_input(fixture).expect("the rule fixture");
        let planned = plan(&input, document.document()).expect("the program plans");
        assert_eq!(
            planned.authority_lane(),
            AuthorityLaneV2::FrontendChecked,
            "{fixture:?}"
        );
        assert_eq!(
            planned.evidence(),
            EvidenceRequirementV2::None,
            "{fixture:?}"
        );
        assert_eq!(planned.effect(), EffectClassV2::ReadOnly, "{fixture:?}");
        assert_eq!(
            planned.result().result_kind(),
            ResultKindV2::Table,
            "{fixture:?}"
        );
        assert_eq!(
            planned.result().multiplicity_order().cardinality,
            CardinalityV1::Set,
            "{fixture:?}"
        );
        assert!(planned
            .result()
            .multiplicity_order()
            .eliminates_duplicates());
    }
}

/// The authority lane is `FrontendChecked` for ordinary sealed arms and the
/// certified lane is reachable only through an exact `REQUIRE CERTIFIED`
/// directive, whose proof/effect/result policy is retained end to end.
#[test]
fn the_certified_lane_is_reserved_for_a_require_certified_directive() {
    for fixture in [
        ReferenceRelationFixtureV2::Scan,
        ReferenceRelationFixtureV2::GroupedAggregateSum,
    ] {
        let (_document, planned) = planned_relation(fixture).expect("the fixture plans");
        assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);
    }
    let (document, input) = graph_input(quanta_gqlang_v2_sema::GraphQueryKindV1::Summary, 2)
        .expect("the reference summary FIND");
    let planned = plan(&input, document.document()).expect("a bounded FIND plans");
    assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);

    let (document, input) =
        rule_input(ReferenceRuleFixtureV2::SingleRule).expect("the single-rule fixture");
    let planned = plan(&input, document.document()).expect("the program plans");
    assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);

    let (document, input) = support::meta_input(ReferenceMetaFixtureV2::RequireCertified)
        .expect("the REQUIRE CERTIFIED fixture seals");
    let planned = plan(&input, document.document()).expect("REQUIRE CERTIFIED plans");
    assert_eq!(
        planned.authority_lane(),
        AuthorityLaneV2::CertifiedRequirementValidated
    );
    assert_eq!(planned.evidence(), EvidenceRequirementV2::ProofTree);
    assert_eq!(planned.effect(), EffectClassV2::ProofEvidenceOnly);
    assert_eq!(planned.result().result_kind(), ResultKindV2::ProofExplain);
    assert_eq!(
        planned.result().render_requirement(),
        RenderRequirementV2::ProvenanceTree
    );
    assert_eq!(planned.policy().keyword(), "meta_require_certified");
    assert_eq!(
        planned.authority_lane().keyword(),
        "certified_requirement_validated"
    );
}

/// An unordered (set) result keeps an unordered contract, and a bag keeps its
/// duplicates: the language does not pretend an order exists (`D26`).
#[test]
fn an_unordered_result_stays_unordered() {
    let (_document, bag) =
        planned_relation(ReferenceRelationFixtureV2::Scan).expect("the Scan fixture");
    assert_eq!(
        bag.semantic_cost().ordering_contract,
        OrderingContractV2::Unordered
    );
    assert!(!bag.result().multiplicity_order().is_ordered());
    assert!(!bag.result().multiplicity_order().eliminates_duplicates());

    let (document, input) = graph_input(quanta_gqlang_v2_sema::GraphQueryKindV1::Slice, 2)
        .expect("the reference slice FIND");
    let set = plan(&input, document.document()).expect("a bounded FIND plans");
    assert_eq!(
        set.semantic_cost().ordering_contract,
        OrderingContractV2::Unordered
    );
    assert!(set.result().multiplicity_order().eliminates_duplicates());
}

// ---- determinism -----------------------------------------------------------

/// Two identical plans compare EQUAL, and every field is stable under repeated
/// construction: the plan is a deterministic value, not an accumulator.
#[test]
fn two_identical_plans_are_equal_and_field_stable() {
    let (document, input) =
        support::relation_input(ReferenceRelationFixtureV2::GroupedAggregateSum)
            .expect("the grouped-aggregate fixture");
    let first = plan(&input, document.document()).expect("the fixture plans");
    let second = plan(&input, document.document()).expect("the fixture plans again");
    assert_eq!(first, second);

    assert_eq!(
        plan_outcome(&input, document.document()).expect("both calls agree"),
        plan_outcome(&input, document.document()).expect("both calls agree")
    );

    assert_eq!(
        first.semantic_cost().fields(),
        second.semantic_cost().fields()
    );
    assert_eq!(first.explain().families(), second.explain().families());
    assert_eq!(
        first.explain().root_location(),
        second.explain().root_location()
    );
    assert_eq!(first.policy(), second.policy());
    assert_eq!(first.result(), second.result());
    assert_eq!(first.capabilities(), second.capabilities());
    assert_eq!(first.authority_lane(), second.authority_lane());
    assert_eq!(
        first.evaluation_determinism(),
        second.evaluation_determinism()
    );
}

/// The same stability holds for a non-relational arm and across the two entry
/// points: `plan` and `plan_outcome` agree on the assembled plan.
#[test]
fn a_graph_plan_is_stable_across_entry_points() {
    let (document, input) = graph_input(quanta_gqlang_v2_sema::GraphQueryKindV1::PointsTo, 3)
        .expect("the reference points-to FIND");
    let planned = plan(&input, document.document()).expect("a bounded FIND plans");
    let outcome = plan_outcome(&input, document.document()).expect("the outcome agrees");
    match outcome {
        PlanOutcomeV2::Plan(same) => assert_eq!(planned, *same),
        PlanOutcomeV2::NoPlan(reason) => panic!("a FIND plans, got NoPlan({reason:?})"),
    }
    assert_eq!(
        planned,
        plan(&input, document.document()).expect("repeated construction agrees")
    );
}

/// The same stability holds for a typed non-plan outcome: two identical
/// refusals carry the same reason and span.
#[test]
fn a_typed_non_plan_outcome_is_stable() {
    let (document, _input) =
        support::relation_input(ReferenceRelationFixtureV2::Scan).expect("the Scan fixture");
    let contract =
        quanta_gqlang_v2_logic::RuleProgramContractV2::from_document(document.document())
            .expect("the document seals an empty program");
    let input = SemanticPlanInputV2::Rule(contract);
    let first =
        plan_outcome(&input, document.document()).expect("an empty program is not an error");
    let second =
        plan_outcome(&input, document.document()).expect("an empty program is not an error");
    assert_eq!(first, second);
    assert_eq!(
        first,
        PlanOutcomeV2::NoPlan(NoPlanReasonV2::EmptyRuleProgram)
    );
}

/// The assembled plan's accessors are the whole read surface: a plan built from
/// the same sealed carrier twice exposes identical values through every
/// accessor, including the fixed cost field ORDER.
#[test]
fn every_plan_field_is_read_through_one_stable_accessor() {
    let (document, input) = support::relation_input(ReferenceRelationFixtureV2::SortDesc)
        .expect("the SortDesc fixture");
    let planned: SemanticPlanV2 = plan(&input, document.document()).expect("a sort plans");
    let repeated = plan(&input, document.document()).expect("a sort plans again");
    assert_eq!(
        planned, repeated,
        "the whole plan must be stable, not just the sampled accessors"
    );
    assert_eq!(
        planned.semantic_cost().fields(),
        [
            ("estimated_cardinality_class", "linear_in_input"),
            ("recursion_risk_class", "none"),
            ("witness_explosion_risk_class", "none"),
            ("ordering_contract", "ordered"),
        ]
    );
    assert_eq!(
        repeated.semantic_cost().fields(),
        planned.semantic_cost().fields()
    );
    assert_eq!(
        planned.explain().recursive(),
        repeated.explain().recursive()
    );
    assert_eq!(
        planned.result().multiplicity_order(),
        repeated.result().multiplicity_order()
    );
}
