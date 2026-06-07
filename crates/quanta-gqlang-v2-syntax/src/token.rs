use crate::span::SpanV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKindV1 {
    Word(String),
    Symbol(char),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenV1 {
    pub kind: TokenKindV1,
    pub span: SpanV1,
}
