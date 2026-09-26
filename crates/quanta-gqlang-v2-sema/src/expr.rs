//! Sealed typed-expression and aggregate-context contracts.

use crate::diagnostic::{
    FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2,
};
use crate::ResolvedValueBindingV2;
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, LocatedSpanV2, LogicalQuantifierKindV1, RelationExistsPolarityV1,
};
use quanta_gqlang_v2_types::{
    AggregationClassV1, ExpressionEffectsV2, ExpressionTypeStateV2, OperatorIdV2,
    ResolvedCallTargetV2, SourceQualifiedNodeIdV2, TypedExpressionFactV2, TypedLiteralFactV2,
    ValueTypeV1,
};

/// Position in which one expression was semantically checked.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AggregatePositionV2 {
    /// Ordinary scalar projection/value position.
    Scalar,
    /// Projection position allowed to introduce an aggregate.
    Projection,
    /// Grouping-key position; aggregates are forbidden.
    GroupKey,
    /// Predicate position; aggregates are forbidden.
    Predicate,
    /// Ordering position over an already produced row.
    OrderKey,
    /// Rule body position; aggregate polarity is retained for logic.
    RuleBody,
}

/// Logical quantifier kind after semantic sealing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LogicalQuantifierKindV2 {
    /// Existential quantification.
    Exists,
    /// Universal quantification.
    Forall,
}

impl From<LogicalQuantifierKindV1> for LogicalQuantifierKindV2 {
    fn from(kind: LogicalQuantifierKindV1) -> Self {
        match kind {
            LogicalQuantifierKindV1::Exists => Self::Exists,
            LogicalQuantifierKindV1::Forall => Self::Forall,
        }
    }
}

/// EXISTS polarity after semantic sealing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RelationExistsPolarityV2 {
    /// `exists (from ...)`.
    Exists,
    /// `not exists (from ...)`.
    NotExists,
}

impl From<RelationExistsPolarityV1> for RelationExistsPolarityV2 {
    fn from(polarity: RelationExistsPolarityV1) -> Self {
        match polarity {
            RelationExistsPolarityV1::Exists => Self::Exists,
            RelationExistsPolarityV1::NotExists => Self::NotExists,
        }
    }
}

/// Checked aggregate placement retained for algebra and logic consumers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AggregateContextV2 {
    position: AggregatePositionV2,
    aggregation: AggregationClassV1,
    star: bool,
}

/// One call argument in closed semantic expression shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedCallArgumentV2 {
    /// Typed expression argument.
    Expression(SourceQualifiedNodeIdV2),
    /// Authored aggregate star.
    Star,
}

/// One named record field in a closed semantic expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedRecordFieldV2 {
    name: String,
    name_location: CheckedLocatedSpanV2,
    value: SourceQualifiedNodeIdV2,
}

impl TypedRecordFieldV2 {
    /// Construct one field.
    pub(crate) fn try_new(
        name: impl Into<String>,
        name_location: CheckedLocatedSpanV2,
        value: SourceQualifiedNodeIdV2,
    ) -> Result<Self, &'static str> {
        let name = name.into();
        if name.is_empty() {
            return Err("typed record field name cannot be empty");
        }
        Ok(Self {
            name,
            name_location,
            value,
        })
    }
    /// Field name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Original checked field-name span.
    #[must_use]
    pub const fn name_location(&self) -> &CheckedLocatedSpanV2 {
        &self.name_location
    }
    /// Typed child identity.
    #[must_use]
    pub const fn value(&self) -> SourceQualifiedNodeIdV2 {
        self.value
    }
}

/// One authored path segment with its exact checked subspan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedPathSegmentV2 {
    name: String,
    location: CheckedLocatedSpanV2,
}

impl TypedPathSegmentV2 {
    pub(crate) fn new(name: String, location: CheckedLocatedSpanV2) -> Self {
        Self { name, location }
    }
    /// Exact authored segment name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Checked segment location.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

/// One explicit outer binding captured at an inner reference site. Multiple
/// references to the same binding remain distinct edges.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedCorrelationEdgeV2 {
    binding: ResolvedValueBindingV2,
    reference: SourceQualifiedNodeIdV2,
    location: CheckedLocatedSpanV2,
}

impl TypedCorrelationEdgeV2 {
    pub(crate) fn new(
        binding: ResolvedValueBindingV2,
        reference: SourceQualifiedNodeIdV2,
        location: CheckedLocatedSpanV2,
    ) -> Self {
        Self {
            binding,
            reference,
            location,
        }
    }
    /// Exact binder owner and ordinal.
    #[must_use]
    pub const fn binding(&self) -> &ResolvedValueBindingV2 {
        &self.binding
    }
    /// Inner symbol-reference node, not just the outer binding.
    #[must_use]
    pub const fn reference(&self) -> SourceQualifiedNodeIdV2 {
        self.reference
    }
    /// Inner checked reference span.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

/// Closed semantic expression shape. It retains only semantic identities and
/// checked operator/call/literal facts; downstream crates never revisit HIR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedExprShapeV2 {
    /// Checked literal value.
    Literal(TypedLiteralFactV2),
    /// Resolved value binding.
    Binding(ResolvedValueBindingV2),
    /// Checked field path.
    Path {
        base: SourceQualifiedNodeIdV2,
        segments: Vec<TypedPathSegmentV2>,
    },
    /// Checked unary operator.
    Unary {
        operator: OperatorIdV2,
        operand: SourceQualifiedNodeIdV2,
    },
    /// Checked binary operator.
    Binary {
        operator: OperatorIdV2,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    /// Resolved call including the authored star distinction.
    Call {
        target: ResolvedCallTargetV2,
        arguments: Vec<TypedCallArgumentV2>,
    },
    /// Explicit grouping.
    Paren { inner: SourceQualifiedNodeIdV2 },
    /// Typed record construction.
    Record(Vec<TypedRecordFieldV2>),
    /// Typed list construction.
    List(Vec<SourceQualifiedNodeIdV2>),
    /// Bool node forwarding one value child.
    BoolValue { value: SourceQualifiedNodeIdV2 },
    /// Checked comparison.
    Compare {
        operator: OperatorIdV2,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    /// Checked logical conjunction.
    And {
        operator: OperatorIdV2,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    /// Checked logical disjunction.
    Or {
        operator: OperatorIdV2,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    /// Checked logical negation.
    Not {
        operator: OperatorIdV2,
        operand: SourceQualifiedNodeIdV2,
    },
    /// Rule-only logical quantifier.
    LogicalQuantifier {
        kind: LogicalQuantifierKindV2,
        binders: Vec<ResolvedValueBindingV2>,
        body: SourceQualifiedNodeIdV2,
    },
    /// Relational existence with an explicit capture set, including empty.
    RelationExists {
        polarity: RelationExistsPolarityV2,
        relation_root: SourceQualifiedNodeIdV2,
        correlations: Vec<TypedCorrelationEdgeV2>,
    },
}

impl AggregateContextV2 {
    /// Construct a placement proof, rejecting aggregate calls in positions
    /// that never admit them and rejecting star outside aggregate calls.
    pub(crate) fn try_new(
        position: AggregatePositionV2,
        aggregation: AggregationClassV1,
        star: bool,
        location: LocatedSpanV2,
    ) -> Result<Self, FrontendFailureV2> {
        if aggregation == AggregationClassV1::Aggregate
            && matches!(
                position,
                AggregatePositionV2::Scalar
                    | AggregatePositionV2::GroupKey
                    | AggregatePositionV2::Predicate
                    | AggregatePositionV2::OrderKey
            )
        {
            return Err(expression_failure(
                "sema.illegal_aggregate_context",
                location,
                "aggregate expression is not legal in this position",
            ));
        }
        if star && aggregation != AggregationClassV1::Aggregate {
            return Err(expression_failure(
                "sema.illegal_aggregate_star",
                location,
                "star is legal only for an aggregate call",
            ));
        }
        Ok(Self {
            position,
            aggregation,
            star,
        })
    }

    /// Checked position.
    #[must_use]
    pub const fn position(&self) -> AggregatePositionV2 {
        self.position
    }
    /// Aggregate/scalar classification from the types-owned effect fact.
    #[must_use]
    pub const fn aggregation(&self) -> AggregationClassV1 {
        self.aggregation
    }
    /// Whether the authored call used star.
    #[must_use]
    pub const fn uses_star(&self) -> bool {
        self.star
    }
}

/// One source-qualified expression with its complete types-owned result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedExprV2 {
    key: SourceQualifiedNodeIdV2,
    location: CheckedLocatedSpanV2,
    type_state: ExpressionTypeStateV2,
    effects: ExpressionEffectsV2,
    resolved_call: Option<ResolvedCallTargetV2>,
    intrinsic_call_aggregation: Option<AggregationClassV1>,
    aggregate_context: AggregateContextV2,
    shape: TypedExprShapeV2,
}

impl TypedExprV2 {
    /// Retain one types-owned fact without re-resolving its type or callable.
    pub(crate) fn try_from_fact(
        key: SourceQualifiedNodeIdV2,
        fact: &TypedExpressionFactV2,
        aggregate_context: AggregateContextV2,
        shape: TypedExprShapeV2,
        location: CheckedLocatedSpanV2,
    ) -> Result<Self, FrontendFailureV2> {
        let raw_location = location.location();
        if raw_location.source_id != key.source_id {
            return Err(expression_failure(
                "sema.document_validation_failed",
                raw_location,
                "expression location belongs to another source",
            ));
        }
        if fact.effects.aggregation != aggregate_context.aggregation() {
            return Err(expression_failure(
                "sema.document_validation_failed",
                raw_location,
                "aggregate context disagrees with the types-owned effect fact",
            ));
        }
        let shape_call = match &shape {
            TypedExprShapeV2::Call { target, .. } => Some(target),
            TypedExprShapeV2::Literal(_)
            | TypedExprShapeV2::Binding(_)
            | TypedExprShapeV2::Path { .. }
            | TypedExprShapeV2::Unary { .. }
            | TypedExprShapeV2::Binary { .. }
            | TypedExprShapeV2::Paren { .. }
            | TypedExprShapeV2::Record(_)
            | TypedExprShapeV2::List(_)
            | TypedExprShapeV2::BoolValue { .. }
            | TypedExprShapeV2::Compare { .. }
            | TypedExprShapeV2::And { .. }
            | TypedExprShapeV2::Or { .. }
            | TypedExprShapeV2::Not { .. }
            | TypedExprShapeV2::LogicalQuantifier { .. }
            | TypedExprShapeV2::RelationExists { .. } => None,
        };
        if shape_call != fact.resolved_call() {
            return Err(expression_failure(
                "sema.document_validation_failed",
                raw_location,
                "semantic call shape disagrees with the types-owned resolved target",
            ));
        }
        if shape_call.is_some() != fact.intrinsic_call_aggregation().is_some() {
            return Err(expression_failure(
                "sema.document_validation_failed",
                raw_location,
                "semantic call shape disagrees with the types-owned intrinsic aggregation",
            ));
        }
        Ok(Self {
            key,
            location,
            type_state: fact.state.clone(),
            effects: fact.effects,
            resolved_call: fact.resolved_call().cloned(),
            intrinsic_call_aggregation: fact.intrinsic_call_aggregation(),
            aggregate_context,
            shape,
        })
    }

    /// Source-qualified HIR node identity.
    #[must_use]
    pub const fn key(&self) -> SourceQualifiedNodeIdV2 {
        self.key
    }
    /// Exact checked node span available to downstream consumers without HIR.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    /// Known or explicitly unresolved editor type state.
    #[must_use]
    pub const fn type_state(&self) -> &ExpressionTypeStateV2 {
        &self.type_state
    }
    /// Complete effects retained from the types kernel.
    #[must_use]
    pub const fn effects(&self) -> ExpressionEffectsV2 {
        self.effects
    }
    /// Resolved types-owned call identity, when this node is a call.
    #[must_use]
    pub const fn resolved_call(&self) -> Option<&ResolvedCallTargetV2> {
        self.resolved_call.as_ref()
    }
    /// Types-owned aggregation of this call itself, excluding argument effects.
    #[must_use]
    pub const fn intrinsic_call_aggregation(&self) -> Option<AggregationClassV1> {
        self.intrinsic_call_aggregation
    }
    /// Checked aggregate placement.
    #[must_use]
    pub const fn aggregate_context(&self) -> &AggregateContextV2 {
        &self.aggregate_context
    }
    /// Closed semantic shape.
    #[must_use]
    pub const fn shape(&self) -> &TypedExprShapeV2 {
        &self.shape
    }
    /// Test-support-only shape substitution used to assemble complete forged
    /// documents without exposing a public expression constructor.
    #[cfg(any(test, feature = "test"))]
    pub(crate) fn forged_with_shape(mut self, shape: TypedExprShapeV2) -> Self {
        self.shape = shape;
        self
    }
    /// Test-only minting seam: replace the capture set of a RelationExists
    /// expression so adversarial-document rails can assemble forged
    /// correlation vectors against the real document finalizer. Compiled
    /// for unit tests or under the `test` feature; never part of a normal build.
    #[cfg(any(test, feature = "test"))]
    #[expect(
        clippy::wildcard_enum_match_arm,
        reason = "future non-RelationExists shapes must take the same rejection"
    )]
    pub(crate) fn with_relation_exists_correlations(
        self,
        correlations: Vec<TypedCorrelationEdgeV2>,
    ) -> Result<Self, FrontendFailureV2> {
        match self.shape {
            TypedExprShapeV2::RelationExists {
                polarity,
                relation_root,
                ..
            } => Ok(Self {
                shape: TypedExprShapeV2::RelationExists {
                    polarity,
                    relation_root,
                    correlations,
                },
                ..self
            }),
            _ => Err(expression_failure(
                "sema.document_validation_failed",
                self.location.location(),
                "correlation reshape requires a RelationExists expression",
            )),
        }
    }
}

/// Bool-proven expression. Construction rejects Unknown and non-Bool facts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedBoolExprV2(TypedExprV2);

impl TypedBoolExprV2 {
    /// Prove a typed expression has the closed Bool value type.
    pub fn try_new(
        expression: TypedExprV2,
        location: LocatedSpanV2,
    ) -> Result<Self, FrontendFailureV2> {
        match expression.type_state() {
            ExpressionTypeStateV2::Known(actual) if actual == &ValueTypeV1::bool() => {
                Ok(Self(expression))
            }
            ExpressionTypeStateV2::Known(_) => Err(expression_failure(
                "sema.non_boolean_predicate",
                location,
                "predicate root must have Bool type",
            )),
            ExpressionTypeStateV2::Unknown(_) => Err(expression_failure(
                "sema.unknown_type_in_closed_compile",
                location,
                "closed predicate root cannot retain an unknown type",
            )),
        }
    }

    /// Borrow the underlying typed expression.
    #[must_use]
    pub const fn expression(&self) -> &TypedExprV2 {
        &self.0
    }
}

pub(crate) fn expression_failure(
    code: &'static str,
    location: LocatedSpanV2,
    message: impl AsRef<str>,
) -> FrontendFailureV2 {
    FrontendFailureV2::one(FrontendDiagnosticV2::new(
        FrontendStageV2::Typing,
        code,
        FrontendSeverityV2::Error,
        location,
        Vec::new(),
        message,
    ))
}
