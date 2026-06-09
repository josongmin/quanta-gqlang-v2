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
fn core_examples_round_trip_through_read_only_cli_v1() {
    let cases = [
        (
            "examples/select/basic.qglang",
            "select",
            "select",
            "select",
            None,
        ),
        (
            "examples/find/taint.qglang",
            "find",
            "find",
            "find",
            Some("runtime-host-routing-unavailable"),
        ),
        (
            "examples/check/hybrid_rule.qglang",
            "check",
            "check",
            "check",
            Some("semantic-confirm-lane-unavailable"),
        ),
        (
            "examples/explain/why_no_result.qglang",
            "explain",
            "explain",
            "explain",
            Some("truth-evidence-materialization-unavailable"),
        ),
    ];

    for (example_relative, syntax_kind, typed_kind, ir_kind, degraded_reason) in cases {
        let parse_stdout = stdout_success_v1(run_cli_file_v1("parse", example_relative));
        assert!(parse_stdout.contains("parse.root_count=1"));
        assert!(parse_stdout.contains(&format!("parse.root[0].kind={syntax_kind}")));

        let sema_stdout = stdout_success_v1(run_cli_file_v1("sema", example_relative));
        assert!(sema_stdout.contains("sema.root_count=1"));
        assert!(sema_stdout.contains(&format!("sema.root[0].kind={typed_kind}")));

        let query_stdout = stdout_success_v1(run_cli_file_v1("query", example_relative));
        assert!(query_stdout.contains("query.root_count=1"));
        assert!(query_stdout.contains(&format!("query.root[0].syntax.kind={syntax_kind}")));
        assert!(query_stdout.contains(&format!("query.root[0].typed.kind={typed_kind}")));
        assert!(query_stdout.contains(&format!("query.root[0].ir.kind={ir_kind}")));

        let headless_stdout = stdout_success_v1(run_cli_file_v1("headless", example_relative));
        assert!(headless_stdout.contains("headless.mode=read-only"));
        assert!(headless_stdout.contains("headless.command_plane=forbidden"));
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

#[test]
fn explain_front_door_surfaces_witness_and_no_result_guidance_v1() {
    let stdout = stdout_success_v1(run_cli_file_v1(
        "explain",
        "examples/explain/why_no_result.qglang",
    ));

    assert!(stdout.contains("explain.subject=rule::owasp.sql-injection"));
    assert!(stdout.contains("explain.evidence=trace"));
    assert!(stdout.contains("explain.truth=confirm"));
    assert!(stdout.contains("explain.path_witness.kind=path"));
    assert!(stdout.contains("explain.why_no_result=read-only-shell-has-no-execution-backend"));
}
