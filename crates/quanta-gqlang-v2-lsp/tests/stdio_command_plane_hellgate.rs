#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]
//! LSP command-plane barrier hellgate.
//!
//! Locks the read-only command-plane barrier at the real spawned stdio binary.
//! Every mutating / command-plane method must ALWAYS fail closed with
//! `command_plane_forbidden` and exit code 1, the barrier must stay specific to
//! mutating methods (not a catch-all), and a forbidden frame must not poison the
//! session for a subsequent valid request.

mod support;

use support::{response_lines_v1, run_binary_bytes_v1, run_binary_line_v1};

/// One row of the mutating-method matrix. The barrier fires on the method name
/// alone, so each case is a minimal `{"method":...}` request.
struct MutatingCaseV1 {
    /// Stable human label threaded through every assertion for this case.
    label: &'static str,
    /// The exact JSON method string sent to the front door.
    method: &'static str,
}

const MUTATING_CASES_V1: &[MutatingCaseV1] = &[
    MutatingCaseV1 {
        label: "bare rename",
        method: "rename",
    },
    MutatingCaseV1 {
        label: "textDocument rename",
        method: "textDocument/rename",
    },
    MutatingCaseV1 {
        label: "workspace applyEdit",
        method: "workspace/applyEdit",
    },
    MutatingCaseV1 {
        label: "bare applyEdit",
        method: "applyEdit",
    },
    MutatingCaseV1 {
        label: "workspace executeCommand",
        method: "workspace/executeCommand",
    },
    MutatingCaseV1 {
        label: "bare executeCommand",
        method: "executeCommand",
    },
    MutatingCaseV1 {
        label: "bare codeAction",
        method: "codeAction",
    },
    MutatingCaseV1 {
        label: "textDocument codeAction",
        method: "textDocument/codeAction",
    },
    MutatingCaseV1 {
        label: "workspace workspaceEdit",
        method: "workspace/workspaceEdit",
    },
    MutatingCaseV1 {
        label: "bare apply",
        method: "apply",
    },
    // Boundary: the barrier normalizes by stripping `/-_` and lowercasing
    // before matching, so a dash-separated form must still fire (kills a
    // mutation that drops the separator-normalization step).
    MutatingCaseV1 {
        label: "dash-separated apply-edit",
        method: "apply-edit",
    },
    // Boundary: case-insensitive recognition (kills a mutation that forgets to
    // lowercase before matching).
    MutatingCaseV1 {
        label: "uppercase RENAME",
        method: "RENAME",
    },
];

#[test]
fn mutating_methods_fail_closed_v1() {
    for case in MUTATING_CASES_V1 {
        let request = format!("{{\"method\":\"{}\"}}", case.method);
        let (code, line) = run_binary_line_v1(&request);

        assert_eq!(
            code,
            Some(1),
            "{}: forbidden method must exit with code 1",
            case.label
        );
        // EXACT full-line equality: ok:false + command_plane_forbidden + the
        // verbatim method echo, in one mutation-proof assertion. The line is
        // short and the serializer is deterministic, so this is not brittle.
        let expected = format!(
            r#"{{"ok":false,"error":{{"code":"command_plane_forbidden","message":"mutating method `{}` is forbidden on the read-only front door"}}}}"#,
            case.method
        );
        assert_eq!(line, expected, "{}: full line drifted", case.label);
        // Non-degradation: the barrier must classify as command_plane_forbidden,
        // never collapse into any other error class.
        assert!(
            !line.contains("unsupported_method")
                && !line.contains("handler_rejected")
                && !line.contains("malformed_json")
                && !line.contains("unexpected_field"),
            "{}: barrier must not degrade into another error class, got `{line}`",
            case.label
        );
    }
}

/// Contrast: an unknown but NON-mutating method must fall through to
/// `unsupported_method`, proving the barrier is specific to mutating methods and
/// not a blanket reject of everything it does not recognize.
#[test]
fn unknown_non_mutating_method_is_unsupported_not_forbidden_v1() {
    // Each probe is an unknown but NON-mutating method that must fall through to
    // `unsupported_method`, pinned by EXACT full line. The second/third rows are
    // boundary cases: the barrier's `apply` branch matches the *exact* normalized
    // token, so `applyfoo` (normalizes to `applyfoo`) and `workspace/apply`
    // (normalizes to `workspaceapply`) must NOT be forbidden — this kills a
    // mutation that loosens `normalized == "apply"` into a substring match.
    struct ContrastCaseV1 {
        label: &'static str,
        method: &'static str,
    }
    let cases = [
        ContrastCaseV1 {
            label: "workspace/index probe",
            method: "workspace/index",
        },
        ContrastCaseV1 {
            label: "applyfoo is not the exact apply token",
            method: "applyfoo",
        },
        ContrastCaseV1 {
            label: "workspace/apply is not the exact apply token",
            method: "workspace/apply",
        },
    ];

    for case in &cases {
        let request = format!(
            "{{\"method\":\"{}\",\"query\":\"SELECT id FROM catalog.files\"}}",
            case.method
        );
        let (code, line) = run_binary_line_v1(&request);
        let label = case.label;

        assert_eq!(
            code,
            Some(1),
            "{label}: unknown method must exit with code 1"
        );
        let expected = format!(
            r#"{{"ok":false,"error":{{"code":"unsupported_method","message":"unknown read-only method `{}`"}}}}"#,
            case.method
        );
        assert_eq!(line, expected, "{label}: full line drifted");
        assert!(
            !line.contains("command_plane_forbidden"),
            "{label}: barrier must stay specific and not fire here, got `{line}`"
        );
    }
}

/// Recovery: in ONE byte stream, a forbidden frame must not poison the session.
/// The forbidden frame fails closed, then a valid `zero_query` frame still
/// succeeds, and the process as a whole still exits non-zero because of the
/// earlier error frame.
#[test]
fn forbidden_frame_does_not_poison_session_v1() {
    let label = "forbidden then zero_query recovery";
    let stream = "{\"method\":\"workspace/executeCommand\"}\n{\"method\":\"zero_query\"}\n";
    let output = run_binary_bytes_v1(stream.as_bytes());
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout utf-8");
    let lines = response_lines_v1(&stdout);

    assert_eq!(
        lines.len(),
        2,
        "{label}: expected exactly two response frames, got `{stdout}`"
    );

    // First frame: the EXACT forbidden envelope for the executeCommand method.
    let forbidden = lines[0];
    assert_eq!(
        forbidden,
        r#"{"ok":false,"error":{"code":"command_plane_forbidden","message":"mutating method `workspace/executeCommand` is forbidden on the read-only front door"}}"#,
        "{label}: first frame must be the exact forbidden envelope"
    );

    // Second frame: the recovered zero_query, pinned to its deterministic
    // read-only envelope so recovery can never pass on a stub/empty line.
    let recovered = lines[1];
    assert!(
        recovered.starts_with(
            r#"{"ok":true,"method":"zero_query","result":{"mode":"read-only","provenance":"typed-catalog-example-metadata","groups":[{"id":"catalog","#
        ),
        "{label}: second frame must recover the read-only zero_query envelope, got `{recovered}`"
    );
    assert!(
        recovered.ends_with("]}}"),
        "{label}: recovered zero_query must close groups array + result + envelope, got `{recovered}`"
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "{label}: session must still exit 1 because of the forbidden frame"
    );
}
