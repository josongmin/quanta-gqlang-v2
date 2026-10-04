#![forbid(unsafe_code)]

//! Canonical downstream seam for external qglang-v3 consumers.
//!
//! Sibling repos depend on this crate — never on an owner crate. The facade
//! exposes pinned source-document entry points and a curated allowlist of
//! host-agnostic DTO roots, including selected owner-defined values and plan
//! results. Owner modules and proof constructors are not re-exported; no
//! crate-wide glob is used.
//!
//! Closed compilation authority is constructed through the curated
//! [`ClosedFrontendAuthorityBuilderV2`] DTO transaction; owner proof types stay
//! outside this facade.

/// Analyze one source document under the open authority.
pub use quanta_gqlang_v2_pipeline::analyze_source_document_v2;
/// Count the exact lexer-owned portion of a source's frontend storage.
/// This is a partial census, not a frontend execution admission.
pub use quanta_gqlang_v2_pipeline::census_source_lexical_materialization_v3;
/// Parse and classify exactly one source root without minting host authority.
pub use quanta_gqlang_v2_pipeline::classify_single_source_root_v3;
/// Versioned route classification with an explicit lexical storage owner.
pub use quanta_gqlang_v2_pipeline::classify_single_source_root_with_lexical_owner_v1;
/// Compile one closed source document into a source-bound V3 execution document.
pub use quanta_gqlang_v2_pipeline::compile_source_document_v3;
/// Versioned closed compile that retains its lexical-only storage owner.
pub use quanta_gqlang_v2_pipeline::compile_source_document_with_lexical_owner_v1;
/// Format one source document under the canonical budget.
pub use quanta_gqlang_v2_pipeline::format_source_document_v2;
pub use quanta_gqlang_v2_pipeline::LexicalMaterializationCensusV1;
/// SHOW-only parsed/integrity owner. Semantic and plan materialization remain open.
pub use quanta_gqlang_v2_pipeline::{
    admit_show_syntax_v2, AdmittedShowDocumentV2, ShowSyntaxAdmissionErrorV2,
};
pub use quanta_gqlang_v2_pipeline::{
    AdmittedLexicalCompiledSourceDocumentV1, FrontendLexicalHostCeilingV1,
    FrontendLexicalMaterializationRootV1,
};

// ---- assembled document DTOs ----------------------------------------------
pub use quanta_gqlang_v2_pipeline::{
    execution_operator_descriptor_v3, AnalysisDocumentV2, AnalysisItemV2, AnalysisStatusV2,
    CompiledItemV2, CompiledRuleProgramV2, CompiledSourceDocumentV3, ExecutionAuthorityV3,
    ExecutionBindingV3, ExecutionCallTargetV3, ExecutionCardinalityV3, ExecutionCatalogIdentityV3,
    ExecutionCheckV3, ExecutionCommandPhaseV3, ExecutionCompareMetricV3,
    ExecutionCompositionBindingV3, ExecutionCompositionBodyV3, ExecutionCompositionProducerV3,
    ExecutionCompositionV3, ExecutionConfirmModeV3, ExecutionCorrelationV3,
    ExecutionDetectionModeV3, ExecutionDeterminismV3, ExecutionDigestV3, ExecutionDocumentV3,
    ExecutionEffectV3, ExecutionEmitModeV3, ExecutionEvidenceModeV3, ExecutionEvidenceV3,
    ExecutionExistentialTermV3, ExecutionExistsPolarityV3, ExecutionExplainTargetV3,
    ExecutionExportFormatV3, ExecutionExpressionShapeV3, ExecutionExpressionV3, ExecutionFamilyV3,
    ExecutionFieldV3, ExecutionFiniteFloatV3, ExecutionGraphCapabilityV3,
    ExecutionGraphDedupIdentityV3, ExecutionGraphDirectionV3, ExecutionGraphKindV3,
    ExecutionGraphQueryV3, ExecutionGraphReferenceKindV3, ExecutionGraphReferenceRoleV3,
    ExecutionGraphReferenceV3, ExecutionGraphRelationShapeV3, ExecutionGraphTargetV3,
    ExecutionGraphWitnessTieBreakV3, ExecutionItemV3, ExecutionJoinModeV3, ExecutionLiteralV3,
    ExecutionLogicalQuantifierV3, ExecutionNestedGraphV3, ExecutionNullOrderV3,
    ExecutionOperatorDescriptorErrorV3, ExecutionOperatorDescriptorV3, ExecutionOperatorKindV3,
    ExecutionOperatorV3, ExecutionOrderKeyV3, ExecutionPlanV3, ExecutionPolicyFamilyV3,
    ExecutionPresentationKindV3, ExecutionRecordFieldV3, ExecutionRelationAuthorityV3,
    ExecutionRelationV3, ExecutionRelationalOperatorV3, ExecutionRenameCommandV3,
    ExecutionSampleModeV3, ExecutionScalarTypeV3, ExecutionSchemaV3, ExecutionShowCategoryV3,
    ExecutionSortDirectionV3, ExecutionSourceKindV3, ExecutionSourceV3, ExecutionSymbolTargetV3,
    ExecutionTruthModeV3, ExecutionUnionVariantV3, ExecutionValueTypeV3, PlannedItemV2,
    RuleProgramMemberV2, UnresolvedAnalysisFactV2, ANALYSIS_DIAGNOSTIC_LIMIT_CODE,
    OPEN_AUTHORITY_INCOMPLETE_CODE,
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
    FrontendSymbolTargetInputV3, FrontendTransactionErrorV2, GraphReferenceKindV2,
    OpenAnalysisAuthorityV2,
};
pub use quanta_gqlang_v2_pipeline::{
    FrontendSourceRootClassificationErrorV3, FrontendSourceRootFamilyV3,
};

// ---- inert value/location closure types ------------------------------------
pub use quanta_gqlang_v2_pipeline::{
    AlgebraFamilyV2, CheckedLocatedSpanV2, LocatedSpanV2, SemanticItemIdV2, SourceIdV2,
    SourceQualifiedNodeIdV2, SpanV1,
};
