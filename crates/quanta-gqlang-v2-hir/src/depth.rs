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

/// A request-owned depth scan can fail before its finite scratch is created.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HirDepthAdmissionErrorV3 {
    /// The existing source-depth limit was exceeded.
    Depth(HirDepthDiagnosticV2),
    /// Finite scratch could not be created or a traversal exceeded its census.
    Storage,
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
    scratch_slots: Option<usize>,
    storage_error: bool,
}

impl DepthGuard {
    fn push_frame<'a>(&mut self, stack: &mut Vec<Frame<'a>>, frame: Frame<'a>) -> Option<()> {
        if self.scratch_slots.is_some_and(|slots| stack.len() >= slots) {
            self.storage_error = true;
            return None;
        }
        stack.push(frame);
        Some(())
    }

    fn scan(&mut self, root: Frame<'_>) {
        if self.error.is_some() || self.storage_error {
            return;
        }
        let mut stack = match self.scratch_slots {
            Some(slots) => {
                if slots == 0 {
                    self.storage_error = true;
                    return;
                }
                let mut stack = Vec::new();
                if stack.try_reserve_exact(slots).is_err() || stack.capacity() > slots {
                    self.storage_error = true;
                    return;
                }
                stack.push(root);
                stack
            }
            None => vec![root],
        };
        while let Some(frame) = stack.pop() {
            if self.error.is_some() || self.storage_error {
                return;
            }
            let complete = match frame {
                Frame::Value(node, expression_depth, relation_depth) => {
                    self.scan_value(node, expression_depth, relation_depth, &mut stack)
                }
                Frame::Bool(node, expression_depth, relation_depth) => {
                    self.scan_bool(node, expression_depth, relation_depth, &mut stack)
                }
                Frame::Relation(node, relation_depth, expression_depth) => {
                    self.scan_relation(node, relation_depth, expression_depth, &mut stack)
                }
                Frame::Shape(node, shape_depth) => self.scan_shape(node, shape_depth, &mut stack),
            };
            if complete.is_none() {
                return;
            }
        }
    }

    fn scan_value<'a>(
        &mut self,
        node: &'a ValueExprHirV1,
        expression_depth: usize,
        relation_depth: usize,
        stack: &mut Vec<Frame<'a>>,
    ) -> Option<()> {
        if expression_depth > self.expression_limit {
            self.error = Some(HirDepthDiagnosticV2 {
                dimension: HirDepthDimensionV2::Expression,
                limit: self.expression_limit,
                observed: expression_depth,
                span: node.span,
            });
            return None;
        }
        let child = expression_depth.saturating_add(1);
        match &node.kind {
            ValueExprKindV1::Literal(_) | ValueExprKindV1::Symbol(_) => {}
            ValueExprKindV1::Path(path) => {
                self.push_frame(stack, Frame::Value(&path.base, child, relation_depth))?;
            }
            ValueExprKindV1::Unary(unary) => {
                self.push_frame(stack, Frame::Value(&unary.operand, child, relation_depth))?;
            }
            ValueExprKindV1::Binary(binary) => {
                self.push_frame(stack, Frame::Value(&binary.rhs, child, relation_depth))?;
                self.push_frame(stack, Frame::Value(&binary.lhs, child, relation_depth))?;
            }
            ValueExprKindV1::Call(call) => {
                for arg in call.args().as_slice().iter().rev() {
                    if let CallArgHirV1::Expr(value) = arg {
                        self.push_frame(stack, Frame::Value(value, child, relation_depth))?;
                    }
                }
            }
            ValueExprKindV1::Paren(inner) => {
                self.push_frame(stack, Frame::Value(inner, child, relation_depth))?;
            }
            ValueExprKindV1::Record(record) => {
                for field in record.fields.iter().rev() {
                    self.push_frame(stack, Frame::Value(&field.value, child, relation_depth))?;
                }
            }
            ValueExprKindV1::List(list) => {
                for value in list.elements.iter().rev() {
                    self.push_frame(stack, Frame::Value(value, child, relation_depth))?;
                }
            }
        }

        Some(())
    }

    fn scan_bool<'a>(
        &mut self,
        node: &'a BoolExprHirV1,
        expression_depth: usize,
        relation_depth: usize,
        stack: &mut Vec<Frame<'a>>,
    ) -> Option<()> {
        if expression_depth > self.expression_limit {
            self.error = Some(HirDepthDiagnosticV2 {
                dimension: HirDepthDimensionV2::Expression,
                limit: self.expression_limit,
                observed: expression_depth,
                span: node.span,
            });
            return None;
        }
        let child = expression_depth.saturating_add(1);
        match &node.kind {
            BoolExprKindV1::Literal(_) => {}
            BoolExprKindV1::Value(value) => {
                self.push_frame(stack, Frame::Value(value, child, relation_depth))?;
            }
            BoolExprKindV1::Compare(compare) => {
                self.push_frame(stack, Frame::Value(&compare.rhs, child, relation_depth))?;
                self.push_frame(stack, Frame::Value(&compare.lhs, child, relation_depth))?;
            }
            BoolExprKindV1::And(binary) | BoolExprKindV1::Or(binary) => {
                self.push_frame(stack, Frame::Bool(&binary.rhs, child, relation_depth))?;
                self.push_frame(stack, Frame::Bool(&binary.lhs, child, relation_depth))?;
            }
            BoolExprKindV1::Not(inner) | BoolExprKindV1::Paren(inner) => {
                self.push_frame(stack, Frame::Bool(inner, child, relation_depth))?;
            }
            BoolExprKindV1::LogicalQuantifier(quantifier) => {
                self.push_frame(stack, Frame::Bool(&quantifier.body, child, relation_depth))?;
            }
            BoolExprKindV1::RelationExists(exists) => {
                self.push_frame(
                    stack,
                    Frame::Relation(
                        &exists.relation,
                        relation_depth.saturating_add(1),
                        expression_depth,
                    ),
                )?;
            }
        }

        Some(())
    }

    fn scan_relation<'a>(
        &mut self,
        node: &'a RelationExprHirV1,
        relation_depth: usize,
        expression_depth: usize,
        stack: &mut Vec<Frame<'a>>,
    ) -> Option<()> {
        if relation_depth > self.relation_limit {
            self.error = Some(HirDepthDiagnosticV2 {
                dimension: HirDepthDimensionV2::Relation,
                limit: self.relation_limit,
                observed: relation_depth,
                span: node.span,
            });
            return None;
        }
        let relation_child = relation_depth.saturating_add(1);
        let expression_child = expression_depth.saturating_add(1);
        match &node.kind {
            RelationExprKindV1::Source(_) => {}
            RelationExprKindV1::Project(project) => {
                for item in project.items().iter().rev() {
                    self.push_frame(
                        stack,
                        Frame::Value(&item.value, expression_child, relation_depth),
                    )?;
                }
                self.push_frame(
                    stack,
                    Frame::Relation(&project.input, relation_child, expression_depth),
                )?;
            }
            RelationExprKindV1::Filter(filter) => {
                self.push_frame(
                    stack,
                    Frame::Bool(&filter.predicate, expression_child, relation_depth),
                )?;
                self.push_frame(
                    stack,
                    Frame::Relation(&filter.input, relation_child, expression_depth),
                )?;
            }
            RelationExprKindV1::Join(join) => match join {
                JoinHirV1::Inner { left, right, on } => {
                    self.push_frame(stack, Frame::Bool(on, expression_child, relation_depth))?;
                    self.push_frame(
                        stack,
                        Frame::Relation(right, relation_child, expression_depth),
                    )?;
                    self.push_frame(
                        stack,
                        Frame::Relation(left, relation_child, expression_depth),
                    )?;
                }
                JoinHirV1::Cross { left, right } => {
                    self.push_frame(
                        stack,
                        Frame::Relation(right, relation_child, expression_depth),
                    )?;
                    self.push_frame(
                        stack,
                        Frame::Relation(left, relation_child, expression_depth),
                    )?;
                }
            },
            RelationExprKindV1::GroupBy(group) => {
                for key in group.keys().iter().rev() {
                    self.push_frame(stack, Frame::Value(key, expression_child, relation_depth))?;
                }
                self.push_frame(
                    stack,
                    Frame::Relation(&group.input, relation_child, expression_depth),
                )?;
            }
            RelationExprKindV1::Limit(limit) => {
                self.push_frame(
                    stack,
                    Frame::Relation(&limit.input, relation_child, expression_depth),
                )?;
            }
            RelationExprKindV1::OrderBy(order) => {
                for key in order.keys().iter().rev() {
                    self.push_frame(
                        stack,
                        Frame::Value(&key.value, expression_child, relation_depth),
                    )?;
                }
                self.push_frame(
                    stack,
                    Frame::Relation(&order.input, relation_child, expression_depth),
                )?;
            }
        }

        Some(())
    }

    fn scan_shape<'a>(
        &mut self,
        node: &'a ResultShapeHirV1,
        shape_depth: usize,
        stack: &mut Vec<Frame<'a>>,
    ) -> Option<()> {
        if shape_depth > self.shape_limit {
            self.error = Some(HirDepthDiagnosticV2 {
                dimension: HirDepthDimensionV2::Shape,
                limit: self.shape_limit,
                observed: shape_depth,
                span: node.span,
            });
            return None;
        }
        let child = shape_depth.saturating_add(1);
        match &node.kind {
            ResultShapeKindV1::Leaf(value) => {
                self.push_frame(stack, Frame::Value(value, 1, 0))?;
            }
            ResultShapeKindV1::Record(record) => {
                for field in record.fields.iter().rev() {
                    self.push_frame(stack, Frame::Shape(&field.shape, child))?;
                }
            }
            ResultShapeKindV1::List(list) => {
                self.push_frame(stack, Frame::Shape(&list.element, child))?;
            }
        }

        Some(())
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
        scratch_slots: None,
        storage_error: false,
    };
    root.accept(&mut guard);
    guard.error.map_or(Ok(()), Err)
}

/// The requested heap layout of one iterative depth stack with at most one
/// frame per HIR node issued by the source parser's builder.
#[must_use]
pub fn depth_stack_storage_bytes_v3(issued_node_slots: usize) -> Option<usize> {
    issued_node_slots.checked_mul(std::mem::size_of::<Frame<'static>>())
}

/// Check the same depth contract using only caller-admitted scratch slots.
/// The source parser must reserve `depth_stack_storage_bytes_v3` for its
/// item's issued-node census before calling this function. The caller keeps
/// that temporary reservation until the scan returns. No extra Vec growth is
/// allowed even when the allocator reports spare capacity.
pub fn check_depths_with_source_slots_v3(
    root: HirRootRefV1<'_>,
    expression_limit: usize,
    relation_limit: usize,
    shape_limit: usize,
    issued_node_slots: usize,
) -> Result<(), HirDepthAdmissionErrorV3> {
    let mut guard = DepthGuard {
        expression_limit,
        relation_limit,
        shape_limit,
        error: None,
        scratch_slots: Some(issued_node_slots),
        storage_error: false,
    };
    root.accept(&mut guard);
    if guard.storage_error {
        Err(HirDepthAdmissionErrorV3::Storage)
    } else {
        guard
            .error
            .map_or(Ok(()), |error| Err(HirDepthAdmissionErrorV3::Depth(error)))
    }
}

#[cfg(test)]
mod request_scratch_tests {
    use super::*;
    use crate::build::HirBuilderV1;

    #[test]
    fn issued_node_scratch_accepts_real_expr_and_refuses_short_slots_v3() {
        let builder = HirBuilderV1::new();
        let span = SpanV1::new(0, 5);
        let lhs = builder.int("1", SpanV1::new(0, 1));
        let rhs = builder.int("2", SpanV1::new(4, 5));
        let binary = builder.binary("+", lhs, rhs, span);
        let slots = 3;
        assert_eq!(builder.minted(), 3);
        assert_eq!(
            depth_stack_storage_bytes_v3(slots),
            Some(slots * std::mem::size_of::<Frame<'static>>())
        );
        assert_eq!(
            check_depths_with_source_slots_v3(HirRootRefV1::Value(&binary), 64, 128, 64, slots),
            Ok(())
        );
        assert_eq!(
            check_depths_with_source_slots_v3(HirRootRefV1::Value(&binary), 64, 128, 64, 1),
            Err(HirDepthAdmissionErrorV3::Storage)
        );
    }
}
