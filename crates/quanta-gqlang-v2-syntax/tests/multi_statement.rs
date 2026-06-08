#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! Multi-statement documents are newline-separated (matching `format_document_v1`,
//! which joins roots with `\n`). The parser must:
//!   * parse each line into its own root so `roots.len()` reflects reality,
//!   * recover from errors so every malformed statement is reported at once, and
//!   * keep single-statement behaviour byte-identical.

use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::{format_document_v1, parse_document_v1};

#[test]
fn newline_separated_statements_parse_as_distinct_roots() {
    let doc = parse_document_v1("FIND path api.entry -> db.write\nSELECT id FROM catalog.users")
        .expect("two-statement document must parse");
    assert_eq!(doc.roots.len(), 2);
    assert!(matches!(doc.roots.first(), Some(AstRootV1::Find(_))));
    assert!(matches!(doc.roots.get(1), Some(AstRootV1::Select(_))));
}

#[test]
fn single_line_statement_is_still_one_root() {
    let doc =
        parse_document_v1("SELECT id FROM catalog.users").expect("single statement must parse");
    assert_eq!(doc.roots.len(), 1);
}

#[test]
fn statements_are_one_per_line_no_implicit_continuation() {
    // Statement boundaries are physical newlines (the canonical `format_document_v1`
    // emits one root per line). A wrapped statement is therefore two per-line
    // statements with honest diagnostics, never a silent merge.
    let page = parse_document_v1("SELECT id\nFROM catalog.users")
        .expect_err("a wrapped statement is two per-line statements");
    // The second line is reported as an unsupported root, not glued into line one.
    assert!(page
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "unsupported_root"));
}

#[test]
fn a_typod_root_keyword_on_its_own_line_is_diagnosed_not_swallowed() {
    // Regression: a continuation-merging splitter silently absorbed a misspelled
    // second statement into the first. Each line is now diagnosed independently.
    let page = parse_document_v1("FIND path api.entry -> db.write\nSELECTX id FROM t")
        .expect_err("the misspelled second statement must surface");
    assert!(page
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "unsupported_root"));
}

#[test]
fn a_non_keyword_second_line_is_its_own_diagnosed_statement() {
    // Regression: merging a non-keyword line onto the previous statement fabricated
    // a diagnostic on the valid first statement and dropped the bogus second one.
    let page = parse_document_v1("SHOW capabilities\nBOGUS thing")
        .expect_err("the bogus second statement must surface");
    assert!(page
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "unsupported_root"));
    assert!(
        !page
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "show_category_invalid_shape"),
        "the valid first statement must not be corrupted into a fabricated error"
    );
}

#[test]
fn a_string_literal_may_not_span_lines() {
    // statement=line means a quote opened on a line must close on the same line; a
    // newline inside a literal is an honest unterminated-literal error, not a silent
    // newline->space rewrite.
    let page = parse_document_v1("SELECT id FROM t WHERE note = 'line1\nline2'")
        .expect_err("a multi-line string literal must fail closed");
    assert!(page
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "unterminated_string_literal"));
}

#[test]
fn error_recovery_reports_every_malformed_statement() {
    let page = parse_document_v1("CHECK rule.hybrid\nDESCRIBE")
        .expect_err("both statements are malformed");
    assert_eq!(page.diagnostics.len(), 2);
    assert!(page
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "check_missing_rule_or_scope"));
    assert!(page
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == "describe_missing_target"));
}

#[test]
fn multi_root_document_round_trips_through_the_formatter() {
    let doc = parse_document_v1("SELECT id FROM catalog.users\nSHOW capabilities")
        .expect("multi-root document must parse");
    let formatted = format_document_v1(&doc);
    let reparsed = parse_document_v1(&formatted).expect("formatted multi-root output must reparse");
    assert_eq!(reparsed.roots.len(), 2);
    assert_eq!(format_document_v1(&reparsed), formatted);
}

#[test]
fn blank_lines_between_statements_are_ignored() {
    let doc = parse_document_v1("SHOW capabilities\n\n  \nSHOW models")
        .expect("blank lines must not create empty statements");
    assert_eq!(doc.roots.len(), 2);
}
