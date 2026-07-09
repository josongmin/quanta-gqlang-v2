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
    compile_document_structural, compile_document_with_context, compile_document_with_schema,
    compile_item_structural, compile_item_with_context, compile_item_with_schema,
    compile_program_structural, compile_program_with_schema, compile_with_context, CompiledItemV1,
    CompiledProgramV1, FrontendContextV1, NoServeReasonV1,
};
pub use diagnostic::{PipelineDiagnosticV1, PipelineStageV1};

// The `FrontendContextV1` primary bundle authority types (GQW3-18): re-export
// the module-graph and pack-manifest authorities so downstream consumers can
// build a fully-authoritative context without a second owner-crate edge.
pub use quanta_gqlang_v2_binder::ModuleGraphV1;
pub use quanta_gqlang_v2_modules::{
    PackGraphV1, PackIdV1, PackManifestV1, VersionRangeV1, VersionV1,
};
