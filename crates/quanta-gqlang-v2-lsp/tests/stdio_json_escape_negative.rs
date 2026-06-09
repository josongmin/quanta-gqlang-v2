#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Hellgate: LSP JSON escape / surrogate-pair rejection.
//!
//! `server.rs` carries a precise hand-rolled string decoder with several
//! surrogate-pair error branches (`parse_unicode_escape_v1`). The existing
//! stdio rail only proves the *positive* path (a valid `😀` pair), so
//! every negative branch is locked here against the real binary front door:
//! each malformed escape must come back as a single `malformed_json` line with
//! exit code 1, and a poisoned request must not poison the session.

mod support;

use support::{response_lines_v1, run_binary_bytes_v1, run_binary_line_v1};

struct EscapeRejectionCaseV1 {
    label: &'static str,
    request: &'static str,
    /// The EXACT, full trimmed response line. Each surrogate/escape branch emits
    /// a short deterministic error object, so pinning the whole line (not a
    /// substring) kills every field-value, message-text, and code mutation in
    /// `parse_unicode_escape_v1`. Captured from the real binary.
    expected_line: &'static str,
}

#[test]
fn invalid_escapes_and_surrogates_fail_closed_v1() {
    let cases = [
        EscapeRejectionCaseV1 {
            label: "lone low surrogate",
            request: r#"{"method":"completion","prefix":"\uDE00"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"unexpected low surrogate `\\uDE00`"}}"#,
        },
        EscapeRejectionCaseV1 {
            label: "high surrogate not followed by an escape separator",
            request: r#"{"method":"completion","prefix":"\uD83DA"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"expected `\\`, found `A`"}}"#,
        },
        EscapeRejectionCaseV1 {
            label: "high surrogate followed by another high surrogate",
            request: r#"{"method":"completion","prefix":"\uD83D\uD83D"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"expected low surrogate after `\\uD83D`"}}"#,
        },
        EscapeRejectionCaseV1 {
            label: "high surrogate with a non-`u` escape separator",
            request: r#"{"method":"completion","prefix":"\uD83D\n"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"expected `u` after surrogate pair separator, found `n`"}}"#,
        },
        EscapeRejectionCaseV1 {
            label: "non-hex unicode escape digit",
            request: r#"{"method":"completion","prefix":"\uXYZ0"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"invalid unicode escape digit `X`"}}"#,
        },
        EscapeRejectionCaseV1 {
            label: "unsupported escape sequence",
            request: r#"{"method":"completion","prefix":"\x"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"unsupported JSON escape `\\x`"}}"#,
        },
    ];

    for case in &cases {
        let (code, line) = run_binary_line_v1(case.request);
        assert_eq!(code, Some(1), "{}: must fail closed (exit 1)", case.label);
        // Full-line equality: covers ok:false, the exact code, AND the exact
        // surrogate/escape message in one mutation-proof assertion.
        assert_eq!(
            line, case.expected_line,
            "{}: full line drifted",
            case.label
        );
    }
}

#[test]
fn session_recovers_after_a_malformed_escape_v1() {
    // A bad-escape frame followed by a valid frame in the same byte stream: the
    // first must fail closed without swallowing or corrupting the second.
    let stream = concat!(
        "{\"method\":\"completion\",\"prefix\":\"\\uDE00\"}\n",
        "{\"method\":\"zero_query\"}\n",
    );
    let output = run_binary_bytes_v1(stream.as_bytes());
    assert_eq!(
        output.status.code(),
        Some(1),
        "the failing frame sets exit 1"
    );

    let stdout = support::binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_eq!(
        lines.len(),
        2,
        "both frames must produce a response, got {lines:?}"
    );
    // First line: the exact lone-low-surrogate rejection (not just any error).
    assert_eq!(
        lines[0],
        r#"{"ok":false,"error":{"code":"malformed_json","message":"unexpected low surrogate `\\uDE00`"}}"#,
        "first frame must be the exact lone-low-surrogate rejection"
    );
    // Second line: the recovered zero_query envelope, pinned to its
    // deterministic read-only prefix so recovery cannot pass on a stub line.
    assert!(
        lines[1].starts_with(
            r#"{"ok":true,"method":"zero_query","result":{"mode":"read-only","provenance":"typed-catalog-example-metadata","groups":[{"id":"catalog","#
        ),
        "second frame must recover the read-only zero_query envelope, got {}",
        lines[1]
    );
    assert!(
        lines[1].ends_with("]}}"),
        "recovered zero_query must close groups array + result + envelope, got {}",
        lines[1]
    );
}
