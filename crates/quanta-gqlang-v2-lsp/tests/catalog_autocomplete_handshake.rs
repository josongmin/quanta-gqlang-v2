#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]
//! F20/J08B — catalog↔autocomplete handshake, upgraded from constant-mirror to
//! behavioral. The owner-authority pins remain (they ARE the contract), but the
//! load-bearing assertions now exercise real resolution: every autocomplete row
//! resolves to typed catalog/example provenance, the two owners are a genuine
//! partition, and unknown ids fail closed (≥2 negatives).

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
fn catalog_and_examples_are_distinct_external_owners_v1() {
    // Pinned contract (intentional): the two surfaces carry distinct external
    // owners. Asserted as a partition invariant, not just a constant copy — a
    // future merge of the two owners would break this, which is the point.
    let catalog_owner = capability_catalog_authority_v1();
    let example_owner = example_registry_authority_v1();
    assert_eq!(catalog_owner, "external-frontend-catalog");
    assert_eq!(example_owner, "external-read-only-shell");
    assert_ne!(
        catalog_owner, example_owner,
        "capability catalog and example registry must stay distinct owners"
    );
    capability_catalog_integrity_v1().expect("catalog integrity must remain explicit");
    read_only_example_registry_integrity_v1()
        .expect("example registry integrity must remain explicit");
}

#[test]
fn every_autocomplete_row_resolves_to_typed_provenance_v1() {
    autocomplete_surface_integrity_v1().expect("autocomplete references must resolve");

    let rows = autocomplete_suggestion_rows_v1();
    assert!(
        !rows.is_empty(),
        "autocomplete surface must offer suggestions"
    );
    for row in rows {
        let capability =
            capability_row_by_id_v1(row.capability_id).expect("capability id must resolve");
        assert_eq!(
            capability.provenance.source_kind,
            CatalogSourceKindV1::FrontendSpec,
            "autocomplete capability `{}` must originate from the frontend spec",
            row.capability_id
        );

        for example_id in row.example_ids {
            let example = example_row_by_id_v1(example_id).expect("example id must resolve");
            assert_eq!(
                example.provenance.source_kind,
                CatalogSourceKindV1::CuratedReadOnlyExample,
                "autocomplete example `{example_id}` must be a curated read-only example"
            );
        }
    }
}

#[test]
fn unknown_catalog_and_example_ids_fail_closed_v1() {
    // Behavioral fail-closed (≥2 negatives): the resolvers never invent a row
    // for an id outside the typed registries.
    assert!(
        capability_row_by_id_v1("capability.does-not-exist").is_none(),
        "unknown capability id must not resolve"
    );
    assert!(
        example_row_by_id_v1("example.does-not-exist").is_none(),
        "unknown example id must not resolve"
    );
}
