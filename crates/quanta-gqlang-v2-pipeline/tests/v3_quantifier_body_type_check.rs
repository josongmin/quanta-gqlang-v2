//! v3 rail: type errors inside a quantifier body must NOT pass silently.
//!
//! Defect `quanta-v3-quantifier-type-skip`: `type_check_bool` recursed into a
//! quantifier body without introducing the quantifier binders into the type
//! environment. Every body reference to a binder (e.g. `t`) was therefore a
//! free, env-open symbol, so `type_check_value` skipped the whole expression and
//! a genuine type error (`t + "string"`) compiled straight through to planning.
//!
//! The fix extends the env with the quantifier binders (typed `null` as a
//! placeholder) before checking the body, so a body that is now closed under the
//! extended env gets type-checked and rejects at the `type` stage.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    clippy::match_wildcard_for_single_variants,
    reason = "integration test rail: assertion panics and exhaustive matches are intentional"
)]

use quanta_gqlang_v2_pipeline::{compile_program, PipelineDiagnosticV1, PipelineStageV1};

/// A type error inside an `exists`-quantifier body rejects at the `type` stage
/// rather than compiling through. The binder `t` is in scope of the body, so
/// `t + "string"` is now closed under the (binder-extended) env and the
/// arithmetic-on-string mistake is caught.
#[test]
fn quantifier_body_type_error_rejects_at_type() {
    let src = "from users u | where exists t in ( t + \"string\" );";
    let diag = compile_program(src).expect_err("type error inside quantifier body must reject");

    assert_eq!(diag.stage(), PipelineStageV1::Type, "tagged type stage");
    assert_eq!(diag.stage().as_str(), "type");
    match &diag {
        PipelineDiagnosticV1::Type(inner) => {
            assert_eq!(inner.code(), "types.bad_overload");
        }
        other => panic!("expected Type diagnostic, got {other:?}"),
    }
}

/// Control: a quantifier body that is well typed under the binder-extended env
/// still compiles cleanly. A bare binder reference `( t )` types as the
/// placeholder `null` with no operator applied, so no spurious rejection appears
/// and the binder-extension fix does not over-reject.
#[test]
fn well_typed_quantifier_body_still_compiles() {
    let src = "from users u | where exists t in ( t ) | project u.id as id";
    let _ = compile_program(src).expect("well-typed quantifier body must compile cleanly");
}
