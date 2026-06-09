#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! LSP stdio negative-DSL binary hellgate.
//!
//! Promotes the DSL *negative / fail-closed* contract to the real spawned
//! `quanta-gqlang-v2-lsp` stdio binary front door. For each malformed DSL root
//! we prove the process either fails closed (`hover` -> `handler_rejected`,
//! exit 1) or surfaces a classified diagnostic item (`diagnostics` ->
//! `ok:true`, exit 0) deterministically. Schema-level rejections (unknown
//! method, field shape, framing, escapes) are owned by the sibling stdio test
//! files and are intentionally not duplicated here.

mod support;

use support::{response_lines_v1, run_binary_bytes_v1, run_binary_line_v1};

/// One table-driven negative case: a labeled request plus the stable substring
/// fragment the observed response line must contain.
struct NegativeCaseV1 {
    label: &'static str,
    request: &'static str,
    expect_fragment: &'static str,
}

/// Malformed DSL roots routed through `hover` must FAIL CLOSED: exit code 1,
/// an `ok:false` envelope, the `handler_rejected` outer code, and the embedded
/// DSL `code=<...>` classification of the rejecting parser/sema stage.
#[test]
fn stdio_hover_fails_closed_on_malformed_dsl_roots_v1() {
    let cases = [
        NegativeCaseV1 {
            label: "find-unknown-subkind",
            request: r#"{"method":"hover","query":"FIND unknown src -> sink"}"#,
            expect_fragment: "code=unsupported_find_subkind",
        },
        NegativeCaseV1 {
            label: "describe-invalid-shape",
            request: r#"{"method":"hover","query":"DESCRIBE model::"}"#,
            expect_fragment: "code=describe_target_invalid_shape",
        },
        NegativeCaseV1 {
            label: "check-missing-rule-or-scope",
            request: r#"{"method":"hover","query":"CHECK rule.hybrid"}"#,
            expect_fragment: "code=check_missing_rule_or_scope",
        },
        NegativeCaseV1 {
            label: "explain-invalid-shape",
            request: r#"{"method":"hover","query":"EXPLAIN rule::owasp.sql-injection ;; truth=maybe"}"#,
            expect_fragment: "code=invalid_explain_shape",
        },
        NegativeCaseV1 {
            label: "require-unsupported-meta",
            request: r#"{"method":"hover","query":"REQUIRE CERTIFIED ;; truth=maybe"}"#,
            expect_fragment: "code=unsupported_require_meta",
        },
        NegativeCaseV1 {
            label: "use-unsupported-meta",
            request: r#"{"method":"hover","query":"USE PROFILE strict-review"}"#,
            expect_fragment: "code=unsupported_use_meta",
        },
        NegativeCaseV1 {
            label: "nonsense-unsupported-root",
            request: r#"{"method":"hover","query":"FROBNICATE x"}"#,
            expect_fragment: "code=unsupported_root",
        },
    ];

    for case in cases {
        let (exit, line) = run_binary_line_v1(case.request);
        assert_eq!(exit, Some(1), "{}: must exit fail-closed", case.label);
        assert!(
            line.contains(r#""ok":false"#),
            "{}: expected ok:false envelope, got {line}",
            case.label
        );
        assert!(
            line.contains(r#""code":"handler_rejected""#),
            "{}: expected handler_rejected outer code, got {line}",
            case.label
        );
        assert!(
            line.contains(case.expect_fragment),
            "{}: expected embedded `{}`, got {line}",
            case.label,
            case.expect_fragment
        );
    }
}

/// Malformed DSL roots routed through `diagnostics` must REPORT, not fail
/// closed: exit code 0, an `ok:true` envelope, and a classified diagnostic
/// item carrying the expected `"code":"<...>"`.
#[test]
fn stdio_diagnostics_reports_classified_dsl_items_v1() {
    let cases = [
        NegativeCaseV1 {
            label: "use-pack-invalid-lockfile",
            request: r#"{"method":"diagnostics","query":"USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.txt"}"#,
            expect_fragment: r#""code":"use_pack_invalid_lockfile""#,
        },
        NegativeCaseV1 {
            label: "select-missing-from",
            request: r#"{"method":"diagnostics","query":"SELECT id catalog.users"}"#,
            expect_fragment: r#""code":"expected_from_keyword""#,
        },
        NegativeCaseV1 {
            label: "show-unsupported-category",
            request: r#"{"method":"diagnostics","query":"SHOW catalog"}"#,
            expect_fragment: r#""code":"unsupported_show_category""#,
        },
    ];

    for case in cases {
        let (exit, line) = run_binary_line_v1(case.request);
        assert_eq!(
            exit,
            Some(0),
            "{}: diagnostics must report (exit 0)",
            case.label
        );
        assert!(
            line.contains(r#""ok":true"#),
            "{}: expected ok:true envelope, got {line}",
            case.label
        );
        assert!(
            line.contains(case.expect_fragment),
            "{}: expected classified item `{}`, got {line}",
            case.label,
            case.expect_fragment
        );
    }
}

/// A DSL fail-closed frame must not poison the session: a malformed `hover`
/// frame followed by a valid `zero_query` frame in ONE byte stream yields a
/// fail-closed first line and an `ok:true` second line, with the process still
/// exiting code 1 (the worst response wins).
#[test]
fn stdio_session_recovers_after_dsl_fail_closed_v1() {
    let stream = concat!(
        r#"{"method":"hover","query":"FROBNICATE x"}"#,
        "\n",
        r#"{"method":"zero_query"}"#,
        "\n",
    );
    let output = run_binary_bytes_v1(stream.as_bytes());
    let stdout = String::from_utf8(output.stdout.clone()).expect("stdout utf-8");
    let lines = response_lines_v1(&stdout);

    assert_eq!(
        lines.len(),
        2,
        "expected exactly two response lines, got {stdout}"
    );

    let fail_line = lines[0];
    assert!(
        fail_line.contains(r#""ok":false"#),
        "first line must be fail-closed, got {fail_line}"
    );
    assert!(
        fail_line.contains(r#""code":"handler_rejected""#),
        "first line must carry handler_rejected, got {fail_line}"
    );
    assert!(
        fail_line.contains("code=unsupported_root"),
        "first line must carry the embedded DSL code, got {fail_line}"
    );

    let recovered_line = lines[1];
    assert!(
        recovered_line.contains(r#""ok":true"#),
        "second line must recover ok:true, got {recovered_line}"
    );
    assert!(
        recovered_line.contains(r#""method":"zero_query""#),
        "second line must be the zero_query response, got {recovered_line}"
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "session that observed a fail-closed frame must exit code 1"
    );
}
