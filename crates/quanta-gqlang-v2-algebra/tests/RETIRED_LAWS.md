# Retired algebra laws — awaiting re-seat (ticket 05)

These integration laws were removed from the compiling algebra suites because the
HIR reconstruction removed the relation constructs they exercised. They are kept
verbatim here so ticket 05 can re-seat them against the sealed Sema-input
contract instead of re-deriving them.

## Why they cannot compile

`RelationExprKindV1` now has exactly six variants — `Source`, `Project`,
`Filter`, `Join`, `GroupBy`, `OrderBy` — and `HirBuilderV1` no longer exposes
`compose`/`path`/`project`/`quantifier`. The surface grammar agrees: relation
stages are exactly `where`, `project`, `join`, `cross`, `group`, `order`
(`crates/quanta-gqlang-v2-surface/src/query.rs:105-110`). Therefore `union`,
`limit`, `apply`, and symbol-named composition are **not authorable**, and
`algebra/src/normalize.rs` keeps their handlers only as the `#[allow(dead_code)]`
chain rooted at `normalize_compose` (`normalize.rs:451`, no caller —
`CompositionV1` at `:151` is never constructed). rustc confirms 24 never-used
items in that crate.

## Disposition per law

| law | construct | disposition |
|---|---|---|
| union laws (3) | `union` / `union_all` | **permanent retirement** — ticket 05 forbids making Limit/Union source-reachable |
| limit laws (2) | `limit` (+ `with_limit_window`) | **permanent retirement** — same clause; `limit_window` sidecar loses its only consumer |
| composition symbol / arity (2) | `crossapply++`, 3-operand `join` | **permanent retirement** — closed HIR fixes the operator set and arity by construction |
| correlation / Apply laws (8) | `apply` | **temporary** — ticket 05 re-seats these on the sealed predicate (`Implement … EXISTS semi/anti … over the six typed stages`) |
| predicate-less `join` (1) | `join` without `on` | **structural**: `JoinHirV1::Inner` requires `on: BoolExprHirV1`, so the law is now a type-level fact |

Note the surviving invariants are still covered: aggregate set/bag
(`duplicate_sensitive_aggregate_makes_set_vs_bag_explicit`, `scalar_aggregate_…`),
projection (`projection_with_non_aggregate_call_stays_a_plain_project`), the
closed relation authority rails (4), the exact keyword rails (2), cross join
(`explicit_cross_join_still_accepted`), and ill-formed aggregates.

---

## `v3_algebra_hellgate.rs` :: `join_without_predicate_rejects_empty_predicate`

```rust
#[test]
fn join_without_predicate_rejects_empty_predicate() {
    // `from users u | join orders o => { }` -- a `join` (NOT a `cross`) whose right
    // operand is a bare source, so no ON/where filter wraps it. The author asked for
    // a join but supplied no join condition. Fail-closed (D10): this must NOT be
    // silently downgraded to a cross/cartesian product (which `predicate = None`
    // would otherwise mean). Reject with a typed, DISTINCT diagnostic.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", sp(0, 5), None, sp(0, 5));
    let orders = b.relation_source("orders", sp(6, 12), None, sp(6, 12));
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
```

## `v3_algebra_hellgate.rs` :: `union_column_name_mismatch_rejects`

```rust
#[test]
fn union_column_name_mismatch_rejects() {
    // Both arms project exactly ONE column (arity matches), but the left names it
    // `id` and the right names it `name`. Same count, disagreeing names: the merged
    // schema must not silently adopt the left name. Reject with a typed code that
    // is DISTINCT from the arity diagnostic.
    let b = HirBuilderV1::new();
    let l_src = b.relation_source("a", sp(0, 1), None, sp(0, 1));
    let l_id = {
        let base = b.symbol("a", sp(2, 3));
        b.try_path(base, vec![seg("id", &b, 4, 6)], sp(2, 6)).expect("path requires at least one segment")
    };
    let left = b.try_project(
        l_src,
        vec![quanta_gqlang_v2_hir::ProjectionItemV1 {
            value: l_id,
            alias: None,
        }],
        sp(0, 7),
    ).expect("projection requires at least one item");
    let r_src = b.relation_source("c", sp(8, 9), None, sp(8, 9));
    let r_name = {
        let base = b.symbol("c", sp(10, 11));
        b.try_path(base, vec![seg("name", &b, 12, 16)], sp(10, 16)).expect("path requires at least one segment")
    };
    let right = b.try_project(
        r_src,
        vec![quanta_gqlang_v2_hir::ProjectionItemV1 {
            value: r_name,
            alias: None,
        }],
        sp(8, 17),
    ).expect("projection requires at least one item");
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
```

## `v3_algebra_hellgate.rs` :: `union_arity_mismatch_rejects`

```rust
#[test]
fn union_arity_mismatch_rejects() {
    let b = HirBuilderV1::new();
    // left projects one column, right projects two -> union-incompatible.
    let l_src = b.relation_source("a", sp(0, 1), None, sp(0, 1));
    let l_id = {
        let base = b.symbol("a", sp(2, 3));
        b.try_path(base, vec![seg("id", &b, 4, 6)], sp(2, 6)).expect("path requires at least one segment")
    };
    let left = b.try_project(
        l_src,
        vec![quanta_gqlang_v2_hir::ProjectionItemV1 {
            value: l_id,
            alias: None,
        }],
        sp(0, 7),
    ).expect("projection requires at least one item");
    let r_src = b.relation_source("c", sp(8, 9), None, sp(8, 9));
    let r_id = {
        let base = b.symbol("c", sp(10, 11));
        b.try_path(base, vec![seg("id", &b, 12, 14)], sp(10, 14)).expect("path requires at least one segment")
    };
    let r_name = {
        let base = b.symbol("c", sp(15, 16));
        b.try_path(base, vec![seg("name", &b, 17, 21)], sp(15, 21)).expect("path requires at least one segment")
    };
    let right = b.try_project(
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
    ).expect("projection requires at least one item");
    let union = b.compose("union", vec![left, right], sp(0, 30));

    let err = normalize_relation(&union, &NormalizeOptionsV1::default())
        .expect_err("union arity mismatch must reject");
    assert_eq!(err.code(), "algebra.union_arity");
    assert_eq!(
        err.message(),
        "union arms are not compatible: left has 1 column(s), right has 2"
    );
}
```

## `v3_algebra_hellgate.rs` :: `limit_over_unordered_input_rejects`

```rust
#[test]
fn limit_over_unordered_input_rejects() {
    let b = HirBuilderV1::new();
    // A bare source is a BAG; limiting it without a sort is non-deterministic.
    let users = b.relation_source("users", sp(0, 5), None, sp(0, 5));
    let limit = b.compose("limit", vec![users], sp(0, 12));

    let err = normalize_relation(&limit, &NormalizeOptionsV1::default())
        .expect_err("limit over a bag must reject");
    assert_eq!(err.code(), "algebra.unordered_limit");
    assert_eq!(
        err.message(),
        "limit requires an ordered input; sort before limiting"
    );
}
```

## `v3_algebra_hellgate.rs` :: `uncorrelated_apply_rejects_missing_correlation`

```rust
#[test]
fn uncorrelated_apply_rejects_missing_correlation() {
    let b = HirBuilderV1::new();
    let outer = b.relation_source("users", sp(0, 5), None, sp(0, 5));
    // Inner references nothing from the outer side -> not actually correlated.
    let inner = b.relation_source("orders", sp(6, 12), None, sp(6, 12));
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
```

## `v3_algebra_hellgate.rs` :: `join_arity_mismatch_rejects`

```rust
#[test]
fn join_arity_mismatch_rejects() {
    let b = HirBuilderV1::new();
    let a = b.relation_source("a", sp(0, 1), None, sp(0, 1));
    let c = b.relation_source("c", sp(2, 3), None, sp(2, 3));
    let d = b.relation_source("d", sp(4, 5), None, sp(4, 5));
    let three = b.compose("join", vec![a, c, d], sp(0, 10));

    let err = normalize_relation(&three, &NormalizeOptionsV1::default())
        .expect_err("ternary join must reject");
    assert_eq!(err.code(), "algebra.arity_mismatch");
    assert_eq!(
        err.message(),
        "composition `join` expects 2 operand(s) but found 3"
    );
}
```

## `v3_algebra_hellgate.rs` :: `unsupported_composition_rejects_with_typed_code`

```rust
#[test]
fn unsupported_composition_rejects_with_typed_code() {
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", sp(0, 5), None, sp(0, 5));
    let orders = b.relation_source("orders", sp(6, 12), None, sp(6, 12));
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
```

## `v3_algebra_mutants.rs` :: `correlated_apply_resolves_the_correct_outer_column_index`

```rust
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
    let data = b.relation_source("data", sp(0, 4), Some(alias("d", 0, 1)), sp(0, 7));
    let d_x = {
        let base = b.symbol("d", sp(8, 9));
        b.try_path(base, vec![seg("x", 10, 11)], sp(8, 11)).expect("path requires at least one segment")
    };
    let d_y = {
        let base = b.symbol("d", sp(12, 13));
        b.try_path(base, vec![seg("y", 14, 15)], sp(12, 15)).expect("path requires at least one segment")
    };
    let a_item = ProjectionItemV1 {
        value: d_x,
        alias: Some(alias("a", 16, 17)),
    };
    let b_item = ProjectionItemV1 {
        value: d_y,
        alias: Some(alias("b", 18, 19)),
    };
    let left = b.try_project(data, vec![a_item, b_item], sp(0, 20)).expect("projection requires at least one item");

    // Right: orders o | project { out: b }  -- `b` is a bare symbol referencing the
    // OUTER column `b` (outer index 1), not anything local.
    let orders = b.relation_source("orders", sp(21, 27), Some(alias("o", 21, 22)), sp(21, 28));
    let b_ref = b.symbol("b", sp(29, 30));
    let out_item = ProjectionItemV1 {
        value: b_ref,
        alias: Some(alias("out", 31, 34)),
    };
    let right = b.try_project(orders, vec![out_item], sp(21, 35)).expect("projection requires at least one item");

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
```

## `v3_algebra_normalization.rs` :: `unary_limit_uses_caller_supplied_window`

```rust
#[test]
fn unary_limit_uses_caller_supplied_window() {
    // sort( users u ) | limit  — caller supplies the (count, offset) window the
    // structure-only HIR cannot carry. The normalizer must thread it into the
    // emitted LimitV1 rather than hardcoding count=u64::MAX, offset=0.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", sp(0, 5), Some(alias("u", 0, 1)), sp(0, 7));
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
```

## `v3_algebra_normalization.rs` :: `apply_inner_quantifier_binder_shadowing_outer_name_is_not_a_correlation`

```rust
#[test]
fn apply_inner_quantifier_binder_shadowing_outer_name_is_not_a_correlation() {
    // users x APPLY ( orders | WHERE exists x ( x.id = x.parent ) )
    //
    // The inner filter's predicate is a quantifier that BINDS `x` locally. Every
    // bare `x` inside the quantifier body resolves to that LOCAL quantifier
    // binder, NOT the outer column also named `x`. The inner subtree therefore
    // references nothing from the outer side, so the apply is UNCORRELATED and
    // must be rejected with MissingCorrelation rather than fabricating a
    // correlation column for the coincidental name `x`.
    let b = HirBuilderV1::new();

    // Left side: users x  (outer column named `x`).
    let left_users = b.relation_source("users", sp(0, 5), Some(alias("x", 0, 1)), sp(0, 7));

    // Right side: orders | WHERE exists x ( x.id = x.parent )
    let orders = b.relation_source("orders", sp(8, 14), None, sp(8, 14));
    let x_base_lhs = b.symbol("x", sp(30, 31));
    let x_id = b.try_path(x_base_lhs, vec![seg("id", 32, 34)], sp(30, 34)).expect("path requires at least one segment");
    let x_base_rhs = b.symbol("x", sp(37, 38));
    let x_parent = b.try_path(x_base_rhs, vec![seg("parent", 39, 45)], sp(37, 45)).expect("path requires at least one segment");
    let body = b.compare("=", x_id, x_parent, sp(30, 45));
    let quant = b.quantifier("exists", vec!["x".to_owned()], body, sp(22, 46));
    let inner = b.relation_filter(orders, quant, sp(8, 46));

    let apply = b.compose("apply", vec![left_users, inner], sp(0, 50));

    let err = normalize_relation(&apply, &NormalizeOptionsV1::default())
        .expect_err("quantifier-bound `x` is local; apply is uncorrelated");
    assert_eq!(err.code(), "algebra.missing_correlation");
    assert_eq!(err.span(), sp(0, 50));
}
```

## `v3_algebra_normalization.rs` :: `apply_inner_binding_shadows_outer_column_name`

```rust
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
    let left_users = b.relation_source("users", sp(0, 5), Some(alias("u", 0, 1)), sp(0, 7));

    // Right side: users u | project u  (bare symbol referencing the LOCAL `u`).
    let right_users = b.relation_source("users", sp(8, 13), Some(alias("u", 8, 9)), sp(8, 16));
    let u_ref = b.symbol("u", sp(20, 21));
    let item = ProjectionItemV1 {
        value: u_ref,
        alias: None,
    };
    let right_proj = b.try_project(right_users, vec![item], sp(8, 24)).expect("projection requires at least one item");

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
```

## `v3_algebra_normalization.rs` :: `project_filter_sort_limit_pipeline_has_stable_provenance`

```rust
#[test]
fn project_filter_sort_limit_pipeline_has_stable_provenance() {
    // limit( sort( project { id: u.id, score: u.score } ( users u WHERE u.active ) ) )
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", sp(0, 5), Some(alias("u", 0, 1)), sp(0, 7));

    let active = {
        let base = b.symbol("u", sp(8, 9));
        let path = b.try_path(base, vec![seg("active", 10, 16)], sp(8, 16)).expect("path requires at least one segment");
        b.bool_value(path, sp(8, 16))
    };
    let filtered = b.relation_filter(users, active, sp(0, 16));

    let id_item = {
        let base = b.symbol("u", sp(20, 21));
        let path = b.try_path(base, vec![seg("id", 22, 24)], sp(20, 24)).expect("path requires at least one segment");
        ProjectionItemV1 {
            value: path,
            alias: Some(alias("id", 25, 27)),
        }
    };
    let score_item = {
        let base = b.symbol("u", sp(28, 29));
        let path = b.try_path(base, vec![seg("score", 30, 35)], sp(28, 35)).expect("path requires at least one segment");
        ProjectionItemV1 {
            value: path,
            alias: Some(alias("score", 36, 41)),
        }
    };
    let projected = b.try_project(filtered, vec![id_item, score_item], sp(0, 42)).expect("projection requires at least one item");

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
```

## `v3_algebra_normalization.rs` :: `apply_inner_aliasing_outer_column_to_new_name_carries_correlated_provenance`

```rust
#[test]
fn apply_inner_aliasing_outer_column_to_new_name_carries_correlated_provenance() {
    // users u APPLY ( orders o | project { x: u } )  -- the inner side projects the
    // outer alias column `u` but RENAMES it to `x`. The output column is named `x`
    // yet its VALUE references outer `u`, so it must carry Correlated provenance
    // pointing at outer `u`. Name-matching on the OUTPUT name (`x`) instead of the
    // referenced symbol (`u`) would wrongly leave it as Derived (D23).
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", sp(0, 5), Some(alias("u", 0, 1)), sp(0, 7));

    let orders = b.relation_source("orders", sp(8, 14), Some(alias("o", 8, 9)), sp(8, 16));
    let u_ref = b.symbol("u", sp(20, 21));
    let item = ProjectionItemV1 {
        value: u_ref,
        alias: Some(alias("x", 22, 23)),
    };
    let inner = b.try_project(orders, vec![item], sp(8, 24)).expect("projection requires at least one item");

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
```

## `v3_algebra_normalization.rs` :: `apply_inner_projecting_outer_column_carries_correlated_provenance`

```rust
#[test]
fn apply_inner_projecting_outer_column_carries_correlated_provenance() {
    // users u APPLY ( orders o | project { u } )  -- the inner side projects the
    // outer alias column `u`, so that OUTPUT column carries Correlated provenance.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", sp(0, 5), Some(alias("u", 0, 1)), sp(0, 7));

    let orders = b.relation_source("orders", sp(8, 14), Some(alias("o", 8, 9)), sp(8, 16));
    let u_ref = b.symbol("u", sp(20, 21));
    let item = ProjectionItemV1 {
        value: u_ref,
        alias: Some(alias("u", 22, 23)),
    };
    let inner = b.try_project(orders, vec![item], sp(8, 24)).expect("projection requires at least one item");

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
```

## `v3_algebra_normalization.rs` :: `correlated_subquery_normalizes_into_explicit_apply_with_visible_correlation`

```rust
#[test]
fn correlated_subquery_normalizes_into_explicit_apply_with_visible_correlation() {
    // users u APPLY (orders o WHERE o.user = u.id)
    // The inner orders relation references the outer alias `u`, so the apply is
    // correlated and the correlation column must be VISIBLE.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", sp(0, 5), Some(alias("u", 0, 1)), sp(0, 7));

    let orders = b.relation_source("orders", sp(8, 14), Some(alias("o", 8, 9)), sp(8, 16));
    let o_user = {
        let base = b.symbol("o", sp(17, 18));
        b.try_path(base, vec![seg("user", 19, 23)], sp(17, 23)).expect("path requires at least one segment")
    };
    let u_id = {
        let base = b.symbol("u", sp(24, 25));
        b.try_path(base, vec![seg("id", 26, 28)], sp(24, 28)).expect("path requires at least one segment")
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
```

## `v3_algebra_probe.rs` :: `apply_inner_column_sharing_outer_name_keeps_distinct_local_provenance`

```rust
#[test]
fn apply_inner_column_sharing_outer_name_keeps_distinct_local_provenance() {
    // users u APPLY ( orders o WHERE o.user = u.id | project { u: o.amount } )
    // The inner output column is NAMED `u` (same name as the outer column) but its
    // VALUE references local `o.amount`. Provenance must keep the two `u` columns
    // distinct: the outer stays Source(users), the inner stays Source(orders), and
    // the inner is NOT mislabelled Correlated just because it shares the name `u`.
    let b = HirBuilderV1::new();
    let users = b.relation_source("users", sp(0, 5), Some(alias("u", 0, 1)), sp(0, 7));
    let orders = b.relation_source("orders", sp(8, 14), Some(alias("o", 8, 9)), sp(8, 16));
    let o_user = {
        let base = b.symbol("o", sp(17, 18));
        b.try_path(base, vec![seg("user", 19, 23)], sp(17, 23)).expect("path requires at least one segment")
    };
    let u_id = {
        let base = b.symbol("u", sp(24, 25));
        b.try_path(base, vec![seg("id", 26, 28)], sp(24, 28)).expect("path requires at least one segment")
    };
    let pred = b.compare("=", o_user, u_id, sp(17, 28));
    let filtered = b.relation_filter(orders, pred, sp(8, 28));
    let o_amount = {
        let base = b.symbol("o", sp(29, 30));
        b.try_path(base, vec![seg("amount", 31, 37)], sp(29, 37)).expect("path requires at least one segment")
    };
    let item = ProjectionItemV1 {
        value: o_amount,
        alias: Some(alias("u", 38, 39)),
    };
    let inner = b.try_project(filtered, vec![item], sp(8, 40)).expect("projection requires at least one item");
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
```

## `v3_algebra_probe.rs` :: `union_arms_agreeing_on_names_merge_with_unioned_provenance`

```rust
#[test]
fn union_arms_agreeing_on_names_merge_with_unioned_provenance() {
    // Both arms project a column NAMED `id`: names agree position-for-position, so
    // the union is accepted and the merged column carries Unioned provenance.
    let b = HirBuilderV1::new();
    let l_src = b.relation_source("a", sp(0, 1), None, sp(0, 1));
    let l_id = {
        let base = b.symbol("a", sp(2, 3));
        b.try_path(base, vec![seg("id", 4, 6)], sp(2, 6)).expect("path requires at least one segment")
    };
    let left = b.try_project(
        l_src,
        vec![ProjectionItemV1 {
            value: l_id,
            alias: None,
        }],
        sp(0, 7),
    ).expect("projection requires at least one item");
    let r_src = b.relation_source("c", sp(8, 9), None, sp(8, 9));
    let r_id = {
        let base = b.symbol("c", sp(10, 11));
        b.try_path(base, vec![seg("id", 12, 14)], sp(10, 14)).expect("path requires at least one segment")
    };
    let right = b.try_project(
        r_src,
        vec![ProjectionItemV1 {
            value: r_id,
            alias: None,
        }],
        sp(8, 15),
    ).expect("projection requires at least one item");
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
```


---

## W2 barrier disposition reconciliation — 2026-09-21 (integrator, ticket 05 / RR-05C item 4)

Reconciled against the sealed V2 contract landed in wave W2 (fingerprint in
Receipt 05). No retired API is restored anywhere.

| law family | final disposition | V2 seating |
|---|---|---|
| union laws (3) | **PERMANENT RETIREMENT** | Apply/Limit/Union are omitted from the ticket's completion claim (no landed Ticket 07 planner consumer); the variants stay unconstructed |
| limit laws (2) | **PERMANENT RETIREMENT** | same |
| composition symbol / arity (2) | **PERMANENT RETIREMENT** | closed HIR fixes the operator set; `unsupported_composition`/ternary-join classes are structurally unreachable |
| correlation / Apply laws (8) | **RE-SEATED** | correlation visibility and identity now live in the sealed predicate: W2 rails `T12/T14` (correlated EXISTS/NOT-EXISTS exact edge: alias kind, ordinal, reference = inner symbol Binding fact, edge location = symbol span), `T15` (conjunctive left-to-right), `T16` (single residual Filter with full ordinary conjunction), `T18` (nested capture separation), `T34/T36/T37` (Dropped/Duplicate/Swapped/Foreign capture mutations through the real finalizer); name-shadowing and wrong-owner resolution authority moved to Sema's capture-vector validation (W1-C oracle matrix, Receipt 03/04) |
| predicate-less `join` (1) | **RE-SEATED (structural)** | `InnerJoinOn` fixture asserts ON identity; `forged_inner_join_without_on_seal_v2` rejects at the checked-constructor boundary; Cross never gains a predicate (`explicit_cross_join_never_gains_a_synthetic_predicate` original retained) |

The archived V1 code blocks above remain verbatim as history; none compile
against the sealed surface and none are pending re-seat.
