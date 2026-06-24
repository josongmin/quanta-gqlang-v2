//! v3 rail: query grammar (syntax -> hir relation/query/result-shape).
//!
//! syntax->sema->ir path pinned: surface `from ... | project ... => {shape}`
//! text -> `parse_program` -> `hir::QueryHirV1` embedding `RelationExprHirV1`
//! (project with aliases) and a nested `ResultShapeHirV1`. Asserts the relation
//! producer structure, alias capture, nested record/list shape, and exact spans
//! (hellgate rail 2).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    clippy::match_wildcard_for_single_variants,
    reason = "integration test rail: assertion panics and exhaustive match arms are intentional"
)]

use quanta_gqlang_v2_hir::{
    check_tree, HirBuilderV1, HirRootRefV1, RelationExprKindV1, ResultShapeKindV1, ValueExprKindV1,
};
use quanta_gqlang_v2_surface::{parse_program, SurfaceItemV1};

#[test]
fn aliased_projection_with_nested_shape() {
    // alias `u` on the source; aliased projection items; nested result shape
    // with a record holding a list.
    let src = "from users u | project u.id as id, u.name as name => { id: u.id, tags: [ u.t1 ] }";
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).expect("parses");

    let query = match parsed.item {
        SurfaceItemV1::Query(q) => q,
        other => panic!("expected query, got {other:?}"),
    };

    // body is Project over a Source aliased `u`.
    let project = match &query.body.kind {
        RelationExprKindV1::Project(p) => p,
        other => panic!("body should be Project, got {other:?}"),
    };
    match &project.input.kind {
        RelationExprKindV1::Source(s) => {
            assert_eq!(s.name, "users");
            let alias = s.alias.as_ref().expect("source alias");
            assert_eq!(alias.name, "u");
            // `u` alias span: byte 11..12 (after "from users ").
            assert_eq!(alias.span.start, 11);
            assert_eq!(alias.span.end, 12);
        }
        other => panic!("input should be Source, got {other:?}"),
    }

    // two projection items, each aliased.
    assert_eq!(project.items.len(), 2);
    assert_eq!(project.items[0].alias.as_ref().unwrap().name, "id");
    assert_eq!(project.items[1].alias.as_ref().unwrap().name, "name");

    // nested result shape: record { id: leaf, tags: [ leaf ] }.
    let shape = query.result_shape.as_ref().expect("result shape present");
    let record = match &shape.kind {
        ResultShapeKindV1::Record(r) => r,
        other => panic!("shape should be record, got {other:?}"),
    };
    assert_eq!(record.fields.len(), 2);
    assert_eq!(record.fields[0].name, "id");
    assert!(matches!(
        record.fields[0].shape.kind,
        ResultShapeKindV1::Leaf(_)
    ));
    assert_eq!(record.fields[1].name, "tags");
    let list = match &record.fields[1].shape.kind {
        ResultShapeKindV1::List(l) => l,
        other => panic!("tags should be a list shape, got {other:?}"),
    };
    assert!(matches!(list.element.kind, ResultShapeKindV1::Leaf(_)));

    // whole query tree is integrity-sound.
    check_tree(HirRootRefV1::Query(&query), src.len()).expect("sound tree");
}

#[test]
fn where_filter_and_join_compose() {
    let src = "from findings f | where f.score > 1 | join repos r on f.repo = r.id";
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).expect("parses");
    let query = match parsed.item {
        SurfaceItemV1::Query(q) => q,
        other => panic!("expected query, got {other:?}"),
    };
    // outermost is the `on` filter wrapping the join compose.
    let filter = match &query.body.kind {
        RelationExprKindV1::Filter(f) => f,
        other => panic!("outer should be join-on Filter, got {other:?}"),
    };
    let compose = match &filter.input.kind {
        RelationExprKindV1::Compose(c) => c,
        other => panic!("filter input should be Compose, got {other:?}"),
    };
    assert_eq!(compose.op.symbol, "join");
    assert_eq!(compose.operands.len(), 2);
    // left operand is itself the `where` filter over the aliased source.
    assert!(matches!(
        compose.operands[0].kind,
        RelationExprKindV1::Filter(_)
    ));
    check_tree(HirRootRefV1::Query(&query), src.len()).expect("sound tree");
}

#[test]
fn group_by_with_aggregate_call_projection() {
    // count(*) aggregate-style call in a projection; group by a key.
    let src = "from findings | project repo, count(*) as n | group by repo";
    let builder = HirBuilderV1::new();
    let parsed = parse_program(src, &builder).expect("parses");
    let query = match parsed.item {
        SurfaceItemV1::Query(q) => q,
        other => panic!("expected query, got {other:?}"),
    };
    let compose = match &query.body.kind {
        RelationExprKindV1::Compose(c) => c,
        other => panic!("outer should be group_by Compose, got {other:?}"),
    };
    assert_eq!(compose.op.symbol, "group_by");
    // the aggregate call lives in the inner projection.
    let project = match &compose.operands[0].kind {
        RelationExprKindV1::Project(p) => p,
        other => panic!("group input should be Project, got {other:?}"),
    };
    assert_eq!(project.items.len(), 2);
    assert!(matches!(
        project.items[1].value.kind,
        ValueExprKindV1::Call(_)
    ));
    check_tree(HirRootRefV1::Query(&query), src.len()).expect("sound tree");
}
