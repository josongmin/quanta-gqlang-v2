#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed indices/variants and panic on unexpected shapes"
)]
//! Hellgate (fail-closed) rails for algebra normalization (ticket 04).
//!
//! syntax->sema->ir path pinned: shared HIR relation/query shapes -> algebra
//! normalization. Each test asserts the ACTUAL typed diagnostic code+message
//! emitted by [`quanta_gqlang_v2_algebra`], not merely that an error occurred.
//! These rails fail RED before the normalization impl exists and prove the
//! D10 "no silent downgrade" contract.

use quanta_gqlang_v2_algebra::{
    normalize_relation, AlgebraDiagnosticV1, IllFormedAggregateReasonV1, NormalizeOptionsV1,
};
use quanta_gqlang_v2_hir::{HirBuilderV1, SpanV1};

fn sp(a: usize, b: usize) -> SpanV1 {
    SpanV1::new(a, b)
}

#[test]
fn unsupported_composition_rejects_with_typed_code() {
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", None, sp(0, 5));
    let orders = b.relation_source("orders", None, sp(6, 12));
    let bogus = b.compose("crossapply++", vec![users, orders], sp(0, 30));

    let err = normalize_relation(&bogus, &NormalizeOptionsV1::default())
        .expect_err("unknown composition must reject");
    assert_eq!(err.code(), "algebra.unsupported_composition");
    assert_eq!(
        err.message(),
        "unsupported composition operator `crossapply++` has no algebra lowering"
    );
    assert_eq!(err.span(), sp(0, 30));
}

#[test]
fn join_arity_mismatch_rejects() {
    let b = HirBuilderV1::new();
    let a = b.relation_source("a", None, sp(0, 1));
    let c = b.relation_source("c", None, sp(2, 3));
    let d = b.relation_source("d", None, sp(4, 5));
    let three = b.compose("join", vec![a, c, d], sp(0, 10));

    let err = normalize_relation(&three, &NormalizeOptionsV1::default())
        .expect_err("ternary join must reject");
    assert_eq!(err.code(), "algebra.arity_mismatch");
    assert_eq!(
        err.message(),
        "composition `join` expects 2 operand(s) but found 3"
    );
}

#[test]
fn uncorrelated_apply_rejects_missing_correlation() {
    let b = HirBuilderV1::new();
    let outer = b.relation_source("users", None, sp(0, 5));
    // Inner references nothing from the outer side -> not actually correlated.
    let inner = b.relation_source("orders", None, sp(6, 12));
    let apply = b.compose("apply", vec![outer, inner], sp(0, 20));

    let err = normalize_relation(&apply, &NormalizeOptionsV1::default())
        .expect_err("uncorrelated apply must reject");
    assert_eq!(err.code(), "algebra.missing_correlation");
    assert_eq!(
        err.message(),
        "correlated apply has no visible correlation columns referencing the outer input"
    );
    assert_eq!(err.span(), sp(0, 20));
}

#[test]
fn limit_over_unordered_input_rejects() {
    let b = HirBuilderV1::new();
    // A bare source is a BAG; limiting it without a sort is non-deterministic.
    let users = b.relation_source("users", None, sp(0, 5));
    let limit = b.compose("limit", vec![users], sp(0, 12));

    let err = normalize_relation(&limit, &NormalizeOptionsV1::default())
        .expect_err("limit over a bag must reject");
    assert_eq!(err.code(), "algebra.unordered_limit");
    assert_eq!(
        err.message(),
        "limit requires an ordered input; sort before limiting"
    );
}

#[test]
fn nested_aggregate_rejects_ill_formed() {
    // project { total: sum(x) + 1 } -- aggregate nested inside a binary expr.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", None, sp(0, 5));
    let x = b.symbol("x", sp(10, 11));
    let sum = b.call("sum", sp(6, 9), vec![x], sp(6, 12));
    let one = b.int("1", sp(15, 16));
    let nested = b.binary("+", sum, one, sp(6, 16));
    let item = quanta_gqlang_v2_hir::ProjectionItemV1 {
        value: nested,
        alias: None,
    };
    let proj = b.project(users, vec![item], sp(0, 20));

    let opts = NormalizeOptionsV1::with_aggregates(["sum"]);
    let err = normalize_relation(&proj, &opts).expect_err("nested aggregate must reject");
    assert_eq!(err.code(), "algebra.ill_formed_aggregate.nested_aggregate");
    assert_eq!(
        err.message(),
        "aggregate call must be a top-level projection item, not nested in an expression"
    );
    match err {
        AlgebraDiagnosticV1::IllFormedAggregate { reason, .. } => {
            assert_eq!(reason, IllFormedAggregateReasonV1::NestedAggregate);
        }
        other => panic!("expected IllFormedAggregate, got {other:?}"),
    }
}

#[test]
fn union_arity_mismatch_rejects() {
    let b = HirBuilderV1::new();
    // left projects one column, right projects two -> union-incompatible.
    let l_src = b.relation_source("a", None, sp(0, 1));
    let l_id = {
        let base = b.symbol("a", sp(2, 3));
        b.path(base, vec![seg("id", &b, 4, 6)], sp(2, 6))
    };
    let left = b.project(
        l_src,
        vec![quanta_gqlang_v2_hir::ProjectionItemV1 {
            value: l_id,
            alias: None,
        }],
        sp(0, 7),
    );
    let r_src = b.relation_source("c", None, sp(8, 9));
    let r_id = {
        let base = b.symbol("c", sp(10, 11));
        b.path(base, vec![seg("id", &b, 12, 14)], sp(10, 14))
    };
    let r_name = {
        let base = b.symbol("c", sp(15, 16));
        b.path(base, vec![seg("name", &b, 17, 21)], sp(15, 21))
    };
    let right = b.project(
        r_src,
        vec![
            quanta_gqlang_v2_hir::ProjectionItemV1 {
                value: r_id,
                alias: None,
            },
            quanta_gqlang_v2_hir::ProjectionItemV1 {
                value: r_name,
                alias: None,
            },
        ],
        sp(8, 22),
    );
    let union = b.compose("union", vec![left, right], sp(0, 30));

    let err = normalize_relation(&union, &NormalizeOptionsV1::default())
        .expect_err("union arity mismatch must reject");
    assert_eq!(err.code(), "algebra.union_arity");
    assert_eq!(
        err.message(),
        "union arms are not compatible: left has 1 column(s), right has 2"
    );
}

#[test]
fn union_column_name_mismatch_rejects() {
    // Both arms project exactly ONE column (arity matches), but the left names it
    // `id` and the right names it `name`. Same count, disagreeing names: the merged
    // schema must not silently adopt the left name. Reject with a typed code that
    // is DISTINCT from the arity diagnostic.
    let b = HirBuilderV1::new();
    let l_src = b.relation_source("a", None, sp(0, 1));
    let l_id = {
        let base = b.symbol("a", sp(2, 3));
        b.path(base, vec![seg("id", &b, 4, 6)], sp(2, 6))
    };
    let left = b.project(
        l_src,
        vec![quanta_gqlang_v2_hir::ProjectionItemV1 {
            value: l_id,
            alias: None,
        }],
        sp(0, 7),
    );
    let r_src = b.relation_source("c", None, sp(8, 9));
    let r_name = {
        let base = b.symbol("c", sp(10, 11));
        b.path(base, vec![seg("name", &b, 12, 16)], sp(10, 16))
    };
    let right = b.project(
        r_src,
        vec![quanta_gqlang_v2_hir::ProjectionItemV1 {
            value: r_name,
            alias: None,
        }],
        sp(8, 17),
    );
    let union = b.compose("union", vec![left, right], sp(0, 30));

    let err = normalize_relation(&union, &NormalizeOptionsV1::default())
        .expect_err("union with positionally-disagreeing column names must reject");
    assert_eq!(err.code(), "algebra.union_column_mismatch");
    assert_eq!(
        err.message(),
        "union arms disagree on column 0: left names it `id`, right names it `name`"
    );
    assert_eq!(err.span(), sp(0, 30));
}

#[test]
fn join_without_predicate_rejects_empty_predicate() {
    // `from users u | join orders o => { }` -- a `join` (NOT a `cross`) whose right
    // operand is a bare source, so no ON/where filter wraps it. The author asked for
    // a join but supplied no join condition. Fail-closed (D10): this must NOT be
    // silently downgraded to a cross/cartesian product (which `predicate = None`
    // would otherwise mean). Reject with a typed, DISTINCT diagnostic.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", None, sp(0, 5));
    let orders = b.relation_source("orders", None, sp(6, 12));
    let join = b.compose("join", vec![users, orders], sp(0, 20));

    let err = normalize_relation(&join, &NormalizeOptionsV1::default())
        .expect_err("join without a predicate must reject");
    assert_eq!(err.code(), "algebra.empty_join_predicate");
    assert_eq!(
        err.message(),
        "join requires an ON/where predicate; use `cross` for a cartesian product"
    );
    assert_eq!(err.span(), sp(0, 20));
    match err {
        AlgebraDiagnosticV1::EmptyJoinPredicate { symbol, .. } => assert_eq!(symbol, "join"),
        other => panic!("expected EmptyJoinPredicate, got {other:?}"),
    }
}

#[test]
fn explicit_cross_join_still_accepted() {
    // The DISTINCTION the fix must preserve: an EXPLICIT `cross` over two bare
    // sources is a legitimate cartesian product and must still normalize to a Join
    // with `predicate = None`. Only the predicate-less `join` is rejected.
    use quanta_gqlang_v2_algebra::AlgebraOpV1;

    let b = HirBuilderV1::new();
    let users = b.relation_source("users", None, sp(0, 5));
    let orders = b.relation_source("orders", None, sp(6, 12));
    let cross = b.compose("cross", vec![users, orders], sp(0, 20));

    let node = normalize_relation(&cross, &NormalizeOptionsV1::default())
        .expect("explicit cross join must be accepted");
    let AlgebraOpV1::Join(join) = &node.op else {
        panic!("cross must lower to a Join node");
    };
    assert!(
        join.predicate.is_none(),
        "an explicit cross is a cartesian product with no predicate"
    );
}

fn seg(name: &str, _b: &HirBuilderV1, a: usize, z: usize) -> quanta_gqlang_v2_hir::PathSegmentV1 {
    quanta_gqlang_v2_hir::PathSegmentV1 {
        name: name.to_owned(),
        span: SpanV1::new(a, z),
    }
}
