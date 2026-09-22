#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Cardinality-class / recursion-risk / witness-risk derivation edge cases
//! (ticket 07, `D24`/`D26`/`D32`) plus the closed keyword sets of every cost,
//! effect, authority/determinism, evidence, and result-kind class.
//!
//! V2 cut: `PlanRequestV1` is DELETED, so every V1 rail that selected
//! `result_kind` / `effect` / `truth_lane` / `recursion` / `witness` by hand now
//! drives the SAME classes from a sealed carrier:
//!
//! - cardinality class comes from the one traversal's post-order fold over the
//!   checked algebra tree (`plan.rs::exit_class`);
//! - recursion risk comes from the document RULE PROGRAM's stratification proof
//!   (a least-fixpoint closure) or from a sealed cyclic FIND/confirm witness
//!   bound — never from a caller-requested `RecursionShapeV1`;
//! - witness-selection policy comes from the SEALED max-witness cap, so the V1
//!   `TopK`/canonical-one selection rails have no V2 caller spelling.
//!
//! Rails whose V2 shape needs a fixture that does not exist yet are listed in
//! the lane report as INTEGRATOR-FIXTURE-PENDING and are marked inline below; the
//! corresponding class/keyword vocabulary stays asserted verbatim.

use quanta_gqlang_v2_planning::{
    plan, AuthorityLaneV2, CardinalityClassV2, EffectClassV2, EvaluationDeterminismV2,
    EvidenceRequirementV2, OrderingContractV2, RecursionRiskClassV2, RenderRequirementV2,
    ResultKindV2, WitnessExplosionRiskClassV2, WitnessSelectionPolicyV2,
};
use quanta_gqlang_v2_sema::test_support::{ReferenceRelationFixtureV2, ReferenceRuleFixtureV2};
use quanta_gqlang_v2_types::{CardinalityV1, ResolvedPredicateIdV2};

mod support;

use support::{check_input, graph_input, planned_relation, rule_input};

// ---- cardinality class -----------------------------------------------------

/// A `project` root is transparent: it forwards the underlying input's class.
#[test]
fn project_forwards_input_class() {
    let (_document, planned) =
        planned_relation(ReferenceRelationFixtureV2::ProjectMultipleExpressions).expect("project");
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::LinearInInput
    );
}

/// A bare scan is linear in its single input.
#[test]
fn scan_root_is_linear_in_input() {
    let (_document, planned) = planned_relation(ReferenceRelationFixtureV2::Scan).expect("scan");
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::LinearInInput
    );
}

/// A `filter` is transparent: it forwards the input's class (linear).
#[test]
fn filter_forwards_input_class() {
    let (_document, planned) =
        planned_relation(ReferenceRelationFixtureV2::Filter).expect("filter");
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::LinearInInput
    );
}

/// A join can multiply input sizes; an ON predicate does not change the class,
/// only the folded monotonicity.
#[test]
fn join_root_is_multiplicative() {
    for fixture in [
        ReferenceRelationFixtureV2::CrossJoin,
        ReferenceRelationFixtureV2::InnerJoinOn,
    ] {
        let (_document, planned) = planned_relation(fixture).expect("join plans");
        assert_eq!(
            planned.semantic_cost().estimated_cardinality_class,
            CardinalityClassV2::Multiplicative,
            "{fixture:?} is multiplicative"
        );
    }
}

/// A self join multiplies two arms of one relation, so it is multiplicative too.
#[test]
fn self_join_root_is_multiplicative() {
    let (_document, planned) =
        planned_relation(ReferenceRelationFixtureV2::SelfJoinInner).expect("self join");
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::Multiplicative
    );
}

/// A GROUPED aggregate forwards its partition's class (one row per distinct
/// key), so it stays linear in its input.
#[test]
fn grouped_aggregate_is_linear_in_input() {
    for fixture in [
        ReferenceRelationFixtureV2::GroupedAggregateSum,
        ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys,
    ] {
        let (_document, planned) = planned_relation(fixture).expect("grouped aggregate plans");
        assert_eq!(
            planned.semantic_cost().estimated_cardinality_class,
            CardinalityClassV2::LinearInInput,
            "{fixture:?} keeps its partition class"
        );
    }
}

/// A SCALAR aggregate (`count(*)` with NO partition keys) collapses every input
/// row to exactly one output row, so its class is `ScalarOrSingleton` — the fold
/// reads the ABSENCE of a non-empty partition from the sealed operator rather
/// than the class of the scan beneath it (`D24`).
#[test]
fn scalar_aggregate_is_scalar_or_singleton() {
    let (_document, planned) =
        planned_relation(ReferenceRelationFixtureV2::ScalarCountStar).expect("scalar aggregate");
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::ScalarOrSingleton
    );
}

/// A relational EXISTS conjunct under a residual filter is bounded by its left
/// arm: the nested existence scan contributes no multiplicity of its own.
#[test]
fn a_filter_over_an_existence_conjunct_stays_linear_in_input() {
    let (_document, planned) = planned_relation(ReferenceRelationFixtureV2::ExistsWithBoolConjunct)
        .expect("exists filter");
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::LinearInInput
    );
}

/// INTEGRATOR-FIXTURE-PENDING: `CardinalityClassV2::Bounded` is derived only from
/// a sealed `Limit` algebra stage or a bounded `SAMPLE` presentation, and
/// `ReferenceRelationFixtureV2` has neither a `Limit` nor a `Union` variant. The
/// keyword is still asserted verbatim below.
#[test]
fn the_unreachable_cardinality_classes_render_stable_keywords() {
    assert_eq!(CardinalityClassV2::Bounded.keyword(), "bounded");
    assert_eq!(
        CardinalityClassV2::Multiplicative.keyword(),
        "multiplicative"
    );
}

// ---- recursion risk --------------------------------------------------------

/// No recursion -> recursion risk `None`, for both a plain relation and a
/// nonrecursive rule program.
#[test]
fn no_recursion_is_risk_none() {
    let (_document, planned) = planned_relation(ReferenceRelationFixtureV2::Scan).expect("scan");
    assert_eq!(
        planned.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::None
    );

    let (document, input) =
        rule_input(ReferenceRuleFixtureV2::SingleRule).expect("the single-rule fixture");
    let planned = plan(&input, document.document()).expect("a single rule plans");
    assert_eq!(
        planned.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::None
    );
}

/// A self-recursive document rule program is a least-fixpoint closure: it
/// terminates by stratification (no path bound needed), so its class is
/// `BoundedFixpoint` and its cardinality class is `Closure`.
#[test]
fn a_recursive_rule_program_is_a_bounded_fixpoint_closure() {
    let (document, input) =
        rule_input(ReferenceRuleFixtureV2::PositiveSelfRecursion).expect("recursive program");
    let planned = plan(&input, document.document()).expect("positive recursion admits");
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::Closure
    );
    assert_eq!(
        planned.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::BoundedFixpoint
    );
    assert!(planned
        .capabilities()
        .contains(&quanta_gqlang_v2_planning::CapabilityV2::RecursiveClosure));
    assert!(planned.explain().recursive());
}

/// A mutual-recursion pair is one closure stratum whose exact recursive member
/// identities are retained by the plan policy.
#[test]
fn a_mutual_recursion_pair_retains_every_recursive_member() {
    let (document, input) =
        rule_input(ReferenceRuleFixtureV2::MutualRecursionPair).expect("mutual program");
    let planned = plan(&input, document.document()).expect("mutual recursion admits");
    assert_eq!(
        planned.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::BoundedFixpoint
    );
    let policy = support::rule_policy(&planned).expect("a rule program policy");
    assert_eq!(policy.stratum_count(), 1);
    assert_eq!(
        policy.recursive_members(),
        &[
            ResolvedPredicateIdV2::try_new("r").expect("a stable predicate identity"),
            ResolvedPredicateIdV2::try_new("s").expect("a stable predicate identity"),
        ]
    );
}

/// A cyclic FIND closure terminates only because an explicit witness/path bound
/// caps it, so its recursion risk is `BoundDependent` (never
/// `BoundedFixpoint`).
#[test]
fn a_cyclic_witness_closure_is_bound_dependent() {
    let (document, input) = graph_input(quanta_gqlang_v2_sema::GraphQueryKindV1::Taint, 4)
        .expect("the reference taint FIND");
    let planned = plan(&input, document.document()).expect("a bounded FIND plans");
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::Closure
    );
    assert_eq!(
        planned.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::BoundDependent
    );
}

/// A CHECK whose nested semantic-confirm FIND is cyclic carries the same
/// bound-dependent closure risk as the standalone FIND.
#[test]
fn a_nested_confirm_closure_is_bound_dependent() {
    let (document, input) = check_input().expect("the nested CHECK fixture");
    let planned = plan(&input, document.document()).expect("the CHECK plans");
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::Closure
    );
    assert_eq!(
        planned.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::BoundDependent
    );
}

// ---- witness explosion risk / selection policy -----------------------------

/// No witness family -> witness explosion risk `None`, for a relation and for a
/// rule program.
#[test]
fn no_witness_is_explosion_none() {
    let (_document, planned) = planned_relation(ReferenceRelationFixtureV2::Scan).expect("scan");
    assert_eq!(
        planned.semantic_cost().witness_explosion_risk_class,
        WitnessExplosionRiskClassV2::None
    );

    let (document, input) =
        rule_input(ReferenceRuleFixtureV2::SingleRule).expect("the single-rule fixture");
    let planned = plan(&input, document.document()).expect("a single rule plans");
    assert_eq!(
        planned.semantic_cost().witness_explosion_risk_class,
        WitnessExplosionRiskClassV2::None
    );
}

/// A cyclic witness family is cap-bounded regardless of the sealed cap: its
/// termination depends on the explicit path-length bound, and the sealed cap
/// selects every witness up to that cap.
#[test]
fn a_cyclic_find_witness_is_cap_bounded_and_admits_all_witnesses() {
    let (document, input) = graph_input(quanta_gqlang_v2_sema::GraphQueryKindV1::Taint, 4)
        .expect("the reference taint FIND");
    let planned = plan(&input, document.document()).expect("a bounded FIND plans");
    assert_eq!(
        planned.semantic_cost().witness_explosion_risk_class,
        WitnessExplosionRiskClassV2::CapBounded
    );
    assert_eq!(
        planned.result().witness_selection(),
        Some(WitnessSelectionPolicyV2::AllWitness)
    );
    // The selection is DERIVED from the sealed graph cap, not chosen by the
    // caller: the plan carries the sealed witness provenance itself.
    assert_eq!(planned.policy().keyword(), "graph");
}

/// The witness-selection policy is derived from the SEALED max-witness cap
/// alone: a cap wider than one admits every witness. INTEGRATOR-FIXTURE-PENDING
/// for the other reachable arm: `CanonicalOne` requires a sealed cap of exactly
/// one, and every reference document seals the standard witness budget
/// (`max_witnesses = 10000`), so no fixture reaches it; `TopK` has no V2 spelling
/// at all because no caller may select a witness policy.
#[test]
fn witness_selection_is_derived_from_the_sealed_cap() {
    let (document, input) = graph_input(quanta_gqlang_v2_sema::GraphQueryKindV1::Path, 4)
        .expect("the reference path FIND");
    let contract = match &input {
        quanta_gqlang_v2_planning::SemanticPlanInputV2::Graph(contract) => contract,
        other => panic!("the fixture seals a graph carrier, got {other:?}"),
    };
    assert_eq!(contract.max_witnesses(), 10_000);
    let planned = plan(&input, document.document()).expect("a bounded FIND plans");
    assert_eq!(
        planned.result().witness_selection(),
        Some(WitnessSelectionPolicyV2::AllWitness)
    );
}

// ---- ordering contract -----------------------------------------------------

/// A bag result (no order) is unordered, like a set, and preserves duplicates.
#[test]
fn bag_result_is_unordered_and_keeps_duplicates() {
    let (_document, planned) = planned_relation(ReferenceRelationFixtureV2::Scan).expect("scan");
    assert_eq!(
        planned.semantic_cost().ordering_contract,
        OrderingContractV2::Unordered
    );
    let multiplicity = planned.result().multiplicity_order();
    assert!(!multiplicity.is_ordered());
    assert!(!multiplicity.eliminates_duplicates());
    assert_eq!(multiplicity.cardinality, CardinalityV1::Bag);
}

/// A set result is unordered but DOES eliminate duplicates (`D26`: an unordered
/// result stays unordered; the language never invents an order).
#[test]
fn set_result_is_unordered_and_eliminates_duplicates() {
    let (_document, planned) =
        planned_relation(ReferenceRelationFixtureV2::GroupedAggregateSum).expect("grouped plan");
    assert_eq!(
        planned.semantic_cost().ordering_contract,
        OrderingContractV2::Unordered
    );
    let multiplicity = planned.result().multiplicity_order();
    assert!(!multiplicity.is_ordered());
    assert!(multiplicity.eliminates_duplicates());
    assert_eq!(multiplicity.cardinality, CardinalityV1::Set);
}

/// A checked sort makes ORDER semantic, so the ordering contract and the
/// multiplicity/order contract both report `ordered` on a sequence result.
#[test]
fn a_checked_sort_is_ordered() {
    for fixture in [
        ReferenceRelationFixtureV2::SortAsc,
        ReferenceRelationFixtureV2::SortDesc,
        ReferenceRelationFixtureV2::SortAliasKeyed,
        ReferenceRelationFixtureV2::SortByProjectedExpression,
    ] {
        let (_document, planned) = planned_relation(fixture).expect("sort plans");
        assert_eq!(
            planned.semantic_cost().ordering_contract,
            OrderingContractV2::Ordered,
            "{fixture:?} is ordered"
        );
        assert!(planned.result().multiplicity_order().is_ordered());
        assert_eq!(
            planned.result().multiplicity_order().cardinality,
            CardinalityV1::Sequence
        );
        assert!(!planned
            .result()
            .multiplicity_order()
            .eliminates_duplicates());
    }
}

/// Every witness-bearing arm materializes witnesses under a SET outcome, so it
/// is unordered and duplicate-eliminating — including the CHECK alert arm whose
/// nested confirm FIND is bounded.
#[test]
fn witness_and_alert_results_are_unordered_sets() {
    let (document, input) = graph_input(quanta_gqlang_v2_sema::GraphQueryKindV1::Slice, 2)
        .expect("the reference slice FIND");
    let planned = plan(&input, document.document()).expect("a bounded FIND plans");
    assert!(!planned.result().multiplicity_order().is_ordered());
    assert!(planned
        .result()
        .multiplicity_order()
        .eliminates_duplicates());

    let (document, input) = check_input().expect("the nested CHECK fixture");
    let planned = plan(&input, document.document()).expect("the CHECK plans");
    assert_eq!(planned.result().result_kind(), ResultKindV2::Alert);
    assert!(!planned.result().multiplicity_order().is_ordered());
}

// ---- closed keyword sets ---------------------------------------------------

#[test]
fn cardinality_class_keywords_are_stable() {
    assert_eq!(
        CardinalityClassV2::ScalarOrSingleton.keyword(),
        "scalar_or_singleton"
    );
    assert_eq!(CardinalityClassV2::Bounded.keyword(), "bounded");
    assert_eq!(
        CardinalityClassV2::LinearInInput.keyword(),
        "linear_in_input"
    );
    assert_eq!(
        CardinalityClassV2::Multiplicative.keyword(),
        "multiplicative"
    );
    assert_eq!(CardinalityClassV2::Closure.keyword(), "closure");
}

#[test]
fn recursion_risk_keywords_are_stable() {
    assert_eq!(RecursionRiskClassV2::None.keyword(), "none");
    assert_eq!(
        RecursionRiskClassV2::BoundedFixpoint.keyword(),
        "bounded_fixpoint"
    );
    assert_eq!(
        RecursionRiskClassV2::BoundDependent.keyword(),
        "bound_dependent"
    );
}

#[test]
fn witness_explosion_keywords_are_stable() {
    assert_eq!(WitnessExplosionRiskClassV2::None.keyword(), "none");
    assert_eq!(
        WitnessExplosionRiskClassV2::SingleWitness.keyword(),
        "single_witness"
    );
    assert_eq!(
        WitnessExplosionRiskClassV2::BoundedSet.keyword(),
        "bounded_set"
    );
    assert_eq!(
        WitnessExplosionRiskClassV2::CapBounded.keyword(),
        "cap_bounded"
    );
}

#[test]
fn ordering_contract_keywords_are_stable() {
    assert_eq!(OrderingContractV2::Unordered.keyword(), "unordered");
    assert_eq!(OrderingContractV2::Ordered.keyword(), "ordered");
}

/// V2 replacement for the deleted `TruthLaneV1` keyword rail: the V1 lane
/// conflated WHO checked the result with HOW deterministic it is, and the V2 cut
/// split it into [`AuthorityLaneV2`] and [`EvaluationDeterminismV2`]. Both closed
/// sets are asserted verbatim, and the deleted advisory/candidate lane has no
/// spelling in either.
#[test]
fn authority_lane_keywords_are_stable() {
    assert_eq!(
        AuthorityLaneV2::FrontendChecked.keyword(),
        "frontend_checked"
    );
    assert_eq!(
        AuthorityLaneV2::CertifiedRequirementValidated.keyword(),
        "certified_requirement_validated"
    );
}

#[test]
fn evaluation_determinism_keywords_are_stable() {
    assert_eq!(
        EvaluationDeterminismV2::Deterministic.keyword(),
        "deterministic"
    );
    assert_eq!(
        EvaluationDeterminismV2::NonDeterministic.keyword(),
        "non_deterministic"
    );
}

#[test]
fn evidence_requirement_keywords_are_stable() {
    assert_eq!(EvidenceRequirementV2::None.keyword(), "none");
    assert_eq!(
        EvidenceRequirementV2::SourceLocation.keyword(),
        "source_location"
    );
    assert_eq!(
        EvidenceRequirementV2::WitnessPayload.keyword(),
        "witness_payload"
    );
    assert_eq!(EvidenceRequirementV2::ProofTree.keyword(), "proof_tree");
}

#[test]
fn effect_class_keywords_are_stable() {
    assert_eq!(EffectClassV2::ReadOnly.keyword(), "read_only");
    assert_eq!(
        EffectClassV2::ProofEvidenceOnly.keyword(),
        "proof_evidence_only"
    );
    assert_eq!(EffectClassV2::ExportOnly.keyword(), "export_only");
    assert_eq!(
        EffectClassV2::MutationForbidden.keyword(),
        "mutation_forbidden"
    );
}

#[test]
fn result_kind_keywords_are_stable() {
    assert_eq!(ResultKindV2::Table.keyword(), "table");
    assert_eq!(ResultKindV2::Alert.keyword(), "alert");
    assert_eq!(ResultKindV2::PathWitness.keyword(), "path_witness");
    assert_eq!(ResultKindV2::ProofExplain.keyword(), "proof_explain");
}

#[test]
fn witness_selection_keywords_are_stable() {
    assert_eq!(
        WitnessSelectionPolicyV2::CanonicalOne.keyword(),
        "canonical_one"
    );
    assert_eq!(WitnessSelectionPolicyV2::TopK { k: 7 }.keyword(), "top_k");
    assert_eq!(
        WitnessSelectionPolicyV2::AllWitness.keyword(),
        "all_witness"
    );
}

/// V2 cut: `RenderRequirementV2::SarifFindings` is the CHECK/EXPORT SARIF arm
/// that the V1 five-keyword rail did not carry.
#[test]
fn render_requirement_keywords_are_stable() {
    assert_eq!(RenderRequirementV2::RowsOnly.keyword(), "rows_only");
    assert_eq!(
        RenderRequirementV2::SourceLocation.keyword(),
        "source_location"
    );
    assert_eq!(
        RenderRequirementV2::SarifFindings.keyword(),
        "sarif_findings"
    );
    assert_eq!(
        RenderRequirementV2::WitnessPayload.keyword(),
        "witness_payload"
    );
    assert_eq!(
        RenderRequirementV2::ProvenanceTree.keyword(),
        "provenance_tree"
    );
}
