//! QGLang v2 single shared frontend compiler service.
//!
//! The one pipeline that front doors, formatter, and tests consume:
//! surface parse -> bind -> type -> normalize to algebra/logic -> plan.
//!
//! ## Entry point
//!
//! [`compile_source_document_v2`] is the closed semantic entry point.
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
    analyze_resolved_document_v2, analyze_source_document_v2, compile_resolved_document_v2,
    compile_source_document_v2, format_source_document_v2, AnalysisDocumentV2, AnalysisItemV2,
    AnalysisStatusV2, CompiledDocumentV2, CompiledItemV2, CompiledRuleProgramV2, PlannedItemV2,
    RuleProgramMemberV2, UnresolvedAnalysisFactV2, OPEN_AUTHORITY_INCOMPLETE_CODE,
    PARSE_BUDGET_PROJECTION_FAILED_CODE,
};
pub use diagnostic::{
    algebra_code, diagnostic_sort_key, logic_code, no_plan_code, planning_code,
    DocumentAssemblyCodeV2, PipelineStageV2,
};
pub use execution::{
    ExecutionAuthorityV2, ExecutionBindingV2, ExecutionCompositionBindingV2,
    ExecutionCompositionV2, ExecutionCorrelationV2, ExecutionDeterminismV2, ExecutionDigestV2,
    ExecutionDocumentV2, ExecutionEffectV2, ExecutionEvidenceV2, ExecutionExpressionShapeV2,
    ExecutionExpressionV2, ExecutionFamilyV2, ExecutionFieldV2, ExecutionItemV2,
    ExecutionOperatorV2, ExecutionOrderKeyV2, ExecutionPlanV2, ExecutionPolicyFamilyV2,
    ExecutionRelationV2, ExecutionRelationalOperatorV2, ExecutionSchemaV2, ExecutionSourceV2,
};
pub use quanta_gqlang_v2_algebra::AlgebraFamilyV2;
pub use quanta_gqlang_v2_planning::{NoPlanReasonV2, SemanticPlanV2};
pub use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, ClosedFrontendAuthorityBuilderV2, ClosedFrontendAuthorityV2,
    FrontendAuthorityErrorV2, FrontendAuthorityLocationV2, FrontendAuthoritySourceV2,
    FrontendBudgetV2, FrontendCallableParameterInputV2, FrontendCatalogAssetInputV2,
    FrontendCatalogPayloadV2, FrontendDiagnosticV2, FrontendFailureV2,
    FrontendGraphReferenceInputV2, FrontendLanguageVersionV2, FrontendLockfileInputV2,
    FrontendPackInputV2, FrontendRelationCardinalityV2, FrontendRelationFieldInputV2,
    FrontendRelationInputV2, FrontendScalarTypeV2, FrontendSeverityV2, FrontendStageV2,
    FrontendTransactionErrorV2, GraphReferenceKindV2, LocatedSpanV2, OpenAnalysisAuthorityV2,
    SemanticItemIdV2, SourceIdV2, SourceQualifiedNodeIdV2, SpanV1,
};
