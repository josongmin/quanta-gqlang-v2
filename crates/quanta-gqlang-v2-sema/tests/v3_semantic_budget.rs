#![expect(clippy::unwrap_used, reason = "owner test assertions inspect exact budget values")]

use quanta_gqlang_v2_sema::FrontendBudgetV2;

fn budget(source: u32, frame: u32, semantic: u32) -> Result<FrontendBudgetV2, quanta_gqlang_v2_sema::FrontendAuthorityErrorV2> {
    FrontendBudgetV2::try_new(
        source, frame, 3, 3, 3, 3, 3, 3, semantic, 3, 3, 3, 3, 3,
        3, 3, 3, 3, 3,
    )
}

#[test]
fn zero_dimensions_and_smaller_frame_reject_without_fabricating_source_span() {
    let error = budget(0, 2, 2).unwrap_err();
    assert_eq!(error.code(), "frontend.invalid_budget");
    let error = budget(3, 2, 2).unwrap_err();
    assert_eq!(error.code(), "frontend.invalid_budget");
    let error = budget(2, 2, 0).unwrap_err();
    assert_eq!(error.code(), "frontend.invalid_budget");
}

#[test]
fn exact_positive_limits_and_authority_profile_are_preserved() {
    let budget = budget(2, 2, 1).unwrap();
    assert_eq!(budget.max_source_bytes(), 2);
    assert_eq!(budget.max_frame_bytes(), 2);
    assert_eq!(budget.max_semantic_steps(), 1);
    let profile = budget.authority_profile().unwrap();
    assert_eq!(profile.module_nodes(), 3);
    assert_eq!(profile.module_edges(), 3);
    assert_eq!(profile.module_steps(), 3);
    assert_eq!(profile.authority_bytes(), 3);
    assert_eq!(profile.authority_string_bytes(), 3);
}

#[test]
fn pinned_standard_limits_have_no_zero_or_unbounded_dimensions() {
    let budget = FrontendBudgetV2::standard_v2().unwrap();
    assert_eq!(budget.max_source_bytes(), 1_048_576);
    assert_eq!(budget.max_frame_bytes(), 4_194_304);
    assert_eq!(budget.max_tokens(), 131_072);
    assert_eq!(budget.max_document_items(), 4_096);
    assert_eq!(budget.max_expression_depth(), 64);
    assert_eq!(budget.max_expression_nodes(), 262_144);
    assert_eq!(budget.max_collection_width(), 4_096);
    assert_eq!(budget.max_projection_width(), 4_096);
    assert_eq!(budget.max_semantic_steps(), 1_000_000);
    assert_eq!(budget.max_module_graph_nodes(), 50_000);
    assert_eq!(budget.max_module_graph_edges(), 200_000);
    assert_eq!(budget.max_module_steps(), 1_000_000);
    assert_eq!(budget.max_authority_bytes(), 16_777_216);
    assert_eq!(budget.max_authority_string_bytes(), 65_536);
    assert_eq!(budget.max_rule_graph_nodes(), 50_000);
    assert_eq!(budget.max_rule_graph_edges(), 200_000);
    assert_eq!(budget.max_rule_steps(), 1_000_000);
    assert_eq!(budget.max_witness_depth(), 1_024);
    assert_eq!(budget.max_witnesses(), 10_000);
}
