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
    BindResultV1, Binder, BinderEvidenceReaderV3, BinderKindV1, ModuleGraphViewV1, ParamV1,
    ResolvedTargetV1,
};
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, CheckedHirDocumentOwnedV2, ExtendedSourceRegistryViewV2,
    HirDocumentV1, HirNodeIdV1, HirVisitorV1, ProgramItemHirV1, SourceIdV2,
    SourceRegistryIdentityV2, SpanV1, ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_hir::{
    FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3, FrontendStorageRefusalV3,
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
    AggregationClassV1, BuiltinEvidenceReaderV3, BuiltinRegistryV2,
    DeferredRelationRowFieldInputV2, DeferredRelationRowInputV2, ExpressionEffectsV2,
    ExpressionTypeKernelV2, ExpressionTypeStateV2, RelationPreEvaluationInputV2,
    RelationRowFieldSourceV2, RelationTypeV1, ResolvedCallTargetV2, ResolvedCallableInputV2,
    ResolvedPredicateIdV2, ResolvedRelationCaptureInputV2, ResolvedRelationPredicateInputV2,
    ResolvedTypeInputEntryV2, ResolvedTypeInputValueV2, ResolvedTypeInputsV2, RuleBindingIdV2,
    RuleCallInputV2, RuleCallableTargetV2, RulePredicateInputV2, RuleSymbolInputV2,
    RuleTypeBudgetV2, RuleTypeSolverV2, ScalarTyV1, SourceQualifiedNodeIdV2, TypeDiagnosticV2,
    TypeFactsReaderV3, TypedExpressionFactsV2, ValueTypeV1,
};
use std::collections::{BTreeMap, BTreeSet};

mod bodies;
mod body_materialization;
mod body_types;
mod catalog_inputs;
mod declaration;
mod depth;
mod expression_materialization;
mod inline_view;
mod item_resolution;
mod query_materialization;
mod relation_materialization;
mod request_body;
mod scalar_semantic;
mod semantic_document;
mod session;
mod syntax;
mod type_inputs;

pub use body_materialization::{
    materialize_source_inline_view_with_request_owner_v3,
    topologize_source_inline_view_with_request_owner_v3, AdmittedSourceInlineViewBodyV3,
    AdmittedSourceViewTopologyV3, SourceBodyMaterializationRequestErrorV3,
};
pub use body_types::{
    type_one_let_composition_body_with_request_owner_v3,
    type_source_inline_view_with_request_owner_v3, AdmittedSourceBodyTypesV3,
    SourceBodyTypesRequestErrorV3,
};

impl DeclaredDocumentV2 {
    /// Identify the finite Composition Type continuation without exposing HIR
    /// or adding a pipeline dependency on the syntax crate.
    #[must_use]
    pub fn is_one_let_composition_v3(&self) -> bool {
        matches!(
            self.checked().checked().document().items.as_slice(),
            [ProgramItemHirV1::Composition(composition)] if composition.bindings.len() == 1
        )
    }
}
pub use catalog_inputs::{
    attach_source_catalog_assets_with_request_owner_v3,
    collect_catalog_asset_inputs_with_request_owner_v3, CatalogAssetInputsRequestErrorV3,
    SourceCatalogAssetsV3, SourceCatalogAttachErrorV3,
};
use declaration::SemanticItemKindV2;
pub use declaration::{
    CheckedSyntaxDocumentV2, DeclarationBuildErrorV3, DeclaredDocumentV2, DeclaredRequestErrorV3,
    DeclaredShowSourceV2,
};
use depth::{check_relation_depth_impl, DepthScratchRequestV3};
use expression_materialization::{materialize_bool, materialize_declaration_expr};
use inline_view::{
    collect_inline_view_plan, inline_query_for_decl, InlineViewPlanV2, ViewSourceTargetV2,
};
use item_resolution::{
    resolve_catalog_family_reference, resolve_check_item, resolve_composition_explain,
    resolve_graph_item, resolve_graph_target_use, resolve_presentation_item, ModuleImportAdapterV2,
};
pub use quanta_gqlang_v2_modules::DocumentTopologyRequestErrorV3;
use query_materialization::{materialize_query, materialize_query_composition};
pub use request_body::{
    begin_bind_document_with_request_owner_v3,
    begin_prepare_document_body_types_with_request_owner_v3,
    resolve_source_document_binder_with_request_owner_v3, AdmittedBinderDeclarationsV3,
    AdmittedSourceBinderResolutionV3,
};
pub use scalar_semantic::{
    seal_source_scalar_catalog_with_request_owner_v3, AdmittedScalarSemanticDocumentV3,
    AdmittedScalarSemanticKindV3, ScalarSemanticAssemblyErrorV3, ScalarSourceProjectReadV3,
    SourceCatalogSealErrorV3,
};
pub use session::{
    begin_resolving_document_session_with_request_owner_v3,
    predeclare_source_catalog_with_request_owner_v3, ResolvingDocumentSessionV3,
    SemanticRequestPredeclareErrorV3,
};

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

/// Predeclare source-issued HIR under the same request-root storage owner.
/// The returned registry extension is the authority for later resolution.
pub fn declare_checked_document_with_request_owner_v3(
    checked: CheckedSyntaxDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    permit: &mut FrontendRetainedStoragePermitV3,
    issued_node_slots: usize,
) -> Result<(DeclaredDocumentV2, ExtendedSourceRegistryViewV2), DeclaredRequestErrorV3> {
    permit
        .validate_owner_v3(root, checked.source_id(), source)
        .map_err(DeclaredRequestErrorV3::Storage)?;
    if checked.source_len() != source.len() {
        return Err(DeclaredRequestErrorV3::SourceMismatch);
    }
    let extended = authority
        .try_extend_document(checked.source_id(), checked.source_len())
        .map_err(DeclaredRequestErrorV3::Registry)?;
    let declared = DeclaredDocumentV2::try_new_with_request_owner_v3(
        checked,
        authority.budget(),
        &extended,
        root,
        source,
        permit,
        issued_node_slots,
    )?;
    Ok((declared, extended))
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
    let catalog_items = SourceCatalogAssetsV3::try_from_modules_raw(&declared, predeclared)
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                declared.checked().source_id(),
                SpanV1::point(0),
                format!("modules predeclaration source-index attachment failed: {error:?}"),
            )
        })?;
    let mut inline_views = collect_inline_view_plan(
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
        .validate_view_topology(std::mem::take(&mut inline_views.dependency_edges))
        .map_err(module_failure)?;
    let result = bodies::resolve_document_bodies(
        &declared,
        authority,
        &extended,
        &catalog_items,
        &mut inline_views,
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
    source_targets: &'a [(HirNodeIdV1, ViewSourceTargetV2)],
    outputs: &'a BTreeMap<CatalogAssetIdV2, RelationOutputContractV2>,
}

fn materialize_relation(
    relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &dyn BinderEvidenceReaderV3,
    facts: &dyn TypeFactsReaderV3,
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
                    RelationExprKindV1::Limit(limit) => stack.push(Frame::Visit(&limit.input)),
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
