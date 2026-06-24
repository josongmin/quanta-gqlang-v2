#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "audit rails assert on fixed indices/variants and panic on unexpected shapes"
)]
//! Punch-list audit rails for algebra normalization.
//!
//! These pin the contract for several self-audit findings: scalar-aggregate
//! cardinality (a no-key aggregate is a Set, never a Bag); positional union name
//! AGREEMENT (matching names merge cleanly, the mismatch is covered in hellgate);
//! and Apply provenance distinctness under outer-name shadowing (an inner column
//! that merely shares a NAME with an outer column, but references a local source,
//! keeps its own Source provenance and is not mislabelled Correlated).

use quanta_gqlang_v2_algebra::{
    normalize_relation, AlgebraOpV1, ColumnProvenanceV1, NormalizeOptionsV1,
};
use quanta_gqlang_v2_hir::{AliasV1, HirBuilderV1, PathSegmentV1, ProjectionItemV1, SpanV1};
use quanta_gqlang_v2_types::CardinalityV1;

fn sp(a: usize, b: usize) -> SpanV1 {
    SpanV1::new(a, b)
}
fn seg(name: &str, a: usize, z: usize) -> PathSegmentV1 {
    PathSegmentV1 {
        name: name.to_owned(),
        span: sp(a, z),
    }
}
fn alias(name: &str, a: usize, z: usize) -> AliasV1 {
    AliasV1 {
        name: name.to_owned(),
        span: sp(a, z),
    }
}

#[test]
fn scalar_aggregate_with_no_group_keys_is_a_set_not_a_bag() {
    // project { n: count(u.id) } over users u -- NO grouping keys (scalar agg).
    // A scalar aggregate folds the whole input into a single row, so its
    // cardinality is a Set (one row), NOT the input's Bag. The empty-key Group it
    // builds is the legitimate scalar-aggregate path (distinct from an explicit
    // `group_by` with no keys, which is rejected as ill-formed).
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", Some(alias("u", 0, 1)), sp(0, 7));
    let id = {
        let base = b.symbol("u", sp(8, 9));
        b.path(base, vec![seg("id", 10, 12)], sp(8, 12))
    };
    let count = b.call("count", sp(13, 18), vec![id], sp(13, 20));
    let item = ProjectionItemV1 {
        value: count,
        alias: Some(alias("n", 21, 22)),
    };
    let proj = b.project(users, vec![item], sp(0, 30));
    let opts = NormalizeOptionsV1::with_aggregates(["count"]);
    let algebra = normalize_relation(&proj, &opts).expect("scalar agg must normalize");

    assert_eq!(
        algebra.cardinality,
        CardinalityV1::Set,
        "scalar aggregate yields exactly one row: a Set, never a Bag"
    );
    let AlgebraOpV1::Aggregate(agg) = &algebra.op else {
        panic!("expected Aggregate, got {:?}", algebra.family());
    };
    let AlgebraOpV1::Group(group) = &agg.input.op else {
        panic!("aggregate input must be a Group (the scalar-agg path)");
    };
    assert_eq!(group.keys.len(), 0, "scalar aggregate has no grouping keys");
    assert_eq!(
        agg.input.cardinality,
        CardinalityV1::Set,
        "the no-key Group is a Set of one (empty) partition tuple"
    );
}

#[test]
fn union_arms_agreeing_on_names_merge_with_unioned_provenance() {
    // Both arms project a column NAMED `id`: names agree position-for-position, so
    // the union is accepted and the merged column carries Unioned provenance.
    let b = HirBuilderV1::new();
    let l_src = b.relation_source("a", None, sp(0, 1));
    let l_id = {
        let base = b.symbol("a", sp(2, 3));
        b.path(base, vec![seg("id", 4, 6)], sp(2, 6))
    };
    let left = b.project(
        l_src,
        vec![ProjectionItemV1 {
            value: l_id,
            alias: None,
        }],
        sp(0, 7),
    );
    let r_src = b.relation_source("c", None, sp(8, 9));
    let r_id = {
        let base = b.symbol("c", sp(10, 11));
        b.path(base, vec![seg("id", 12, 14)], sp(10, 14))
    };
    let right = b.project(
        r_src,
        vec![ProjectionItemV1 {
            value: r_id,
            alias: None,
        }],
        sp(8, 15),
    );
    let union = b.compose("union", vec![left, right], sp(0, 30));
    let algebra =
        normalize_relation(&union, &NormalizeOptionsV1::default()).expect("name-agreeing union ok");

    assert_eq!(algebra.columns.len(), 1);
    assert_eq!(algebra.columns[0].name, "id");
    assert!(
        matches!(
            algebra.columns[0].provenance,
            ColumnProvenanceV1::Unioned { .. }
        ),
        "merged column records both arms via Unioned provenance"
    );
    // `union` (distinct) is set semantics.
    assert_eq!(algebra.cardinality, CardinalityV1::Set);
}

#[test]
fn apply_inner_column_sharing_outer_name_keeps_distinct_local_provenance() {
    // users u APPLY ( orders o WHERE o.user = u.id | project { u: o.amount } )
    // The inner output column is NAMED `u` (same name as the outer column) but its
    // VALUE references local `o.amount`. Provenance must keep the two `u` columns
    // distinct: the outer stays Source(users), the inner stays Source(orders), and
    // the inner is NOT mislabelled Correlated just because it shares the name `u`.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", Some(alias("u", 0, 1)), sp(0, 7));
    let orders = b.relation_source("orders", Some(alias("o", 8, 9)), sp(8, 16));
    let o_user = {
        let base = b.symbol("o", sp(17, 18));
        b.path(base, vec![seg("user", 19, 23)], sp(17, 23))
    };
    let u_id = {
        let base = b.symbol("u", sp(24, 25));
        b.path(base, vec![seg("id", 26, 28)], sp(24, 28))
    };
    let pred = b.compare("=", o_user, u_id, sp(17, 28));
    let filtered = b.relation_filter(orders, pred, sp(8, 28));
    let o_amount = {
        let base = b.symbol("o", sp(29, 30));
        b.path(base, vec![seg("amount", 31, 37)], sp(29, 37))
    };
    let item = ProjectionItemV1 {
        value: o_amount,
        alias: Some(alias("u", 38, 39)),
    };
    let inner = b.project(filtered, vec![item], sp(8, 40));
    let apply = b.compose("apply", vec![users, inner], sp(0, 50));
    let algebra = normalize_relation(&apply, &NormalizeOptionsV1::default())
        .expect("correlated apply must normalize");

    assert_eq!(algebra.columns.len(), 2, "outer `u` + inner `u`");
    // Both columns are named `u`, yet provenance distinguishes them.
    assert_eq!(algebra.columns[0].name, "u");
    assert_eq!(algebra.columns[1].name, "u");
    match &algebra.columns[0].provenance {
        ColumnProvenanceV1::Source { relation, .. } => assert_eq!(relation, "users"),
        other => panic!("outer column must stay Source(users), got {other:?}"),
    }
    match &algebra.columns[1].provenance {
        ColumnProvenanceV1::Source {
            relation, field, ..
        } => {
            assert_eq!(relation, "orders");
            assert_eq!(field.as_deref(), Some("amount"));
        }
        other => panic!(
            "inner column references local `o.amount`, must stay Source(orders), not Correlated; \
             got {other:?}"
        ),
    }
}
