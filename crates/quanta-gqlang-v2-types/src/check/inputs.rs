//! Checked type-input vocabulary and source-bound input validation.

use super::{
    BTreeMap, BTreeSet, BoolExprHirV1, BoolExprKindV1, BuiltinFamilyIdV2, BuiltinIdV2,
    CheckedHirDocumentV2, CheckedLocatedSpanV2, ExpressionEffectsV2, ExtendedSourceRegistryViewV2,
    HirNodeIdV1, HirVisitorV1, LocatedSpanV2, RelationExprHirV1, RelationExprKindV1, SourceIdV2,
    TypeDiagnosticV2, ValueExprHirV1, ValueExprKindV1, ValueTypeV1,
};

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
    pub(super) ordinal: usize,
    pub(super) name: String,
    pub(super) name_location: CheckedLocatedSpanV2,
    pub(super) source: RelationRowFieldSourceV2,
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
    pub(super) kind: DeferredRelationRowKindV2,
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
    pub(super) id: ResolvedPredicateIdV2,
    pub(super) params: Vec<ValueTypeV1>,
    pub(super) result: ValueTypeV1,
    pub(super) effects: ExpressionEffectsV2,
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
    pub(super) key: ResolvedHostPredicateKeyV2,
    pub(super) params: Vec<ValueTypeV1>,
    pub(super) result: ValueTypeV1,
    pub(super) effects: ExpressionEffectsV2,
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
    pub(super) kind: ResolvedRelationPredicateKindV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ResolvedRelationPredicateKindV2 {
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
    pub(super) checked: CheckedHirDocumentV2<'a>,
    pub(super) values: BTreeMap<SourceQualifiedNodeIdV2, ResolvedTypeInputValueV2>,
    pub(super) contexts: BTreeMap<SourceQualifiedNodeIdV2, (ValueTypeV1, ContextualTypeReasonV2)>,
    pub(super) context_locations: BTreeMap<SourceQualifiedNodeIdV2, quanta_gqlang_v2_hir::SpanV1>,
    pub(super) source_registry: ExtendedSourceRegistryViewV2,
    pub(super) declaration_bodies: BTreeMap<HirNodeIdV1, &'a ValueExprHirV1>,
    pub(super) relation_rows: BTreeMap<SourceQualifiedNodeIdV2, DeferredRelationRowInputV2>,
    pub(super) pre_evaluation_roots: Vec<RelationExprHirV1>,
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
                | quanta_gqlang_v2_hir::ProgramItemHirV1::Meta(_)
                | quanta_gqlang_v2_hir::ProgramItemHirV1::Command(_) => None,
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
                    | RelationRequirementKindV2::Sort
                    | RelationRequirementKindV2::Limit => unreachable!("kind checked above"),
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
    Limit,
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
            RelationExprKindV1::Limit(_) => RelationRequirementKindV2::Limit,
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
