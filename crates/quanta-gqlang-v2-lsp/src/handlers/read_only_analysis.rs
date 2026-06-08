//! Shared read-only analysis owner. Validity is delegated to the canonical parser
//! (`parse_document_v1`); hover/diagnostics/symbols/quickfix are thin adapters over
//! the typed summaries and diagnostics produced here.

mod errors;
mod summary;

use quanta_gqlang_v2_ir::frontend::{document_statements_v1, parse_document_v1};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolItemV1 {
    pub name: String,
    pub kind: String,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticItemV1 {
    pub class: String,
    pub code: String,
    pub message: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum DiagnosticClassV1 {
    Syntax,
    Sema,
}

impl DiagnosticClassV1 {
    #[must_use]
    pub(crate) const fn label_v1(self) -> &'static str {
        match self {
            Self::Syntax => "syntax",
            Self::Sema => "sema",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReadOnlyLspErrorV1 {
    pub class: DiagnosticClassV1,
    pub code: &'static str,
    pub message: String,
}

impl ReadOnlyLspErrorV1 {
    #[must_use]
    pub(crate) fn render_v1(&self) -> String {
        format!(
            "class={} code={} message={}",
            self.class.label_v1(),
            self.code,
            self.message
        )
    }

    #[must_use]
    pub(crate) fn diagnostic_item_v1(&self) -> DiagnosticItemV1 {
        DiagnosticItemV1 {
            class: self.class.label_v1().to_string(),
            code: self.code.to_string(),
            message: self.message.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReadOnlyRootSummaryV1 {
    pub root_keyword: &'static str,
    pub category: &'static str,
    pub typed_target: String,
    pub summary: Vec<String>,
    pub symbols: Vec<SymbolItemV1>,
}

pub(crate) fn analyze_query_v1(query: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    // The single-card read-only surfaces (hover/symbols/quickfix) summarize the
    // first statement; `analyze_statements_v1` covers a multi-statement document.
    // Statement boundaries match the canonical parser so the read-only surface can
    // never disagree with it about where one statement ends and the next begins.
    match document_statements_v1(query).into_iter().next() {
        Some(statement) => analyze_statement_v1(&statement),
        None => Err(errors::syntax_error_v1(
            "empty_document",
            "document is empty",
        )),
    }
}

pub(crate) fn analyze_statements_v1(
    query: &str,
) -> Vec<Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1>> {
    let statements = document_statements_v1(query);
    if statements.is_empty() {
        return vec![Err(errors::syntax_error_v1(
            "empty_document",
            "document is empty",
        ))];
    }
    statements
        .iter()
        .map(|statement| analyze_statement_v1(statement))
        .collect()
}

fn analyze_statement_v1(statement: &str) -> Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1> {
    // VALIDITY is owned entirely by the canonical parser: the read-only surface no
    // longer re-implements parsing (no shadow grammar that can accept what the
    // parser rejects or vice versa). On success the statement is a fully validated
    // typed `AstRootV1`, which we only FORMAT for the hover/symbols presentation.
    let document = parse_document_v1(statement).map_err(errors::syntax_page_to_lsp_error_v1)?;
    match document.roots.first() {
        Some(root) => Ok(summary::summary_from_root_v1(root)),
        None => Err(errors::syntax_error_v1(
            "empty_document",
            "document is empty",
        )),
    }
}
