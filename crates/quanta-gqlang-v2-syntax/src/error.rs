//! E01 named seam: the typed parse-error view.
//!
//! `ParseErrorV1` is a typed, read-only view over a single entry of the existing
//! diagnostic channel ([`crate::diagnostics::SyntaxDiagnosticV1`]). It does NOT
//! introduce a second error channel: `parse_document_v1` still fails with a
//! [`crate::diagnostics::SyntaxDiagnosticPageV1`], and this type just gives
//! callers a stable, code-bearing handle over each diagnostic so a parse reject
//! is never reduced to opaque prose. Keeping it a borrow-backed projection means
//! there is no duplicated error state to drift out of sync with the page.

use crate::diagnostics::{SyntaxDiagnosticPageV1, SyntaxDiagnosticV1};
use crate::span::SpanV1;

/// A typed view over one diagnostic produced by a fail-closed parse rejection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParseErrorV1<'a> {
    diagnostic: &'a SyntaxDiagnosticV1,
}

impl<'a> ParseErrorV1<'a> {
    /// Wrap a single diagnostic as a typed parse error.
    #[must_use]
    pub const fn from_diagnostic_v1(diagnostic: &'a SyntaxDiagnosticV1) -> Self {
        Self { diagnostic }
    }

    /// Project every diagnostic in a rejection page into typed parse errors, in
    /// order. The page is the single source of truth; this never adds, drops, or
    /// reorders entries.
    #[must_use]
    pub fn from_page_v1(page: &'a SyntaxDiagnosticPageV1) -> Vec<Self> {
        page.diagnostics
            .iter()
            .map(Self::from_diagnostic_v1)
            .collect()
    }

    /// The stable machine-readable reject code (e.g. `"unsupported_root"`).
    #[must_use]
    pub const fn code(&self) -> &'a str {
        self.diagnostic.code
    }

    /// The human-readable reject message.
    #[must_use]
    pub fn message(&self) -> &'a str {
        &self.diagnostic.message
    }

    /// The source span of the reject, when the producing parser attached one.
    #[must_use]
    pub const fn span(&self) -> Option<&'a SpanV1> {
        self.diagnostic.span.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::ParseErrorV1;
    use crate::diagnostics::{SyntaxDiagnosticPageV1, SyntaxDiagnosticV1};

    #[test]
    fn typed_view_preserves_code_message_and_order() {
        let page = SyntaxDiagnosticPageV1 {
            diagnostics: vec![
                SyntaxDiagnosticV1 {
                    code: "first_code",
                    message: "first message".to_string(),
                    span: None,
                },
                SyntaxDiagnosticV1 {
                    code: "second_code",
                    message: "second message".to_string(),
                    span: None,
                },
            ],
        };

        let typed = ParseErrorV1::from_page_v1(&page);
        let codes: Vec<&str> = typed.iter().map(ParseErrorV1::code).collect();
        assert_eq!(codes, ["first_code", "second_code"]);
        let messages: Vec<&str> = typed.iter().map(ParseErrorV1::message).collect();
        assert_eq!(messages, ["first message", "second message"]);
        assert!(typed.iter().all(|err| err.span().is_none()));
    }
}
