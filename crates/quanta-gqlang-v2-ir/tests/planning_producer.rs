#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    reason = "planning-producer tests assert plan production over the real lowering chain"
)]

//! F03 hellgate (jun-10-gqlnag-v2-followup) — production planning-IR producer.
//!
//! NOTE TO OTHER AGENTS: this pins that the planning IR has a PRODUCTION
//! producer (previously test-only). Every query-plane root kind yields a valid
//! `PublicPlanV1`; Declaration/Meta yield none; and `lowering_ready_for_root_v1`
//! supplies the plan so the planning-required roots pass lowering-ready
//! validation (the previously-unreachable production path).

use quanta_gqlang_v2_ir::planning::{CapabilityNameV1, PublicPlanRootKindV1, TruthExpectationV1};
use quanta_gqlang_v2_ir::planning_producer::{
    lowering_ready_for_root_v1, plan_for_ai_retrieval_v1, plan_for_root_v1,
};
use quanta_gqlang_v2_ir::{lower_document_to_ir_v1, PublicIrRootV1};
use quanta_gqlang_v2_sema::{analyze_document_v1, parse_document_v1};

fn lower_one_v1(input: &str) -> PublicIrRootV1 {
    let doc = parse_document_v1(input).expect("parse");
    let typed = analyze_document_v1(&doc).expect("sema");
    let mut roots = lower_document_to_ir_v1(&typed).expect("ir");
    roots.pop().expect("one root")
}

#[test]
fn plan_is_produced_and_valid_for_query_roots_v1() {
    let cases = [
        ("SELECT id FROM catalog.files LIMIT 3", CapabilityNameV1::Search),
        (
            "FIND taint api.entry -> db.write",
            CapabilityNameV1::Path,
        ),
        (
            "CHECK rule.x check:v1|class=security|scope=repo%3A%2F%2Fr|pattern=~|confirm=skip",
            CapabilityNameV1::PatternLint,
        ),
        ("EXPLAIN rule::owasp.x ;; evidence=trace", CapabilityNameV1::Explain),
        ("SHOW capabilities", CapabilityNameV1::CatalogIntrospection),
    ];
    for (input, expected_capability) in cases {
        let root = lower_one_v1(input);
        let plan = plan_for_root_v1(&root).unwrap_or_else(|| panic!("plan for `{input}`"));
        plan.validate_v1()
            .unwrap_or_else(|err| panic!("plan for `{input}` must validate: {err:?}"));
        assert_eq!(
            plan.required_capabilities[0].capability, expected_capability,
            "capability mismatch for `{input}`"
        );
    }
}

#[test]
fn explain_plan_carries_explain_evidence_truth_v1() {
    let plan = plan_for_root_v1(&lower_one_v1("EXPLAIN rule::owasp.x ;; evidence=trace"))
        .expect("explain plan");
    assert_eq!(plan.truth_expectation, TruthExpectationV1::ExplainEvidence);
    assert_eq!(plan.root, PublicPlanRootKindV1::Explain);
}

#[test]
fn declaration_and_meta_have_no_plan_v1() {
    let view = lower_one_v1("VIEW catalog.users ;; query-ref=queries/catalog/users");
    assert!(plan_for_root_v1(&view).is_none(), "declaration carries no plan");
    let meta = lower_one_v1("REQUIRE CERTIFIED");
    assert!(plan_for_root_v1(&meta).is_none(), "meta carries no plan");
}

#[test]
fn lowering_ready_supplies_plan_for_planning_required_roots_v1() {
    // Find/Check/Explain/Retrieval/Interactive REQUIRE planning metadata; the
    // producer must supply it so lowering-ready validation passes (the formerly
    // unreachable production path).
    for input in [
        "FIND taint api.entry -> db.write",
        "EXPLAIN rule::owasp.x ;; evidence=trace",
        "SHOW capabilities",
    ] {
        let ready = lowering_ready_for_root_v1(lower_one_v1(input))
            .unwrap_or_else(|err| panic!("lowering-ready for `{input}` must succeed: {err:?}"));
        assert!(ready.planning.is_some(), "`{input}` must carry planning metadata");
    }
}

#[test]
fn ai_lane_plan_reaches_the_some_ai_plan_validation_branch_v1() {
    // E05E supplement: the AI-lane producer constructs a Some(ai_plan) plan from
    // a real RETRIEVAL root, so the AI validation branch (final-truth forbidden,
    // working-set bridge, reconfirmation discipline) is reachable from production.
    let retrieval = lower_one_v1("SAMPLE query=auth middleware ;; mode=semantic ;; limit=20");
    let plan = plan_for_ai_retrieval_v1(&retrieval).expect("retrieval yields an AI plan");
    assert!(plan.ai_plan.is_some());
    assert_eq!(plan.truth_expectation, TruthExpectationV1::ReconfirmRequired);
    plan.validate_v1()
        .expect("AI-lane plan must satisfy the AI validation branch");
    // non-retrieval roots get no AI plan
    assert!(plan_for_ai_retrieval_v1(&lower_one_v1("SELECT id FROM catalog.files LIMIT 3")).is_none());
}
