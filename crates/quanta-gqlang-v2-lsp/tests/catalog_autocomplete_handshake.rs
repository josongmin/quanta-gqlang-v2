#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::catalog::autocomplete_surface::{
    autocomplete_suggestion_rows_v1, autocomplete_surface_integrity_v1,
};
use quanta_gqlang_v2_ir::catalog::capability_catalog::{
    capability_catalog_authority_v1, capability_catalog_integrity_v1, capability_row_by_id_v1,
    CatalogSourceKindV1,
};
use quanta_gqlang_v2_ir::catalog::example_registry::{
    example_registry_authority_v1, example_row_by_id_v1, read_only_example_registry_integrity_v1,
};

#[test]
fn catalog_and_examples_keep_external_owner_authority_v1() {
    assert_eq!(
        capability_catalog_authority_v1(),
        "external-frontend-catalog"
    );
    assert_eq!(example_registry_authority_v1(), "external-read-only-shell");
    capability_catalog_integrity_v1().expect("catalog integrity must remain explicit");
    read_only_example_registry_integrity_v1()
        .expect("example registry integrity must remain explicit");
}

#[test]
fn autocomplete_rows_only_reference_external_owner_metadata_v1() {
    autocomplete_surface_integrity_v1().expect("autocomplete references must resolve");

    for row in autocomplete_suggestion_rows_v1() {
        let capability =
            capability_row_by_id_v1(row.capability_id).expect("capability id must resolve");
        assert_eq!(
            capability.provenance.source_kind,
            CatalogSourceKindV1::FrontendSpec
        );

        for example_id in row.example_ids {
            let example = example_row_by_id_v1(example_id).expect("example id must resolve");
            assert_eq!(
                example.provenance.source_kind,
                CatalogSourceKindV1::CuratedReadOnlyExample
            );
        }
    }
}
