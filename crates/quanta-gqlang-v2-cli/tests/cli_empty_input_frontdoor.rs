#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Hellgate: CLI empty / whitespace-only front door.
//!
//! `main.rs` rejects a blank document (`input.trim().is_empty()`) as a usage
//! error *after* reading `--file`, so an empty or whitespace-only file must
//! fail closed with exit code 2 and a usage message — never reach a command and
//! never emit a partial result on stdout.

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1, write_target_tmp_input_v1};

#[test]
fn empty_file_is_a_fail_closed_usage_error_v1() {
    let path = write_target_tmp_input_v1("empty_input.qglang", "");
    let out = run_cli_argv_v1(&["query", "--file", path.to_str().expect("utf8 path")]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        cli_stdout_v1(&out).is_empty(),
        "no partial result on stdout"
    );
    let stderr = cli_stderr_v1(&out);
    assert!(stderr.contains("cli.error.class=usage"), "{stderr}");
    assert!(
        stderr.contains("cli.error.message=input cannot be empty"),
        "{stderr}"
    );
}

#[test]
fn whitespace_only_file_is_a_fail_closed_usage_error_v1() {
    // Independent of the command in front of it — the blank-input gate runs
    // before any command executes.
    let path = write_target_tmp_input_v1("whitespace_input.qglang", "   \n\t  \r\n  ");
    let out = run_cli_argv_v1(&["doctor", "--file", path.to_str().expect("utf8 path")]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        cli_stdout_v1(&out).is_empty(),
        "no doctor classification on a blank doc"
    );
    let stderr = cli_stderr_v1(&out);
    assert!(stderr.contains("cli.error.class=usage"), "{stderr}");
    assert!(
        stderr.contains("cli.error.message=input cannot be empty"),
        "{stderr}"
    );
}
