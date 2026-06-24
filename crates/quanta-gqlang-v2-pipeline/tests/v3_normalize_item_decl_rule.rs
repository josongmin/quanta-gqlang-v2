//! v3 rail: decl/rule units compile into explicit non-servable artifacts.
//!
//! These rails pin the explicit artifact/no-serve contract: a standalone `let`
//! becomes a value artifact, and a horn `rule` becomes a logic artifact. Neither
//! is forced through relational algebra or pretends to be servable.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "integration test rail: assertion panics are intentional"
)]

use quanta_gqlang_v2_pipeline::{
    compile_program_structural, compile_program_with_schema, CompiledItemV1, NoServeReasonV1,
    PipelineStageV1,
};
use quanta_gqlang_v2_types::{TypeEnvV1, TypeV1};

/// A `let` declaration compiles into an explicit value artifact with no plan.
#[test]
fn decl_compiles_through_algebra_seam() {
    let src = "let x = 1 + 2";
    let compiled = compile_program_structural(src).expect("decl compiles end to end");
    assert!(
        compiled.algebra().is_none(),
        "decl has no relational algebra root"
    );
    assert!(compiled.plan().is_none(), "decl has no serving plan");
    match compiled.compiled() {
        CompiledItemV1::ValueArtifact {
            value_type,
            no_serve_reason,
            ..
        } => {
            assert_eq!(*no_serve_reason, NoServeReasonV1::ValueDeclaration);
            assert_eq!(
                value_type.as_ref().map(ToString::to_string),
                Some("int".to_owned())
            );
        }
        other @ CompiledItemV1::RelationalQuery { .. }
        | other @ CompiledItemV1::GraphQuery { .. }
        | other @ CompiledItemV1::RuleArtifact { .. }
        | other @ CompiledItemV1::Legality { .. }
        | other @ CompiledItemV1::Presentation { .. }
        | other @ CompiledItemV1::CatalogRegistration { .. }
        | other @ CompiledItemV1::Composition { .. }
        | other @ CompiledItemV1::Meta { .. } => {
            panic!("expected ValueArtifact, got {other:?}")
        }
    }
}

/// A closed declaration body with a real type error is still type-checked; the
/// declaration lane does not silently skip closed bodies just because they are
/// non-servable artifacts.
#[test]
fn decl_closed_body_type_error_rejects_at_type() {
    let diag = compile_program_structural("let bad = 1 + \"x\"")
        .expect_err("closed decl body with a bad overload must reject at type");
    assert_eq!(diag.stage(), PipelineStageV1::Type);
    assert_eq!(diag.code(), "types.bad_overload");
}

/// A free symbol in a standalone declaration body is rejected at bind, not
/// silently type-skipped under the caller schema. Standalone `let` bodies have
/// no binder-visible locals to lift.
#[test]
fn decl_free_symbol_rejects_at_bind_not_type_skip() {
    let schema = TypeEnvV1::new().with("k", TypeV1::int());
    let diag = compile_program_with_schema("let bad = k + 1", &schema)
        .expect_err("free decl-body symbol must fail closed at bind");
    assert_eq!(diag.stage(), PipelineStageV1::Bind);
    assert_eq!(diag.code(), "binder.unresolved_symbol");
}

/// A horn `rule` compiles into an explicit logic artifact with no plan.
#[test]
fn rule_compiles_through_algebra_seam() {
    let src = "rule adult : person.id if person.age >= 18";
    let compiled = compile_program_structural(src).expect("rule compiles end to end");
    assert!(
        compiled.algebra().is_none(),
        "rule has no relational algebra root"
    );
    assert!(compiled.plan().is_none(), "rule has no serving plan");
    match compiled.compiled() {
        CompiledItemV1::RuleArtifact {
            proof,
            no_serve_reason,
            ..
        } => {
            assert_eq!(*no_serve_reason, NoServeReasonV1::RuleArtifact);
            assert_eq!(proof.stratum_count, 1);
        }
        other @ CompiledItemV1::RelationalQuery { .. }
        | other @ CompiledItemV1::ValueArtifact { .. }
        | other @ CompiledItemV1::GraphQuery { .. }
        | other @ CompiledItemV1::Legality { .. }
        | other @ CompiledItemV1::Presentation { .. }
        | other @ CompiledItemV1::CatalogRegistration { .. }
        | other @ CompiledItemV1::Composition { .. }
        | other @ CompiledItemV1::Meta { .. } => {
            panic!("expected RuleArtifact, got {other:?}")
        }
    }
}
