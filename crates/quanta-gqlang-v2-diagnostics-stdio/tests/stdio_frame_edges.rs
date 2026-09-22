#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
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
//! The vehicle is the v3-only `v3_diagnostics` method; a clean compile yields a
//! fully deterministic empty-result line, so every response line is pinned whole.
//! Each invariant is pinned twice: once in-process over an exact byte buffer
//! (`run_session_bytes_v1`) for byte precision, and once against the spawned
//! binary (`run_binary_bytes_v1`) for the true process-level contract.

mod support;

use support::{
    binary_stderr_v1, binary_stdout_v1, response_lines_v1, run_binary_bytes_v1,
    run_session_bytes_v1,
};

/// A clean-compiling query used as a deterministic transport vehicle.
const CLEAN_QUERY_V1: &str =
    r#"{"method":"v3_diagnostics","query":"from users u | project u.id as id"}"#;

/// A second clean query, distinct from `CLEAN_QUERY_V1`, so glued-frame ordering
/// can be told apart frame-by-frame.
const CLEAN_QUERY_TWO_V1: &str =
    r#"{"method":"v3_diagnostics","query":"from orders o | project o.id as id"}"#;

/// An object left unclosed at EOF: the trailing `"` opens a string that the
/// closing brace never escapes out of, so the frame never balances.
const UNCLOSED_OBJECT_V1: &str =
    r#"{"method":"v3_diagnostics","query":"from users u | project u.id as id""#;

/// The exact, full-line response for `CLEAN_QUERY_V1` / `CLEAN_QUERY_TWO_V1`
/// (both compile cleanly, so both surface an empty diagnostics list).
const CLEAN_QUERY_LINE_V1: &str = r#"{"ok":true,"method":"v3_diagnostics","result":[]}"#;

/// The exact fail-closed line for an object left unclosed at EOF.
const UNCLOSED_ERROR_LINE_V1: &str = r#"{"ok":false,"error":{"code":"malformed_json","message":"incomplete JSON object at end of input"}}"#;

// ------------------------------------------------------------------------
// In-process, byte-precise framing.
// ------------------------------------------------------------------------

#[test]
fn session_drains_final_frame_without_trailing_newline_v1() {
    // No `\n` after the closing brace: the frame must still flush at EOF.
    let (status, stdout) = run_session_bytes_v1(CLEAN_QUERY_V1.as_bytes());
    assert_eq!(status, 0);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 1, "exactly one response, got {lines:?}");
    assert_eq!(lines[0].trim(), CLEAN_QUERY_LINE_V1);
}

#[test]
fn session_processes_two_glued_objects_in_one_buffer_v1() {
    // Concatenated with no separator and no trailing newline.
    let glued = format!("{CLEAN_QUERY_V1}{CLEAN_QUERY_TWO_V1}");
    let (status, stdout) = run_session_bytes_v1(glued.as_bytes());
    assert_eq!(status, 0);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 2, "both frames must dispatch, got {lines:?}");
    assert_eq!(lines[0].trim(), CLEAN_QUERY_LINE_V1);
    assert_eq!(lines[1].trim(), CLEAN_QUERY_LINE_V1);
}

#[test]
fn session_processes_whitespace_separated_objects_v1() {
    let separated = format!("{CLEAN_QUERY_V1}\n  \n{CLEAN_QUERY_TWO_V1}\n");
    let (status, stdout) = run_session_bytes_v1(separated.as_bytes());
    assert_eq!(status, 0);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 2, "got {lines:?}");
    assert_eq!(lines[0].trim(), CLEAN_QUERY_LINE_V1);
    assert_eq!(lines[1].trim(), CLEAN_QUERY_LINE_V1);
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
    let output = run_binary_bytes_v1(CLEAN_QUERY_V1.as_bytes());
    assert_eq!(output.status.code(), Some(0));
    assert!(
        binary_stderr_v1(&output).is_empty(),
        "clean drain must not write to stderr"
    );
    let stdout = binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 1, "got {lines:?}");
    assert_eq!(lines[0].trim(), CLEAN_QUERY_LINE_V1);
}

#[test]
fn binary_processes_glued_frames_then_clean_eof_v1() {
    let glued = format!("{CLEAN_QUERY_V1}{CLEAN_QUERY_TWO_V1}");
    let output = run_binary_bytes_v1(glued.as_bytes());
    assert_eq!(output.status.code(), Some(0));
    let stdout = binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 2, "got {lines:?}");
    assert_eq!(lines[0].trim(), CLEAN_QUERY_LINE_V1);
    assert_eq!(lines[1].trim(), CLEAN_QUERY_LINE_V1);
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
