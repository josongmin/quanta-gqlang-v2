//! Hellgate (fail-closed) rails for the v3 type checker (ticket 03).
//!
//! syntax -> sema(types) reject path: malformed value trees reject with TYPED
//! diagnostics (`TypeErrorV1`) carrying a stable `code()` and source span. Each
//! test asserts the ACTUAL code and Display text, per the hellgate protocol.
//!
//! Covers ticket scenarios:
//!   2. type mismatch `"x" + 1` -> fail-closed mismatch/bad-overload diagnostic
//!   3. bad overload (operator applied to unsupported types) -> diagnostic
//! plus: unknown symbol/operator/builtin, arity, and illegal-aggregate rejects.

#![allow(clippy::unwrap_used, reason = "tests assert on Result directly")]
#![allow(clippy::expect_used, reason = "tests assert on Result directly")]
#![allow(clippy::indexing_slicing, reason = "tests index known-shape fixtures")]
#![allow(clippy::panic, reason = "tests panic on unexpected error shapes")]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "tests match the one expected variant and panic otherwise"
)]

use quanta_gqlang_v2_hir::{HirBuilderV1, SpanV1};
use quanta_gqlang_v2_types::{
    IllegalAggregateReasonV1, ScalarTyV1, TypeCheckerV1, TypeEnvV1, TypeErrorV1, TypeV1,
};

fn sp(start: usize, end: usize) -> SpanV1 {
    SpanV1::new(start, end)
}

#[test]
fn scenario_2_string_plus_int_rejects_bad_overload() {
    // "x" + 1  has no `+` overload for (string, int) -> bad overload.
    let b = HirBuilderV1::new();
    let s = b.string("x", sp(0, 3));
    let one = b.int("1", sp(6, 7));
    let bad = b.binary("+", s, one, sp(0, 7));

    let checker = TypeCheckerV1::new();
    let err = checker
        .type_value(&bad, &TypeEnvV1::new())
        .expect_err("string + int must reject");

    assert_eq!(err.code(), "types.bad_overload");
    match &err {
        TypeErrorV1::BadOverload {
            operator, operands, ..
        } => {
            assert_eq!(operator, "+");
            assert_eq!(operands, &vec![TypeV1::string(), TypeV1::int()]);
        }
        other => panic!("expected BadOverload, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "types.bad_overload: operator `+` has no overload for (string, int)"
    );
    assert_eq!(err.span(), sp(0, 7));
}

#[test]
fn scenario_3_ordering_on_bool_rejects_bad_overload() {
    // true < false : `<` has no (bool, bool) overload -> bad overload.
    let b = HirBuilderV1::new();
    let t = b.bool_lit(true, sp(0, 4));
    let f = b.bool_lit(false, sp(7, 12));
    let bad = b.binary("<", t, f, sp(0, 12));

    let checker = TypeCheckerV1::new();
    let err = checker
        .type_value(&bad, &TypeEnvV1::new())
        .expect_err("bool < bool must reject");

    assert_eq!(err.code(), "types.bad_overload");
    assert_eq!(
        err.to_string(),
        "types.bad_overload: operator `<` has no overload for (bool, bool)"
    );
}

#[test]
fn explicit_context_mismatch_rejects_mismatch() {
    // A list of mixed element types triggers a type mismatch.
    let b = HirBuilderV1::new();
    let i = b.int("1", sp(1, 2));
    let s = b.string("a", sp(4, 7));
    let list = b.list(vec![i, s], sp(0, 8));

    let checker = TypeCheckerV1::new();
    let err = checker
        .type_value(&list, &TypeEnvV1::new())
        .expect_err("heterogeneous list must reject");

    assert_eq!(err.code(), "types.mismatch");
    assert_eq!(
        err.to_string(),
        "types.mismatch: expected type `int`, found `string`"
    );
}

#[test]
fn unknown_symbol_rejects() {
    let b = HirBuilderV1::new();
    let sym = b.symbol("ghost", sp(0, 5));
    let checker = TypeCheckerV1::new();
    let err = checker
        .type_value(&sym, &TypeEnvV1::new())
        .expect_err("unbound symbol must reject");

    assert_eq!(err.code(), "types.unknown_symbol");
    assert_eq!(
        err.to_string(),
        "types.unknown_symbol: unknown symbol `ghost` in type environment"
    );
}

#[test]
fn unknown_binary_operator_rejects() {
    let b = HirBuilderV1::new();
    let a = b.int("1", sp(0, 1));
    let c = b.int("2", sp(4, 5));
    let bad = b.binary("<=>", a, c, sp(0, 5));
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&bad, &TypeEnvV1::new()).unwrap_err();

    assert_eq!(err.code(), "types.unknown_binary_operator");
    assert_eq!(
        err.to_string(),
        "types.unknown_binary_operator: unknown binary operator `<=>`"
    );
}

#[test]
fn unknown_builtin_rejects() {
    let b = HirBuilderV1::new();
    let arg = b.int("1", sp(7, 8));
    let call = b.call("frobnicate", sp(0, 6), vec![arg], sp(0, 9));
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&call, &TypeEnvV1::new()).unwrap_err();

    assert_eq!(err.code(), "types.unknown_builtin");
    assert_eq!(
        err.to_string(),
        "types.unknown_builtin: unknown builtin function `frobnicate`"
    );
}

#[test]
fn builtin_arity_mismatch_rejects() {
    // starts_with expects 2 args; supply 1.
    let b = HirBuilderV1::new();
    let s = b.string("src/", sp(12, 18));
    let call = b.call("starts_with", sp(0, 11), vec![s], sp(0, 19));
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&call, &TypeEnvV1::new()).unwrap_err();

    assert_eq!(err.code(), "types.arity_mismatch");
    assert_eq!(
        err.to_string(),
        "types.arity_mismatch: builtin `starts_with` expects 2 argument(s), found 1"
    );
}

#[test]
fn aggregate_on_non_aggregable_argument_rejects_illegal_aggregate() {
    // sum over a string is not aggregable: there is no string overload of sum.
    let b = HirBuilderV1::new();
    let s = b.symbol("name", sp(4, 8));
    let call = b.call("sum", sp(0, 3), vec![s], sp(0, 9));
    let env = TypeEnvV1::new().with("name", TypeV1::string());
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&call, &env).unwrap_err();

    assert_eq!(err.code(), "types.illegal_aggregate");
    match &err {
        TypeErrorV1::IllegalAggregate {
            function, reason, ..
        } => {
            assert_eq!(function, "sum");
            assert_eq!(
                reason,
                &IllegalAggregateReasonV1::NonAggregableArgument {
                    argument: TypeV1::string()
                }
            );
        }
        other => panic!("expected IllegalAggregate, got {other:?}"),
    }
    assert_eq!(
        err.to_string(),
        "types.illegal_aggregate: aggregate `sum` cannot aggregate argument of type `string`"
    );
}

#[test]
fn aggregate_bad_arity_rejects_illegal_aggregate() {
    // sum expects exactly 1 argument; supply 0.
    let b = HirBuilderV1::new();
    let call = b.call("sum", sp(0, 3), vec![], sp(0, 5));
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&call, &TypeEnvV1::new()).unwrap_err();

    assert_eq!(err.code(), "types.illegal_aggregate");
    assert_eq!(
        err.to_string(),
        "types.illegal_aggregate: aggregate `sum` expects 1 argument(s), found 0"
    );
}

#[test]
fn bad_path_projection_rejects() {
    // u.missing where u is a record without `missing`.
    use quanta_gqlang_v2_hir::PathSegmentV1;
    use quanta_gqlang_v2_types::RecordFieldTyV1;
    let b = HirBuilderV1::new();
    let base = b.symbol("u", sp(0, 1));
    let access = b.path(
        base,
        vec![PathSegmentV1 {
            name: "missing".to_owned(),
            span: sp(2, 9),
        }],
        sp(0, 9),
    );
    let env = TypeEnvV1::new().with(
        "u",
        TypeV1::record(vec![RecordFieldTyV1::new("id", TypeV1::int())]),
    );
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&access, &env).unwrap_err();

    assert_eq!(err.code(), "types.bad_projection");
    assert_eq!(
        err.to_string(),
        "types.bad_projection: type `{ id: int }` has no field `missing`"
    );
}

#[test]
fn malformed_int_literal_rejects() {
    let b = HirBuilderV1::new();
    let bad = b.int("12x", sp(0, 3));
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&bad, &TypeEnvV1::new()).unwrap_err();

    assert_eq!(err.code(), "types.malformed_literal");
    assert_eq!(
        err.to_string(),
        "types.malformed_literal: malformed `int` literal `12x`"
    );
}

#[test]
fn unsupported_proof_arithmetic_rejects_bad_overload() {
    // proof_tree + proof_tree has no arithmetic overload.
    let b = HirBuilderV1::new();
    let p1 = b.symbol("p", sp(0, 1));
    let p2 = b.symbol("q", sp(4, 5));
    let bad = b.binary("+", p1, p2, sp(0, 5));
    let env = TypeEnvV1::new()
        .with("p", TypeV1::scalar(ScalarTyV1::ProofTree))
        .with("q", TypeV1::scalar(ScalarTyV1::ProofTree));
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&bad, &env).unwrap_err();

    assert_eq!(err.code(), "types.bad_overload");
    assert_eq!(
        err.to_string(),
        "types.bad_overload: operator `+` has no overload for (proof_tree, proof_tree)"
    );
}

#[test]
fn repo_path_exact_equality_is_typed_bool_v1() {
    let b = HirBuilderV1::new();
    let left = b.symbol("left", sp(0, 4));
    let right = b.symbol("right", sp(7, 12));
    let equality = b.binary("=", left, right, sp(0, 12));
    let env = TypeEnvV1::new()
        .with("left", TypeV1::scalar(ScalarTyV1::RepoPath))
        .with("right", TypeV1::scalar(ScalarTyV1::RepoPath));

    let actual = TypeCheckerV1::new()
        .type_value(&equality, &env)
        .expect("exact repo_path equality must be supported");
    assert_eq!(actual, TypeV1::bool());
}

#[test]
fn repo_path_does_not_coerce_to_broad_path_v1() {
    let b = HirBuilderV1::new();
    let repo = b.symbol("repo", sp(0, 4));
    let broad = b.symbol("broad", sp(7, 12));
    let equality = b.binary("=", repo, broad, sp(0, 12));
    let env = TypeEnvV1::new()
        .with("repo", TypeV1::scalar(ScalarTyV1::RepoPath))
        .with("broad", TypeV1::scalar(ScalarTyV1::Path));

    let error = TypeCheckerV1::new()
        .type_value(&equality, &env)
        .expect_err("repo_path and path must remain distinct scalar identities");
    assert_eq!(error.code(), "types.bad_overload");
    assert_eq!(
        error.to_string(),
        "types.bad_overload: operator `=` has no overload for (repo_path, path)"
    );
    assert_eq!(error.span(), sp(0, 12));
}
