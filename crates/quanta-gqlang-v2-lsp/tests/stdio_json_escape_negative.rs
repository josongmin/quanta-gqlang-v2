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
    message_fragment: &'static str,
}

#[test]
fn invalid_escapes_and_surrogates_fail_closed_v1() {
    let cases = [
        EscapeRejectionCaseV1 {
            label: "lone low surrogate",
            request: r#"{"method":"completion","prefix":"\uDE00"}"#,
            message_fragment: "unexpected low surrogate",
        },
        EscapeRejectionCaseV1 {
            label: "high surrogate not followed by an escape separator",
            request: r#"{"method":"completion","prefix":"\uD83DA"}"#,
            message_fragment: "found `A`",
        },
        EscapeRejectionCaseV1 {
            label: "high surrogate followed by another high surrogate",
            request: r#"{"method":"completion","prefix":"\uD83D\uD83D"}"#,
            message_fragment: "expected low surrogate after",
        },
        EscapeRejectionCaseV1 {
            label: "high surrogate with a non-`u` escape separator",
            request: r#"{"method":"completion","prefix":"\uD83D\n"}"#,
            message_fragment: "after surrogate pair separator",
        },
        EscapeRejectionCaseV1 {
            label: "non-hex unicode escape digit",
            request: r#"{"method":"completion","prefix":"\uXYZ0"}"#,
            message_fragment: "invalid unicode escape digit",
        },
        EscapeRejectionCaseV1 {
            label: "unsupported escape sequence",
            request: r#"{"method":"completion","prefix":"\x"}"#,
            message_fragment: "unsupported JSON escape",
        },
    ];

    for case in &cases {
        let (code, line) = run_binary_line_v1(case.request);
        assert_eq!(code, Some(1), "{}: must fail closed (exit 1)", case.label);
        assert!(
            line.contains(r#""ok":false"#),
            "{}: expected ok:false, got {line}",
            case.label
        );
        assert!(
            line.contains(r#""code":"malformed_json""#),
            "{}: expected malformed_json, got {line}",
            case.label
        );
        assert!(
            line.contains(case.message_fragment),
            "{}: expected message fragment `{}`, got {line}",
            case.label,
            case.message_fragment
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
    assert!(
        lines[0].contains(r#""code":"malformed_json""#),
        "got {}",
        lines[0]
    );
    assert!(lines[1].contains(r#""ok":true"#), "got {}", lines[1]);
    assert!(
        lines[1].contains(r#""method":"zero_query""#),
        "got {}",
        lines[1]
    );
}
