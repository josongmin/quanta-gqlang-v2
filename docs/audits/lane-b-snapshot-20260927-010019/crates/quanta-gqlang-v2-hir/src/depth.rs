//! Stack-safe depth admission shared by parsed and programmatically built HIR.

use crate::expr::{BoolExprHirV1, BoolExprKindV1, CallArgHirV1, ValueExprHirV1, ValueExprKindV1};
use crate::integrity::HirRootRefV1;
use crate::relation::{JoinHirV1, RelationExprHirV1, RelationExprKindV1};
use crate::shape::{ResultShapeHirV1, ResultShapeKindV1};
use crate::span::SpanV1;
use crate::visit::HirVisitorV1;

/// Recursive HIR dimension subject to a stack-safety ceiling.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HirDepthDimensionV2 {
    Expression,
    Relation,
    Shape,
}

impl HirDepthDimensionV2 {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expression => "expression_depth",
            Self::Relation => "relation_depth",
            Self::Shape => "shape_depth",
        }
    }
}

/// First depth violation in deterministic pre-order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HirDepthDiagnosticV2 {
    pub dimension: HirDepthDimensionV2,
    pub limit: usize,
    pub observed: usize,
    pub span: SpanV1,
}

enum Frame<'a> {
    Value(&'a ValueExprHirV1, usize, usize),
    Bool(&'a BoolExprHirV1, usize, usize),
    Relation(&'a RelationExprHirV1, usize, usize),
    Shape(&'a ResultShapeHirV1, usize),
}

struct DepthGuard {
    expression_limit: usize,
    relation_limit: usize,
    shape_limit: usize,
    error: Option<HirDepthDiagnosticV2>,
}

impl DepthGuard {
    fn scan(&mut self, root: Frame<'_>) {
        let mut stack = vec![root];
        while let Some(frame) = stack.pop() {
            if self.error.is_some() {
                return;
            }
            match frame {
                Frame::Value(node, expression_depth, relation_depth) => {
                    if expression_depth > self.expression_limit {
                        self.error = Some(HirDepthDiagnosticV2 {
                            dimension: HirDepthDimensionV2::Expression,
                            limit: self.expression_limit,
                            observed: expression_depth,
                            span: node.span,
                        });
                        return;
                    }
                    let child = expression_depth.saturating_add(1);
                    match &node.kind {
                        ValueExprKindV1::Literal(_) | ValueExprKindV1::Symbol(_) => {}
                        ValueExprKindV1::Path(path) => {
                            stack.push(Frame::Value(&path.base, child, relation_depth));
                        }
                        ValueExprKindV1::Unary(unary) => {
                            stack.push(Frame::Value(&unary.operand, child, relation_depth));
                        }
                        ValueExprKindV1::Binary(binary) => {
                            stack.push(Frame::Value(&binary.rhs, child, relation_depth));
                            stack.push(Frame::Value(&binary.lhs, child, relation_depth));
                        }
                        ValueExprKindV1::Call(call) => {
                            for arg in call.args().as_slice().iter().rev() {
                                if let CallArgHirV1::Expr(value) = arg {
                                    stack.push(Frame::Value(value, child, relation_depth));
                                }
                            }
                        }
                        ValueExprKindV1::Paren(inner) => {
                            stack.push(Frame::Value(inner, child, relation_depth));
                        }
                        ValueExprKindV1::Record(record) => {
                            for field in record.fields.iter().rev() {
                                stack.push(Frame::Value(&field.value, child, relation_depth));
                            }
                        }
                        ValueExprKindV1::List(list) => {
                            for value in list.elements.iter().rev() {
                                stack.push(Frame::Value(value, child, relation_depth));
                            }
                        }
                    }
                }
                Frame::Bool(node, expression_depth, relation_depth) => {
                    if expression_depth > self.expression_limit {
                        self.error = Some(HirDepthDiagnosticV2 {
                            dimension: HirDepthDimensionV2::Expression,
                            limit: self.expression_limit,
                            observed: expression_depth,
                            span: node.span,
                        });
                        return;
                    }
                    let child = expression_depth.saturating_add(1);
                    match &node.kind {
                        BoolExprKindV1::Literal(_) => {}
                        BoolExprKindV1::Value(value) => {
                            stack.push(Frame::Value(value, child, relation_depth));
                        }
                        BoolExprKindV1::Compare(compare) => {
                            stack.push(Frame::Value(&compare.rhs, child, relation_depth));
                            stack.push(Frame::Value(&compare.lhs, child, relation_depth));
                        }
                        BoolExprKindV1::And(binary) | BoolExprKindV1::Or(binary) => {
                            stack.push(Frame::Bool(&binary.rhs, child, relation_depth));
                            stack.push(Frame::Bool(&binary.lhs, child, relation_depth));
                        }
                        BoolExprKindV1::Not(inner) | BoolExprKindV1::Paren(inner) => {
                            stack.push(Frame::Bool(inner, child, relation_depth));
                        }
                        BoolExprKindV1::LogicalQuantifier(quantifier) => {
                            stack.push(Frame::Bool(&quantifier.body, child, relation_depth));
                        }
                        BoolExprKindV1::RelationExists(exists) => {
                            stack.push(Frame::Relation(
                                &exists.relation,
                                relation_depth.saturating_add(1),
                                expression_depth,
                            ));
                        }
                    }
                }
                Frame::Relation(node, relation_depth, expression_depth) => {
                    if relation_depth > self.relation_limit {
                        self.error = Some(HirDepthDiagnosticV2 {
                            dimension: HirDepthDimensionV2::Relation,
                            limit: self.relation_limit,
                            observed: relation_depth,
                            span: node.span,
                        });
                        return;
                    }
                    let relation_child = relation_depth.saturating_add(1);
                    let expression_child = expression_depth.saturating_add(1);
                    match &node.kind {
                        RelationExprKindV1::Source(_) => {}
                        RelationExprKindV1::Project(project) => {
                            for item in project.items().iter().rev() {
                                stack.push(Frame::Value(
                                    &item.value,
                                    expression_child,
                                    relation_depth,
                                ));
                            }
                            stack.push(Frame::Relation(
                                &project.input,
                                relation_child,
                                expression_depth,
                            ));
                        }
                        RelationExprKindV1::Filter(filter) => {
                            stack.push(Frame::Bool(
                                &filter.predicate,
                                expression_child,
                                relation_depth,
                            ));
                            stack.push(Frame::Relation(
                                &filter.input,
                                relation_child,
                                expression_depth,
                            ));
                        }
                        RelationExprKindV1::Join(join) => match join {
                            JoinHirV1::Inner { left, right, on } => {
                                stack.push(Frame::Bool(on, expression_child, relation_depth));
                                stack.push(Frame::Relation(
                                    right,
                                    relation_child,
                                    expression_depth,
                                ));
                                stack.push(Frame::Relation(left, relation_child, expression_depth));
                            }
                            JoinHirV1::Cross { left, right } => {
                                stack.push(Frame::Relation(
                                    right,
                                    relation_child,
                                    expression_depth,
                                ));
                                stack.push(Frame::Relation(left, relation_child, expression_depth));
                            }
                        },
                        RelationExprKindV1::GroupBy(group) => {
                            for key in group.keys().iter().rev() {
                                stack.push(Frame::Value(key, expression_child, relation_depth));
                            }
                            stack.push(Frame::Relation(
                                &group.input,
                                relation_child,
                                expression_depth,
                            ));
                        }
                        RelationExprKindV1::OrderBy(order) => {
                            for key in order.keys().iter().rev() {
                                stack.push(Frame::Value(
                                    &key.value,
                                    expression_child,
                                    relation_depth,
                                ));
                            }
                            stack.push(Frame::Relation(
                                &order.input,
                                relation_child,
                                expression_depth,
                            ));
                        }
                    }
                }
                Frame::Shape(node, shape_depth) => {
                    if shape_depth > self.shape_limit {
                        self.error = Some(HirDepthDiagnosticV2 {
                            dimension: HirDepthDimensionV2::Shape,
                            limit: self.shape_limit,
                            observed: shape_depth,
                            span: node.span,
                        });
                        return;
                    }
                    let child = shape_depth.saturating_add(1);
                    match &node.kind {
                        ResultShapeKindV1::Leaf(value) => {
                            stack.push(Frame::Value(value, 1, 0));
                        }
                        ResultShapeKindV1::Record(record) => {
                            for field in record.fields.iter().rev() {
                                stack.push(Frame::Shape(&field.shape, child));
                            }
                        }
                        ResultShapeKindV1::List(list) => {
                            stack.push(Frame::Shape(&list.element, child));
                        }
                    }
                }
            }
        }
    }
}

impl HirVisitorV1 for DepthGuard {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        self.scan(Frame::Value(node, 1, 0));
    }

    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        self.scan(Frame::Bool(node, 1, 0));
    }

    fn visit_relation(&mut self, node: &RelationExprHirV1) {
        self.scan(Frame::Relation(node, 1, 0));
    }

    fn visit_shape(&mut self, node: &ResultShapeHirV1) {
        self.scan(Frame::Shape(node, 1));
    }
}

/// Check all recursive HIR dimensions without using the Rust call stack.
pub fn check_depths_v2(
    root: HirRootRefV1<'_>,
    expression_limit: usize,
    relation_limit: usize,
    shape_limit: usize,
) -> Result<(), HirDepthDiagnosticV2> {
    let mut guard = DepthGuard {
        expression_limit,
        relation_limit,
        shape_limit,
        error: None,
    };
    root.accept(&mut guard);
    guard.error.map_or(Ok(()), Err)
}
