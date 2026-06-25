#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "mutation-pinning rails assert on fixed indices/variants and panic on unexpected shapes"
)]
//! Mutation-pinning rails for algebra (kills surviving cargo-mutants mutants).
//!
//! Each test pins an EXACT observable behavior so a specific code mutation would
//! flip it red:
//! - `AlgebraFamilyV1::keyword` / `SortDirectionV1::keyword` return the precise
//!   stable keyword strings (kills `-> ""` and `-> "xyzzy"`).
//! - a non-aggregate call stays a plain `Project`, never an `Aggregate`
//!   (kills `is_aggregate -> true`).
//! - a `join` whose right operand carries a filter predicate KEEPS that predicate
//!   (kills `join_predicate -> None`).
//! - a correlated apply matching the RIGHT outer column by name records the
//!   correct outer-column index (kills `== -> !=` in the correlation lookup).
//! - `order by <key> desc` lowers to `Descending` (kills deleting the
//!   `Some("desc")` match arm in `sort_direction_of`).

use quanta_gqlang_v2_algebra::{
    normalize_relation, AlgebraFamilyV1, AlgebraOpV1, ColumnProvenanceV1, NormalizeOptionsV1,
    SortDirectionV1,
};
use quanta_gqlang_v2_hir::{AliasV1, HirBuilderV1, PathSegmentV1, ProjectionItemV1, SpanV1};

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
fn algebra_family_keywords_are_exact_stable_strings() {
    // Pins node.rs AlgebraFamilyV1::keyword: each family maps to its EXACT
    // lowercase keyword. Kills `keyword -> ""` and `keyword -> "xyzzy"`.
    assert_eq!(AlgebraFamilyV1::Scan.keyword(), "scan");
    assert_eq!(AlgebraFamilyV1::Project.keyword(), "project");
    assert_eq!(AlgebraFamilyV1::Filter.keyword(), "filter");
    assert_eq!(AlgebraFamilyV1::Join.keyword(), "join");
    assert_eq!(AlgebraFamilyV1::Apply.keyword(), "apply");
    assert_eq!(AlgebraFamilyV1::Group.keyword(), "group");
    assert_eq!(AlgebraFamilyV1::Aggregate.keyword(), "aggregate");
    assert_eq!(AlgebraFamilyV1::Sort.keyword(), "sort");
    assert_eq!(AlgebraFamilyV1::Limit.keyword(), "limit");
    assert_eq!(AlgebraFamilyV1::Union.keyword(), "union");
    assert_eq!(AlgebraFamilyV1::SemiJoin.keyword(), "semijoin");
    assert_eq!(AlgebraFamilyV1::AntiJoin.keyword(), "antijoin");
}

#[test]
fn sort_direction_keywords_are_exact_stable_strings() {
    // Pins node.rs SortDirectionV1::keyword. Kills `keyword -> ""` and
    // `keyword -> "xyzzy"`.
    assert_eq!(SortDirectionV1::Ascending.keyword(), "asc");
    assert_eq!(SortDirectionV1::Descending.keyword(), "desc");
}

#[test]
fn projection_with_non_aggregate_call_stays_a_plain_project() {
    // project { y: lower(u.name) } over users, where `lower` is NOT registered as
    // an aggregate. A non-aggregate call must classify as a plain key, so the
    // result is a Project node (no Group/Aggregate split). The mutation
    // `is_aggregate -> true` would treat EVERY call as an aggregate, lowering this
    // to an Aggregate node instead.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", Some(alias("u", 0, 1)), sp(0, 7));
    let name = {
        let base = b.symbol("u", sp(8, 9));
        b.path(base, vec![seg("name", 10, 14)], sp(8, 14))
    };
    let lower = b.call("lower", sp(16, 21), vec![name], sp(16, 24));
    let item = ProjectionItemV1 {
        value: lower,
        alias: Some(alias("y", 25, 26)),
    };
    let proj = b.project(users, vec![item], sp(0, 30));

    // Aggregate set deliberately EXCLUDES `lower`.
    let opts = NormalizeOptionsV1::with_aggregates(["count", "sum"]);
    let algebra =
        normalize_relation(&proj, &opts).expect("non-aggregate projection must normalize");

    match &algebra.op {
        AlgebraOpV1::Project(_) => {}
        AlgebraOpV1::Aggregate(_) => {
            panic!("a non-aggregate call must NOT be lowered to an Aggregate node")
        }
        other => panic!("expected a plain Project, got {other:?}"),
    }
}

#[test]
fn join_with_filter_predicate_keeps_the_predicate() {
    // users u | join (orders o WHERE o.user = u.id)
    // The right operand is a Filter carrying the join condition. The join must
    // normalize AND keep that predicate. The mutation `join_predicate -> None`
    // would drop it, making `require_join_predicate` reject a valid join.
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
    let right = b.relation_filter(orders, pred, sp(8, 28));

    let join = b.compose("join", vec![users, right], sp(0, 30));

    let algebra = normalize_relation(&join, &NormalizeOptionsV1::default())
        .expect("a join WITH a filter predicate must normalize, not be dropped");

    let AlgebraOpV1::Join(join_op) = &algebra.op else {
        panic!("expected Join, got {:?}", algebra.family());
    };
    assert!(
        join_op.predicate.is_some(),
        "the join condition from the right Filter MUST be preserved, not dropped"
    );
}

#[test]
fn correlated_apply_resolves_the_correct_outer_column_index() {
    // Left: data d | project { a: d.x, b: d.y }  -> outer columns [a (idx 0), b (idx 1)]
    // Right: orders o | project { out: b }       -> output column references outer `b`
    //
    // The single correlation column is `b` at outer index 1. The renamed inner
    // output column `out` references outer `b`, so its Correlated provenance must
    // carry outer_column = 1. The mutation `== -> !=` in the correlation lookup
    // would match the FIRST entry whose name is NOT `b` (here: none), falling back
    // to the default index 0 -> wrong outer_column.
    let b = HirBuilderV1::new();

    // Left: data d | project { a: d.x, b: d.y }
    let data = b.relation_source("data", Some(alias("d", 0, 1)), sp(0, 7));
    let d_x = {
        let base = b.symbol("d", sp(8, 9));
        b.path(base, vec![seg("x", 10, 11)], sp(8, 11))
    };
    let d_y = {
        let base = b.symbol("d", sp(12, 13));
        b.path(base, vec![seg("y", 14, 15)], sp(12, 15))
    };
    let a_item = ProjectionItemV1 {
        value: d_x,
        alias: Some(alias("a", 16, 17)),
    };
    let b_item = ProjectionItemV1 {
        value: d_y,
        alias: Some(alias("b", 18, 19)),
    };
    let left = b.project(data, vec![a_item, b_item], sp(0, 20));

    // Right: orders o | project { out: b }  -- `b` is a bare symbol referencing the
    // OUTER column `b` (outer index 1), not anything local.
    let orders = b.relation_source("orders", Some(alias("o", 21, 22)), sp(21, 28));
    let b_ref = b.symbol("b", sp(29, 30));
    let out_item = ProjectionItemV1 {
        value: b_ref,
        alias: Some(alias("out", 31, 34)),
    };
    let right = b.project(orders, vec![out_item], sp(21, 35));

    let apply = b.compose("apply", vec![left, right], sp(0, 40));
    let algebra = normalize_relation(&apply, &NormalizeOptionsV1::default())
        .expect("correlated apply must normalize");

    let AlgebraOpV1::Apply(ap) = &algebra.op else {
        panic!("expected Apply, got {:?}", algebra.family());
    };
    // Exactly one correlation column: outer `b` at index 1.
    assert_eq!(ap.correlation.len(), 1);
    assert_eq!(ap.correlation[0].outer_name, "b");
    assert_eq!(ap.correlation[0].outer_column, 1);

    // The inner output column `out` references outer `b`, so it carries Correlated
    // provenance pointing at outer_column 1 (NOT the default 0).
    let out_col = algebra
        .columns
        .iter()
        .find(|c| c.name == "out")
        .expect("inner output column `out` must exist");
    match &out_col.provenance {
        ColumnProvenanceV1::Correlated {
            outer_column,
            outer_name,
        } => {
            assert_eq!(outer_name, "b");
            assert_eq!(
                *outer_column, 1,
                "the correlation lookup must resolve outer `b` to its real index 1"
            );
        }
        other => panic!("output column `out` must be Correlated to outer `b`, got {other:?}"),
    }
}

#[test]
fn order_by_desc_lowers_to_descending() {
    // order_by( users u, project { score: u.score (alias `desc`) } )
    // The sort-key item alias `desc` is the open tag for descending order. Decoding
    // it must yield SortDirectionV1::Descending. Deleting the `Some("desc")` match
    // arm in `sort_direction_of` would default it to Ascending.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", Some(alias("u", 0, 1)), sp(0, 7));

    let score = {
        let base = b.symbol("u", sp(8, 9));
        b.path(base, vec![seg("score", 10, 15)], sp(8, 15))
    };
    // The direction tag is folded into the key item's alias: `desc`.
    let key_item = ProjectionItemV1 {
        value: score,
        alias: Some(alias("desc", 16, 20)),
    };
    let keys = b.project(users.clone(), vec![key_item], sp(8, 21));

    let order = b.compose("order_by", vec![users, keys], sp(0, 30));
    let algebra = normalize_relation(&order, &NormalizeOptionsV1::default())
        .expect("order_by must normalize");

    let AlgebraOpV1::Sort(sort) = &algebra.op else {
        panic!("expected Sort, got {:?}", algebra.family());
    };
    assert_eq!(sort.keys.len(), 1, "exactly one sort key");
    assert_eq!(
        sort.keys[0].direction,
        SortDirectionV1::Descending,
        "an `order by ... desc` key must lower to Descending, not the Ascending default"
    );
}
