#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{
    check_tree, HirBuilderV1, HirRootRefV1, RelationExprKindV1, ResultShapeKindV1, ValueExprKindV1,
};
use quanta_gqlang_v2_surface::{parse_program, ParseErrorCodeV1, SurfaceItemV1};

#[test]
fn empty_result_shape_record_is_rejected() {
    // D10 fail-closed: an empty result-shape record `{}` carries no output
    // fields and must reject explicitly at the brace span rather than silently
    // compiling to a zero-field shape, mirroring the projection/value-key
    // "at least one item" requirement.
    let src = "from users => {}";
    let builder = HirBuilderV1::new();
    let err = parse_program(src, &builder).expect_err("empty result shape rejects");
    assert_eq!(err.code, ParseErrorCodeV1::UnsupportedConstruct);
    // span points at the empty `{}` braces (bytes 14..16).
    assert_eq!(err.span.start, 14);
    assert_eq!(err.span.end, 16);
}

#[test]
fn duplicate_result_shape_field_is_rejected() {
    // D10 fail-closed: a result-shape record must not declare the same field
    // name twice. A silent last-wins would drop an output field, so the parser
    // rejects at the offending (second) field name's exact span.
    let src = "from users => { a: 1, a: 2 }";
    let builder = HirBuilderV1::new();
    let err = parse_program(src, &builder).expect_err("duplicate shape field rejects");
    assert_eq!(err.code, ParseErrorCodeV1::UnsupportedConstruct);
    assert_eq!(err.code_str(), "surface.parse.unsupported_construct");
    // second `a` starts at byte 22 in `from users => { a: 1, a: 2 }`.
    assert_eq!(err.span.start, 22);
    assert_eq!(err.span.end, 23);
    assert!(
        err.message.contains("duplicate"),
        "message: {}",
        err.message
    );
}

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
    assert_eq!(project.items().len(), 2);
    assert_eq!(project.items()[0].alias.as_ref().unwrap().name, "id");
    assert_eq!(project.items()[1].alias.as_ref().unwrap().name, "name");

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
    let join = match &query.body.kind {
        RelationExprKindV1::Join(j) => j,
        other => panic!("outer should be Join, got {other:?}"),
    };
    let quanta_gqlang_v2_hir::JoinHirV1::Inner { left, .. } = join else {
        panic!("inner join")
    };
    assert!(matches!(left.kind, RelationExprKindV1::Filter(_)));
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
    let group = match &query.body.kind {
        RelationExprKindV1::GroupBy(group) => group,
        other => panic!("outer should be GroupBy, got {other:?}"),
    };
    // the aggregate call lives in the inner projection.
    let project = match &group.input.kind {
        RelationExprKindV1::Project(p) => p,
        other => panic!("group input should be Project, got {other:?}"),
    };
    assert_eq!(project.items().len(), 2);
    assert!(matches!(
        project.items()[1].value.kind,
        ValueExprKindV1::Call(_)
    ));
    check_tree(HirRootRefV1::Query(&query), src.len()).expect("sound tree");
}

#[test]
fn relation_pipeline_stage_set_is_closed() {
    // The relation pipeline accepts exactly `where | project | join | cross |
    // group | order` (`surface/src/query.rs::parse_stage`). The retired
    // open-tagged operators must NOT parse: silently accepting `union`,
    // `limit`, `sort`, or `apply` would reopen capability the closed HIR can no
    // longer represent, and their algebra handlers are already unreachable from
    // public HIR (see algebra/tests/RETIRED_LAWS.md for the disposition).
    let builder = HirBuilderV1::new();

    // STAGE-01: the closed six-token vocabulary owns both parsing and the
    // diagnostic. Every retired stage asserts the COMPLETE expected message.
    const CLOSED_STAGE_MESSAGE_V1: &str =
        "expected a pipeline stage keyword (where, project, join, cross, group, order)";
    for removed in ["union", "limit", "sort", "apply"] {
        let src = format!("from users | {removed} 1");
        let err =
            parse_program(&src, &builder).expect_err("a retired relation operator must not parse");
        assert_eq!(err.code, ParseErrorCodeV1::UnexpectedToken);
        assert_eq!(err.message, CLOSED_STAGE_MESSAGE_V1, "`{removed}` drift");
    }

    // The closed stage set itself still parses: one positive per stage.
    for stage in [
        "from findings f | where f.score > 1",
        "from findings | project repo, count(*) as n",
        "from a | join b on a = b",
        "from a | cross join b",
        "from a | group by k",
        "from a | order by k desc",
    ] {
        parse_program(stage, &builder).expect(stage);
    }
}
