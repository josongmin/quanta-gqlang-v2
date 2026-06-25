//! Precedence-aware expression parsing producing shared HIR value/bool nodes.
//!
//! syntax -> hir: this is the precedence-climbing kernel that lowers v3 surface
//! expressions into `hir::ValueExprHirV1` / `hir::BoolExprHirV1` via
//! `hir::HirBuilderV1`. It owns SHAPE only: operator/comparison/quantifier
//! symbols become open HIR tags; binding, overload choice, and type legality are
//! NOT decided here (D13, owned by tickets 02/03).
//!
//! Precedence (tightest first):
//!   1. primary / paren / call / path / unary `-` `+`
//!   2. `*` `/`        (value, left-assoc)
//!   3. `+` `-`        (value, left-assoc)
//!   4. comparison `= <> != < <= > >=`  (value x value -> bool)
//!   5. `not`          (bool prefix)
//!   6. `and`          (bool, left-assoc)
//!   7. `or`           (bool, left-assoc)
//!
//! So `a + b * c > d and e` nests as `(((a + (b*c)) > d) and e)` — the rail-1
//! shape — because `*` binds tighter than `+`, comparison tighter than `and`.

use crate::cursor::CursorV1;
use crate::diag::ParseDiagnosticV1;
use crate::lexer::TokenKindV1;
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, PathSegmentV1, ProjectionItemV1, RecordFieldV1, SpanV1, ValueExprHirV1,
};

/// Parse a full boolean expression (the predicate grammar entry point).
///
/// Boolean position is the outer grammar: a bare value expression with no
/// comparison/connective lowers to `BoolExprKind::Value`, so `where x` and
/// `where x > 1` share one entry point.
pub(crate) fn parse_bool(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    parse_or(cur)
}

/// Parse a full value expression (entry point for decl bodies, args, etc.).
pub(crate) fn parse_value(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    parse_additive(cur)
}

// ---- boolean precedence levels --------------------------------------------

fn parse_or(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    let mut lhs = parse_and(cur)?;
    while cur.is_keyword("or") {
        let _ = cur.bump();
        let rhs = parse_and(cur)?;
        let span = lhs.span.merge(&rhs.span);
        lhs = cur.builder.or(lhs, rhs, span);
    }
    Ok(lhs)
}

fn parse_and(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    let mut lhs = parse_not(cur)?;
    while cur.is_keyword("and") {
        let _ = cur.bump();
        let rhs = parse_not(cur)?;
        let span = lhs.span.merge(&rhs.span);
        lhs = cur.builder.and(lhs, rhs, span);
    }
    Ok(lhs)
}

fn parse_not(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    if cur.is_keyword("not") {
        let kw = cur.bump();
        let inner = parse_not(cur)?;
        let span = kw.span.merge(&inner.span);
        return Ok(cur.builder.not(inner, span));
    }
    parse_comparison(cur)
}

fn parse_comparison(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    // quantifier / parenthesized-bool / value-or-compare
    if cur.is_keyword("exists") || cur.is_keyword("forall") {
        return parse_quantifier(cur);
    }

    // Parenthesized BOOLEAN grouping: `(a or b)`, `(not a)`, etc. A `(` here only
    // opens a bool group when its top-level (depth-1) content carries a boolean
    // connective the value grammar cannot hold (`and`/`or`/`not`). A plain
    // `(a + b)` or a comparison-bearing `(a + b) > c` stays a VALUE paren so its
    // author grouping is preserved as a value node and any following compare op
    // still binds — so this branch is taken only for the genuinely-boolean case.
    if matches!(cur.peek().kind, TokenKindV1::LParen) && paren_opens_bool(cur) {
        let open = cur.bump();
        let inner = parse_bool(cur)?;
        let close = cur.expect(
            &TokenKindV1::RParen,
            "')' to close parenthesized boolean expression",
        )?;
        let span = open.span.merge(&close.span);
        return Ok(cur.builder.paren_bool(inner, span));
    }

    let lhs = parse_additive(cur)?;
    if let TokenKindV1::Op(sym) = &cur.peek().kind {
        if is_compare_op(sym) {
            let sym = sym.clone();
            let _ = cur.bump();
            let rhs = parse_additive(cur)?;
            let span = lhs.span.merge(&rhs.span);
            return Ok(cur.builder.compare(sym, lhs, rhs, span));
        }
    }
    // bare value in boolean position
    let span = lhs.span;
    Ok(cur.builder.bool_value(lhs, span))
}

fn parse_quantifier(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    let kw_tok = cur.bump();
    // Callers gate on `is_keyword`, so this is always an Ident; fall back to an
    // empty keyword rather than a wildcard match to stay fail-safe.
    let keyword = if let TokenKindV1::Ident(s) = &kw_tok.kind {
        s.clone()
    } else {
        String::new()
    };
    // optional binder list: exists x, y in ( body )  /  exists ( body )
    let mut binders = Vec::new();
    if matches!(cur.peek().kind, TokenKindV1::Ident(_)) && !cur.is_keyword("in") {
        loop {
            let (name, _span) = cur.expect_ident("quantifier binder name")?;
            binders.push(name);
            if cur.eat(&TokenKindV1::Comma).is_some() {
                continue;
            }
            break;
        }
        // optional `in`
        let _ = cur.eat_keyword("in");
    }
    cur.expect(&TokenKindV1::LParen, "'(' to open quantifier body")?;
    let body = parse_bool(cur)?;
    let close = cur.expect(&TokenKindV1::RParen, "')' to close quantifier body")?;
    let span = kw_tok.span.merge(&close.span);
    Ok(cur.builder.quantifier(keyword, binders, body, span))
}

fn is_compare_op(sym: &str) -> bool {
    matches!(sym, "=" | "<>" | "!=" | "<" | "<=" | ">" | ">=")
}

/// Whether the `(` at the cursor opens a BOOLEAN group rather than a value paren.
///
/// Scans forward to the matching `)` and reports `true` iff, at paren-depth 1
/// (i.e. directly inside this paren, not within a nested paren), it sees a token
/// only the boolean grammar can hold: an `and`/`or`/`not`/`exists`/`forall`
/// keyword or a comparison operator. A purely arithmetic `(a + b)` has none of
/// those at depth 1, so it stays a value paren; `(a + b) > c` keeps its `>` at
/// depth 0, so it is also not a bool paren here. Fail-safe: an unbalanced run
/// (no matching `)` before EOF) reports `false`, leaving the value path to emit
/// the precise unclosed-paren diagnostic.
fn paren_opens_bool(cur: &CursorV1<'_>) -> bool {
    // cursor is on the opening `(`; start scanning the token after it.
    let mut depth: usize = 1;
    let mut n = 1usize;
    loop {
        let kind = &cur.peek_ahead(n).kind;
        if matches!(kind, TokenKindV1::Eof) {
            return false;
        }
        if matches!(kind, TokenKindV1::LParen) {
            depth += 1;
        } else if matches!(kind, TokenKindV1::RParen) {
            depth -= 1;
            if depth == 0 {
                return false;
            }
        } else if depth == 1 {
            // A boolean-only token directly inside this paren marks a bool group.
            if let TokenKindV1::Ident(s) = kind {
                if matches!(s.as_str(), "and" | "or" | "not" | "exists" | "forall") {
                    return true;
                }
            }
            if let TokenKindV1::Op(sym) = kind {
                if is_compare_op(sym) {
                    return true;
                }
            }
        }
        n += 1;
    }
}

// ---- value precedence levels ----------------------------------------------

fn parse_additive(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    let mut lhs = parse_multiplicative(cur)?;
    while let Some(sym) = op_here(cur, &["+", "-"]) {
        let _ = cur.bump();
        let rhs = parse_multiplicative(cur)?;
        let span = lhs.span.merge(&rhs.span);
        lhs = cur.builder.binary(sym, lhs, rhs, span);
    }
    Ok(lhs)
}

fn parse_multiplicative(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    let mut lhs = parse_unary(cur)?;
    while let Some(sym) = op_here(cur, &["*", "/"]) {
        let _ = cur.bump();
        let rhs = parse_unary(cur)?;
        let span = lhs.span.merge(&rhs.span);
        lhs = cur.builder.binary(sym, lhs, rhs, span);
    }
    Ok(lhs)
}

fn parse_unary(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    if let TokenKindV1::Op(sym) = &cur.peek().kind {
        if sym == "-" || sym == "+" {
            let sym = sym.clone();
            let op_tok = cur.bump();
            let operand = parse_unary(cur)?;
            let span = op_tok.span.merge(&operand.span);
            return Ok(cur.builder.unary(sym, operand, span));
        }
    }
    parse_postfix(cur)
}

/// Primary followed by zero or more `.field` path segments.
fn parse_postfix(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    let base = parse_primary(cur)?;
    if !matches!(cur.peek().kind, TokenKindV1::Dot) {
        return Ok(base);
    }
    let mut segments = Vec::new();
    while cur.eat(&TokenKindV1::Dot).is_some() {
        let (name, span) = cur.expect_ident("path segment name")?;
        segments.push(PathSegmentV1 { name, span });
    }
    let last = segments.last().map(|s| s.span).unwrap_or(base.span);
    let span = base.span.merge(&last);
    Ok(cur.builder.path(base, segments, span))
}

fn parse_primary(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    // Numeric/string literals carry their raw text in the token; clone it before
    // bumping so the value lowers verbatim.
    if let TokenKindV1::Int(raw) = &cur.peek().kind {
        let raw = raw.clone();
        let tok = cur.bump();
        return Ok(cur.builder.int(raw, tok.span));
    }
    if let TokenKindV1::Float(raw) = &cur.peek().kind {
        let raw = raw.clone();
        let tok = cur.bump();
        return Ok(cur.builder.float(raw, tok.span));
    }
    if let TokenKindV1::Str(content) = &cur.peek().kind {
        let content = content.clone();
        let tok = cur.bump();
        return Ok(cur.builder.string(content, tok.span));
    }
    if matches!(cur.peek().kind, TokenKindV1::LParen) {
        let open = cur.bump();
        let inner = parse_value(cur)?;
        let close = cur.expect(
            &TokenKindV1::RParen,
            "')' to close parenthesized expression",
        )?;
        let span = open.span.merge(&close.span);
        return Ok(cur.builder.paren_value(inner, span));
    }
    if matches!(cur.peek().kind, TokenKindV1::LBrace) {
        return parse_record(cur);
    }
    if matches!(cur.peek().kind, TokenKindV1::LBracket) {
        return parse_list(cur);
    }
    if let TokenKindV1::Ident(s) = &cur.peek().kind {
        let name = s.clone();
        // keyword literals first
        if name == "true" {
            let tok = cur.bump();
            return Ok(cur.builder.bool_lit(true, tok.span));
        }
        if name == "false" {
            let tok = cur.bump();
            return Ok(cur.builder.bool_lit(false, tok.span));
        }
        if name == "null" {
            let tok = cur.bump();
            return Ok(cur.builder.null(tok.span));
        }
        // call vs bare symbol
        let name_tok = cur.bump();
        if matches!(cur.peek().kind, TokenKindV1::LParen) {
            return parse_call(cur, name, name_tok.span);
        }
        return Ok(cur.builder.symbol(name, name_tok.span));
    }
    Err(cur.unexpected("a value expression"))
}

fn parse_call(
    cur: &mut CursorV1<'_>,
    callee: String,
    callee_span: SpanV1,
) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    let _open = cur.expect(&TokenKindV1::LParen, "'(' to open call arguments")?;
    let mut args = Vec::new();
    if !matches!(cur.peek().kind, TokenKindV1::RParen) {
        loop {
            // `count(*)` aggregate-style argument lowers to a `*` symbol.
            if let TokenKindV1::Op(sym) = &cur.peek().kind {
                if sym == "*" {
                    let star = cur.bump();
                    args.push(cur.builder.symbol("*", star.span));
                    if cur.eat(&TokenKindV1::Comma).is_some() {
                        continue;
                    }
                    break;
                }
            }
            let arg = parse_value(cur)?;
            args.push(arg);
            if cur.eat(&TokenKindV1::Comma).is_some() {
                continue;
            }
            break;
        }
    }
    let close = cur.expect(&TokenKindV1::RParen, "')' to close call arguments")?;
    let span = callee_span.merge(&close.span);
    Ok(cur.builder.call(callee, callee_span, args, span))
}

fn parse_record(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    let open = cur.expect(&TokenKindV1::LBrace, "'{' to open record")?;
    let mut fields: Vec<RecordFieldV1> = Vec::new();
    if !matches!(cur.peek().kind, TokenKindV1::RBrace) {
        loop {
            let (name, name_span) = cur.expect_ident("record field name")?;
            // D10 fail-closed: a record literal must not declare the same field
            // name twice (silent last-wins would drop a field). Reject at the
            // offending name's exact span, mirroring the composition binding and
            // clause-key duplicate guards.
            if fields.iter().any(|f| f.name == name) {
                return Err(cur.unsupported(name_span, format!("duplicate record field '{name}'")));
            }
            cur.expect(&TokenKindV1::Colon, "':' after record field name")?;
            let value = parse_value(cur)?;
            fields.push(RecordFieldV1 {
                name,
                name_span,
                value,
            });
            if cur.eat(&TokenKindV1::Comma).is_some() {
                continue;
            }
            break;
        }
    }
    let close = cur.expect(&TokenKindV1::RBrace, "'}' to close record")?;
    let span = open.span.merge(&close.span);
    Ok(cur.builder.record(fields, span))
}

fn parse_list(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    let open = cur.expect(&TokenKindV1::LBracket, "'[' to open list")?;
    let mut elements = Vec::new();
    if !matches!(cur.peek().kind, TokenKindV1::RBracket) {
        loop {
            let elem = parse_value(cur)?;
            elements.push(elem);
            if cur.eat(&TokenKindV1::Comma).is_some() {
                continue;
            }
            break;
        }
    }
    let close = cur.expect(&TokenKindV1::RBracket, "']' to close list")?;
    let span = open.span.merge(&close.span);
    Ok(cur.builder.list(elements, span))
}

/// Return the current operator symbol if it is one of `wanted`, else `None`.
/// Avoids a wildcard match over the full token enum at each precedence level.
fn op_here(cur: &CursorV1<'_>, wanted: &[&str]) -> Option<String> {
    if let TokenKindV1::Op(sym) = &cur.peek().kind {
        if wanted.contains(&sym.as_str()) {
            return Some(sym.clone());
        }
    }
    None
}

/// Parse one projection item: `<value> [as <alias>]`.
///
/// Shared between query `project` stages and the top-level `select` surface so
/// derived columns lower through one path.
pub(crate) fn parse_projection_item(
    cur: &mut CursorV1<'_>,
) -> Result<ProjectionItemV1, ParseDiagnosticV1> {
    let value = parse_value(cur)?;
    let alias = parse_optional_alias(cur)?;
    Ok(ProjectionItemV1 { value, alias })
}

/// Parse an optional `as <ident>` alias, returning the alias node if present.
pub(crate) fn parse_optional_alias(
    cur: &mut CursorV1<'_>,
) -> Result<Option<quanta_gqlang_v2_hir::AliasV1>, ParseDiagnosticV1> {
    if cur.eat_keyword("as").is_some() {
        let (name, span) = cur.expect_ident("alias name after 'as'")?;
        return Ok(Some(quanta_gqlang_v2_hir::AliasV1 { name, span }));
    }
    Ok(None)
}
