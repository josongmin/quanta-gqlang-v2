#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]
#![allow(
    dead_code,
    reason = "shared test helpers are used by a subset of the integration-test crates that include this module"
)]

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use quanta_gqlang_v2_cli::commands::{headless, parse, query, sema};

pub(crate) struct ExampleRoundTripExpectationV1 {
    pub example_relative: &'static str,
    pub syntax_kind: &'static str,
    pub syntax_summary: &'static str,
    pub typed_kind: &'static str,
    pub typed_summary_fragments: &'static [&'static str],
    pub ir_kind: &'static str,
    pub ir_summary: &'static str,
    pub degraded_reason: Option<&'static str>,
}

pub(crate) struct BinaryFrontDoorExpectationV1 {
    pub example_relative: &'static str,
    pub syntax_kind: &'static str,
    pub typed_kind: &'static str,
    pub ir_kind: &'static str,
    pub degraded_reason: Option<&'static str>,
}

pub(crate) const SHIPPED_EXAMPLE_DOC_PATHS_V1: &[&str] = &[
    "examples/select/basic.qglang",
    "examples/find/taint.qglang",
    "examples/check/hybrid_rule.qglang",
    "examples/explain/why_no_result.qglang",
    "examples/show/capabilities.qglang",
    "examples/describe/rule_detail.qglang",
    "examples/sample/retrieval.qglang",
    "examples/export/sarif.qglang",
    "examples/compare/semantic_delta.qglang",
    "examples/view/catalog_users.qglang",
    "examples/rule/shell_exec.qglang",
    "examples/model/embeddings_code.qglang",
    "examples/pack/security_default.qglang",
    "examples/profile/strict_review.qglang",
    "examples/gate/release_gate.qglang",
    "examples/use_pack/secure_pack.qglang",
    "examples/require_certified/strict_only.qglang",
];

pub(crate) fn example_path_v1(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

pub(crate) fn example_source_v1(relative: &str) -> String {
    fs::read_to_string(example_path_v1(relative)).expect("read example")
}

fn run_cli_file_v1(command: &str, example_relative: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-cli"))
        .arg(command)
        .arg("--file")
        .arg(example_path_v1(example_relative))
        .output()
        .expect("spawn qglang cli")
}

fn stdout_success_v1(output: Output) -> String {
    assert!(
        output.status.success(),
        "status={:?}\nstderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("stdout utf-8")
}

pub(crate) fn assert_shipped_examples_listed_v1(doc: &str, doc_label: &str) {
    for example_relative in SHIPPED_EXAMPLE_DOC_PATHS_V1 {
        assert!(
            doc.contains(example_relative),
            "{doc_label} is missing shipped example `{example_relative}`"
        );
    }
}

pub(crate) fn assert_read_only_cli_binary_round_trip_v1(
    expectation: &BinaryFrontDoorExpectationV1,
) {
    let parse_stdout = stdout_success_v1(run_cli_file_v1("parse", expectation.example_relative));
    assert!(
        parse_stdout.contains("parse.root_count=1"),
        "missing parse root count for {}:\n{parse_stdout}",
        expectation.example_relative
    );
    assert!(
        parse_stdout.contains(&format!("parse.root[0].kind={}", expectation.syntax_kind)),
        "missing parse kind for {}:\n{parse_stdout}",
        expectation.example_relative
    );

    let sema_stdout = stdout_success_v1(run_cli_file_v1("sema", expectation.example_relative));
    assert!(
        sema_stdout.contains("sema.root_count=1"),
        "missing sema root count for {}:\n{sema_stdout}",
        expectation.example_relative
    );
    assert!(
        sema_stdout.contains(&format!("sema.root[0].kind={}", expectation.typed_kind)),
        "missing sema kind for {}:\n{sema_stdout}",
        expectation.example_relative
    );

    let query_stdout = stdout_success_v1(run_cli_file_v1("query", expectation.example_relative));
    assert!(
        query_stdout.contains("query.root_count=1"),
        "missing query root count for {}:\n{query_stdout}",
        expectation.example_relative
    );
    assert!(
        query_stdout.contains(&format!("query.root[0].syntax.kind={}", expectation.syntax_kind)),
        "missing query syntax kind for {}:\n{query_stdout}",
        expectation.example_relative
    );
    assert!(
        query_stdout.contains(&format!("query.root[0].typed.kind={}", expectation.typed_kind)),
        "missing query typed kind for {}:\n{query_stdout}",
        expectation.example_relative
    );
    assert!(
        query_stdout.contains(&format!("query.root[0].ir.kind={}", expectation.ir_kind)),
        "missing query ir kind for {}:\n{query_stdout}",
        expectation.example_relative
    );

    let headless_stdout =
        stdout_success_v1(run_cli_file_v1("headless", expectation.example_relative));
    assert!(
        headless_stdout.contains("headless.mode=read-only"),
        "missing headless read-only mode for {}:\n{headless_stdout}",
        expectation.example_relative
    );
    assert!(
        headless_stdout.contains("headless.command_plane=forbidden"),
        "missing headless command-plane flag for {}:\n{headless_stdout}",
        expectation.example_relative
    );
    assert!(
        headless_stdout
            .contains("headless.no_result_reason=read-only-shell-has-no-execution-backend"),
        "missing headless no-result reason for {}:\n{headless_stdout}",
        expectation.example_relative
    );
    assert!(
        headless_stdout.contains(&format!("headless.root[0].kind={}", expectation.syntax_kind)),
        "missing headless root kind for {}:\n{headless_stdout}",
        expectation.example_relative
    );
    match expectation.degraded_reason {
        Some(reason) => assert!(
            headless_stdout.contains(reason),
            "missing degraded reason `{reason}` for {}:\n{headless_stdout}",
            expectation.example_relative
        ),
        None => assert!(
            !headless_stdout.contains("headless.degraded[0]="),
            "unexpected degraded reason for {}:\n{headless_stdout}",
            expectation.example_relative
        ),
    }
}

pub(crate) fn assert_cli_fmt_round_trip_v1(example_relative: &str) {
    let source = example_source_v1(example_relative);
    let formatted = stdout_success_v1(run_cli_file_v1("fmt", example_relative));
    assert_eq!(
        formatted.trim(),
        source.trim(),
        "fmt front door drifted for {example_relative}"
    );
}

pub(crate) fn assert_read_only_cli_library_round_trip_v1(
    expectation: &ExampleRoundTripExpectationV1,
) {
    let input = example_source_v1(expectation.example_relative);

    let parse_report = parse::run(&input).expect("parse report");
    assert_eq!(parse_report.root_count, 1);
    assert_eq!(parse_report.root_kinds, vec![expectation.syntax_kind]);
    assert_eq!(
        parse_report.root_summaries,
        vec![expectation.syntax_summary.to_string()]
    );
    assert!(parse_report.render_v1().contains(&format!(
        "parse.root[0].summary={}",
        expectation.syntax_summary
    )));

    let sema_report = sema::run(&input).expect("sema report");
    assert_eq!(sema_report.root_count, 1);
    assert_eq!(sema_report.root_kinds, vec![expectation.typed_kind]);
    let sema_render = sema_report.render_v1();
    for fragment in expectation.typed_summary_fragments {
        assert!(
            sema_report.root_summaries[0].contains(fragment),
            "missing sema summary fragment `{fragment}` in `{}`",
            sema_report.root_summaries[0]
        );
        assert!(
            sema_render.contains(fragment),
            "missing sema render fragment `{fragment}` in `{sema_render}`"
        );
    }

    let query_summary = query::run(&input).expect("query summary");
    assert_eq!(query_summary.root_count, 1);
    assert_eq!(query_summary.roots[0].syntax_kind, expectation.syntax_kind);
    assert_eq!(
        query_summary.roots[0].syntax_summary,
        expectation.syntax_summary
    );
    assert_eq!(query_summary.roots[0].typed_kind, expectation.typed_kind);
    for fragment in expectation.typed_summary_fragments {
        assert!(
            query_summary.roots[0].typed_summary.contains(fragment),
            "missing query typed summary fragment `{fragment}` in `{}`",
            query_summary.roots[0].typed_summary
        );
    }
    assert_eq!(query_summary.roots[0].ir_kind, expectation.ir_kind);
    assert_eq!(query_summary.roots[0].ir_summary, expectation.ir_summary);

    let headless_report = headless::run(&input).expect("headless report");
    assert_eq!(headless_report.mode, "read-only");
    assert_eq!(headless_report.command_plane, "forbidden");
    assert_eq!(headless_report.root_count, 1);
    assert_eq!(headless_report.root_kinds, vec![expectation.syntax_kind]);
    assert_eq!(
        headless_report.no_result_reason,
        "read-only-shell-has-no-execution-backend"
    );
    match expectation.degraded_reason {
        Some(reason) => assert_eq!(headless_report.degraded_reasons, vec![reason.to_string()]),
        None => assert!(headless_report.degraded_reasons.is_empty()),
    }
    assert_eq!(headless_report.query_summary, query_summary);
}
