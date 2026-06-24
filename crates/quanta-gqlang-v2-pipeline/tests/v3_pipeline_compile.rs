//! v3 rail: the ONE shared frontend compiler service (ticket 08, D22).
//!
//! syntax->sema->ir path pinned: source text -> `compile_program_structural` runs
//! surface::parse_program -> binder resolution -> types check -> algebra
//! normalization -> planning, and returns a `CompiledProgramV1` exposing the
//! HIR, the algebra root, and the public plan with the FIXED cost schema.
//! This is the success / determinism half of the integration hellgate; the
//! fail-closed stage-tagging half lives in `v3_pipeline_hellgate.rs`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration test rail: assertion panics and indexing are intentional"
)]

use quanta_gqlang_v2_pipeline::{compile_program_structural, CompiledProgramV1};
use quanta_gqlang_v2_planning::SemanticCostV1;

/// rail 1: a non-trivial advanced v3 query compiles cleanly through every stage
/// to a plan whose fixed cost-schema fields are all populated.
#[test]
fn e2e_success_full_stack_to_plan() {
    let src = "from users u | where u.id > 1 | project u.id as id, u.name as name";
    let compiled: CompiledProgramV1 = compile_program_structural(src).expect("compiles end to end");

    // HIR surfaced.
    assert!(compiled.query().is_some(), "query HIR present");

    // Algebra root surfaced: outermost is a Project over a Filter over a Scan.
    let root = compiled
        .algebra()
        .expect("relational query has an algebra root");
    assert!(!root.columns.is_empty(), "algebra columns derived");

    // Plan surfaced with the fixed cost schema, all 5 fields populated.
    let plan = compiled.plan().expect("relational query has a plan");
    let cost: &SemanticCostV1 = &plan.cost;
    // Drift guard: exactly the five fixed field names, in order.
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
    // The class fields render a stable keyword (populated, not a placeholder).
    assert!(!cost.estimated_cardinality_class.keyword().is_empty());
    assert!(!cost.recursion_risk_class.keyword().is_empty());
    assert!(!cost.witness_explosion_risk_class.keyword().is_empty());
    assert!(!cost.ordering_contract.keyword().is_empty());
    // No unsupported reason on a clean compile.
    assert!(cost.unsupported_reason_code.is_none());
}

/// rail 5: determinism — compiling the same source twice yields an identical
/// `CompiledProgramV1` (no nondeterministic ordering anywhere in the stack).
#[test]
fn e2e_determinism_same_source_twice() {
    let src = "from findings f | where f.score > 1 | project f.id as id";
    let a = compile_program_structural(src).expect("compiles a");
    let b = compile_program_structural(src).expect("compiles b");
    assert_eq!(a, b, "identical compile outputs for identical source");
}
