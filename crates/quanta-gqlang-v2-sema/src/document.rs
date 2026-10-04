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

mod bodies;
mod expression_materialization;
mod item_resolution;
mod query_materialization;
mod relation_materialization;
mod semantic_document;
mod syntax;
mod type_inputs;

use expression_materialization::{materialize_bool, materialize_declaration_expr};
use item_resolution::{
    resolve_catalog_family_reference, resolve_check_item, resolve_composition_explain,
    resolve_graph_item, resolve_graph_target_use, resolve_presentation_item, ModuleImportAdapterV2,
};
use query_materialization::{materialize_query, materialize_query_composition};

#[cfg(test)]
use semantic_document::{
    check_correlation_witnesses, check_nested_relation_finalizer, CorrelationWitnessV2,
};
#[cfg(test)]
use type_inputs::PartitionCandidateShapeV2;
use type_inputs::{
    capture_vector_witness, check_partition_candidate_shape, collect_partition_candidate_shapes,
    reject_nested_aggregate_calls, validate_capture_vector, DeclarationTypeInputsV2,
    QueryTypeInputsV2, RuleSymbolInputsV2,
};

/// Owned proof that all HIR roots share one valid document-wide identity and
/// source-length bound. Fields are private; the HIR owner checks exactly once.
#[derive(Clone, Debug)]
pub struct CheckedSyntaxDocumentV2 {
    checked: CheckedHirDocumentOwnedV2,
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

/// SHOW-only source declaration before semantic item construction.
///
/// The checked HIR supplies authored order and exact item IDs. The source
/// registry's first document extension retains the host owner and one inline
/// registration, so this stage does not create the generic declaration map or
/// order Vec. It is not a resolved semantic document or a plan.
#[derive(Debug)]
pub struct DeclaredShowSourceV2 {
    checked: CheckedSyntaxDocumentV2,
    extended: ExtendedSourceRegistryViewV2,
    source_anchor: CheckedLocatedSpanV2,
    budget_receipt: DownstreamBudgetReceiptV2,
}

impl DeclaredShowSourceV2 {
    /// Consume checked SHOW syntax and bind it to one exact closed authority.
    /// This standalone constructor has no lexical allocation owner; the
    /// pipeline's consuming wrapper must retain that earlier reservation.
    pub fn try_new(
        checked: CheckedSyntaxDocumentV2,
        authority: &ClosedFrontendAuthorityV2,
    ) -> Result<Self, FrontendFailureV2> {
        let source_id = checked.source_id();
        let source_len = checked.source_len();
        let document = checked.checked().document();
        quanta_gqlang_v2_hir::check_document_show_monotonic_v2(document).map_err(|_error| {
            document_failure(
                "sema.show_source_incomplete",
                source_id,
                SpanV1::point(0),
                "SHOW source does not satisfy the checked monotonic item contract",
            )
        })?;
        if document.items.len()
            > usize::try_from(authority.budget().max_document_items()).unwrap_or(usize::MAX)
        {
            return Err(document_failure(
                "frontend.resource_limit_exceeded",
                source_id,
                SpanV1::point(0),
                "document_items exceeds frontend limit",
            ));
        }
        let extended = authority
            .try_extend_document(source_id, source_len)
            .map_err(|error| {
                document_failure(error.code(), source_id, SpanV1::point(0), error.to_string())
            })?;
        if extended.source_len(source_id) != Some(source_len) {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                SpanV1::point(0),
                "checked SHOW source differs from the extended authority registry",
            ));
        }
        let source_anchor = extended
            .combined()
            .try_check_location(location(source_id, SpanV1::point(0)))
            .map_err(|error| {
                document_failure(error.code(), source_id, SpanV1::point(0), error.to_string())
            })?;
        Ok(Self {
            checked,
            extended,
            source_anchor,
            budget_receipt: authority.budget().downstream_receipt(),
        })
    }

    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.checked.source_id()
    }

    #[must_use]
    pub fn item_count(&self) -> usize {
        self.checked.checked().document().items.len()
    }

    #[must_use]
    pub fn source_anchor(&self) -> &CheckedLocatedSpanV2 {
        &self.source_anchor
    }

    #[must_use]
    pub fn source_len(&self) -> Option<usize> {
        self.extended.source_len(self.source_id())
    }

    /// Resolve one parser-issued SHOW item without constructing the generic
    /// semantic item map, catalog, or order vector. The checked document owns
    /// the exact item order and the extended registry owns its location proof.
    pub fn resolve_show_item_v2(
        &self,
        index: usize,
    ) -> Result<ResolvedPresentationV2, &'static str> {
        let source_id = self.source_id();
        let Some(ProgramItemHirV1::Presentation(presentation)) =
            self.checked.checked().document().items.get(index)
        else {
            return Err("sema.show_item_absent_or_other_family");
        };
        let quanta_gqlang_v2_hir::PresentationKindV1::Show { category } = &presentation.kind else {
            return Err("sema.presentation_is_not_show");
        };
        let location = self
            .extended
            .combined()
            .try_check_location(location(source_id, presentation.span))
            .map_err(|_error| "sema.show_location_registry_mismatch")?;
        ResolvedPresentationV2::try_new(
            SemanticItemIdV2::new(source_id, presentation.id),
            location,
            ResolvedPresentationKindV2::Show {
                category: *category,
            },
        )
        .map_err(|_error| "sema.invalid_show_item_shape")
    }

    /// The same closed authority budget that admitted the source declaration.
    #[must_use]
    pub const fn budget_receipt_v2(&self) -> DownstreamBudgetReceiptV2 {
        self.budget_receipt
    }
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
    Command,
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
            ProgramItemHirV1::Command(_) => Self::Command,
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
            SemanticItemV2::Command(_) => Self::Command,
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

#[expect(
    clippy::too_many_lines,
    reason = "legacy inline-view collection awaits phase extraction"
)]
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
            | ProgramItemHirV1::Meta(_)
            | ProgramItemHirV1::Command(_) => {
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
    let result = bodies::resolve_document_bodies(
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
                let typed = relation_materialization::materialize_relation_node(
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

#[cfg(test)]
#[path = "document/tests.rs"]
mod tests;
