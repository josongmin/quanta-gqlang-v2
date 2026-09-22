#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Mutation-kill rails for the two surviving REAL mutants the prior pass missed:
//! the `<=` / `<>` / `!=` two-byte-operator match arms in `lex_symbol`
//! (lexer.rs:248/250/251) and `project_span` (query.rs:266). These were caught
//! only as survivors by a confirmation cargo-mutants re-run; here we pin them.

use quanta_gqlang_v2_hir::HirBuilderV1;
use quanta_gqlang_v2_surface::{lex, parse_program, SurfaceItemV1, TokenKindV1};

/// Each two-byte comparison operator must lex as ONE `Op(sym)` token (then Eof),
/// not split into two single-byte tokens. Deleting the `b"<="` / `b"<>"` /
/// `b"!="` match arms (lexer.rs:248/250/251) makes the operator fall through to
/// the single-byte path, producing a different token stream — which these
/// assertions detect.
#[test]
fn two_byte_comparison_operators_lex_as_single_op_token() {
    for sym in ["<=", ">=", "<>", "!="] {
        let toks = lex(sym).expect("operator lexes");
        assert_eq!(
            toks.len(),
            2,
            "`{sym}` must lex as exactly one operator token plus Eof, got {toks:?}"
        );
        match &toks[0].kind {
            TokenKindV1::Op(got) => {
                assert_eq!(got, sym, "`{sym}` must lex as Op(\"{sym}\")");
            }
            other => panic!("`{sym}` expected Op(\"{sym}\"), got {other:?}"),
        }
        assert!(
            matches!(toks[1].kind, TokenKindV1::Eof),
            "`{sym}` must be a single two-byte op followed by Eof, got {toks:?}"
        );
    }
}

/// A projection's relation node must carry the real source span covering the
/// `project ...` clause, not the default empty `0..0`. `project_span`
/// (query.rs:266) computes it by merging the input span with each item span;
/// replacing it with `Default::default()` collapses the span to `0..0`, which
/// this rail rejects.
#[test]
fn projection_relation_span_covers_the_clause_not_default() {
    let builder = HirBuilderV1::new();
    let src = "from t | project a as x, b as y";
    let item = parse_program(src, &builder).expect("query parses");
    let query = match item.item {
        SurfaceItemV1::Query(q) => q,
        other => panic!("expected a Query item, got {other:?}"),
    };
    let span = query.body.span;
    assert!(
        span.start < span.end,
        "projection span must be a real non-empty range, got {span:?}"
    );
    assert!(
        span.end >= src.len() - 1,
        "projection span must extend to the end of the `project` clause \
         (src len {}), got {span:?}",
        src.len()
    );
}
