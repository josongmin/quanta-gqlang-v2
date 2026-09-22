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

//! v3 cost-schema stability rail (ticket 07, decision `D32`).
//!
//! `D32` requires the semantic-cost output to use a FIXED schema with stable
//! field names for CLI/LSP/tests. This rail is the drift guard: it asserts the
//! FOUR field names VERBATIM and in declaration order, and that the canonical
//! `fields()` iterator agrees with them, so any rename of a cost field breaks
//! the build instead of silently changing the public contract.
//!
//! V2 cut: the V1 schema carried a fifth `unsupported_reason_code` field that was
//! always `None` on a successful plan (a dead success field). The V2 cut replaced
//! it with the typed non-plan outcome [`quanta_gqlang_v2_planning::NoPlanReasonV2`],
//! so the equivalent V2 assertion is not "the fifth field is `None`" but "a
//! non-plan input carries no cost schema at all, and no field of the fixed
//! schema is an `unsupported_reason_code`".

use quanta_gqlang_v2_logic::RuleProgramContractV2;
use quanta_gqlang_v2_planning::{
    plan, plan_outcome, CardinalityClassV2, NoPlanReasonV2, OrderingContractV2, PlanOutcomeV2,
    PlanningDiagnosticV1, RecursionRiskClassV2, SemanticCostV2, SemanticPlanInputV2,
    WitnessExplosionRiskClassV2,
};
use quanta_gqlang_v2_sema::test_support::ReferenceRelationFixtureV2;

mod support;

use support::{document_item_location, relation_input};

/// The four fixed field names, repeated here as the contract text so a field
/// rename in `cost.rs` cannot pass by editing one list.
const EXPECTED_FIELD_NAMES: [&str; 4] = [
    "estimated_cardinality_class",
    "recursion_risk_class",
    "witness_explosion_risk_class",
    "ordering_contract",
];

#[test]
fn fixed_cost_field_names_are_stable_verbatim() {
    assert_eq!(SemanticCostV2::FIELD_NAMES, EXPECTED_FIELD_NAMES);
}

#[test]
fn there_are_exactly_four_cost_fields() {
    assert_eq!(SemanticCostV2::FIELD_NAMES.len(), 4);
    // The dead V1 success field must have no spelling left, and no physical
    // estimate dimension may appear in the host-agnostic schema.
    for deleted in [
        "unsupported_reason_code",
        "cpu",
        "memory",
        "io",
        "network",
        "coordination",
        "estimated_rows",
    ] {
        assert!(
            !SemanticCostV2::FIELD_NAMES.contains(&deleted),
            "the fixed cost schema must not carry `{deleted}`"
        );
    }
}

/// The canonical `fields()` iterator is the CLI renderer's input, so it must
/// agree with `FIELD_NAMES` elementwise and render the plan's exact class
/// keywords — not merely four non-empty strings.
#[test]
fn cost_fields_render_in_declaration_order_with_exact_keywords() {
    let (document, input) =
        relation_input(ReferenceRelationFixtureV2::Scan).expect("the reference Scan fixture");
    let planned = plan(&input, document.document()).expect("a Scan plans");
    let cost = planned.semantic_cost();

    assert_eq!(
        cost.estimated_cardinality_class,
        CardinalityClassV2::LinearInInput
    );
    assert_eq!(cost.recursion_risk_class, RecursionRiskClassV2::None);
    assert_eq!(
        cost.witness_explosion_risk_class,
        WitnessExplosionRiskClassV2::None
    );
    assert_eq!(cost.ordering_contract, OrderingContractV2::Unordered);

    let rendered = cost.fields();
    assert_eq!(
        rendered,
        [
            ("estimated_cardinality_class", "linear_in_input"),
            ("recursion_risk_class", "none"),
            ("witness_explosion_risk_class", "none"),
            ("ordering_contract", "unordered"),
        ]
    );
    for (index, (name, keyword)) in rendered.iter().enumerate() {
        assert_eq!(Some(*name), SemanticCostV2::FIELD_NAMES.get(index).copied());
        assert!(!keyword.is_empty());
    }
}

/// An ordered relational plan renders the `ordered` keyword in the last slot and
/// leaves the other three classes untouched: the keyword set is read from the
/// sealed cardinality, not from a per-plan string.
#[test]
fn an_ordered_plan_renders_the_ordered_keyword_in_the_last_field() {
    let (document, input) =
        relation_input(ReferenceRelationFixtureV2::SortAsc).expect("the reference SortAsc fixture");
    let planned = plan(&input, document.document()).expect("a checked sort plans");
    assert_eq!(
        planned.semantic_cost().fields(),
        [
            ("estimated_cardinality_class", "linear_in_input"),
            ("recursion_risk_class", "none"),
            ("witness_explosion_risk_class", "none"),
            ("ordering_contract", "ordered"),
        ]
    );
}

/// V2 replacement for "`unsupported_reason_code` is `None` on success": a
/// checked non-plan input carries NO cost schema at all, and both entry points
/// spell the absence with the typed reason rather than a dead success field.
#[test]
fn a_non_plan_input_carries_no_cost_schema_at_all() {
    let (document, _input) =
        relation_input(ReferenceRelationFixtureV2::Scan).expect("the reference Scan fixture");
    let contract = RuleProgramContractV2::from_document(document.document())
        .expect("a document without a Rule item still seals an empty program");
    let input = SemanticPlanInputV2::Rule(contract);
    let anchor = document_item_location(document.document(), document.item_id());

    match plan_outcome(&input, document.document()).expect("an empty program is not an error") {
        PlanOutcomeV2::NoPlan(reason) => assert_eq!(reason, NoPlanReasonV2::EmptyRuleProgram),
        PlanOutcomeV2::Plan(planned) => {
            panic!("an empty rule program must not plan: {planned:?}")
        }
    }

    match plan(&input, document.document()) {
        Err(PlanningDiagnosticV1::NoPlan { reason, span }) => {
            assert_eq!(reason, NoPlanReasonV2::EmptyRuleProgram);
            assert_eq!(Some(&span), anchor.as_ref());
            assert_eq!(reason.code(), "empty_rule_program");
            assert_eq!(reason.catalog_kind(), None);
        }
        other => panic!("expected the typed non-plan refusal, got {other:?}"),
    }
}
