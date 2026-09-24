//! Stack-safe semantic equivalence for checked surface serialization.
//!
//! Parser-assigned ids and source spans are provenance, not surface semantics.
//! Parenthesis carriers are also ignored once their authored grouping has been
//! reflected in the tree shape. Every other payload is compared exhaustively.

use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CallArgHirV1, CatalogPayloadV1, CompositionBodyV1,
    CompositionProducerV1, GraphQueryHirV1, GraphRefV1, JoinHirV1, LegalityHirV1, MetaKindV1,
    PresentationKindV1, ProgramItemHirV1, QueryHirV1, RelationExprHirV1, RelationExprKindV1,
    ResultShapeHirV1, ResultShapeKindV1, ValueExprHirV1, ValueExprKindV1, ViewBodyV1,
};

enum Frame<'a> {
    Item(&'a ProgramItemHirV1, &'a ProgramItemHirV1),
    Query(&'a QueryHirV1, &'a QueryHirV1),
    Relation(&'a RelationExprHirV1, &'a RelationExprHirV1),
    Bool(&'a BoolExprHirV1, &'a BoolExprHirV1),
    Value(&'a ValueExprHirV1, &'a ValueExprHirV1),
    Shape(&'a ResultShapeHirV1, &'a ResultShapeHirV1),
    Graph(&'a GraphQueryHirV1, &'a GraphQueryHirV1),
    Legality(&'a LegalityHirV1, &'a LegalityHirV1),
}

fn ref_name(reference: &GraphRefV1) -> &str {
    &reference.name
}

fn same_ref(left: &GraphRefV1, right: &GraphRefV1) -> bool {
    ref_name(left) == ref_name(right)
}

fn same_ref_option(left: &Option<GraphRefV1>, right: &Option<GraphRefV1>) -> bool {
    left.as_ref().map(ref_name) == right.as_ref().map(ref_name)
}

fn same_ref_list(left: &[GraphRefV1], right: &[GraphRefV1]) -> bool {
    left.len() == right.len() && left.iter().zip(right).all(|(a, b)| same_ref(a, b))
}

fn same_graph(left: &GraphQueryHirV1, right: &GraphQueryHirV1) -> bool {
    let a = &left.endpoints;
    let b = &right.endpoints;
    let x = &left.constraints;
    let y = &right.constraints;
    left.kind == right.kind
        && same_ref_option(&a.source, &b.source)
        && same_ref_option(&a.sink, &b.sink)
        && same_ref_option(&a.anchor, &b.anchor)
        && same_ref_option(&a.reference, &b.reference)
        && same_ref_option(&a.pointee, &b.pointee)
        && same_ref_option(&a.callable, &b.callable)
        && same_ref_list(&x.via, &y.via)
        && same_ref_list(&x.through, &y.through)
        && same_ref_list(&x.sanitizers, &y.sanitizers)
        && same_ref_option(&x.scope, &y.scope)
        && same_ref_option(&x.window, &y.window)
        && x.direction.as_ref().map(|value| value.keyword.as_str())
            == y.direction.as_ref().map(|value| value.keyword.as_str())
        && same_ref_option(&x.frontier, &y.frontier)
        && x.depth == y.depth
        && x.summary_facets == y.summary_facets
}

fn strip_value_parens(mut value: &ValueExprHirV1) -> &ValueExprHirV1 {
    while let ValueExprKindV1::Paren(inner) = &value.kind {
        value = inner;
    }
    value
}

fn strip_bool_parens(mut value: &BoolExprHirV1) -> &BoolExprHirV1 {
    while let BoolExprKindV1::Paren(inner) = &value.kind {
        value = inner;
    }
    value
}

/// Compare every surface-semantic field without recursion.
///
/// This is deliberately independent of canonical text: two different raw HIR
/// shapes must never be accepted merely because they render to the same bytes.
pub(crate) fn semantically_equivalent_items(
    left: &ProgramItemHirV1,
    right: &ProgramItemHirV1,
) -> bool {
    let mut stack = vec![Frame::Item(left, right)];
    while let Some(frame) = stack.pop() {
        match frame {
            Frame::Item(left, right) => match (left, right) {
                (ProgramItemHirV1::Query(a), ProgramItemHirV1::Query(b)) => {
                    stack.push(Frame::Query(a, b));
                }
                (ProgramItemHirV1::Decl(a), ProgramItemHirV1::Decl(b)) => {
                    if a.name != b.name {
                        return false;
                    }
                    stack.push(Frame::Value(&a.body, &b.body));
                }
                (ProgramItemHirV1::Rule(a), ProgramItemHirV1::Rule(b)) => {
                    if a.name != b.name {
                        return false;
                    }
                    stack.push(Frame::Value(&a.head, &b.head));
                    stack.push(Frame::Bool(&a.body, &b.body));
                }
                (ProgramItemHirV1::GraphQuery(a), ProgramItemHirV1::GraphQuery(b)) => {
                    stack.push(Frame::Graph(a, b));
                }
                (ProgramItemHirV1::Legality(a), ProgramItemHirV1::Legality(b)) => {
                    stack.push(Frame::Legality(a, b));
                }
                (ProgramItemHirV1::Presentation(a), ProgramItemHirV1::Presentation(b)) => {
                    let same = match (&a.kind, &b.kind) {
                        (
                            PresentationKindV1::Explain {
                                target_ref: at,
                                evidence: ae,
                                truth: av,
                            },
                            PresentationKindV1::Explain {
                                target_ref: bt,
                                evidence: be,
                                truth: bv,
                            },
                        ) => same_ref(at, bt) && ae == be && av == bv,
                        (
                            PresentationKindV1::Show { category: a },
                            PresentationKindV1::Show { category: b },
                        ) => a == b,
                        (
                            PresentationKindV1::Describe { kind: ak, path: ap },
                            PresentationKindV1::Describe { kind: bk, path: bp },
                        ) => ak == bk && same_ref(ap, bp),
                        (
                            PresentationKindV1::Sample {
                                query_text: aq,
                                mode: am,
                                limit: al,
                                scope: ascope,
                            },
                            PresentationKindV1::Sample {
                                query_text: bq,
                                mode: bm,
                                limit: bl,
                                scope: bscope,
                            },
                        ) => aq == bq && am == bm && al == bl && same_ref_option(ascope, bscope),
                        (
                            PresentationKindV1::Compare {
                                baseline_ref: ab,
                                candidate_ref: ac,
                                metric: am,
                            },
                            PresentationKindV1::Compare {
                                baseline_ref: bb,
                                candidate_ref: bc,
                                metric: bm,
                            },
                        ) => same_ref(ab, bb) && same_ref(ac, bc) && am == bm,
                        (
                            PresentationKindV1::Export {
                                source_ref: asource,
                                format: af,
                                profile: ap,
                            },
                            PresentationKindV1::Export {
                                source_ref: bsource,
                                format: bf,
                                profile: bp,
                            },
                        ) => same_ref(asource, bsource) && af == bf && same_ref_option(ap, bp),
                        _ => false,
                    };
                    if !same {
                        return false;
                    }
                }
                (ProgramItemHirV1::CatalogDecl(a), ProgramItemHirV1::CatalogDecl(b)) => {
                    if a.name != b.name {
                        return false;
                    }
                    match (&a.payload, &b.payload) {
                        (
                            CatalogPayloadV1::View(ViewBodyV1::QueryRef(a)),
                            CatalogPayloadV1::View(ViewBodyV1::QueryRef(b)),
                        ) if same_ref(a, b) => {}
                        (
                            CatalogPayloadV1::View(ViewBodyV1::Inline(a)),
                            CatalogPayloadV1::View(ViewBodyV1::Inline(b)),
                        ) => stack.push(Frame::Query(a, b)),
                        (
                            CatalogPayloadV1::Model {
                                provider: ap,
                                source: asource,
                            },
                            CatalogPayloadV1::Model {
                                provider: bp,
                                source: bsource,
                            },
                        ) if ap == bp && same_ref(asource, bsource) => {}
                        (
                            CatalogPayloadV1::Pack {
                                source: asource,
                                activation: aa,
                            },
                            CatalogPayloadV1::Pack {
                                source: bsource,
                                activation: b_activation,
                            },
                        ) if same_ref(asource, bsource) && aa == b_activation => {}
                        (
                            CatalogPayloadV1::Profile {
                                capabilities: ac,
                                admission: aa,
                            },
                            CatalogPayloadV1::Profile {
                                capabilities: bc,
                                admission: b_admission,
                            },
                        ) if same_ref_list(ac, bc) && aa == b_admission => {}
                        (
                            CatalogPayloadV1::Gate {
                                checks: ac,
                                binding: ab,
                            },
                            CatalogPayloadV1::Gate {
                                checks: bc,
                                binding: bb,
                            },
                        ) if same_ref_list(ac, bc) && ab == bb => {}
                        (
                            CatalogPayloadV1::RuleBinding { check_ref: a },
                            CatalogPayloadV1::RuleBinding { check_ref: b },
                        ) if same_ref(a, b) => {}
                        _ => return false,
                    }
                }
                (ProgramItemHirV1::Composition(a), ProgramItemHirV1::Composition(b)) => {
                    if a.with_name != b.with_name || a.bindings.len() != b.bindings.len() {
                        return false;
                    }
                    for (a, b) in a.bindings.iter().zip(&b.bindings) {
                        if a.name != b.name {
                            return false;
                        }
                        match (&a.producer, &b.producer) {
                            (CompositionProducerV1::Query(a), CompositionProducerV1::Query(b)) => {
                                stack.push(Frame::Query(a, b))
                            }
                            (
                                CompositionProducerV1::GraphQuery(a),
                                CompositionProducerV1::GraphQuery(b),
                            ) => stack.push(Frame::Graph(a, b)),
                            (
                                CompositionProducerV1::Sample(a),
                                CompositionProducerV1::Sample(b),
                            ) if a.query_text == b.query_text
                                && a.mode == b.mode
                                && a.limit == b.limit
                                && same_ref_option(&a.scope, &b.scope) => {}
                            _ => return false,
                        }
                    }
                    match (&a.body, &b.body) {
                        (CompositionBodyV1::Query(a), CompositionBodyV1::Query(b)) => {
                            stack.push(Frame::Query(a, b));
                        }
                        (CompositionBodyV1::GraphQuery(a), CompositionBodyV1::GraphQuery(b)) => {
                            stack.push(Frame::Graph(a, b))
                        }
                        (CompositionBodyV1::Legality(a), CompositionBodyV1::Legality(b)) => {
                            stack.push(Frame::Legality(a, b));
                        }
                        (CompositionBodyV1::Explain(a), CompositionBodyV1::Explain(b))
                            if same_ref(&a.target_ref, &b.target_ref)
                                && a.evidence == b.evidence
                                && a.truth == b.truth => {}
                        _ => return false,
                    }
                }
                (ProgramItemHirV1::Meta(a), ProgramItemHirV1::Meta(b)) => {
                    match (&a.kind, &b.kind) {
                        (MetaKindV1::RequireCertified, MetaKindV1::RequireCertified) => {}
                        (MetaKindV1::UsePack(a), MetaKindV1::UsePack(b)) if a == b => {}
                        _ => return false,
                    }
                }
                _ => return false,
            },
            Frame::Query(left, right) => {
                match (&left.result_shape, &right.result_shape) {
                    (Some(a), Some(b)) => stack.push(Frame::Shape(a, b)),
                    (None, None) => {}
                    _ => return false,
                }
                stack.push(Frame::Relation(&left.body, &right.body));
            }
            Frame::Graph(left, right) => {
                if !same_graph(left, right) {
                    return false;
                }
            }
            Frame::Legality(left, right) => {
                if !same_ref(&left.rule_ref, &right.rule_ref)
                    || !same_ref_option(&left.scope, &right.scope)
                    || !same_ref_option(&left.pattern, &right.pattern)
                    || left.confirm != right.confirm
                    || left.emit != right.emit
                    || left.mode != right.mode
                {
                    return false;
                }
                match (&left.semantic_confirm, &right.semantic_confirm) {
                    (Some(a), Some(b)) => stack.push(Frame::Graph(a, b)),
                    (None, None) => {}
                    _ => return false,
                }
            }
            Frame::Value(left, right) => {
                let left = strip_value_parens(left);
                let right = strip_value_parens(right);
                match (&left.kind, &right.kind) {
                    (ValueExprKindV1::Literal(a), ValueExprKindV1::Literal(b)) if a == b => {}
                    (ValueExprKindV1::Symbol(a), ValueExprKindV1::Symbol(b))
                        if a.name == b.name => {}
                    (ValueExprKindV1::Path(a), ValueExprKindV1::Path(b)) => {
                        if a.segments.len() != b.segments.len()
                            || !a
                                .segments
                                .iter()
                                .zip(b.segments.iter())
                                .all(|(a, b)| a.name == b.name)
                        {
                            return false;
                        }
                        stack.push(Frame::Value(&a.base, &b.base));
                    }
                    (ValueExprKindV1::Unary(a), ValueExprKindV1::Unary(b))
                        if a.op.symbol == b.op.symbol =>
                    {
                        stack.push(Frame::Value(&a.operand, &b.operand));
                    }
                    (ValueExprKindV1::Binary(a), ValueExprKindV1::Binary(b))
                        if a.op.symbol == b.op.symbol =>
                    {
                        stack.push(Frame::Value(&a.lhs, &b.lhs));
                        stack.push(Frame::Value(&a.rhs, &b.rhs));
                    }
                    (ValueExprKindV1::Call(a), ValueExprKindV1::Call(b)) => {
                        if a.callee.name != b.callee.name || a.args().len() != b.args().len() {
                            return false;
                        }
                        for (a, b) in a.args().as_slice().iter().zip(b.args().as_slice()) {
                            match (a, b) {
                                (CallArgHirV1::Expr(a), CallArgHirV1::Expr(b)) => {
                                    stack.push(Frame::Value(a, b));
                                }
                                (CallArgHirV1::Star { .. }, CallArgHirV1::Star { .. }) => {}
                                _ => return false,
                            }
                        }
                    }
                    (ValueExprKindV1::Record(a), ValueExprKindV1::Record(b)) => {
                        if a.fields.len() != b.fields.len() {
                            return false;
                        }
                        for (a, b) in a.fields.iter().zip(&b.fields) {
                            if a.name != b.name {
                                return false;
                            }
                            stack.push(Frame::Value(&a.value, &b.value));
                        }
                    }
                    (ValueExprKindV1::List(a), ValueExprKindV1::List(b)) => {
                        if a.elements.len() != b.elements.len() {
                            return false;
                        }
                        for (a, b) in a.elements.iter().zip(&b.elements) {
                            stack.push(Frame::Value(a, b));
                        }
                    }
                    _ => return false,
                }
            }
            Frame::Bool(left, right) => {
                let left = strip_bool_parens(left);
                let right = strip_bool_parens(right);
                match (&left.kind, &right.kind) {
                    (BoolExprKindV1::Literal(a), BoolExprKindV1::Literal(b)) if a == b => {}
                    (BoolExprKindV1::Value(a), BoolExprKindV1::Value(b)) => {
                        stack.push(Frame::Value(a, b));
                    }
                    (BoolExprKindV1::Compare(a), BoolExprKindV1::Compare(b))
                        if a.op.symbol == b.op.symbol =>
                    {
                        stack.push(Frame::Value(&a.lhs, &b.lhs));
                        stack.push(Frame::Value(&a.rhs, &b.rhs));
                    }
                    (BoolExprKindV1::And(a), BoolExprKindV1::And(b))
                    | (BoolExprKindV1::Or(a), BoolExprKindV1::Or(b)) => {
                        stack.push(Frame::Bool(&a.lhs, &b.lhs));
                        stack.push(Frame::Bool(&a.rhs, &b.rhs));
                    }
                    (BoolExprKindV1::Not(a), BoolExprKindV1::Not(b)) => {
                        stack.push(Frame::Bool(a, b));
                    }
                    (
                        BoolExprKindV1::LogicalQuantifier(a),
                        BoolExprKindV1::LogicalQuantifier(b),
                    ) => {
                        if a.kind != b.kind
                            || a.binders.as_slice().len() != b.binders.as_slice().len()
                            || !a
                                .binders
                                .as_slice()
                                .iter()
                                .zip(b.binders.as_slice())
                                .all(|(a, b)| a.name == b.name)
                        {
                            return false;
                        }
                        stack.push(Frame::Bool(&a.body, &b.body));
                    }
                    (BoolExprKindV1::RelationExists(a), BoolExprKindV1::RelationExists(b))
                        if a.polarity == b.polarity =>
                    {
                        stack.push(Frame::Relation(&a.relation, &b.relation));
                    }
                    _ => return false,
                }
            }
            Frame::Relation(left, right) => match (&left.kind, &right.kind) {
                (RelationExprKindV1::Source(a), RelationExprKindV1::Source(b)) => {
                    if a.name != b.name
                        || a.alias.as_ref().map(|value| value.name.as_str())
                            != b.alias.as_ref().map(|value| value.name.as_str())
                    {
                        return false;
                    }
                }
                (RelationExprKindV1::Project(a), RelationExprKindV1::Project(b)) => {
                    if a.items().len() != b.items().len() {
                        return false;
                    }
                    stack.push(Frame::Relation(&a.input, &b.input));
                    for (a, b) in a.items().iter().zip(b.items()) {
                        if a.alias.as_ref().map(|value| value.name.as_str())
                            != b.alias.as_ref().map(|value| value.name.as_str())
                        {
                            return false;
                        }
                        stack.push(Frame::Value(&a.value, &b.value));
                    }
                }
                (RelationExprKindV1::Filter(a), RelationExprKindV1::Filter(b)) => {
                    stack.push(Frame::Relation(&a.input, &b.input));
                    stack.push(Frame::Bool(&a.predicate, &b.predicate));
                }
                (RelationExprKindV1::Join(a), RelationExprKindV1::Join(b)) => match (a, b) {
                    (
                        JoinHirV1::Inner {
                            left: al,
                            right: ar,
                            on: ao,
                        },
                        JoinHirV1::Inner {
                            left: bl,
                            right: br,
                            on: bo,
                        },
                    ) => {
                        stack.push(Frame::Relation(al, bl));
                        stack.push(Frame::Relation(ar, br));
                        stack.push(Frame::Bool(ao, bo));
                    }
                    (
                        JoinHirV1::Cross {
                            left: al,
                            right: ar,
                        },
                        JoinHirV1::Cross {
                            left: bl,
                            right: br,
                        },
                    ) => {
                        stack.push(Frame::Relation(al, bl));
                        stack.push(Frame::Relation(ar, br));
                    }
                    _ => return false,
                },
                (RelationExprKindV1::GroupBy(a), RelationExprKindV1::GroupBy(b)) => {
                    if a.keys().len() != b.keys().len() {
                        return false;
                    }
                    stack.push(Frame::Relation(&a.input, &b.input));
                    for (a, b) in a.keys().iter().zip(b.keys()) {
                        stack.push(Frame::Value(a, b));
                    }
                }
                (RelationExprKindV1::OrderBy(a), RelationExprKindV1::OrderBy(b)) => {
                    if a.keys().len() != b.keys().len() {
                        return false;
                    }
                    stack.push(Frame::Relation(&a.input, &b.input));
                    for (a, b) in a.keys().iter().zip(b.keys()) {
                        if a.direction != b.direction {
                            return false;
                        }
                        stack.push(Frame::Value(&a.value, &b.value));
                    }
                }
                _ => return false,
            },
            Frame::Shape(left, right) => match (&left.kind, &right.kind) {
                (ResultShapeKindV1::Leaf(a), ResultShapeKindV1::Leaf(b)) => {
                    stack.push(Frame::Value(a, b));
                }
                (ResultShapeKindV1::Record(a), ResultShapeKindV1::Record(b)) => {
                    if a.fields.len() != b.fields.len() {
                        return false;
                    }
                    for (a, b) in a.fields.iter().zip(&b.fields) {
                        if a.name != b.name {
                            return false;
                        }
                        stack.push(Frame::Shape(&a.shape, &b.shape));
                    }
                }
                (ResultShapeKindV1::List(a), ResultShapeKindV1::List(b)) => {
                    stack.push(Frame::Shape(&a.element, &b.element));
                }
                _ => return false,
            },
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::semantically_equivalent_items;
    use quanta_gqlang_v2_hir::{HirBuilderV1, ProgramItemHirV1, SpanV1};

    fn deep_shape_item(literal: &str) -> ProgramItemHirV1 {
        let builder = HirBuilderV1::new();
        let span = SpanV1::new(0, 1);
        let mut shape = builder.shape_leaf(builder.int(literal, span), span);
        for _ in 0..100_000 {
            shape = builder.shape_list(shape, span);
        }
        ProgramItemHirV1::from(builder.query(
            builder.relation_source("r", span, None, span),
            Some(shape),
            span,
        ))
    }

    #[test]
    fn full_tree_equivalence_is_stack_safe_and_reaches_deep_leaf_payloads() {
        let left = deep_shape_item("1");
        let right = deep_shape_item("2");
        assert!(!semantically_equivalent_items(&left, &right));
        // Dropping this intentionally adversarial raw tree is HIR-owner work;
        // keep this rail scoped to the comparator's stack behavior.
        std::mem::forget(left);
        std::mem::forget(right);
    }
}
