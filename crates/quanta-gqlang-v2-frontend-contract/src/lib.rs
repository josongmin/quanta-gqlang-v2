#![forbid(unsafe_code)]

//! Canonical downstream seam for external qglang-v3 consumers.
//!
//! Sibling repos must depend on this crate rather than scattering direct owner
//! crate edges across `surface` / `hir` / `algebra` / `planning` / `pipeline`.
//! The owner crates remain authoritative, but this facade is the stable
//! frontend-only bundle consumed outside the repo boundary.

pub use quanta_gqlang_v2_algebra::*;
pub use quanta_gqlang_v2_hir::*;
pub use quanta_gqlang_v2_pipeline::{
    compile_document_structural, compile_document_with_context, compile_document_with_schema,
    compile_item_structural, compile_item_with_context, compile_item_with_schema,
    compile_program_structural, compile_program_with_schema, compile_with_context, CompiledItemV1,
    CompiledProgramV1, FrontendContextV1, ModuleGraphV1, NoServeReasonV1, PackGraphV1, PackIdV1,
    PackManifestV1, PipelineDiagnosticV1, PipelineStageV1, VersionRangeV1, VersionV1,
};
pub use quanta_gqlang_v2_planning::*;
pub use quanta_gqlang_v2_surface::{
    format_document, format_item, parse_document, parse_expr, parse_predicate, parse_program,
    LexErrorV1, ParseDiagnosticV1, ParseErrorCodeV1, ParsedDocumentV1, ParsedProgramV1,
    SurfaceItemV1, TokenKindV1, TokenV1,
};
pub use quanta_gqlang_v2_types::*;

/// Unified item view helper for downstream family inspection.
///
/// External consumers often need the one canonical `ProgramItemHirV1` without
/// depending on `SurfaceItemV1::to_program_item()` directly.
#[must_use]
pub fn compiled_program_item_v1(program: &CompiledProgramV1) -> ProgramItemHirV1 {
    program.item().to_program_item()
}
