//! v3 composition LET...WITH binder hellgate rail.
//!
//! Covers the composition scope the binder owns (per the task split):
//! a `LET s = <producer>  <body> WITH s` resolves both the body's reference to
//! `s` and the `WITH s` consumer to the LET binding; `WITH` naming an undefined
//! binding rejects (unresolved); a duplicate `LET` name rejects (duplicate). It
//! asserts the ACTUAL typed diagnostic codes and Display text. Ref legality of
//! rule_ref/check_ref existence stays a catalog/modules concern (D30): the
//! binder only resolves LOCAL composition scope.
//!
//! HIR is built directly with `HirBuilderV1` (the v3 parser is ticket 06).

#![allow(clippy::unwrap_used, reason = "tests may unwrap")]
#![allow(clippy::expect_used, reason = "tests may expect")]
#![allow(clippy::indexing_slicing, reason = "tests may index")]
#![allow(clippy::panic, reason = "tests assert by panicking")]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "tests match the expected variant and panic on the rest"
)]

use quanta_gqlang_v2_binder::{
    BindDiagnosticV1, Binder, BinderKindV1, ModuleGraphV1, ResolvedTargetV1,
};
use quanta_gqlang_v2_hir::{
    ConfirmModeV1, DetectionModeV1, EmitModeV1, GraphConstraintsV1, GraphEndpointsV1,
    GraphQueryKindV1, GraphRefV1, HirBuilderV1, ProgramItemHirV1, SpanV1,
};

// Build a SAMPLE-ish producer item: a relation source query is the simplest
// producer that carries no free symbol references of its own.
fn working_set_producer(b: &HirBuilderV1) -> ProgramItemHirV1 {
    let source = b.relation_source("catalog.users", None, SpanV1::new(8, 13));
    let query = b.query(source, None, SpanV1::new(0, 13));
    ProgramItemHirV1::from(query)
}

// A FIND body whose `source` endpoint names the LET-bound working set `s`.
fn find_body_referencing(b: &HirBuilderV1, name: &str, ref_span: SpanV1) -> ProgramItemHirV1 {
    let endpoints = GraphEndpointsV1 {
        source: Some(GraphRefV1 {
            name: name.to_owned(),
            span: ref_span,
        }),
        ..GraphEndpointsV1::default()
    };
    let find = b.graph_query(
        GraphQueryKindV1::Reachability,
        endpoints,
        GraphConstraintsV1::default(),
        SpanV1::new(40, 80),
    );
    ProgramItemHirV1::from(find)
}

// `LET s = <producer>  FIND reach s -> ... WITH s`: the body's `s` and the
// `WITH s` both resolve to the LET binding.
#[test]
fn composition_body_and_with_resolve_to_let() {
    let b = HirBuilderV1::new();
    let producer = working_set_producer(&b);
    let binding = b.composition_binding("s", SpanV1::new(4, 5), producer);
    let body = find_body_referencing(&b, "s", SpanV1::new(50, 51));
    let body_ref_node = body.id();
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
    // the FIND body's `source` ref resolved to the LET-bound working set.
    match result.target_of(body_ref_node).unwrap() {
        ResolvedTargetV1::Local(b) => {
            assert_eq!(b.name, "s");
            assert_eq!(b.kind, BinderKindV1::CompositionLet);
        }
        other => panic!("expected composition-let local, got {other:?}"),
    }
}

#[test]
fn check_semantic_confirm_endpoint_resolves_to_composition_let() {
    let b = HirBuilderV1::new();
    let binding = b.composition_binding("s", SpanV1::new(4, 5), working_set_producer(&b));
    let confirm = b.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 {
                name: "s".to_owned(),
                span: SpanV1::new(60, 61),
            }),
            sink: Some(GraphRefV1 {
                name: "catalog.sink".to_owned(),
                span: SpanV1::new(65, 77),
            }),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1 {
            depth: Some(4),
            ..GraphConstraintsV1::default()
        },
        SpanV1::new(45, 87),
    );
    let confirm_node = confirm.id;
    let body = b.legality_with_semantic_confirm(
        GraphRefV1 {
            name: "rule.hybrid".to_owned(),
            span: SpanV1::new(20, 31),
        },
        None,
        None,
        confirm,
        ConfirmModeV1::Require,
        EmitModeV1::Findings,
        DetectionModeV1::Hybrid,
        SpanV1::new(15, 87),
    );
    let composition = b.composition(
        vec![binding],
        "s",
        SpanV1::new(90, 91),
        ProgramItemHirV1::from(body),
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    match result.target_of(confirm_node).unwrap() {
        ResolvedTargetV1::Local(binding) => {
            assert_eq!(binding.name, "s");
            assert_eq!(binding.kind, BinderKindV1::CompositionLet);
        }
        other => panic!("expected composition-let local, got {other:?}"),
    }
}

// `WITH` naming an undefined binding rejects (unresolved), pointing at the WITH
// span.
#[test]
fn with_undefined_binding_rejects() {
    let b = HirBuilderV1::new();
    let producer = working_set_producer(&b);
    let binding = b.composition_binding("s", SpanV1::new(4, 5), producer);
    let body = find_body_referencing(&b, "s", SpanV1::new(50, 51));
    // WITH names `t`, which is never LET-bound.
    let composition = b.composition(
        vec![binding],
        "t",
        SpanV1::new(90, 91),
        body,
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();

    assert!(!result.is_ok());
    let diag = result
        .diagnostics()
        .iter()
        .find(|d| d.name() == "t")
        .expect("expected a diagnostic about `t`");
    assert_eq!(diag.code(), "binder.unresolved_symbol");
    assert_eq!(
        diag.to_string(),
        "binder.unresolved_symbol: unresolved symbol `t` at 90..91"
    );
}

// A nested composition's `WITH` must consume a LET in its OWN frame, never an
// enclosing composition's LET. Outer `LET x = p1` is in scope for the inner
// composition's body, but the inner `WITH x` (no inner LET `x`) must reject as
// unresolved rather than illegally bind to the outer LET `x`.
#[test]
fn inner_with_naming_outer_let_rejects() {
    let b = HirBuilderV1::new();
    // Inner composition: no LETs of its own; body references nothing local;
    // WITH names `x`, which is only an OUTER composition LET.
    let inner_body = find_body_referencing(&b, "x", SpanV1::new(60, 61));
    let inner = b.composition(
        Vec::new(),
        "x",
        SpanV1::new(70, 71),
        inner_body,
        SpanV1::new(50, 71),
    );
    let inner_item = ProgramItemHirV1::from(inner);

    // Outer composition: `LET x = p1  <inner composition> WITH x`.
    let producer = working_set_producer(&b);
    let outer_binding = b.composition_binding("x", SpanV1::new(4, 5), producer);
    let outer = b.composition(
        vec![outer_binding],
        "x",
        SpanV1::new(90, 91),
        inner_item,
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&outer);
    let result = binder.finish();

    assert!(
        !result.is_ok(),
        "inner WITH naming an outer LET must reject; diags: {:?}",
        result.diagnostics()
    );
    let diag = result
        .diagnostics()
        .iter()
        .find(|d| d.code() == "binder.unresolved_symbol" && d.name() == "x")
        .expect("expected an unresolved diagnostic about inner `WITH x`");
    assert_eq!(
        diag.to_string(),
        "binder.unresolved_symbol: unresolved symbol `x` at 70..71"
    );
}

// composition_catalog_scope_leak: an inner composition's BODY must NOT resolve
// a reference to an ENCLOSING composition's LET working set. The outer `LET x`
// is a frame-local working set; an inner composition is its own scope, so the
// inner body's `x` reference must NOT silently bind to the outer LET. It must
// fail closed (illegal capture across the composition scope boundary), the same
// way `inner_with_naming_outer_let_rejects` proves the WITH consumer is
// frame-local. Otherwise the inner body silently captures an outer working set
// the language never put in scope (a scope leak).
#[test]
fn inner_body_naming_outer_let_does_not_leak() {
    let b = HirBuilderV1::new();
    // Inner composition: no LETs of its own; body's FIND `source` names `x`,
    // which is only an OUTER composition LET; WITH names nothing local.
    let inner_body = find_body_referencing(&b, "x", SpanV1::new(60, 61));
    let inner_body_node = inner_body.id();
    let inner = b.composition(
        Vec::new(),
        "x",
        SpanV1::new(70, 71),
        inner_body,
        SpanV1::new(50, 71),
    );
    let inner_item = ProgramItemHirV1::from(inner);

    // Outer composition: `LET x = p1  <inner composition> WITH x`.
    let producer = working_set_producer(&b);
    let outer_binding = b.composition_binding("x", SpanV1::new(4, 5), producer);
    let outer = b.composition(
        vec![outer_binding],
        "x",
        SpanV1::new(90, 91),
        inner_item,
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&outer);
    let result = binder.finish();

    // The inner body's `x` must NOT resolve to the OUTER composition's LET `x`.
    let leaked = result.resolutions().iter().any(|r| {
        r.reference == inner_body_node
            && matches!(
                &r.target,
                ResolvedTargetV1::Local(binding)
                    if binding.kind == BinderKindV1::CompositionLet && binding.name == "x"
            )
    });
    assert!(
        !leaked,
        "inner body must NOT capture the outer composition's LET working set; \
         resolutions: {:?}",
        result.resolutions()
    );
    // And it must fail closed: an inner reference reaching the outer LET working
    // set is an illegal capture across the composition scope boundary (D25).
    let diag = result
        .diagnostics()
        .iter()
        .find(|d| d.code() == "binder.illegal_capture" && d.name() == "x")
        .expect("expected an illegal-capture diagnostic for the inner body `x`");
    match diag {
        BindDiagnosticV1::IllegalCapture { span, .. } => {
            assert_eq!(span.start, 60);
            assert_eq!(span.end, 61);
        }
        other => panic!("expected illegal capture, got {other:?}"),
    }
}

// composition_binding_name_shadowing: an inner composition `LET x` whose name
// collides with an OUTER composition `LET x` must be handled deterministically
// and fail-closed (no silent last-wins). The inner body's later `WITH x` /
// reference to `x` must bind to the INNER LET, never the outer one, AND the
// collision is reported as a deterministic shadow reject (both spans named).
#[test]
fn inner_let_shadowing_outer_let_is_deterministic() {
    let b = HirBuilderV1::new();
    // Inner composition: `LET x = p_inner  FIND ... x  WITH x`. Its own LET `x`
    // is at span 54..55; the body and WITH name `x`.
    let p_inner = working_set_producer(&b);
    let inner_binding = b.composition_binding("x", SpanV1::new(54, 55), p_inner);
    let inner_body = find_body_referencing(&b, "x", SpanV1::new(60, 61));
    let inner_body_node = inner_body.id();
    let inner = b.composition(
        vec![inner_binding],
        "x",
        SpanV1::new(70, 71),
        inner_body,
        SpanV1::new(50, 71),
    );
    let inner_item = ProgramItemHirV1::from(inner);

    // Outer composition: `LET x = p_outer  <inner composition> WITH x`.
    let p_outer = working_set_producer(&b);
    let outer_binding = b.composition_binding("x", SpanV1::new(4, 5), p_outer);
    let outer = b.composition(
        vec![outer_binding],
        "x",
        SpanV1::new(90, 91),
        inner_item,
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&outer);
    let result = binder.finish();

    // The inner body's `x` must bind to the INNER LET (span 54..55), never the
    // outer LET (span 4..5): no silent last/first-wins across compositions.
    match result.target_of(inner_body_node) {
        Some(ResolvedTargetV1::Local(binding)) => {
            assert_eq!(binding.kind, BinderKindV1::CompositionLet);
            assert_eq!(
                binding.span.start, 54,
                "inner body `x` must bind to the INNER LET, not the outer; got {binding:?}"
            );
        }
        other => panic!("expected inner composition-let local, got {other:?}"),
    }

    // The inner `LET x` does NOT shadow the outer `LET x`: a composition frame
    // is a NON-capturing barrier, so the outer working set is invisible to the
    // inner composition. A spurious `binder.shadowed_binding` reject here would
    // be a false positive (the inner binding hides nothing visible).
    assert!(
        result
            .diagnostics()
            .iter()
            .all(|d| d.code() != "binder.shadowed_binding"),
        "inner LET across a composition barrier must not be a spurious shadow; \
         diags: {:?}",
        result.diagnostics()
    );
}

// A duplicate `LET` name rejects (duplicate), reporting both LET-name spans.
#[test]
fn duplicate_let_name_rejects() {
    let b = HirBuilderV1::new();
    let p1 = working_set_producer(&b);
    let p2 = working_set_producer(&b);
    let b1 = b.composition_binding("s", SpanV1::new(4, 5), p1);
    let b2 = b.composition_binding("s", SpanV1::new(24, 25), p2);
    let body = find_body_referencing(&b, "s", SpanV1::new(50, 51));
    let composition = b.composition(
        vec![b1, b2],
        "s",
        SpanV1::new(90, 91),
        body,
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();

    assert!(!result.is_ok());
    let diag = result
        .diagnostics()
        .iter()
        .find(|d| d.code() == "binder.duplicate_declaration")
        .expect("expected a duplicate-declaration diagnostic");
    assert_eq!(
        diag.to_string(),
        "binder.duplicate_declaration: duplicate declaration `s` \
         (first at 4..5, again at 24..25)"
    );
}
