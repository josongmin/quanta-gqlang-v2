//! v3 rail: declaration and rule grammar embedding shared expressions.
//!
//! syntax->sema->ir path pinned: surface `let ...` / `rule ... if ...` text ->
//! `parse_program` -> `hir::DeclHirV1` / `hir::RuleHirV1`, each EMBEDDING the
//! shared value/bool expression kernel (hellgate rail 3).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    clippy::match_wildcard_for_single_variants,
    reason = "integration test rail: assertion panics and exhaustive match arms are intentional"
)]

use quanta_gqlang_v2_hir::{
    check_tree, BoolExprKindV1, HirBuilderV1, HirRootRefV1, ValueExprKindV1,
};
use quanta_gqlang_v2_surface::{parse_program, ParseErrorCodeV1, SurfaceItemV1};

#[test]
fn declaration_embeds_shared_value_expr() {
    let src = "let total = base + tax * 2";
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).expect("parses");
    let decl = match parsed.item {
        SurfaceItemV1::Decl(d) => d,
        other => panic!("expected decl, got {other:?}"),
    };
    assert_eq!(decl.name, "total");
    // name span: `total` is bytes 4..9.
    assert_eq!(decl.name_span.start, 4);
    assert_eq!(decl.name_span.end, 9);
    // body is `base + (tax * 2)` — a shared value expr, `+` at the top.
    let add = match &decl.body.kind {
        ValueExprKindV1::Binary(b) => b,
        other => panic!("body should be binary, got {other:?}"),
    };
    assert_eq!(add.op.symbol, "+");
    assert!(matches!(add.rhs.kind, ValueExprKindV1::Binary(_)));
    check_tree(HirRootRefV1::Decl(&decl), src.len()).expect("sound tree");
}

#[test]
fn rule_embeds_shared_value_head_and_bool_body() {
    let src = "rule adult : person.id if person.age >= 18 and person.active";
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).expect("parses");
    let rule = match parsed.item {
        SurfaceItemV1::Rule(r) => r,
        other => panic!("expected rule, got {other:?}"),
    };
    assert_eq!(rule.name, "adult");
    // head is a shared value path expression.
    assert!(matches!(rule.head.kind, ValueExprKindV1::Path(_)));
    // body is a shared bool expr: And(Compare(>=), Value(person.active)).
    let and = match &rule.body.kind {
        BoolExprKindV1::And(b) => b,
        other => panic!("body should be And, got {other:?}"),
    };
    match &and.lhs.kind {
        BoolExprKindV1::Compare(c) => assert_eq!(c.op.symbol, ">="),
        other => panic!("and.lhs should be Compare, got {other:?}"),
    }
    check_tree(HirRootRefV1::Rule(&rule), src.len()).expect("sound tree");
}

/// A `let` declaration missing its `=` after the name fails closed with a typed
/// token error (the `expect('=')` reject arm in `parse_decl`).
#[test]
fn decl_missing_equals_rejects() {
    let builder = HirBuilderV1::new();
    let diag = parse_program("let total 1", &builder).expect_err("must reject");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    assert!(diag.message.contains('='), "message: {}", diag.message);
}

/// A `rule` missing its `:` after the name fails closed with a typed token error.
#[test]
fn rule_missing_colon_rejects() {
    let builder = HirBuilderV1::new();
    let diag = parse_program("rule adult person.id if x", &builder).expect_err("must reject");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
}

/// A `rule` missing its `if` between head and body fails closed.
#[test]
fn rule_missing_if_rejects() {
    let builder = HirBuilderV1::new();
    let diag = parse_program("rule adult : person.id person.age", &builder).expect_err("rejects");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
}

#[test]
fn rule_body_supports_quantifier_placeholder() {
    let src = "rule has_tag : f.id if exists t in ( t.fid = f.id )";
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).expect("parses");
    let rule = match parsed.item {
        SurfaceItemV1::Rule(r) => r,
        other => panic!("expected rule, got {other:?}"),
    };
    match &rule.body.kind {
        BoolExprKindV1::Quantifier(q) => {
            assert_eq!(q.quantifier.keyword, "exists");
            assert_eq!(q.binders, vec!["t".to_string()]);
        }
        other => panic!("body should be Quantifier, got {other:?}"),
    }
    check_tree(HirRootRefV1::Rule(&rule), src.len()).expect("sound tree");
}
