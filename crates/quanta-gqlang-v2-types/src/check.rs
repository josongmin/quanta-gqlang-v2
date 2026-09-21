use crate::builtin::{
    BuiltinFamilyIdV2, BuiltinIdV2, BuiltinRegistryV2, CallableSignatureV2, ExpressionEffectsV2,
};
use crate::op::{OperatorIdV2, OperatorRegistryV2};
use crate::ty::{FiniteF64V2, RecordFieldTypeV1, RecordTypeV1, TypedLiteralValueV2, ValueTypeV1};
use crate::TypeDiagnosticV2;
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CallArgHirV1, CheckedHirDocumentV2, CheckedLocatedSpanV2,
    ExtendedSourceRegistryViewV2, HirNodeIdV1, HirVisitorV1, LiteralV1, LocatedSpanV2,
    RelationExprHirV1, RelationExprKindV1, SourceIdV2, ValueExprHirV1, ValueExprKindV1,
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
    /// A record value copied from an exact relation root completed earlier in
    /// the same document traversal.
    BindingFromRelationRow(SourceQualifiedNodeIdV2),
    Callable(ResolvedCallableInputV2),
    RelationPredicate(ResolvedRelationPredicateInputV2),
}

/// Type authority for one deferred relation-row field. `PriorExpression`
/// never triggers a second kernel pass: the referenced expression must already
/// have been typed when its owning relation root completes traversal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelationRowFieldSourceV2 {
    Known(ValueTypeV1),
    PriorExpression(SourceQualifiedNodeIdV2),
}

/// One ordered field submitted for a relation root whose row is sealed by the
/// types owner. Names and locations stay row-aligned with the resulting fact.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeferredRelationRowFieldInputV2 {
    ordinal: usize,
    name: String,
    name_location: CheckedLocatedSpanV2,
    source: RelationRowFieldSourceV2,
}

impl DeferredRelationRowFieldInputV2 {
    #[must_use]
    pub fn new(
        ordinal: usize,
        name: impl Into<String>,
        name_location: CheckedLocatedSpanV2,
        source: RelationRowFieldSourceV2,
    ) -> Self {
        Self {
            ordinal,
            name: name.into(),
            name_location,
            source,
        }
    }
}

/// How the row for one relation stage is produced. Filter, sort, and join rows
/// are inferred from their checked HIR children and therefore have no caller
/// supplied row input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeferredRelationRowKindV2 {
    /// Authoritative source fields with already sealed scalar/value types.
    KnownSource {
        expected_field_count: usize,
        fields: Vec<DeferredRelationRowFieldInputV2>,
    },
    /// A Composition-local scan of a relation root completed earlier.
    PriorRelation { target: SourceQualifiedNodeIdV2 },
    /// Project/group fields whose types come from exact expressions in this
    /// stage. The types owner constructs the record after those expressions.
    ExpressionFields {
        expected_field_count: usize,
        fields: Vec<DeferredRelationRowFieldInputV2>,
    },
}

/// Complete ordered row request for one exact checked-HIR relation root. Only
/// field-producing variants carry an expected count; copying a prior sealed
/// row has no caller-supplied schema assertion to ignore or forge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeferredRelationRowInputV2 {
    root: SourceQualifiedNodeIdV2,
    kind: DeferredRelationRowKindV2,
}

impl DeferredRelationRowInputV2 {
    #[must_use]
    pub fn known_source(
        root: SourceQualifiedNodeIdV2,
        expected_field_count: usize,
        fields: Vec<DeferredRelationRowFieldInputV2>,
    ) -> Self {
        Self {
            root,
            kind: DeferredRelationRowKindV2::KnownSource {
                expected_field_count,
                fields,
            },
        }
    }

    #[must_use]
    pub const fn prior_relation(
        root: SourceQualifiedNodeIdV2,
        target: SourceQualifiedNodeIdV2,
    ) -> Self {
        Self {
            root,
            kind: DeferredRelationRowKindV2::PriorRelation { target },
        }
    }

    #[must_use]
    pub fn expression_fields(
        root: SourceQualifiedNodeIdV2,
        expected_field_count: usize,
        fields: Vec<DeferredRelationRowFieldInputV2>,
    ) -> Self {
        Self {
            root,
            kind: DeferredRelationRowKindV2::ExpressionFields {
                expected_field_count,
                fields,
            },
        }
    }
}

/// Ordered relation roots that must be completed before the normal authored
/// document traversal. This is an input, not a proof: `ResolvedTypeInputsV2`
/// validates its source, membership, uniqueness, and every deferred-row
/// dependency before the kernel can consume it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RelationPreEvaluationInputV2 {
    roots: Vec<SourceQualifiedNodeIdV2>,
}

impl RelationPreEvaluationInputV2 {
    #[must_use]
    pub fn new(roots: Vec<SourceQualifiedNodeIdV2>) -> Self {
        Self { roots }
    }

    #[must_use]
    pub fn roots(&self) -> &[SourceQualifiedNodeIdV2] {
        &self.roots
    }
}

/// One exact Binder capture edge retained without importing Binder types.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResolvedRelationCaptureInputV2 {
    reference: SourceQualifiedNodeIdV2,
    target_owner: SourceQualifiedNodeIdV2,
    target_ordinal: usize,
}

impl ResolvedRelationCaptureInputV2 {
    #[must_use]
    pub const fn new(
        reference: SourceQualifiedNodeIdV2,
        target_owner: SourceQualifiedNodeIdV2,
        target_ordinal: usize,
    ) -> Self {
        Self {
            reference,
            target_owner,
            target_ordinal,
        }
    }
    #[must_use]
    pub const fn reference(&self) -> SourceQualifiedNodeIdV2 {
        self.reference
    }
    #[must_use]
    pub const fn target_owner(&self) -> SourceQualifiedNodeIdV2 {
        self.target_owner
    }
    #[must_use]
    pub const fn target_ordinal(&self) -> usize {
        self.target_ordinal
    }
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
        relation_root: SourceQualifiedNodeIdV2,
        captures: Vec<ResolvedRelationCaptureInputV2>,
    },
    Unknown(UnknownTypeCauseV2),
}

impl ResolvedRelationPredicateInputV2 {
    pub fn try_resolved(
        relation_root: SourceQualifiedNodeIdV2,
        captures: Vec<ResolvedRelationCaptureInputV2>,
    ) -> Result<Self, TypeDiagnosticV2> {
        let mut seen_references = BTreeSet::new();
        for capture in &captures {
            if capture.reference.source_id != relation_root.source_id
                || capture.target_owner.source_id != relation_root.source_id
                || !seen_references.insert(capture.reference.node_id)
            {
                return Err(TypeDiagnosticV2::InvalidRelationPredicateInput {
                    node_id: relation_root.node_id,
                    location: LocatedSpanV2 {
                        source_id: relation_root.source_id,
                        span: quanta_gqlang_v2_hir::SpanV1::point(0),
                    },
                });
            }
        }
        Ok(Self {
            kind: ResolvedRelationPredicateKindV2::Resolved {
                relation_root,
                captures,
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
    relation_rows: BTreeMap<SourceQualifiedNodeIdV2, DeferredRelationRowInputV2>,
    pre_evaluation_roots: Vec<RelationExprHirV1>,
}

impl<'a> ResolvedTypeInputsV2<'a> {
    pub fn try_new(
        checked: CheckedHirDocumentV2<'a>,
        source_registry: &ExtendedSourceRegistryViewV2,
        entries: Vec<ResolvedTypeInputEntryV2>,
        contexts: Vec<ContextualExpectedTypeInputV2>,
        relation_rows: Vec<DeferredRelationRowInputV2>,
    ) -> Result<Self, TypeDiagnosticV2> {
        Self::try_new_with_relation_pre_evaluation(
            checked,
            source_registry,
            entries,
            contexts,
            relation_rows,
            RelationPreEvaluationInputV2::default(),
        )
    }

    /// Construct checked type inputs with an explicit dependency-first
    /// relation pre-evaluation plan. The legacy constructor is deliberately a
    /// no-plan shim so existing authored-order callers keep their contract.
    pub fn try_new_with_relation_pre_evaluation(
        checked: CheckedHirDocumentV2<'a>,
        source_registry: &ExtendedSourceRegistryViewV2,
        entries: Vec<ResolvedTypeInputEntryV2>,
        contexts: Vec<ContextualExpectedTypeInputV2>,
        relation_rows: Vec<DeferredRelationRowInputV2>,
        pre_evaluation: RelationPreEvaluationInputV2,
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
                    RequiredInputKind::Binding,
                    ResolvedTypeInputValueV2::BindingFromRelationRow(_)
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
            if let ResolvedTypeInputValueV2::BindingFromRelationRow(target) = &entry.value {
                if target.source_id != source_id
                    || !required.relations.contains_key(&target.node_id)
                    || target.node_id == entry.key.node_id
                {
                    return Err(TypeDiagnosticV2::InvalidBindingDependency {
                        node_id: entry.key.node_id,
                        target_node_id: target.node_id,
                        location: LocatedSpanV2 { source_id, span },
                    });
                }
            }
            if let ResolvedTypeInputValueV2::RelationPredicate(input) = &entry.value {
                validate_relation_predicate_input(entry.key, span, input, &required)?;
            }
            if values.insert(entry.key, entry.value).is_some() {
                return Err(TypeDiagnosticV2::DuplicateNodeFact {
                    node_id: entry.key.node_id,
                    location: LocatedSpanV2 { source_id, span },
                });
            }
        }
        let pre_evaluation_roots =
            validate_relation_pre_evaluation(source_id, pre_evaluation, &required)?;
        let relation_rows = validate_relation_rows(
            source_id,
            source_registry,
            relation_rows,
            &required,
            &pre_evaluation_roots.positions,
        )?;
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
            relation_rows,
            pre_evaluation_roots: pre_evaluation_roots.roots,
        })
    }
}

struct ValidatedRelationPreEvaluationV2 {
    positions: BTreeMap<HirNodeIdV1, usize>,
    roots: Vec<RelationExprHirV1>,
}

fn validate_relation_pre_evaluation(
    source_id: SourceIdV2,
    input: RelationPreEvaluationInputV2,
    required: &RequiredInputs,
) -> Result<ValidatedRelationPreEvaluationV2, TypeDiagnosticV2> {
    let mut positions = BTreeMap::new();
    let mut roots = Vec::with_capacity(input.roots.len());
    for (position, root) in input.roots.into_iter().enumerate() {
        if root.source_id != source_id {
            return Err(TypeDiagnosticV2::ForeignDocumentFact {
                node_id: root.node_id,
                location: LocatedSpanV2 {
                    source_id: root.source_id,
                    span: quanta_gqlang_v2_hir::SpanV1::point(0),
                },
            });
        }
        let Some(requirement) = required.relations.get(&root.node_id) else {
            return Err(TypeDiagnosticV2::InvalidRelationPreEvaluationPlan {
                root: root.node_id,
                location: LocatedSpanV2 {
                    source_id,
                    span: quanta_gqlang_v2_hir::SpanV1::point(0),
                },
            });
        };
        if positions.insert(root.node_id, position).is_some() {
            return Err(TypeDiagnosticV2::InvalidRelationPreEvaluationPlan {
                root: root.node_id,
                location: LocatedSpanV2 {
                    source_id,
                    span: requirement.span,
                },
            });
        }
        let relation = required.relation_nodes.get(&root.node_id).cloned().ok_or(
            TypeDiagnosticV2::InvalidRelationPreEvaluationPlan {
                root: root.node_id,
                location: LocatedSpanV2 {
                    source_id,
                    span: requirement.span,
                },
            },
        )?;
        roots.push(relation);
    }
    Ok(ValidatedRelationPreEvaluationV2 { positions, roots })
}

fn validate_relation_predicate_input(
    key: SourceQualifiedNodeIdV2,
    span: quanta_gqlang_v2_hir::SpanV1,
    input: &ResolvedRelationPredicateInputV2,
    required: &RequiredInputs,
) -> Result<(), TypeDiagnosticV2> {
    let invalid = || TypeDiagnosticV2::InvalidRelationPredicateInput {
        node_id: key.node_id,
        location: LocatedSpanV2 {
            source_id: key.source_id,
            span,
        },
    };
    let Some(requirement) = required.relation_predicates.get(&key.node_id) else {
        return Err(invalid());
    };
    let ResolvedRelationPredicateKindV2::Resolved {
        relation_root,
        captures,
        ..
    } = &input.kind
    else {
        return Ok(());
    };
    if relation_root.source_id != key.source_id
        || relation_root.node_id != requirement.relation_root
    {
        return Err(invalid());
    }
    let positions = requirement
        .ordered_references
        .iter()
        .enumerate()
        .map(|(position, node)| (*node, position))
        .collect::<BTreeMap<_, _>>();
    let mut prior_position = None;
    let mut seen_references = BTreeSet::new();
    for capture in captures {
        let Some(position) = positions.get(&capture.reference.node_id).copied() else {
            return Err(invalid());
        };
        if capture.reference.source_id != key.source_id
            || capture.target_owner.source_id != key.source_id
            || !required
                .capture_owner_nodes
                .contains(&capture.target_owner.node_id)
            || !seen_references.insert(capture.reference.node_id)
            || prior_position.is_some_and(|prior| prior >= position)
        {
            return Err(invalid());
        }
        prior_position = Some(position);
    }
    Ok(())
}

fn validate_relation_rows(
    source_id: SourceIdV2,
    source_registry: &ExtendedSourceRegistryViewV2,
    inputs: Vec<DeferredRelationRowInputV2>,
    required: &RequiredInputs,
    pre_evaluation_positions: &BTreeMap<HirNodeIdV1, usize>,
) -> Result<BTreeMap<SourceQualifiedNodeIdV2, DeferredRelationRowInputV2>, TypeDiagnosticV2> {
    let mut rows = BTreeMap::new();
    for input in inputs {
        let Some(requirement) = required.relations.get(&input.root.node_id) else {
            return Err(TypeDiagnosticV2::InvalidDeferredRelationRow {
                root: input.root.node_id,
                location: LocatedSpanV2 {
                    source_id: input.root.source_id,
                    span: quanta_gqlang_v2_hir::SpanV1::point(0),
                },
            });
        };
        let location = LocatedSpanV2 {
            source_id,
            span: requirement.span,
        };
        if input.root.source_id != source_id {
            return Err(TypeDiagnosticV2::InvalidDeferredRelationRow {
                root: input.root.node_id,
                location,
            });
        }
        let valid_kind = matches!(
            (&requirement.kind, &input.kind),
            (
                RelationRequirementKindV2::Source,
                DeferredRelationRowKindV2::KnownSource { .. }
            ) | (
                RelationRequirementKindV2::Source,
                DeferredRelationRowKindV2::PriorRelation { .. }
            ) | (
                RelationRequirementKindV2::Project(_),
                DeferredRelationRowKindV2::ExpressionFields { .. }
            ) | (
                RelationRequirementKindV2::Group(_),
                DeferredRelationRowKindV2::ExpressionFields { .. }
            )
        );
        if !valid_kind {
            return Err(TypeDiagnosticV2::InvalidDeferredRelationRow {
                root: input.root.node_id,
                location,
            });
        }
        match &input.kind {
            DeferredRelationRowKindV2::KnownSource {
                expected_field_count,
                fields,
            }
            | DeferredRelationRowKindV2::ExpressionFields {
                expected_field_count,
                fields,
            } => {
                let expected_expressions = match &requirement.kind {
                    RelationRequirementKindV2::Project(expressions)
                    | RelationRequirementKindV2::Group(expressions) => Some(expressions.as_slice()),
                    RelationRequirementKindV2::Source => None,
                    RelationRequirementKindV2::Filter
                    | RelationRequirementKindV2::Join
                    | RelationRequirementKindV2::Sort => unreachable!("kind checked above"),
                };
                if fields.len() != *expected_field_count
                    || expected_expressions.is_some_and(|expected| expected.len() != fields.len())
                {
                    return Err(TypeDiagnosticV2::InvalidDeferredRelationRow {
                        root: input.root.node_id,
                        location,
                    });
                }
                let mut names = BTreeSet::new();
                for (ordinal, field) in fields.iter().enumerate() {
                    let field_location = field.name_location.location();
                    let valid_source = match (&input.kind, &field.source, expected_expressions) {
                        (
                            DeferredRelationRowKindV2::KnownSource { .. },
                            RelationRowFieldSourceV2::Known(_),
                            None,
                        ) => true,
                        (
                            DeferredRelationRowKindV2::ExpressionFields { .. },
                            RelationRowFieldSourceV2::PriorExpression(target),
                            Some(expected),
                        ) => {
                            target.source_id == source_id
                                && expected.get(ordinal) == Some(&target.node_id)
                        }
                        _ => false,
                    };
                    if field.ordinal != ordinal
                        || field.name.is_empty()
                        || !names.insert(field.name.as_str())
                        || field_location.source_id != source_id
                        || field.name_location.registry_identity() != &source_registry.identity()
                        || !valid_source
                    {
                        return Err(TypeDiagnosticV2::InvalidDeferredRelationRow {
                            root: input.root.node_id,
                            location: field_location,
                        });
                    }
                }
            }
            DeferredRelationRowKindV2::PriorRelation { target } => {
                let root_plan =
                    required.pre_evaluation_scope(input.root.node_id, pre_evaluation_positions);
                let target_plan =
                    required.pre_evaluation_scope(target.node_id, pre_evaluation_positions);
                let root_order = required.relation_order.get(&input.root.node_id).copied();
                let target_order = required.relation_order.get(&target.node_id).copied();
                let available = match root_plan {
                    // A root prewalked ahead of authored traversal may only
                    // depend on another exact prewalk root completed earlier.
                    Some(root_position) => {
                        matches!(target_plan, Some(target_position) if target_position < root_position)
                    }
                    // An unplanned root still uses the historical
                    // authored-order proof unless the exact target belongs
                    // to the validated pre-evaluation plan. The kernel
                    // completes every plan root before the ordinary document
                    // traversal, so a normal query may consume a View row
                    // even when that View was authored later. It still cannot
                    // rely on an arbitrary non-plan forward row.
                    None => {
                        target_plan.is_some()
                            || matches!(
                                (target_order, root_order),
                                (Some(target), Some(root)) if target < root
                            )
                    }
                };
                if target.source_id != source_id
                    || target.node_id == input.root.node_id
                    || !available
                {
                    return Err(TypeDiagnosticV2::UnavailableRelationRowDependency {
                        root: input.root.node_id,
                        target: target.node_id,
                        location,
                    });
                }
            }
        }
        let root = input.root;
        if rows.insert(root, input).is_some() {
            return Err(TypeDiagnosticV2::InvalidDeferredRelationRow {
                root: root.node_id,
                location,
            });
        }
    }
    for (root, requirement) in &required.relations {
        let requires_input = matches!(
            requirement.kind,
            RelationRequirementKindV2::Source
                | RelationRequirementKindV2::Project(_)
                | RelationRequirementKindV2::Group(_)
        );
        let key = SourceQualifiedNodeIdV2 {
            source_id,
            node_id: *root,
        };
        if requires_input && !rows.contains_key(&key) {
            return Err(TypeDiagnosticV2::InvalidDeferredRelationRow {
                root: *root,
                location: LocatedSpanV2 {
                    source_id,
                    span: requirement.span,
                },
            });
        }
    }
    Ok(rows)
}

#[derive(Default)]
struct RequiredInputs {
    nodes: BTreeMap<HirNodeIdV1, (quanta_gqlang_v2_hir::SpanV1, RequiredInputKind)>,
    context_nodes: BTreeMap<HirNodeIdV1, quanta_gqlang_v2_hir::SpanV1>,
    value_nodes: BTreeMap<HirNodeIdV1, quanta_gqlang_v2_hir::SpanV1>,
    declaration_value_nodes: BTreeSet<HirNodeIdV1>,
    relations: BTreeMap<HirNodeIdV1, RelationRequirementV2>,
    relation_order: BTreeMap<HirNodeIdV1, usize>,
    relation_nodes: BTreeMap<HirNodeIdV1, RelationExprHirV1>,
    relation_ancestors: BTreeMap<HirNodeIdV1, Vec<HirNodeIdV1>>,
    relation_stack: Vec<HirNodeIdV1>,
    relation_predicates: BTreeMap<HirNodeIdV1, RelationPredicateRequirementV2>,
    capture_owner_nodes: BTreeSet<HirNodeIdV1>,
}

#[derive(Clone)]
struct RelationRequirementV2 {
    span: quanta_gqlang_v2_hir::SpanV1,
    kind: RelationRequirementKindV2,
}

#[derive(Clone)]
enum RelationRequirementKindV2 {
    Source,
    Project(Vec<HirNodeIdV1>),
    Filter,
    Join,
    Group(Vec<HirNodeIdV1>),
    Sort,
}

#[derive(Clone)]
struct RelationPredicateRequirementV2 {
    relation_root: HirNodeIdV1,
    ordered_references: Vec<HirNodeIdV1>,
}
#[derive(Clone, Copy)]
enum RequiredInputKind {
    Binding,
    Callable,
    RelationPredicate,
}
impl RequiredInputs {
    fn pre_evaluation_scope(
        &self,
        node_id: HirNodeIdV1,
        positions: &BTreeMap<HirNodeIdV1, usize>,
    ) -> Option<usize> {
        positions.get(&node_id).copied().or_else(|| {
            self.relation_ancestors
                .get(&node_id)
                .into_iter()
                .flatten()
                .rev()
                .find_map(|ancestor| positions.get(ancestor).copied())
        })
    }
}
impl HirVisitorV1 for RequiredInputs {
    fn visit_item(&mut self, node: &quanta_gqlang_v2_hir::ProgramItemHirV1) {
        let _ = self.capture_owner_nodes.insert(node.id());
        quanta_gqlang_v2_hir::walk_item(self, node);
    }
    fn visit_query(&mut self, node: &quanta_gqlang_v2_hir::QueryHirV1) {
        let _ = self.capture_owner_nodes.insert(node.id);
        quanta_gqlang_v2_hir::walk_query(self, node);
    }
    fn visit_composition(&mut self, node: &quanta_gqlang_v2_hir::CompositionHirV1) {
        let _ = self.capture_owner_nodes.insert(node.id);
        quanta_gqlang_v2_hir::walk_composition(self, node);
    }
    fn visit_decl(&mut self, node: &quanta_gqlang_v2_hir::DeclHirV1) {
        let _ = self.capture_owner_nodes.insert(node.id);
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
    fn visit_rule(&mut self, node: &quanta_gqlang_v2_hir::RuleHirV1) {
        let _ = self.capture_owner_nodes.insert(node.id);
        quanta_gqlang_v2_hir::walk_rule(self, node);
    }
    fn visit_relation(&mut self, node: &RelationExprHirV1) {
        let _ = self.capture_owner_nodes.insert(node.id);
        let _ = self
            .relation_ancestors
            .insert(node.id, self.relation_stack.clone());
        let _ = self.relation_nodes.insert(node.id, node.clone());
        let kind = match &node.kind {
            RelationExprKindV1::Source(_) => RelationRequirementKindV2::Source,
            RelationExprKindV1::Project(project) => RelationRequirementKindV2::Project(
                project.items().iter().map(|item| item.value.id).collect(),
            ),
            RelationExprKindV1::Filter(_) => RelationRequirementKindV2::Filter,
            RelationExprKindV1::Join(_) => RelationRequirementKindV2::Join,
            RelationExprKindV1::GroupBy(group) => {
                RelationRequirementKindV2::Group(group.keys().iter().map(|key| key.id).collect())
            }
            RelationExprKindV1::OrderBy(_) => RelationRequirementKindV2::Sort,
        };
        let _ = self.relations.insert(
            node.id,
            RelationRequirementV2 {
                span: node.span,
                kind,
            },
        );
        self.relation_stack.push(node.id);
        quanta_gqlang_v2_hir::walk_relation(self, node);
        let popped = self.relation_stack.pop();
        debug_assert_eq!(popped, Some(node.id));
        let order = self.relation_order.len();
        let _ = self.relation_order.insert(node.id, order);
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
        if matches!(node.kind, BoolExprKindV1::LogicalQuantifier(_)) {
            let _ = self.capture_owner_nodes.insert(node.id);
        }
        if let BoolExprKindV1::RelationExists(exists) = &node.kind {
            let _ = self
                .nodes
                .insert(node.id, (node.span, RequiredInputKind::RelationPredicate));
            #[derive(Default)]
            struct OrderedCaptureReferencesV2(Vec<HirNodeIdV1>);
            impl HirVisitorV1 for OrderedCaptureReferencesV2 {
                fn visit_relation(&mut self, relation: &RelationExprHirV1) {
                    // Binder records an exact capture edge for a Composition
                    // relation-source use at the relation node itself. Keep
                    // that source reference in the same authored traversal as
                    // scalar symbols, while Sema remains responsible for
                    // proving which candidates are actually lexical captures.
                    if matches!(relation.kind, RelationExprKindV1::Source(_)) {
                        self.0.push(relation.id);
                    }
                    quanta_gqlang_v2_hir::walk_relation(self, relation);
                }
                fn visit_value(&mut self, value: &ValueExprHirV1) {
                    if matches!(value.kind, ValueExprKindV1::Symbol(_)) {
                        self.0.push(value.id);
                    }
                    quanta_gqlang_v2_hir::walk_value(self, value);
                }
            }
            let mut references = OrderedCaptureReferencesV2::default();
            references.visit_relation(&exists.relation);
            let _ = self.relation_predicates.insert(
                node.id,
                RelationPredicateRequirementV2 {
                    relation_root: exists.relation.id,
                    ordered_references: references.0,
                },
            );
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
    relation_rows: BTreeMap<SourceQualifiedNodeIdV2, RecordTypeV1>,
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
    /// Types-owned row for one exact checked relation stage.
    #[must_use]
    pub fn relation_row(&self, root: SourceQualifiedNodeIdV2) -> Option<&RecordTypeV1> {
        self.relation_rows.get(&root)
    }
    #[must_use]
    pub fn relation_row_count(&self) -> usize {
        self.relation_rows.len()
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
            relation_rows: BTreeMap::new(),
            visiting: BTreeSet::new(),
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
    visiting: BTreeSet<HirNodeIdV1>,
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
                self.error = Some(error);
            }
        }
    }
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
                let left_row = self.relation_rows.get(&self.key_id(left)).cloned().ok_or(
                    TypeDiagnosticV2::UnavailableRelationRowDependency {
                        root: node.id,
                        target: left,
                        location,
                    },
                )?;
                let right_row = self.relation_rows.get(&self.key_id(right)).cloned().ok_or(
                    TypeDiagnosticV2::UnavailableRelationRowDependency {
                        root: node.id,
                        target: right,
                        location,
                    },
                )?;
                let fields = left_row
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
                    .map(|(field, field_location)| {
                        RecordFieldTypeV1::try_new(
                            field.name(),
                            field.value_type().clone(),
                            field_location.clone(),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut locations = left_row.locations;
                locations.extend(right_row.locations);
                CheckedRelationRowV2 {
                    record: RecordTypeV1::try_new(fields)?,
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
        let mut typed_fields = Vec::with_capacity(fields.len());
        for field in fields {
            let value_type = match &field.source {
                RelationRowFieldSourceV2::Known(value_type) => value_type.clone(),
                RelationRowFieldSourceV2::PriorExpression(target) => {
                    match self.facts.get(target).map(|fact| &fact.state) {
                        Some(ExpressionTypeStateV2::Known(value_type)) => value_type.clone(),
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
            typed_fields.push(RecordFieldTypeV1::try_new(
                &field.name,
                value_type,
                field.name_location.clone(),
            )?);
        }
        Ok(CheckedRelationRowV2 {
            record: RecordTypeV1::try_new(typed_fields)?,
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
                Some(ResolvedTypeInputValueV2::BindingFromRelationRow(target)) => {
                    let row = self.relation_rows.get(target).ok_or(
                        TypeDiagnosticV2::UnavailableRelationRowDependency {
                            root: node.id,
                            target: target.node_id,
                            location: self.location(node),
                        },
                    )?;
                    ExpressionTypeStateV2::Known(ValueTypeV1::try_record(row.record.clone())?)
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
