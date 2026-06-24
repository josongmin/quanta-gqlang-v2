//! v3 red rail: relation-source schemas are not lifted into alias/local type envs.
//!
//! The public query pipeline accepts a caller-provided [`TypeEnvV1`], but today
//! `compile_with` threads that env unchanged into value/bool type checking. A
//! source like `from users u` therefore does NOT synthesize `u -> <users row>`
//! in the local type environment, so alias-dependent expressions are treated as
//! "env-open" and skipped by the type stage rather than validated.
//!
//! These rails pin the structural gap. They stay RED until the pipeline grows a
//! real relation-source -> local-row-type lift (or another equivalent typed
//! binder->types handoff), at which point alias-dependent mistakes must reject
//! tagged `type` rather than compile through planning.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration rail"
)]

use quanta_gqlang_v2_pipeline::compile_with;
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

/// A typed row schema for `users` should make `u.id + "x"` reject at the type
/// stage once `from users u` lifts `u` into the local type environment.
///
#[test]
fn alias_bad_overload_should_reject_once_row_schema_is_lifted() {
    let src = "from users u | where u.id + \"x\" > 1 | project u.id as id";
    let diag = compile_with(src, &users_relation_schema())
        .expect_err("typed alias expression should reject at type once alias env lifting exists");
    assert_eq!(diag.stage().as_str(), "type");
    assert_eq!(diag.code(), "types.bad_overload");
}

/// A typed row schema for `users` should also make `u.ghost` reject as a bad
/// projection once alias/env lifting exists.
///
#[test]
fn alias_bad_projection_should_reject_once_row_schema_is_lifted() {
    let src = "from users u | where u.id > 1 | project u.ghost as id";
    let diag = compile_with(src, &users_relation_schema())
        .expect_err("unknown alias field should reject at type once alias env lifting exists");
    assert_eq!(diag.stage().as_str(), "type");
    assert_eq!(diag.code(), "types.bad_projection");
}
