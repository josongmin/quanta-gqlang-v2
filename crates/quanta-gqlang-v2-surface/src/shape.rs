//! Nested result-shape parsing producing the shared shape HIR.
//!
//! syntax -> hir: lowers the `=> <shape>` result surface into the single
//! frontend-owned `hir::ResultShapeHirV1` (D29). A shape is recursively a record
//! `{ field: <shape>, ... }`, a list `[ <shape> ]`, or a leaf value expression.
//! Leaf values reuse the shared value kernel, so nested shaping carries the same
//! exact spans as scalar expressions. No second shape tree is introduced.

use crate::cursor::CursorV1;
use crate::diag::ParseDiagnosticV1;
use crate::expr::parse_value;
use crate::lexer::TokenKindV1;
use quanta_gqlang_v2_hir::{ResultShapeHirV1, ShapeFieldV1};

/// Parse one result shape node.
pub(crate) fn parse_result_shape(
    cur: &mut CursorV1<'_>,
) -> Result<ResultShapeHirV1, ParseDiagnosticV1> {
    if matches!(cur.peek().kind, TokenKindV1::LBrace) {
        return parse_shape_record(cur);
    }
    if matches!(cur.peek().kind, TokenKindV1::LBracket) {
        return parse_shape_list(cur);
    }
    let value = parse_value(cur)?;
    let span = value.span;
    Ok(cur.builder.shape_leaf(value, span))
}

fn parse_shape_record(cur: &mut CursorV1<'_>) -> Result<ResultShapeHirV1, ParseDiagnosticV1> {
    let open = cur.expect(&TokenKindV1::LBrace, "'{' to open a result-shape record")?;
    let mut fields = Vec::new();
    if !matches!(cur.peek().kind, TokenKindV1::RBrace) {
        loop {
            let (name, name_span) = cur.expect_ident("result-shape field name")?;
            cur.expect(&TokenKindV1::Colon, "':' after result-shape field name")?;
            let shape = parse_result_shape(cur)?;
            fields.push(ShapeFieldV1 {
                name,
                name_span,
                shape,
            });
            if cur.eat(&TokenKindV1::Comma).is_some() {
                continue;
            }
            break;
        }
    }
    let close = cur.expect(&TokenKindV1::RBrace, "'}' to close a result-shape record")?;
    let span = open.span.merge(&close.span);
    Ok(cur.builder.shape_record(fields, span))
}

fn parse_shape_list(cur: &mut CursorV1<'_>) -> Result<ResultShapeHirV1, ParseDiagnosticV1> {
    let open = cur.expect(&TokenKindV1::LBracket, "'[' to open a result-shape list")?;
    let element = parse_result_shape(cur)?;
    let close = cur.expect(&TokenKindV1::RBracket, "']' to close a result-shape list")?;
    let span = open.span.merge(&close.span);
    Ok(cur.builder.shape_list(element, span))
}
