//! Fail-closed integrity checking for HIR trees.
//!
//! This is the crate's integrity gate (ticket 01 scenario 5). It walks any HIR
//! root via the [`crate::visit`] visitor and enforces three load-bearing
//! invariants that downstream tickets rely on:
//!
//! 1. every node id is unique within the tree,
//! 2. every span is well formed (`start <= end`),
//! 3. every span lies within the declared source length (`end <= source_len`).
//!
//! Violations are returned as typed [`HirIntegrityErrorV1`] values with stable
//! `code()` strings and `Display` text, never panics. [`check_tree`] is the
//! checked constructor: it returns `Ok(CheckedHirV1)` only when all invariants
//! hold, so a `CheckedHirV1` is a proof token that the embedded tree is sound.

use crate::expr::{BoolExprHirV1, ValueExprHirV1};
use crate::id::HirNodeIdV1;
use crate::item::{
    CatalogDeclHirV1, CompositionHirV1, GraphQueryHirV1, LegalityHirV1, MetaHirV1,
    PresentationHirV1, ProgramItemHirV1,
};
use crate::relation::RelationExprHirV1;
use crate::shape::ResultShapeHirV1;
use crate::span::SpanV1;
use crate::unit::{DeclHirV1, QueryHirV1, RuleHirV1};
use crate::visit::HirVisitorV1;
use std::collections::HashMap;
use std::fmt;

/// A borrowed view of any HIR root, so the integrity gate (and any generic
/// consumer) can traverse a tree without knowing its concrete top kind.
#[derive(Clone, Copy, Debug)]
pub enum HirRootRefV1<'a> {
    /// A value expression root.
    Value(&'a ValueExprHirV1),
    /// A boolean expression root.
    Bool(&'a BoolExprHirV1),
    /// A relation expression root.
    Relation(&'a RelationExprHirV1),
    /// A result-shape root.
    Shape(&'a ResultShapeHirV1),
    /// A query unit root.
    Query(&'a QueryHirV1),
    /// A declaration unit root.
    Decl(&'a DeclHirV1),
    /// A rule unit root.
    Rule(&'a RuleHirV1),
    /// A unified program-item root (any of the 18 families).
    Item(&'a ProgramItemHirV1),
    /// A graph query (FIND) root.
    GraphQuery(&'a GraphQueryHirV1),
    /// A legality check (CHECK) root.
    Legality(&'a LegalityHirV1),
    /// A presentation verb root.
    Presentation(&'a PresentationHirV1),
    /// A catalog declaration root.
    CatalogDecl(&'a CatalogDeclHirV1),
    /// A composition root.
    Composition(&'a CompositionHirV1),
    /// A meta directive root.
    Meta(&'a MetaHirV1),
}

impl<'a> HirRootRefV1<'a> {
    /// Drive a visitor over this root.
    pub fn accept<V: HirVisitorV1>(&self, visitor: &mut V) {
        match self {
            Self::Value(node) => visitor.visit_value(node),
            Self::Bool(node) => visitor.visit_bool(node),
            Self::Relation(node) => visitor.visit_relation(node),
            Self::Shape(node) => visitor.visit_shape(node),
            Self::Query(node) => visitor.visit_query(node),
            Self::Decl(node) => visitor.visit_decl(node),
            Self::Rule(node) => visitor.visit_rule(node),
            Self::Item(node) => visitor.visit_item(node),
            Self::GraphQuery(node) => visitor.visit_graph_query(node),
            Self::Legality(node) => visitor.visit_legality(node),
            Self::Presentation(node) => visitor.visit_presentation(node),
            Self::CatalogDecl(node) => visitor.visit_catalog_decl(node),
            Self::Composition(node) => visitor.visit_composition(node),
            Self::Meta(node) => visitor.visit_meta(node),
        }
    }
}

/// A typed integrity violation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HirIntegrityErrorV1 {
    /// The same node id appears on two distinct nodes.
    DuplicateNodeId {
        /// The colliding id.
        id: HirNodeIdV1,
        /// Span of the first node seen with this id.
        first_span: SpanV1,
        /// Span of the second (colliding) node.
        second_span: SpanV1,
    },
    /// A span has `start > end`.
    InvertedSpan {
        /// The offending node id.
        id: HirNodeIdV1,
        /// The inverted span as found.
        span: SpanV1,
    },
    /// A span ends past the declared source length.
    SpanOutOfBounds {
        /// The offending node id.
        id: HirNodeIdV1,
        /// The offending span.
        span: SpanV1,
        /// The declared source length the span exceeded.
        source_len: usize,
    },
}

impl HirIntegrityErrorV1 {
    /// Stable machine-readable diagnostic code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::DuplicateNodeId { .. } => "hir.integrity.duplicate_node_id",
            Self::InvertedSpan { .. } => "hir.integrity.inverted_span",
            Self::SpanOutOfBounds { .. } => "hir.integrity.span_out_of_bounds",
        }
    }
}

impl fmt::Display for HirIntegrityErrorV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateNodeId {
                id,
                first_span,
                second_span,
            } => write!(
                f,
                "{}: duplicate HIR node id {} (first at {}..{}, again at {}..{})",
                self.code(),
                id.raw(),
                first_span.start,
                first_span.end,
                second_span.start,
                second_span.end,
            ),
            Self::InvertedSpan { id, span } => write!(
                f,
                "{}: HIR node id {} has inverted span {}..{} (start > end)",
                self.code(),
                id.raw(),
                span.start,
                span.end,
            ),
            Self::SpanOutOfBounds {
                id,
                span,
                source_len,
            } => write!(
                f,
                "{}: HIR node id {} span {}..{} exceeds source length {}",
                self.code(),
                id.raw(),
                span.start,
                span.end,
                source_len,
            ),
        }
    }
}

impl std::error::Error for HirIntegrityErrorV1 {}

/// A tree that has passed [`check_tree`]. Holding one is a proof that the
/// referenced root satisfies all HIR integrity invariants for `source_len`.
#[derive(Clone, Copy, Debug)]
pub struct CheckedHirV1<'a> {
    root: HirRootRefV1<'a>,
    source_len: usize,
    node_count: usize,
}

impl<'a> CheckedHirV1<'a> {
    /// The checked root.
    #[must_use]
    pub fn root(&self) -> HirRootRefV1<'a> {
        self.root
    }

    /// The source length the tree was validated against.
    #[must_use]
    pub fn source_len(&self) -> usize {
        self.source_len
    }

    /// Number of nodes that carry an id in the tree.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.node_count
    }
}

/// Checked constructor: validate every invariant on `root` against `source_len`.
///
/// Returns `Ok(CheckedHirV1)` when all ids are unique, all spans are well
/// formed, and all spans fit within `source_len`. Otherwise returns the first
/// typed [`HirIntegrityErrorV1`] encountered in deterministic pre-order. The
/// caller is fully fail-closed: no `CheckedHirV1` is produced for an unsound
/// tree.
pub fn check_tree(
    root: HirRootRefV1<'_>,
    source_len: usize,
) -> Result<CheckedHirV1<'_>, HirIntegrityErrorV1> {
    let mut collector = IntegrityCollector::new(source_len);
    root.accept(&mut collector);
    match collector.error {
        Some(error) => Err(error),
        None => Ok(CheckedHirV1 {
            root,
            source_len,
            node_count: collector.seen.len(),
        }),
    }
}

/// Collect every node's `(id, span)` and check invariants. Stops recording new
/// errors after the first, but traversal still completes (cheap, bounded).
struct IntegrityCollector {
    source_len: usize,
    seen: HashMap<HirNodeIdV1, SpanV1>,
    error: Option<HirIntegrityErrorV1>,
}

impl IntegrityCollector {
    fn new(source_len: usize) -> Self {
        Self {
            source_len,
            seen: HashMap::new(),
            error: None,
        }
    }

    fn record(&mut self, id: HirNodeIdV1, span: SpanV1) {
        if self.error.is_some() {
            return;
        }
        if !span.is_well_formed() {
            self.error = Some(HirIntegrityErrorV1::InvertedSpan { id, span });
            return;
        }
        if span.end > self.source_len {
            self.error = Some(HirIntegrityErrorV1::SpanOutOfBounds {
                id,
                span,
                source_len: self.source_len,
            });
            return;
        }
        if let Some(first_span) = self.seen.get(&id) {
            self.error = Some(HirIntegrityErrorV1::DuplicateNodeId {
                id,
                first_span: *first_span,
                second_span: span,
            });
            return;
        }
        let _ = self.seen.insert(id, span);
    }
}

impl HirVisitorV1 for IntegrityCollector {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_value(self, node);
    }
    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_bool(self, node);
    }
    fn visit_relation(&mut self, node: &RelationExprHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_relation(self, node);
    }
    fn visit_shape(&mut self, node: &ResultShapeHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_shape(self, node);
    }
    fn visit_query(&mut self, node: &QueryHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_query(self, node);
    }
    fn visit_decl(&mut self, node: &DeclHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_decl(self, node);
    }
    fn visit_rule(&mut self, node: &RuleHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_rule(self, node);
    }
    fn visit_graph_query(&mut self, node: &GraphQueryHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_graph_query(self, node);
    }
    fn visit_legality(&mut self, node: &LegalityHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_legality(self, node);
    }
    fn visit_presentation(&mut self, node: &PresentationHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_presentation(self, node);
    }
    fn visit_catalog_decl(&mut self, node: &CatalogDeclHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_catalog_decl(self, node);
    }
    fn visit_composition(&mut self, node: &CompositionHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_composition(self, node);
    }
    fn visit_meta(&mut self, node: &MetaHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_meta(self, node);
    }
}
