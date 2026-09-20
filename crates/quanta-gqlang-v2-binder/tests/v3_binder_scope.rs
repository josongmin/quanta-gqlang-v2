//! v3 binder positive scope rail.
//!
//! syntax->HIR->binder path: a query HIR with a relation alias plus a local
//! `let`-binding resolves every bare symbol reference to the right binder
//! (`D3`, `D25`). Builds HIR directly with `HirBuilderV1` because the v3 parser
//! (ticket 06) is not present yet.

#![allow(clippy::unwrap_used, reason = "tests may unwrap")]
#![allow(clippy::indexing_slicing, reason = "tests may index")]
#![allow(clippy::panic, reason = "tests assert by panicking")]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "tests match the expected variant and panic on the rest"
)]

use quanta_gqlang_v2_binder::{
    Binder, BinderKindV1, CaptureV1, ModuleGraphV1, ParamV1, ResolvedTargetV1,
};
use quanta_gqlang_v2_hir::{AliasV1, HirBuilderV1, ProjectionItemV1, SpanV1};

// `SELECT u.id FROM catalog.users AS u WHERE u.active = true` (scenario 1):
// alias `u` introduced by the source, referenced by both projection and filter.
#[test]
fn alias_resolves_in_projection_and_filter() {
    let b = HirBuilderV1::new();
    let alias = AliasV1 {
        name: "u".to_owned(),
        span: SpanV1::new(30, 31),
    };
    let source = b.relation_source("catalog.users", Some(alias), SpanV1::new(15, 31));

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
    let projected = b
        .try_project(
            source,
            vec![ProjectionItemV1 {
                value: proj_path,
                alias: None,
            }],
            SpanV1::new(0, 40),
        )
        .unwrap();

    // filter: u.active = true
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
    let filtered = b.relation_filter(projected, pred, SpanV1::new(0, 53));

    let query = b.query(filtered, None, SpanV1::new(0, 53));

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
