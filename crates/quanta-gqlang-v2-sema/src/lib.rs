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

// Host-agnostic checked coordinates and direction remain the canonical HIR
// primitives; downstream consumers need no direct HIR dependency to name them.
pub use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, GraphQueryKindV1, LocatedSpanV2, SortDirectionHirV1, SourceIdV2};

pub use authority::{
    ClosedFrontendAuthorityV2, FrontendAuthorityErrorV2, FrontendBudgetV2,
    GraphReferenceCatalogV2, GraphReferenceIdV2, GraphReferenceInputV2,
    GraphReferenceKindV2, OpenAnalysisAuthorityInputV2, OpenAnalysisAuthorityV2,
};
pub use diagnostic::{
    FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2,
};
pub use document::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, DeclaredDocumentV2,
    FrontendTransactionErrorV2, IncompleteSemanticDocumentV2, ResolvedDocumentV2,
    SemanticAnalysisV2, SemanticDocumentV2,
    UnresolvedSemanticFactV2,
};
pub use expr::{
    AggregateContextV2, AggregatePositionV2, TypedBoolExprV2, TypedCallArgumentV2, TypedExprShapeV2,
    TypedCorrelationEdgeV2, TypedExprV2, TypedPathSegmentV2, TypedRecordFieldV2,
};
pub use item::{
    GraphDirectionV2, GraphReferenceRoleV2, GraphWitnessBoundsV2, ResolvedCatalogItemV2, ResolvedCatalogPayloadV2, ResolvedCheckV2,
    ResolvedCompositionBindingV2, ResolvedCompositionV2, ResolvedDeclarationV2,
    ResolvedExplainTargetV2,
    ResolvedGraphQueryV2, ResolvedGraphReferenceV2, ResolvedGraphUseV2, ResolvedMetaKindV2, ResolvedMetaV2,
    ResolvedPresentationKindV2, ResolvedPresentationV2, SemanticItemV2,
};
pub use names::{
    ResolvedBindingKindV2, ResolvedCatalogReferenceV2, ResolvedRelationSourceKindV2,
    ResolvedRelationSourceV2, ResolvedValueBindingV2, SemanticItemIdV2,
};
pub use relation::{
    JoinModeV2, NullOrderV2, OrderKeyContractV2, OutputColumnContractV2, OutputColumnOriginV2, OutputProvenanceV2, RelationOutputContractV2,
    ResolvedScanSourceV2, TypedRelationExprV2, TypedRelationStageInputV2,
    TypedRelationStageKindV2, TypedRelationV2,
};
pub use rule::{ResolvedRuleCallV2, ResolvedRuleV2, RuleCallPolarityV2};
