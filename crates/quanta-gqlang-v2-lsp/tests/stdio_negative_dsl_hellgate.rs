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

use quanta_gqlang_v2_conformance::root_surface_specs_v1;
use quanta_gqlang_v2_ir::diagnostic_class::front_door_diagnostic_class_v1;
use support::{response_lines_v1, run_binary_bytes_v1, run_binary_line_v1};

/// One table-driven negative case: a labeled request plus the EXACT, full
/// trimmed response line the binary must emit. Full-line equality (captured
/// from the real binary) kills error-class, code, and message mutations that a
/// substring `contains` would silently survive.
struct NegativeCaseV1 {
    label: &'static str,
    request: &'static str,
    expected_line: &'static str,
}

/// Malformed DSL roots routed through `hover` must FAIL CLOSED: exit code 1,
/// an `ok:false` envelope, the `handler_rejected` outer code, and the embedded
/// `class=.. code=.. message=..` classification of the rejecting parser/sema
/// stage — all pinned exactly.
#[test]
fn stdio_hover_fails_closed_on_malformed_dsl_roots_v1() {
    let cases = [
        NegativeCaseV1 {
            label: "find-unknown-subkind",
            request: r#"{"method":"hover","query":"FIND unknown src -> sink"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"handler_rejected","message":"class=sema code=unsupported_find_subkind message=unsupported FIND subkind `unknown`"}}"#,
        },
        NegativeCaseV1 {
            label: "describe-invalid-shape",
            request: r#"{"method":"hover","query":"DESCRIBE model::"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"handler_rejected","message":"class=syntax code=describe_target_invalid_shape message=DESCRIBE target must use `<kind>::<identifier>`"}}"#,
        },
        NegativeCaseV1 {
            label: "check-missing-rule-or-scope",
            request: r#"{"method":"hover","query":"CHECK rule.hybrid"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"handler_rejected","message":"class=syntax code=check_missing_rule_or_scope message=CHECK requires `<rule_id> <scope>`"}}"#,
        },
        NegativeCaseV1 {
            label: "explain-invalid-shape",
            request: r#"{"method":"hover","query":"EXPLAIN rule::owasp.sql-injection ;; truth=maybe"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"handler_rejected","message":"class=syntax code=invalid_explain_shape message=EXPLAIN truth expectation 'maybe' is unsupported"}}"#,
        },
        NegativeCaseV1 {
            label: "require-unsupported-meta",
            request: r#"{"method":"hover","query":"REQUIRE CERTIFIED ;; truth=maybe"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"handler_rejected","message":"class=syntax code=unsupported_require_meta message=unsupported REQUIRE target `CERTIFIED ;; truth=maybe`"}}"#,
        },
        NegativeCaseV1 {
            label: "use-unsupported-meta",
            request: r#"{"method":"hover","query":"USE PROFILE strict-review"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"handler_rejected","message":"class=syntax code=unsupported_use_meta message=unsupported USE target `PROFILE`"}}"#,
        },
        NegativeCaseV1 {
            label: "nonsense-unsupported-root",
            request: r#"{"method":"hover","query":"FROBNICATE x"}"#,
            expected_line: r#"{"ok":false,"error":{"code":"handler_rejected","message":"class=syntax code=unsupported_root message=unsupported root keyword `FROBNICATE`"}}"#,
        },
    ];

    for case in cases {
        let (exit, line) = run_binary_line_v1(case.request);
        assert_eq!(exit, Some(1), "{}: must exit fail-closed", case.label);
        // Full-line equality pins ok:false + handler_rejected + the exact
        // embedded class/code/message in one mutation-proof assertion.
        assert_eq!(
            line, case.expected_line,
            "{}: full line drifted",
            case.label
        );
        // Non-degradation: a malformed DSL root must reject through the handler,
        // never collapse into a schema-level error class.
        assert!(
            !line.contains(r#""code":"method_not_allowed""#)
                && !line.contains(r#""code":"malformed_json""#)
                && !line.contains(r#""code":"unexpected_field""#),
            "{}: must not degrade to a schema-level error class, got {line}",
            case.label
        );
    }
}

/// Malformed DSL roots routed through `diagnostics` must REPORT, not fail
/// closed: exit code 0, an `ok:true` envelope, and a classified diagnostic
/// item — pinned by exact full line.
#[test]
fn stdio_diagnostics_reports_classified_dsl_items_v1() {
    let cases = [
        NegativeCaseV1 {
            label: "use-pack-invalid-lockfile",
            request: r#"{"method":"diagnostics","query":"USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.txt"}"#,
            expected_line: r#"{"ok":true,"method":"diagnostics","result":[{"class":"syntax","code":"use_pack_invalid_lockfile","message":"USE PACK lockfile `packs/secure-pack.txt` must end with `.lock` and include a non-empty file stem"}]}"#,
        },
        NegativeCaseV1 {
            label: "select-missing-from",
            request: r#"{"method":"diagnostics","query":"SELECT id catalog.users"}"#,
            expected_line: r#"{"ok":true,"method":"diagnostics","result":[{"class":"syntax","code":"expected_from_keyword","message":"SELECT projection must be followed by FROM"}]}"#,
        },
        NegativeCaseV1 {
            label: "show-unsupported-category",
            request: r#"{"method":"diagnostics","query":"SHOW catalog"}"#,
            expected_line: r#"{"ok":true,"method":"diagnostics","result":[{"class":"sema","code":"unsupported_show_category","message":"SHOW category `catalog` is unsupported; expected one of capabilities, models, views, rules, packs, profiles, gates"}]}"#,
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
        // Full-line equality: ok:true + a single, fully classified diagnostic
        // item (class + code + message). A non-empty `result` array is implied
        // by the literal item, so this can never pass on an empty report.
        assert_eq!(
            line, case.expected_line,
            "{}: full line drifted",
            case.label
        );
        // Non-degradation: diagnostics REPORTS; it must never fail-closed into
        // the handler_rejected envelope used by the hover path above.
        assert!(
            !line.contains(r#""code":"handler_rejected""#) && !line.contains(r#""ok":false"#),
            "{}: diagnostics must report, not fail closed, got {line}",
            case.label
        );
    }
}

/// Registry-driven breadth: every family's canonical negative, reported through
/// `diagnostics` at the spawned binary, must carry the registry's exact code AND
/// the display class derived from that code via the single ir authority
/// (`front_door_diagnostic_class_v1`). This makes the LSP negative hellgate
/// consume the one registry table (every front-door family, not a hand-picked
/// subset) and pins the code→class authority end-to-end at the binary.
#[test]
fn stdio_diagnostics_reports_every_registry_family_negative_v1() {
    for spec in root_surface_specs_v1() {
        let request = format!(
            r#"{{"method":"diagnostics","query":"{}"}}"#,
            spec.negative.source
        );
        let (exit, line) = run_binary_line_v1(&request);
        assert_eq!(
            exit,
            Some(0),
            "{:?}: diagnostics reports (exit 0), got {line}",
            spec.family
        );
        assert!(
            line.contains(r#""ok":true"#),
            "{:?}: diagnostics must report ok:true, got {line}",
            spec.family
        );
        assert!(
            line.contains(&format!(r#""code":"{}""#, spec.negative.code)),
            "{:?}: missing registry code `{}` in {line}",
            spec.family,
            spec.negative.code
        );
        let class = front_door_diagnostic_class_v1(spec.negative.code).as_str_v1();
        assert!(
            line.contains(&format!(r#""class":"{class}""#)),
            "{:?}: missing derived class `{class}` in {line}",
            spec.family
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

    // First line: the EXACT fail-closed envelope for the unsupported root.
    let fail_line = lines[0];
    assert_eq!(
        fail_line,
        r#"{"ok":false,"error":{"code":"handler_rejected","message":"class=syntax code=unsupported_root message=unsupported root keyword `FROBNICATE`"}}"#,
        "first line must be the exact unsupported-root rejection"
    );

    // Second line: the recovered zero_query, pinned to its deterministic
    // read-only envelope prefix so recovery cannot pass on a stub/empty line.
    let recovered_line = lines[1];
    assert!(
        recovered_line.starts_with(
            r#"{"ok":true,"method":"zero_query","result":{"mode":"read-only","provenance":"typed-catalog-example-metadata","groups":[{"id":"catalog","#
        ),
        "second line must recover the read-only zero_query envelope, got {recovered_line}"
    );
    assert!(
        recovered_line.ends_with("]}}"),
        "recovered zero_query must close groups array + result + envelope, got {recovered_line}"
    );

    assert_eq!(
        output.status.code(),
        Some(1),
        "session that observed a fail-closed frame must exit code 1"
    );
}
