#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! v3 `v3-query` plan + fixed semantic-cost summary rail (ticket 08, D32 / D22).
//!
//! syntax->sema->ir path pinned: source text -> the shared
//! `pipeline::compile_program_structural` (parse -> bind -> type -> normalize -> plan) ->
//! the deterministic plan/cost renderer surfaced through the real CLI binary's
//! `v3-query` command. The asserted seam is that the FIVE fixed
//! `SemanticCostV1` fields (D32) all render with stable keyword text, and that
//! the unordered-set / explain payloads render deterministically (D26 / D24).

mod support;

use std::collections::HashSet;

use support::{cli_stdout_v1, run_cli_argv_v1};

/// Run `v3-query <inline>` through the real binary; assert exit 0; return stdout.
fn v3_query_stdout_v1(inline: &str) -> String {
    let output = run_cli_argv_v1(&["v3-query", inline]);
    assert!(
        output.status.success(),
        "v3-query `{inline}` failed (status={:?})\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    cli_stdout_v1(&output)
}

/// An advanced query (alias capture + correlated path filter + projection) must
/// print all FIVE fixed semantic-cost fields with stable keyword text. The field
/// keys are the verbatim `SemanticCostV1::FIELD_NAMES`, so a rename downstream
/// breaks this rail.
#[test]
fn v3_plan_prints_all_five_fixed_cost_fields_with_stable_text_v1() {
    let advanced = "from users u | where u.id > 1 | project u.id as id, u.name as name";
    let out = v3_query_stdout_v1(advanced);

    let expected_cost_lines = [
        "cost.estimated_cardinality_class=linear_in_input",
        "cost.recursion_risk_class=none",
        "cost.witness_explosion_risk_class=none",
        "cost.ordering_contract=unordered",
        "cost.unsupported_reason_code=none",
    ];
    for line in expected_cost_lines {
        assert!(
            out.lines().any(|l| l == line),
            "advanced plan missing fixed cost line `{line}`:\n{out}"
        );
    }

    // Exactly the five fixed cost fields appear — no ad-hoc cost knobs leak.
    let cost_fields: HashSet<&str> = out
        .lines()
        .filter_map(|l| l.strip_prefix("cost."))
        .filter_map(|l| l.split('=').next())
        .collect();
    let expected_fields: HashSet<&str> = [
        "estimated_cardinality_class",
        "recursion_risk_class",
        "witness_explosion_risk_class",
        "ordering_contract",
        "unsupported_reason_code",
    ]
    .into_iter()
    .collect();
    assert_eq!(
        cost_fields, expected_fields,
        "cost field set drifted from the fixed D32 schema:\n{out}"
    );
}

/// The plan summary header and the explain provenance render deterministically:
/// the same advanced query yields byte-identical output across runs (D22), the
/// capability set is stable-ordered, and an unordered result reports
/// `is_ordered=false` (D26: an unordered set stays unordered).
#[test]
fn v3_plan_summary_is_deterministic_and_unordered_stays_unordered_v1() {
    let advanced = "from users u | where u.id > 1 | project u.id as id, u.name as name";
    let first = v3_query_stdout_v1(advanced);
    let second = v3_query_stdout_v1(advanced);
    assert_eq!(first, second, "v3-query output is not deterministic");

    for line in [
        "plan.result_kind=table",
        "plan.truth_lane=deterministic",
        "plan.effect=read_only",
        "plan.capabilities=relation_scan,filtering",
        "plan.multiplicity.is_ordered=false",
        "plan.explain.recursive=false",
        "plan.explain.families=project,filter,scan",
    ] {
        assert!(
            first.lines().any(|l| l == line),
            "advanced plan missing stable line `{line}`:\n{first}"
        );
    }
}
