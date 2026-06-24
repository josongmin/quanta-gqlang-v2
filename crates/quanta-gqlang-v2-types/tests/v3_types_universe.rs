//! Hellgate (positive) rails for the v3 type universe and registry (ticket 03).
//!
//! syntax -> sema(types) path pinned: a built `ValueExprHirV1` tree (ticket 01
//! HIR) is typed by `TypeCheckerV1` against a caller-provided `TypeEnvV1` into a
//! closed `TypeV1`. No binder (ticket 02), no algebra (ticket 04).
//!
//! Covers ticket scenarios:
//!   1. positive: `a + 1` with `a: int` -> int
//!   4. nested record `{ id: int, tags: list<string> }` types correctly
//!   5. path/proof payload value types under the registry
//!   6. an aggregate builtin reports aggregation class + pure/deterministic flag

#![allow(clippy::unwrap_used, reason = "tests assert on Result/Option directly")]
#![allow(clippy::expect_used, reason = "tests assert on Result/Option directly")]
#![allow(clippy::indexing_slicing, reason = "tests index known-shape fixtures")]

use quanta_gqlang_v2_hir::{HirBuilderV1, PathSegmentV1, RecordFieldV1, SpanV1};
use quanta_gqlang_v2_types::{
    AggregationClassV1, CardinalityV1, RecordFieldTyV1, ScalarTyV1, TypeCheckerV1, TypeEnvV1,
    TypeV1,
};

fn sp(start: usize, end: usize) -> SpanV1 {
    SpanV1::new(start, end)
}

#[test]
fn scenario_1_int_addition_types_to_int() {
    // a + 1  with a: int  => int
    let b = HirBuilderV1::new();
    let a = b.symbol("a", sp(0, 1));
    let one = b.int("1", sp(4, 5));
    let sum = b.binary("+", a, one, sp(0, 5));

    let env = TypeEnvV1::new().with("a", TypeV1::int());
    let checker = TypeCheckerV1::new();

    let ty = checker.type_value(&sum, &env).expect("should type");
    assert_eq!(ty, TypeV1::int(), "a + 1 must be int");
}

#[test]
fn float_addition_types_to_float() {
    let b = HirBuilderV1::new();
    let x = b.symbol("x", sp(0, 1));
    let y = b.float("1.5", sp(4, 7));
    let sum = b.binary("+", x, y, sp(0, 7));

    let env = TypeEnvV1::new().with("x", TypeV1::float());
    let checker = TypeCheckerV1::new();

    assert_eq!(checker.type_value(&sum, &env).unwrap(), TypeV1::float());
}

#[test]
fn comparison_types_to_bool() {
    let b = HirBuilderV1::new();
    let a = b.symbol("a", sp(0, 1));
    let one = b.int("1", sp(4, 5));
    let lt = b.binary("<", a, one, sp(0, 5));

    let env = TypeEnvV1::new().with("a", TypeV1::int());
    let checker = TypeCheckerV1::new();

    assert_eq!(checker.type_value(&lt, &env).unwrap(), TypeV1::bool());
}

#[test]
fn scenario_4_nested_record_with_list_field_types() {
    // { id: 1, tags: ["a", "b"] }  =>  { id: int, tags: list<string> }
    let b = HirBuilderV1::new();

    let id_val = b.int("1", sp(8, 9));
    let tag_a = b.string("a", sp(18, 21));
    let tag_b = b.string("b", sp(23, 26));
    let tags_list = b.list(vec![tag_a, tag_b], sp(17, 27));

    let record = b.record(
        vec![
            RecordFieldV1 {
                name: "id".to_owned(),
                name_span: sp(2, 4),
                value: id_val,
            },
            RecordFieldV1 {
                name: "tags".to_owned(),
                name_span: sp(11, 15),
                value: tags_list,
            },
        ],
        sp(0, 29),
    );

    let checker = TypeCheckerV1::new();
    let ty = checker
        .type_value(&record, &TypeEnvV1::new())
        .expect("record should type");

    let expected = TypeV1::record(vec![
        RecordFieldTyV1::new("id", TypeV1::int()),
        RecordFieldTyV1::new("tags", TypeV1::list(TypeV1::string())),
    ]);
    assert_eq!(ty, expected);
    // Field typing is exact + ordered.
    assert_eq!(format!("{ty}"), "{ id: int, tags: list<string> }");
}

#[test]
fn scenario_5_path_and_proof_payloads_type_under_registry() {
    // path_length(p)         => int      (p: path)
    // path_locations(p)      => list<location>
    // proof_size(explain(b)) => int      (b: bool)
    let checker = TypeCheckerV1::new();

    let b = HirBuilderV1::new();
    let p = b.symbol("p", sp(12, 13));
    let len_call = b.call("path_length", sp(0, 11), vec![p], sp(0, 14));
    let env = TypeEnvV1::new().with("p", TypeV1::scalar(ScalarTyV1::Path));
    assert_eq!(checker.type_value(&len_call, &env).unwrap(), TypeV1::int());

    let b2 = HirBuilderV1::new();
    let p2 = b2.symbol("p", sp(15, 16));
    let locs = b2.call("path_locations", sp(0, 14), vec![p2], sp(0, 17));
    assert_eq!(
        checker.type_value(&locs, &env).unwrap(),
        TypeV1::collection(
            TypeV1::scalar(ScalarTyV1::Location),
            CardinalityV1::Sequence
        )
    );

    // proof_tree value family: explain(bool) -> proof_tree; proof_size(...) -> int
    let b3 = HirBuilderV1::new();
    let flag = b3.symbol("flag", sp(20, 24));
    let explained = b3.call("explain", sp(11, 18), vec![flag], sp(11, 25));
    let proof_size = b3.call("proof_size", sp(0, 10), vec![explained], sp(0, 26));
    let env3 = TypeEnvV1::new().with("flag", TypeV1::bool());
    assert_eq!(
        checker.type_value(&proof_size, &env3).unwrap(),
        TypeV1::int()
    );
}

#[test]
fn path_access_projects_record_field_type() {
    // u.id  where u: { id: int, name: string }  => int
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
        TypeV1::record(vec![
            RecordFieldTyV1::new("id", TypeV1::int()),
            RecordFieldTyV1::new("name", TypeV1::string()),
        ]),
    );
    let checker = TypeCheckerV1::new();
    assert_eq!(checker.type_value(&access, &env).unwrap(), TypeV1::int());
}

#[test]
fn scenario_6_aggregate_metadata_reports_class_and_flags() {
    let checker = TypeCheckerV1::new();
    let entry = checker
        .builtins()
        .lookup("sum")
        .expect("sum is a registered aggregate");

    assert_eq!(entry.meta.aggregation, AggregationClassV1::Aggregate);
    assert!(entry.meta.is_aggregate());
    assert!(
        entry.meta.is_pure_deterministic(),
        "sum is pure + deterministic"
    );

    // count is an aggregate over any single argument and yields int.
    let b = HirBuilderV1::new();
    let metric = b.symbol("metric", sp(6, 12));
    let count = b.call("count", sp(0, 5), vec![metric], sp(0, 13));
    let env = TypeEnvV1::new().with("metric", TypeV1::int());
    assert_eq!(checker.type_value(&count, &env).unwrap(), TypeV1::int());

    // A non-deterministic builtin is visible as such to downstream planning.
    let now = checker.builtins().lookup("now").expect("now exists");
    assert!(!now.meta.is_pure_deterministic());
}

#[test]
fn sum_overload_picks_float_for_float_argument() {
    let checker = TypeCheckerV1::new();
    let b = HirBuilderV1::new();
    let m = b.symbol("m", sp(4, 5));
    let call = b.call("sum", sp(0, 3), vec![m], sp(0, 6));
    let env = TypeEnvV1::new().with("m", TypeV1::float());
    assert_eq!(checker.type_value(&call, &env).unwrap(), TypeV1::float());
}

#[test]
fn null_is_assignable_into_optional() {
    assert!(TypeV1::Null.is_assignable_to(&TypeV1::optional(TypeV1::int())));
    assert!(TypeV1::int().is_assignable_to(&TypeV1::optional(TypeV1::int())));
    assert!(!TypeV1::int().is_assignable_to(&TypeV1::float()));
}
