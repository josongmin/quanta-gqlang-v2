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

use quanta_gqlang_v2_frontend_contract::{
    compile_document_structural, compile_item_structural, compile_program_structural,
    compile_with_context, compiled_program_item_v1, format_item, CatalogDeclHirV1,
    CatalogPayloadV1, CompiledItemV1, FrontendContextV1, GraphRefV1, HirBuilderV1, HirNodeIdV1,
    PipelineStageV1, ProgramItemHirV1, SpanV1, ViewBodyV1,
};
use quanta_gqlang_v2_types::{CardinalityV1, RecordFieldTyV1, TypeEnvV1, TypeV1};

fn span() -> SpanV1 {
    SpanV1::new(0, 1)
}

fn view_decl(raw_id: u32, name: &str) -> ProgramItemHirV1 {
    let span = span();
    let decl = CatalogDeclHirV1::new(
        HirNodeIdV1::from_raw(raw_id),
        span,
        name,
        span,
        CatalogPayloadV1::View(ViewBodyV1::QueryRef(GraphRefV1 {
            name: "base".to_owned(),
            span,
        })),
    );
    ProgramItemHirV1::CatalogDecl(Box::new(decl))
}

#[test]
fn downstream_contract_reexports_compile_bundle_and_formatter_v1() {
    let program =
        compile_program_structural("from users | project 1 as x").expect("query compiles");
    assert!(matches!(
        program.compiled(),
        CompiledItemV1::RelationalQuery { .. }
    ));
    let item = compiled_program_item_v1(&program);
    assert!(matches!(item, ProgramItemHirV1::Query(_)));
    assert_eq!(format_item(&item), "from users | project 1 as x");
}

#[test]
fn downstream_contract_exposes_non_relational_family_inspection_v1() {
    let program = compile_program_structural("require certified").expect("meta compiles");
    assert!(matches!(program.compiled(), CompiledItemV1::Meta { .. }));
    let item = compiled_program_item_v1(&program);
    assert!(matches!(item, ProgramItemHirV1::Meta(_)));
}

#[test]
fn downstream_contract_reexports_context_first_compile_surface_v1() {
    let schema = TypeEnvV1::new().with(
        "users",
        TypeV1::Relation {
            row: Box::new(TypeV1::record(vec![RecordFieldTyV1::new(
                "id",
                TypeV1::int(),
            )])),
            cardinality: CardinalityV1::Bag,
        },
    );
    let context = FrontendContextV1::schema_only(schema);
    let diag = compile_with_context("from users u | where u.id + \"x\" > 1", &context)
        .expect_err("context-first compile reexport must be live");
    assert_eq!(diag.code(), "types.bad_overload");
}

#[test]
fn downstream_contract_reexports_item_structural_helper_v1() {
    let builder = HirBuilderV1::new();
    let span = span();
    let decl = builder.decl("x", span, builder.int("1", span), span);
    let item = ProgramItemHirV1::from(decl);

    let compiled = compile_item_structural(&item).expect("structural item helper must compile");
    assert!(
        matches!(compiled, CompiledItemV1::ValueArtifact { .. }),
        "structural item helper must surface the explicit non-servable artifact"
    );
}

#[test]
fn downstream_contract_reexports_document_structural_scope_v1() {
    let items = vec![view_decl(1, "x"), view_decl(2, "x")];

    let diag = compile_document_structural(&items)
        .expect_err("document helper must preserve shared catalog scope");
    assert_eq!(diag.stage(), PipelineStageV1::Modules);
    assert_eq!(diag.code(), "modules.duplicate_catalog_name");
}
