#![expect(
    clippy::expect_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Capability derivation rails (ticket 07, decision `D6`): every checked algebra
//! operator family maps to its host-agnostic capability over the ONE budgeted
//! traversal, and every [`CapabilityV2`] renders a stable keyword.
//!
//! V2 cut: `derive_algebra_capabilities` (the V1 recursive capability walk) is
//! DELETED. A capability is now collected only for a checked algebra node the
//! traversal actually charged a semantic step for, so every rail below drives the
//! real entry point (`plan`) over a sealed ticket-05 lowering product and asserts
//! the plan's EXACT capability set — never a caller-built algebra node.
//!
//! Root-reanchored Semi/Anti joins are driven directly, so both
//! `ExistenceCheck` and `NegationCheck` are owner-local behavioral proofs.

use std::collections::BTreeSet;

use quanta_gqlang_v2_planning::{plan, CapabilityV2, SemanticPlanV2};
use quanta_gqlang_v2_sema::test_support::{ReferenceRelationFixtureV2, ReferenceRuleFixtureV2};

mod support;

use support::{planned_relation, relational_policy};

/// The visited checked algebra node count of one relational fixture's plan,
/// asserting the arm really carries the relational policy.
fn nodes(fixture: ReferenceRelationFixtureV2) -> (u32, SemanticPlanV2) {
    let (_document, planned) = planned_relation(fixture).expect("the fixture plans");
    let (nodes, _effects) =
        relational_policy(&planned).expect("a relational carrier plans a relational policy");
    (nodes, planned)
}

/// A bare scan derives exactly the relation-scan capability.
#[test]
fn scan_derives_relation_scan_only() {
    let (nodes, planned) = nodes(ReferenceRelationFixtureV2::Scan);
    assert_eq!(nodes, 1);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([CapabilityV2::RelationScan])
    );
}

/// Project recurses into its input but adds no capability of its own.
#[test]
fn project_recurses_without_adding_capability() {
    let (nodes, planned) = nodes(ReferenceRelationFixtureV2::ProjectMultipleExpressions);
    assert_eq!(nodes, 2);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([CapabilityV2::RelationScan])
    );
    assert_eq!(
        planned.explain().families(),
        &[
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Project,
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Scan,
        ]
    );
}

/// Filter adds `Filtering` and still reaches the underlying scan.
#[test]
fn filter_derives_filtering_and_scan() {
    let (nodes, planned) = nodes(ReferenceRelationFixtureV2::Filter);
    assert_eq!(nodes, 2);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([CapabilityV2::RelationScan, CapabilityV2::Filtering])
    );
}

/// Join adds `Join` and walks BOTH arms (each a distinct scan).
#[test]
fn join_derives_join_and_both_arms() {
    let (nodes, planned) = nodes(ReferenceRelationFixtureV2::CrossJoin);
    assert_eq!(nodes, 3);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([CapabilityV2::RelationScan, CapabilityV2::Join])
    );
    assert_eq!(
        planned.explain().families(),
        &[
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Join,
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Scan,
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Scan,
        ]
    );
}

/// An ON-join over two distinct host scans adds the same capability as the
/// predicate-free cross join, and folds its non-monotone predicate effect.
#[test]
fn an_on_join_derives_join_and_folds_its_effect() {
    let (nodes, planned) = nodes(ReferenceRelationFixtureV2::InnerJoinOn);
    assert_eq!(nodes, 3);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([CapabilityV2::RelationScan, CapabilityV2::Join])
    );
    let (_nodes, effects) = relational_policy(&planned).expect("relational policy");
    assert_eq!(
        effects.monotonicity,
        quanta_gqlang_v2_types::MonotonicityV1::NonMonotone
    );
}

/// Group adds `Grouping`.
#[test]
fn group_derives_grouping() {
    let (nodes, planned) = nodes(ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys);
    assert_eq!(nodes, 3);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([
            CapabilityV2::RelationScan,
            CapabilityV2::Grouping,
            CapabilityV2::Aggregation,
        ])
    );
}

/// Aggregate adds `Aggregation`; over a Group it also surfaces `Grouping`, even
/// when the group partition is the producer-sealed implicit EMPTY partition of a
/// scalar aggregate.
#[test]
fn aggregate_derives_aggregation_and_grouping() {
    let (nodes, planned) = nodes(ReferenceRelationFixtureV2::ScalarCountStar);
    assert_eq!(nodes, 3);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([
            CapabilityV2::RelationScan,
            CapabilityV2::Grouping,
            CapabilityV2::Aggregation,
        ])
    );
}

/// Sort adds `Ordering`.
#[test]
fn sort_derives_ordering() {
    let (nodes, planned) = nodes(ReferenceRelationFixtureV2::SortAsc);
    assert_eq!(nodes, 2);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([CapabilityV2::RelationScan, CapabilityV2::Ordering])
    );
}

/// Every arm of a self join is visited exactly once, so a five-node shape still
/// derives only the capabilities it actually requires.
#[test]
fn self_join_walks_every_arm_once() {
    let (nodes, planned) = nodes(ReferenceRelationFixtureV2::SelfJoinInner);
    assert_eq!(nodes, 5);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([CapabilityV2::RelationScan, CapabilityV2::Join])
    );
}

/// A relational `EXISTS` conjunct under a residual filter reaches the Semi/Anti
/// join family, so `ExistenceCheck` is derived alongside the residual
/// `Filtering` capability.
#[test]
fn semijoin_derives_existence_check_under_a_filter_root() {
    let (nodes, planned) = nodes(ReferenceRelationFixtureV2::ExistsWithBoolConjunct);
    assert_eq!(nodes, 4);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([
            CapabilityV2::RelationScan,
            CapabilityV2::Filtering,
            CapabilityV2::ExistenceCheck,
        ])
    );
    assert_eq!(
        planned.explain().families(),
        &[
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Filter,
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::SemiJoin,
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Scan,
            quanta_gqlang_v2_algebra::AlgebraFamilyV2::Scan,
        ]
    );
}

/// Pure EXISTS/NOT EXISTS roots and their correlated/nested forms derive the
/// exact capability set from the promoted Semi/Anti join; correlated/nested
/// fixtures additionally retain the real nested Filter they lower.
#[test]
fn every_reanchored_exists_family_derives_its_exact_capabilities() {
    for (fixture, expected) in [
        (
            ReferenceRelationFixtureV2::Exists,
            BTreeSet::from([CapabilityV2::RelationScan, CapabilityV2::ExistenceCheck]),
        ),
        (
            ReferenceRelationFixtureV2::NotExists,
            BTreeSet::from([CapabilityV2::RelationScan, CapabilityV2::NegationCheck]),
        ),
        (
            ReferenceRelationFixtureV2::CorrelatedExists,
            BTreeSet::from([
                CapabilityV2::RelationScan,
                CapabilityV2::Filtering,
                CapabilityV2::ExistenceCheck,
            ]),
        ),
        (
            ReferenceRelationFixtureV2::CorrelatedNotExists,
            BTreeSet::from([
                CapabilityV2::RelationScan,
                CapabilityV2::Filtering,
                CapabilityV2::NegationCheck,
            ]),
        ),
        (
            ReferenceRelationFixtureV2::NestedExistsTwoLevels,
            BTreeSet::from([
                CapabilityV2::RelationScan,
                CapabilityV2::Filtering,
                CapabilityV2::ExistenceCheck,
            ]),
        ),
        (
            ReferenceRelationFixtureV2::ExistsWithMultiplicativeRight,
            BTreeSet::from([
                CapabilityV2::RelationScan,
                CapabilityV2::Join,
                CapabilityV2::ExistenceCheck,
            ]),
        ),
        (
            ReferenceRelationFixtureV2::ConjunctiveExistsPair,
            BTreeSet::from([CapabilityV2::RelationScan, CapabilityV2::ExistenceCheck]),
        ),
    ] {
        let (_nodes, planned) = nodes(fixture);
        assert_eq!(planned.capabilities(), &expected, "{fixture:?}");
    }
}

/// Every `CapabilityV2` variant renders its stable keyword verbatim.
#[test]
fn capability_keywords_are_stable_verbatim() {
    assert_eq!(CapabilityV2::RelationScan.keyword(), "relation_scan");
    assert_eq!(CapabilityV2::Filtering.keyword(), "filtering");
    assert_eq!(CapabilityV2::Join.keyword(), "join");
    assert_eq!(CapabilityV2::Grouping.keyword(), "grouping");
    assert_eq!(CapabilityV2::Aggregation.keyword(), "aggregation");
    assert_eq!(CapabilityV2::Ordering.keyword(), "ordering");
    assert_eq!(CapabilityV2::ExistenceCheck.keyword(), "existence_check");
    assert_eq!(CapabilityV2::NegationCheck.keyword(), "negation_check");
    assert_eq!(
        CapabilityV2::RecursiveClosure.keyword(),
        "recursive_closure"
    );
    assert_eq!(
        CapabilityV2::WitnessMaterialization.keyword(),
        "witness_materialization"
    );
}

/// The `BTreeSet` ordering is deterministic and follows the `Ord` derive
/// (declaration order), so CLI/LSP/tests see a stable capability sequence.
#[test]
fn capability_set_iterates_in_stable_declaration_order() {
    let (_document, grouped) =
        planned_relation(ReferenceRelationFixtureV2::GroupedAggregateSum).expect("grouped plan");
    let ordered: Vec<CapabilityV2> = grouped.capabilities().iter().copied().collect();
    assert_eq!(
        ordered,
        vec![
            CapabilityV2::RelationScan,
            CapabilityV2::Grouping,
            CapabilityV2::Aggregation,
        ]
    );

    let (_document, joined) =
        planned_relation(ReferenceRelationFixtureV2::CrossJoin).expect("joined plan");
    let ordered: Vec<CapabilityV2> = joined.capabilities().iter().copied().collect();
    assert_eq!(
        ordered,
        vec![CapabilityV2::RelationScan, CapabilityV2::Join]
    );
}

/// A rule program with no recursive member derives NO algebra capability at all:
/// its members are sealed logic facts, not algebra nodes, so the walk never
/// charges one.
#[test]
fn a_nonrecursive_rule_program_derives_no_algebra_capability() {
    let (document, input) =
        support::rule_input(ReferenceRuleFixtureV2::SingleRule).expect("the single-rule fixture");
    let planned = plan(&input, document.document()).expect("a single rule plans");
    assert_eq!(planned.capabilities(), &BTreeSet::new());
}
