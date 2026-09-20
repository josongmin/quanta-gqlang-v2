//! Query grammar producing shared relation/query/result-shape HIR.
//!
//! syntax -> hir: lowers the v3 pipeline query surface into
//! `hir::RelationExprHirV1` / `hir::QueryHirV1` / `hir::ResultShapeHirV1` via
//! `hir::HirBuilderV1`. STRUCTURE ONLY — alias binding, join algebra, group/sort
//! legality, and recursion semantics are owned by tickets 02/04/05 (D12, D13).
//!
//! Surface shape (pipeline form):
//!
//! ```text
//! from <source> [<alias>]
//!   | where <bool-expr>
//!   | project <value> [as <alias>], ...
//!   | join <source> [<alias>] [on <bool-expr>]
//!   | group by <value>, ...
//!   | order by <value> [desc|asc], ...
//!   => <result-shape>          # optional nested result shape
//! ```
//!
//! Each `| stage` wraps the current relation, so spans grow left-to-right and
//! the producer nests in pipeline order.
//!
//! Lowering of the structure-only carriers (no algebra applied here):
//!
//! - `where` lowers to `RelationExprKind::Filter`.
//! - `project` lowers to `RelationExprKind::Project`.
//! - `join` lowers to `Compose("join", [left, right])`; an optional `on`
//!   predicate wraps the compose in a `Filter`.
//! - `group by` lowers to `Compose("group_by", [input, keys])` where `keys` is a
//!   `Project` carrying the group-key value exprs.
//! - `order by` lowers to `Compose("order_by", [input, keys])` likewise; sort
//!   direction is folded into the key alias as `asc`/`desc` (an open tag,
//!   interpreted by ticket 04). No bag/order semantics are decided here (D26).

use crate::cursor::CursorV1;
use crate::diag::ParseDiagnosticV1;
use crate::expr::{parse_bool, parse_optional_alias, parse_projection_item, parse_value};
use crate::lexer::TokenKindV1;
use crate::shape::parse_result_shape;
use quanta_gqlang_v2_hir::{
    AliasV1, OrderKeyHirV1, ProjectionItemV1, QueryHirV1, RelationExprHirV1, SortDirectionHirV1,
    SpanV1,
};

/// Parse a full query unit starting at `from`.
pub(crate) fn parse_query(cur: &mut CursorV1<'_>) -> Result<QueryHirV1, ParseDiagnosticV1> {
    let from_kw = cur.expect_keyword("from", "'from' to start a query")?;
    let mut relation = parse_source(cur)?;
    let mut end_span = relation.span;

    while matches!(cur.peek().kind, TokenKindV1::Pipe) {
        let _pipe = cur.bump();
        relation = parse_stage(cur, relation)?;
        end_span = relation.span;
    }

    // optional result shape after `=>`
    let mut result_shape = None;
    if matches!(cur.peek().kind, TokenKindV1::Arrow) {
        let _arrow = cur.bump();
        let shape = parse_result_shape(cur)?;
        end_span = shape.span;
        result_shape = Some(shape);
    }

    let span = from_kw.span.merge(&end_span);
    Ok(cur.builder.query(relation, result_shape, span))
}

/// Parse a relation source: `<name> [<alias>]`.
fn parse_source(cur: &mut CursorV1<'_>) -> Result<RelationExprHirV1, ParseDiagnosticV1> {
    let (name, name_span) = cur.expect_ident("a relation source name")?;
    let alias = parse_bare_alias(cur)?;
    let span = match &alias {
        Some(a) => name_span.merge(&a.span),
        None => name_span,
    };
    Ok(cur.builder.relation_source(name, alias, span))
}

/// Parse an optional bare alias: a trailing identifier that is not a reserved
/// pipeline keyword. `users u` binds `u`; `users | where ...` binds nothing.
fn parse_bare_alias(cur: &mut CursorV1<'_>) -> Result<Option<AliasV1>, ParseDiagnosticV1> {
    // explicit `as <alias>` always wins
    if let Some(alias) = parse_optional_alias(cur)? {
        return Ok(Some(alias));
    }
    if let TokenKindV1::Ident(s) = &cur.peek().kind {
        if !is_reserved(s) {
            let span = cur.peek().span;
            let name = s.clone();
            let _ = cur.bump();
            return Ok(Some(AliasV1 { name, span }));
        }
    }
    Ok(None)
}

/// Reserved pipeline keywords that must not be eaten as a bare alias.
fn is_reserved(word: &str) -> bool {
    matches!(
        word,
        "from"
            | "where"
            | "project"
            | "join"
            | "on"
            | "group"
            | "order"
            | "by"
            | "as"
            | "and"
            | "or"
            | "not"
            | "asc"
            | "desc"
            | "let"
            | "rule"
            | "with"
            | "if"
            | "exists"
            | "forall"
            | "in"
    )
}

/// Parse one `| stage` body, wrapping `input`.
fn parse_stage(
    cur: &mut CursorV1<'_>,
    input: RelationExprHirV1,
) -> Result<RelationExprHirV1, ParseDiagnosticV1> {
    if cur.eat_keyword("where").is_some() {
        return parse_where(cur, input);
    }
    if cur.eat_keyword("project").is_some() {
        return parse_project(cur, input);
    }
    if cur.eat_keyword("join").is_some() {
        return parse_join(cur, input);
    }
    if cur.eat_keyword("cross").is_some() {
        return parse_cross_join(cur, input);
    }
    if cur.eat_keyword("group").is_some() {
        return parse_group(cur, input);
    }
    if cur.eat_keyword("order").is_some() {
        return parse_order(cur, input);
    }
    Err(cur.unexpected("a pipeline stage keyword (where, project, join, group, order)"))
}

fn parse_where(
    cur: &mut CursorV1<'_>,
    input: RelationExprHirV1,
) -> Result<RelationExprHirV1, ParseDiagnosticV1> {
    let input_span = input.span;
    let predicate = parse_bool(cur)?;
    let span = input_span.merge(&predicate.span);
    Ok(cur.builder.relation_filter(input, predicate, span))
}

fn parse_project(
    cur: &mut CursorV1<'_>,
    input: RelationExprHirV1,
) -> Result<RelationExprHirV1, ParseDiagnosticV1> {
    let input_span = input.span;
    let items = parse_projection_items(cur)?;
    let span = project_span(input_span, &items);
    cur.builder
        .try_project(input, items, span)
        .map_err(|message| {
            ParseDiagnosticV1::new(
                crate::diag::ParseErrorCodeV1::UnexpectedToken,
                span,
                message,
            )
        })
}

fn parse_join(
    cur: &mut CursorV1<'_>,
    left: RelationExprHirV1,
) -> Result<RelationExprHirV1, ParseDiagnosticV1> {
    let left_span = left.span;
    let right = parse_source(cur)?;
    cur.expect_keyword("on", "'on' after an inner join source")?;
    let predicate = parse_bool(cur)?;
    let span = left_span.merge(&predicate.span);
    Ok(cur.builder.inner_join(left, right, predicate, span))
}

fn parse_cross_join(
    cur: &mut CursorV1<'_>,
    left: RelationExprHirV1,
) -> Result<RelationExprHirV1, ParseDiagnosticV1> {
    cur.expect_keyword("join", "'join' after 'cross'")?;
    let right = parse_source(cur)?;
    let span = left.span.merge(&right.span);
    Ok(cur.builder.cross_join(left, right, span))
}

fn parse_group(
    cur: &mut CursorV1<'_>,
    input: RelationExprHirV1,
) -> Result<RelationExprHirV1, ParseDiagnosticV1> {
    cur.expect_keyword("by", "'by' after 'group'")?;
    let input_span = input.span;
    let items = parse_value_key_items(cur, false)?;
    let span = project_span(input_span, &items);
    let keys = items.into_iter().map(|item| item.value).collect();
    cur.builder.group_by(input, keys, span).map_err(|message| {
        ParseDiagnosticV1::new(
            crate::diag::ParseErrorCodeV1::UnexpectedToken,
            span,
            message,
        )
    })
}

fn parse_order(
    cur: &mut CursorV1<'_>,
    input: RelationExprHirV1,
) -> Result<RelationExprHirV1, ParseDiagnosticV1> {
    cur.expect_keyword("by", "'by' after 'order'")?;
    let input_span = input.span;
    let items = parse_value_key_items(cur, true)?;
    let span = project_span(input_span, &items);
    let keys = items
        .into_iter()
        .map(|item| OrderKeyHirV1 {
            value: item.value,
            direction: match item.alias.as_ref().map(|alias| alias.name.as_str()) {
                Some("desc") => SortDirectionHirV1::Desc,
                _ => SortDirectionHirV1::Asc,
            },
        })
        .collect();
    cur.builder.order_by(input, keys, span).map_err(|message| {
        ParseDiagnosticV1::new(
            crate::diag::ParseErrorCodeV1::UnexpectedToken,
            span,
            message,
        )
    })
}

fn parse_projection_items(
    cur: &mut CursorV1<'_>,
) -> Result<Vec<ProjectionItemV1>, ParseDiagnosticV1> {
    let mut items = Vec::new();
    loop {
        cur.budget
            .charge_projection_element(items.len() + 1, cur.here_span())?;
        items.push(parse_projection_item(cur)?);
        if cur.eat(&TokenKindV1::Comma).is_some() {
            continue;
        }
        break;
    }
    Ok(items)
}

/// Parse comma-separated value keys. When `allow_direction` is set, an optional
/// trailing `asc`/`desc` becomes the item alias (an open sort-direction tag).
fn parse_value_key_items(
    cur: &mut CursorV1<'_>,
    allow_direction: bool,
) -> Result<Vec<ProjectionItemV1>, ParseDiagnosticV1> {
    let mut items = Vec::new();
    loop {
        cur.budget
            .charge_projection_element(items.len() + 1, cur.here_span())?;
        let value = parse_value(cur)?;
        let mut alias = None;
        if allow_direction {
            if let TokenKindV1::Ident(s) = &cur.peek().kind {
                if s == "asc" || s == "desc" {
                    let span = cur.peek().span;
                    let name = s.clone();
                    let _ = cur.bump();
                    alias = Some(AliasV1 { name, span });
                }
            }
        }
        items.push(ProjectionItemV1 { value, alias });
        if cur.eat(&TokenKindV1::Comma).is_some() {
            continue;
        }
        break;
    }
    Ok(items)
}

fn project_span(input_span: SpanV1, items: &[ProjectionItemV1]) -> SpanV1 {
    let mut span = input_span;
    for item in items {
        span = span.merge(&item.value.span);
        if let Some(alias) = &item.alias {
            span = span.merge(&alias.span);
        }
    }
    span
}
