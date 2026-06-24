#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed indices/variants and panic on unexpected shapes"
)]
//! Positive normalization rails for algebra (ticket 04).
//!
//! syntax->sema->ir path pinned: shared HIR relation/query shapes -> algebra
//! normalization with explicit column provenance and cardinality. Proves:
//! correlated subqueries become explicit Apply nodes with visible correlation
//! (D23); duplicate-sensitive aggregates carry explicit set/bag semantics (D26);
//! a project+filter+sort+limit pipeline normalizes with stable provenance.

use quanta_gqlang_v2_algebra::{
    normalize_query, normalize_relation, AlgebraOpV1, ColumnProvenanceV1, NormalizeOptionsV1,
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
fn correlated_subquery_normalizes_into_explicit_apply_with_visible_correlation() {
    // users u APPLY (orders o WHERE o.user = u.id)
    // The inner orders relation references the outer alias `u`, so the apply is
    // correlated and the correlation column must be VISIBLE.
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
    let inner = b.relation_filter(orders, pred, sp(8, 28));

    let apply = b.compose("apply", vec![users, inner], sp(0, 30));

    let algebra = normalize_relation(&apply, &NormalizeOptionsV1::default())
        .expect("correlated apply must normalize");

    // Top node is an explicit Apply.
    let AlgebraOpV1::Apply(ap) = &algebra.op else {
        panic!("expected Apply, got {:?}", algebra.family());
    };

    // Correlation columns are visible and reference the outer alias `u`.
    assert_eq!(ap.correlation.len(), 1);
    assert_eq!(ap.correlation[0].outer_name, "u");
    assert_eq!(ap.correlation[0].outer_column, 0);

    // The outer column provenance is preserved (Source for `users` via `u`).
    match &algebra.columns[0].provenance {
        ColumnProvenanceV1::Source {
            relation, alias, ..
        } => {
            assert_eq!(relation, "users");
            assert_eq!(alias.as_deref(), Some("u"));
        }
        other => panic!("expected Source provenance, got {other:?}"),
    }

    // The correlation is the VISIBLE carrier: the inner subtree references the
    // outer alias `u` (inside its predicate), so the Apply records it explicitly
    // rather than via a root-local side channel (D23).
    assert!(
        ap.correlation.iter().any(|c| c.outer_name == "u"),
        "correlation must visibly reference outer `u`"
    );

    // Lateral expansion is a bag.
    assert_eq!(algebra.cardinality, CardinalityV1::Bag);
}

#[test]
fn apply_inner_projecting_outer_column_carries_correlated_provenance() {
    // users u APPLY ( orders o | project { u } )  -- the inner side projects the
    // outer alias column `u`, so that OUTPUT column carries Correlated provenance.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", Some(alias("u", 0, 1)), sp(0, 7));

    let orders = b.relation_source("orders", Some(alias("o", 8, 9)), sp(8, 16));
    let u_ref = b.symbol("u", sp(20, 21));
    let item = ProjectionItemV1 {
        value: u_ref,
        alias: Some(alias("u", 22, 23)),
    };
    let inner = b.project(orders, vec![item], sp(8, 24));

    let apply = b.compose("apply", vec![users, inner], sp(0, 30));
    let algebra = normalize_relation(&apply, &NormalizeOptionsV1::default())
        .expect("correlated apply must normalize");

    let correlated = algebra.columns.iter().any(|c| {
        matches!(&c.provenance, ColumnProvenanceV1::Correlated { outer_name, .. } if outer_name == "u")
    });
    assert!(
        correlated,
        "inner output column `u` must carry Correlated provenance"
    );
}

#[test]
fn apply_inner_aliasing_outer_column_to_new_name_carries_correlated_provenance() {
    // users u APPLY ( orders o | project { x: u } )  -- the inner side projects the
    // outer alias column `u` but RENAMES it to `x`. The output column is named `x`
    // yet its VALUE references outer `u`, so it must carry Correlated provenance
    // pointing at outer `u`. Name-matching on the OUTPUT name (`x`) instead of the
    // referenced symbol (`u`) would wrongly leave it as Derived (D23).
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", Some(alias("u", 0, 1)), sp(0, 7));

    let orders = b.relation_source("orders", Some(alias("o", 8, 9)), sp(8, 16));
    let u_ref = b.symbol("u", sp(20, 21));
    let item = ProjectionItemV1 {
        value: u_ref,
        alias: Some(alias("x", 22, 23)),
    };
    let inner = b.project(orders, vec![item], sp(8, 24));

    let apply = b.compose("apply", vec![users, inner], sp(0, 30));
    let algebra = normalize_relation(&apply, &NormalizeOptionsV1::default())
        .expect("correlated apply must normalize");

    // The apply is correlated: the inner references outer `u`.
    let AlgebraOpV1::Apply(ap) = &algebra.op else {
        panic!("expected Apply, got {:?}", algebra.family());
    };
    assert_eq!(ap.correlation.len(), 1);
    assert_eq!(ap.correlation[0].outer_name, "u");

    // The renamed inner output column `x` must carry Correlated provenance whose
    // outer_name is the referenced outer symbol `u`, NOT the output name `x`.
    let renamed = algebra
        .columns
        .iter()
        .find(|c| c.name == "x")
        .expect("renamed inner output column `x` must exist");
    match &renamed.provenance {
        ColumnProvenanceV1::Correlated { outer_name, .. } => {
            assert_eq!(
                outer_name, "u",
                "renamed column `x` references outer `u`, so Correlated outer_name must be `u`"
            );
        }
        other => panic!(
            "renamed inner output column `x` must carry Correlated provenance, got {other:?}"
        ),
    }
}

#[test]
fn duplicate_sensitive_aggregate_makes_set_vs_bag_explicit() {
    // project { region: u.region, n: count(distinct u.id), total: count(u.id) }
    // group key separate from the two count folds; one distinct (set), one not.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", Some(alias("u", 0, 1)), sp(0, 7));

    let region = {
        let base = b.symbol("u", sp(8, 9));
        b.path(base, vec![seg("region", 10, 16)], sp(8, 16))
    };
    let key_item = ProjectionItemV1 {
        value: region,
        alias: Some(alias("region", 18, 24)),
    };

    // count(distinct u.id)
    let distinct_id = {
        let base = b.symbol("u", sp(30, 31));
        let path = b.path(base, vec![seg("id", 32, 34)], sp(30, 34));
        b.call("distinct", sp(25, 33), vec![path], sp(25, 35))
    };
    let count_distinct = b.call("count", sp(20, 24), vec![distinct_id], sp(20, 36));
    let agg_set = ProjectionItemV1 {
        value: count_distinct,
        alias: Some(alias("n", 37, 38)),
    };

    // count(u.id)  -- bag-sensitive
    let plain_id = {
        let base = b.symbol("u", sp(40, 41));
        b.path(base, vec![seg("id", 42, 44)], sp(40, 44))
    };
    let count_plain = b.call("count", sp(45, 49), vec![plain_id], sp(45, 50));
    let agg_bag = ProjectionItemV1 {
        value: count_plain,
        alias: Some(alias("total", 51, 56)),
    };

    let proj = b.project(users, vec![key_item, agg_set, agg_bag], sp(0, 60));

    let opts = NormalizeOptionsV1::with_aggregates(["count"]);
    let algebra = normalize_relation(&proj, &opts).expect("aggregate projection must normalize");

    // Top node is Aggregate over a Group; keys and folds are SEPARATE.
    let AlgebraOpV1::Aggregate(agg) = &algebra.op else {
        panic!("expected Aggregate, got {:?}", algebra.family());
    };
    let AlgebraOpV1::Group(group) = &agg.input.op else {
        panic!("aggregate input must be a Group");
    };

    // Group carries ONLY the region key; folds carry ONLY the aggregates.
    assert_eq!(group.keys.len(), 1, "exactly one grouping key");
    assert_eq!(agg.aggregates.len(), 2, "two aggregate folds");

    // Explicit duplicate-elimination per fold: first distinct (set), second not.
    assert!(
        agg.aggregates[0].distinct,
        "count(distinct) is set-semantic"
    );
    assert!(
        !agg.aggregates[1].distinct,
        "count over a bag preserves duplicates"
    );
    assert_eq!(agg.aggregates[0].function, "count");

    // A grouped aggregate yields one row per group: a Set.
    assert_eq!(algebra.cardinality, CardinalityV1::Set);
    assert_eq!(agg.input.cardinality, CardinalityV1::Set);

    // Output schema: region key + two aggregate columns, in order.
    assert_eq!(algebra.columns.len(), 3);
    assert_eq!(algebra.columns[0].name, "region");
    assert_eq!(algebra.columns[1].name, "n");
    assert_eq!(algebra.columns[2].name, "total");
}

#[test]
fn project_filter_sort_limit_pipeline_has_stable_provenance() {
    // limit( sort( project { id: u.id, score: u.score } ( users u WHERE u.active ) ) )
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", Some(alias("u", 0, 1)), sp(0, 7));

    let active = {
        let base = b.symbol("u", sp(8, 9));
        let path = b.path(base, vec![seg("active", 10, 16)], sp(8, 16));
        b.bool_value(path, sp(8, 16))
    };
    let filtered = b.relation_filter(users, active, sp(0, 16));

    let id_item = {
        let base = b.symbol("u", sp(20, 21));
        let path = b.path(base, vec![seg("id", 22, 24)], sp(20, 24));
        ProjectionItemV1 {
            value: path,
            alias: Some(alias("id", 25, 27)),
        }
    };
    let score_item = {
        let base = b.symbol("u", sp(28, 29));
        let path = b.path(base, vec![seg("score", 30, 35)], sp(28, 35));
        ProjectionItemV1 {
            value: path,
            alias: Some(alias("score", 36, 41)),
        }
    };
    let projected = b.project(filtered, vec![id_item, score_item], sp(0, 42));

    let sorted = b.compose("sort", vec![projected], sp(0, 48));
    let limited = b.compose("limit", vec![sorted], sp(0, 54));

    let query = b.query(limited, None, sp(0, 60));
    let algebra =
        normalize_query(&query, &NormalizeOptionsV1::default()).expect("pipeline must normalize");

    // Top is Limit over Sequence (sort made order semantic).
    let AlgebraOpV1::Limit(limit) = &algebra.op else {
        panic!("expected Limit, got {:?}", algebra.family());
    };
    assert_eq!(algebra.cardinality, CardinalityV1::Sequence);

    let AlgebraOpV1::Sort(sort) = &limit.input.op else {
        panic!("limit input must be Sort");
    };
    assert_eq!(limit.input.cardinality, CardinalityV1::Sequence);
    assert_eq!(sort.keys.len(), 2, "two sort keys mirror the two columns");

    let AlgebraOpV1::Project(_) = &sort.input.op else {
        panic!("sort input must be Project");
    };
    // Project preserved the input bag cardinality (no implicit dedup).
    assert_eq!(sort.input.cardinality, CardinalityV1::Bag);

    let AlgebraOpV1::Filter(filter) = &sort.input.op.project_input().op else {
        panic!("project input must be Filter");
    };
    let AlgebraOpV1::Scan(scan) = &filter.input.op else {
        panic!("filter input must be Scan");
    };
    assert_eq!(scan.relation, "users");
    assert_eq!(scan.alias.as_deref(), Some("u"));

    // Stable provenance: both output columns trace to source `users` via alias
    // `u`, with the right field names, all the way through the pipeline.
    assert_eq!(algebra.columns.len(), 2);
    assert_eq!(algebra.columns[0].name, "id");
    assert_eq!(algebra.columns[1].name, "score");
    for (col, field) in algebra.columns.iter().zip(["id", "score"]) {
        match &col.provenance {
            ColumnProvenanceV1::Source {
                relation,
                alias,
                field: f,
            } => {
                assert_eq!(relation, "users");
                assert_eq!(alias.as_deref(), Some("u"));
                assert_eq!(f.as_deref(), Some(field));
            }
            other => panic!("expected Source provenance, got {other:?}"),
        }
    }
}

// Small helper so the pipeline test can reach through a Project node.
trait ProjectInput {
    fn project_input(&self) -> &quanta_gqlang_v2_algebra::AlgebraNodeV1;
}

impl ProjectInput for AlgebraOpV1 {
    fn project_input(&self) -> &quanta_gqlang_v2_algebra::AlgebraNodeV1 {
        match self {
            AlgebraOpV1::Project(p) => &p.input,
            _ => panic!("not a Project"),
        }
    }
}

#[test]
fn apply_inner_binding_shadows_outer_column_name() {
    // users u APPLY ( users u | project u )
    // The inner subtree binds its OWN alias `u` (its own `users u` source), so the
    // bare `u` it projects resolves to that LOCAL binding, NOT the outer column of
    // the same name. The inner therefore references nothing from the outer side,
    // so this is an uncorrelated apply and must be rejected with MissingCorrelation
    // rather than fabricating a correlation column / Correlated provenance for a
    // name that merely shadows the outer.
    let b = HirBuilderV1::new();

    // Left side: users u
    let left_users = b.relation_source("users", Some(alias("u", 0, 1)), sp(0, 7));

    // Right side: users u | project u  (bare symbol referencing the LOCAL `u`).
    let right_users = b.relation_source("users", Some(alias("u", 8, 9)), sp(8, 16));
    let u_ref = b.symbol("u", sp(20, 21));
    let item = ProjectionItemV1 {
        value: u_ref,
        alias: None,
    };
    let right_proj = b.project(right_users, vec![item], sp(8, 24));

    let apply = b.compose("apply", vec![left_users, right_proj], sp(0, 30));

    let err = normalize_relation(&apply, &NormalizeOptionsV1::default())
        .expect_err("locally-shadowed name is not an outer reference; apply is uncorrelated");
    assert_eq!(err.code(), "algebra.missing_correlation");
    assert_eq!(
        err.message(),
        "correlated apply has no visible correlation columns referencing the outer input"
    );
    assert_eq!(err.span(), sp(0, 30));
}

#[test]
fn unary_limit_uses_caller_supplied_window() {
    // sort( users u ) | limit  — caller supplies the (count, offset) window the
    // structure-only HIR cannot carry. The normalizer must thread it into the
    // emitted LimitV1 rather than hardcoding count=u64::MAX, offset=0.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", Some(alias("u", 0, 1)), sp(0, 7));
    let sorted = b.compose("sort", vec![users], sp(0, 12));
    let limited = b.compose("limit", vec![sorted], sp(0, 18));

    let opts = NormalizeOptionsV1::default().with_limit_window(10, 5);
    let algebra = normalize_relation(&limited, &opts).expect("limit pipeline must normalize");

    let AlgebraOpV1::Limit(limit) = &algebra.op else {
        panic!("expected Limit, got {:?}", algebra.family());
    };
    assert_eq!(limit.count, 10, "count must come from the caller window");
    assert_eq!(limit.offset, 5, "offset must come from the caller window");
}
