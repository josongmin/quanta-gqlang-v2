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
//! error *after* reading `--file` and *before* any command executes, so an
//! empty or whitespace-only document must fail closed with exit code 2 and the
//! canonical `input cannot be empty` usage block — never reach a command and
//! never emit a partial result on stdout. This rail pins the *whole* stderr
//! block byte-for-byte and proves the gate is command-agnostic.

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1, write_target_tmp_input_v1};

/// The exact stderr `main.rs` writes for the blank-input usage fault: the class
/// line, the `input cannot be empty` message line, then the verbatim usage
/// block, with a single trailing newline from `eprintln!`.
const EXPECTED_EMPTY_STDERR_V1: &str = concat!(
    "cli.error.class=usage\n",
    "cli.error.message=input cannot be empty\n",
    "cli.usage=quanta-gqlang-v2-cli <parse|sema|query|fmt|explain|doctor|headless> (--file <path> | <input...>) | zero-query\n",
    "cli.read_only=true\n",
    "cli.command_plane=forbidden\n",
);

/// Assert that `argv` fails closed as a blank-input usage error: exit 2, empty
/// stdout, and the whole stderr block byte-for-byte.
fn assert_blank_input_fails_closed_v1(argv: &[&str]) {
    let out = run_cli_argv_v1(argv);
    assert_eq!(out.status.code(), Some(2), "argv={argv:?}");
    assert!(
        cli_stdout_v1(&out).is_empty(),
        "blank input must not print any result on stdout, argv={argv:?}"
    );
    assert_eq!(
        cli_stderr_v1(&out),
        EXPECTED_EMPTY_STDERR_V1,
        "argv={argv:?}: whole stderr block must match the blank-input usage contract"
    );
}

/// An empty `--file` document fails closed identically across every read-only
/// command — the blank gate runs before command dispatch, so the command name
/// in front of it cannot change the outcome.
#[test]
fn empty_file_is_a_command_agnostic_fail_closed_usage_error_v1() {
    let path = write_target_tmp_input_v1("empty_input.qglang", "");
    let path_str = path.to_str().expect("utf8 path");
    for command in ["query", "doctor", "headless", "fmt"] {
        assert_blank_input_fails_closed_v1(&[command, "--file", path_str]);
    }
}

/// A whitespace-only `--file` document (spaces, tabs, CR/LF) trims to empty and
/// fails closed identically across a *different* set of commands, pinning the
/// same whole stderr block. Covering a disjoint command set from the empty-file
/// case widens the command-agnosticism proof.
#[test]
fn whitespace_only_file_is_a_command_agnostic_fail_closed_usage_error_v1() {
    let path = write_target_tmp_input_v1("whitespace_input.qglang", "   \n\t  \r\n  ");
    let path_str = path.to_str().expect("utf8 path");
    for command in ["doctor", "parse", "sema", "headless"] {
        assert_blank_input_fails_closed_v1(&[command, "--file", path_str]);
    }
}

/// A single inline argument that is only spaces is joined to a blank string and
/// trips the same gate — the blank rejection is on the *content*, not the
/// transport, so inline whitespace fails closed exactly like a blank file.
#[test]
fn inline_whitespace_argument_is_a_fail_closed_usage_error_v1() {
    assert_blank_input_fails_closed_v1(&["parse", "   "]);
}
