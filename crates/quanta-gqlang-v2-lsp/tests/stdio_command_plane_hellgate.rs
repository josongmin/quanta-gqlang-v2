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
        assert!(
            line.contains("\"ok\":false"),
            "{}: response must report ok:false, got `{line}`",
            case.label
        );
        assert!(
            line.contains("\"code\":\"command_plane_forbidden\""),
            "{}: response must carry command_plane_forbidden, got `{line}`",
            case.label
        );
        assert!(
            line.contains(&format!("mutating method `{}` is forbidden", case.method)),
            "{}: response must echo the method verbatim, got `{line}`",
            case.label
        );
        assert!(
            !line.contains("unsupported_method"),
            "{}: barrier must not degrade into unsupported_method, got `{line}`",
            case.label
        );
    }
}

/// Contrast: an unknown but NON-mutating method must fall through to
/// `unsupported_method`, proving the barrier is specific to mutating methods and
/// not a blanket reject of everything it does not recognize.
#[test]
fn unknown_non_mutating_method_is_unsupported_not_forbidden_v1() {
    let label = "workspace/index probe";
    let request = "{\"method\":\"workspace/index\",\"query\":\"SELECT id FROM catalog.files\"}";
    let (code, line) = run_binary_line_v1(request);

    assert_eq!(
        code,
        Some(1),
        "{label}: unknown method must exit with code 1"
    );
    assert!(
        line.contains("\"ok\":false"),
        "{label}: response must report ok:false, got `{line}`"
    );
    assert!(
        line.contains("\"code\":\"unsupported_method\""),
        "{label}: non-mutating unknown method must be unsupported_method, got `{line}`"
    );
    assert!(
        !line.contains("command_plane_forbidden"),
        "{label}: barrier must stay specific and not fire here, got `{line}`"
    );
    assert!(
        line.contains("unknown read-only method `workspace/index`"),
        "{label}: response must echo the unknown method, got `{line}`"
    );
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

    let forbidden = lines[0];
    assert!(
        forbidden.contains("\"ok\":false"),
        "{label}: first frame must report ok:false, got `{forbidden}`"
    );
    assert!(
        forbidden.contains("\"code\":\"command_plane_forbidden\""),
        "{label}: first frame must be command_plane_forbidden, got `{forbidden}`"
    );
    assert!(
        forbidden.contains("mutating method `workspace/executeCommand` is forbidden"),
        "{label}: first frame must echo the forbidden method, got `{forbidden}`"
    );

    let recovered = lines[1];
    assert!(
        recovered.contains("\"ok\":true"),
        "{label}: second frame must report ok:true after the barrier, got `{recovered}`"
    );
    assert!(
        recovered.contains("\"method\":\"zero_query\""),
        "{label}: second frame must be the recovered zero_query, got `{recovered}`"
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "{label}: session must still exit 1 because of the forbidden frame"
    );
}
