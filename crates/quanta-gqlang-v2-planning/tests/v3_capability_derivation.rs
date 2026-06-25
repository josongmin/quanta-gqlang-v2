#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed shapes and panic on unexpected variants"
)]
//! Capability derivation rails (ticket 07, decision `D6`): every algebra operator
//! family maps to its host-agnostic capability, recursively over the tree, and
//! every [`CapabilityV1`] renders a stable keyword. These drive the
//! per-operator arms of `capability::collect` (the worst-covered module) and
//! lock the closed keyword set.

use quanta_gqlang_v2_algebra::{
    AggregateExprV1, AggregateV1, AlgebraNodeV1, AlgebraOpV1, AntiJoinV1, ApplyV1,
    ColumnProvenanceV1, ColumnV1, FilterV1, GroupV1, JoinV1, LimitV1, ProjectV1, ScanV1,
    SemiJoinV1, SortDirectionV1, SortKeyV1, SortV1, UnionV1,
};
use quanta_gqlang_v2_hir::{
    BoolExprHirV1, BoolExprKindV1, HirNodeIdV1, LiteralV1, SpanV1, ValueExprHirV1, ValueExprKindV1,
};
use quanta_gqlang_v2_planning::{derive_algebra_capabilities, CapabilityV1};
use quanta_gqlang_v2_types::CardinalityV1;

fn id(n: u32) -> HirNodeIdV1 {
    HirNodeIdV1::from_raw(n)
}

fn span() -> SpanV1 {
    SpanV1::new(0, 10)
}

fn col(name: &str) -> ColumnV1 {
    ColumnV1::new(
        name,
        ColumnProvenanceV1::Source {
            relation: "rel".to_owned(),
            alias: None,
            field: Some(name.to_owned()),
        },
    )
}

fn lit(n: u32) -> ValueExprHirV1 {
    ValueExprHirV1::new(
        id(n),
        span(),
        ValueExprKindV1::Literal(LiteralV1::Int("1".to_owned())),
    )
}

fn pred() -> Box<BoolExprHirV1> {
    Box::new(BoolExprHirV1::new(
        id(99),
        span(),
        BoolExprKindV1::Literal(true),
    ))
}

fn node(card: CardinalityV1, op: AlgebraOpV1) -> AlgebraNodeV1 {
    AlgebraNodeV1::new(id(1), span(), vec![col("x")], card, op)
}

fn scan(name: &str) -> AlgebraNodeV1 {
    node(
        CardinalityV1::Bag,
        AlgebraOpV1::Scan(ScanV1 {
            relation: name.to_owned(),
            alias: None,
        }),
    )
}

/// A bare scan derives exactly the relation-scan capability.
#[test]
fn scan_derives_relation_scan_only() {
    let caps = derive_algebra_capabilities(&scan("users"));
    assert_eq!(caps.len(), 1);
    assert!(caps.contains(&CapabilityV1::RelationScan));
}

/// Project recurses into its input but adds no capability of its own.
#[test]
fn project_recurses_without_adding_capability() {
    let projected = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Project(ProjectV1 {
            input: Box::new(scan("users")),
            items: vec![lit(2)],
        }),
    );
    let caps = derive_algebra_capabilities(&projected);
    assert_eq!(caps.len(), 1);
    assert!(caps.contains(&CapabilityV1::RelationScan));
}

/// Filter adds `Filtering` and still reaches the underlying scan.
#[test]
fn filter_derives_filtering_and_scan() {
    let filtered = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Filter(FilterV1 {
            input: Box::new(scan("users")),
            predicate: pred(),
        }),
    );
    let caps = derive_algebra_capabilities(&filtered);
    assert!(caps.contains(&CapabilityV1::Filtering));
    assert!(caps.contains(&CapabilityV1::RelationScan));
    assert_eq!(caps.len(), 2);
}

/// Join adds `Join` and walks BOTH arms (each a distinct scan).
#[test]
fn join_derives_join_and_both_arms() {
    let joined = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Join(JoinV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            predicate: Some(pred()),
        }),
    );
    let caps = derive_algebra_capabilities(&joined);
    assert!(caps.contains(&CapabilityV1::Join));
    assert!(caps.contains(&CapabilityV1::RelationScan));
    assert_eq!(caps.len(), 2);
}

/// Apply (correlated/lateral) adds `CorrelatedApply` and walks both sides.
#[test]
fn apply_derives_correlated_apply() {
    let applied = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Apply(ApplyV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            correlation: vec![],
        }),
    );
    let caps = derive_algebra_capabilities(&applied);
    assert!(caps.contains(&CapabilityV1::CorrelatedApply));
    assert!(caps.contains(&CapabilityV1::RelationScan));
}

/// Group adds `Grouping`.
#[test]
fn group_derives_grouping() {
    let grouped = node(
        CardinalityV1::Set,
        AlgebraOpV1::Group(GroupV1 {
            input: Box::new(scan("users")),
            keys: vec![lit(2)],
        }),
    );
    let caps = derive_algebra_capabilities(&grouped);
    assert!(caps.contains(&CapabilityV1::Grouping));
    assert!(caps.contains(&CapabilityV1::RelationScan));
}

/// Aggregate adds `Aggregation`; over a Group it also surfaces `Grouping`.
#[test]
fn aggregate_derives_aggregation_and_grouping() {
    let grouped = node(
        CardinalityV1::Set,
        AlgebraOpV1::Group(GroupV1 {
            input: Box::new(scan("users")),
            keys: vec![lit(2)],
        }),
    );
    let aggregate = node(
        CardinalityV1::Set,
        AlgebraOpV1::Aggregate(AggregateV1 {
            input: Box::new(grouped),
            aggregates: vec![AggregateExprV1 {
                function: "count".to_owned(),
                origin: id(11),
                args: vec![],
                distinct: false,
            }],
        }),
    );
    let caps = derive_algebra_capabilities(&aggregate);
    assert!(caps.contains(&CapabilityV1::Aggregation));
    assert!(caps.contains(&CapabilityV1::Grouping));
    assert!(caps.contains(&CapabilityV1::RelationScan));
}

/// Sort adds `Ordering`.
#[test]
fn sort_derives_ordering() {
    let sorted = node(
        CardinalityV1::Sequence,
        AlgebraOpV1::Sort(SortV1 {
            input: Box::new(scan("users")),
            keys: vec![SortKeyV1 {
                expr: lit(2),
                direction: SortDirectionV1::Ascending,
            }],
        }),
    );
    let caps = derive_algebra_capabilities(&sorted);
    assert!(caps.contains(&CapabilityV1::Ordering));
    assert!(caps.contains(&CapabilityV1::RelationScan));
}

/// Limit adds `Limiting`.
#[test]
fn limit_derives_limiting() {
    let limited = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Limit(LimitV1 {
            input: Box::new(scan("users")),
            count: 5,
            offset: 0,
        }),
    );
    let caps = derive_algebra_capabilities(&limited);
    assert!(caps.contains(&CapabilityV1::Limiting));
    assert!(caps.contains(&CapabilityV1::RelationScan));
}

/// Union adds `SetUnion` and walks both arms.
#[test]
fn union_derives_set_union() {
    let unioned = node(
        CardinalityV1::Set,
        AlgebraOpV1::Union(UnionV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            distinct: true,
        }),
    );
    let caps = derive_algebra_capabilities(&unioned);
    assert!(caps.contains(&CapabilityV1::SetUnion));
    assert!(caps.contains(&CapabilityV1::RelationScan));
}

/// SemiJoin adds `ExistenceCheck` and walks both arms.
#[test]
fn semijoin_derives_existence_check() {
    let semi = node(
        CardinalityV1::Bag,
        AlgebraOpV1::SemiJoin(SemiJoinV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            predicate: Some(pred()),
        }),
    );
    let caps = derive_algebra_capabilities(&semi);
    assert!(caps.contains(&CapabilityV1::ExistenceCheck));
    assert!(caps.contains(&CapabilityV1::RelationScan));
}

/// AntiJoin adds `NegationCheck` and walks both arms.
#[test]
fn antijoin_derives_negation_check() {
    let anti = node(
        CardinalityV1::Bag,
        AlgebraOpV1::AntiJoin(AntiJoinV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            predicate: None,
        }),
    );
    let caps = derive_algebra_capabilities(&anti);
    assert!(caps.contains(&CapabilityV1::NegationCheck));
    assert!(caps.contains(&CapabilityV1::RelationScan));
}

/// A deeply nested mix accumulates one capability per distinct family, ordered
/// deterministically by the `BTreeSet`.
#[test]
fn nested_mix_accumulates_all_present_capabilities() {
    // antijoin( filter(scan a), semijoin(scan b, scan c) )
    let left = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Filter(FilterV1 {
            input: Box::new(scan("a")),
            predicate: pred(),
        }),
    );
    let right = node(
        CardinalityV1::Bag,
        AlgebraOpV1::SemiJoin(SemiJoinV1 {
            left: Box::new(scan("b")),
            right: Box::new(scan("c")),
            predicate: None,
        }),
    );
    let anti = node(
        CardinalityV1::Bag,
        AlgebraOpV1::AntiJoin(AntiJoinV1 {
            left: Box::new(left),
            right: Box::new(right),
            predicate: None,
        }),
    );
    let caps = derive_algebra_capabilities(&anti);
    assert!(caps.contains(&CapabilityV1::NegationCheck));
    assert!(caps.contains(&CapabilityV1::Filtering));
    assert!(caps.contains(&CapabilityV1::ExistenceCheck));
    assert!(caps.contains(&CapabilityV1::RelationScan));
    assert_eq!(caps.len(), 4);
}

/// Every `CapabilityV1` variant renders its stable keyword verbatim.
#[test]
fn capability_keywords_are_stable_verbatim() {
    assert_eq!(CapabilityV1::RelationScan.keyword(), "relation_scan");
    assert_eq!(CapabilityV1::Filtering.keyword(), "filtering");
    assert_eq!(CapabilityV1::Join.keyword(), "join");
    assert_eq!(CapabilityV1::CorrelatedApply.keyword(), "correlated_apply");
    assert_eq!(CapabilityV1::Grouping.keyword(), "grouping");
    assert_eq!(CapabilityV1::Aggregation.keyword(), "aggregation");
    assert_eq!(CapabilityV1::Ordering.keyword(), "ordering");
    assert_eq!(CapabilityV1::Limiting.keyword(), "limiting");
    assert_eq!(CapabilityV1::SetUnion.keyword(), "set_union");
    assert_eq!(CapabilityV1::ExistenceCheck.keyword(), "existence_check");
    assert_eq!(CapabilityV1::NegationCheck.keyword(), "negation_check");
    assert_eq!(
        CapabilityV1::RecursiveClosure.keyword(),
        "recursive_closure"
    );
    assert_eq!(
        CapabilityV1::WitnessMaterialization.keyword(),
        "witness_materialization"
    );
}

/// The `BTreeSet` ordering is deterministic and follows the `Ord` derive
/// (declaration order), so CLI/LSP/tests see a stable capability sequence.
#[test]
fn capability_set_iterates_in_stable_declaration_order() {
    let joined = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Join(JoinV1 {
            left: Box::new(scan("a")),
            right: Box::new(scan("b")),
            predicate: None,
        }),
    );
    let caps = derive_algebra_capabilities(&joined);
    let ordered: Vec<CapabilityV1> = caps.into_iter().collect();
    // RelationScan is declared before Join, so it sorts first.
    assert_eq!(
        ordered,
        vec![CapabilityV1::RelationScan, CapabilityV1::Join]
    );
}
