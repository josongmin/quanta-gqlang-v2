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
/// Parse and classify exactly one source root without minting host authority.
pub use quanta_gqlang_v2_pipeline::classify_single_source_root_v3;
/// Compile one closed source document into a source-bound V3 execution document.
pub use quanta_gqlang_v2_pipeline::compile_source_document_v3;
/// Format one source document under the canonical budget.
pub use quanta_gqlang_v2_pipeline::format_source_document_v2;

// ---- assembled document DTOs ----------------------------------------------
pub use quanta_gqlang_v2_pipeline::{
    AnalysisDocumentV2, AnalysisItemV2, AnalysisStatusV2, CompiledItemV2, CompiledRuleProgramV2,
    CompiledSourceDocumentV3, ExecutionAuthorityV3, ExecutionBindingV3, ExecutionCallTargetV3,
    ExecutionCardinalityV3, ExecutionCompositionBindingV3, ExecutionCompositionBodyV3,
    ExecutionCompositionProducerV3, ExecutionCompositionV3, ExecutionCorrelationV3,
    ExecutionDeterminismV3, ExecutionDigestV3, ExecutionDocumentV3, ExecutionEffectV3,
    ExecutionEvidenceV3, ExecutionExistsPolarityV3, ExecutionExpressionShapeV3,
    ExecutionExpressionV3, ExecutionFamilyV3, ExecutionFieldV3, ExecutionFiniteFloatV3,
    ExecutionGraphKindV3, ExecutionItemV3, ExecutionJoinModeV3, ExecutionLiteralV3,
    ExecutionLogicalQuantifierV3, ExecutionNullOrderV3, ExecutionOperatorV3, ExecutionOrderKeyV3,
    ExecutionPlanV3, ExecutionPolicyFamilyV3, ExecutionRecordFieldV3, ExecutionRelationAuthorityV3,
    ExecutionRelationV3, ExecutionRelationalOperatorV3, ExecutionSampleModeV3,
    ExecutionScalarTypeV3, ExecutionSchemaV3, ExecutionSortDirectionV3, ExecutionSourceKindV3,
    ExecutionSourceV3, ExecutionUnionVariantV3, ExecutionValueTypeV3, PlannedItemV2,
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
pub use quanta_gqlang_v2_pipeline::{
    FrontendSourceRootClassificationErrorV3, FrontendSourceRootFamilyV3,
};

// ---- inert value/location closure types ------------------------------------
pub use quanta_gqlang_v2_pipeline::{
    AlgebraFamilyV2, CheckedLocatedSpanV2, LocatedSpanV2, SemanticItemIdV2, SourceIdV2,
    SourceQualifiedNodeIdV2, SpanV1,
};
