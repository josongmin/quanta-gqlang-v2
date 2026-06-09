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

#[test]
fn hover_summarizes_only_the_first_statement_v1() {
    let (code, line) = run_binary_line_v1(MULTI_SHOW_V1);
    assert_eq!(code, Some(0));
    assert!(line.contains(r#""ok":true"#), "got {line}");
    assert!(line.contains(r#""root_keyword":"SHOW""#), "got {line}");
    assert!(line.contains(r#""typed_target":"models""#), "got {line}");
    assert!(
        !line.contains("views"),
        "the first-statement card must not leak the second statement: {line}"
    );
}

#[test]
fn symbols_summarize_only_the_first_statement_v1() {
    let (code, line) =
        run_binary_line_v1(r#"{"method":"symbols","query":"SHOW models\nSHOW views"}"#);
    assert_eq!(code, Some(0));
    assert!(line.contains(r#""ok":true"#), "got {line}");
    assert!(line.contains(r#""name":"SHOW""#), "got {line}");
    assert!(line.contains(r#""name":"models""#), "got {line}");
    assert!(
        !line.contains("views"),
        "first-statement symbols must not leak the second statement: {line}"
    );
}

#[test]
fn diagnostics_span_the_whole_document_v1() {
    // A valid two-statement document is clean across both statements.
    let (clean_code, clean) =
        run_binary_line_v1(r#"{"method":"diagnostics","query":"SHOW models\nSHOW views"}"#);
    assert_eq!(clean_code, Some(0));
    assert!(clean.contains(r#""ok":true"#), "got {clean}");
    assert!(
        clean.contains(r#""result":[]"#),
        "valid 2-statement doc must be clean: {clean}"
    );

    // A malformed *second* statement is still surfaced — proving diagnostics is
    // whole-document, the complement of the first-statement card surfaces.
    let (mixed_code, mixed) =
        run_binary_line_v1(r#"{"method":"diagnostics","query":"SHOW capabilities\nSHOW bogus"}"#);
    assert_eq!(mixed_code, Some(0));
    assert!(mixed.contains(r#""ok":true"#), "got {mixed}");
    assert!(
        mixed.contains(r#""code":"unsupported_show_category""#),
        "the malformed second statement must be diagnosed: {mixed}"
    );
}
