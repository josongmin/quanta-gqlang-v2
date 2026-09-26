//! Nested result-shape construction (the single shared shape authority).
//!
//! Per packet decision D29 and ticket 01, [`ResultShapeHirV1`] is the ONLY
//! frontend-owned structural authority for nested result shaping. Later tickets
//! type it (03), normalize relation producers (04), and derive render semantics
//! from it (07), but MUST NOT introduce a second shape tree. It embeds the
//! shared [`ValueExprHirV1`] for leaf values so scalar shaping reuses the value
//! kernel.

use crate::expr::ValueExprHirV1;
use crate::id::HirNodeIdV1;
use crate::span::SpanV1;

/// A nested result-shape node.
#[derive(Clone, Debug, PartialEq)]
pub struct ResultShapeHirV1 {
    /// Unique identity within the owning tree.
    pub id: HirNodeIdV1,
    /// Source byte range.
    pub span: SpanV1,
    /// The structural payload.
    pub kind: ResultShapeKindV1,
}

impl ResultShapeHirV1 {
    /// Pair a kind with its id and span.
    #[must_use]
    pub fn new(id: HirNodeIdV1, span: SpanV1, kind: ResultShapeKindV1) -> Self {
        Self { id, span, kind }
    }
}

/// Structural payload of a [`ResultShapeHirV1`].
#[derive(Clone, Debug, PartialEq)]
pub enum ResultShapeKindV1 {
    /// A leaf scalar value position, carrying a shared value expression.
    Leaf(Box<ValueExprHirV1>),
    /// A named record of nested shapes, e.g. `{ id: ..., children: [...] }`.
    Record(ShapeRecordV1),
    /// A list of a nested shape, e.g. `[ <child-shape> ]`.
    List(ShapeListV1),
}

/// A named field within a result-shape record.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeFieldV1 {
    /// Output field name.
    pub name: String,
    /// Span of the field name.
    pub name_span: SpanV1,
    /// Nested shape for this field.
    pub shape: ResultShapeHirV1,
}

/// A record of nested result shapes.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeRecordV1 {
    /// Ordered fields; order is the output order.
    pub fields: Vec<ShapeFieldV1>,
}

/// A list whose elements all take one nested shape.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapeListV1 {
    /// The per-element nested shape.
    pub element: Box<ResultShapeHirV1>,
}
