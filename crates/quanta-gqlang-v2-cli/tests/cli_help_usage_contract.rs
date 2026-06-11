#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Hellgate: CLI usage / help channel + exit-code contract.
//!
//! `main.rs` routes help to stdout (exit 0) and every usage failure to stderr
//! (exit 2), always echoing the canonical usage block. The library cannot pin
//! a process exit code or an output channel, so this rail locks both against
//! the shipped binary byte-for-byte: help is the exact usage text on stdout and
//! silent on stderr; every usage fault reproduces the *entire* stderr block
//! (class line + message line + verbatim usage block) and is silent on stdout.

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

/// The exact usage block `main.rs` renders (no trailing newline). Duplicated
/// here on purpose: this test *is* the contract that the front-door text may
/// not drift unnoticed. Pinned against the real binary output, which is:
///
/// ```text
/// cli.usage=quanta-gqlang-v2-cli <parse|sema|query|fmt|explain|doctor|headless> (--file <path> | <input...>) | zero-query
/// cli.read_only=true
/// cli.command_plane=forbidden
/// ```
const USAGE_CONTRACT_V1: &str = concat!(
    "cli.usage=quanta-gqlang-v2-cli <parse|sema|query|fmt|explain|doctor|headless> (--file <path> | <input...>) | zero-query\n",
    "cli.read_only=true\n",
    "cli.command_plane=forbidden"
);

/// Reconstruct the *whole* stderr a usage fault must produce: `main.rs` writes
/// `cli.error.class=usage\ncli.error.message={message}\n{USAGE}` via `eprintln!`,
/// which appends exactly one trailing newline after the usage block.
fn expected_usage_stderr_v1(message: &str) -> String {
    format!("cli.error.class=usage\ncli.error.message={message}\n{USAGE_CONTRACT_V1}\n")
}

/// `--help` / `help` in command position print the usage block to stdout with a
/// single trailing newline (`println!`), exit 0, and write nothing to stderr.
#[test]
fn help_prints_exact_usage_to_stdout_and_exits_clean_v1() {
    let expected_stdout = format!("{USAGE_CONTRACT_V1}\n");
    for argv in [["--help"], ["help"]] {
        let out = run_cli_argv_v1(&argv);
        assert_eq!(out.status.code(), Some(0), "argv={argv:?}");
        assert_eq!(
            cli_stdout_v1(&out),
            expected_stdout,
            "help stdout must be exactly the canonical usage block + newline, argv={argv:?}"
        );
        assert!(
            cli_stderr_v1(&out).is_empty(),
            "help must be silent on stderr, argv={argv:?}"
        );
    }
}

/// `--help` / `help` are only help in *command* position. In input position
/// (after a real command) they are consumed as the lone non-flag argument, so
/// the front door reports a missing-input usage fault — never help, never a
/// partial result.
#[test]
fn help_in_input_position_is_a_full_usage_error_on_stderr_v1() {
    let expected = expected_usage_stderr_v1("missing input or --file <path>");
    for argv in [["query", "--help"], ["sema", "help"]] {
        let out = run_cli_argv_v1(&argv);
        assert_eq!(out.status.code(), Some(2), "argv={argv:?}");
        assert!(
            cli_stdout_v1(&out).is_empty(),
            "usage errors must not print to stdout, argv={argv:?}"
        );
        assert_eq!(
            cli_stderr_v1(&out),
            expected,
            "argv={argv:?}: whole stderr block must match the usage contract"
        );
    }
}

/// `--file` with zero or many path operands is a usage fault. Both arities pin
/// the entire stderr block byte-for-byte, exit 2, and an empty stdout.
#[test]
fn file_flag_arity_errors_fail_closed_with_exact_stderr_v1() {
    let expected = expected_usage_stderr_v1("--file requires exactly one path argument");
    let argvs: &[&[&str]] = &[
        &["parse", "--file"],
        &["query", "--file", "a.qglang", "b.qglang"],
    ];
    for argv in argvs {
        let out = run_cli_argv_v1(argv);
        assert_eq!(out.status.code(), Some(2), "argv={argv:?}");
        assert!(
            cli_stdout_v1(&out).is_empty(),
            "usage errors must not print to stdout, argv={argv:?}"
        );
        assert_eq!(
            cli_stderr_v1(&out),
            expected,
            "argv={argv:?}: whole stderr block must match the usage contract"
        );
    }
}

/// A bare command with no input is a missing-input usage fault: whole stderr
/// block pinned, exit 2, empty stdout.
#[test]
fn bare_command_missing_input_fails_closed_with_exact_stderr_v1() {
    let out = run_cli_argv_v1(&["sema"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        cli_stdout_v1(&out).is_empty(),
        "missing-input must not print to stdout"
    );
    assert_eq!(
        cli_stderr_v1(&out),
        expected_usage_stderr_v1("missing input or --file <path>"),
    );
}

/// An unrecognized command is a usage fault whose message embeds the offending
/// token verbatim. Pinning the whole stderr for two *distinct* tokens kills a
/// constant-message mutation: the message body must track the actual argv.
#[test]
fn unknown_command_carries_offending_token_verbatim_v1() {
    let cases: &[(&[&str], &str)] = &[
        (&["frobnicate", "SHOW", "models"], "frobnicate"),
        (&["zzz-bogus", "SELECT", "id"], "zzz-bogus"),
    ];
    for (argv, token) in cases {
        let out = run_cli_argv_v1(argv);
        assert_eq!(out.status.code(), Some(2), "argv={argv:?}");
        assert!(
            cli_stdout_v1(&out).is_empty(),
            "unknown-command must not print to stdout, argv={argv:?}"
        );
        assert_eq!(
            cli_stderr_v1(&out),
            expected_usage_stderr_v1(&format!("unsupported command `{token}`")),
            "argv={argv:?}: unknown-command stderr must embed `{token}` verbatim"
        );
    }
}
