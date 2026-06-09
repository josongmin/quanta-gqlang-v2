//! Wiring unit tests for the sema surface: they assert that `analyze_document_v1`
//! lowers each parsed root to the expected typed variant and that the crate
//! boundary constants stay visible. Kept out of `lib.rs` so the production module
//! carries no `#[cfg(test)]` content.

#![expect(
    clippy::expect_used,
    reason = "sema wiring tests assert validated parse and analyze success"
)]
#![expect(
    clippy::indexing_slicing,
    reason = "wiring tests assert the single lowered root directly"
)]

use crate::{analyze_document_v1, TypedRootV1, CRATE_ID_V1, SYNTAX_CRATE_ID_V1};

#[test]
fn sema_keeps_syntax_boundary_visible() {
    assert_eq!(CRATE_ID_V1, "quanta-gqlang-v2-sema");
    assert_eq!(SYNTAX_CRATE_ID_V1, "quanta-gqlang-v2-syntax");
}

#[test]
fn sema_surface_is_wired() {
    let doc =
        quanta_gqlang_v2_syntax::parse_document_v1("SELECT id FROM catalog.users").expect("parse");
    let typed = analyze_document_v1(&doc).expect("sema");
    assert_eq!(typed.roots.len(), 1);
}

#[test]
fn sema_preserves_structured_interactive_declaration_and_meta_roots() {
    let show = quanta_gqlang_v2_syntax::parse_document_v1("SHOW profiles").expect("show");
    let sample = quanta_gqlang_v2_syntax::parse_document_v1(
        "SAMPLE query=payments latency ;; scope=repo::frontend ;; mode=hybrid ;; limit=5",
    )
    .expect("sample");
    let pack = quanta_gqlang_v2_syntax::parse_document_v1(
        "PACK secure-pack ;; source=registry::security ;; activation=workspace",
    )
    .expect("pack");
    let require = quanta_gqlang_v2_syntax::parse_document_v1("REQUIRE CERTIFIED").expect("require");

    let show_typed = analyze_document_v1(&show).expect("show sema");
    let sample_typed = analyze_document_v1(&sample).expect("sample sema");
    let pack_typed = analyze_document_v1(&pack).expect("pack sema");
    let require_typed = analyze_document_v1(&require).expect("require sema");

    assert!(matches!(show_typed.roots[0], TypedRootV1::ShowDescribe(_)));
    assert!(matches!(
        sample_typed.roots[0],
        TypedRootV1::SampleCompareExport(_)
    ));
    assert!(matches!(pack_typed.roots[0], TypedRootV1::Declaration(_)));
    assert!(matches!(require_typed.roots[0], TypedRootV1::Meta(_)));
}
