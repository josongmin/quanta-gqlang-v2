#![allow(clippy::unwrap_used, reason = "budget rails inspect exact results")]

use quanta_gqlang_v2_hir::SourceIdV2;
use quanta_gqlang_v2_surface::{
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
    ParseBudgetV2::try_new(tokens, items, nodes, depth, width, projection).unwrap()
}

fn exhausted(source: &str, budget: &ParseBudgetV2, dimension: ParseBudgetDimensionV2) {
    let error = parse_source_document_v2(SourceIdV2::from_raw(7), source, budget).unwrap_err();
    assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
    let resource = error.resource.unwrap();
    assert_eq!(resource.dimension, dimension);
    assert_eq!(resource.observed, resource.limit + 1);
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
    let at_limit = format!("rule r : x if {}true", "not ".repeat(63));
    parse_source_document_v2(
        SourceIdV2::from_raw(7),
        &at_limit,
        &ParseBudgetV2::standard_v2(),
    )
    .unwrap();

    let over = format!("rule r : x if {}true", "not ".repeat(64));
    exhausted(
        &over,
        &ParseBudgetV2::standard_v2(),
        ParseBudgetDimensionV2::ExpressionDepth,
    );
}
