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
//! the shipped binary: help is silent on stderr, usage errors are silent on
//! stdout, and `--file` arity / unknown-command / missing-input failures all
//! fail closed with code 2.

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

/// The exact usage block `main.rs` renders. Duplicated here on purpose: this
/// test *is* the contract that the front-door text may not drift unnoticed.
const USAGE_CONTRACT_V1: &str = concat!(
    "cli.usage=quanta-gqlang-v2-cli <parse|sema|query|fmt|doctor|headless> (--file <path> | <input...>)\n",
    "cli.read_only=true\n",
    "cli.command_plane=forbidden"
);

#[test]
fn help_prints_usage_to_stdout_and_exits_clean_v1() {
    for argv in [["--help"], ["help"]] {
        let out = run_cli_argv_v1(&argv);
        assert_eq!(out.status.code(), Some(0), "argv={argv:?}");
        assert_eq!(
            cli_stdout_v1(&out).trim_end(),
            USAGE_CONTRACT_V1,
            "help stdout must be the canonical usage block, argv={argv:?}"
        );
        assert!(
            cli_stderr_v1(&out).is_empty(),
            "help must be silent on stderr, argv={argv:?}"
        );
    }
}

#[test]
fn help_in_input_position_is_a_usage_error_on_stderr_v1() {
    for argv in [["query", "--help"], ["sema", "help"]] {
        let out = run_cli_argv_v1(&argv);
        assert_eq!(out.status.code(), Some(2), "argv={argv:?}");
        assert!(
            cli_stdout_v1(&out).is_empty(),
            "usage errors must not print to stdout, argv={argv:?}"
        );
        let stderr = cli_stderr_v1(&out);
        assert!(
            stderr.contains("cli.error.class=usage"),
            "argv={argv:?}: {stderr}"
        );
        assert!(
            stderr.contains("cli.error.message=missing input or --file <path>"),
            "argv={argv:?}: {stderr}"
        );
        assert!(
            stderr.contains(USAGE_CONTRACT_V1),
            "usage block must be echoed on failure, argv={argv:?}: {stderr}"
        );
    }
}

#[test]
fn file_flag_arity_errors_fail_closed_v1() {
    let cases: &[(&[&str], &str)] = &[
        (
            &["parse", "--file"],
            "--file requires exactly one path argument",
        ),
        (
            &["query", "--file", "a.qglang", "b.qglang"],
            "--file requires exactly one path argument",
        ),
    ];
    for (argv, message) in cases {
        let out = run_cli_argv_v1(argv);
        assert_eq!(out.status.code(), Some(2), "argv={argv:?}");
        assert!(cli_stdout_v1(&out).is_empty(), "argv={argv:?}");
        let stderr = cli_stderr_v1(&out);
        assert!(
            stderr.contains("cli.error.class=usage"),
            "argv={argv:?}: {stderr}"
        );
        assert!(
            stderr.contains(&format!("cli.error.message={message}")),
            "argv={argv:?}: {stderr}"
        );
    }
}

#[test]
fn missing_input_and_unknown_command_fail_closed_v1() {
    let missing = run_cli_argv_v1(&["sema"]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(
        cli_stderr_v1(&missing).contains("cli.error.message=missing input or --file <path>"),
        "{}",
        cli_stderr_v1(&missing)
    );

    let unknown = run_cli_argv_v1(&["frobnicate", "SHOW", "models"]);
    assert_eq!(unknown.status.code(), Some(2));
    let stderr = cli_stderr_v1(&unknown);
    assert!(stderr.contains("cli.error.class=usage"), "{stderr}");
    assert!(
        stderr.contains("cli.error.message=unsupported command `frobnicate`"),
        "{stderr}"
    );
}
