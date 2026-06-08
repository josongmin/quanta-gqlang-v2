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

pub(crate) fn assert_shipped_examples_listed_v1(doc: &str, doc_label: &str) {
    for example_relative in SHIPPED_EXAMPLE_DOC_PATHS_V1 {
        assert!(
            doc.contains(example_relative),
            "{doc_label} is missing shipped example `{example_relative}`"
        );
    }
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
