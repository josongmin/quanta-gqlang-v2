//! Stack-safe structural expression-depth admission for parsed HIR.

use crate::budget::ParseBudgetDimensionV2;
use crate::diag::ParseDiagnosticV1;
use crate::parser::SurfaceItemV1;
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CallArgHirV1, HirVisitorV1, JoinHirV1, RelationExprHirV1,
    RelationExprKindV1, ValueExprHirV1, ValueExprKindV1, MAX_SAFE_RELATION_DEPTH_V2,
};

enum Expression<'a> {
    Value(&'a ValueExprHirV1, usize),
    Bool(&'a BoolExprHirV1, usize),
}

struct DepthGuard {
    expression_limit: usize,
    relation_limit: usize,
    inherited_depth: usize,
    inherited_relation_depth: usize,
    error: Option<ParseDiagnosticV1>,
}

impl DepthGuard {
    fn new(expression_limit: usize, relation_limit: usize) -> Self {
        Self {
            expression_limit,
            relation_limit,
            inherited_depth: 0,
            inherited_relation_depth: 0,
            error: None,
        }
    }

    fn finish(self) -> Result<(), ParseDiagnosticV1> {
        self.error.map_or(Ok(()), Err)
    }

    fn scan_expression(&mut self, root: Expression<'_>) {
        let mut stack = vec![root];
        while let Some(expression) = stack.pop() {
            if self.error.is_some() {
                return;
            }
            match expression {
                Expression::Value(node, depth) => {
                    if depth > self.expression_limit {
                        self.error = Some(ParseDiagnosticV1::resource_exhausted(
                            ParseBudgetDimensionV2::ExpressionDepth,
                            self.expression_limit,
                            depth,
                            node.span,
                        ));
                        return;
                    }
                    let child_depth = depth.saturating_add(1);
                    match &node.kind {
                        ValueExprKindV1::Literal(_) | ValueExprKindV1::Symbol(_) => {}
                        ValueExprKindV1::Path(path) => {
                            stack.push(Expression::Value(&path.base, child_depth));
                        }
                        ValueExprKindV1::Unary(unary) => {
                            stack.push(Expression::Value(&unary.operand, child_depth));
                        }
                        ValueExprKindV1::Binary(binary) => {
                            stack.push(Expression::Value(&binary.rhs, child_depth));
                            stack.push(Expression::Value(&binary.lhs, child_depth));
                        }
                        ValueExprKindV1::Call(call) => {
                            for argument in call.args().as_slice().iter().rev() {
                                if let CallArgHirV1::Expr(value) = argument {
                                    stack.push(Expression::Value(value, child_depth));
                                }
                            }
                        }
                        ValueExprKindV1::Paren(inner) => {
                            stack.push(Expression::Value(inner, child_depth));
                        }
                        ValueExprKindV1::Record(record) => {
                            for field in record.fields.iter().rev() {
                                stack.push(Expression::Value(&field.value, child_depth));
                            }
                        }
                        ValueExprKindV1::List(list) => {
                            for value in list.elements.iter().rev() {
                                stack.push(Expression::Value(value, child_depth));
                            }
                        }
                    }
                }
                Expression::Bool(node, depth) => {
                    if depth > self.expression_limit {
                        self.error = Some(ParseDiagnosticV1::resource_exhausted(
                            ParseBudgetDimensionV2::ExpressionDepth,
                            self.expression_limit,
                            depth,
                            node.span,
                        ));
                        return;
                    }
                    let child_depth = depth.saturating_add(1);
                    match &node.kind {
                        BoolExprKindV1::Literal(_) => {}
                        BoolExprKindV1::Value(value) => {
                            stack.push(Expression::Value(value, child_depth));
                        }
                        BoolExprKindV1::Compare(compare) => {
                            stack.push(Expression::Value(&compare.rhs, child_depth));
                            stack.push(Expression::Value(&compare.lhs, child_depth));
                        }
                        BoolExprKindV1::And(binary) | BoolExprKindV1::Or(binary) => {
                            stack.push(Expression::Bool(&binary.rhs, child_depth));
                            stack.push(Expression::Bool(&binary.lhs, child_depth));
                        }
                        BoolExprKindV1::Not(inner) | BoolExprKindV1::Paren(inner) => {
                            stack.push(Expression::Bool(inner, child_depth));
                        }
                        BoolExprKindV1::LogicalQuantifier(quantifier) => {
                            stack.push(Expression::Bool(&quantifier.body, child_depth));
                        }
                        BoolExprKindV1::RelationExists(exists) => {
                            let previous = self.inherited_depth;
                            self.inherited_depth = depth;
                            self.visit_relation(&exists.relation);
                            self.inherited_depth = previous;
                        }
                    }
                }
            }
        }
    }
}

impl HirVisitorV1 for DepthGuard {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        self.scan_expression(Expression::Value(
            node,
            self.inherited_depth.saturating_add(1),
        ));
    }

    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        self.scan_expression(Expression::Bool(
            node,
            self.inherited_depth.saturating_add(1),
        ));
    }

    fn visit_relation(&mut self, root: &RelationExprHirV1) {
        let previous = self.inherited_relation_depth;
        let mut stack = vec![(root, previous.saturating_add(1))];
        while let Some((node, depth)) = stack.pop() {
            if self.error.is_some() {
                break;
            }
            if depth > self.relation_limit {
                self.error = Some(ParseDiagnosticV1::resource_exhausted(
                    ParseBudgetDimensionV2::RelationDepth,
                    self.relation_limit,
                    depth,
                    node.span,
                ));
                break;
            }
            self.inherited_relation_depth = depth;
            let child_depth = depth.saturating_add(1);
            match &node.kind {
                RelationExprKindV1::Source(_) => {}
                RelationExprKindV1::Project(project) => {
                    for item in project.items() {
                        self.visit_value(&item.value);
                    }
                    stack.push((&project.input, child_depth));
                }
                RelationExprKindV1::Filter(filter) => {
                    self.visit_bool(&filter.predicate);
                    stack.push((&filter.input, child_depth));
                }
                RelationExprKindV1::Join(join) => {
                    let (left, right) = match join {
                        JoinHirV1::Inner { left, right, on } => {
                            self.visit_bool(on);
                            (left, right)
                        }
                        JoinHirV1::Cross { left, right } => (left, right),
                    };
                    stack.push((right, child_depth));
                    stack.push((left, child_depth));
                }
                RelationExprKindV1::GroupBy(group) => {
                    for key in group.keys() {
                        self.visit_value(key);
                    }
                    stack.push((&group.input, child_depth));
                }
                RelationExprKindV1::OrderBy(order) => {
                    for key in order.keys() {
                        self.visit_value(&key.value);
                    }
                    stack.push((&order.input, child_depth));
                }
            }
        }
        self.inherited_relation_depth = previous;
    }
}

pub(crate) fn check_item(
    item: &SurfaceItemV1,
    expression_limit: usize,
    relation_limit: usize,
) -> Result<(), ParseDiagnosticV1> {
    let mut guard = DepthGuard::new(expression_limit, relation_limit);
    match item {
        SurfaceItemV1::Query(query) => guard.visit_query(query),
        SurfaceItemV1::Decl(decl) => guard.visit_decl(decl),
        SurfaceItemV1::Rule(rule) => guard.visit_rule(rule),
        SurfaceItemV1::Program(program) => guard.visit_item(program),
    }
    guard.finish()
}

pub(crate) fn check_value(value: &ValueExprHirV1, limit: usize) -> Result<(), ParseDiagnosticV1> {
    let mut guard = DepthGuard::new(limit, MAX_SAFE_RELATION_DEPTH_V2);
    guard.visit_value(value);
    guard.finish()
}

pub(crate) fn check_bool(value: &BoolExprHirV1, limit: usize) -> Result<(), ParseDiagnosticV1> {
    let mut guard = DepthGuard::new(limit, MAX_SAFE_RELATION_DEPTH_V2);
    guard.visit_bool(value);
    guard.finish()
}
