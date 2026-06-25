//! Identity / accessor rails for [`ProgramItemHirV1`] and the checked-root view.
//!
//! Every program-item variant must report the id and span of the payload it
//! carries through the `id()`/`span()` accessors, every `From<_>` conversion must
//! wrap the right variant, and the checked root produced by `check_tree` must
//! expose the root view plus the validated source length. These accessor arms are
//! the seam downstream crates key their side tables off, so a wrong arm (returning
//! a sibling's id) would silently corrupt later passes; these rails lock each arm
//! to its own payload.

#![allow(clippy::unwrap_used, reason = "tests assert on Result directly")]
#![allow(clippy::panic, reason = "tests panic on unexpected variants")]

use quanta_gqlang_v2_hir::{
    check_tree, ActivationV1, CatalogPayloadV1, ConfirmModeV1, DetectionModeV1, EmitModeV1,
    GraphConstraintsV1, GraphEndpointsV1, GraphQueryKindV1, GraphRefV1, HirBuilderV1, HirRootRefV1,
    MetaKindV1, PresentationKindV1, ProgramItemHirV1, ShowCategoryV1, SpanV1, ViewBodyV1,
};

fn gref(name: &str, start: usize, end: usize) -> GraphRefV1 {
    GraphRefV1 {
        name: name.into(),
        span: SpanV1::new(start, end),
    }
}

/// `id()` and `span()` must return the payload node's own id and span.
#[test]
fn id_and_span_accessors_report_the_carried_payload_for_every_variant() {
    let b = HirBuilderV1::new();

    // Query
    let q = b.query(
        b.relation_source("t", None, SpanV1::new(0, 1)),
        None,
        SpanV1::new(0, 7),
    );
    let (qid, qspan) = (q.id, q.span);
    let item = ProgramItemHirV1::from(q);
    assert_eq!(item.id(), qid);
    assert_eq!(item.span(), qspan);

    // Decl
    let d = b.decl(
        "x",
        SpanV1::new(0, 1),
        b.int("1", SpanV1::new(4, 5)),
        SpanV1::new(0, 5),
    );
    let (did, dspan) = (d.id, d.span);
    let item = ProgramItemHirV1::from(d);
    assert!(matches!(item, ProgramItemHirV1::Decl(_)));
    assert_eq!(item.id(), did);
    assert_eq!(item.span(), dspan);

    // Rule
    let r = b.rule(
        "r",
        SpanV1::new(0, 1),
        b.symbol("h", SpanV1::new(5, 6)),
        b.bool_literal(true, SpanV1::new(9, 13)),
        SpanV1::new(0, 13),
    );
    let (rid, rspan) = (r.id, r.span);
    let item = ProgramItemHirV1::from(r);
    assert!(matches!(item, ProgramItemHirV1::Rule(_)));
    assert_eq!(item.id(), rid);
    assert_eq!(item.span(), rspan);

    // GraphQuery
    let g = b.graph_query(
        GraphQueryKindV1::Slice,
        GraphEndpointsV1 {
            anchor: Some(gref("a", 5, 6)),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
        SpanV1::new(0, 10),
    );
    let (gid, gspan) = (g.id, g.span);
    let item = ProgramItemHirV1::from(g);
    assert_eq!(item.id(), gid);
    assert_eq!(item.span(), gspan);

    // Legality
    let l = b.legality(
        gref("rules.x", 6, 13),
        None,
        None,
        ConfirmModeV1::Prompt,
        EmitModeV1::Findings,
        DetectionModeV1::SemanticOnly,
        SpanV1::new(0, 20),
    );
    let (lid, lspan) = (l.id, l.span);
    let item = ProgramItemHirV1::from(l);
    assert_eq!(item.id(), lid);
    assert_eq!(item.span(), lspan);

    // Presentation
    let p = b.presentation(
        PresentationKindV1::Show {
            category: ShowCategoryV1::Views,
        },
        SpanV1::new(0, 9),
    );
    let (pid, pspan) = (p.id, p.span);
    let item = ProgramItemHirV1::from(p);
    assert_eq!(item.id(), pid);
    assert_eq!(item.span(), pspan);

    // CatalogDecl
    let c = b.catalog_decl(
        "v",
        SpanV1::new(5, 6),
        CatalogPayloadV1::View(ViewBodyV1::QueryRef(gref("base", 10, 14))),
        SpanV1::new(0, 14),
    );
    let (cid, cspan) = (c.id, c.span);
    let item = ProgramItemHirV1::from(c);
    assert_eq!(item.id(), cid);
    assert_eq!(item.span(), cspan);

    // Composition
    let producer = b.query(
        b.relation_source("u", None, SpanV1::new(10, 11)),
        None,
        SpanV1::new(10, 11),
    );
    let binding = b.composition_binding("c", SpanV1::new(4, 5), producer.into());
    let body = b.presentation(
        PresentationKindV1::Show {
            category: ShowCategoryV1::Rules,
        },
        SpanV1::new(20, 30),
    );
    let comp = b.composition(
        vec![binding],
        "c",
        SpanV1::new(40, 41),
        body.into(),
        SpanV1::new(0, 50),
    );
    let (compid, compspan) = (comp.id, comp.span);
    let item = ProgramItemHirV1::from(comp);
    assert_eq!(item.id(), compid);
    assert_eq!(item.span(), compspan);

    // Meta
    let m = b.meta(MetaKindV1::RequireCertified, SpanV1::new(0, 16));
    let (mid, mspan) = (m.id, m.span);
    let item = ProgramItemHirV1::from(m);
    assert_eq!(item.id(), mid);
    assert_eq!(item.span(), mspan);
}

/// `check_tree` returns a `CheckedHirV1` exposing the root view and the
/// validated source length; the root must dispatch back to the same item.
#[test]
fn checked_root_exposes_item_root_and_source_len() {
    let b = HirBuilderV1::new();
    let m = b.meta(MetaKindV1::RequireCertified, SpanV1::new(0, 16));
    let item = ProgramItemHirV1::from(m);
    let checked = check_tree(HirRootRefV1::Item(&item), 64).unwrap();
    assert_eq!(checked.source_len(), 64);
    assert_eq!(checked.node_count(), 1);
    let root = checked.root();
    let HirRootRefV1::Item(item_root) = root else {
        panic!("expected Item root, got {root:?}");
    };
    assert!(matches!(item_root, ProgramItemHirV1::Meta(_)));
}

/// `HirRootRefV1::accept` dispatches each leaf-root variant to the matching
/// visitor hook; `check_tree` over each must validate exactly the carried node.
#[test]
fn check_tree_validates_each_leaf_root_variant_directly() {
    let b = HirBuilderV1::new();

    let value = b.int("42", SpanV1::new(0, 2));
    assert_eq!(
        check_tree(HirRootRefV1::Value(&value), 2)
            .unwrap()
            .node_count(),
        1
    );

    let boolean = b.bool_literal(true, SpanV1::new(0, 4));
    assert_eq!(
        check_tree(HirRootRefV1::Bool(&boolean), 4)
            .unwrap()
            .node_count(),
        1
    );

    let relation = b.relation_source("t", None, SpanV1::new(0, 1));
    assert_eq!(
        check_tree(HirRootRefV1::Relation(&relation), 1)
            .unwrap()
            .node_count(),
        1
    );

    let shape = b.shape_leaf(b.int("1", SpanV1::new(0, 1)), SpanV1::new(0, 1));
    // shape leaf + its embedded value = 2 id-bearing nodes.
    assert_eq!(
        check_tree(HirRootRefV1::Shape(&shape), 1)
            .unwrap()
            .node_count(),
        2
    );

    let decl = b.decl(
        "x",
        SpanV1::new(0, 1),
        b.int("1", SpanV1::new(4, 5)),
        SpanV1::new(0, 5),
    );
    // decl + body value = 2.
    assert_eq!(
        check_tree(HirRootRefV1::Decl(&decl), 5)
            .unwrap()
            .node_count(),
        2
    );

    let rule = b.rule(
        "r",
        SpanV1::new(0, 1),
        b.symbol("h", SpanV1::new(5, 6)),
        b.bool_literal(true, SpanV1::new(9, 13)),
        SpanV1::new(0, 13),
    );
    // rule + head value + body bool = 3.
    assert_eq!(
        check_tree(HirRootRefV1::Rule(&rule), 13)
            .unwrap()
            .node_count(),
        3
    );
}

/// The dedicated graph-query / legality / presentation / catalog / composition /
/// meta root variants of `accept` must also be reachable directly (not only
/// through an `Item` wrapper).
#[test]
fn check_tree_validates_each_dedicated_item_root_variant() {
    let b = HirBuilderV1::new();

    let g = b.graph_query(
        GraphQueryKindV1::Path,
        GraphEndpointsV1::default(),
        GraphConstraintsV1::default(),
        SpanV1::new(0, 10),
    );
    assert_eq!(
        check_tree(HirRootRefV1::GraphQuery(&g), 10)
            .unwrap()
            .node_count(),
        1
    );

    let l = b.legality(
        gref("rules.x", 6, 13),
        None,
        None,
        ConfirmModeV1::Skip,
        EmitModeV1::Sarif,
        DetectionModeV1::Hybrid,
        SpanV1::new(0, 20),
    );
    assert_eq!(
        check_tree(HirRootRefV1::Legality(&l), 20)
            .unwrap()
            .node_count(),
        1
    );

    let p = b.presentation(
        PresentationKindV1::Show {
            category: ShowCategoryV1::Packs,
        },
        SpanV1::new(0, 9),
    );
    assert_eq!(
        check_tree(HirRootRefV1::Presentation(&p), 9)
            .unwrap()
            .node_count(),
        1
    );

    let c = b.catalog_decl(
        "p",
        SpanV1::new(5, 6),
        CatalogPayloadV1::Pack {
            source: gref("reg", 10, 13),
            activation: ActivationV1::Manual,
        },
        SpanV1::new(0, 13),
    );
    assert_eq!(
        check_tree(HirRootRefV1::CatalogDecl(&c), 13)
            .unwrap()
            .node_count(),
        1
    );

    let inner = b.meta(MetaKindV1::RequireCertified, SpanV1::new(20, 30));
    let comp = b.composition(
        vec![],
        "w",
        SpanV1::new(40, 41),
        inner.into(),
        SpanV1::new(0, 50),
    );
    // composition + body meta = 2.
    assert_eq!(
        check_tree(HirRootRefV1::Composition(&comp), 50)
            .unwrap()
            .node_count(),
        2
    );

    let m = b.meta(
        MetaKindV1::UsePack {
            pack_id: "x".into(),
            activation: ActivationV1::Session,
            lockfile: None,
        },
        SpanV1::new(0, 16),
    );
    assert_eq!(
        check_tree(HirRootRefV1::Meta(&m), 16).unwrap().node_count(),
        1
    );
}
