#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "front-door contract assertions intentionally use expect/unwrap/indexing and exercise fail-closed behavior"
)]

//! Front-door argv contract for the real CLI binary (mutation hardening).
//!
//! These tests pin the exact dispatch behavior of `run_cli_from_env_v1` and
//! `parse_invocation_v1` through the shipped binary so the boundary operators,
//! match guards, error class -> exit-code mapping, and the help short-circuit
//! cannot drift. Each case asserts the EXACT exit code (2 = usage, 1 = io /
//! command, 0 = ok) and the precise stderr class/message line, so flipping a
//! single operator or guard observably breaks at least one assertion.

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

/// `help` / `--help` as the SOLE argument short-circuits to the usage banner on
/// stdout with exit 0. This pins line 130 (`args.len() == 1 && first.is_help`)
/// and `is_help_v1`: replacing `==` with `!=`, or stubbing `is_help_v1` to
/// `false`, drops out of the short-circuit so `help`/`--help` would be parsed as
/// a (nonexistent) command and fail-close with a usage error / exit 2 instead.
#[test]
fn lone_help_token_prints_usage_with_exit_zero_v1() {
    for token in ["help", "--help"] {
        let output = run_cli_argv_v1(&[token]);
        assert_eq!(
            output.status.code(),
            Some(0),
            "`{token}` alone must exit 0 (help short-circuit)"
        );
        let stdout = cli_stdout_v1(&output);
        assert!(
            stdout.contains(
                "cli.usage=quanta-gqlang-v2-cli <query|fmt|v3-query|v3-fmt> (--file <path> | <input...>)"
            ),
            "`{token}` must print the usage banner on stdout:\n{stdout}"
        );
        assert!(
            cli_stderr_v1(&output).is_empty(),
            "`{token}` must be silent on stderr"
        );
    }
}

/// A single NON-help token (e.g. `query`) is NOT the help short-circuit: it is a
/// real command missing its input, so it fail-closes with a usage error and
/// exit 2 — never the help banner with exit 0. This pins the `&&` at line 130:
/// replacing it with `||` would make any single argument take the help
/// short-circuit and exit 0.
#[test]
fn lone_command_token_is_not_help_short_circuit_v1() {
    let output = run_cli_argv_v1(&["query"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "`query` alone is a usage error (missing input), not the help banner"
    );
    assert!(
        cli_stdout_v1(&output).is_empty(),
        "`query` alone must be silent on stdout"
    );
    let stderr = cli_stderr_v1(&output);
    assert!(
        stderr.contains("cli.error.class=usage"),
        "`query` alone must report a usage error class:\n{stderr}"
    );
    assert!(
        stderr.contains("cli.error.message=missing input or --file <path>"),
        "`query` alone must report the missing-input message:\n{stderr}"
    );
}

/// A usage-class error exits 2 while an io-class error exits 1. This pins
/// `CliErrorV1::exit_code_v1`: collapsing the match to a constant `1` would make
/// the usage case exit 1, and the io case already exits 1, so the contrast
/// between the two is what locks the mapping.
#[test]
fn error_class_drives_distinct_exit_codes_v1() {
    let usage = run_cli_argv_v1(&["bogus-command", "x"]);
    assert_eq!(
        usage.status.code(),
        Some(2),
        "an unsupported command is a usage error -> exit 2"
    );
    assert!(
        cli_stderr_v1(&usage).contains("cli.error.class=usage"),
        "unsupported command must be class=usage"
    );

    let io = run_cli_argv_v1(&["query", "--file", "/nonexistent/qglang-front-door-xyzzy"]);
    assert_eq!(
        io.status.code(),
        Some(1),
        "a missing --file path is an io error -> exit 1"
    );
    assert!(
        cli_stderr_v1(&io).contains("cli.error.class=io"),
        "missing --file path must be class=io"
    );
}

/// `--help` AFTER the command (as the sole input arg) is rejected as a usage
/// error: the help token is NOT a valid query/format input. This pins the
/// `[flag] if is_help_v1(flag)` guard at line 175 — stubbing that guard to
/// `false` would treat `--help` as inline input and route it into the compiler
/// (a non-usage outcome).
#[test]
fn help_token_as_command_input_is_usage_error_v1() {
    let output = run_cli_argv_v1(&["query", "--help"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "`query --help` must be a usage error -> exit 2"
    );
    let stderr = cli_stderr_v1(&output);
    assert!(
        stderr.contains("cli.error.class=usage"),
        "`query --help` must be class=usage:\n{stderr}"
    );
    assert!(
        stderr.contains("cli.error.message=missing input or --file <path>"),
        "`query --help` must report the missing-input message:\n{stderr}"
    );
}

/// Exactly `--file <path>` reads from that path: a missing path surfaces the io
/// read error with the exact path echoed. This pins the
/// `[flag, path] if flag == "--file"` guard at line 180 (both the `== -> !=`
/// flip and the guard-stub-to-`false`): if the guard never fires, the input
/// falls through to the `[flag, ..] if flag == "--file"` arm and reports the
/// "requires exactly one path" usage error (exit 2) instead of the io error.
#[test]
fn file_flag_with_one_path_reads_that_path_v1() {
    let path = "/nonexistent/qglang-front-door-read-target";
    let output = run_cli_argv_v1(&["query", "--file", path]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "`--file <missing>` must be an io error -> exit 1"
    );
    let stderr = cli_stderr_v1(&output);
    assert!(
        stderr.contains("cli.error.class=io"),
        "`--file <missing>` must be class=io:\n{stderr}"
    );
    assert!(
        stderr.contains(&format!("cli.error.message=failed to read {path}")),
        "`--file <missing>` must echo the exact path in the io message:\n{stderr}"
    );
}

/// Two inline tokens that are NOT `--file` are joined as inline input and
/// compiled — they are never treated as a `--file <path>` pair. This pins the
/// `flag == "--file"` guard at line 180 against the `== -> !=` flip and the
/// guard-stub-to-`true`: either mutation would make `query from users` read a
/// file named `users` (io error, exit 1) instead of compiling `from users`.
#[test]
fn two_non_file_inline_tokens_are_joined_and_compiled_v1() {
    let output = run_cli_argv_v1(&["query", "from", "users"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "`query from users` is inline input and must compile -> exit 0"
    );
    let stdout = cli_stdout_v1(&output);
    assert!(
        stdout.contains("item.kind=relational_query"),
        "`query from users` must render a relational plan:\n{stdout}"
    );
    assert!(
        cli_stderr_v1(&output).is_empty(),
        "`query from users` must be silent on stderr"
    );
}

/// `--file` with MORE than one trailing argument is a usage error with the
/// exact "requires exactly one path" message. This pins the
/// `[flag, ..] if flag == "--file"` guard at line 181 — stubbing it to `false`
/// would join `--file a b` as inline input and route it into the compiler
/// rather than fail-closing with the usage error.
#[test]
fn file_flag_with_multiple_paths_is_usage_error_v1() {
    let output = run_cli_argv_v1(&["query", "--file", "a", "b"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "`--file a b` must be a usage error -> exit 2"
    );
    let stderr = cli_stderr_v1(&output);
    assert!(
        stderr.contains("cli.error.class=usage"),
        "`--file a b` must be class=usage:\n{stderr}"
    );
    assert!(
        stderr.contains("cli.error.message=--file requires exactly one path argument"),
        "`--file a b` must report the exact too-many-paths message:\n{stderr}"
    );
}
