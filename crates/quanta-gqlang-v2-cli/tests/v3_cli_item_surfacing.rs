#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "hellgate assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! v3 CLI per-family surfacing hellgate (D22 / D32 / D10).
//!
//! `v3-query` consumes the ONE shared `pipeline::compile_program_structural` (D22) and
//! renders EVERY item kind deterministically through the real CLI binary:
//!
//! - a plan-bearing family (FIND / CHECK) renders its `item.kind`, plan summary,
//!   and the fixed 5-field semantic-cost schema (D32);
//! - a standalone `let` renders as a non-servable value artifact;
//! - a horn `rule` renders as a non-servable logic artifact;
//! - a catalog declaration (VIEW) renders its minted asset id and carries NO
//!   plan;
//! - a composition renders its body plan plus its bound producers' kinds;
//! - a meta directive (USE PACK) renders its effect.
//!
//! Every positive case asserts byte-identical output across two runs (D22), and
//! a failing case fails closed with the exact stage-tagged inner code (D10).

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

/// Run `v3-query <inline>` through the real binary; assert exit 0; return stdout.
fn v3_query_ok_v1(inline: &str) -> String {
    let output = run_cli_argv_v1(&["v3-query", inline]);
    assert!(
        output.status.success(),
        "v3-query `{inline}` failed (status={:?})\nstderr={}",
        output.status.code(),
        cli_stderr_v1(&output)
    );
    cli_stdout_v1(&output)
}

/// Assert `stdout` carries each expected line verbatim (line-exact match).
fn assert_lines_v1(label: &str, stdout: &str, expected: &[&str]) {
    for line in expected {
        assert!(
            stdout.lines().any(|l| l == *line),
            "{label}: missing line `{line}`:\n{stdout}"
        );
    }
}

/// A bounded FIND surfaces as a `graph_query` with a `path_witness` plan and the
/// full fixed cost schema, deterministically.
#[test]
fn find_surfaces_graph_query_plan_with_cost_schema_v1() {
    let src = "find reachability api.entry -> db.write | depth=3";
    let first = v3_query_ok_v1(src);
    let second = v3_query_ok_v1(src);
    assert_eq!(first, second, "find surfacing is not deterministic");
    assert_lines_v1(
        "find",
        &first,
        &[
            "item.kind=graph_query",
            "plan.result_kind=path_witness",
            "cost.estimated_cardinality_class=closure",
            "cost.unsupported_reason_code=none",
        ],
    );
}

/// A CHECK surfaces as a `legality` item with an `alert` plan.
#[test]
fn check_surfaces_legality_alert_plan_v1() {
    let src = "check rule.hybrid | scope=repo | confirm=require | emit=sarif | mode=hybrid";
    let first = v3_query_ok_v1(src);
    let second = v3_query_ok_v1(src);
    assert_eq!(first, second, "check surfacing is not deterministic");
    assert_lines_v1(
        "check",
        &first,
        &[
            "item.kind=legality",
            "plan.result_kind=alert",
            "cost.unsupported_reason_code=none",
        ],
    );
}

/// A standalone `let` surfaces as an explicit non-servable value artifact.
#[test]
fn decl_surfaces_value_artifact_v1() {
    let src = "let x = 1 + 2";
    let first = v3_query_ok_v1(src);
    let second = v3_query_ok_v1(src);
    assert_eq!(first, second, "decl surfacing is not deterministic");
    assert_lines_v1(
        "decl",
        &first,
        &[
            "item.kind=value_artifact",
            "artifact.serveability=no_serve",
            "artifact.no_serve_reason=value_declaration",
            "artifact.value_type=int",
        ],
    );
}

/// A horn `rule` surfaces as an explicit non-servable logic artifact.
#[test]
fn rule_surfaces_logic_artifact_v1() {
    let src = "rule adult : person.id if person.age >= 18";
    let first = v3_query_ok_v1(src);
    let second = v3_query_ok_v1(src);
    assert_eq!(first, second, "rule surfacing is not deterministic");
    assert_lines_v1(
        "rule",
        &first,
        &[
            "item.kind=rule_artifact",
            "artifact.serveability=no_serve",
            "artifact.no_serve_reason=rule_artifact",
        ],
    );
}

/// A VIEW catalog declaration surfaces as a `catalog_registration` carrying its
/// minted `view::<name>` asset id and NO plan.
#[test]
fn view_surfaces_catalog_registration_asset_id_v1() {
    let src = "view active | select=from users u | where u.active";
    let first = v3_query_ok_v1(src);
    let second = v3_query_ok_v1(src);
    assert_eq!(first, second, "view surfacing is not deterministic");
    assert_lines_v1(
        "view",
        &first,
        &["item.kind=catalog_registration", "registered view::active"],
    );
    // A catalog registration carries no plan: no plan/cost lines leak.
    assert!(
        !first.lines().any(|l| l.starts_with("plan.result_kind=")),
        "view registration must not surface a plan:\n{first}"
    );
}

/// A composition surfaces as a `composition`, listing each bound producer's kind
/// and rendering the body's consumed plan through the shared per-item renderer.
#[test]
fn composition_surfaces_body_plan_and_bound_producers_v1() {
    let src = "let x = from users u | where u.active find reachability api.entry -> db.write | depth=3 with x";
    let first = v3_query_ok_v1(src);
    let second = v3_query_ok_v1(src);
    assert_eq!(first, second, "composition surfacing is not deterministic");
    assert_lines_v1(
        "composition",
        &first,
        &[
            "item.kind=composition",
            "composition.binding_count=1",
            "composition.binding[0].kind=relational_query",
            "composition.body:",
            // the body is a bounded FIND -> its path_witness plan is surfaced.
            "item.kind=graph_query",
            "plan.result_kind=path_witness",
        ],
    );
}

/// A REQUIRE CERTIFIED meta directive surfaces its gate effect AND its
/// proof-confirmed plan.
#[test]
fn require_certified_surfaces_meta_gate_plan_v1() {
    let src = "require certified";
    let first = v3_query_ok_v1(src);
    let second = v3_query_ok_v1(src);
    assert_eq!(
        first, second,
        "require certified surfacing is not deterministic"
    );
    assert_lines_v1(
        "require certified",
        &first,
        &[
            "item.kind=meta",
            "meta.effect=require certified gate",
            "plan.result_kind=proof_explain",
        ],
    );
}

/// A failing FIND (unbounded cyclic witness) fails closed with the exact
/// `logic`-stage inner code, exit 1, silent stdout.
#[test]
fn unbounded_find_fails_closed_with_exact_logic_code_v1() {
    let output = run_cli_argv_v1(&["v3-query", "find taint api.entry -> db.write"]);
    assert_eq!(output.status.code(), Some(1), "unbounded find must exit 1");
    assert!(
        cli_stdout_v1(&output).is_empty(),
        "unbounded find must be silent on stdout"
    );
    let stderr = cli_stderr_v1(&output);
    assert!(
        stderr.contains("pipeline.error.stage=logic"),
        "missing logic stage tag:\n{stderr}"
    );
    assert!(
        stderr.contains("pipeline.error.code=logic.unbounded_witness.missing_path_length_bound"),
        "missing exact logic code:\n{stderr}"
    );
}
