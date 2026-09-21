#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{
    check_tree, BinaryExprV1, BoolExprKindV1, CompareExprV1, HirBuilderV1, HirRootRefV1,
    ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_surface::{parse_expr, parse_predicate};

fn binary(v: &ValueExprHirV1) -> &BinaryExprV1 {
    match &v.kind {
        ValueExprKindV1::Binary(b) => b,
        other => panic!("expected binary value, got {other:?}"),
    }
}

fn symbol_name(v: &ValueExprHirV1) -> &str {
    match &v.kind {
        ValueExprKindV1::Symbol(s) => s.name.as_str(),
        other => panic!("expected symbol, got {other:?}"),
    }
}

/// `a + b * c > d and e` must nest as `(((a + (b*c)) > d) and e)`:
/// `*` binds tighter than `+`, comparison tighter than `and`.
#[test]
fn precedence_nests_mul_over_add_compare_over_and() {
    let src = "a + b * c > d and e";
    let builder = HirBuilderV1::new();
    let pred = parse_predicate(src, &builder).expect("parses");

    // top: And(lhs = compare(...), rhs = value(e))
    let and = match &pred.kind {
        BoolExprKindV1::And(b) => b,
        other => panic!("top should be And, got {other:?}"),
    };
    // rhs of and is bare value `e`
    match &and.rhs.kind {
        BoolExprKindV1::Value(v) => assert_eq!(symbol_name(v), "e"),
        other => panic!("and.rhs should be value e, got {other:?}"),
    }
    // lhs of and is the comparison `(a + b*c) > d`
    let cmp: &CompareExprV1 = match &and.lhs.kind {
        BoolExprKindV1::Compare(c) => c,
        other => panic!("and.lhs should be Compare, got {other:?}"),
    };
    assert_eq!(cmp.op.symbol, ">");
    assert_eq!(symbol_name(&cmp.rhs), "d");

    // cmp.lhs is `a + (b * c)`: outer is `+`, its rhs is `b * c`.
    let add = binary(&cmp.lhs);
    assert_eq!(add.op.symbol, "+");
    assert_eq!(symbol_name(&add.lhs), "a");
    let mul = binary(&add.rhs);
    assert_eq!(mul.op.symbol, "*");
    assert_eq!(symbol_name(&mul.lhs), "b");
    assert_eq!(symbol_name(&mul.rhs), "c");

    // whole tree is integrity-sound against the source bounds.
    check_tree(HirRootRefV1::Bool(&pred), src.len()).expect("sound tree");
}

/// Parens override precedence and are preserved as explicit nodes with exact
/// spans covering the parentheses.
#[test]
fn parens_override_precedence_with_exact_span() {
    let src = "(a + b) * c";
    let builder = HirBuilderV1::new();
    let value = parse_expr(src, &builder).expect("parses");

    let mul = binary(&value);
    assert_eq!(mul.op.symbol, "*");
    assert_eq!(symbol_name(&mul.rhs), "c");
    // lhs is a Paren node spanning "(a + b)" exactly: bytes 0..7.
    match &mul.lhs.kind {
        ValueExprKindV1::Paren(inner) => {
            assert_eq!(mul.lhs.span.start, 0);
            assert_eq!(mul.lhs.span.end, 7);
            let add = binary(inner);
            assert_eq!(add.op.symbol, "+");
        }
        other => panic!("expected paren, got {other:?}"),
    }
    check_tree(HirRootRefV1::Value(&value), src.len()).expect("sound tree");
}

/// `not` binds tighter than `and`/`or`; `or` is the loosest connective.
#[test]
fn not_and_or_precedence() {
    let src = "not a and b or c";
    let builder = HirBuilderV1::new();
    let pred = parse_predicate(src, &builder).expect("parses");
    // top is Or(And(Not(a), b), c)
    let or = match &pred.kind {
        BoolExprKindV1::Or(b) => b,
        other => panic!("top should be Or, got {other:?}"),
    };
    let and = match &or.lhs.kind {
        BoolExprKindV1::And(b) => b,
        other => panic!("or.lhs should be And, got {other:?}"),
    };
    assert!(matches!(and.lhs.kind, BoolExprKindV1::Not(_)));
}

/// A path access `u.profile.name` lowers to one Path node; segment spans are
/// exact.
#[test]
fn path_access_segments_have_exact_spans() {
    let src = "u.profile.name";
    let builder = HirBuilderV1::new();
    let value = parse_expr(src, &builder).expect("parses");
    match &value.kind {
        ValueExprKindV1::Path(p) => {
            assert_eq!(p.segments.len(), 2);
            // "profile" occupies bytes 2..9, "name" bytes 10..14.
            assert_eq!(p.segments[0].name, "profile");
            assert_eq!(p.segments[0].span.start, 2);
            assert_eq!(p.segments[0].span.end, 9);
            assert_eq!(p.segments[1].name, "name");
            assert_eq!(p.segments[1].span.start, 10);
            assert_eq!(p.segments[1].span.end, 14);
        }
        other => panic!("expected path, got {other:?}"),
    }
    check_tree(HirRootRefV1::Value(&value), src.len()).expect("sound tree");
}
