#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use std::path::PathBuf;
use std::process::{Command, Output};

use quanta_gqlang_v2_cli::NO_RESULT_REASON_V1;

fn example_path_v1(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
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

#[test]
fn surface_examples_round_trip_through_read_only_cli_v1() {
    let cases = [
        (
            "examples/show/capabilities.qglang",
            "show",
            "show_describe",
            "interactive",
            Some("catalog-and-interactive-host-truth-unavailable"),
        ),
        (
            "examples/describe/rule_detail.qglang",
            "describe",
            "show_describe",
            "interactive",
            Some("catalog-and-interactive-host-truth-unavailable"),
        ),
        (
            "examples/sample/retrieval.qglang",
            "sample",
            "sample_compare_export",
            "retrieval",
            Some("retrieval-result-materialization-unavailable"),
        ),
        (
            "examples/export/sarif.qglang",
            "export",
            "sample_compare_export",
            "interactive",
            Some("catalog-and-interactive-host-truth-unavailable"),
        ),
        (
            "examples/profile/strict_review.qglang",
            "profile",
            "declaration",
            "declaration",
            None,
        ),
        (
            "examples/use_pack/secure_pack.qglang",
            "meta",
            "meta",
            "meta",
            None,
        ),
    ];

    for (example_relative, syntax_kind, typed_kind, ir_kind, degraded_reason) in cases {
        let query_stdout = stdout_success_v1(run_cli_file_v1("query", example_relative));
        assert!(query_stdout.contains("query.root_count=1"));
        assert!(query_stdout.contains(&format!("query.root[0].syntax.kind={syntax_kind}")));
        assert!(query_stdout.contains(&format!("query.root[0].typed.kind={typed_kind}")));
        assert!(query_stdout.contains(&format!("query.root[0].ir.kind={ir_kind}")));

        let headless_stdout = stdout_success_v1(run_cli_file_v1("headless", example_relative));
        assert!(headless_stdout.contains("headless.mode=read-only"));
        assert!(headless_stdout.contains(&format!(
            "headless.no_result_reason={NO_RESULT_REASON_V1}"
        )));
        assert!(headless_stdout.contains(&format!("headless.root[0].kind={syntax_kind}")));
        if let Some(reason) = degraded_reason {
            assert!(headless_stdout.contains(reason));
        } else {
            assert!(!headless_stdout.contains("headless.degraded[0]="));
        }
    }
}
