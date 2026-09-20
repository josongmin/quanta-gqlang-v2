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
    /// Inner or explicit cross join.
    Join(JoinHirV1),
    /// Grouping keys over an input relation.
    GroupBy(GroupByHirV1),
    /// Ordered sort keys over an input relation.
    OrderBy(OrderByHirV1),
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
    items: NonEmptyProjectionItemsV1,
}

impl ProjectionV1 {
    pub fn try_new(
        input: RelationExprHirV1,
        items: Vec<ProjectionItemV1>,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            input: Box::new(input),
            items: NonEmptyProjectionItemsV1::try_new(items)?,
        })
    }

    #[must_use]
    pub fn items(&self) -> &[ProjectionItemV1] {
        self.items.as_slice()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NonEmptyProjectionItemsV1(Vec<ProjectionItemV1>);

impl NonEmptyProjectionItemsV1 {
    pub fn try_new(items: Vec<ProjectionItemV1>) -> Result<Self, &'static str> {
        if items.is_empty() {
            Err("projection requires at least one item")
        } else {
            Ok(Self(items))
        }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[ProjectionItemV1] {
        &self.0
    }
}

/// A filter over an input relation.
#[derive(Clone, Debug, PartialEq)]
pub struct RelationFilterV1 {
    /// The input relation being filtered.
    pub input: Box<RelationExprHirV1>,
    /// The shared boolean predicate.
    pub predicate: Box<BoolExprHirV1>,
}

/// A source-reachable join. The enum payload makes an inner join without an
/// `on` predicate, or a cross join with one, unrepresentable.
#[derive(Clone, Debug, PartialEq)]
pub enum JoinHirV1 {
    /// `join <source> on <predicate>`.
    Inner {
        /// Left pipeline input.
        left: Box<RelationExprHirV1>,
        /// Right relation source.
        right: Box<RelationExprHirV1>,
        /// Required join predicate.
        on: BoolExprHirV1,
    },
    /// `cross join <source>`.
    Cross {
        /// Left pipeline input.
        left: Box<RelationExprHirV1>,
        /// Right relation source.
        right: Box<RelationExprHirV1>,
    },
}

/// Grouping keys over one relation input.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupByHirV1 {
    /// Input relation.
    pub input: Box<RelationExprHirV1>,
    /// Non-empty ordered key expressions as authored.
    keys: NonEmptyRelationKeysV1<ValueExprHirV1>,
}

impl GroupByHirV1 {
    /// Construct a grouping stage, rejecting an empty key set.
    pub fn try_new(
        input: RelationExprHirV1,
        keys: Vec<ValueExprHirV1>,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            input: Box::new(input),
            keys: NonEmptyRelationKeysV1::try_new(keys)?,
        })
    }

    /// Ordered non-empty grouping keys.
    #[must_use]
    pub fn keys(&self) -> &[ValueExprHirV1] {
        self.keys.as_slice()
    }
}

/// Sort direction attached directly to an order key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirectionHirV1 {
    /// Ascending order, including an omitted source direction.
    Asc,
    /// Descending order.
    Desc,
}

/// One order key and its direction.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderKeyHirV1 {
    /// Key expression.
    pub value: ValueExprHirV1,
    /// Canonical direction.
    pub direction: SortDirectionHirV1,
}

/// Ordered keys over one relation input.
#[derive(Clone, Debug, PartialEq)]
pub struct OrderByHirV1 {
    /// Input relation.
    pub input: Box<RelationExprHirV1>,
    /// Non-empty ordered keys.
    keys: NonEmptyRelationKeysV1<OrderKeyHirV1>,
}

impl OrderByHirV1 {
    /// Construct an ordering stage, rejecting an empty key set.
    pub fn try_new(
        input: RelationExprHirV1,
        keys: Vec<OrderKeyHirV1>,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            input: Box::new(input),
            keys: NonEmptyRelationKeysV1::try_new(keys)?,
        })
    }

    /// Ordered non-empty sort keys.
    #[must_use]
    pub fn keys(&self) -> &[OrderKeyHirV1] {
        self.keys.as_slice()
    }
}

/// Private-field, checked non-empty relation-key carrier.
#[derive(Clone, Debug, PartialEq)]
pub struct NonEmptyRelationKeysV1<T>(Vec<T>);

impl<T> NonEmptyRelationKeysV1<T> {
    /// Validate and construct a non-empty key list.
    pub fn try_new(keys: Vec<T>) -> Result<Self, &'static str> {
        if keys.is_empty() {
            Err("relation stage requires at least one key")
        } else {
            Ok(Self(keys))
        }
    }

    /// Borrow keys in authored order.
    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }
}
