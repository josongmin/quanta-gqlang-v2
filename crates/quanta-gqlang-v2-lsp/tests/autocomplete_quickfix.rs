#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_lsp::handlers::completion::handle_completion;
use quanta_gqlang_v2_lsp::handlers::quickfix::{
    example_backed_quickfix_v1, handle_quickfix, QuickfixSuggestionV1,
};

#[test]
fn ranked_completion_prefers_profile_root_before_catalog_and_describe_suffix_v1() {
    let items = handle_completion("pro").expect("completion should succeed");
    assert_eq!(items[0].label, "PROFILE");
    assert_eq!(items[1].label, "profiles");
    assert_eq!(items[2].label, "profile::");
    assert_eq!(items[0].kind, "declaration-root");
    assert_eq!(items[0].scope, "document-root");
    assert_eq!(items[0].match_reason, "exact-scope");
    assert!(items[0]
        .example_query
        .as_deref()
        .is_some_and(|query| query.starts_with("PROFILE strict-review")));

    let show_items = handle_completion("SHOW pro").expect("SHOW completion should succeed");
    assert_eq!(show_items.len(), 1);
    assert_eq!(show_items[0].label, "profiles");
    assert_eq!(show_items[0].scope, "show-category-tail");
    assert_eq!(show_items[0].match_reason, "exact-scope");

    let describe_items =
        handle_completion("DESCRIBE pro").expect("DESCRIBE completion should succeed");
    assert_eq!(describe_items.len(), 1);
    assert_eq!(describe_items[0].label, "profile::");
    assert_eq!(describe_items[0].scope, "describe-target-tail");
    assert_eq!(describe_items[0].match_reason, "exact-scope");
}

#[test]
fn quickfix_suggests_missing_model_scaffold_without_applying_edits_v1() {
    let fixes =
        handle_quickfix("DESCRIBE model::billing_warehouse").expect("quickfix should succeed");
    assert_eq!(
        fixes,
        vec![QuickfixSuggestionV1 {
            code: "missing-model-scaffold".to_string(),
            label: "Declare MODEL billing_warehouse".to_string(),
            detail: "provider declaration".to_string(),
            scaffold: "MODEL billing_warehouse ;; provider=provider.default ;; source=repo::billing_warehouse".to_string(),
            edits_document: false,
        }]
    );
}

#[test]
fn quickfix_suggests_query_scaffold_for_missing_view_query_shape_v1() {
    let fixes = handle_quickfix("VIEW catalog.users").expect("quickfix should succeed");
    assert_eq!(
        fixes,
        vec![QuickfixSuggestionV1 {
            code: "missing-query-scaffold".to_string(),
            label: "Add VIEW query scaffold for catalog.users".to_string(),
            detail: "named query declaration".to_string(),
            scaffold: "VIEW catalog.users ;; body=SELECT id FROM catalog.users LIMIT 25"
                .to_string(),
            edits_document: false,
        }]
    );
    // A quickfix scaffold handed to the user must parse under the canonical grammar.
    assert!(
        quanta_gqlang_v2_ir::frontend::parse_document_v1(&fixes[0].scaffold).is_ok(),
        "VIEW quickfix scaffold must re-parse, got {:?}",
        fixes[0].scaffold
    );
}

#[test]
fn quickfix_root_detection_is_case_insensitive_v1() {
    // Hover analysis upper-cases the root keyword, so a lowercase `model` still
    // produces a `missing_required_clause` diagnostic; the quickfix must resolve
    // the same scaffold instead of silently dropping it on a case mismatch.
    let lower = handle_quickfix("model billing").expect("lowercase MODEL should scaffold");
    let upper = handle_quickfix("MODEL billing").expect("uppercase MODEL should scaffold");
    assert_eq!(lower, upper);
    assert_eq!(lower.len(), 1);
    assert_eq!(lower[0].code, "missing-model-scaffold");
}

#[test]
fn quickfix_fails_closed_on_malformed_describe_target_v1() {
    let err = handle_quickfix("DESCRIBE model::")
        .expect_err("malformed DESCRIBE target should fail closed");
    assert_eq!(
        err,
        "class=syntax code=describe_target_invalid_shape message=DESCRIBE target must use `<kind>::<identifier>`"
    );
}

#[test]
fn quickfix_can_surface_example_backed_read_only_suggestion_v1() {
    let fix = example_backed_quickfix_v1("example.require_certified")
        .expect("example-backed quickfix should exist");
    assert_eq!(fix.code, "example:example.require_certified");
    assert_eq!(fix.label, "REQUIRE CERTIFIED policy");
    assert_eq!(fix.scaffold, "REQUIRE CERTIFIED");
    assert!(!fix.edits_document);

    let use_pack = example_backed_quickfix_v1("example.use_pack.secure_pack")
        .expect("example-backed USE PACK quickfix should exist");
    assert_eq!(use_pack.code, "example:example.use_pack.secure_pack");
    assert_eq!(
        use_pack.scaffold,
        "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock"
    );
    assert!(!use_pack.edits_document);
}
