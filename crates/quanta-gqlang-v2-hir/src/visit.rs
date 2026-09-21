//! Read-only traversal over HIR trees.
//!
//! [`HirVisitorV1`] is a borrow-only visitor: every node type has a
//! `visit_*` hook (default no-op) and a matching `walk_*` free function that
//! recurses into children. The contract relied on by [`crate::integrity`] is
//! that `walk_*` visits EVERY node in the subtree exactly once, in a
//! deterministic pre-order (parent before children, children left-to-right).
//!
//! To visit a tree, implement [`HirVisitorV1`] and call the matching entry
//! `walk_*`. Each `visit_*` default delegates to its `walk_*`, so an
//! implementation that overrides only the leaf hooks still gets full traversal.

use crate::expr::{
    BoolExprHirV1, BoolExprKindV1, CallArgHirV1, CallExprV1, CompareExprV1, ValueExprHirV1,
    ValueExprKindV1,
};
use crate::item::{
    CatalogDeclHirV1, CatalogPayloadV1, CompositionBodyV1, CompositionHirV1, CompositionProducerV1,
    ExplainHirV1, GraphQueryHirV1, LegalityHirV1, MetaHirV1, PresentationHirV1, ProgramItemHirV1,
    SampleHirV1, ViewBodyV1,
};
use crate::relation::{RelationExprHirV1, RelationExprKindV1};
use crate::shape::{ResultShapeHirV1, ResultShapeKindV1};
use crate::unit::{DeclHirV1, QueryHirV1, RuleHirV1};

/// Borrow-only pre-order visitor over HIR nodes.
///
/// Default `visit_*` hooks recurse via the matching `walk_*`; override a hook to
/// observe that node, and call the corresponding `walk_*` (or the default) to
/// continue into children.
pub trait HirVisitorV1 {
    /// Visit any non-node source span carried by a HIR payload.
    fn visit_auxiliary_span(&mut self, _span: crate::span::SpanV1) {}
    /// Visit a value expression.
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        walk_value(self, node);
    }
    /// Visit a boolean expression.
    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        walk_bool(self, node);
    }
    /// Visit the auxiliary span of a call `*` argument.
    fn visit_call_star(&mut self, span: crate::span::SpanV1) {
        self.visit_auxiliary_span(span);
    }
    /// Visit the auxiliary span carried by a logical binder.
    fn visit_logical_binder(&mut self, span: crate::span::SpanV1) {
        self.visit_auxiliary_span(span);
    }
    /// Visit a relation expression.
    fn visit_relation(&mut self, node: &RelationExprHirV1) {
        walk_relation(self, node);
    }
    /// Visit a result-shape node.
    fn visit_shape(&mut self, node: &ResultShapeHirV1) {
        walk_shape(self, node);
    }
    /// Visit a query unit.
    fn visit_query(&mut self, node: &QueryHirV1) {
        walk_query(self, node);
    }
    /// Visit a declaration unit.
    fn visit_decl(&mut self, node: &DeclHirV1) {
        walk_decl(self, node);
    }
    /// Visit a rule unit.
    fn visit_rule(&mut self, node: &RuleHirV1) {
        walk_rule(self, node);
    }
    /// Visit a unified program item.
    fn visit_item(&mut self, node: &ProgramItemHirV1) {
        walk_item(self, node);
    }
    /// Visit a graph query (FIND) item.
    fn visit_graph_query(&mut self, node: &GraphQueryHirV1) {
        walk_graph_query(self, node);
    }
    /// Visit a legality check (CHECK) item.
    fn visit_legality(&mut self, node: &LegalityHirV1) {
        walk_legality(self, node);
    }
    /// Visit a presentation verb item.
    fn visit_presentation(&mut self, node: &PresentationHirV1) {
        walk_presentation(self, node);
    }
    /// Visit a composition-specific SAMPLE node.
    fn visit_sample(&mut self, node: &SampleHirV1) {
        walk_sample(self, node);
    }
    /// Visit a composition-specific EXPLAIN node.
    fn visit_explain(&mut self, node: &ExplainHirV1) {
        walk_explain(self, node);
    }
    /// Visit a catalog declaration item.
    fn visit_catalog_decl(&mut self, node: &CatalogDeclHirV1) {
        walk_catalog_decl(self, node);
    }
    /// Visit a composition item.
    fn visit_composition(&mut self, node: &CompositionHirV1) {
        walk_composition(self, node);
    }
    /// Visit a meta directive item.
    fn visit_meta(&mut self, node: &MetaHirV1) {
        walk_meta(self, node);
    }
}

/// Recurse into the children of a value expression.
pub fn walk_value<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &ValueExprHirV1) {
    match &node.kind {
        ValueExprKindV1::Literal(_) | ValueExprKindV1::Symbol(_) => {}
        ValueExprKindV1::Path(path) => {
            visitor.visit_value(&path.base);
            for segment in path.segments.as_slice() {
                visitor.visit_auxiliary_span(segment.span);
            }
        }
        ValueExprKindV1::Unary(unary) => {
            visitor.visit_value(&unary.operand);
        }
        ValueExprKindV1::Binary(binary) => {
            visitor.visit_value(&binary.lhs);
            visitor.visit_value(&binary.rhs);
        }
        ValueExprKindV1::Call(call) => {
            visitor.visit_auxiliary_span(call.callee.span);
            walk_call_args(visitor, call);
        }
        ValueExprKindV1::Paren(inner) => {
            visitor.visit_value(inner);
        }
        ValueExprKindV1::Record(record) => {
            for field in &record.fields {
                visitor.visit_auxiliary_span(field.name_span);
                visitor.visit_value(&field.value);
            }
        }
        ValueExprKindV1::List(list) => {
            for element in &list.elements {
                visitor.visit_value(element);
            }
        }
    }
}

fn walk_call_args<V: HirVisitorV1 + ?Sized>(visitor: &mut V, call: &CallExprV1) {
    for arg in call.args().as_slice() {
        match arg {
            CallArgHirV1::Expr(value) => visitor.visit_value(value),
            CallArgHirV1::Star { .. } => {
                if let Some(span) = arg.star_span() {
                    visitor.visit_call_star(span);
                }
            }
        }
    }
}

/// Recurse into the children of a boolean expression.
pub fn walk_bool<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &BoolExprHirV1) {
    match &node.kind {
        BoolExprKindV1::Literal(_) => {}
        BoolExprKindV1::Value(value) => {
            visitor.visit_value(value);
        }
        BoolExprKindV1::Compare(compare) => {
            walk_compare(visitor, compare);
        }
        BoolExprKindV1::And(bin) | BoolExprKindV1::Or(bin) => {
            visitor.visit_bool(&bin.lhs);
            visitor.visit_bool(&bin.rhs);
        }
        BoolExprKindV1::Not(inner) | BoolExprKindV1::Paren(inner) => {
            visitor.visit_bool(inner);
        }
        BoolExprKindV1::LogicalQuantifier(quant) => {
            for binder in quant.binders.as_slice() {
                visitor.visit_logical_binder(binder.span);
            }
            visitor.visit_bool(&quant.body);
        }
        BoolExprKindV1::RelationExists(exists) => visitor.visit_relation(&exists.relation),
    }
}

fn walk_compare<V: HirVisitorV1 + ?Sized>(visitor: &mut V, compare: &CompareExprV1) {
    visitor.visit_value(&compare.lhs);
    visitor.visit_value(&compare.rhs);
}

/// Recurse into the children of a relation expression.
pub fn walk_relation<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &RelationExprHirV1) {
    match &node.kind {
        RelationExprKindV1::Source(source) => {
            visitor.visit_auxiliary_span(source.name_span);
            if let Some(alias) = &source.alias {
                visitor.visit_auxiliary_span(alias.span);
            }
        }
        RelationExprKindV1::Project(project) => {
            visitor.visit_relation(&project.input);
            for item in project.items() {
                if let Some(alias) = &item.alias {
                    visitor.visit_auxiliary_span(alias.span);
                }
                visitor.visit_value(&item.value);
            }
        }
        RelationExprKindV1::Filter(filter) => {
            visitor.visit_relation(&filter.input);
            visitor.visit_bool(&filter.predicate);
        }
        RelationExprKindV1::Join(join) => match join {
            crate::relation::JoinHirV1::Inner { left, right, on } => {
                visitor.visit_relation(left);
                visitor.visit_relation(right);
                visitor.visit_bool(on);
            }
            crate::relation::JoinHirV1::Cross { left, right } => {
                visitor.visit_relation(left);
                visitor.visit_relation(right);
            }
        },
        RelationExprKindV1::GroupBy(group) => {
            visitor.visit_relation(&group.input);
            for key in group.keys() {
                visitor.visit_value(key);
            }
        }
        RelationExprKindV1::OrderBy(order) => {
            visitor.visit_relation(&order.input);
            for key in order.keys() {
                visitor.visit_value(&key.value);
            }
        }
    }
}

/// Recurse into the children of a result-shape node.
pub fn walk_shape<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &ResultShapeHirV1) {
    match &node.kind {
        ResultShapeKindV1::Leaf(value) => {
            visitor.visit_value(value);
        }
        ResultShapeKindV1::Record(record) => {
            for field in &record.fields {
                visitor.visit_auxiliary_span(field.name_span);
                visitor.visit_shape(&field.shape);
            }
        }
        ResultShapeKindV1::List(list) => {
            visitor.visit_shape(&list.element);
        }
    }
}

/// Recurse into the children of a query unit.
pub fn walk_query<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &QueryHirV1) {
    visitor.visit_relation(&node.body);
    if let Some(shape) = &node.result_shape {
        visitor.visit_shape(shape);
    }
}

/// Recurse into the children of a declaration unit.
pub fn walk_decl<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &DeclHirV1) {
    visitor.visit_auxiliary_span(node.name_span);
    visitor.visit_value(&node.body);
}

/// Recurse into the children of a rule unit.
pub fn walk_rule<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &RuleHirV1) {
    visitor.visit_auxiliary_span(node.name_span);
    visitor.visit_value(&node.head);
    visitor.visit_bool(&node.body);
}

/// Dispatch to the matching `visit_*` for whichever item kind this is.
pub fn walk_item<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &ProgramItemHirV1) {
    match node {
        ProgramItemHirV1::Query(query) => visitor.visit_query(query),
        ProgramItemHirV1::Decl(decl) => visitor.visit_decl(decl),
        ProgramItemHirV1::Rule(rule) => visitor.visit_rule(rule),
        ProgramItemHirV1::GraphQuery(graph) => visitor.visit_graph_query(graph),
        ProgramItemHirV1::Legality(legality) => visitor.visit_legality(legality),
        ProgramItemHirV1::Presentation(pres) => visitor.visit_presentation(pres),
        ProgramItemHirV1::CatalogDecl(decl) => visitor.visit_catalog_decl(decl),
        ProgramItemHirV1::Composition(comp) => visitor.visit_composition(comp),
        ProgramItemHirV1::Meta(meta) => visitor.visit_meta(meta),
    }
}

/// Recurse into the children of a graph query (FIND). Endpoints/constraints are
/// open refs (no shared expr kernel), so there are no id-bearing children here.
pub fn walk_graph_query<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &GraphQueryHirV1) {
    for reference in [
        node.endpoints.source.as_ref(),
        node.endpoints.sink.as_ref(),
        node.endpoints.anchor.as_ref(),
        node.endpoints.reference.as_ref(),
        node.endpoints.pointee.as_ref(),
        node.endpoints.callable.as_ref(),
        node.constraints.scope.as_ref(),
        node.constraints.window.as_ref(),
        node.constraints.frontier.as_ref(),
    ]
    .into_iter()
    .flatten()
    .chain(node.constraints.via.iter())
    .chain(node.constraints.through.iter())
    .chain(node.constraints.sanitizers.iter())
    {
        visitor.visit_auxiliary_span(reference.span);
    }
}

/// Recurse into the explicit graph confirmation child of a legality check.
pub fn walk_legality<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &LegalityHirV1) {
    visitor.visit_auxiliary_span(node.rule_ref.span);
    if let Some(scope) = &node.scope {
        visitor.visit_auxiliary_span(scope.span);
    }
    if let Some(pattern) = &node.pattern {
        visitor.visit_auxiliary_span(pattern.span);
    }
    if let Some(confirm) = &node.semantic_confirm {
        visitor.visit_graph_query(confirm);
    }
}

/// Recurse into the children of a presentation verb. Configs are open refs/tags,
/// so there are no id-bearing children here.
pub fn walk_presentation<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &PresentationHirV1) {
    use crate::item::PresentationKindV1;
    match &node.kind {
        PresentationKindV1::Explain { target_ref, .. } => {
            visitor.visit_auxiliary_span(target_ref.span);
        }
        PresentationKindV1::Describe { path, .. } => visitor.visit_auxiliary_span(path.span),
        PresentationKindV1::Sample { scope, .. } => {
            if let Some(scope) = scope {
                visitor.visit_auxiliary_span(scope.span);
            }
        }
        PresentationKindV1::Compare {
            baseline_ref,
            candidate_ref,
            ..
        } => {
            visitor.visit_auxiliary_span(baseline_ref.span);
            visitor.visit_auxiliary_span(candidate_ref.span);
        }
        PresentationKindV1::Export {
            source_ref,
            profile,
            ..
        } => {
            visitor.visit_auxiliary_span(source_ref.span);
            if let Some(profile) = profile {
                visitor.visit_auxiliary_span(profile.span);
            }
        }
        PresentationKindV1::Show { .. } => {}
    }
}

/// SAMPLE carries no id-bearing child expressions.
pub fn walk_sample<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &SampleHirV1) {
    if let Some(scope) = &node.scope {
        visitor.visit_auxiliary_span(scope.span);
    }
}

/// EXPLAIN carries no id-bearing child expressions.
pub fn walk_explain<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &ExplainHirV1) {
    visitor.visit_auxiliary_span(node.target_ref.span);
}

/// Recurse into the children of a catalog declaration. Only a VIEW with an
/// inline SELECT embeds an id-bearing child (the inline query).
pub fn walk_catalog_decl<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &CatalogDeclHirV1) {
    visitor.visit_auxiliary_span(node.name_span);
    match &node.payload {
        CatalogPayloadV1::View(ViewBodyV1::Inline(query)) => visitor.visit_query(query),
        CatalogPayloadV1::View(ViewBodyV1::QueryRef(_))
        | CatalogPayloadV1::Model { .. }
        | CatalogPayloadV1::Pack { .. }
        | CatalogPayloadV1::Profile { .. }
        | CatalogPayloadV1::Gate { .. }
        | CatalogPayloadV1::RuleBinding { .. } => {}
    }
}

/// Recurse into the children of a composition: every producer item then the
/// body item, all of which are id-bearing `ProgramItemHirV1` subtrees.
pub fn walk_composition<V: HirVisitorV1 + ?Sized>(visitor: &mut V, node: &CompositionHirV1) {
    visitor.visit_auxiliary_span(node.with_name_span);
    for binding in &node.bindings {
        visitor.visit_auxiliary_span(binding.name_span);
        match &binding.producer {
            CompositionProducerV1::Query(query) => visitor.visit_query(query),
            CompositionProducerV1::Sample(sample) => visitor.visit_sample(sample),
            CompositionProducerV1::GraphQuery(graph) => visitor.visit_graph_query(graph),
        }
    }
    match &node.body {
        CompositionBodyV1::Query(query) => visitor.visit_query(query),
        CompositionBodyV1::GraphQuery(graph) => visitor.visit_graph_query(graph),
        CompositionBodyV1::Legality(legality) => visitor.visit_legality(legality),
        CompositionBodyV1::Explain(explain) => visitor.visit_explain(explain),
    }
}

/// Recurse into the children of a meta directive. Payloads are open tags/strings,
/// so there are no id-bearing children here.
pub fn walk_meta<V: HirVisitorV1 + ?Sized>(_visitor: &mut V, _node: &MetaHirV1) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expr::{
        BoolExprHirV1, BoolExprKindV1, CompareExprV1, CompareOpV1, LiteralV1, ValueExprHirV1,
        ValueExprKindV1,
    };
    use crate::id::HirNodeIdV1;
    use crate::span::SpanV1;

    /// A visitor that overrides ONLY `visit_value` (to count leaves) and leaves
    /// every other hook at its trait default. Driving it therefore exercises the
    /// default-method bodies of the non-overridden hooks: if a default delegates
    /// to its `walk_*` it recurses into children (and into `visit_value`); if the
    /// default body were replaced with `()` the recursion would never happen.
    struct ValueCounter {
        values: usize,
    }

    impl HirVisitorV1 for ValueCounter {
        fn visit_value(&mut self, node: &ValueExprHirV1) {
            self.values += 1;
            walk_value(self, node);
        }
    }

    fn lit_value(id: u32) -> ValueExprHirV1 {
        ValueExprHirV1::new(
            HirNodeIdV1::from_raw(id),
            SpanV1::point(0),
            ValueExprKindV1::Literal(LiteralV1::Int("1".to_string())),
        )
    }

    /// Default `visit_bool` (NOT overridden by `ValueCounter`) must delegate to
    /// `walk_bool`, which descends a Compare into its two operand values. Driving
    /// a Bool root therefore counts exactly two values. If the default
    /// `visit_bool` body were replaced with `()`, zero values would be counted.
    #[test]
    fn default_visit_bool_recurses_into_compare_operands() {
        let compare = BoolExprHirV1::new(
            HirNodeIdV1::from_raw(10),
            SpanV1::point(0),
            BoolExprKindV1::Compare(CompareExprV1 {
                op: CompareOpV1 {
                    symbol: ">".to_string(),
                },
                lhs: Box::new(lit_value(11)),
                rhs: Box::new(lit_value(12)),
            }),
        );
        let mut counter = ValueCounter { values: 0 };
        // visit_bool uses the trait DEFAULT body (delegates to walk_bool).
        counter.visit_bool(&compare);
        assert_eq!(counter.values, 2);
    }
}
