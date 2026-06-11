#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "query-plan summary test asserts the CLI's non-test consumption of the planning producer"
)]
//! F03/E06C hellgate — the CLI query summary is the non-test consumer of the
//! `LoweringReadyIrV1` planning producer. Proof rail: `cli --test
//! query_plan_summary`. A query-plane root surfaces its planned capability +
//! truth lane; a Declaration root (no query plane) carries no plan line.

use quanta_gqlang_v2_cli::commands::query;

#[test]
fn query_plane_root_surfaces_plan_capability_and_truth_v1() {
    let summary = query::run("SELECT id FROM catalog.files LIMIT 3").expect("query summary");
    let root = &summary.roots[0];
    let plan = root.plan.as_ref().expect("a query-plane root must carry a plan");
    assert!(
        !plan.capability.is_empty(),
        "plan must name at least one required capability"
    );
    assert_eq!(plan.truth, "Observational", "SELECT commits to observational truth");

    let rendered = summary.render_v1();
    assert!(rendered.contains("query.root[0].plan.capability="));
    assert!(rendered.contains("query.root[0].plan.truth=Observational"));
}

#[test]
fn explain_root_plan_commits_to_explain_evidence_v1() {
    let summary = query::run("EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm")
        .expect("query summary");
    let plan = summary.roots[0]
        .plan
        .as_ref()
        .expect("EXPLAIN is a query-plane root");
    assert_eq!(plan.truth, "ExplainEvidence");
}

#[test]
fn declaration_root_has_no_plan_line_v1() {
    // A VIEW declaration is not on the query plane; the producer yields no plan,
    // and the summary must not fabricate one.
    let summary = query::run("VIEW catalog.users ;; query-ref=queries/catalog/users")
        .expect("query summary");
    assert!(
        summary.roots[0].plan.is_none(),
        "declaration roots carry no planning metadata"
    );
    assert!(!summary.render_v1().contains(".plan.capability="));
}
