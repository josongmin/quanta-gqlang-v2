//! `fmt` / `v3-fmt` command: canonical v3 formatter over the shared surface.
//!
//! Parses the input through the ONE shared v3 surface parser
//! (`surface::parse_document`, D22) and renders canonical source through the ONE
//! shared formatter (`surface::format_document` / `surface::format_item`). The
//! command is IDEMPOTENT: `fmt(fmt(x)) == fmt(x)` for every family.
//!
//! Input may be a SINGLE item or a `;`-separated multi-item document; the
//! formatter renders each top-level item joined by the document separator.
//!
//! Fail-closed (D10): a non-parsing input returns the typed parse diagnostic
//! (stable code + exact span) rather than a partial format.

use quanta_gqlang_v2_hir::{HirBuilderV1, ProgramItemHirV1};
use quanta_gqlang_v2_surface::{format_document, parse_document, ParseDiagnosticV1};

/// Run `fmt`: parse `input` as a v3 document and render canonical v3 source.
///
/// # Errors
///
/// Returns a rendered parse-diagnostic string (stable code + span) when the
/// input does not parse — fail-closed, never a partial format.
pub fn run(input: &str) -> Result<String, String> {
    format_source_v1(input).map_err(|diag| render_parse_diagnostic_v1(&diag))
}

/// Parse `input` through the shared v3 surface document parser and render it back
/// as canonical source via the shared formatter.
///
/// # Errors
///
/// Returns the typed [`ParseDiagnosticV1`] (stable code + exact span) verbatim
/// when the input does not parse.
fn format_source_v1(input: &str) -> Result<String, ParseDiagnosticV1> {
    let builder = HirBuilderV1::new();
    let document = parse_document(input, &builder)?;
    let items: Vec<ProgramItemHirV1> = document
        .items
        .into_iter()
        .map(quanta_gqlang_v2_surface::SurfaceItemV1::into_program_item)
        .collect();
    Ok(format_document(&items))
}

/// Render a surface parse diagnostic as a stable, line-oriented report carrying
/// the stable code and exact span verbatim.
fn render_parse_diagnostic_v1(diag: &ParseDiagnosticV1) -> String {
    [
        format!("v3fmt.error.code={}", diag.code_str()),
        format!("v3fmt.error.span={}..{}", diag.span.start, diag.span.end),
        format!("v3fmt.error.message={diag}"),
    ]
    .join("\n")
}
