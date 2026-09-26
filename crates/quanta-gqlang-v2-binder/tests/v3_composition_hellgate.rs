#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_binder::{Binder, BinderKindV1, ModuleGraphV1, ResolvedTargetV1};
use quanta_gqlang_v2_hir::{
    CompositionBodyV1, CompositionProducerV1, ConfirmModeV1, DetectionModeV1, EmitModeV1,
    GraphConstraintsV1, GraphEndpointsV1, GraphQueryKindV1, GraphRefV1, HirBuilderV1, HirNodeIdV1,
    SampleHirV1, SampleModeV1, SpanV1,
};

// Build a SAMPLE-ish producer item: a relation source query is the simplest
// producer that carries no free symbol references of its own.
fn working_set_producer(b: &HirBuilderV1) -> CompositionProducerV1 {
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(8, 12),
        None,
        SpanV1::new(8, 13),
    );
    let query = b.query(source, None, SpanV1::new(0, 13));
    query.into()
}

// A FIND body whose `source` endpoint names the LET-bound working set `s`.
fn find_body_referencing(
    b: &HirBuilderV1,
    name: &str,
    ref_span: SpanV1,
) -> (CompositionBodyV1, HirNodeIdV1) {
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
    let node = find.id;
    (find.into(), node)
}

// `LET s = <producer>  FIND reach s -> ... WITH s`: the body's `s` and the
// `WITH s` both resolve to the LET binding.
#[test]
fn composition_body_and_with_resolve_to_let() {
    let b = HirBuilderV1::new();
    let producer = working_set_producer(&b);
    let binding = b.composition_binding("s", SpanV1::new(4, 5), producer);
    let (body, body_ref_node) = find_body_referencing(&b, "s", SpanV1::new(50, 51));
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
    // the `WITH s` consumer must record its own resolution to the same LET.
    assert!(matches!(
        result.target_of_use(composition.id, SpanV1::new(90, 91), "s"),
        Some(ResolvedTargetV1::Local(binding))
            if binding.name == "s"
                && binding.kind == BinderKindV1::CompositionLet
                && binding.identity() == (composition.id, 0)
    ));
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
        body.into(),
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
    let (body, _) = find_body_referencing(&b, "s", SpanV1::new(50, 51));
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

// Nested compositions are not representable in `CompositionBodyV1`; the HIR
// closed enum now owns that invalid-state prevention. Binder tests therefore
// cover only the legal producer/body families admitted by that carrier.

// A duplicate `LET` name rejects (duplicate), reporting both LET-name spans.
#[test]
fn duplicate_let_name_rejects() {
    let b = HirBuilderV1::new();
    let p1 = working_set_producer(&b);
    let p2 = working_set_producer(&b);
    let b1 = b.composition_binding("s", SpanV1::new(4, 5), p1);
    let b2 = b.composition_binding("s", SpanV1::new(24, 25), p2);
    let (body, _) = find_body_referencing(&b, "s", SpanV1::new(50, 51));
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

// Every legality reference slot (rule ref, scope, pattern) must resolve to the
// composition LET, exercising `resolve_legality`'s scope/pattern loop.
#[test]
fn legality_reference_slots_resolve_to_the_composition_let() {
    let b = HirBuilderV1::new();
    let binding = b.composition_binding("s", SpanV1::new(4, 5), working_set_producer(&b));
    let legality = b.legality(
        GraphRefV1 {
            name: "s".to_owned(),
            span: SpanV1::new(20, 21),
        },
        Some(GraphRefV1 {
            name: "s".to_owned(),
            span: SpanV1::new(25, 26),
        }),
        Some(GraphRefV1 {
            name: "s".to_owned(),
            span: SpanV1::new(28, 29),
        }),
        ConfirmModeV1::Skip,
        EmitModeV1::Findings,
        DetectionModeV1::PatternOnly,
        SpanV1::new(15, 40),
    );
    let legality_id = legality.id;
    let composition = b.composition(
        vec![binding],
        "s",
        SpanV1::new(90, 91),
        legality.into(),
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    for span in [
        SpanV1::new(20, 21),
        SpanV1::new(25, 26),
        SpanV1::new(28, 29),
    ] {
        assert!(
            matches!(
                result.target_of_use(legality_id, span, "s"),
                Some(ResolvedTargetV1::Local(binding))
                    if binding.kind == BinderKindV1::CompositionLet
                        && binding.identity() == (composition.id, 0)
            ),
            "legality ref at {span:?} must resolve to the LET; got {:?}",
            result.target_of_use(legality_id, span, "s")
        );
    }
}

// Every graph-constraint reference chain (via/through/sanitizers/scope/window/
// frontier) must resolve to the composition LET, exercising the `.chain(..)`
// arms of `resolve_graph_query`.
#[test]
fn graph_constraint_reference_chains_resolve_to_the_composition_let() {
    let b = HirBuilderV1::new();
    let binding = b.composition_binding("s", SpanV1::new(4, 5), working_set_producer(&b));
    let reference = |start: usize| GraphRefV1 {
        name: "s".to_owned(),
        span: SpanV1::new(start, start + 1),
    };
    let constraints = GraphConstraintsV1 {
        via: vec![reference(20)],
        through: vec![reference(22)],
        sanitizers: vec![reference(24)],
        scope: Some(reference(26)),
        window: Some(reference(28)),
        frontier: Some(reference(30)),
        ..GraphConstraintsV1::default()
    };
    let find = b.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1::default(),
        constraints,
        SpanV1::new(15, 35),
    );
    let find_id = find.id;
    let composition = b.composition(
        vec![binding],
        "s",
        SpanV1::new(90, 91),
        find.into(),
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    for start in [20, 22, 24, 26, 28, 30] {
        let span = SpanV1::new(start, start + 1);
        assert!(
            matches!(
                result.target_of_use(find_id, span, "s"),
                Some(ResolvedTargetV1::Local(binding))
                    if binding.kind == BinderKindV1::CompositionLet
                        && binding.identity() == (composition.id, 0)
            ),
            "constraint ref at {span:?} must resolve to the LET"
        );
    }
}

// A SAMPLE producer's scope ref and a FIND producer's endpoint must resolve
// through `resolve_composition_producer`'s Sample / GraphQuery arms.
#[test]
fn sample_scope_and_graph_producer_resolve_to_a_prior_let() {
    let b = HirBuilderV1::new();
    let first = b.composition_binding("a", SpanV1::new(4, 5), working_set_producer(&b));

    let sample = SampleHirV1 {
        id: b.next_id(),
        span: SpanV1::new(15, 25),
        query_text: "sql injection".to_owned(),
        mode: SampleModeV1::Hybrid,
        limit: 5,
        scope: Some(GraphRefV1 {
            name: "a".to_owned(),
            span: SpanV1::new(20, 21),
        }),
    };
    let sample_id = sample.id;
    let second = b.composition_binding(
        "b",
        SpanV1::new(6, 7),
        CompositionProducerV1::Sample(Box::new(sample)),
    );

    let find = b.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 {
                name: "a".to_owned(),
                span: SpanV1::new(30, 31),
            }),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
        SpanV1::new(28, 34),
    );
    let find_id = find.id;
    let third = b.composition_binding("c", SpanV1::new(8, 9), find.into());

    let (body, _) = find_body_referencing(&b, "a", SpanV1::new(50, 51));
    let composition = b.composition(
        vec![first, second, third],
        "a",
        SpanV1::new(90, 91),
        body,
        SpanV1::new(0, 91),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    for (node, span) in [
        (sample_id, SpanV1::new(20, 21)),
        (find_id, SpanV1::new(30, 31)),
    ] {
        assert!(
            matches!(
                result.target_of_use(node, span, "a"),
                Some(ResolvedTargetV1::Local(binding))
                    if binding.kind == BinderKindV1::CompositionLet
                        && binding.identity() == (composition.id, 0)
            ),
            "producer ref at {span:?} must resolve to the prior LET `a`"
        );
    }
}
