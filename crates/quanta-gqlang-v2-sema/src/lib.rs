#![forbid(unsafe_code)]

//! Document-scoped semantic transaction owner.
//!
//! This crate coordinates the canonical HIR integrity, Binder, types, modules,
//! and relation-authority owners. It owns semantic carriers and atomic document
//! ordering; it does not copy their algorithms.

mod authority;
mod diagnostic;
mod document;
mod expr;
mod item;
mod names;
mod relation;
mod rule;

#[cfg(any(test, feature = "test"))]
#[doc(hidden)]
pub mod test_support;

// Host-agnostic checked coordinates remain the canonical HIR primitive. Semantic
// direction and expression tags below prevent downstream direct HIR dependency.
pub use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, CompareMetricV1, ConfirmModeV1, DetectionModeV1, EmitModeV1,
    EvidenceModeV1, ExportFormatV1, GraphQueryKindV1, HirNodeIdV1, LocatedSpanV2, SampleModeV1,
    ShowCategoryV1, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1, TruthModeV1,
};
pub use quanta_gqlang_v2_types::{
    CardinalityV1, ExpressionTypeStateV2, OperatorIdV2, ProvenanceNamespaceV2, RecordFieldTypeV1,
    RecordTypeV1, RelationAuthorityV2, RelationCatalogV2, RelationSourceInputV2,
    ResolvedCallTargetV2, ScalarTyV1, SourceQualifiedNodeIdV2, TypedLiteralFactV2,
    TypedLiteralValueV2, ValueTypeV1,
};

pub use authority::{
    ClosedFrontendAuthorityBuilderV2, ClosedFrontendAuthorityV2, DownstreamBudgetReceiptV2,
    FrontendAuthorityErrorV2, FrontendAuthorityLocationV2, FrontendAuthoritySourceV2,
    FrontendBudgetV2, FrontendCallableParameterInputV2, FrontendCatalogAssetInputV2,
    FrontendCatalogPayloadV2, FrontendGraphReferenceInputV2, FrontendLanguageVersionV2,
    FrontendLockfileInputV2, FrontendPackInputV2, FrontendRelationAuthorityV2,
    FrontendRelationCardinalityV2, FrontendRelationFieldInputV2, FrontendRelationInputV2,
    FrontendScalarTypeV2, GraphReferenceCatalogV2, GraphReferenceIdV2, GraphReferenceInputV2,
    GraphReferenceKindV2, OpenAnalysisAuthorityInputV2, OpenAnalysisAuthorityV2,
};
pub use diagnostic::{
    FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2,
};
pub use document::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, DeclaredDocumentV2,
    FrontendTransactionErrorV2, IncompleteSemanticDocumentV2, SemanticAnalysisV2,
    SemanticDocumentV2, UnresolvedSemanticFactV2,
};
pub use expr::{
    AggregateContextV2, AggregatePositionV2, LogicalQuantifierKindV2, RelationExistsPolarityV2,
    TypedBoolExprV2, TypedCallArgumentV2, TypedCorrelationEdgeV2, TypedExprShapeV2, TypedExprV2,
    TypedPathSegmentV2, TypedRecordFieldV2,
};
pub use item::{
    CompositionGraphCapabilityV2, CompositionInstanceV2, GraphDirectionV2, GraphReferenceRoleV2,
    GraphWitnessBoundsV2, ResolvedCatalogItemV2, ResolvedCatalogPayloadV2, ResolvedCheckV2,
    ResolvedCompositionBindingV2, ResolvedCompositionBodyV2, ResolvedCompositionGraphTargetV2,
    ResolvedCompositionProducerV2, ResolvedCompositionV2, ResolvedDeclarationV2,
    ResolvedExplainTargetV2, ResolvedGraphQueryV2, ResolvedGraphReferenceV2, ResolvedGraphTargetV2,
    ResolvedGraphUseV2, ResolvedMetaKindV2, ResolvedMetaV2, ResolvedPresentationKindV2,
    ResolvedPresentationV2, SemanticItemV2,
};
pub use names::{
    ResolvedBindingKindV2, ResolvedCatalogReferenceV2, ResolvedCompositionSourceIdV2,
    ResolvedRelationSourceKindV2, ResolvedRelationSourceV2, ResolvedValueBindingV2,
    SemanticItemIdV2,
};
pub use relation::{
    JoinModeV2, NullOrderV2, OrderKeyContractV2, OutputColumnContractV2, OutputColumnOriginV2,
    OutputProvenanceV2, PartitionInputBindingV2, RelationOutputContractV2, ResolvedScanSourceV2,
    SortDirectionV2, TypedRelationExprV2, TypedRelationStageInputV2, TypedRelationStageKindV2,
    TypedRelationV2,
};
pub use rule::{ResolvedRuleCallV2, ResolvedRuleV2, RuleCallPolarityV2};
