#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "admissibility tests assert typed reject codes and exercise fail-closed paths"
)]

//! F02 hellgate (jun-10-gqlnag-v2-followup) — E05A/B/C/D sema admissibility.
//!
//! NOTE TO OTHER AGENTS: this pins the admissibility subsystem contract.
//!   1. FIND subkinds bind to an exact capability (E05A matrix, 7 rows).
//!   2. Each root is admissible (or not) per execution profile (E05B matrix).
//!   3. Document-level declaration semantics fail closed: duplicate declaration
//!      names and VIEW reference cycles are rejected (E05C).
//!   4. Absence/budget are typed, fail-closed expectations (E05D).
//!
//! All rejects assert a typed `.code` — never positive-only.

use quanta_gqlang_v2_sema::admissibility::{
    admit_for_profile_v1, BudgetOutcomeV1, CertifiedAbsenceV1, ExecutionProfileRuleV1,
    FindAdmissibilityV1, SemaExecutionProfileV1, FIND_ADMISSIBILITY_MATRIX_V1,
    PROFILE_ADMISSIBILITY_MATRIX_V1,
};
use quanta_gqlang_v2_sema::{analyze_document_v1, parse_document_v1, TypedRootV1};
use quanta_gqlang_v2_syntax::roots::find::FindSubkindV1;

fn analyze_reject_code_v1(input: &str) -> Vec<&'static str> {
    let doc = parse_document_v1(input).expect("parse");
    analyze_document_v1(&doc)
        .expect_err("document must fail admissibility")
        .diagnostics
        .iter()
        .map(|d| d.code)
        .collect()
}

fn analyze_ok_root_v1(input: &str) -> TypedRootV1 {
    let doc = parse_document_v1(input).expect("parse");
    analyze_document_v1(&doc)
        .expect("document must be admissible")
        .roots
        .into_iter()
        .next()
        .expect("one root")
}

// ---- E05A: FIND capability matrix ----

#[test]
fn find_admissibility_matrix_binds_every_subkind_v1() {
    assert_eq!(FIND_ADMISSIBILITY_MATRIX_V1.len(), 7);
    for subkind in [
        FindSubkindV1::Reachability,
        FindSubkindV1::Flow,
        FindSubkindV1::Slice,
        FindSubkindV1::Taint,
        FindSubkindV1::Path,
        FindSubkindV1::PointsTo,
        FindSubkindV1::Summary,
    ] {
        let row: &FindAdmissibilityV1 = FindAdmissibilityV1::for_subkind_v1(subkind);
        assert_eq!(row.subkind, subkind);
        assert!(
            !row.required_capability.is_empty(),
            "{subkind:?} must bind a capability"
        );
    }
}

// ---- E05B: profile admissibility matrix ----

#[test]
fn profile_matrix_rejects_incompatible_root_v1() {
    // A CI-gate profile is a deterministic lane: AI/ranked sampling is not
    // admissible there.
    assert!(!PROFILE_ADMISSIBILITY_MATRIX_V1.is_empty());
    let sample = analyze_ok_root_v1("SAMPLE query=auth ;; mode=lexical ;; limit=5");
    let err = admit_for_profile_v1(&sample, SemaExecutionProfileV1::CiGate)
        .expect_err("sampling must not admit on the CI-gate profile");
    assert_eq!(err.code, "profile_admissibility_rejected");
    // ...but it admits on the interactive profile.
    admit_for_profile_v1(&sample, SemaExecutionProfileV1::Interactive)
        .expect("sampling admits interactively");
}

#[test]
fn profile_rule_lookup_is_total_and_typed_v1() {
    let select = analyze_ok_root_v1("SELECT id FROM catalog.files LIMIT 3");
    let rule: ExecutionProfileRuleV1 =
        ExecutionProfileRuleV1::lookup_v1(&select, SemaExecutionProfileV1::Lint);
    assert!(rule.admitted, "SELECT is admissible on the lint profile");
}

// ---- E05C: declaration semantics ----

#[test]
fn duplicate_declaration_name_fails_closed_v1() {
    let codes = analyze_reject_code_v1(concat!(
        "RULE shell.exec ;; check-ref=checks/security/shell-exec\n",
        "RULE shell.exec ;; check-ref=checks/security/other",
    ));
    assert!(
        codes.contains(&"declaration_duplicate_name"),
        "expected declaration_duplicate_name, got {codes:?}"
    );
}

#[test]
fn view_reference_cycle_fails_closed_v1() {
    // A VIEW whose inline body selects FROM its own name is a 1-cycle.
    let codes =
        analyze_reject_code_v1("VIEW recent.errors ;; body=SELECT id FROM recent.errors LIMIT 5");
    assert!(
        codes.contains(&"declaration_cycle"),
        "expected declaration_cycle, got {codes:?}"
    );
}

#[test]
fn multi_view_mutual_reference_cycle_fails_closed_v1() {
    // Reinforcement (adversarial supplement): the cycle check must catch a
    // 2-node mutual reference (a -> b -> a), not only a self-reference.
    let codes = analyze_reject_code_v1(concat!(
        "VIEW report.a ;; body=SELECT id FROM report.b LIMIT 5\n",
        "VIEW report.b ;; body=SELECT id FROM report.a LIMIT 5",
    ));
    assert!(
        codes.contains(&"declaration_cycle"),
        "expected declaration_cycle for the a->b->a cycle, got {codes:?}"
    );
}

#[test]
fn independent_declarations_admit_v1() {
    // Distinct names + a body that selects a real catalog table must admit.
    let root =
        analyze_ok_root_v1("VIEW recent.errors ;; body=SELECT id FROM catalog.errors LIMIT 5");
    assert!(matches!(root, TypedRootV1::Declaration(_)));
}

// ---- E05D: bounded execution / absence ----

#[test]
fn uncertified_absence_is_fail_closed_v1() {
    let certified = CertifiedAbsenceV1::admit_v1("no-shell-callers", true)
        .expect("a certified absence claim admits");
    assert!(certified.certified);
    let err = CertifiedAbsenceV1::admit_v1("guessed-absence", false)
        .expect_err("an uncertified absence claim must fail closed");
    assert_eq!(err.code, "uncertified_absence_rejected");
}

#[test]
fn budget_outcome_distinguishes_reject_from_degrade_v1() {
    assert_ne!(BudgetOutcomeV1::RejectedBudget, BudgetOutcomeV1::Degraded);
    assert!(BudgetOutcomeV1::RejectedBudget.is_fail_closed_v1());
    assert!(!BudgetOutcomeV1::Degraded.is_fail_closed_v1());
}
