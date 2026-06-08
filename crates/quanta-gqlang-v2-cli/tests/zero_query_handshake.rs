#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::catalog::capability_catalog::capability_catalog_authority_v1;
use quanta_gqlang_v2_ir::catalog::example_registry::example_registry_authority_v1;
use quanta_gqlang_v2_lsp::handlers::starter_scaffold;

#[test]
fn zero_query_scaffold_stays_read_only_and_external_owner_v1() {
    let report = starter_scaffold::build_starter_scaffold_v1().expect("starter scaffold");

    assert_eq!(report.mode, starter_scaffold::ZeroQueryModeV1::ReadOnly);
    assert_eq!(
        report.provenance,
        starter_scaffold::ZeroQueryProvenanceV1::TypedCatalogExampleMetadata
    );
    assert_eq!(report.authority, example_registry_authority_v1());
    assert_eq!(
        report
            .groups
            .iter()
            .map(|group| group.kind.id_v1())
            .collect::<Vec<_>>(),
        vec!["catalog", "analysis", "retrieval"]
    );
    assert!(report
        .groups
        .iter()
        .flat_map(|group| group.suggestions.iter())
        .all(|suggestion| {
            suggestion.behavior == starter_scaffold::StarterBehaviorV1::ReadOnlyScaffold
        }));
    assert!(report
        .groups
        .iter()
        .flat_map(|group| group.suggestions.iter())
        .all(|suggestion| {
            !suggestion.query_scaffold.starts_with("RENAME")
                && !suggestion.query_scaffold.starts_with("APPLY")
                && !suggestion.query_scaffold.starts_with("CODE ACTION")
        }));
}

#[test]
fn zero_query_scaffold_keeps_catalog_and_example_authority_split_v1() {
    let report = starter_scaffold::build_starter_scaffold_v1().expect("starter scaffold");
    let catalog = &report.groups[0];

    assert!(catalog
        .suggestions
        .iter()
        .all(|suggestion| suggestion.authority == capability_catalog_authority_v1()));
    assert!(report
        .groups
        .iter()
        .skip(1)
        .flat_map(|group| group.suggestions.iter())
        .all(|suggestion| suggestion.authority == example_registry_authority_v1()));

    let rendered = report.render_v1();
    assert!(rendered.contains("starter.authority=external-read-only-shell"));
    assert!(rendered.contains("starter.group[0].suggestion[0].authority=external-frontend-catalog"));
}
