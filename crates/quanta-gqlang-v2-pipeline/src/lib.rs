//! QGLang v3 single shared frontend compiler service (ticket 08).
//!
//! The one pipeline (D22) that CLI, LSP, formatter, and tests all consume:
//! surface parse -> bind -> type -> normalize to algebra/logic -> plan. No
//! shadow parser/sema may exist outside this seam.
//!
//! ## Entry point
//!
//! [`compile_with_context`] is the semantic entry point over
//! [`FrontendContextV1`]. Explicit non-semantic helpers also exist:
//! [`compile_program_structural`] / [`compile_item_structural`] /
//! [`compile_document_structural`] for structural-only authority and
//! [`compile_program_with_schema`] / [`compile_item_with_schema`] /
//! [`compile_document_with_schema`] for schema-only authority.
//!
//! Stages run in order and SHORT-CIRCUIT on the first failure, returning a
//! stage-tagged [`PipelineDiagnosticV1`] that preserves the inner stage
//! diagnostic's stable code and span (D10 fail-closed).
//!
//! ## Cited packet decisions
//!
//! - `D22`: CLI / LSP / formatter / proof tooling MUST share this one compiler
//!   pipeline; shadow parser/sema implementations are forbidden.
//! - `D10`: every stage failure is an explicit typed reject; no silent downgrade.
//! - `D32`: the plan carries the fixed-field semantic-cost schema from ticket 07
//!   unchanged ([`quanta_gqlang_v2_planning::SemanticCostV1`]).

mod compile;
mod diagnostic;

pub use compile::{
    analyze_resolved_document_v2, analyze_source_document_v2, compile_resolved_document_v2,
    compile_source_document_v2, format_source_document_v2, AnalysisDocumentV2, AnalysisItemV2,
    CompiledDocumentV2, CompiledItemV2, CompiledRuleProgramV2, PlannedItemV2, RuleProgramMemberV2,
    UnresolvedAnalysisFactV2, OPEN_AUTHORITY_INCOMPLETE_CODE, PARSE_BUDGET_PROJECTION_FAILED_CODE,
};
pub use diagnostic::{
    algebra_code, diagnostic_sort_key, logic_code, no_plan_code, planning_code,
    DocumentAssemblyCodeV2, PipelineStageV2,
};
pub use quanta_gqlang_v2_algebra::AlgebraFamilyV2;
pub use quanta_gqlang_v2_planning::{NoPlanReasonV2, SemanticPlanV2};
pub use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, ClosedFrontendAuthorityV2, FrontendAuthorityErrorV2, FrontendBudgetV2,
    FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2,
    FrontendTransactionErrorV2, LocatedSpanV2, OpenAnalysisAuthorityV2, SourceIdV2, SpanV1,
};
