//! QGLang v3 single shared frontend compiler service (ticket 08).
//!
//! The one pipeline (D22) that CLI, LSP, formatter, and tests all consume:
//! surface parse -> bind -> type -> normalize to algebra/logic -> plan. No
//! shadow parser/sema may exist outside this seam.
//!
//! ## Entry point
//!
//! [`compile_program`] runs the FULL stack against a source string and returns a
//! [`CompiledProgramV1`] (the shared HIR item, the normalized algebra root, and
//! the host-agnostic public plan or explicit non-servable artifact). The
//! context-first semantic surface is [`compile_with_context`] over
//! [`FrontendContextV1`]; [`compile_program`] / [`compile_with`] remain
//! structural/compat shims.
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
    compile_document, compile_document_with_context, compile_item, compile_item_with_context,
    compile_program, compile_with, compile_with_context, CompiledItemV1, CompiledProgramV1,
    FrontendContextV1, NoServeReasonV1,
};
pub use diagnostic::{PipelineDiagnosticV1, PipelineStageV1};
