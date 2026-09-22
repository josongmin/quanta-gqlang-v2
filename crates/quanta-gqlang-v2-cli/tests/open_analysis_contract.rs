#![expect(
    clippy::expect_used,
    reason = "binary contract tests require successful process and UTF-8 assertions"
)]

mod support;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

/// Collision-resistant temporary file removed on drop, even when an
/// assertion fails mid-test.
struct TempFileV1 {
    path: PathBuf,
}

static TEMP_FILE_COUNTER_V1: AtomicU64 = AtomicU64::new(0);

impl TempFileV1 {
    fn new(tag: &str, bytes: &[u8]) -> Self {
        let id = TEMP_FILE_COUNTER_V1.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "qglang-cli-empty-{}-{}-{}.gql",
            std::process::id(),
            tag,
            id
        ));
        fs::write(&path, bytes).expect("write temp fixture");
        Self { path }
    }

    fn text(&self) -> &str {
        self.path.to_str().expect("temp path UTF-8")
    }
}

impl Drop for TempFileV1 {
    fn drop(&mut self) {
        let _ignored = fs::remove_file(&self.path);
    }
}

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

/// CLI-02: the shared trim-empty guard fails closed through BOTH ingress
/// modes. Each case asserts exit 2, empty stdout, error class `usage`, the
/// exact message, and exactly one usage block.
#[test]
fn empty_inputs_reject_at_the_shared_ingress_guard() {
    let empty_file = TempFileV1::new("zero-byte", b"");
    let blank_file = TempFileV1::new("whitespace", b"  \n\t ");
    let cases: [Vec<&str>; 3] = [
        vec!["query", "   "],
        vec!["query", "--file", empty_file.text()],
        vec!["query", "--file", blank_file.text()],
    ];
    for argv in cases {
        let output = run_cli_argv_v1(&argv);
        assert_eq!(output.status.code(), Some(2), "argv {argv:?}");
        assert!(
            cli_stdout_v1(&output).is_empty(),
            "empty input must produce no stdout: {argv:?}"
        );
        let stderr = cli_stderr_v1(&output);
        assert!(
            stderr.lines().any(|line| line == "cli.error.class=usage"),
            "error class drift: {argv:?}\n{stderr}"
        );
        assert!(
            stderr
                .lines()
                .any(|line| line == "cli.error.message=input cannot be empty"),
            "exact message drift: {argv:?}\n{stderr}"
        );
        assert_eq!(
            stderr
                .lines()
                .filter(|line| line.contains("cli.usage="))
                .count(),
            1,
            "exactly one usage block: {argv:?}\n{stderr}"
        );
    }
}

/// CLI-02 positive neighbors: non-empty inline and file inputs continue past
/// usage validation into analysis (exit 1 incomplete), proving the guard
/// rejects emptiness rather than the ingress mode.
#[test]
fn non_empty_inputs_pass_the_shared_ingress_guard() {
    let inline = run_cli_argv_v1(&["query", "show capabilities"]);
    assert_eq!(inline.status.code(), Some(1));
    assert!(cli_stderr_v1(&inline).is_empty());
    assert!(cli_stdout_v1(&inline)
        .lines()
        .any(|line| line == "analysis.status=incomplete"));

    let file = TempFileV1::new("non-empty", b"show capabilities");
    let filed = run_cli_argv_v1(&["query", "--file", file.text()]);
    assert_eq!(filed.status.code(), Some(1));
    assert!(cli_stdout_v1(&filed).contains("analysis.status=incomplete"));
}
