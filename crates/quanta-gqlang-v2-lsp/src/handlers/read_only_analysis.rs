//! Shared read-only analysis owner. Validity is delegated to the canonical parser
//! (`parse_document_v1`); hover/diagnostics/symbols/quickfix are thin adapters over
//! the typed summaries and diagnostics produced here.

mod errors;
mod summary;

use quanta_gqlang_v2_ir::frontend::{
    analyze_document_v1, parse_document_v1, AstRootV1, DocumentAstV1, SemaDiagnosticPageV1,
    SemaDiagnosticV1, COMPOSITION_INVALID_PRODUCER_CODE_V1, COMPOSITION_UNDEFINED_BINDING_CODE_V1,
};

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
    // FIRST root of the document. The canonical parser owns statement boundaries
    // AND the composition grouping pass (D2: a `[LET]* WITH <body>` run folds into
    // one root), so the first root may itself be a whole composition block — the
    // read-only surface can never disagree with the parser about where one
    // statement (or one composition block) ends and the next begins.
    let document = parse_first_page_v1(query)?;
    match document.roots.first() {
        Some(root) => Ok(summary::summary_from_root_v1(root)),
        None => Err(errors::syntax_error_v1(
            "empty_document",
            "document is empty",
        )),
    }
}

pub(crate) fn analyze_statements_v1(
    query: &str,
) -> Vec<Result<ReadOnlyRootSummaryV1, ReadOnlyLspErrorV1>> {
    // Diagnostics span the WHOLE document. Parse once so the grouping pass folds
    // composition runs into single roots (a `LET`/`WITH` line parsed in isolation
    // would fail closed); the parser's phase-1 per-line recovery still surfaces
    // every malformed line at once.
    let document = match parse_document_v1(query) {
        Ok(document) => document,
        Err(page) => {
            return errors::syntax_page_to_lsp_errors_v1(page)
                .map(Err)
                .collect()
        }
    };
    if document.roots.is_empty() {
        return vec![Err(errors::syntax_error_v1(
            "empty_document",
            "document is empty",
        ))];
    }
    // Composition is the one root whose rejects live in the SEMA layer
    // (`composition_undefined_binding` / `composition_invalid_producer`, D3/D4):
    // run sema and surface those with `class=sema`, exactly as the lower layer
    // classified them. Sema runs only when a composition is present so every
    // non-composition document stays byte-identical to the parser-only path.
    let sema_errors = composition_sema_errors_v1(&document);
    document
        .roots
        .iter()
        .map(|root| {
            if matches!(root, AstRootV1::Composition(_)) {
                if let Some(err) = composition_root_error_v1(&sema_errors) {
                    return Err(err);
                }
            }
            Ok(summary::summary_from_root_v1(root))
        })
        .collect()
}

/// Parse a document but, on failure, collapse the page to a single
/// first-statement error (the single-card surfaces present one root).
fn parse_first_page_v1(query: &str) -> Result<DocumentAstV1, ReadOnlyLspErrorV1> {
    parse_document_v1(query).map_err(errors::syntax_page_to_lsp_error_v1)
}

/// Run sema over the document and keep only the `composition_*` diagnostics. The
/// sema page is fail-fast, so this surfaces the first composition reject; the
/// composition contract docs carry exactly one composition, so one item is the
/// whole story. Non-composition sema codes are intentionally NOT surfaced here:
/// the parser-only path already pins those, and widening would regress it.
fn composition_sema_errors_v1(document: &DocumentAstV1) -> Vec<ReadOnlyLspErrorV1> {
    if !document
        .roots
        .iter()
        .any(|root| matches!(root, AstRootV1::Composition(_)))
    {
        return Vec::new();
    }
    match analyze_document_v1(document) {
        Ok(_) => Vec::new(),
        Err(page) => sema_page_to_composition_errors_v1(page),
    }
}

/// Map a sema diagnostic page to LSP errors, keeping only composition codes.
/// The code is a typed field on the diagnostic and is compared against the
/// registered constants — no string-parsing of the message, no drift surface.
fn sema_page_to_composition_errors_v1(page: SemaDiagnosticPageV1) -> Vec<ReadOnlyLspErrorV1> {
    page.diagnostics
        .into_iter()
        .filter_map(composition_sema_error_v1)
        .collect()
}

fn composition_sema_error_v1(diagnostic: SemaDiagnosticV1) -> Option<ReadOnlyLspErrorV1> {
    let static_code = composition_sema_code_v1(diagnostic.code)?;
    Some(ReadOnlyLspErrorV1 {
        class: DiagnosticClassV1::Sema,
        code: static_code,
        message: diagnostic.message,
    })
}

/// The closed set of sema-classified composition codes (D3/D4), matched against
/// the constants the sema layer itself raises. A new composition sema code must
/// be admitted here deliberately; everything else stays on the parser-only path.
fn composition_sema_code_v1(code: &'static str) -> Option<&'static str> {
    match code {
        COMPOSITION_INVALID_PRODUCER_CODE_V1 | COMPOSITION_UNDEFINED_BINDING_CODE_V1 => Some(code),
        _ => None,
    }
}

fn composition_root_error_v1(errors: &[ReadOnlyLspErrorV1]) -> Option<ReadOnlyLspErrorV1> {
    errors.first().cloned()
}
