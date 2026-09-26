//! Value evaluation over syntax children and resolved declaration dependencies.
//!
//! HIR depth bounds syntax, not a chain of shallow declaration bodies. Both
//! kinds of edge therefore use the same explicit work stack. Continuations
//! retain left-to-right evaluation, per-argument contexts and early rejection;
//! no expression is restarted when a forward declaration is needed.

use super::{
    DocumentChecker, ExpressionTypeStateV2, ResolvedTypeInputValueV2, TypedExpressionFactV2,
    TypedLiteralFactV2, UnknownTypeCauseV2,
};
use crate::{
    CardinalityV1, ExpressionEffectsV2, RecordFieldTypeV1, RecordTypeV1, TypeDiagnosticV2,
    TypedLiteralValueV2, ValueTypeV1,
};
use quanta_gqlang_v2_hir::{
    BinaryExprV1, CallArgHirV1, CallExprV1, ListExprV1, LiteralV1, LocatedSpanV2, PathAccessV1,
    RecordExprV1, UnaryExprV1, ValueExprHirV1, ValueExprKindV1,
};
use std::collections::BTreeSet;

enum ValueFrame<'a> {
    Enter(&'a ValueExprHirV1, bool),
    Copy(&'a ValueExprHirV1, &'a ValueExprHirV1),
    Unary(&'a ValueExprHirV1, &'a UnaryExprV1, bool),
    Binary(&'a ValueExprHirV1, &'a BinaryExprV1),
    Path(&'a ValueExprHirV1, &'a PathAccessV1),
    Call {
        node: &'a ValueExprHirV1,
        call: &'a CallExprV1,
        input: ResolvedTypeInputValueV2,
        contexts: Vec<Option<ValueTypeV1>>,
        next: usize,
        types: Vec<ExpressionTypeStateV2>,
    },
    Record {
        node: &'a ValueExprHirV1,
        record: &'a RecordExprV1,
        next: usize,
        fields: Vec<RecordFieldTypeV1>,
        unknown: Option<UnknownTypeCauseV2>,
        effects: ExpressionEffectsV2,
    },
    List {
        node: &'a ValueExprHirV1,
        list: &'a ListExprV1,
        next: usize,
        selected: Option<ValueTypeV1>,
        unknown: Option<UnknownTypeCauseV2>,
        effects: ExpressionEffectsV2,
    },
}

fn value_fact(state: ExpressionTypeStateV2, effects: ExpressionEffectsV2) -> TypedExpressionFactV2 {
    TypedExpressionFactV2 {
        state,
        operator_id: None,
        resolved_call: None,
        intrinsic_call_aggregation: None,
        effects,
        literal: None,
    }
}

impl DocumentChecker<'_, '_> {
    fn value_state(
        &self,
        node: &ValueExprHirV1,
    ) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        self.facts
            .get(&self.key(node))
            .map(|fact| fact.state.clone())
            .ok_or(TypeDiagnosticV2::MissingNodeFact {
                node_id: node.id,
                location: self.location(node),
            })
    }

    pub(super) fn type_expr(
        &mut self,
        root: &ValueExprHirV1,
        deferred_min_child: bool,
    ) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let mut stack = vec![ValueFrame::Enter(root, deferred_min_child)];
        let mut active = BTreeSet::new();
        while let Some(frame) = stack.pop() {
            let (node, fact) = match frame {
                ValueFrame::Enter(node, deferred) => {
                    let key = self.key(node);
                    if self.facts.contains_key(&key) {
                        continue;
                    }
                    if !active.insert(node.id) {
                        return Err(TypeDiagnosticV2::CyclicDeclarationDependency {
                            node_id: node.id,
                            location: self.location(node),
                        });
                    }
                    match &node.kind {
                        ValueExprKindV1::Literal(value) => {
                            let (ty, literal) = self.type_literal(node, value, deferred)?;
                            let mut fact = value_fact(
                                ExpressionTypeStateV2::Known(ty),
                                ExpressionEffectsV2::pure_scalar(),
                            );
                            fact.literal = Some(literal);
                            (node, fact)
                        }
                        ValueExprKindV1::Symbol(_) => {
                            let fact = match self.inputs.values.get(&key) {
                                Some(ResolvedTypeInputValueV2::Binding(state)) => {
                                    value_fact(state.clone(), ExpressionEffectsV2::pure_scalar())
                                }
                                Some(ResolvedTypeInputValueV2::BindingFromExpression(target)) => {
                                    let fact = self.facts.get(target).ok_or(
                                        TypeDiagnosticV2::UnavailableBindingDependency {
                                            node_id: node.id,
                                            target_node_id: target.node_id,
                                            location: self.location(node),
                                        },
                                    )?;
                                    value_fact(fact.state.clone(), fact.effects)
                                }
                                Some(ResolvedTypeInputValueV2::BindingFromDeclaration(target)) => {
                                    let body = self
                                        .inputs
                                        .declaration_bodies
                                        .get(&target.node_id)
                                        .copied()
                                        .ok_or(TypeDiagnosticV2::InvalidBindingDependency {
                                            node_id: node.id,
                                            target_node_id: target.node_id,
                                            location: self.location(node),
                                        })?;
                                    stack.push(ValueFrame::Copy(node, body));
                                    stack.push(ValueFrame::Enter(body, false));
                                    continue;
                                }
                                Some(ResolvedTypeInputValueV2::BindingFromRelationRow(target)) => {
                                    let row = self.relation_rows.get(target).ok_or(
                                        TypeDiagnosticV2::UnavailableRelationRowDependency {
                                            root: node.id,
                                            target: target.node_id,
                                            location: self.location(node),
                                        },
                                    )?;
                                    value_fact(
                                        ExpressionTypeStateV2::Known(ValueTypeV1::try_record(
                                            row.record.clone(),
                                        )?),
                                        ExpressionEffectsV2::pure_scalar(),
                                    )
                                }
                                _ => {
                                    return Err(TypeDiagnosticV2::MissingNodeFact {
                                        node_id: node.id,
                                        location: self.location(node),
                                    })
                                }
                            };
                            (node, fact)
                        }
                        ValueExprKindV1::Paren(inner) => {
                            stack.push(ValueFrame::Copy(node, inner));
                            stack.push(ValueFrame::Enter(inner, deferred));
                            continue;
                        }
                        ValueExprKindV1::Unary(unary) => {
                            let mut magnitude = &unary.operand;
                            while let ValueExprKindV1::Paren(inner) = &magnitude.kind {
                                magnitude = inner;
                            }
                            let min_case = unary.op.symbol == "-"
                                && matches!(&magnitude.kind, ValueExprKindV1::Literal(LiteralV1::Int(text)) if text == "170141183460469231731687303715884105728");
                            stack.push(ValueFrame::Unary(node, unary, min_case));
                            stack.push(ValueFrame::Enter(&unary.operand, min_case));
                            continue;
                        }
                        ValueExprKindV1::Binary(binary) => {
                            stack.push(ValueFrame::Binary(node, binary));
                            stack.push(ValueFrame::Enter(&binary.rhs, false));
                            stack.push(ValueFrame::Enter(&binary.lhs, false));
                            continue;
                        }
                        ValueExprKindV1::Path(path) => {
                            stack.push(ValueFrame::Path(node, path));
                            stack.push(ValueFrame::Enter(&path.base, false));
                            continue;
                        }
                        ValueExprKindV1::Call(call) => {
                            let input = self
                                .inputs
                                .values
                                .get(&key)
                                .ok_or(TypeDiagnosticV2::MissingNodeFact {
                                    node_id: node.id,
                                    location: self.location(node),
                                })?
                                .clone();
                            let contexts =
                                self.callable_argument_contexts(&input, call.args().as_slice());
                            stack.push(ValueFrame::Call {
                                node,
                                call,
                                input,
                                contexts,
                                next: 0,
                                types: Vec::new(),
                            });
                            continue;
                        }
                        ValueExprKindV1::Record(record) => {
                            stack.push(ValueFrame::Record {
                                node,
                                record,
                                next: 0,
                                fields: Vec::new(),
                                unknown: None,
                                effects: ExpressionEffectsV2::pure_scalar(),
                            });
                            continue;
                        }
                        ValueExprKindV1::List(list) => {
                            stack.push(ValueFrame::List {
                                node,
                                list,
                                next: 0,
                                selected: None,
                                unknown: None,
                                effects: ExpressionEffectsV2::pure_scalar(),
                            });
                            continue;
                        }
                    }
                }
                ValueFrame::Copy(node, child) => (
                    node,
                    value_fact(self.value_state(child)?, self.effects_for(child.id)?),
                ),
                ValueFrame::Unary(node, unary, min_case) => {
                    let operand = self.value_state(&unary.operand)?;
                    let mut operator_id = None;
                    let mut effects = ExpressionEffectsV2::pure_scalar();
                    let state = self.resolve_unary(
                        node,
                        &unary.op.symbol,
                        if min_case {
                            ExpressionTypeStateV2::Known(ValueTypeV1::int())
                        } else {
                            operand
                        },
                        &mut operator_id,
                        &mut effects,
                    )?;
                    let mut fact =
                        value_fact(state, effects.join(self.effects_for(unary.operand.id)?));
                    fact.operator_id = operator_id;
                    if min_case {
                        fact.literal = Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Int(
                            i128::MIN,
                        )));
                    }
                    (node, fact)
                }
                ValueFrame::Binary(node, binary) => {
                    let mut operator_id = None;
                    let mut effects = ExpressionEffectsV2::pure_scalar();
                    let state = self.resolve_binary(
                        node,
                        &binary.op.symbol,
                        self.value_state(&binary.lhs)?,
                        self.value_state(&binary.rhs)?,
                        &mut operator_id,
                        &mut effects,
                    )?;
                    let mut fact = value_fact(
                        state,
                        effects.compose([
                            self.effects_for(binary.lhs.id)?,
                            self.effects_for(binary.rhs.id)?,
                        ]),
                    );
                    fact.operator_id = operator_id;
                    (node, fact)
                }
                ValueFrame::Path(node, path) => {
                    let mut current = self.value_state(&path.base)?;
                    for segment in path.segments.as_slice() {
                        current = match current {
                            ExpressionTypeStateV2::Unknown(cause) => {
                                ExpressionTypeStateV2::Unknown(cause)
                            }
                            ExpressionTypeStateV2::Known(base) => {
                                let bad_projection = || TypeDiagnosticV2::BadProjection {
                                    node_id: node.id,
                                    base: base.clone(),
                                    field: segment.name.clone(),
                                    location: LocatedSpanV2 {
                                        source_id: self.key(node).source_id,
                                        span: segment.span,
                                    },
                                };
                                let (record, wrap_optional) = if let Some(record) = base.as_record()
                                {
                                    (record, false)
                                } else if let Some(optional) = base.as_optional() {
                                    (
                                        optional.inner().as_record().ok_or_else(bad_projection)?,
                                        true,
                                    )
                                } else {
                                    return Err(bad_projection());
                                };
                                let field =
                                    record.field(&segment.name).ok_or_else(bad_projection)?;
                                let projected = if wrap_optional
                                    && field.value_type().as_optional().is_none()
                                {
                                    ValueTypeV1::try_optional(field.value_type().clone())?
                                } else {
                                    field.value_type().clone()
                                };
                                ExpressionTypeStateV2::Known(projected)
                            }
                        };
                    }
                    (node, value_fact(current, self.effects_for(path.base.id)?))
                }
                ValueFrame::Call {
                    node,
                    call,
                    input,
                    contexts,
                    next,
                    mut types,
                } => {
                    let args = call.args().as_slice();
                    if let Some(CallArgHirV1::Expr(previous)) =
                        next.checked_sub(1).and_then(|index| args.get(index))
                    {
                        types.push(self.value_state(previous)?);
                    }
                    if let Some(argument) = args.get(next) {
                        if let CallArgHirV1::Expr(expr) = argument {
                            if let Some(Some(expected)) = contexts.get(next) {
                                self.callable_contexts
                                    .insert(self.key(expr), expected.clone());
                            }
                        }
                        stack.push(ValueFrame::Call {
                            node,
                            call,
                            input,
                            contexts,
                            next: next + 1,
                            types,
                        });
                        if let CallArgHirV1::Expr(expr) = argument {
                            stack.push(ValueFrame::Enter(expr, false));
                        }
                        continue;
                    }
                    let (state, call_effects, target) =
                        self.resolve_call(node, input, args, &types)?;
                    let effects = call_effects.compose(args.iter().filter_map(|arg| match arg {
                        CallArgHirV1::Expr(expr) => {
                            self.facts.get(&self.key(expr)).map(|fact| fact.effects)
                        }
                        CallArgHirV1::Star { .. } => None,
                    }));
                    let mut fact = value_fact(state, effects);
                    fact.intrinsic_call_aggregation = Some(call_effects.aggregation);
                    fact.resolved_call = target;
                    (node, fact)
                }
                ValueFrame::Record {
                    node,
                    record,
                    next,
                    mut fields,
                    mut unknown,
                    mut effects,
                } => {
                    if let Some(field) = next
                        .checked_sub(1)
                        .and_then(|index| record.fields.get(index))
                    {
                        match self.value_state(&field.value)? {
                            ExpressionTypeStateV2::Known(ty) => {
                                let location = LocatedSpanV2 {
                                    source_id: self.key(node).source_id,
                                    span: field.name_span,
                                };
                                let checked = self
                                    .inputs
                                    .source_registry
                                    .try_check_location(location)
                                    .map_err(|_source_error| TypeDiagnosticV2::InvalidTypeName {
                                        location,
                                    })?;
                                fields.push(RecordFieldTypeV1::try_new(&field.name, ty, checked)?);
                            }
                            ExpressionTypeStateV2::Unknown(cause) => unknown = Some(cause),
                        }
                        effects = effects.join(self.effects_for(field.value.id)?);
                    }
                    if let Some(field) = record.fields.get(next) {
                        stack.push(ValueFrame::Record {
                            node,
                            record,
                            next: next + 1,
                            fields,
                            unknown,
                            effects,
                        });
                        stack.push(ValueFrame::Enter(&field.value, false));
                        continue;
                    }
                    let state = match unknown {
                        Some(cause) => ExpressionTypeStateV2::Unknown(cause),
                        None => ExpressionTypeStateV2::Known(ValueTypeV1::try_record(
                            RecordTypeV1::try_new(fields)?,
                        )?),
                    };
                    (node, value_fact(state, effects))
                }
                ValueFrame::List {
                    node,
                    list,
                    next,
                    mut selected,
                    mut unknown,
                    mut effects,
                } => {
                    if let Some(value) = next
                        .checked_sub(1)
                        .and_then(|index| list.elements.get(index))
                    {
                        match self.value_state(value)? {
                            ExpressionTypeStateV2::Unknown(cause) => unknown = Some(cause),
                            ExpressionTypeStateV2::Known(ty) => match &selected {
                                None => selected = Some(ty),
                                Some(expected) if ty.is_assignable_to(expected) => {}
                                Some(expected) if expected.is_assignable_to(&ty) => {
                                    selected = Some(ty)
                                }
                                Some(expected) => {
                                    return Err(TypeDiagnosticV2::HeterogeneousCollection {
                                        node_id: node.id,
                                        expected: expected.clone(),
                                        actual: ty,
                                        location: self.location(node),
                                    })
                                }
                            },
                        }
                        effects = effects.join(self.effects_for(value.id)?);
                    }
                    if let Some(value) = list.elements.get(next) {
                        stack.push(ValueFrame::List {
                            node,
                            list,
                            next: next + 1,
                            selected,
                            unknown,
                            effects,
                        });
                        stack.push(ValueFrame::Enter(value, false));
                        continue;
                    }
                    let key = self.key(node);
                    let state = match (unknown, selected) {
                        (Some(cause), _) => ExpressionTypeStateV2::Unknown(cause),
                        (None, Some(element)) => {
                            ExpressionTypeStateV2::Known(ValueTypeV1::try_list(element)?)
                        }
                        (None, None) => match self.inputs.contexts.get(&key) {
                            Some((expected, _))
                                if expected.as_collection().is_some_and(|collection| {
                                    collection.cardinality() == CardinalityV1::Sequence
                                }) =>
                            {
                                self.used_contexts.insert(key);
                                ExpressionTypeStateV2::Known(expected.clone())
                            }
                            _ => match self.callable_contexts.get(&key) {
                                Some(expected)
                                    if expected.as_collection().is_some_and(|collection| {
                                        collection.cardinality() == CardinalityV1::Sequence
                                    }) =>
                                {
                                    ExpressionTypeStateV2::Known(expected.clone())
                                }
                                _ => ExpressionTypeStateV2::Unknown(
                                    UnknownTypeCauseV2::EmptyCollectionElement,
                                ),
                            },
                        },
                    };
                    (node, value_fact(state, effects))
                }
            };
            active.remove(&node.id);
            self.facts.insert(self.key(node), fact);
        }
        self.value_state(root)
    }
}
