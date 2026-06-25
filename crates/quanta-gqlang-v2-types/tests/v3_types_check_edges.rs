//! Edge / reject path rails for the value-expression type checker (ticket 03).
//!
//! These drive the uncovered branches of `check.rs` — unary resolution
//! (success / bad-overload / unknown-operator), parenthesized expressions,
//! malformed float literals, the free `scalar` helper, optional/non-record path
//! projection, and the variadic builtin-matching path — asserting the exact
//! typed diagnostic code/message or the exact result type, never a smoke check.

#![allow(clippy::unwrap_used, reason = "tests assert on Result/Option directly")]
#![allow(clippy::expect_used, reason = "tests assert on Result/Option directly")]
#![allow(clippy::panic, reason = "tests panic on unexpected error shapes")]
#![allow(
    clippy::wildcard_enum_match_arm,
    reason = "tests match the one expected variant and panic otherwise"
)]

use quanta_gqlang_v2_hir::{HirBuilderV1, PathSegmentV1, SpanV1};
use quanta_gqlang_v2_types::{
    scalar, CardinalityV1, RecordFieldTyV1, ScalarTyV1, TypeCheckerV1, TypeEnvV1, TypeErrorV1,
    TypeV1,
};

fn sp(start: usize, end: usize) -> SpanV1 {
    SpanV1::new(start, end)
}

#[test]
fn unary_negation_types_to_operand_type() {
    // -x with x: int  => int  (drives resolve_unary success path).
    let b = HirBuilderV1::new();
    let x = b.symbol("x", sp(1, 2));
    let neg = b.unary("-", x, sp(0, 2));
    let env = TypeEnvV1::new().with("x", TypeV1::int());
    let checker = TypeCheckerV1::new();
    assert_eq!(checker.type_value(&neg, &env).unwrap(), TypeV1::int());

    // not b with b: bool => bool
    let b2 = HirBuilderV1::new();
    let flag = b2.symbol("flag", sp(4, 8));
    let notted = b2.unary("not", flag, sp(0, 8));
    let env2 = TypeEnvV1::new().with("flag", TypeV1::bool());
    assert_eq!(checker.type_value(&notted, &env2).unwrap(), TypeV1::bool());
}

#[test]
fn unary_on_wrong_operand_rejects_bad_overload() {
    // not s  with s: string  => bad overload on a KNOWN unary operator.
    let b = HirBuilderV1::new();
    let s = b.symbol("s", sp(4, 5));
    let notted = b.unary("not", s, sp(0, 5));
    let env = TypeEnvV1::new().with("s", TypeV1::string());
    let checker = TypeCheckerV1::new();

    let err = checker.type_value(&notted, &env).unwrap_err();
    assert_eq!(err.code(), "types.bad_overload");
    assert_eq!(err.span(), sp(0, 5));
    match err {
        TypeErrorV1::BadOverload {
            operator, operands, ..
        } => {
            assert_eq!(operator, "not");
            assert_eq!(operands, vec![TypeV1::string()]);
        }
        other => panic!("expected BadOverload, got {other:?}"),
    }
    // Message renders the operand type.
    assert_eq!(
        format!(
            "{}",
            checker
                .type_value(&b.unary("not", b.symbol("s", sp(4, 5)), sp(0, 5)), &env)
                .unwrap_err()
        ),
        "types.bad_overload: operator `not` has no overload for (string)"
    );
}

#[test]
fn unknown_unary_operator_rejects() {
    // `~x` — `~` is not in the closed unary registry.
    let b = HirBuilderV1::new();
    let x = b.symbol("x", sp(1, 2));
    let bad = b.unary("~", x, sp(0, 2));
    let env = TypeEnvV1::new().with("x", TypeV1::int());
    let checker = TypeCheckerV1::new();

    let err = checker.type_value(&bad, &env).unwrap_err();
    assert_eq!(err.code(), "types.unknown_unary_operator");
    assert_eq!(err.span(), sp(0, 2));
    match err {
        TypeErrorV1::UnknownUnaryOperator { operator, .. } => assert_eq!(operator, "~"),
        other => panic!("expected UnknownUnaryOperator, got {other:?}"),
    }
}

#[test]
fn parenthesized_expression_types_to_inner_type() {
    // (1 + 2) => int  (drives the Paren arm, which delegates to the inner expr).
    let b = HirBuilderV1::new();
    let one = b.int("1", sp(1, 2));
    let two = b.int("2", sp(5, 6));
    let sum = b.binary("+", one, two, sp(1, 6));
    let paren = b.paren_value(sum, sp(0, 7));
    let checker = TypeCheckerV1::new();
    assert_eq!(
        checker.type_value(&paren, &TypeEnvV1::new()).unwrap(),
        TypeV1::int()
    );
}

#[test]
fn malformed_float_literal_rejects() {
    // A float literal whose text does not parse as f64 rejects with the float
    // expected type (drives the LiteralV1::Float error arm).
    let b = HirBuilderV1::new();
    let bad = b.float("1.2.3", sp(0, 5));
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&bad, &TypeEnvV1::new()).unwrap_err();
    assert_eq!(err.code(), "types.malformed_literal");
    match err {
        TypeErrorV1::MalformedLiteral { expected, text, .. } => {
            assert_eq!(expected, TypeV1::float());
            assert_eq!(text, "1.2.3");
        }
        other => panic!("expected MalformedLiteral, got {other:?}"),
    }
}

#[test]
fn null_and_bool_literals_type_directly() {
    // Drives the Null and Bool literal arms.
    let b = HirBuilderV1::new();
    let checker = TypeCheckerV1::new();
    let null = b.null(sp(0, 4));
    assert_eq!(
        checker.type_value(&null, &TypeEnvV1::new()).unwrap(),
        TypeV1::Null
    );
    let t = b.bool_lit(true, sp(0, 4));
    assert_eq!(
        checker.type_value(&t, &TypeEnvV1::new()).unwrap(),
        TypeV1::bool()
    );
}

#[test]
fn scalar_free_helper_matches_typev1_scalar() {
    // The re-exported `scalar` free fn must equal the inherent constructor.
    assert_eq!(scalar(ScalarTyV1::Path), TypeV1::scalar(ScalarTyV1::Path));
    assert_eq!(scalar(ScalarTyV1::Int), TypeV1::int());
}

#[test]
fn checker_exposes_operator_and_builtin_registries() {
    // Drives the `operators()` / `builtins()` accessors.
    let checker = TypeCheckerV1::new();
    assert!(checker.operators().has_binary("+"));
    assert!(checker.operators().has_unary("not"));
    assert!(checker.builtins().has("count"));
    assert!(!checker.builtins().has("frobnicate"));
}

#[test]
fn path_projection_through_optional_record_wraps_result_in_optional() {
    // u.id  where u: optional<{ id: int }>  => optional<int>
    // Drives project_field's optional-record branch (wrap_optional = true).
    let b = HirBuilderV1::new();
    let base = b.symbol("u", sp(0, 1));
    let access = b.path(
        base,
        vec![PathSegmentV1 {
            name: "id".to_owned(),
            span: sp(2, 4),
        }],
        sp(0, 4),
    );
    let env = TypeEnvV1::new().with(
        "u",
        TypeV1::optional(TypeV1::record(vec![RecordFieldTyV1::new(
            "id",
            TypeV1::int(),
        )])),
    );
    let checker = TypeCheckerV1::new();
    assert_eq!(
        checker.type_value(&access, &env).unwrap(),
        TypeV1::optional(TypeV1::int())
    );
}

#[test]
fn path_projection_on_scalar_base_rejects_bad_projection() {
    // i.field where i: int  => bad projection on a scalar base.
    let b = HirBuilderV1::new();
    let base = b.symbol("i", sp(0, 1));
    let access = b.path(
        base,
        vec![PathSegmentV1 {
            name: "field".to_owned(),
            span: sp(2, 7),
        }],
        sp(0, 7),
    );
    let env = TypeEnvV1::new().with("i", TypeV1::int());
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&access, &env).unwrap_err();
    assert_eq!(err.code(), "types.bad_projection");
    assert_eq!(err.span(), sp(2, 7));
    match err {
        TypeErrorV1::BadProjection { base, segment, .. } => {
            assert_eq!(base, TypeV1::int());
            assert_eq!(segment, "field");
        }
        other => panic!("expected BadProjection, got {other:?}"),
    }
}

#[test]
fn path_projection_through_optional_non_record_rejects() {
    // o.field where o: optional<int>  => bad projection (optional inner is not a
    // record). Drives the optional-but-not-record branch.
    let b = HirBuilderV1::new();
    let base = b.symbol("o", sp(0, 1));
    let access = b.path(
        base,
        vec![PathSegmentV1 {
            name: "field".to_owned(),
            span: sp(2, 7),
        }],
        sp(0, 7),
    );
    let env = TypeEnvV1::new().with("o", TypeV1::optional(TypeV1::int()));
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&access, &env).unwrap_err();
    assert_eq!(err.code(), "types.bad_projection");
    match err {
        TypeErrorV1::BadProjection { segment, .. } => assert_eq!(segment, "field"),
        other => panic!("expected BadProjection, got {other:?}"),
    }
}

#[test]
fn path_projection_unknown_field_on_record_rejects() {
    // r.missing where r: { id: int } => bad projection (no such field).
    let b = HirBuilderV1::new();
    let base = b.symbol("r", sp(0, 1));
    let access = b.path(
        base,
        vec![PathSegmentV1 {
            name: "missing".to_owned(),
            span: sp(2, 9),
        }],
        sp(0, 9),
    );
    let env = TypeEnvV1::new().with(
        "r",
        TypeV1::record(vec![RecordFieldTyV1::new("id", TypeV1::int())]),
    );
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&access, &env).unwrap_err();
    assert_eq!(err.code(), "types.bad_projection");
    match err {
        TypeErrorV1::BadProjection { segment, .. } => assert_eq!(segment, "missing"),
        other => panic!("expected BadProjection, got {other:?}"),
    }
}

#[test]
fn size_builtin_matches_fixed_collection_param() {
    // size(xs) with xs: list<int> => int. Drives match_signature's fixed-param
    // assignability loop with a non-scalar param.
    let b = HirBuilderV1::new();
    let xs = b.symbol("xs", sp(5, 7));
    let call = b.call("size", sp(0, 4), vec![xs], sp(0, 8));
    let env = TypeEnvV1::new().with(
        "xs",
        TypeV1::collection(TypeV1::int(), CardinalityV1::Sequence),
    );
    let checker = TypeCheckerV1::new();
    assert_eq!(checker.type_value(&call, &env).unwrap(), TypeV1::int());
}

#[test]
fn size_builtin_on_wrong_element_type_rejects_bad_overload() {
    // size(xs) with xs: list<string> => the only `size` overload wants list<int>,
    // and `string` is not assignable to `int`, so no overload matches: scalar
    // builtin with matching arity but bad types => bad overload.
    let b = HirBuilderV1::new();
    let xs = b.symbol("xs", sp(5, 7));
    let call = b.call("size", sp(0, 4), vec![xs], sp(0, 8));
    let env = TypeEnvV1::new().with(
        "xs",
        TypeV1::collection(TypeV1::string(), CardinalityV1::Sequence),
    );
    let checker = TypeCheckerV1::new();
    let err = checker.type_value(&call, &env).unwrap_err();
    assert_eq!(err.code(), "types.bad_overload");
    match err {
        TypeErrorV1::BadOverload {
            operator, operands, ..
        } => {
            assert_eq!(operator, "size");
            assert_eq!(
                operands,
                vec![TypeV1::collection(
                    TypeV1::string(),
                    CardinalityV1::Sequence
                )]
            );
        }
        other => panic!("expected BadOverload, got {other:?}"),
    }
}
