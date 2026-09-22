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

//! Hellgate: object-level JSON parse rejections at the stdio front door.
//!
//! `json::JsonCursorV1::parse_flat_object_v1` is the hand-rolled flat-object
//! reader the dispatch runs FIRST on every brace-balanced frame. Its reject
//! branches (non-string value, duplicate key, missing `,`/`:`, control char in a
//! string, malformed `\u` escapes) and the field-policy rejects layered on top of
//! it (`missing_field`, `unexpected_field`) are the fail-closed contract for a
//! malformed-but-balanced request. Every one is pinned here as an EXACT full
//! response line from the spawned binary, so a code/message/field mutation in the
//! parser flips a test. Each case also proves the failing frame exits 1 and does
//! not poison a following valid frame.
//!
//! Note on reachability: the brace-balanced frame extractor
//! (`server::extract_next_request_frame_v1`) hands the parser EXACTLY one balanced
//! object, so the parser's own trailing-content / unterminated-string branches are
//! unreachable from the front door and are pinned instead by the in-crate unit
//! tests in `src/json.rs`.

mod support;

use support::{response_lines_v1, run_binary_bytes_v1, run_binary_line_v1};

/// A clean-compiling query used as the recovery vehicle after a rejected frame.
const CLEAN_QUERY_V1: &str =
    r#"{"method":"v3_diagnostics","query":"from users u | project u.id as id"}"#;
const CLEAN_QUERY_LINE_V1: &str = r#"{"ok":true,"method":"v3_diagnostics","result":[]}"#;

struct ObjectRejectionCaseV1 {
    label: &'static str,
    /// A single balanced object whose bytes are valid UTF-8 but malformed JSON or
    /// field policy.
    request: &'static str,
    /// The EXACT full trimmed response line from the binary.
    expected_line: &'static str,
}

#[test]
fn malformed_balanced_objects_fail_closed_with_exact_lines_v1() {
    let cases = [
        ObjectRejectionCaseV1 {
            label: "non-string field value (number)",
            request: r#"{"method":42}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"only string field values are supported"}}"#,
        },
        ObjectRejectionCaseV1 {
            label: "non-string field value (bare true)",
            request: r#"{"method":true}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"only string field values are supported"}}"#,
        },
        ObjectRejectionCaseV1 {
            label: "duplicate field",
            request: r#"{"method":"v3_diagnostics","method":"x"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"duplicate_field","message":"duplicate field `method`"}}"#,
        },
        ObjectRejectionCaseV1 {
            label: "missing separator between pairs",
            request: r#"{"method":"v3_diagnostics" "query":"x"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"expected `,` or `}` in JSON object"}}"#,
        },
        ObjectRejectionCaseV1 {
            label: "missing colon after key",
            request: r#"{"method" "v3_diagnostics"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"expected `:`, found `\"`"}}"#,
        },
        ObjectRejectionCaseV1 {
            label: "control character (raw tab) inside a string value",
            request: "{\"method\":\"v3_diagnostics\",\"query\":\"a\tb\"}",
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"control character in JSON string"}}"#,
        },
        ObjectRejectionCaseV1 {
            label: "unicode escape with a non-hex digit",
            request: r#"{"method":"v3_diagnostics","query":"\u00g0"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"invalid unicode escape digit `g`"}}"#,
        },
        ObjectRejectionCaseV1 {
            label: "unicode escape truncated by the closing quote",
            request: r#"{"method":"v3_diagnostics","query":"\u12"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"malformed_json","message":"invalid unicode escape digit `\"`"}}"#,
        },
        ObjectRejectionCaseV1 {
            label: "missing required method field",
            request: r#"{}"#,
            expected_line: r#"{"ok":false,"error":{"code":"missing_field","message":"missing required field `method`"}}"#,
        },
        ObjectRejectionCaseV1 {
            label: "missing required query field",
            request: r#"{"method":"v3_diagnostics"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"missing_field","message":"missing required field `query`"}}"#,
        },
        ObjectRejectionCaseV1 {
            label: "single unexpected extra field",
            request: r#"{"method":"v3_diagnostics","query":"x","extra":"y"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"unexpected_field","message":"unexpected fields for `v3_diagnostics`: extra"}}"#,
        },
    ];

    for case in &cases {
        let (code, line) = run_binary_line_v1(case.request);
        assert_eq!(code, Some(1), "{}: must fail closed (exit 1)", case.label);
        assert_eq!(
            line, case.expected_line,
            "{}: full line drifted",
            case.label
        );
    }
}

/// Multiple extra fields are reported in BTreeMap (sorted) order, NOT insertion
/// order: `zzz` and `aaa` come back as `aaa, zzz`. This pins the deterministic
/// join order in `reject_extra_fields_v1`.
#[test]
fn multiple_unexpected_fields_are_reported_sorted_v1() {
    let (code, line) =
        run_binary_line_v1(r#"{"method":"v3_diagnostics","query":"x","zzz":"1","aaa":"2"}"#);
    assert_eq!(code, Some(1), "extra fields must fail closed");
    assert_eq!(
        line,
        r#"{"ok":false,"error":{"code":"unexpected_field","message":"unexpected fields for `v3_diagnostics`: aaa, zzz"}}"#,
        "extra fields must be reported in sorted order"
    );
}

/// A malformed-but-balanced frame must not poison a following valid frame in the
/// same byte stream: the parser reject is per-frame, the session recovers.
#[test]
fn malformed_object_does_not_poison_session_v1() {
    let stream = format!("{}\n{}\n", r#"{"method":42}"#, CLEAN_QUERY_V1);
    let output = run_binary_bytes_v1(stream.as_bytes());
    assert_eq!(
        output.status.code(),
        Some(1),
        "the failing frame sets exit 1"
    );
    let stdout = support::binary_stdout_v1(&output);
    let lines = response_lines_v1(&stdout);
    assert_eq!(lines.len(), 2, "both frames must respond, got {lines:?}");
    assert_eq!(
        lines[0],
        r#"{"ok":false,"error":{"code":"malformed_json","message":"only string field values are supported"}}"#,
        "first frame is the exact non-string-value rejection"
    );
    assert_eq!(
        lines[1], CLEAN_QUERY_LINE_V1,
        "session must recover the trailing clean frame"
    );
}
