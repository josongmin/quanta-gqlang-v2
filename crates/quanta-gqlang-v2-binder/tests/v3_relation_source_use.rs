//! Exact relation-source use proofs; host fallback is only for unbound names.

use quanta_gqlang_v2_binder::{
    BindDiagnosticV1, Binder, BinderKindV1, ModuleGraphV1, ResolvedTargetV1,
};
use quanta_gqlang_v2_hir::{
    AliasV1, GraphConstraintsV1, GraphEndpointsV1, GraphQueryKindV1, GraphRefV1, HirBuilderV1,
    RelationExistsPolarityV1, SpanV1,
};

#[test]
fn composition_from_use_resolves_to_exact_prior_let_before_alias_frame() {
    let b = HirBuilderV1::new();
    let producer_source = b.relation_source(
        "catalog.users",
        SpanV1::new(8, 21),
        None,
        SpanV1::new(8, 21),
    );
    let producer_id = producer_source.id;
    let producer = b.query(producer_source, None, SpanV1::new(0, 21));
    let binding = b.composition_binding("p", SpanV1::new(30, 31), producer.into());
    let use_source = b.relation_source(
        "p",
        SpanV1::new(40, 41),
        Some(AliasV1 {
            name: "q".into(),
            span: SpanV1::new(42, 43),
        }),
        SpanV1::new(40, 43),
    );
    let use_id = use_source.id;
    let body = b.query(use_source, None, SpanV1::new(35, 43));
    let composition = b.composition(
        vec![binding],
        "p",
        SpanV1::new(49, 50),
        body.into(),
        SpanV1::new(0, 50),
    );
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();
    assert!(result.is_ok(), "{:?}", result.diagnostics());
    assert!(result.target_of(producer_id).is_none());
    assert!(matches!(result.composition_binding(composition.id, 0),
        Some(binding) if binding.name == "p"
            && binding.kind == BinderKindV1::CompositionLet
            && binding.identity() == (composition.id, 0)));
    let source_use = result.resolutions().iter().find(|resolution| {
        resolution.reference == use_id && resolution.span == SpanV1::new(40, 41)
    });
    assert!(matches!(source_use.map(|resolution| &resolution.target),
        Some(ResolvedTargetV1::Local(binding))
            if binding.kind == BinderKindV1::CompositionLet
                && binding.identity() == (composition.id, 0)));
    assert_eq!(
        result.relation_output(use_id).map(|frame| frame
            .iter()
            .map(|binding| (binding.name.as_str(), binding.identity()))
            .collect::<Vec<_>>()),
        Some(vec![("q", (use_id, 0))])
    );
}

#[test]
fn composition_exists_sources_keep_exact_resolution_without_row_captures() {
    for correlated in [false, true] {
        let b = HirBuilderV1::new();
        let producer = b.query(
            b.relation_source("host", SpanV1::new(0, 4), None, SpanV1::new(0, 4)),
            None,
            SpanV1::new(0, 4),
        );
        let binding = b.composition_binding("p", SpanV1::new(5, 6), producer.into());
        let outer = b.relation_source(
            "p",
            SpanV1::new(10, 11),
            Some(AliasV1 {
                name: "r".into(),
                span: SpanV1::new(12, 13),
            }),
            SpanV1::new(10, 13),
        );
        let middle = b.relation_source(
            "p",
            SpanV1::new(20, 21),
            Some(AliasV1 {
                name: "m".into(),
                span: SpanV1::new(22, 23),
            }),
            SpanV1::new(20, 23),
        );
        let inner = b.relation_source("p", SpanV1::new(30, 31), None, SpanV1::new(30, 31));
        let root_ref = b.symbol("r", SpanV1::new(40, 41));
        let middle_ref = b.symbol("m", SpanV1::new(44, 45));
        let inner_relation = if correlated {
            b.relation_filter(
                inner.clone(),
                b.compare(
                    "=",
                    root_ref.clone(),
                    middle_ref.clone(),
                    SpanV1::new(40, 45),
                ),
                SpanV1::new(30, 45),
            )
        } else {
            inner.clone()
        };
        let inner_exists = b.relation_exists(
            RelationExistsPolarityV1::NotExists,
            inner_relation,
            SpanV1::new(28, 46),
        );
        let inner_id = inner_exists.id;
        let outer_exists = b.relation_exists(
            RelationExistsPolarityV1::Exists,
            b.relation_filter(middle.clone(), inner_exists, SpanV1::new(20, 46)),
            SpanV1::new(18, 47),
        );
        let outer_id = outer_exists.id;
        let body = b.query(
            b.relation_filter(outer.clone(), outer_exists, SpanV1::new(10, 47)),
            None,
            SpanV1::new(10, 47),
        );
        let composition = b.composition(
            vec![binding],
            "p",
            SpanV1::new(50, 51),
            body.into(),
            SpanV1::new(0, 51),
        );
        let graph = ModuleGraphV1::new();
        let mut binder = Binder::new(&graph);
        binder.resolve_composition(&composition);
        let result = binder.finish();
        assert!(result.is_ok(), "{:?}", result.diagnostics());
        for source in [&outer, &middle, &inner] {
            assert!(
                matches!(result.target_of_use(source.id, match &source.kind {
                quanta_gqlang_v2_hir::RelationExprKindV1::Source(source) => source.name_span,
                quanta_gqlang_v2_hir::RelationExprKindV1::Project(_)
                | quanta_gqlang_v2_hir::RelationExprKindV1::Filter(_)
                | quanta_gqlang_v2_hir::RelationExprKindV1::Join(_)
                | quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(_)
                | quanta_gqlang_v2_hir::RelationExprKindV1::OrderBy(_) => unreachable!(),
            }, "p"), Some(ResolvedTargetV1::Local(binding))
                if binding.kind == BinderKindV1::CompositionLet
                    && binding.identity() == (composition.id, 0))
            );
        }
        assert!(result.has_capture_set(outer_id));
        assert!(result.has_capture_set(inner_id));
        if correlated {
            assert_eq!(
                result.capture_edges(outer_id),
                &[(root_ref.id, outer.id, 0)]
            );
            assert_eq!(
                result.capture_edges(inner_id),
                &[(root_ref.id, outer.id, 0), (middle_ref.id, middle.id, 0),]
            );
        } else {
            assert!(result.capture_edges(outer_id).is_empty());
            assert!(result.capture_edges(inner_id).is_empty());
        }
    }
}

#[test]
fn composition_let_spelling_shadowed_by_row_alias_is_not_a_relation_source() {
    let b = HirBuilderV1::new();
    let producer = b.query(
        b.relation_source("host", SpanV1::new(0, 4), None, SpanV1::new(0, 4)),
        None,
        SpanV1::new(0, 4),
    );
    let binding = b.composition_binding("p", SpanV1::new(6, 7), producer.into());
    let outer = b.relation_source(
        "p",
        SpanV1::new(10, 11),
        Some(AliasV1 {
            name: "p".into(),
            span: SpanV1::new(12, 13),
        }),
        SpanV1::new(10, 13),
    );
    let inner = b.relation_source("p", SpanV1::new(20, 21), None, SpanV1::new(20, 21));
    let exists = b.relation_exists(RelationExistsPolarityV1::Exists, inner, SpanV1::new(18, 22));
    let body = b.query(
        b.relation_filter(outer, exists, SpanV1::new(10, 22)),
        None,
        SpanV1::new(10, 22),
    );
    let composition = b.composition(
        vec![binding],
        "p",
        SpanV1::new(25, 26),
        body.into(),
        SpanV1::new(0, 26),
    );
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();
    assert!(result.diagnostics().iter().any(|diagnostic| matches!(diagnostic,
        BindDiagnosticV1::WrongRelationSourceKind { name, span, binding_span }
            if name == "p" && *span == SpanV1::new(20, 21) && *binding_span == SpanV1::new(12, 13))));
}

#[test]
fn composition_source_barrier_still_rejects_enclosing_let() {
    let b = HirBuilderV1::new();
    let outer = b.relation_source("host", SpanV1::new(0, 4), None, SpanV1::new(0, 4));
    let inner = b.relation_source("outer", SpanV1::new(20, 25), None, SpanV1::new(20, 25));
    let exists = b.relation_exists(RelationExistsPolarityV1::Exists, inner, SpanV1::new(18, 26));
    let body = b.query(
        b.relation_filter(outer.clone(), exists, SpanV1::new(0, 26)),
        None,
        SpanV1::new(0, 26),
    );
    let producer = b.query(outer.clone(), None, SpanV1::new(0, 4));
    let binding = b.composition_binding("p", SpanV1::new(6, 7), producer.into());
    let composition = b.composition(
        vec![binding],
        "p",
        SpanV1::new(30, 31),
        body.into(),
        SpanV1::new(0, 31),
    );
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.open_scope(quanta_gqlang_v2_binder::CaptureV1::Capturing);
    binder.bind_let("outer", outer.id, SpanV1::new(8, 13));
    binder.resolve_composition(&composition);
    let result = binder.finish();
    assert!(
        matches!(result.diagnostics(), [BindDiagnosticV1::IllegalCapture { name, span, binding_span }]
        if name == "outer" && *span == SpanV1::new(20, 25) && *binding_span == SpanV1::new(8, 13))
    );
}

#[test]
fn composition_self_and_forward_sources_never_fall_back_to_host() {
    for (first_name, first_source) in [("self_ref", "self_ref"), ("first", "later")] {
        let b = HirBuilderV1::new();
        let first_source_node = b.relation_source(
            first_source,
            SpanV1::new(10, 10 + first_source.len()),
            None,
            SpanV1::new(10, 10 + first_source.len()),
        );
        let first = b.query(first_source_node, None, SpanV1::new(0, 30));
        let second = b.query(
            b.relation_source("host", SpanV1::new(40, 44), None, SpanV1::new(40, 44)),
            None,
            SpanV1::new(35, 45),
        );
        let first_binding = b.composition_binding(
            first_name,
            SpanV1::new(50, 50 + first_name.len()),
            first.into(),
        );
        let later_binding = b.composition_binding("later", SpanV1::new(60, 65), second.into());
        let body_source = b.relation_source(
            first_name,
            SpanV1::new(70, 70 + first_name.len()),
            None,
            SpanV1::new(70, 70 + first_name.len()),
        );
        let body_source_id = body_source.id;
        let body = b.query(body_source, None, SpanV1::new(68, 82));
        let composition = b.composition(
            vec![first_binding, later_binding],
            first_name,
            SpanV1::new(85, 85 + first_name.len()),
            body.into(),
            SpanV1::new(0, 96),
        );
        let graph = ModuleGraphV1::new();
        let mut binder = Binder::new(&graph);
        binder.resolve_composition(&composition);
        let result = binder.finish();
        assert_eq!(
            result.diagnostics().len(),
            1,
            "only the forbidden composition source may reject; got {:?}",
            result.diagnostics()
        );
        assert!(matches!(result.diagnostics().first(),
            Some(BindDiagnosticV1::Unresolved { name, .. }) if name == first_source));
        // The valid body source still resolves to its own LET working set.
        // `target_of_use` refuses to answer on a non-ok result, so inspect the
        // recorded resolutions directly.
        let body_span = SpanV1::new(70, 70 + first_name.len());
        assert!(
            result.resolutions().iter().any(|resolution| {
                resolution.reference == body_source_id
                    && resolution.span == body_span
                    && resolution.name == first_name
                    && matches!(&resolution.target,
                        ResolvedTargetV1::Local(binding)
                            if binding.kind == BinderKindV1::CompositionLet)
            }),
            "body source `{first_name}` must resolve to the LET; resolutions: {:?}",
            result.resolutions()
        );
    }
}

#[test]
fn from_cannot_fall_back_to_host_when_local_name_is_a_row_alias() {
    let b = HirBuilderV1::new();
    let outer = b.relation_source(
        "catalog.users",
        SpanV1::new(0, 13),
        Some(AliasV1 {
            name: "u".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(0, 15),
    );
    let inner = b.relation_source("u", SpanV1::new(22, 23), None, SpanV1::new(22, 23));
    let exists = b.relation_exists(RelationExistsPolarityV1::Exists, inner, SpanV1::new(20, 24));
    let filter = b.relation_filter(outer, exists, SpanV1::new(0, 24));
    let query = b.query(filter, None, SpanV1::new(0, 24));
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();
    assert!(matches!(result.diagnostics().first(),
        Some(BindDiagnosticV1::WrongRelationSourceKind { name, span, binding_span })
            if name == "u" && *span == SpanV1::new(22, 23)
                && *binding_span == SpanV1::new(14, 15)));
}

#[test]
fn graph_uses_with_one_enclosing_node_select_distinct_let_targets() {
    let b = HirBuilderV1::new();
    let first = b.query(
        b.relation_source(
            "catalog.users",
            SpanV1::new(8, 21),
            None,
            SpanV1::new(8, 21),
        ),
        None,
        SpanV1::new(0, 21),
    );
    let second = b.query(
        b.relation_source(
            "catalog.files",
            SpanV1::new(25, 38),
            None,
            SpanV1::new(25, 38),
        ),
        None,
        SpanV1::new(23, 38),
    );
    let p = b.composition_binding("p", SpanV1::new(40, 41), first.into());
    let q = b.composition_binding("q", SpanV1::new(42, 43), second.into());
    let graph_body = b.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 {
                name: "p".into(),
                span: SpanV1::new(50, 51),
            }),
            sink: Some(GraphRefV1 {
                name: "q".into(),
                span: SpanV1::new(60, 61),
            }),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
        SpanV1::new(45, 70),
    );
    let graph_id = graph_body.id;
    let composition = b.composition(
        vec![p, q],
        "q",
        SpanV1::new(80, 81),
        graph_body.into(),
        SpanV1::new(0, 81),
    );
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();
    assert!(result.is_ok(), "{:?}", result.diagnostics());
    assert!(
        matches!(result.target_of_use(graph_id, SpanV1::new(50, 51), "p"),
        Some(ResolvedTargetV1::Local(binding)) if binding.identity() == (composition.id, 0))
    );
    assert!(
        matches!(result.target_of_use(graph_id, SpanV1::new(60, 61), "q"),
        Some(ResolvedTargetV1::Local(binding)) if binding.identity() == (composition.id, 1))
    );
    assert!(result
        .target_of_use(graph_id, SpanV1::new(60, 61), "p")
        .is_none());
}

#[test]
fn duplicate_exact_graph_use_key_rejects_instead_of_first_match() {
    let b = HirBuilderV1::new();
    let producer = b.query(
        b.relation_source(
            "catalog.users",
            SpanV1::new(8, 21),
            None,
            SpanV1::new(8, 21),
        ),
        None,
        SpanV1::new(0, 21),
    );
    let p = b.composition_binding("p", SpanV1::new(30, 31), producer.into());
    let graph_body = b.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 {
                name: "p".into(),
                span: SpanV1::new(50, 51),
            }),
            sink: Some(GraphRefV1 {
                name: "p".into(),
                span: SpanV1::new(50, 51),
            }),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
        SpanV1::new(45, 70),
    );
    let graph_id = graph_body.id;
    let composition = b.composition(
        vec![p],
        "p",
        SpanV1::new(80, 81),
        graph_body.into(),
        SpanV1::new(0, 81),
    );
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();
    assert!(matches!(result.diagnostics().first(),
        Some(BindDiagnosticV1::DuplicateReferenceUse { name, span })
            if name == "p" && *span == SpanV1::new(50, 51)));
    assert!(result
        .target_of_use(graph_id, SpanV1::new(50, 51), "p")
        .is_none());
}

#[test]
fn duplicate_unbound_graph_use_key_rejects_before_host_lookup() {
    let b = HirBuilderV1::new();
    let producer = b.query(
        b.relation_source(
            "catalog.users",
            SpanV1::new(8, 21),
            None,
            SpanV1::new(8, 21),
        ),
        None,
        SpanV1::new(0, 21),
    );
    let p = b.composition_binding("p", SpanV1::new(30, 31), producer.into());
    let graph_body = b.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 {
                name: "host".into(),
                span: SpanV1::new(50, 54),
            }),
            sink: Some(GraphRefV1 {
                name: "host".into(),
                span: SpanV1::new(50, 54),
            }),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
        SpanV1::new(45, 70),
    );
    let graph_id = graph_body.id;
    let composition = b.composition(
        vec![p],
        "p",
        SpanV1::new(80, 81),
        graph_body.into(),
        SpanV1::new(0, 81),
    );
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();
    assert!(matches!(result.diagnostics().first(),
        Some(BindDiagnosticV1::DuplicateReferenceUse { name, span })
            if name == "host" && *span == SpanV1::new(50, 54)));
    assert!(result
        .target_of_use(graph_id, SpanV1::new(50, 54), "host")
        .is_none());
}

#[test]
fn three_max_width_reverse_graph_lists_keep_first_use_and_reject_exact_duplicate() {
    let b = HirBuilderV1::new();
    let producer = b.query(
        b.relation_source(
            "catalog.users",
            SpanV1::new(8, 21),
            None,
            SpanV1::new(8, 21),
        ),
        None,
        SpanV1::new(0, 21),
    );
    let binding = b.composition_binding("p", SpanV1::new(30, 31), producer.into());
    let reverse_refs = |offset: usize, count: usize| {
        (offset..offset + count)
            .map(|ordinal| {
                let start = 200_000 - ordinal * 8;
                GraphRefV1 {
                    name: format!("host_{ordinal:05}"),
                    span: SpanV1::new(start, start + 7),
                }
            })
            .collect::<Vec<_>>()
    };
    let duplicate = GraphRefV1 {
        name: "p".into(),
        span: SpanV1::new(40, 41),
    };
    let mut via = reverse_refs(0, 4095);
    via.push(duplicate.clone());
    let mut through = reverse_refs(4095, 4095);
    through.push(duplicate.clone());
    let sanitizers = reverse_refs(8190, 4096);
    assert_eq!(
        (via.len(), through.len(), sanitizers.len()),
        (4096, 4096, 4096)
    );
    assert!(via
        .first()
        .zip(via.get(4094))
        .is_some_and(|(first, last)| first.span.start > last.span.start));
    let graph_body = b.graph_query(
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 {
                name: "p".into(),
                span: SpanV1::new(50, 51),
            }),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1 {
            via,
            through,
            sanitizers,
            ..GraphConstraintsV1::default()
        },
        SpanV1::new(35, 200_010),
    );
    let graph_id = graph_body.id;
    let composition = b.composition(
        vec![binding],
        "p",
        SpanV1::new(200_020, 200_021),
        graph_body.into(),
        SpanV1::new(0, 200_021),
    );
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_composition(&composition);
    let result = binder.finish();
    assert!(matches!(result.diagnostics(),
        [BindDiagnosticV1::DuplicateReferenceUse { name, span }]
            if name == "p" && *span == duplicate.span));
    let local_uses = result
        .resolutions()
        .iter()
        .filter(|resolution| resolution.reference == graph_id && resolution.name == "p")
        .collect::<Vec<_>>();
    assert_eq!(
        local_uses
            .iter()
            .map(|resolution| resolution.span)
            .collect::<Vec<_>>(),
        vec![SpanV1::new(50, 51), duplicate.span],
    );
    assert!(local_uses.iter().all(|resolution| matches!(
        &resolution.target,
        ResolvedTargetV1::Local(binding)
            if binding.kind == BinderKindV1::CompositionLet
                && binding.identity() == (composition.id, 0)
    )));
    assert!(result
        .target_of_use(graph_id, duplicate.span, "p")
        .is_none());
}
