//! Fail-closed integrity checking for HIR trees.
//!
//! This is the crate's integrity gate (ticket 01 scenario 5). Before the
//! recursive visitor runs, a stack-safe shared depth pass rejects excessive
//! expression, relation, or result-shape nesting. It then enforces three
//! load-bearing invariants that downstream tickets rely on:
//!
//! 1. every node id is unique within the tree,
//! 2. every span is well formed (`start <= end`),
//! 3. every span lies within the declared source length (`end <= source_len`).
//!
//! Violations are returned as typed [`HirIntegrityErrorV1`] values with stable
//! `code()` strings and `Display` text, never panics. [`check_tree`] is the
//! checked constructor: it returns `Ok(CheckedHirV1)` only when all invariants
//! hold, so a `CheckedHirV1` is a proof token that the embedded tree is sound.

use crate::depth::{check_depths_v2, HirDepthDimensionV2};
use crate::expr::{BoolExprHirV1, BoolExprKindV1, CallArgHirV1, ValueExprHirV1, ValueExprKindV1};
use crate::id::HirNodeIdV1;
use crate::item::{
    CatalogDeclHirV1, CatalogPayloadV1, CompositionBodyV1, CompositionHirV1, CompositionProducerV1,
    ExplainHirV1, GraphQueryHirV1, LegalityHirV1, MetaHirV1, PresentationHirV1, ProgramItemHirV1,
    SampleHirV1, ViewBodyV1,
};
use crate::relation::{JoinHirV1, RelationExprHirV1, RelationExprKindV1};
use crate::shape::{ResultShapeHirV1, ResultShapeKindV1};
use crate::span::{SourceIdV2, SpanV1};
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
    /// An id-less structural child span is inverted.
    InvertedAuxiliarySpan {
        /// The offending span.
        span: SpanV1,
    },
    /// An id-less structural child span exceeds the source.
    AuxiliarySpanOutOfBounds {
        /// The offending span.
        span: SpanV1,
        /// Registered source length.
        source_len: usize,
    },
    /// A programmatically supplied tree exceeds a recursive traversal ceiling.
    ExcessiveDepth {
        dimension: HirDepthDimensionV2,
        span: SpanV1,
        limit: usize,
        observed: usize,
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
            Self::InvertedAuxiliarySpan { .. } => "hir.integrity.inverted_auxiliary_span",
            Self::AuxiliarySpanOutOfBounds { .. } => "hir.integrity.auxiliary_span_out_of_bounds",
            Self::ExcessiveDepth { .. } => "hir.resource_limit_exceeded",
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
            Self::InvertedAuxiliarySpan { span } => write!(
                f,
                "{}: auxiliary HIR span {}..{} is inverted",
                self.code(),
                span.start,
                span.end,
            ),
            Self::AuxiliarySpanOutOfBounds { span, source_len } => write!(
                f,
                "{}: auxiliary HIR span {}..{} exceeds source length {}",
                self.code(),
                span.start,
                span.end,
                source_len,
            ),
            Self::ExcessiveDepth {
                dimension,
                limit,
                observed,
                ..
            } => write!(
                f,
                "{}: {} exceeds safe limit {} (observed {})",
                self.code(),
                dimension.as_str(),
                limit,
                observed,
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
/// Returns `Ok(CheckedHirV1)` when depth is safe, all ids are unique, all
/// spans are well formed, and all spans fit within `source_len`. Resource
/// admission runs first; within admitted trees integrity returns the first
/// typed [`HirIntegrityErrorV1`] in deterministic pre-order. No proof is
/// produced for a rejected tree.
///
/// # Who must call this
///
/// The source parser uses [`crate::HirBuilderV1`] and bounds depth before
/// returning HIR. Pipeline additionally checks the complete parsed document
/// for shared identity. This borrowed seal is for hand-built HIR roots used by
/// an embedder, downstream frontend, or test fixture; skipping it leaves a raw
/// tree unchecked.
pub fn check_tree(
    root: HirRootRefV1<'_>,
    source_len: usize,
) -> Result<CheckedHirV1<'_>, HirIntegrityErrorV1> {
    check_safe_depth(root)?;
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

/// One parsed HIR document with a single source-length authority.
#[derive(Clone, Debug, PartialEq)]
pub struct HirDocumentV1 {
    /// Source identity carried into later diagnostics.
    pub source_id: SourceIdV2,
    /// Byte length of the source.
    pub source_len: usize,
    /// Top-level items in authored order.
    pub items: Vec<ProgramItemHirV1>,
}

/// Document integrity diagnostic. The existing stable tree diagnostic remains
/// the single error vocabulary for node identity and span failures.
pub type HirIntegrityDiagnosticV2 = HirIntegrityErrorV1;

/// Borrowed proof that every root in one document passed one shared integrity
/// traversal.
#[derive(Clone, Copy, Debug)]
pub struct CheckedHirDocumentV2<'a> {
    document: &'a HirDocumentV1,
    node_count: usize,
}

/// Owned proof that a complete HIR document passed the document-wide
/// integrity traversal exactly once.
///
/// The document and its proof metadata cannot be separated through the public
/// API. Downstream document transactions can borrow a [`CheckedHirDocumentV2`]
/// without repeating integrity validation.
#[derive(Clone, Debug)]
pub struct CheckedHirDocumentOwnedV2 {
    document: HirDocumentV1,
    node_count: usize,
}

impl CheckedHirDocumentOwnedV2 {
    /// Consume and validate one document, retaining the validated document.
    pub fn try_new(document: HirDocumentV1) -> Result<Self, HirIntegrityDiagnosticV2> {
        let node_count = match check_document_v2(&document).map(|checked| checked.node_count()) {
            Ok(count) => count,
            Err(error) => {
                drop_document_iterative(document);
                return Err(error);
            }
        };
        Ok(Self {
            document,
            node_count,
        })
    }

    /// Borrow the checked document and its retained proof metadata.
    #[must_use]
    pub const fn as_checked(&self) -> CheckedHirDocumentV2<'_> {
        CheckedHirDocumentV2 {
            document: &self.document,
            node_count: self.node_count,
        }
    }

    /// Borrow the underlying document without discarding its proof carrier.
    #[must_use]
    pub const fn document(&self) -> &HirDocumentV1 {
        &self.document
    }

    /// Total id-bearing nodes across all roots.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.node_count
    }
}

impl<'a> CheckedHirDocumentV2<'a> {
    /// Checked document.
    #[must_use]
    pub const fn document(&self) -> &'a HirDocumentV1 {
        self.document
    }

    /// Total id-bearing nodes across all roots.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.node_count
    }
}

/// Validate every document root with one shared id set and source bound.
pub fn check_document_v2(
    document: &HirDocumentV1,
) -> Result<CheckedHirDocumentV2<'_>, HirIntegrityDiagnosticV2> {
    let mut collector = IntegrityCollector::new(document.source_len);
    for item in &document.items {
        check_safe_depth(HirRootRefV1::Item(item))?;
        HirRootRefV1::Item(item).accept(&mut collector);
        if collector.error.is_some() {
            break;
        }
    }
    match collector.error {
        Some(error) => Err(error),
        None => Ok(CheckedHirDocumentV2 {
            document,
            node_count: collector.seen.len(),
        }),
    }
}

fn check_safe_depth(root: HirRootRefV1<'_>) -> Result<(), HirIntegrityErrorV1> {
    check_depths_v2(
        root,
        crate::MAX_SAFE_HIR_EXPRESSION_DEPTH_V2,
        crate::MAX_SAFE_RELATION_DEPTH_V2,
        crate::MAX_SAFE_EXPRESSION_DEPTH_V2,
    )
    .map_err(|error| HirIntegrityErrorV1::ExcessiveDepth {
        dimension: error.dimension,
        span: error.span,
        limit: error.limit,
        observed: error.observed,
    })
}

/// A rejected document can contain a deep subtree even when its first error
/// occurs in an earlier root. Drain every recursive child with an explicit
/// worklist so error return itself cannot overflow during destruction.
fn drop_document_iterative(document: HirDocumentV1) {
    enum Owned {
        Item(ProgramItemHirV1),
        Query(QueryHirV1),
        Value(ValueExprHirV1),
        Bool(BoolExprHirV1),
        Relation(RelationExprHirV1),
        Shape(ResultShapeHirV1),
    }

    let mut stack: Vec<Owned> = document.items.into_iter().map(Owned::Item).collect();
    while let Some(node) = stack.pop() {
        match node {
            Owned::Item(item) => match item {
                ProgramItemHirV1::Query(query) => stack.push(Owned::Query(*query)),
                ProgramItemHirV1::Decl(decl) => stack.push(Owned::Value(decl.body)),
                ProgramItemHirV1::Rule(rule) => {
                    stack.push(Owned::Bool(rule.body));
                    stack.push(Owned::Value(rule.head));
                }
                ProgramItemHirV1::CatalogDecl(decl) => {
                    if let CatalogPayloadV1::View(ViewBodyV1::Inline(query)) = decl.payload {
                        stack.push(Owned::Query(*query));
                    }
                }
                ProgramItemHirV1::Composition(composition) => {
                    for binding in composition.bindings {
                        if let CompositionProducerV1::Query(query) = binding.producer {
                            stack.push(Owned::Query(*query));
                        }
                    }
                    if let CompositionBodyV1::Query(query) = composition.body {
                        stack.push(Owned::Query(*query));
                    }
                }
                ProgramItemHirV1::GraphQuery(_)
                | ProgramItemHirV1::Legality(_)
                | ProgramItemHirV1::Presentation(_)
                | ProgramItemHirV1::Meta(_) => {}
            },
            Owned::Query(query) => {
                if let Some(shape) = query.result_shape {
                    stack.push(Owned::Shape(shape));
                }
                stack.push(Owned::Relation(query.body));
            }
            Owned::Value(value) => match value.kind {
                ValueExprKindV1::Literal(_) | ValueExprKindV1::Symbol(_) => {}
                ValueExprKindV1::Path(path) => stack.push(Owned::Value(*path.base)),
                ValueExprKindV1::Unary(unary) => stack.push(Owned::Value(*unary.operand)),
                ValueExprKindV1::Binary(binary) => {
                    stack.push(Owned::Value(*binary.rhs));
                    stack.push(Owned::Value(*binary.lhs));
                }
                ValueExprKindV1::Call(call) => {
                    for arg in call.into_args() {
                        if let CallArgHirV1::Expr(value) = arg {
                            stack.push(Owned::Value(value));
                        }
                    }
                }
                ValueExprKindV1::Paren(inner) => stack.push(Owned::Value(*inner)),
                ValueExprKindV1::Record(record) => {
                    stack.extend(
                        record
                            .fields
                            .into_iter()
                            .map(|field| Owned::Value(field.value)),
                    );
                }
                ValueExprKindV1::List(list) => {
                    stack.extend(list.elements.into_iter().map(Owned::Value));
                }
            },
            Owned::Bool(boolean) => match boolean.kind {
                BoolExprKindV1::Literal(_) => {}
                BoolExprKindV1::Value(value) => stack.push(Owned::Value(*value)),
                BoolExprKindV1::Compare(compare) => {
                    stack.push(Owned::Value(*compare.rhs));
                    stack.push(Owned::Value(*compare.lhs));
                }
                BoolExprKindV1::And(binary) | BoolExprKindV1::Or(binary) => {
                    stack.push(Owned::Bool(*binary.rhs));
                    stack.push(Owned::Bool(*binary.lhs));
                }
                BoolExprKindV1::Not(inner) | BoolExprKindV1::Paren(inner) => {
                    stack.push(Owned::Bool(*inner));
                }
                BoolExprKindV1::LogicalQuantifier(quantifier) => {
                    stack.push(Owned::Bool(*quantifier.body));
                }
                BoolExprKindV1::RelationExists(exists) => {
                    stack.push(Owned::Relation(*exists.relation));
                }
            },
            Owned::Relation(relation) => match relation.kind {
                RelationExprKindV1::Source(_) => {}
                RelationExprKindV1::Project(project) => {
                    let (input, items) = project.into_parts();
                    stack.extend(items.into_iter().map(|item| Owned::Value(item.value)));
                    stack.push(Owned::Relation(input));
                }
                RelationExprKindV1::Filter(filter) => {
                    stack.push(Owned::Bool(*filter.predicate));
                    stack.push(Owned::Relation(*filter.input));
                }
                RelationExprKindV1::Join(join) => match join {
                    JoinHirV1::Inner { left, right, on } => {
                        stack.push(Owned::Bool(on));
                        stack.push(Owned::Relation(*right));
                        stack.push(Owned::Relation(*left));
                    }
                    JoinHirV1::Cross { left, right } => {
                        stack.push(Owned::Relation(*right));
                        stack.push(Owned::Relation(*left));
                    }
                },
                RelationExprKindV1::GroupBy(group) => {
                    let (input, keys) = group.into_parts();
                    stack.extend(keys.into_iter().map(Owned::Value));
                    stack.push(Owned::Relation(input));
                }
                RelationExprKindV1::OrderBy(order) => {
                    let (input, keys) = order.into_parts();
                    stack.extend(keys.into_iter().map(|key| Owned::Value(key.value)));
                    stack.push(Owned::Relation(input));
                }
            },
            Owned::Shape(shape) => match shape.kind {
                ResultShapeKindV1::Leaf(value) => stack.push(Owned::Value(*value)),
                ResultShapeKindV1::Record(record) => {
                    stack.extend(
                        record
                            .fields
                            .into_iter()
                            .map(|field| Owned::Shape(field.shape)),
                    );
                }
                ResultShapeKindV1::List(list) => stack.push(Owned::Shape(*list.element)),
            },
        }
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

    fn record_auxiliary_span(&mut self, span: SpanV1) {
        if self.error.is_some() {
            return;
        }
        if !span.is_well_formed() {
            self.error = Some(HirIntegrityErrorV1::InvertedAuxiliarySpan { span });
        } else if span.end > self.source_len {
            self.error = Some(HirIntegrityErrorV1::AuxiliarySpanOutOfBounds {
                span,
                source_len: self.source_len,
            });
        }
    }
}

impl HirVisitorV1 for IntegrityCollector {
    fn visit_auxiliary_span(&mut self, span: SpanV1) {
        self.record_auxiliary_span(span);
    }
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_value(self, node);
    }
    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_bool(self, node);
    }
    fn visit_call_star(&mut self, span: SpanV1) {
        self.record_auxiliary_span(span);
    }

    fn visit_logical_binder(&mut self, span: SpanV1) {
        self.record_auxiliary_span(span);
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
    fn visit_sample(&mut self, node: &SampleHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_sample(self, node);
    }
    fn visit_explain(&mut self, node: &ExplainHirV1) {
        self.record(node.id, node.span);
        crate::visit::walk_explain(self, node);
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
