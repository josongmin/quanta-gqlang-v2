#![expect(
    clippy::expect_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{HirBuilderV1, SpanV1};
use quanta_gqlang_v2_surface::{parse_expr, parse_program, ParseErrorCodeV1};

fn err_program(src: &str) -> quanta_gqlang_v2_surface::ParseDiagnosticV1 {
    let builder = HirBuilderV1::new();
    parse_program(src, &builder).expect_err("must reject")
}

/// A v2 uppercase root keyword is rejected EXPLICITLY rather than reparsed into
/// a v2 subset. This is the no-silent-downgrade guard.
#[test]
fn rejects_v2_select_isms_explicitly() {
    let src = "SELECT repo FROM findings";
    let diag = err_program(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.code_str(), "surface.parse.unsupported_construct");
    // span points exactly at the offending `SELECT` token: bytes 0..6.
    assert_eq!(diag.span.start, 0);
    assert_eq!(diag.span.end, 6);
    // and the message names the v3 alternative, proving it is a deliberate
    // reject and not a lenient narrowing.
    assert!(diag.message.contains("v3"), "message: {}", diag.message);
}

/// A malformed query (missing closing paren) rejects with a typed parse
/// diagnostic carrying an exact span — it does not get truncated to a smaller
/// valid query.
#[test]
fn malformed_query_unclosed_paren_rejects_with_span() {
    let src = "from users | where (a + 1";
    let diag = err_program(src);
    // inside the paren we parse `a + 1`, then expect `)` but hit EOF.
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedEof);
    assert_eq!(diag.code_str(), "surface.parse.unexpected_eof");
    // span is the zero-width EOF point at end of source.
    assert_eq!(diag.span.start, src.len());
    assert_eq!(diag.span.end, src.len());
}

/// An unknown pipeline-stage keyword rejects explicitly; the parser does not
/// fall back to dropping the unrecognized stage.
#[test]
fn unknown_pipeline_stage_rejects() {
    let src = "from users | frobnicate x";
    let diag = err_program(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(diag.code_str(), "surface.parse.unexpected_token");
    // `frobnicate` starts at byte 13 (after "from users | ").
    assert_eq!(diag.span.start, 13);
}

/// Trailing tokens after a complete item reject; the parser does not silently
/// stop at the first valid item and ignore the rest.
#[test]
fn trailing_tokens_reject() {
    let src = "let x = 1 garbage";
    let diag = err_program(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    // `garbage` starts at byte 10.
    assert_eq!(diag.span.start, 10);
}

/// An illegal byte rejects at the lexical layer with an exact 1-byte span.
#[test]
fn unexpected_char_rejects_with_span() {
    let src = "let x = a ^ b";
    let diag = err_program(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedChar);
    assert_eq!(diag.code_str(), "surface.parse.unexpected_char");
    // `^` is at byte 10.
    assert_eq!(diag.span.start, 10);
    assert_eq!(diag.span.end, 11);
}

/// An unterminated string rejects with a span covering the open quote to EOF.
#[test]
fn unterminated_string_rejects() {
    let src = "let s = \"oops";
    let diag = err_program(src);
    assert_eq!(diag.code, ParseErrorCodeV1::UnterminatedString);
    // open quote at byte 8.
    assert_eq!(diag.span.start, 8);
    assert_eq!(diag.span.end, src.len());
}

/// A bare expression entry point also fails closed on trailing tokens.
#[test]
fn parse_expr_rejects_trailing() {
    let builder = HirBuilderV1::new();
    let diag = parse_expr("a + b )", &builder).expect_err("must reject");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(diag.message, "unexpected trailing tokens after the expression");
    assert_eq!(diag.span, SpanV1::new(6, 7));
}

/// D10 fail-closed: a record literal must not declare the same field name twice.
/// A silent last-wins would drop a field, so the parser rejects at the offending
/// (second) field name's exact span.
#[test]
fn record_literal_duplicate_field_rejects() {
    let builder = HirBuilderV1::new();
    let src = "{ a: 1, a: 2 }";
    let diag = parse_expr(src, &builder).expect_err("duplicate record field rejects");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.code_str(), "surface.parse.unsupported_construct");
    // second `a` is at byte 8 in `{ a: 1, a: 2 }`.
    assert_eq!(diag.span.start, 8);
    assert_eq!(diag.span.end, 9);
    assert!(
        diag.message.contains("duplicate"),
        "message: {}",
        diag.message
    );
}
