#![forbid(unsafe_code)]

//! Canonical downstream seam for external qglang-v3 consumers.
//!
//! Sibling repos depend on this crate — never on an owner crate. The facade
//! exposes exactly the three pinned source-document entry points plus a curated
//! allowlist of host-agnostic DTO roots; owner internals (HIR, Binder, Types,
//! Modules, Algebra, Logic, Planning, Pipeline) stay unreachable, and no
//! crate-wide glob is used.
//!
//! The public closed-authority builder path is intentionally absent until
//! Ticket 07's builder DTOs land; hosts construct authorities only through the
//! allowlisted authority types below.

/// Analyze one source document under the open authority.
pub use quanta_gqlang_v2_pipeline::analyze_source_document_v2;
/// Compile one closed source document into a fully assembled document.
pub use quanta_gqlang_v2_pipeline::compile_source_document_v2;
/// Format one source document under the canonical budget.
pub use quanta_gqlang_v2_pipeline::format_source_document_v2;

// ---- assembled document DTOs ----------------------------------------------
pub use quanta_gqlang_v2_pipeline::{
    AnalysisDocumentV2, AnalysisItemV2, CompiledDocumentV2, CompiledItemV2, CompiledRuleProgramV2,
    PlannedItemV2, RuleProgramMemberV2, UnresolvedAnalysisFactV2, OPEN_AUTHORITY_INCOMPLETE_CODE,
};

// ---- planning DTOs reachable from the assembled documents ------------------
pub use quanta_gqlang_v2_pipeline::{NoPlanReasonV2, SemanticPlanV2};

// ---- authorities, diagnostics, and transaction vocabulary ------------------
pub use quanta_gqlang_v2_pipeline::{
    ClosedFrontendAuthorityV2, FrontendAuthorityErrorV2, FrontendBudgetV2, FrontendDiagnosticV2,
    FrontendFailureV2, FrontendSeverityV2, FrontendStageV2, FrontendTransactionErrorV2,
    OpenAnalysisAuthorityV2,
};

// ---- inert value/location closure types ------------------------------------
pub use quanta_gqlang_v2_pipeline::{
    AlgebraFamilyV2, CheckedLocatedSpanV2, LocatedSpanV2, SemanticItemIdV2, SourceIdV2, SpanV1,
};
