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
//! Scalar operators and callee names retain authored symbols. Relation stages,
//! quantifiers, relational existence, and composition slots lower to dedicated
//! closed HIR variants. Binding and type legality remain downstream concerns.
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
//! - [`format_item`] / [`format_document`]: re-render parser-produced HIR back
//!   to canonical, idempotent v3 surface text.
//! - [`try_format_item`]: checked surface-representability boundary for
//!   caller-built HIR; rejects unrepresentable shapes and parser-invalid
//!   output. HIR identity/span integrity remains the HIR owner's check.
//!
//! Every error path returns a typed [`ParseDiagnosticV1`] carrying a stable
//! [`ParseErrorCodeV1`] and an exact `hir::SpanV1`.
//!
//! ## Example
//!
//! ```
//! use quanta_gqlang_v2_hir::HirBuilderV1;
//! use quanta_gqlang_v2_surface::{format_document, parse_document};
//!
//! let builder = HirBuilderV1::new();
//! // One builder, two roots separated by the document `;` terminator, so the
//! // lowered node ids stay globally unique across the whole document.
//! let doc = parse_document("let total = base + tax * 2; from users u | project u.id as id", &builder)?;
//! assert_eq!(doc.items.len(), 2);
//!
//! // `format_document` re-renders the lowered HIR as canonical v3 surface text.
//! let items: Vec<_> = doc.items.iter().map(|item| item.to_program_item()).collect();
//! let text = format_document(&items);
//!
//! // Fixed point: the canonical text PARSES again and re-renders identically.
//! // (A fresh builder, because a new document mints its own node ids.)
//! let reparsed = parse_document(&text, &HirBuilderV1::new())?;
//! let items2: Vec<_> = reparsed.items.iter().map(|item| item.to_program_item()).collect();
//! assert_eq!(
//!     format_document(&items2),
//!     text,
//!     "format -> parse -> format must be a fixed point"
//! );
//! # Ok::<(), quanta_gqlang_v2_surface::ParseDiagnosticV1>(())
//! ```

mod budget;
mod cursor;
mod depth;
mod diag;
mod equivalence;
mod expr;
mod family;
mod format;
mod lexer;
mod parser;
mod query;
mod shape;
mod unit;

pub use budget::{ParseBudgetDimensionV2, ParseBudgetV2};
pub use diag::{ParseDiagnosticV1, ParseErrorCodeV1, ResourceExhaustionV2, SurfaceDiagnosticV2};
pub use format::{
    format_document, format_item, try_format_item, FormatDiagnosticV1, FormatErrorCodeV1,
};
pub use lexer::{lex, LexErrorV1, TokenKindV1, TokenV1};
pub use parser::{
    classify_parsed_root_v3, parse_document, parse_program, parse_source_document_v2,
    ParsedDocumentV1, ParsedDocumentV2, ParsedProgramV1, SourceRootFamilyV3, SurfaceItemV1,
};

use cursor::CursorV1;
use quanta_gqlang_v2_hir::{BoolExprHirV1, HirBuilderV1, SourceIdV2, ValueExprHirV1};

/// Parse a single bare value expression from `source` into shared value HIR.
///
/// Entry point for declaration bodies, call arguments, and projection values.
/// Fail-closed and span-exact like [`parse_program`]; rejects trailing tokens.
pub fn parse_expr(
    source: &str,
    builder: &HirBuilderV1,
) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    parse_expr_with_budget(source, builder, &ParseBudgetV2::standard_v2())
}

fn parse_expr_with_budget(
    source: &str,
    builder: &HirBuilderV1,
    budget: &ParseBudgetV2,
) -> Result<ValueExprHirV1, ParseDiagnosticV1> {
    let counter = budget::ParseBudgetCounterV2::new(budget);
    let tokens = lexer::lex_with_budget(source, &counter)?;
    let mut cur = CursorV1::new(tokens, builder, counter);
    let value = expr::parse_value(&mut cur)?;
    finish(&cur)?;
    depth::check_value(&value, budget.max_expression_depth())?;
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
    let budget = ParseBudgetV2::standard_v2();
    let counter = budget::ParseBudgetCounterV2::new(&budget);
    let tokens = lexer::lex_with_budget(source, &counter)?;
    let mut cur = CursorV1::new(tokens, builder, counter);
    let pred = expr::parse_bool(&mut cur)?;
    finish(&cur)?;
    depth::check_bool(&pred, budget.max_expression_depth())?;
    Ok(pred)
}

/// Parse then canonically format a document with one explicit parse budget.
/// Successful output is also admitted by that budget. Output rejection retains
/// the caller's source identity and anchors on the complete original document;
/// generated-text offsets must never be attributed to the original source.
pub fn format_source_document_with_parse_budget_v2(
    source_id: SourceIdV2,
    source: &str,
    budget: &ParseBudgetV2,
) -> Result<String, SurfaceDiagnosticV2> {
    let builder = HirBuilderV1::new();
    let parsed = parser::parse_document_items_with_budget(source, &builder, budget)
        .map_err(|error| SurfaceDiagnosticV2::from_parse(source_id, error))?;
    let items = parsed
        .items
        .into_iter()
        .map(SurfaceItemV1::into_program_item)
        .collect::<Vec<_>>();
    let rendered = format_document(&items);
    parser::parse_document_items_with_budget(&rendered, &HirBuilderV1::new(), budget).map_err(
        |mut error| {
            error.span = quanta_gqlang_v2_hir::SpanV1::new(0, source.len());
            error.message = format!("canonical output: {}", error.message);
            SurfaceDiagnosticV2::from_parse(source_id, error)
        },
    )?;
    Ok(rendered)
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
