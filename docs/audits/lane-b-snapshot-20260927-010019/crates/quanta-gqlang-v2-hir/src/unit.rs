//! Top-level HIR carriers: query, declaration, and rule.
//!
//! These EMBED the shared expr/bool/relation/shape nodes by value rather than
//! re-declaring root-local structs (ticket 01 sharing invariant; D2). A
//! `DeclHirV1` and a `RuleHirV1` each carry shared [`ValueExprHirV1`] bodies, a
//! `QueryHirV1` carries a shared [`RelationExprHirV1`] plus a shared
//! [`ResultShapeHirV1`].

use crate::expr::{BoolExprHirV1, ValueExprHirV1};
use crate::id::HirNodeIdV1;
use crate::relation::RelationExprHirV1;
use crate::shape::ResultShapeHirV1;
use crate::span::SpanV1;

/// A query unit: a relation producer plus an optional nested result shape.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// The relation producer (shared relation kernel).
    pub body: RelationExprHirV1,
    /// Optional nested result shape (shared shape kernel). When `None`, the
    /// projection of `body` is the result shape.
    pub result_shape: Option<ResultShapeHirV1>,
}

impl QueryHirV1 {
    /// Construct a query unit.
    #[must_use]
    pub fn new(
        id: HirNodeIdV1,
        span: SpanV1,
        body: RelationExprHirV1,
        result_shape: Option<ResultShapeHirV1>,
    ) -> Self {
        Self {
            id,
            span,
            body,
            result_shape,
        }
    }
}

/// A declaration unit binding a name to a shared value expression body.
///
/// e.g. a view/let/const-style binding. The body is the shared value kernel so
/// declaration-local typed expressions need no root-specific scalar carrier
/// (ticket 01 scenario 3).
#[derive(Clone, Debug, PartialEq)]
pub struct DeclHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// Declared name.
    pub name: String,
    /// Span of the declared name.
    pub name_span: SpanV1,
    /// The shared value expression this declaration is bound to.
    pub body: ValueExprHirV1,
}

impl DeclHirV1 {
    /// Construct a declaration unit.
    #[must_use]
    pub fn new(
        id: HirNodeIdV1,
        span: SpanV1,
        name: String,
        name_span: SpanV1,
        body: ValueExprHirV1,
    ) -> Self {
        Self {
            id,
            span,
            name,
            name_span,
            body,
        }
    }
}

/// A rule unit: a head value expression implied by a boolean body.
///
/// Recursion/least-fixpoint semantics are NOT decided here (ticket 05/07). This
/// is the shared structural carrier only: the head is a shared
/// [`ValueExprHirV1`] and the body a shared [`BoolExprHirV1`].
#[derive(Clone, Debug, PartialEq)]
pub struct RuleHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// Rule name.
    pub name: String,
    /// Span of the rule name.
    pub name_span: SpanV1,
    /// The rule head (shared value kernel).
    pub head: ValueExprHirV1,
    /// The rule body predicate (shared boolean kernel).
    pub body: BoolExprHirV1,
}

impl RuleHirV1 {
    /// Construct a rule unit.
    #[must_use]
    pub fn new(
        id: HirNodeIdV1,
        span: SpanV1,
        name: String,
        name_span: SpanV1,
        head: ValueExprHirV1,
        body: BoolExprHirV1,
    ) -> Self {
        Self {
            id,
            span,
            name,
            name_span,
            head,
            body,
        }
    }
}
