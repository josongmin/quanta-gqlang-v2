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
