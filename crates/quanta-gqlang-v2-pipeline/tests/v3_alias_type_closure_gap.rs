//! v3 lock rail: relation-source schemas must keep lifting into alias/local type
//! envs.
//!
//! The public query pipeline accepts a caller-provided [`TypeEnvV1`], and the
//! schema-only entry point must now type alias-dependent expressions under a
//! lifted local environment rather than skipping them.

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "integration rail"
)]

use quanta_gqlang_v2_pipeline::compile_program_with_schema;
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
    let diag = compile_program_with_schema(src, &users_relation_schema())
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
    let diag = compile_program_with_schema(src, &users_relation_schema())
        .expect_err("unknown alias field should reject at type once alias env lifting exists");
    assert_eq!(diag.stage().as_str(), "type");
    assert_eq!(diag.code(), "types.bad_projection");
}

/// D25 lock rail: when a source has an explicit alias, the bare source name must
/// NOT be lifted into the local type environment as a hidden row binding.
///
/// Contrast the alias form, which DOES resolve against the lifted local row and
/// therefore rejects with `types.bad_projection` on a missing field. The bare
/// source-name form must not reach that same typed row through an implicit
/// binding.
#[test]
fn aliased_source_does_not_lift_bare_source_name_into_local_env() {
    let via_alias = "from users u | project u.ghost as id";
    let alias_diag = compile_program_with_schema(via_alias, &users_relation_schema())
        .expect_err("aliased reference to a missing field should reject at type");
    assert_eq!(alias_diag.stage().as_str(), "type");
    assert_eq!(alias_diag.code(), "types.bad_projection");

    let via_source_name = "from users u | project users.ghost as id";
    let source_diag = compile_program_with_schema(via_source_name, &users_relation_schema())
        .expect_err("bare source name must be unbound in the value namespace (D25)");
    assert_eq!(
        source_diag.stage().as_str(),
        "bind",
        "bare source name `users` must reject at bind, not lift a hidden row binding (D25)"
    );
    assert_eq!(source_diag.code(), "binder.unresolved_symbol");

    let no_alias = "from users | project users.id as id";
    let no_alias_diag = compile_program_with_schema(no_alias, &users_relation_schema())
        .expect_err("bare source name must be unbound even without an alias (D25)");
    assert_eq!(no_alias_diag.stage().as_str(), "bind");
    assert_eq!(no_alias_diag.code(), "binder.unresolved_symbol");
}
