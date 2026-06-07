use crate::diagnostics::SyntaxDiagnosticPageV1;
use crate::span::SpanV1;
use crate::token::{TokenKindV1, TokenV1};

pub fn tokenize_v1(input: &str) -> Result<Vec<TokenV1>, SyntaxDiagnosticPageV1> {
    let tokens = input
        .split_whitespace()
        .scan(0usize, |cursor, part| {
            let start = *cursor;
            let end = start + part.len();
            *cursor = end + 1;
            Some(TokenV1 {
                kind: TokenKindV1::Word(part.to_string()),
                span: SpanV1 { start, end },
            })
        })
        .collect();

    Ok(tokens)
}
