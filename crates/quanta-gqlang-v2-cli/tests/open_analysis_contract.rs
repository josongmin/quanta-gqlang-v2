#![expect(
    clippy::expect_used,
    reason = "binary contract tests require successful process and UTF-8 assertions"
)]

mod support;

use std::fs;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

#[test]
fn query_reports_incomplete_without_fabricating_a_plan() {
    let output = run_cli_argv_v1(&["query", "show capabilities"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(cli_stderr_v1(&output).is_empty());
    let stdout = cli_stdout_v1(&output);
    assert!(stdout
        .lines()
        .any(|line| line == "analysis.status=incomplete"));
    assert!(stdout
        .lines()
        .any(|line| line == "analysis.unresolved_count=1"));
    assert!(!stdout.contains("plan.result_kind="));
}

#[test]
fn hard_source_error_is_deterministic_and_source_anchored() {
    let source = "from users | where (a + 1";
    let first = run_cli_argv_v1(&["query", source]);
    let second = run_cli_argv_v1(&["query", source]);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.stdout, second.stdout);
    assert_eq!(first.stderr, second.stderr);
    let stderr = cli_stderr_v1(&first);
    assert!(stderr.contains("analysis.status=error"));
    assert!(stderr.contains("diagnostic[0].code=surface.parse."));
    assert!(stderr.contains("diagnostic[0].source_id=0"));
}

#[test]
fn file_ingress_enforces_exact_utf8_source_budget() {
    const LIMIT: usize = 1_048_576;
    let path = std::env::temp_dir().join(format!(
        "qglang-cli-boundary-{}-{}.gql",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let suffix = "show capabilities";
    let mut exact = " ".repeat(LIMIT - suffix.len());
    exact.push_str(suffix);
    fs::write(&path, exact).expect("write exact fixture");
    let path_text = path.to_str().expect("temp path UTF-8");
    let exact_output = run_cli_argv_v1(&["query", "--file", path_text]);
    assert_eq!(exact_output.status.code(), Some(1));
    assert!(cli_stdout_v1(&exact_output).contains("analysis.status=incomplete"));

    fs::write(&path, "x".repeat(LIMIT + 1)).expect("write over-limit fixture");
    let over_output = run_cli_argv_v1(&["query", "--file", path_text]);
    assert_eq!(over_output.status.code(), Some(2));
    assert!(cli_stderr_v1(&over_output).contains("input exceeds max_source_bytes"));
    fs::remove_file(path).expect("remove fixture");
}
