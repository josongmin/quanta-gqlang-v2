//! v3 read-only diagnostics over the ONE shared compiler pipeline (ticket 08).
//!
//! D22: CLI, LSP, formatter, and tests must consume a SINGLE compiler pipeline;
//! shadow parser/sema implementations are forbidden. This handler is a thin,
//! read-only adapter over [`quanta_gqlang_v2_pipeline::compile_program`]. It does
//! NOT re-parse, re-bind, or re-type anything: it asks the shared pipeline to
//! compile the source and, on the first failing stage, surfaces that stage's
//! already-typed diagnostic verbatim — stage tag, stable code, exact byte span.
//!
//! Because the pipeline short-circuits on the FIRST failing stage (D10
//! fail-closed; no silent downgrade), at most one diagnostic is produced per
//! source. The surface is still modeled as an ordered list so it stays a proper
//! diagnostics channel and so determinism is asserted over the list, not a
//! special-cased single value. A clean compile yields an empty list.
//!
//! Parity (ticket 08 required rail 7): the `code` and `span` carried here are the
//! exact strings/offsets the pipeline diagnostic exposes, so the SAME failing
//! advanced query produces the SAME diagnostic code the CLI surfaces from the
//! other side of the same pipeline — there is no LSP-local rewrite to drift.

use quanta_gqlang_v2_pipeline::{compile_program, PipelineDiagnosticV1};

/// A single stage-tagged pipeline diagnostic, flattened for the read-only LSP
/// wire surface. Every field is copied verbatim from the typed pipeline
/// diagnostic; nothing is synthesized here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct V3DiagnosticItemV1 {
    /// The pipeline stage that short-circuited: one of
    /// `parse` | `bind` | `type` | `algebra` | `logic` | `planning`.
    pub stage: String,
    /// The inner stage's stable, machine-readable code, verbatim.
    pub code: String,
    /// The human-readable rendered diagnostic (stage-prefixed `Display`).
    pub message: String,
    /// Inclusive start byte offset of the primary source span.
    pub span_start: usize,
    /// Exclusive end byte offset of the primary source span.
    pub span_end: usize,
}

impl V3DiagnosticItemV1 {
    fn from_pipeline_v1(diagnostic: &PipelineDiagnosticV1) -> Self {
        let span = diagnostic.span();
        Self {
            stage: diagnostic.stage().as_str().to_owned(),
            code: diagnostic.code(),
            message: diagnostic.to_string(),
            span_start: span.start,
            span_end: span.end,
        }
    }
}

/// Run the shared pipeline over `source` and return its diagnostics.
///
/// Deterministic by construction: the pipeline is a pure function of the source,
/// short-circuits on the first failing stage, and we copy that one typed
/// diagnostic without reordering. A clean compile returns an empty list. This is
/// `Result::Ok` for any well-formed request: a compile FAILURE is data on the
/// read-only surface (a diagnostic), not a transport error.
#[must_use]
pub fn handle_v3_diagnostics_v1(source: &str) -> Vec<V3DiagnosticItemV1> {
    match compile_program(source) {
        Ok(_) => Vec::new(),
        Err(diagnostic) => vec![V3DiagnosticItemV1::from_pipeline_v1(&diagnostic)],
    }
}
