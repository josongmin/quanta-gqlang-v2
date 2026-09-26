#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{HirBuilderV1, SourceIdV2, SpanV1};
use quanta_gqlang_v2_surface::{
    format_source_document_with_parse_budget_v2, parse_expr, parse_predicate,
    parse_source_document_v2, ParseBudgetDimensionV2, ParseBudgetV2, ParseErrorCodeV1,
    SurfaceDiagnosticV2,
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

#[test]
fn formatter_checks_canonical_output_at_the_exact_token_boundary() {
    let source = "from t | order by a";
    let expected = "from t | order by a asc";
    let source_id = SourceIdV2::from_raw(91);
    let exact = budget(8, 1, 16, 8, 8, 8);
    let formatted = format_source_document_with_parse_budget_v2(source_id, source, &exact).unwrap();
    assert_eq!(formatted, expected);
    assert_eq!(
        format_source_document_with_parse_budget_v2(source_id, &formatted, &exact).unwrap(),
        expected
    );
    let limited = budget(7, 1, 16, 8, 8, 8);
    parse_source_document_v2(source_id, source, &limited).unwrap();
    let failure =
        format_source_document_with_parse_budget_v2(source_id, source, &limited).unwrap_err();
    assert_eq!(failure.source_id, source_id);
    assert_eq!(failure.code, ParseErrorCodeV1::ResourceExhausted);
    assert_eq!(failure.span, SpanV1::new(0, source.len()));
    assert_eq!(
        failure.message,
        "canonical output: tokens budget exhausted: limit 7, observed 8"
    );
    let resource = failure.resource.unwrap();
    assert_eq!(resource.dimension, ParseBudgetDimensionV2::Tokens);
    assert_eq!((resource.limit, resource.observed), (7, 8));
}

fn exhausted(
    source: &str,
    budget: &ParseBudgetV2,
    dimension: ParseBudgetDimensionV2,
    limit: usize,
    span: (usize, usize),
) -> SurfaceDiagnosticV2 {
    let error = parse_source_document_v2(SourceIdV2::from_raw(7), source, budget).unwrap_err();
    assert_eq!(error.source_id, SourceIdV2::from_raw(7));
    assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
    assert_eq!(error.span, SpanV1::new(span.0, span.1));
    let resource = error.resource.as_ref().unwrap();
    assert_eq!(resource.dimension, dimension);
    assert_eq!((resource.limit, resource.observed), (limit, limit + 1));
    assert_eq!(error.code.code(), "surface.resource_exhausted");
    assert_eq!(error.code_str(), "surface.resource_exhausted");
    assert_eq!(
        error.message,
        format!(
            "{} budget exhausted: limit {}, observed {}",
            dimension.as_str(),
            limit,
            limit + 1
        )
    );
    error
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
        4,
        (9, 9),
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
        1,
        (11, 14),
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
        1,
        (12, 13),
    );

    let query = "from t | group by a, b";
    parse_source_document_v2(SourceIdV2::from_raw(7), query, &budget(12, 1, 16, 8, 2, 2)).unwrap();
    exhausted(
        query,
        &budget(12, 1, 16, 8, 2, 1),
        ParseBudgetDimensionV2::ProjectionWidth,
        1,
        (21, 22),
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
        2,
        (8, 13),
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
        64,
        (266, 270),
    );
}

#[test]
fn flat_expression_tree_depth_is_bounded_before_formatter_or_semantic_traversal() {
    let standard = ParseBudgetV2::standard_v2();
    for (prefix, operator, exact_count, span) in [
        ("from t | where true", " and true", 62, (15, 19)),
        ("let x = 1", " + 1", 63, (8, 9)),
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
        let expected = exhausted(
            &over,
            &standard,
            ParseBudgetDimensionV2::ExpressionDepth,
            64,
            span,
        );
        let error =
            format_source_document_with_parse_budget_v2(SourceIdV2::from_raw(7), &over, &standard)
                .unwrap_err();
        assert_eq!(
            error, expected,
            "formatter ingress preserves the full diagnostic"
        );
    }

    let bool_error = parse_predicate(
        &format!("true{}", " and true".repeat(63)),
        &HirBuilderV1::new(),
    )
    .unwrap_err();
    assert_eq!(bool_error.code, ParseErrorCodeV1::ResourceExhausted);
    assert_eq!(bool_error.span, SpanV1::new(0, 4));
    assert_eq!(
        bool_error.message,
        "expression_depth budget exhausted: limit 64, observed 65"
    );
    let resource = bool_error.resource.unwrap();
    assert_eq!(resource.dimension, ParseBudgetDimensionV2::ExpressionDepth);
    assert_eq!((resource.limit, resource.observed), (64, 65));
    let value_error =
        parse_expr(&format!("1{}", " + 1".repeat(64)), &HirBuilderV1::new()).unwrap_err();
    assert_eq!(value_error.code, ParseErrorCodeV1::ResourceExhausted);
    assert_eq!(value_error.span, SpanV1::new(0, 1));
    assert_eq!(
        value_error.message,
        "expression_depth budget exhausted: limit 64, observed 65"
    );
    let resource = value_error.resource.unwrap();
    assert_eq!(resource.dimension, ParseBudgetDimensionV2::ExpressionDepth);
    assert_eq!((resource.limit, resource.observed), (64, 65));
}

#[test]
fn recursive_depth_rejection_pins_the_first_over_limit_token() {
    // Each pair differs by one wrapper. Shape leaves also enter the value
    // parser; pin the first token reaching descent 5, independently of HIR ids.
    let narrow = budget(100, 1, 100, 4, 10, 10);
    for (accepted, rejected, span) in [
        ("let x = ---1", "let x = ----1", (11, 12)),
        ("let x = (((1)))", "let x = ((((1))))", (12, 13)),
        ("from t => [[1]]", "from t => [[[1]]]", (13, 14)),
        (
            "rule r : x if not not true",
            "rule r : x if not not not true",
            (26, 30),
        ),
    ] {
        parse_source_document_v2(SourceIdV2::from_raw(7), accepted, &narrow).unwrap();
        exhausted(
            rejected,
            &narrow,
            ParseBudgetDimensionV2::ExpressionDepth,
            4,
            span,
        );
    }
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
    let last_pipe = over.rfind('|').unwrap();
    exhausted(
        &over,
        &ParseBudgetV2::standard_v2(),
        ParseBudgetDimensionV2::RelationDepth,
        128,
        (last_pipe, last_pipe + 1),
    );
}

#[test]
fn relation_budget_is_shared_across_nested_exists_and_caps_custom_profiles() {
    let allowed = ParseBudgetV2::try_new(100, 1, 100, 64, 4, 10, 10).unwrap();
    let source = "from t | where exists (from u | where true)";
    parse_source_document_v2(SourceIdV2::from_raw(7), source, &allowed).unwrap();

    let narrow = ParseBudgetV2::try_new(100, 1, 100, 64, 3, 10, 10).unwrap();
    exhausted(
        source,
        &narrow,
        ParseBudgetDimensionV2::RelationDepth,
        3,
        (30, 31),
    );

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

#[test]
fn every_collection_carrier_rejects_one_over_width_at_the_authored_element() {
    // Each source authors exactly two elements in the target collection.
    // Other dimensions are deliberately generous; a rejection must identify
    // the second element of this collection, not an unrelated depth/token cap.
    let cases = [
        ("list", "let x = [1, 2]", "2"),
        ("record", "let x = { first: 1, second: 2 }", "second"),
        ("call", "let x = f(1, 2)", "2"),
        ("path", "let x = root.first.second", "second"),
        (
            "quantifier",
            "rule r : 1 if forall first, second in (true)",
            "second",
        ),
        (
            "result shape",
            "from users => { first: 1, second: 2 }",
            "second",
        ),
        (
            "composition",
            "let first = from users let second = from users from users with first",
            "second",
        ),
        (
            "graph via",
            "find taint a -> b | via=first,second",
            "second",
        ),
        (
            "graph through",
            "find taint a -> b | through=first,second",
            "second",
        ),
        (
            "graph sanitizers",
            "find taint a -> b | sanitizers=first,second",
            "second",
        ),
        (
            "profile capabilities",
            "profile p | capabilities=first,second | admission=allowlist",
            "second",
        ),
        (
            "gate checks",
            "gate g | checks=first,second | binding=all",
            "second",
        ),
        (
            "summary facets",
            "find summary | facets=first,second",
            "second",
        ),
        (
            "projection collection",
            "from users | project first, second",
            "second",
        ),
        (
            "group collection",
            "from users | group by first, second",
            "second",
        ),
        (
            "order collection",
            "from users | order by first, second",
            "second",
        ),
    ];
    assert_eq!(cases.len(), 16);
    let source_id = SourceIdV2::from_raw(73);
    for (carrier, source, second_element) in cases {
        let exact = ParseBudgetV2::try_new(256, 8, 128, 64, 128, 2, 8).unwrap();
        let parsed = parse_source_document_v2(source_id, source, &exact)
            .unwrap_or_else(|error| panic!("{carrier}: exact width must admit: {error:?}"));
        assert_eq!(parsed.hir.items.len(), 1, "{carrier}");
        let narrow = ParseBudgetV2::try_new(256, 8, 128, 64, 128, 1, 8).unwrap();
        let error = parse_source_document_v2(source_id, source, &narrow).unwrap_err();
        assert_eq!(error.source_id, source_id, "{carrier}");
        assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted, "{carrier}");
        let resource = error.resource.unwrap();
        assert_eq!(
            resource.dimension,
            ParseBudgetDimensionV2::CollectionWidth,
            "{carrier}"
        );
        assert_eq!((resource.limit, resource.observed), (1, 2), "{carrier}");
        let start = source.rfind(second_element).unwrap();
        assert_eq!(
            error.span,
            quanta_gqlang_v2_hir::SpanV1::new(start, start + second_element.len()),
            "{carrier}"
        );
        assert_eq!(
            error.message, "collection_width budget exhausted: limit 1, observed 2",
            "{carrier}"
        );
    }
}

#[test]
fn every_projection_carrier_has_an_independent_exact_width_limit() {
    for (carrier, source) in [
        ("projection", "from users | project first, second"),
        ("group", "from users | group by first, second"),
        ("order", "from users | order by first, second"),
    ] {
        let exact = ParseBudgetV2::try_new(256, 8, 128, 64, 128, 8, 2).unwrap();
        parse_source_document_v2(SourceIdV2::from_raw(73), source, &exact).unwrap();
        let narrow = ParseBudgetV2::try_new(256, 8, 128, 64, 128, 8, 1).unwrap();
        let error =
            parse_source_document_v2(SourceIdV2::from_raw(73), source, &narrow).unwrap_err();
        assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted, "{carrier}");
        let resource = error.resource.unwrap();
        assert_eq!(
            resource.dimension,
            ParseBudgetDimensionV2::ProjectionWidth,
            "{carrier}"
        );
        assert_eq!((resource.limit, resource.observed), (1, 2), "{carrier}");
        let start = source.rfind("second").unwrap();
        assert_eq!(
            error.span,
            quanta_gqlang_v2_hir::SpanV1::new(start, start + 6),
            "{carrier}"
        );
    }
}

#[test]
fn expression_node_budget_is_not_reset_between_document_roots() {
    // First declaration: one integer node. Second: two integer nodes + Add.
    let source = "let x = 1; let y = 2 + 3";
    let exact = ParseBudgetV2::try_new(256, 8, 4, 64, 128, 8, 8).unwrap();
    let parsed = parse_source_document_v2(SourceIdV2::from_raw(73), source, &exact).unwrap();
    assert_eq!(parsed.hir.items.len(), 2);
    let narrow = ParseBudgetV2::try_new(256, 8, 3, 64, 128, 8, 8).unwrap();
    let error = parse_source_document_v2(SourceIdV2::from_raw(73), source, &narrow).unwrap_err();
    assert_eq!(error.source_id, SourceIdV2::from_raw(73));
    assert_eq!(error.code, ParseErrorCodeV1::ResourceExhausted);
    let resource = error.resource.unwrap();
    assert_eq!(resource.dimension, ParseBudgetDimensionV2::ExpressionNodes);
    assert_eq!((resource.limit, resource.observed), (3, 4));
    assert_eq!(error.span, quanta_gqlang_v2_hir::SpanV1::new(19, 24));
    assert_eq!(
        error.message,
        "expression_nodes budget exhausted: limit 3, observed 4"
    );
}
