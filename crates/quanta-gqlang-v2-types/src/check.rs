use crate::builtin::{
    BuiltinFamilyIdV2, BuiltinIdV2, BuiltinRegistryV2, CallableSignatureV2, ExpressionEffectsV2,
};
use crate::op::{OperatorIdV2, OperatorRegistryV2};
use crate::ty::{FiniteF64V2, RecordFieldTypeV1, RecordTypeV1, TypedLiteralValueV2, TypeV1};
use crate::TypeDiagnosticV2;
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CallArgHirV1, CheckedHirDocumentV2, HirNodeIdV1,
    HirVisitorV1, LocatedSpanV2, LiteralV1, SourceIdV2, ValueExprHirV1, ValueExprKindV1,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceQualifiedNodeIdV2 { pub source_id: SourceIdV2, pub node_id: HirNodeIdV1 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnknownTypeCauseV2 { UnresolvedEditorBinding, UnresolvedEditorCallable, EmptyCollectionNeedsContext }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionTypeStateV2 { Known(TypeV1), Unknown(UnknownTypeCauseV2) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedTypeInputValueV2 {
    Binding(ExpressionTypeStateV2),
    Callable(ResolvedCallableInputV2),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PredicateIdV2(String);
impl PredicateIdV2 {
    pub fn try_new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if value.is_empty() || value.trim() != value { return Err("invalid predicate id"); }
        Ok(Self(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedCallableInputV2 {
    Builtin(BuiltinFamilyIdV2),
    Predicate { id: PredicateIdV2, signature: PredicateCallableSignatureV2, effects: ExpressionEffectsV2 },
    Unknown(UnknownTypeCauseV2),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PredicateCallableSignatureV2 { params: Vec<TypeV1>, result: TypeV1 }
impl PredicateCallableSignatureV2 {
    #[must_use] pub fn new(params: Vec<TypeV1>, result: TypeV1) -> Self { Self { params, result } }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedCallTargetV2 { Builtin(BuiltinIdV2), Predicate(PredicateIdV2) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedTypeInputEntryV2 { pub key: SourceQualifiedNodeIdV2, pub value: ResolvedTypeInputValueV2 }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextualTypeReasonV2 { ExpectedArgument, ExpectedBinding, ExpectedResult }
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextualExpectedTypeInputV2 { pub key: SourceQualifiedNodeIdV2, pub expected_type: TypeV1, pub reason: ContextualTypeReasonV2 }

#[derive(Clone, Debug)]
pub struct ResolvedTypeInputsV2<'a> {
    checked: CheckedHirDocumentV2<'a>,
    values: BTreeMap<SourceQualifiedNodeIdV2, ResolvedTypeInputValueV2>,
    contexts: BTreeMap<SourceQualifiedNodeIdV2, TypeV1>,
}

impl<'a> ResolvedTypeInputsV2<'a> {
    pub fn try_new(
        checked: CheckedHirDocumentV2<'a>, entries: Vec<ResolvedTypeInputEntryV2>, contexts: Vec<ContextualExpectedTypeInputV2>,
    ) -> Result<Self, TypeDiagnosticV2> {
        let source_id = checked.document().source_id;
        let mut required = RequiredInputs::default();
        for item in &checked.document().items { required.visit_item(item); }
        let mut values = BTreeMap::new();
        for entry in entries {
            if entry.key.source_id != source_id {
                return Err(TypeDiagnosticV2::ForeignDocumentFact { node_id: entry.key.node_id, location: LocatedSpanV2 { source_id: entry.key.source_id, span: quanta_gqlang_v2_hir::SpanV1::point(0) } });
            }
            let Some((span, expected_kind)) = required.nodes.get(&entry.key.node_id).copied() else {
                return Err(TypeDiagnosticV2::ExtraNodeFact { node_id: entry.key.node_id });
            };
            let kind_matches = matches!((expected_kind, &entry.value),
                (RequiredInputKind::Binding, ResolvedTypeInputValueV2::Binding(_))
                | (RequiredInputKind::Callable, ResolvedTypeInputValueV2::Callable(_)));
            if !kind_matches { return Err(TypeDiagnosticV2::WrongNodeFactKind { node_id: entry.key.node_id, location: LocatedSpanV2 { source_id, span } }); }
            if values.insert(entry.key, entry.value).is_some() {
                return Err(TypeDiagnosticV2::DuplicateNodeFact { node_id: entry.key.node_id, location: LocatedSpanV2 { source_id, span } });
            }
        }
        for (node_id, (span, _)) in &required.nodes {
            let key = SourceQualifiedNodeIdV2 { source_id, node_id: *node_id };
            if !values.contains_key(&key) {
                return Err(TypeDiagnosticV2::MissingNodeFact { node_id: *node_id, location: LocatedSpanV2 { source_id, span: *span } });
            }
        }
        let mut checked_contexts = BTreeMap::new();
        for context in contexts {
            if context.key.source_id != source_id { return Err(TypeDiagnosticV2::ForeignDocumentFact { node_id: context.key.node_id, location: LocatedSpanV2 { source_id: context.key.source_id, span: quanta_gqlang_v2_hir::SpanV1::point(0) } }); }
            if !required.context_nodes.contains(&context.key.node_id) { return Err(TypeDiagnosticV2::ExtraNodeFact { node_id: context.key.node_id }); }
            if checked_contexts.insert(context.key, context.expected_type).is_some() { return Err(TypeDiagnosticV2::DuplicateNodeFact { node_id: context.key.node_id, location: LocatedSpanV2 { source_id, span: quanta_gqlang_v2_hir::SpanV1::point(0) } }); }
        }
        Ok(Self { checked, values, contexts: checked_contexts })
    }
}

#[derive(Default)]
struct RequiredInputs { nodes: BTreeMap<HirNodeIdV1, (quanta_gqlang_v2_hir::SpanV1, RequiredInputKind)>, context_nodes: BTreeSet<HirNodeIdV1> }
#[derive(Clone, Copy)] enum RequiredInputKind { Binding, Callable }
impl HirVisitorV1 for RequiredInputs {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        let kind = match node.kind { ValueExprKindV1::Symbol(_) => Some(RequiredInputKind::Binding), ValueExprKindV1::Call(_) => Some(RequiredInputKind::Callable), ValueExprKindV1::Literal(_) | ValueExprKindV1::Path(_) | ValueExprKindV1::Unary(_) | ValueExprKindV1::Binary(_) | ValueExprKindV1::Paren(_) | ValueExprKindV1::Record(_) | ValueExprKindV1::List(_) => None };
        if let Some(kind) = kind {
            let _ = self.nodes.insert(node.id, (node.span, kind));
        }
        if matches!(&node.kind, ValueExprKindV1::List(list) if list.elements.is_empty()) { let _ = self.context_nodes.insert(node.id); }
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedLiteralFactV2 { Value(TypedLiteralValueV2), DeferredI128MinMagnitude }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedExpressionFactV2 {
    pub state: ExpressionTypeStateV2,
    pub operator_id: Option<OperatorIdV2>,
    resolved_call: Option<ResolvedCallTargetV2>,
    pub effects: ExpressionEffectsV2,
    pub literal: Option<TypedLiteralFactV2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedExpressionFactsV2 { facts: BTreeMap<SourceQualifiedNodeIdV2, TypedExpressionFactV2> }
impl TypedExpressionFactsV2 {
    #[must_use] pub fn get(&self, key: SourceQualifiedNodeIdV2) -> Option<&TypedExpressionFactV2> { self.facts.get(&key) }
    #[must_use] pub fn len(&self) -> usize { self.facts.len() }
    #[must_use] pub fn is_empty(&self) -> bool { self.facts.is_empty() }
}
impl TypedExpressionFactV2 {
    #[must_use] pub fn resolved_call(&self) -> Option<&ResolvedCallTargetV2> { self.resolved_call.as_ref() }
}

#[derive(Clone, Debug)]
pub struct ExpressionTypeKernelV2 { operators: OperatorRegistryV2, builtins: BuiltinRegistryV2 }
impl ExpressionTypeKernelV2 {
    pub fn try_new(operators: OperatorRegistryV2, builtins: BuiltinRegistryV2) -> Result<Self, TypeDiagnosticV2> { Ok(Self { operators, builtins }) }
    pub fn try_closed() -> Result<Self, TypeDiagnosticV2> {
        Ok(Self { operators: OperatorRegistryV2::try_closed()?, builtins: BuiltinRegistryV2::try_closed()? })
    }

    pub fn check_document(&self, inputs: &ResolvedTypeInputsV2<'_>) -> Result<TypedExpressionFactsV2, TypeDiagnosticV2> {
        let mut checker = DocumentChecker { kernel: self, inputs, facts: BTreeMap::new(), visiting: BTreeSet::new(), error: None };
        for item in &inputs.checked.document().items { checker.visit_item(item); if let Some(error) = checker.error.take() { return Err(error); } }
        Ok(TypedExpressionFactsV2 { facts: checker.facts })
    }
}

struct DocumentChecker<'a, 'b> {
    kernel: &'a ExpressionTypeKernelV2,
    inputs: &'a ResolvedTypeInputsV2<'b>,
    facts: BTreeMap<SourceQualifiedNodeIdV2, TypedExpressionFactV2>,
    visiting: BTreeSet<HirNodeIdV1>,
    error: Option<TypeDiagnosticV2>,
}

impl HirVisitorV1 for DocumentChecker<'_, '_> {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        if self.error.is_none() {
            if let Err(error) = self.type_expr(node, false) { self.error = Some(error); }
        }
    }
    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        if self.error.is_none() {
            if let Err(error) = self.type_bool(node) { self.error = Some(error); }
        }
    }
}

impl DocumentChecker<'_, '_> {
    fn location(&self, node: &ValueExprHirV1) -> LocatedSpanV2 { LocatedSpanV2 { source_id: self.inputs.checked.document().source_id, span: node.span } }
    fn key(&self, node: &ValueExprHirV1) -> SourceQualifiedNodeIdV2 { SourceQualifiedNodeIdV2 { source_id: self.inputs.checked.document().source_id, node_id: node.id } }
    fn key_id(&self, node_id: HirNodeIdV1) -> SourceQualifiedNodeIdV2 { SourceQualifiedNodeIdV2 { source_id: self.inputs.checked.document().source_id, node_id } }
    fn bool_location(&self, node: &BoolExprHirV1) -> LocatedSpanV2 { LocatedSpanV2 { source_id: self.inputs.checked.document().source_id, span: node.span } }

    fn type_bool(&mut self, node: &BoolExprHirV1) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let key = self.key_id(node.id);
        if let Some(fact) = self.facts.get(&key) { return Ok(fact.state.clone()); }
        let mut operator_id = None;
        let mut literal = None;
        let state = match &node.kind {
            BoolExprKindV1::Literal(value) => { literal = Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Bool(*value))); ExpressionTypeStateV2::Known(TypeV1::bool()) }
            BoolExprKindV1::Value(value) => {
                let state = self.type_expr(value, false)?;
                match state {
                    ExpressionTypeStateV2::Known(actual) if actual == TypeV1::bool() => ExpressionTypeStateV2::Known(TypeV1::bool()),
                    ExpressionTypeStateV2::Known(actual) => return Err(TypeDiagnosticV2::BadOverload { node_id: node.id, name: "bool-context".into(), operands: vec![actual], location: self.bool_location(node) }),
                    unknown @ ExpressionTypeStateV2::Unknown(_) => unknown,
                }
            }
            BoolExprKindV1::Compare(compare) => {
                let lhs = self.type_expr(&compare.lhs, false)?; let rhs = self.type_expr(&compare.rhs, false)?;
                self.resolve_binary_bool(node, &compare.op.symbol, lhs, rhs, &mut operator_id)?
            }
            BoolExprKindV1::And(binary) => {
                let lhs = self.type_bool(&binary.lhs)?; let rhs = self.type_bool(&binary.rhs)?;
                self.resolve_binary_bool(node, "and", lhs, rhs, &mut operator_id)?
            }
            BoolExprKindV1::Or(binary) => {
                let lhs = self.type_bool(&binary.lhs)?; let rhs = self.type_bool(&binary.rhs)?;
                self.resolve_binary_bool(node, "or", lhs, rhs, &mut operator_id)?
            }
            BoolExprKindV1::Not(inner) => {
                let operand = self.type_bool(inner)?;
                self.resolve_unary_bool(node, "not", operand, &mut operator_id)?
            }
            BoolExprKindV1::Paren(inner) => self.type_bool(inner)?,
            BoolExprKindV1::LogicalQuantifier(quantifier) => {
                let body = self.type_bool(&quantifier.body)?;
                match body { ExpressionTypeStateV2::Known(_) => ExpressionTypeStateV2::Known(TypeV1::bool()), unknown @ ExpressionTypeStateV2::Unknown(_) => unknown }
            }
            BoolExprKindV1::RelationExists(_) => ExpressionTypeStateV2::Known(TypeV1::bool()),
        };
        let _ = self.facts.insert(key, TypedExpressionFactV2 { state: state.clone(), operator_id, resolved_call: None, effects: ExpressionEffectsV2::pure_scalar(), literal });
        Ok(state)
    }
    fn type_expr(&mut self, node: &ValueExprHirV1, deferred_min_child: bool) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let key = self.key(node);
        if let Some(fact) = self.facts.get(&key) { return Ok(fact.state.clone()); }
        let _ = self.visiting.insert(node.id);
        let mut operator_id = None; let mut resolved_call = None; let mut literal = None;
        let mut effects = ExpressionEffectsV2::pure_scalar();
        let state = match &node.kind {
            ValueExprKindV1::Literal(value) => {
                let (ty, fact) = self.type_literal(node, value, deferred_min_child)?;
                literal = Some(fact); ExpressionTypeStateV2::Known(ty)
            }
            ValueExprKindV1::Symbol(_) => match self.inputs.values.get(&key) {
                Some(ResolvedTypeInputValueV2::Binding(state)) => state.clone(),
                _ => return Err(TypeDiagnosticV2::MissingNodeFact { node_id: node.id, location: self.location(node) }),
            },
            ValueExprKindV1::Paren(inner) => self.type_expr(inner, false)?,
            ValueExprKindV1::Unary(unary) => {
                let min_case = unary.op.symbol == "-" && matches!(&unary.operand.kind, ValueExprKindV1::Literal(LiteralV1::Int(text)) if text.trim() == "170141183460469231731687303715884105728");
                let operand = self.type_expr(&unary.operand, min_case)?;
                if min_case {
                    literal = Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Int(i128::MIN)));
                    ExpressionTypeStateV2::Known(TypeV1::int())
                } else { self.resolve_unary(node, &unary.op.symbol, operand, &mut operator_id)? }
            }
            ValueExprKindV1::Binary(binary) => {
                let lhs = self.type_expr(&binary.lhs, false)?; let rhs = self.type_expr(&binary.rhs, false)?;
                self.resolve_binary(node, &binary.op.symbol, lhs, rhs, &mut operator_id)?
            }
            ValueExprKindV1::Call(call) => {
                let input = self.inputs.values.get(&key).ok_or(TypeDiagnosticV2::MissingNodeFact { node_id: node.id, location: self.location(node) })?.clone();
                let args = call.args().as_slice();
                let mut arg_types = Vec::new();
                for arg in args { if let CallArgHirV1::Expr(expr) = arg { arg_types.push(self.type_expr(expr, false)?); } }
                let (state, call_effects, call_target) = self.resolve_call(node, input, args, &arg_types)?;
                effects = call_effects; resolved_call = call_target; state
            }
            ValueExprKindV1::Path(path) => {
                let mut current = self.type_expr(&path.base, false)?;
                for segment in &path.segments {
                    current = match current {
                        ExpressionTypeStateV2::Unknown(cause) => ExpressionTypeStateV2::Unknown(cause),
                        ExpressionTypeStateV2::Known(TypeV1::Record(record)) => record.field(&segment.name).map(|field| ExpressionTypeStateV2::Known(field.ty().clone())).ok_or_else(|| TypeDiagnosticV2::BadProjection { node_id: node.id, base: TypeV1::Record(record), field: segment.name.clone(), location: LocatedSpanV2 { source_id: key.source_id, span: segment.span } })?,
                        ExpressionTypeStateV2::Known(base) => return Err(TypeDiagnosticV2::BadProjection { node_id: node.id, base, field: segment.name.clone(), location: LocatedSpanV2 { source_id: key.source_id, span: segment.span } }),
                    };
                }
                current
            }
            ValueExprKindV1::Record(record) => {
                let mut fields = Vec::new();
                let mut unknown = None;
                for field in &record.fields {
                    match self.type_expr(&field.value, false)? {
                        ExpressionTypeStateV2::Known(ty) => fields.push(RecordFieldTypeV1::try_new(field.name.clone(), ty, LocatedSpanV2 { source_id: key.source_id, span: field.name_span })?),
                        ExpressionTypeStateV2::Unknown(cause) => unknown = Some(cause),
                    }
                }
                match unknown { Some(cause) => ExpressionTypeStateV2::Unknown(cause), None => ExpressionTypeStateV2::Known(TypeV1::record(RecordTypeV1::try_new(fields)?)) }
            }
            ValueExprKindV1::List(list) => {
                let mut selected: Option<TypeV1> = None;
                let mut unknown = None;
                for value in &list.elements {
                    match self.type_expr(value, false)? {
                        ExpressionTypeStateV2::Unknown(cause) => unknown = Some(cause),
                        ExpressionTypeStateV2::Known(ty) => match &selected {
                            None => selected = Some(ty),
                            Some(expected) if ty.is_assignable_to(expected) => {}
                            Some(expected) if expected.is_assignable_to(&ty) => selected = Some(ty),
                            Some(expected) => return Err(TypeDiagnosticV2::HeterogeneousCollection { node_id: node.id, expected: expected.clone(), actual: ty, location: self.location(node) }),
                        },
                    }
                }
                match (unknown, selected) {
                    (Some(cause), _) => ExpressionTypeStateV2::Unknown(cause),
                    (None, Some(element)) => ExpressionTypeStateV2::Known(TypeV1::list(element)),
                    (None, None) => match self.inputs.contexts.get(&key) {
                        Some(TypeV1::Collection { element, cardinality: crate::CardinalityV1::Sequence }) => ExpressionTypeStateV2::Known(TypeV1::list((**element).clone())),
                        _ => ExpressionTypeStateV2::Unknown(UnknownTypeCauseV2::EmptyCollectionNeedsContext),
                    },
                }
            }
        };
        let _ = self.visiting.remove(&node.id);
        let _ = self.facts.insert(key, TypedExpressionFactV2 { state: state.clone(), operator_id, resolved_call, effects, literal });
        Ok(state)
    }

    fn type_literal(&self, node: &ValueExprHirV1, value: &LiteralV1, deferred: bool) -> Result<(TypeV1, TypedLiteralFactV2), TypeDiagnosticV2> {
        let location = self.location(node);
        match value {
            LiteralV1::Int(text) if deferred && text == "170141183460469231731687303715884105728" => Ok((TypeV1::int(), TypedLiteralFactV2::DeferredI128MinMagnitude)),
            LiteralV1::Int(text) => {
                if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) { return Err(TypeDiagnosticV2::MalformedLiteral { node_id: node.id, location }); }
                text.parse::<i128>().map(|value| (TypeV1::int(), TypedLiteralFactV2::Value(TypedLiteralValueV2::Int(value)))).map_err(|_parse_error| TypeDiagnosticV2::IntegerOutOfRange { node_id: node.id, location, text: text.clone() })
            },
            LiteralV1::Float(text) => {
                let Some((whole,fraction)) = text.split_once('.') else { return Err(TypeDiagnosticV2::MalformedLiteral { node_id: node.id, location }); };
                if whole.is_empty() || fraction.is_empty() || text.matches('.').count()!=1 || !whole.bytes().chain(fraction.bytes()).all(|byte| byte.is_ascii_digit()) { return Err(TypeDiagnosticV2::MalformedLiteral { node_id: node.id, location }); }
                let value = text.parse::<f64>().map_err(|_parse_error| TypeDiagnosticV2::FloatNonFinite { location })?;
                if !value.is_finite() { return Err(TypeDiagnosticV2::FloatNonFinite { location }); }
                if value == 0.0 && lexical_nonzero(text) { return Err(TypeDiagnosticV2::FloatUnderflow { node_id: node.id, location, text: text.clone() }); }
                Ok((TypeV1::float(), TypedLiteralFactV2::Value(TypedLiteralValueV2::Float(FiniteF64V2::try_new(value, location)?))))
            }
            LiteralV1::Str(value) => Ok((TypeV1::string(), TypedLiteralFactV2::Value(TypedLiteralValueV2::String(value.clone())))),
            LiteralV1::Bool(value) => Ok((TypeV1::bool(), TypedLiteralFactV2::Value(TypedLiteralValueV2::Bool(*value)))),
            LiteralV1::Null => Ok((TypeV1::Null, TypedLiteralFactV2::Value(TypedLiteralValueV2::Null))),
        }
    }

    fn resolve_unary(&self, node: &ValueExprHirV1, symbol: &str, operand: ExpressionTypeStateV2, selected: &mut Option<OperatorIdV2>) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let ExpressionTypeStateV2::Known(operand) = operand else { return Ok(operand); };
        let overloads=self.kernel.operators.unary_overloads(symbol);if overloads.is_empty(){return Err(TypeDiagnosticV2::UnknownOperator{node_id:node.id,symbol:symbol.into(),location:self.location(node)});}for overload in overloads { if operand.is_assignable_to(&overload.operand) { *selected = Some(overload.id); return Ok(ExpressionTypeStateV2::Known(overload.result.clone())); } }
        Err(TypeDiagnosticV2::BadOverload { node_id: node.id, name: symbol.into(), operands: vec![operand], location: self.location(node) })
    }
    fn resolve_binary(&self, node: &ValueExprHirV1, symbol: &str, lhs: ExpressionTypeStateV2, rhs: ExpressionTypeStateV2, selected: &mut Option<OperatorIdV2>) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let (ExpressionTypeStateV2::Known(lhs), ExpressionTypeStateV2::Known(rhs)) = (lhs, rhs) else { return Ok(ExpressionTypeStateV2::Unknown(UnknownTypeCauseV2::UnresolvedEditorBinding)); };
        let overloads=self.kernel.operators.binary_overloads(symbol);if overloads.is_empty(){return Err(TypeDiagnosticV2::UnknownOperator{node_id:node.id,symbol:symbol.into(),location:self.location(node)});}for overload in overloads { if lhs.is_assignable_to(&overload.lhs) && rhs.is_assignable_to(&overload.rhs) { *selected = Some(overload.id); return Ok(ExpressionTypeStateV2::Known(overload.result.clone())); } }
        Err(TypeDiagnosticV2::BadOverload { node_id: node.id, name: symbol.into(), operands: vec![lhs, rhs], location: self.location(node) })
    }
    fn resolve_binary_bool(&self, node: &BoolExprHirV1, symbol: &str, lhs: ExpressionTypeStateV2, rhs: ExpressionTypeStateV2, selected: &mut Option<OperatorIdV2>) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let (ExpressionTypeStateV2::Known(lhs), ExpressionTypeStateV2::Known(rhs)) = (lhs, rhs) else { return Ok(ExpressionTypeStateV2::Unknown(UnknownTypeCauseV2::UnresolvedEditorBinding)); };
        for overload in self.kernel.operators.binary_overloads(symbol) { if lhs.is_assignable_to(&overload.lhs) && rhs.is_assignable_to(&overload.rhs) { *selected = Some(overload.id); return Ok(ExpressionTypeStateV2::Known(overload.result.clone())); } }
        Err(TypeDiagnosticV2::BadOverload { node_id: node.id, name: symbol.into(), operands: vec![lhs, rhs], location: self.bool_location(node) })
    }
    fn resolve_unary_bool(&self, node: &BoolExprHirV1, symbol: &str, operand: ExpressionTypeStateV2, selected: &mut Option<OperatorIdV2>) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let ExpressionTypeStateV2::Known(operand) = operand else { return Ok(operand); };
        for overload in self.kernel.operators.unary_overloads(symbol) { if operand.is_assignable_to(&overload.operand) { *selected = Some(overload.id); return Ok(ExpressionTypeStateV2::Known(overload.result.clone())); } }
        Err(TypeDiagnosticV2::BadOverload { node_id: node.id, name: symbol.into(), operands: vec![operand], location: self.bool_location(node) })
    }
    fn resolve_call(&self, node: &ValueExprHirV1, input: ResolvedTypeInputValueV2, args: &[CallArgHirV1], arg_types: &[ExpressionTypeStateV2]) -> Result<(ExpressionTypeStateV2, ExpressionEffectsV2, Option<ResolvedCallTargetV2>), TypeDiagnosticV2> {
        match input {
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(family)) => {
                let has_star = matches!(args, [CallArgHirV1::Star { .. }]);
                for entry in self.kernel.builtins.lookup_family(family) {
                    let matches = match &entry.signature {
                        CallableSignatureV2::AggregateStar { .. } => has_star,
                        CallableSignatureV2::AnySingle { .. } => !has_star && arg_types.len() == 1,
                        CallableSignatureV2::Fixed { params, .. } => !has_star && params.len() == arg_types.len() && params.iter().zip(arg_types).all(|(expected, actual)| matches!(actual, ExpressionTypeStateV2::Known(actual) if actual.is_assignable_to(expected))),
                    };
                    if matches { return Ok((ExpressionTypeStateV2::Known(entry.signature.result().clone()), entry.effects, Some(ResolvedCallTargetV2::Builtin(entry.id)))); }
                }
                if has_star { Err(TypeDiagnosticV2::InvalidStarArgument { node_id: node.id, location: self.location(node) }) } else { Err(TypeDiagnosticV2::BadOverload { node_id: node.id, name: format!("builtin-family-{}", family.raw()), operands: arg_types.iter().filter_map(|state| if let ExpressionTypeStateV2::Known(ty) = state { Some(ty.clone()) } else { None }).collect(), location: self.location(node) }) }
            }
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Predicate { id, signature, effects }) => {
                if args.iter().any(|arg| matches!(arg, CallArgHirV1::Star { .. })) { return Err(TypeDiagnosticV2::InvalidStarArgument { node_id: node.id, location: self.location(node) }); }
                if signature.params.len() == arg_types.len() && signature.params.iter().zip(arg_types).all(|(expected, actual)| matches!(actual, ExpressionTypeStateV2::Known(actual) if actual.is_assignable_to(expected))) {
                    Ok((ExpressionTypeStateV2::Known(signature.result), effects, Some(ResolvedCallTargetV2::Predicate(id))))
                } else { Err(TypeDiagnosticV2::BadOverload { node_id: node.id, name: "predicate".into(), operands: arg_types.iter().filter_map(|state| if let ExpressionTypeStateV2::Known(ty) = state { Some(ty.clone()) } else { None }).collect(), location: self.location(node) }) }
            }
            ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Unknown(cause)) => Ok((ExpressionTypeStateV2::Unknown(cause), ExpressionEffectsV2::pure_scalar(), None)),
            ResolvedTypeInputValueV2::Binding(_) => Err(TypeDiagnosticV2::WrongNodeFactKind { node_id: node.id, location: self.location(node) }),
        }
    }
}

fn lexical_nonzero(text: &str) -> bool {
    text.bytes().any(|byte| byte.is_ascii_digit() && byte != b'0')
}
