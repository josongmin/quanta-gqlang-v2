//! Atomic document semantic transaction and exact item-key assembly.

use crate::authority::{ClosedFrontendAuthorityV2, FrontendAuthorityErrorV2, FrontendBudgetV2, GraphReferenceCatalogV2};
use crate::diagnostic::{location, FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2};
use crate::{AggregateContextV2, AggregatePositionV2, GraphDirectionV2, GraphReferenceRoleV2, GraphWitnessBoundsV2, NullOrderV2, OrderKeyContractV2, OutputColumnContractV2, OutputColumnOriginV2, OutputProvenanceV2, RelationOutputContractV2, ResolvedBindingKindV2, ResolvedCatalogItemV2, ResolvedCatalogPayloadV2, ResolvedCatalogReferenceV2, ResolvedCheckV2, ResolvedDeclarationV2, ResolvedExplainTargetV2, ResolvedGraphQueryV2, ResolvedGraphReferenceV2, ResolvedGraphUseV2, ResolvedMetaKindV2, ResolvedMetaV2, ResolvedPresentationKindV2, ResolvedPresentationV2, ResolvedRuleCallV2, ResolvedRuleV2, ResolvedScanSourceV2, ResolvedValueBindingV2, RuleCallPolarityV2, SemanticItemIdV2, SemanticItemV2, TypedBoolExprV2, TypedCallArgumentV2, TypedExprShapeV2, TypedExprV2, TypedPathSegmentV2, TypedRecordFieldV2, TypedRelationExprV2, TypedRelationV2};
use quanta_gqlang_v2_binder::{BindResultV1, Binder, BinderKindV1, ModuleGraphViewV1, ParamV1, ResolvedTargetV1};
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CheckedHirDocumentOwnedV2, ExtendedSourceRegistryViewV2, HirDocumentV1, HirNodeIdV1,
    HirVisitorV1, ProgramItemHirV1, SourceIdV2, SourceRegistryIdentityV2, SpanV1,
    ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_modules::{CatalogAssetRefInputV2, DocumentCatalogAssetInputV2, DocumentCatalogAuthorityV2, DocumentCatalogFamilyReferenceInputV2, DocumentCatalogFinalInputV2, DocumentCatalogReferenceFamilyV2, DocumentCatalogReferenceInputV2, DocumentCatalogValidatorV2, NamespaceKindV2, PredeclaredDocumentAssetV2, ProfileAdmissionInputV2, ProfileAdmissionV2, RequireCertifiedInputV2, ResolvedGraphV2};
use quanta_gqlang_v2_types::{AggregationClassV1, BuiltinRegistryV2, ExpressionEffectsV2, ExpressionTypeKernelV2, ExpressionTypeStateV2, RecordFieldTypeV1, RecordTypeV1, ResolvedCallTargetV2, ResolvedCallableInputV2, ResolvedPredicateIdV2, ResolvedTypeInputEntryV2, ResolvedTypeInputValueV2, ResolvedTypeInputsV2, RuleBindingIdV2, RuleCallInputV2, RuleCallableTargetV2, RulePredicateInputV2, RuleSymbolInputV2, RuleTypeBudgetV2, RuleTypeSolverV2, SourceQualifiedNodeIdV2, TypeDiagnosticV2, TypedExpressionFactsV2, ValueTypeV1};
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
                quanta_gqlang_v2_hir::HirIntegrityErrorV1::DuplicateNodeId { first_span, second_span, .. } => (*second_span, vec![location(source_id, *first_span)]),
                quanta_gqlang_v2_hir::HirIntegrityErrorV1::InvertedSpan { span, .. }
                | quanta_gqlang_v2_hir::HirIntegrityErrorV1::SpanOutOfBounds { span, .. }
                | quanta_gqlang_v2_hir::HirIntegrityErrorV1::InvertedAuxiliarySpan { span }
                | quanta_gqlang_v2_hir::HirIntegrityErrorV1::AuxiliarySpanOutOfBounds { span, .. } => (*span, Vec::new()),
            };
            FrontendFailureV2::one(FrontendDiagnosticV2::new(
                FrontendStageV2::Integrity, error.code(), FrontendSeverityV2::Error,
                location(source_id, span), secondary, error.to_string(),
            ))
        })?;
        Ok(Self { checked })
    }

    /// Checked source identity.
    #[must_use] pub const fn source_id(&self) -> SourceIdV2 { self.checked.document().source_id }
    /// Sole source-length authority.
    #[must_use] pub const fn source_len(&self) -> usize { self.checked.document().source_len }
    /// Document-wide count of checked HIR nodes.
    #[must_use] pub const fn node_count(&self) -> usize { self.checked.node_count() }
    /// Borrow the integrity-proven document for owner kernels.
    #[must_use] pub const fn checked(&self) -> &CheckedHirDocumentOwnedV2 { &self.checked }
}

/// Checked declarations collected before any body resolution.
#[derive(Clone, Debug)]
pub struct DeclaredDocumentV2 {
    checked: CheckedSyntaxDocumentV2,
    expected: BTreeMap<SemanticItemIdV2, (SpanV1, SemanticItemKindV2)>,
    order: Vec<SemanticItemIdV2>,
    expected_expressions: BTreeSet<SourceQualifiedNodeIdV2>,
    registry_identity: SourceRegistryIdentityV2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SemanticItemKindV2 { Relation, Declaration, Rule, Graph, Check, Presentation, Catalog, Composition, Meta }

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
    pub fn try_new(checked: CheckedSyntaxDocumentV2, budget: &FrontendBudgetV2, extended: &ExtendedSourceRegistryViewV2) -> Result<Self, FrontendFailureV2> {
        let source_id = checked.source_id();
        if extended.source_len(source_id) != Some(checked.source_len()) {
            return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0), "checked syntax source differs from the extended authority registry"));
        }
        if checked.checked.document().items.len() > usize::try_from(budget.max_document_items()).unwrap_or(usize::MAX) {
            return Err(document_failure("frontend.resource_limit_exceeded", source_id, SpanV1::point(0), "document_items exceeds frontend limit"));
        }
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
            if expected.insert(id, (item.span(), SemanticItemKindV2::from_hir(item))).is_some() {
                return Err(document_failure("sema.document_validation_failed", source_id, item.span(), "duplicate semantic item identity"));
            }
            order.push(id);
            expression_nodes.visit_item(item);
        }
        let expected_expressions = expression_nodes.0.into_iter().map(|node_id| SourceQualifiedNodeIdV2 { source_id, node_id }).collect();
        Ok(Self { checked, expected, order, expected_expressions, registry_identity: extended.identity() })
    }

    /// Checked syntax document.
    #[must_use] pub const fn checked(&self) -> &CheckedSyntaxDocumentV2 { &self.checked }
    /// Authored item order.
    #[must_use] pub fn order(&self) -> &[SemanticItemIdV2] { &self.order }
    /// Expected item IDs and original spans.
    pub fn expected(&self) -> impl Iterator<Item = (SemanticItemIdV2, SpanV1)> + '_ {
        self.expected.iter().map(|(id, (span, _))| (*id, *span))
    }
}

/// Document whose names/references have been resolved, before typed body
/// assembly and global catalog validation.
#[derive(Clone, Debug)]
pub struct ResolvedDocumentV2 {
    declared: DeclaredDocumentV2,
}

impl ResolvedDocumentV2 {
    /// Advance the declared state only after the canonical owner resolution
    /// phase reports no diagnostics.
    #[must_use]
    pub fn from_declared(declared: DeclaredDocumentV2) -> Self { Self { declared } }
    /// Declared checked source.
    #[must_use] pub const fn declared(&self) -> &DeclaredDocumentV2 { &self.declared }
}

/// One unresolved editor-only fact. It cannot enter a closed document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnresolvedSemanticFactV2 {
    code: &'static str,
    location: quanta_gqlang_v2_hir::LocatedSpanV2,
}

impl UnresolvedSemanticFactV2 {
    /// Construct one explicit unresolved fact.
    #[must_use] pub const fn new(code: &'static str, location: quanta_gqlang_v2_hir::LocatedSpanV2) -> Self { Self { code, location } }
    /// Stable unresolved code.
    #[must_use] pub const fn code(&self) -> &'static str { self.code }
    /// Source location.
    #[must_use] pub const fn location(&self) -> quanta_gqlang_v2_hir::LocatedSpanV2 { self.location }
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
    pub fn try_new(source_id: SourceIdV2, unresolved: Vec<UnresolvedSemanticFactV2>, diagnostics: Vec<FrontendDiagnosticV2>) -> Result<Self, &'static str> {
        if unresolved.is_empty() { return Err("incomplete analysis requires an unresolved fact"); }
        if unresolved.iter().any(|fact| fact.location.source_id != source_id) { return Err("foreign unresolved analysis fact"); }
        Ok(Self { source_id, unresolved, diagnostics })
    }
    /// Source identity.
    #[must_use] pub const fn source_id(&self) -> SourceIdV2 { self.source_id }
    /// Exact unresolved facts.
    #[must_use] pub fn unresolved(&self) -> &[UnresolvedSemanticFactV2] { &self.unresolved }
    /// Ordered analysis diagnostics.
    #[must_use] pub fn diagnostics(&self) -> &[FrontendDiagnosticV2] { &self.diagnostics }
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
    fn from(failure: FrontendFailureV2) -> Self { Self::Document(failure) }
}

impl From<FrontendAuthorityErrorV2> for FrontendTransactionErrorV2 {
    fn from(error: FrontendAuthorityErrorV2) -> Self { Self::Authority(error) }
}

/// Atomic semantic document. Construction requires an exact item-ID set, the
/// original source order, and a once-validated document catalog proof.
#[derive(Clone, Debug)]
pub struct SemanticDocumentV2 {
    source_id: SourceIdV2,
    order: Vec<SemanticItemIdV2>,
    items: BTreeMap<SemanticItemIdV2, SemanticItemV2>,
    expressions: BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    catalog: DocumentCatalogAuthorityV2,
}

impl SemanticDocumentV2 {
    /// Assemble exact-key semantic outputs; no positional zip or partial
    /// success path is available.
    pub(crate) fn try_assemble(
        declared: &DeclaredDocumentV2,
        produced: Vec<SemanticItemV2>,
        expressions: BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
        type_fact_count: usize,
        catalog: DocumentCatalogAuthorityV2,
        graph_authority: &GraphReferenceCatalogV2,
    ) -> Result<Self, FrontendFailureV2> {
        let source_id = declared.checked.source_id();
        if catalog.source_registry_identity() != &declared.registry_identity {
            return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0), "catalog proof belongs to another document authority registry"));
        }
        let expected: BTreeSet<_> = declared.expected.keys().copied().collect();
        let mut items = BTreeMap::new();
        for item in produced {
            let id = item.item_id();
            if id.source_id() != source_id || !expected.contains(&id) {
                return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0), "foreign or extra semantic item identity"));
            }
            if declared.expected.get(&id).is_some_and(|(_, kind)| *kind != SemanticItemKindV2::from_semantic(&item)) {
                let span = declared.expected.get(&id).map_or(SpanV1::point(0), |(span, _)| *span);
                return Err(document_failure("sema.document_validation_failed", source_id, span, "semantic item kind differs from checked syntax"));
            }
            if items.insert(id, item).is_some() {
                return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0), "duplicate semantic item identity"));
            }
        }
        if items.keys().copied().collect::<BTreeSet<_>>() != expected {
            let missing = declared.expected.iter().find(|(id, _)| !items.contains_key(id)).map_or(SpanV1::point(0), |(_, (span, _))| *span);
            return Err(document_failure("sema.document_validation_failed", source_id, missing, "missing semantic item identity"));
        }
        let semantic_assets = items.values().filter_map(|item| match item {
            SemanticItemV2::Catalog(catalog) => Some((catalog.asset_id().clone(), catalog.asset_id().namespace_kind())),
            _ => None,
        }).collect::<BTreeSet<_>>();
        let validated_assets = catalog.assets().map(|(id, kind)| (id.clone(), kind)).collect::<BTreeSet<_>>();
        if semantic_assets != validated_assets {
            return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0), "semantic catalog assets differ from modules-validated document assets"));
        }
        let semantic_profiles = items.values().filter_map(|item| {
            let SemanticItemV2::Catalog(catalog) = item else { return None };
            let ResolvedCatalogPayloadV2::Profile { admission, .. } = catalog.payload() else { return None };
            let admission = match admission {
                quanta_gqlang_v2_hir::AdmissionV1::Allowlist => ProfileAdmissionV2::Allowlist,
                quanta_gqlang_v2_hir::AdmissionV1::ReviewRequired => ProfileAdmissionV2::ReviewRequired,
                quanta_gqlang_v2_hir::AdmissionV1::CertifiedOnly => ProfileAdmissionV2::CertifiedOnly,
            };
            Some((catalog.asset_id().clone(), admission))
        }).collect::<BTreeSet<_>>();
        let validated_profiles = catalog.profiles().map(|(id, admission)| (id.clone(), admission)).collect::<BTreeSet<_>>();
        if semantic_profiles != validated_profiles {
            return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0),
                "semantic profile admissions differ from modules-validated policy"));
        }
        let certified = items.values().filter_map(|item| match item {
            SemanticItemV2::Meta(meta) if matches!(meta.kind(), ResolvedMetaKindV2::RequireCertified) =>
                Some(meta.location().location()),
            _ => None,
        }).collect::<Vec<_>>();
        if certified.len() > 1 || certified.first().copied() != catalog.require_certified_location() {
            return Err(document_failure("sema.document_validation_failed", source_id,
                certified.first().map_or(SpanV1::point(0), |location| location.span),
                "semantic certification directive differs from modules-validated exact location/count"));
        }
        let mut semantic_edges = Vec::new();
        for item in items.values() {
            let SemanticItemV2::Catalog(catalog) = item else { continue };
            let mut add = |reference: &ResolvedCatalogReferenceV2, family| semantic_edges.push((
                catalog.asset_id().clone(), reference.id().clone(), reference.kind(), Some(family), reference.location(),
            ));
            match catalog.payload() {
                ResolvedCatalogPayloadV2::View { query_ref } => {
                    if let Some(reference) = query_ref { add(reference, DocumentCatalogReferenceFamilyV2::ViewQuery); }
                }
                ResolvedCatalogPayloadV2::Model { source } => add(source, DocumentCatalogReferenceFamilyV2::ModelSource),
                ResolvedCatalogPayloadV2::Pack { source, .. } => add(source, DocumentCatalogReferenceFamilyV2::PackSource),
                ResolvedCatalogPayloadV2::Profile { capabilities, .. } => {
                    for reference in capabilities { add(reference, DocumentCatalogReferenceFamilyV2::ProfileCapability); }
                }
                ResolvedCatalogPayloadV2::Gate { checks, .. } => {
                    for reference in checks { add(reference, DocumentCatalogReferenceFamilyV2::GateCheck); }
                }
                ResolvedCatalogPayloadV2::RuleBinding { check_ref } => add(check_ref, DocumentCatalogReferenceFamilyV2::RuleBindingCheck),
            }
        }
        let mut validated_edges = catalog.dependencies().iter().map(|edge| (
            edge.source_asset_id().clone(), edge.target_asset_id().clone(),
            edge.expected_target_kind(), edge.family(), edge.location().location(),
        )).collect::<Vec<_>>();
        semantic_edges.sort_by(|left, right| (&left.0, &left.1).cmp(&(&right.0, &right.1)));
        validated_edges.sort_by(|left, right| (&left.0, &left.1).cmp(&(&right.0, &right.1)));
        if semantic_edges != validated_edges {
            return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0), "semantic catalog references differ from modules-validated dependency graph"));
        }
        if expressions.iter().any(|(key, fact)| key.source_id != source_id || *key != fact.key()) {
            return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0), "foreign or mismatched typed expression identity"));
        }
        if type_fact_count != declared.expected_expressions.len()
            || expressions.keys().copied().collect::<BTreeSet<_>>() != declared.expected_expressions {
            return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0), "typed expression set differs from checked HIR and types owner facts"));
        }
        for item in items.values() {
            if let SemanticItemV2::Declaration(decl) = item {
                if expressions.get(&decl.body().key()) != Some(decl.body()) {
                    return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0), "declaration body does not match document expression proof"));
                }
            }
            if let SemanticItemV2::Rule(rule) = item {
                if expressions.get(&rule.head_term().key()) != Some(rule.head_term())
                    || expressions.get(&rule.body().expression().key()) != Some(rule.body().expression()) {
                    return Err(document_failure("sema.document_validation_failed", source_id, SpanV1::point(0),
                        "rule head/body disagree with the document expression proof"));
                }
            }
            if let SemanticItemV2::Graph(graph) = item {
                for reference in graph.references() {
                    if !graph_authority.admits(reference.target())
                        || reference.use_location().registry_identity() != &declared.registry_identity
                        || reference.use_location().location().source_id != source_id {
                        return Err(document_failure("sema.document_validation_failed", source_id, reference.use_location().location().span, "graph reference does not belong to this document and sealed graph authority"));
                    }
                }
            }
            if let SemanticItemV2::Presentation(presentation) = item {
                let mut uses = Vec::new();
                match presentation.kind() {
                    ResolvedPresentationKindV2::Explain { target: ResolvedExplainTargetV2::Entity(graph), .. }
                    | ResolvedPresentationKindV2::Describe { target: graph } => uses.push(graph),
                    ResolvedPresentationKindV2::Sample { scope, .. } => uses.extend(scope.as_ref()),
                    ResolvedPresentationKindV2::Compare { baseline, candidate, .. } => {
                        uses.push(baseline); uses.push(candidate);
                    }
                    ResolvedPresentationKindV2::Export { source, .. } => uses.push(source),
                    ResolvedPresentationKindV2::Explain { target: ResolvedExplainTargetV2::Rule(_), .. }
                    | ResolvedPresentationKindV2::Show { .. } => {}
                }
                for reference in uses {
                    if !graph_authority.admits(reference.target())
                        || reference.location().registry_identity() != &declared.registry_identity
                        || reference.location().location().source_id != source_id {
                        return Err(document_failure("sema.document_validation_failed", source_id, reference.location().location().span, "presentation selector differs from the sealed graph/document authority"));
                    }
                }
            }
            if let SemanticItemV2::Check(check) = item {
                let uses = check.scope().into_iter().chain(check.pattern());
                for reference in uses {
                    if !graph_authority.admits(reference.target())
                        || reference.location().registry_identity() != &declared.registry_identity
                        || reference.location().location().source_id != source_id {
                        return Err(document_failure("sema.document_validation_failed", source_id, reference.location().location().span, "CHECK selector differs from sealed graph/document authority"));
                    }
                }
                if let Some(nested) = check.nested_graph() {
                    for reference in nested.references() {
                        if !graph_authority.admits(reference.target())
                            || reference.use_location().registry_identity() != &declared.registry_identity
                            || reference.use_location().location().source_id != source_id {
                            return Err(document_failure("sema.document_validation_failed", source_id, reference.use_location().location().span, "nested CHECK/FIND selector differs from sealed graph/document authority"));
                        }
                    }
                }
            }
        }
        Ok(Self { source_id, order: declared.order.clone(), items, expressions, catalog })
    }

    /// Source identity.
    #[must_use] pub const fn source_id(&self) -> SourceIdV2 { self.source_id }
    /// Original item order.
    #[must_use] pub fn order(&self) -> &[SemanticItemIdV2] { &self.order }
    /// Exact semantic item by ID.
    #[must_use] pub fn item(&self, id: SemanticItemIdV2) -> Option<&SemanticItemV2> { self.items.get(&id) }
    /// All semantic items by canonical identity.
    pub fn items(&self) -> impl Iterator<Item = (&SemanticItemIdV2, &SemanticItemV2)> { self.items.iter() }
    /// Complete source-rule set from the same sealed document transaction.
    pub fn rules(&self) -> impl Iterator<Item = &ResolvedRuleV2> {
        self.items.values().filter_map(|item| match item {
            SemanticItemV2::Rule(rule) => Some(rule),
            _ => None,
        })
    }
    /// Closed typed expression by exact source-qualified node ID.
    #[must_use] pub fn expression(&self, key: SourceQualifiedNodeIdV2) -> Option<&TypedExprV2> { self.expressions.get(&key) }
    /// Every typed expression fact, including child nodes, in stable key order.
    pub fn expressions(&self) -> impl Iterator<Item = (&SourceQualifiedNodeIdV2, &TypedExprV2)> { self.expressions.iter() }
    /// Document catalog proof.
    #[must_use] pub const fn catalog(&self) -> &DocumentCatalogAuthorityV2 { &self.catalog }
}

/// Resolve one checked document against complete immutable authority. Every
/// body and metadata directive is handled inside one transaction; any failure
/// returns diagnostics and no semantic document.
pub fn resolve_checked_document_v2(
    checked: CheckedSyntaxDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
) -> Result<SemanticDocumentV2, FrontendTransactionErrorV2> {
    let extended = authority.try_extend_document(checked.source_id(), checked.source_len())
        .map_err(|error| FrontendTransactionErrorV2::Document(document_failure(error.code(), checked.source_id(), SpanV1::point(0), error.to_string())))?;
    let declared = DeclaredDocumentV2::try_new(checked, authority.budget(), &extended)?;
    let resolved = ResolvedDocumentV2::from_declared(declared);
    let mut steps = SemanticStepCounterV2::new(authority.budget());
    let mut validator = DocumentCatalogValidatorV2::try_new(extended.clone(), authority.modules())
        .map_err(module_failure)?;
    // Mint local IDs through the modules owner before any body/reference
    // resolution. No semantic path derives CatalogAssetId from raw spelling.
    let mut authored_assets = Vec::new();
    for item in &resolved.declared.checked().checked().document().items {
        if let ProgramItemHirV1::CatalogDecl(decl) = item {
            let kind = match &decl.payload {
                quanta_gqlang_v2_hir::CatalogPayloadV1::View(_) => NamespaceKindV2::View,
                quanta_gqlang_v2_hir::CatalogPayloadV1::Model { .. } => NamespaceKindV2::Model,
                quanta_gqlang_v2_hir::CatalogPayloadV1::Pack { .. } => NamespaceKindV2::Pack,
                quanta_gqlang_v2_hir::CatalogPayloadV1::Profile { .. } => NamespaceKindV2::Profile,
                quanta_gqlang_v2_hir::CatalogPayloadV1::Gate { .. } => NamespaceKindV2::Gate,
                quanta_gqlang_v2_hir::CatalogPayloadV1::RuleBinding { .. } => NamespaceKindV2::Rule,
            };
            let location = quanta_gqlang_v2_hir::LocatedSpanV2 { source_id: resolved.declared.checked().source_id(), span: decl.name_span };
            let checked_location = extended.combined().try_check_location(location)
                .map_err(|error| document_failure("sema.document_validation_failed", location.source_id, location.span, error.to_string()))?;
            authored_assets.push(DocumentCatalogAssetInputV2 {
                owner_module_id: authority.modules().resolved_graph().current_module().clone(),
                local_name: decl.name.clone(), namespace_kind: kind, location: checked_location,
            });
        }
    }
    let predeclared = validator.predeclare_assets(authored_assets).map_err(module_failure)?;
    let mut catalog_items = BTreeMap::new();
    for item in &resolved.declared.checked().checked().document().items {
        if let ProgramItemHirV1::CatalogDecl(decl) = item {
            let matches = predeclared.iter().filter(|asset| asset.id().local_name() == decl.name
                && asset.location().location().span == decl.name_span).collect::<Vec<_>>();
            let [asset] = matches.as_slice() else {
                return Err(document_failure("sema.document_validation_failed", resolved.declared.checked().source_id(), decl.name_span,
                    "modules predeclaration omitted or ambiguously identified a catalog item").into());
            };
            catalog_items.insert(decl.id, (*asset).clone());
        }
    }
    let result = resolve_document_bodies(&resolved, authority, &extended, &catalog_items, &mut validator, &mut steps)?;
    let catalog = validator.validate_predeclared(result.catalog_input)
        .map_err(module_failure)?;
    Ok(SemanticDocumentV2::try_assemble(&resolved.declared, result.items, result.expressions, result.type_fact_count, catalog, authority.graph_references())?)
}

pub(crate) struct SemanticStepCounterV2 {
    limit: u32,
    used: u32,
}

impl SemanticStepCounterV2 {
    pub(crate) fn new(budget: &FrontendBudgetV2) -> Self { Self { limit: budget.max_semantic_steps(), used: 0 } }
    pub(crate) fn charge(&mut self, source_id: SourceIdV2, span: SpanV1) -> Result<(), FrontendFailureV2> {
        self.used = self.used.checked_add(1).unwrap_or(u32::MAX);
        if self.used > self.limit {
            return Err(document_failure("frontend.resource_limit_exceeded", source_id, span, "semantic_steps exceeds frontend limit"));
        }
        Ok(())
    }
}

pub(crate) fn document_failure(code: &'static str, source_id: SourceIdV2, span: SpanV1, message: impl AsRef<str>) -> FrontendFailureV2 {
    FrontendFailureV2::one(FrontendDiagnosticV2::new(
        FrontendStageV2::Validation, code, FrontendSeverityV2::Error,
        location(source_id, span), Vec::new(), message,
    ))
}

pub(crate) fn module_failure(error: quanta_gqlang_v2_modules::ModuleDiagnosticV2) -> FrontendTransactionErrorV2 {
    let Some(primary) = error.primary() else {
        return FrontendTransactionErrorV2::Authority(FrontendAuthorityErrorV2::from_external(error.code(), error.to_string()));
    };
    let mut secondary = error.secondary().into_iter().collect::<Vec<_>>();
    secondary.extend_from_slice(error.edge_witness());
    FrontendTransactionErrorV2::Document(FrontendFailureV2::one(FrontendDiagnosticV2::new(
        FrontendStageV2::Validation, error.code(), FrontendSeverityV2::Error,
        primary, secondary, error.to_string(),
    )))
}

fn type_failure(error: TypeDiagnosticV2) -> FrontendTransactionErrorV2 {
    let Some(primary) = error.primary() else {
        return FrontendTransactionErrorV2::Authority(FrontendAuthorityErrorV2::from_external(error.code(), error.to_string()));
    };
    let secondary = error.secondary().into_iter().collect();
    FrontendTransactionErrorV2::Document(FrontendFailureV2::one(FrontendDiagnosticV2::new(
        FrontendStageV2::Typing, error.code(), FrontendSeverityV2::Error, primary, secondary, error.to_string(),
    )))
}

struct BodyResolutionV2 {
    items: Vec<SemanticItemV2>,
    expressions: BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
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
                        reference: SourceQualifiedNodeIdV2 { source_id: self.source_id, node_id: node.id },
                        binding: RuleBindingIdV2::new(self.source_id, owner, ordinal),
                    });
                }
                _ if self.error.is_none() => self.error = Some(document_failure("sema.unresolved_binding_type",
                    self.source_id, node.span, "rule symbol is not an exact local Binder binding")),
                _ => {}
            },
            ValueExprKindV1::Call(call) => self.calls.push((SourceQualifiedNodeIdV2 {
                source_id: self.source_id, node_id: node.id,
            }, call.callee.name.clone(), node.span)),
            _ => {}
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
}

struct RuleDependencyCollectorV2<'a> {
    source_id: SourceIdV2,
    targets: &'a BTreeMap<HirNodeIdV1, RuleCallableTargetV2>,
    facts: &'a TypedExpressionFactsV2,
    extended: &'a ExtendedSourceRegistryViewV2,
    negated: bool,
    aggregated: bool,
    edges: Vec<(SourceQualifiedNodeIdV2, ResolvedCallTargetV2, RuleCallPolarityV2, quanta_gqlang_v2_hir::CheckedLocatedSpanV2)>,
    error: Option<FrontendFailureV2>,
}

struct QueryTypeInputsV2<'a> {
    source_id: SourceIdV2,
    authority: &'a ClosedFrontendAuthorityV2,
    bound: &'a BindResultV1,
    builtins: &'a BuiltinRegistryV2,
    owners: BTreeMap<(HirNodeIdV1, usize), ResolvedTypeInputValueV2>,
    entries: Vec<ResolvedTypeInputEntryV2>,
    error: Option<FrontendFailureV2>,
}

impl HirVisitorV1 for QueryTypeInputsV2<'_> {
    fn visit_relation(&mut self, relation: &quanta_gqlang_v2_hir::RelationExprHirV1) {
        match &relation.kind {
            quanta_gqlang_v2_hir::RelationExprKindV1::Source(source) if source.alias.is_some() => {
                let input = self.authority.relations().get(&source.name).map(|host|
                    ValueTypeV1::try_record(host.relation_type().row().clone())
                        .map(ExpressionTypeStateV2::Known)
                        .map(ResolvedTypeInputValueV2::Binding)
                );
                match input {
                    Some(Ok(value)) => { self.owners.insert((relation.id, 0), value); }
                    Some(Err(error)) if self.error.is_none() => self.error = Some(document_failure(
                        error.code(), self.source_id, relation.span, error.to_string())),
                    None if self.error.is_none() => self.error = Some(document_failure(
                        "sema.unresolved_reference", self.source_id, relation.span, "host relation source is absent")),
                    _ => {}
                }
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) => {
                for (ordinal, item) in project.items().iter().enumerate() {
                    self.owners.insert((relation.id, ordinal), ResolvedTypeInputValueV2::BindingFromExpression(
                        SourceQualifiedNodeIdV2 { source_id: self.source_id, node_id: item.value.id },
                    ));
                }
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(group) => {
                for (ordinal, key) in group.keys().iter().enumerate() {
                    self.owners.insert((relation.id, ordinal), ResolvedTypeInputValueV2::BindingFromExpression(
                        SourceQualifiedNodeIdV2 { source_id: self.source_id, node_id: key.id },
                    ));
                }
            }
            _ => {}
        }
        quanta_gqlang_v2_hir::walk_relation(self, relation);
    }

    fn visit_value(&mut self, node: &ValueExprHirV1) {
        let entry = match &node.kind {
            ValueExprKindV1::Symbol(_) => match self.bound.target_of(node.id) {
                Some(ResolvedTargetV1::Local(binding)) => self.owners.get(&binding.identity()).cloned(),
                Some(ResolvedTargetV1::Declaration { .. }) => None,
                _ => None,
            },
            ValueExprKindV1::Call(call) => self.builtins.family_id(&call.callee.name)
                .map(|id| ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(id))),
            _ => { quanta_gqlang_v2_hir::walk_value(self, node); return; }
        };
        if let Some(value) = entry {
            self.entries.push(ResolvedTypeInputEntryV2 {
                key: SourceQualifiedNodeIdV2 { source_id: self.source_id, node_id: node.id }, value,
            });
        } else if self.error.is_none() {
            self.error = Some(document_failure("sema.unresolved_binding_type", self.source_id, node.span,
                "query reference has no exact Binder/host or prior-stage type authority"));
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
}

impl HirVisitorV1 for RuleDependencyCollectorV2<'_> {
    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        if let BoolExprKindV1::Not(inner) = &node.kind {
            self.negated = !self.negated;
            self.visit_bool(inner);
            self.negated = !self.negated;
        } else {
            quanta_gqlang_v2_hir::walk_bool(self, node);
        }
    }

    fn visit_value(&mut self, node: &ValueExprHirV1) {
        let previous_aggregate = self.aggregated;
        if let ValueExprKindV1::Call(_) = &node.kind {
            let key = SourceQualifiedNodeIdV2 { source_id: self.source_id, node_id: node.id };
            let Some(fact) = self.facts.get(key) else {
                if self.error.is_none() { self.error = Some(document_failure("sema.document_validation_failed", self.source_id, node.span,
                    "types owner omitted rule callable fact")); }
                return;
            };
            match self.targets.get(&node.id) {
                Some(RuleCallableTargetV2::SourcePredicate(id)) => {
                    let target = ResolvedCallTargetV2::Predicate(id.clone());
                    if fact.resolved_call() != Some(&target) || self.aggregated && self.negated {
                        if self.error.is_none() { self.error = Some(document_failure("sema.invalid_item_shape", self.source_id, node.span,
                            "source call target or combined aggregate/negation polarity cannot be sealed")); }
                        return;
                    }
                    let polarity = if self.aggregated { RuleCallPolarityV2::Aggregated }
                        else if self.negated { RuleCallPolarityV2::Negated }
                        else { RuleCallPolarityV2::Positive };
                    match self.extended.combined().try_check_location(location(self.source_id, node.span)) {
                        Ok(checked) => self.edges.push((key, target, polarity, checked)),
                        Err(error) if self.error.is_none() => self.error = Some(document_failure(
                            "sema.document_validation_failed", self.source_id, node.span, error.to_string())),
                        Err(_) => {}
                    }
                }
                Some(RuleCallableTargetV2::Builtin(_)) => {
                    if !matches!(fact.resolved_call(), Some(ResolvedCallTargetV2::Builtin(_))) && self.error.is_none() {
                        self.error = Some(document_failure("sema.document_validation_failed", self.source_id, node.span,
                            "builtin call target differs from types-owned fact"));
                    }
                }
                Some(RuleCallableTargetV2::HostPredicate(callable)) => {
                    if fact.resolved_call() != Some(&ResolvedCallTargetV2::HostPredicate(callable.key().clone()))
                        && self.error.is_none() {
                        self.error = Some(document_failure("sema.document_validation_failed", self.source_id, node.span,
                            "host call target differs from types-owned exact catalog key"));
                    }
                }
                Some(RuleCallableTargetV2::Unknown) | None => {
                    if self.error.is_none() { self.error = Some(document_failure("sema.document_validation_failed", self.source_id, node.span,
                        "rule callable lacks an exact source/host authority target")); }
                }
            }
            self.aggregated |= fact.effects.aggregation == AggregationClassV1::Aggregate;
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
        self.aggregated = previous_aggregate;
    }
}

impl HirVisitorV1 for DeclarationTypeInputsV2<'_> {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        let value = match &node.kind {
            ValueExprKindV1::Symbol(_) => match self.bound.target_of(node.id) {
                Some(ResolvedTargetV1::Declaration { node: target, .. }) => self.declaration_bodies.get(target).copied().map(|body| {
                    ResolvedTypeInputValueV2::BindingFromDeclaration(SourceQualifiedNodeIdV2 { source_id: self.source_id, node_id: body })
                }),
                _ => None,
            },
            ValueExprKindV1::Call(call) => self.builtins.family_id(&call.callee.name)
                .map(|id| ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(id))),
            _ => None,
        };
        if matches!(node.kind, ValueExprKindV1::Symbol(_) | ValueExprKindV1::Call(_)) {
            if let Some(value) = value {
                self.entries.push(ResolvedTypeInputEntryV2 {
                    key: SourceQualifiedNodeIdV2 { source_id: self.source_id, node_id: node.id }, value,
                });
            } else if self.error.is_none() {
                let code = if matches!(node.kind, ValueExprKindV1::Call(_)) { "types.unknown_builtin" } else { "sema.unresolved_binding_type" };
                self.error = Some(document_failure(code, self.source_id, node.span, "no closed authoritative type for value reference"));
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
    let key_of = |id| SourceQualifiedNodeIdV2 { source_id, node_id: id };
    let mut child = |child: &ValueExprHirV1, output: &mut BTreeMap<_, _>| {
        materialize_declaration_expr(child, source_id, bound, facts, extended, output, steps, position).map(|typed| typed.key())
    };
    let key = key_of(node.id);
    let fact = facts.get(key).ok_or_else(|| document_failure("sema.document_validation_failed", source_id, node.span, "types owner omitted expression fact"))?;
    if !matches!(fact.state, ExpressionTypeStateV2::Known(_)) {
        return Err(document_failure("sema.unknown_type_in_closed_compile", source_id, node.span, "closed expression cannot retain unknown type"));
    }
    let shape = match &node.kind {
        ValueExprKindV1::Literal(_) => TypedExprShapeV2::Literal(fact.literal.clone().ok_or_else(|| document_failure("sema.document_validation_failed", source_id, node.span, "types owner omitted literal fact"))?),
        ValueExprKindV1::Symbol(_) => {
            let (owner, ordinal, kind, name) = match bound.target_of(node.id) {
                Some(ResolvedTargetV1::Declaration { name, node: owner, .. }) => (*owner, 0, ResolvedBindingKindV2::Declaration, name.clone()),
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
                _ => return Err(document_failure("sema.unresolved_binding_type", source_id, node.span, "binding lacks a closed canonical identity")),
            };
            let binding = ResolvedValueBindingV2::try_new(source_id, owner, ordinal, kind, name)
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, node.span, error))?;
            TypedExprShapeV2::Binding(binding)
        }
        ValueExprKindV1::Path(path) => {
            let base = child(&path.base, output)?;
            let mut segments = Vec::new();
            for segment in path.segments.as_slice() {
                let raw = location(source_id, segment.span);
                let checked = extended.combined().try_check_location(raw)
                    .map_err(|error| document_failure("sema.document_validation_failed", source_id, segment.span, error.to_string()))?;
                segments.push(TypedPathSegmentV2::new(segment.name.clone(), checked));
            }
            TypedExprShapeV2::Path { base, segments }
        },
        ValueExprKindV1::Unary(unary) => TypedExprShapeV2::Unary {
            operator: fact.operator_id.ok_or_else(|| document_failure("sema.document_validation_failed", source_id, node.span, "types owner omitted unary operator"))?,
            operand: child(&unary.operand, output)?,
        },
        ValueExprKindV1::Binary(binary) => TypedExprShapeV2::Binary {
            operator: fact.operator_id.ok_or_else(|| document_failure("sema.document_validation_failed", source_id, node.span, "types owner omitted binary operator"))?,
            lhs: child(&binary.lhs, output)?, rhs: child(&binary.rhs, output)?,
        },
        ValueExprKindV1::Call(call) => {
            let mut arguments = Vec::new();
            for arg in call.args().as_slice() {
                arguments.push(match arg {
                    quanta_gqlang_v2_hir::CallArgHirV1::Expr(expr) => TypedCallArgumentV2::Expression(child(expr, output)?),
                    quanta_gqlang_v2_hir::CallArgHirV1::Star { .. } => TypedCallArgumentV2::Star,
                });
            }
            let target = fact.resolved_call().cloned().ok_or_else(|| document_failure("sema.document_validation_failed", source_id, node.span, "types owner omitted callable target"))?;
            TypedExprShapeV2::Call { target, arguments }
        }
        ValueExprKindV1::Paren(inner) => TypedExprShapeV2::Paren { inner: child(inner, output)? },
        ValueExprKindV1::Record(record) => {
            let mut fields = Vec::new();
            for field in &record.fields {
                let value = child(&field.value, output)?;
                let raw = location(source_id, field.name_span);
                let checked = extended.combined().try_check_location(raw)
                    .map_err(|error| document_failure("sema.document_validation_failed", source_id, field.name_span, error.to_string()))?;
                fields.push(TypedRecordFieldV2::try_new(field.name.clone(), checked, value)
                    .map_err(|error| document_failure("sema.document_validation_failed", source_id, field.name_span, error))?);
            }
            TypedExprShapeV2::Record(fields)
        }
        ValueExprKindV1::List(list) => {
            let mut elements = Vec::new();
            for element in &list.elements { elements.push(child(element, output)?); }
            TypedExprShapeV2::List(elements)
        }
    };
    let location = location(source_id, node.span);
    let star = matches!(&shape, TypedExprShapeV2::Call { arguments, .. } if arguments.iter().any(|arg| matches!(arg, TypedCallArgumentV2::Star)));
    let context = AggregateContextV2::try_new(position, fact.effects.aggregation, star, location)?;
    let checked = extended.combined().try_check_location(location)
        .map_err(|error| document_failure("sema.document_validation_failed", source_id, node.span, error.to_string()))?;
    let typed = TypedExprV2::try_from_fact(key, fact, context, shape, checked)?;
    if output.insert(key, typed.clone()).is_some() {
        return Err(document_failure("sema.document_validation_failed", source_id, node.span, "duplicate typed expression identity"));
    }
    Ok(typed)
}

fn materialize_bool(
    node: &BoolExprHirV1,
    source_id: SourceIdV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &ExtendedSourceRegistryViewV2,
    output: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    steps: &mut SemanticStepCounterV2,
    position: AggregatePositionV2,
) -> Result<TypedBoolExprV2, FrontendFailureV2> {
    steps.charge(source_id, node.span)?;
    let key = |id| SourceQualifiedNodeIdV2 { source_id, node_id: id };
    let fact = facts.get(key(node.id)).ok_or_else(|| document_failure("sema.document_validation_failed", source_id, node.span, "types owner omitted rule boolean fact"))?;
    let shape = match &node.kind {
        BoolExprKindV1::Literal(_) => TypedExprShapeV2::Literal(fact.literal.clone().ok_or_else(||
            document_failure("sema.document_validation_failed", source_id, node.span, "types owner omitted boolean literal fact"))?),
        BoolExprKindV1::Value(value) => {
            let value = materialize_declaration_expr(value, source_id, bound, facts, extended, output, steps, position)?;
            TypedExprShapeV2::BoolValue { value: value.key() }
        }
        BoolExprKindV1::Compare(compare) => {
            let lhs = materialize_declaration_expr(&compare.lhs, source_id, bound, facts, extended, output, steps, position)?;
            let rhs = materialize_declaration_expr(&compare.rhs, source_id, bound, facts, extended, output, steps, position)?;
            TypedExprShapeV2::Compare {
                operator: fact.operator_id.ok_or_else(|| document_failure("sema.document_validation_failed", source_id, node.span, "types owner omitted comparison operator"))?,
                lhs: lhs.key(), rhs: rhs.key(),
            }
        }
        BoolExprKindV1::And(binary) | BoolExprKindV1::Or(binary) => {
            let lhs = materialize_bool(&binary.lhs, source_id, bound, facts, extended, output, steps, position)?;
            let rhs = materialize_bool(&binary.rhs, source_id, bound, facts, extended, output, steps, position)?;
            let operator = fact.operator_id.ok_or_else(|| document_failure("sema.document_validation_failed", source_id, node.span, "types owner omitted logical operator"))?;
            if matches!(node.kind, BoolExprKindV1::And(_)) {
                TypedExprShapeV2::And { operator, lhs: lhs.expression().key(), rhs: rhs.expression().key() }
            } else {
                TypedExprShapeV2::Or { operator, lhs: lhs.expression().key(), rhs: rhs.expression().key() }
            }
        }
        BoolExprKindV1::Not(inner) => {
            let operand = materialize_bool(inner, source_id, bound, facts, extended, output, steps, position)?;
            TypedExprShapeV2::Not {
                operator: fact.operator_id.ok_or_else(|| document_failure("sema.document_validation_failed", source_id, node.span, "types owner omitted negation operator"))?,
                operand: operand.expression().key(),
            }
        }
        BoolExprKindV1::Paren(inner) => {
            let inner = materialize_bool(inner, source_id, bound, facts, extended, output, steps, position)?;
            TypedExprShapeV2::Paren { inner: inner.expression().key() }
        }
        BoolExprKindV1::LogicalQuantifier(quantifier) => {
            let body = materialize_bool(&quantifier.body, source_id, bound, facts, extended, output, steps, position)?;
            let binders = quantifier.binders.as_slice().iter().enumerate().map(|(ordinal, binder)|
                ResolvedValueBindingV2::try_new(source_id, node.id, ordinal, ResolvedBindingKindV2::Quantified, binder.name.clone())
                    .map_err(|error| document_failure("sema.document_validation_failed", source_id, binder.span, error))
            ).collect::<Result<Vec<_>, _>>()?;
            TypedExprShapeV2::LogicalQuantifier { kind: quantifier.kind, binders, body: body.expression().key() }
        }
        BoolExprKindV1::RelationExists(_) => return Err(document_failure("sema.invalid_item_shape", source_id, node.span,
            "relational EXISTS is not admitted inside a source rule body")),
    };
    let raw = location(source_id, node.span);
    let context = AggregateContextV2::try_new(position, fact.effects.aggregation, false, raw)?;
    let checked = extended.combined().try_check_location(raw)
        .map_err(|error| document_failure("sema.document_validation_failed", source_id, node.span, error.to_string()))?;
    let typed = TypedExprV2::try_from_fact(key(node.id), fact, context, shape, checked)?;
    let bool_fact = TypedBoolExprV2::try_new(typed.clone(), raw)?;
    if output.insert(key(node.id), typed).is_some() {
        return Err(document_failure("sema.document_validation_failed", source_id, node.span, "duplicate typed boolean identity"));
    }
    Ok(bool_fact)
}

fn materialize_relation(
    relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &ExtendedSourceRegistryViewV2,
    expressions: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    steps: &mut SemanticStepCounterV2,
) -> Result<TypedRelationExprV2, FrontendFailureV2> {
    steps.charge(source_id, relation.span)?;
    let key = SourceQualifiedNodeIdV2 { source_id, node_id: relation.id };
    let checked = extended.combined().try_check_location(location(source_id, relation.span))
        .map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error.to_string()))?;
    if bound.relation_output(relation.id).is_none() {
        return Err(document_failure("sema.document_validation_failed", source_id, relation.span,
            "binder omitted relation output frame"));
    }
    match &relation.kind {
        quanta_gqlang_v2_hir::RelationExprKindV1::Source(source) => {
            let host = authority.relations().get(&source.name).ok_or_else(|| document_failure(
                "sema.unresolved_reference", source_id, relation.span,
                "relation source is absent from the sealed host catalog",
            ))?;
            let scan = ResolvedScanSourceV2::Relation(host.id().clone());
            let columns = host.relation_type().row().fields().iter().map(|field|
                OutputColumnContractV2::new(field.name().to_owned(), OutputColumnOriginV2::SourceField {
                    source: scan.clone(), scan_stage: key, field_name: field.name().to_owned(),
                }, host.location().clone())
            ).collect();
            let output = RelationOutputContractV2::try_new(
                host.relation_type().row().clone(), OutputProvenanceV2::new([scan.clone()]),
                columns, host.relation_type().cardinality(), Vec::new(), Vec::new(),
                ExpressionEffectsV2::pure_scalar(),
            ).map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))?;
            TypedRelationExprV2::scan(key, checked, scan, output)
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))
        }
        quanta_gqlang_v2_hir::RelationExprKindV1::Filter(filter) => {
            let input = materialize_relation(&filter.input, source_id, authority, bound, facts, extended, expressions, steps)?;
            let predicate = materialize_bool(&filter.predicate, source_id, bound, facts, extended,
                expressions, steps, AggregatePositionV2::Predicate)?;
            let previous = input.output();
            let output = RelationOutputContractV2::try_new(
                previous.row().clone(), previous.provenance().clone(), previous.columns().to_vec(),
                previous.cardinality(), previous.partition_keys().to_vec(), previous.ordering().to_vec(),
                previous.effects().compose([predicate.expression().effects()]),
            ).map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))?;
            TypedRelationExprV2::filter(key, checked, input, predicate, output)
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))
        }
        quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) => {
            let input = materialize_relation(&project.input, source_id, authority, bound, facts, extended, expressions, steps)?;
            let frame = bound.relation_output(relation.id).ok_or_else(|| document_failure(
                "sema.document_validation_failed", source_id, relation.span, "binder omitted projection output frame"))?;
            if frame.len() != project.items().len() {
                return Err(document_failure("sema.document_validation_failed", source_id, relation.span,
                    "binder projection output differs from authored item count"));
            }
            let mut fields = Vec::new();
            let mut columns = Vec::new();
            let mut keys = Vec::new();
            let mut effects = Vec::new();
            for (ordinal, (item, binding)) in project.items().iter().zip(frame).enumerate() {
                if binding.identity() != (relation.id, ordinal) {
                    return Err(document_failure("sema.document_validation_failed", source_id, item.value.span,
                        "projection output does not retain its Binder-owned ordinal"));
                }
                let value = materialize_declaration_expr(&item.value, source_id, bound, facts, extended,
                    expressions, steps, AggregatePositionV2::Projection)?;
                let ExpressionTypeStateV2::Known(ty) = value.type_state() else {
                    return Err(document_failure("sema.unknown_type_in_closed_compile", source_id, item.value.span,
                        "projection field has no concrete value type"));
                };
                let name_span = item.alias.as_ref().map_or(item.value.span, |alias| alias.span);
                let field_location = extended.combined().try_check_location(location(source_id, name_span))
                    .map_err(|error| document_failure("sema.document_validation_failed", source_id, name_span, error.to_string()))?;
                fields.push(RecordFieldTypeV1::try_new(&binding.name, ty.clone(), field_location.clone())
                    .map_err(|error| document_failure(error.code(), source_id, name_span, error.to_string()))?);
                columns.push(OutputColumnContractV2::new(binding.name.clone(), OutputColumnOriginV2::Expression(value.key()), field_location));
                keys.push(value.key());
                effects.push(value.effects());
            }
            let row = RecordTypeV1::try_new(fields)
                .map_err(|error| document_failure(error.code(), source_id, relation.span, error.to_string()))?;
            let previous = input.output();
            let output = RelationOutputContractV2::try_new(row, previous.provenance().clone(), columns,
                previous.cardinality(), Vec::new(), Vec::new(), previous.effects().compose(effects))
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))?;
            TypedRelationExprV2::try_project(key, checked, input, keys, output)
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))
        }
        quanta_gqlang_v2_hir::RelationExprKindV1::Join(join) => {
            let (left_node, right_node, predicate_node) = match join {
                quanta_gqlang_v2_hir::JoinHirV1::Inner { left, right, on } => (left, right, Some(on)),
                quanta_gqlang_v2_hir::JoinHirV1::Cross { left, right } => (left, right, None),
            };
            let left = materialize_relation(left_node, source_id, authority, bound, facts, extended, expressions, steps)?;
            let right = materialize_relation(right_node, source_id, authority, bound, facts, extended, expressions, steps)?;
            let predicate = predicate_node.map(|node| materialize_bool(node, source_id, bound, facts,
                extended, expressions, steps, AggregatePositionV2::Predicate)).transpose()?;
            let left_output = left.output();
            let right_output = right.output();
            let mut fields = Vec::new();
            let mut columns = Vec::new();
            let mut names = BTreeSet::new();
            for (field, column) in left_output.row().fields().iter().zip(left_output.columns())
                .chain(right_output.row().fields().iter().zip(right_output.columns())) {
                if !names.insert(field.name()) {
                    return Err(document_failure("sema.duplicate_output_column", source_id, relation.span,
                        "join operands produce the same unqualified output column name"));
                }
                fields.push(RecordFieldTypeV1::try_new(field.name(), field.value_type().clone(), column.location().clone())
                    .map_err(|error| document_failure(error.code(), source_id, relation.span, error.to_string()))?);
                columns.push(column.clone());
            }
            let row = RecordTypeV1::try_new(fields)
                .map_err(|error| document_failure(error.code(), source_id, relation.span, error.to_string()))?;
            let provenance = OutputProvenanceV2::new(left_output.provenance().iter().cloned()
                .chain(right_output.provenance().iter().cloned()));
            let effects = ExpressionEffectsV2::pure_scalar().compose([
                left_output.effects(), right_output.effects(),
                predicate.as_ref().map_or(ExpressionEffectsV2::pure_scalar(), |on| on.expression().effects()),
            ]);
            let output = RelationOutputContractV2::try_new(row, provenance, columns,
                quanta_gqlang_v2_types::CardinalityV1::Bag, Vec::new(), Vec::new(), effects)
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))?;
            match predicate {
                Some(on) => TypedRelationExprV2::join(key, checked, left, right, on, output),
                None => TypedRelationExprV2::cross(key, checked, left, right, output),
            }.map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))
        }
        quanta_gqlang_v2_hir::RelationExprKindV1::OrderBy(order) => {
            let input = materialize_relation(&order.input, source_id, authority, bound, facts, extended, expressions, steps)?;
            let mut keys = Vec::new();
            let mut ordering = Vec::new();
            let mut effects = Vec::new();
            for item in order.keys() {
                let value = materialize_declaration_expr(&item.value, source_id, bound, facts, extended,
                    expressions, steps, AggregatePositionV2::OrderKey)?;
                if value.effects().aggregation == AggregationClassV1::Aggregate {
                    return Err(document_failure("sema.illegal_aggregate_context", source_id, item.value.span,
                        "ORDER BY cannot introduce an aggregate outside the grouped output"));
                }
                let ExpressionTypeStateV2::Known(ty) = value.type_state() else {
                    return Err(document_failure("sema.unknown_type_in_closed_compile", source_id, item.value.span,
                        "order key has no concrete value type"));
                };
                keys.push(value.key());
                ordering.push(OrderKeyContractV2::new(value.key(), item.direction, NullOrderV2::Last, ty.clone()));
                effects.push(value.effects());
            }
            let previous = input.output();
            let output = RelationOutputContractV2::try_new(
                previous.row().clone(), previous.provenance().clone(), previous.columns().to_vec(),
                quanta_gqlang_v2_types::CardinalityV1::Sequence, previous.partition_keys().to_vec(),
                ordering, previous.effects().compose(effects),
            ).map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))?;
            TypedRelationExprV2::try_sort(key, checked, input, keys, output)
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))
        }
        quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(group) => {
            let input = materialize_relation(&group.input, source_id, authority, bound, facts, extended, expressions, steps)?;
            let frame = bound.relation_output(relation.id).ok_or_else(|| document_failure(
                "sema.document_validation_failed", source_id, relation.span, "binder omitted group key output frame"))?;
            if frame.len() != group.keys().len() || bound.partition_input(relation.id) != bound.relation_output(group.input.id) {
                return Err(document_failure("sema.document_validation_failed", source_id, relation.span,
                    "group output and separately sealed partition input disagree with Binder"));
            }
            let mut fields = Vec::new();
            let mut columns = Vec::new();
            let mut keys = Vec::new();
            let mut effects = Vec::new();
            for (ordinal, (key_expr, binding)) in group.keys().iter().zip(frame).enumerate() {
                if binding.identity() != (relation.id, ordinal) {
                    return Err(document_failure("sema.document_validation_failed", source_id, key_expr.span,
                        "group key output lost its Binder-owned ordinal"));
                }
                let value = materialize_declaration_expr(key_expr, source_id, bound, facts, extended,
                    expressions, steps, AggregatePositionV2::GroupKey)?;
                let ExpressionTypeStateV2::Known(ty) = value.type_state() else {
                    return Err(document_failure("sema.unknown_type_in_closed_compile", source_id, key_expr.span,
                        "group key has no concrete value type"));
                };
                let field_location = extended.combined().try_check_location(location(source_id, binding.span))
                    .map_err(|error| document_failure("sema.document_validation_failed", source_id, binding.span, error.to_string()))?;
                fields.push(RecordFieldTypeV1::try_new(&binding.name, ty.clone(), field_location.clone())
                    .map_err(|error| document_failure(error.code(), source_id, binding.span, error.to_string()))?);
                columns.push(OutputColumnContractV2::new(binding.name.clone(), OutputColumnOriginV2::Expression(value.key()), field_location));
                keys.push(value.key());
                effects.push(value.effects());
            }
            let row = RecordTypeV1::try_new(fields)
                .map_err(|error| document_failure(error.code(), source_id, relation.span, error.to_string()))?;
            let output = RelationOutputContractV2::try_new(row, input.output().provenance().clone(), columns,
                quanta_gqlang_v2_types::CardinalityV1::Set, keys.clone(), Vec::new(),
                input.output().effects().compose(effects),
            ).map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))?;
            TypedRelationExprV2::try_group(key, checked, input, keys, output)
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, relation.span, error))
        }
    }
}

fn resolve_graph_item(
    graph: &quanta_gqlang_v2_hir::GraphQueryHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
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
        (GraphReferenceRoleV2::Reference, endpoints.reference.as_ref()),
        (GraphReferenceRoleV2::Pointee, endpoints.pointee.as_ref()),
        (GraphReferenceRoleV2::Callable, endpoints.callable.as_ref()),
        (GraphReferenceRoleV2::Scope, constraints.scope.as_ref()),
        (GraphReferenceRoleV2::Window, constraints.window.as_ref()),
        (GraphReferenceRoleV2::Frontier, constraints.frontier.as_ref()),
    ];
    let repeated = constraints.via.iter().map(|reference| (GraphReferenceRoleV2::Via, reference))
        .chain(constraints.through.iter().map(|reference| (GraphReferenceRoleV2::Through, reference)))
        .chain(constraints.sanitizers.iter().map(|reference| (GraphReferenceRoleV2::Sanitizer, reference)));
    for (role, reference) in singular.into_iter().filter_map(|(role, reference)| reference.map(|reference| (role, reference))).chain(repeated) {
        steps.charge(source_id, reference.span)?;
        let location = location(source_id, reference.span);
        let expected = match role {
            GraphReferenceRoleV2::Source | GraphReferenceRoleV2::Sink | GraphReferenceRoleV2::Anchor
            | GraphReferenceRoleV2::Reference | GraphReferenceRoleV2::Pointee
            | GraphReferenceRoleV2::Via | GraphReferenceRoleV2::Through => crate::GraphReferenceKindV2::Entity,
            GraphReferenceRoleV2::Callable => crate::GraphReferenceKindV2::Callable,
            GraphReferenceRoleV2::Sanitizer => crate::GraphReferenceKindV2::Sanitizer,
            GraphReferenceRoleV2::Scope => crate::GraphReferenceKindV2::Scope,
            GraphReferenceRoleV2::Window => crate::GraphReferenceKindV2::Window,
            GraphReferenceRoleV2::Frontier => crate::GraphReferenceKindV2::Frontier,
        };
        let target = authority.graph_references().resolve(&reference.name, expected, location)?;
        let checked = extended.combined().try_check_location(location)
            .map_err(|error| document_failure("sema.document_validation_failed", source_id, reference.span, error.to_string()))?;
        references.push(ResolvedGraphReferenceV2::try_new(role, target, checked)
            .map_err(|error| document_failure("sema.invalid_item_shape", source_id, reference.span, error))?);
    }
    let direction = constraints.direction.as_ref().map(|direction| GraphDirectionV2::try_from_keyword(&direction.keyword))
        .transpose().map_err(|error| document_failure("sema.invalid_item_shape", source_id, graph.span, error))?;
    if constraints.depth.is_some_and(|depth| depth > authority.budget().max_witness_depth()) {
        return Err(document_failure("frontend.resource_limit_exceeded", source_id, graph.span,
            "FIND depth exceeds the frontend witness-depth budget"));
    }
    let graph_location = extended.combined().try_check_location(location(source_id, graph.span))
        .map_err(|error| document_failure("sema.document_validation_failed", source_id, graph.span, error.to_string()))?;
    ResolvedGraphQueryV2::try_new(
        SemanticItemIdV2::new(source_id, graph.id), graph_location, graph.kind, references,
        constraints.depth, direction, constraints.summary_facets.clone(),
        GraphWitnessBoundsV2::new(authority.budget().max_witness_depth(), authority.budget().max_witnesses()),
    ).map_err(|error| document_failure("sema.invalid_item_shape", source_id, graph.span, error))
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
        let target = authority.graph_references().resolve(&reference.name, kind, location)?;
        let checked = extended.combined().try_check_location(location)
            .map_err(|error| document_failure("sema.document_validation_failed", source_id, reference.span, error.to_string()))?;
        Ok::<_, FrontendFailureV2>(ResolvedGraphUseV2::new(target, checked))
    };
    let kind = match &presentation.kind {
        quanta_gqlang_v2_hir::PresentationKindV1::Show { category } => ResolvedPresentationKindV2::Show { category: *category },
        quanta_gqlang_v2_hir::PresentationKindV1::Describe { kind, path } => {
            steps.charge(source_id, path.span)?;
            let location = location(source_id, path.span);
            let target = authority.graph_references().resolve_entity_subtype(&path.name, kind, location)?;
            let checked = extended.combined().try_check_location(location)
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, path.span, error.to_string()))?;
            let target = ResolvedGraphUseV2::new(target, checked);
            ResolvedPresentationKindV2::Describe { target }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Sample { query_text, mode, limit, scope } => {
            let scope = scope.as_ref().map(|reference| {
                steps.charge(source_id, reference.span)?;
                graph(reference, crate::GraphReferenceKindV2::Scope)
            }).transpose()?;
            ResolvedPresentationKindV2::Sample { query_text: query_text.clone(), mode: *mode, limit: *limit, scope }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Compare { baseline_ref, candidate_ref, metric } => {
            steps.charge(source_id, baseline_ref.span)?;
            steps.charge(source_id, candidate_ref.span)?;
            ResolvedPresentationKindV2::Compare {
                baseline: graph(baseline_ref, crate::GraphReferenceKindV2::Snapshot)?,
                candidate: graph(candidate_ref, crate::GraphReferenceKindV2::Snapshot)?, metric: *metric,
            }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Export { source_ref, format, profile } => {
            steps.charge(source_id, source_ref.span)?;
            let source = graph(source_ref, crate::GraphReferenceKindV2::ExportSource)?;
            let profile = profile.as_ref().map(|reference| {
                steps.charge(source_id, reference.span)?;
                let checked = extended.combined().try_check_location(location(source_id, reference.span))
                    .map_err(|error| FrontendTransactionErrorV2::Document(document_failure("sema.document_validation_failed", source_id, reference.span, error.to_string())))?;
                let proof = validator.resolve_reference(DocumentCatalogReferenceInputV2 {
                    module_id: None, local_name: reference.name.clone(),
                    expected_kind: NamespaceKindV2::Profile, location: checked,
                }).map_err(module_failure)?;
                Ok::<_, FrontendTransactionErrorV2>(ResolvedCatalogReferenceV2::from_proof(&proof))
            }).transpose()?;
            ResolvedPresentationKindV2::Export { source, format: *format, profile }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Explain { target_ref, evidence, truth } => {
            steps.charge(source_id, target_ref.span)?;
            let raw = location(source_id, target_ref.span);
            let checked = extended.combined().try_check_location(raw)
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, target_ref.span, error.to_string()))?;
            let rule = validator.probe_reference(DocumentCatalogReferenceInputV2 {
                module_id: None, local_name: target_ref.name.clone(),
                expected_kind: NamespaceKindV2::Rule, location: checked.clone(),
            }).map_err(module_failure)?;
            let entity = authority.graph_references().probe(&target_ref.name, crate::GraphReferenceKindV2::Entity, raw)?;
            let target = match (rule, entity) {
                (Some(rule), None) => ResolvedExplainTargetV2::Rule(ResolvedCatalogReferenceV2::from_proof(&rule)),
                (None, Some(entity)) => ResolvedExplainTargetV2::Entity(ResolvedGraphUseV2::new(entity, checked)),
                (Some(rule), Some(entity)) => return Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
                    FrontendStageV2::Resolution, "sema.ambiguous_reference", FrontendSeverityV2::Error,
                    raw, vec![rule.location().location(), entity.declaration_location().location()],
                    "EXPLAIN target resolves in both Rule and Entity namespaces",
                )).into()),
                (None, None) => return Err(document_failure("sema.unresolved_reference", source_id, target_ref.span,
                    "EXPLAIN target is absent from Rule and Entity authorities").into()),
            };
            ResolvedPresentationKindV2::Explain { target, evidence: *evidence, truth: *truth }
        }
    };
    ResolvedPresentationV2::try_new(SemanticItemIdV2::new(source_id, presentation.id), kind)
        .map_err(|error| document_failure("sema.invalid_item_shape", source_id, presentation.span, error).into())
}

fn resolve_check_item(
    check: &quanta_gqlang_v2_hir::LegalityHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedCheckV2, FrontendTransactionErrorV2> {
    steps.charge(source_id, check.rule_ref.span)?;
    let rule_location = location(source_id, check.rule_ref.span);
    let checked_rule = extended.combined().try_check_location(rule_location)
        .map_err(|error| document_failure("sema.document_validation_failed", source_id, check.rule_ref.span, error.to_string()))?;
    let rule = validator.resolve_reference(DocumentCatalogReferenceInputV2 {
        module_id: None, local_name: check.rule_ref.name.clone(),
        expected_kind: NamespaceKindV2::Rule, location: checked_rule,
    }).map_err(module_failure)?;
    let resolve_optional = |reference: Option<&quanta_gqlang_v2_hir::GraphRefV1>, kind, steps: &mut SemanticStepCounterV2| {
        reference.map(|reference| {
            steps.charge(source_id, reference.span)?;
            let raw = location(source_id, reference.span);
            let target = authority.graph_references().resolve(&reference.name, kind, raw)?;
            let checked = extended.combined().try_check_location(raw)
                .map_err(|error| document_failure("sema.document_validation_failed", source_id, reference.span, error.to_string()))?;
            Ok::<_, FrontendFailureV2>(ResolvedGraphUseV2::new(target, checked))
        }).transpose()
    };
    let scope = resolve_optional(check.scope.as_ref(), crate::GraphReferenceKindV2::Scope, steps)?;
    let pattern = resolve_optional(check.pattern.as_ref(), crate::GraphReferenceKindV2::Pattern, steps)?;
    let nested = check.semantic_confirm.as_ref().map(|graph| resolve_graph_item(graph, source_id, authority, extended, steps)).transpose()?;
    let expected_nested = check.semantic_confirm.as_ref().map(|graph| SemanticItemIdV2::new(source_id, graph.id));
    ResolvedCheckV2::try_new(
        SemanticItemIdV2::new(source_id, check.id), expected_nested,
        ResolvedCatalogReferenceV2::from_proof(&rule), scope, pattern, nested,
        check.confirm, check.emit, check.mode,
    ).map_err(|error| document_failure("sema.invalid_item_shape", source_id, check.span, error).into())
}

/// Projection of the modules-owned frozen graph into binder's read-only
/// imported-target interface; no precedence, search, or ID is recomputed.
struct ModuleImportAdapterV2<'a>(&'a ResolvedGraphV2);

impl ModuleGraphViewV1 for ModuleImportAdapterV2<'_> {
    fn imported_targets(&self, name: &str) -> Vec<String> {
        self.0.visible_bindings().iter()
            .filter(|binding| binding.namespace_kind() == NamespaceKindV2::Value && binding.local_name() == name)
            .map(|binding| format!("{}::{}", binding.resolved_asset_id().module_id().as_str(), binding.resolved_asset_id().local_name()))
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
    let checked = extended.combined().try_check_location(location(source_id, reference.span))
        .map_err(|error| document_failure("sema.document_validation_failed", source_id, reference.span, error.to_string()))?;
    let target = validator.resolve_reference(DocumentCatalogReferenceInputV2 {
        module_id: None, local_name: reference.name.clone(), expected_kind: family.target_kind(),
        location: checked.clone(),
    }).map_err(module_failure)?;
    edges.push(DocumentCatalogFamilyReferenceInputV2 {
        family, source: source.clone(), target: target.clone(), location: checked,
    });
    Ok(ResolvedCatalogReferenceV2::from_proof(&target))
}

fn resolve_document_bodies(
    resolved: &ResolvedDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    predeclared: &BTreeMap<HirNodeIdV1, PredeclaredDocumentAssetV2>,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<BodyResolutionV2, FrontendTransactionErrorV2> {
    let checked = resolved.declared.checked();
    let syntax = checked.checked().document();
    struct RuleRelationExistsV2(Option<SpanV1>);
    impl HirVisitorV1 for RuleRelationExistsV2 {
        fn visit_bool(&mut self, node: &quanta_gqlang_v2_hir::BoolExprHirV1) {
            if matches!(node.kind, quanta_gqlang_v2_hir::BoolExprKindV1::RelationExists(_)) {
                if self.0.is_none() { self.0 = Some(node.span); }
            } else { quanta_gqlang_v2_hir::walk_bool(self, node); }
        }
    }
    for item in &syntax.items {
        if let ProgramItemHirV1::Rule(rule) = item {
            if !matches!(rule.head.kind, ValueExprKindV1::Symbol(_)) {
                return Err(document_failure("sema.invalid_item_shape", checked.source_id(), rule.head.span,
                    "source rules require one symbol head term").into());
            }
            let mut forbidden = RuleRelationExistsV2(None);
            forbidden.visit_bool(&rule.body);
            if let Some(span) = forbidden.0 {
                return Err(document_failure("sema.invalid_item_shape", checked.source_id(), span,
                    "relational EXISTS is not admitted inside a source rule body").into());
            }
        }
    }
    // This first source-backed slice exercises the canonical binder and type
    // kernels. Unsupported root families remain explicit RED until their
    // complete owner-produced reference inputs can be supplied.
    if let Some(item) = syntax.items.iter().find(|item| !matches!(item, ProgramItemHirV1::Query(_) | ProgramItemHirV1::Decl(_) | ProgramItemHirV1::Rule(_) | ProgramItemHirV1::Meta(_) | ProgramItemHirV1::GraphQuery(_) | ProgramItemHirV1::Presentation(_) | ProgramItemHirV1::Legality(_) | ProgramItemHirV1::CatalogDecl(_))) {
        return Err(document_failure("sema.document_validation_failed", checked.source_id(), item.span(), "semantic body and reference owner resolution is not yet available").into());
    }
    let graph = ModuleImportAdapterV2(authority.modules().resolved_graph());
    let mut binder = Binder::new(&graph);
    let declarations = syntax.items.iter().filter_map(|item| match item { ProgramItemHirV1::Decl(decl) => Some((**decl).clone()), _ => None }).collect::<Vec<_>>();
    let rules = syntax.items.iter().filter_map(|item| match item { ProgramItemHirV1::Rule(rule) => Some((**rule).clone()), _ => None }).collect::<Vec<_>>();
    binder.declare_program(&declarations, &rules);
    for decl in &declarations { binder.resolve_decl(decl, &[]); }
    for rule in &rules {
        let ValueExprKindV1::Symbol(head) = &rule.head.kind else {
            return Err(document_failure("sema.invalid_item_shape", checked.source_id(), rule.head.span,
                "source rules require one symbol head term").into());
        };
        binder.resolve_rule(rule, &[ParamV1::new(&head.name, rule.head.span, rule.id)]);
    }
    for item in &syntax.items {
        if let ProgramItemHirV1::Query(query) = item { binder.resolve_query(query); }
    }
    let bound = binder.finish();
    if let Some(error) = bound.diagnostics().first() {
        return Err(document_failure(error.code(), checked.source_id(), error.primary_span(), error.to_string()).into());
    }
    let declaration_bodies = declarations.iter().map(|decl| (decl.id, decl.body.id)).collect::<BTreeMap<_, _>>();
    let builtins = BuiltinRegistryV2::try_closed().map_err(type_failure)?;
    let mut rule_collector = RuleSymbolInputsV2 {
        source_id: checked.source_id(), bound: &bound, symbols: Vec::new(), calls: Vec::new(), error: None,
    };
    for rule in &rules { rule_collector.visit_rule(rule); }
    if let Some(error) = rule_collector.error { return Err(error.into()); }
    let predicates = rules.iter().map(|rule| {
        let predicate_id = ResolvedPredicateIdV2::try_new(&rule.name).map_err(type_failure)?;
        Ok::<_, FrontendTransactionErrorV2>(RulePredicateInputV2 {
            rule_node: SourceQualifiedNodeIdV2 { source_id: checked.source_id(), node_id: rule.id },
            predicate_id,
            head_binding: RuleBindingIdV2::new(checked.source_id(), rule.id, 0),
        })
    }).collect::<Result<Vec<_>, _>>()?;
    let mut rule_calls = Vec::new();
    let mut host_call_proofs = Vec::new();
    for (call, name, span) in &rule_collector.calls {
        let source = predicates.iter().find(|candidate| candidate.predicate_id.as_str() == name);
        let builtin = builtins.family_id(name);
        if source.is_some() || builtin.is_some() {
            let checked_location = extended.combined().try_check_location(location(checked.source_id(), *span))
                .map_err(|error| document_failure("sema.document_validation_failed", checked.source_id(), *span, error.to_string()))?;
            if validator.probe_reference(DocumentCatalogReferenceInputV2 {
                module_id: None, local_name: name.clone(), expected_kind: NamespaceKindV2::Predicate,
                location: checked_location,
            }).map_err(module_failure)?.is_some() {
                return Err(document_failure("sema.ambiguous_callable", checked.source_id(), *span,
                    "source predicate or builtin family conflicts with visible host Predicate").into());
            }
        }
        let target = match (source, builtin) {
            (Some(_), Some(_)) => return Err(document_failure("sema.ambiguous_callable", checked.source_id(), *span,
                "source predicate and builtin family share one call spelling").into()),
            (Some(predicate), None) => RuleCallableTargetV2::SourcePredicate(predicate.predicate_id.clone()),
            (None, Some(family)) => RuleCallableTargetV2::Builtin(family),
            (None, None) => {
                let checked_location = extended.combined().try_check_location(location(checked.source_id(), *span))
                    .map_err(|error| document_failure("sema.document_validation_failed", checked.source_id(), *span, error.to_string()))?;
                let proof = validator.resolve_host_predicate_call(DocumentCatalogReferenceInputV2 {
                    module_id: None, local_name: name.clone(), expected_kind: NamespaceKindV2::Predicate,
                    location: checked_location,
                }).map_err(module_failure)?;
                let target = RuleCallableTargetV2::HostPredicate(proof.callable().clone());
                host_call_proofs.push((*call, proof));
                target
            }
        };
        rule_calls.push(RuleCallInputV2 { call: *call, target });
    }
    let rule_proof = if rules.is_empty() { None } else {
        let solver = RuleTypeSolverV2::try_closed().map_err(type_failure)?;
        let budget = RuleTypeBudgetV2::try_new(authority.budget().max_rule_graph_nodes(),
            authority.budget().max_rule_graph_edges(), authority.budget().max_rule_steps()).map_err(type_failure)?;
        Some(solver.solve(checked.checked().as_checked(), extended, predicates.clone(), rule_collector.symbols.clone(), rule_calls.clone(), budget)
            .map_err(type_failure)?)
    };
    let mut collector = DeclarationTypeInputsV2 {
        source_id: checked.source_id(), bound: &bound, declaration_bodies: &declaration_bodies,
        builtins: &builtins, entries: Vec::new(), error: None,
    };
    for decl in &declarations { collector.visit_value(&decl.body); }
    if let Some(error) = collector.error { return Err(error.into()); }
    let mut query_inputs = QueryTypeInputsV2 {
        source_id: checked.source_id(), authority, bound: &bound, builtins: &builtins,
        owners: BTreeMap::new(), entries: Vec::new(), error: None,
    };
    for item in &syntax.items {
        if let ProgramItemHirV1::Query(query) = item { query_inputs.visit_relation(&query.body); }
    }
    if let Some(error) = query_inputs.error { return Err(error.into()); }
    collector.entries.extend(query_inputs.entries);
    if let Some(proof) = &rule_proof {
        for symbol in &rule_collector.symbols {
            let Some(ty) = proof.binding_type(symbol.binding) else {
                return Err(document_failure("sema.document_validation_failed", checked.source_id(), SpanV1::point(0),
                    "rule solver omitted a Binder-identified concrete binding").into());
            };
            collector.entries.push(ResolvedTypeInputEntryV2 { key: symbol.reference,
                value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(ty.clone())) });
        }
        for call in &rule_calls {
            let callable = match &call.target {
                RuleCallableTargetV2::Builtin(family) => ResolvedCallableInputV2::Builtin(*family),
                RuleCallableTargetV2::SourcePredicate(id) => {
                    let Some(signature) = proof.try_source_predicate_callable(id).map_err(type_failure)? else {
                        return Err(document_failure("sema.document_validation_failed", checked.source_id(), SpanV1::point(0),
                            "source predicate signature is absent from rule type proof").into());
                    };
                    ResolvedCallableInputV2::Predicate(signature)
                }
                RuleCallableTargetV2::HostPredicate(signature) =>
                    ResolvedCallableInputV2::HostPredicate(signature.clone()),
                RuleCallableTargetV2::Unknown =>
                    return Err(document_failure("sema.document_validation_failed", checked.source_id(), SpanV1::point(0),
                        "unknown rule callable cannot enter closed types inputs").into()),
            };
            collector.entries.push(ResolvedTypeInputEntryV2 { key: call.call,
                value: ResolvedTypeInputValueV2::Callable(callable) });
        }
    }
    let type_inputs = ResolvedTypeInputsV2::try_new(checked.checked().as_checked(), extended, collector.entries, Vec::new())
        .map_err(type_failure)?;
    let kernel = ExpressionTypeKernelV2::try_closed().map_err(type_failure)?;
    let facts = kernel.check_document(&type_inputs).map_err(type_failure)?;
    for (call, proof) in &host_call_proofs {
        let target = facts.get(*call).and_then(quanta_gqlang_v2_types::TypedExpressionFactV2::resolved_call);
        let Some(ResolvedCallTargetV2::HostPredicate(expected_key)) = target else {
            return Err(document_failure("sema.document_validation_failed", checked.source_id(), proof.location().location().span,
                "types kernel omitted host predicate call target").into());
        };
        validator.validate_host_predicate_proof(proof, expected_key, proof.location()).map_err(module_failure)?;
    }
    let rule_call_targets = rule_calls.iter().map(|call| (call.call.node_id, call.target.clone())).collect::<BTreeMap<_, _>>();
    let mut items = Vec::new();
    let mut expressions = BTreeMap::new();
    let mut certified = Vec::new();
    let mut family_references = Vec::new();
    let mut profile_admissions = Vec::new();
    for item in &checked.checked().document().items {
        steps.charge(checked.source_id(), item.span())?;
        match item {
            ProgramItemHirV1::Query(query) => {
                if query.result_shape.is_some() {
                    return Err(document_failure("sema.document_validation_failed", checked.source_id(), query.span,
                        "nested query result-shape resolution is not yet available").into());
                }
                let stage_key = SourceQualifiedNodeIdV2 { source_id: checked.source_id(), node_id: query.body.id };
                let item_location = extended.combined().try_check_location(location(checked.source_id(), query.span))
                    .map_err(|error| document_failure("sema.document_validation_failed", checked.source_id(), query.span, error.to_string()))?;
                let root = materialize_relation(&query.body, checked.source_id(), authority, &bound,
                    &facts, extended, &mut expressions, steps)?;
                let output = root.output().clone();
                let typed = TypedRelationV2::try_new(SemanticItemIdV2::new(checked.source_id(), query.id), item_location, stage_key, root, output)
                    .map_err(|error| document_failure("sema.document_validation_failed", checked.source_id(), query.span, error))?;
                items.push(SemanticItemV2::Relation(typed));
            }
            ProgramItemHirV1::GraphQuery(graph) => {
                items.push(SemanticItemV2::Graph(resolve_graph_item(graph, checked.source_id(), authority, extended, steps)?));
            }
            ProgramItemHirV1::Presentation(presentation) => {
                items.push(SemanticItemV2::Presentation(resolve_presentation_item(presentation, checked.source_id(), authority, extended, validator, steps)?));
            }
            ProgramItemHirV1::Legality(check) => {
                items.push(SemanticItemV2::Check(resolve_check_item(check, checked.source_id(), authority, extended, validator, steps)?));
            }
            ProgramItemHirV1::CatalogDecl(decl) => {
                let Some(asset) = predeclared.get(&decl.id) else {
                    return Err(document_failure("sema.document_validation_failed", checked.source_id(), decl.span, "catalog declaration lacks modules-owned predeclaration").into());
                };
                let mut resolve = |reference, family| resolve_catalog_family_reference(
                    reference, family, asset, checked.source_id(), extended, validator, steps, &mut family_references,
                );
                let (payload, provider_metadata) = match &decl.payload {
                    quanta_gqlang_v2_hir::CatalogPayloadV1::View(quanta_gqlang_v2_hir::ViewBodyV1::QueryRef(reference)) =>
                        (ResolvedCatalogPayloadV2::View { query_ref: Some(resolve(reference, DocumentCatalogReferenceFamilyV2::ViewQuery)?) }, None),
                    quanta_gqlang_v2_hir::CatalogPayloadV1::View(quanta_gqlang_v2_hir::ViewBodyV1::Inline(_)) =>
                        return Err(document_failure("sema.document_validation_failed", checked.source_id(), decl.span,
                            "inline VIEW requires the non-scan relation producer").into()),
                    quanta_gqlang_v2_hir::CatalogPayloadV1::Model { provider, source } =>
                        (ResolvedCatalogPayloadV2::Model { source: resolve(source, DocumentCatalogReferenceFamilyV2::ModelSource)? }, Some(provider.clone())),
                    quanta_gqlang_v2_hir::CatalogPayloadV1::Pack { source, activation } =>
                        (ResolvedCatalogPayloadV2::Pack { source: resolve(source, DocumentCatalogReferenceFamilyV2::PackSource)?, activation: *activation }, None),
                    quanta_gqlang_v2_hir::CatalogPayloadV1::Profile { capabilities, admission } => {
                        let capabilities = capabilities.iter().map(|reference| resolve(reference, DocumentCatalogReferenceFamilyV2::ProfileCapability))
                            .collect::<Result<Vec<_>, _>>()?;
                        let admission_proof = match admission {
                            quanta_gqlang_v2_hir::AdmissionV1::Allowlist => ProfileAdmissionV2::Allowlist,
                            quanta_gqlang_v2_hir::AdmissionV1::ReviewRequired => ProfileAdmissionV2::ReviewRequired,
                            quanta_gqlang_v2_hir::AdmissionV1::CertifiedOnly => ProfileAdmissionV2::CertifiedOnly,
                        };
                        profile_admissions.push(ProfileAdmissionInputV2 {
                            profile: CatalogAssetRefInputV2 {
                                module_id: asset.id().module_id().clone(), namespace_kind: NamespaceKindV2::Profile,
                                local_name: decl.name.clone(), location: asset.location().clone(),
                            },
                            admission: admission_proof, location: asset.location().clone(),
                        });
                        (ResolvedCatalogPayloadV2::Profile { capabilities, admission: *admission }, None)
                    }
                    quanta_gqlang_v2_hir::CatalogPayloadV1::Gate { checks, binding } => {
                        let checks = checks.iter().map(|reference| resolve(reference, DocumentCatalogReferenceFamilyV2::GateCheck))
                            .collect::<Result<Vec<_>, _>>()?;
                        (ResolvedCatalogPayloadV2::Gate { checks, binding: *binding }, None)
                    }
                    quanta_gqlang_v2_hir::CatalogPayloadV1::RuleBinding { check_ref } =>
                        (ResolvedCatalogPayloadV2::RuleBinding { check_ref: resolve(check_ref, DocumentCatalogReferenceFamilyV2::RuleBindingCheck)? }, None),
                };
                let catalog = ResolvedCatalogItemV2::try_new(
                    SemanticItemIdV2::new(checked.source_id(), decl.id), asset.id().clone(), provider_metadata, None, payload,
                ).map_err(|error| document_failure("sema.invalid_item_shape", checked.source_id(), decl.span, error))?;
                items.push(SemanticItemV2::Catalog(catalog));
            }
            ProgramItemHirV1::Decl(decl) => {
                let typed = materialize_declaration_expr(&decl.body, checked.source_id(), &bound, &facts, extended, &mut expressions, steps, AggregatePositionV2::Scalar)?;
                let declaration = ResolvedDeclarationV2::try_new(SemanticItemIdV2::new(checked.source_id(), decl.id), decl.name.clone(), typed)
                    .map_err(|error| document_failure("sema.document_validation_failed", checked.source_id(), decl.span, error))?;
                items.push(SemanticItemV2::Declaration(declaration));
            }
            ProgramItemHirV1::Rule(rule) => {
                let Some(predicate) = predicates.iter().find(|predicate| predicate.rule_node.node_id == rule.id) else {
                    return Err(document_failure("sema.document_validation_failed", checked.source_id(), rule.span,
                        "rule lacks a types-owned predicate predeclaration").into());
                };
                let head = materialize_declaration_expr(&rule.head, checked.source_id(), &bound, &facts, extended, &mut expressions, steps, AggregatePositionV2::RuleBody)?;
                let body = materialize_bool(&rule.body, checked.source_id(), &bound, &facts, extended, &mut expressions, steps, AggregatePositionV2::RuleBody)?;
                let Some(proof) = &rule_proof else {
                    return Err(document_failure("sema.document_validation_failed", checked.source_id(), rule.span,
                        "rule type proof is absent from the document transaction").into());
                };
                if !matches!(head.type_state(), ExpressionTypeStateV2::Known(ty)
                    if proof.predicate_type(&predicate.predicate_id) == Some(ty)) {
                    return Err(document_failure("sema.document_validation_failed", checked.source_id(), rule.head.span,
                        "rule head type disagrees with the predicate signature proof").into());
                }
                let mut dependencies = RuleDependencyCollectorV2 {
                    source_id: checked.source_id(), targets: &rule_call_targets, facts: &facts, extended,
                    negated: false, aggregated: false, edges: Vec::new(), error: None,
                };
                dependencies.visit_bool(&rule.body);
                if let Some(error) = dependencies.error { return Err(error.into()); }
                let expected_calls = dependencies.edges;
                let calls = expected_calls.iter().map(|(key, target, polarity, location)|
                    ResolvedRuleCallV2::new(*key, target.clone(), *polarity, location.clone())
                ).collect();
                let typed = ResolvedRuleV2::try_new(
                    SemanticItemIdV2::new(checked.source_id(), rule.id), head.key(), body.expression().key(),
                    &expected_calls, predicate.predicate_id.clone(), head, body, calls,
                ).map_err(|error| document_failure("sema.document_validation_failed", checked.source_id(), rule.span, error))?;
                items.push(SemanticItemV2::Rule(typed));
            }
            ProgramItemHirV1::Meta(meta) if matches!(meta.kind, quanta_gqlang_v2_hir::MetaKindV1::RequireCertified) => {
                let location = quanta_gqlang_v2_hir::LocatedSpanV2 { source_id: checked.source_id(), span: meta.span };
                let checked_location = extended.combined().try_check_location(location)
                    .map_err(|error| document_failure("sema.document_validation_failed", checked.source_id(), meta.span, error.to_string()))?;
                certified.push(RequireCertifiedInputV2 { location: checked_location.clone() });
                items.push(SemanticItemV2::Meta(ResolvedMetaV2::new(
                    SemanticItemIdV2::new(checked.source_id(), meta.id), checked_location,
                    ResolvedMetaKindV2::RequireCertified,
                )));
            }
            _ => return Err(document_failure(
                "sema.document_validation_failed", checked.source_id(), item.span(),
                "semantic body and reference owner resolution is not yet available",
            ).into()),
        }
    }
    Ok(BodyResolutionV2 {
        items,
        expressions,
        type_fact_count: facts.len(),
        catalog_input: DocumentCatalogFinalInputV2 {
            profile_admissions,
            activations: Vec::new(),
            require_certified_directives: certified,
            dependency_edges: Vec::new(),
            family_references,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_steps_fail_at_exact_bound_with_source_location() {
        let budget = FrontendBudgetV2::try_new(
            10, 10, 10, 10, 10, 10, 10, 10, 2, 10, 10, 10, 10, 10, 10, 10, 10, 10, 10,
        );
        let Ok(budget) = budget else { panic!("positive budget must be constructible") };
        let mut steps = SemanticStepCounterV2::new(&budget);
        let source_id = SourceIdV2::from_raw(7);
        let span = SpanV1::new(1, 2);
        assert!(steps.charge(source_id, span).is_ok());
        assert!(steps.charge(source_id, span).is_ok());
        let Err(error) = steps.charge(source_id, span) else { panic!("third semantic node exceeds two-step bound") };
        assert_eq!(error.diagnostics()[0].code(), "frontend.resource_limit_exceeded");
        assert_eq!(error.diagnostics()[0].primary(), location(source_id, span));
    }
}
