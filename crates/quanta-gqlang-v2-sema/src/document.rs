//! Atomic document semantic transaction and exact item-key assembly.

use crate::authority::{
    ClosedFrontendAuthorityV2, DownstreamBudgetReceiptV2, FrontendAuthorityErrorV2,
    FrontendBudgetV2, GraphReferenceCatalogV2,
};
use crate::diagnostic::{
    location, FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2,
};
use crate::rule::{classify_rule_body_calls, RuleBodyClassificationErrorV2};
use crate::{
    AggregateContextV2, AggregatePositionV2, CheckedLocatedSpanV2, GraphDirectionV2,
    GraphReferenceRoleV2, GraphWitnessBoundsV2, NullOrderV2, OrderKeyContractV2,
    OutputColumnContractV2, OutputColumnOriginV2, OutputProvenanceV2, PartitionInputBindingV2,
    RelationOutputContractV2, ResolvedBindingKindV2, ResolvedCatalogItemV2,
    ResolvedCatalogPayloadV2, ResolvedCatalogReferenceV2, ResolvedCheckV2,
    ResolvedCompositionBindingV2, ResolvedCompositionBodyV2, ResolvedCompositionProducerV2,
    ResolvedCompositionSourceIdV2, ResolvedCompositionV2, ResolvedDeclarationV2,
    ResolvedExplainTargetV2, ResolvedGraphQueryV2, ResolvedGraphReferenceV2, ResolvedGraphTargetV2,
    ResolvedGraphUseV2, ResolvedMetaKindV2, ResolvedMetaV2, ResolvedPresentationKindV2,
    ResolvedPresentationV2, ResolvedRuleCallV2, ResolvedRuleV2, ResolvedScanSourceV2,
    ResolvedValueBindingV2, SemanticItemIdV2, SemanticItemV2, TypedBoolExprV2, TypedCallArgumentV2,
    TypedCorrelationEdgeV2, TypedExprShapeV2, TypedExprV2, TypedPathSegmentV2, TypedRecordFieldV2,
    TypedRelationExprV2, TypedRelationV2,
};
use quanta_gqlang_v2_binder::{
    BindResultV1, Binder, BinderKindV1, ModuleGraphViewV1, ParamV1, ResolvedTargetV1,
};
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CheckedHirDocumentOwnedV2, ExtendedSourceRegistryViewV2,
    HirDocumentV1, HirNodeIdV1, HirVisitorV1, ProgramItemHirV1, SourceIdV2,
    SourceRegistryIdentityV2, SpanV1, ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_modules::{
    CatalogAssetIdV2, CatalogAssetRefInputV2, DocumentCatalogAssetInputV2,
    DocumentCatalogAuthorityV2, DocumentCatalogDependencyInputV2,
    DocumentCatalogFamilyReferenceInputV2, DocumentCatalogFinalInputV2,
    DocumentCatalogReferenceFamilyV2, DocumentCatalogReferenceInputV2,
    DocumentCatalogReferenceOriginV2, DocumentCatalogValidatorV2, DocumentViewTopologyProofV2,
    ExternalCatalogPayloadV2, LockReferenceInputV2, NamespaceKindV2, PackActivationInputV2,
    PackIdV2, PredeclaredDocumentAssetV2, ProfileAdmissionInputV2, ProfileAdmissionV2,
    RequireCertifiedInputV2, ResolvedGraphV2, UsePackActivationV2,
};
use quanta_gqlang_v2_types::{
    AggregationClassV1, BuiltinRegistryV2, DeferredRelationRowFieldInputV2,
    DeferredRelationRowInputV2, ExpressionEffectsV2, ExpressionTypeKernelV2, ExpressionTypeStateV2,
    RelationPreEvaluationInputV2, RelationRowFieldSourceV2, RelationTypeV1, ResolvedCallTargetV2,
    ResolvedCallableInputV2, ResolvedPredicateIdV2, ResolvedRelationCaptureInputV2,
    ResolvedRelationPredicateInputV2, ResolvedTypeInputEntryV2, ResolvedTypeInputValueV2,
    ResolvedTypeInputsV2, RuleBindingIdV2, RuleCallInputV2, RuleCallableTargetV2,
    RulePredicateInputV2, RuleSymbolInputV2, RuleTypeBudgetV2, RuleTypeSolverV2, ScalarTyV1,
    SourceQualifiedNodeIdV2, TypeDiagnosticV2, TypedExpressionFactsV2, ValueTypeV1,
};
use std::collections::{BTreeMap, BTreeSet};

/// Owned proof that all HIR roots share one valid document-wide identity and
/// source-length bound. Fields are private; the HIR owner checks exactly once.
#[derive(Clone, Debug)]
pub struct CheckedSyntaxDocumentV2 {
    checked: CheckedHirDocumentOwnedV2,
}

impl CheckedSyntaxDocumentV2 {
    /// Consume raw HIR and invoke the sole HIR document integrity checker.
    pub fn try_from_hir(document: HirDocumentV1) -> Result<Self, FrontendFailureV2> {
        let source_id = document.source_id;
        let checked = CheckedHirDocumentOwnedV2::try_new(document).map_err(|error| {
            let (span, secondary) = match &error {
                quanta_gqlang_v2_hir::HirIntegrityErrorV1::DuplicateNodeId {
                    first_span,
                    second_span,
                    ..
                } => (*second_span, vec![location(source_id, *first_span)]),
                quanta_gqlang_v2_hir::HirIntegrityErrorV1::InvertedSpan { span, .. }
                | quanta_gqlang_v2_hir::HirIntegrityErrorV1::SpanOutOfBounds { span, .. }
                | quanta_gqlang_v2_hir::HirIntegrityErrorV1::InvertedAuxiliarySpan { span }
                | quanta_gqlang_v2_hir::HirIntegrityErrorV1::AuxiliarySpanOutOfBounds {
                    span,
                    ..
                }
                | quanta_gqlang_v2_hir::HirIntegrityErrorV1::ExcessiveDepth { span, .. } => {
                    (*span, Vec::new())
                }
            };
            FrontendFailureV2::one(FrontendDiagnosticV2::new(
                if matches!(
                    error,
                    quanta_gqlang_v2_hir::HirIntegrityErrorV1::ExcessiveDepth { .. }
                ) {
                    FrontendStageV2::Resource
                } else {
                    FrontendStageV2::Integrity
                },
                error.code(),
                FrontendSeverityV2::Error,
                location(source_id, span),
                secondary,
                error.to_string(),
            ))
        })?;
        Ok(Self { checked })
    }

    /// Checked source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.checked.document().source_id
    }
    /// Sole source-length authority.
    #[must_use]
    pub const fn source_len(&self) -> usize {
        self.checked.document().source_len
    }
    /// Document-wide count of checked HIR nodes.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.checked.node_count()
    }
    /// Borrow the integrity-proven document for owner kernels.
    #[must_use]
    pub const fn checked(&self) -> &CheckedHirDocumentOwnedV2 {
        &self.checked
    }
}

/// Checked declarations collected before any body resolution.
#[derive(Clone, Debug)]
pub struct DeclaredDocumentV2 {
    checked: CheckedSyntaxDocumentV2,
    source_anchor: CheckedLocatedSpanV2,
    expected: BTreeMap<SemanticItemIdV2, (SpanV1, SemanticItemKindV2)>,
    order: Vec<SemanticItemIdV2>,
    expected_expressions: BTreeSet<SourceQualifiedNodeIdV2>,
    registry_identity: SourceRegistryIdentityV2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SemanticItemKindV2 {
    Relation,
    Declaration,
    Rule,
    Graph,
    Check,
    Presentation,
    Catalog,
    Composition,
    Meta,
}

impl SemanticItemKindV2 {
    const fn from_hir(item: &ProgramItemHirV1) -> Self {
        match item {
            ProgramItemHirV1::Query(_) => Self::Relation,
            ProgramItemHirV1::Decl(_) => Self::Declaration,
            ProgramItemHirV1::Rule(_) => Self::Rule,
            ProgramItemHirV1::GraphQuery(_) => Self::Graph,
            ProgramItemHirV1::Legality(_) => Self::Check,
            ProgramItemHirV1::Presentation(_) => Self::Presentation,
            ProgramItemHirV1::CatalogDecl(_) => Self::Catalog,
            ProgramItemHirV1::Composition(_) => Self::Composition,
            ProgramItemHirV1::Meta(_) => Self::Meta,
        }
    }

    const fn from_semantic(item: &SemanticItemV2) -> Self {
        match item {
            SemanticItemV2::Relation(_) => Self::Relation,
            SemanticItemV2::Declaration(_) => Self::Declaration,
            SemanticItemV2::Rule(_) => Self::Rule,
            SemanticItemV2::Graph(_) => Self::Graph,
            SemanticItemV2::Check(_) => Self::Check,
            SemanticItemV2::Presentation(_) => Self::Presentation,
            SemanticItemV2::Catalog(_) => Self::Catalog,
            SemanticItemV2::Composition(_) => Self::Composition,
            SemanticItemV2::Meta(_) => Self::Meta,
        }
    }
}

impl DeclaredDocumentV2 {
    /// Predeclare all authored items, preserving original order while
    /// rejecting duplicate semantic identities before any semantic output.
    pub fn try_new(
        checked: CheckedSyntaxDocumentV2,
        budget: &FrontendBudgetV2,
        extended: &ExtendedSourceRegistryViewV2,
    ) -> Result<Self, FrontendFailureV2> {
        let source_id = checked.source_id();
        if extended.source_len(source_id) != Some(checked.source_len()) {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "checked syntax source differs from the extended authority registry",
            ));
        }
        if checked.checked.document().items.len()
            > usize::try_from(budget.max_document_items()).unwrap_or(usize::MAX)
        {
            return Err(document_failure(
                "frontend.resource_limit_exceeded",
                source_id,
                SpanV1::point(0),
                "document_items exceeds frontend limit",
            ));
        }
        check_relation_depth(&checked, budget)?;
        let mut expected = BTreeMap::new();
        let mut order = Vec::new();
        #[derive(Default)]
        struct ExpressionNodesV2(BTreeSet<HirNodeIdV1>);
        impl HirVisitorV1 for ExpressionNodesV2 {
            fn visit_value(&mut self, value: &ValueExprHirV1) {
                let _ = self.0.insert(value.id);
                quanta_gqlang_v2_hir::walk_value(self, value);
            }
            fn visit_bool(&mut self, value: &quanta_gqlang_v2_hir::BoolExprHirV1) {
                let _ = self.0.insert(value.id);
                quanta_gqlang_v2_hir::walk_bool(self, value);
            }
        }
        let mut expression_nodes = ExpressionNodesV2::default();
        for item in &checked.checked.document().items {
            let id = SemanticItemIdV2::new(source_id, item.id());
            if expected
                .insert(id, (item.span(), SemanticItemKindV2::from_hir(item)))
                .is_some()
            {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    item.span(),
                    "duplicate semantic item identity",
                ));
            }
            order.push(id);
            expression_nodes.visit_item(item);
        }
        let expected_expressions = expression_nodes
            .0
            .into_iter()
            .map(|node_id| SourceQualifiedNodeIdV2 { source_id, node_id })
            .collect();
        let source_anchor = extended
            .combined()
            .try_check_location(location(source_id, SpanV1::point(0)))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    SpanV1::point(0),
                    error.to_string(),
                )
            })?;
        Ok(Self {
            checked,
            source_anchor,
            expected,
            order,
            expected_expressions,
            registry_identity: extended.identity(),
        })
    }

    /// Checked syntax document.
    #[must_use]
    pub const fn checked(&self) -> &CheckedSyntaxDocumentV2 {
        &self.checked
    }
    /// Authored item order.
    #[must_use]
    pub fn order(&self) -> &[SemanticItemIdV2] {
        &self.order
    }
    /// Expected item IDs and original spans.
    pub fn expected(&self) -> impl Iterator<Item = (SemanticItemIdV2, SpanV1)> + '_ {
        self.expected.iter().map(|(id, (span, _))| (*id, *span))
    }
}

/// One unresolved editor-only fact. It cannot enter a closed document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedSemanticFactV2 {
    code: &'static str,
    location: quanta_gqlang_v2_hir::LocatedSpanV2,
}

impl UnresolvedSemanticFactV2 {
    /// Construct one explicit unresolved fact.
    #[must_use]
    pub const fn new(code: &'static str, location: quanta_gqlang_v2_hir::LocatedSpanV2) -> Self {
        Self { code, location }
    }
    /// Stable unresolved code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }
    /// Source location.
    #[must_use]
    pub const fn location(&self) -> quanta_gqlang_v2_hir::LocatedSpanV2 {
        self.location
    }
}

/// Incomplete editor analysis. There is no conversion to closed semantics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IncompleteSemanticDocumentV2 {
    source_id: SourceIdV2,
    unresolved: Vec<UnresolvedSemanticFactV2>,
    diagnostics: Vec<FrontendDiagnosticV2>,
}

impl IncompleteSemanticDocumentV2 {
    /// Seal an incomplete state only if at least one unresolved fact remains.
    pub fn try_new(
        source_id: SourceIdV2,
        mut unresolved: Vec<UnresolvedSemanticFactV2>,
        mut diagnostics: Vec<FrontendDiagnosticV2>,
    ) -> Result<Self, &'static str> {
        if unresolved.is_empty() {
            return Err("incomplete analysis requires an unresolved fact");
        }
        if unresolved
            .iter()
            .any(|fact| fact.code.is_empty() || fact.location.source_id != source_id)
        {
            return Err("unresolved analysis fact has an empty code or foreign source");
        }
        unresolved.sort_by_key(|fact| {
            (
                fact.location.source_id.raw(),
                fact.location.span.start,
                fact.location.span.end,
                fact.code,
            )
        });
        if unresolved.windows(2).any(|pair| {
            let [left, right] = pair else {
                return false;
            };
            left == right
        }) {
            return Err("duplicate unresolved analysis fact");
        }
        if diagnostics.iter().any(|diagnostic| {
            diagnostic.primary().source_id != source_id
                || diagnostic.severity() == FrontendSeverityV2::Error
        }) {
            return Err("incomplete analysis diagnostic is foreign or erroneous");
        }
        diagnostics.sort_by(|left, right| left.sort_key().cmp(&right.sort_key()));
        if diagnostics.windows(2).any(|pair| {
            let [left, right] = pair else {
                return false;
            };
            left == right
        }) {
            return Err("duplicate incomplete analysis diagnostic");
        }
        Ok(Self {
            source_id,
            unresolved,
            diagnostics,
        })
    }
    /// Source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.source_id
    }
    /// Exact unresolved facts.
    #[must_use]
    pub fn unresolved(&self) -> &[UnresolvedSemanticFactV2] {
        &self.unresolved
    }
    /// Ordered analysis diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[FrontendDiagnosticV2] {
        &self.diagnostics
    }
}

/// Open editor semantic result. Only `Complete` has a closed semantic document;
/// no incomplete analysis implicitly promotes to a compiled result.
#[derive(Clone, Debug)]
pub enum SemanticAnalysisV2 {
    /// Fully resolved analysis result (still analysis, not compiled output).
    Complete(SemanticDocumentV2),
    /// Explicitly unresolved but non-erroneous analysis.
    Incomplete(IncompleteSemanticDocumentV2),
    /// Hard semantic error.
    Error(FrontendTransactionErrorV2),
}

/// Semantic entrypoint failure. Source-less authority errors cannot be
/// mislabeled as document diagnostics with fabricated source coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrontendTransactionErrorV2 {
    /// Host/configuration/authority session failure with no source anchor.
    Authority(FrontendAuthorityErrorV2),
    /// At least one genuine source-anchored diagnostic.
    Document(FrontendFailureV2),
}

impl From<FrontendFailureV2> for FrontendTransactionErrorV2 {
    fn from(failure: FrontendFailureV2) -> Self {
        Self::Document(failure)
    }
}

impl From<FrontendAuthorityErrorV2> for FrontendTransactionErrorV2 {
    fn from(error: FrontendAuthorityErrorV2) -> Self {
        Self::Authority(error)
    }
}

/// Atomic semantic document. Construction requires an exact item-ID set, the
/// original source order, and a once-validated document catalog proof.
#[derive(Clone, Debug)]
pub struct SemanticDocumentV2 {
    source_id: SourceIdV2,
    source_anchor: CheckedLocatedSpanV2,
    order: Vec<SemanticItemIdV2>,
    items: BTreeMap<SemanticItemIdV2, SemanticItemV2>,
    expressions: BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    catalog: DocumentCatalogAuthorityV2,
    budget_receipt: DownstreamBudgetReceiptV2,
}

fn graph_target_is_admitted(
    target: &ResolvedGraphTargetV2,
    authority: &GraphReferenceCatalogV2,
) -> bool {
    match target {
        ResolvedGraphTargetV2::Host(target) => authority.admits(target),
        // Composition targets are sealed from a prior Binder-derived local
        // producer, not minted by the host graph catalog. Their producer
        // membership is checked by the Composition semantic carrier.
        ResolvedGraphTargetV2::Composition(_) => true,
    }
}

impl SemanticDocumentV2 {
    /// Test-only minting seam: mutate the item map and document order of an
    /// already-sealed document so adversarial-document rails can forge
    /// membership mismatches (dropped item, orphan item, duplicate order id)
    /// against the real Logic validator. Compiled for unit tests or under
    /// the `test` feature; never part of a normal build.
    #[cfg(any(test, feature = "test"))]
    #[doc(hidden)]
    pub fn forged_membership<F>(mut self, mutate: F) -> Self
    where
        F: FnOnce(&mut Vec<SemanticItemIdV2>, &mut BTreeMap<SemanticItemIdV2, SemanticItemV2>),
    {
        mutate(&mut self.order, &mut self.items);
        self
    }

    /// Test-only minting seam for high-level forged expression-tree fixtures.
    /// The seam stays crate-private; downstream tests receive only a complete
    /// fixture document from `test_support`.
    #[cfg(any(test, feature = "test"))]
    pub(crate) fn forged_expressions<F>(mut self, mutate: F) -> Self
    where
        F: FnOnce(&mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>),
    {
        mutate(&mut self.expressions);
        self
    }

    /// Assemble exact-key semantic outputs; no positional zip or partial
    /// success path is available.
    pub(crate) fn try_assemble(
        declared: &DeclaredDocumentV2,
        produced: Vec<SemanticItemV2>,
        expressions: BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
        nested_relations: BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
        type_fact_count: usize,
        catalog: DocumentCatalogAuthorityV2,
        graph_authority: &GraphReferenceCatalogV2,
        budget_receipt: DownstreamBudgetReceiptV2,
    ) -> Result<Self, FrontendFailureV2> {
        let source_id = declared.checked.source_id();
        if catalog.source_registry_identity() != &declared.registry_identity {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "catalog proof belongs to another document authority registry",
            ));
        }
        let expected: BTreeSet<_> = declared.expected.keys().copied().collect();
        let mut items = BTreeMap::new();
        for item in produced {
            let id = item.item_id();
            if id.source_id() != source_id || !expected.contains(&id) {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    SpanV1::point(0),
                    "foreign or extra semantic item identity",
                ));
            }
            if declared
                .expected
                .get(&id)
                .is_some_and(|(_, kind)| *kind != SemanticItemKindV2::from_semantic(&item))
            {
                let span = declared
                    .expected
                    .get(&id)
                    .map_or(SpanV1::point(0), |(span, _)| *span);
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    span,
                    "semantic item kind differs from checked syntax",
                ));
            }
            if items.insert(id, item).is_some() {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    SpanV1::point(0),
                    "duplicate semantic item identity",
                ));
            }
        }
        if items.keys().copied().collect::<BTreeSet<_>>() != expected {
            let missing = declared
                .expected
                .iter()
                .find(|(id, _)| !items.contains_key(id))
                .map_or(SpanV1::point(0), |(_, (span, _))| *span);
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                missing,
                "missing semantic item identity",
            ));
        }
        let semantic_assets = items
            .values()
            .filter_map(|item| match item {
                SemanticItemV2::Catalog(catalog) => Some((
                    catalog.asset_id().clone(),
                    catalog.asset_id().namespace_kind(),
                )),
                SemanticItemV2::Relation(_)
                | SemanticItemV2::Rule(_)
                | SemanticItemV2::Graph(_)
                | SemanticItemV2::Check(_)
                | SemanticItemV2::Presentation(_)
                | SemanticItemV2::Declaration(_)
                | SemanticItemV2::Composition(_)
                | SemanticItemV2::Meta(_) => None,
            })
            .collect::<BTreeSet<_>>();
        let validated_assets = catalog
            .assets()
            .map(|(id, kind)| (id.clone(), kind))
            .collect::<BTreeSet<_>>();
        if semantic_assets != validated_assets {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "semantic catalog assets differ from modules-validated document assets",
            ));
        }
        let semantic_profiles = items
            .values()
            .filter_map(|item| {
                let SemanticItemV2::Catalog(catalog) = item else {
                    return None;
                };
                let ResolvedCatalogPayloadV2::Profile { admission, .. } = catalog.payload() else {
                    return None;
                };
                let admission = match admission {
                    quanta_gqlang_v2_hir::AdmissionV1::Allowlist => ProfileAdmissionV2::Allowlist,
                    quanta_gqlang_v2_hir::AdmissionV1::ReviewRequired => {
                        ProfileAdmissionV2::ReviewRequired
                    }
                    quanta_gqlang_v2_hir::AdmissionV1::CertifiedOnly => {
                        ProfileAdmissionV2::CertifiedOnly
                    }
                };
                Some((catalog.asset_id().clone(), admission))
            })
            .collect::<BTreeSet<_>>();
        let validated_profiles = catalog
            .profiles()
            .map(|(id, admission)| (id.clone(), admission))
            .collect::<BTreeSet<_>>();
        if semantic_profiles != validated_profiles {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "semantic profile admissions differ from modules-validated policy",
            ));
        }
        let certified = items
            .values()
            .filter_map(|item| match item {
                SemanticItemV2::Meta(meta)
                    if matches!(meta.kind(), ResolvedMetaKindV2::RequireCertified) =>
                {
                    Some(meta.location().location())
                }
                SemanticItemV2::Relation(_)
                | SemanticItemV2::Rule(_)
                | SemanticItemV2::Graph(_)
                | SemanticItemV2::Check(_)
                | SemanticItemV2::Presentation(_)
                | SemanticItemV2::Catalog(_)
                | SemanticItemV2::Declaration(_)
                | SemanticItemV2::Composition(_)
                | SemanticItemV2::Meta(_) => None,
            })
            .collect::<Vec<_>>();
        if certified.len() > 1 || certified.first().copied() != catalog.require_certified_location()
        {
            return Err(document_failure("sema.document_validation_failed", source_id,
                certified.first().map_or(SpanV1::point(0), |location| location.span),
                "semantic certification directive differs from modules-validated exact location/count"));
        }
        // Modules seals one dependency per (source, target), retaining the
        // earliest checked location for repeated identical references. Mirror
        // that exact canonicalization before comparing the semantic payload.
        let mut semantic_edges: BTreeMap<
            (CatalogAssetIdV2, CatalogAssetIdV2),
            (
                NamespaceKindV2,
                Option<DocumentCatalogReferenceFamilyV2>,
                quanta_gqlang_v2_hir::LocatedSpanV2,
            ),
        > = BTreeMap::new();
        let mut semantic_conflict = false;
        for item in items.values() {
            let SemanticItemV2::Catalog(catalog) = item else {
                continue;
            };
            let mut add = |reference: &ResolvedCatalogReferenceV2, family| {
                let key = (catalog.asset_id().clone(), reference.id().clone());
                let candidate = (reference.kind(), family, reference.location().location());
                if let Some(existing) = semantic_edges.get_mut(&key) {
                    if existing.0 != candidate.0 || existing.1 != candidate.1 {
                        semantic_conflict = true;
                    } else {
                        let candidate_location = candidate.2;
                        let existing_location = existing.2;
                        if (
                            candidate_location.source_id.raw(),
                            candidate_location.span.start,
                            candidate_location.span.end,
                        ) < (
                            existing_location.source_id.raw(),
                            existing_location.span.start,
                            existing_location.span.end,
                        ) {
                            *existing = candidate;
                        }
                    }
                } else {
                    semantic_edges.insert(key, candidate);
                }
            };
            match catalog.payload() {
                ResolvedCatalogPayloadV2::View { query_ref } => {
                    if let Some(reference) = query_ref {
                        add(reference, Some(DocumentCatalogReferenceFamilyV2::ViewQuery));
                    }
                    for reference in catalog.inline_view_dependencies() {
                        add(reference, None);
                    }
                }
                ResolvedCatalogPayloadV2::Model { source } => {
                    add(source, Some(DocumentCatalogReferenceFamilyV2::ModelSource))
                }
                ResolvedCatalogPayloadV2::Pack { source, .. } => {
                    add(source, Some(DocumentCatalogReferenceFamilyV2::PackSource))
                }
                ResolvedCatalogPayloadV2::Profile { capabilities, .. } => {
                    for reference in capabilities {
                        add(
                            reference,
                            Some(DocumentCatalogReferenceFamilyV2::ProfileCapability),
                        );
                    }
                }
                ResolvedCatalogPayloadV2::Gate { checks, .. } => {
                    for reference in checks {
                        add(reference, Some(DocumentCatalogReferenceFamilyV2::GateCheck));
                    }
                }
                ResolvedCatalogPayloadV2::RuleBinding { check_ref } => add(
                    check_ref,
                    Some(DocumentCatalogReferenceFamilyV2::RuleBindingCheck),
                ),
            }
        }
        let validated_edges = catalog
            .dependencies()
            .iter()
            .map(|edge| {
                (
                    (
                        edge.source_asset_id().clone(),
                        edge.target_asset_id().clone(),
                    ),
                    (
                        edge.expected_target_kind(),
                        edge.family(),
                        edge.location().location(),
                    ),
                )
            })
            .collect::<BTreeMap<_, _>>();
        if semantic_conflict
            || validated_edges.len() != catalog.dependencies().len()
            || semantic_edges != validated_edges
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "semantic catalog references differ from modules-validated dependency graph",
            ));
        }
        if expressions
            .iter()
            .any(|(key, fact)| key.source_id != source_id || *key != fact.key())
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "foreign or mismatched typed expression identity",
            ));
        }
        if type_fact_count != declared.expected_expressions.len()
            || expressions.keys().copied().collect::<BTreeSet<_>>() != declared.expected_expressions
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "typed expression set differs from checked HIR and types owner facts",
            ));
        }
        for item in items.values() {
            if let SemanticItemV2::Declaration(decl) = item {
                if expressions.get(&decl.body().key()) != Some(decl.body()) {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        SpanV1::point(0),
                        "declaration body does not match document expression proof",
                    ));
                }
            }
            if let SemanticItemV2::Rule(rule) = item {
                if expressions.get(&rule.head_term().key()) != Some(rule.head_term())
                    || expressions.get(&rule.body().expression().key())
                        != Some(rule.body().expression())
                {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        SpanV1::point(0),
                        "rule head/body disagree with the document expression proof",
                    ));
                }
            }
            if let SemanticItemV2::Graph(graph) = item {
                for reference in graph.references() {
                    if !graph_target_is_admitted(reference.target(), graph_authority)
                        || reference.use_location().registry_identity()
                            != &declared.registry_identity
                        || reference.use_location().location().source_id != source_id
                    {
                        return Err(document_failure("sema.document_validation_failed", source_id, reference.use_location().location().span, "graph reference does not belong to this document and sealed graph authority"));
                    }
                }
            }
            if let SemanticItemV2::Presentation(presentation) = item {
                let mut uses = Vec::new();
                match presentation.kind() {
                    ResolvedPresentationKindV2::Explain {
                        target: ResolvedExplainTargetV2::Entity(graph),
                        ..
                    }
                    | ResolvedPresentationKindV2::Describe { target: graph } => uses.push(graph),
                    ResolvedPresentationKindV2::Sample { scope, .. } => uses.extend(scope.as_ref()),
                    ResolvedPresentationKindV2::Compare {
                        baseline,
                        candidate,
                        ..
                    } => {
                        uses.push(baseline);
                        uses.push(candidate);
                    }
                    ResolvedPresentationKindV2::Export { source, .. } => uses.push(source),
                    ResolvedPresentationKindV2::Explain {
                        target: ResolvedExplainTargetV2::Rule(_),
                        ..
                    }
                    | ResolvedPresentationKindV2::Show { .. } => {}
                }
                for reference in uses {
                    if !graph_target_is_admitted(reference.target(), graph_authority)
                        || reference.location().registry_identity() != &declared.registry_identity
                        || reference.location().location().source_id != source_id
                    {
                        return Err(document_failure("sema.document_validation_failed", source_id, reference.location().location().span, "presentation selector differs from the sealed graph/document authority"));
                    }
                }
            }
            if let SemanticItemV2::Check(check) = item {
                let uses = check.scope().into_iter().chain(check.pattern());
                for reference in uses {
                    if !graph_target_is_admitted(reference.target(), graph_authority)
                        || reference.location().registry_identity() != &declared.registry_identity
                        || reference.location().location().source_id != source_id
                    {
                        return Err(document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            reference.location().location().span,
                            "CHECK selector differs from sealed graph/document authority",
                        ));
                    }
                }
                if let Some(nested) = check.nested_graph() {
                    for reference in nested.references() {
                        if !graph_target_is_admitted(reference.target(), graph_authority)
                            || reference.use_location().registry_identity()
                                != &declared.registry_identity
                            || reference.use_location().location().source_id != source_id
                        {
                            return Err(document_failure("sema.document_validation_failed", source_id, reference.use_location().location().span, "nested CHECK/FIND selector differs from sealed graph/document authority"));
                        }
                    }
                }
            }
        }
        check_nested_relation_finalizer(
            source_id,
            &declared.registry_identity,
            &expressions,
            &nested_relations,
        )
        .map_err(|message| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                message,
            )
        })?;
        Ok(Self {
            source_id,
            order: declared.order.clone(),
            source_anchor: declared.source_anchor.clone(),
            items,
            expressions,
            nested_relations,
            catalog,
            budget_receipt,
        })
    }

    /// Source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.source_id
    }
    /// Original item order.
    #[must_use]
    pub fn order(&self) -> &[SemanticItemIdV2] {
        &self.order
    }
    /// Byte-zero diagnostic anchor checked against this transaction's original
    /// extended source registry, retained even when the document has no items.
    #[must_use]
    pub const fn source_anchor(&self) -> &CheckedLocatedSpanV2 {
        &self.source_anchor
    }
    /// Exact semantic item by ID.
    #[must_use]
    pub fn item(&self, id: SemanticItemIdV2) -> Option<&SemanticItemV2> {
        self.items.get(&id)
    }
    /// All semantic items by canonical identity.
    pub fn items(&self) -> impl Iterator<Item = (&SemanticItemIdV2, &SemanticItemV2)> {
        self.items.iter()
    }
    /// Complete source-rule set from the same sealed document transaction.
    pub fn rules(&self) -> impl Iterator<Item = &ResolvedRuleV2> {
        self.items.values().filter_map(|item| match item {
            SemanticItemV2::Rule(rule) => Some(rule),
            SemanticItemV2::Relation(_)
            | SemanticItemV2::Graph(_)
            | SemanticItemV2::Check(_)
            | SemanticItemV2::Presentation(_)
            | SemanticItemV2::Catalog(_)
            | SemanticItemV2::Declaration(_)
            | SemanticItemV2::Composition(_)
            | SemanticItemV2::Meta(_) => None,
        })
    }
    /// Closed typed expression by exact source-qualified node ID.
    #[must_use]
    pub fn expression(&self, key: SourceQualifiedNodeIdV2) -> Option<&TypedExprV2> {
        self.expressions.get(&key)
    }
    /// Every typed expression fact, including child nodes, in stable key order.
    pub fn expressions(&self) -> impl Iterator<Item = (&SourceQualifiedNodeIdV2, &TypedExprV2)> {
        self.expressions.iter()
    }
    /// Exact typed nested relation owned by one relational EXISTS root.
    #[must_use]
    pub fn nested_relation(&self, root: SourceQualifiedNodeIdV2) -> Option<&TypedRelationExprV2> {
        self.nested_relations.get(&root)
    }
    /// Every nested relation in stable root-identity order.
    pub fn nested_relations(
        &self,
    ) -> impl Iterator<Item = (&SourceQualifiedNodeIdV2, &TypedRelationExprV2)> {
        self.nested_relations.iter()
    }
    /// Independently classify this document's typed rule-body source calls.
    /// Logic uses the same shape proof as the semantic producer.
    pub fn classify_rule_body_calls(
        &self,
        root: SourceQualifiedNodeIdV2,
    ) -> Result<Vec<ResolvedRuleCallV2>, RuleBodyClassificationErrorV2> {
        classify_rule_body_calls(
            root,
            |key| self.expression(key),
            |key| self.nested_relation(key),
        )
    }
    /// Document catalog proof.
    #[must_use]
    pub const fn catalog(&self) -> &DocumentCatalogAuthorityV2 {
        &self.catalog
    }
    /// Immutable downstream budget receipt projected once from the closed
    /// authority's `FrontendBudgetV2`. Downstream stages read every traversal
    /// and output cap from this receipt; they never accept an independent
    /// budget argument.
    #[must_use]
    pub const fn budget_receipt(&self) -> DownstreamBudgetReceiptV2 {
        self.budget_receipt
    }
}

/// Finalizer of the nested-relation registry and every retained correlation
/// edge. It requires exact equality between the relational-EXISTS expression
/// roots and the registry keys, document/registry identity of each stored
/// relation, and — for every correlation reference — membership in the sealed
/// expression map, uniqueness, and a checked location minted by this
/// document's registry. A dropped or extra nested root, a dropped, duplicate,
/// swapped, foreign, or cross-registry correlation reference rejects before
/// the document is assembled.
fn check_nested_relation_finalizer(
    source_id: SourceIdV2,
    registry_identity: &SourceRegistryIdentityV2,
    expressions: &BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
) -> Result<(), &'static str> {
    let expected_nested_roots = expressions
        .values()
        .filter_map(|expression| match expression.shape() {
            TypedExprShapeV2::RelationExists { relation_root, .. } => Some(*relation_root),
            TypedExprShapeV2::Literal(_)
            | TypedExprShapeV2::Binding(_)
            | TypedExprShapeV2::Path { .. }
            | TypedExprShapeV2::Unary { .. }
            | TypedExprShapeV2::Binary { .. }
            | TypedExprShapeV2::Call { .. }
            | TypedExprShapeV2::Paren { .. }
            | TypedExprShapeV2::Record(_)
            | TypedExprShapeV2::List(_)
            | TypedExprShapeV2::BoolValue { .. }
            | TypedExprShapeV2::Compare { .. }
            | TypedExprShapeV2::And { .. }
            | TypedExprShapeV2::Or { .. }
            | TypedExprShapeV2::Not { .. }
            | TypedExprShapeV2::LogicalQuantifier { .. } => None,
        })
        .collect::<BTreeSet<_>>();
    if nested_relations.keys().copied().collect::<BTreeSet<_>>() != expected_nested_roots {
        return Err("nested relation registry differs from relational EXISTS expression facts");
    }
    if nested_relations.iter().any(|(key, relation)| {
        key.source_id != source_id
            || relation.key() != *key
            || relation.location().location().source_id != source_id
            || relation.location().registry_identity() != registry_identity
    }) {
        return Err("nested relation registry identity differs from this document");
    }
    let expression_keys = expressions.keys().copied().collect::<BTreeSet<_>>();
    for expression in expressions.values() {
        let TypedExprShapeV2::RelationExists { correlations, .. } = expression.shape() else {
            continue;
        };
        let witnesses = correlations
            .iter()
            .map(|edge| CorrelationWitnessV2 {
                reference: edge.reference(),
                binding_source: edge.binding().source_id(),
                registry: edge.location().registry_identity().clone(),
            })
            .collect::<Vec<_>>();
        check_correlation_witnesses(source_id, registry_identity, &expression_keys, &witnesses)?;
    }
    Ok(())
}

/// Read-only correlation witness compared by the document finalizer. This is
/// comparison data only; Binder and the sealing path remain the authority.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CorrelationWitnessV2 {
    reference: quanta_gqlang_v2_types::SourceQualifiedNodeIdV2,
    binding_source: SourceIdV2,
    registry: SourceRegistryIdentityV2,
}

/// Private per-nested-relation correlation finalizer: every retained capture
/// reference must be a member of the sealed expression set of this document,
/// appear at most once in one vector, and carry a location minted by this
/// document's extended source registry.
fn check_correlation_witnesses(
    source_id: SourceIdV2,
    registry_identity: &SourceRegistryIdentityV2,
    expression_keys: &BTreeSet<SourceQualifiedNodeIdV2>,
    witnesses: &[CorrelationWitnessV2],
) -> Result<(), &'static str> {
    let mut references = BTreeSet::new();
    for witness in witnesses {
        if witness.reference.source_id != source_id
            || witness.binding_source != source_id
            || !expression_keys.contains(&witness.reference)
        {
            return Err("correlation reference is foreign to this document's expressions");
        }
        if !references.insert(witness.reference) {
            return Err("duplicate correlation reference in one nested relation");
        }
        if &witness.registry != registry_identity {
            return Err("correlation reference location belongs to another source registry");
        }
    }
    Ok(())
}

/// Pre-topology facts for every inline document View. Modules remains the
/// topology authority; this plan only retains the exact source use and the
/// HIR body that must be materialized after its opaque proof is issued.
struct InlineViewPlanV2 {
    /// Local inline View asset -> catalog declaration HIR identity.
    declarations: BTreeMap<CatalogAssetIdV2, HirNodeIdV1>,
    /// Local inline View asset -> root relation identity of its query body.
    relation_roots: BTreeMap<CatalogAssetIdV2, SourceQualifiedNodeIdV2>,
    /// Exact relation-source HIR node -> modules-owned local View asset.
    source_targets: BTreeMap<HirNodeIdV1, ViewSourceTargetV2>,
    /// Every direct source use retained by its owning inline View declaration.
    direct_references: BTreeMap<HirNodeIdV1, Vec<ResolvedCatalogReferenceV2>>,
    /// Modules input; only inline View -> View uses belong in this graph.
    dependency_edges: Vec<DocumentCatalogDependencyInputV2>,
}

/// A relation source whose spelling resolved through the Modules View namespace.
/// Local Views are materialized by the document topology. External Views carry
/// a Modules-owned typed relation contract and require no local body.
#[derive(Clone)]
enum ViewSourceTargetV2 {
    Inline(CatalogAssetIdV2),
    External {
        asset: CatalogAssetIdV2,
        relation_type: RelationTypeV1,
    },
}

struct ViewSourceUseCollectorV2 {
    source_asset: Option<CatalogAssetIdV2>,
    uses: Vec<(HirNodeIdV1, String, SpanV1)>,
}

impl HirVisitorV1 for ViewSourceUseCollectorV2 {
    fn visit_relation(&mut self, relation: &quanta_gqlang_v2_hir::RelationExprHirV1) {
        if let quanta_gqlang_v2_hir::RelationExprKindV1::Source(source) = &relation.kind {
            self.uses
                .push((relation.id, source.name.clone(), source.name_span));
        }
        quanta_gqlang_v2_hir::walk_relation(self, relation);
    }
}

fn asset_ref_from_predeclared(asset: &PredeclaredDocumentAssetV2) -> CatalogAssetRefInputV2 {
    CatalogAssetRefInputV2 {
        module_id: asset.id().module_id().clone(),
        namespace_kind: asset.kind(),
        local_name: asset.id().local_name().to_owned(),
        location: asset.location().clone(),
    }
}

fn asset_ref_from_view_reference(
    reference: &quanta_gqlang_v2_modules::ResolvedDocumentCatalogReferenceV2,
) -> CatalogAssetRefInputV2 {
    CatalogAssetRefInputV2 {
        module_id: reference.id().module_id().clone(),
        namespace_kind: reference.kind(),
        local_name: reference.id().local_name().to_owned(),
        location: reference.location().clone(),
    }
}

fn inline_query_for_decl(
    syntax: &HirDocumentV1,
    declaration: HirNodeIdV1,
) -> Option<&quanta_gqlang_v2_hir::QueryHirV1> {
    syntax.items.iter().find_map(|item| {
        let ProgramItemHirV1::CatalogDecl(decl) = item else {
            return None;
        };
        if decl.id != declaration {
            return None;
        }
        match &decl.payload {
            quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                quanta_gqlang_v2_hir::ViewBodyV1::Inline(query),
            ) => Some(query.as_ref()),
            quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                quanta_gqlang_v2_hir::ViewBodyV1::QueryRef(_),
            )
            | quanta_gqlang_v2_hir::CatalogPayloadV1::Model { .. }
            | quanta_gqlang_v2_hir::CatalogPayloadV1::Pack { .. }
            | quanta_gqlang_v2_hir::CatalogPayloadV1::Profile { .. }
            | quanta_gqlang_v2_hir::CatalogPayloadV1::Gate { .. }
            | quanta_gqlang_v2_hir::CatalogPayloadV1::RuleBinding { .. } => None,
        }
    })
}

fn collect_inline_view_plan(
    syntax: &HirDocumentV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &ExtendedSourceRegistryViewV2,
    catalog_items: &BTreeMap<HirNodeIdV1, PredeclaredDocumentAssetV2>,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<InlineViewPlanV2, FrontendTransactionErrorV2> {
    let mut plan = InlineViewPlanV2 {
        declarations: BTreeMap::new(),
        relation_roots: BTreeMap::new(),
        source_targets: BTreeMap::new(),
        direct_references: BTreeMap::new(),
        dependency_edges: Vec::new(),
    };
    let mut uses = Vec::new();
    for item in &syntax.items {
        match item {
            ProgramItemHirV1::CatalogDecl(decl)
                if matches!(
                    decl.payload,
                    quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                        quanta_gqlang_v2_hir::ViewBodyV1::Inline(_)
                    )
                ) =>
            {
                let asset = catalog_items.get(&decl.id).ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        decl.span,
                        "inline View declaration lacks modules predeclaration",
                    )
                })?;
                let query = inline_query_for_decl(syntax, decl.id).ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        decl.span,
                        "inline View declaration lost its exact query body",
                    )
                })?;
                if plan
                    .declarations
                    .insert(asset.id().clone(), decl.id)
                    .is_some()
                    || plan
                        .relation_roots
                        .insert(
                            asset.id().clone(),
                            SourceQualifiedNodeIdV2 {
                                source_id,
                                node_id: query.body.id,
                            },
                        )
                        .is_some()
                {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        decl.span,
                        "duplicate inline View materialization identity",
                    )
                    .into());
                }
                let mut collector = ViewSourceUseCollectorV2 {
                    source_asset: Some(asset.id().clone()),
                    uses: Vec::new(),
                };
                collector.visit_query(query);
                uses.extend(
                    collector.uses.into_iter().map(|(node, name, span)| {
                        (collector.source_asset.clone(), node, name, span)
                    }),
                );
            }
            ProgramItemHirV1::CatalogDecl(_) => {}
            ProgramItemHirV1::Query(_)
            | ProgramItemHirV1::Decl(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::Composition(_)
            | ProgramItemHirV1::Meta(_) => {
                let mut collector = ViewSourceUseCollectorV2 {
                    source_asset: None,
                    uses: Vec::new(),
                };
                collector.visit_item(item);
                uses.extend(
                    collector
                        .uses
                        .into_iter()
                        .map(|(node, name, span)| (None, node, name, span)),
                );
            }
        }
    }
    for (source_asset, node, name, span) in uses {
        steps.charge(source_id, span)?;
        let checked = extended
            .combined()
            .try_check_location(location(source_id, span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    span,
                    error.to_string(),
                )
            })?;
        let Some(target) = validator
            .resolve_reference_if_present(DocumentCatalogReferenceInputV2 {
                module_id: None,
                local_name: name,
                expected_kind: NamespaceKindV2::View,
                location: checked.clone(),
            })
            .map_err(module_failure)?
        else {
            continue;
        };
        let source_target = match target.origin() {
            DocumentCatalogReferenceOriginV2::Local
                if plan.declarations.contains_key(target.id()) =>
            {
                ViewSourceTargetV2::Inline(target.id().clone())
            }
            DocumentCatalogReferenceOriginV2::External => {
                let Some(asset) = authority.modules().external_catalog().asset(target.id()) else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        span,
                        "Modules View reference lacks its external typed payload",
                    )
                    .into());
                };
                let ExternalCatalogPayloadV2::View(relation_type) = asset.payload() else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        span,
                        "Modules View reference payload differs from its namespace",
                    )
                    .into());
                };
                ViewSourceTargetV2::External {
                    asset: target.id().clone(),
                    relation_type: relation_type.clone(),
                }
            }
            DocumentCatalogReferenceOriginV2::Local
            | DocumentCatalogReferenceOriginV2::VisibleHost => return Err(document_failure(
                "sema.unmaterialized_view_source",
                source_id,
                span,
                "relation source resolves to a View without a local inline or external typed body",
            )
            .into()),
        };
        if plan.source_targets.insert(node, source_target).is_some() {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                span,
                "duplicate inline View source-use identity",
            )
            .into());
        }
        if let (Some(source_asset), Some(ViewSourceTargetV2::Inline(_))) =
            (source_asset, plan.source_targets.get(&node))
        {
            let source_decl = plan
                .declarations
                .get(&source_asset)
                .copied()
                .ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        span,
                        "inline View source lacks its local materialization declaration",
                    )
                })?;
            let source = catalog_items.get(&source_decl).ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    span,
                    "inline View source lacks its modules predeclaration",
                )
            })?;
            plan.dependency_edges
                .push(DocumentCatalogDependencyInputV2 {
                    source: asset_ref_from_predeclared(source),
                    target: asset_ref_from_view_reference(&target),
                    expected_target_kind: NamespaceKindV2::View,
                    location: checked,
                });
            plan.direct_references
                .entry(source_decl)
                .or_default()
                .push(ResolvedCatalogReferenceV2::from_proof(&target));
        }
    }
    for references in plan.direct_references.values_mut() {
        references.sort_by(|left, right| {
            (
                left.id(),
                left.location().location().source_id,
                left.location().location().span.start,
                left.location().location().span.end,
            )
                .cmp(&(
                    right.id(),
                    right.location().location().source_id,
                    right.location().location().span.start,
                    right.location().location().span.end,
                ))
        });
        references.dedup_by(|left, right| left.id() == right.id());
    }
    Ok(plan)
}

/// Reject relation trees before recursive Binder and type-owner traversal can
/// exceed the transaction's checked stack-depth budget.
fn check_relation_depth(
    checked: &CheckedSyntaxDocumentV2,
    budget: &FrontendBudgetV2,
) -> Result<(), FrontendFailureV2> {
    struct Guard {
        current_depth: u32,
        limit: u32,
        exceeded: Option<SpanV1>,
    }
    impl HirVisitorV1 for Guard {
        fn visit_relation(&mut self, root: &quanta_gqlang_v2_hir::RelationExprHirV1) {
            use quanta_gqlang_v2_hir::{JoinHirV1, RelationExprKindV1};

            if self.exceeded.is_some() {
                return;
            }
            let previous = self.current_depth;
            let mut stack = vec![(root, previous.saturating_add(1))];
            while let Some((node, depth)) = stack.pop() {
                if depth > self.limit {
                    self.exceeded = Some(node.span);
                    break;
                }
                self.current_depth = depth;
                match &node.kind {
                    RelationExprKindV1::Source(_) => {}
                    RelationExprKindV1::Project(project) => {
                        for item in project.items() {
                            self.visit_value(&item.value);
                        }
                        stack.push((&project.input, depth.saturating_add(1)));
                    }
                    RelationExprKindV1::Filter(filter) => {
                        self.visit_bool(&filter.predicate);
                        stack.push((&filter.input, depth.saturating_add(1)));
                    }
                    RelationExprKindV1::Join(join) => {
                        let (left, right) = match join {
                            JoinHirV1::Inner { left, right, on } => {
                                self.visit_bool(on);
                                (left, right)
                            }
                            JoinHirV1::Cross { left, right } => (left, right),
                        };
                        stack.push((right, depth.saturating_add(1)));
                        stack.push((left, depth.saturating_add(1)));
                    }
                    RelationExprKindV1::GroupBy(group) => {
                        for key in group.keys() {
                            self.visit_value(key);
                        }
                        stack.push((&group.input, depth.saturating_add(1)));
                    }
                    RelationExprKindV1::OrderBy(order) => {
                        for key in order.keys() {
                            self.visit_value(&key.value);
                        }
                        stack.push((&order.input, depth.saturating_add(1)));
                    }
                }
                if self.exceeded.is_some() {
                    break;
                }
            }
            self.current_depth = previous;
        }
    }
    let mut guard = Guard {
        current_depth: 0,
        limit: budget.max_relation_depth(),
        exceeded: None,
    };
    for item in &checked.checked().document().items {
        guard.visit_item(item);
        if let Some(span) = guard.exceeded {
            return Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
                FrontendStageV2::Resource,
                "frontend.resource_limit_exceeded",
                FrontendSeverityV2::Error,
                location(checked.source_id(), span),
                Vec::new(),
                "relation_depth exceeds frontend limit",
            )));
        }
    }
    Ok(())
}

/// Resolve one checked document against complete immutable authority. Every
/// body and metadata directive is handled inside one transaction; any failure
/// returns diagnostics and no semantic document.
pub fn resolve_checked_document_v2(
    checked: CheckedSyntaxDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
) -> Result<SemanticDocumentV2, FrontendTransactionErrorV2> {
    let extended = authority
        .try_extend_document(checked.source_id(), checked.source_len())
        .map_err(|error| {
            FrontendTransactionErrorV2::Document(document_failure(
                error.code(),
                checked.source_id(),
                SpanV1::point(0),
                error.to_string(),
            ))
        })?;
    let declared = DeclaredDocumentV2::try_new(checked, authority.budget(), &extended)?;
    let mut steps = SemanticStepCounterV2::new(authority.budget());
    let mut validator = DocumentCatalogValidatorV2::try_new(extended.clone(), authority.modules())
        .map_err(module_failure)?;
    // Mint local IDs through the modules owner before any body/reference
    // resolution. No semantic path derives CatalogAssetId from raw spelling.
    let mut authored_assets = Vec::new();
    for item in &declared.checked().checked().document().items {
        if let ProgramItemHirV1::CatalogDecl(decl) = item {
            let kind = match &decl.payload {
                quanta_gqlang_v2_hir::CatalogPayloadV1::View(_) => NamespaceKindV2::View,
                quanta_gqlang_v2_hir::CatalogPayloadV1::Model { .. } => NamespaceKindV2::Model,
                quanta_gqlang_v2_hir::CatalogPayloadV1::Pack { .. } => NamespaceKindV2::Pack,
                quanta_gqlang_v2_hir::CatalogPayloadV1::Profile { .. } => NamespaceKindV2::Profile,
                quanta_gqlang_v2_hir::CatalogPayloadV1::Gate { .. } => NamespaceKindV2::Gate,
                quanta_gqlang_v2_hir::CatalogPayloadV1::RuleBinding { .. } => NamespaceKindV2::Rule,
            };
            let location = quanta_gqlang_v2_hir::LocatedSpanV2 {
                source_id: declared.checked().source_id(),
                span: decl.name_span,
            };
            let checked_location =
                extended
                    .combined()
                    .try_check_location(location)
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            location.source_id,
                            location.span,
                            error.to_string(),
                        )
                    })?;
            authored_assets.push(DocumentCatalogAssetInputV2 {
                owner_module_id: authority
                    .modules()
                    .resolved_graph()
                    .current_module()
                    .clone(),
                local_name: decl.name.clone(),
                namespace_kind: kind,
                location: checked_location,
            });
        }
    }
    let predeclared = validator
        .predeclare_assets(authored_assets)
        .map_err(module_failure)?;
    let mut catalog_items = BTreeMap::new();
    for item in &declared.checked().checked().document().items {
        if let ProgramItemHirV1::CatalogDecl(decl) = item {
            let matches = predeclared
                .iter()
                .filter(|asset| {
                    asset.id().local_name() == decl.name
                        && asset.location().location().span == decl.name_span
                })
                .collect::<Vec<_>>();
            let [asset] = matches.as_slice() else {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    declared.checked().source_id(),
                    decl.name_span,
                    "modules predeclaration omitted or ambiguously identified a catalog item",
                )
                .into());
            };
            catalog_items.insert(decl.id, (*asset).clone());
        }
    }
    let inline_views = collect_inline_view_plan(
        declared.checked().checked().document(),
        declared.checked().source_id(),
        authority,
        &extended,
        &catalog_items,
        &mut validator,
        &mut steps,
    )?;
    // Modules owns the only View graph. Sema supplies direct use facts once,
    // materializes only in its opaque dependency-first order, and hands the
    // exact proof back to the same validator for final sealing.
    let topology = validator
        .validate_view_topology(inline_views.dependency_edges.clone())
        .map_err(module_failure)?;
    let result = resolve_document_bodies(
        &declared,
        authority,
        &extended,
        &catalog_items,
        &inline_views,
        &topology,
        &mut validator,
        &mut steps,
    )?;
    let catalog = validator
        .validate_predeclared(topology, result.catalog_input)
        .map_err(module_failure)?;
    Ok(SemanticDocumentV2::try_assemble(
        &declared,
        result.items,
        result.expressions,
        result.nested_relations,
        result.type_fact_count,
        catalog,
        authority.graph_references(),
        authority.budget().downstream_receipt(),
    )?)
}

pub(crate) struct SemanticStepCounterV2 {
    limit: u32,
    used: u32,
}

impl SemanticStepCounterV2 {
    pub(crate) fn new(budget: &FrontendBudgetV2) -> Self {
        Self {
            limit: budget.max_semantic_steps(),
            used: 0,
        }
    }
    pub(crate) fn charge(
        &mut self,
        source_id: SourceIdV2,
        span: SpanV1,
    ) -> Result<(), FrontendFailureV2> {
        self.used = self.used.saturating_add(1);
        if self.used > self.limit {
            return Err(document_failure(
                "frontend.resource_limit_exceeded",
                source_id,
                span,
                "semantic_steps exceeds frontend limit",
            ));
        }
        Ok(())
    }
}

pub(crate) fn document_failure(
    code: &'static str,
    source_id: SourceIdV2,
    span: SpanV1,
    message: impl AsRef<str>,
) -> FrontendFailureV2 {
    FrontendFailureV2::one(FrontendDiagnosticV2::new(
        FrontendStageV2::Validation,
        code,
        FrontendSeverityV2::Error,
        location(source_id, span),
        Vec::new(),
        message,
    ))
}

pub(crate) fn module_failure(
    error: quanta_gqlang_v2_modules::ModuleDiagnosticV2,
) -> FrontendTransactionErrorV2 {
    let Some(primary) = error.primary() else {
        return FrontendTransactionErrorV2::Authority(FrontendAuthorityErrorV2::from_external(
            error.code(),
            error.to_string(),
        ));
    };
    let mut secondary = error.secondary().into_iter().collect::<Vec<_>>();
    secondary.extend_from_slice(error.edge_witness());
    FrontendTransactionErrorV2::Document(FrontendFailureV2::one(FrontendDiagnosticV2::new(
        FrontendStageV2::Validation,
        error.code(),
        FrontendSeverityV2::Error,
        primary,
        secondary,
        error.to_string(),
    )))
}

fn type_failure(error: TypeDiagnosticV2) -> FrontendTransactionErrorV2 {
    let Some(primary) = error.primary() else {
        return FrontendTransactionErrorV2::Authority(FrontendAuthorityErrorV2::from_external(
            error.code(),
            error.to_string(),
        ));
    };
    let secondary = error.secondary().into_iter().collect();
    FrontendTransactionErrorV2::Document(FrontendFailureV2::one(FrontendDiagnosticV2::new(
        FrontendStageV2::Typing,
        error.code(),
        FrontendSeverityV2::Error,
        primary,
        secondary,
        error.to_string(),
    )))
}

struct BodyResolutionV2 {
    items: Vec<SemanticItemV2>,
    expressions: BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    type_fact_count: usize,
    catalog_input: DocumentCatalogFinalInputV2,
}

struct DeclarationTypeInputsV2<'a> {
    source_id: SourceIdV2,
    bound: &'a BindResultV1,
    declaration_bodies: &'a BTreeMap<HirNodeIdV1, HirNodeIdV1>,
    builtins: &'a BuiltinRegistryV2,
    entries: Vec<ResolvedTypeInputEntryV2>,
    error: Option<FrontendFailureV2>,
}

struct RuleSymbolInputsV2<'a> {
    source_id: SourceIdV2,
    bound: &'a BindResultV1,
    symbols: Vec<RuleSymbolInputV2>,
    calls: Vec<(SourceQualifiedNodeIdV2, String, SpanV1)>,
    error: Option<FrontendFailureV2>,
}

impl HirVisitorV1 for RuleSymbolInputsV2<'_> {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        match &node.kind {
            ValueExprKindV1::Symbol(_) => match self.bound.target_of(node.id) {
                Some(ResolvedTargetV1::Local(binding)) => {
                    let (owner, ordinal) = binding.identity();
                    self.symbols.push(RuleSymbolInputV2 {
                        reference: SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: node.id,
                        },
                        binding: RuleBindingIdV2::new(self.source_id, owner, ordinal),
                    });
                }
                _ if self.error.is_none() => {
                    self.error = Some(document_failure(
                        "sema.unresolved_binding_type",
                        self.source_id,
                        node.span,
                        "rule symbol is not an exact local Binder binding",
                    ))
                }
                _ => {}
            },
            ValueExprKindV1::Call(call) => self.calls.push((
                SourceQualifiedNodeIdV2 {
                    source_id: self.source_id,
                    node_id: node.id,
                },
                call.callee.name.clone(),
                node.span,
            )),
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => {}
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
}

/// Read-only witness of one Binder capture edge. This is comparison data for
/// the private cross-owner equality validator; it is never a public forgery
/// API and confers no authority by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CaptureEdgeWitnessV2 {
    reference: HirNodeIdV1,
    owner: HirNodeIdV1,
    ordinal: usize,
}

/// Independently comparable witness of one relational-EXISTS capture vector,
/// including the nested relation root it belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
struct CaptureVectorWitnessV2 {
    relation_root: HirNodeIdV1,
    edges: Vec<CaptureEdgeWitnessV2>,
}

/// Project one Binder capture set into the read-only witness compared by
/// [`validate_capture_vector`]. Binder remains the sole lexical authority.
fn capture_vector_witness(
    bound: &BindResultV1,
    exists: HirNodeIdV1,
    relation_root: HirNodeIdV1,
) -> CaptureVectorWitnessV2 {
    CaptureVectorWitnessV2 {
        relation_root,
        edges: bound
            .capture_edges(exists)
            .iter()
            .map(|&(reference, owner, ordinal)| CaptureEdgeWitnessV2 {
                reference,
                owner,
                ordinal,
            })
            .collect(),
    }
}

/// Private cross-owner capture-vector equality validator. Sema alone performs
/// this comparison: the witness must equal Binder's complete ordered edge
/// vector for the exact EXISTS node, every edge must resolve to the exact
/// local `(owner, ordinal)` Binder binding, and the witnessed nested relation
/// root must equal the checked HIR root. Dropped, extra, duplicated,
/// reordered, transplanted, foreign, wrong-root, and omitted-capture-set
/// witnesses all reject with a distinct fail-closed reason.
fn validate_capture_vector(
    exists_node: HirNodeIdV1,
    hir_relation_root: HirNodeIdV1,
    witness: &CaptureVectorWitnessV2,
    bound: &BindResultV1,
) -> Result<Vec<CaptureEdgeWitnessV2>, &'static str> {
    if !bound.has_capture_set(exists_node) {
        return Err("Binder omitted the explicit relational EXISTS capture set");
    }
    if witness.relation_root != hir_relation_root {
        return Err("capture witness nested relation root differs from checked HIR");
    }
    let mut references = BTreeSet::new();
    for edge in &witness.edges {
        if !references.insert(edge.reference) {
            return Err("duplicate capture reference in witness");
        }
    }
    let binder_edges = bound.capture_edges(exists_node);
    if binder_edges.len() != witness.edges.len() {
        return Err("capture vector dropped or added an edge");
    }
    for (&(binder_reference, binder_owner, binder_ordinal), edge) in
        binder_edges.iter().zip(&witness.edges)
    {
        if edge.reference != binder_reference {
            return Err("capture edge reference differs from Binder proof");
        }
        if edge.owner != binder_owner {
            return Err("capture edge owner differs from Binder proof");
        }
        if edge.ordinal != binder_ordinal {
            return Err("capture edge ordinal differs from Binder proof");
        }
        let Some(ResolvedTargetV1::Local(binding)) = bound.target_of(edge.reference) else {
            return Err("capture reference has no exact local resolution");
        };
        if binding.identity() != (edge.owner, edge.ordinal) {
            return Err("capture resolution identity disagrees with the witness");
        }
    }
    Ok(witness.edges.clone())
}

struct QueryTypeInputsV2<'a> {
    source_id: SourceIdV2,
    authority: &'a ClosedFrontendAuthorityV2,
    bound: &'a BindResultV1,
    builtins: &'a BuiltinRegistryV2,
    extended: &'a ExtendedSourceRegistryViewV2,
    view_source_targets: &'a BTreeMap<HirNodeIdV1, ViewSourceTargetV2>,
    view_relation_roots: &'a BTreeMap<CatalogAssetIdV2, SourceQualifiedNodeIdV2>,
    owners: BTreeMap<(HirNodeIdV1, usize), ResolvedTypeInputValueV2>,
    entries: Vec<ResolvedTypeInputEntryV2>,
    rows: Vec<DeferredRelationRowInputV2>,
    composition_rows: BTreeMap<(HirNodeIdV1, usize), CompositionRelationRowV2>,
    error: Option<FrontendFailureV2>,
}

/// The only relation row forms a prior Composition producer can expose to a
/// later authored `FROM` source. Generated rows are known at that consuming
/// source; they are never forged as a non-HIR relation root.
enum CompositionRelationRowV2 {
    PriorRelation(SourceQualifiedNodeIdV2),
    Generated {
        field_name: &'static str,
        scalar: ScalarTyV1,
    },
    Absent,
}

impl HirVisitorV1 for QueryTypeInputsV2<'_> {
    fn visit_relation(&mut self, relation: &quanta_gqlang_v2_hir::RelationExprHirV1) {
        match &relation.kind {
            quanta_gqlang_v2_hir::RelationExprKindV1::Source(source) => {
                let root = SourceQualifiedNodeIdV2 {
                    source_id: self.source_id,
                    node_id: relation.id,
                };
                if let Some(view) = self.view_source_targets.get(&relation.id) {
                    match view {
                        ViewSourceTargetV2::Inline(view) => {
                            let Some(target_root) = self.view_relation_roots.get(view).copied()
                            else {
                                if self.error.is_none() {
                                    self.error = Some(document_failure(
                                        "sema.unmaterialized_view_source",
                                        self.source_id,
                                        source.name_span,
                                        "resolved View source has no local inline relation root",
                                    ));
                                }
                                return;
                            };
                            self.rows.push(DeferredRelationRowInputV2::prior_relation(
                                root,
                                target_root,
                            ));
                        }
                        ViewSourceTargetV2::External { relation_type, .. } => {
                            let checked = match self
                                .extended
                                .combined()
                                .try_check_location(location(self.source_id, source.name_span))
                            {
                                Ok(checked) => checked,
                                Err(error) => {
                                    if self.error.is_none() {
                                        self.error = Some(document_failure(
                                            "sema.document_validation_failed",
                                            self.source_id,
                                            source.name_span,
                                            error.to_string(),
                                        ));
                                    }
                                    return;
                                }
                            };
                            let fields = relation_type
                                .row()
                                .fields()
                                .iter()
                                .enumerate()
                                .map(|(ordinal, field)| {
                                    DeferredRelationRowFieldInputV2::new(
                                        ordinal,
                                        field.name(),
                                        checked.clone(),
                                        RelationRowFieldSourceV2::Known(field.value_type().clone()),
                                    )
                                })
                                .collect::<Vec<_>>();
                            self.rows.push(DeferredRelationRowInputV2::known_source(
                                root,
                                fields.len(),
                                fields,
                            ));
                        }
                    }
                    if source.alias.is_some() {
                        self.owners.insert(
                            (relation.id, 0),
                            ResolvedTypeInputValueV2::BindingFromRelationRow(root),
                        );
                    }
                    quanta_gqlang_v2_hir::walk_relation(self, relation);
                    return;
                }
                if let Some(target) =
                    self.bound
                        .target_of_use(relation.id, source.name_span, &source.name)
                {
                    match target {
                        ResolvedTargetV1::Local(binding)
                            if binding.kind == BinderKindV1::CompositionLet =>
                        {
                            let Some(producer_row) = self.composition_rows.get(&binding.identity())
                            else {
                                if self.error.is_none() {
                                    self.error = Some(document_failure(
                                        "sema.non_relation_composition_source",
                                        self.source_id,
                                        source.name_span,
                                        "Composition binding has no relation row",
                                    ));
                                }
                                return;
                            };
                            match producer_row {
                                CompositionRelationRowV2::PriorRelation(producer_root) => {
                                    self.rows.push(DeferredRelationRowInputV2::prior_relation(
                                        root,
                                        *producer_root,
                                    ))
                                }
                                CompositionRelationRowV2::Generated { field_name, scalar } => {
                                    let checked = match self.extended.combined().try_check_location(
                                        location(self.source_id, source.name_span),
                                    ) {
                                        Ok(checked) => checked,
                                        Err(error) => {
                                            if self.error.is_none() {
                                                self.error = Some(document_failure(
                                                    "sema.document_validation_failed",
                                                    self.source_id,
                                                    source.name_span,
                                                    error.to_string(),
                                                ));
                                            }
                                            return;
                                        }
                                    };
                                    self.rows.push(DeferredRelationRowInputV2::known_source(
                                        root,
                                        1,
                                        vec![DeferredRelationRowFieldInputV2::new(
                                            0,
                                            *field_name,
                                            checked,
                                            RelationRowFieldSourceV2::Known(ValueTypeV1::scalar(
                                                scalar.clone(),
                                            )),
                                        )],
                                    ));
                                }
                                CompositionRelationRowV2::Absent => {
                                    if self.error.is_none() {
                                        self.error = Some(document_failure(
                                            "sema.non_relation_composition_source",
                                            self.source_id,
                                            source.name_span,
                                            "Composition binding has no relation row",
                                        ));
                                    }
                                    return;
                                }
                            }
                            if source.alias.is_some() {
                                self.owners.insert(
                                    (relation.id, 0),
                                    ResolvedTypeInputValueV2::BindingFromRelationRow(root),
                                );
                            }
                            quanta_gqlang_v2_hir::walk_relation(self, relation);
                            return;
                        }
                        ResolvedTargetV1::Local(_)
                        | ResolvedTargetV1::Declaration { .. }
                        | ResolvedTargetV1::Import { .. } => {
                            if self.error.is_none() {
                                self.error = Some(document_failure(
                                    "sema.invalid_relation_source_kind",
                                    self.source_id,
                                    source.name_span,
                                    "resolved source is not a Composition relation binding",
                                ));
                            }
                            return;
                        }
                    }
                }
                let Some(host) = self.authority.relations().get(&source.name) else {
                    if self.error.is_none() {
                        self.error = Some(document_failure(
                            "sema.unresolved_reference",
                            self.source_id,
                            source.name_span,
                            "host relation source is absent",
                        ));
                    }
                    return;
                };
                let checked = match self
                    .extended
                    .combined()
                    .try_check_location(location(self.source_id, source.name_span))
                {
                    Ok(checked) => checked,
                    Err(error) => {
                        if self.error.is_none() {
                            self.error = Some(document_failure(
                                "sema.document_validation_failed",
                                self.source_id,
                                source.name_span,
                                error.to_string(),
                            ));
                        }
                        return;
                    }
                };
                let fields = host
                    .relation_type()
                    .row()
                    .fields()
                    .iter()
                    .enumerate()
                    .map(|(ordinal, field)| {
                        DeferredRelationRowFieldInputV2::new(
                            ordinal,
                            field.name(),
                            checked.clone(),
                            RelationRowFieldSourceV2::Known(field.value_type().clone()),
                        )
                    })
                    .collect::<Vec<_>>();
                self.rows.push(DeferredRelationRowInputV2::known_source(
                    root,
                    fields.len(),
                    fields,
                ));
                if source.alias.is_some() {
                    self.owners.insert(
                        (relation.id, 0),
                        ResolvedTypeInputValueV2::BindingFromRelationRow(root),
                    );
                }
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) => {
                let Some(frame) = self.bound.relation_output(relation.id) else {
                    if self.error.is_none() {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            relation.span,
                            "Binder omitted projection output",
                        ));
                    }
                    return;
                };
                let mut fields = Vec::new();
                for (ordinal, (item, binding)) in project.items().iter().zip(frame).enumerate() {
                    self.owners.insert(
                        (relation.id, ordinal),
                        ResolvedTypeInputValueV2::BindingFromExpression(SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: item.value.id,
                        }),
                    );
                    let name_span = item
                        .alias
                        .as_ref()
                        .map_or(item.value.span, |alias| alias.span);
                    match self
                        .extended
                        .combined()
                        .try_check_location(location(self.source_id, name_span))
                    {
                        Ok(checked) => fields.push(DeferredRelationRowFieldInputV2::new(
                            ordinal,
                            binding.name.clone(),
                            checked,
                            RelationRowFieldSourceV2::PriorExpression(SourceQualifiedNodeIdV2 {
                                source_id: self.source_id,
                                node_id: item.value.id,
                            }),
                        )),
                        Err(error) => {
                            if self.error.is_none() {
                                self.error = Some(document_failure(
                                    "sema.document_validation_failed",
                                    self.source_id,
                                    name_span,
                                    error.to_string(),
                                ));
                            }
                        }
                    }
                }
                self.rows
                    .push(DeferredRelationRowInputV2::expression_fields(
                        SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: relation.id,
                        },
                        frame.len(),
                        fields,
                    ));
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(group) => {
                let Some(frame) = self.bound.relation_output(relation.id) else {
                    if self.error.is_none() {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            relation.span,
                            "Binder omitted group output",
                        ));
                    }
                    return;
                };
                let mut fields = Vec::new();
                for (ordinal, (key, binding)) in group.keys().iter().zip(frame).enumerate() {
                    self.owners.insert(
                        (relation.id, ordinal),
                        ResolvedTypeInputValueV2::BindingFromExpression(SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: key.id,
                        }),
                    );
                    match self
                        .extended
                        .combined()
                        .try_check_location(location(self.source_id, binding.span))
                    {
                        Ok(checked) => fields.push(DeferredRelationRowFieldInputV2::new(
                            ordinal,
                            binding.name.clone(),
                            checked,
                            RelationRowFieldSourceV2::PriorExpression(SourceQualifiedNodeIdV2 {
                                source_id: self.source_id,
                                node_id: key.id,
                            }),
                        )),
                        Err(error) => {
                            if self.error.is_none() {
                                self.error = Some(document_failure(
                                    "sema.document_validation_failed",
                                    self.source_id,
                                    binding.span,
                                    error.to_string(),
                                ));
                            }
                        }
                    }
                }
                self.rows
                    .push(DeferredRelationRowInputV2::expression_fields(
                        SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: relation.id,
                        },
                        frame.len(),
                        fields,
                    ));
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::Filter(_)
            | quanta_gqlang_v2_hir::RelationExprKindV1::Join(_)
            | quanta_gqlang_v2_hir::RelationExprKindV1::OrderBy(_) => {}
        }
        quanta_gqlang_v2_hir::walk_relation(self, relation);
    }

    fn visit_composition(&mut self, node: &quanta_gqlang_v2_hir::CompositionHirV1) {
        for (ordinal, binding) in node.bindings.iter().enumerate() {
            let Some(proof) = self.bound.composition_binding(node.id, ordinal) else {
                if self.error.is_none() {
                    self.error = Some(document_failure(
                        "sema.document_validation_failed",
                        self.source_id,
                        binding.name_span,
                        "Binder omitted exact Composition binding proof",
                    ));
                }
                return;
            };
            if proof.kind != BinderKindV1::CompositionLet
                || proof.identity() != (node.id, ordinal)
                || proof.name != binding.name
                || proof.span != binding.name_span
            {
                if self.error.is_none() {
                    self.error = Some(document_failure(
                        "sema.document_validation_failed",
                        self.source_id,
                        binding.name_span,
                        "Binder Composition binding proof differs from checked HIR",
                    ));
                }
                return;
            }
            let identity = proof.identity();
            match &binding.producer {
                quanta_gqlang_v2_hir::CompositionProducerV1::Query(query) => {
                    self.visit_query(query);
                    if self
                        .composition_rows
                        .insert(
                            identity,
                            CompositionRelationRowV2::PriorRelation(SourceQualifiedNodeIdV2 {
                                source_id: self.source_id,
                                node_id: query.body.id,
                            }),
                        )
                        .is_some()
                        && self.error.is_none()
                    {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            binding.name_span,
                            "duplicate Composition binding identity",
                        ));
                    }
                }
                quanta_gqlang_v2_hir::CompositionProducerV1::Sample(_) => {
                    if self
                        .composition_rows
                        .insert(
                            identity,
                            CompositionRelationRowV2::Generated {
                                field_name: "path",
                                scalar: ScalarTyV1::RepoPath,
                            },
                        )
                        .is_some()
                        && self.error.is_none()
                    {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            binding.name_span,
                            "duplicate Composition binding identity",
                        ));
                    }
                }
                quanta_gqlang_v2_hir::CompositionProducerV1::GraphQuery(graph) => {
                    let row = if matches!(
                        graph.kind,
                        quanta_gqlang_v2_hir::GraphQueryKindV1::Path
                            | quanta_gqlang_v2_hir::GraphQueryKindV1::Reachability
                    ) {
                        CompositionRelationRowV2::Generated {
                            field_name: "function",
                            scalar: ScalarTyV1::EntityRef,
                        }
                    } else {
                        CompositionRelationRowV2::Absent
                    };
                    if self.composition_rows.insert(identity, row).is_some() && self.error.is_none()
                    {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            binding.name_span,
                            "duplicate Composition binding identity",
                        ));
                    }
                }
            }
        }
        match &node.body {
            quanta_gqlang_v2_hir::CompositionBodyV1::Query(query) => self.visit_query(query),
            quanta_gqlang_v2_hir::CompositionBodyV1::GraphQuery(_)
            | quanta_gqlang_v2_hir::CompositionBodyV1::Legality(_)
            | quanta_gqlang_v2_hir::CompositionBodyV1::Explain(_) => {}
        }
    }

    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        if let BoolExprKindV1::RelationExists(exists) = &node.kind {
            if !self.bound.has_capture_set(node.id) {
                if self.error.is_none() {
                    self.error = Some(document_failure(
                        "sema.document_validation_failed",
                        self.source_id,
                        node.span,
                        "Binder omitted explicit relational EXISTS capture set",
                    ));
                }
                return;
            }
            let witness = capture_vector_witness(self.bound, node.id, exists.relation.id);
            let edges = validate_capture_vector(node.id, exists.relation.id, &witness, self.bound)
                .map_err(|message| {
                    if self.error.is_none() {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            node.span,
                            message,
                        ));
                    }
                });
            let Ok(edges) = edges else {
                return;
            };
            let captures = edges
                .into_iter()
                .map(|edge| {
                    ResolvedRelationCaptureInputV2::new(
                        SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: edge.reference,
                        },
                        SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: edge.owner,
                        },
                        edge.ordinal,
                    )
                })
                .collect::<Vec<_>>();
            match ResolvedRelationPredicateInputV2::try_resolved(
                SourceQualifiedNodeIdV2 {
                    source_id: self.source_id,
                    node_id: exists.relation.id,
                },
                captures,
            ) {
                Ok(input) => self.entries.push(ResolvedTypeInputEntryV2 {
                    key: SourceQualifiedNodeIdV2 {
                        source_id: self.source_id,
                        node_id: node.id,
                    },
                    value: ResolvedTypeInputValueV2::RelationPredicate(input),
                }),
                Err(error) => {
                    if self.error.is_none() {
                        self.error = Some(document_failure(
                            error.code(),
                            self.source_id,
                            node.span,
                            error.to_string(),
                        ));
                    }
                }
            }
        }
        quanta_gqlang_v2_hir::walk_bool(self, node);
    }

    fn visit_value(&mut self, node: &ValueExprHirV1) {
        let entry = match &node.kind {
            ValueExprKindV1::Symbol(_) => match self.bound.target_of(node.id) {
                Some(ResolvedTargetV1::Local(binding)) => {
                    self.owners.get(&binding.identity()).cloned()
                }
                Some(ResolvedTargetV1::Declaration { .. }) => None,
                _ => None,
            },
            ValueExprKindV1::Call(call) => self
                .builtins
                .family_id(&call.callee.name)
                .map(|id| ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(id))),
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => {
                quanta_gqlang_v2_hir::walk_value(self, node);
                return;
            }
        };
        if let Some(value) = entry {
            self.entries.push(ResolvedTypeInputEntryV2 {
                key: SourceQualifiedNodeIdV2 {
                    source_id: self.source_id,
                    node_id: node.id,
                },
                value,
            });
        } else if self.error.is_none() {
            self.error = Some(document_failure(
                "sema.unresolved_binding_type",
                self.source_id,
                node.span,
                "query reference has no exact Binder/host or prior-stage type authority",
            ));
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
}

/// Independently reconstructed HIR path for one tentative Binder partition
/// reference. This is a structural witness, not a second name resolver or a
/// callable-effect proof.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PartitionCandidateShapeV2 {
    span: SpanV1,
    direct_group: HirNodeIdV1,
    call_arguments: Vec<HirNodeIdV1>,
}

struct PartitionCandidateShapeCollectorV2 {
    targets: BTreeSet<HirNodeIdV1>,
    direct_group: Option<HirNodeIdV1>,
    call_arguments: Vec<HirNodeIdV1>,
    witnesses: BTreeMap<HirNodeIdV1, PartitionCandidateShapeV2>,
    duplicate: Option<SpanV1>,
}

impl HirVisitorV1 for PartitionCandidateShapeCollectorV2 {
    fn visit_relation(&mut self, relation: &quanta_gqlang_v2_hir::RelationExprHirV1) {
        if let quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) = &relation.kind {
            self.visit_relation(&project.input);
            let previous = self.direct_group;
            self.direct_group = matches!(
                project.input.kind,
                quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(_)
            )
            .then_some(project.input.id);
            for item in project.items() {
                self.visit_value(&item.value);
            }
            self.direct_group = previous;
        } else {
            quanta_gqlang_v2_hir::walk_relation(self, relation);
        }
    }

    fn visit_value(&mut self, node: &ValueExprHirV1) {
        match &node.kind {
            ValueExprKindV1::Symbol(_) if self.targets.contains(&node.id) => {
                if let Some(direct_group) = self.direct_group {
                    let shape = PartitionCandidateShapeV2 {
                        span: node.span,
                        direct_group,
                        call_arguments: self.call_arguments.clone(),
                    };
                    if self.witnesses.insert(node.id, shape).is_some() {
                        self.duplicate = Some(node.span);
                    }
                }
            }
            ValueExprKindV1::Call(_) => {
                self.call_arguments.push(node.id);
                // HIR's call walker visits only expression arguments, never
                // the callee spelling or a sibling outside this call.
                quanta_gqlang_v2_hir::walk_value(self, node);
                self.call_arguments.pop();
            }
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Symbol(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => quanta_gqlang_v2_hir::walk_value(self, node),
        }
    }
}

fn collect_partition_candidate_shapes(
    checked: &CheckedSyntaxDocumentV2,
    bound: &BindResultV1,
) -> Result<BTreeMap<HirNodeIdV1, PartitionCandidateShapeV2>, FrontendFailureV2> {
    let targets = bound
        .partition_candidates()
        .map(|candidate| candidate.reference())
        .collect::<BTreeSet<_>>();
    if targets.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut collector = PartitionCandidateShapeCollectorV2 {
        targets,
        direct_group: None,
        call_arguments: Vec::new(),
        witnesses: BTreeMap::new(),
        duplicate: None,
    };
    for item in &checked.checked().document().items {
        if let ProgramItemHirV1::Query(query) = item {
            collector.visit_relation(&query.body);
        }
    }
    if let Some(span) = collector.duplicate {
        return Err(document_failure(
            "sema.document_validation_failed",
            checked.source_id(),
            span,
            "partition candidate reference occurs more than once in checked HIR",
        ));
    }
    Ok(collector.witnesses)
}

struct NestedAggregateCallGuardV2<'a> {
    source_id: SourceIdV2,
    facts: &'a TypedExpressionFactsV2,
    builtins: &'a BuiltinRegistryV2,
    aggregate_depth: u32,
    violation: Option<SpanV1>,
}

impl HirVisitorV1 for NestedAggregateCallGuardV2<'_> {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        if self.violation.is_some() {
            return;
        }
        let aggregate_call = if matches!(node.kind, ValueExprKindV1::Call(_)) {
            let key = SourceQualifiedNodeIdV2 {
                source_id: self.source_id,
                node_id: node.id,
            };
            self.facts
                .get(key)
                .and_then(quanta_gqlang_v2_types::TypedExpressionFactV2::resolved_call)
                .and_then(|target| match target {
                    ResolvedCallTargetV2::Builtin(id) => self.builtins.effects_of_resolved(*id),
                    ResolvedCallTargetV2::Predicate(_) | ResolvedCallTargetV2::HostPredicate(_) => {
                        None
                    }
                })
                .is_some_and(|effects| effects.aggregation == AggregationClassV1::Aggregate)
        } else {
            false
        };
        if aggregate_call && self.aggregate_depth > 0 {
            self.violation = Some(node.span);
            return;
        }
        if aggregate_call {
            self.aggregate_depth += 1;
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
        if aggregate_call {
            self.aggregate_depth -= 1;
        }
    }
}

fn reject_nested_aggregate_calls(
    checked: &CheckedSyntaxDocumentV2,
    facts: &TypedExpressionFactsV2,
    builtins: &BuiltinRegistryV2,
) -> Result<(), FrontendFailureV2> {
    let mut guard = NestedAggregateCallGuardV2 {
        source_id: checked.source_id(),
        facts,
        builtins,
        aggregate_depth: 0,
        violation: None,
    };
    for item in &checked.checked().document().items {
        guard.visit_item(item);
        if let Some(span) = guard.violation {
            return Err(document_failure(
                "sema.illegal_aggregate_context",
                checked.source_id(),
                span,
                "aggregate call cannot be nested inside another aggregate call",
            ));
        }
    }
    Ok(())
}

fn check_partition_candidate_shape(
    witnesses: &BTreeMap<HirNodeIdV1, PartitionCandidateShapeV2>,
    reference: HirNodeIdV1,
    span: SpanV1,
    group: HirNodeIdV1,
    call_stack: &[HirNodeIdV1],
) -> Result<(), &'static str> {
    let Some(shape) = witnesses.get(&reference) else {
        return Err("partition candidate is absent from a direct Project-over-Group expression");
    };
    if shape.span != span || shape.direct_group != group || shape.call_arguments != call_stack {
        return Err("partition candidate differs from the checked HIR reference, group, or call-argument path");
    }
    Ok(())
}

impl HirVisitorV1 for DeclarationTypeInputsV2<'_> {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        let value = match &node.kind {
            ValueExprKindV1::Symbol(_) => match self.bound.target_of(node.id) {
                Some(ResolvedTargetV1::Declaration { node: target, .. }) => {
                    self.declaration_bodies.get(target).copied().map(|body| {
                        ResolvedTypeInputValueV2::BindingFromDeclaration(SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: body,
                        })
                    })
                }
                _ => None,
            },
            ValueExprKindV1::Call(call) => self
                .builtins
                .family_id(&call.callee.name)
                .map(|id| ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(id))),
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => None,
        };
        if matches!(
            node.kind,
            ValueExprKindV1::Symbol(_) | ValueExprKindV1::Call(_)
        ) {
            if let Some(value) = value {
                self.entries.push(ResolvedTypeInputEntryV2 {
                    key: SourceQualifiedNodeIdV2 {
                        source_id: self.source_id,
                        node_id: node.id,
                    },
                    value,
                });
            } else if self.error.is_none() {
                let code = if matches!(node.kind, ValueExprKindV1::Call(_)) {
                    "types.unknown_builtin"
                } else {
                    "sema.unresolved_binding_type"
                };
                self.error = Some(document_failure(
                    code,
                    self.source_id,
                    node.span,
                    "no closed authoritative type for value reference",
                ));
            }
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
}

fn materialize_declaration_expr(
    node: &ValueExprHirV1,
    source_id: SourceIdV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &ExtendedSourceRegistryViewV2,
    output: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    steps: &mut SemanticStepCounterV2,
    position: AggregatePositionV2,
) -> Result<TypedExprV2, FrontendFailureV2> {
    steps.charge(source_id, node.span)?;
    let key_of = |id| SourceQualifiedNodeIdV2 {
        source_id,
        node_id: id,
    };
    let mut child = |child: &ValueExprHirV1, output: &mut BTreeMap<_, _>| {
        materialize_declaration_expr(
            child, source_id, bound, facts, extended, output, steps, position,
        )
        .map(|typed| typed.key())
    };
    let key = key_of(node.id);
    let fact = facts.get(key).ok_or_else(|| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            node.span,
            "types owner omitted expression fact",
        )
    })?;
    if !matches!(fact.state, ExpressionTypeStateV2::Known(_)) {
        return Err(document_failure(
            "sema.unknown_type_in_closed_compile",
            source_id,
            node.span,
            "closed expression cannot retain unknown type",
        ));
    }
    // The types owner records the authored `-2^127` expression as the checked
    // `i128::MIN` value on the unary node. Its positive magnitude child is only
    // a parsing/type-checking sentinel and must not become executable IR: doing
    // so would encode a second, overflowing negation of an already-minimum
    // value. Preserve the authored root identity while materializing the
    // authoritative checked literal fact.
    let shape = match &node.kind {
        ValueExprKindV1::Unary(unary) if fact.literal.is_some() => {
            // Retain a typed proof for every checked HIR node. The sentinel remains
            // in the semantic document but is unreachable from the folded root, so
            // execution closure projection excludes it.
            let _ = child(&unary.operand, output)?;
            TypedExprShapeV2::Literal(fact.literal.clone().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted folded unary literal fact",
                )
            })?)
        }
        ValueExprKindV1::Literal(_) => {
            TypedExprShapeV2::Literal(fact.literal.clone().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted literal fact",
                )
            })?)
        }
        ValueExprKindV1::Symbol(_) => {
            let (owner, ordinal, kind, name) = match bound.target_of(node.id) {
                Some(ResolvedTargetV1::Declaration {
                    name, node: owner, ..
                }) => (*owner, 0, ResolvedBindingKindV2::Declaration, name.clone()),
                Some(ResolvedTargetV1::Local(binding)) => {
                    let (owner, ordinal) = binding.identity();
                    let kind = match binding.kind {
                        BinderKindV1::Alias => ResolvedBindingKindV2::Alias,
                        BinderKindV1::Param => ResolvedBindingKindV2::Parameter,
                        BinderKindV1::Let => ResolvedBindingKindV2::Local,
                        BinderKindV1::Quantified => ResolvedBindingKindV2::Quantified,
                        BinderKindV1::CompositionLet => ResolvedBindingKindV2::Composition,
                    };
                    (owner, ordinal, kind, binding.name.clone())
                }
                _ => {
                    return Err(document_failure(
                        "sema.unresolved_binding_type",
                        source_id,
                        node.span,
                        "binding lacks a closed canonical identity",
                    ))
                }
            };
            let binding = ResolvedValueBindingV2::try_new(source_id, owner, ordinal, kind, name)
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        error,
                    )
                })?;
            TypedExprShapeV2::Binding(binding)
        }
        ValueExprKindV1::Path(path) => {
            let base = child(&path.base, output)?;
            let mut segments = Vec::new();
            for segment in path.segments.as_slice() {
                let raw = location(source_id, segment.span);
                let checked = extended
                    .combined()
                    .try_check_location(raw)
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            segment.span,
                            error.to_string(),
                        )
                    })?;
                segments.push(TypedPathSegmentV2::new(segment.name.clone(), checked));
            }
            TypedExprShapeV2::Path { base, segments }
        }
        ValueExprKindV1::Unary(unary) => TypedExprShapeV2::Unary {
            operator: fact.operator_id.ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted unary operator",
                )
            })?,
            operand: child(&unary.operand, output)?,
        },
        ValueExprKindV1::Binary(binary) => TypedExprShapeV2::Binary {
            operator: fact.operator_id.ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted binary operator",
                )
            })?,
            lhs: child(&binary.lhs, output)?,
            rhs: child(&binary.rhs, output)?,
        },
        ValueExprKindV1::Call(call) => {
            let mut arguments = Vec::new();
            for arg in call.args().as_slice() {
                arguments.push(match arg {
                    quanta_gqlang_v2_hir::CallArgHirV1::Expr(expr) => {
                        TypedCallArgumentV2::Expression(child(expr, output)?)
                    }
                    quanta_gqlang_v2_hir::CallArgHirV1::Star { .. } => TypedCallArgumentV2::Star,
                });
            }
            let target = fact.resolved_call().cloned().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted callable target",
                )
            })?;
            TypedExprShapeV2::Call { target, arguments }
        }
        ValueExprKindV1::Paren(inner) => TypedExprShapeV2::Paren {
            inner: child(inner, output)?,
        },
        ValueExprKindV1::Record(record) => {
            let mut fields = Vec::new();
            for field in &record.fields {
                let value = child(&field.value, output)?;
                let raw = location(source_id, field.name_span);
                let checked = extended
                    .combined()
                    .try_check_location(raw)
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            field.name_span,
                            error.to_string(),
                        )
                    })?;
                fields.push(
                    TypedRecordFieldV2::try_new(field.name.clone(), checked, value).map_err(
                        |error| {
                            document_failure(
                                "sema.document_validation_failed",
                                source_id,
                                field.name_span,
                                error,
                            )
                        },
                    )?,
                );
            }
            TypedExprShapeV2::Record(fields)
        }
        ValueExprKindV1::List(list) => {
            let mut elements = Vec::new();
            for element in &list.elements {
                elements.push(child(element, output)?);
            }
            TypedExprShapeV2::List(elements)
        }
    };
    let location = location(source_id, node.span);
    let star = matches!(&shape, TypedExprShapeV2::Call { arguments, .. } if arguments.iter().any(|arg| matches!(arg, TypedCallArgumentV2::Star)));
    let context = AggregateContextV2::try_new(position, fact.effects.aggregation, star, location)?;
    let checked = extended
        .combined()
        .try_check_location(location)
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                node.span,
                error.to_string(),
            )
        })?;
    let typed = TypedExprV2::try_from_fact(key, fact, context, shape, checked)?;
    if output.insert(key, typed.clone()).is_some() {
        return Err(document_failure(
            "sema.document_validation_failed",
            source_id,
            node.span,
            "duplicate typed expression identity",
        ));
    }
    Ok(typed)
}

fn materialize_bool(
    node: &BoolExprHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &ExtendedSourceRegistryViewV2,
    composition_outputs: &BTreeMap<(HirNodeIdV1, usize), RelationOutputContractV2>,
    views: &ViewMaterializationInputsV2<'_>,
    output: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &mut SemanticStepCounterV2,
    position: AggregatePositionV2,
) -> Result<TypedBoolExprV2, FrontendFailureV2> {
    steps.charge(source_id, node.span)?;
    let key = |id| SourceQualifiedNodeIdV2 {
        source_id,
        node_id: id,
    };
    let fact = facts.get(key(node.id)).ok_or_else(|| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            node.span,
            "types owner omitted rule boolean fact",
        )
    })?;
    let shape = match &node.kind {
        BoolExprKindV1::Literal(_) => {
            TypedExprShapeV2::Literal(fact.literal.clone().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted boolean literal fact",
                )
            })?)
        }
        BoolExprKindV1::Value(value) => {
            let value = materialize_declaration_expr(
                value, source_id, bound, facts, extended, output, steps, position,
            )?;
            TypedExprShapeV2::BoolValue { value: value.key() }
        }
        BoolExprKindV1::Compare(compare) => {
            let lhs = materialize_declaration_expr(
                &compare.lhs,
                source_id,
                bound,
                facts,
                extended,
                output,
                steps,
                position,
            )?;
            let rhs = materialize_declaration_expr(
                &compare.rhs,
                source_id,
                bound,
                facts,
                extended,
                output,
                steps,
                position,
            )?;
            TypedExprShapeV2::Compare {
                operator: fact.operator_id.ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        "types owner omitted comparison operator",
                    )
                })?,
                lhs: lhs.key(),
                rhs: rhs.key(),
            }
        }
        BoolExprKindV1::And(binary) | BoolExprKindV1::Or(binary) => {
            let lhs = materialize_bool(
                &binary.lhs,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
                position,
            )?;
            let rhs = materialize_bool(
                &binary.rhs,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
                position,
            )?;
            let operator = fact.operator_id.ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted logical operator",
                )
            })?;
            if matches!(node.kind, BoolExprKindV1::And(_)) {
                TypedExprShapeV2::And {
                    operator,
                    lhs: lhs.expression().key(),
                    rhs: rhs.expression().key(),
                }
            } else {
                TypedExprShapeV2::Or {
                    operator,
                    lhs: lhs.expression().key(),
                    rhs: rhs.expression().key(),
                }
            }
        }
        BoolExprKindV1::Not(inner) => {
            let operand = materialize_bool(
                inner,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
                position,
            )?;
            TypedExprShapeV2::Not {
                operator: fact.operator_id.ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        "types owner omitted negation operator",
                    )
                })?,
                operand: operand.expression().key(),
            }
        }
        BoolExprKindV1::Paren(inner) => {
            let inner = materialize_bool(
                inner,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
                position,
            )?;
            TypedExprShapeV2::Paren {
                inner: inner.expression().key(),
            }
        }
        BoolExprKindV1::LogicalQuantifier(quantifier) => {
            let body = materialize_bool(
                &quantifier.body,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
                position,
            )?;
            let binders = quantifier
                .binders
                .as_slice()
                .iter()
                .enumerate()
                .map(|(ordinal, binder)| {
                    ResolvedValueBindingV2::try_new(
                        source_id,
                        node.id,
                        ordinal,
                        ResolvedBindingKindV2::Quantified,
                        binder.name.clone(),
                    )
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            binder.span,
                            error,
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            TypedExprShapeV2::LogicalQuantifier {
                kind: quantifier.kind.into(),
                binders,
                body: body.expression().key(),
            }
        }
        BoolExprKindV1::RelationExists(exists) => {
            if position == AggregatePositionV2::RuleBody {
                return Err(document_failure(
                    "sema.invalid_item_shape",
                    source_id,
                    node.span,
                    "relational EXISTS is not admitted inside a source rule body",
                ));
            }
            if !bound.has_capture_set(node.id) {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "Binder omitted the explicit relational EXISTS capture set",
                ));
            }
            let relation = materialize_relation(
                &exists.relation,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
            )?;
            let relation_root = SourceQualifiedNodeIdV2 {
                source_id,
                node_id: exists.relation.id,
            };
            if relation.key() != relation_root {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    exists.relation.span,
                    "nested relational EXISTS root differs from checked HIR identity",
                ));
            }
            if nested_relations.insert(relation_root, relation).is_some() {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "duplicate nested relational EXISTS root",
                ));
            }
            let witness = capture_vector_witness(bound, node.id, exists.relation.id);
            let edges = validate_capture_vector(node.id, exists.relation.id, &witness, bound)
                .map_err(|message| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        message,
                    )
                })?;
            let mut correlations = Vec::new();
            for edge in edges {
                let Some(resolution) = bound
                    .resolutions()
                    .iter()
                    .find(|resolution| resolution.reference == edge.reference)
                else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        "Binder capture reference lacks its exact resolution proof",
                    ));
                };
                let ResolvedTargetV1::Local(binding) = &resolution.target else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        resolution.span,
                        "Binder capture target is not a local binding",
                    ));
                };
                if binding.identity() != (edge.owner, edge.ordinal) {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        resolution.span,
                        "Binder capture owner or ordinal differs from its resolution proof",
                    ));
                }
                let kind = match binding.kind {
                    BinderKindV1::Alias => ResolvedBindingKindV2::Alias,
                    BinderKindV1::Param => ResolvedBindingKindV2::Parameter,
                    BinderKindV1::Let => ResolvedBindingKindV2::Local,
                    BinderKindV1::Quantified => ResolvedBindingKindV2::Quantified,
                    BinderKindV1::CompositionLet => ResolvedBindingKindV2::Composition,
                };
                let binding = ResolvedValueBindingV2::try_new(
                    source_id,
                    edge.owner,
                    edge.ordinal,
                    kind,
                    binding.name.clone(),
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        resolution.span,
                        error,
                    )
                })?;
                let location = extended
                    .combined()
                    .try_check_location(location(source_id, resolution.span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            resolution.span,
                            error.to_string(),
                        )
                    })?;
                correlations.push(TypedCorrelationEdgeV2::new(
                    binding,
                    SourceQualifiedNodeIdV2 {
                        source_id,
                        node_id: edge.reference,
                    },
                    location,
                ));
            }
            TypedExprShapeV2::RelationExists {
                polarity: exists.polarity.into(),
                relation_root,
                correlations,
            }
        }
    };
    let raw = location(source_id, node.span);
    let context = AggregateContextV2::try_new(position, fact.effects.aggregation, false, raw)?;
    let checked = extended
        .combined()
        .try_check_location(raw)
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                node.span,
                error.to_string(),
            )
        })?;
    let typed = TypedExprV2::try_from_fact(key(node.id), fact, context, shape, checked)?;
    let bool_fact = TypedBoolExprV2::try_new(typed.clone(), raw)?;
    if output.insert(key(node.id), typed).is_some() {
        return Err(document_failure(
            "sema.document_validation_failed",
            source_id,
            node.span,
            "duplicate typed boolean identity",
        ));
    }
    Ok(bool_fact)
}

struct ViewMaterializationInputsV2<'a> {
    source_targets: &'a BTreeMap<HirNodeIdV1, ViewSourceTargetV2>,
    outputs: &'a BTreeMap<CatalogAssetIdV2, RelationOutputContractV2>,
}

fn materialize_relation(
    relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &ExtendedSourceRegistryViewV2,
    composition_outputs: &BTreeMap<(HirNodeIdV1, usize), RelationOutputContractV2>,
    views: &ViewMaterializationInputsV2<'_>,
    expressions: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &mut SemanticStepCounterV2,
) -> Result<TypedRelationExprV2, FrontendFailureV2> {
    use quanta_gqlang_v2_hir::{JoinHirV1, RelationExprHirV1, RelationExprKindV1};

    enum Frame<'a> {
        Visit(&'a RelationExprHirV1),
        Finish(
            &'a RelationExprHirV1,
            quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
        ),
    }

    let mut stack = vec![Frame::Visit(relation)];
    let mut completed = BTreeMap::new();
    while let Some(frame) = stack.pop() {
        match frame {
            Frame::Visit(node) => {
                steps.charge(source_id, node.span)?;
                let checked = extended
                    .combined()
                    .try_check_location(location(source_id, node.span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            node.span,
                            error.to_string(),
                        )
                    })?;
                if bound.relation_output(node.id).is_none() {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        "binder omitted relation output frame",
                    ));
                }
                stack.push(Frame::Finish(node, checked));
                match &node.kind {
                    RelationExprKindV1::Source(_) => {}
                    RelationExprKindV1::Project(project) => {
                        stack.push(Frame::Visit(&project.input));
                    }
                    RelationExprKindV1::Filter(filter) => {
                        stack.push(Frame::Visit(&filter.input));
                    }
                    RelationExprKindV1::Join(join) => {
                        let (left, right) = match join {
                            JoinHirV1::Inner { left, right, .. }
                            | JoinHirV1::Cross { left, right } => (left, right),
                        };
                        stack.push(Frame::Visit(right));
                        stack.push(Frame::Visit(left));
                    }
                    RelationExprKindV1::GroupBy(group) => {
                        stack.push(Frame::Visit(&group.input));
                    }
                    RelationExprKindV1::OrderBy(order) => {
                        stack.push(Frame::Visit(&order.input));
                    }
                }
            }
            Frame::Finish(node, checked) => {
                let typed = materialize_relation_node(
                    node,
                    checked,
                    source_id,
                    authority,
                    bound,
                    facts,
                    extended,
                    composition_outputs,
                    views,
                    expressions,
                    nested_relations,
                    steps,
                    &mut completed,
                )?;
                if completed.insert(node.id, typed).is_some() {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        "duplicate materialized relation identity",
                    ));
                }
            }
        }
    }
    completed.remove(&relation.id).ok_or_else(|| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            relation.span,
            "materialized relation root is missing",
        )
    })
}

fn take_materialized_relation(
    completed: &mut BTreeMap<HirNodeIdV1, TypedRelationExprV2>,
    child: &quanta_gqlang_v2_hir::RelationExprHirV1,
    source_id: SourceIdV2,
) -> Result<TypedRelationExprV2, FrontendFailureV2> {
    completed.remove(&child.id).ok_or_else(|| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            child.span,
            "materialized relation child is missing",
        )
    })
}

fn materialize_relation_node(
    relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
    checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &ExtendedSourceRegistryViewV2,
    composition_outputs: &BTreeMap<(HirNodeIdV1, usize), RelationOutputContractV2>,
    views: &ViewMaterializationInputsV2<'_>,
    expressions: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &mut SemanticStepCounterV2,
    completed: &mut BTreeMap<HirNodeIdV1, TypedRelationExprV2>,
) -> Result<TypedRelationExprV2, FrontendFailureV2> {
    let key = SourceQualifiedNodeIdV2 {
        source_id,
        node_id: relation.id,
    };
    match &relation.kind {
        quanta_gqlang_v2_hir::RelationExprKindV1::Source(source) => {
            if let Some(view) = views.source_targets.get(&relation.id) {
                let (scan, expected_row, cardinality, partition_keys, ordering, effects) =
                    match view {
                        ViewSourceTargetV2::Inline(view) => {
                            let previous = views.outputs.get(view).ok_or_else(|| {
                                document_failure(
                                    "sema.view_materialization_order",
                                    source_id,
                                    source.name_span,
                                    "View source was not available in Modules dependency-first order",
                                )
                            })?;
                            (
                                ResolvedScanSourceV2::View(view.clone()),
                                previous.row(),
                                previous.cardinality(),
                                previous.partition_keys().to_vec(),
                                previous.ordering().to_vec(),
                                previous.effects(),
                            )
                        }
                        ViewSourceTargetV2::External {
                            asset,
                            relation_type,
                        } => (
                            ResolvedScanSourceV2::View(asset.clone()),
                            relation_type.row(),
                            relation_type.cardinality(),
                            Vec::new(),
                            Vec::new(),
                            ExpressionEffectsV2::pure_scalar(),
                        ),
                    };
                let checked_row = facts.relation_row(key).ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        "types owner omitted View scan row fact",
                    )
                })?;
                if checked_row != expected_row {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        "View scan row differs from the sealed typed View contract",
                    ));
                }
                let use_location = extended
                    .combined()
                    .try_check_location(location(source_id, source.name_span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            source.name_span,
                            error.to_string(),
                        )
                    })?;
                let columns = expected_row
                    .fields()
                    .iter()
                    .map(|field| {
                        OutputColumnContractV2::new(
                            field.name().to_owned(),
                            OutputColumnOriginV2::SourceField {
                                source: scan.clone(),
                                scan_stage: key,
                                field_name: field.name().to_owned(),
                            },
                            use_location.clone(),
                        )
                    })
                    .collect();
                let output = RelationOutputContractV2::try_new(
                    checked_row.clone(),
                    OutputProvenanceV2::new([scan.clone()]),
                    columns,
                    cardinality,
                    partition_keys,
                    ordering,
                    effects,
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                })?;
                return TypedRelationExprV2::scan(key, checked, scan, output).map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                });
            }
            if let Some(target) = bound.target_of_use(relation.id, source.name_span, &source.name) {
                let ResolvedTargetV1::Local(binding) = target else {
                    return Err(document_failure(
                        "sema.invalid_relation_source_kind",
                        source_id,
                        source.name_span,
                        "resolved relation source is not a local Composition binding",
                    ));
                };
                if binding.kind != BinderKindV1::CompositionLet {
                    return Err(document_failure(
                        "sema.invalid_relation_source_kind",
                        source_id,
                        source.name_span,
                        "resolved local source is not a Composition relation binding",
                    ));
                }
                let identity = binding.identity();
                let previous = composition_outputs.get(&identity).ok_or_else(|| {
                    document_failure(
                        "sema.non_relation_composition_source",
                        source_id,
                        source.name_span,
                        "Composition binding has no prior sealed relation output",
                    )
                })?;
                let checked_row = facts.relation_row(key).ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        "types owner omitted Composition-local scan row fact",
                    )
                })?;
                if checked_row != previous.row() {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        "Composition-local scan row differs from the prior sealed producer",
                    ));
                }
                let composition_binding = ResolvedValueBindingV2::try_new(
                    source_id,
                    identity.0,
                    identity.1,
                    ResolvedBindingKindV2::Composition,
                    binding.name.clone(),
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        source.name_span,
                        error,
                    )
                })?;
                let scan = ResolvedScanSourceV2::Composition(
                    ResolvedCompositionSourceIdV2::try_from_binding(composition_binding).map_err(
                        |error| {
                            document_failure(
                                "sema.document_validation_failed",
                                source_id,
                                source.name_span,
                                error,
                            )
                        },
                    )?,
                );
                let use_location = extended
                    .combined()
                    .try_check_location(location(source_id, source.name_span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            source.name_span,
                            error.to_string(),
                        )
                    })?;
                let columns = previous
                    .row()
                    .fields()
                    .iter()
                    .map(|field| {
                        OutputColumnContractV2::new(
                            field.name().to_owned(),
                            OutputColumnOriginV2::SourceField {
                                source: scan.clone(),
                                scan_stage: key,
                                field_name: field.name().to_owned(),
                            },
                            use_location.clone(),
                        )
                    })
                    .collect();
                let output = RelationOutputContractV2::try_new(
                    checked_row.clone(),
                    OutputProvenanceV2::new([scan.clone()]),
                    columns,
                    previous.cardinality(),
                    previous.partition_keys().to_vec(),
                    previous.ordering().to_vec(),
                    previous.effects(),
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                })?;
                return TypedRelationExprV2::scan(key, checked, scan, output).map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                });
            }
            let host = authority.relations().get(&source.name).ok_or_else(|| {
                document_failure(
                    "sema.unresolved_reference",
                    source_id,
                    relation.span,
                    "relation source is absent from the sealed host catalog",
                )
            })?;
            let scan = ResolvedScanSourceV2::Relation(host.id().clone());
            let columns = host
                .relation_type()
                .row()
                .fields()
                .iter()
                .map(|field| {
                    OutputColumnContractV2::new(
                        field.name().to_owned(),
                        OutputColumnOriginV2::SourceField {
                            source: scan.clone(),
                            scan_stage: key,
                            field_name: field.name().to_owned(),
                        },
                        host.location().clone(),
                    )
                })
                .collect();
            let output = RelationOutputContractV2::try_new(
                host.relation_type().row().clone(),
                OutputProvenanceV2::new([scan.clone()]),
                columns,
                host.relation_type().cardinality(),
                Vec::new(),
                Vec::new(),
                ExpressionEffectsV2::pure_scalar(),
            )
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })?;
            TypedRelationExprV2::scan(key, checked, scan, output).map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })
        }
        quanta_gqlang_v2_hir::RelationExprKindV1::Filter(filter) => {
            let input = take_materialized_relation(completed, &filter.input, source_id)?;
            let predicate = materialize_bool(
                &filter.predicate,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                expressions,
                nested_relations,
                steps,
                AggregatePositionV2::Predicate,
            )?;
            let previous = input.output();
            let output = RelationOutputContractV2::try_new(
                previous.row().clone(),
                previous.provenance().clone(),
                previous.columns().to_vec(),
                previous.cardinality(),
                previous.partition_keys().to_vec(),
                previous.ordering().to_vec(),
                previous
                    .effects()
                    .compose([predicate.expression().effects()]),
            )
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })?;
            TypedRelationExprV2::filter(key, checked, input, predicate, output).map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })
        }
        quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) => {
            let input = take_materialized_relation(completed, &project.input, source_id)?;
            let frame = bound.relation_output(relation.id).ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    "binder omitted projection output frame",
                )
            })?;
            if frame.len() != project.items().len() {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    "binder projection output differs from authored item count",
                ));
            }
            let mut columns = Vec::new();
            let mut keys = Vec::new();
            let mut effects = Vec::new();
            let mut aggregate_classes = Vec::new();
            for (ordinal, (item, binding)) in project.items().iter().zip(frame).enumerate() {
                if binding.identity() != (relation.id, ordinal) {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        item.value.span,
                        "projection output does not retain its Binder-owned ordinal",
                    ));
                }
                let value = materialize_declaration_expr(
                    &item.value,
                    source_id,
                    bound,
                    facts,
                    extended,
                    expressions,
                    steps,
                    AggregatePositionV2::Projection,
                )?;
                let ExpressionTypeStateV2::Known(_) = value.type_state() else {
                    return Err(document_failure(
                        "sema.unknown_type_in_closed_compile",
                        source_id,
                        item.value.span,
                        "projection field has no concrete value type",
                    ));
                };
                let name_span = item
                    .alias
                    .as_ref()
                    .map_or(item.value.span, |alias| alias.span);
                let field_location = extended
                    .combined()
                    .try_check_location(location(source_id, name_span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            name_span,
                            error.to_string(),
                        )
                    })?;
                columns.push(OutputColumnContractV2::new(
                    binding.name.clone(),
                    OutputColumnOriginV2::Expression(value.key()),
                    field_location,
                ));
                aggregate_classes.push(value.aggregate_context().aggregation());
                keys.push(value.key());
                effects.push(value.effects());
            }
            let row = facts.relation_row(key).cloned().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    "types owner omitted projection row fact",
                )
            })?;
            let previous = input.output();
            // One aggregate classification over the complete projection
            // expression vector, via each expression's sealed
            // `AggregateContextV2`. Mixed and illegal shapes reject here,
            // before any aggregate-ready stage is constructed.
            let aggregate = aggregate_classes.contains(&AggregationClassV1::Aggregate);
            if !aggregate {
                let output = RelationOutputContractV2::try_new(
                    row,
                    previous.provenance().clone(),
                    columns,
                    previous.cardinality(),
                    Vec::new(),
                    Vec::new(),
                    previous.effects().compose(effects),
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                })?;
                return TypedRelationExprV2::try_project(key, checked, input, keys, output)
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            relation.span,
                            error,
                        )
                    });
            }
            let composed = previous.effects().compose(effects);
            if let TypedRelationExprV2::Group(_) = &input {
                // Explicit authored Group child: preserve its exact
                // partition-key IDs and partition-input proof. No second
                // Group is synthesized.
                let partition_keys = input.output().partition_keys().to_vec();
                let output = RelationOutputContractV2::try_aggregate(
                    row,
                    previous.provenance().clone(),
                    columns,
                    partition_keys.clone(),
                    composed,
                    &partition_keys,
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                })?;
                return TypedRelationExprV2::try_aggregate_project(
                    key, checked, input, keys, output,
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                });
            }
            // Scalar (global) aggregate over a non-Group input. Every column
            // must be aggregate-composed; a plain column cannot coexist with
            // a global fold.
            if let Some(plain) = aggregate_classes
                .iter()
                .position(|class| *class != AggregationClassV1::Aggregate)
            {
                return Err(document_failure(
                    "sema.mixed_aggregate_projection",
                    source_id,
                    project
                        .items()
                        .get(plain)
                        .map_or(relation.span, |item| item.value.span),
                    "global aggregate projection cannot retain a non-aggregated column",
                ));
            }
            // One private Sema-owned empty-partition Group wraps the pre-fold
            // input; the aggregate Project consumes it directly.
            steps.charge(source_id, relation.span)?;
            let group = TypedRelationExprV2::implicit_group(key, checked.clone(), input).map_err(
                |error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                },
            )?;
            let output = RelationOutputContractV2::try_aggregate(
                row,
                group.output().provenance().clone(),
                columns,
                Vec::new(),
                composed,
                &[],
            )
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })?;
            TypedRelationExprV2::try_aggregate_project(key, checked, group, keys, output).map_err(
                |error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                },
            )
        }
        quanta_gqlang_v2_hir::RelationExprKindV1::Join(join) => {
            let (left_node, right_node, predicate_node) = match join {
                quanta_gqlang_v2_hir::JoinHirV1::Inner { left, right, on } => {
                    (left, right, Some(on))
                }
                quanta_gqlang_v2_hir::JoinHirV1::Cross { left, right } => (left, right, None),
            };
            let left = take_materialized_relation(completed, left_node, source_id)?;
            let right = take_materialized_relation(completed, right_node, source_id)?;
            let predicate = predicate_node
                .map(|node| {
                    materialize_bool(
                        node,
                        source_id,
                        authority,
                        bound,
                        facts,
                        extended,
                        composition_outputs,
                        views,
                        expressions,
                        nested_relations,
                        steps,
                        AggregatePositionV2::Predicate,
                    )
                })
                .transpose()?;
            let left_output = left.output();
            let right_output = right.output();
            let mut columns = Vec::new();
            for (field, column) in left_output
                .row()
                .fields()
                .iter()
                .zip(left_output.columns())
                .chain(
                    right_output
                        .row()
                        .fields()
                        .iter()
                        .zip(right_output.columns()),
                )
            {
                let _ = field;
                columns.push(column.clone());
            }
            let row = facts.relation_row(key).cloned().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    "types owner omitted join row fact",
                )
            })?;
            let provenance = OutputProvenanceV2::new(
                left_output
                    .provenance()
                    .iter()
                    .cloned()
                    .chain(right_output.provenance().iter().cloned()),
            );
            let effects = ExpressionEffectsV2::pure_scalar().compose([
                left_output.effects(),
                right_output.effects(),
                predicate
                    .as_ref()
                    .map_or(ExpressionEffectsV2::pure_scalar(), |on| {
                        on.expression().effects()
                    }),
            ]);
            let output = RelationOutputContractV2::try_new(
                row,
                provenance,
                columns,
                quanta_gqlang_v2_types::CardinalityV1::Bag,
                Vec::new(),
                Vec::new(),
                effects,
            )
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })?;
            match predicate {
                Some(on) => TypedRelationExprV2::join(key, checked, left, right, on, output),
                None => TypedRelationExprV2::cross(key, checked, left, right, output),
            }
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })
        }
        quanta_gqlang_v2_hir::RelationExprKindV1::OrderBy(order) => {
            let input = take_materialized_relation(completed, &order.input, source_id)?;
            let mut keys = Vec::new();
            let mut ordering = Vec::new();
            let mut effects = Vec::new();
            for item in order.keys() {
                let value = materialize_declaration_expr(
                    &item.value,
                    source_id,
                    bound,
                    facts,
                    extended,
                    expressions,
                    steps,
                    AggregatePositionV2::OrderKey,
                )?;
                if value.effects().aggregation == AggregationClassV1::Aggregate {
                    return Err(document_failure(
                        "sema.illegal_aggregate_context",
                        source_id,
                        item.value.span,
                        "ORDER BY cannot introduce an aggregate outside the grouped output",
                    ));
                }
                let ExpressionTypeStateV2::Known(ty) = value.type_state() else {
                    return Err(document_failure(
                        "sema.unknown_type_in_closed_compile",
                        source_id,
                        item.value.span,
                        "order key has no concrete value type",
                    ));
                };
                keys.push(value.key());
                ordering.push(OrderKeyContractV2::new(
                    value.key(),
                    item.direction.into(),
                    NullOrderV2::Last,
                    ty.clone(),
                ));
                effects.push(value.effects());
            }
            let previous = input.output();
            let output = RelationOutputContractV2::try_new(
                previous.row().clone(),
                previous.provenance().clone(),
                previous.columns().to_vec(),
                quanta_gqlang_v2_types::CardinalityV1::Sequence,
                previous.partition_keys().to_vec(),
                ordering,
                previous.effects().compose(effects),
            )
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })?;
            TypedRelationExprV2::try_sort(key, checked, input, keys, output).map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })
        }
        quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(group) => {
            let input = take_materialized_relation(completed, &group.input, source_id)?;
            let frame = bound.relation_output(relation.id).ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    "binder omitted group key output frame",
                )
            })?;
            if frame.len() != group.keys().len()
                || bound.partition_input(relation.id) != bound.relation_output(group.input.id)
            {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    "group output and separately sealed partition input disagree with Binder",
                ));
            }
            let mut columns = Vec::new();
            let mut keys = Vec::new();
            let mut effects = Vec::new();
            for (ordinal, (key_expr, binding)) in group.keys().iter().zip(frame).enumerate() {
                if binding.identity() != (relation.id, ordinal) {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        key_expr.span,
                        "group key output lost its Binder-owned ordinal",
                    ));
                }
                let value = materialize_declaration_expr(
                    key_expr,
                    source_id,
                    bound,
                    facts,
                    extended,
                    expressions,
                    steps,
                    AggregatePositionV2::GroupKey,
                )?;
                let ExpressionTypeStateV2::Known(_) = value.type_state() else {
                    return Err(document_failure(
                        "sema.unknown_type_in_closed_compile",
                        source_id,
                        key_expr.span,
                        "group key has no concrete value type",
                    ));
                };
                let field_location = extended
                    .combined()
                    .try_check_location(location(source_id, binding.span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            binding.span,
                            error.to_string(),
                        )
                    })?;
                columns.push(OutputColumnContractV2::new(
                    binding.name.clone(),
                    OutputColumnOriginV2::Expression(value.key()),
                    field_location,
                ));
                keys.push(value.key());
                effects.push(value.effects());
            }
            let row = facts.relation_row(key).cloned().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    "types owner omitted group row fact",
                )
            })?;
            let output = RelationOutputContractV2::try_new(
                row,
                input.output().provenance().clone(),
                columns,
                quanta_gqlang_v2_types::CardinalityV1::Set,
                keys.clone(),
                Vec::new(),
                input.output().effects().compose(effects),
            )
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })?;
            let partition_input = bound.partition_input(relation.id).ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    "group has no Binder partition input",
                )
            })?;
            let partition_input = partition_input
                .iter()
                .map(|binding| {
                    let (owner, ordinal) = binding.identity();
                    let identity = ResolvedValueBindingV2::try_new(
                        source_id,
                        owner,
                        ordinal,
                        ResolvedBindingKindV2::Alias,
                        binding.name.clone(),
                    )
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            binding.span,
                            error,
                        )
                    })?;
                    let span = extended
                        .combined()
                        .try_check_location(location(source_id, binding.span))
                        .map_err(|error| {
                            document_failure(
                                "sema.document_validation_failed",
                                source_id,
                                binding.span,
                                error.to_string(),
                            )
                        })?;
                    PartitionInputBindingV2::try_new(identity, span).map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            binding.span,
                            error,
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            TypedRelationExprV2::try_group(key, checked, input, keys, partition_input, output)
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                })
        }
    }
}

fn materialize_query(
    query: &quanta_gqlang_v2_hir::QueryHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &ExtendedSourceRegistryViewV2,
    composition_outputs: &BTreeMap<(HirNodeIdV1, usize), RelationOutputContractV2>,
    views: &ViewMaterializationInputsV2<'_>,
    expressions: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &mut SemanticStepCounterV2,
) -> Result<TypedRelationV2, FrontendFailureV2> {
    if query.result_shape.is_some() {
        return Err(document_failure(
            "sema.document_validation_failed",
            source_id,
            query.span,
            "nested query result-shape resolution is not yet available",
        ));
    }
    let item_location = extended
        .combined()
        .try_check_location(location(source_id, query.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                query.span,
                error.to_string(),
            )
        })?;
    let root = materialize_relation(
        &query.body,
        source_id,
        authority,
        bound,
        facts,
        extended,
        composition_outputs,
        views,
        expressions,
        nested_relations,
        steps,
    )?;
    let output = root.output().clone();
    TypedRelationV2::try_new(
        SemanticItemIdV2::new(source_id, query.id),
        item_location,
        SourceQualifiedNodeIdV2 {
            source_id,
            node_id: query.body.id,
        },
        root,
        output,
    )
    .map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            query.span,
            error,
        )
    })
}

fn materialize_composition_sample(
    sample: &quanta_gqlang_v2_hir::SampleHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    steps: &mut SemanticStepCounterV2,
) -> Result<(ResolvedPresentationV2, RelationOutputContractV2), FrontendFailureV2> {
    let scope = sample
        .scope
        .as_ref()
        .map(|reference| {
            resolve_graph_target_use(
                sample.id,
                reference,
                crate::GraphReferenceKindV2::Scope,
                source_id,
                authority,
                bound,
                composition_targets,
                extended,
                steps,
            )
            .map(|(target, location)| ResolvedGraphUseV2::new(target, location))
        })
        .transpose()?;
    let location = extended
        .combined()
        .try_check_location(location(source_id, sample.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                sample.span,
                error.to_string(),
            )
        })?;
    let presentation = ResolvedPresentationV2::try_new(
        SemanticItemIdV2::new(source_id, sample.id),
        location.clone(),
        ResolvedPresentationKindV2::Sample {
            query_text: sample.query_text.clone(),
            mode: sample.mode,
            limit: sample.limit,
            scope,
        },
    )
    .map_err(|error| document_failure("sema.invalid_item_shape", source_id, sample.span, error))?;
    let field = quanta_gqlang_v2_types::RecordFieldTypeV1::try_new(
        "path",
        ValueTypeV1::scalar(ScalarTyV1::RepoPath),
        location.clone(),
    )
    .map_err(|error| {
        document_failure(
            "sema.invalid_item_shape",
            source_id,
            sample.span,
            error.to_string(),
        )
    })?;
    let row = quanta_gqlang_v2_types::RecordTypeV1::try_new(vec![field]).map_err(|error| {
        document_failure(
            "sema.invalid_item_shape",
            source_id,
            sample.span,
            error.to_string(),
        )
    })?;
    let output = RelationOutputContractV2::try_new(
        row,
        OutputProvenanceV2::default(),
        vec![OutputColumnContractV2::new(
            "path".to_owned(),
            OutputColumnOriginV2::Generated(presentation.item_id()),
            location,
        )],
        quanta_gqlang_v2_types::CardinalityV1::Set,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(|error| document_failure("sema.invalid_item_shape", source_id, sample.span, error))?;
    Ok((presentation, output))
}

fn materialize_composition_graph(
    graph: &quanta_gqlang_v2_hir::GraphQueryHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    steps: &mut SemanticStepCounterV2,
) -> Result<(ResolvedGraphQueryV2, Option<RelationOutputContractV2>), FrontendFailureV2> {
    let resolved = resolve_graph_item(
        graph,
        source_id,
        authority,
        bound,
        composition_targets,
        extended,
        steps,
    )?;
    if !matches!(
        graph.kind,
        quanta_gqlang_v2_hir::GraphQueryKindV1::Path
            | quanta_gqlang_v2_hir::GraphQueryKindV1::Reachability
    ) {
        return Ok((resolved, None));
    }
    let location = extended
        .combined()
        .try_check_location(location(source_id, graph.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                graph.span,
                error.to_string(),
            )
        })?;
    let row = quanta_gqlang_v2_types::RecordTypeV1::try_new(vec![
        quanta_gqlang_v2_types::RecordFieldTypeV1::try_new(
            "function",
            ValueTypeV1::scalar(ScalarTyV1::EntityRef),
            location.clone(),
        )
        .map_err(|error| {
            document_failure(
                "sema.invalid_item_shape",
                source_id,
                graph.span,
                error.to_string(),
            )
        })?,
    ])
    .map_err(|error| {
        document_failure(
            "sema.invalid_item_shape",
            source_id,
            graph.span,
            error.to_string(),
        )
    })?;
    let output = RelationOutputContractV2::try_new(
        row,
        OutputProvenanceV2::default(),
        vec![OutputColumnContractV2::new(
            "function".to_owned(),
            OutputColumnOriginV2::Generated(resolved.item_id()),
            location,
        )],
        quanta_gqlang_v2_types::CardinalityV1::Set,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(|error| document_failure("sema.invalid_item_shape", source_id, graph.span, error))?;
    Ok((resolved, Some(output)))
}

fn materialize_query_composition(
    composition: &quanta_gqlang_v2_hir::CompositionHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    views: &ViewMaterializationInputsV2<'_>,
    expressions: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedCompositionV2, FrontendTransactionErrorV2> {
    let item_id = SemanticItemIdV2::new(source_id, composition.id);
    let item_location = extended
        .combined()
        .try_check_location(location(source_id, composition.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                composition.span,
                error.to_string(),
            )
        })?;
    let mut outputs = BTreeMap::new();
    let mut bindings: Vec<ResolvedCompositionBindingV2> =
        Vec::with_capacity(composition.bindings.len());
    for (ordinal, authored) in composition.bindings.iter().enumerate() {
        let proof = bound
            .composition_binding(composition.id, ordinal)
            .ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    authored.name_span,
                    "Binder omitted exact Composition binding proof",
                )
            })?;
        if proof.identity() != (composition.id, ordinal)
            || proof.kind != BinderKindV1::CompositionLet
            || proof.name != authored.name
            || proof.span != authored.name_span
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                authored.name_span,
                "Binder Composition binding proof differs from checked HIR",
            )
            .into());
        }
        let binding = ResolvedValueBindingV2::try_new(
            source_id,
            proof.node,
            proof.identity().1,
            ResolvedBindingKindV2::Composition,
            proof.name.clone(),
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                authored.name_span,
                error,
            )
        })?;
        let name_location = extended
            .combined()
            .try_check_location(location(source_id, authored.name_span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    authored.name_span,
                    error.to_string(),
                )
            })?;
        let composition_graph_targets: BTreeMap<
            (HirNodeIdV1, usize),
            ResolvedCompositionBindingV2,
        > = bindings
            .iter()
            .map(|binding| {
                (
                    (binding.binding().target_node(), binding.binding().ordinal()),
                    binding.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let (producer, output, producer_id) = match &authored.producer {
            quanta_gqlang_v2_hir::CompositionProducerV1::Query(query) => {
                let typed = materialize_query(
                    query,
                    source_id,
                    authority,
                    bound,
                    facts,
                    extended,
                    &outputs,
                    views,
                    expressions,
                    nested_relations,
                    steps,
                )?;
                let output = typed.output().clone();
                let producer_id = typed.item_id();
                (
                    ResolvedCompositionProducerV2::Query(typed),
                    Some(output),
                    producer_id,
                )
            }
            quanta_gqlang_v2_hir::CompositionProducerV1::Sample(sample) => {
                let (presentation, output) = materialize_composition_sample(
                    sample,
                    source_id,
                    authority,
                    bound,
                    &composition_graph_targets,
                    extended,
                    steps,
                )?;
                let producer_id = presentation.item_id();
                let producer =
                    ResolvedCompositionProducerV2::try_sample(presentation, output.clone())
                        .map_err(|error| {
                            document_failure(
                                "sema.invalid_item_shape",
                                source_id,
                                sample.span,
                                error,
                            )
                        })?;
                (producer, Some(output), producer_id)
            }
            quanta_gqlang_v2_hir::CompositionProducerV1::GraphQuery(graph) => {
                let graph_span = graph.span;
                let (graph, output) = materialize_composition_graph(
                    graph,
                    source_id,
                    authority,
                    bound,
                    &composition_graph_targets,
                    extended,
                    steps,
                )?;
                let producer_id = graph.item_id();
                let producer = ResolvedCompositionProducerV2::try_graph(graph, output.clone())
                    .map_err(|error| {
                        document_failure("sema.invalid_item_shape", source_id, graph_span, error)
                    })?;
                (producer, output, producer_id)
            }
        };
        let resolved = ResolvedCompositionBindingV2::try_new(
            item_id,
            ordinal,
            producer_id,
            binding,
            name_location,
            producer,
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                authored.name_span,
                error,
            )
        })?;
        if let Some(output) = output {
            if outputs.insert((composition.id, ordinal), output).is_some() {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    authored.name_span,
                    "duplicate Composition producer identity",
                )
                .into());
            }
        }
        bindings.push(resolved);
    }
    let composition_graph_targets = bindings
        .iter()
        .map(|binding| {
            (
                (binding.binding().target_node(), binding.binding().ordinal()),
                binding.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let (body, expected_body_id) = match &composition.body {
        quanta_gqlang_v2_hir::CompositionBodyV1::Query(query) => {
            let body = materialize_query(
                query,
                source_id,
                authority,
                bound,
                facts,
                extended,
                &outputs,
                views,
                expressions,
                nested_relations,
                steps,
            )?;
            let expected_body_id = body.item_id();
            (
                ResolvedCompositionBodyV2::Query(Box::new(body)),
                expected_body_id,
            )
        }
        quanta_gqlang_v2_hir::CompositionBodyV1::GraphQuery(graph) => {
            let body = resolve_graph_item(
                graph,
                source_id,
                authority,
                bound,
                &composition_graph_targets,
                extended,
                steps,
            )?;
            let expected_body_id = body.item_id();
            (ResolvedCompositionBodyV2::Graph(body), expected_body_id)
        }
        quanta_gqlang_v2_hir::CompositionBodyV1::Legality(check) => {
            let body = resolve_check_item(
                check,
                source_id,
                authority,
                bound,
                &composition_graph_targets,
                extended,
                validator,
                steps,
            )?;
            let expected_body_id = body.item_id();
            (
                ResolvedCompositionBodyV2::Check(Box::new(body)),
                expected_body_id,
            )
        }
        quanta_gqlang_v2_hir::CompositionBodyV1::Explain(explain) => {
            let body = resolve_composition_explain(
                explain,
                source_id,
                authority,
                bound,
                &composition_graph_targets,
                extended,
                validator,
                steps,
            )?;
            let expected_body_id = body.item_id();
            (ResolvedCompositionBodyV2::Explain(body), expected_body_id)
        }
    };
    let with_proof = bound
        .target_of_use(
            composition.id,
            composition.with_name_span,
            &composition.with_name,
        )
        .ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                composition.with_name_span,
                "Binder omitted exact Composition WITH target",
            )
        })?;
    let ResolvedTargetV1::Local(with_proof) = with_proof else {
        return Err(document_failure(
            "sema.invalid_item_shape",
            source_id,
            composition.with_name_span,
            "Composition WITH target is not local",
        )
        .into());
    };
    if with_proof.kind != BinderKindV1::CompositionLet {
        return Err(document_failure(
            "sema.invalid_item_shape",
            source_id,
            composition.with_name_span,
            "Composition WITH target is not a LET binding",
        )
        .into());
    }
    let with_target = ResolvedValueBindingV2::try_new(
        source_id,
        with_proof.node,
        with_proof.identity().1,
        ResolvedBindingKindV2::Composition,
        with_proof.name.clone(),
    )
    .map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            composition.with_name_span,
            error,
        )
    })?;
    let with_location = extended
        .combined()
        .try_check_location(location(source_id, composition.with_name_span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                composition.with_name_span,
                error.to_string(),
            )
        })?;
    ResolvedCompositionV2::try_new(
        item_id,
        item_location,
        bindings,
        expected_body_id,
        body,
        with_target,
        with_location,
    )
    .map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            composition.span,
            error,
        )
    })
    .map_err(Into::into)
}

fn resolve_graph_target_use(
    owner: HirNodeIdV1,
    reference: &quanta_gqlang_v2_hir::GraphRefV1,
    required: crate::GraphReferenceKindV2,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    steps: &mut SemanticStepCounterV2,
) -> Result<
    (
        ResolvedGraphTargetV2,
        quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
    ),
    FrontendFailureV2,
> {
    steps.charge(source_id, reference.span)?;
    let location = location(source_id, reference.span);
    let checked = extended
        .combined()
        .try_check_location(location)
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                reference.span,
                error.to_string(),
            )
        })?;
    if let Some(target) = bound.target_of_use(owner, reference.span, &reference.name) {
        let ResolvedTargetV1::Local(binding) = target else {
            return Err(document_failure(
                "sema.invalid_composition_graph_target",
                source_id,
                reference.span,
                "resolved graph reference is not a local Composition binding",
            ));
        };
        if binding.kind != BinderKindV1::CompositionLet {
            return Err(document_failure(
                "sema.invalid_composition_graph_target",
                source_id,
                reference.span,
                "resolved graph reference is not a Composition binding",
            ));
        }
        let local = composition_targets
            .get(&binding.identity())
            .ok_or_else(|| {
                document_failure(
                    "sema.invalid_composition_graph_target",
                    source_id,
                    reference.span,
                    "Composition graph target is not a prior sealed producer",
                )
            })?;
        if local.binding().target_node() != binding.node
            || local.binding().ordinal() != binding.identity().1
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                reference.span,
                "Composition graph target differs from Binder binding proof",
            ));
        }
        let target = local.try_graph_target(required).map_err(|error| {
            document_failure(
                "sema.invalid_composition_graph_target",
                source_id,
                reference.span,
                error,
            )
        })?;
        return Ok((target, checked));
    }
    let target = authority
        .graph_references()
        .resolve(&reference.name, required, location)?;
    Ok((ResolvedGraphTargetV2::host(target), checked))
}

fn resolve_graph_item(
    graph: &quanta_gqlang_v2_hir::GraphQueryHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedGraphQueryV2, FrontendFailureV2> {
    let mut references = Vec::new();
    let endpoints = &graph.endpoints;
    let constraints = &graph.constraints;
    let singular = [
        (GraphReferenceRoleV2::Source, endpoints.source.as_ref()),
        (GraphReferenceRoleV2::Sink, endpoints.sink.as_ref()),
        (GraphReferenceRoleV2::Anchor, endpoints.anchor.as_ref()),
        (
            GraphReferenceRoleV2::Reference,
            endpoints.reference.as_ref(),
        ),
        (GraphReferenceRoleV2::Pointee, endpoints.pointee.as_ref()),
        (GraphReferenceRoleV2::Callable, endpoints.callable.as_ref()),
        (GraphReferenceRoleV2::Scope, constraints.scope.as_ref()),
        (GraphReferenceRoleV2::Window, constraints.window.as_ref()),
        (
            GraphReferenceRoleV2::Frontier,
            constraints.frontier.as_ref(),
        ),
    ];
    let repeated = constraints
        .via
        .iter()
        .map(|reference| (GraphReferenceRoleV2::Via, reference))
        .chain(
            constraints
                .through
                .iter()
                .map(|reference| (GraphReferenceRoleV2::Through, reference)),
        )
        .chain(
            constraints
                .sanitizers
                .iter()
                .map(|reference| (GraphReferenceRoleV2::Sanitizer, reference)),
        );
    for (role, reference) in singular
        .into_iter()
        .filter_map(|(role, reference)| reference.map(|reference| (role, reference)))
        .chain(repeated)
    {
        let expected = match role {
            GraphReferenceRoleV2::Source
            | GraphReferenceRoleV2::Sink
            | GraphReferenceRoleV2::Anchor
            | GraphReferenceRoleV2::Reference
            | GraphReferenceRoleV2::Pointee
            | GraphReferenceRoleV2::Via
            | GraphReferenceRoleV2::Through => crate::GraphReferenceKindV2::Entity,
            GraphReferenceRoleV2::Callable => crate::GraphReferenceKindV2::Callable,
            GraphReferenceRoleV2::Sanitizer => crate::GraphReferenceKindV2::Sanitizer,
            GraphReferenceRoleV2::Scope => crate::GraphReferenceKindV2::Scope,
            GraphReferenceRoleV2::Window => crate::GraphReferenceKindV2::Window,
            GraphReferenceRoleV2::Frontier => crate::GraphReferenceKindV2::Frontier,
        };
        let (target, checked) = resolve_graph_target_use(
            graph.id,
            reference,
            expected,
            source_id,
            authority,
            bound,
            composition_targets,
            extended,
            steps,
        )?;
        references.push(
            ResolvedGraphReferenceV2::try_new(role, target, checked).map_err(|error| {
                document_failure("sema.invalid_item_shape", source_id, reference.span, error)
            })?,
        );
    }
    let direction = constraints
        .direction
        .as_ref()
        .map(|direction| GraphDirectionV2::try_from_keyword(&direction.keyword))
        .transpose()
        .map_err(|error| {
            document_failure("sema.invalid_item_shape", source_id, graph.span, error)
        })?;
    if constraints
        .depth
        .is_some_and(|depth| depth > authority.budget().max_witness_depth())
    {
        return Err(document_failure(
            "frontend.resource_limit_exceeded",
            source_id,
            graph.span,
            "FIND depth exceeds the frontend witness-depth budget",
        ));
    }
    let graph_location = extended
        .combined()
        .try_check_location(location(source_id, graph.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                graph.span,
                error.to_string(),
            )
        })?;
    ResolvedGraphQueryV2::try_new(
        SemanticItemIdV2::new(source_id, graph.id),
        graph_location,
        graph.kind,
        references,
        constraints.depth,
        direction,
        constraints.summary_facets.clone(),
        GraphWitnessBoundsV2::new(
            authority.budget().max_witness_depth(),
            authority.budget().max_witnesses(),
        ),
    )
    .map_err(|error| document_failure("sema.invalid_item_shape", source_id, graph.span, error))
}

fn resolve_presentation_item(
    presentation: &quanta_gqlang_v2_hir::PresentationHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedPresentationV2, FrontendTransactionErrorV2> {
    let graph = |reference: &quanta_gqlang_v2_hir::GraphRefV1, kind| {
        let location = location(source_id, reference.span);
        let target = authority
            .graph_references()
            .resolve(&reference.name, kind, location)?;
        let checked = extended
            .combined()
            .try_check_location(location)
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    reference.span,
                    error.to_string(),
                )
            })?;
        Ok::<_, FrontendFailureV2>(ResolvedGraphUseV2::new(
            ResolvedGraphTargetV2::host(target),
            checked,
        ))
    };
    let kind = match &presentation.kind {
        quanta_gqlang_v2_hir::PresentationKindV1::Show { category } => {
            ResolvedPresentationKindV2::Show {
                category: *category,
            }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Describe { kind, path } => {
            steps.charge(source_id, path.span)?;
            let location = location(source_id, path.span);
            let target = authority
                .graph_references()
                .resolve_entity_subtype(&path.name, kind, location)?;
            let checked = extended
                .combined()
                .try_check_location(location)
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        path.span,
                        error.to_string(),
                    )
                })?;
            let target = ResolvedGraphUseV2::new(ResolvedGraphTargetV2::host(target), checked);
            ResolvedPresentationKindV2::Describe { target }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Sample {
            query_text,
            mode,
            limit,
            scope,
        } => {
            let scope = scope
                .as_ref()
                .map(|reference| {
                    steps.charge(source_id, reference.span)?;
                    graph(reference, crate::GraphReferenceKindV2::Scope)
                })
                .transpose()?;
            ResolvedPresentationKindV2::Sample {
                query_text: query_text.clone(),
                mode: *mode,
                limit: *limit,
                scope,
            }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Compare {
            baseline_ref,
            candidate_ref,
            metric,
        } => {
            steps.charge(source_id, baseline_ref.span)?;
            steps.charge(source_id, candidate_ref.span)?;
            ResolvedPresentationKindV2::Compare {
                baseline: graph(baseline_ref, crate::GraphReferenceKindV2::Snapshot)?,
                candidate: graph(candidate_ref, crate::GraphReferenceKindV2::Snapshot)?,
                metric: *metric,
            }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Export {
            source_ref,
            format,
            profile,
        } => {
            steps.charge(source_id, source_ref.span)?;
            let source = graph(source_ref, crate::GraphReferenceKindV2::ExportSource)?;
            let profile = profile
                .as_ref()
                .map(|reference| {
                    steps.charge(source_id, reference.span)?;
                    let checked = extended
                        .combined()
                        .try_check_location(location(source_id, reference.span))
                        .map_err(|error| {
                            FrontendTransactionErrorV2::Document(document_failure(
                                "sema.document_validation_failed",
                                source_id,
                                reference.span,
                                error.to_string(),
                            ))
                        })?;
                    let proof = validator
                        .resolve_reference(DocumentCatalogReferenceInputV2 {
                            module_id: None,
                            local_name: reference.name.clone(),
                            expected_kind: NamespaceKindV2::Profile,
                            location: checked,
                        })
                        .map_err(module_failure)?;
                    Ok::<_, FrontendTransactionErrorV2>(ResolvedCatalogReferenceV2::from_proof(
                        &proof,
                    ))
                })
                .transpose()?;
            ResolvedPresentationKindV2::Export {
                source,
                format: *format,
                profile,
            }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Explain {
            target_ref,
            evidence,
            truth,
        } => {
            steps.charge(source_id, target_ref.span)?;
            let raw = location(source_id, target_ref.span);
            let checked = extended
                .combined()
                .try_check_location(raw)
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        target_ref.span,
                        error.to_string(),
                    )
                })?;
            let rule = validator
                .probe_reference(DocumentCatalogReferenceInputV2 {
                    module_id: None,
                    local_name: target_ref.name.clone(),
                    expected_kind: NamespaceKindV2::Rule,
                    location: checked.clone(),
                })
                .map_err(module_failure)?;
            let entity = authority.graph_references().probe(
                &target_ref.name,
                crate::GraphReferenceKindV2::Entity,
                raw,
            )?;
            let target = match (rule, entity) {
                (Some(rule), None) => {
                    ResolvedExplainTargetV2::Rule(ResolvedCatalogReferenceV2::from_proof(&rule))
                }
                (None, Some(entity)) => ResolvedExplainTargetV2::Entity(ResolvedGraphUseV2::new(
                    ResolvedGraphTargetV2::host(entity),
                    checked,
                )),
                (Some(rule), Some(entity)) => {
                    return Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
                        FrontendStageV2::Resolution,
                        "sema.ambiguous_reference",
                        FrontendSeverityV2::Error,
                        raw,
                        vec![
                            rule.location().location(),
                            entity.declaration_location().location(),
                        ],
                        "EXPLAIN target resolves in both Rule and Entity namespaces",
                    ))
                    .into())
                }
                (None, None) => {
                    return Err(document_failure(
                        "sema.unresolved_reference",
                        source_id,
                        target_ref.span,
                        "EXPLAIN target is absent from Rule and Entity authorities",
                    )
                    .into())
                }
            };
            ResolvedPresentationKindV2::Explain {
                target,
                evidence: *evidence,
                truth: *truth,
            }
        }
    };
    ResolvedPresentationV2::try_new(
        SemanticItemIdV2::new(source_id, presentation.id),
        extended
            .combined()
            .try_check_location(location(source_id, presentation.span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    presentation.span,
                    error.to_string(),
                )
            })?,
        kind,
    )
    .map_err(|error| {
        document_failure(
            "sema.invalid_item_shape",
            source_id,
            presentation.span,
            error,
        )
        .into()
    })
}

fn resolve_composition_explain(
    explain: &quanta_gqlang_v2_hir::ExplainHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedPresentationV2, FrontendTransactionErrorV2> {
    let raw = location(source_id, explain.target_ref.span);
    let target = if bound
        .target_of_use(
            explain.id,
            explain.target_ref.span,
            &explain.target_ref.name,
        )
        .is_some()
    {
        let (target, location) = resolve_graph_target_use(
            explain.id,
            &explain.target_ref,
            crate::GraphReferenceKindV2::Entity,
            source_id,
            authority,
            bound,
            composition_targets,
            extended,
            steps,
        )?;
        ResolvedExplainTargetV2::Entity(ResolvedGraphUseV2::new(target, location))
    } else {
        steps.charge(source_id, explain.target_ref.span)?;
        let checked = extended
            .combined()
            .try_check_location(raw)
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    explain.target_ref.span,
                    error.to_string(),
                )
            })?;
        let rule = validator
            .probe_reference(DocumentCatalogReferenceInputV2 {
                module_id: None,
                local_name: explain.target_ref.name.clone(),
                expected_kind: NamespaceKindV2::Rule,
                location: checked.clone(),
            })
            .map_err(module_failure)?;
        let entity = authority.graph_references().probe(
            &explain.target_ref.name,
            crate::GraphReferenceKindV2::Entity,
            raw,
        )?;
        match (rule, entity) {
            (Some(rule), None) => {
                ResolvedExplainTargetV2::Rule(ResolvedCatalogReferenceV2::from_proof(&rule))
            }
            (None, Some(entity)) => ResolvedExplainTargetV2::Entity(ResolvedGraphUseV2::new(
                ResolvedGraphTargetV2::host(entity),
                checked,
            )),
            (Some(rule), Some(entity)) => {
                return Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
                    FrontendStageV2::Resolution,
                    "sema.ambiguous_reference",
                    FrontendSeverityV2::Error,
                    raw,
                    vec![
                        rule.location().location(),
                        entity.declaration_location().location(),
                    ],
                    "EXPLAIN target resolves in both Rule and Entity namespaces",
                ))
                .into())
            }
            (None, None) => {
                return Err(document_failure(
                    "sema.unresolved_reference",
                    source_id,
                    explain.target_ref.span,
                    "EXPLAIN target is absent from Rule and Entity authorities",
                )
                .into())
            }
        }
    };
    ResolvedPresentationV2::try_new(
        SemanticItemIdV2::new(source_id, explain.id),
        extended
            .combined()
            .try_check_location(location(source_id, explain.span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    explain.span,
                    error.to_string(),
                )
            })?,
        ResolvedPresentationKindV2::Explain {
            target,
            evidence: explain.evidence,
            truth: explain.truth,
        },
    )
    .map_err(|error| {
        document_failure("sema.invalid_item_shape", source_id, explain.span, error).into()
    })
}

fn resolve_check_item(
    check: &quanta_gqlang_v2_hir::LegalityHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedCheckV2, FrontendTransactionErrorV2> {
    steps.charge(source_id, check.rule_ref.span)?;
    let rule_location = location(source_id, check.rule_ref.span);
    let checked_rule = extended
        .combined()
        .try_check_location(rule_location)
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                check.rule_ref.span,
                error.to_string(),
            )
        })?;
    let rule = validator
        .resolve_reference(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: check.rule_ref.name.clone(),
            expected_kind: NamespaceKindV2::Rule,
            location: checked_rule,
        })
        .map_err(module_failure)?;
    let resolve_optional = |reference: Option<&quanta_gqlang_v2_hir::GraphRefV1>,
                            kind,
                            steps: &mut SemanticStepCounterV2| {
        reference
            .map(|reference| {
                resolve_graph_target_use(
                    check.id,
                    reference,
                    kind,
                    source_id,
                    authority,
                    bound,
                    composition_targets,
                    extended,
                    steps,
                )
                .map(|(target, checked)| ResolvedGraphUseV2::new(target, checked))
            })
            .transpose()
    };
    let scope = resolve_optional(
        check.scope.as_ref(),
        crate::GraphReferenceKindV2::Scope,
        steps,
    )?;
    let pattern = resolve_optional(
        check.pattern.as_ref(),
        crate::GraphReferenceKindV2::Pattern,
        steps,
    )?;
    let nested = check
        .semantic_confirm
        .as_ref()
        .map(|graph| {
            resolve_graph_item(
                graph,
                source_id,
                authority,
                bound,
                composition_targets,
                extended,
                steps,
            )
        })
        .transpose()?;
    let expected_nested = check
        .semantic_confirm
        .as_ref()
        .map(|graph| SemanticItemIdV2::new(source_id, graph.id));
    ResolvedCheckV2::try_new(
        SemanticItemIdV2::new(source_id, check.id),
        extended
            .combined()
            .try_check_location(location(source_id, check.span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    check.span,
                    error.to_string(),
                )
            })?,
        expected_nested,
        ResolvedCatalogReferenceV2::from_proof(&rule),
        scope,
        pattern,
        nested,
        check.confirm,
        check.emit,
        check.mode,
    )
    .map_err(|error| {
        document_failure("sema.invalid_item_shape", source_id, check.span, error).into()
    })
}

/// Projection of the modules-owned frozen graph into binder's read-only
/// imported-target interface; no precedence, search, or ID is recomputed.
struct ModuleImportAdapterV2<'a>(&'a ResolvedGraphV2);

impl ModuleGraphViewV1 for ModuleImportAdapterV2<'_> {
    fn imported_targets(&self, name: &str) -> Vec<String> {
        self.0
            .visible_bindings()
            .iter()
            .filter(|binding| {
                binding.namespace_kind() == NamespaceKindV2::Value && binding.local_name() == name
            })
            .map(|binding| {
                format!(
                    "{}::{}",
                    binding.resolved_asset_id().module_id().as_str(),
                    binding.resolved_asset_id().local_name()
                )
            })
            .collect()
    }
}

fn resolve_catalog_family_reference(
    reference: &quanta_gqlang_v2_hir::GraphRefV1,
    family: DocumentCatalogReferenceFamilyV2,
    source: &PredeclaredDocumentAssetV2,
    source_id: SourceIdV2,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
    edges: &mut Vec<DocumentCatalogFamilyReferenceInputV2>,
) -> Result<ResolvedCatalogReferenceV2, FrontendTransactionErrorV2> {
    steps.charge(source_id, reference.span)?;
    let checked = extended
        .combined()
        .try_check_location(location(source_id, reference.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                reference.span,
                error.to_string(),
            )
        })?;
    let target = validator
        .resolve_reference(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: reference.name.clone(),
            expected_kind: family.target_kind(),
            location: checked.clone(),
        })
        .map_err(module_failure)?;
    edges.push(DocumentCatalogFamilyReferenceInputV2 {
        family,
        source: source.clone(),
        target: target.clone(),
        location: checked,
    });
    Ok(ResolvedCatalogReferenceV2::from_proof(&target))
}

fn resolve_document_bodies(
    declared: &DeclaredDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    predeclared: &BTreeMap<HirNodeIdV1, PredeclaredDocumentAssetV2>,
    inline_views: &InlineViewPlanV2,
    topology: &DocumentViewTopologyProofV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<BodyResolutionV2, FrontendTransactionErrorV2> {
    let checked = declared.checked();
    let syntax = checked.checked().document();
    struct RuleRelationExistsV2(Option<SpanV1>);
    impl HirVisitorV1 for RuleRelationExistsV2 {
        fn visit_bool(&mut self, node: &quanta_gqlang_v2_hir::BoolExprHirV1) {
            if matches!(
                node.kind,
                quanta_gqlang_v2_hir::BoolExprKindV1::RelationExists(_)
            ) {
                if self.0.is_none() {
                    self.0 = Some(node.span);
                }
            } else {
                quanta_gqlang_v2_hir::walk_bool(self, node);
            }
        }
    }
    for item in &syntax.items {
        if let ProgramItemHirV1::Rule(rule) = item {
            if !matches!(rule.head.kind, ValueExprKindV1::Symbol(_)) {
                return Err(document_failure(
                    "sema.invalid_item_shape",
                    checked.source_id(),
                    rule.head.span,
                    "source rules require one symbol head term",
                )
                .into());
            }
            let mut forbidden = RuleRelationExistsV2(None);
            forbidden.visit_bool(&rule.body);
            if let Some(span) = forbidden.0 {
                return Err(document_failure(
                    "sema.invalid_item_shape",
                    checked.source_id(),
                    span,
                    "relational EXISTS is not admitted inside a source rule body",
                )
                .into());
            }
        }
    }
    // This first source-backed slice exercises the canonical binder and type
    // kernels. Unsupported root families remain explicit RED until their
    // complete owner-produced reference inputs can be supplied.
    if let Some(item) = syntax.items.iter().find(|item| {
        !matches!(
            item,
            ProgramItemHirV1::Query(_)
                | ProgramItemHirV1::Decl(_)
                | ProgramItemHirV1::Rule(_)
                | ProgramItemHirV1::Meta(_)
                | ProgramItemHirV1::GraphQuery(_)
                | ProgramItemHirV1::Presentation(_)
                | ProgramItemHirV1::Legality(_)
                | ProgramItemHirV1::CatalogDecl(_)
                | ProgramItemHirV1::Composition(_)
        )
    }) {
        return Err(document_failure(
            "sema.document_validation_failed",
            checked.source_id(),
            item.span(),
            "semantic body and reference owner resolution is not yet available",
        )
        .into());
    }
    let graph = ModuleImportAdapterV2(authority.modules().resolved_graph());
    let mut binder = Binder::new(&graph);
    let declarations = syntax
        .items
        .iter()
        .filter_map(|item| match item {
            ProgramItemHirV1::Decl(decl) => Some((**decl).clone()),
            ProgramItemHirV1::Query(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::CatalogDecl(_)
            | ProgramItemHirV1::Composition(_)
            | ProgramItemHirV1::Meta(_) => None,
        })
        .collect::<Vec<_>>();
    let rules = syntax
        .items
        .iter()
        .filter_map(|item| match item {
            ProgramItemHirV1::Rule(rule) => Some((**rule).clone()),
            ProgramItemHirV1::Query(_)
            | ProgramItemHirV1::Decl(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::CatalogDecl(_)
            | ProgramItemHirV1::Composition(_)
            | ProgramItemHirV1::Meta(_) => None,
        })
        .collect::<Vec<_>>();
    binder.declare_program(&declarations, &rules);
    for decl in &declarations {
        binder.resolve_decl(decl, &[]);
    }
    for rule in &rules {
        let ValueExprKindV1::Symbol(head) = &rule.head.kind else {
            return Err(document_failure(
                "sema.invalid_item_shape",
                checked.source_id(),
                rule.head.span,
                "source rules require one symbol head term",
            )
            .into());
        };
        binder.resolve_rule(rule, &[ParamV1::new(&head.name, rule.head.span, rule.id)]);
    }
    for item in &syntax.items {
        match item {
            ProgramItemHirV1::Query(query) => binder.resolve_query(query),
            ProgramItemHirV1::Composition(composition) => binder.resolve_composition(composition),
            ProgramItemHirV1::CatalogDecl(decl) => binder.resolve_catalog_decl(decl),
            ProgramItemHirV1::Decl(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::Meta(_) => {}
        }
    }
    let bound = binder.finish();
    if let Some(error) = bound.diagnostics().first() {
        return Err(document_failure(
            error.code(),
            checked.source_id(),
            error.primary_span(),
            error.to_string(),
        )
        .into());
    }
    let declaration_bodies = declarations
        .iter()
        .map(|decl| (decl.id, decl.body.id))
        .collect::<BTreeMap<_, _>>();
    let builtins = BuiltinRegistryV2::try_closed().map_err(type_failure)?;
    let mut rule_collector = RuleSymbolInputsV2 {
        source_id: checked.source_id(),
        bound: &bound,
        symbols: Vec::new(),
        calls: Vec::new(),
        error: None,
    };
    for rule in &rules {
        rule_collector.visit_rule(rule);
    }
    if let Some(error) = rule_collector.error {
        return Err(error.into());
    }
    let predicates = rules
        .iter()
        .map(|rule| {
            let predicate_id = ResolvedPredicateIdV2::try_new(&rule.name).map_err(type_failure)?;
            Ok::<_, FrontendTransactionErrorV2>(RulePredicateInputV2 {
                rule_node: SourceQualifiedNodeIdV2 {
                    source_id: checked.source_id(),
                    node_id: rule.id,
                },
                predicate_id,
                head_binding: RuleBindingIdV2::new(checked.source_id(), rule.id, 0),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut rule_calls = Vec::new();
    let mut host_call_proofs = Vec::new();
    for (call, name, span) in &rule_collector.calls {
        let source = predicates
            .iter()
            .find(|candidate| candidate.predicate_id.as_str() == name);
        let builtin = builtins.family_id(name);
        if source.is_some() || builtin.is_some() {
            let checked_location = extended
                .combined()
                .try_check_location(location(checked.source_id(), *span))
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        checked.source_id(),
                        *span,
                        error.to_string(),
                    )
                })?;
            if validator
                .probe_reference(DocumentCatalogReferenceInputV2 {
                    module_id: None,
                    local_name: name.clone(),
                    expected_kind: NamespaceKindV2::Predicate,
                    location: checked_location,
                })
                .map_err(module_failure)?
                .is_some()
            {
                return Err(document_failure(
                    "sema.ambiguous_callable",
                    checked.source_id(),
                    *span,
                    "source predicate or builtin family conflicts with visible host Predicate",
                )
                .into());
            }
        }
        let target = match (source, builtin) {
            (Some(_), Some(_)) => {
                return Err(document_failure(
                    "sema.ambiguous_callable",
                    checked.source_id(),
                    *span,
                    "source predicate and builtin family share one call spelling",
                )
                .into())
            }
            (Some(predicate), None) => {
                RuleCallableTargetV2::SourcePredicate(predicate.predicate_id.clone())
            }
            (None, Some(family)) => RuleCallableTargetV2::Builtin(family),
            (None, None) => {
                let checked_location = extended
                    .combined()
                    .try_check_location(location(checked.source_id(), *span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            checked.source_id(),
                            *span,
                            error.to_string(),
                        )
                    })?;
                let proof = validator
                    .resolve_host_predicate_call(DocumentCatalogReferenceInputV2 {
                        module_id: None,
                        local_name: name.clone(),
                        expected_kind: NamespaceKindV2::Predicate,
                        location: checked_location,
                    })
                    .map_err(module_failure)?;
                let target = RuleCallableTargetV2::HostPredicate(proof.callable().clone());
                host_call_proofs.push((*call, proof));
                target
            }
        };
        rule_calls.push(RuleCallInputV2 {
            call: *call,
            target,
        });
    }
    let rule_proof = if rules.is_empty() {
        None
    } else {
        let solver = RuleTypeSolverV2::try_closed().map_err(type_failure)?;
        let budget = RuleTypeBudgetV2::try_new(
            authority.budget().max_rule_graph_nodes(),
            authority.budget().max_rule_graph_edges(),
            authority.budget().max_rule_steps(),
        )
        .map_err(type_failure)?;
        Some(
            solver
                .solve(
                    checked.checked().as_checked(),
                    extended,
                    predicates.clone(),
                    rule_collector.symbols.clone(),
                    rule_calls.clone(),
                    budget,
                )
                .map_err(type_failure)?,
        )
    };
    let mut collector = DeclarationTypeInputsV2 {
        source_id: checked.source_id(),
        bound: &bound,
        declaration_bodies: &declaration_bodies,
        builtins: &builtins,
        entries: Vec::new(),
        error: None,
    };
    for decl in &declarations {
        collector.visit_value(&decl.body);
    }
    if let Some(error) = collector.error {
        return Err(error.into());
    }
    let mut query_inputs = QueryTypeInputsV2 {
        source_id: checked.source_id(),
        authority,
        bound: &bound,
        builtins: &builtins,
        extended,
        view_source_targets: &inline_views.source_targets,
        view_relation_roots: &inline_views.relation_roots,
        owners: BTreeMap::new(),
        entries: Vec::new(),
        rows: Vec::new(),
        composition_rows: BTreeMap::new(),
        error: None,
    };
    for asset in topology.dependency_first_order() {
        let Some(declaration) = inline_views.declarations.get(asset).copied() else {
            continue;
        };
        let query = inline_query_for_decl(syntax, declaration).ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                SpanV1::point(0),
                "topology-selected inline View lost its exact HIR query",
            )
        })?;
        query_inputs.visit_query(query);
    }
    for item in &syntax.items {
        match item {
            ProgramItemHirV1::Query(query) => query_inputs.visit_relation(&query.body),
            ProgramItemHirV1::Composition(composition) => {
                query_inputs.visit_composition(composition)
            }
            ProgramItemHirV1::Decl(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::CatalogDecl(_)
            | ProgramItemHirV1::Meta(_) => {}
        }
    }
    if let Some(error) = query_inputs.error {
        return Err(error.into());
    }
    collector.entries.extend(query_inputs.entries);
    if let Some(proof) = &rule_proof {
        for symbol in &rule_collector.symbols {
            let Some(ty) = proof.binding_type(symbol.binding) else {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    checked.source_id(),
                    SpanV1::point(0),
                    "rule solver omitted a Binder-identified concrete binding",
                )
                .into());
            };
            collector.entries.push(ResolvedTypeInputEntryV2 {
                key: symbol.reference,
                value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(ty.clone())),
            });
        }
        for call in &rule_calls {
            let callable = match &call.target {
                RuleCallableTargetV2::Builtin(family) => ResolvedCallableInputV2::Builtin(*family),
                RuleCallableTargetV2::SourcePredicate(id) => {
                    let Some(signature) = proof
                        .try_source_predicate_callable(id)
                        .map_err(type_failure)?
                    else {
                        return Err(document_failure(
                            "sema.document_validation_failed",
                            checked.source_id(),
                            SpanV1::point(0),
                            "source predicate signature is absent from rule type proof",
                        )
                        .into());
                    };
                    ResolvedCallableInputV2::Predicate(signature)
                }
                RuleCallableTargetV2::HostPredicate(signature) => {
                    ResolvedCallableInputV2::HostPredicate(signature.clone())
                }
                RuleCallableTargetV2::Unknown => {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        checked.source_id(),
                        SpanV1::point(0),
                        "unknown rule callable cannot enter closed types inputs",
                    )
                    .into())
                }
            };
            collector.entries.push(ResolvedTypeInputEntryV2 {
                key: call.call,
                value: ResolvedTypeInputValueV2::Callable(callable),
            });
        }
    }
    let pre_evaluation_roots = topology
        .dependency_first_order()
        .filter_map(|asset| inline_views.relation_roots.get(asset).copied())
        .collect();
    let type_inputs = ResolvedTypeInputsV2::try_new_with_relation_pre_evaluation(
        checked.checked().as_checked(),
        extended,
        collector.entries,
        Vec::new(),
        query_inputs.rows,
        RelationPreEvaluationInputV2::new(pre_evaluation_roots),
    )
    .map_err(type_failure)?;
    let kernel = ExpressionTypeKernelV2::try_closed().map_err(type_failure)?;
    let facts = kernel.check_document(&type_inputs).map_err(type_failure)?;
    let partition_witnesses = collect_partition_candidate_shapes(checked, &bound)?;
    for candidate in bound.partition_candidates() {
        check_partition_candidate_shape(
            &partition_witnesses,
            candidate.reference(),
            candidate.span(),
            candidate.group(),
            candidate.call_stack(),
        )
        .map_err(|message| {
            document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                candidate.span(),
                message,
            )
        })?;
        let Some(partition) = bound.partition_input(candidate.group()) else {
            return Err(document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                candidate.span(),
                "deferred group binding has no exact partition input",
            )
            .into());
        };
        if !partition.contains(candidate.binding())
            || !matches!(bound.target_of(candidate.reference()),
            Some(ResolvedTargetV1::Local(binding)) if binding == candidate.binding())
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                candidate.span(),
                "deferred aggregate argument differs from Binder partition identity",
            )
            .into());
        }
        if candidate.call_stack().is_empty() {
            return Err(document_failure(
                "sema.illegal_aggregate_context",
                checked.source_id(),
                candidate.span(),
                "partition-input name cannot escape into an ordinary projection",
            )
            .into());
        }
        for (index, &call_node) in candidate.call_stack().iter().enumerate() {
            let key = SourceQualifiedNodeIdV2 {
                source_id: checked.source_id(),
                node_id: call_node,
            };
            let Some(ResolvedCallTargetV2::Builtin(id)) = facts
                .get(key)
                .and_then(quanta_gqlang_v2_types::TypedExpressionFactV2::resolved_call)
            else {
                return Err(document_failure(
                    "sema.illegal_aggregate_context",
                    checked.source_id(),
                    candidate.span(),
                    "partition-input argument is not enclosed by exact builtin overloads",
                )
                .into());
            };
            let Some(intrinsic) = builtins.effects_of_resolved(*id) else {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    checked.source_id(),
                    candidate.span(),
                    "types owner omitted resolved builtin overload effects",
                )
                .into());
            };
            if (index == 0 && intrinsic.aggregation != AggregationClassV1::Aggregate)
                || (index > 0 && intrinsic.aggregation != AggregationClassV1::Scalar)
            {
                return Err(document_failure("sema.illegal_aggregate_context", checked.source_id(), candidate.span(),
                    "partition input is available only under one outer aggregate, never scalar-over-aggregate or nested aggregate").into());
            }
        }
    }
    reject_nested_aggregate_calls(checked, &facts, &builtins)?;
    for (call, proof) in &host_call_proofs {
        let target = facts
            .get(*call)
            .and_then(quanta_gqlang_v2_types::TypedExpressionFactV2::resolved_call);
        let Some(ResolvedCallTargetV2::HostPredicate(expected_key)) = target else {
            return Err(document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                proof.location().location().span,
                "types kernel omitted host predicate call target",
            )
            .into());
        };
        validator
            .validate_host_predicate_proof(proof, expected_key, proof.location())
            .map_err(module_failure)?;
    }
    let mut items = Vec::new();
    let mut expressions = BTreeMap::new();
    let mut nested_relations = BTreeMap::new();
    let mut certified = Vec::new();
    let mut activations = Vec::new();
    let mut family_references = Vec::new();
    let mut profile_admissions = Vec::new();
    let mut materialized_inline_views = BTreeMap::new();
    let mut inline_view_outputs = BTreeMap::new();
    let empty_composition_graph_targets = BTreeMap::new();
    for asset in topology.dependency_first_order() {
        let Some(declaration) = inline_views.declarations.get(asset).copied() else {
            continue;
        };
        let query = inline_query_for_decl(syntax, declaration).ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                SpanV1::point(0),
                "topology-selected inline View lost its exact HIR query",
            )
        })?;
        let available_views = ViewMaterializationInputsV2 {
            source_targets: &inline_views.source_targets,
            outputs: &inline_view_outputs,
        };
        let typed = materialize_query(
            query,
            checked.source_id(),
            authority,
            &bound,
            &facts,
            extended,
            &BTreeMap::new(),
            &available_views,
            &mut expressions,
            &mut nested_relations,
            steps,
        )?;
        if inline_view_outputs
            .insert(asset.clone(), typed.output().clone())
            .is_some()
            || materialized_inline_views
                .insert(asset.clone(), typed)
                .is_some()
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                SpanV1::point(0),
                "duplicate dependency-first inline View materialization",
            )
            .into());
        }
    }
    let available_views = ViewMaterializationInputsV2 {
        source_targets: &inline_views.source_targets,
        outputs: &inline_view_outputs,
    };
    for item in &checked.checked().document().items {
        steps.charge(checked.source_id(), item.span())?;
        match item {
            ProgramItemHirV1::Query(query) => {
                let typed = materialize_query(
                    query,
                    checked.source_id(),
                    authority,
                    &bound,
                    &facts,
                    extended,
                    &BTreeMap::new(),
                    &available_views,
                    &mut expressions,
                    &mut nested_relations,
                    steps,
                )?;
                items.push(SemanticItemV2::Relation(typed));
            }
            ProgramItemHirV1::GraphQuery(graph) => {
                items.push(SemanticItemV2::Graph(resolve_graph_item(
                    graph,
                    checked.source_id(),
                    authority,
                    &bound,
                    &empty_composition_graph_targets,
                    extended,
                    steps,
                )?));
            }
            ProgramItemHirV1::Presentation(presentation) => {
                items.push(SemanticItemV2::Presentation(resolve_presentation_item(
                    presentation,
                    checked.source_id(),
                    authority,
                    extended,
                    validator,
                    steps,
                )?));
            }
            ProgramItemHirV1::Legality(check) => {
                items.push(SemanticItemV2::Check(Box::new(resolve_check_item(
                    check,
                    checked.source_id(),
                    authority,
                    &bound,
                    &empty_composition_graph_targets,
                    extended,
                    validator,
                    steps,
                )?)));
            }
            ProgramItemHirV1::CatalogDecl(decl) => {
                let Some(asset) = predeclared.get(&decl.id) else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        checked.source_id(),
                        decl.span,
                        "catalog declaration lacks modules-owned predeclaration",
                    )
                    .into());
                };
                let mut resolve = |reference, family| {
                    resolve_catalog_family_reference(
                        reference,
                        family,
                        asset,
                        checked.source_id(),
                        extended,
                        validator,
                        steps,
                        &mut family_references,
                    )
                };
                let (payload, provider_metadata) = match &decl.payload {
                    quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                        quanta_gqlang_v2_hir::ViewBodyV1::QueryRef(reference),
                    ) => (
                        ResolvedCatalogPayloadV2::View {
                            query_ref: Some(resolve(
                                reference,
                                DocumentCatalogReferenceFamilyV2::ViewQuery,
                            )?),
                        },
                        None,
                    ),
                    quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                        quanta_gqlang_v2_hir::ViewBodyV1::Inline(_),
                    ) => (ResolvedCatalogPayloadV2::View { query_ref: None }, None),
                    quanta_gqlang_v2_hir::CatalogPayloadV1::Model { provider, source } => (
                        ResolvedCatalogPayloadV2::Model {
                            source: resolve(source, DocumentCatalogReferenceFamilyV2::ModelSource)?,
                        },
                        Some(provider.clone()),
                    ),
                    quanta_gqlang_v2_hir::CatalogPayloadV1::Pack { source, activation } => (
                        ResolvedCatalogPayloadV2::Pack {
                            source: resolve(source, DocumentCatalogReferenceFamilyV2::PackSource)?,
                            activation: *activation,
                        },
                        None,
                    ),
                    quanta_gqlang_v2_hir::CatalogPayloadV1::Profile {
                        capabilities,
                        admission,
                    } => {
                        let capabilities = capabilities
                            .iter()
                            .map(|reference| {
                                resolve(
                                    reference,
                                    DocumentCatalogReferenceFamilyV2::ProfileCapability,
                                )
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        let admission_proof = match admission {
                            quanta_gqlang_v2_hir::AdmissionV1::Allowlist => {
                                ProfileAdmissionV2::Allowlist
                            }
                            quanta_gqlang_v2_hir::AdmissionV1::ReviewRequired => {
                                ProfileAdmissionV2::ReviewRequired
                            }
                            quanta_gqlang_v2_hir::AdmissionV1::CertifiedOnly => {
                                ProfileAdmissionV2::CertifiedOnly
                            }
                        };
                        profile_admissions.push(ProfileAdmissionInputV2 {
                            profile: CatalogAssetRefInputV2 {
                                module_id: asset.id().module_id().clone(),
                                namespace_kind: NamespaceKindV2::Profile,
                                local_name: decl.name.clone(),
                                location: asset.location().clone(),
                            },
                            admission: admission_proof,
                            location: asset.location().clone(),
                        });
                        (
                            ResolvedCatalogPayloadV2::Profile {
                                capabilities,
                                admission: *admission,
                            },
                            None,
                        )
                    }
                    quanta_gqlang_v2_hir::CatalogPayloadV1::Gate { checks, binding } => {
                        let checks = checks
                            .iter()
                            .map(|reference| {
                                resolve(reference, DocumentCatalogReferenceFamilyV2::GateCheck)
                            })
                            .collect::<Result<Vec<_>, _>>()?;
                        (
                            ResolvedCatalogPayloadV2::Gate {
                                checks,
                                binding: *binding,
                            },
                            None,
                        )
                    }
                    quanta_gqlang_v2_hir::CatalogPayloadV1::RuleBinding { check_ref } => (
                        ResolvedCatalogPayloadV2::RuleBinding {
                            check_ref: resolve(
                                check_ref,
                                DocumentCatalogReferenceFamilyV2::RuleBindingCheck,
                            )?,
                        },
                        None,
                    ),
                };
                let (expected_inline_view_id, inline_view) = match &decl.payload {
                    quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                        quanta_gqlang_v2_hir::ViewBodyV1::Inline(query),
                    ) => {
                        let view = materialized_inline_views.get(asset.id()).cloned().ok_or_else(|| {
                            document_failure(
                                "sema.view_materialization_order",
                                checked.source_id(),
                                decl.span,
                                "inline View was absent from dependency-first materialization cache",
                            )
                        })?;
                        (
                            Some(SemanticItemIdV2::new(checked.source_id(), query.id)),
                            Some(view),
                        )
                    }
                    quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                        quanta_gqlang_v2_hir::ViewBodyV1::QueryRef(_),
                    )
                    | quanta_gqlang_v2_hir::CatalogPayloadV1::Model { .. }
                    | quanta_gqlang_v2_hir::CatalogPayloadV1::Pack { .. }
                    | quanta_gqlang_v2_hir::CatalogPayloadV1::Profile { .. }
                    | quanta_gqlang_v2_hir::CatalogPayloadV1::Gate { .. }
                    | quanta_gqlang_v2_hir::CatalogPayloadV1::RuleBinding { .. } => (None, None),
                };
                let catalog = ResolvedCatalogItemV2::try_new(
                    SemanticItemIdV2::new(checked.source_id(), decl.id),
                    asset.id().clone(),
                    provider_metadata,
                    expected_inline_view_id,
                    inline_view,
                    inline_views
                        .direct_references
                        .get(&decl.id)
                        .cloned()
                        .unwrap_or_default(),
                    payload,
                )
                .map_err(|error| {
                    document_failure(
                        "sema.invalid_item_shape",
                        checked.source_id(),
                        decl.span,
                        error,
                    )
                })?;
                items.push(SemanticItemV2::Catalog(catalog));
            }
            ProgramItemHirV1::Decl(decl) => {
                let typed = materialize_declaration_expr(
                    &decl.body,
                    checked.source_id(),
                    &bound,
                    &facts,
                    extended,
                    &mut expressions,
                    steps,
                    AggregatePositionV2::Scalar,
                )?;
                let declaration = ResolvedDeclarationV2::try_new(
                    SemanticItemIdV2::new(checked.source_id(), decl.id),
                    decl.name.clone(),
                    typed,
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        checked.source_id(),
                        decl.span,
                        error,
                    )
                })?;
                items.push(SemanticItemV2::Declaration(declaration));
            }
            ProgramItemHirV1::Rule(rule) => {
                let Some(predicate) = predicates
                    .iter()
                    .find(|predicate| predicate.rule_node.node_id == rule.id)
                else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        checked.source_id(),
                        rule.span,
                        "rule lacks a types-owned predicate predeclaration",
                    )
                    .into());
                };
                let head = materialize_declaration_expr(
                    &rule.head,
                    checked.source_id(),
                    &bound,
                    &facts,
                    extended,
                    &mut expressions,
                    steps,
                    AggregatePositionV2::RuleBody,
                )?;
                let body = materialize_bool(
                    &rule.body,
                    checked.source_id(),
                    authority,
                    &bound,
                    &facts,
                    extended,
                    &BTreeMap::new(),
                    &available_views,
                    &mut expressions,
                    &mut nested_relations,
                    steps,
                    AggregatePositionV2::RuleBody,
                )?;
                let Some(proof) = &rule_proof else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        checked.source_id(),
                        rule.span,
                        "rule type proof is absent from the document transaction",
                    )
                    .into());
                };
                if !matches!(head.type_state(), ExpressionTypeStateV2::Known(ty)
                    if proof.predicate_type(&predicate.predicate_id) == Some(ty))
                {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        checked.source_id(),
                        rule.head.span,
                        "rule head type disagrees with the predicate signature proof",
                    )
                    .into());
                }
                let calls = classify_rule_body_calls(
                    body.expression().key(),
                    |key| expressions.get(&key),
                    |key| nested_relations.get(&key),
                )
                .map_err(|error| {
                    document_failure(
                        "sema.invalid_item_shape",
                        checked.source_id(),
                        error
                            .location()
                            .map_or(rule.body.span, |location| location.location().span),
                        error.reason(),
                    )
                })?;
                let typed = ResolvedRuleV2::try_new(
                    SemanticItemIdV2::new(checked.source_id(), rule.id),
                    head.key(),
                    body.expression().key(),
                    predicate.predicate_id.clone(),
                    head,
                    body,
                    calls,
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        checked.source_id(),
                        rule.span,
                        error,
                    )
                })?;
                items.push(SemanticItemV2::Rule(typed));
            }
            ProgramItemHirV1::Meta(meta)
                if matches!(
                    meta.kind,
                    quanta_gqlang_v2_hir::MetaKindV1::RequireCertified
                ) =>
            {
                let location = quanta_gqlang_v2_hir::LocatedSpanV2 {
                    source_id: checked.source_id(),
                    span: meta.span,
                };
                let checked_location =
                    extended
                        .combined()
                        .try_check_location(location)
                        .map_err(|error| {
                            document_failure(
                                "sema.document_validation_failed",
                                checked.source_id(),
                                meta.span,
                                error.to_string(),
                            )
                        })?;
                certified.push(RequireCertifiedInputV2 {
                    location: checked_location.clone(),
                });
                items.push(SemanticItemV2::Meta(ResolvedMetaV2::new(
                    SemanticItemIdV2::new(checked.source_id(), meta.id),
                    checked_location,
                    ResolvedMetaKindV2::RequireCertified,
                )));
            }
            ProgramItemHirV1::Meta(meta) => {
                let quanta_gqlang_v2_hir::MetaKindV1::UsePack(use_pack) = &meta.kind else {
                    unreachable!("REQUIRE CERTIFIED is handled by the preceding arm");
                };
                let checked_location = extended
                    .combined()
                    .try_check_location(location(checked.source_id(), meta.span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            checked.source_id(),
                            meta.span,
                            error.to_string(),
                        )
                    })?;
                let pack = PackIdV2::try_new(&use_pack.pack_id, checked_location.clone())
                    .map_err(module_failure)?;
                let activation = match use_pack.activation {
                    quanta_gqlang_v2_hir::UsePackActivationV1::Session => {
                        UsePackActivationV2::Session
                    }
                    quanta_gqlang_v2_hir::UsePackActivationV1::Workspace => {
                        UsePackActivationV2::Workspace
                    }
                };
                activations.push(PackActivationInputV2 {
                    pack_id: pack.clone(),
                    activation,
                    lock_reference: LockReferenceInputV2 {
                        path: use_pack.lockfile.clone(),
                        location: checked_location.clone(),
                    },
                    location: checked_location.clone(),
                });
                items.push(SemanticItemV2::Meta(ResolvedMetaV2::new(
                    SemanticItemIdV2::new(checked.source_id(), meta.id),
                    checked_location,
                    ResolvedMetaKindV2::UsePack {
                        pack,
                        activation: use_pack.activation,
                    },
                )));
            }
            ProgramItemHirV1::Composition(composition) => {
                let composition = materialize_query_composition(
                    composition,
                    checked.source_id(),
                    authority,
                    &bound,
                    &facts,
                    extended,
                    validator,
                    &available_views,
                    &mut expressions,
                    &mut nested_relations,
                    steps,
                )?;
                items.push(SemanticItemV2::Composition(composition));
            }
        }
    }
    Ok(BodyResolutionV2 {
        items,
        expressions,
        nested_relations,
        type_fact_count: facts.len(),
        catalog_input: DocumentCatalogFinalInputV2 {
            profile_admissions,
            activations,
            require_certified_directives: certified,
            family_references,
        },
    })
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::indexing_slicing,
        clippy::panic,
        clippy::unwrap_used,
        reason = "owner assertions inspect exact transaction shapes and fail on any mismatch"
    )]

    use super::*;

    #[test]
    fn partition_candidate_shape_rejects_every_transplanted_identity_dimension() {
        let reference = HirNodeIdV1::from_raw(11);
        let group = HirNodeIdV1::from_raw(7);
        let outer = HirNodeIdV1::from_raw(13);
        let inner = HirNodeIdV1::from_raw(17);
        let span = SpanV1::new(40, 41);
        let witnesses = BTreeMap::from([(
            reference,
            PartitionCandidateShapeV2 {
                span,
                direct_group: group,
                call_arguments: vec![outer, inner],
            },
        )]);
        assert!(check_partition_candidate_shape(
            &witnesses,
            reference,
            span,
            group,
            &[outer, inner],
        )
        .is_ok());
        for (candidate_reference, candidate_span, candidate_group, candidate_calls) in [
            (HirNodeIdV1::from_raw(12), span, group, vec![outer, inner]),
            (reference, SpanV1::new(41, 42), group, vec![outer, inner]),
            (
                reference,
                span,
                HirNodeIdV1::from_raw(8),
                vec![outer, inner],
            ),
            (
                reference,
                span,
                group,
                vec![HirNodeIdV1::from_raw(14), inner],
            ),
            (reference, span, group, vec![outer]),
            (
                reference,
                span,
                group,
                vec![outer, HirNodeIdV1::from_raw(18)],
            ),
        ] {
            assert!(check_partition_candidate_shape(
                &witnesses,
                candidate_reference,
                candidate_span,
                candidate_group,
                &candidate_calls,
            )
            .is_err());
        }
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "owner test assertions must fail on a budget mismatch"
    )]
    fn semantic_steps_fail_at_exact_bound_with_source_location() -> Result<(), String> {
        let budget = FrontendBudgetV2::try_new(
            10, 10, 10, 10, 10, 10, 10, 10, 10, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
        )
        .map_err(|error| format!("{error:?}"))?;
        let mut steps = SemanticStepCounterV2::new(&budget);
        let source_id = SourceIdV2::from_raw(7);
        let span = SpanV1::new(1, 2);
        assert!(steps.charge(source_id, span).is_ok());
        assert!(steps.charge(source_id, span).is_ok());
        let error = steps
            .charge(source_id, span)
            .err()
            .ok_or("third semantic node must exceed two-step bound")?;
        let diagnostic = error
            .diagnostics()
            .first()
            .ok_or("resource failure must have one diagnostic")?;
        assert_eq!(diagnostic.code(), "frontend.resource_limit_exceeded");
        assert_eq!(diagnostic.primary(), location(source_id, span));
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "owner test assertions must reject forged incomplete states"
    )]
    fn incomplete_analysis_canonicalizes_valid_input_and_rejects_invalid_states(
    ) -> Result<(), String> {
        let source = SourceIdV2::from_raw(9);
        let earlier = UnresolvedSemanticFactV2::new("sema.a", location(source, SpanV1::new(1, 2)));
        let later = UnresolvedSemanticFactV2::new("sema.b", location(source, SpanV1::new(3, 4)));
        let warning = FrontendDiagnosticV2::new(
            FrontendStageV2::Resolution,
            "sema.warning",
            FrontendSeverityV2::Warning,
            location(source, SpanV1::new(3, 4)),
            Vec::new(),
            "warning",
        );
        let note = FrontendDiagnosticV2::new(
            FrontendStageV2::Resolution,
            "sema.note",
            FrontendSeverityV2::Note,
            location(source, SpanV1::new(1, 2)),
            Vec::new(),
            "note",
        );
        let canonical = IncompleteSemanticDocumentV2::try_new(
            source,
            vec![later.clone(), earlier.clone()],
            vec![warning.clone(), note.clone()],
        )
        .map_err(str::to_owned)?;
        assert_eq!(canonical.unresolved(), &[earlier.clone(), later]);
        assert_eq!(canonical.diagnostics(), &[note, warning.clone()]);
        assert!(IncompleteSemanticDocumentV2::try_new(
            source,
            vec![earlier.clone(), earlier],
            Vec::new(),
        )
        .is_err());
        assert!(IncompleteSemanticDocumentV2::try_new(
            source,
            vec![UnresolvedSemanticFactV2::new(
                "sema.foreign",
                location(SourceIdV2::from_raw(10), SpanV1::new(1, 2)),
            )],
            Vec::new(),
        )
        .is_err());
        assert!(IncompleteSemanticDocumentV2::try_new(
            source,
            vec![UnresolvedSemanticFactV2::new(
                "sema.live",
                location(source, SpanV1::new(1, 2))
            )],
            vec![warning.clone(), warning],
        )
        .is_err());
        assert!(IncompleteSemanticDocumentV2::try_new(
            source,
            vec![UnresolvedSemanticFactV2::new(
                "sema.live",
                location(source, SpanV1::new(1, 2))
            )],
            vec![FrontendDiagnosticV2::new(
                FrontendStageV2::Validation,
                "sema.error",
                FrontendSeverityV2::Error,
                location(source, SpanV1::new(1, 2)),
                Vec::new(),
                "error",
            )],
        )
        .is_err());
        Ok(())
    }

    // ---- SS-03H-S + SS-04W owner oracles ---------------------------------

    use quanta_gqlang_v2_hir::{
        AliasV1, CallArgHirV1, CallArgsHirV1, CatalogDeclHirV1, CatalogPayloadV1,
        CompositionBindingV1, CompositionBodyV1, CompositionHirV1, CompositionProducerV1,
        DeclHirV1, HirBuilderV1, LocatedSpanV2, PathSegmentV1, ProjectionItemV1,
        RelationExistsPolarityV1, SourceRegistrationV2, SourceRegistryViewV2, ViewBodyV1,
    };
    use quanta_gqlang_v2_modules::{
        CurrentModuleSelectionInputV2, ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2,
        VersionV2,
    };
    use quanta_gqlang_v2_types::{
        CardinalityV1, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1, RelationCatalogV2,
        RelationSourceInputV2,
    };

    struct EmptyModuleGraphV1;
    impl ModuleGraphViewV1 for EmptyModuleGraphV1 {
        fn imported_targets(&self, _name: &str) -> Vec<String> {
            Vec::new()
        }
    }

    fn owner_authority(budget: &FrontendBudgetV2) -> Result<ClosedFrontendAuthorityV2, String> {
        let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: 512,
        }])
        .map_err(|error| format!("{error:?}"))?;
        let location = host
            .try_check_location(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(1),
                span: SpanV1::new(0, 1),
            })
            .map_err(|error| format!("{error:?}"))?;
        let module = ModuleIdV2::try_new("app", location.clone()).map_err(|e| format!("{e:?}"))?;
        let profile = budget
            .authority_profile()
            .map_err(|error| format!("{error:?}"))?;
        let mut modules = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone())
            .map_err(|error| format!("{error:?}"))?;
        modules
            .try_add_module(ModuleNodeInputV2 {
                module_id: module.clone(),
                location: location.clone(),
            })
            .map_err(|error| format!("{error:?}"))?;
        let modules = modules
            .finish(
                CurrentModuleSelectionInputV2 {
                    module_id: module,
                    location: location.clone(),
                },
                VersionV2::new(1, 0, 0),
            )
            .map_err(|error| error.to_string())?;
        let field = RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), location.clone())
            .map_err(|error| format!("{error:?}"))?;
        let row = RecordTypeV1::try_new(vec![field]).map_err(|error| format!("{error:?}"))?;
        let relations = RelationCatalogV2::try_new(
            vec![RelationSourceInputV2 {
                authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
                provenance_namespace: ProvenanceNamespaceV2::try_new("host", location.clone())
                    .map_err(|error| format!("{error:?}"))?,
                qualified_name: "records".into(),
                lookup_name: "records".into(),
                row,
                cardinality: CardinalityV1::Bag,
                provenance: "host".into(),
                location: location.clone(),
            }],
            &host,
            profile.clone(),
        )
        .map_err(|error| format!("{error:?}"))?;
        let graphs = GraphReferenceCatalogV2::try_new(Vec::new(), &host, profile)
            .map_err(|error| format!("{error:?}"))?;
        ClosedFrontendAuthorityV2::try_new(modules, relations, graphs, budget.clone())
            .map_err(|error| format!("{error:?}"))
    }

    /// One `from records as r | project count(*)` document: the legal scalar
    /// aggregate shape, including the star call node identity.
    fn scalar_count_star_document() -> (HirDocumentV1, HirNodeIdV1, HirNodeIdV1) {
        let builder = HirBuilderV1::new();
        let scan = builder.relation_source(
            "records",
            SpanV1::new(5, 12),
            Some(AliasV1 {
                name: "r".into(),
                span: SpanV1::new(13, 14),
            }),
            SpanV1::new(5, 14),
        );
        let scan_id = scan.id;
        let count = builder.call(
            "count",
            SpanV1::new(24, 29),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Star {
                span: SpanV1::new(30, 31),
            }])
            .unwrap(),
            SpanV1::new(24, 31),
        );
        let count_id = count.id;
        let project = builder
            .try_project(
                scan,
                vec![ProjectionItemV1 {
                    value: count,
                    alias: Some(AliasV1 {
                        name: "total".into(),
                        span: SpanV1::new(36, 41),
                    }),
                }],
                SpanV1::new(16, 41),
            )
            .unwrap();
        let query = builder.query(project, None, SpanV1::new(0, 41));
        (
            HirDocumentV1 {
                source_id: SourceIdV2::from_raw(9),
                source_len: 64,
                items: vec![ProgramItemHirV1::Query(Box::new(query))],
            },
            scan_id,
            count_id,
        )
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "owner assertions inspect the sealed scalar aggregate shape"
    )]
    fn scalar_count_star_seals_one_private_empty_partition_group() -> Result<(), String> {
        let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
        let authority = owner_authority(&budget)?;
        let (document, scan_id, count_id) = scalar_count_star_document();
        let document = resolve_checked_document_v2(
            CheckedSyntaxDocumentV2::try_from_hir(document)
                .map_err(|error| format!("{error:?}"))?,
            &authority,
        )
        .map_err(|error| format!("{error:?}"))?;
        let item = document
            .items()
            .next()
            .map(|(_, item)| item)
            .unwrap_or_else(|| {
                panic!("one relation item must be sealed");
            });
        let SemanticItemV2::Relation(relation) = item else {
            panic!("scalar aggregate must retain its typed relation item");
        };
        // Exact Group(empty) -> Project(aggregate) shape; no caller option.
        let TypedRelationExprV2::Project(_) = relation.root() else {
            panic!("aggregate projection root expected");
        };
        let group = relation.root().inputs().next().unwrap();
        let TypedRelationExprV2::Group(_) = group else {
            panic!("aggregate projection must consume one Group");
        };
        assert!(group.expressions().is_empty());
        assert!(group.partition_input().is_empty());
        assert_eq!(group.output().cardinality(), CardinalityV1::Set);
        assert!(group.output().partition_keys().is_empty());
        assert!(group.output().columns().is_empty());
        assert!(group.output().row().fields().is_empty());
        // Pre-fold input contract: the exact pre-fold scan stage with its row.
        let scan = group.inputs().next().unwrap();
        let TypedRelationExprV2::Scan(_) = scan else {
            panic!("implicit group must fold the exact prior scan");
        };
        assert_eq!(scan.key().node_id, scan_id);
        assert_eq!(scan.output().row().fields().len(), 1);
        // Aggregate output contract.
        assert_eq!(relation.output().cardinality(), CardinalityV1::Set);
        assert!(relation.output().partition_keys().is_empty());
        assert!(relation.output().ordering().is_empty());
        assert_eq!(relation.output().columns().len(), 1);
        // Star retained and the resolved aggregate call stays a types fact.
        let count_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: count_id,
        };
        let count = document.expression(count_key).unwrap();
        let TypedExprShapeV2::Call { arguments, .. } = count.shape() else {
            panic!("count(*) must retain its call shape");
        };
        assert_eq!(arguments, &[TypedCallArgumentV2::Star]);
        assert_eq!(
            count.aggregate_context().aggregation(),
            AggregationClassV1::Aggregate
        );
        assert!(count.aggregate_context().uses_star());
        assert_eq!(
            relation.root().expressions().first().map(|key| key.node_id),
            Some(count_id)
        );
        Ok(())
    }

    #[test]
    fn bare_star_outside_an_aggregate_call_is_rejected_by_the_position_contract() {
        let raw = location(SourceIdV2::from_raw(9), SpanV1::new(3, 4));
        // A bare star never carries aggregate effects, so the central
        // aggregate-position contract rejects it before any relation stage.
        let error = AggregateContextV2::try_new(
            AggregatePositionV2::Projection,
            AggregationClassV1::Scalar,
            true,
            raw,
        )
        .unwrap_err();
        assert_eq!(
            error.diagnostics().first().unwrap().code(),
            "sema.illegal_aggregate_star"
        );
        assert_eq!(
            error.diagnostics().first().unwrap().primary().span,
            SpanV1::new(3, 4)
        );
    }

    #[test]
    fn mixed_document_transaction_resolves_all_items() -> Result<(), String> {
        let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
        let authority = owner_authority(&budget)?;
        let builder = HirBuilderV1::new();
        // Declaration.
        let decl = DeclHirV1::new(
            HirNodeIdV1::from_raw(900),
            SpanV1::new(0, 16),
            "total".into(),
            SpanV1::new(4, 9),
            builder.int("42", SpanV1::new(12, 14)),
        );
        // Plain query with a projection.
        let plain_scan = builder.relation_source(
            "records",
            SpanV1::new(24, 31),
            Some(AliasV1 {
                name: "r".into(),
                span: SpanV1::new(32, 33),
            }),
            SpanV1::new(24, 33),
        );
        let plain_key = builder
            .try_path(
                builder.symbol("r", SpanV1::new(42, 43)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(44, 46),
                }],
                SpanV1::new(42, 46),
            )
            .unwrap();
        let plain_project = builder
            .try_project(
                plain_scan,
                vec![ProjectionItemV1 {
                    value: plain_key,
                    alias: None,
                }],
                SpanV1::new(36, 46),
            )
            .unwrap();
        let plain_query = builder.query(plain_project, None, SpanV1::new(20, 46));
        // Correlated relational EXISTS query.
        let outer_scan = builder.relation_source(
            "records",
            SpanV1::new(56, 63),
            Some(AliasV1 {
                name: "o".into(),
                span: SpanV1::new(64, 65),
            }),
            SpanV1::new(56, 65),
        );
        let inner_scan = builder.relation_source(
            "records",
            SpanV1::new(76, 83),
            Some(AliasV1 {
                name: "f".into(),
                span: SpanV1::new(84, 85),
            }),
            SpanV1::new(76, 85),
        );
        let inner_path = builder
            .try_path(
                builder.symbol("f", SpanV1::new(90, 91)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(92, 94),
                }],
                SpanV1::new(90, 94),
            )
            .unwrap();
        let outer_path = builder
            .try_path(
                builder.symbol("o", SpanV1::new(97, 98)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(99, 101),
                }],
                SpanV1::new(97, 101),
            )
            .unwrap();
        let predicate = builder.compare("=", inner_path, outer_path, SpanV1::new(90, 101));
        let inner_filter = builder.relation_filter(inner_scan, predicate, SpanV1::new(76, 104));
        let exists = builder.relation_exists(
            RelationExistsPolarityV1::Exists,
            inner_filter,
            SpanV1::new(68, 105),
        );
        let outer_filter = builder.relation_filter(outer_scan, exists, SpanV1::new(56, 106));
        let exists_query = builder.query(outer_filter, None, SpanV1::new(50, 106));
        // Inline VIEW (a candidate for a second kernel pass).
        let inline = builder.query(
            builder.relation_source(
                "records",
                SpanV1::new(124, 131),
                None,
                SpanV1::new(124, 131),
            ),
            None,
            SpanV1::new(117, 131),
        );
        let view = CatalogDeclHirV1::new(
            HirNodeIdV1::from_raw(200),
            SpanV1::new(110, 131),
            "v",
            SpanV1::new(115, 116),
            CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(inline))),
        );
        // Composition with one Query LET and a Query body.
        let let_scan = builder.relation_source(
            "records",
            SpanV1::new(150, 157),
            None,
            SpanV1::new(150, 157),
        );
        let let_query = builder.query(let_scan, None, SpanV1::new(144, 157));
        let body_scan =
            builder.relation_source("q", SpanV1::new(170, 171), None, SpanV1::new(170, 171));
        let body_query = builder.query(body_scan, None, SpanV1::new(164, 171));
        let composition = CompositionHirV1::new(
            HirNodeIdV1::from_raw(300),
            SpanV1::new(136, 171),
            vec![CompositionBindingV1::new(
                "q",
                SpanV1::new(140, 141),
                CompositionProducerV1::Query(Box::new(let_query)),
            )],
            "q",
            SpanV1::new(166, 167),
            CompositionBodyV1::Query(Box::new(body_query)),
        );
        let source_id = SourceIdV2::from_raw(9);
        let expected_order = [
            SemanticItemIdV2::new(source_id, decl.id),
            SemanticItemIdV2::new(source_id, plain_query.id),
            SemanticItemIdV2::new(source_id, exists_query.id),
            SemanticItemIdV2::new(source_id, view.id),
            SemanticItemIdV2::new(source_id, composition.id),
        ];
        let document = resolve_checked_document_v2(
            CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
                source_id,
                source_len: 512,
                items: vec![
                    ProgramItemHirV1::Decl(Box::new(decl)),
                    ProgramItemHirV1::Query(Box::new(plain_query)),
                    ProgramItemHirV1::Query(Box::new(exists_query)),
                    ProgramItemHirV1::CatalogDecl(Box::new(view)),
                    ProgramItemHirV1::Composition(Box::new(composition)),
                ],
            })
            .map_err(|error| format!("{error:?}"))?,
            &authority,
        )
        .map_err(|error| format!("{error:?}"))?;
        if document.order() != expected_order {
            return Err(format!(
                "mixed document item order differs: expected {expected_order:?}, got {:?}",
                document.order()
            ));
        }
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "forged witnesses must fail the cross-owner equality validator"
    )]
    fn capture_vector_forgery_matrix_rejects_every_mutated_witness() -> Result<(), String> {
        let builder = HirBuilderV1::new();
        let outer_scan = builder.relation_source(
            "records",
            SpanV1::new(5, 12),
            Some(AliasV1 {
                name: "r".into(),
                span: SpanV1::new(13, 14),
            }),
            SpanV1::new(5, 14),
        );
        let inner_scan = builder.relation_source(
            "records",
            SpanV1::new(25, 32),
            Some(AliasV1 {
                name: "f".into(),
                span: SpanV1::new(33, 34),
            }),
            SpanV1::new(25, 34),
        );
        let inner_path = builder
            .try_path(
                builder.symbol("f", SpanV1::new(38, 39)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(40, 42),
                }],
                SpanV1::new(38, 42),
            )
            .unwrap();
        let captured = builder
            .try_path(
                builder.symbol("r", SpanV1::new(45, 46)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(47, 49),
                }],
                SpanV1::new(45, 49),
            )
            .unwrap();
        let predicate = builder.compare("=", inner_path, captured, SpanV1::new(38, 49));
        let inner_filter = builder.relation_filter(inner_scan, predicate, SpanV1::new(25, 52));
        let inner_filter_id = inner_filter.id;
        let exists = builder.relation_exists(
            RelationExistsPolarityV1::Exists,
            inner_filter,
            SpanV1::new(20, 53),
        );
        let exists_id = exists.id;
        let outer_filter = builder.relation_filter(outer_scan.clone(), exists, SpanV1::new(5, 54));
        let query = builder.query(outer_filter, None, SpanV1::new(0, 54));
        let mut binder = Binder::new(&EmptyModuleGraphV1);
        binder.declare_program(&[], &[]);
        binder.resolve_query(&query);
        let bound = binder.finish();
        assert!(bound.is_ok(), "{:?}", bound.diagnostics());
        let witness = capture_vector_witness(&bound, exists_id, inner_filter_id);
        assert_eq!(witness.edges.len(), 1, "one exact capture edge expected");
        // Positive: the untampered Binder witness validates.
        assert_eq!(
            validate_capture_vector(exists_id, inner_filter_id, &witness, &bound)
                .map_err(str::to_owned)?,
            witness.edges
        );
        // Dropped edge.
        let mut dropped = witness.clone();
        dropped.edges.pop();
        assert_eq!(
            validate_capture_vector(exists_id, inner_filter_id, &dropped, &bound).err(),
            Some("capture vector dropped or added an edge")
        );
        // Duplicate reference.
        let mut duplicate = witness.clone();
        duplicate.edges.push(duplicate.edges[0]);
        assert_eq!(
            validate_capture_vector(exists_id, inner_filter_id, &duplicate, &bound).err(),
            Some("duplicate capture reference in witness")
        );
        // Reversed two-capture order: a second edge on the same binding.
        let second = builder
            .try_path(
                builder.symbol("r", SpanV1::new(57, 58)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(59, 61),
                }],
                SpanV1::new(57, 61),
            )
            .unwrap();
        let lhs = builder
            .try_path(
                builder.symbol("g", SpanV1::new(64, 65)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(66, 68),
                }],
                SpanV1::new(64, 68),
            )
            .unwrap();
        // A second, distinct reference to the same outer alias: the capture
        // vector must retain two ordered edges, one per authored reference.
        let second_again = builder
            .try_path(
                builder.symbol("r", SpanV1::new(91, 92)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(93, 95),
                }],
                SpanV1::new(91, 95),
            )
            .unwrap();
        let lhs_again = builder
            .try_path(
                builder.symbol("g", SpanV1::new(96, 97)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(98, 100),
                }],
                SpanV1::new(96, 100),
            )
            .unwrap();
        let second_predicate = builder.and(
            builder.compare("=", lhs, second, SpanV1::new(64, 68)),
            builder.compare("=", lhs_again, second_again, SpanV1::new(91, 100)),
            SpanV1::new(64, 100),
        );
        let second_inner = builder.relation_filter(
            builder.relation_source(
                "records",
                SpanV1::new(70, 77),
                Some(AliasV1 {
                    name: "g".into(),
                    span: SpanV1::new(78, 79),
                }),
                SpanV1::new(70, 79),
            ),
            second_predicate,
            SpanV1::new(70, 82),
        );
        let second_inner_id = second_inner.id;
        let second_exists = builder.relation_exists(
            RelationExistsPolarityV1::Exists,
            second_inner,
            SpanV1::new(84, 85),
        );
        let second_exists_id = second_exists.id;
        let second_outer =
            builder.relation_filter(outer_scan.clone(), second_exists, SpanV1::new(5, 90));
        let second_query = builder.query(second_outer, None, SpanV1::new(0, 90));
        let mut second_binder = Binder::new(&EmptyModuleGraphV1);
        second_binder.declare_program(&[], &[]);
        second_binder.resolve_query(&second_query);
        let second_bound = second_binder.finish();
        assert!(second_bound.is_ok(), "{:?}", second_bound.diagnostics());
        let two = capture_vector_witness(&second_bound, second_exists_id, second_inner_id);
        assert_eq!(two.edges.len(), 2, "two exact capture edges expected");
        let mut reversed = two.clone();
        reversed.edges.reverse();
        assert_eq!(
            validate_capture_vector(second_exists_id, second_inner_id, &reversed, &second_bound)
                .err(),
            Some("capture edge reference differs from Binder proof")
        );
        assert_eq!(
            validate_capture_vector(second_exists_id, second_inner_id, &two, &second_bound)
                .map_err(str::to_owned)?,
            two.edges
        );
        // Wrong owner.
        let mut wrong_owner = witness.clone();
        wrong_owner.edges[0].owner = HirNodeIdV1::from_raw(999);
        assert_eq!(
            validate_capture_vector(exists_id, inner_filter_id, &wrong_owner, &bound).err(),
            Some("capture edge owner differs from Binder proof")
        );
        // Wrong ordinal.
        let mut wrong_ordinal = witness.clone();
        wrong_ordinal.edges[0].ordinal += 1;
        assert_eq!(
            validate_capture_vector(exists_id, inner_filter_id, &wrong_ordinal, &bound).err(),
            Some("capture edge ordinal differs from Binder proof")
        );
        // Foreign reference node.
        let mut foreign = witness.clone();
        foreign.edges[0].reference = HirNodeIdV1::from_raw(4242);
        assert_eq!(
            validate_capture_vector(exists_id, inner_filter_id, &foreign, &bound).err(),
            Some("capture edge reference differs from Binder proof")
        );
        // Wrong nested relation root.
        let mut wrong_root = witness.clone();
        wrong_root.relation_root = HirNodeIdV1::from_raw(777);
        assert_eq!(
            validate_capture_vector(exists_id, inner_filter_id, &wrong_root, &bound).err(),
            Some("capture witness nested relation root differs from checked HIR")
        );
        // Empty-set omission: an EXISTS node Binder never visited.
        assert_eq!(
            validate_capture_vector(
                HirNodeIdV1::from_raw(888),
                inner_filter_id,
                &witness,
                &bound
            )
            .err(),
            Some("Binder omitted the explicit relational EXISTS capture set")
        );
        Ok(())
    }

    #[test]
    #[expect(
        clippy::panic_in_result_fn,
        reason = "finalizer mutations must fail before document assembly"
    )]
    fn nested_relation_finalizer_rejects_dropped_extra_and_forged_correlations(
    ) -> Result<(), String> {
        // Real transaction output: the finalizer consumes the sealed maps.
        let budget = FrontendBudgetV2::standard_v2().map_err(|error| format!("{error:?}"))?;
        let authority = owner_authority(&budget)?;
        let builder = HirBuilderV1::new();
        let outer_scan = builder.relation_source(
            "records",
            SpanV1::new(5, 12),
            Some(AliasV1 {
                name: "r".into(),
                span: SpanV1::new(13, 14),
            }),
            SpanV1::new(5, 14),
        );
        let inner_scan = builder.relation_source(
            "records",
            SpanV1::new(25, 32),
            Some(AliasV1 {
                name: "f".into(),
                span: SpanV1::new(33, 34),
            }),
            SpanV1::new(25, 34),
        );
        let inner_path = builder
            .try_path(
                builder.symbol("f", SpanV1::new(38, 39)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(40, 42),
                }],
                SpanV1::new(38, 42),
            )
            .unwrap();
        let captured = builder
            .try_path(
                builder.symbol("r", SpanV1::new(45, 46)),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(47, 49),
                }],
                SpanV1::new(45, 49),
            )
            .unwrap();
        let predicate = builder.compare("=", inner_path, captured, SpanV1::new(38, 49));
        let inner_filter = builder.relation_filter(inner_scan, predicate, SpanV1::new(25, 52));
        let exists = builder.relation_exists(
            RelationExistsPolarityV1::Exists,
            inner_filter,
            SpanV1::new(20, 53),
        );
        let outer_filter = builder.relation_filter(outer_scan, exists, SpanV1::new(5, 54));
        let query = builder.query(outer_filter, None, SpanV1::new(0, 54));
        let document = resolve_checked_document_v2(
            CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
                source_id: SourceIdV2::from_raw(9),
                source_len: 96,
                items: vec![ProgramItemHirV1::Query(Box::new(query))],
            })
            .map_err(|error| format!("{error:?}"))?,
            &authority,
        )
        .map_err(|error| format!("{error:?}"))?;
        let source = document.source_id();
        let mut expressions = BTreeMap::new();
        for (key, value) in document.expressions() {
            expressions.insert(*key, value.clone());
        }
        let nested = document
            .nested_relations()
            .map(|(key, value)| (*key, value.clone()))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(nested.len(), 1, "one nested relation expected");
        let identity = document
            .nested_relations()
            .next()
            .unwrap()
            .1
            .location()
            .registry_identity()
            .clone();
        // Positive: the sealed transaction finalizes.
        check_nested_relation_finalizer(source, &identity, &expressions, &nested)
            .map_err(str::to_owned)?;
        // Dropped nested root.
        let root = *nested.keys().next().unwrap();
        let mut dropped = nested.clone();
        dropped.remove(&root);
        assert_eq!(
            check_nested_relation_finalizer(source, &identity, &expressions, &dropped).err(),
            Some("nested relation registry differs from relational EXISTS expression facts")
        );
        // Extra nested root: a registry-consistent scan under an absent root.
        let extra_binding = ResolvedValueBindingV2::try_new(
            source,
            HirNodeIdV1::from_raw(700),
            0,
            ResolvedBindingKindV2::Composition,
            "q".to_owned(),
        )
        .map_err(|error| error.to_string())?;
        let extra_source = ResolvedScanSourceV2::Composition(
            ResolvedCompositionSourceIdV2::try_from_binding(extra_binding)
                .map_err(|error| error.to_string())?,
        );
        let extra_output = RelationOutputContractV2::try_new(
            RecordTypeV1::try_new(Vec::new()).map_err(|error| format!("{error:?}"))?,
            OutputProvenanceV2::new([extra_source.clone()]),
            Vec::new(),
            CardinalityV1::Bag,
            Vec::new(),
            Vec::new(),
            ExpressionEffectsV2::pure_scalar(),
        )
        .map_err(str::to_owned)?;
        let extra_scan = TypedRelationExprV2::scan(
            quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
                source_id: source,
                node_id: HirNodeIdV1::from_raw(701),
            },
            document
                .nested_relations()
                .next()
                .unwrap()
                .1
                .location()
                .clone(),
            extra_source,
            extra_output,
        )
        .map_err(str::to_owned)?;
        let mut extra = nested.clone();
        extra.insert(
            quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
                source_id: source,
                node_id: HirNodeIdV1::from_raw(701),
            },
            extra_scan,
        );
        assert_eq!(
            check_nested_relation_finalizer(source, &identity, &expressions, &extra).err(),
            Some("nested relation registry differs from relational EXISTS expression facts")
        );
        // Dropped correlation-reference expression.
        let correlation_reference = expressions
            .values()
            .find_map(|expression| {
                let TypedExprShapeV2::RelationExists { correlations, .. } = expression.shape()
                else {
                    return None;
                };
                correlations.first().map(|edge| edge.reference())
            })
            .unwrap();
        let mut no_reference = expressions.clone();
        no_reference.remove(&correlation_reference);
        assert_eq!(
            check_nested_relation_finalizer(source, &identity, &no_reference, &nested).err(),
            Some("correlation reference is foreign to this document's expressions")
        );
        // Forged correlation witnesses against the real expression keys.
        let keys = expressions.keys().copied().collect::<BTreeSet<_>>();
        let real = CorrelationWitnessV2 {
            reference: correlation_reference,
            binding_source: source,
            registry: identity.clone(),
        };
        check_correlation_witnesses(source, &identity, &keys, std::slice::from_ref(&real))
            .map_err(str::to_owned)?;
        let foreign = CorrelationWitnessV2 {
            reference: quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(9),
                node_id: HirNodeIdV1::from_raw(4242),
            },
            binding_source: source,
            registry: identity.clone(),
        };
        assert_eq!(
            check_correlation_witnesses(source, &identity, &keys, &[foreign]).err(),
            Some("correlation reference is foreign to this document's expressions")
        );
        assert_eq!(
            check_correlation_witnesses(source, &identity, &keys, &[real.clone(), real.clone()])
                .err(),
            Some("duplicate correlation reference in one nested relation")
        );
        // Cross-registry checked location: a second host registry admitting
        // the same document source yields a different registry identity.
        let other_host = SourceRegistryViewV2::try_new(vec![
            SourceRegistrationV2 {
                source_id: SourceIdV2::from_raw(1),
                source_len: 512,
            },
            SourceRegistrationV2 {
                source_id: SourceIdV2::from_raw(2),
                source_len: 8,
            },
        ])
        .map_err(|error| format!("{error:?}"))?;
        let cross = CorrelationWitnessV2 {
            reference: correlation_reference,
            binding_source: source,
            registry: other_host.identity().clone(),
        };
        assert_eq!(
            check_correlation_witnesses(source, &identity, &keys, &[cross]).err(),
            Some("correlation reference location belongs to another source registry")
        );
        Ok(())
    }
}
