#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_cli::commands::{format, headless, parse, query, sema};

#[test]
fn parse_report_is_deterministic_and_structured() {
    let input = "SELECT id FROM catalog.files LIMIT 2";
    let first = parse::run(input).expect("parse report");
    let second = parse::run(input).expect("parse report rerun");

    assert_eq!(first, second);
    assert_eq!(first.root_count, 1);
    assert_eq!(first.root_kinds, vec!["select"]);
    assert!(first
        .render_v1()
        .contains("parse.root[0].summary=query=SELECT id FROM catalog.files LIMIT 2"));
}

#[test]
fn sema_report_is_deterministic_and_structured() {
    let input = "FIND reachability api.entry -> db.write | via=auth | depth=3";
    let first = sema::run(input).expect("sema report");
    let second = sema::run(input).expect("sema report rerun");

    assert_eq!(first, second);
    assert_eq!(first.root_count, 1);
    assert_eq!(first.root_kinds, vec!["find"]);
    assert!(first.render_v1().contains("sema.root[0].kind=find"));
}

#[test]
fn format_command_stays_deterministic() {
    let input = "SELECT id FROM catalog.files";
    let first = format::run(input).expect("format");
    let second = format::run(input).expect("format rerun");

    assert_eq!(first, second);

    let doc = quanta_gqlang_v2_ir::frontend::parse_document_v1(input).expect("parse");
    assert_eq!(
        first,
        quanta_gqlang_v2_ir::frontend::format_document_v1(&doc)
    );
}

#[test]
fn query_summary_reports_root_and_ir_shape() {
    let summary = query::run("CHECK owasp.sql scope=repo://runtime ;; pattern=call = \"std::process::Command\" ;; confirm=require")
        .expect("query summary");

    assert_eq!(summary.root_count, 1);
    assert_eq!(summary.roots[0].syntax_kind, "check");
    assert_eq!(summary.roots[0].typed_kind, "check");
    assert_eq!(summary.roots[0].ir_kind, "check");
    assert!(summary.render_v1().contains("query.root[0].ir.kind=check"));
}

use quanta_gqlang_v2_cli::NO_RESULT_REASON_V1;

#[test]
fn headless_report_stays_read_only_and_never_edits() {
    let report = headless::run("SHOW capabilities").expect("headless");

    assert_eq!(report.mode, "read-only");
    assert_eq!(report.command_plane, "forbidden");
    assert_eq!(report.no_result_reason, NO_RESULT_REASON_V1);
    assert_eq!(report.root_kinds, vec!["show"]);
    assert!(report
        .render_v1()
        .contains("headless.command_plane=forbidden"));
}
