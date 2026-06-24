#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "contract test assertions intentionally fail hard on seam drift"
)]

use quanta_gqlang_v2_frontend_contract::{
    compile_program, compile_with_context, compiled_program_item_v1, format_item, CompiledItemV1,
    FrontendContextV1, ProgramItemHirV1,
};
use quanta_gqlang_v2_types::{CardinalityV1, RecordFieldTyV1, TypeEnvV1, TypeV1};

#[test]
fn downstream_contract_reexports_compile_bundle_and_formatter_v1() {
    let program = compile_program("from users | project 1 as x").expect("query compiles");
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
    let program = compile_program("require certified").expect("meta compiles");
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
    let context = FrontendContextV1::from_schema(schema);
    let diag = compile_with_context("from users u | where u.id + \"x\" > 1", &context)
        .expect_err("context-first compile reexport must be live");
    assert_eq!(diag.code(), "types.bad_overload");
}
