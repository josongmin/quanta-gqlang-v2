//! v3 seam rail: surface(06) -> algebra(04) `order by` lowering.
//!
//! The surface parser emits `Compose("order_by", [input, keys])` where `keys`
//! is a `Project` carrying the sort-key value exprs and a sort direction folded
//! into each item alias (`asc`/`desc`). Before the fix the algebra normalizer
//! had no lowering for the `order_by` symbol and rejected with
//! `algebra.unsupported_composition`. This rail pins the end-to-end behavior:
//! `from findings f | order by f.id` must compile through every stage and
//! produce an explicit `Sort` algebra root whose keys carry the ordering value
//! expressions and decoded directions end-to-end, over the real producer.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration test rail: assertion panics and indexing are intentional"
)]

use quanta_gqlang_v2_algebra::{AlgebraOpV1, SortDirectionV1};
use quanta_gqlang_v2_hir::ValueExprKindV1;
use quanta_gqlang_v2_pipeline::compile_program;
use quanta_gqlang_v2_types::CardinalityV1;

/// rail: a single-key `order by` compiles end to end into an explicit Sort
/// whose key value expression is preserved (the `f.id` path), direction
/// defaults to ascending, the Sort input is the real Scan (not the synthetic
/// `_keys` placeholder), and the ordered output is a Sequence.
#[test]
fn order_by_lowers_to_explicit_sort_with_carried_key() {
    let src = "from findings f | order by f.id";
    let compiled = compile_program(src).expect("order by compiles end to end");

    let root = compiled
        .algebra()
        .expect("relational query has an algebra root");

    let AlgebraOpV1::Sort(sort) = &root.op else {
        panic!("order by must lower to an explicit Sort, got {:?}", root.op);
    };

    // The ordering key value expression is carried end-to-end: it is the
    // `f.id` path written in the surface, not dropped or re-synthesized.
    assert_eq!(sort.keys.len(), 1, "exactly one sort key");
    let key = &sort.keys[0];
    assert_eq!(
        key.direction,
        SortDirectionV1::Ascending,
        "default direction is ascending"
    );
    let ValueExprKindV1::Path(path) = &key.expr.kind else {
        panic!("sort key must be the `f.id` path, got {:?}", key.expr.kind);
    };
    assert_eq!(
        path.segments.last().map(|s| s.name.as_str()),
        Some("id"),
        "key path resolves to the `id` field"
    );

    // The Sort input is the underlying Scan over `findings`, NOT the synthetic
    // `_keys` placeholder source the surface attaches to the keys projection.
    let AlgebraOpV1::Scan(scan) = &sort.input.op else {
        panic!("sort input must be the scan, got {:?}", sort.input.op);
    };
    assert_eq!(scan.relation, "findings");

    // Sort preserves the input schema and makes order semantic: a Sequence.
    assert_eq!(root.cardinality, CardinalityV1::Sequence);
    assert_eq!(root.columns.len(), 1, "sort preserves the input schema");
}

/// rail: a `desc` direction is decoded from the key alias tag, and a multi-key
/// `order by` preserves key order (more significant key first).
#[test]
fn order_by_decodes_directions_and_preserves_key_order() {
    let src = "from findings f | order by f.severity desc, f.id asc";
    let compiled = compile_program(src).expect("order by compiles end to end");

    let root = compiled
        .algebra()
        .expect("relational query has an algebra root");

    let AlgebraOpV1::Sort(sort) = &root.op else {
        panic!("order by must lower to an explicit Sort, got {:?}", root.op);
    };

    assert_eq!(sort.keys.len(), 2, "two sort keys");
    assert_eq!(
        sort.keys[0].direction,
        SortDirectionV1::Descending,
        "first key is descending (decoded from `desc` tag)"
    );
    assert_eq!(
        sort.keys[1].direction,
        SortDirectionV1::Ascending,
        "second key is ascending"
    );

    let ValueExprKindV1::Path(path0) = &sort.keys[0].expr.kind else {
        panic!("first key must be a path, got {:?}", sort.keys[0].expr.kind);
    };
    assert_eq!(
        path0.segments.last().map(|s| s.name.as_str()),
        Some("severity")
    );
    let ValueExprKindV1::Path(path1) = &sort.keys[1].expr.kind else {
        panic!(
            "second key must be a path, got {:?}",
            sort.keys[1].expr.kind
        );
    };
    assert_eq!(path1.segments.last().map(|s| s.name.as_str()), Some("id"));
}
