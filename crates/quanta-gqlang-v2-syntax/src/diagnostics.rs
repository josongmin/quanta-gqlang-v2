use crate::span::SpanV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyntaxDiagnosticV1 {
    pub code: &'static str,
    pub message: String,
    pub span: Option<SpanV1>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SyntaxDiagnosticPageV1 {
    pub diagnostics: Vec<SyntaxDiagnosticV1>,
}
