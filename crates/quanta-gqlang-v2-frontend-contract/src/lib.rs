#![forbid(unsafe_code)]

//! Canonical downstream seam for external qglang-v3 consumers.
//!
//! Sibling repos depend on this crate — never on an owner crate. The facade
//! exposes exactly the three pinned source-document entry points plus a curated
//! allowlist of host-agnostic DTO roots; owner internals (HIR, Binder, Types,
//! Modules, Algebra, Logic, Planning, Pipeline) stay unreachable, and no
//! crate-wide glob is used.
//!
//! Closed compilation authority is constructed through the curated
//! [`ClosedFrontendAuthorityBuilderV2`] DTO transaction; owner proof types stay
//! outside this facade.

/// Analyze one source document under the open authority.
pub use quanta_gqlang_v2_pipeline::analyze_source_document_v2;
/// Compile one closed source document into a fully assembled document.
pub use quanta_gqlang_v2_pipeline::compile_source_document_v2;
/// Format one source document under the canonical budget.
pub use quanta_gqlang_v2_pipeline::format_source_document_v2;

// ---- assembled document DTOs ----------------------------------------------
pub use quanta_gqlang_v2_pipeline::{
    AnalysisDocumentV2, AnalysisItemV2, AnalysisStatusV2, CompiledDocumentV2, CompiledItemV2,
    CompiledRuleProgramV2, ExecutionAuthorityV2, ExecutionBindingV2, ExecutionCompositionBindingV2,
    ExecutionCompositionBodyV2, ExecutionCompositionProducerV2, ExecutionCompositionV2,
    ExecutionCorrelationV2, ExecutionDeterminismV2, ExecutionDigestV2, ExecutionDocumentV2,
    ExecutionEffectV2, ExecutionEvidenceV2, ExecutionExpressionShapeV2, ExecutionExpressionV2,
    ExecutionFamilyV2, ExecutionFieldV2, ExecutionGraphKindV2, ExecutionItemV2,
    ExecutionOperatorV2, ExecutionOrderKeyV2, ExecutionPlanV2, ExecutionPolicyFamilyV2,
    ExecutionRelationAuthorityV2, ExecutionRelationV2, ExecutionRelationalOperatorV2,
    ExecutionSampleModeV2, ExecutionSchemaV2, ExecutionSourceV2, PlannedItemV2,
    RuleProgramMemberV2, UnresolvedAnalysisFactV2, OPEN_AUTHORITY_INCOMPLETE_CODE,
};

// ---- planning DTOs reachable from the assembled documents ------------------
pub use quanta_gqlang_v2_pipeline::{NoPlanReasonV2, SemanticPlanV2};

// ---- authorities, diagnostics, and transaction vocabulary ------------------
pub use quanta_gqlang_v2_pipeline::{
    ClosedFrontendAuthorityBuilderV2, ClosedFrontendAuthorityV2, FrontendAuthorityErrorV2,
    FrontendAuthorityLocationV2, FrontendAuthoritySourceV2, FrontendBudgetV2,
    FrontendCallableParameterInputV2, FrontendCatalogAssetInputV2, FrontendCatalogPayloadV2,
    FrontendDiagnosticV2, FrontendFailureV2, FrontendGraphReferenceInputV2,
    FrontendLanguageVersionV2, FrontendLockfileInputV2, FrontendPackInputV2,
    FrontendRelationAuthorityV2, FrontendRelationCardinalityV2, FrontendRelationFieldInputV2,
    FrontendRelationInputV2, FrontendScalarTypeV2, FrontendSeverityV2, FrontendStageV2,
    FrontendTransactionErrorV2, GraphReferenceKindV2, OpenAnalysisAuthorityV2,
};

// ---- inert value/location closure types ------------------------------------
pub use quanta_gqlang_v2_pipeline::{
    AlgebraFamilyV2, CheckedLocatedSpanV2, LocatedSpanV2, SemanticItemIdV2, SourceIdV2,
    SourceQualifiedNodeIdV2, SpanV1,
};
