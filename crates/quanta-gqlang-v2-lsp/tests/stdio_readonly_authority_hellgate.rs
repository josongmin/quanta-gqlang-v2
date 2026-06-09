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
    /// The EXACT, full trimmed response line. A quickfix result is a short,
    /// fully deterministic single-suggestion array, so pinning the whole line
    /// (captured from the real binary) locks the suggestion `code`, the
    /// `edits_document:false` flag, the read-only `authority` value, and the
    /// adjacent `apply_command:null,workspace_edit:null` mutation channels all at
    /// once — far stronger than the scatter of `contains` it replaces.
    expected_line: &'static str,
}

const QUICKFIX_CASES_V1: &[QuickfixCaseV1] = &[
    QuickfixCaseV1 {
        input: "DESCRIBE model::billing_warehouse",
        expected_code: "missing-model-scaffold",
        expected_line: r#"{"ok":true,"method":"quickfix","result":[{"code":"missing-model-scaffold","label":"Declare MODEL billing_warehouse","detail":"provider declaration","scaffold":"MODEL billing_warehouse ;; provider=provider.default ;; source=repo::billing_warehouse","edits_document":false,"authority":"external-read-only-suggestion","apply_command":null,"workspace_edit":null}]}"#,
    },
    QuickfixCaseV1 {
        input: "DESCRIBE profile::strict-review",
        expected_code: "missing-profile-scaffold",
        expected_line: r#"{"ok":true,"method":"quickfix","result":[{"code":"missing-profile-scaffold","label":"Declare PROFILE strict-review","detail":"profile declaration","scaffold":"PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only","edits_document":false,"authority":"external-read-only-suggestion","apply_command":null,"workspace_edit":null}]}"#,
    },
    QuickfixCaseV1 {
        input: "VIEW catalog.users",
        expected_code: "missing-query-scaffold",
        expected_line: r#"{"ok":true,"method":"quickfix","result":[{"code":"missing-query-scaffold","label":"Add VIEW query scaffold for catalog.users","detail":"named query declaration","scaffold":"VIEW catalog.users ;; body=SELECT id FROM catalog.users LIMIT 25","edits_document":false,"authority":"external-read-only-suggestion","apply_command":null,"workspace_edit":null}]}"#,
    },
    QuickfixCaseV1 {
        input: "SELECT",
        expected_code: "missing-query-scaffold",
        expected_line: r#"{"ok":true,"method":"quickfix","result":[{"code":"missing-query-scaffold","label":"Add SELECT starter","detail":"Read-only shell needs a deterministic SELECT starter with no runtime truth claims.","scaffold":"SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 7","edits_document":false,"authority":"external-read-only-suggestion","apply_command":null,"workspace_edit":null}]}"#,
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

        // Strongest pin: the entire deterministic suggestion line. This locks
        // the method echo, the suggestion `code`, the non-editing flag, the
        // read-only authority VALUE, and the null mutation channels at once.
        assert_eq!(
            line, case.expected_line,
            "quickfix `{}` drifted",
            case.input
        );

        // Keep the explicit structural invariants so a refactor of the constant
        // table cannot silently weaken the contract these tests defend.
        assert!(
            line.contains(&format!(r#""code":"{}""#, case.expected_code)),
            "quickfix `{}` should surface code `{}`: {line}",
            case.input,
            case.expected_code
        );

        // Metadata-only invariant: the suggestion advertises its read-only
        // authority by VALUE (not just the field key) and pins every mutation
        // channel to a null / false value, with apply/workspace adjacent.
        assert!(
            line.contains(r#""edits_document":false"#),
            "quickfix `{}` must mark itself non-editing: {line}",
            case.input
        );
        assert!(
            line.contains(r#""authority":"external-read-only-suggestion""#),
            "quickfix `{}` must declare the read-only authority value, not just the key: {line}",
            case.input
        );
        assert!(
            line.contains(r#""apply_command":null,"workspace_edit":null"#),
            "quickfix `{}` must pin apply_command and workspace_edit null and adjacent: {line}",
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

        // Exactly one suggestion is present (non-vacuous): the result is a
        // single-element array, never an empty `[]` that would pass vacuously.
        assert!(
            line.contains(r#""result":[{"#) && line.contains("}]}"),
            "quickfix `{}` must carry exactly one non-empty suggestion: {line}",
            case.input
        );

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

    // Envelope pin: the read-only mode and example-metadata provenance appear in
    // a fixed order immediately after the result object opens, and the response
    // closes all three nested objects. Anchoring the prefix/suffix kills a
    // mutation that reorders/relabels the authority header or truncates the body.
    assert!(
        line.starts_with(
            r#"{"ok":true,"method":"zero_query","result":{"mode":"read-only","provenance":"typed-catalog-example-metadata","groups":[{"id":"catalog","#
        ),
        "zero_query authority envelope/prefix drifted: {line}"
    );
    assert!(
        line.ends_with("]}}"),
        "zero_query response must close groups array + result + envelope: {line}"
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

    // Non-vacuous positive content: the three known groups are present and the
    // body actually carries suggestions — this can never pass on an empty
    // `groups:[]`.
    for group_id in [
        r#""id":"catalog""#,
        r#""id":"analysis""#,
        r#""id":"retrieval""#,
    ] {
        assert!(
            line.contains(group_id),
            "zero_query must include the {group_id} group: {line}"
        );
    }

    // Every suggestion is a read-only scaffold, and EVERY suggestion (one
    // `query_scaffold` each) carries that behavior — equal counts prove no
    // suggestion silently advertises a different (e.g. mutating) behavior.
    let scaffold_behaviors = line.matches(r#""behavior":"read-only-scaffold""#).count();
    let suggestion_count = line.matches(r#""query_scaffold":"#).count();
    assert!(
        suggestion_count >= 1,
        "zero_query must carry at least one suggestion: {line}"
    );
    assert_eq!(
        scaffold_behaviors, suggestion_count,
        "every zero_query suggestion must be a read-only scaffold: {line}"
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
