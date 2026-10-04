use crate::builtin::{
    AggregationClassV1, BuiltinFamilyIdV2, BuiltinIdV2, BuiltinRegistryV2, CallableSignatureV2,
    ExpressionEffectsV2,
};
use crate::op::{OperatorIdV2, OperatorRegistryV2};
use crate::ty::{FiniteF64V2, RecordTypeBuilderV1, RecordTypeV1, TypedLiteralValueV2, ValueTypeV1};
use crate::TypeDiagnosticV2;
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CallArgHirV1, CheckedHirDocumentV2, CheckedLocatedSpanV2,
    ExtendedSourceRegistryViewV2, HirNodeIdV1, HirVisitorV1, LiteralV1, LocatedSpanV2,
    RelationExprHirV1, RelationExprKindV1, SourceIdV2, ValueExprHirV1, ValueExprKindV1,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

mod scalar_request;
mod value;
pub use scalar_request::{
    check_one_let_composition_with_request_owner_v3, check_scalar_projection_with_request_owner_v3,
    one_let_composition_type_storage_upper_v3, scalar_projection_type_storage_upper_v3,
    AdmittedScalarProjectionFactsV3, ScalarProjectionCheckRequestErrorV3, TypeFactsReaderV3,
};

mod inputs;
use inputs::ResolvedRelationPredicateKindV2;

pub use inputs::{
    ContextualExpectedTypeInputV2, ContextualTypeReasonV2, DeferredRelationRowFieldInputV2,
    DeferredRelationRowInputV2, DeferredRelationRowKindV2, ExpressionTypeStateV2,
    HostCallableNamespaceV2, RelationPreEvaluationInputV2, RelationRowFieldSourceV2,
    ResolvedCallTargetV2, ResolvedCallableInputV2, ResolvedHostPredicateCallableV2,
    ResolvedHostPredicateKeyV2, ResolvedPredicateCallableV2, ResolvedPredicateIdV2,
    ResolvedRelationCaptureInputV2, ResolvedRelationPredicateInputV2, ResolvedTypeInputEntryV2,
    ResolvedTypeInputValueV2, ResolvedTypeInputsV2, SourceQualifiedNodeIdV2, UnknownTypeCauseV2,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedLiteralFactV2 {
    Value(TypedLiteralValueV2),
    DeferredI128MinMagnitude,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedExpressionFactV2 {
    pub state: ExpressionTypeStateV2,
    pub operator_id: Option<OperatorIdV2>,
    resolved_call: Option<ResolvedCallTargetV2>,
    intrinsic_call_aggregation: Option<AggregationClassV1>,
    pub effects: ExpressionEffectsV2,
    pub literal: Option<TypedLiteralFactV2>,
}

#[derive(Clone, Debug)]
pub struct TypedExpressionFactsV2 {
    facts: BTreeMap<SourceQualifiedNodeIdV2, TypedExpressionFactV2>,
    relation_rows: BTreeMap<SourceQualifiedNodeIdV2, RecordTypeV1>,
    request_facts: Option<Vec<(SourceQualifiedNodeIdV2, TypedExpressionFactV2)>>,
    request_rows: Option<Vec<(SourceQualifiedNodeIdV2, RecordTypeV1)>>,
}
impl PartialEq for TypedExpressionFactsV2 {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len()
            && self.relation_row_count() == other.relation_row_count()
            && self.request_facts.as_ref().map_or_else(
                || {
                    self.facts
                        .iter()
                        .all(|(key, fact)| other.get(*key) == Some(fact))
                },
                |facts| {
                    facts
                        .iter()
                        .all(|(key, fact)| other.get(*key) == Some(fact))
                },
            )
            && self.request_rows.as_ref().map_or_else(
                || {
                    self.relation_rows
                        .iter()
                        .all(|(key, row)| other.relation_row(*key) == Some(row))
                },
                |rows| {
                    rows.iter()
                        .all(|(key, row)| other.relation_row(*key) == Some(row))
                },
            )
    }
}
impl Eq for TypedExpressionFactsV2 {}
impl TypedExpressionFactsV2 {
    #[must_use]
    pub fn get(&self, key: SourceQualifiedNodeIdV2) -> Option<&TypedExpressionFactV2> {
        self.request_facts.as_ref().map_or_else(
            || self.facts.get(&key),
            |facts| {
                facts
                    .binary_search_by_key(&key, |entry| entry.0)
                    .ok()
                    .and_then(|i| facts.get(i).map(|entry| &entry.1))
            },
        )
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.request_facts
            .as_ref()
            .map_or(self.facts.len(), Vec::len)
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Types-owned row for one exact checked relation stage.
    #[must_use]
    pub fn relation_row(&self, root: SourceQualifiedNodeIdV2) -> Option<&RecordTypeV1> {
        self.request_rows.as_ref().map_or_else(
            || self.relation_rows.get(&root),
            |rows| {
                rows.binary_search_by_key(&root, |entry| entry.0)
                    .ok()
                    .and_then(|i| rows.get(i).map(|entry| &entry.1))
            },
        )
    }
    #[must_use]
    pub fn relation_row_count(&self) -> usize {
        self.request_rows
            .as_ref()
            .map_or(self.relation_rows.len(), Vec::len)
    }
}
impl TypedExpressionFactV2 {
    #[must_use]
    pub fn resolved_call(&self) -> Option<&ResolvedCallTargetV2> {
        self.resolved_call.as_ref()
    }

    /// Aggregation of this call itself, before composing effects of its arguments.
    #[must_use]
    pub const fn intrinsic_call_aggregation(&self) -> Option<AggregationClassV1> {
        self.intrinsic_call_aggregation
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionTypeKernelV2 {
    operators: Option<OperatorRegistryV2>,
    builtins: KernelBuiltinsV2,
}

#[derive(Clone, Debug)]
enum KernelBuiltinsV2 {
    Owned(BuiltinRegistryV2),
    Shared(Arc<crate::builtin::AdmittedBuiltinRegistryInnerV3>),
}

impl std::ops::Deref for KernelBuiltinsV2 {
    type Target = BuiltinRegistryV2;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(value) => value,
            Self::Shared(value) => &value.registry,
        }
    }
}
impl ExpressionTypeKernelV2 {
    pub fn try_new(
        operators: OperatorRegistryV2,
        builtins: BuiltinRegistryV2,
    ) -> Result<Self, TypeDiagnosticV2> {
        Ok(Self {
            operators: Some(operators),
            builtins: KernelBuiltinsV2::Owned(builtins),
        })
    }
    pub fn try_closed() -> Result<Self, TypeDiagnosticV2> {
        Ok(Self {
            operators: Some(OperatorRegistryV2::try_closed()?),
            builtins: KernelBuiltinsV2::Owned(BuiltinRegistryV2::try_closed()?),
        })
    }

    /// Share the issued builtin inventory and its permit without allocation.
    pub(crate) fn for_admitted_scalar_projection_v3(
        builtins: &crate::AdmittedBuiltinRegistryV3,
    ) -> Self {
        Self {
            operators: None,
            builtins: KernelBuiltinsV2::Shared(builtins.shared_inner()),
        }
    }

    pub fn check_document(
        &self,
        inputs: &ResolvedTypeInputsV2<'_>,
    ) -> Result<TypedExpressionFactsV2, TypeDiagnosticV2> {
        let mut checker = DocumentChecker {
            kernel: self,
            inputs,
            facts: BTreeMap::new(),
            relation_rows: BTreeMap::new(),
            used_contexts: BTreeSet::new(),
            callable_contexts: BTreeMap::new(),
            error: None,
        };
        for root in &inputs.pre_evaluation_roots {
            checker.visit_relation(root);
            if let Some(error) = checker.error.take() {
                return Err(error);
            }
        }
        for item in &inputs.checked.document().items {
            checker.visit_item(item);
            if let Some(error) = checker.error.take() {
                return Err(error);
            }
        }
        if let Some(key) = inputs
            .contexts
            .keys()
            .find(|key| !checker.used_contexts.contains(key))
        {
            let span = inputs
                .context_locations
                .get(key)
                .copied()
                .unwrap_or_else(|| quanta_gqlang_v2_hir::SpanV1::point(0));
            return Err(TypeDiagnosticV2::UnusedContextualInput {
                node_id: key.node_id,
                location: LocatedSpanV2 {
                    source_id: key.source_id,
                    span,
                },
            });
        }
        Ok(TypedExpressionFactsV2 {
            facts: checker.facts,
            relation_rows: checker
                .relation_rows
                .into_iter()
                .map(|(key, row)| (key, row.record))
                .collect(),
            request_facts: None,
            request_rows: None,
        })
    }
}

#[derive(Clone)]
struct CheckedRelationRowV2 {
    record: RecordTypeV1,
    locations: Vec<CheckedLocatedSpanV2>,
}

struct DocumentChecker<'a, 'b> {
    kernel: &'a ExpressionTypeKernelV2,
    inputs: &'a ResolvedTypeInputsV2<'b>,
    facts: BTreeMap<SourceQualifiedNodeIdV2, TypedExpressionFactV2>,
    relation_rows: BTreeMap<SourceQualifiedNodeIdV2, CheckedRelationRowV2>,
    used_contexts: BTreeSet<SourceQualifiedNodeIdV2>,
    callable_contexts: BTreeMap<SourceQualifiedNodeIdV2, ValueTypeV1>,
    error: Option<TypeDiagnosticV2>,
}

impl HirVisitorV1 for DocumentChecker<'_, '_> {
    fn visit_relation(&mut self, node: &RelationExprHirV1) {
        if self.error.is_some() {
            return;
        }
        quanta_gqlang_v2_hir::walk_relation(self, node);
        if self.error.is_none() {
            if let Err(error) = self.complete_relation_row(node) {
                self.error = Some(error.at_location(LocatedSpanV2 {
                    source_id: self.inputs.checked.document().source_id,
                    span: node.span,
                }));
            }
        }
    }
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        if self.error.is_none() {
            if let Err(error) = self.type_expr(node, false) {
                self.error = Some(error.at_location(self.location(node)));
            }
        }
    }
    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        if self.error.is_none() {
            if let Err(error) = self.type_bool(node) {
                self.error = Some(error.at_location(self.bool_location(node)));
            }
        }
    }
}

impl DocumentChecker<'_, '_> {
    fn complete_relation_row(&mut self, node: &RelationExprHirV1) -> Result<(), TypeDiagnosticV2> {
        let key = self.key_id(node.id);
        if self.relation_rows.contains_key(&key) {
            return Ok(());
        }
        let location = LocatedSpanV2 {
            source_id: key.source_id,
            span: node.span,
        };
        let row = match &node.kind {
            RelationExprKindV1::Source(_) => {
                let input = self.inputs.relation_rows.get(&key).ok_or(
                    TypeDiagnosticV2::InvalidDeferredRelationRow {
                        root: node.id,
                        location,
                    },
                )?;
                match &input.kind {
                    DeferredRelationRowKindV2::KnownSource { fields, .. } => {
                        self.build_relation_row(node.id, fields)?
                    }
                    DeferredRelationRowKindV2::PriorRelation { target } => {
                        self.relation_rows.get(target).cloned().ok_or(
                            TypeDiagnosticV2::UnavailableRelationRowDependency {
                                root: node.id,
                                target: target.node_id,
                                location,
                            },
                        )?
                    }
                    DeferredRelationRowKindV2::ExpressionFields { .. } => {
                        return Err(TypeDiagnosticV2::InvalidDeferredRelationRow {
                            root: node.id,
                            location,
                        });
                    }
                }
            }
            RelationExprKindV1::Project(_) | RelationExprKindV1::GroupBy(_) => {
                let input = self.inputs.relation_rows.get(&key).ok_or(
                    TypeDiagnosticV2::InvalidDeferredRelationRow {
                        root: node.id,
                        location,
                    },
                )?;
                let DeferredRelationRowKindV2::ExpressionFields { fields, .. } = &input.kind else {
                    return Err(TypeDiagnosticV2::InvalidDeferredRelationRow {
                        root: node.id,
                        location,
                    });
                };
                self.build_relation_row(node.id, fields)?
            }
            RelationExprKindV1::Filter(filter) => self
                .relation_rows
                .get(&self.key_id(filter.input.id))
                .cloned()
                .ok_or(TypeDiagnosticV2::UnavailableRelationRowDependency {
                    root: node.id,
                    target: filter.input.id,
                    location,
                })?,
            RelationExprKindV1::OrderBy(order) => self
                .relation_rows
                .get(&self.key_id(order.input.id))
                .cloned()
                .ok_or(TypeDiagnosticV2::UnavailableRelationRowDependency {
                    root: node.id,
                    target: order.input.id,
                    location,
                })?,
            RelationExprKindV1::Join(join) => {
                let (left, right) = match join {
                    quanta_gqlang_v2_hir::JoinHirV1::Inner { left, right, .. }
                    | quanta_gqlang_v2_hir::JoinHirV1::Cross { left, right } => (left.id, right.id),
                };
                let left_row = self.relation_rows.get(&self.key_id(left)).ok_or(
                    TypeDiagnosticV2::UnavailableRelationRowDependency {
                        root: node.id,
                        target: left,
                        location,
                    },
                )?;
                let right_row = self.relation_rows.get(&self.key_id(right)).ok_or(
                    TypeDiagnosticV2::UnavailableRelationRowDependency {
                        root: node.id,
                        target: right,
                        location,
                    },
                )?;
                let mut fields = RecordTypeBuilderV1::new();
                for (field, field_location) in left_row
                    .record
                    .fields()
                    .iter()
                    .zip(left_row.locations.iter())
                    .chain(
                        right_row
                            .record
                            .fields()
                            .iter()
                            .zip(right_row.locations.iter()),
                    )
                {
                    fields.try_push(field.name(), field.value_type(), field_location.clone())?;
                }
                let mut locations = left_row.locations.clone();
                locations.extend(right_row.locations.iter().cloned());
                CheckedRelationRowV2 {
                    record: fields.finish()?,
                    locations,
                }
            }
        };
        let _ = self.relation_rows.insert(key, row);
        Ok(())
    }

    fn build_relation_row(
        &self,
        root: HirNodeIdV1,
        fields: &[DeferredRelationRowFieldInputV2],
    ) -> Result<CheckedRelationRowV2, TypeDiagnosticV2> {
        let mut locations = Vec::with_capacity(fields.len());
        let mut typed_fields = RecordTypeBuilderV1::new();
        for field in fields {
            let value_type = match &field.source {
                RelationRowFieldSourceV2::Known(value_type) => value_type,
                RelationRowFieldSourceV2::PriorExpression(target) => {
                    match self.facts.get(target).map(|fact| &fact.state) {
                        Some(ExpressionTypeStateV2::Known(value_type)) => value_type,
                        _ => {
                            return Err(TypeDiagnosticV2::UnavailableRelationRowDependency {
                                root,
                                target: target.node_id,
                                location: field.name_location.location(),
                            });
                        }
                    }
                }
            };
            locations.push(field.name_location.clone());
            typed_fields.try_push(&field.name, value_type, field.name_location.clone())?;
        }
        Ok(CheckedRelationRowV2 {
            record: typed_fields.finish()?,
            locations,
        })
    }
    fn location(&self, node: &ValueExprHirV1) -> LocatedSpanV2 {
        LocatedSpanV2 {
            source_id: self.inputs.checked.document().source_id,
            span: node.span,
        }
    }
    fn key(&self, node: &ValueExprHirV1) -> SourceQualifiedNodeIdV2 {
        SourceQualifiedNodeIdV2 {
            source_id: self.inputs.checked.document().source_id,
            node_id: node.id,
        }
    }
    fn key_id(&self, node_id: HirNodeIdV1) -> SourceQualifiedNodeIdV2 {
        SourceQualifiedNodeIdV2 {
            source_id: self.inputs.checked.document().source_id,
            node_id,
        }
    }
    fn bool_location(&self, node: &BoolExprHirV1) -> LocatedSpanV2 {
        LocatedSpanV2 {
            source_id: self.inputs.checked.document().source_id,
            span: node.span,
        }
    }

    fn type_bool(
        &mut self,
        node: &BoolExprHirV1,
    ) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let key = self.key_id(node.id);
        if let Some(fact) = self.facts.get(&key) {
            return Ok(fact.state.clone());
        }
        let mut operator_id = None;
        let mut literal = None;
        let mut effects = ExpressionEffectsV2::pure_scalar();
        let state = match &node.kind {
            BoolExprKindV1::Literal(value) => {
                literal = Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Bool(*value)));
                ExpressionTypeStateV2::Known(ValueTypeV1::bool())
            }
            BoolExprKindV1::Value(value) => {
                let state = self.type_expr(value, false)?;
                effects = self.effects_for(value.id)?;
                match state {
                    ExpressionTypeStateV2::Known(actual) if actual == ValueTypeV1::bool() => {
                        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
                    }
                    ExpressionTypeStateV2::Known(actual) => {
                        return Err(TypeDiagnosticV2::BadOverload {
                            node_id: node.id,
                            name: "bool-context".into(),
                            operands: vec![actual],
                            location: self.bool_location(node),
                        })
                    }
                    unknown @ ExpressionTypeStateV2::Unknown(_) => unknown,
                }
            }
            BoolExprKindV1::Compare(compare) => {
                let lhs = self.type_expr(&compare.lhs, false)?;
                let rhs = self.type_expr(&compare.rhs, false)?;
                let child_effects = [
                    self.effects_for(compare.lhs.id)?,
                    self.effects_for(compare.rhs.id)?,
                ];
                let state = self.resolve_binary_bool(
                    node,
                    &compare.op.symbol,
                    lhs,
                    rhs,
                    &mut operator_id,
                    &mut effects,
                )?;
                effects = effects.compose(child_effects);
                state
            }
            BoolExprKindV1::And(binary) => {
                let lhs = self.type_bool(&binary.lhs)?;
                let rhs = self.type_bool(&binary.rhs)?;
                let child_effects = [
                    self.effects_for(binary.lhs.id)?,
                    self.effects_for(binary.rhs.id)?,
                ];
                let state = self.resolve_binary_bool(
                    node,
                    "and",
                    lhs,
                    rhs,
                    &mut operator_id,
                    &mut effects,
                )?;
                effects = effects.compose(child_effects);
                state
            }
            BoolExprKindV1::Or(binary) => {
                let lhs = self.type_bool(&binary.lhs)?;
                let rhs = self.type_bool(&binary.rhs)?;
                let child_effects = [
                    self.effects_for(binary.lhs.id)?,
                    self.effects_for(binary.rhs.id)?,
                ];
                let state =
                    self.resolve_binary_bool(node, "or", lhs, rhs, &mut operator_id, &mut effects)?;
                effects = effects.compose(child_effects);
                state
            }
            BoolExprKindV1::Not(inner) => {
                let operand = self.type_bool(inner)?;
                let state =
                    self.resolve_unary_bool(node, "not", operand, &mut operator_id, &mut effects)?;
                effects = effects.join(self.effects_for(inner.id)?);
                state
            }
            BoolExprKindV1::Paren(inner) => {
                let state = self.type_bool(inner)?;
                effects = self.effects_for(inner.id)?;
                state
            }
            BoolExprKindV1::LogicalQuantifier(quantifier) => {
                let body = self.type_bool(&quantifier.body)?;
                effects = self.effects_for(quantifier.body.id)?;
                match body {
                    ExpressionTypeStateV2::Known(_) => {
                        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
                    }
                    unknown @ ExpressionTypeStateV2::Unknown(_) => unknown,
                }
            }
            BoolExprKindV1::RelationExists(exists) => {
                let before = self.facts.keys().copied().collect::<BTreeSet<_>>();
                self.visit_relation(&exists.relation);
                if let Some(error) = self.error.take() {
                    return Err(error);
                }
                let child_effects = self
                    .facts
                    .iter()
                    .filter(|(child_key, _)| !before.contains(child_key))
                    .map(|(_, fact)| fact.effects)
                    .collect::<Vec<_>>();
                match self.inputs.values.get(&key) {
                    Some(ResolvedTypeInputValueV2::RelationPredicate(input)) => match &input.kind {
                        ResolvedRelationPredicateKindV2::Resolved { .. } => {
                            // A resolved EXISTS proves syntax/root/capture
                            // identity, not a relation implementation's
                            // effects. Types owns this conservative base so a
                            // consumer cannot forge a pure/cheap predicate.
                            effects = ExpressionEffectsV2::conservative_source_predicate()
                                .compose(child_effects);
                            ExpressionTypeStateV2::Known(ValueTypeV1::bool())
                        }
                        ResolvedRelationPredicateKindV2::Unknown(cause) => {
                            effects =
                                ExpressionEffectsV2::conservative_unknown().compose(child_effects);
                            ExpressionTypeStateV2::Unknown(cause.clone())
                        }
                    },
                    _ => {
                        return Err(TypeDiagnosticV2::MissingNodeFact {
                            node_id: node.id,
                            location: self.bool_location(node),
                        })
                    }
                }
            }
        };
        let _ = self.facts.insert(
            key,
            TypedExpressionFactV2 {
                state: state.clone(),
                operator_id,
                resolved_call: None,
                intrinsic_call_aggregation: None,
                effects,
                literal,
            },
        );
        Ok(state)
    }
    fn callable_argument_contexts(
        &self,
        input: &ResolvedTypeInputValueV2,
        args: &[CallArgHirV1],
    ) -> Vec<Option<ValueTypeV1>> {
        if args
            .iter()
            .any(|arg| matches!(arg, CallArgHirV1::Star { .. }))
        {
            return vec![None; args.len()];
        }
        match input {
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Predicate(predicate)) => {
                if predicate.params.len() == args.len() {
                    predicate.params.iter().cloned().map(Some).collect()
                } else {
                    vec![None; args.len()]
                }
            }
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::HostPredicate(
                predicate,
            )) => {
                if predicate.params.len() == args.len() {
                    predicate.params.iter().cloned().map(Some).collect()
                } else {
                    vec![None; args.len()]
                }
            }
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(family)) => self
                .kernel
                .builtins
                .agreed_fixed_parameters(*family, args.len()),
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Unknown(_))
            | ResolvedTypeInputValueV2::Binding(_)
            | ResolvedTypeInputValueV2::BindingFromExpression(_)
            | ResolvedTypeInputValueV2::BindingFromDeclaration(_)
            | ResolvedTypeInputValueV2::BindingFromRelationRow(_)
            | ResolvedTypeInputValueV2::RelationPredicate(_) => vec![None; args.len()],
        }
    }

    fn type_literal(
        &self,
        node: &ValueExprHirV1,
        value: &LiteralV1,
        deferred: bool,
    ) -> Result<(ValueTypeV1, TypedLiteralFactV2), TypeDiagnosticV2> {
        let location = self.location(node);
        match value {
            LiteralV1::Int(text)
                if deferred && text == "170141183460469231731687303715884105728" =>
            {
                Ok((
                    ValueTypeV1::int(),
                    TypedLiteralFactV2::DeferredI128MinMagnitude,
                ))
            }
            LiteralV1::Int(text) => {
                if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
                    return Err(TypeDiagnosticV2::MalformedLiteral {
                        node_id: node.id,
                        location,
                    });
                }
                text.parse::<i128>()
                    .map(|value| {
                        (
                            ValueTypeV1::int(),
                            TypedLiteralFactV2::Value(TypedLiteralValueV2::Int(value)),
                        )
                    })
                    .map_err(|_parse_error| TypeDiagnosticV2::IntegerOutOfRange {
                        node_id: node.id,
                        location,
                        text: text.clone(),
                    })
            }
            LiteralV1::Float(text) => {
                let Some((whole, fraction)) = text.split_once('.') else {
                    return Err(TypeDiagnosticV2::MalformedLiteral {
                        node_id: node.id,
                        location,
                    });
                };
                if whole.is_empty()
                    || fraction.is_empty()
                    || text.matches('.').count() != 1
                    || !whole
                        .bytes()
                        .chain(fraction.bytes())
                        .all(|byte| byte.is_ascii_digit())
                {
                    return Err(TypeDiagnosticV2::MalformedLiteral {
                        node_id: node.id,
                        location,
                    });
                }
                let value = text
                    .parse::<f64>()
                    .map_err(|_parse_error| TypeDiagnosticV2::FloatNonFinite { location })?;
                if !value.is_finite() {
                    return Err(TypeDiagnosticV2::FloatNonFinite { location });
                }
                if value == 0.0 && lexical_nonzero(text) {
                    return Err(TypeDiagnosticV2::FloatUnderflow {
                        node_id: node.id,
                        location,
                        text: text.clone(),
                    });
                }
                Ok((
                    ValueTypeV1::float(),
                    TypedLiteralFactV2::Value(TypedLiteralValueV2::Float(FiniteF64V2::try_new(
                        value, location,
                    )?)),
                ))
            }
            LiteralV1::Str(value) => Ok((
                ValueTypeV1::string(),
                TypedLiteralFactV2::Value(TypedLiteralValueV2::String(value.clone())),
            )),
            LiteralV1::Bool(value) => Ok((
                ValueTypeV1::bool(),
                TypedLiteralFactV2::Value(TypedLiteralValueV2::Bool(*value)),
            )),
            LiteralV1::Null => Ok((
                ValueTypeV1::null(),
                TypedLiteralFactV2::Value(TypedLiteralValueV2::Null),
            )),
        }
    }

    fn resolve_unary(
        &self,
        node: &ValueExprHirV1,
        symbol: &str,
        operand: ExpressionTypeStateV2,
        selected: &mut Option<OperatorIdV2>,
        effects: &mut ExpressionEffectsV2,
    ) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        self.resolve_unary_at(
            node.id,
            self.location(node),
            symbol,
            operand,
            selected,
            effects,
        )
    }

    fn resolve_unary_bool(
        &self,
        node: &BoolExprHirV1,
        symbol: &str,
        operand: ExpressionTypeStateV2,
        selected: &mut Option<OperatorIdV2>,
        effects: &mut ExpressionEffectsV2,
    ) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        self.resolve_unary_at(
            node.id,
            self.bool_location(node),
            symbol,
            operand,
            selected,
            effects,
        )
    }

    fn resolve_unary_at(
        &self,
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
        symbol: &str,
        operand: ExpressionTypeStateV2,
        selected: &mut Option<OperatorIdV2>,
        effects: &mut ExpressionEffectsV2,
    ) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let Some(operators) = self.kernel.operators.as_ref() else {
            return Err(TypeDiagnosticV2::UnknownOperator {
                node_id,
                symbol: symbol.into(),
                location,
            });
        };
        let overloads = operators.unary_overloads(symbol);
        if overloads.is_empty() {
            return Err(TypeDiagnosticV2::UnknownOperator {
                node_id,
                symbol: symbol.into(),
                location,
            });
        }
        let candidates = overloads
            .iter()
            .filter(|overload| match &operand {
                ExpressionTypeStateV2::Known(actual) => actual.is_assignable_to(&overload.operand),
                ExpressionTypeStateV2::Unknown(_) => true,
            })
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            let operands = match operand {
                ExpressionTypeStateV2::Known(actual) => vec![actual],
                ExpressionTypeStateV2::Unknown(_) => Vec::new(),
            };
            return Err(TypeDiagnosticV2::BadOverload {
                node_id,
                name: symbol.into(),
                operands,
                location,
            });
        }
        if let [candidate] = candidates.as_slice() {
            let candidate = *candidate;
            *selected = Some(candidate.id);
            *effects = candidate.effects;
            return Ok(ExpressionTypeStateV2::Known(candidate.result.clone()));
        }
        let candidate_effects = candidates
            .iter()
            .map(|entry| entry.effects)
            .reduce(ExpressionEffectsV2::join)
            .unwrap_or_else(ExpressionEffectsV2::pure_scalar);
        if let Some(result) = common_result(candidates.iter().map(|entry| &entry.result)) {
            *effects = candidate_effects;
            return Ok(ExpressionTypeStateV2::Known(result));
        }
        *effects = candidate_effects;
        Ok(operand)
    }

    fn resolve_binary(
        &self,
        node: &ValueExprHirV1,
        symbol: &str,
        lhs: ExpressionTypeStateV2,
        rhs: ExpressionTypeStateV2,
        selected: &mut Option<OperatorIdV2>,
        effects: &mut ExpressionEffectsV2,
    ) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        self.resolve_binary_at(
            node.id,
            self.location(node),
            symbol,
            lhs,
            rhs,
            selected,
            effects,
        )
    }

    fn resolve_binary_bool(
        &self,
        node: &BoolExprHirV1,
        symbol: &str,
        lhs: ExpressionTypeStateV2,
        rhs: ExpressionTypeStateV2,
        selected: &mut Option<OperatorIdV2>,
        effects: &mut ExpressionEffectsV2,
    ) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        self.resolve_binary_at(
            node.id,
            self.bool_location(node),
            symbol,
            lhs,
            rhs,
            selected,
            effects,
        )
    }

    fn resolve_binary_at(
        &self,
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
        symbol: &str,
        lhs: ExpressionTypeStateV2,
        rhs: ExpressionTypeStateV2,
        selected: &mut Option<OperatorIdV2>,
        effects: &mut ExpressionEffectsV2,
    ) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let Some(operators) = self.kernel.operators.as_ref() else {
            return Err(TypeDiagnosticV2::UnknownOperator {
                node_id,
                symbol: symbol.into(),
                location,
            });
        };
        let overloads = operators.binary_overloads(symbol);
        if overloads.is_empty() {
            return Err(TypeDiagnosticV2::UnknownOperator {
                node_id,
                symbol: symbol.into(),
                location,
            });
        }
        let candidates = overloads
            .iter()
            .filter(|overload| {
                state_accepts(&lhs, &overload.lhs) && state_accepts(&rhs, &overload.rhs)
            })
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            let operands = [lhs, rhs]
                .into_iter()
                .filter_map(|state| match state {
                    ExpressionTypeStateV2::Known(actual) => Some(actual),
                    ExpressionTypeStateV2::Unknown(_) => None,
                })
                .collect();
            return Err(TypeDiagnosticV2::BadOverload {
                node_id,
                name: symbol.into(),
                operands,
                location,
            });
        }
        if let [candidate] = candidates.as_slice() {
            let candidate = *candidate;
            *selected = Some(candidate.id);
            *effects = candidate.effects;
            return Ok(ExpressionTypeStateV2::Known(candidate.result.clone()));
        }
        let candidate_effects = candidates
            .iter()
            .map(|entry| entry.effects)
            .reduce(ExpressionEffectsV2::join)
            .unwrap_or_else(ExpressionEffectsV2::pure_scalar);
        if let Some(result) = common_result(candidates.iter().map(|entry| &entry.result)) {
            *effects = candidate_effects;
            return Ok(ExpressionTypeStateV2::Known(result));
        }
        *effects = candidate_effects;
        let cause =
            first_unknown_cause(&lhs, &rhs).unwrap_or(UnknownTypeCauseV2::UnresolvedEditorBinding);
        Ok(ExpressionTypeStateV2::Unknown(cause))
    }
    fn resolve_predicate_signature(
        &self,
        node: &ValueExprHirV1,
        any_star: bool,
        arg_types: &[ExpressionTypeStateV2],
        params: &[ValueTypeV1],
        result: ValueTypeV1,
        effects: ExpressionEffectsV2,
        target: ResolvedCallTargetV2,
        diagnostic_name: &'static str,
    ) -> Result<
        (
            ExpressionTypeStateV2,
            ExpressionEffectsV2,
            Option<ResolvedCallTargetV2>,
        ),
        TypeDiagnosticV2,
    > {
        if any_star {
            return Err(TypeDiagnosticV2::InvalidStarArgument {
                node_id: node.id,
                location: self.location(node),
            });
        }
        if params.len() == arg_types.len()
            && params
                .iter()
                .zip(arg_types)
                .all(|(expected, actual)| state_accepts(actual, expected))
        {
            Ok((ExpressionTypeStateV2::Known(result), effects, Some(target)))
        } else {
            Err(TypeDiagnosticV2::BadOverload {
                node_id: node.id,
                name: diagnostic_name.into(),
                operands: known_operands(arg_types),
                location: self.location(node),
            })
        }
    }

    fn resolve_call(
        &self,
        node: &ValueExprHirV1,
        input: ResolvedTypeInputValueV2,
        args: &[CallArgHirV1],
        arg_types: &[ExpressionTypeStateV2],
    ) -> Result<
        (
            ExpressionTypeStateV2,
            ExpressionEffectsV2,
            Option<ResolvedCallTargetV2>,
        ),
        TypeDiagnosticV2,
    > {
        let any_star = args
            .iter()
            .any(|arg| matches!(arg, CallArgHirV1::Star { .. }));
        let exact_star = matches!(args, [CallArgHirV1::Star { .. }]);
        if any_star && !exact_star {
            return Err(TypeDiagnosticV2::InvalidStarArgument {
                node_id: node.id,
                location: self.location(node),
            });
        }
        match input {
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(family)) => {
                let family_entries = self
                    .kernel
                    .builtins
                    .lookup_family(family)
                    .collect::<Vec<_>>();
                if family_entries.is_empty() {
                    return Err(TypeDiagnosticV2::UnknownBuiltin {
                        node_id: node.id,
                        name: format!("builtin-family-{}", family.raw()),
                        location: self.location(node),
                    });
                }
                let candidates = family_entries
                    .into_iter()
                    .filter(|entry| match &entry.signature {
                        CallableSignatureV2::AggregateStar { .. } => exact_star,
                        CallableSignatureV2::AnySingle { .. } => !any_star && arg_types.len() == 1,
                        CallableSignatureV2::Fixed { params, .. } => {
                            !any_star
                                && params.len() == arg_types.len()
                                && params
                                    .iter()
                                    .zip(arg_types)
                                    .all(|(expected, actual)| state_accepts(actual, expected))
                        }
                    })
                    .collect::<Vec<_>>();
                if candidates.is_empty() {
                    if any_star {
                        return Err(TypeDiagnosticV2::InvalidStarArgument {
                            node_id: node.id,
                            location: self.location(node),
                        });
                    }
                    return Err(TypeDiagnosticV2::BadOverload {
                        node_id: node.id,
                        name: format!("builtin-family-{}", family.raw()),
                        operands: known_operands(arg_types),
                        location: self.location(node),
                    });
                }
                if let [candidate] = candidates.as_slice() {
                    let candidate = *candidate;
                    return Ok((
                        ExpressionTypeStateV2::Known(candidate.signature.result().clone()),
                        candidate.effects,
                        Some(ResolvedCallTargetV2::Builtin(candidate.id)),
                    ));
                }
                let candidate_effects = candidates
                    .iter()
                    .map(|entry| entry.effects)
                    .reduce(ExpressionEffectsV2::join)
                    .unwrap_or_else(ExpressionEffectsV2::pure_scalar);
                if let Some(result) =
                    common_result(candidates.iter().map(|entry| entry.signature.result()))
                {
                    return Ok((
                        ExpressionTypeStateV2::Known(result),
                        candidate_effects,
                        None,
                    ));
                }
                let cause = arg_types
                    .iter()
                    .find_map(|state| match state {
                        ExpressionTypeStateV2::Unknown(cause) => Some(cause.clone()),
                        ExpressionTypeStateV2::Known(_) => None,
                    })
                    .unwrap_or(UnknownTypeCauseV2::UnresolvedEditorBinding);
                Ok((
                    ExpressionTypeStateV2::Unknown(cause),
                    candidate_effects,
                    None,
                ))
            }
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Predicate(predicate)) => {
                self.resolve_predicate_signature(
                    node,
                    any_star,
                    arg_types,
                    &predicate.params,
                    predicate.result,
                    predicate.effects,
                    ResolvedCallTargetV2::Predicate(predicate.id),
                    "predicate",
                )
            }
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::HostPredicate(
                predicate,
            )) => self.resolve_predicate_signature(
                node,
                any_star,
                arg_types,
                &predicate.params,
                predicate.result,
                predicate.effects,
                ResolvedCallTargetV2::HostPredicate(predicate.key),
                "host predicate",
            ),
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Unknown(cause)) => Ok((
                ExpressionTypeStateV2::Unknown(cause),
                ExpressionEffectsV2::conservative_unknown(),
                None,
            )),
            ResolvedTypeInputValueV2::Binding(_)
            | ResolvedTypeInputValueV2::BindingFromExpression(_)
            | ResolvedTypeInputValueV2::BindingFromDeclaration(_)
            | ResolvedTypeInputValueV2::BindingFromRelationRow(_)
            | ResolvedTypeInputValueV2::RelationPredicate(_) => {
                Err(TypeDiagnosticV2::WrongNodeFactKind {
                    node_id: node.id,
                    location: self.location(node),
                })
            }
        }
    }

    fn effects_for(&self, node_id: HirNodeIdV1) -> Result<ExpressionEffectsV2, TypeDiagnosticV2> {
        self.facts
            .get(&self.key_id(node_id))
            .map(|fact| fact.effects)
            .ok_or(TypeDiagnosticV2::MissingNodeFact {
                node_id,
                location: LocatedSpanV2 {
                    source_id: self.inputs.checked.document().source_id,
                    span: quanta_gqlang_v2_hir::SpanV1::point(0),
                },
            })
    }
}

fn lexical_nonzero(text: &str) -> bool {
    text.bytes()
        .any(|byte| byte.is_ascii_digit() && byte != b'0')
}

fn state_accepts(state: &ExpressionTypeStateV2, expected: &ValueTypeV1) -> bool {
    match state {
        ExpressionTypeStateV2::Known(actual) => actual.is_assignable_to(expected),
        ExpressionTypeStateV2::Unknown(_) => true,
    }
}

fn known_operands(states: &[ExpressionTypeStateV2]) -> Vec<ValueTypeV1> {
    states
        .iter()
        .filter_map(|state| match state {
            ExpressionTypeStateV2::Known(actual) => Some(actual.clone()),
            ExpressionTypeStateV2::Unknown(_) => None,
        })
        .collect()
}

fn common_result<'a>(mut results: impl Iterator<Item = &'a ValueTypeV1>) -> Option<ValueTypeV1> {
    let first = results.next()?.clone();
    results.all(|result| result == &first).then_some(first)
}

fn first_unknown_cause(
    lhs: &ExpressionTypeStateV2,
    rhs: &ExpressionTypeStateV2,
) -> Option<UnknownTypeCauseV2> {
    [lhs, rhs].into_iter().find_map(|state| match state {
        ExpressionTypeStateV2::Unknown(cause) => Some(cause.clone()),
        ExpressionTypeStateV2::Known(_) => None,
    })
}
