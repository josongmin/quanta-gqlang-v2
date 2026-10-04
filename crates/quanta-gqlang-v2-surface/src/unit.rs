//! Declaration and rule grammar producing shared unit HIR.
//!
//! syntax -> hir: lowers the v3 declaration and rule surfaces into
//! `hir::DeclHirV1` / `hir::RuleHirV1` via `hir::HirBuilderV1`, EMBEDDING the
//! shared value/bool expression kernel (ticket 01 scenario 3). No binding,
//! recursion, or type semantics are decided here (D13, owned by 02/03/05).
//!
//! Surface shape:
//!
//! ```text
//! let <name> = <value-expr>
//! rule <name> : <head-value-expr> if <bool-expr>
//! ```

use crate::cursor::CursorV1;
use crate::diag::ParseDiagnosticV1;
use crate::expr::{parse_bool, parse_value};
use crate::lexer::TokenKindV1;
use quanta_gqlang_v2_hir::{DeclHirV1, RuleHirV1};

/// Parse a `let` declaration into a shared-value-bodied [`DeclHirV1`].
pub(crate) fn parse_decl(cur: &mut CursorV1<'_>) -> Result<DeclHirV1, ParseDiagnosticV1> {
    let let_kw = cur.expect_keyword("let", "'let' to start a declaration")?;
    let (name, name_span) = cur.expect_ident("a declaration name")?;
    cur.expect_op("=", "'=' after the declaration name")?;
    let body = parse_value(cur)?;
    let span = let_kw.span.merge(&body.span);
    Ok(cur.builder.decl(name, name_span, body, span))
}

/// Parse a `rule` into a [`RuleHirV1`] embedding a shared value head and a
/// shared boolean body.
pub(crate) fn parse_rule(cur: &mut CursorV1<'_>) -> Result<RuleHirV1, ParseDiagnosticV1> {
    let rule_kw = cur.expect_keyword("rule", "'rule' to start a rule")?;
    let (name, name_span) = cur.expect_ident("a rule name")?;
    cur.expect(&TokenKindV1::Colon, "':' after the rule name")?;
    let head = parse_value(cur)?;
    cur.expect_keyword("if", "'if' between a rule head and its body")?;
    let body = parse_bool(cur)?;
    let span = rule_kw.span.merge(&body.span);
    Ok(cur.builder.rule(name, name_span, head, body, span))
}
