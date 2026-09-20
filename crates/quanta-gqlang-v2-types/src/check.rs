use crate::builtin::{
    BuiltinFamilyIdV2, BuiltinIdV2, BuiltinRegistryV2, CallableSignatureV2, ExpressionEffectsV2,
};
use crate::op::{OperatorIdV2, OperatorRegistryV2};
use crate::ty::{FiniteF64V2, RecordFieldTypeV1, RecordTypeV1, TypedLiteralValueV2, ValueTypeV1};
use crate::TypeDiagnosticV2;
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CallArgHirV1, CheckedHirDocumentV2,
    ExtendedSourceRegistryViewV2, HirNodeIdV1, HirVisitorV1, LiteralV1, LocatedSpanV2, SourceIdV2,
    ValueExprHirV1, ValueExprKindV1,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceQualifiedNodeIdV2 {
    pub source_id: SourceIdV2,
    pub node_id: HirNodeIdV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnknownTypeCauseV2 {
    UnresolvedEditorBinding,
    UnresolvedEditorCallable,
    EmptyCollectionElement,
    UnanchoredRuleVariable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExpressionTypeStateV2 {
    Known(ValueTypeV1),
    Unknown(UnknownTypeCauseV2),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedTypeInputValueV2 {
    Binding(ExpressionTypeStateV2),
    BindingFromExpression(SourceQualifiedNodeIdV2),
    /// A predeclared top-level declaration body. Unlike a materialized
    /// relation-stage binding this may be evaluated forward, within the same
    /// document-kernel invocation.
    BindingFromDeclaration(SourceQualifiedNodeIdV2),
    Callable(ResolvedCallableInputV2),
    RelationPredicate(ResolvedRelationPredicateInputV2),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedPredicateIdV2(String);
impl ResolvedPredicateIdV2 {
    pub fn try_new(value: &str) -> Result<Self, TypeDiagnosticV2> {
        if !canonical_identity(value) {
            return Err(TypeDiagnosticV2::InvalidResolvedIdentity);
        }
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The namespace kind is part of host catalog identity, not a source rule ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HostCallableNamespaceV2 {
    Predicate,
    Rule,
}

impl HostCallableNamespaceV2 {
    const fn tag(self) -> u8 {
        match self {
            Self::Predicate => 1,
            Self::Rule => 2,
        }
    }
}

/// Exact host catalog identity encoded without punctuation rewriting or a
/// source-rule identifier size limit. Module authority, not this neutral DTO,
/// proves that the parts came from a visible catalog asset.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedHostPredicateKeyV2(Vec<u8>);

impl ResolvedHostPredicateKeyV2 {
    const PREFIX: &'static [u8] = b"gqlang-host-predicate-v2\0";

    /// No allocation. The modules owner charges this exact byte count before
    /// constructing a key under its sealed authority budget.
    pub fn checked_canonical_len(
        module_id: &str,
        _kind: HostCallableNamespaceV2,
        local_name: &str,
    ) -> Result<usize, TypeDiagnosticV2> {
        if module_id.is_empty() || local_name.is_empty() {
            return Err(TypeDiagnosticV2::InvalidResolvedIdentity);
        }
        let _module_len = u64::try_from(module_id.len())
            .map_err(|_conversion_error| TypeDiagnosticV2::HostPredicateIdentityLimit)?;
        let _local_len = u64::try_from(local_name.len())
            .map_err(|_conversion_error| TypeDiagnosticV2::HostPredicateIdentityLimit)?;
        Self::PREFIX
            .len()
            .checked_add(8)
            .and_then(|size| size.checked_add(module_id.len()))
            .and_then(|size| size.checked_add(1))
            .and_then(|size| size.checked_add(8))
            .and_then(|size| size.checked_add(local_name.len()))
            .ok_or(TypeDiagnosticV2::HostPredicateIdentityLimit)
    }

    /// `max_encoded_bytes` must come from the caller's charged authority
    /// budget; a zero or undersized limit rejects before any allocation.
    pub fn try_from_catalog_parts(
        module_id: &str,
        kind: HostCallableNamespaceV2,
        local_name: &str,
        max_encoded_bytes: usize,
    ) -> Result<Self, TypeDiagnosticV2> {
        let total = Self::checked_canonical_len(module_id, kind, local_name)?;
        if max_encoded_bytes == 0 || total > max_encoded_bytes {
            return Err(TypeDiagnosticV2::HostPredicateIdentityLimit);
        }
        let module_len = u64::try_from(module_id.len())
            .map_err(|_conversion_error| TypeDiagnosticV2::HostPredicateIdentityLimit)?;
        let local_len = u64::try_from(local_name.len())
            .map_err(|_conversion_error| TypeDiagnosticV2::HostPredicateIdentityLimit)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(total)
            .map_err(|_allocation_error| TypeDiagnosticV2::HostPredicateIdentityLimit)?;
        bytes.extend_from_slice(Self::PREFIX);
        bytes.extend_from_slice(&module_len.to_be_bytes());
        bytes.extend_from_slice(module_id.as_bytes());
        bytes.push(kind.tag());
        bytes.extend_from_slice(&local_len.to_be_bytes());
        bytes.extend_from_slice(local_name.as_bytes());
        Ok(Self(bytes))
    }

    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedCallableInputV2 {
    Builtin(BuiltinFamilyIdV2),
    Predicate(ResolvedPredicateCallableV2),
    HostPredicate(ResolvedHostPredicateCallableV2),
    Unknown(UnknownTypeCauseV2),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedPredicateCallableV2 {
    id: ResolvedPredicateIdV2,
    params: Vec<ValueTypeV1>,
    result: ValueTypeV1,
    effects: ExpressionEffectsV2,
}
impl ResolvedPredicateCallableV2 {
    pub fn try_new(
        id: ResolvedPredicateIdV2,
        params: Vec<ValueTypeV1>,
        result: ValueTypeV1,
        effects: ExpressionEffectsV2,
    ) -> Result<Self, TypeDiagnosticV2> {
        if id.as_str().is_empty() {
            return Err(TypeDiagnosticV2::InvalidResolvedIdentity);
        }
        Ok(Self {
            id,
            params,
            result,
            effects,
        })
    }

    #[must_use]
    pub const fn id(&self) -> &ResolvedPredicateIdV2 {
        &self.id
    }
    #[must_use]
    pub fn params(&self) -> &[ValueTypeV1] {
        &self.params
    }
    #[must_use]
    pub const fn result(&self) -> &ValueTypeV1 {
        &self.result
    }
    #[must_use]
    pub const fn effects(&self) -> ExpressionEffectsV2 {
        self.effects
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedHostPredicateCallableV2 {
    key: ResolvedHostPredicateKeyV2,
    params: Vec<ValueTypeV1>,
    result: ValueTypeV1,
    effects: ExpressionEffectsV2,
}

impl ResolvedHostPredicateCallableV2 {
    pub fn try_new(
        key: ResolvedHostPredicateKeyV2,
        params: Vec<ValueTypeV1>,
        result: ValueTypeV1,
        effects: ExpressionEffectsV2,
    ) -> Result<Self, TypeDiagnosticV2> {
        Ok(Self {
            key,
            params,
            result,
            effects,
        })
    }

    #[must_use]
    pub const fn key(&self) -> &ResolvedHostPredicateKeyV2 {
        &self.key
    }
    #[must_use]
    pub fn params(&self) -> &[ValueTypeV1] {
        &self.params
    }
    #[must_use]
    pub const fn result(&self) -> &ValueTypeV1 {
        &self.result
    }
    #[must_use]
    pub const fn effects(&self) -> ExpressionEffectsV2 {
        self.effects
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRelationPredicateInputV2 {
    kind: ResolvedRelationPredicateKindV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ResolvedRelationPredicateKindV2 {
    Resolved {
        relation_source_id: crate::RelationSourceIdV2,
        capture_identity: String,
        effects: ExpressionEffectsV2,
    },
    Unknown(UnknownTypeCauseV2),
}

impl ResolvedRelationPredicateInputV2 {
    pub fn try_resolved(
        relation_source_id: crate::RelationSourceIdV2,
        capture_identity: &str,
        effects: ExpressionEffectsV2,
    ) -> Result<Self, TypeDiagnosticV2> {
        if !canonical_identity(capture_identity) {
            return Err(TypeDiagnosticV2::InvalidResolvedIdentity);
        }
        Ok(Self {
            kind: ResolvedRelationPredicateKindV2::Resolved {
                relation_source_id,
                capture_identity: capture_identity.to_owned(),
                effects,
            },
        })
    }

    #[must_use]
    pub const fn unknown(cause: UnknownTypeCauseV2) -> Self {
        Self {
            kind: ResolvedRelationPredicateKindV2::Unknown(cause),
        }
    }

    #[must_use]
    pub fn resolved_parts(
        &self,
    ) -> Option<(&crate::RelationSourceIdV2, &str, ExpressionEffectsV2)> {
        match &self.kind {
            ResolvedRelationPredicateKindV2::Resolved {
                relation_source_id,
                capture_identity,
                effects,
            } => Some((relation_source_id, capture_identity, *effects)),
            ResolvedRelationPredicateKindV2::Unknown(_) => None,
        }
    }

    #[must_use]
    pub const fn unknown_cause(&self) -> Option<&UnknownTypeCauseV2> {
        match &self.kind {
            ResolvedRelationPredicateKindV2::Resolved { .. } => None,
            ResolvedRelationPredicateKindV2::Unknown(cause) => Some(cause),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolvedCallTargetV2 {
    Builtin(BuiltinIdV2),
    Predicate(ResolvedPredicateIdV2),
    HostPredicate(ResolvedHostPredicateKeyV2),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedTypeInputEntryV2 {
    pub key: SourceQualifiedNodeIdV2,
    pub value: ResolvedTypeInputValueV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ContextualTypeReasonV2 {
    ExpectedArgument,
    ExpectedBinding,
    ExpectedResult,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextualExpectedTypeInputV2 {
    pub key: SourceQualifiedNodeIdV2,
    pub expected_type: ValueTypeV1,
    pub reason: ContextualTypeReasonV2,
}

#[derive(Clone, Debug)]
pub struct ResolvedTypeInputsV2<'a> {
    checked: CheckedHirDocumentV2<'a>,
    values: BTreeMap<SourceQualifiedNodeIdV2, ResolvedTypeInputValueV2>,
    contexts: BTreeMap<SourceQualifiedNodeIdV2, (ValueTypeV1, ContextualTypeReasonV2)>,
    context_locations: BTreeMap<SourceQualifiedNodeIdV2, quanta_gqlang_v2_hir::SpanV1>,
    source_registry: ExtendedSourceRegistryViewV2,
    declaration_bodies: BTreeMap<HirNodeIdV1, &'a ValueExprHirV1>,
}

impl<'a> ResolvedTypeInputsV2<'a> {
    pub fn try_new(
        checked: CheckedHirDocumentV2<'a>,
        source_registry: &ExtendedSourceRegistryViewV2,
        entries: Vec<ResolvedTypeInputEntryV2>,
        contexts: Vec<ContextualExpectedTypeInputV2>,
    ) -> Result<Self, TypeDiagnosticV2> {
        let source_id = checked.document().source_id;
        if source_registry.source_len(source_id) != Some(checked.document().source_len) {
            return Err(TypeDiagnosticV2::SourceRegistryMismatch {
                location: LocatedSpanV2 {
                    source_id,
                    span: quanta_gqlang_v2_hir::SpanV1::new(0, checked.document().source_len),
                },
            });
        }
        let mut required = RequiredInputs::default();
        for item in &checked.document().items {
            required.visit_item(item);
        }
        let declaration_bodies = checked
            .document()
            .items
            .iter()
            .filter_map(|item| match item {
                quanta_gqlang_v2_hir::ProgramItemHirV1::Decl(decl) => {
                    Some((decl.body.id, &decl.body))
                }
                quanta_gqlang_v2_hir::ProgramItemHirV1::Query(_)
                | quanta_gqlang_v2_hir::ProgramItemHirV1::Rule(_)
                | quanta_gqlang_v2_hir::ProgramItemHirV1::GraphQuery(_)
                | quanta_gqlang_v2_hir::ProgramItemHirV1::Legality(_)
                | quanta_gqlang_v2_hir::ProgramItemHirV1::Presentation(_)
                | quanta_gqlang_v2_hir::ProgramItemHirV1::CatalogDecl(_)
                | quanta_gqlang_v2_hir::ProgramItemHirV1::Composition(_)
                | quanta_gqlang_v2_hir::ProgramItemHirV1::Meta(_) => None,
            })
            .collect::<BTreeMap<_, _>>();
        let mut values = BTreeMap::new();
        for entry in entries {
            if entry.key.source_id != source_id {
                return Err(TypeDiagnosticV2::ForeignDocumentFact {
                    node_id: entry.key.node_id,
                    location: LocatedSpanV2 {
                        source_id: entry.key.source_id,
                        span: quanta_gqlang_v2_hir::SpanV1::point(0),
                    },
                });
            }
            let Some((span, expected_kind)) = required.nodes.get(&entry.key.node_id).copied()
            else {
                return Err(TypeDiagnosticV2::ExtraNodeFact {
                    node_id: entry.key.node_id,
                });
            };
            let kind_matches = matches!(
                (expected_kind, &entry.value),
                (
                    RequiredInputKind::Binding,
                    ResolvedTypeInputValueV2::Binding(_)
                ) | (
                    RequiredInputKind::Binding,
                    ResolvedTypeInputValueV2::BindingFromExpression(_)
                ) | (
                    RequiredInputKind::Binding,
                    ResolvedTypeInputValueV2::BindingFromDeclaration(_)
                ) | (
                    RequiredInputKind::Callable,
                    ResolvedTypeInputValueV2::Callable(_)
                ) | (
                    RequiredInputKind::RelationPredicate,
                    ResolvedTypeInputValueV2::RelationPredicate(_)
                )
            );
            if !kind_matches {
                return Err(TypeDiagnosticV2::WrongNodeFactKind {
                    node_id: entry.key.node_id,
                    location: LocatedSpanV2 { source_id, span },
                });
            }
            if let ResolvedTypeInputValueV2::BindingFromExpression(target) = &entry.value {
                if target.source_id != source_id
                    || target.node_id == entry.key.node_id
                    || !required.value_nodes.contains_key(&target.node_id)
                {
                    return Err(TypeDiagnosticV2::InvalidBindingDependency {
                        node_id: entry.key.node_id,
                        target_node_id: target.node_id,
                        location: LocatedSpanV2 { source_id, span },
                    });
                }
            }
            if let ResolvedTypeInputValueV2::BindingFromDeclaration(target) = &entry.value {
                if target.source_id != source_id
                    || !required
                        .declaration_value_nodes
                        .contains(&entry.key.node_id)
                    || !declaration_bodies.contains_key(&target.node_id)
                {
                    return Err(TypeDiagnosticV2::InvalidBindingDependency {
                        node_id: entry.key.node_id,
                        target_node_id: target.node_id,
                        location: LocatedSpanV2 { source_id, span },
                    });
                }
            }
            if values.insert(entry.key, entry.value).is_some() {
                return Err(TypeDiagnosticV2::DuplicateNodeFact {
                    node_id: entry.key.node_id,
                    location: LocatedSpanV2 { source_id, span },
                });
            }
        }
        for (node_id, (span, _)) in &required.nodes {
            let key = SourceQualifiedNodeIdV2 {
                source_id,
                node_id: *node_id,
            };
            if !values.contains_key(&key) {
                return Err(TypeDiagnosticV2::MissingNodeFact {
                    node_id: *node_id,
                    location: LocatedSpanV2 {
                        source_id,
                        span: *span,
                    },
                });
            }
        }
        let mut checked_contexts = BTreeMap::new();
        for context in contexts {
            if context.key.source_id != source_id {
                return Err(TypeDiagnosticV2::ForeignDocumentFact {
                    node_id: context.key.node_id,
                    location: LocatedSpanV2 {
                        source_id: context.key.source_id,
                        span: quanta_gqlang_v2_hir::SpanV1::point(0),
                    },
                });
            }
            if !required.context_nodes.contains_key(&context.key.node_id) {
                return Err(TypeDiagnosticV2::ExtraNodeFact {
                    node_id: context.key.node_id,
                });
            }
            if checked_contexts
                .insert(context.key, (context.expected_type, context.reason))
                .is_some()
            {
                return Err(TypeDiagnosticV2::DuplicateNodeFact {
                    node_id: context.key.node_id,
                    location: LocatedSpanV2 {
                        source_id,
                        span: quanta_gqlang_v2_hir::SpanV1::point(0),
                    },
                });
            }
        }
        let context_locations = required
            .context_nodes
            .into_iter()
            .map(|(node_id, span)| (SourceQualifiedNodeIdV2 { source_id, node_id }, span))
            .collect();
        Ok(Self {
            checked,
            values,
            contexts: checked_contexts,
            context_locations,
            source_registry: source_registry.clone(),
            declaration_bodies,
        })
    }
}

#[derive(Default)]
struct RequiredInputs {
    nodes: BTreeMap<HirNodeIdV1, (quanta_gqlang_v2_hir::SpanV1, RequiredInputKind)>,
    context_nodes: BTreeMap<HirNodeIdV1, quanta_gqlang_v2_hir::SpanV1>,
    value_nodes: BTreeMap<HirNodeIdV1, quanta_gqlang_v2_hir::SpanV1>,
    declaration_value_nodes: BTreeSet<HirNodeIdV1>,
}
#[derive(Clone, Copy)]
enum RequiredInputKind {
    Binding,
    Callable,
    RelationPredicate,
}
impl HirVisitorV1 for RequiredInputs {
    fn visit_decl(&mut self, node: &quanta_gqlang_v2_hir::DeclHirV1) {
        struct DeclValueCollector<'a>(&'a mut BTreeSet<HirNodeIdV1>);
        impl HirVisitorV1 for DeclValueCollector<'_> {
            fn visit_value(&mut self, value: &ValueExprHirV1) {
                let _ = self.0.insert(value.id);
                quanta_gqlang_v2_hir::walk_value(self, value);
            }
        }
        DeclValueCollector(&mut self.declaration_value_nodes).visit_value(&node.body);
        quanta_gqlang_v2_hir::walk_decl(self, node);
    }
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        let _ = self.value_nodes.insert(node.id, node.span);
        let kind = match node.kind {
            ValueExprKindV1::Symbol(_) => Some(RequiredInputKind::Binding),
            ValueExprKindV1::Call(_) => Some(RequiredInputKind::Callable),
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => None,
        };
        if let Some(kind) = kind {
            let _ = self.nodes.insert(node.id, (node.span, kind));
        }
        if matches!(&node.kind, ValueExprKindV1::List(list) if list.elements.is_empty()) {
            let _ = self.context_nodes.insert(node.id, node.span);
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        if matches!(node.kind, BoolExprKindV1::RelationExists(_)) {
            let _ = self
                .nodes
                .insert(node.id, (node.span, RequiredInputKind::RelationPredicate));
        }
        quanta_gqlang_v2_hir::walk_bool(self, node);
    }
}

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
    pub effects: ExpressionEffectsV2,
    pub literal: Option<TypedLiteralFactV2>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedExpressionFactsV2 {
    facts: BTreeMap<SourceQualifiedNodeIdV2, TypedExpressionFactV2>,
}
impl TypedExpressionFactsV2 {
    #[must_use]
    pub fn get(&self, key: SourceQualifiedNodeIdV2) -> Option<&TypedExpressionFactV2> {
        self.facts.get(&key)
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.facts.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }
}
impl TypedExpressionFactV2 {
    #[must_use]
    pub fn resolved_call(&self) -> Option<&ResolvedCallTargetV2> {
        self.resolved_call.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct ExpressionTypeKernelV2 {
    operators: OperatorRegistryV2,
    builtins: BuiltinRegistryV2,
}
impl ExpressionTypeKernelV2 {
    pub fn try_new(
        operators: OperatorRegistryV2,
        builtins: BuiltinRegistryV2,
    ) -> Result<Self, TypeDiagnosticV2> {
        Ok(Self {
            operators,
            builtins,
        })
    }
    pub fn try_closed() -> Result<Self, TypeDiagnosticV2> {
        Ok(Self {
            operators: OperatorRegistryV2::try_closed()?,
            builtins: BuiltinRegistryV2::try_closed()?,
        })
    }

    pub fn check_document(
        &self,
        inputs: &ResolvedTypeInputsV2<'_>,
    ) -> Result<TypedExpressionFactsV2, TypeDiagnosticV2> {
        let mut checker = DocumentChecker {
            kernel: self,
            inputs,
            facts: BTreeMap::new(),
            visiting: BTreeSet::new(),
            used_contexts: BTreeSet::new(),
            callable_contexts: BTreeMap::new(),
            error: None,
        };
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
        })
    }
}

struct DocumentChecker<'a, 'b> {
    kernel: &'a ExpressionTypeKernelV2,
    inputs: &'a ResolvedTypeInputsV2<'b>,
    facts: BTreeMap<SourceQualifiedNodeIdV2, TypedExpressionFactV2>,
    visiting: BTreeSet<HirNodeIdV1>,
    used_contexts: BTreeSet<SourceQualifiedNodeIdV2>,
    callable_contexts: BTreeMap<SourceQualifiedNodeIdV2, ValueTypeV1>,
    error: Option<TypeDiagnosticV2>,
}

impl HirVisitorV1 for DocumentChecker<'_, '_> {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        if self.error.is_none() {
            if let Err(error) = self.type_expr(node, false) {
                self.error = Some(error);
            }
        }
    }
    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        if self.error.is_none() {
            if let Err(error) = self.type_bool(node) {
                self.error = Some(error);
            }
        }
    }
}

impl DocumentChecker<'_, '_> {
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
                quanta_gqlang_v2_hir::walk_relation(self, &exists.relation);
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
                        ResolvedRelationPredicateKindV2::Resolved { effects: base, .. } => {
                            effects = base.compose(child_effects);
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
                effects,
                literal,
            },
        );
        Ok(state)
    }
    fn type_expr(
        &mut self,
        node: &ValueExprHirV1,
        deferred_min_child: bool,
    ) -> Result<ExpressionTypeStateV2, TypeDiagnosticV2> {
        let key = self.key(node);
        if let Some(fact) = self.facts.get(&key) {
            return Ok(fact.state.clone());
        }
        if self.visiting.contains(&node.id) {
            return Err(TypeDiagnosticV2::CyclicDeclarationDependency {
                node_id: node.id,
                location: self.location(node),
            });
        }
        let _ = self.visiting.insert(node.id);
        let mut operator_id = None;
        let mut resolved_call = None;
        let mut literal = None;
        let mut effects = ExpressionEffectsV2::pure_scalar();
        let state = match &node.kind {
            ValueExprKindV1::Literal(value) => {
                let (ty, fact) = self.type_literal(node, value, deferred_min_child)?;
                literal = Some(fact);
                ExpressionTypeStateV2::Known(ty)
            }
            ValueExprKindV1::Symbol(_) => match self.inputs.values.get(&key) {
                Some(ResolvedTypeInputValueV2::Binding(state)) => state.clone(),
                Some(ResolvedTypeInputValueV2::BindingFromExpression(target)) => self
                    .facts
                    .get(target)
                    .map(|fact| fact.state.clone())
                    .ok_or(TypeDiagnosticV2::UnavailableBindingDependency {
                        node_id: node.id,
                        target_node_id: target.node_id,
                        location: self.location(node),
                    })?,
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
                    self.type_expr(body, false)?
                }
                _ => {
                    return Err(TypeDiagnosticV2::MissingNodeFact {
                        node_id: node.id,
                        location: self.location(node),
                    })
                }
            },
            ValueExprKindV1::Paren(inner) => {
                let state = self.type_expr(inner, false)?;
                effects = self.effects_for(inner.id)?;
                state
            }
            ValueExprKindV1::Unary(unary) => {
                let min_case = unary.op.symbol == "-"
                    && matches!(&unary.operand.kind, ValueExprKindV1::Literal(LiteralV1::Int(text)) if text.trim() == "170141183460469231731687303715884105728");
                let operand = self.type_expr(&unary.operand, min_case)?;
                if min_case {
                    literal = Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Int(
                        i128::MIN,
                    )));
                    let mut ignored = ExpressionEffectsV2::pure_scalar();
                    let _ = self.resolve_unary(
                        node,
                        "-",
                        ExpressionTypeStateV2::Known(ValueTypeV1::int()),
                        &mut operator_id,
                        &mut ignored,
                    )?;
                    effects = ignored.join(self.effects_for(unary.operand.id)?);
                    ExpressionTypeStateV2::Known(ValueTypeV1::int())
                } else {
                    let state = self.resolve_unary(
                        node,
                        &unary.op.symbol,
                        operand,
                        &mut operator_id,
                        &mut effects,
                    )?;
                    effects = effects.join(self.effects_for(unary.operand.id)?);
                    state
                }
            }
            ValueExprKindV1::Binary(binary) => {
                let lhs = self.type_expr(&binary.lhs, false)?;
                let rhs = self.type_expr(&binary.rhs, false)?;
                let child_effects = [
                    self.effects_for(binary.lhs.id)?,
                    self.effects_for(binary.rhs.id)?,
                ];
                let state = self.resolve_binary(
                    node,
                    &binary.op.symbol,
                    lhs,
                    rhs,
                    &mut operator_id,
                    &mut effects,
                )?;
                effects = effects.compose(child_effects);
                state
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
                let args = call.args().as_slice();
                let callable_contexts = self.callable_argument_contexts(&input, args);
                let mut arg_types = Vec::new();
                for (index, arg) in args.iter().enumerate() {
                    if let CallArgHirV1::Expr(expr) = arg {
                        if let Some(Some(expected)) = callable_contexts.get(index) {
                            let _ = self
                                .callable_contexts
                                .insert(self.key(expr), expected.clone());
                        }
                        arg_types.push(self.type_expr(expr, false)?);
                    }
                }
                let (state, call_effects, call_target) =
                    self.resolve_call(node, input, args, &arg_types)?;
                let arg_effects = args.iter().filter_map(|arg| match arg {
                    CallArgHirV1::Expr(expr) => {
                        self.facts.get(&self.key(expr)).map(|fact| fact.effects)
                    }
                    CallArgHirV1::Star { .. } => None,
                });
                effects = call_effects.compose(arg_effects);
                resolved_call = call_target;
                state
            }
            ValueExprKindV1::Path(path) => {
                let mut current = self.type_expr(&path.base, false)?;
                effects = self.effects_for(path.base.id)?;
                for segment in path.segments.as_slice() {
                    current = match current {
                        ExpressionTypeStateV2::Unknown(cause) => {
                            ExpressionTypeStateV2::Unknown(cause)
                        }
                        ExpressionTypeStateV2::Known(base) => {
                            let (record, wrap_optional) = if let Some(record) = base.as_record() {
                                (record, false)
                            } else if let Some(optional) = base.as_optional() {
                                let Some(record) = optional.inner().as_record() else {
                                    return Err(TypeDiagnosticV2::BadProjection {
                                        node_id: node.id,
                                        base,
                                        field: segment.name.clone(),
                                        location: LocatedSpanV2 {
                                            source_id: key.source_id,
                                            span: segment.span,
                                        },
                                    });
                                };
                                (record, true)
                            } else {
                                return Err(TypeDiagnosticV2::BadProjection {
                                    node_id: node.id,
                                    base,
                                    field: segment.name.clone(),
                                    location: LocatedSpanV2 {
                                        source_id: key.source_id,
                                        span: segment.span,
                                    },
                                });
                            };
                            let field = record.field(&segment.name).ok_or_else(|| {
                                TypeDiagnosticV2::BadProjection {
                                    node_id: node.id,
                                    base: base.clone(),
                                    field: segment.name.clone(),
                                    location: LocatedSpanV2 {
                                        source_id: key.source_id,
                                        span: segment.span,
                                    },
                                }
                            })?;
                            let projected =
                                if wrap_optional && field.value_type().as_optional().is_none() {
                                    ValueTypeV1::try_optional(field.value_type().clone())?
                                } else {
                                    field.value_type().clone()
                                };
                            ExpressionTypeStateV2::Known(projected)
                        }
                    };
                }
                current
            }
            ValueExprKindV1::Record(record) => {
                let mut fields = Vec::new();
                let mut unknown = None;
                for field in &record.fields {
                    match self.type_expr(&field.value, false)? {
                        ExpressionTypeStateV2::Known(ty) => {
                            let location = self
                                .inputs
                                .source_registry
                                .try_check_location(LocatedSpanV2 {
                                    source_id: key.source_id,
                                    span: field.name_span,
                                })
                                .map_err(|_source_error| TypeDiagnosticV2::InvalidTypeName {
                                    location: LocatedSpanV2 {
                                        source_id: key.source_id,
                                        span: field.name_span,
                                    },
                                })?;
                            fields.push(RecordFieldTypeV1::try_new(&field.name, ty, location)?);
                        }
                        ExpressionTypeStateV2::Unknown(cause) => unknown = Some(cause),
                    }
                }
                effects = ExpressionEffectsV2::pure_scalar().compose(
                    record.fields.iter().filter_map(|field| {
                        self.facts
                            .get(&self.key(&field.value))
                            .map(|fact| fact.effects)
                    }),
                );
                match unknown {
                    Some(cause) => ExpressionTypeStateV2::Unknown(cause),
                    None => ExpressionTypeStateV2::Known(ValueTypeV1::try_record(
                        RecordTypeV1::try_new(fields)?,
                    )?),
                }
            }
            ValueExprKindV1::List(list) => {
                let mut selected: Option<ValueTypeV1> = None;
                let mut unknown = None;
                for value in &list.elements {
                    match self.type_expr(value, false)? {
                        ExpressionTypeStateV2::Unknown(cause) => unknown = Some(cause),
                        ExpressionTypeStateV2::Known(ty) => match &selected {
                            None => selected = Some(ty),
                            Some(expected) if ty.is_assignable_to(expected) => {}
                            Some(expected) if expected.is_assignable_to(&ty) => selected = Some(ty),
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
                }
                effects =
                    ExpressionEffectsV2::pure_scalar().compose(list.elements.iter().filter_map(
                        |value| self.facts.get(&self.key(value)).map(|fact| fact.effects),
                    ));
                match (unknown, selected) {
                    (Some(cause), _) => ExpressionTypeStateV2::Unknown(cause),
                    (None, Some(element)) => {
                        ExpressionTypeStateV2::Known(ValueTypeV1::try_list(element)?)
                    }
                    (None, None) => match self.inputs.contexts.get(&key) {
                        Some((expected, _))
                            if expected.as_collection().is_some_and(|collection| {
                                collection.cardinality() == crate::CardinalityV1::Sequence
                            }) =>
                        {
                            let _ = self.used_contexts.insert(key);
                            ExpressionTypeStateV2::Known(expected.clone())
                        }
                        _ => match self.callable_contexts.get(&key) {
                            Some(expected)
                                if expected.as_collection().is_some_and(|collection| {
                                    collection.cardinality() == crate::CardinalityV1::Sequence
                                }) =>
                            {
                                ExpressionTypeStateV2::Known(expected.clone())
                            }
                            _ => ExpressionTypeStateV2::Unknown(
                                UnknownTypeCauseV2::EmptyCollectionElement,
                            ),
                        },
                    },
                }
            }
        };
        let _ = self.visiting.remove(&node.id);
        let _ = self.facts.insert(
            key,
            TypedExpressionFactV2 {
                state: state.clone(),
                operator_id,
                resolved_call,
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
        let overloads = self.kernel.operators.unary_overloads(symbol);
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
        let overloads = self.kernel.operators.binary_overloads(symbol);
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

fn canonical_identity(value: &str) -> bool {
    if value.len() > crate::MAX_TYPE_NAME_BYTES {
        return false;
    }
    let mut segments = value.split("::");
    let valid_segment = |segment: &str| {
        let mut bytes = segment.bytes();
        let Some(first) = bytes.next() else {
            return false;
        };
        (first.is_ascii_alphabetic() || first == b'_')
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    };
    segments.all(valid_segment)
}
