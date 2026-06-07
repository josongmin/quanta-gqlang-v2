use quanta_gqlang_v2_lsp::handlers::{starter_scaffold, zero_query};

#[test]
fn starter_scaffold_is_deterministic_and_read_only() {
    let first = starter_scaffold::build_starter_scaffold_v1();
    let second = starter_scaffold::build_starter_scaffold_v1();

    assert_eq!(first, second);
    assert_eq!(first.mode, starter_scaffold::ZeroQueryModeV1::ReadOnly);
    assert_eq!(
        first.provenance,
        starter_scaffold::ZeroQueryProvenanceV1::TypedCatalogExampleMetadata
    );
    assert_eq!(
        first
            .groups
            .iter()
            .map(|group| group.kind)
            .collect::<Vec<_>>(),
        vec![
            starter_scaffold::StarterGroupKindV1::Catalog,
            starter_scaffold::StarterGroupKindV1::Analysis,
            starter_scaffold::StarterGroupKindV1::Retrieval,
        ]
    );
    assert!(first
        .groups
        .iter()
        .all(|group| !group.suggestions.is_empty()));
    assert!(first
        .groups
        .iter()
        .flat_map(|group| group.suggestions.iter())
        .all(|suggestion| {
            suggestion.behavior == starter_scaffold::StarterBehaviorV1::ReadOnlyScaffold
        }));
}

#[test]
fn catalog_starters_are_stable_show_describe_pairs() {
    let report = starter_scaffold::build_starter_scaffold_v1();
    let catalog = &report.groups[0];

    assert_eq!(catalog.kind, starter_scaffold::StarterGroupKindV1::Catalog);
    assert_eq!(catalog.suggestions.len(), 14);
    assert_eq!(catalog.suggestions[0].query_scaffold, "SHOW capabilities");
    assert_eq!(
        catalog.suggestions[1].query_scaffold,
        "DESCRIBE capability::code.search"
    );
    assert_eq!(catalog.suggestions[12].query_scaffold, "SHOW gates");
    assert_eq!(
        catalog.suggestions[13].query_scaffold,
        "DESCRIBE gate::release.blockers"
    );
    assert!(catalog.suggestions.iter().all(|suggestion| {
        matches!(
            suggestion.root_kind,
            starter_scaffold::StarterRootKindV1::Show
                | starter_scaffold::StarterRootKindV1::Describe
        )
    }));
    assert_eq!(
        catalog.suggestions[0].reason.code,
        starter_scaffold::StarterReasonCodeV1::CatalogOverview
    );
    assert_eq!(
        catalog.suggestions[1].reason.code,
        starter_scaffold::StarterReasonCodeV1::TypedTargetDetail
    );
}

#[test]
fn zero_query_scaffold_stays_non_mutating_and_never_claims_runtime_counts() {
    let report = starter_scaffold::build_starter_scaffold_v1();
    let rendered = report.render_v1();
    let queries = report
        .groups
        .iter()
        .flat_map(|group| group.suggestions.iter())
        .map(|suggestion| suggestion.query_scaffold)
        .collect::<Vec<_>>();

    assert!(queries.iter().any(|query| query.starts_with("SELECT ")));
    assert!(queries.iter().any(|query| query.starts_with("SAMPLE ")));
    assert!(!queries
        .iter()
        .any(|query| query.starts_with("USE ") || query.starts_with("REQUIRE ")));
    assert!(!rendered.contains("result_count"));
    assert!(!rendered.contains("code_action"));
    assert!(rendered.contains("starter.provenance=typed-catalog-example-metadata"));
}

#[test]
fn lsp_zero_query_translates_typed_groups_and_reasons() {
    let first = zero_query::handle_zero_query().expect("zero query");
    let second = zero_query::handle_zero_query().expect("zero query rerun");

    assert_eq!(first, second);
    assert_eq!(first.mode, "read-only");
    assert_eq!(first.provenance, "typed-catalog-example-metadata");
    assert_eq!(
        first
            .groups
            .iter()
            .map(|group| group.id)
            .collect::<Vec<_>>(),
        vec!["catalog", "analysis", "retrieval"]
    );
    assert_eq!(
        first.groups[0].suggestions[0].reason_code,
        "catalog_overview"
    );
    assert_eq!(
        first.groups[0].suggestions[1].reason_code,
        "typed_target_detail"
    );
    assert_eq!(
        first.groups[1].suggestions[0].query_scaffold,
        "SELECT id, path FROM catalog.files WHERE severity = 'high' LIMIT 10"
    );
    assert_eq!(
        first.groups[2].suggestions[0].reason_code,
        "retrieval_shape_preview"
    );
    assert!(first
        .groups
        .iter()
        .flat_map(|group| group.suggestions.iter())
        .all(|suggestion| suggestion.behavior == "read-only-scaffold"));
    assert!(!first.render_v1().contains("result_count"));
}
