#![allow(
    clippy::expect_used,
    reason = "integration-test assertions use expect on a fixed read-only fixture"
)]
//! F06/E11A — DESCRIBE of a published entity renders its real schema fields in
//! the hover card (catalog-facing root referencing the public fact-schema
//! surface). Proof rail: `--test fact_schema_describe`.

use quanta_gqlang_v2_lsp::server::LspServerV1;

#[test]
fn describe_published_entity_renders_schema_fields_v1() {
    let card = LspServerV1::new()
        .hover("DESCRIBE model::catalog.deployments")
        .expect("DESCRIBE of a published entity should hover");
    assert_eq!(card.root_keyword, "DESCRIBE");
    assert!(
        card.summary
            .iter()
            .any(|line| line == "schema.field=environment:string"),
        "hover must surface published schema fields, got {:?}",
        card.summary
    );
}

#[test]
fn describe_unknown_entity_omits_schema_fields_v1() {
    // Fail-closed: an unknown target keeps only the bare `target=` line — the
    // client can tell "no published schema" from "echoed back".
    let card = LspServerV1::new()
        .hover("DESCRIBE model::nope.absent")
        .expect("DESCRIBE of an unknown target should still hover");
    assert!(
        card.summary
            .iter()
            .all(|line| !line.starts_with("schema.field=")),
        "unknown target must not invent schema fields, got {:?}",
        card.summary
    );
}
