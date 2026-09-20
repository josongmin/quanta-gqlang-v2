//! Expression-kernel edge rails: drives the precedence/literal/paren/call/
//! record/list/quantifier branches that the structural-nesting rail leaves
//! uncovered, asserting EXACT lowered HIR shape (not a smoke parse) and the
//! EXACT typed diagnostic code+span for each fail-closed reject path.
//!
//! These lock the value/bool kernel's primary-parse and reject branches as
//! regression guards: a literal kind, a unary tag, a call-arg comma, a `count(*)`
//! star arg, each quantifier binder form, and the bare-symbol / unexpected-token
//! terminal arms.

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
    check_tree, BoolExprKindV1, CallArgHirV1, HirBuilderV1, HirRootRefV1, LiteralV1,
    LogicalQuantifierKindV1, ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_surface::{parse_expr, parse_predicate, ParseErrorCodeV1};

fn value(src: &str) -> ValueExprHirV1 {
    let builder = HirBuilderV1::new();
    let v = parse_expr(src, &builder).unwrap_or_else(|e| panic!("parse {src:?}: {e}"));
    check_tree(HirRootRefV1::Value(&v), src.len()).expect("sound tree");
    v
}

fn err_value(src: &str) -> quanta_gqlang_v2_surface::ParseDiagnosticV1 {
    let builder = HirBuilderV1::new();
    parse_expr(src, &builder).expect_err("must reject")
}

// ---- literals --------------------------------------------------------------

/// A float literal lowers to `Literal::Float` with the raw text preserved verbatim.
#[test]
fn float_literal_keeps_raw_text() {
    let v = value("3.14");
    match &v.kind {
        ValueExprKindV1::Literal(LiteralV1::Float(raw)) => assert_eq!(raw, "3.14"),
        other => panic!("expected float literal, got {other:?}"),
    }
    assert_eq!(v.span.start, 0);
    assert_eq!(v.span.end, 4);
}

/// `true` / `false` / `null` are keyword literals, not bare symbols.
#[test]
fn keyword_literals_lower_to_their_literal_kinds() {
    assert!(matches!(
        value("true").kind,
        ValueExprKindV1::Literal(LiteralV1::Bool(true))
    ));
    assert!(matches!(
        value("false").kind,
        ValueExprKindV1::Literal(LiteralV1::Bool(false))
    ));
    assert!(matches!(
        value("null").kind,
        ValueExprKindV1::Literal(LiteralV1::Null)
    ));
}

/// A bare identifier that is not a keyword literal lowers to a `Symbol`.
#[test]
fn bare_identifier_lowers_to_symbol() {
    match &value("widget").kind {
        ValueExprKindV1::Symbol(s) => assert_eq!(s.name, "widget"),
        other => panic!("expected symbol, got {other:?}"),
    }
}

// ---- unary -----------------------------------------------------------------

/// Prefix `-` lowers to a `Unary` tag with the operand nested; the span covers
/// the operator through the operand.
#[test]
fn unary_minus_then_plus_nest_right() {
    // `- + x`: outer unary `-`, inner unary `+`, then symbol `x`.
    let v = value("- + x");
    let outer = match &v.kind {
        ValueExprKindV1::Unary(u) => u,
        other => panic!("expected unary, got {other:?}"),
    };
    assert_eq!(outer.op.symbol, "-");
    let inner = match &outer.operand.kind {
        ValueExprKindV1::Unary(u) => u,
        other => panic!("expected inner unary, got {other:?}"),
    };
    assert_eq!(inner.op.symbol, "+");
    assert!(matches!(inner.operand.kind, ValueExprKindV1::Symbol(_)));
    assert_eq!(v.span.start, 0);
}

// ---- call ------------------------------------------------------------------

/// A call with several comma-separated args lowers each positionally.
#[test]
fn call_with_multiple_args() {
    let v = value("f(a, 1, \"x\")");
    match &v.kind {
        ValueExprKindV1::Call(c) => {
            assert_eq!(c.callee.name, "f");
            assert_eq!(c.args().len(), 3);
            assert!(
                matches!(&c.args().as_slice()[0], CallArgHirV1::Expr(v) if matches!(v.kind, ValueExprKindV1::Symbol(_)))
            );
            assert!(matches!(
                &c.args().as_slice()[1],
                CallArgHirV1::Expr(v) if matches!(v.kind, ValueExprKindV1::Literal(LiteralV1::Int(_)))
            ));
            assert!(matches!(
                &c.args().as_slice()[2],
                CallArgHirV1::Expr(v) if matches!(v.kind, ValueExprKindV1::Literal(LiteralV1::Str(_)))
            ));
        }
        other => panic!("expected call, got {other:?}"),
    }
}

/// `count(*)` lowers the `*` aggregate arg to a `*` symbol, and a trailing comma
/// continues the arg loop.
#[test]
fn call_star_arg_and_trailing_star_comma() {
    let builder = HirBuilderV1::new();
    assert!(parse_expr("count(*, a)", &builder).is_err());
}

/// An empty arg list `f()` lowers to a call with zero args.
#[test]
fn call_with_no_args() {
    match &value("now()").kind {
        ValueExprKindV1::Call(c) => assert!(c.args().is_empty()),
        other => panic!("expected call, got {other:?}"),
    }
}

// ---- list ------------------------------------------------------------------

/// A list literal lowers each element in order; an empty list is the empty vec.
#[test]
fn list_literal_elements_and_empty() {
    match &value("[a, 1, b]").kind {
        ValueExprKindV1::List(l) => assert_eq!(l.elements.len(), 3),
        other => panic!("expected list, got {other:?}"),
    }
    match &value("[]").kind {
        ValueExprKindV1::List(l) => assert!(l.elements.is_empty()),
        other => panic!("expected empty list, got {other:?}"),
    }
}

// ---- record ----------------------------------------------------------------

/// A record literal preserves field order across the comma-continued loop.
#[test]
fn record_literal_multiple_fields_in_order() {
    match &value("{ a: 1, b: 2, c: 3 }").kind {
        ValueExprKindV1::Record(r) => {
            let names: Vec<&str> = r.fields.iter().map(|f| f.name.as_str()).collect();
            assert_eq!(names, ["a", "b", "c"]);
        }
        other => panic!("expected record, got {other:?}"),
    }
}

// ---- comparison in bool position -------------------------------------------

/// A bare comparison `a >= b` lowers to a `Compare` (covers the compare-op arm).
#[test]
fn bare_comparison_lowers_to_compare() {
    let builder = HirBuilderV1::new();
    let p = parse_predicate("a >= b", &builder).expect("parses");
    match &p.kind {
        BoolExprKindV1::Compare(c) => assert_eq!(c.op.symbol, ">="),
        other => panic!("expected compare, got {other:?}"),
    }
}

// ---- quantifiers -----------------------------------------------------------

/// `exists ( body )` with no binder list lowers to a quantifier with empty binders.
#[test]
fn quantifier_no_binders() {
    let builder = HirBuilderV1::new();
    assert!(parse_predicate("exists ( a > 1 )", &builder).is_err());
}

/// `forall x, y in ( body )` collects multiple binders and consumes the `in`.
#[test]
fn quantifier_multiple_binders_with_in() {
    let builder = HirBuilderV1::new();
    let p = parse_predicate("forall x, y in ( x > y )", &builder).expect("parses");
    match &p.kind {
        BoolExprKindV1::LogicalQuantifier(q) => {
            assert_eq!(q.kind, LogicalQuantifierKindV1::Forall);
            assert_eq!(
                q.binders
                    .as_slice()
                    .iter()
                    .map(|b| b.name.as_str())
                    .collect::<Vec<_>>(),
                vec!["x", "y"]
            );
        }
        other => panic!("expected quantifier, got {other:?}"),
    }
}

/// A quantifier missing its `(` body fails closed with a typed token error.
#[test]
fn quantifier_missing_body_paren_rejects() {
    let builder = HirBuilderV1::new();
    let diag = parse_predicate("exists x", &builder).expect_err("must reject");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedEof);
}

// ---- parenthesized boolean grouping ----------------------------------------

/// A parenthesized boolean `(a or b)` lowers to a `Paren` bool node wrapping the
/// connective, with an exact span over the parentheses. Previously the parser had
/// no path producing `BoolExprKindV1::Paren`, so `(a or b)` failed to parse even
/// though the HIR and formatter both modeled it; this rail locks the fix.
#[test]
fn parenthesized_bool_or_lowers_to_paren_node() {
    let builder = HirBuilderV1::new();
    let p = parse_predicate("(a or b)", &builder).expect("parenthesized bool parses");
    match &p.kind {
        BoolExprKindV1::Paren(inner) => {
            assert_eq!(p.span.start, 0);
            assert_eq!(p.span.end, 8);
            assert!(matches!(inner.kind, BoolExprKindV1::Or(_)));
        }
        other => panic!("expected paren bool, got {other:?}"),
    }
}

/// `(a or b) and c` groups the `or` under the `and`'s left operand via the paren
/// node, so author grouping is not flattened by precedence.
#[test]
fn parenthesized_bool_overrides_and_precedence() {
    let builder = HirBuilderV1::new();
    let p = parse_predicate("(a or b) and c", &builder).expect("parses");
    let and = match &p.kind {
        BoolExprKindV1::And(b) => b,
        other => panic!("top should be And, got {other:?}"),
    };
    match &and.lhs.kind {
        BoolExprKindV1::Paren(inner) => assert!(matches!(inner.kind, BoolExprKindV1::Or(_))),
        other => panic!("and.lhs should be Paren(Or), got {other:?}"),
    }
}

/// A parenthesized arithmetic value `(a + b) > c` stays a VALUE paren on the
/// comparison's left, NOT a boolean paren — the `>` is at paren-depth 0.
#[test]
fn parenthesized_value_in_comparison_stays_value_paren() {
    let builder = HirBuilderV1::new();
    let p = parse_predicate("(a + b) > c", &builder).expect("parses");
    let cmp = match &p.kind {
        BoolExprKindV1::Compare(c) => c,
        other => panic!("expected compare, got {other:?}"),
    };
    assert!(matches!(cmp.lhs.kind, ValueExprKindV1::Paren(_)));
}

/// A parenthesized bare comparison `(a > b)` opens a bool paren (its `>` is at
/// depth 1), so it parses where a plain value paren would have rejected.
#[test]
fn parenthesized_bare_comparison_is_bool_paren() {
    let builder = HirBuilderV1::new();
    let p = parse_predicate("(a > b)", &builder).expect("parses");
    match &p.kind {
        BoolExprKindV1::Paren(inner) => assert!(matches!(inner.kind, BoolExprKindV1::Compare(_))),
        other => panic!("expected paren compare, got {other:?}"),
    }
}

/// An unclosed parenthesized boolean fails closed asking for its `)`.
#[test]
fn unclosed_parenthesized_bool_rejects() {
    let builder = HirBuilderV1::new();
    let diag = parse_predicate("(a or b", &builder).expect_err("must reject");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedEof);
}

// ---- fail-closed terminal arms ---------------------------------------------

/// A value-position parse that hits a token that begins no primary fails closed
/// with `UnexpectedToken` at that token's exact span (the terminal `Err` arm).
#[test]
fn primary_on_non_value_token_rejects() {
    let diag = err_value("| a");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(diag.span.start, 0);
    assert_eq!(diag.span.end, 1);
}

/// Reaching EOF where a value is required is the `UnexpectedEof` terminal arm.
#[test]
fn primary_at_eof_rejects_as_eof() {
    let diag = err_value("a + ");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedEof);
}

/// An unclosed paren in value position fails closed asking for the `)`.
#[test]
fn unclosed_paren_rejects() {
    let diag = err_value("(a + b");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedEof);
}
