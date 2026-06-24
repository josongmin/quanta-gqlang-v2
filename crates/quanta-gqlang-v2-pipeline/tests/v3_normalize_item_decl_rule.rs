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

use quanta_gqlang_v2_pipeline::{compile_program, CompiledItemV1, NoServeReasonV1};

/// A `let` declaration compiles into an explicit value artifact with no plan.
#[test]
fn decl_compiles_through_algebra_seam() {
    let src = "let x = 1 + 2";
    let compiled = compile_program(src).expect("decl compiles end to end");
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

/// A horn `rule` compiles into an explicit logic artifact with no plan.
#[test]
fn rule_compiles_through_algebra_seam() {
    let src = "rule adult : person.id if person.age >= 18";
    let compiled = compile_program(src).expect("rule compiles end to end");
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
