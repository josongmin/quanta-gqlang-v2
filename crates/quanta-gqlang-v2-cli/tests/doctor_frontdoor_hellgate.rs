#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Hellgate: CLI `doctor` three-way classification at the binary front door.
//!
//! `doctor_shell.rs` pins parse-error / sema-error / no-result at the library
//! level. This rail promotes the same three lanes to the real binary, locking
//! each lane's exit code, output channel, and *exact* classification text:
//!
//! * parse-error / sema-error → the WHOLE stdout (status line + structured
//!   diagnostic line(s) + trailing newline) is pinned byte-for-byte, exit 1,
//!   and stderr is empty (a classified outcome rides stdout, never stderr).
//! * no-result → the classification head (status / root kind / no_result_reason
//!   / degraded_reason[0]) is pinned exactly, exit 0, stderr empty, and the
//!   lowered `query.root[0].ir.kind=` is present (proves the query report is
//!   actually appended below the classification head).

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

/// `main.rs` emits the doctor report via `println!`, so the binary's stdout is
/// the report render plus exactly one trailing newline.
fn doctor_stdout_with_newline_v1(render: &str) -> String {
    format!("{render}\n")
}

/// Parse-error lane: `SELECT` with no projection cannot parse. The whole stdout
/// is the status line + the structured parse diagnostic + a trailing newline,
/// exit 1, with an empty stderr (channel discipline).
#[test]
fn doctor_parse_error_lane_pins_whole_stdout_v1() {
    let out = run_cli_argv_v1(&["doctor", "SELECT"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "parse-error exit code, stderr={}",
        cli_stderr_v1(&out)
    );
    assert!(
        cli_stderr_v1(&out).is_empty(),
        "classified parse-error must not write stderr, got: {}",
        cli_stderr_v1(&out)
    );
    assert_eq!(
        cli_stdout_v1(&out),
        doctor_stdout_with_newline_v1(concat!(
            "doctor.status=parse-error\n",
            "doctor.parse_diagnostic[0]=expected_projection: ",
            "expected at least one projection field after SELECT"
        )),
        "parse-error stdout must match byte-for-byte"
    );
}

/// Sema-error lane: a well-formed `FIND` with `depth=0` parses but fails the
/// positive-depth semantic check. The whole stdout is the status line + the
/// classified root kind + the structured sema diagnostic + a trailing newline,
/// exit 1, with an empty stderr.
#[test]
fn doctor_sema_error_lane_pins_whole_stdout_v1() {
    let out = run_cli_argv_v1(&[
        "doctor",
        "FIND",
        "path",
        "api.entry",
        "->",
        "db.write",
        "|",
        "depth=0",
    ]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "sema-error exit code, stderr={}",
        cli_stderr_v1(&out)
    );
    assert!(
        cli_stderr_v1(&out).is_empty(),
        "classified sema-error must not write stderr, got: {}",
        cli_stderr_v1(&out)
    );
    assert_eq!(
        cli_stdout_v1(&out),
        doctor_stdout_with_newline_v1(concat!(
            "doctor.status=sema-error\n",
            "doctor.root[0].kind=find\n",
            "doctor.sema_diagnostic[0]=invalid_find_seed: `depth` must be greater than zero"
        )),
        "sema-error stdout must match byte-for-byte"
    );
}

/// No-result lane: a valid `EXPLAIN` parses and type-checks cleanly, but the
/// read-only shell has no execution backend. Exit 0, empty stderr, the exact
/// classification head (status / root kind / no_result_reason / degraded
/// reason[0]) pinned as a stdout *prefix*, and the lowered IR kind present below
/// it (the query report is appended after the head).
#[test]
fn doctor_no_result_lane_pins_classification_head_and_lowered_ir_v1() {
    let out = run_cli_argv_v1(&[
        "doctor",
        "EXPLAIN",
        "rule::owasp.sql-injection",
        ";;",
        "evidence=trace",
        ";;",
        "truth=confirm",
    ]);
    assert_eq!(
        out.status.code(),
        Some(0),
        "no-result exit code, stderr={}",
        cli_stderr_v1(&out)
    );
    assert!(
        cli_stderr_v1(&out).is_empty(),
        "no-result classification must not write stderr, got: {}",
        cli_stderr_v1(&out)
    );
    let stdout = cli_stdout_v1(&out);
    let expected_head = concat!(
        "doctor.status=no-result\n",
        "doctor.root[0].kind=explain\n",
        "doctor.no_result_reason=read-only-shell-has-no-execution-backend\n",
        "doctor.degraded_reason[0]=truth-evidence-materialization-unavailable\n",
    );
    assert!(
        stdout.starts_with(expected_head),
        "no-result classification head must be pinned exactly, got:\n{stdout}"
    );
    assert!(
        stdout.contains("query.root[0].ir.kind=explain"),
        "no-result stdout must carry the lowered IR kind below the head:\n{stdout}"
    );
    assert!(
        stdout.ends_with('\n'),
        "no-result stdout must end with the println newline:\n{stdout}"
    );
}
