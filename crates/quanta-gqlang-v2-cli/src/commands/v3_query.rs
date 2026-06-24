//! `query` / `v3-query` command: plan + semantic-cost summary over the shared
//! v3 pipeline.
//!
//! Consumes the ONE compiler service (`quanta_gqlang_v2_pipeline`, D22) and
//! renders the host-agnostic plan plus the fixed semantic-cost schema
//! deterministically (D32). It never re-parses or re-analyzes (D22).
//!
//! Input may be a SINGLE top-level item or a `;`-separated multi-item document:
//! the input is parsed through `surface::parse_document` (the one shared parser)
//! and compiled through `pipeline::compile_document_structural` so cross-item
//! catalog rules (duplicate registration, VIEW cycles, REQUIRE CERTIFIED gates)
//! fire without smuggling an empty schema as semantic authority. Each item's
//! per-family outcome is rendered in source order.
//!
//! Fail-closed (D10): a failing stage returns the stage-tagged pipeline
//! diagnostic carrying the inner stage's exact stable code and span, formatted by
//! [`render_pipeline_diagnostic_v1`].

use quanta_gqlang_v2_hir::{HirBuilderV1, ProgramItemHirV1};
use quanta_gqlang_v2_pipeline::{compile_document_structural, PipelineDiagnosticV1};
use quanta_gqlang_v2_surface::parse_document;

use crate::v3_plan::render_compiled_item_v1;

/// Run `query`: parse the input as a v3 document and render each item's full
/// per-family compiled outcome, in source order.
///
/// A single-item input renders exactly one item block; a multi-item document
/// renders each block separated by a blank line, in written order. The renderer
/// dispatches on the compiled outcome (not just its optional plan), so EVERY item
/// kind surfaces deterministically. It never re-parses or re-derives the family
/// here (D22).
///
/// # Errors
///
/// Returns the rendered parse diagnostic if the document does not parse, or the
/// rendered stage-tagged pipeline diagnostic on the first failing item/stage.
pub fn run(input: &str) -> Result<String, String> {
    let items = parse_items_v1(input).map_err(|diag| render_pipeline_diagnostic_v1(&diag))?;
    let compiled =
        compile_document_structural(&items).map_err(|diag| render_pipeline_diagnostic_v1(&diag))?;
    Ok(compiled
        .iter()
        .map(render_compiled_item_v1)
        .collect::<Vec<_>>()
        .join("\n\n"))
}

/// Parse the input through the ONE shared multi-root parser and map each surface
/// carrier onto the unified [`ProgramItemHirV1`] view the pipeline compiles.
///
/// A parse failure is wrapped into [`PipelineDiagnosticV1::Parse`] so the parse
/// stage renders byte-identically to the pipeline's own parse-stage reject
/// (D22): one diagnostic payload for the same failing input across surfaces.
fn parse_items_v1(input: &str) -> Result<Vec<ProgramItemHirV1>, PipelineDiagnosticV1> {
    let builder = HirBuilderV1::new();
    let document = parse_document(input, &builder).map_err(PipelineDiagnosticV1::Parse)?;
    Ok(document
        .items
        .into_iter()
        .map(quanta_gqlang_v2_surface::SurfaceItemV1::into_program_item)
        .collect())
}

/// Render a [`PipelineDiagnosticV1`] as a stable, stage-tagged report. The inner
/// stage's stable code and span are surfaced verbatim (never rewritten), so CLI
/// / LSP / tests see one diagnostic payload for the same failing query.
#[must_use]
pub fn render_pipeline_diagnostic_v1(diag: &PipelineDiagnosticV1) -> String {
    let span = diag.span();
    [
        format!("pipeline.error.stage={}", diag.stage().as_str()),
        format!("pipeline.error.code={}", diag.code()),
        format!("pipeline.error.span={}..{}", span.start, span.end),
        format!("pipeline.error.message={diag}"),
    ]
    .join("\n")
}
