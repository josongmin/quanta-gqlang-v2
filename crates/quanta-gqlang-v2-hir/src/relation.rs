//! Relation expression HIR nodes (structure only).
//!
//! [`RelationExprHirV1`] captures relation *sources*, *projection* with aliases,
//! and *composition* as STRUCTURE ONLY. Per packet decision D12 and ticket 01
//! non-goals, join/algebra/normalization semantics belong to ticket 04 and are
//! NOT decided here. Projection items embed the shared [`ValueExprHirV1`] so the
//! value kernel is reused, and filters embed the shared [`BoolExprHirV1`].

use crate::expr::{BoolExprHirV1, ValueExprHirV1};
use crate::id::HirNodeIdV1;
use crate::span::SpanV1;

/// A relation expression node.
#[derive(Clone, Debug, PartialEq)]
pub struct RelationExprHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// The structural payload.
    pub kind: RelationExprKindV1,
}

impl RelationExprHirV1 {
    /// Pair a kind with its id and span.
    #[must_use]
    pub fn new(id: HirNodeIdV1, span: SpanV1, kind: RelationExprKindV1) -> Self {
        Self { id, span, kind }
    }
}

/// Structural payload of a [`RelationExprHirV1`].
#[derive(Clone, Debug, PartialEq)]
pub enum RelationExprKindV1 {
    /// A reference to a named relation source (table/view/binding), optionally
    /// aliased, e.g. `users u`.
    Source(RelationSourceV1),
    /// Projection over an input relation producing aliased columns, e.g.
    /// `<input> | project { id: u.id, tags: [u.t1, u.t2] }`.
    Project(ProjectionV1),
    /// A filter over an input relation, e.g. `<input> | where x > 1`. Embeds the
    /// shared boolean kernel. No algebra is applied here.
    Filter(RelationFilterV1),
    /// Composition of relations, recorded structurally as an open-tagged
    /// operator over operands. Algebra/normalization of this is ticket 04.
    Compose(CompositionV1),
}

/// A reference to a named relation source.
#[derive(Clone, Debug, PartialEq)]
pub struct RelationSourceV1 {
    /// Source name as written.
    pub name: String,
    /// Optional binding alias (the `u` in `users u`).
    pub alias: Option<AliasV1>,
}

/// A binding alias.
#[derive(Clone, Debug, PartialEq)]
pub struct AliasV1 {
    /// Alias name as written.
    pub name: String,
    /// Span of the alias name.
    pub span: SpanV1,
}

/// A single projection item, `value AS alias`.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectionItemV1 {
    /// The projected value expression (shared value kernel).
    pub value: ValueExprHirV1,
    /// Output alias for this item, if written.
    pub alias: Option<AliasV1>,
}

/// Projection over an input relation.
#[derive(Clone, Debug, PartialEq)]
pub struct ProjectionV1 {
    /// The input relation being projected.
    pub input: Box<RelationExprHirV1>,
    /// Ordered projection items.
    pub items: Vec<ProjectionItemV1>,
}

/// A filter over an input relation.
#[derive(Clone, Debug, PartialEq)]
pub struct RelationFilterV1 {
    /// The input relation being filtered.
    pub input: Box<RelationExprHirV1>,
    /// The shared boolean predicate.
    pub predicate: Box<BoolExprHirV1>,
}

/// Open symbolic composition operator tag (structure only).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompositionOpV1 {
    /// Composition keyword/symbol as written (e.g. `"join"`, `"union"`).
    pub symbol: String,
}

/// Structural composition of relation operands.
#[derive(Clone, Debug, PartialEq)]
pub struct CompositionV1 {
    /// The open composition operator tag. No join algebra is applied here.
    pub op: CompositionOpV1,
    /// Operand relations, in written order.
    pub operands: Vec<RelationExprHirV1>,
}
