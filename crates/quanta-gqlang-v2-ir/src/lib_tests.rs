//! Test-only wiring checks for the IR lowering surface. Kept out of `lib.rs`
//! so the production module carries no inline `#[cfg(test)]` content.

use crate::{lower_document_to_ir_v1, PublicIrRootV1, CRATE_ID_V1, SEMA_CRATE_ID_V1};

#[test]
fn ir_keeps_sema_boundary_visible() {
    assert_eq!(CRATE_ID_V1, "quanta-gqlang-v2-ir");
    assert_eq!(SEMA_CRATE_ID_V1, "quanta-gqlang-v2-sema");
}

#[test]
fn ir_surface_is_wired() {
    let doc =
        quanta_gqlang_v2_syntax::parse_document_v1("SELECT id FROM catalog.users").expect("parse");
    let typed = quanta_gqlang_v2_sema::analyze_document_v1(&doc).expect("sema");
    let ir = lower_document_to_ir_v1(&typed).expect("ir");
    assert_eq!(ir.len(), 1);
}

#[test]
fn ir_lowers_interactive_declaration_and_meta_roots_structurally() {
    let show = quanta_gqlang_v2_syntax::parse_document_v1("SHOW packs").expect("show");
    let export = quanta_gqlang_v2_syntax::parse_document_v1(
        "EXPORT source=result::hotspots/latest ;; kind=sarif ;; profile=security-audit",
    )
    .expect("export");
    let gate = quanta_gqlang_v2_syntax::parse_document_v1(
        "GATE release-gate ;; checks=owasp.sql,authz.path ;; binding=all",
    )
    .expect("gate");
    let require = quanta_gqlang_v2_syntax::parse_document_v1("REQUIRE CERTIFIED").expect("require");

    let show_ir = lower_document_to_ir_v1(
        &quanta_gqlang_v2_sema::analyze_document_v1(&show).expect("show sema"),
    )
    .expect("show ir");
    let export_ir = lower_document_to_ir_v1(
        &quanta_gqlang_v2_sema::analyze_document_v1(&export).expect("export sema"),
    )
    .expect("export ir");
    let gate_ir = lower_document_to_ir_v1(
        &quanta_gqlang_v2_sema::analyze_document_v1(&gate).expect("gate sema"),
    )
    .expect("gate ir");
    let require_ir = lower_document_to_ir_v1(
        &quanta_gqlang_v2_sema::analyze_document_v1(&require).expect("require sema"),
    )
    .expect("require ir");

    assert!(matches!(
        &show_ir[0],
        PublicIrRootV1::Interactive(crate::root_ir::InteractiveIrV1::Show { .. })
    ));
    assert!(matches!(
        &export_ir[0],
        PublicIrRootV1::Interactive(crate::root_ir::InteractiveIrV1::Export { .. })
    ));
    assert!(matches!(
        &gate_ir[0],
        PublicIrRootV1::Declaration(crate::root_ir::DeclarationIrV1::Gate { .. })
    ));
    assert!(matches!(
        &require_ir[0],
        PublicIrRootV1::Meta(crate::root_ir::MetaIrV1::RequireCertified { .. })
    ));
}
