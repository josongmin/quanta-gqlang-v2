//! Typed, fail-closed diagnostics for unsupported / ill-formed algebra states
//! (ticket 04, decision `D10`).
//!
//! Normalization NEVER silently downgrades an unsupported HIR composition into a
//! narrower algebra shape. When it cannot produce a well-formed algebra node it
//! emits one of these typed diagnostics. Each variant renders a STABLE
//! `code()` and a human `message()` so CLI/LSP/tests assert exact text (`D22`).

use quanta_gqlang_v2_hir::SpanV1;

/// A typed algebra-normalization diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AlgebraDiagnosticV1 {
    /// A composition operator symbol has no algebra lowering.
    UnsupportedComposition {
        /// The unknown composition symbol as written (e.g. `"crossapply++"`).
        symbol: String,
        /// Span of the offending composition.
        span: SpanV1,
    },
    /// A composition operator was applied with the wrong number of operands
    /// (e.g. a binary `join` given three operands, or a `union` given one).
    ArityMismatch {
        /// The composition symbol.
        symbol: String,
        /// The number of operands the operator requires.
        expected: usize,
        /// The number of operands actually present.
        found: usize,
        /// Span of the offending composition.
        span: SpanV1,
    },
    /// An aggregate projection mixed grouping keys and aggregate folds in a way
    /// that cannot be split into separate Group/Aggregate nodes (e.g. an
    /// aggregate call nested inside another expression alongside a bare key).
    IllFormedAggregate {
        /// Why the aggregate shape is ill-formed.
        reason: IllFormedAggregateReasonV1,
        /// Span of the offending projection.
        span: SpanV1,
    },
    /// An `apply`/lateral composition was declared correlated but exposed no
    /// visible correlation columns referencing the outer input — a hidden /
    /// side-channel correlation, which is rejected (`D23`).
    MissingCorrelation {
        /// Span of the offending apply composition.
        span: SpanV1,
    },
    /// A `limit` or `top-k` shape was applied to an input whose order is not
    /// semantic (a bag/set), so the kept rows would be non-deterministic.
    UnorderedLimit {
        /// Span of the offending limit composition.
        span: SpanV1,
    },
    /// Two union arms are not union-compatible (different column counts).
    UnionArity {
        /// Column count of the left arm.
        left: usize,
        /// Column count of the right arm.
        right: usize,
        /// Span of the offending union.
        span: SpanV1,
    },
    /// Two union arms have the same column COUNT but a positionally-aligned
    /// column disagrees on its output NAME, so the merged schema would silently
    /// pick one arm's name over the other. Union arms must agree on column names
    /// position-for-position; a name disagreement rejects rather than letting the
    /// left arm's name win unannounced (`D26`, fail-closed).
    UnionColumnMismatch {
        /// Zero-based index of the first column whose names disagree.
        position: usize,
        /// The left arm's name at that position.
        left_name: String,
        /// The right arm's name at that position.
        right_name: String,
        /// Span of the offending union.
        span: SpanV1,
    },
}

/// Why an aggregate projection is ill-formed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IllFormedAggregateReasonV1 {
    /// An aggregate call appeared nested inside a larger expression rather than
    /// as a top-level projection item, so grouping keys and folds cannot be
    /// cleanly separated.
    NestedAggregate,
    /// A projection produced zero output columns while requesting aggregation.
    EmptyAggregateProjection,
}

impl IllFormedAggregateReasonV1 {
    /// Stable code suffix for this reason.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NestedAggregate => "nested_aggregate",
            Self::EmptyAggregateProjection => "empty_aggregate_projection",
        }
    }
}

impl AlgebraDiagnosticV1 {
    /// The stable diagnostic code, shared by CLI/LSP/tests. Codes are
    /// dot-namespaced under `algebra.` and never change spelling.
    #[must_use]
    pub fn code(&self) -> String {
        match self {
            Self::UnsupportedComposition { .. } => "algebra.unsupported_composition".to_owned(),
            Self::ArityMismatch { .. } => "algebra.arity_mismatch".to_owned(),
            Self::IllFormedAggregate { reason, .. } => {
                format!("algebra.ill_formed_aggregate.{}", reason.code())
            }
            Self::MissingCorrelation { .. } => "algebra.missing_correlation".to_owned(),
            Self::UnorderedLimit { .. } => "algebra.unordered_limit".to_owned(),
            Self::UnionArity { .. } => "algebra.union_arity".to_owned(),
            Self::UnionColumnMismatch { .. } => "algebra.union_column_mismatch".to_owned(),
        }
    }

    /// The span the diagnostic points at.
    #[must_use]
    pub fn span(&self) -> SpanV1 {
        match self {
            Self::UnsupportedComposition { span, .. }
            | Self::ArityMismatch { span, .. }
            | Self::IllFormedAggregate { span, .. }
            | Self::MissingCorrelation { span }
            | Self::UnorderedLimit { span }
            | Self::UnionArity { span, .. }
            | Self::UnionColumnMismatch { span, .. } => *span,
        }
    }

    /// A stable human-readable message. Text is part of the contract: tests
    /// assert it verbatim.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::UnsupportedComposition { symbol, .. } => {
                format!("unsupported composition operator `{symbol}` has no algebra lowering")
            }
            Self::ArityMismatch {
                symbol,
                expected,
                found,
                ..
            } => format!("composition `{symbol}` expects {expected} operand(s) but found {found}"),
            Self::IllFormedAggregate { reason, .. } => match reason {
                IllFormedAggregateReasonV1::NestedAggregate => {
                    "aggregate call must be a top-level projection item, not nested in an \
                     expression"
                        .to_owned()
                }
                IllFormedAggregateReasonV1::EmptyAggregateProjection => {
                    "aggregate projection produced no output columns".to_owned()
                }
            },
            Self::MissingCorrelation { .. } => {
                "correlated apply has no visible correlation columns referencing the outer input"
                    .to_owned()
            }
            Self::UnorderedLimit { .. } => {
                "limit requires an ordered input; sort before limiting".to_owned()
            }
            Self::UnionArity { left, right, .. } => format!(
                "union arms are not compatible: left has {left} column(s), right has {right}"
            ),
            Self::UnionColumnMismatch {
                position,
                left_name,
                right_name,
                ..
            } => format!(
                "union arms disagree on column {position}: left names it `{left_name}`, right \
                 names it `{right_name}`"
            ),
        }
    }
}
