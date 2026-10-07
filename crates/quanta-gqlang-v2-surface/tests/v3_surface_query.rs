#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{
    check_tree, BoolExprKindV1, CallArgHirV1, HirBuilderV1, HirRootRefV1, RelationExprKindV1,
    ResultShapeKindV1, SortDirectionHirV1, SpanV1, ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_surface::{
    classify_parsed_root_v3, parse_program, ParseErrorCodeV1, SourceRootFamilyV3, SurfaceItemV1,
};

/// The `(base symbol, path segments)` of a path value; panics on any other node.
fn path_parts(v: &ValueExprHirV1) -> (&str, Vec<&str>) {
    let ValueExprKindV1::Path(p) = &v.kind else {
        panic!("expected path, got {:?}", v.kind);
    };
    let ValueExprKindV1::Symbol(base) = &p.base.kind else {
        panic!("expected symbol path base, got {:?}", p.base.kind);
    };
    (
        base.name.as_str(),
        p.segments.iter().map(|s| s.name.as_str()).collect(),
    )
}

/// The symbol name of a value node; panics on any other node.
fn symbol_name(v: &ValueExprHirV1) -> &str {
    let ValueExprKindV1::Symbol(s) = &v.kind else {
        panic!("expected symbol, got {:?}", v.kind);
    };
    s.name.as_str()
}

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

    // two projection items, each aliased, with the authored value paths retained.
    assert_eq!(project.items().len(), 2);
    assert_eq!(project.items()[0].alias.as_ref().unwrap().name, "id");
    assert_eq!(project.items()[1].alias.as_ref().unwrap().name, "name");
    assert_eq!(path_parts(&project.items()[0].value), ("u", vec!["id"]));
    assert_eq!(path_parts(&project.items()[1].value), ("u", vec!["name"]));

    // nested result shape: record { id: leaf, tags: [ leaf ] }.
    let shape = query.result_shape.as_ref().expect("result shape present");
    let record = match &shape.kind {
        ResultShapeKindV1::Record(r) => r,
        other => panic!("shape should be record, got {other:?}"),
    };
    assert_eq!(record.fields.len(), 2);
    assert_eq!(record.fields[0].name, "id");
    let ResultShapeKindV1::Leaf(leaf0) = &record.fields[0].shape.kind else {
        panic!(
            "id field must be a leaf, got {:?}",
            record.fields[0].shape.kind
        );
    };
    assert_eq!(path_parts(leaf0), ("u", vec!["id"]));
    assert_eq!(record.fields[1].name, "tags");
    let list = match &record.fields[1].shape.kind {
        ResultShapeKindV1::List(l) => l,
        other => panic!("tags should be a list shape, got {other:?}"),
    };
    let ResultShapeKindV1::Leaf(leaf1) = &list.element.kind else {
        panic!("tags element must be a leaf, got {:?}", list.element.kind);
    };
    assert_eq!(path_parts(leaf1), ("u", vec!["t1"]));
    let list_open = src.find('[').expect("list bracket in source");
    let list_close = src.rfind(']').expect("list close bracket in source");
    assert_eq!(
        record.fields[1].shape.span,
        SpanV1::new(list_open, list_close + 1)
    );

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
    let quanta_gqlang_v2_hir::JoinHirV1::Inner { left, right, on } = join else {
        panic!("inner join")
    };
    // Both join inputs and the ON predicate are part of the oracle: checking
    // only "left is a Filter" would accept a dropped right source or ON clause.
    let RelationExprKindV1::Filter(filter) = &left.kind else {
        panic!("join left must be the WHERE filter, got {:?}", left.kind);
    };
    let BoolExprKindV1::Compare(filter_compare) = &filter.predicate.kind else {
        panic!(
            "WHERE predicate must be a comparison, got {:?}",
            filter.predicate.kind
        );
    };
    assert_eq!(filter_compare.op.symbol, ">");
    let RelationExprKindV1::Source(source) = &right.kind else {
        panic!("join right must be a source, got {:?}", right.kind);
    };
    assert_eq!(source.name, "repos");
    assert_eq!(source.alias.as_ref().expect("join alias r").name, "r");
    let BoolExprKindV1::Compare(on_compare) = &on.kind else {
        panic!("join ON must be a comparison, got {:?}", on.kind);
    };
    assert_eq!(on_compare.op.symbol, "=");
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
    let ValueExprKindV1::Call(call) = &project.items()[1].value.kind else {
        panic!(
            "item 1 must be the aggregate call, got {:?}",
            project.items()[1].value.kind
        );
    };
    assert_eq!(call.callee.name, "count");
    assert_eq!(call.args().len(), 1);
    assert!(
        matches!(&call.args().as_slice()[0], CallArgHirV1::Star { .. }),
        "count aggregate must carry a star argument, got {:?}",
        call.args().as_slice()[0]
    );
    assert_eq!(
        project.items()[1]
            .alias
            .as_ref()
            .expect("aggregate alias n")
            .name,
        "n"
    );
    // The grouping key itself is part of the oracle, not just the aggregate.
    assert_eq!(group.keys().len(), 1);
    assert_eq!(symbol_name(&group.keys()[0]), "repo");
    check_tree(HirRootRefV1::Query(&query), src.len()).expect("sound tree");
}

#[test]
fn relation_pipeline_stage_set_is_closed() {
    // The relation pipeline accepts exactly `where | project | join | cross |
    // group | order` (`surface/src/query.rs::parse_stage`). The retired
    // open-tagged operators must NOT parse: silently accepting `union`,
    // `limit`, `sort`, or `apply` would reopen capability the closed HIR can no
    // longer represent, and their algebra handlers are already unreachable from
    // public HIR (see docs/adr/0002-test-authority-and-adversarial-oracles.md).
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

    // `order by ... desc` must retain the explicit direction, not default to asc.
    let ordered = parse_program("from a | order by k desc", &builder).expect("order parses");
    let SurfaceItemV1::Query(query) = &ordered.item else {
        panic!("order source must parse to a query, got {:?}", ordered.item);
    };
    let RelationExprKindV1::OrderBy(order_by) = &query.body.kind else {
        panic!("body must be OrderBy, got {:?}", query.body.kind);
    };
    assert_eq!(order_by.keys().len(), 1);
    assert_eq!(order_by.keys()[0].direction, SortDirectionHirV1::Desc);

    // An omitted direction must default to ascending, not descending.
    let default_order =
        parse_program("from a | order by k", &builder).expect("order default parses");
    let SurfaceItemV1::Query(query) = &default_order.item else {
        panic!(
            "order source must parse to a query, got {:?}",
            default_order.item
        );
    };
    let RelationExprKindV1::OrderBy(order_by) = &query.body.kind else {
        panic!("body must be OrderBy, got {:?}", query.body.kind);
    };
    assert_eq!(order_by.keys()[0].direction, SortDirectionHirV1::Asc);
}

#[test]
fn parsed_root_classification_matches_every_family() {
    // The parser-owner boundary classification must map each family exactly; a
    // swapped arm silently misroutes a root downstream.
    for (src, expected) in [
        ("from users", SourceRootFamilyV3::Relation),
        ("let x = 1", SourceRootFamilyV3::Declaration),
        ("rule r : x if a", SourceRootFamilyV3::Rule),
        ("find reachability a.b -> c.d", SourceRootFamilyV3::Graph),
        ("check r.x | confirm=skip", SourceRootFamilyV3::Check),
        ("show capabilities", SourceRootFamilyV3::Presentation),
        ("view v | query-ref=r.x", SourceRootFamilyV3::Catalog),
        (
            "use pack p | activation=session | lockfile=p.lock",
            SourceRootFamilyV3::Meta,
        ),
    ] {
        let parsed = parse_program(src, &HirBuilderV1::new()).expect(src);
        assert_eq!(
            classify_parsed_root_v3(&parsed.item.into_program_item()),
            expected,
            "{src}"
        );
    }
}

/// A keyword reserved in bare-alias position must not be silently consumed
/// as a table alias; it is a trailing token, so the parse fails closed.
#[test]
fn reserved_word_after_source_is_not_an_alias() {
    let diag = parse_program("from t exists", &HirBuilderV1::new())
        .expect_err("a reserved word must not become an alias");
    assert_eq!(diag.code, ParseErrorCodeV1::UnexpectedToken);
}
