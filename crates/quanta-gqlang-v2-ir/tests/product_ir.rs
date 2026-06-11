#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "product-ir tests assert exhaustive declaration lowering"
)]
//! F04/E06B hellgate — every declaration kind lowers to a product asset with a
//! distinct, stable AssetId. NOTE: do not add a catch-all to the lowering match.
use quanta_gqlang_v2_ir::product::{lower_declaration_to_product_v1, ProductAssetKindV1};
use quanta_gqlang_v2_ir::{lower_document_to_ir_v1, PublicIrRootV1};
use quanta_gqlang_v2_sema::{analyze_document_v1, parse_document_v1};
use std::collections::BTreeSet;

fn declaration_of_v1(input: &str) -> quanta_gqlang_v2_ir::root_ir::DeclarationIrV1 {
    let doc = parse_document_v1(input).expect("parse");
    let typed = analyze_document_v1(&doc).expect("sema");
    match lower_document_to_ir_v1(&typed).expect("ir").pop().expect("root") {
        PublicIrRootV1::Declaration(d) => d,
        other => panic!("expected declaration, got {other:?}"),
    }
}

#[test]
fn every_declaration_kind_lowers_to_a_distinct_product_asset_v1() {
    let cases = [
        ("VIEW catalog.users ;; query-ref=queries/catalog/users", ProductAssetKindV1::View),
        ("RULE shell.exec ;; check-ref=checks/security/shell-exec", ProductAssetKindV1::Rule),
        ("PACK secure-pack ;; source=registry::security ;; activation=workspace", ProductAssetKindV1::Pack),
        ("PROFILE strict-review ;; capabilities=graph.read ;; admission=certified-only", ProductAssetKindV1::Profile),
        ("GATE release-gate ;; checks=owasp.sql ;; binding=all", ProductAssetKindV1::Gate),
    ];
    let mut ids = BTreeSet::new();
    for (input, expected_kind) in cases {
        let asset = lower_declaration_to_product_v1(declaration_of_v1(input));
        assert_eq!(asset.kind, expected_kind, "kind for `{input}`");
        assert!(ids.insert(asset.asset_id.clone()), "asset id must be unique: {:?}", asset.asset_id);
        assert!(asset.asset_id.0.contains("::"), "asset id is kind-tagged");
    }
}
