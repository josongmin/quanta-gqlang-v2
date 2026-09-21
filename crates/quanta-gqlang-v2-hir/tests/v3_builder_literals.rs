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
    check_tree, HirBuilderV1, HirRootRefV1, LiteralV1, SpanV1, ValueExprKindV1,
};

#[test]
fn float_constructor_carries_raw_float_literal() {
    let b = HirBuilderV1::new();
    let f = b.float("3.14", SpanV1::new(0, 4));
    let ValueExprKindV1::Literal(LiteralV1::Float(raw)) = &f.kind else {
        panic!("expected Float literal, got {:?}", f.kind);
    };
    assert_eq!(raw, "3.14");
    // built node validates as a single in-bounds id-bearing node.
    assert_eq!(
        check_tree(HirRootRefV1::Value(&f), 4).unwrap().node_count(),
        1
    );
}

#[test]
fn bool_lit_constructor_carries_bool_literal() {
    let b = HirBuilderV1::new();
    let t = b.bool_lit(true, SpanV1::new(0, 4));
    let f = b.bool_lit(false, SpanV1::new(5, 10));
    assert!(matches!(
        t.kind,
        ValueExprKindV1::Literal(LiteralV1::Bool(true))
    ));
    assert!(matches!(
        f.kind,
        ValueExprKindV1::Literal(LiteralV1::Bool(false))
    ));
}

#[test]
fn null_constructor_carries_null_literal() {
    let b = HirBuilderV1::new();
    let n = b.null(SpanV1::new(0, 4));
    assert!(matches!(n.kind, ValueExprKindV1::Literal(LiteralV1::Null)));
}

#[test]
fn next_id_mints_fresh_monotonic_ids_tracked_by_minted() {
    let b = HirBuilderV1::new();
    assert_eq!(b.minted(), 0);
    let a = b.next_id();
    let c = b.next_id();
    // monotonic, gap-free, distinct.
    assert_eq!(a.raw(), 0);
    assert_eq!(c.raw(), 1);
    assert_ne!(a, c);
    assert_eq!(b.minted(), 2);
    // a node built after manual minting continues the same sequence (no reuse).
    let lit = b.int("1", SpanV1::new(0, 1));
    assert_eq!(lit.id.raw(), 2);
    assert_eq!(b.minted(), 3);
}
