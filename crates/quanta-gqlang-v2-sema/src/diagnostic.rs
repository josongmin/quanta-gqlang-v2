//! Stable, bounded diagnostics emitted by the semantic transaction.

use quanta_gqlang_v2_hir::{LocatedSpanV2, SourceIdV2, SpanV1};

const MAX_IDENTIFIER_EXCERPT_BYTES: usize = 128;
const MAX_MESSAGE_BYTES: usize = 1_024;

/// Ordered frontend stage attached to a diagnostic.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrontendStageV2 {
    /// HIR integrity validation.
    Integrity,
    /// Document declaration and namespace construction.
    Declaration,
    /// Name and catalog reference resolution.
    Resolution,
    /// Lexical binding.
    Binding,
    /// Expression and relation typing.
    Typing,
    /// Document-wide semantic validation.
    Validation,
    /// Owner-local resource accounting.
    Resource,
}

impl FrontendStageV2 {
    /// Stable wire/CLI spelling.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Integrity => "integrity",
            Self::Declaration => "declaration",
            Self::Resolution => "resolution",
            Self::Binding => "binding",
            Self::Typing => "typing",
            Self::Validation => "validation",
            Self::Resource => "resource",
        }
    }
}

/// Ordered diagnostic severity. Errors sort before warnings and notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrontendSeverityV2 {
    /// The transaction cannot produce a semantic document.
    Error,
    /// Recoverable analysis warning.
    Warning,
    /// Additional explanatory information.
    Note,
}

impl FrontendSeverityV2 {
    /// Stable wire/CLI spelling.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
        }
    }
}

/// One deterministic diagnostic with source-rich edges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendDiagnosticV2 {
    stage: FrontendStageV2,
    code: &'static str,
    severity: FrontendSeverityV2,
    primary: LocatedSpanV2,
    secondary: Vec<LocatedSpanV2>,
    message: String,
}

impl FrontendDiagnosticV2 {
    /// Construct a bounded diagnostic and canonicalize its edge order.
    #[must_use]
    pub fn new(
        stage: FrontendStageV2,
        code: &'static str,
        severity: FrontendSeverityV2,
        primary: LocatedSpanV2,
        mut secondary: Vec<LocatedSpanV2>,
        message: impl AsRef<str>,
    ) -> Self {
        secondary.sort_by_key(location_key);
        secondary.dedup();
        Self {
            stage,
            code,
            severity,
            primary,
            secondary,
            message: truncate_utf8(message.as_ref(), MAX_MESSAGE_BYTES),
        }
    }

    /// Diagnostic stage.
    #[must_use]
    pub const fn stage(&self) -> FrontendStageV2 {
        self.stage
    }

    /// Stable machine-readable code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }

    /// Severity.
    #[must_use]
    pub const fn severity(&self) -> FrontendSeverityV2 {
        self.severity
    }

    /// Primary source location.
    #[must_use]
    pub const fn primary(&self) -> LocatedSpanV2 {
        self.primary
    }

    /// Sorted, deduplicated secondary and edge locations.
    #[must_use]
    pub fn secondary(&self) -> &[LocatedSpanV2] {
        &self.secondary
    }

    /// Bounded human-readable message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn sort_key(
        &self,
    ) -> (
        (u32, usize, usize),
        FrontendStageV2,
        FrontendSeverityV2,
        &str,
        &str,
    ) {
        (
            (
                self.primary.source_id.raw(),
                self.primary.span.start,
                self.primary.span.end,
            ),
            self.stage,
            self.severity,
            self.code,
            &self.message,
        )
    }
}

/// Atomic semantic failure. Its diagnostic list is always non-empty and sorted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendFailureV2 {
    diagnostics: Vec<FrontendDiagnosticV2>,
}

impl FrontendFailureV2 {
    /// Construct a non-empty deterministic failure.
    pub fn try_new(mut diagnostics: Vec<FrontendDiagnosticV2>) -> Result<Self, &'static str> {
        if diagnostics.is_empty() {
            return Err("frontend failure requires at least one diagnostic");
        }
        diagnostics.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
        Ok(Self { diagnostics })
    }

    /// Construct a one-diagnostic failure without exposing an empty state.
    #[must_use]
    pub fn one(diagnostic: FrontendDiagnosticV2) -> Self {
        Self {
            diagnostics: vec![diagnostic],
        }
    }

    /// Remove adjacent identical diagnostics without exposing an empty state.
    ///
    /// The owner has already sorted by its canonical key. Distinct diagnostics
    /// tied on that key retain their input order, including secondary spans.
    /// This preserves the pipeline's existing adjacent-deduplication policy.
    #[must_use]
    pub fn deduplicated(mut self) -> Self {
        self.diagnostics.dedup();
        self
    }

    /// Ordered diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[FrontendDiagnosticV2] {
        &self.diagnostics
    }
}

pub(crate) fn location(source_id: SourceIdV2, span: SpanV1) -> LocatedSpanV2 {
    LocatedSpanV2 { source_id, span }
}

pub(crate) fn bounded_identifier(value: &str) -> String {
    truncate_utf8(value, MAX_IDENTIFIER_EXCERPT_BYTES)
}

fn truncate_utf8(value: &str, limit: usize) -> String {
    if value.len() <= limit {
        return value.to_owned();
    }
    let mut end = limit;
    while !value.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    value[..end].to_owned()
}

fn location_key(location: &LocatedSpanV2) -> (u32, usize, usize) {
    (
        location.source_id.raw(),
        location.span.start,
        location.span.end,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diagnostic(message: &str) -> FrontendDiagnosticV2 {
        FrontendDiagnosticV2::new(
            FrontendStageV2::Integrity,
            "sema.test",
            FrontendSeverityV2::Error,
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(7),
                span: SpanV1::new(11, 13),
            },
            Vec::new(),
            message,
        )
    }

    #[test]
    #[expect(
        clippy::expect_used,
        reason = "test constructs a nonempty diagnostic list"
    )]
    fn nonempty_failure_dedup_preserves_diagnostic_identity_and_tie_order() {
        assert_eq!(
            FrontendFailureV2::try_new(Vec::new()),
            Err("frontend failure requires at least one diagnostic")
        );
        let a = diagnostic("same");
        let mut b = a.clone();
        b.secondary
            .push(location(SourceIdV2::from_raw(9), SpanV1::new(21, 25)));
        let failure = FrontendFailureV2::try_new(vec![a.clone(), a.clone(), b.clone(), a.clone()])
            .expect("nonempty fixture failure");
        // Secondary locations distinguish diagnostics but are not sort keys.
        assert_eq!(
            failure.deduplicated().diagnostics(),
            &[a.clone(), b, a.clone()]
        );
        assert_eq!(
            FrontendFailureV2::one(a.clone())
                .deduplicated()
                .diagnostics(),
            &[a]
        );
    }

    #[test]
    fn stage_and_severity_keywords_are_exhaustive_and_literal() {
        let stages = [
            (FrontendStageV2::Integrity, "integrity"),
            (FrontendStageV2::Declaration, "declaration"),
            (FrontendStageV2::Resolution, "resolution"),
            (FrontendStageV2::Binding, "binding"),
            (FrontendStageV2::Typing, "typing"),
            (FrontendStageV2::Validation, "validation"),
            (FrontendStageV2::Resource, "resource"),
        ];
        for (stage, expected) in stages {
            assert_eq!(stage.keyword(), expected);
        }

        let severities = [
            (FrontendSeverityV2::Error, "error"),
            (FrontendSeverityV2::Warning, "warning"),
            (FrontendSeverityV2::Note, "note"),
        ];
        for (severity, expected) in severities {
            assert_eq!(severity.keyword(), expected);
        }
    }

    #[test]
    fn diagnostic_message_boundary_is_exact_for_ascii_and_utf8() {
        for length in [1_023, 1_024, 1_025] {
            let input = "a".repeat(length);
            let expected = "a".repeat(length.min(MAX_MESSAGE_BYTES));
            let actual = diagnostic(&input);
            assert_eq!(actual.message(), expected);
            assert_eq!(actual.message().len(), length.min(MAX_MESSAGE_BYTES));
        }

        let utf8_cases = [
            (
                format!("{}é", "a".repeat(1_021)),
                format!("{}é", "a".repeat(1_021)),
            ),
            (
                format!("{}é", "a".repeat(1_022)),
                format!("{}é", "a".repeat(1_022)),
            ),
            (format!("{}é", "a".repeat(1_023)), "a".repeat(1_023)),
        ];
        for (input, expected) in utf8_cases {
            let actual = diagnostic(&input);
            assert_eq!(actual.message(), expected);
            assert_eq!(actual.message().len(), expected.len());
        }
    }
}
