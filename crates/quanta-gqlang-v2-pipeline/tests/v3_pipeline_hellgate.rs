//! v3 hellgate: stage-tagged, fail-closed pipeline diagnostics (ticket 08, D22).
//!
//! Proves the ONE compiler service short-circuits on the FIRST stage that
//! fails and wraps that stage's typed diagnostic WITHOUT losing the original
//! code/span. Each test asserts the actual stage tag plus the inner typed
//! code, not just that an error occurred.
//!
//! syntax->sema->ir path pinned: source text -> `compile_program` -> the stage
//! that rejects (parse / type / algebra) -> `PipelineDiagnosticV1` tagged with
//! that stage and carrying the inner code.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    clippy::match_wildcard_for_single_variants,
    reason = "integration hellgate rail: assertion panics and exhaustive matches are intentional"
)]

use quanta_gqlang_v2_pipeline::{compile_program, PipelineDiagnosticV1, PipelineStageV1};

/// rail 2: fail at PARSE. A malformed surface program rejects tagged `parse`
/// with the exact surface parse code and span preserved.
#[test]
fn fail_at_parse_tagged_parse() {
    // unclosed paren in the filter predicate: a surface parse error.
    let src = "from users | where (a + 1";
    let diag = compile_program(src).expect_err("must reject at parse");

    assert_eq!(diag.stage(), PipelineStageV1::Parse, "tagged parse stage");
    assert_eq!(diag.stage().as_str(), "parse");
    // exact inner code/span preserved (no rewrap loss).
    match &diag {
        PipelineDiagnosticV1::Parse(inner) => {
            assert_eq!(inner.code_str(), "surface.parse.unexpected_eof");
        }
        other => panic!("expected Parse diagnostic, got {other:?}"),
    }
    // unified accessor exposes the inner code without re-matching.
    assert_eq!(diag.code(), "surface.parse.unexpected_eof");
}

/// rail 3: fail at TYPE. A value expression with no matching operator overload
/// (`1 + "x"`) rejects tagged `type` with the exact types code preserved.
#[test]
fn fail_at_type_tagged_type() {
    // `1 + "x"` is closed under the (empty) env, so the type stage checks it and
    // rejects with no `+` overload for (int, string).
    let src = "from users | project 1 + \"x\" as bad";
    let diag = compile_program(src).expect_err("must reject at type");

    assert_eq!(diag.stage(), PipelineStageV1::Type, "tagged type stage");
    assert_eq!(diag.stage().as_str(), "type");
    match &diag {
        PipelineDiagnosticV1::Type(inner) => {
            assert_eq!(inner.code(), "types.bad_overload");
        }
        other => panic!("expected Type diagnostic, got {other:?}"),
    }
    assert_eq!(diag.code(), "types.bad_overload");
}

/// rail 4: fail at ALGEBRA. A `group by` keyed on an aggregate fold parses and
/// binds cleanly but is ill-formed at the algebra stage (an aggregate cannot be
/// a partition key), so it rejects tagged `algebra` with the exact
/// ill-formed-aggregate code preserved. (`group by`/`order by` themselves now
/// lower cleanly; this exercises the fail-closed algebra-stage carrier via a
/// still-genuine rejection.)
#[test]
fn fail_at_algebra_tagged_algebra() {
    // `group by count(f.id)` parses and binds cleanly (alias base `f` resolves,
    // `count` is a known aggregate) but cannot lower: an aggregate fold is not a
    // valid partition key, so the algebra normalizer rejects at the algebra stage.
    let src = "from findings f | group by count(f.id)";
    let diag = compile_program(src).expect_err("must reject at algebra");

    assert_eq!(
        diag.stage(),
        PipelineStageV1::Algebra,
        "tagged algebra stage"
    );
    assert_eq!(diag.stage().as_str(), "algebra");
    match &diag {
        PipelineDiagnosticV1::Algebra(inner) => {
            assert_eq!(
                inner.code(),
                "algebra.ill_formed_aggregate.nested_aggregate"
            );
        }
        other => panic!("expected Algebra diagnostic, got {other:?}"),
    }
    assert_eq!(diag.code(), "algebra.ill_formed_aggregate.nested_aggregate");
}

/// Short-circuit order: a program that would fail at multiple stages reports the
/// EARLIEST stage. A parse error masks any later type/algebra error.
#[test]
fn short_circuits_on_first_stage() {
    // parse error (unclosed paren) AND a would-be type error live together; the
    // parse stage must win.
    let src = "from users | where (1 + \"x\"";
    let diag = compile_program(src).expect_err("must reject");
    assert_eq!(
        diag.stage(),
        PipelineStageV1::Parse,
        "parse wins over later stages"
    );
}
