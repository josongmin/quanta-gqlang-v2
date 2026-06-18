#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]
//! LSP command-plane barrier hellgate.
//!
//! Locks the read-only front door as a pure, closed allowlist at the real
//! spawned stdio binary. The dispatch recognizes exactly seven read-only
//! methods; EVERY other method — whether it looks mutating (rename, applyEdit,
//! executeCommand, codeAction, workspaceEdit) or is simply an unknown name
//! (delete, drop, didChange, arbitrary garbage) — fails closed with the SINGLE
//! code `method_not_allowed`, exit code 1, and must not poison the session for a
//! subsequent valid request. (Escaped-unicode method names decode to their
//! plain form BEFORE this allowlist match — that decode path is pinned by
//! `stdio_json_escape_negative.rs` — so they cannot smuggle past the allowlist.)
//!
//! NOTE: the old "barrier is mutation-specific / not a catch-all" contrast
//! property is intentionally DELETED. Classifying *why* a method was refused
//! (forbidden-mutation vs unsupported-unknown) was a name-pattern heuristic; the
//! principled front door is a closed allowlist that never guesses. There is now
//! one rejection code for all non-allowlisted methods.

mod support;

use support::{response_lines_v1, run_binary_bytes_v1, run_binary_line_v1};

/// One row of the non-allowlisted-method matrix. Recognition is by the method
/// name alone, so each case is a minimal `{"method":...}` request.
struct RejectedCaseV1 {
    /// Stable human label threaded through every assertion for this case.
    label: &'static str,
    /// The exact JSON method string sent to the front door.
    method: &'static str,
}

/// Broad adversarial corpus: mutating-looking names AND plain unknown names.
/// The point of this hellgate is that they ALL collapse to the same single
/// `method_not_allowed` code now — there is no name-pattern classification.
const REJECTED_CASES_V1: &[RejectedCaseV1] = &[
    // --- mutating-looking names (formerly command_plane_forbidden) ---
    RejectedCaseV1 {
        label: "bare rename",
        method: "rename",
    },
    RejectedCaseV1 {
        label: "textDocument rename",
        method: "textDocument/rename",
    },
    RejectedCaseV1 {
        label: "workspace applyEdit",
        method: "workspace/applyEdit",
    },
    RejectedCaseV1 {
        label: "bare applyEdit",
        method: "applyEdit",
    },
    RejectedCaseV1 {
        label: "workspace executeCommand",
        method: "workspace/executeCommand",
    },
    RejectedCaseV1 {
        label: "bare executeCommand",
        method: "executeCommand",
    },
    RejectedCaseV1 {
        label: "bare codeAction",
        method: "codeAction",
    },
    RejectedCaseV1 {
        label: "textDocument codeAction",
        method: "textDocument/codeAction",
    },
    RejectedCaseV1 {
        label: "workspace workspaceEdit",
        method: "workspace/workspaceEdit",
    },
    RejectedCaseV1 {
        label: "bare apply",
        method: "apply",
    },
    RejectedCaseV1 {
        label: "dash-separated apply-edit",
        method: "apply-edit",
    },
    RejectedCaseV1 {
        label: "uppercase RENAME",
        method: "RENAME",
    },
    // --- plain unknown names (formerly unsupported_method) ---
    RejectedCaseV1 {
        label: "workspace index probe",
        method: "workspace/index",
    },
    RejectedCaseV1 {
        label: "applyfoo near-miss",
        method: "applyfoo",
    },
    RejectedCaseV1 {
        label: "workspace apply near-miss",
        method: "workspace/apply",
    },
    RejectedCaseV1 {
        label: "bare delete",
        method: "delete",
    },
    RejectedCaseV1 {
        label: "bare drop",
        method: "drop",
    },
    RejectedCaseV1 {
        label: "textDocument didChange",
        method: "textDocument/didChange",
    },
    RejectedCaseV1 {
        label: "arbitrary unknown method",
        method: "totally-unknown-method",
    },
];

#[test]
fn non_allowlisted_methods_fail_closed_v1() {
    for case in REJECTED_CASES_V1 {
        let request = format!("{{\"method\":\"{}\"}}", case.method);
        let (code, line) = run_binary_line_v1(&request);

        assert_eq!(
            code,
            Some(1),
            "{}: non-allowlisted method must exit with code 1",
            case.label
        );
        // EXACT full-line equality: ok:false + the SINGLE method_not_allowed code
        // + the verbatim method echo, in one mutation-proof assertion. The line is
        // short and the serializer is deterministic, so this is not brittle.
        let expected = format!(
            r#"{{"ok":false,"error":{{"code":"method_not_allowed","message":"method `{}` is not an allowed read-only method"}}}}"#,
            case.method
        );
        assert_eq!(line, expected, "{}: full line drifted", case.label);
        // Single-code property: every non-allowlisted method gets the SAME code.
        // There is no longer a forbidden-vs-unsupported split, and the rejection
        // must not collapse into any other error class either.
        assert!(
            !line.contains("command_plane_forbidden")
                && !line.contains("unsupported_method")
                && !line.contains("handler_rejected")
                && !line.contains("malformed_json")
                && !line.contains("unexpected_field"),
            "{}: rejection must be the single method_not_allowed code, got `{line}`",
            case.label
        );
    }
}

/// Recovery: in ONE byte stream, a rejected frame must not poison the session.
/// The rejected frame fails closed, then a valid `zero_query` frame still
/// succeeds, and the process as a whole still exits non-zero because of the
/// earlier error frame.
#[test]
fn rejected_frame_does_not_poison_session_v1() {
    let label = "rejected then zero_query recovery";
    let stream = "{\"method\":\"workspace/executeCommand\"}\n{\"method\":\"zero_query\"}\n";
    let output = run_binary_bytes_v1(stream.as_bytes());
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout utf-8");
    let lines = response_lines_v1(&stdout);

    assert_eq!(
        lines.len(),
        2,
        "{label}: expected exactly two response frames, got `{stdout}`"
    );

    // First frame: the EXACT method_not_allowed envelope for executeCommand.
    let rejected = lines[0];
    assert_eq!(
        rejected,
        r#"{"ok":false,"error":{"code":"method_not_allowed","message":"method `workspace/executeCommand` is not an allowed read-only method"}}"#,
        "{label}: first frame must be the exact method_not_allowed envelope"
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
        "{label}: session must still exit 1 because of the rejected frame"
    );
}

/// An escaped-unicode method NAME decodes to its plain form BEFORE the allowlist
/// match, so it cannot smuggle past as a distinct string: `rename` decodes
/// to `rename` and is rejected with the single `method_not_allowed` code, echoing
/// the DECODED name (not the raw escape). This pins the decode-then-match path
/// for method names specifically (the value-escape path is pinned separately by
/// stdio_json_escape_negative.rs).
#[test]
fn escaped_unicode_method_name_decodes_then_is_not_allowed_v1() {
    // `rename` is the JSON-escaped form of `rename` (r == 'r'); the
    // request carries the literal escape, and the hand-rolled parser decodes it
    // before the allowlist match. Written as a normal string with `\\u` so a
    // real backslash-u reaches the binary.
    let (code, line) = run_binary_line_v1("{\"method\":\"\\u0072ename\"}");
    assert_eq!(code, Some(1), "escaped-unicode method must exit 1");
    assert_eq!(
        line,
        r#"{"ok":false,"error":{"code":"method_not_allowed","message":"method `rename` is not an allowed read-only method"}}"#,
        "escaped-unicode method must decode to `rename` and be method_not_allowed"
    );
}

/// An empty OR whitespace-only method fails closed with the dedicated
/// `empty_method` code (the method string is trimmed first, so `"   "` collapses
/// to empty). Pins both shapes — previously the empty/whitespace path had no
/// test.
#[test]
fn empty_or_whitespace_method_yields_empty_method_v1() {
    for request in [r#"{"method":""}"#, r#"{"method":"   "}"#] {
        let (code, line) = run_binary_line_v1(request);
        assert_eq!(
            code,
            Some(1),
            "`{request}`: empty/whitespace method must exit 1"
        );
        assert_eq!(
            line,
            r#"{"ok":false,"error":{"code":"empty_method","message":"method cannot be empty"}}"#,
            "`{request}`: must be the exact empty_method envelope"
        );
    }
}
