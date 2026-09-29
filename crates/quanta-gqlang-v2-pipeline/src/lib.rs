//! QGLang v2 single shared frontend compiler service.
//!
//! The one pipeline that front doors, formatter, and tests consume:
//! surface parse -> bind -> type -> normalize to algebra/logic -> plan.
//!
//! ## Entry point
//!
//! [`compile_source_document_v3`] is the source-bound closed execution entry point.
//! [`analyze_source_document_v2`] is the explicitly incomplete editor path;
//! it never promotes unresolved analysis into closed compilation.
//!
//! Stages run in order and SHORT-CIRCUIT on the first failure, returning a
//! stage-tagged [`FrontendTransactionErrorV2`] that preserves the owner
//! diagnostic's stable code and checked location.
//!
//! ## Cited packet decisions
//!
//! - `D22`: every front door shares this compiler pipeline; shadow semantic
//!   implementations are forbidden.
//! - `D10`: every stage failure is an explicit typed reject; no silent downgrade.
//! - `D32`: the plan carries the fixed-field semantic-cost schema unchanged
//!   ([`quanta_gqlang_v2_planning::SemanticCostV2`]).

mod compile;
mod diagnostic;
mod execution;

pub use compile::{
    analyze_resolved_document_v2, analyze_source_document_v2, classify_single_source_root_v3,
    compile_resolved_document_v2, compile_source_document_v3, format_source_document_v2,
    AnalysisDocumentV2, AnalysisItemV2, AnalysisStatusV2, CompiledDocumentV2, CompiledItemV2,
    CompiledRuleProgramV2, CompiledSourceDocumentV3, FrontendSourceRootClassificationErrorV3,
    FrontendSourceRootFamilyV3, PlannedItemV2, RuleProgramMemberV2, UnresolvedAnalysisFactV2,
    ANALYSIS_DIAGNOSTIC_LIMIT_CODE, OPEN_AUTHORITY_INCOMPLETE_CODE,
    PARSE_BUDGET_PROJECTION_FAILED_CODE,
};
pub use diagnostic::{
    algebra_code, diagnostic_sort_key, logic_code, no_plan_code, planning_code,
    DocumentAssemblyCodeV2, PipelineStageV2,
};
pub use execution::{
    execution_operator_descriptor_v3, ExecutionAuthorityV3, ExecutionBindingV3,
    ExecutionCallTargetV3, ExecutionCardinalityV3, ExecutionCatalogIdentityV3, ExecutionCheckV3,
    ExecutionCommandPhaseV3, ExecutionCompareMetricV3, ExecutionCompositionBindingV3,
    ExecutionCompositionBodyV3, ExecutionCompositionProducerV3, ExecutionCompositionV3,
    ExecutionConfirmModeV3, ExecutionCorrelationV3, ExecutionDetectionModeV3,
    ExecutionDeterminismV3, ExecutionDigestV3, ExecutionDocumentV3, ExecutionEffectV3,
    ExecutionEmitModeV3, ExecutionEvidenceModeV3, ExecutionEvidenceV3, ExecutionExistentialTermV3,
    ExecutionExistsPolarityV3, ExecutionExplainTargetV3, ExecutionExportFormatV3,
    ExecutionExpressionShapeV3, ExecutionExpressionV3, ExecutionFamilyV3, ExecutionFieldV3,
    ExecutionFiniteFloatV3, ExecutionGraphCapabilityV3, ExecutionGraphDedupIdentityV3,
    ExecutionGraphDirectionV3, ExecutionGraphKindV3, ExecutionGraphQueryV3,
    ExecutionGraphReferenceKindV3, ExecutionGraphReferenceRoleV3, ExecutionGraphReferenceV3,
    ExecutionGraphRelationShapeV3, ExecutionGraphTargetV3, ExecutionGraphWitnessTieBreakV3,
    ExecutionItemV3, ExecutionJoinModeV3, ExecutionLiteralV3, ExecutionLogicalQuantifierV3,
    ExecutionNestedGraphV3, ExecutionNullOrderV3, ExecutionOperatorDescriptorErrorV3,
    ExecutionOperatorDescriptorV3, ExecutionOperatorKindV3, ExecutionOperatorV3,
    ExecutionOrderKeyV3, ExecutionPlanV3, ExecutionPolicyFamilyV3, ExecutionPresentationKindV3,
    ExecutionRecordFieldV3, ExecutionRelationAuthorityV3, ExecutionRelationV3,
    ExecutionRelationalOperatorV3, ExecutionRenameCommandV3, ExecutionSampleModeV3,
    ExecutionScalarTypeV3, ExecutionSchemaV3, ExecutionShowCategoryV3, ExecutionSortDirectionV3,
    ExecutionSourceKindV3, ExecutionSourceV3, ExecutionSymbolTargetV3, ExecutionTruthModeV3,
    ExecutionUnionVariantV3, ExecutionValueTypeV3,
};
pub use quanta_gqlang_v2_algebra::AlgebraFamilyV2;
pub use quanta_gqlang_v2_planning::{NoPlanReasonV2, SemanticPlanV2};
pub use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, ClosedFrontendAuthorityBuilderV2, ClosedFrontendAuthorityV2,
    FrontendAuthorityErrorV2, FrontendAuthorityLocationV2, FrontendAuthoritySourceV2,
    FrontendBudgetV2, FrontendCallableParameterInputV2, FrontendCatalogAssetInputV2,
    FrontendCatalogPayloadV2, FrontendDiagnosticV2, FrontendFailureV2,
    FrontendGraphReferenceInputV2, FrontendLanguageVersionV2, FrontendLockfileInputV2,
    FrontendPackInputV2, FrontendRelationAuthorityV2, FrontendRelationCardinalityV2,
    FrontendRelationFieldInputV2, FrontendRelationInputV2, FrontendScalarTypeV2,
    FrontendSeverityV2, FrontendStageV2, FrontendSymbolTargetInputV3, FrontendTransactionErrorV2,
    GraphReferenceKindV2, LocatedSpanV2, OpenAnalysisAuthorityV2, SemanticItemIdV2, SourceIdV2,
    SourceQualifiedNodeIdV2, SpanV1,
};
