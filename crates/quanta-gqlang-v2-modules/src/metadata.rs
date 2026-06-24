//! Query metadata model (deliverable 5).
//!
//! Query metadata is part of the authoring contract, not an optional comment
//! (ticket pinned semantics 5). Every distributable query carries a
//! [`QueryMetadataV1`]: a query kind, a severity / truth class, a deterministic
//! set of tags/categories, and help/doc hooks. The truth class deliberately
//! does NOT encode planner truth semantics (`D24`/ticket 07 own that); it is the
//! authoring-facing classification used for suite selection and presentation.

use std::collections::BTreeSet;

/// The authoring kind of a query.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum QueryKindV1 {
    /// A problem/alert-style query that flags findings.
    Problem,
    /// A path/dataflow-style query that reports a trace.
    Path,
    /// A metric/diagnostic query that reports an aggregate value.
    Metric,
}

impl QueryKindV1 {
    /// Stable wire token for this kind.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Problem => "problem",
            Self::Path => "path",
            Self::Metric => "metric",
        }
    }
}

/// The authoring severity / truth class of a query result.
///
/// This is the presentation-facing classification (`D20`): it is fixed and
/// host-agnostic. It is NOT the planner's truth evaluation (ticket 07).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SeverityClassV1 {
    /// Informational, no action implied.
    Note,
    /// A warning-level finding.
    Warning,
    /// An error-level finding.
    Error,
}

impl SeverityClassV1 {
    /// Stable wire token for this severity.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

/// Help/doc hooks for a query.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HelpDocV1 {
    /// A short one-line summary shown in suite listings.
    pub summary: String,
    /// Optional long-form help / remediation guidance.
    pub description: String,
}

/// The complete authoring metadata for a distributable query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryMetadataV1 {
    /// Stable query id (unique within its pack).
    pub id: String,
    /// The query kind.
    pub kind: QueryKindV1,
    /// The severity / truth class.
    pub severity: SeverityClassV1,
    /// Deterministically ordered, de-duplicated tags/categories.
    pub tags: BTreeSet<String>,
    /// Help / doc hooks.
    pub help: HelpDocV1,
}

impl QueryMetadataV1 {
    /// Construct query metadata with no tags and empty help.
    #[must_use]
    pub fn new(id: impl Into<String>, kind: QueryKindV1, severity: SeverityClassV1) -> Self {
        Self {
            id: id.into(),
            kind,
            severity,
            tags: BTreeSet::new(),
            help: HelpDocV1::default(),
        }
    }

    /// Add a tag/category (idempotent; ordering is canonical via `BTreeSet`).
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        let _ = self.tags.insert(tag.into());
        self
    }

    /// Attach help/doc hooks.
    #[must_use]
    pub fn with_help(mut self, summary: impl Into<String>, description: impl Into<String>) -> Self {
        self.help = HelpDocV1 {
            summary: summary.into(),
            description: description.into(),
        };
        self
    }
}
