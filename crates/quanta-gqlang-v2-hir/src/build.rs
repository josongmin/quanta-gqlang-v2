//! Ergonomic, id-allocating builders for HIR trees.
//!
//! The v3 surface parser lowers source through this builder
//! (`surface::parse_document` / `parse_program`). Tests and embedders may also
//! construct trees programmatically. [`HirBuilderV1`] owns a single
//! [`HirNodeIdGen`] and mints a fresh unique id for every node it constructs, so
//! trees built entirely through one builder satisfy the id-uniqueness invariant
//! by construction. Spans are always caller-supplied (the builder never invents
//! source offsets).

use crate::expr::{
    BinaryBoolV1, LogicalQuantifierHirV1, LogicalQuantifierKindV1, NonEmptyBindersV1,
    NonEmptyPathSegmentsV1, RelationExistsHirV1, RelationExistsPolarityV1,
};
use crate::expr::{
    BinaryExprV1, BinaryOpV1, BoolExprHirV1, BoolExprKindV1, CallArgsHirV1, CallExprV1,
    CalleeRefV1, CompareExprV1, CompareOpV1, ListExprV1, LiteralV1, PathAccessV1, PathSegmentV1,
    RecordExprV1, RecordFieldV1, SymbolRefV1, UnaryExprV1, UnaryOpV1, ValueExprHirV1,
    ValueExprKindV1,
};
use crate::id::{HirNodeIdGen, HirNodeIdV1};
use crate::item::{
    CatalogDeclHirV1, CatalogPayloadV1, CompositionBindingV1, CompositionBodyV1, CompositionHirV1,
    CompositionProducerV1, GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1,
    LegalityHirV1, MetaHirV1, MetaKindV1, PresentationHirV1, PresentationKindV1,
};
use crate::relation::{
    AliasV1, GroupByHirV1, JoinHirV1, LimitHirV1, OrderByHirV1, OrderKeyHirV1, ProjectionItemV1,
    ProjectionV1, RelationExprHirV1, RelationExprKindV1, RelationFilterV1, RelationSourceV1,
};
use crate::shape::{ResultShapeHirV1, ResultShapeKindV1, ShapeFieldV1, ShapeListV1, ShapeRecordV1};
use crate::span::SpanV1;
use crate::unit::{DeclHirV1, QueryHirV1, RuleHirV1};

/// An id-allocating HIR builder.
///
/// Every `*_at` constructor mints one fresh id. Construct one builder per tree;
/// because ids are unique within a generator, the resulting tree passes
/// [`crate::integrity::check_tree`]'s uniqueness invariant by construction.
#[derive(Debug, Default)]
pub struct HirBuilderV1 {
    ids: HirNodeIdGen,
}

impl HirBuilderV1 {
    /// A fresh builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ids: HirNodeIdGen::new(),
        }
    }

    /// Mint the next raw node id, for callers assembling nodes by hand.
    pub fn next_id(&self) -> HirNodeIdV1 {
        self.ids.next()
    }

    /// Number of nodes built so far (ids minted).
    #[must_use]
    pub fn minted(&self) -> u32 {
        self.ids.minted()
    }

    // ---- value expressions -------------------------------------------------

    /// Integer literal value.
    pub fn int(&self, raw: impl Into<String>, span: SpanV1) -> ValueExprHirV1 {
        self.value(span, ValueExprKindV1::Literal(LiteralV1::Int(raw.into())))
    }

    /// Float literal value.
    pub fn float(&self, raw: impl Into<String>, span: SpanV1) -> ValueExprHirV1 {
        self.value(span, ValueExprKindV1::Literal(LiteralV1::Float(raw.into())))
    }

    /// String literal value.
    pub fn string(&self, content: impl Into<String>, span: SpanV1) -> ValueExprHirV1 {
        self.value(
            span,
            ValueExprKindV1::Literal(LiteralV1::Str(content.into())),
        )
    }

    /// Boolean literal value.
    pub fn bool_lit(&self, value: bool, span: SpanV1) -> ValueExprHirV1 {
        self.value(span, ValueExprKindV1::Literal(LiteralV1::Bool(value)))
    }

    /// Null literal value.
    pub fn null(&self, span: SpanV1) -> ValueExprHirV1 {
        self.value(span, ValueExprKindV1::Literal(LiteralV1::Null))
    }

    /// Bare symbol reference.
    pub fn symbol(&self, name: impl Into<String>, span: SpanV1) -> ValueExprHirV1 {
        self.value(
            span,
            ValueExprKindV1::Symbol(SymbolRefV1 { name: name.into() }),
        )
    }

    /// Path access `base.seg0.seg1...` over an existing base expression.
    pub fn try_path(
        &self,
        base: ValueExprHirV1,
        segments: Vec<PathSegmentV1>,
        span: SpanV1,
    ) -> Result<ValueExprHirV1, &'static str> {
        let segments = NonEmptyPathSegmentsV1::try_new(segments)?;
        Ok(self.value(
            span,
            ValueExprKindV1::Path(PathAccessV1 {
                base: Box::new(base),
                segments,
            }),
        ))
    }

    /// Unary operation with an open operator symbol.
    pub fn unary(
        &self,
        symbol: impl Into<String>,
        operand: ValueExprHirV1,
        span: SpanV1,
    ) -> ValueExprHirV1 {
        self.value(
            span,
            ValueExprKindV1::Unary(UnaryExprV1 {
                op: UnaryOpV1 {
                    symbol: symbol.into(),
                },
                operand: Box::new(operand),
            }),
        )
    }

    /// Binary operation with an open operator symbol.
    pub fn binary(
        &self,
        symbol: impl Into<String>,
        lhs: ValueExprHirV1,
        rhs: ValueExprHirV1,
        span: SpanV1,
    ) -> ValueExprHirV1 {
        self.value(
            span,
            ValueExprKindV1::Binary(BinaryExprV1 {
                op: BinaryOpV1 {
                    symbol: symbol.into(),
                },
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
        )
    }

    /// Function/operator call with positional arguments.
    pub fn call(
        &self,
        callee: impl Into<String>,
        callee_span: SpanV1,
        args: CallArgsHirV1,
        span: SpanV1,
    ) -> ValueExprHirV1 {
        self.value(
            span,
            ValueExprKindV1::Call(CallExprV1::new(
                CalleeRefV1 {
                    name: callee.into(),
                    span: callee_span,
                },
                args,
            )),
        )
    }

    /// Parenthesized value expression (preserves author grouping).
    pub fn paren_value(&self, inner: ValueExprHirV1, span: SpanV1) -> ValueExprHirV1 {
        self.value(span, ValueExprKindV1::Paren(Box::new(inner)))
    }

    /// Record construction from already-built fields.
    pub fn record(&self, fields: Vec<RecordFieldV1>, span: SpanV1) -> ValueExprHirV1 {
        self.value(span, ValueExprKindV1::Record(RecordExprV1 { fields }))
    }

    /// List construction from already-built elements.
    pub fn list(&self, elements: Vec<ValueExprHirV1>, span: SpanV1) -> ValueExprHirV1 {
        self.value(span, ValueExprKindV1::List(ListExprV1 { elements }))
    }

    fn value(&self, span: SpanV1, kind: ValueExprKindV1) -> ValueExprHirV1 {
        ValueExprHirV1::new(self.ids.next(), span, kind)
    }

    // ---- boolean expressions ----------------------------------------------

    /// Boolean literal predicate.
    pub fn bool_literal(&self, value: bool, span: SpanV1) -> BoolExprHirV1 {
        self.boolean(span, BoolExprKindV1::Literal(value))
    }

    /// A value expression used in boolean position.
    pub fn bool_value(&self, value: ValueExprHirV1, span: SpanV1) -> BoolExprHirV1 {
        self.boolean(span, BoolExprKindV1::Value(Box::new(value)))
    }

    /// Comparison with an open comparison symbol.
    pub fn compare(
        &self,
        symbol: impl Into<String>,
        lhs: ValueExprHirV1,
        rhs: ValueExprHirV1,
        span: SpanV1,
    ) -> BoolExprHirV1 {
        self.boolean(
            span,
            BoolExprKindV1::Compare(CompareExprV1 {
                op: CompareOpV1 {
                    symbol: symbol.into(),
                },
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
        )
    }

    /// Logical `and`.
    pub fn and(&self, lhs: BoolExprHirV1, rhs: BoolExprHirV1, span: SpanV1) -> BoolExprHirV1 {
        self.boolean(
            span,
            BoolExprKindV1::And(BinaryBoolV1 {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
        )
    }

    /// Logical `or`.
    pub fn or(&self, lhs: BoolExprHirV1, rhs: BoolExprHirV1, span: SpanV1) -> BoolExprHirV1 {
        self.boolean(
            span,
            BoolExprKindV1::Or(BinaryBoolV1 {
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            }),
        )
    }

    /// Logical `not`.
    pub fn not(&self, inner: BoolExprHirV1, span: SpanV1) -> BoolExprHirV1 {
        self.boolean(span, BoolExprKindV1::Not(Box::new(inner)))
    }

    /// Parenthesized boolean expression (preserves author grouping).
    pub fn paren_bool(&self, inner: BoolExprHirV1, span: SpanV1) -> BoolExprHirV1 {
        self.boolean(span, BoolExprKindV1::Paren(Box::new(inner)))
    }

    /// Logical quantifier with a validated non-empty binder list.
    pub fn logical_quantifier(
        &self,
        kind: LogicalQuantifierKindV1,
        binders: NonEmptyBindersV1,
        body: BoolExprHirV1,
        span: SpanV1,
    ) -> BoolExprHirV1 {
        self.boolean(
            span,
            BoolExprKindV1::LogicalQuantifier(LogicalQuantifierHirV1 {
                kind,
                binders,
                body: Box::new(body),
            }),
        )
    }

    /// Relational existence predicate.
    pub fn relation_exists(
        &self,
        polarity: RelationExistsPolarityV1,
        relation: RelationExprHirV1,
        span: SpanV1,
    ) -> BoolExprHirV1 {
        self.boolean(
            span,
            BoolExprKindV1::RelationExists(RelationExistsHirV1 {
                polarity,
                relation: Box::new(relation),
            }),
        )
    }

    fn boolean(&self, span: SpanV1, kind: BoolExprKindV1) -> BoolExprHirV1 {
        BoolExprHirV1::new(self.ids.next(), span, kind)
    }

    // ---- relation expressions ---------------------------------------------

    /// Relation source reference with optional alias.
    pub fn relation_source(
        &self,
        name: impl Into<String>,
        name_span: SpanV1,
        alias: Option<AliasV1>,
        span: SpanV1,
    ) -> RelationExprHirV1 {
        self.relation(
            span,
            RelationExprKindV1::Source(RelationSourceV1 {
                name: name.into(),
                name_span,
                alias,
            }),
        )
    }

    /// Projection over an input relation.
    pub fn try_project(
        &self,
        input: RelationExprHirV1,
        items: Vec<ProjectionItemV1>,
        span: SpanV1,
    ) -> Result<RelationExprHirV1, &'static str> {
        Ok(self.relation(
            span,
            RelationExprKindV1::Project(ProjectionV1::try_new(input, items)?),
        ))
    }

    /// Filter over an input relation, embedding a shared boolean predicate.
    pub fn relation_filter(
        &self,
        input: RelationExprHirV1,
        predicate: BoolExprHirV1,
        span: SpanV1,
    ) -> RelationExprHirV1 {
        self.relation(
            span,
            RelationExprKindV1::Filter(RelationFilterV1 {
                input: Box::new(input),
                predicate: Box::new(predicate),
            }),
        )
    }

    /// Inner join with a required predicate.
    pub fn inner_join(
        &self,
        left: RelationExprHirV1,
        right: RelationExprHirV1,
        on: BoolExprHirV1,
        span: SpanV1,
    ) -> RelationExprHirV1 {
        self.relation(
            span,
            RelationExprKindV1::Join(JoinHirV1::Inner {
                left: Box::new(left),
                right: Box::new(right),
                on,
            }),
        )
    }

    /// Predicate-free explicit cross join.
    pub fn cross_join(
        &self,
        left: RelationExprHirV1,
        right: RelationExprHirV1,
        span: SpanV1,
    ) -> RelationExprHirV1 {
        self.relation(
            span,
            RelationExprKindV1::Join(JoinHirV1::Cross {
                left: Box::new(left),
                right: Box::new(right),
            }),
        )
    }

    /// Grouping over explicit keys.
    pub fn group_by(
        &self,
        input: RelationExprHirV1,
        keys: Vec<ValueExprHirV1>,
        span: SpanV1,
    ) -> Result<RelationExprHirV1, &'static str> {
        let group = GroupByHirV1::try_new(input, keys)?;
        Ok(self.relation(span, RelationExprKindV1::GroupBy(group)))
    }

    /// Ordering over explicit keys and directions.
    pub fn order_by(
        &self,
        input: RelationExprHirV1,
        keys: Vec<OrderKeyHirV1>,
        span: SpanV1,
    ) -> Result<RelationExprHirV1, &'static str> {
        let order = OrderByHirV1::try_new(input, keys)?;
        Ok(self.relation(span, RelationExprKindV1::OrderBy(order)))
    }

    /// Relational truncation preserves the source count, including zero.
    pub fn limit(&self, input: RelationExprHirV1, count: u64, span: SpanV1) -> RelationExprHirV1 {
        self.relation(
            span,
            RelationExprKindV1::Limit(LimitHirV1 {
                input: Box::new(input),
                count,
            }),
        )
    }

    fn relation(&self, span: SpanV1, kind: RelationExprKindV1) -> RelationExprHirV1 {
        RelationExprHirV1::new(self.ids.next(), span, kind)
    }

    // ---- result shapes -----------------------------------------------------

    /// Leaf result-shape position wrapping a shared value expression.
    pub fn shape_leaf(&self, value: ValueExprHirV1, span: SpanV1) -> ResultShapeHirV1 {
        self.shape(span, ResultShapeKindV1::Leaf(Box::new(value)))
    }

    /// Record result shape from already-built fields.
    pub fn shape_record(&self, fields: Vec<ShapeFieldV1>, span: SpanV1) -> ResultShapeHirV1 {
        self.shape(span, ResultShapeKindV1::Record(ShapeRecordV1 { fields }))
    }

    /// List result shape over a per-element nested shape.
    pub fn shape_list(&self, element: ResultShapeHirV1, span: SpanV1) -> ResultShapeHirV1 {
        self.shape(
            span,
            ResultShapeKindV1::List(ShapeListV1 {
                element: Box::new(element),
            }),
        )
    }

    fn shape(&self, span: SpanV1, kind: ResultShapeKindV1) -> ResultShapeHirV1 {
        ResultShapeHirV1::new(self.ids.next(), span, kind)
    }

    // ---- units -------------------------------------------------------------

    /// A query unit embedding a shared relation body and optional shared shape.
    pub fn query(
        &self,
        body: RelationExprHirV1,
        result_shape: Option<ResultShapeHirV1>,
        span: SpanV1,
    ) -> QueryHirV1 {
        QueryHirV1::new(self.ids.next(), span, body, result_shape)
    }

    /// A declaration unit embedding a shared value body.
    pub fn decl(
        &self,
        name: impl Into<String>,
        name_span: SpanV1,
        body: ValueExprHirV1,
        span: SpanV1,
    ) -> DeclHirV1 {
        DeclHirV1::new(self.ids.next(), span, name.into(), name_span, body)
    }

    /// A rule unit embedding a shared value head and shared boolean body.
    pub fn rule(
        &self,
        name: impl Into<String>,
        name_span: SpanV1,
        head: ValueExprHirV1,
        body: BoolExprHirV1,
        span: SpanV1,
    ) -> RuleHirV1 {
        RuleHirV1::new(self.ids.next(), span, name.into(), name_span, head, body)
    }

    // ---- program items -----------------------------------------------------

    /// A graph query (FIND) item.
    pub fn graph_query(
        &self,
        kind: GraphQueryKindV1,
        endpoints: GraphEndpointsV1,
        constraints: GraphConstraintsV1,
        span: SpanV1,
    ) -> GraphQueryHirV1 {
        GraphQueryHirV1::new(self.ids.next(), span, kind, endpoints, constraints)
    }

    /// A legality check (CHECK) item.
    pub fn legality(
        &self,
        rule_ref: crate::item::GraphRefV1,
        scope: Option<crate::item::GraphRefV1>,
        pattern: Option<crate::item::GraphRefV1>,
        confirm: crate::item::ConfirmModeV1,
        emit: crate::item::EmitModeV1,
        mode: crate::item::DetectionModeV1,
        span: SpanV1,
    ) -> LegalityHirV1 {
        LegalityHirV1::new(
            self.ids.next(),
            span,
            rule_ref,
            scope,
            pattern,
            confirm,
            emit,
            mode,
        )
    }

    /// A legality check with an explicit graph-backed semantic confirmation.
    pub fn legality_with_semantic_confirm(
        &self,
        rule_ref: crate::item::GraphRefV1,
        scope: Option<crate::item::GraphRefV1>,
        pattern: Option<crate::item::GraphRefV1>,
        semantic_confirm: GraphQueryHirV1,
        confirm: crate::item::ConfirmModeV1,
        emit: crate::item::EmitModeV1,
        mode: crate::item::DetectionModeV1,
        span: SpanV1,
    ) -> LegalityHirV1 {
        self.legality(rule_ref, scope, pattern, confirm, emit, mode, span)
            .with_semantic_confirm(semantic_confirm)
    }

    /// A presentation verb item from an already-built verb payload.
    pub fn presentation(&self, kind: PresentationKindV1, span: SpanV1) -> PresentationHirV1 {
        PresentationHirV1::new(self.ids.next(), span, kind)
    }

    /// A catalog declaration item from an already-built payload.
    pub fn catalog_decl(
        &self,
        name: impl Into<String>,
        name_span: SpanV1,
        payload: CatalogPayloadV1,
        span: SpanV1,
    ) -> CatalogDeclHirV1 {
        CatalogDeclHirV1::new(self.ids.next(), span, name, name_span, payload)
    }

    /// A `LET <name> = <producer>` binding (no id of its own; the producer item
    /// carries one).
    #[must_use]
    pub fn composition_binding(
        &self,
        name: impl Into<String>,
        name_span: SpanV1,
        producer: CompositionProducerV1,
    ) -> CompositionBindingV1 {
        CompositionBindingV1::new(name, name_span, producer)
    }

    /// A composition item embedding producer items and a body item.
    pub fn composition(
        &self,
        bindings: Vec<CompositionBindingV1>,
        with_name: impl Into<String>,
        with_name_span: SpanV1,
        body: CompositionBodyV1,
        span: SpanV1,
    ) -> CompositionHirV1 {
        CompositionHirV1::new(
            self.ids.next(),
            span,
            bindings,
            with_name,
            with_name_span,
            body,
        )
    }

    /// A meta directive item from an already-built payload.
    pub fn meta(&self, kind: MetaKindV1, span: SpanV1) -> MetaHirV1 {
        MetaHirV1::new(self.ids.next(), span, kind)
    }

    /// Source-authored rename intent; semantic target sealing is downstream.
    pub fn rename_command(
        &self,
        target: crate::item::GraphRefV1,
        new_name: String,
        phase: crate::item::RenamePhaseV3,
        expected_snapshot: String,
        preview_token: Option<String>,
        span: SpanV1,
    ) -> crate::item::RenameCommandHirV3 {
        crate::item::RenameCommandHirV3 {
            id: self.ids.next(),
            span,
            target,
            new_name,
            phase,
            expected_snapshot,
            preview_token,
        }
    }
}
