#![expect(
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_binder::{
    Binder, BinderKindV1, CaptureV1, ModuleGraphV1, ParamV1, ResolvedTargetV1,
};
use quanta_gqlang_v2_hir::{
    AliasV1, CallArgHirV1, CallArgsHirV1, HirBuilderV1, OrderKeyHirV1, ProjectionItemV1,
    SortDirectionHirV1, SpanV1,
};

fn alias(name: &str, start: usize) -> AliasV1 {
    AliasV1 {
        name: name.to_owned(),
        span: SpanV1::new(start, start + 1),
    }
}

#[test]
fn group_output_exposes_only_ordered_key_bindings_and_retains_private_partition_input() {
    let b = HirBuilderV1::new();
    let scan = b.relation_source(
        "catalog.repos",
        SpanV1::new(0, 10),
        Some(alias("u", 10)),
        SpanV1::new(0, 11),
    );
    let scan_id = scan.id;
    let first = b
        .try_path(
            b.symbol("u", SpanV1::new(20, 21)),
            vec![quanta_gqlang_v2_hir::PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let second = b.int("1", SpanV1::new(29, 30));
    let group = b
        .group_by(scan, vec![first, second], SpanV1::new(0, 30))
        .unwrap();
    let group_id = group.id;
    let order_ref = b.symbol("id", SpanV1::new(39, 41));
    let order_ref_id = order_ref.id;
    let order = b
        .order_by(
            group,
            vec![OrderKeyHirV1::nulls_last(
                order_ref,
                SortDirectionHirV1::Asc,
            )],
            SpanV1::new(0, 41),
        )
        .unwrap();
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&b.query(order, None, SpanV1::new(0, 41)));
    let result = binder.finish();
    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    let output = result.relation_output(group_id).unwrap();
    assert_eq!(
        output
            .iter()
            .map(|binding| (binding.name.as_str(), binding.identity()))
            .collect::<Vec<_>>(),
        vec![("id", (group_id, 0)), ("col", (group_id, 1))]
    );
    assert_eq!(
        result
            .partition_input(group_id)
            .unwrap()
            .iter()
            .map(|binding| (binding.name.as_str(), binding.identity()))
            .collect::<Vec<_>>(),
        vec![("u", (scan_id, 0))]
    );
    let ResolvedTargetV1::Local(binding) = result.target_of(order_ref_id).unwrap() else {
        panic!("key-only group frame required")
    };
    assert_eq!(binding.identity(), (group_id, 0));
}

#[test]
fn duplicate_implicit_group_names_reject_at_second_key_span() {
    let b = HirBuilderV1::new();
    let scan = b.relation_source(
        "catalog.repos",
        SpanV1::new(0, 10),
        Some(alias("u", 10)),
        SpanV1::new(0, 11),
    );
    let key = |base_span, segment_span, end| {
        b.try_path(
            b.symbol("u", base_span),
            vec![quanta_gqlang_v2_hir::PathSegmentV1 {
                name: "id".into(),
                span: segment_span,
            }],
            SpanV1::new(base_span.start, end),
        )
        .unwrap()
    };
    let group = b
        .group_by(
            scan,
            vec![
                key(SpanV1::new(20, 21), SpanV1::new(22, 24), 24),
                key(SpanV1::new(30, 31), SpanV1::new(32, 34), 34),
            ],
            SpanV1::new(0, 34),
        )
        .unwrap();
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&b.query(group, None, SpanV1::new(0, 34)));
    let result = binder.finish();
    assert!(!result.is_ok());
    assert_eq!(
        result.diagnostics()[0].code(),
        "binder.duplicate_declaration"
    );
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(32, 34));
}

#[test]
fn duplicate_computed_group_keys_reject_at_second_col_span() {
    let b = HirBuilderV1::new();
    let scan = b.relation_source(
        "catalog.repos",
        SpanV1::new(0, 10),
        None,
        SpanV1::new(0, 11),
    );
    let group = b
        .group_by(
            scan,
            vec![
                b.int("1", SpanV1::new(20, 21)),
                b.int("2", SpanV1::new(30, 31)),
            ],
            SpanV1::new(0, 31),
        )
        .unwrap();
    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&b.query(group, None, SpanV1::new(0, 31)));
    let result = binder.finish();
    assert!(!result.is_ok());
    assert_eq!(
        result.diagnostics()[0].code(),
        "binder.duplicate_declaration"
    );
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(30, 31));
}

#[test]
fn post_group_partition_alias_cannot_be_rebound_to_a_declaration_or_import() {
    for collision in ["declaration", "import", "none"] {
        let b = HirBuilderV1::new();
        let scan = b.relation_source(
            "catalog.repos",
            SpanV1::new(0, 10),
            Some(alias("r", 10)),
            SpanV1::new(0, 11),
        );
        let scan_id = scan.id;
        let group = b
            .group_by(
                scan,
                vec![b.int("1", SpanV1::new(20, 21))],
                SpanV1::new(0, 21),
            )
            .unwrap();
        let group_id = group.id;
        let reference = b.symbol("r", SpanV1::new(37, 38));
        let reference_id = reference.id;
        let arg = b
            .try_path(
                reference,
                vec![quanta_gqlang_v2_hir::PathSegmentV1 {
                    name: "value".into(),
                    span: SpanV1::new(39, 44),
                }],
                SpanV1::new(37, 44),
            )
            .unwrap();
        let sum = b.call(
            "sum",
            SpanV1::new(33, 36),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
            SpanV1::new(33, 45),
        );
        let project = b
            .try_project(
                group,
                vec![ProjectionItemV1 {
                    value: sum,
                    alias: Some(alias("total", 46)),
                }],
                SpanV1::new(0, 47),
            )
            .unwrap();
        let declaration = b.decl(
            "r",
            SpanV1::new(49, 50),
            b.int("1", SpanV1::new(53, 54)),
            SpanV1::new(49, 54),
        );
        let graph = if collision == "import" {
            ModuleGraphV1::new().with_import("r", "lib.r")
        } else {
            ModuleGraphV1::new()
        };
        let mut binder = Binder::new(&graph);
        if collision == "declaration" {
            binder.declare_program(&[declaration], &[]);
        }
        binder.resolve_query(&b.query(project, None, SpanV1::new(0, 47)));
        let result = binder.finish();
        if collision == "none" {
            assert!(result.is_ok(), "{collision}: {:?}", result.diagnostics());
            let ResolvedTargetV1::Local(binding) = result.target_of(reference_id).unwrap() else {
                panic!("pre-group alias required")
            };
            assert_eq!(binding.name, "r");
            assert_eq!(binding.identity(), (scan_id, 0));
            let candidates = result.partition_candidates().collect::<Vec<_>>();
            assert_eq!(candidates.len(), 1);
            let candidate = candidates[0];
            assert_eq!(candidate.reference(), reference_id);
            assert_eq!(candidate.group(), group_id);
            assert_eq!(candidate.binding().identity(), (scan_id, 0));
            assert_eq!(candidate.span(), SpanV1::new(37, 38));
        } else {
            assert_eq!(
                result.diagnostics()[0].code(),
                "binder.partition_authority_collision",
                "{collision}"
            );
            assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(37, 38));
            assert_eq!(result.partition_candidates().len(), 0);
        }
    }
}

// `SELECT u.id FROM catalog.users AS u WHERE u.active = true` (scenario 1):
// alias `u` introduced by the source, referenced by both projection and filter.
#[test]
fn alias_resolves_in_projection_and_filter() {
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
    let source_id = source.id;

    // projection: u.id
    let proj_base = b.symbol("u", SpanV1::new(7, 8));
    let proj_path = b
        .try_path(
            proj_base,
            vec![quanta_gqlang_v2_hir::PathSegmentV1 {
                name: "id".to_owned(),
                span: SpanV1::new(9, 11),
            }],
            SpanV1::new(7, 11),
        )
        .unwrap();
    // filter: u.active = true. The filter precedes projection because a
    // projection replaces the row frame instead of leaking input aliases.
    let filter_base = b.symbol("u", SpanV1::new(38, 39));
    let filter_path = b
        .try_path(
            filter_base,
            vec![quanta_gqlang_v2_hir::PathSegmentV1 {
                name: "active".to_owned(),
                span: SpanV1::new(40, 46),
            }],
            SpanV1::new(38, 46),
        )
        .unwrap();
    let rhs = b.bool_lit(true, SpanV1::new(49, 53));
    let pred = b.compare("=", filter_path, rhs, SpanV1::new(38, 53));
    let filtered = b.relation_filter(source, pred, SpanV1::new(0, 53));
    let projected = b
        .try_project(
            filtered,
            vec![ProjectionItemV1 {
                value: proj_path,
                alias: None,
            }],
            SpanV1::new(0, 53),
        )
        .unwrap();

    let query = b.query(projected, None, SpanV1::new(0, 53));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    // both `u` references resolved to the same alias binder.
    assert_eq!(result.resolutions().len(), 2);
    for res in result.resolutions() {
        match &res.target {
            ResolvedTargetV1::Local(binding) => {
                assert_eq!(binding.name, "u");
                assert_eq!(binding.kind, BinderKindV1::Alias);
                assert_eq!(binding.identity(), (source_id, 0));
            }
            other => panic!("expected local alias, got {other:?}"),
        }
    }
}

// Local `let` binding plus capture: a let `x` is visible to a later reference.
#[test]
fn let_binding_resolves() {
    let b = HirBuilderV1::new();
    let body_ref = b.symbol("x", SpanV1::new(20, 21));
    let decl = b.decl("v", SpanV1::new(0, 1), body_ref, SpanV1::new(0, 22));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    // open the decl scope manually, introduce a let, resolve the body ref.
    binder.open_scope(quanta_gqlang_v2_binder::CaptureV1::Capturing);
    binder.bind_let("x", decl.id, SpanV1::new(10, 11));
    binder.resolve_decl(&decl, &[]);
    binder.close_scope();
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    let target = result.target_of(decl.body.id).unwrap();
    match target {
        ResolvedTargetV1::Local(binding) => {
            assert_eq!(binding.kind, BinderKindV1::Let);
            assert_eq!(binding.name, "x");
        }
        other => panic!("expected let binding, got {other:?}"),
    }
}

// Imported symbol resolves to a single unambiguous frozen-graph target (D30).
#[test]
fn unambiguous_import_resolves() {
    let b = HirBuilderV1::new();
    let body = b.symbol("helper", SpanV1::new(0, 6));
    let decl = b.decl("v", SpanV1::new(0, 1), body, SpanV1::new(0, 6));

    let graph = ModuleGraphV1::new().with_import("helper", "std.lib.helper");
    let mut binder = Binder::new(&graph);
    binder.resolve_decl(&decl, &[]);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    match result.target_of(decl.body.id).unwrap() {
        ResolvedTargetV1::Import { target, .. } => {
            assert_eq!(target, "std.lib.helper");
        }
        other => panic!("expected import, got {other:?}"),
    }
}

// Correlated existence check: an inner reference across a CAPTURING edge
// resolves to the outer alias (scenario 2). The explicit capturing edge models
// `WHERE EXISTS (... WHERE f.repo = r.id)` where `r` is an outer alias.
#[test]
fn correlated_capture_resolves_across_capturing_edge() {
    let b = HirBuilderV1::new();
    let inner_ref = b.symbol("r", SpanV1::new(60, 61));
    let decl = b.decl(
        "v",
        SpanV1::new(0, 1),
        inner_ref.clone(),
        SpanV1::new(0, 61),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.open_scope(CaptureV1::Capturing);
    binder.bind_let("r", decl.id, SpanV1::new(10, 11)); // stands in for outer alias
    binder.open_scope(CaptureV1::Capturing); // correlated subquery may capture
    binder.resolve_decl(&decl, &[]);
    binder.close_scope();
    binder.close_scope();
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    match result.target_of(inner_ref.id).unwrap() {
        ResolvedTargetV1::Local(binding) => assert_eq!(binding.name, "r"),
        other => panic!("expected captured local, got {other:?}"),
    }
}

// Parametric view: a formal parameter is visible in the body (scenario 3).
#[test]
fn parameter_resolves_in_body() {
    let b = HirBuilderV1::new();
    let body = b.symbol("repo", SpanV1::new(30, 34));
    let decl = b.decl(
        "risky",
        SpanV1::new(5, 10),
        body.clone(),
        SpanV1::new(0, 34),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_decl(&decl, &[ParamV1::new("repo", SpanV1::new(11, 15), decl.id)]);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    match result.target_of(body.id).unwrap() {
        ResolvedTargetV1::Local(binding) => {
            assert_eq!(binding.kind, BinderKindV1::Param);
            assert_eq!(binding.name, "repo");
        }
        other => panic!("expected param, got {other:?}"),
    }
}

// A reference resolves to a top-level declaration in the same program.
#[test]
fn declaration_reference_resolves() {
    let b = HirBuilderV1::new();
    let other_body = b.int("1", SpanV1::new(0, 1));
    let other = b.decl("base", SpanV1::new(0, 4), other_body, SpanV1::new(0, 5));

    let ref_to_base = b.symbol("base", SpanV1::new(10, 14));
    let user = b.decl(
        "derived",
        SpanV1::new(8, 9),
        ref_to_base,
        SpanV1::new(8, 14),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.declare_program(&[other.clone(), user.clone()], &[]);
    binder.resolve_decl(&user, &[]);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    match result.target_of(user.body.id).unwrap() {
        ResolvedTargetV1::Declaration { name, node, .. } => {
            assert_eq!(name, "base");
            assert_eq!(*node, other.id);
        }
        other => panic!("expected declaration, got {other:?}"),
    }
}

#[test]
fn join_rhs_alias_is_hidden_from_lhs_but_visible_to_on_and_later_stages() {
    let b = HirBuilderV1::new();
    let lhs_source = b.relation_source(
        "catalog.left",
        SpanV1::new(0, 10),
        Some(alias("l", 10)),
        SpanV1::new(0, 11),
    );
    let lhs_source_id = lhs_source.id;
    let premature_rhs_ref = b.symbol("r", SpanV1::new(20, 21));
    let lhs = b.relation_filter(
        lhs_source,
        b.bool_value(premature_rhs_ref.clone(), SpanV1::new(20, 21)),
        SpanV1::new(0, 21),
    );
    let rhs = b.relation_source(
        "catalog.right",
        SpanV1::new(30, 40),
        Some(alias("r", 40)),
        SpanV1::new(30, 41),
    );
    let rhs_id = rhs.id;
    let on_lhs_ref = b.symbol("l", SpanV1::new(48, 49));
    let on_rhs_ref = b.symbol("r", SpanV1::new(50, 51));
    let joined = b.inner_join(
        lhs,
        rhs,
        b.compare(
            "=",
            on_lhs_ref.clone(),
            on_rhs_ref.clone(),
            SpanV1::new(48, 51),
        ),
        SpanV1::new(0, 51),
    );
    let joined_id = joined.id;
    let later_rhs_ref = b.symbol("r", SpanV1::new(60, 61));
    let query = b.query(
        b.relation_filter(
            joined,
            b.bool_value(later_rhs_ref.clone(), SpanV1::new(60, 61)),
            SpanV1::new(0, 61),
        ),
        None,
        SpanV1::new(0, 61),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), "binder.unresolved_symbol");
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(20, 21));
    assert!(result.target_of(premature_rhs_ref.id).is_none());
    for (reference, expected_name, expected_identity) in [
        (on_lhs_ref.id, "l", (lhs_source_id, 0)),
        (on_rhs_ref.id, "r", (rhs_id, 0)),
        (later_rhs_ref.id, "r", (rhs_id, 0)),
    ] {
        match result.target_of(reference).unwrap() {
            ResolvedTargetV1::Local(binding) => {
                assert_eq!(binding.name, expected_name);
                assert_eq!(binding.kind, BinderKindV1::Alias);
                assert_eq!(binding.identity(), expected_identity);
            }
            other => panic!("expected relation alias, got {other:?}"),
        }
    }
    assert_eq!(
        result
            .relation_output(joined_id)
            .unwrap()
            .iter()
            .map(|binding| (binding.name.as_str(), binding.identity()))
            .collect::<Vec<_>>(),
        vec![("l", (lhs_source_id, 0)), ("r", (rhs_id, 0))]
    );
}

#[test]
fn projection_replaces_row_frame_and_exposes_only_named_outputs() {
    let b = HirBuilderV1::new();
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(0, 10),
        Some(alias("u", 10)),
        SpanV1::new(0, 11),
    );
    let source_id = source.id;
    let projected_input = b.symbol("u", SpanV1::new(20, 21));
    let projected = b
        .try_project(
            source,
            vec![ProjectionItemV1 {
                value: projected_input.clone(),
                alias: Some(alias("user", 30)),
            }],
            SpanV1::new(0, 31),
        )
        .unwrap();
    let projected_id = projected.id;
    let output_ref = b.symbol("user", SpanV1::new(40, 44));
    let after_projection = b
        .order_by(
            projected,
            vec![OrderKeyHirV1::nulls_last(
                output_ref.clone(),
                SortDirectionHirV1::Asc,
            )],
            SpanV1::new(0, 44),
        )
        .unwrap();
    let order_id = after_projection.id;
    let query = b.query(after_projection, None, SpanV1::new(0, 44));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    assert!(matches!(
        result.target_of(projected_input.id),
        Some(ResolvedTargetV1::Local(binding))
            if binding.name == "u"
                && binding.kind == BinderKindV1::Alias
                && binding.identity() == (source_id, 0)
    ));
    assert!(matches!(
        result.target_of(output_ref.id),
        Some(ResolvedTargetV1::Local(binding))
            if binding.name == "user"
                && binding.kind == BinderKindV1::Alias
                && binding.identity() == (projected_id, 0)
    ));
    assert_eq!(
        result
            .relation_output(source_id)
            .unwrap()
            .iter()
            .map(|binding| binding.name.as_str())
            .collect::<Vec<_>>(),
        vec!["u"]
    );
    for stage in [projected_id, order_id] {
        assert_eq!(
            result
                .relation_output(stage)
                .unwrap()
                .iter()
                .map(|binding| binding.name.as_str())
                .collect::<Vec<_>>(),
            vec!["user"]
        );
    }
    assert_eq!(result.relation_outputs().len(), 3);
}

#[test]
fn aliasless_path_projection_exposes_final_segment_to_order_by() {
    let b = HirBuilderV1::new();
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(0, 10),
        Some(alias("u", 10)),
        SpanV1::new(0, 11),
    );
    let path = b
        .try_path(
            b.symbol("u", SpanV1::new(20, 21)),
            vec![quanta_gqlang_v2_hir::PathSegmentV1 {
                name: "id".to_owned(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let projected = b
        .try_project(
            source,
            vec![ProjectionItemV1 {
                value: path,
                alias: None,
            }],
            SpanV1::new(0, 24),
        )
        .unwrap();
    let output_id = projected.id;
    let order_ref = b.symbol("id", SpanV1::new(35, 37));
    let ordered = b
        .order_by(
            projected,
            vec![OrderKeyHirV1::nulls_last(
                order_ref.clone(),
                SortDirectionHirV1::Asc,
            )],
            SpanV1::new(0, 37),
        )
        .unwrap();
    let query = b.query(ordered, None, SpanV1::new(0, 37));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    assert!(matches!(
        result.target_of(order_ref.id),
        Some(ResolvedTargetV1::Local(binding))
            if binding.name == "id" && binding.identity() == (output_id, 0)
    ));
    assert_eq!(result.relation_output(output_id).unwrap().len(), 1);
}

#[test]
fn aliasless_symbol_projection_exposes_its_symbol_name() {
    let b = HirBuilderV1::new();
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(0, 10),
        Some(alias("u", 10)),
        SpanV1::new(0, 11),
    );
    let projected = b
        .try_project(
            source,
            vec![ProjectionItemV1 {
                value: b.symbol("u", SpanV1::new(20, 21)),
                alias: None,
            }],
            SpanV1::new(0, 21),
        )
        .unwrap();
    let projected_id = projected.id;
    let query = b.query(projected, None, SpanV1::new(0, 21));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    assert_eq!(result.relation_output(projected_id).unwrap()[0].name, "u");
    assert_eq!(
        result.relation_output(projected_id).unwrap()[0].identity(),
        (projected_id, 0)
    );
}

#[test]
fn projection_drops_input_aliases_from_later_stages() {
    let b = HirBuilderV1::new();
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(0, 10),
        Some(alias("u", 10)),
        SpanV1::new(0, 11),
    );
    let projected_input = b.symbol("u", SpanV1::new(20, 21));
    let projected = b
        .try_project(
            source,
            vec![ProjectionItemV1 {
                value: projected_input,
                alias: Some(alias("user", 30)),
            }],
            SpanV1::new(0, 31),
        )
        .unwrap();
    let dropped_ref = b.symbol("u", SpanV1::new(40, 41));
    let query = b.query(
        b.relation_filter(
            projected,
            b.bool_value(dropped_ref.clone(), SpanV1::new(40, 41)),
            SpanV1::new(0, 41),
        ),
        None,
        SpanV1::new(0, 41),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), "binder.unresolved_symbol");
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(40, 41));
    assert!(result.target_of(dropped_ref.id).is_none());
}

#[test]
fn duplicate_projection_aliases_reject_without_last_wins_resolution() {
    let b = HirBuilderV1::new();
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(0, 10),
        Some(alias("u", 10)),
        SpanV1::new(0, 11),
    );
    let first_value = b.symbol("u", SpanV1::new(20, 21));
    let second_value = b.symbol("u", SpanV1::new(24, 25));
    let projected = b
        .try_project(
            source,
            vec![
                ProjectionItemV1 {
                    value: first_value,
                    alias: Some(alias("same", 30)),
                },
                ProjectionItemV1 {
                    value: second_value,
                    alias: Some(AliasV1 {
                        name: "same".to_owned(),
                        span: SpanV1::new(40, 44),
                    }),
                },
            ],
            SpanV1::new(0, 44),
        )
        .unwrap();
    let query = b.query(projected, None, SpanV1::new(0, 44));

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(
        result.diagnostics()[0].code(),
        "binder.duplicate_declaration"
    );
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(40, 44));
}

#[test]
fn relational_exists_records_exact_nonempty_and_explicit_empty_capture_sets() {
    use quanta_gqlang_v2_hir::RelationExistsPolarityV1;

    let b = HirBuilderV1::new();
    let outer = b.relation_source(
        "catalog.repos",
        SpanV1::new(0, 10),
        Some(alias("r", 10)),
        SpanV1::new(0, 11),
    );
    let inner_source = b.relation_source(
        "catalog.files",
        SpanV1::new(20, 30),
        Some(alias("f", 30)),
        SpanV1::new(20, 31),
    );
    let inner_local = b.symbol("f", SpanV1::new(40, 41));
    let outer_capture = b.symbol("r", SpanV1::new(44, 45));
    let inner_relation = b.relation_filter(
        inner_source,
        b.compare("=", inner_local, outer_capture.clone(), SpanV1::new(40, 45)),
        SpanV1::new(20, 45),
    );
    let correlated = b.relation_exists(
        RelationExistsPolarityV1::Exists,
        inner_relation,
        SpanV1::new(18, 46),
    );
    let correlated_id = correlated.id;

    let uncorrelated_relation = b.relation_source(
        "catalog.owners",
        SpanV1::new(50, 60),
        Some(alias("o", 60)),
        SpanV1::new(50, 61),
    );
    let uncorrelated = b.relation_exists(
        RelationExistsPolarityV1::Exists,
        uncorrelated_relation,
        SpanV1::new(48, 62),
    );
    let uncorrelated_id = uncorrelated.id;
    let predicate = b.and(correlated, uncorrelated, SpanV1::new(18, 62));
    let query = b.query(
        b.relation_filter(outer.clone(), predicate, SpanV1::new(0, 62)),
        None,
        SpanV1::new(0, 62),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    assert_eq!(
        result.capture_edges(correlated_id),
        &[(outer_capture.id, outer.id, 0)]
    );
    assert!(result.has_capture_set(correlated_id));
    assert!(result.has_capture_set(uncorrelated_id));
    assert!(result.capture_edges(uncorrelated_id).is_empty());
    let capture_sets = result.capture_sets().collect::<Vec<_>>();
    assert_eq!(
        capture_sets,
        vec![
            (correlated_id, [(outer_capture.id, outer.id, 0)].as_slice()),
            (uncorrelated_id, [].as_slice()),
        ]
    );
}

#[test]
fn nested_exists_attributes_capture_to_each_crossed_boundary() {
    use quanta_gqlang_v2_hir::RelationExistsPolarityV1;

    let b = HirBuilderV1::new();
    let root = b.relation_source(
        "catalog.root",
        SpanV1::new(0, 10),
        Some(alias("r", 10)),
        SpanV1::new(0, 11),
    );
    let middle_source = b.relation_source(
        "catalog.middle",
        SpanV1::new(20, 30),
        Some(alias("m", 30)),
        SpanV1::new(20, 31),
    );
    let inner_source = b.relation_source(
        "catalog.inner",
        SpanV1::new(40, 50),
        Some(alias("i", 50)),
        SpanV1::new(40, 51),
    );
    let root_ref = b.symbol("r", SpanV1::new(60, 61));
    let middle_ref = b.symbol("m", SpanV1::new(64, 65));
    let inner_relation = b.relation_filter(
        inner_source,
        b.compare(
            "=",
            root_ref.clone(),
            middle_ref.clone(),
            SpanV1::new(60, 65),
        ),
        SpanV1::new(40, 65),
    );
    let inner_exists = b.relation_exists(
        RelationExistsPolarityV1::Exists,
        inner_relation,
        SpanV1::new(38, 66),
    );
    let inner_id = inner_exists.id;
    let middle_relation =
        b.relation_filter(middle_source.clone(), inner_exists, SpanV1::new(20, 66));
    let outer_exists = b.relation_exists(
        RelationExistsPolarityV1::Exists,
        middle_relation,
        SpanV1::new(18, 67),
    );
    let outer_id = outer_exists.id;
    let query = b.query(
        b.relation_filter(root.clone(), outer_exists, SpanV1::new(0, 67)),
        None,
        SpanV1::new(0, 67),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_query(&query);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    assert_eq!(
        result.capture_edges(inner_id),
        &[
            (root_ref.id, root.id, 0),
            (middle_ref.id, middle_source.id, 0)
        ]
    );
    assert_eq!(result.capture_edges(outer_id), &[(root_ref.id, root.id, 0)]);
}

#[test]
fn multiple_quantifier_binders_have_distinct_stable_identities() {
    use quanta_gqlang_v2_hir::{LogicalBinderV1, LogicalQuantifierKindV1, NonEmptyBindersV1};

    let b = HirBuilderV1::new();
    let x_ref = b.symbol("x", SpanV1::new(30, 31));
    let y_ref = b.symbol("y", SpanV1::new(34, 35));
    let body = b.compare("=", x_ref.clone(), y_ref.clone(), SpanV1::new(30, 35));
    let binders = NonEmptyBindersV1::try_new(vec![
        LogicalBinderV1 {
            name: "x".to_owned(),
            span: SpanV1::new(20, 21),
        },
        LogicalBinderV1 {
            name: "y".to_owned(),
            span: SpanV1::new(24, 25),
        },
    ])
    .unwrap();
    let quant = b.logical_quantifier(
        LogicalQuantifierKindV1::Exists,
        binders,
        body,
        SpanV1::new(18, 35),
    );
    let quant_id = quant.id;
    let rule = b.rule(
        "r",
        SpanV1::new(0, 1),
        b.int("1", SpanV1::new(10, 11)),
        quant,
        SpanV1::new(0, 35),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_rule(&rule, &[]);
    let result = binder.finish();

    assert!(result.is_ok(), "diags: {:?}", result.diagnostics());
    assert!(matches!(
        result.target_of(x_ref.id),
        Some(ResolvedTargetV1::Local(binding)) if binding.identity() == (quant_id, 0)
    ));
    assert!(matches!(
        result.target_of(y_ref.id),
        Some(ResolvedTargetV1::Local(binding)) if binding.identity() == (quant_id, 1)
    ));
}
