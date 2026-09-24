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
use crate::diag::{ParseDiagnosticV1, ParseErrorCodeV1};
use crate::lexer::TokenKindV1;
use crate::query::parse_query;
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, CallArgHirV1, CallArgsHirV1, LogicalBinderV1, LogicalQuantifierKindV1,
    NonEmptyBindersV1, PathSegmentV1, ProjectionItemV1, RecordFieldV1, RelationExistsPolarityV1,
    SpanV1, ValueExprHirV1,
};

/// Parse a full boolean expression (the predicate grammar entry point).
///
/// Boolean position is the outer grammar: a bare value expression with no
/// comparison/connective lowers to `BoolExprKind::Value`, so `where x` and
/// `where x > 1` share one entry point.
pub(crate) fn parse_bool(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    // Fail closed against adversarially-deep nesting before recursing.
    cur.with_recursion_depth(parse_or)
}

/// Parse a full value expression (entry point for decl bodies, args, etc.).
pub(crate) fn parse_value(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    // Fail closed against adversarially-deep nesting before recursing.
    cur.with_recursion_depth(parse_additive)
}

// ---- boolean precedence levels --------------------------------------------

fn parse_or(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    let mut lhs = parse_and(cur)?;
    while cur.is_keyword("or") {
        let _ = cur.bump();
        let rhs = parse_and(cur)?;
        let span = lhs.span.merge(&rhs.span);
        cur.charge_expression_node(span)?;
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
        cur.charge_expression_node(span)?;
        lhs = cur.builder.and(lhs, rhs, span);
    }
    Ok(lhs)
}

fn parse_not(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    if cur.is_keyword("not")
        && cur.is_keyword_ahead(1, "exists")
        && matches!(cur.peek_ahead(2).kind, TokenKindV1::LParen)
        && cur.is_keyword_ahead(3, "from")
    {
        let not = cur.bump();
        return parse_relation_exists(cur, RelationExistsPolarityV1::NotExists, not.span);
    }
    if cur.is_keyword("not") {
        // `not not not ...` self-recurses without re-entering parse_bool; guard.
        return cur.with_recursion_depth(|cur| {
            let kw = cur.bump();
            let inner = parse_not(cur)?;
            let span = kw.span.merge(&inner.span);
            cur.charge_expression_node(span)?;
            Ok(cur.builder.not(inner, span))
        });
    }
    parse_comparison(cur)
}

fn parse_comparison(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    // quantifier / parenthesized-bool / value-or-compare
    if cur.is_keyword("exists")
        && matches!(cur.peek_ahead(1).kind, TokenKindV1::LParen)
        && cur.is_keyword_ahead(2, "from")
    {
        let start = cur.here_span();
        return parse_relation_exists(cur, RelationExistsPolarityV1::Exists, start);
    }
    if cur.is_keyword("exists") || cur.is_keyword("forall") {
        return parse_logical_quantifier(cur);
    }

    // Parenthesized BOOLEAN grouping: `(a or b)`, `(not a)`, `((a > b))`, etc.
    // A `(` here opens a bool group when its grouped expression carries boolean
    // syntax outside value-only containers. A plain `(a + b)` or a
    // comparison-bearing `(a + b) > c` stays a VALUE paren, preserving its
    // authored grouping while allowing the following comparison to bind.
    if matches!(cur.peek().kind, TokenKindV1::LParen) && paren_opens_bool(cur) {
        let open = cur.bump();
        let inner = parse_bool(cur)?;
        let close = cur.expect(
            &TokenKindV1::RParen,
            "')' to close parenthesized boolean expression",
        )?;
        let span = open.span.merge(&close.span);
        cur.charge_expression_node(span)?;
        return Ok(cur.builder.paren_bool(inner, span));
    }

    let lhs = parse_additive(cur)?;
    if let TokenKindV1::Op(sym) = &cur.peek().kind {
        if is_compare_op(sym) {
            let sym = sym.clone();
            let _ = cur.bump();
            let rhs = parse_additive(cur)?;
            let span = lhs.span.merge(&rhs.span);
            cur.charge_expression_node(span)?;
            return Ok(cur.builder.compare(sym, lhs, rhs, span));
        }
    }
    // bare value in boolean position
    let span = lhs.span;
    cur.charge_expression_node(span)?;
    Ok(cur.builder.bool_value(lhs, span))
}

fn parse_logical_quantifier(cur: &mut CursorV1<'_>) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    let kw_tok = cur.bump();
    // Callers gate on `is_keyword`, so this is always an Ident; fall back to an
    // empty keyword rather than a wildcard match to stay fail-safe.
    let kind = if matches!(&kw_tok.kind, TokenKindV1::Ident(s) if s == "forall") {
        LogicalQuantifierKindV1::Forall
    } else {
        LogicalQuantifierKindV1::Exists
    };
    let mut binders = Vec::new();
    loop {
        let observed = binders.len().saturating_add(1);
        cur.budget
            .charge_collection_element(observed, cur.here_span())?;
        let (name, span) = cur.expect_ident("quantifier binder name")?;
        binders.push(LogicalBinderV1 { name, span });
        if cur.eat(&TokenKindV1::Comma).is_some() {
            continue;
        }
        break;
    }
    cur.expect_keyword("in", "'in' after quantifier binders")?;
    cur.expect(&TokenKindV1::LParen, "'(' to open quantifier body")?;
    let body = parse_bool(cur)?;
    let close = cur.expect(&TokenKindV1::RParen, "')' to close quantifier body")?;
    let span = kw_tok.span.merge(&close.span);
    let binders = NonEmptyBindersV1::try_new(binders)
        .map_err(|message| cur.unsupported(kw_tok.span, message))?;
    cur.charge_expression_node(span)?;
    Ok(cur.builder.logical_quantifier(kind, binders, body, span))
}

fn parse_relation_exists(
    cur: &mut CursorV1<'_>,
    polarity: RelationExistsPolarityV1,
    start: SpanV1,
) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    cur.expect_keyword("exists", "'exists' after optional 'not'")?;
    cur.expect(&TokenKindV1::LParen, "'(' to open relational existence")?;
    let query = parse_query(cur)?;
    if query.result_shape.is_some() {
        return Err(cur.unsupported(
            query.span,
            "relational existence cannot carry a result shape",
        ));
    }
    let close = cur.expect(&TokenKindV1::RParen, "')' to close relational existence")?;
    let span = start.merge(&close.span);
    cur.charge_expression_node(span)?;
    Ok(cur.builder.relation_exists(polarity, query.body, span))
}

fn is_compare_op(sym: &str) -> bool {
    matches!(sym, "=" | "<>" | "!=" | "<" | "<=" | ">" | ">=")
}

fn can_end_bool_operand(kind: &TokenKindV1) -> bool {
    matches!(
        kind,
        TokenKindV1::Ident(_)
            | TokenKindV1::Int(_)
            | TokenKindV1::Float(_)
            | TokenKindV1::Str(_)
            | TokenKindV1::RParen
            | TokenKindV1::RBrace
            | TokenKindV1::RBracket
    )
}

fn can_start_bool_operand(kind: &TokenKindV1) -> bool {
    matches!(
        kind,
        TokenKindV1::Ident(_)
            | TokenKindV1::Int(_)
            | TokenKindV1::Float(_)
            | TokenKindV1::Str(_)
            | TokenKindV1::LParen
            | TokenKindV1::LBrace
            | TokenKindV1::LBracket
    ) || matches!(kind, TokenKindV1::Op(symbol) if matches!(symbol.as_str(), "+" | "-"))
}

fn prefix_bool_position(cur: &CursorV1<'_>, n: usize) -> bool {
    matches!(
        cur.peek_ahead(n.saturating_sub(1)).kind,
        TokenKindV1::LParen
    ) || matches!(
        &cur.peek_ahead(n.saturating_sub(1)).kind,
        TokenKindV1::Ident(word) if matches!(word.as_str(), "and" | "or")
    )
}

fn quantifier_starts_at(cur: &CursorV1<'_>, mut n: usize) -> bool {
    if !matches!(cur.peek_ahead(n).kind, TokenKindV1::Ident(_)) {
        return false;
    }
    n += 1;
    while matches!(cur.peek_ahead(n).kind, TokenKindV1::Comma) {
        n += 1;
        if !matches!(cur.peek_ahead(n).kind, TokenKindV1::Ident(_)) {
            return false;
        }
        n += 1;
    }
    cur.is_keyword_ahead(n, "in")
        && matches!(
            cur.peek_ahead(n.saturating_add(1)).kind,
            TokenKindV1::LParen
        )
}

/// Whether the `(` at the cursor opens a BOOLEAN group rather than a value paren.
///
/// Scans forward to the matching `)` and reports `true` iff the grouped
/// expression contains boolean syntax outside opaque value containers (call
/// arguments, records, and lists). Nested grouping parentheses remain visible,
/// so `((a > b))` is classified as boolean. Keyword spellings used as path
/// segments (`a.exists`) are never treated as operators.
///
/// Fail-safe: an unbalanced or mismatched delimiter reports `false`, leaving
/// the value parser to emit the precise structural diagnostic.
fn paren_opens_bool(cur: &CursorV1<'_>) -> bool {
    #[derive(Clone, Copy)]
    enum Delimiter {
        GroupParen,
        OpaqueParen,
        OpaqueBrace,
        OpaqueBracket,
    }

    let mut delimiters = vec![Delimiter::GroupParen];
    let mut opaque_depth = 0usize;
    let mut n = 1usize;
    let mut saw_boolean_syntax = false;
    loop {
        let kind = &cur.peek_ahead(n).kind;
        if matches!(kind, TokenKindV1::Eof) {
            return saw_boolean_syntax;
        }
        if opaque_depth == 0 {
            if let TokenKindV1::Ident(s) = kind {
                let follows_dot =
                    matches!(cur.peek_ahead(n.saturating_sub(1)).kind, TokenKindV1::Dot);
                if !follows_dot {
                    let previous = &cur.peek_ahead(n.saturating_sub(1)).kind;
                    let next = &cur.peek_ahead(n.saturating_add(1)).kind;
                    saw_boolean_syntax |= match s.as_str() {
                        "and" | "or" => {
                            can_end_bool_operand(previous) && can_start_bool_operand(next)
                        }
                        "not" => prefix_bool_position(cur, n) && can_start_bool_operand(next),
                        "exists" => {
                            prefix_bool_position(cur, n)
                                && ((matches!(next, TokenKindV1::LParen)
                                    && cur.is_keyword_ahead(n.saturating_add(2), "from"))
                                    || quantifier_starts_at(cur, n.saturating_add(1)))
                        }
                        "forall" => {
                            prefix_bool_position(cur, n)
                                && quantifier_starts_at(cur, n.saturating_add(1))
                        }
                        _ => false,
                    };
                }
            }
            if let TokenKindV1::Op(sym) = kind {
                if is_compare_op(sym) {
                    saw_boolean_syntax = true;
                }
            }
        }
        match kind {
            TokenKindV1::LParen => {
                let call = matches!(
                    cur.peek_ahead(n.saturating_sub(1)).kind,
                    TokenKindV1::Ident(_)
                );
                let delimiter = if call {
                    Delimiter::OpaqueParen
                } else {
                    Delimiter::GroupParen
                };
                opaque_depth += usize::from(matches!(delimiter, Delimiter::OpaqueParen));
                delimiters.push(delimiter);
            }
            TokenKindV1::LBrace => {
                delimiters.push(Delimiter::OpaqueBrace);
                opaque_depth += 1;
            }
            TokenKindV1::LBracket => {
                delimiters.push(Delimiter::OpaqueBracket);
                opaque_depth += 1;
            }
            TokenKindV1::RParen => {
                let Some(delimiter @ (Delimiter::GroupParen | Delimiter::OpaqueParen)) =
                    delimiters.pop()
                else {
                    return false;
                };
                opaque_depth -= usize::from(matches!(delimiter, Delimiter::OpaqueParen));
                if delimiters.is_empty() {
                    let followed_by_comparison = matches!(
                        &cur.peek_ahead(n.saturating_add(1)).kind,
                        TokenKindV1::Op(symbol) if is_compare_op(symbol)
                    );
                    return saw_boolean_syntax && !followed_by_comparison;
                }
            }
            TokenKindV1::RBrace => {
                if !matches!(delimiters.pop(), Some(Delimiter::OpaqueBrace)) {
                    return false;
                }
                opaque_depth -= 1;
            }
            TokenKindV1::RBracket => {
                if !matches!(delimiters.pop(), Some(Delimiter::OpaqueBracket)) {
                    return false;
                }
                opaque_depth -= 1;
            }
            TokenKindV1::Ident(_)
            | TokenKindV1::Int(_)
            | TokenKindV1::Float(_)
            | TokenKindV1::Str(_)
            | TokenKindV1::Comma
            | TokenKindV1::Dot
            | TokenKindV1::Colon
            | TokenKindV1::Semicolon
            | TokenKindV1::Pipe
            | TokenKindV1::Arrow
            | TokenKindV1::ThinArrow
            | TokenKindV1::Op(_)
            | TokenKindV1::Eof => {}
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
        cur.charge_expression_node(span)?;
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
        cur.charge_expression_node(span)?;
        lhs = cur.builder.binary(sym, lhs, rhs, span);
    }
    Ok(lhs)
}

fn parse_unary(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    if let TokenKindV1::Op(sym) = &cur.peek().kind {
        if sym == "-" || sym == "+" {
            // `- - - ...` self-recurses without re-entering parse_value; guard.
            return cur.with_recursion_depth(|cur| {
                let sym = if let TokenKindV1::Op(s) = &cur.peek().kind {
                    s.clone()
                } else {
                    String::new()
                };
                let op_tok = cur.bump();
                let operand = parse_unary(cur)?;
                let span = op_tok.span.merge(&operand.span);
                cur.charge_expression_node(span)?;
                Ok(cur.builder.unary(sym, operand, span))
            });
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
        cur.budget
            .charge_collection_element(segments.len().saturating_add(1), cur.here_span())?;
        let (name, span) = cur.expect_ident("path segment name")?;
        segments.push(PathSegmentV1 { name, span });
    }
    let last = segments.last().map(|s| s.span).unwrap_or(base.span);
    let span = base.span.merge(&last);
    cur.charge_expression_node(span)?;
    cur.builder
        .try_path(base, segments, span)
        .map_err(|message| ParseDiagnosticV1::new(ParseErrorCodeV1::UnexpectedToken, span, message))
}

fn parse_primary(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    // Numeric/string literals carry their raw text in the token; clone it before
    // bumping so the value lowers verbatim.
    if let TokenKindV1::Int(raw) = &cur.peek().kind {
        let raw = raw.clone();
        let tok = cur.bump();
        cur.charge_expression_node(tok.span)?;
        return Ok(cur.builder.int(raw, tok.span));
    }
    if let TokenKindV1::Float(raw) = &cur.peek().kind {
        let raw = raw.clone();
        let tok = cur.bump();
        cur.charge_expression_node(tok.span)?;
        return Ok(cur.builder.float(raw, tok.span));
    }
    if let TokenKindV1::Str(content) = &cur.peek().kind {
        let content = content.clone();
        let tok = cur.bump();
        cur.charge_expression_node(tok.span)?;
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
        cur.charge_expression_node(span)?;
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
            cur.charge_expression_node(tok.span)?;
            return Ok(cur.builder.bool_lit(true, tok.span));
        }
        if name == "false" {
            let tok = cur.bump();
            cur.charge_expression_node(tok.span)?;
            return Ok(cur.builder.bool_lit(false, tok.span));
        }
        if name == "null" {
            let tok = cur.bump();
            cur.charge_expression_node(tok.span)?;
            return Ok(cur.builder.null(tok.span));
        }
        // call vs bare symbol
        let name_tok = cur.bump();
        if matches!(cur.peek().kind, TokenKindV1::LParen) {
            return parse_call(cur, name, name_tok.span);
        }
        cur.charge_expression_node(name_tok.span)?;
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
            cur.budget
                .charge_collection_element(args.len().saturating_add(1), cur.here_span())?;
            if let TokenKindV1::Op(sym) = &cur.peek().kind {
                if sym == "*" {
                    let star = cur.bump();
                    if !args.is_empty() || !matches!(cur.peek().kind, TokenKindV1::RParen) {
                        return Err(
                            cur.unsupported(star.span, "'*' must be the sole call argument")
                        );
                    }
                    args.push(CallArgHirV1::star(star.span));
                    break;
                }
            }
            let arg = parse_value(cur)?;
            args.push(CallArgHirV1::Expr(arg));
            if cur.eat(&TokenKindV1::Comma).is_some() {
                continue;
            }
            break;
        }
    }
    let close = cur.expect(&TokenKindV1::RParen, "')' to close call arguments")?;
    let span = callee_span.merge(&close.span);
    cur.charge_expression_node(span)?;
    let args = CallArgsHirV1::try_new(args).map_err(|message| cur.unsupported(span, message))?;
    Ok(cur.builder.call(callee, callee_span, args, span))
}

fn parse_record(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    let open = cur.expect(&TokenKindV1::LBrace, "'{' to open record")?;
    let mut fields: Vec<RecordFieldV1> = Vec::new();
    if !matches!(cur.peek().kind, TokenKindV1::RBrace) {
        loop {
            cur.budget
                .charge_collection_element(fields.len().saturating_add(1), cur.here_span())?;
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
    cur.charge_expression_node(span)?;
    Ok(cur.builder.record(fields, span))
}

fn parse_list(cur: &mut CursorV1<'_>) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    let open = cur.expect(&TokenKindV1::LBracket, "'[' to open list")?;
    let mut elements = Vec::new();
    if !matches!(cur.peek().kind, TokenKindV1::RBracket) {
        loop {
            cur.budget
                .charge_collection_element(elements.len().saturating_add(1), cur.here_span())?;
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
    cur.charge_expression_node(span)?;
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
