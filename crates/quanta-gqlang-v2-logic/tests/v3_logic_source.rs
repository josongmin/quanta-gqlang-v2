#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed indices/variants and panic on unexpected shapes"
)]
//! Source-level exists / not-exists lowering rails (ticket 05).
//!
//! syntax->sema->ir path pinned: source quantifier over normalized algebra
//! inputs -> algebra SemiJoin / AntiJoin (target shape owned by ticket 04, D12).
//! Proves the SOURCE-LEVEL MEANING: `exists` -> semijoin, `not exists` ->
//! antijoin, output columns/cardinality are exactly the outer input's.

use quanta_gqlang_v2_algebra::{normalize_relation, AlgebraOpV1, NormalizeOptionsV1};
use quanta_gqlang_v2_hir::{HirBuilderV1, SpanV1};
use quanta_gqlang_v2_logic::{ExistenceCheckV1, ExistencePolarityV1};

fn sp(a: usize, b: usize) -> SpanV1 {
    SpanV1::new(a, b)
}

fn scan(name: &str) -> quanta_gqlang_v2_algebra::AlgebraNodeV1 {
    let b = HirBuilderV1::new();
    let rel = b.relation_source(name, None, sp(0, 5));
    normalize_relation(&rel, &NormalizeOptionsV1::default()).expect("scan normalizes")
}

#[test]
fn exists_lowers_to_semijoin_preserving_outer_schema() {
    let outer = scan("api");
    let inner = scan("auth");
    let outer_columns = outer.columns.clone();
    let outer_card = outer.cardinality;

    let check = ExistenceCheckV1::from_keyword("exists", outer, inner, None, sp(10, 30))
        .expect("exists is a known quantifier");
    assert_eq!(check.polarity, ExistencePolarityV1::Exists);

    let node = check.lower();
    assert_eq!(node.columns, outer_columns);
    assert_eq!(node.cardinality, outer_card);
    match node.op {
        AlgebraOpV1::SemiJoin(_) => {}
        other => panic!("exists must lower to SemiJoin, got {other:?}"),
    }
}

#[test]
fn not_exists_lowers_to_antijoin() {
    let outer = scan("api");
    let inner = scan("auth");

    let check = ExistenceCheckV1::from_keyword("not exists", outer, inner, None, sp(10, 30))
        .expect("not exists is a known quantifier");
    assert_eq!(check.polarity, ExistencePolarityV1::NotExists);

    let node = check.lower();
    match node.op {
        AlgebraOpV1::AntiJoin(_) => {}
        other => panic!("not exists must lower to AntiJoin, got {other:?}"),
    }
}

#[test]
fn not_exists_underscore_keyword_also_recognized() {
    let outer = scan("api");
    let inner = scan("auth");
    let check = ExistenceCheckV1::from_keyword("not_exists", outer, inner, None, sp(0, 4))
        .expect("not_exists is recognized");
    assert_eq!(check.polarity, ExistencePolarityV1::NotExists);
}

#[test]
fn unknown_quantifier_rejects_with_typed_code() {
    let outer = scan("api");
    let inner = scan("auth");
    let err = ExistenceCheckV1::from_keyword("forsome", outer, inner, None, sp(2, 9))
        .expect_err("unknown quantifier rejects");
    assert_eq!(err.code(), "logic.unknown_quantifier");
    assert_eq!(
        err.message(),
        "unknown source quantifier `forsome`; expected `exists` or `not exists`"
    );
    assert_eq!(err.span(), sp(2, 9));
}
