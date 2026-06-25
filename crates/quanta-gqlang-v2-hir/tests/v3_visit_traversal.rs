//! Traversal-contract rails for the [`HirVisitorV1`] walker.
//!
//! The integrity gate overrides EVERY `visit_*` hook, so the trait's default
//! hooks (each delegating to its `walk_*`) and several `walk_*` recursion arms
//! never run under the integrity tests. These rails drive a visitor that relies
//! on the DEFAULT hooks: it overrides only the leaf observers and asserts the
//! walker reaches every id-bearing node exactly once, in deterministic pre-order,
//! across the value/bool/relation/shape/unit/item families. This is the contract
//! `crate::integrity` relies on, so locking it guards against a silently-dropped
//! recursion arm.

#![allow(clippy::unwrap_used, reason = "tests assert on Result/Option directly")]
#![allow(clippy::indexing_slicing, reason = "tests index known-length vecs")]
#![allow(clippy::panic, reason = "tests panic on unexpected variants")]

use quanta_gqlang_v2_hir::{
    walk_item, ConfirmModeV1, DetectionModeV1, EmitModeV1, GraphConstraintsV1, GraphEndpointsV1,
    GraphQueryKindV1, GraphRefV1, HirBuilderV1, HirNodeIdV1, HirVisitorV1, ProgramItemHirV1,
    SpanV1, ValueExprHirV1,
};

/// Records, in visit order, the id of every value and bool node the DEFAULT
/// `visit_*` hooks reach. Overriding only the leaves means every other node is
/// traversed via the trait-default delegation to `walk_*`.
#[derive(Default)]
struct IdTrace {
    values: Vec<u32>,
    bools: Vec<u32>,
}

impl HirVisitorV1 for IdTrace {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        self.values.push(node.id.raw());
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
    fn visit_bool(&mut self, node: &quanta_gqlang_v2_hir::BoolExprHirV1) {
        self.bools.push(node.id.raw());
        quanta_gqlang_v2_hir::walk_bool(self, node);
    }
}

fn gref(name: &str, start: usize, end: usize) -> GraphRefV1 {
    GraphRefV1 {
        name: name.into(),
        span: SpanV1::new(start, end),
    }
}

#[test]
fn walk_value_recurses_path_paren_unary_binary_call_record_list() {
    let b = HirBuilderV1::new();
    // path over a paren'd binary base: path -> paren -> binary(lhs sym, rhs int).
    let lhs = b.symbol("a", SpanV1::new(0, 1));
    let rhs = b.int("1", SpanV1::new(4, 5));
    let bin = b.binary("+", lhs, rhs, SpanV1::new(0, 5));
    let parened = b.paren_value(bin, SpanV1::new(0, 7));
    let path = b.path(parened, vec![], SpanV1::new(0, 9));

    // a call whose args include a unary, a record, and a list.
    let neg = b.unary("-", b.int("2", SpanV1::new(20, 21)), SpanV1::new(19, 21));
    let rec_field = quanta_gqlang_v2_hir::RecordFieldV1 {
        name: "k".into(),
        name_span: SpanV1::new(30, 31),
        value: b.int("3", SpanV1::new(33, 34)),
    };
    let rec = b.record(vec![rec_field], SpanV1::new(29, 35));
    let list = b.list(
        vec![
            b.int("4", SpanV1::new(40, 41)),
            b.int("5", SpanV1::new(43, 44)),
        ],
        SpanV1::new(39, 45),
    );
    let call = b.call(
        "f",
        SpanV1::new(50, 51),
        vec![neg, rec, list],
        SpanV1::new(50, 60),
    );

    let mut trace = IdTrace::default();
    trace.visit_value(&path);
    trace.visit_value(&call);

    // first tree: path + paren + binary + lhs sym + rhs int = 5 value nodes.
    // second tree: call + unary + unary operand + record + record field value +
    // list + 2 list elements = 8 value nodes. Every node visited exactly once.
    assert_eq!(trace.values.len(), 5 + 8);
    // pre-order: the path node is visited before its (paren) base, regardless of
    // the order ids were minted in.
    assert_eq!(trace.values[0], path.id.raw());
}

#[test]
fn walk_bool_recurses_compare_and_or_not_paren_quantifier_value() {
    let b = HirBuilderV1::new();
    // compare x > 1
    let cmp = b.compare(
        ">",
        b.symbol("x", SpanV1::new(0, 1)),
        b.int("1", SpanV1::new(4, 5)),
        SpanV1::new(0, 5),
    );
    // not (paren (z used as bool value))
    let zval = b.bool_value(b.symbol("z", SpanV1::new(10, 11)), SpanV1::new(10, 11));
    let parened = b.paren_bool(zval, SpanV1::new(9, 12));
    let negated = b.not(parened, SpanV1::new(7, 12));
    // quantifier exists over a literal body
    let quant = b.quantifier(
        "exists",
        vec!["q".into()],
        b.bool_literal(true, SpanV1::new(20, 24)),
        SpanV1::new(15, 24),
    );
    // and(or(cmp, negated), quant)
    let disj = b.or(cmp, negated, SpanV1::new(0, 12));
    let conj = b.and(disj, quant, SpanV1::new(0, 24));

    let mut trace = IdTrace::default();
    trace.visit_bool(&conj);

    // bool nodes: and, or, compare, not, paren, value-wrapper, quantifier,
    // literal = 8.
    assert_eq!(trace.bools.len(), 8);
    // value nodes reached through bool position: compare lhs+rhs, the z symbol
    // under the bool-value wrapper = 3.
    assert_eq!(trace.values.len(), 3);
}

#[test]
fn walk_relation_recurses_filter_project_compose_through_default_hooks() {
    let b = HirBuilderV1::new();
    let src = b.relation_source("users", None, SpanV1::new(0, 5));
    // filter users where active
    let pred = b.bool_value(b.symbol("active", SpanV1::new(12, 18)), SpanV1::new(12, 18));
    let filtered = b.relation_filter(src, pred, SpanV1::new(0, 18));
    // project id from the filtered relation
    let proj_item = quanta_gqlang_v2_hir::ProjectionItemV1 {
        value: b.symbol("id", SpanV1::new(25, 27)),
        alias: None,
    };
    let projected = b.project(filtered, vec![proj_item], SpanV1::new(0, 27));
    // compose two sources via an open symbol
    let other = b.relation_source("roles", None, SpanV1::new(30, 35));
    let composed = b.compose("join", vec![projected, other], SpanV1::new(0, 35));

    let mut trace = IdTrace::default();
    trace.visit_relation(&composed);

    // value nodes reached: the filter predicate's symbol + the projection symbol.
    assert_eq!(trace.values.len(), 2);
    // bool nodes reached: the filter predicate wrapper.
    assert_eq!(trace.bools.len(), 1);
}

#[test]
fn walk_item_dispatches_decl_and_rule_variants() {
    let b = HirBuilderV1::new();
    // Decl: name = (1 + 2)
    let decl = b.decl(
        "x",
        SpanV1::new(0, 1),
        b.binary(
            "+",
            b.int("1", SpanV1::new(4, 5)),
            b.int("2", SpanV1::new(8, 9)),
            SpanV1::new(4, 9),
        ),
        SpanV1::new(0, 9),
    );
    let decl_item: ProgramItemHirV1 = decl.into();
    let mut trace = IdTrace::default();
    walk_item(&mut trace, &decl_item);
    // decl body binary + lhs + rhs = 3 value nodes.
    assert_eq!(trace.values.len(), 3);

    // Rule: head symbol, body compare.
    let rule = b.rule(
        "r",
        SpanV1::new(0, 1),
        b.symbol("h", SpanV1::new(5, 6)),
        b.compare(
            "=",
            b.symbol("a", SpanV1::new(10, 11)),
            b.int("1", SpanV1::new(14, 15)),
            SpanV1::new(10, 15),
        ),
        SpanV1::new(0, 15),
    );
    let rule_item: ProgramItemHirV1 = rule.into();
    let mut rtrace = IdTrace::default();
    walk_item(&mut rtrace, &rule_item);
    // head symbol + compare lhs + compare rhs = 3 value nodes; one compare bool.
    assert_eq!(rtrace.values.len(), 3);
    assert_eq!(rtrace.bools.len(), 1);
}

#[test]
fn walk_item_leaf_items_have_no_id_bearing_value_children() {
    let b = HirBuilderV1::new();
    // FIND / CHECK / META carry only open refs, so the walker reaches no shared
    // value/bool kernel nodes (their walk_* are intentional no-ops).
    let find = b.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(gref("s", 0, 1)),
            sink: Some(gref("t", 4, 5)),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
        SpanV1::new(0, 5),
    );
    let check = b.legality(
        gref("rules.x", 6, 13),
        None,
        None,
        ConfirmModeV1::Skip,
        EmitModeV1::Findings,
        DetectionModeV1::PatternOnly,
        SpanV1::new(0, 20),
    );

    for item in [ProgramItemHirV1::from(find), ProgramItemHirV1::from(check)] {
        let mut trace = IdTrace::default();
        walk_item(&mut trace, &item);
        assert!(trace.values.is_empty(), "{item:?} reached value nodes");
        assert!(trace.bools.is_empty(), "{item:?} reached bool nodes");
    }
}

#[test]
fn shape_default_hook_recurses_record_and_list_into_leaf_values() {
    let b = HirBuilderV1::new();
    // record { a: 1, items: [2] } as a result shape.
    let leaf_a = b.shape_leaf(b.int("1", SpanV1::new(5, 6)), SpanV1::new(5, 6));
    let leaf_el = b.shape_leaf(b.int("2", SpanV1::new(10, 11)), SpanV1::new(10, 11));
    let list = b.shape_list(leaf_el, SpanV1::new(9, 12));
    let rec = b.shape_record(
        vec![
            quanta_gqlang_v2_hir::ShapeFieldV1 {
                name: "a".into(),
                name_span: SpanV1::new(2, 3),
                shape: leaf_a,
            },
            quanta_gqlang_v2_hir::ShapeFieldV1 {
                name: "items".into(),
                name_span: SpanV1::new(7, 12),
                shape: list,
            },
        ],
        SpanV1::new(0, 13),
    );

    let mut trace = IdTrace::default();
    trace.visit_shape(&rec);
    // two leaf value expressions reached through the shape tree.
    assert_eq!(trace.values.len(), 2);
}

/// A visitor that overrides ONLY the two observation hooks it needs, so every
/// other `visit_*` runs through the trait DEFAULT (each delegating to `walk_*`).
/// This is the path the integrity gate never takes (it overrides all hooks), so
/// it locks the default delegation contract for the item-level hooks.
#[derive(Default)]
struct DefaultDelegationCounter {
    relations: usize,
}

// Only `visit_relation` is overridden as the observation point. `visit_value`,
// `visit_bool`, `visit_shape`, `visit_query`, `visit_catalog_decl`,
// `visit_composition`, `visit_presentation`, and `visit_meta` all run through
// their trait DEFAULTS, which is exactly the delegation path the integrity gate
// (overriding every hook) never executes.
impl HirVisitorV1 for DefaultDelegationCounter {
    fn visit_relation(&mut self, node: &quanta_gqlang_v2_hir::RelationExprHirV1) {
        self.relations += 1;
        quanta_gqlang_v2_hir::walk_relation(self, node);
    }
}

#[test]
fn default_item_hooks_delegate_into_their_walks() {
    let b = HirBuilderV1::new();
    // A composition whose producer is a VIEW catalog decl carrying an inline
    // SELECT (filter over a source, plus a record result shape), and whose body
    // is a META directive. Driving this with a visitor that overrides ONLY
    // visit_relation forces the default visit_composition / visit_catalog_decl /
    // visit_query / visit_shape / visit_value / visit_bool / visit_meta hooks to
    // each run through their trait-default delegation.
    use quanta_gqlang_v2_hir::{CatalogPayloadV1, MetaKindV1, ViewBodyV1};

    // producer: VIEW v = SELECT (users WHERE active) with a leaf result shape.
    let users = b.relation_source("users", None, SpanV1::new(20, 25));
    let pred = b.bool_value(b.symbol("active", SpanV1::new(32, 38)), SpanV1::new(32, 38));
    let filtered = b.relation_filter(users, pred, SpanV1::new(20, 38));
    let shape = b.shape_leaf(b.symbol("id", SpanV1::new(40, 42)), SpanV1::new(40, 42));
    let inline = b.query(filtered, Some(shape), SpanV1::new(20, 42));
    let view = b.catalog_decl(
        "v",
        SpanV1::new(14, 15),
        CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(inline))),
        SpanV1::new(10, 42),
    );
    let binding = b.composition_binding("v", SpanV1::new(4, 5), view.into());

    // body: REQUIRE CERTIFIED (a meta directive).
    let body = b.meta(MetaKindV1::RequireCertified, SpanV1::new(45, 60));

    let comp = b.composition(
        vec![binding],
        "v",
        SpanV1::new(62, 63),
        body.into(),
        SpanV1::new(0, 64),
    );
    let item = ProgramItemHirV1::from(comp);

    let mut counter = DefaultDelegationCounter::default();
    counter.visit_item(&item);

    // The inline SELECT reaches the source relation and the filter relation = 2.
    assert_eq!(counter.relations, 2);

    // Also drive a bare presentation item through the default visit_presentation
    // hook (its walk is a no-op, so no relations are reached).
    use quanta_gqlang_v2_hir::{PresentationKindV1, ShowCategoryV1};
    let pres = b.presentation(
        PresentationKindV1::Show {
            category: ShowCategoryV1::Gates,
        },
        SpanV1::new(0, 9),
    );
    let pres_item = ProgramItemHirV1::from(pres);
    let mut pres_counter = DefaultDelegationCounter::default();
    pres_counter.visit_item(&pres_item);
    assert_eq!(pres_counter.relations, 0);
}

#[test]
fn from_raw_round_trips_and_default_hooks_are_pure_observation() {
    // a bare leaf visited through the default value hook records exactly itself.
    let b = HirBuilderV1::new();
    let leaf = b.symbol("solo", SpanV1::new(0, 4));
    let mut trace = IdTrace::default();
    trace.visit_value(&leaf);
    assert_eq!(trace.values, vec![leaf.id.raw()]);
    // id raw round-trips through from_raw.
    assert_eq!(HirNodeIdV1::from_raw(leaf.id.raw()).raw(), leaf.id.raw());
}
