//! Fail-closed diagnostics for semantic-document algebra lowering.

use quanta_gqlang_v2_sema::CheckedLocatedSpanV2;

/// Closed failure categories emitted by [`AlgebraDiagnosticV2`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlgebraFailureKindV2 {
    /// The requested semantic item does not exist or is not relational.
    NonRelationItem,
    /// A stage refers to an expression absent from the same semantic document.
    MissingExpression,
    /// A relation stage or expression belongs to a different source document.
    ForeignSemanticFact,
    /// A stage payload disagrees with its source-reachable family contract.
    InvalidStageContract,
    /// An aggregate projection did not directly follow a checked Group stage.
    AggregateWithoutGroup,
    /// An aggregate was nested instead of being a projection expression itself.
    NestedAggregate,
    /// A relational EXISTS root is not present in the document's nested registry.
    MissingNestedRelation,
    /// A correlation set has duplicate reference identities.
    DuplicateCorrelationReference,
    /// Ownership discovery or lowering exceeded the document's shared sealed
    /// downstream receipt (`max_semantic_steps` for relation stages,
    /// `max_expression_nodes` for expression nodes).
    TraversalBudgetExceeded,
}

impl AlgebraFailureKindV2 {
    /// Full stable diagnostic code, shared with pipeline projections.
    #[must_use]
    pub const fn full_code(self) -> &'static str {
        match self {
            Self::NonRelationItem => "algebra.non_relation_item",
            Self::MissingExpression => "algebra.missing_expression",
            Self::ForeignSemanticFact => "algebra.foreign_semantic_fact",
            Self::InvalidStageContract => "algebra.invalid_stage_contract",
            Self::AggregateWithoutGroup => "algebra.aggregate_without_group",
            Self::NestedAggregate => "algebra.nested_aggregate",
            Self::MissingNestedRelation => "algebra.missing_nested_relation",
            Self::DuplicateCorrelationReference => "algebra.duplicate_correlation_reference",
            Self::TraversalBudgetExceeded => "algebra.traversal_budget_exceeded",
        }
    }

    /// Stable code suffix.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NonRelationItem => "non_relation_item",
            Self::MissingExpression => "missing_expression",
            Self::ForeignSemanticFact => "foreign_semantic_fact",
            Self::InvalidStageContract => "invalid_stage_contract",
            Self::AggregateWithoutGroup => "aggregate_without_group",
            Self::NestedAggregate => "nested_aggregate",
            Self::MissingNestedRelation => "missing_nested_relation",
            Self::DuplicateCorrelationReference => "duplicate_correlation_reference",
            Self::TraversalBudgetExceeded => "traversal_budget_exceeded",
        }
    }
}

/// One typed lowering failure tied to a checked semantic source coordinate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlgebraDiagnosticV2 {
    kind: AlgebraFailureKindV2,
    location: Option<CheckedLocatedSpanV2>,
    message: &'static str,
}

impl AlgebraDiagnosticV2 {
    pub(crate) const fn new(
        kind: AlgebraFailureKindV2,
        location: Option<CheckedLocatedSpanV2>,
        message: &'static str,
    ) -> Self {
        Self {
            kind,
            location,
            message,
        }
    }

    /// Stable diagnostic code.
    #[must_use]
    pub fn code(&self) -> String {
        self.kind.full_code().to_owned()
    }

    /// Closed failure category.
    #[must_use]
    pub const fn kind(&self) -> AlgebraFailureKindV2 {
        self.kind
    }

    /// Exact semantic checked location.
    #[must_use]
    pub const fn location(&self) -> Option<&CheckedLocatedSpanV2> {
        self.location.as_ref()
    }

    /// Stable user-facing message.
    #[must_use]
    pub const fn message(&self) -> &'static str {
        self.message
    }
}

impl std::fmt::Display for AlgebraDiagnosticV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code(), self.message)
    }
}

impl std::error::Error for AlgebraDiagnosticV2 {}
