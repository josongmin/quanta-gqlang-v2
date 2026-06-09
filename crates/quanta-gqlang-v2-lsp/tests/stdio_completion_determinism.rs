#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Completion as a deterministic, namespace-sensitive surface at the binary.
//!
//! `handlers::completion_rank::rank_matches_v1` sorts matches by a total key
//! (match-reason, scope, category, label length, label), so the serialized line
//! is fully deterministic. These tests pin the exact response line for a
//! declaration-namespace prefix (`pro`) and a root-keyword prefix (`SE`), prove
//! run-to-run stability across separate process spawns, and pin the genuine
//! case behavior of the matcher.
//!
//! Discovered behavior (pinned, not invented): matching is case-INSENSITIVE —
//! `normalized_v1` lowercases both the fragment and the label — so `SE` and `se`
//! produce byte-identical responses, and emitted labels are the catalog's
//! canonical casing (`SELECT`, not the user's `se`). The namespace distinction
//! is real and pinned: `pro` yields a declaration root (`PROFILE`), a
//! show-category tail (`profiles`), and a describe-target prefix (`profile::`).

mod support;

use support::run_binary_line_v1;

/// The exact, full completion line for the declaration-namespace prefix `pro`.
/// Ordering is locked by the rank key, so pinning the whole line kills any
/// reorder, field-value, or namespace-collapse mutation in one assertion.
const COMPLETION_PRO_LINE_V1: &str = r#"{"ok":true,"method":"completion","result":[{"label":"PROFILE","kind":"declaration-root","detail":"profile declaration","reason":"Declaration completion stays public and frontend-owned.","scope":"document-root","match_reason":"exact-scope","example_query":"PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only"},{"label":"profiles","kind":"show-category","detail":"SHOW profiles","reason":"SHOW tail completion must be served from deterministic plural catalog nouns.","scope":"show-category-tail","match_reason":"root-fallback","example_query":"PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only"},{"label":"profile::","kind":"describe-kind","detail":"DESCRIBE profile::<path>","reason":"DESCRIBE tail completion must seed a typed target prefix only.","scope":"describe-target-tail","match_reason":"root-fallback","example_query":"PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only"}]}"#;

/// The exact, full completion line for the root-keyword prefix `SE`.
const COMPLETION_SE_LINE_V1: &str = r#"{"ok":true,"method":"completion","result":[{"label":"SELECT","kind":"root","detail":"table-only query root","reason":"Root keyword completion must stay frontend-deterministic.","scope":"document-root","match_reason":"exact-scope","example_query":"SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 7"}]}"#;

fn completion_line_v1(prefix: &str) -> String {
    let request = format!(r#"{{"method":"completion","prefix":"{prefix}"}}"#);
    let (status, line) = run_binary_line_v1(&request);
    assert_eq!(
        status,
        Some(0),
        "completion `{prefix}` must succeed: {line}"
    );
    line
}

// ------------------------------------------------------------------------
// 1. Exact ordered list — full-line pins.
// ------------------------------------------------------------------------

#[test]
fn completion_pro_returns_exact_ordered_list_v1() {
    assert_eq!(completion_line_v1("pro"), COMPLETION_PRO_LINE_V1);
}

#[test]
fn completion_root_keyword_prefix_returns_exact_line_v1() {
    assert_eq!(completion_line_v1("SE"), COMPLETION_SE_LINE_V1);
}

// ------------------------------------------------------------------------
// 2. Run-twice determinism — two separate process spawns.
// ------------------------------------------------------------------------

#[test]
fn completion_is_byte_identical_across_separate_spawns_v1() {
    let first = completion_line_v1("pro");
    let second = completion_line_v1("pro");
    assert_eq!(
        first, second,
        "two independent completion spawns must be byte-identical"
    );
    assert_eq!(first, COMPLETION_PRO_LINE_V1);
}

// ------------------------------------------------------------------------
// 3. Case sensitivity — the genuine, discovered behavior.
// ------------------------------------------------------------------------

#[test]
fn completion_root_keyword_matching_is_case_insensitive_v1() {
    // The matcher normalizes via ASCII-lowercasing both sides, so `SE` and `se`
    // resolve to the same catalog rows and serialize identically. The emitted
    // label is the catalog's canonical uppercase `SELECT`, never the user's case.
    let upper = completion_line_v1("SE");
    let lower = completion_line_v1("se");
    assert_eq!(
        upper, lower,
        "case-insensitive matching must make `SE` and `se` byte-identical"
    );
    assert_eq!(upper, COMPLETION_SE_LINE_V1);
    assert!(
        lower.contains(r#""label":"SELECT""#),
        "emitted label stays canonical uppercase regardless of input case: {lower}"
    );
    assert!(
        !lower.contains(r#""label":"select""#),
        "lowercase input must not echo a lowercased label: {lower}"
    );
}

// ------------------------------------------------------------------------
// 4. Namespace sensitivity — distinct forms the handler emits for `pro`.
// ------------------------------------------------------------------------

#[test]
fn completion_pro_emits_distinct_namespace_forms_v1() {
    // The full-line pin in test #1 already locks ordering; here the three distinct
    // namespace forms a single `pro` fragment fans out into are asserted by name:
    // a declaration root, a SHOW-category tail, and a DESCRIBE target prefix.
    let line = completion_line_v1("pro");
    assert!(
        line.contains(r#""label":"PROFILE""#),
        "declaration-root form missing: {line}"
    );
    assert!(
        line.contains(r#""label":"profiles""#),
        "show-category tail form missing: {line}"
    );
    assert!(
        line.contains(r#""label":"profile::""#),
        "describe-target prefix form missing: {line}"
    );
    // The forms are distinct scopes, not duplicates of one row.
    assert!(line.contains(r#""scope":"document-root""#));
    assert!(line.contains(r#""scope":"show-category-tail""#));
    assert!(line.contains(r#""scope":"describe-target-tail""#));
}
