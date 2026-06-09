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
//! each lane's exit code, output channel, status line, and diagnostic field —
//! in particular the previously binary-untested `doctor.status=sema-error`.

mod support;

use support::{cli_stderr_v1, cli_stdout_v1, run_cli_argv_v1};

struct DoctorLaneV1 {
    label: &'static str,
    argv: &'static [&'static str],
    expect_exit: i32,
    expect_status_line: &'static str,
    expect_fragment: &'static str,
}

#[test]
fn doctor_binary_pins_all_three_classification_lanes_v1() {
    let lanes = [
        DoctorLaneV1 {
            label: "parse-error",
            argv: &["doctor", "SELECT"],
            expect_exit: 1,
            expect_status_line: "doctor.status=parse-error",
            expect_fragment: "doctor.parse_diagnostic[0]=",
        },
        DoctorLaneV1 {
            label: "sema-error",
            argv: &[
                "doctor",
                "FIND",
                "path",
                "api.entry",
                "->",
                "db.write",
                "|",
                "depth=0",
            ],
            expect_exit: 1,
            expect_status_line: "doctor.status=sema-error",
            expect_fragment: "greater than zero",
        },
        DoctorLaneV1 {
            label: "no-result",
            argv: &[
                "doctor",
                "EXPLAIN",
                "rule::owasp.sql-injection",
                ";;",
                "evidence=trace",
                ";;",
                "truth=confirm",
            ],
            expect_exit: 0,
            expect_status_line: "doctor.status=no-result",
            expect_fragment: "doctor.no_result_reason=read-only-shell-has-no-execution-backend",
        },
    ];

    for lane in &lanes {
        let out = run_cli_argv_v1(lane.argv);
        assert_eq!(
            out.status.code(),
            Some(lane.expect_exit),
            "{}: exit code, stderr={}",
            lane.label,
            cli_stderr_v1(&out)
        );
        let stdout = cli_stdout_v1(&out);
        assert!(
            stdout.contains(lane.expect_status_line),
            "{}: missing `{}` in:\n{stdout}",
            lane.label,
            lane.expect_status_line
        );
        assert!(
            stdout.contains(lane.expect_fragment),
            "{}: missing `{}` in:\n{stdout}",
            lane.label,
            lane.expect_fragment
        );
    }
}

#[test]
fn doctor_classification_is_a_stdout_channel_not_a_crash_v1() {
    // A sema-error is a *classified* outcome: it rides stdout with exit 1, and
    // must not leak onto stderr (which is reserved for usage/IO faults).
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
    assert_eq!(out.status.code(), Some(1));
    assert!(
        cli_stderr_v1(&out).is_empty(),
        "classified sema-error must not write stderr, got: {}",
        cli_stderr_v1(&out)
    );
    let stdout = cli_stdout_v1(&out);
    assert!(stdout.contains("doctor.status=sema-error"), "{stdout}");
    assert!(stdout.contains("doctor.sema_diagnostic[0]="), "{stdout}");
    assert!(stdout.contains("greater than zero"), "{stdout}");
}
