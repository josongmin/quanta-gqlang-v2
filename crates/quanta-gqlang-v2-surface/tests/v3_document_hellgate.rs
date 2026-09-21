#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{
    check_tree, HirBuilderV1, HirRootRefV1, ProgramItemHirV1, RelationExprKindV1, SpanV1,
};
use quanta_gqlang_v2_surface::{parse_document, ParseErrorCodeV1, SurfaceItemV1};
use std::collections::HashSet;

#[test]
fn three_ordered_items_with_unique_ids() {
    // VIEW decl, then FIND query, then a multi-line composition: 3 ordered items.
    let src = "view catalog.users | query-ref=queries.catalog.users ;\n\
               find taint api.entry -> db.write | depth=3 ;\n\
               let suspects = sample query=\"auth\" | mode=semantic | limit=3\n\
               find taint api.entry -> db.write with suspects";
    let builder = HirBuilderV1::new();
    let doc = parse_document(src, &builder).expect("must parse document");
    assert_eq!(doc.items.len(), 3, "expected 3 ordered items");

    let items: Vec<ProgramItemHirV1> = doc
        .items
        .into_iter()
        .map(SurfaceItemV1::into_program_item)
        .collect();
    // order is preserved: CatalogDecl, GraphQuery, Composition.
    assert!(matches!(items[0], ProgramItemHirV1::CatalogDecl(_)));
    assert!(matches!(items[1], ProgramItemHirV1::GraphQuery(_)));
    assert!(matches!(items[2], ProgramItemHirV1::Composition(_)));

    // ids globally unique across the whole document (one builder).
    let mut ids = HashSet::new();
    for item in &items {
        assert!(ids.insert(item.id()), "duplicate node id across document");
        // every item still passes shared HIR integrity against the doc length.
        check_tree(HirRootRefV1::Item(item), doc.source_len).expect("tree integrity");
    }
}

#[test]
fn composition_groups_as_one_item() {
    // A bare composition with no `;` is exactly ONE document item even though it
    // spans several lines.
    let src = "let a = from users u\n\
               find taint p -> q with a";
    let builder = HirBuilderV1::new();
    let doc = parse_document(src, &builder).expect("must parse");
    assert_eq!(doc.items.len(), 1);
    assert!(matches!(
        doc.items.into_iter().next().unwrap().into_program_item(),
        ProgramItemHirV1::Composition(_)
    ));
}

#[test]
fn malformed_middle_item_rejects_with_exact_span() {
    // The middle FIND has no `->` between source and sink: fail closed.
    let prefix = "show capabilities ;\n";
    let bad = "find taint api.entry db.write";
    let src = format!("{prefix}{bad} ;\nshow models");
    let builder = HirBuilderV1::new();
    let diag = parse_document(&src, &builder).expect_err("must reject");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
    // span points inside the offending (middle) item, after the first `;`.
    assert!(
        diag.span.start >= prefix.len(),
        "span {} should be within the middle item (>= {})",
        diag.span.start,
        prefix.len()
    );
}

#[test]
fn empty_statement_rejects() {
    // a doubled separator yields an empty statement -> fail closed.
    let builder = HirBuilderV1::new();
    let diag = parse_document("show models ;; show views", &builder).expect_err("must reject");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
}

#[test]
fn leading_separator_rejects() {
    let builder = HirBuilderV1::new();
    let diag = parse_document(" ; show models", &builder).expect_err("must reject");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
}

#[test]
fn trailing_separator_is_tolerated() {
    let builder = HirBuilderV1::new();
    let doc = parse_document("show models ;", &builder).expect("trailing ; is allowed");
    assert_eq!(doc.items.len(), 1);
}

#[test]
fn missing_separator_between_items_rejects() {
    // two items with no `;` between them: trailing garbage on the first item.
    let builder = HirBuilderV1::new();
    let diag = parse_document("show models show views", &builder).expect_err("must reject");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
}

#[test]
fn single_item_document_matches_program() {
    let builder = HirBuilderV1::new();
    let doc = parse_document("from users u | where u.active", &builder).expect("parse");
    assert_eq!(doc.items.len(), 1);
    assert!(matches!(
        doc.items.into_iter().next().unwrap().into_program_item(),
        ProgramItemHirV1::Query(_)
    ));
}

#[test]
fn relation_source_name_span_is_exact_after_multibyte_prefix() {
    let src = "let title = \"한글\"; from users u";
    let builder = HirBuilderV1::new();
    let document = parse_document(src, &builder).expect("multibyte declaration and query parse");
    assert_eq!(document.items.len(), 2);
    let item = document
        .items
        .into_iter()
        .nth(1)
        .expect("query item")
        .into_program_item();
    let ProgramItemHirV1::Query(query) = &item else {
        panic!("expected query")
    };
    let RelationExprKindV1::Source(source) = &query.body.kind else {
        panic!("expected source")
    };
    let start = src.find("users").expect("source name");
    assert_eq!(source.name_span, SpanV1::new(start, start + "users".len()));
    assert_eq!(
        src.get(source.name_span.start..source.name_span.end),
        Some("users")
    );
    assert_eq!(query.body.span.end, src.len());
    assert!(
        source.name_span.end < query.body.span.end,
        "source name excludes alias"
    );
    check_tree(HirRootRefV1::Item(&item), document.source_len)
        .expect("exact byte spans stay within source");
}
