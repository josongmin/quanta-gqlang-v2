#![expect(
    clippy::indexing_slicing,
    clippy::unwrap_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_binder::{Binder, BinderKindV1, ModuleGraphV1, ResolvedTargetV1};
use quanta_gqlang_v2_hir::{
    AliasV1, CatalogPayloadV1, GraphRefV1, HirBuilderV1, ProjectionItemV1, SpanV1, ViewBodyV1,
};

fn alias(name: &str, start: usize) -> AliasV1 {
    AliasV1 {
        name: name.to_owned(),
        span: SpanV1::new(start, start + name.len()),
    }
}

#[test]
fn inline_view_uses_query_alias_frame_for_projection_symbols() {
    let b = HirBuilderV1::new();
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(12, 25),
        Some(alias("u", 29)),
        SpanV1::new(12, 30),
    );
    let source_id = source.id;
    let use_alias = b.symbol("u", SpanV1::new(38, 39));
    let use_alias_id = use_alias.id;
    let query = b.query(
        b.try_project(
            source,
            vec![ProjectionItemV1 {
                value: use_alias,
                alias: Some(alias("user", 43)),
            }],
            SpanV1::new(12, 47),
        )
        .unwrap(),
        None,
        SpanV1::new(5, 47),
    );
    let view = b.catalog_decl(
        "users_view",
        SpanV1::new(0, 10),
        CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(query))),
        SpanV1::new(0, 47),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_catalog_decl(&view);
    let result = binder.finish();

    assert!(result.is_ok(), "diagnostics: {:?}", result.diagnostics());
    assert!(matches!(result.target_of(use_alias_id),
        Some(ResolvedTargetV1::Local(binding))
            if binding.kind == BinderKindV1::Alias
                && binding.name == "u"
                && binding.identity() == (source_id, 0)));
}

#[test]
fn inline_view_reports_unresolved_projection_symbol_at_the_symbol_span() {
    let b = HirBuilderV1::new();
    let source = b.relation_source(
        "catalog.users",
        SpanV1::new(12, 25),
        Some(alias("u", 29)),
        SpanV1::new(12, 30),
    );
    let missing = b.symbol("missing", SpanV1::new(38, 45));
    let query = b.query(
        b.try_project(
            source,
            vec![ProjectionItemV1 {
                value: missing,
                alias: Some(alias("user", 49)),
            }],
            SpanV1::new(12, 53),
        )
        .unwrap(),
        None,
        SpanV1::new(5, 53),
    );
    let view = b.catalog_decl(
        "users_view",
        SpanV1::new(0, 10),
        CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(query))),
        SpanV1::new(0, 53),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_catalog_decl(&view);
    let result = binder.finish();

    assert_eq!(result.diagnostics().len(), 1);
    assert_eq!(result.diagnostics()[0].code(), "binder.unresolved_symbol");
    assert_eq!(result.diagnostics()[0].name(), "missing");
    assert_eq!(result.diagnostics()[0].primary_span(), SpanV1::new(38, 45));
}

#[test]
fn non_inline_catalog_payload_remains_outside_binder_authority() {
    let b = HirBuilderV1::new();
    let model = b.catalog_decl(
        "model",
        SpanV1::new(0, 5),
        CatalogPayloadV1::Model {
            provider: "remote".into(),
            source: GraphRefV1 {
                name: "not.a.local.binder".into(),
                span: SpanV1::new(19, 37),
            },
        },
        SpanV1::new(0, 37),
    );

    let graph = ModuleGraphV1::new();
    let mut binder = Binder::new(&graph);
    binder.resolve_catalog_decl(&model);
    let result = binder.finish();

    assert!(result.is_ok(), "diagnostics: {:?}", result.diagnostics());
    assert!(result.resolutions().is_empty());
}
