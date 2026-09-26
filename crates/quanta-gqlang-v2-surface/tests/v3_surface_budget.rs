#![expect(
    clippy::unwrap_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{HirBuilderV1, SourceIdV2};
use quanta_gqlang_v2_surface::{
    format_source_document_with_parse_budget_v2, parse_expr, parse_predicate,
    parse_source_document_v2, ParseBudgetDimensionV2, ParseBudgetV2, ParseErrorCodeV1,
};

fn budget(
    tokens: usize,
    items: usize,
    nodes: usize,
    depth: usize,
    width: usize,
    projection: usize,
) -> ParseBudgetV2 {
    ParseBudgetV2::try_new(tokens, items, nodes, depth, 128, width, projection).unwrap()
}

#[test]
fn formatter_v2_preserves_source_identity_on_failure() {
    let error = format_source_document_with_parse_budget_v2(
        SourceIdV2::from_raw(91),
        "let x = @",
        &ParseBudgetV2::standard_v2(),
    )
    .unwrap_err();
    assert_eq!(error.source_id, SourceIdV2::from_raw(91));
    assert_eq!(error.code, ParseErrorCodeV1::UnexpectedChar);
}

fn exhausted(source: &str, budget: &ParseBudgetV2, dimension: ParseBudgetDimensionV2) {
    let error = parse_source_document_v2(SourceIdV2::from_raw(7), source, budget).unwrap_err();
    assert_eq!(error.source_id, SourceIdV2::from_raw(7));
    assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
    let resource = error.resource.unwrap();
    assert_eq!(resource.dimension, dimension);
    assert_eq!(resource.observed, resource.limit + 1);
    assert_eq!(error.code.code(), "surface.resource_exhausted");
    assert_eq!(
        error.message,
        format!(
            "{} budget exhausted: limit {}, observed {}",
            dimension.as_str(),
            resource.limit,
            resource.observed
        )
    );
}

#[test]
fn token_limit_includes_eof_and_rejects_before_append() {
    parse_source_document_v2(
        SourceIdV2::from_raw(7),
        "let x = 1",
        &budget(5, 1, 4, 8, 8, 8),
    )
    .unwrap();
    exhausted(
        "let x = 1",
        &budget(4, 1, 4, 8, 8, 8),
        ParseBudgetDimensionV2::Tokens,
    );
}

#[test]
fn document_item_counter_is_shared_across_roots() {
    parse_source_document_v2(
        SourceIdV2::from_raw(7),
        "let x = 1; let y = 2",
        &budget(10, 2, 8, 8, 8, 8),
    )
    .unwrap();
    exhausted(
        "let x = 1; let y = 2",
        &budget(10, 1, 8, 8, 8, 8),
        ParseBudgetDimensionV2::DocumentItems,
    );
}

#[test]
fn collection_and_projection_limits_are_independent() {
    parse_source_document_v2(
        SourceIdV2::from_raw(7),
        "let x = [1, 2]",
        &budget(10, 1, 8, 8, 2, 2),
    )
    .unwrap();
    exhausted(
        "let x = [1, 2]",
        &budget(10, 1, 8, 8, 1, 2),
        ParseBudgetDimensionV2::CollectionWidth,
    );

    let query = "from t | group by a, b";
    parse_source_document_v2(SourceIdV2::from_raw(7), query, &budget(12, 1, 16, 8, 2, 2)).unwrap();
    exhausted(
        query,
        &budget(12, 1, 16, 8, 2, 1),
        ParseBudgetDimensionV2::ProjectionWidth,
    );
}

#[test]
fn expression_node_limit_is_cumulative() {
    parse_source_document_v2(
        SourceIdV2::from_raw(7),
        "let x = 1 + 2",
        &budget(8, 1, 3, 8, 8, 8),
    )
    .unwrap();
    exhausted(
        "let x = 1 + 2",
        &budget(8, 1, 2, 8, 8, 8),
        ParseBudgetDimensionV2::ExpressionNodes,
    );
}

#[test]
fn parsed_document_preserves_caller_source_identity() {
    let parsed = parse_source_document_v2(
        SourceIdV2::from_raw(91),
        "let x = 1",
        &budget(5, 1, 4, 8, 8, 8),
    )
    .unwrap();
    assert_eq!(parsed.hir.source_id, SourceIdV2::from_raw(91));
    assert_eq!(parsed.hir.source_len, "let x = 1".len());
}

#[test]
fn standard_depth_64_succeeds_and_65_rejects() {
    let at_limit = format!("rule r : x if {}true", "not ".repeat(62));
    parse_source_document_v2(
        SourceIdV2::from_raw(7),
        &at_limit,
        &ParseBudgetV2::standard_v2(),
    )
    .unwrap();
    parse_source_document_v2(
        SourceIdV2::from_raw(7),
        &format!("{at_limit}; {at_limit}"),
        &ParseBudgetV2::standard_v2(),
    )
    .unwrap();

    let over = format!("rule r : x if {}true", "not ".repeat(63));
    exhausted(
        &over,
        &ParseBudgetV2::standard_v2(),
        ParseBudgetDimensionV2::ExpressionDepth,
    );
}

#[test]
fn flat_expression_tree_depth_is_bounded_before_formatter_or_semantic_traversal() {
    let standard = ParseBudgetV2::standard_v2();
    for (prefix, operator, exact_count) in [
        ("from t | where true", " and true", 62),
        ("let x = 1", " + 1", 63),
    ] {
        let exact = format!("{prefix}{}", operator.repeat(exact_count));
        parse_source_document_v2(SourceIdV2::from_raw(7), &exact, &standard).unwrap();
        assert_eq!(
            format_source_document_with_parse_budget_v2(
                SourceIdV2::from_raw(7),
                &exact,
                &standard,
            )
            .unwrap(),
            exact
        );

        let over = format!("{prefix}{}", operator.repeat(exact_count + 1));
        exhausted(&over, &standard, ParseBudgetDimensionV2::ExpressionDepth);
        let error =
            format_source_document_with_parse_budget_v2(SourceIdV2::from_raw(7), &over, &standard)
                .unwrap_err();
        assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
        assert_eq!(
            error.resource.unwrap().dimension,
            ParseBudgetDimensionV2::ExpressionDepth
        );
    }

    let bool_error = parse_predicate(
        &format!("true{}", " and true".repeat(63)),
        &HirBuilderV1::new(),
    )
    .unwrap_err();
    assert_eq!(bool_error.code, ParseErrorCodeV1::ResourceExhausted);
    let value_error =
        parse_expr(&format!("1{}", " + 1".repeat(64)), &HirBuilderV1::new()).unwrap_err();
    assert_eq!(value_error.code, ParseErrorCodeV1::ResourceExhausted);
}

#[test]
fn relation_stage_depth_rejects_before_hir_integrity_traversal() {
    let at_limit = format!("from t{}", " | where true".repeat(127));
    parse_source_document_v2(
        SourceIdV2::from_raw(7),
        &at_limit,
        &ParseBudgetV2::standard_v2(),
    )
    .unwrap();

    let over = format!("from t{}", " | where true".repeat(128));
    let error = parse_source_document_v2(
        SourceIdV2::from_raw(7),
        &over,
        &ParseBudgetV2::standard_v2(),
    )
    .unwrap_err();
    assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
    let resource = error.resource.unwrap();
    assert_eq!(resource.dimension.as_str(), "relation_depth");
    assert_eq!((resource.limit, resource.observed), (128, 129));
    let last_pipe = over.rfind('|').unwrap();
    assert_eq!(
        error.span,
        quanta_gqlang_v2_hir::SpanV1::new(last_pipe, last_pipe + 1)
    );
}

#[test]
fn relation_budget_is_shared_across_nested_exists_and_caps_custom_profiles() {
    let allowed = ParseBudgetV2::try_new(100, 1, 100, 64, 4, 10, 10).unwrap();
    let source = "from t | where exists (from u | where true)";
    parse_source_document_v2(SourceIdV2::from_raw(7), source, &allowed).unwrap();

    let narrow = ParseBudgetV2::try_new(100, 1, 100, 64, 3, 10, 10).unwrap();
    let error = parse_source_document_v2(SourceIdV2::from_raw(7), source, &narrow).unwrap_err();
    assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
    let resource = error.resource.unwrap();
    assert_eq!(resource.dimension, ParseBudgetDimensionV2::RelationDepth);
    assert_eq!((resource.limit, resource.observed), (3, 4));

    assert!(ParseBudgetV2::try_new(100, 1, 100, 65, 128, 10, 10).is_err());
    assert!(ParseBudgetV2::try_new(100, 1, 100, 64, 129, 10, 10).is_err());
}

#[test]
fn every_budget_dimension_rejects_zero() {
    // A zero in any one dimension must be rejected; a dropped array entry would
    // silently accept a budget that can never admit anything.
    for zero_index in 0..7usize {
        let args: [usize; 7] = std::array::from_fn(|i| usize::from(i != zero_index));
        assert!(
            ParseBudgetV2::try_new(args[0], args[1], args[2], args[3], args[4], args[5], args[6])
                .is_err(),
            "dimension {zero_index} must reject a zero budget"
        );
    }
}

#[test]
fn flat_sibling_width_restores_expression_depth_per_element() {
    // Each list element must charge and then restore the shared expression-depth
    // counter: 65 shallow siblings must not exhaust a 64-deep budget.
    let wide = format!("[{}]", vec!["1"; 65].join(","));
    parse_expr(&wide, &HirBuilderV1::new()).unwrap();
}
