#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Hellgate: LSP stdio framing contract.
//!
//! `server::extract_next_request_frame_v1` owns the byte-level frame boundary
//! logic (balanced-brace scan with string/escape awareness), so the stdio front
//! door must hold these invariants regardless of how a client chunks its writes:
//!   * a final request with no trailing newline is still drained at EOF,
//!   * several objects glued into one buffer are each dispatched in order,
//!   * an object left unclosed at EOF fails closed as `malformed_json`,
//!   * empty / whitespace-only input exits cleanly with no output.
//!
//! Each invariant is pinned twice: once in-process over an exact byte buffer
//! (`run_session_bytes_v1`) for byte precision, and once against the spawned
//! binary (`run_binary_bytes_v1`) for the true process-level contract.

mod support;

use support::{
    binary_stderr_v1, binary_stdout_v1, response_lines_v1, run_binary_bytes_v1,
    run_session_bytes_v1,
};

const ZERO_QUERY_V1: &str = r#"{"method":"zero_query"}"#;
const HOVER_REQUIRE_V1: &str = r#"{"method":"hover","query":"REQUIRE CERTIFIED"}"#;
const UNCLOSED_OBJECT_V1: &str = r#"{"method":"hover","query":"SELECT id FROM catalog.files""#;

/// The exact, full-line `hover` card for `REQUIRE CERTIFIED`. The server's
/// serializer emits a deterministic field order, so pinning the whole line (not
/// a scatter of `contains`) kills any field-value or ordering mutation in the
/// meta hover path.
const HOVER_REQUIRE_LINE_V1: &str = r#"{"ok":true,"method":"hover","result":{"root_keyword":"REQUIRE CERTIFIED","category":"meta","typed_target":"certified-only","summary":["truth=certified-only","admission=fail-closed"]}}"#;

/// The exact fail-closed line for an object left unclosed at EOF.
const UNCLOSED_ERROR_LINE_V1: &str = r#"{"ok":false,"error":{"code":"malformed_json","message":"incomplete JSON object at end of input"}}"#;

/// Assert that a `zero_query` response line is the genuine read-only scaffold
/// envelope, pinning the deterministic prefix tokens in order so the test can
/// never pass on a truncated or reshaped payload. The body is large and
/// example-driven, so we anchor the stable envelope rather than the full line.
fn assert_zero_query_envelope_v1(line: &str) {
    assert!(
        line.starts_with(
            r#"{"ok":true,"method":"zero_query","result":{"mode":"read-only","provenance":"typed-catalog-example-metadata","groups":[{"id":"catalog","#
        ),
        "zero_query envelope/prefix drifted: {line}"
    );
    assert!(
        line.ends_with("]}}"),
        "zero_query response must close groups array + result + envelope: {line}"
    );
    assert!(
        line.contains(r#""behavior":"read-only-scaffold""#),
        "zero_query body must carry read-only-scaffold suggestions: {line}"
    );
}

// ------------------------------------------------------------------------
// In-process, byte-precise framing.
// ------------------------------------------------------------------------

#[test]
fn session_drains_final_frame_without_trailing_newline_v1() {
    // No `\n` after the closing brace: the frame must still flush at EOF.
    let (status, stdout) = run_session_bytes_v1(ZERO_QUERY_V1.as_bytes());
    assert_eq!(status, 0);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 1, "exactly one response, got {lines:?}");
    assert_zero_query_envelope_v1(lines[0]);
}

#[test]
fn session_processes_two_glued_objects_in_one_buffer_v1() {
    // Concatenated with no separator and no trailing newline.
    let glued = format!("{ZERO_QUERY_V1}{HOVER_REQUIRE_V1}");
    let (status, stdout) = run_session_bytes_v1(glued.as_bytes());
    assert_eq!(status, 0);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 2, "both frames must dispatch, got {lines:?}");
    assert_zero_query_envelope_v1(lines[0]);
    // Second frame is short and fully deterministic: pin the whole line so a
    // glued-frame boundary that drops/duplicates bytes cannot pass.
    assert_eq!(lines[1].trim(), HOVER_REQUIRE_LINE_V1);
}

#[test]
fn session_processes_whitespace_separated_objects_v1() {
    let separated = format!("{ZERO_QUERY_V1}\n  \n{HOVER_REQUIRE_V1}\n");
    let (status, stdout) = run_session_bytes_v1(separated.as_bytes());
    assert_eq!(status, 0);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 2, "got {lines:?}");
    assert_zero_query_envelope_v1(lines[0]);
    assert_eq!(lines[1].trim(), HOVER_REQUIRE_LINE_V1);
}

#[test]
fn session_fails_closed_on_object_left_unclosed_at_eof_v1() {
    let (status, stdout) = run_session_bytes_v1(UNCLOSED_OBJECT_V1.as_bytes());
    assert_eq!(status, 1, "an incomplete frame at EOF must fail closed");
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 1, "got {lines:?}");
    assert_eq!(lines[0].trim(), UNCLOSED_ERROR_LINE_V1);
}

#[test]
fn session_emits_nothing_for_empty_input_v1() {
    let (status, stdout) = run_session_bytes_v1(b"");
    assert_eq!(status, 0);
    assert!(
        stdout.is_empty(),
        "empty input must produce no output, got {stdout:?}"
    );
}

#[test]
fn session_emits_nothing_for_whitespace_only_input_v1() {
    let (status, stdout) = run_session_bytes_v1(b"  \n\t\r\n   \n");
    assert_eq!(status, 0);
    assert!(
        stdout.is_empty(),
        "whitespace-only input must produce no output, got {stdout:?}"
    );
}

// ------------------------------------------------------------------------
// Spawned binary front door — the same contract across a real process.
// ------------------------------------------------------------------------

#[test]
fn binary_drains_request_without_trailing_newline_v1() {
    let output = run_binary_bytes_v1(ZERO_QUERY_V1.as_bytes());
    assert_eq!(output.status.code(), Some(0));
    assert!(
        binary_stderr_v1(&output).is_empty(),
        "clean drain must not write to stderr"
    );
    let stdout = binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 1, "got {lines:?}");
    assert_zero_query_envelope_v1(lines[0]);
}

#[test]
fn binary_processes_glued_frames_then_clean_eof_v1() {
    let glued = format!("{ZERO_QUERY_V1}{HOVER_REQUIRE_V1}");
    let output = run_binary_bytes_v1(glued.as_bytes());
    assert_eq!(output.status.code(), Some(0));
    let stdout = binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 2, "got {lines:?}");
    assert_zero_query_envelope_v1(lines[0]);
    assert_eq!(lines[1].trim(), HOVER_REQUIRE_LINE_V1);
}

#[test]
fn binary_fails_closed_on_unclosed_object_at_eof_v1() {
    let output = run_binary_bytes_v1(UNCLOSED_OBJECT_V1.as_bytes());
    assert_eq!(output.status.code(), Some(1));
    let stdout = binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_eq!(
        lines.len(),
        1,
        "an unclosed frame emits exactly one error line, got {lines:?}"
    );
    assert_eq!(lines[0].trim(), UNCLOSED_ERROR_LINE_V1);
}

#[test]
fn binary_empty_stdin_exits_clean_without_output_v1() {
    let output = run_binary_bytes_v1(b"");
    assert_eq!(output.status.code(), Some(0));
    assert!(binary_stdout_v1(&output).is_empty());
    assert!(binary_stderr_v1(&output).is_empty());
}

#[test]
fn binary_whitespace_only_stdin_exits_clean_without_output_v1() {
    let output = run_binary_bytes_v1(b"   \n\t\n  ");
    assert_eq!(output.status.code(), Some(0));
    assert!(
        binary_stdout_v1(&output).is_empty(),
        "got {:?}",
        binary_stdout_v1(&output)
    );
}
