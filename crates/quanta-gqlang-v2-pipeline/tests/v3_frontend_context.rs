#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration rail"
)]

use quanta_gqlang_v2_binder::ModuleGraphV1;
use quanta_gqlang_v2_pipeline::{
    compile_program_structural, compile_with_context, FrontendContextV1, PipelineStageV1,
};
use quanta_gqlang_v2_types::{CardinalityV1, RecordFieldTyV1, TypeEnvV1, TypeV1};

fn users_relation_schema() -> TypeEnvV1 {
    TypeEnvV1::new().with(
        "users",
        TypeV1::Relation {
            row: Box::new(TypeV1::record(vec![
                RecordFieldTyV1::new("id", TypeV1::int()),
                RecordFieldTyV1::new("name", TypeV1::string()),
            ])),
            cardinality: CardinalityV1::Bag,
        },
    )
}

#[test]
fn context_first_compile_closes_alias_typing_explicitly() {
    let src = "from users u | where u.id + \"x\" > 1 | project u.id as id";

    let structural = compile_program_structural(src)
        .expect("structural compile stays permissive without authority");
    assert!(
        structural.plan().is_some(),
        "structural path still compiles"
    );

    let context = FrontendContextV1::new(ModuleGraphV1::new(), users_relation_schema());
    let diag = compile_with_context(src, &context)
        .expect_err("explicit semantic authority must type-check alias expressions");

    assert_eq!(diag.stage(), PipelineStageV1::Type);
    assert_eq!(diag.code(), "types.bad_overload");
}
