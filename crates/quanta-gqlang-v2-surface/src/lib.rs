//! QGLang v3 surface language parser (ticket 06).
//!
//! Owns the precedence-aware expression/query/declaration/rule grammar that
//! lowers the v3 surface into shared HIR (ticket 01), with EXACT spans for
//! diagnostics and formatting. It depends on `quanta-gqlang-v2-hir` ONLY: it
//! mints every node through `hir::HirBuilderV1` and never re-declares HIR nodes.
//! The legacy v2 parser stack has been removed (ticket 09 cutover); v3 is the
//! sole frontend.
//!
//! ## Cited packet decisions
//!
//! - `D1`: v3 is a new language core; this is a fresh grammar, not a v2 SELECT++.
//! - `D2`: lowers to the shared generic HIR; value/bool/relation/shape nodes are
//!   shared, never root-local.
//! - `D10`: unsupported/v2 constructs reject explicitly; parser recovery never
//!   silently downgrades to the v2 subset (see [`parser`] compat policy).
//! - `D13`: parser acceptance is STRUCTURAL only — binding, overload resolution,
//!   and type legality stay owned by tickets 02/03 and are not done here.
//! - `D25`: nested/path forms use lexical, binder-visible scope only; this crate
//!   applies NO hidden path scoping or implicit capture.
//! - `D29`: result shaping lowers into the single `hir::ResultShapeHirV1`; no
//!   second shape tree is introduced.
//!
//! ## Semantic boundary
//!
//! Operators, comparison symbols, callee names, composition tags (`join`,
//! `group_by`, `order_by`), and quantifier keywords are passed through as the
//! OPEN symbolic tags the HIR defines. Their legality (overloads, join algebra,
//! recursion, sort/group discipline) is owned by tickets 02/03/04/05.
//!
//! ## Public entry points
//!
//! - [`parse_program`]: parse one top-level surface item to a shared HIR unit.
//! - [`parse_document`]: parse a multi-root document (`;`-separated items) into
//!   an ordered list, lowered through one shared `hir::HirBuilderV1` so node ids
//!   stay globally unique across the document.
//! - [`parse_expr`]: parse a bare value expression (decl-body/arg grammar).
//! - [`parse_predicate`]: parse a bare boolean expression (filter/rule-body
//!   grammar).
//! - [`format_item`] / [`format_document`]: re-render a lowered HIR item (or a
//!   whole document) back to canonical, idempotent v3 surface text. The crate
//!   owns the grammar, so it owns the inverse.
//!
//! Every error path returns a typed [`ParseDiagnosticV1`] carrying a stable
//! [`ParseErrorCodeV1`] and an exact `hir::SpanV1`.

mod cursor;
mod diag;
mod expr;
mod family;
mod format;
mod lexer;
mod parser;
mod query;
mod shape;
mod unit;

pub use diag::{ParseDiagnosticV1, ParseErrorCodeV1};
pub use format::{format_document, format_item};
pub use lexer::{lex, LexErrorV1, TokenKindV1, TokenV1};
pub use parser::{parse_document, parse_program, ParsedDocumentV1, ParsedProgramV1, SurfaceItemV1};

use cursor::CursorV1;
use quanta_gqlang_v2_hir::{BoolExprHirV1, HirBuilderV1, ValueExprHirV1};

/// Parse a single bare value expression from `source` into shared value HIR.
///
/// Entry point for declaration bodies, call arguments, and projection values.
/// Fail-closed and span-exact like [`parse_program`]; rejects trailing tokens.
pub fn parse_expr(
    source: &str,
    builder: &HirBuilderV1,
) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    let tokens = lex(source)?;
    let mut cur = CursorV1::new(tokens, builder);
    let value = expr::parse_value(&mut cur)?;
    finish(&cur)?;
    Ok(value)
}

/// Parse a single bare boolean expression from `source` into shared bool HIR.
///
/// Entry point for filter predicates and rule bodies. Fail-closed and
/// span-exact; rejects trailing tokens.
pub fn parse_predicate(
    source: &str,
    builder: &HirBuilderV1,
) -> Result<BoolExprHirV1, ParseDiagnosticV1> {
    let tokens = lex(source)?;
    let mut cur = CursorV1::new(tokens, builder);
    let pred = expr::parse_bool(&mut cur)?;
    finish(&cur)?;
    Ok(pred)
}

/// Assert the cursor is at EOF, else a typed trailing-token reject.
fn finish(cur: &CursorV1<'_>) -> Result<(), ParseDiagnosticV1> {
    if cur.at_eof() {
        Ok(())
    } else {
        Err(ParseDiagnosticV1::new(
            ParseErrorCodeV1::UnexpectedToken,
            cur.here_span(),
            "unexpected trailing tokens after the expression",
        ))
    }
}
