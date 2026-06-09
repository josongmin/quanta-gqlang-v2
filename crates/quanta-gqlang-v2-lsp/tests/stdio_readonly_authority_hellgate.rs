#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]
//! LSP read-only authority hellgate (quickfix metadata-only + zero-query
//! read-only authority).
//!
//! These cases drive the real spawned stdio binary and lock two authority
//! invariants at the front door:
//!
//! 1. `quickfix` is metadata-only: every suggestion advertises its read-only
//!    authority and can never carry a mutating edit, command, or workspace edit.
//! 2. `zero_query` is a read-only scaffold authority: example/catalog metadata
//!    provenance only, every suggestion is a read-only scaffold, and the
//!    response makes no runtime-truth / result-count claim.

mod support;

use support::run_binary_line_v1;

/// A `quickfix` probe and the suggestion `code` we expect it to surface.
///
/// Each input is chosen because it actually yields a suggestion (verified
/// against the real binary): typed-target probes plus declaration/SELECT roots
/// that fall through to a starter scaffold.
struct QuickfixCaseV1 {
    input: &'static str,
    expected_code: &'static str,
}

const QUICKFIX_CASES_V1: &[QuickfixCaseV1] = &[
    QuickfixCaseV1 {
        input: "DESCRIBE model::billing_warehouse",
        expected_code: "missing-model-scaffold",
    },
    QuickfixCaseV1 {
        input: "DESCRIBE profile::strict-review",
        expected_code: "missing-profile-scaffold",
    },
    QuickfixCaseV1 {
        input: "VIEW catalog.users",
        expected_code: "missing-query-scaffold",
    },
    QuickfixCaseV1 {
        input: "SELECT",
        expected_code: "missing-query-scaffold",
    },
];

/// Substrings that would betray a mutating quickfix leaking through the
/// read-only front door. None may ever appear in any response line.
const QUICKFIX_FORBIDDEN_SUBSTRINGS_V1: &[&str] = &[
    r#""edits_document":true"#,
    r#""apply_command":""#,
    r#""workspace_edit":""#,
];

#[test]
fn quickfix_suggestions_stay_metadata_only_v1() {
    for case in QUICKFIX_CASES_V1 {
        let request = format!(r#"{{"method":"quickfix","input":"{}"}}"#, case.input);
        let (exit, line) = run_binary_line_v1(&request);

        assert_eq!(
            exit,
            Some(0),
            "quickfix `{}` should exit 0, got {exit:?}: {line}",
            case.input
        );
        assert!(
            line.contains(r#""ok":true"#),
            "quickfix `{}` should report ok:true: {line}",
            case.input
        );
        assert!(
            line.contains(r#""method":"quickfix""#),
            "quickfix `{}` should echo its method: {line}",
            case.input
        );
        assert!(
            line.contains(&format!(r#""code":"{}""#, case.expected_code)),
            "quickfix `{}` should surface code `{}`: {line}",
            case.input,
            case.expected_code
        );

        // Metadata-only invariant: the suggestion advertises its read-only
        // authority and pins every mutation channel to a null / false value.
        assert!(
            line.contains(r#""edits_document":false"#),
            "quickfix `{}` must mark itself non-editing: {line}",
            case.input
        );
        assert!(
            line.contains(r#""apply_command":null"#),
            "quickfix `{}` must carry a null apply_command: {line}",
            case.input
        );
        assert!(
            line.contains(r#""workspace_edit":null"#),
            "quickfix `{}` must carry a null workspace_edit: {line}",
            case.input
        );
        assert!(
            line.contains(r#""authority":""#),
            "quickfix `{}` must declare a read-only authority: {line}",
            case.input
        );

        // A quickfix can never carry a mutating edit through the read-only
        // door: the mutation channels are never non-null / true.
        for forbidden in QUICKFIX_FORBIDDEN_SUBSTRINGS_V1 {
            assert!(
                !line.contains(forbidden),
                "quickfix `{}` must never contain `{forbidden}`: {line}",
                case.input
            );
        }

        // The scaffold is the metadata payload itself and must be present and
        // non-empty (an empty `"scaffold":""` would be a degenerate fix).
        assert!(
            line.contains(r#""scaffold":""#),
            "quickfix `{}` must include a scaffold field: {line}",
            case.input
        );
        assert!(
            !line.contains(r#""scaffold":"""#),
            "quickfix `{}` scaffold must be non-empty: {line}",
            case.input
        );
    }
}

/// Substrings that would prove `zero_query` claims a mutating mode or a
/// command-plane / workspace-edit capability. None may appear.
const ZERO_QUERY_FORBIDDEN_AUTHORITY_V1: &[&str] = &[
    r#""mode":"read-write""#,
    r#""mode":"mutating""#,
    r#""behavior":"mutating"#,
    r#""behavior":"read-write"#,
    "RENAME",
    "applyEdit",
    "executeCommand",
    "workspace/",
];

/// Runtime result-count keys. A read-only scaffold authority describes shapes
/// only and must never assert how many rows / matches a query would yield.
const ZERO_QUERY_FORBIDDEN_RESULT_COUNTS_V1: &[&str] = &[
    r#""result_count""#,
    r#""row_count""#,
    r#""matches":"#,
    r#""count":"#,
];

#[test]
fn zero_query_is_read_only_scaffold_authority_v1() {
    let (exit, line) = run_binary_line_v1(r#"{"method":"zero_query"}"#);

    assert_eq!(
        exit,
        Some(0),
        "zero_query should exit 0, got {exit:?}: {line}"
    );
    assert!(
        line.contains(r#""ok":true"#),
        "zero_query should report ok:true: {line}"
    );
    assert!(
        line.contains(r#""method":"zero_query""#),
        "zero_query should echo its method: {line}"
    );

    // Read-only mode authority: the result declares read-only and never a
    // mutating mode.
    assert!(
        line.contains(r#""mode":"read-only""#),
        "zero_query result must declare read-only mode: {line}"
    );

    // Example/catalog provenance, not runtime: the suggestions come from typed
    // catalog metadata, so they cannot assert host-side truth.
    assert!(
        line.contains(r#""provenance":"typed-catalog-example-metadata""#),
        "zero_query result must declare typed-catalog-example provenance: {line}"
    );

    // Every suggestion is a read-only scaffold.
    assert!(
        line.contains(r#""behavior":"read-only-scaffold""#),
        "zero_query suggestions must be read-only scaffolds: {line}"
    );

    // No mutating-mode or command-plane / workspace-edit leak.
    for forbidden in ZERO_QUERY_FORBIDDEN_AUTHORITY_V1 {
        assert!(
            !line.contains(forbidden),
            "zero_query must never contain mutating authority `{forbidden}`: {line}"
        );
    }

    // No runtime result-count claim: the scaffold authority pins "count
    // 비주장" (it never asserts how many results a query would produce).
    for forbidden in ZERO_QUERY_FORBIDDEN_RESULT_COUNTS_V1 {
        assert!(
            !line.contains(forbidden),
            "zero_query must never claim a runtime result count `{forbidden}`: {line}"
        );
    }
}
