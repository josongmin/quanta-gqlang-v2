#![expect(
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use std::collections::HashSet;

use quanta_gqlang_v2_hir::{
    check_tree, walk_bool, walk_query, walk_relation, walk_shape, walk_value, AliasV1,
    BoolExprHirV1, BoolExprKindV1, CallArgHirV1, CallArgsHirV1, HirBuilderV1, HirNodeIdV1,
    HirRootRefV1, HirVisitorV1, PathSegmentV1, ProjectionItemV1, QueryHirV1, RecordFieldV1,
    RelationExprHirV1, RelationExprKindV1, ResultShapeHirV1, ResultShapeKindV1, ShapeFieldV1,
    SpanV1, ValueExprHirV1, ValueExprKindV1,
};

/// Collects every visited node's (id, span) across all node kinds.
#[derive(Default)]
struct NodeCollector {
    ids: Vec<HirNodeIdV1>,
    spans: Vec<SpanV1>,
}

impl HirVisitorV1 for NodeCollector {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        self.ids.push(node.id);
        self.spans.push(node.span);
        walk_value(self, node);
    }
    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        self.ids.push(node.id);
        self.spans.push(node.span);
        walk_bool(self, node);
    }
    fn visit_relation(&mut self, node: &RelationExprHirV1) {
        self.ids.push(node.id);
        self.spans.push(node.span);
        walk_relation(self, node);
    }
    fn visit_shape(&mut self, node: &ResultShapeHirV1) {
        self.ids.push(node.id);
        self.spans.push(node.span);
        walk_shape(self, node);
    }
    fn visit_query(&mut self, node: &QueryHirV1) {
        self.ids.push(node.id);
        self.spans.push(node.span);
        walk_query(self, node);
    }
}

fn assert_unique_ids(ids: &[HirNodeIdV1]) {
    let set: HashSet<HirNodeIdV1> = ids.iter().copied().collect();
    assert_eq!(
        set.len(),
        ids.len(),
        "node ids must be unique across the whole tree"
    );
}

fn assert_spans_in_bounds(spans: &[SpanV1], source_len: usize) {
    for span in spans {
        assert!(
            span.is_within(source_len),
            "span {span:?} out of bounds for len {source_len}"
        );
    }
}

/// Scenario 1: value expr `a.b + f(1, "x")` — tree shape, unique ids, spans.
#[test]
fn value_expr_tree_shape_ids_and_spans() {
    // Source layout (byte offsets):  a.b + f(1, "x")
    //                                0123456789...
    let src = r#"a.b + f(1, "x")"#;
    let b = HirBuilderV1::new();

    let a = b.symbol("a", SpanV1::new(0, 1));
    let a_b = b
        .try_path(
            a,
            vec![PathSegmentV1 {
                name: "b".into(),
                span: SpanV1::new(2, 3),
            }],
            SpanV1::new(0, 3),
        )
        .unwrap();

    let one = b.int("1", SpanV1::new(8, 9));
    let x = b.string("x", SpanV1::new(11, 14));
    let call = b.call(
        "f",
        SpanV1::new(6, 7),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(one), CallArgHirV1::Expr(x)]).unwrap(),
        SpanV1::new(6, 15),
    );

    let sum = b.binary("+", a_b, call, SpanV1::new(0, 15));

    // Tree shape: top is binary `+`.
    let ValueExprKindV1::Binary(top) = &sum.kind else {
        panic!("expected binary at root")
    };
    assert_eq!(top.op.symbol, "+");
    // lhs is a path a.b
    let ValueExprKindV1::Path(path) = &top.lhs.kind else {
        panic!("expected path lhs")
    };
    assert_eq!(path.segments.len(), 1);
    assert_eq!(path.segments[0].name, "b");
    let ValueExprKindV1::Symbol(sym) = &path.base.kind else {
        panic!("path base is symbol")
    };
    assert_eq!(sym.name, "a");
    // rhs is call f(1, "x")
    let ValueExprKindV1::Call(c) = &top.rhs.kind else {
        panic!("expected call rhs")
    };
    assert_eq!(c.callee.name, "f");
    assert_eq!(c.args().len(), 2);

    let mut collector = NodeCollector::default();
    collector.visit_value(&sum);
    // 6 value nodes: a, a.b, 1, "x", call, sum
    assert_eq!(collector.ids.len(), 6);
    assert_unique_ids(&collector.ids);
    assert_spans_in_bounds(&collector.spans, src.len());

    // checked constructor accepts the sound tree.
    let checked = check_tree(HirRootRefV1::Value(&sum), src.len()).unwrap();
    assert_eq!(checked.node_count(), 6);
}

#[test]
fn call_args_reject_mixed_and_multiple_star_states() {
    let span = SpanV1::new(0, 1);
    let builder = HirBuilderV1::new();
    assert!(CallArgsHirV1::try_new(vec![
        CallArgHirV1::Expr(builder.int("1", span)),
        CallArgHirV1::star(span),
    ])
    .is_err());
    assert!(
        CallArgsHirV1::try_new(vec![CallArgHirV1::star(span), CallArgHirV1::star(span),]).is_err()
    );
    assert!(CallArgsHirV1::try_new(vec![CallArgHirV1::star(span)]).is_ok());
}

/// Scenario 2: bool formula `(x > 1) and (y = 2 or z)` — grouping preserved.
#[test]
fn bool_formula_preserves_explicit_grouping() {
    let src = "(x > 1) and (y = 2 or z)";
    let b = HirBuilderV1::new();

    let x = b.symbol("x", SpanV1::new(1, 2));
    let one = b.int("1", SpanV1::new(5, 6));
    let x_gt_1 = b.compare(">", x, one, SpanV1::new(1, 6));
    let left_group = b.paren_bool(x_gt_1, SpanV1::new(0, 7));

    let y = b.symbol("y", SpanV1::new(13, 14));
    let two = b.int("2", SpanV1::new(17, 18));
    let y_eq_2 = b.compare("=", y, two, SpanV1::new(13, 18));
    let z = b.symbol("z", SpanV1::new(22, 23));
    let z_bool = b.bool_value(z, SpanV1::new(22, 23));
    let or = b.or(y_eq_2, z_bool, SpanV1::new(13, 23));
    let right_group = b.paren_bool(or, SpanV1::new(12, 24));

    let top = b.and(left_group, right_group, SpanV1::new(0, 24));

    // Grouping must NOT be flattened away: both `and` operands are Paren nodes.
    let BoolExprKindV1::And(and) = &top.kind else {
        panic!("root must be and")
    };
    assert!(
        matches!(and.lhs.kind, BoolExprKindV1::Paren(_)),
        "left grouping preserved"
    );
    assert!(
        matches!(and.rhs.kind, BoolExprKindV1::Paren(_)),
        "right grouping preserved"
    );

    // Inside the right group, the `or` is intact under the paren.
    let BoolExprKindV1::Paren(inner) = &and.rhs.kind else {
        panic!()
    };
    assert!(
        matches!(inner.kind, BoolExprKindV1::Or(_)),
        "or inside paren preserved"
    );

    let checked = check_tree(HirRootRefV1::Bool(&top), src.len()).unwrap();
    // value nodes: x,1,y,2,z = 5; bool nodes: cmp>, paren, cmp=, boolval(z), or, paren, and = 7
    assert_eq!(checked.node_count(), 12);
}

/// Scenario 3: relation source + projection `{ id: u.id, tags: [u.t1, u.t2] }`.
#[test]
fn relation_projection_with_aliases_and_nested_list() {
    let src = "users u | project { id: u.id, tags: [u.t1, u.t2] }";
    let b = HirBuilderV1::new();

    let source = b.relation_source(
        "users",
        SpanV1::new(0, 5),
        Some(AliasV1 {
            name: "u".into(),
            span: SpanV1::new(6, 7),
        }),
        SpanV1::new(0, 7),
    );

    // id: u.id
    let u_id_base = b.symbol("u", SpanV1::new(24, 25));
    let u_id = b
        .try_path(
            u_id_base,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(26, 28),
            }],
            SpanV1::new(24, 28),
        )
        .unwrap();
    let id_item = ProjectionItemV1 {
        value: u_id,
        alias: Some(AliasV1 {
            name: "id".into(),
            span: SpanV1::new(20, 22),
        }),
    };

    // tags: [u.t1, u.t2]
    let u_t1_base = b.symbol("u", SpanV1::new(37, 38));
    let u_t1 = b
        .try_path(
            u_t1_base,
            vec![PathSegmentV1 {
                name: "t1".into(),
                span: SpanV1::new(39, 41),
            }],
            SpanV1::new(37, 41),
        )
        .unwrap();
    let u_t2_base = b.symbol("u", SpanV1::new(43, 44));
    let u_t2 = b
        .try_path(
            u_t2_base,
            vec![PathSegmentV1 {
                name: "t2".into(),
                span: SpanV1::new(45, 47),
            }],
            SpanV1::new(43, 47),
        )
        .unwrap();
    let tags_list = b.list(vec![u_t1, u_t2], SpanV1::new(36, 48));
    let tags_item = ProjectionItemV1 {
        value: tags_list,
        alias: Some(AliasV1 {
            name: "tags".into(),
            span: SpanV1::new(30, 34),
        }),
    };

    let proj = b
        .try_project(source, vec![id_item, tags_item], SpanV1::new(0, src.len()))
        .unwrap();

    let RelationExprKindV1::Project(p) = &proj.kind else {
        panic!("expected projection")
    };
    assert_eq!(p.items().len(), 2);
    assert_eq!(p.items()[0].alias.as_ref().unwrap().name, "id");
    assert_eq!(p.items()[1].alias.as_ref().unwrap().name, "tags");
    // nested list shape in second item
    let ValueExprKindV1::List(list) = &p.items()[1].value.kind else {
        panic!("tags is a list")
    };
    assert_eq!(list.elements.len(), 2);
    // input source carries its alias
    let RelationExprKindV1::Source(s) = &p.input.kind else {
        panic!("input is a source")
    };
    assert_eq!(s.alias.as_ref().unwrap().name, "u");

    let checked = check_tree(HirRootRefV1::Relation(&proj), src.len()).unwrap();
    let mut collector = NodeCollector::default();
    collector.visit_relation(&proj);
    assert_unique_ids(&collector.ids);
    assert_eq!(checked.node_count(), collector.ids.len());
}

#[test]
fn projection_requires_at_least_one_item_at_public_construction() {
    let builder = HirBuilderV1::new();
    let source = builder.relation_source("users", SpanV1::new(0, 5), None, SpanV1::new(0, 5));
    assert_eq!(
        builder
            .try_project(source, vec![], SpanV1::new(0, 15))
            .unwrap_err(),
        "projection requires at least one item"
    );
}

/// Scenario 4: a DeclHirV1 and a RuleHirV1 each EMBED a shared ValueExprHirV1.
#[test]
fn decl_and_rule_embed_shared_value_expr() {
    let src = "risk = a + b";
    let b = HirBuilderV1::new();

    // Decl: risk = a + b
    let da = b.symbol("a", SpanV1::new(7, 8));
    let db = b.symbol("b", SpanV1::new(11, 12));
    let decl_body = b.binary("+", da, db, SpanV1::new(7, 12));
    let decl = b.decl("risk", SpanV1::new(0, 4), decl_body, SpanV1::new(0, 12));

    // Rule: head value expr + bool body, both shared kernels.
    let ra = b.symbol("a", SpanV1::new(7, 8));
    let rb = b.symbol("b", SpanV1::new(11, 12));
    let rule_head = b.binary("+", ra, rb, SpanV1::new(7, 12));
    let cond_lhs = b.symbol("a", SpanV1::new(7, 8));
    let cond_rhs = b.int("0", SpanV1::new(11, 12));
    let rule_body = b.compare(">", cond_lhs, cond_rhs, SpanV1::new(7, 12));
    let rule = b.rule(
        "risky",
        SpanV1::new(0, 5),
        rule_head,
        rule_body,
        SpanV1::new(0, 12),
    );

    // Sharing invariant: the decl body is a ValueExprHirV1, and so is the rule
    // head — both the SAME shared type (proven by passing one helper).
    fn is_binary_plus(v: &ValueExprHirV1) -> bool {
        matches!(&v.kind, ValueExprKindV1::Binary(bin) if bin.op.symbol == "+")
    }
    assert!(is_binary_plus(&decl.body), "decl embeds shared value expr");
    assert!(
        is_binary_plus(&rule.head),
        "rule embeds the same shared value expr type"
    );

    // Both validate.
    check_tree(HirRootRefV1::Decl(&decl), src.len()).unwrap();
    check_tree(HirRootRefV1::Rule(&rule), src.len()).unwrap();
}

/// Nested result-shape fixture proving ResultShapeHirV1 is real and shared.
#[test]
fn nested_result_shape_shares_value_kernel() {
    let src = "{ id: u.id, children: [ { name: c.n } ] }";
    let b = HirBuilderV1::new();

    let u = b.symbol("u", SpanV1::new(6, 7));
    let u_id = b
        .try_path(
            u,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(8, 10),
            }],
            SpanV1::new(6, 10),
        )
        .unwrap();
    let id_leaf = b.shape_leaf(u_id, SpanV1::new(6, 10));

    let c = b.symbol("c", SpanV1::new(32, 33));
    let c_n = b
        .try_path(
            c,
            vec![PathSegmentV1 {
                name: "n".into(),
                span: SpanV1::new(34, 35),
            }],
            SpanV1::new(32, 35),
        )
        .unwrap();
    let name_leaf = b.shape_leaf(c_n, SpanV1::new(32, 35));
    let child_record = b.shape_record(
        vec![ShapeFieldV1 {
            name: "name".into(),
            name_span: SpanV1::new(26, 30),
            shape: name_leaf,
        }],
        SpanV1::new(24, 37),
    );
    let children_list = b.shape_list(child_record, SpanV1::new(22, 39));

    let top = b.shape_record(
        vec![
            ShapeFieldV1 {
                name: "id".into(),
                name_span: SpanV1::new(2, 4),
                shape: id_leaf,
            },
            ShapeFieldV1 {
                name: "children".into(),
                name_span: SpanV1::new(12, 20),
                shape: children_list,
            },
        ],
        SpanV1::new(0, src.len()),
    );

    let ResultShapeKindV1::Record(rec) = &top.kind else {
        panic!("top is a record shape")
    };
    assert_eq!(rec.fields.len(), 2);
    let ResultShapeKindV1::List(list) = &rec.fields[1].shape.kind else {
        panic!("children is a list shape")
    };
    let ResultShapeKindV1::Record(_) = &list.element.kind else {
        panic!("list element is a record shape")
    };

    check_tree(HirRootRefV1::Shape(&top), src.len()).unwrap();
}

/// A query embeds a shared relation body plus a shared result shape; the
/// invariant rail asserts the visitor sees every node exactly once.
#[test]
fn query_visitor_visits_every_node_exactly_once() {
    let src = "users u | project { id: u.id } shape { id: u.id }";
    let b = HirBuilderV1::new();

    let source = b.relation_source(
        "users",
        SpanV1::new(0, 5),
        Some(AliasV1 {
            name: "u".into(),
            span: SpanV1::new(6, 7),
        }),
        SpanV1::new(0, 7),
    );
    let u1 = b.symbol("u", SpanV1::new(24, 25));
    let u1_id = b
        .try_path(
            u1,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(26, 28),
            }],
            SpanV1::new(24, 28),
        )
        .unwrap();
    let proj = b
        .try_project(
            source,
            vec![ProjectionItemV1 {
                value: u1_id,
                alias: Some(AliasV1 {
                    name: "id".into(),
                    span: SpanV1::new(20, 22),
                }),
            }],
            SpanV1::new(0, 30),
        )
        .unwrap();

    let u2 = b.symbol("u", SpanV1::new(43, 44));
    let u2_id = b
        .try_path(
            u2,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(45, 47),
            }],
            SpanV1::new(43, 47),
        )
        .unwrap();
    let leaf = b.shape_leaf(u2_id, SpanV1::new(43, 47));
    let shape = b.shape_record(
        vec![ShapeFieldV1 {
            name: "id".into(),
            name_span: SpanV1::new(39, 41),
            shape: leaf,
        }],
        SpanV1::new(31, src.len()),
    );

    let query = b.query(proj, Some(shape), SpanV1::new(0, src.len()));

    let mut collector = NodeCollector::default();
    collector.visit_query(&query);

    // exactly-once: count == unique-id count == builder mint count.
    assert_unique_ids(&collector.ids);
    assert_eq!(collector.ids.len(), usize::try_from(b.minted()).unwrap());

    let checked = check_tree(HirRootRefV1::Query(&query), src.len()).unwrap();
    assert_eq!(checked.node_count(), collector.ids.len());
}

/// A record-construction fixture proving RecordFieldV1 embeds shared values.
#[test]
fn record_construction_embeds_shared_values() {
    let src = "{ a: 1, b: x }";
    let b = HirBuilderV1::new();
    let one = b.int("1", SpanV1::new(5, 6));
    let x = b.symbol("x", SpanV1::new(11, 12));
    let rec = b.record(
        vec![
            RecordFieldV1 {
                name: "a".into(),
                name_span: SpanV1::new(2, 3),
                value: one,
            },
            RecordFieldV1 {
                name: "b".into(),
                name_span: SpanV1::new(8, 9),
                value: x,
            },
        ],
        SpanV1::new(0, src.len()),
    );
    let ValueExprKindV1::Record(r) = &rec.kind else {
        panic!("expected record")
    };
    assert_eq!(r.fields.len(), 2);
    check_tree(HirRootRefV1::Value(&rec), src.len()).unwrap();
}
