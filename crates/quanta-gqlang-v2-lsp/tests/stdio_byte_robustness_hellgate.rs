#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Hellgate: byte-level robustness of the LSP stdio transport.
//!
//! `run_stdio_session_v1` reads raw bytes in 4 KiB chunks, accumulates them into
//! a frame buffer, and drains brace-balanced JSON objects. Three byte-level gaps
//! sit below the newline-framed happy path and are pinned here:
//!   * a brace-balanced frame whose bytes are **not valid UTF-8** fails closed as
//!     `malformed_json` (the UTF-8 validation branch in `lib.rs`), and the session
//!     still recovers to serve a following valid frame,
//!   * a request delivered one byte per `read()` call is reassembled byte-for-byte
//!     identically to a single-shot read (single and glued requests), and
//!   * a high count of glued frames with no separators each dispatch in order.
//!
//! The vehicle is the v3-only read-only method `v3_diagnostics`; a clean compile
//! yields a fully deterministic empty-result line, so every response line is
//! pinned whole with `assert_eq!`. The invalid-UTF-8 message carries a byte
//! index, so its stable prefix is pinned.

mod support;

use std::io::Read;

use support::{binary_stdout_v1, response_lines_v1, run_binary_bytes_v1, run_session_bytes_v1};

use quanta_gqlang_v2_lsp::run_stdio_session_v1;

/// A clean-compiling relational query, used purely as a deterministic transport
/// vehicle: the pipeline compiles it end to end, so `v3_diagnostics` returns an
/// empty diagnostics list — a byte-stable line we can pin whole.
const CLEAN_QUERY_V1: &str =
    r#"{"method":"v3_diagnostics","query":"from users u | project u.id as id"}"#;

/// The exact, full-line response for `CLEAN_QUERY_V1`.
const CLEAN_QUERY_LINE_V1: &str = r#"{"ok":true,"method":"v3_diagnostics","result":[]}"#;

/// A `Read` adapter that yields **at most one byte per `read()` call**, modelling
/// a client that dribbles a request across many tiny syscalls. Proving the session
/// reassembles such a stream pins the cross-read accumulation in `lib.rs`.
struct OneByteReaderV1<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> OneByteReaderV1<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }
}

impl Read for OneByteReaderV1<'_> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if out.is_empty() || self.cursor >= self.bytes.len() {
            return Ok(0);
        }
        out[0] = self.bytes[self.cursor];
        self.cursor += 1;
        Ok(1)
    }
}

/// Drive the in-process session over `input` delivered one byte per `read()`.
fn run_one_byte_session_v1(input: &[u8]) -> String {
    let mut reader = OneByteReaderV1::new(input);
    let mut output = Vec::new();
    run_stdio_session_v1(&mut reader, &mut output).expect("one-byte session runs");
    String::from_utf8(output).expect("stdout utf-8")
}

// ------------------------------------------------------------------------
// 1. Invalid UTF-8 inside a brace-balanced frame.
// ------------------------------------------------------------------------

/// The 0xFF/0xFE bytes live inside the JSON string value, so braces still balance
/// and the frame extracts cleanly; UTF-8 validation then rejects it. The error
/// message embeds a byte index, so the stable prefix is pinned rather than the
/// whole line.
const INVALID_UTF8_FRAME_V1: &[u8] = b"{\"method\":\"v3_diagnostics\",\"query\":\"\xff\xfe\"}\n";
const INVALID_UTF8_MESSAGE_PREFIX_V1: &str = "request bytes are not valid UTF-8";

#[test]
fn binary_rejects_invalid_utf8_inside_balanced_frame_v1() {
    let output = run_binary_bytes_v1(INVALID_UTF8_FRAME_V1);
    assert_eq!(
        output.status.code(),
        Some(1),
        "invalid UTF-8 must fail closed"
    );
    let stdout = binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 1, "exactly one error line, got {lines:?}");
    let line = lines[0];
    assert!(
        line.contains(r#""ok":false"#),
        "invalid UTF-8 must be a fail-closed envelope: {line}"
    );
    assert!(
        line.contains(r#""code":"malformed_json""#),
        "invalid UTF-8 must surface as malformed_json: {line}"
    );
    assert!(
        line.contains(INVALID_UTF8_MESSAGE_PREFIX_V1),
        "invalid UTF-8 message prefix drifted: {line}"
    );
}

#[test]
fn binary_recovers_after_invalid_utf8_frame_v1() {
    // The invalid-UTF-8 frame is immediately followed by a valid frame in one byte
    // stream. The session must emit the error for the first frame and still serve
    // the second, proving the read-only front door recovers in place.
    let mut input = INVALID_UTF8_FRAME_V1.to_vec();
    input.extend_from_slice(CLEAN_QUERY_V1.as_bytes());
    input.push(b'\n');

    let output = run_binary_bytes_v1(&input);
    assert_eq!(
        output.status.code(),
        Some(1),
        "a fail-closed frame anywhere poisons the exit status"
    );
    let stdout = binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_eq!(
        lines.len(),
        2,
        "error line then recovery line, got {lines:?}"
    );
    assert!(
        lines[0].contains(r#""code":"malformed_json""#)
            && lines[0].contains(INVALID_UTF8_MESSAGE_PREFIX_V1),
        "first line must be the invalid-UTF-8 error: {}",
        lines[0]
    );
    assert_eq!(
        lines[1].trim(),
        CLEAN_QUERY_LINE_V1,
        "session must recover and serve the trailing v3_diagnostics frame: {}",
        lines[1]
    );
}

// ------------------------------------------------------------------------
// 2. Fragmented delivery — one byte per read() call.
// ------------------------------------------------------------------------

#[test]
fn session_reassembles_single_request_from_one_byte_reads_v1() {
    let request = format!("{CLEAN_QUERY_V1}\n");
    let dribbled = run_one_byte_session_v1(request.as_bytes());
    let (single_status, single_shot) = run_session_bytes_v1(request.as_bytes());
    assert_eq!(single_status, 0);
    assert_eq!(
        dribbled, single_shot,
        "one-byte delivery must produce a byte-identical response"
    );
    let lines = response_lines_v1(&dribbled);
    assert_eq!(lines.len(), 1, "got {lines:?}");
    assert_eq!(lines[0].trim(), CLEAN_QUERY_LINE_V1);
}

#[test]
fn session_reassembles_two_glued_requests_from_one_byte_reads_v1() {
    // Two objects glued with no separator, then dribbled one byte at a time. Both
    // must dispatch in order and match the single-shot read exactly.
    let glued = format!("{CLEAN_QUERY_V1}{CLEAN_QUERY_V1}");
    let dribbled = run_one_byte_session_v1(glued.as_bytes());
    let (single_status, single_shot) = run_session_bytes_v1(glued.as_bytes());
    assert_eq!(single_status, 0);
    assert_eq!(
        dribbled, single_shot,
        "one-byte delivery of glued frames must match single-shot bytes"
    );
    let lines = response_lines_v1(&dribbled);
    assert_eq!(
        lines.len(),
        2,
        "both glued frames must dispatch, got {lines:?}"
    );
    assert_eq!(lines[0].trim(), CLEAN_QUERY_LINE_V1);
    assert_eq!(lines[1].trim(), CLEAN_QUERY_LINE_V1);
}

// ------------------------------------------------------------------------
// 3. High-count glued frames.
// ------------------------------------------------------------------------

const GLUED_FRAME_COUNT_V1: usize = 64;

fn glued_clean_query_buffer_v1() -> Vec<u8> {
    CLEAN_QUERY_V1.repeat(GLUED_FRAME_COUNT_V1).into_bytes()
}

fn assert_all_clean_query_v1(lines: &[&str]) {
    assert_eq!(
        lines.len(),
        GLUED_FRAME_COUNT_V1,
        "expected exactly {GLUED_FRAME_COUNT_V1} response lines, got {}",
        lines.len()
    );
    for line in lines {
        assert_eq!(line.trim(), CLEAN_QUERY_LINE_V1);
    }
}

#[test]
fn session_drains_high_count_glued_frames_in_process_v1() {
    let buffer = glued_clean_query_buffer_v1();
    let (status, stdout) = run_session_bytes_v1(&buffer);
    assert_eq!(status, 0);
    let lines = response_lines_v1(&stdout);
    assert_all_clean_query_v1(&lines);
}

#[test]
fn binary_drains_high_count_glued_frames_v1() {
    let buffer = glued_clean_query_buffer_v1();
    let output = run_binary_bytes_v1(&buffer);
    assert_eq!(output.status.code(), Some(0));
    let stdout = binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_all_clean_query_v1(&lines);
}
