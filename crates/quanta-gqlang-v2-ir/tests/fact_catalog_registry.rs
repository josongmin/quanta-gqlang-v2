#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "registry integrity test asserts published schemas validate and resolve"
)]
//! F06/E11A — the catalog-facing fact registry. Proof rail: `--test fact_catalog_registry`.
//! Every published schema must `validate_v1`-clean (a malformed published schema
//! fails the build, not a DESCRIBE at runtime) and resolve by canonical name.

use quanta_gqlang_v2_ir::catalog::fact_catalog::{
    describe_schema_rows_v1, entity_schema_for_v1, published_entity_names_v1, relation_schema_for_v1,
};

#[test]
fn every_published_entity_validates_and_resolves_v1() {
    for name in published_entity_names_v1() {
        let schema = entity_schema_for_v1(&name)
            .unwrap_or_else(|| panic!("published entity `{name}` must resolve"));
        assert_eq!(
            schema.validate_v1(),
            Ok(()),
            "published entity `{name}` must validate clean"
        );
    }
}

#[test]
fn published_relation_validates_v1() {
    let relation = relation_schema_for_v1("catalog.deployment_owns_artifact")
        .expect("published relation must resolve");
    assert_eq!(relation.validate_v1(), Ok(()));
}

#[test]
fn describe_rows_render_entity_fields_v1() {
    // Bare canonical name and DESCRIBE `<kind>::<name>` reference both resolve.
    let bare = describe_schema_rows_v1("catalog.deployments").expect("entity rows");
    let prefixed = describe_schema_rows_v1("model::catalog.deployments").expect("entity rows");
    assert_eq!(bare, prefixed, "kind prefix must not change resolved rows");
    assert!(
        bare.iter().any(|(name, ty)| name == "environment" && ty == "string"),
        "got {bare:?}"
    );
}

#[test]
fn describe_rows_fail_closed_on_unknown_target_v1() {
    assert_eq!(
        describe_schema_rows_v1("model::nope.absent"),
        None,
        "unknown target must not invent fields"
    );
}
