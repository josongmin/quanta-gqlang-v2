#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "scaffold integrity test asserts the moved starter-scaffold core structure"
)]
//! F08/E12A — the starter-scaffold core now lives in the ir catalog plane. This
//! rail (moved out of the CLI test that masked the LSP-owned logic) pins the
//! scaffold structure at its real home. Proof rail: `ir --test zero_query_scaffold`.

use quanta_gqlang_v2_ir::catalog::starter_scaffold::{
    build_starter_scaffold_v1, StarterBehaviorV1, StarterGroupKindV1, StarterReasonCodeV1,
    StarterRootKindV1, ZeroQueryModeV1, ZeroQueryProvenanceV1,
};
use quanta_gqlang_v2_ir::catalog::zero_query_surface::build_zero_query_response_v1;

#[test]
fn starter_scaffold_is_deterministic_and_read_only_v1() {
    let first = build_starter_scaffold_v1().expect("starter scaffold");
    let second = build_starter_scaffold_v1().expect("starter scaffold rerun");

    assert_eq!(first, second);
    assert_eq!(first.mode, ZeroQueryModeV1::ReadOnly);
    assert_eq!(
        first.provenance,
        ZeroQueryProvenanceV1::TypedCatalogExampleMetadata
    );
    assert_eq!(first.authority, "external-read-only-shell");
    assert_eq!(
        first.groups.iter().map(|group| group.kind).collect::<Vec<_>>(),
        vec![
            StarterGroupKindV1::Catalog,
            StarterGroupKindV1::Analysis,
            StarterGroupKindV1::Retrieval,
        ]
    );
    assert!(first.groups.iter().all(|group| !group.suggestions.is_empty()));
    assert!(first
        .groups
        .iter()
        .flat_map(|group| group.suggestions.iter())
        .all(|suggestion| suggestion.behavior == StarterBehaviorV1::ReadOnlyScaffold));
}

#[test]
fn catalog_starters_are_stable_show_describe_pairs_v1() {
    let report = build_starter_scaffold_v1().expect("starter scaffold");
    let catalog = &report.groups[0];

    assert_eq!(catalog.kind, StarterGroupKindV1::Catalog);
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
    assert!(catalog.suggestions.iter().all(|suggestion| matches!(
        suggestion.root_kind,
        StarterRootKindV1::Show | StarterRootKindV1::Describe
    )));
    assert_eq!(
        catalog.suggestions[0].reason.code,
        StarterReasonCodeV1::CatalogOverview
    );
    assert_eq!(
        catalog.suggestions[1].reason.code,
        StarterReasonCodeV1::TypedTargetDetail
    );
    assert_eq!(catalog.suggestions[0].authority, "external-frontend-catalog");
}

#[test]
fn zero_query_scaffold_stays_non_mutating_and_never_claims_runtime_counts_v1() {
    let report = build_starter_scaffold_v1().expect("starter scaffold");
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
    assert!(rendered.contains("starter.authority=external-read-only-shell"));
}

#[test]
fn zero_query_surface_translates_typed_groups_and_reasons_v1() {
    let first = build_zero_query_response_v1().expect("zero query");
    let second = build_zero_query_response_v1().expect("zero query rerun");

    assert_eq!(first, second);
    assert_eq!(first.mode, "read-only");
    assert_eq!(first.provenance, "typed-catalog-example-metadata");
    assert_eq!(
        first.groups.iter().map(|group| group.id).collect::<Vec<_>>(),
        vec!["catalog", "analysis", "retrieval"]
    );
    assert_eq!(first.groups[0].suggestions[0].reason_code, "catalog_overview");
    assert_eq!(
        first.groups[0].suggestions[1].reason_code,
        "typed_target_detail"
    );
    assert_eq!(
        first.groups[1].suggestions[0].query_scaffold,
        "SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 7"
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
