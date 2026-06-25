#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed paths"
)]

//! Hellgate: byte-level frame-extractor reject and brace-depth edges.
//!
//! `server::extract_next_request_frame_v1` scans raw bytes for a brace-balanced
//! object before any JSON parsing runs. Two of its branches are pinned here:
//!   * a frame that does NOT begin with `{` (after leading whitespace) is
//!     rejected up front as `malformed_json` with the start-with-`{` message,
//!     BEFORE the parser ever sees it; and
//!   * a frame with a NESTED object exercises the brace-depth increment/decrement
//!     so the extractor returns the whole outer object (depth back to zero at the
//!     final `}`); the flat-object parser then rejects the nested value as a
//!     non-string field value.
//!
//! Both pin the spawned binary's exact full response line and an exit code of 1,
//! and prove the session recovers for a following valid frame.

mod support;

use support::{binary_stdout_v1, response_lines_v1, run_binary_bytes_v1, run_binary_line_v1};

const CLEAN_QUERY_V1: &str =
    r#"{"method":"v3_diagnostics","query":"from users u | project u.id as id"}"#;

#[test]
fn frame_not_starting_with_brace_is_rejected_up_front_v1() {
    // Leading whitespace is skipped, then the first non-whitespace byte is not
    // `{`, so the extractor rejects before any object parsing.
    let (code, line) = run_binary_line_v1("   garbage");
    assert_eq!(code, Some(1), "non-brace frame must fail closed");
    assert_eq!(
        line,
        r#"{"ok":false,"error":{"code":"malformed_json","message":"expected JSON object to start with `{`"}}"#,
        "non-brace start must be the start-with-brace rejection"
    );
}

#[test]
fn nested_object_frame_extracts_whole_then_parser_rejects_value_v1() {
    // The nested `{...}` drives the brace-depth increment/decrement in the
    // extractor: depth rises to 2 at the inner `{`, falls back to 1, then to 0 at
    // the final `}`, so the WHOLE outer object is returned as one frame. The
    // flat-object parser then rejects the nested object as a non-string value.
    let (code, line) = run_binary_line_v1(r#"{"method":{"x":"y"}}"#);
    assert_eq!(code, Some(1), "nested-value frame must fail closed");
    assert_eq!(
        line,
        r#"{"ok":false,"error":{"code":"malformed_json","message":"only string field values are supported"}}"#,
        "nested object value must be rejected as a non-string field value"
    );
}

/// A non-brace frame is an UNRESYNCABLE position in the byte stream: unlike a
/// per-frame parse reject (which only drops the one balanced object), the
/// extractor cannot find a frame boundary, so the front door emits ONE rejection
/// and clears the entire remaining buffer rather than risk dispatching garbage.
/// This locks that fail-closed-and-halt contract: the trailing clean frame is
/// deliberately NOT served.
#[test]
fn non_brace_frame_clears_the_buffer_and_does_not_serve_the_trailing_frame_v1() {
    let stream = format!("garbage\n{CLEAN_QUERY_V1}\n");
    let output = run_binary_bytes_v1(stream.as_bytes());
    assert_eq!(
        output.status.code(),
        Some(1),
        "the rejected frame sets exit 1"
    );
    let stdout = binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_eq!(
        lines.len(),
        1,
        "an unresyncable frame emits exactly one error and halts, got {lines:?}"
    );
    assert_eq!(
        lines[0],
        r#"{"ok":false,"error":{"code":"malformed_json","message":"expected JSON object to start with `{`"}}"#,
        "the single line is the non-brace rejection"
    );
}
