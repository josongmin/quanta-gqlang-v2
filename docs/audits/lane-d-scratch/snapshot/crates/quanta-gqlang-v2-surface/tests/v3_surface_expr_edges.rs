#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{
    check_tree, BoolExprKindV1, CallArgHirV1, HirBuilderV1, HirRootRefV1, LiteralV1,
    LogicalQuantifierKindV1, SpanV1, ValueExprHirV1, ValueExprKindV1,
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

/// The symbol name of a value node; panics on any non-symbol so payload checks
/// below cannot silently accept a fabricated node.
fn symbol_name(v: &ValueExprHirV1) -> &str {
    match &v.kind {
        ValueExprKindV1::Symbol(s) => s.name.as_str(),
        other => panic!("expected symbol, got {other:?}"),
    }
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
    // The operand payload is the real oracle, not only its variant.
    assert_eq!(symbol_name(&inner.operand), "x");
    assert_eq!(v.span.start, 0);
}

// ---- call ------------------------------------------------------------------

/// A call with several comma-separated args lowers each positionally, with the
/// authored payload retained at each index.
#[test]
fn call_with_multiple_args() {
    let v = value("f(a, 1, \"x\")");
    match &v.kind {
        ValueExprKindV1::Call(c) => {
            assert_eq!(c.callee.name, "f");
            assert_eq!(c.args().len(), 3);
            let CallArgHirV1::Expr(a0) = &c.args().as_slice()[0] else {
                panic!(
                    "arg 0 must be an expression, got {:?}",
                    c.args().as_slice()[0]
                );
            };
            let CallArgHirV1::Expr(a1) = &c.args().as_slice()[1] else {
                panic!(
                    "arg 1 must be an expression, got {:?}",
                    c.args().as_slice()[1]
                );
            };
            let CallArgHirV1::Expr(a2) = &c.args().as_slice()[2] else {
                panic!(
                    "arg 2 must be an expression, got {:?}",
                    c.args().as_slice()[2]
                );
            };
            assert_eq!(symbol_name(a0), "a", "arg 0 payload");
            match &a1.kind {
                ValueExprKindV1::Literal(LiteralV1::Int(raw)) => assert_eq!(raw, "1", "arg 1 raw"),
                other => panic!("arg 1 must be int 1, got {other:?}"),
            }
            match &a2.kind {
                ValueExprKindV1::Literal(LiteralV1::Str(s)) => assert_eq!(s, "x", "arg 2 content"),
                other => panic!("arg 2 must be string x, got {other:?}"),
            }
        }
        other => panic!("expected call, got {other:?}"),
    }
}

/// `*` is a valid sole call argument, but a star followed by another argument
/// must fail closed at the star's exact span rather than be silently lowered as
/// an aggregate plus a positional argument.
#[test]
fn call_star_arg_must_be_the_sole_argument() {
    let builder = HirBuilderV1::new();
    let diag = parse_expr("count(*, a)", &builder).expect_err("star may not be followed by args");
    assert_eq!(diag.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(diag.message, "'*' must be the sole call argument");
    assert_eq!(diag.span, SpanV1::new(6, 7));
}

/// An empty arg list `f()` lowers to a call with zero args and the authored callee.
#[test]
fn call_with_no_args() {
    match &value("now()").kind {
        ValueExprKindV1::Call(c) => {
            assert_eq!(c.callee.name, "now");
            assert!(c.args().is_empty());
        }
        other => panic!("expected call, got {other:?}"),
    }
}

// ---- list ------------------------------------------------------------------

/// A list literal lowers each element in order; an empty list is the empty vec.
#[test]
fn list_literal_elements_and_empty() {
    match &value("[a, 1, b]").kind {
        ValueExprKindV1::List(l) => {
            assert_eq!(l.elements.len(), 3);
            assert_eq!(symbol_name(&l.elements[0]), "a");
            match &l.elements[1].kind {
                ValueExprKindV1::Literal(LiteralV1::Int(raw)) => assert_eq!(raw, "1"),
                other => panic!("element 1 must be int 1, got {other:?}"),
            }
            assert_eq!(symbol_name(&l.elements[2]), "b");
        }
        other => panic!("expected list, got {other:?}"),
    }
    match &value("[]").kind {
        ValueExprKindV1::List(l) => assert!(l.elements.is_empty()),
        other => panic!("expected empty list, got {other:?}"),
    }
}

// ---- record ----------------------------------------------------------------

/// A record literal preserves field order and each field value across the
/// comma-continued loop.
#[test]
fn record_literal_multiple_fields_in_order() {
    match &value("{ a: 1, b: 2, c: 3 }").kind {
        ValueExprKindV1::Record(r) => {
            let names: Vec<&str> = r.fields.iter().map(|f| f.name.as_str()).collect();
            assert_eq!(names, ["a", "b", "c"]);
            let values: Vec<&str> = r
                .fields
                .iter()
                .map(|f| match &f.value.kind {
                    ValueExprKindV1::Literal(LiteralV1::Int(raw)) => raw.as_str(),
                    other => panic!("record field must be an int literal, got {other:?}"),
                })
                .collect();
            assert_eq!(values, ["1", "2", "3"]);
        }
        other => panic!("expected record, got {other:?}"),
    }
}

// ---- comparison in bool position -------------------------------------------

/// A bare comparison `a >= b` lowers to a `Compare` with the operands in
/// authored order (a swapped lhs/rhs must fail).
#[test]
fn bare_comparison_lowers_to_compare() {
    let builder = HirBuilderV1::new();
    let p = parse_predicate("a >= b", &builder).expect("parses");
    match &p.kind {
        BoolExprKindV1::Compare(c) => {
            assert_eq!(c.op.symbol, ">=");
            assert_eq!(symbol_name(&c.lhs), "a");
            assert_eq!(symbol_name(&c.rhs), "b");
        }
        other => panic!("expected compare, got {other:?}"),
    }
}

// ---- quantifiers -----------------------------------------------------------

/// A quantifier requires at least one binder before `in`; a missing binder list
/// must fail closed at the `(` with the binder expectation, not silently lower
/// to a zero-binder quantifier.
#[test]
fn quantifier_without_binders_rejects() {
    let builder = HirBuilderV1::new();
    let diag = parse_predicate("exists ( a > 1 )", &builder).expect_err("binder list is required");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(diag.message, "expected quantifier binder name");
    assert_eq!(diag.span, SpanV1::new(7, 8));
}

/// `forall x, y in ( body )` collects multiple binders, consumes the `in`, and
/// keeps the authored predicate body.
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
            let BoolExprKindV1::Compare(c) = &q.body.kind else {
                panic!(
                    "quantifier body must be a comparison, got {:?}",
                    q.body.kind
                );
            };
            assert_eq!(c.op.symbol, ">");
            assert_eq!(symbol_name(&c.lhs), "x");
            assert_eq!(symbol_name(&c.rhs), "y");
        }
        other => panic!("expected quantifier, got {other:?}"),
    }
}

/// A quantifier missing its `in` and its `(` body each fail closed with the
/// exact typed expectation (the two arms are distinct and must not be conflated).
#[test]
fn quantifier_missing_body_paren_rejects() {
    // Binder list with no `in`: stops before the body paren arm.
    let missing_in = parse_predicate("exists x", &HirBuilderV1::new()).expect_err("must reject");
    assert_eq!(missing_in.code, ParseErrorCodeV1::UnexpectedEof);
    assert_eq!(
        missing_in.message,
        "expected 'in' after quantifier binders but reached end of input"
    );
    assert_eq!(missing_in.span, SpanV1::new(8, 8));

    // `in` present but no `(`: the body-paren arm itself.
    let missing_paren =
        parse_predicate("exists x in x > 1", &HirBuilderV1::new()).expect_err("must reject");
    assert_eq!(missing_paren.code, ParseErrorCodeV1::UnexpectedToken);
    assert_eq!(
        missing_paren.message,
        "expected '(' to open quantifier body"
    );
    assert_eq!(missing_paren.span, SpanV1::new(12, 13));
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
            let BoolExprKindV1::Or(or) = &inner.kind else {
                panic!("paren must wrap Or, got {:?}", inner.kind);
            };
            let BoolExprKindV1::Value(lhs) = &or.lhs.kind else {
                panic!("or.lhs must be a bare value, got {:?}", or.lhs.kind);
            };
            assert_eq!(symbol_name(lhs), "a");
            let BoolExprKindV1::Value(rhs) = &or.rhs.kind else {
                panic!("or.rhs must be a bare value, got {:?}", or.rhs.kind);
            };
            assert_eq!(symbol_name(rhs), "b");
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
        BoolExprKindV1::Paren(inner) => {
            let BoolExprKindV1::Or(or) = &inner.kind else {
                panic!("and.lhs paren must wrap Or, got {:?}", inner.kind);
            };
            let BoolExprKindV1::Value(lhs) = &or.lhs.kind else {
                panic!("or.lhs must be a bare value, got {:?}", or.lhs.kind);
            };
            assert_eq!(symbol_name(lhs), "a");
            let BoolExprKindV1::Value(rhs) = &or.rhs.kind else {
                panic!("or.rhs must be a bare value, got {:?}", or.rhs.kind);
            };
            assert_eq!(symbol_name(rhs), "b");
        }
        other => panic!("and.lhs should be Paren(Or), got {other:?}"),
    }
    let BoolExprKindV1::Value(and_rhs) = &and.rhs.kind else {
        panic!("and.rhs must be a bare value, got {:?}", and.rhs.kind);
    };
    assert_eq!(symbol_name(and_rhs), "c");
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
    let ValueExprKindV1::Paren(inner) = &cmp.lhs.kind else {
        panic!("cmp.lhs must be a value paren, got {:?}", cmp.lhs.kind);
    };
    let ValueExprKindV1::Binary(add) = &inner.kind else {
        panic!("value paren must wrap the add, got {:?}", inner.kind);
    };
    assert_eq!(add.op.symbol, "+");
    assert_eq!(symbol_name(&add.lhs), "a");
    assert_eq!(symbol_name(&add.rhs), "b");
    assert_eq!(symbol_name(&cmp.rhs), "c");
}

/// A parenthesized bare comparison `(a > b)` opens a bool paren (its `>` is at
/// depth 1), so it parses where a plain value paren would have rejected.
#[test]
fn parenthesized_bare_comparison_is_bool_paren() {
    let builder = HirBuilderV1::new();
    let p = parse_predicate("(a > b)", &builder).expect("parses");
    match &p.kind {
        BoolExprKindV1::Paren(inner) => {
            let BoolExprKindV1::Compare(c) = &inner.kind else {
                panic!("paren must wrap a comparison, got {:?}", inner.kind);
            };
            assert_eq!(c.op.symbol, ">");
            assert_eq!(symbol_name(&c.lhs), "a");
            assert_eq!(symbol_name(&c.rhs), "b");
        }
        other => panic!("expected paren compare, got {other:?}"),
    }
}

/// An unclosed parenthesized boolean fails closed asking for its `)` at the EOF.
#[test]
fn unclosed_parenthesized_bool_rejects() {
    let builder = HirBuilderV1::new();
    let diag = parse_predicate("(a or b", &builder).expect_err("must reject");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedEof);
    assert_eq!(
        diag.message,
        "expected ')' to close parenthesized boolean expression but reached end of input"
    );
    assert_eq!(diag.span, SpanV1::new(7, 7));
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

/// An unclosed paren in value position fails closed asking for the `)` at EOF.
#[test]
fn unclosed_paren_rejects() {
    let diag = err_value("(a + b");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedEof);
    assert_eq!(
        diag.message,
        "expected ')' to close parenthesized expression but reached end of input"
    );
    assert_eq!(diag.span, SpanV1::new(6, 6));
}

/// A bare `!=` comparison is a valid operator, not only a lexed token.
#[test]
fn bare_not_equal_comparison_parses() {
    let p = parse_predicate("a != b", &HirBuilderV1::new()).expect("`!=` must parse");
    match &p.kind {
        BoolExprKindV1::Compare(c) => {
            assert_eq!(c.op.symbol, "!=");
            assert_eq!(symbol_name(&c.lhs), "a");
            assert_eq!(symbol_name(&c.rhs), "b");
        }
        other => panic!("expected compare, got {other:?}"),
    }
}

/// A leading underscore is a valid identifier start.
#[test]
fn leading_underscore_identifier_lowers_to_symbol() {
    match &value("_z").kind {
        ValueExprKindV1::Symbol(s) => assert_eq!(s.name, "_z"),
        other => panic!("expected symbol, got {other:?}"),
    }
}
