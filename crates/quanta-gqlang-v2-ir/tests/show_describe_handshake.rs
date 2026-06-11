#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "J03A handshake asserts the external SHOW/DESCRIBE catalog surface end-to-end"
)]
//! F19/J03A (external) — the SHOW/DESCRIBE catalog introspection surface on the
//! ir side. SHOW enumerates published entities; DESCRIBE narrows to one entity's
//! schema rows; an unknown target fails closed (no invented schema). Proof rail:
//! `ir --test show_describe_handshake`. Behavioral over the real `fact_catalog`
//! registry — not a constant mirror.

use quanta_gqlang_v2_ir::catalog::fact_catalog::{
    describe_schema_rows_v1, entity_schema_for_v1, published_entity_names_v1,
};

#[test]
fn show_enumerates_published_entities_v1() {
    let names = published_entity_names_v1();
    assert!(!names.is_empty(), "SHOW must enumerate published entities");
    // Every enumerated entity must actually resolve (SHOW↔DESCRIBE coherence).
    for name in &names {
        assert!(
            entity_schema_for_v1(name).is_some(),
            "enumerated entity `{name}` must resolve"
        );
    }
}

#[test]
fn describe_narrows_to_entity_schema_rows_v1() {
    let name = &published_entity_names_v1()[0];
    let rows = describe_schema_rows_v1(name)
        .unwrap_or_else(|| panic!("DESCRIBE of published `{name}` must yield schema rows"));
    assert!(!rows.is_empty(), "a published entity must publish ≥1 field");
    // Each row is a (field-name, type-label) pair with non-empty parts.
    for (field, type_label) in &rows {
        assert!(!field.is_empty() && !type_label.is_empty());
    }
    // The kind-prefixed DESCRIBE target form resolves to the same rows.
    assert_eq!(describe_schema_rows_v1(&format!("model::{name}")).as_ref(), Some(&rows));
}

#[test]
fn describe_unknown_target_fails_closed_v1() {
    assert_eq!(
        describe_schema_rows_v1("model::nope.absent"),
        None,
        "an unknown DESCRIBE target must not invent schema"
    );
    assert!(entity_schema_for_v1("nope.absent").is_none());
}
