#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "integration-test assertions use expect/indexing on a fixed fixture"
)]
//! F06/E11A — the headless query summary renders a published entity's real
//! schema fields under DESCRIBE (catalog-facing root referencing the public
//! fact-schema surface). Proof rail: `--test fact_schema_describe`.

use quanta_gqlang_v2_cli::commands::query;

#[test]
fn describe_published_entity_summary_carries_schema_fields_v1() {
    let summary = query::run("DESCRIBE model::catalog.deployments").expect("query summary");
    assert_eq!(summary.roots[0].syntax_kind, "describe");
    let syntax_summary = &summary.roots[0].syntax_summary;
    assert!(
        syntax_summary.contains("describe.schema.field[0]=environment:string"),
        "summary must surface published schema fields, got {syntax_summary}"
    );
    assert!(
        summary
            .render_v1()
            .contains("describe.schema.field[1]=released_at:timestamp"),
        "render must include every published field"
    );
}

#[test]
fn describe_unknown_entity_summary_stays_bare_v1() {
    // Fail-closed: unknown targets keep only `target=` — no invented fields.
    let summary = query::run("DESCRIBE model::nope.absent").expect("query summary");
    let syntax_summary = &summary.roots[0].syntax_summary;
    assert_eq!(syntax_summary, "target=model::nope.absent");
}
