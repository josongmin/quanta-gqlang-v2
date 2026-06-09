#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Hellgate: LSP first-statement contract at the binary front door.
//!
//! The parser is multi-statement, but the single-card read-only surfaces
//! (`hover`, `symbols`) summarize only the **first** statement of a document,
//! while `diagnostics` spans the **whole** document. The library rail proves
//! this in-process (`read_only_lsp.rs`); here it is locked through the real
//! spawned stdio front door so the contract cannot silently drift between the
//! handler and the transport.

mod support;

use support::run_binary_line_v1;

const MULTI_SHOW_V1: &str = r#"{"method":"hover","query":"SHOW models\nSHOW views"}"#;

/// EXACT first-statement hover card for `SHOW models` (followed by `SHOW views`
/// that must NOT leak). Captured from the real binary; the serializer is
/// deterministic, so full-line equality pins root/category/target/summary at
/// once and kills any "summarize the wrong statement" or field mutation.
const FIRST_STATEMENT_HOVER_LINE_V1: &str = r#"{"ok":true,"method":"hover","result":{"root_keyword":"SHOW","category":"interactive","typed_target":"models","summary":["catalog_category=models"]}}"#;

/// EXACT first-statement symbol set for the same document.
const FIRST_STATEMENT_SYMBOLS_LINE_V1: &str = r#"{"ok":true,"method":"symbols","result":[{"name":"document","kind":"document","detail":"roots=1"},{"name":"SHOW","kind":"root","detail":"category=interactive"},{"name":"models","kind":"target","detail":"interactive.target"}]}"#;

#[test]
fn hover_summarizes_only_the_first_statement_v1() {
    let (code, line) = run_binary_line_v1(MULTI_SHOW_V1);
    assert_eq!(code, Some(0));
    // Pin WHAT it is (the first statement, `models`) with a full-line assert ...
    assert_eq!(
        line, FIRST_STATEMENT_HOVER_LINE_V1,
        "first-statement card drifted"
    );
    // ... and keep the explicit negative space: the second statement's distinct
    // token (`views`) must never appear anywhere in the card.
    assert!(
        !line.contains("views"),
        "the first-statement card must not leak the second statement: {line}"
    );
    assert!(
        !line.contains("SHOW views"),
        "the second statement must not leak verbatim: {line}"
    );
}

#[test]
fn symbols_summarize_only_the_first_statement_v1() {
    let (code, line) =
        run_binary_line_v1(r#"{"method":"symbols","query":"SHOW models\nSHOW views"}"#);
    assert_eq!(code, Some(0));
    assert_eq!(
        line, FIRST_STATEMENT_SYMBOLS_LINE_V1,
        "first-statement symbol set drifted"
    );
    // The document symbol must report exactly one root (`roots=1`); if the
    // second statement leaked it would read `roots=2`.
    assert!(
        line.contains(r#""detail":"roots=1""#),
        "document symbol must report exactly one root: {line}"
    );
    assert!(
        !line.contains("views"),
        "first-statement symbols must not leak the second statement: {line}"
    );
}

#[test]
fn diagnostics_span_the_whole_document_v1() {
    // A valid two-statement document is clean across both statements: the
    // diagnostics array is exactly empty (full-line equality forbids a stray
    // item or a reshaped envelope).
    let (clean_code, clean) =
        run_binary_line_v1(r#"{"method":"diagnostics","query":"SHOW models\nSHOW views"}"#);
    assert_eq!(clean_code, Some(0));
    assert_eq!(
        clean, r#"{"ok":true,"method":"diagnostics","result":[]}"#,
        "valid 2-statement doc must be exactly clean"
    );

    // A malformed *second* statement is still surfaced — proving diagnostics is
    // whole-document, the complement of the first-statement card surfaces. The
    // full line pins the class, code, and message; the diagnosed token is
    // `bogus` (the SECOND statement), proving the scan reached statement two.
    let (mixed_code, mixed) =
        run_binary_line_v1(r#"{"method":"diagnostics","query":"SHOW capabilities\nSHOW bogus"}"#);
    assert_eq!(mixed_code, Some(0));
    assert_eq!(
        mixed,
        r#"{"ok":true,"method":"diagnostics","result":[{"class":"sema","code":"unsupported_show_category","message":"SHOW category `bogus` is unsupported; expected one of capabilities, models, views, rules, packs, profiles, gates"}]}"#,
        "the malformed second statement must be diagnosed by exact item"
    );
    // The clean FIRST statement (`capabilities`) must NOT itself be flagged:
    // the diagnostic names `bogus`, not `capabilities`.
    assert!(
        mixed.contains("`bogus`") && !mixed.contains("category `capabilities`"),
        "only the malformed second statement may be diagnosed: {mixed}"
    );
}
