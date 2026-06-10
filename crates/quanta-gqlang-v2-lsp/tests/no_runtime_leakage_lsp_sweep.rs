#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Cross-method regression guard: the read-only LSP front door never leaks
//! runtime truth.
//!
//! The stdio shell is frontend-only — it parses, classifies, and scaffolds, but
//! never executes a query, routes to a backend, materializes rows, or applies an
//! edit. This sweep runs a representative request for every read-only method via
//! the spawned binary and asserts that no runtime-truth token (row counts, timing,
//! provider/host plumbing, applied-edit markers, confirmed-truth flags) appears in
//! the response. Each method is also asserted to have actually produced a
//! response, so the guard can never pass vacuously.
//!
//! Positive authority pins lock the read-only posture from the other side:
//! `zero_query` carries `"mode":"read-only"` and every suggestion is a
//! `"behavior":"read-only-scaffold"`; every `quickfix` suggestion carries
//! `"apply_command":null`, `"workspace_edit":null`, and `"edits_document":false`.

mod support;

use quanta_gqlang_v2_conformance::leakage::find_runtime_truth_token_v1;
use support::run_binary_line_v1;

/// `(method label, request JSON)` — known-good inputs mirrored from
/// `stdio_read_only.rs` so this guard tracks the real surface.
const SWEEP_REQUESTS_V1: &[(&str, &str)] = &[
    (
        "hover",
        r#"{"method":"hover","query":"SELECT id FROM catalog.files"}"#,
    ),
    ("completion", r#"{"method":"completion","prefix":"pro"}"#),
    (
        "symbols",
        r#"{"method":"symbols","query":"PACK secure-pack ;; source=registry::security ;; activation=session"}"#,
    ),
    (
        "diagnostics",
        r#"{"method":"diagnostics","query":"SELECT id catalog.users"}"#,
    ),
    ("zero_query", r#"{"method":"zero_query"}"#),
    (
        "quickfix",
        r#"{"method":"quickfix","input":"DESCRIBE model::billing_warehouse"}"#,
    ),
];

fn assert_no_runtime_leakage_v1(method: &str, response: &str) {
    assert!(
        !response.trim().is_empty(),
        "method `{method}` produced no response — guard would be vacuous"
    );
    assert!(
        response.contains(r#""ok":true"#),
        "method `{method}` must produce a successful read-only response: {response}"
    );
    assert!(
        response.contains(&format!(r#""method":"{method}""#)),
        "method `{method}` response must echo its method: {response}"
    );
    // Single shared, boundary-aware authority (same list + matcher the CLI sweep
    // uses) so the two sweeps can never drift and an honest identifier that
    // merely contains a token as a substring does not false-positive.
    if let Some(token) = find_runtime_truth_token_v1(response) {
        panic!("method `{method}` leaked runtime-truth token `{token}`: {response}");
    }
}

#[test]
fn every_read_only_method_omits_runtime_truth_tokens_v1() {
    for (method, request) in SWEEP_REQUESTS_V1 {
        let (status, response) = run_binary_line_v1(request);
        assert_eq!(
            status,
            Some(0),
            "method `{method}` must exit clean on a read-only request: {response}"
        );
        assert_no_runtime_leakage_v1(method, &response);
    }
}

#[test]
fn zero_query_pins_read_only_authority_v1() {
    let (status, response) = run_binary_line_v1(r#"{"method":"zero_query"}"#);
    assert_eq!(status, Some(0));
    assert!(!response.trim().is_empty(), "zero_query must respond");
    assert!(
        response.contains(r#""mode":"read-only""#),
        "zero_query must declare read-only mode: {response}"
    );

    // Every suggestion carries the read-only-scaffold behavior, and that is the
    // ONLY behavior value present — no scaffold may claim execution authority.
    let behaviors = response.matches(r#""behavior":""#).count();
    let read_only_scaffolds = response
        .matches(r#""behavior":"read-only-scaffold""#)
        .count();
    assert!(
        behaviors > 0,
        "zero_query must emit suggestions: {response}"
    );
    assert_eq!(
        behaviors, read_only_scaffolds,
        "every zero_query suggestion behavior must be read-only-scaffold: {response}"
    );
}

#[test]
fn quickfix_pins_non_applying_read_only_suggestions_v1() {
    let (status, response) =
        run_binary_line_v1(r#"{"method":"quickfix","input":"DESCRIBE model::billing_warehouse"}"#);
    assert_eq!(status, Some(0));
    assert!(!response.trim().is_empty(), "quickfix must respond");

    // One suggestion is emitted for this input; its non-applying contract must
    // hold across every suggestion, so the marker counts must agree with the
    // suggestion count (here exactly one).
    let suggestions = response.matches(r#""code":""#).count();
    assert!(
        suggestions > 0,
        "quickfix must emit a suggestion: {response}"
    );
    assert_eq!(
        response.matches(r#""apply_command":null"#).count(),
        suggestions,
        "every quickfix suggestion must carry apply_command:null: {response}"
    );
    assert_eq!(
        response.matches(r#""workspace_edit":null"#).count(),
        suggestions,
        "every quickfix suggestion must carry workspace_edit:null: {response}"
    );
    assert_eq!(
        response.matches(r#""edits_document":false"#).count(),
        suggestions,
        "every quickfix suggestion must carry edits_document:false: {response}"
    );
}
