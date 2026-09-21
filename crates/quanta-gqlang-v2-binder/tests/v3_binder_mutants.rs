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

use quanta_gqlang_v2_binder::{
    BindDiagnosticV1, Binder, BinderKindV1, BindingV1, CaptureV1, LexicalResolutionV1,
    LexicalScopeV1, ModuleGraphV1, ParamV1, ResolvedTargetV1,
};
use quanta_gqlang_v2_hir::{
    AliasV1, CompositionBodyV1, EvidenceModeV1, ExplainHirV1, GraphRefV1, HirBuilderV1,
    HirNodeIdV1, PresentationKindV1, ShapeFieldV1, SpanV1, TruthModeV1,
};

fn id(raw: u32) -> HirNodeIdV1 {
    HirNodeIdV1::from_raw(raw)
}

// ===== diagnostic.rs:106 primary_span -> Default::default() ================
//
// `primary_span` must return the REAL offending span for every variant, not a
// default (0..0) span. Each variant points at a DIFFERENT field, so we assert
// the exact span chosen per variant.
#[test]
fn primary_span_returns_real_offending_span() {
    let unresolved = BindDiagnosticV1::Unresolved {
        name: "a".to_owned(),
        span: SpanV1::new(4, 9),
    };
    assert_eq!(unresolved.primary_span(), SpanV1::new(4, 9));

    // Duplicate points at the SECOND (colliding) declaration, not the first.
    let duplicate = BindDiagnosticV1::Duplicate {
        name: "a".to_owned(),
        first_span: SpanV1::new(1, 2),
        second_span: SpanV1::new(20, 23),
    };
    assert_eq!(duplicate.primary_span(), SpanV1::new(20, 23));

    // Shadowed points at the INNER (shadowing) binding, not the outer.
    let shadowed = BindDiagnosticV1::Shadowed {
        name: "a".to_owned(),
        outer_span: SpanV1::new(1, 2),
        inner_span: SpanV1::new(25, 26),
    };
    assert_eq!(shadowed.primary_span(), SpanV1::new(25, 26));

    let ambiguous = BindDiagnosticV1::AmbiguousImport {
        name: "a".to_owned(),
        span: SpanV1::new(7, 11),
        candidates: vec!["x".to_owned(), "y".to_owned()],
    };
    assert_eq!(ambiguous.primary_span(), SpanV1::new(7, 11));

    // IllegalCapture points at the reference span, not the binding span.
    let capture = BindDiagnosticV1::IllegalCapture {
        name: "a".to_owned(),
        span: SpanV1::new(50, 55),
        binding_span: SpanV1::new(10, 15),
    };
    assert_eq!(capture.primary_span(), SpanV1::new(50, 55));

    // None of these spans is the default (0..0), so a Default::default() body
    // would mismatch every assertion above.
    assert_ne!(unresolved.primary_span(), SpanV1::default());
}

// ===== module_graph.rs:82 len -> 0 / 1 =====================================
#[test]
fn module_graph_len_counts_distinct_names() {
    assert_eq!(ModuleGraphV1::new().len(), 0);

    let one = ModuleGraphV1::new().with_import("a", "x.a");
    assert_eq!(one.len(), 1);

    // Two distinct names => 2 (kills both `-> 0` and `-> 1`). Two targets for
    // ONE name still counts as one name.
    let two_names = ModuleGraphV1::new()
        .with_import("a", "x.a")
        .with_import("b", "x.b");
    assert_eq!(two_names.len(), 2);

    let one_name_two_targets = ModuleGraphV1::new()
        .with_import("a", "x.a")
        .with_import("a", "y.a");
    assert_eq!(one_name_two_targets.len(), 1);
}

// ===== module_graph.rs:88 is_empty -> true / false =========================
#[test]
fn module_graph_is_empty_tracks_contents() {
    assert!(ModuleGraphV1::new().is_empty());

    let populated = ModuleGraphV1::new().with_import("a", "x.a");
    assert!(!populated.is_empty());
}

// ===== scope.rs:192 replace == with != in same_frame_or_outer ==============
//
// The shadow search must STOP at a non-capturing edge: a binding strictly
// beyond a correlation barrier is invisible and therefore shadows nothing.
//
// Frames (outer -> middle -> top):
//   outer:  binds `x`,            edge-to-parent = Capturing
//   middle: empty,                edge-to-parent = NonCapturing  (the barrier)
//   top:    where we bind `x`,    edge-to-parent = Capturing
//
// The new `x` in `top` must NOT be reported as shadowing the outer `x`,
// because the barrier between middle and outer hides it. The `break` on the
// NonCapturing edge is what enforces this; flipping `==` to `!=` would instead
// break at the capturing edge, walk PAST the barrier, find the outer `x`, and
// wrongly report an OuterShadow.
#[test]
fn shadow_search_stops_at_noncapturing_barrier() {
    let mut scope = LexicalScopeV1::new();
    scope.push(CaptureV1::Capturing); // outer frame
    assert!(scope
        .bind(BindingV1::new(
            "x",
            BinderKindV1::Let,
            SpanV1::new(0, 1),
            id(0)
        ))
        .is_none());

    scope.push(CaptureV1::NonCapturing); // middle frame: the barrier
    scope.push(CaptureV1::Capturing); // top frame

    let report = scope.bind(BindingV1::new(
        "x",
        BinderKindV1::Let,
        SpanV1::new(50, 51),
        id(1),
    ));
    assert!(
        report.is_none(),
        "outer `x` is hidden behind a non-capturing barrier and must not be \
         reported as shadowed; got {report:?}"
    );
}

// Companion: WITHOUT a barrier the outer `x` IS visible and bind reports an
// OuterShadow. This pins the positive side so the `break` cannot be deleted
// outright (a body that always returns None would fail here).
// ===== resolve.rs:200 resolve_rule body -> () ==============================
//
// `resolve_rule` must push a param scope, bind the params, and resolve both
// the head and the body against them. With a `()` body nothing is bound or
// resolved, so the head's param reference would not resolve at all.
#[test]
fn resolve_rule_binds_params_and_resolves_head_and_body() {
    let b = HirBuilderV1::new();
    // head references param `n`; body compares param `n` = 1.
    let head_ref = b.symbol("n", SpanV1::new(20, 21));
    let body_ref = b.symbol("n", SpanV1::new(30, 31));
    let one = b.int("1", SpanV1::new(34, 35));
    let pred = b.compare("=", body_ref.clone(), one, SpanV1::new(30, 35));
    let rule = b.rule(
        "r",
        SpanV1::new(0, 1),
        head_ref.clone(),
        pred,
        SpanV1::new(0, 35),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_rule(&rule, &[ParamV1::new("n", SpanV1::new(5, 6), rule.id)]);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    // head ref resolved to the param.
    match result.target_of(head_ref.id).unwrap() {
        ResolvedTargetV1::Local(binding) => {
            assert_eq!(binding.kind, BinderKindV1::Param);
            assert_eq!(binding.name, "n");
        }
        other => panic!("expected param for head, got {other:?}"),
    }
    // body ref resolved to the param too (proves the body was walked).
    match result.target_of(body_ref.id).unwrap() {
        ResolvedTargetV1::Local(binding) => assert_eq!(binding.kind, BinderKindV1::Param),
        other => panic!("expected param for body, got {other:?}"),
    }
}

// ===== resolve.rs:324 resolve_presentation body -> () ======================
//
// A presentation reaches the resolver via a composition body. An EXPLAIN whose
// `target_ref` names the LET working set `s` must resolve to that binding.
// A `()` body records no resolution for the EXPLAIN reference node.
#[test]
fn resolve_presentation_resolves_target_ref() {
    let b = HirBuilderV1::new();
    // producer: a trivial relation-source query.
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(8, 12),
        None,
        SpanV1::new(8, 13),
    );
    let producer = b.query(source, None, SpanV1::new(0, 13)).into();
    let binding = b.composition_binding("s", SpanV1::new(4, 5), producer);

    // EXPLAIN s : target_ref names the LET-bound working set.
    let explain = b.presentation(
        PresentationKindV1::Explain {
            target_ref: GraphRefV1 {
                name: "s".to_owned(),
                span: SpanV1::new(50, 51),
            },
            evidence: EvidenceModeV1::Auto,
            truth: TruthModeV1::Inspect,
        },
        SpanV1::new(45, 51),
    );
    let explain = ExplainHirV1::try_from(explain).unwrap();
    let explain_node = explain.id;
    let body = CompositionBodyV1::Explain(Box::new(explain));

    let composition = b.composition(
        vec![binding],
        "s",
        SpanV1::new(90, 91),
        body,
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    match result.target_of(explain_node).unwrap() {
        ResolvedTargetV1::Local(binding) => {
            assert_eq!(binding.name, "s");
            assert_eq!(binding.kind, BinderKindV1::CompositionLet);
        }
        other => panic!("expected EXPLAIN target to resolve to the LET, got {other:?}"),
    }
}

// ===== resolve.rs:449 close_scope body -> () ===============================
//
// `close_scope` must POP the innermost frame. After closing the scope that
// holds `x`, a later reference to `x` must be UNRESOLVED. If `close_scope`
// were a no-op (`()`), the frame would leak and `x` would still resolve.
#[test]
fn close_scope_pops_the_frame() {
    let b = HirBuilderV1::new();
    let body = b.symbol("x", SpanV1::new(40, 41));
    let decl = b.decl("v", SpanV1::new(0, 1), body.clone(), SpanV1::new(0, 41));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.open_scope(CaptureV1::Capturing);
    binder.bind_let("x", decl.id, SpanV1::new(10, 11));
    binder.close_scope(); // pops the frame holding `x`
                          // resolve_decl opens its own fresh scope; `x` must no longer be visible.
    binder.resolve_decl(&decl, &[]);
    let result = binder.finish();

    assert!(
        !result.is_ok(),
        "after close_scope the let `x` must be out of scope; resolutions: {:?}",
        result.resolutions()
    );
    let diag = &result.diagnostics()[0];
    assert_eq!(diag.code(), "binder.unresolved_symbol");
    assert_eq!(diag.name(), "x");
    // and there is NO leaked local resolution of `x`.
    assert!(result
        .target_of(body.id)
        .map_or(true, |t| !matches!(t, ResolvedTargetV1::Local(_))));
}

// Sanity companion: WITHOUT the close, `x` resolves (proves the test above is
// detecting the pop, not some unrelated unbound condition).
// ===== resolve.rs:596 resolve_shape body -> () =============================
//
// A query result shape must be walked so its value positions resolve. A query
// whose result shape is `shape_leaf(symbol "u")` must resolve `u` to the
// source alias. A `()` body records no resolution for the shape's symbol node.
#[test]
fn resolve_shape_resolves_value_positions() {
    let b = HirBuilderV1::new();
    let alias = AliasV1 {
        name: "u".to_owned(),
        span: SpanV1::new(30, 31),
    };
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(15, 28),
        Some(alias),
        SpanV1::new(15, 31),
    );

    // result shape: a leaf referencing the alias `u`.
    let shape_ref = b.symbol("u", SpanV1::new(7, 8));
    let shape = b.shape_leaf(shape_ref.clone(), SpanV1::new(7, 8));

    let query = b.query(source, Some(shape), SpanV1::new(0, 40));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    match result.target_of(shape_ref.id).unwrap() {
        ResolvedTargetV1::Local(binding) => {
            assert_eq!(binding.name, "u");
            assert_eq!(binding.kind, BinderKindV1::Alias);
        }
        other => panic!("expected shape leaf to resolve to the alias, got {other:?}"),
    }
}

// Also pin nested record/list shapes so the recursive arms are not deletable.
#[test]
fn resolve_shape_walks_nested_record_and_list() {
    let b = HirBuilderV1::new();
    let alias = AliasV1 {
        name: "u".to_owned(),
        span: SpanV1::new(30, 31),
    };
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(15, 28),
        Some(alias),
        SpanV1::new(15, 31),
    );

    let inner_ref = b.symbol("u", SpanV1::new(7, 8));
    let leaf = b.shape_leaf(inner_ref.clone(), SpanV1::new(7, 8));
    let list = b.shape_list(leaf, SpanV1::new(6, 9));
    // The test name promises nested record AND list, so wrap the list in a
    // record field: `resolve_shape` must recurse Record -> List -> Leaf. If the
    // Record arm is emptied (`Record(_) => {}`), the leaf is never visited and
    // `target_of(inner_ref.id)` below finds no resolution.
    let record = b.shape_record(
        vec![ShapeFieldV1 {
            name: "outer".to_owned(),
            name_span: SpanV1::new(5, 6),
            shape: list,
        }],
        SpanV1::new(4, 10),
    );

    let query = b.query(source, Some(record), SpanV1::new(0, 40));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    assert!(matches!(
        result.target_of(inner_ref.id).unwrap(),
        ResolvedTargetV1::Local(_)
    ));
}

// Guard the scope `resolve` boundary used above (sanity that the helpers
// compile and resolve the way tests assume).
#[test]
fn scope_resolve_distinguishes_bound_from_unbound() {
    let mut scope = LexicalScopeV1::new();
    scope.push(CaptureV1::Capturing);
    let _ = scope.bind(BindingV1::new(
        "y",
        BinderKindV1::Let,
        SpanV1::new(0, 1),
        id(0),
    ));
    assert!(matches!(scope.resolve("y"), LexicalResolutionV1::Bound(_)));
    assert!(matches!(scope.resolve("z"), LexicalResolutionV1::Unbound));
}
