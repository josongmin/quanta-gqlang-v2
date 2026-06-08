#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::catalog::autocomplete_surface::{
    autocomplete_matches_v1, autocomplete_row_by_label_v1, autocomplete_suggestion_rows_v1,
    autocomplete_surface_integrity_v1, AutocompleteSuggestionCategoryV1,
    AutocompleteSuggestionKindV1,
};
use quanta_gqlang_v2_ir::catalog::capability_catalog::{
    capability_row_by_id_v1, capability_row_by_label_v1, CatalogScopeMarkerV1,
};
use quanta_gqlang_v2_ir::catalog::example_registry::{example_row_by_id_v1, ExampleCategoryV1};

#[test]
fn typed_autocomplete_rows_cover_existing_completion_expectations_v1() {
    let root_rows = autocomplete_matches_v1("SE");
    assert!(root_rows.iter().any(|row| row.label == "SELECT"));
    assert!(root_rows
        .iter()
        .all(|row| normalized_v1(row.label).starts_with("se")));

    let noun_rows = autocomplete_matches_v1("pro");
    assert!(noun_rows.iter().any(|row| row.label == "PROFILE"));
    assert!(noun_rows.iter().any(|row| row.label == "profiles"));
    assert!(noun_rows.iter().any(|row| row.label == "profile::"));

    for row in noun_rows {
        assert!(!row.reason.is_empty());
        assert!(!row.detail.is_empty());
        assert!(!row.provenance.source_file.is_empty());
        assert!(!row.provenance.source_marker.is_empty());
    }
}

#[test]
fn typed_autocomplete_rows_resolve_catalog_and_example_links_v1() {
    autocomplete_surface_integrity_v1().expect("autocomplete asset references must resolve");

    for row in autocomplete_suggestion_rows_v1() {
        let capability =
            capability_row_by_id_v1(row.capability_id).expect("capability id must resolve");
        assert_eq!(capability.label, row.label);
        assert_eq!(capability.scope, row.scope);

        for example_id in row.example_ids {
            let example = example_row_by_id_v1(example_id).expect("example id must resolve");
            assert_eq!(example.scope, CatalogScopeMarkerV1::ReadOnlyShell);
            assert!(!example.capability_ids.is_empty());
            for capability_id in example.capability_ids {
                capability_row_by_id_v1(capability_id).expect("example capability must resolve");
            }
        }
    }
}

#[test]
fn profile_family_rows_carry_expected_scope_and_categories_v1() {
    let profile = autocomplete_row_by_label_v1("PROFILE").expect("PROFILE row");
    assert_eq!(profile.kind, AutocompleteSuggestionKindV1::DeclarationRoot);
    assert_eq!(
        profile.category,
        AutocompleteSuggestionCategoryV1::DeclarationKeyword
    );
    assert_eq!(profile.scope, CatalogScopeMarkerV1::DocumentRoot);

    let profiles = autocomplete_row_by_label_v1("profiles").expect("profiles row");
    assert_eq!(profiles.kind, AutocompleteSuggestionKindV1::ShowCategory);
    assert_eq!(
        profiles.category,
        AutocompleteSuggestionCategoryV1::CatalogCategoryKeyword
    );
    assert_eq!(profiles.scope, CatalogScopeMarkerV1::ShowCategoryTail);

    let describe_profile = autocomplete_row_by_label_v1("profile::").expect("profile:: row");
    assert_eq!(
        describe_profile.kind,
        AutocompleteSuggestionKindV1::DescribeKind
    );
    assert_eq!(
        describe_profile.category,
        AutocompleteSuggestionCategoryV1::DescribeTargetPrefix
    );
    assert_eq!(
        describe_profile.scope,
        CatalogScopeMarkerV1::DescribeTargetTail
    );

    let capability = capability_row_by_label_v1("PROFILE").expect("PROFILE capability");
    assert_eq!(capability.label, profile.label);

    let example = example_row_by_id_v1("example.profile.strict_review")
        .expect("profile example should exist");
    assert_eq!(example.category, ExampleCategoryV1::DeclarationStarter);
}

fn normalized_v1(input: &str) -> String {
    input
        .trim()
        .to_ascii_lowercase()
        .replace(['_', '-', ' '], "")
}
