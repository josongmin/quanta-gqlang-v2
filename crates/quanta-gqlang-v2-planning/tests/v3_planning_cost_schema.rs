#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed shapes and panic on unexpected variants"
)]
//! v3 cost-schema stability rail (ticket 07, decision `D32`).
//!
//! `D32` requires the semantic-cost output to use a FIXED schema with stable
//! field names for CLI/LSP/tests. This rail is the drift guard: it asserts the
//! five field names VERBATIM and in declaration order, so any rename of a cost
//! field breaks the build instead of silently changing the public contract.

use quanta_gqlang_v2_planning::SemanticCostV1;

#[test]
fn fixed_cost_field_names_are_stable_verbatim() {
    assert_eq!(
        SemanticCostV1::FIELD_NAMES,
        [
            "estimated_cardinality_class",
            "recursion_risk_class",
            "witness_explosion_risk_class",
            "ordering_contract",
            "unsupported_reason_code",
        ]
    );
}

#[test]
fn there_are_exactly_five_cost_fields() {
    assert_eq!(SemanticCostV1::FIELD_NAMES.len(), 5);
}
