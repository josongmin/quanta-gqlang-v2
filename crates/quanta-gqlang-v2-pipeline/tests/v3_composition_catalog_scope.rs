//! Composition producers and body must share the document catalog scope so that
//! cross-item catalog rules (duplicate `<kind>::<name>`) fire across a
//! composition's nested items, just as they do at the document top level.
//!
//! Regression: `composition_catalog_scope_leak` — `compile_composition` used to
//! compile each producer and the body through `compile_item` (which mints a
//! throwaway catalog per item) instead of threading the shared document catalog,
//! so a duplicate VIEW name declared by a producer and the body was silently
//! accepted instead of rejected with `modules.duplicate_catalog_name`.
#![allow(
    clippy::expect_used,
    reason = "integration test: panic on the absence of the expected reject is intentional"
)]

use quanta_gqlang_v2_hir::{
    CatalogDeclHirV1, CatalogPayloadV1, CompositionBindingV1, CompositionHirV1, GraphRefV1,
    HirNodeIdV1, ProgramItemHirV1, SpanV1, ViewBodyV1,
};
use quanta_gqlang_v2_pipeline::{compile_document, PipelineStageV1};
use quanta_gqlang_v2_types::TypeEnvV1;

/// A `VIEW <name> = query-ref <target>` catalog declaration carrier.
fn view_decl(raw_id: u32, name: &str) -> ProgramItemHirV1 {
    let span = SpanV1::new(0, 1);
    let decl = CatalogDeclHirV1::new(
        HirNodeIdV1::from_raw(raw_id),
        span,
        name,
        span,
        CatalogPayloadV1::View(ViewBodyV1::QueryRef(GraphRefV1 {
            name: "base".to_owned(),
            span,
        })),
    );
    ProgramItemHirV1::CatalogDecl(Box::new(decl))
}

/// A composition `LET p1 = VIEW x ... VIEW x WITH p1` declares the SAME
/// `view::x` asset in a producer and the body. Because the two nested items
/// share the composition's catalog scope, the body's registration collides and
/// the whole document must reject tagged `modules` with
/// `modules.duplicate_catalog_name`.
#[test]
fn composition_producer_and_body_share_catalog_scope() {
    let span = SpanV1::new(0, 1);
    let producer = view_decl(1, "x");
    let body = view_decl(2, "x");

    let composition = CompositionHirV1::new(
        HirNodeIdV1::from_raw(0),
        span,
        vec![CompositionBindingV1::new("p1", span, producer)],
        "p1",
        span,
        body,
    );
    let items = vec![ProgramItemHirV1::Composition(Box::new(composition))];

    let result = compile_document(&items, &TypeEnvV1::new());

    let diagnostic = result.expect_err(
        "a composition whose producer and body declare the same VIEW name must reject \
         on the shared catalog scope",
    );
    assert_eq!(
        diagnostic.stage(),
        PipelineStageV1::Modules,
        "duplicate catalog name across composition items is a modules-stage reject"
    );
    assert_eq!(
        diagnostic.code(),
        "modules.duplicate_catalog_name",
        "the inner stable code is preserved verbatim"
    );
}
