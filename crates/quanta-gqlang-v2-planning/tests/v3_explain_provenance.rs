#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Logical explain-note provenance rails (ticket 07, decision `D24`).
//!
//! [`ExplainNoteV2`] is the host-agnostic provenance summary: the CHECKED
//! location of the planned item, the ordered, root-first list of checked algebra
//! families the result is built from, and whether the plan carries an admitted
//! recursive closure.
//!
//! V2 cut: the V1 free constructors `ExplainNoteV1::from_root` / `for_item` and
//! their recursive `collect_families` walk are DELETED. A note is now built only
//! from the ONE budgeted traversal's visited family list, so every rail below
//! drives `plan` over a sealed ticket-05 lowering product and asserts the EXACT
//! family sequence and checked anchor — never a note assembled outside the
//! planner.
//!
//! Rails whose family arm needs a fixture that does not exist yet are listed in
//! the lane report as INTEGRATOR-FIXTURE-PENDING and marked inline
//! (`Apply`, `Union`, `Limit` chain arms: `ReferenceRelationFixtureV2` has no such
//! variant).

use quanta_gqlang_v2_algebra::AlgebraFamilyV2;
use quanta_gqlang_v2_planning::{plan, SemanticPlanV2};
use quanta_gqlang_v2_sema::test_support::{ReferenceRelationFixtureV2, ReferenceRuleFixtureV2};

mod support;

use support::{
    check_input, document_item_location, graph_input, planned_relation, relational_policy,
    rule_input,
};

/// The visited family list and charge count of one relational fixture's plan.
fn families(fixture: ReferenceRelationFixtureV2) -> (Vec<AlgebraFamilyV2>, u32, SemanticPlanV2) {
    let (_document, planned) = planned_relation(fixture).expect("the fixture plans");
    let (nodes, _effects) =
        relational_policy(&planned).expect("a relational carrier plans a relational policy");
    (planned.explain().families().to_vec(), nodes, planned)
}

/// A bare scan: a single-family note, anchored to the planned item's own CHECKED
/// location, not recursive.
#[test]
fn a_scan_note_is_single_family_at_the_checked_item_location() {
    let (document, input) =
        support::relation_input(ReferenceRelationFixtureV2::Scan).expect("scan fixture");
    let planned = plan(&input, document.document()).expect("a scan plans");
    assert_eq!(planned.explain().families(), &[AlgebraFamilyV2::Scan]);
    assert!(!planned.explain().recursive());
    assert_eq!(
        Some(planned.explain().root_location()),
        document_item_location(document.document(), document.item_id()).as_ref()
    );
}

/// A unary chain (filter over scan, project over scan) lists families root-first.
#[test]
fn a_unary_chain_lists_families_root_first() {
    let (filtered, nodes, _planned) = families(ReferenceRelationFixtureV2::Filter);
    assert_eq!(nodes, 2);
    assert_eq!(
        filtered,
        vec![AlgebraFamilyV2::Filter, AlgebraFamilyV2::Scan]
    );

    let (projected, nodes, _planned) =
        families(ReferenceRelationFixtureV2::ProjectMultipleExpressions);
    assert_eq!(nodes, 2);
    assert_eq!(
        projected,
        vec![AlgebraFamilyV2::Project, AlgebraFamilyV2::Scan]
    );
}

/// A group + aggregate chain lists the fold before its partition and its scan.
#[test]
fn a_group_aggregate_chain_lists_families_root_first() {
    for fixture in [
        ReferenceRelationFixtureV2::GroupedAggregateSum,
        ReferenceRelationFixtureV2::GroupedAggregateTwoPartitionKeys,
        ReferenceRelationFixtureV2::ScalarCountStar,
    ] {
        let (families, nodes, _planned) = families(fixture);
        assert_eq!(nodes, 3, "{fixture:?} charges three nodes");
        assert_eq!(
            families,
            vec![
                AlgebraFamilyV2::Aggregate,
                AlgebraFamilyV2::Group,
                AlgebraFamilyV2::Scan,
            ],
            "{fixture:?} lists the fold root-first"
        );
    }
}

/// A sort chain lists the sort first, then every stage beneath it — including a
/// projection inserted between the sort and its scan.
#[test]
fn a_sort_chain_lists_families_root_first() {
    let (sorted, nodes, _planned) = families(ReferenceRelationFixtureV2::SortAsc);
    assert_eq!(nodes, 2);
    assert_eq!(sorted, vec![AlgebraFamilyV2::Sort, AlgebraFamilyV2::Scan]);

    let (keyed, nodes, _planned) = families(ReferenceRelationFixtureV2::SortAliasKeyed);
    assert_eq!(nodes, 3);
    assert_eq!(
        keyed,
        vec![
            AlgebraFamilyV2::Sort,
            AlgebraFamilyV2::Project,
            AlgebraFamilyV2::Scan,
        ]
    );
}

/// A join lists the join family then the LEFT subtree then the RIGHT subtree, in
/// pre-order.
#[test]
fn a_join_lists_the_join_then_left_then_right() {
    let (joined, nodes, _planned) = families(ReferenceRelationFixtureV2::CrossJoin);
    assert_eq!(nodes, 3);
    assert_eq!(
        joined,
        vec![
            AlgebraFamilyV2::Join,
            AlgebraFamilyV2::Scan,
            AlgebraFamilyV2::Scan,
        ]
    );
}

/// A self join proves left-before-right nesting: the left projection subtree is
/// fully visited before the right one.
#[test]
fn a_self_join_walks_every_arm_in_preorder() {
    let (families, nodes, _planned) = families(ReferenceRelationFixtureV2::SelfJoinInner);
    assert_eq!(nodes, 5);
    assert_eq!(
        families,
        vec![
            AlgebraFamilyV2::Join,
            AlgebraFamilyV2::Project,
            AlgebraFamilyV2::Scan,
            AlgebraFamilyV2::Project,
            AlgebraFamilyV2::Scan,
        ]
    );
}

/// A relational EXISTS conjunct is lowered into the traversal as a Semi/Anti
/// join, so the note lists the residual filter first and then BOTH semi-join
/// arms.
#[test]
fn a_semijoin_lists_both_arms_under_its_filter_root() {
    let (families, nodes, _planned) = families(ReferenceRelationFixtureV2::ExistsWithBoolConjunct);
    assert_eq!(nodes, 4);
    assert_eq!(
        families,
        vec![
            AlgebraFamilyV2::Filter,
            AlgebraFamilyV2::SemiJoin,
            AlgebraFamilyV2::Scan,
            AlgebraFamilyV2::Scan,
        ]
    );
}

/// A non-algebra arm retains NO family at all (the V1 `for_item` empty-list
/// behaviour) while carrying its recursion flag verbatim: a cyclic FIND and a
/// nested cyclic confirm are recursive, a plain rule program is not.
#[test]
fn a_non_algebra_arm_has_no_families() {
    let (document, input) = graph_input(quanta_gqlang_v2_sema::GraphQueryKindV1::Taint, 4)
        .expect("the reference taint FIND");
    let planned = plan(&input, document.document()).expect("a bounded FIND plans");
    assert!(planned.explain().families().is_empty());
    assert!(planned.explain().recursive());

    let (document, input) = check_input().expect("the nested CHECK fixture");
    let planned = plan(&input, document.document()).expect("the CHECK plans");
    assert!(planned.explain().families().is_empty());
    assert!(planned.explain().recursive());

    let (document, input) =
        rule_input(ReferenceRuleFixtureV2::SingleRule).expect("the single-rule fixture");
    let planned = plan(&input, document.document()).expect("a single rule plans");
    assert!(planned.explain().families().is_empty());
    assert!(!planned.explain().recursive());
}

/// A rule-program note is anchored on the planned program's FIRST member body
/// location, and a recursive program reports `recursive = true` there.
#[test]
fn a_rule_program_note_is_anchored_on_its_first_member() {
    let (document, input) =
        rule_input(ReferenceRuleFixtureV2::PositiveSelfRecursion).expect("recursive program");
    let planned = plan(&input, document.document()).expect("positive recursion admits");
    let first_member = *document.item_ids().first().expect("one member");
    assert!(planned.explain().recursive());
    assert!(planned.explain().families().is_empty());
    assert_eq!(
        Some(planned.explain().root_location()),
        document_item_location(document.document(), first_member).as_ref()
    );
}

/// The note is produced by the ONE budgeted traversal: a plan that could not be
/// walked (refused charge) has no note at all, and a walked plan's family count
/// equals the number of charged nodes exactly.
#[test]
fn families_are_charged_one_step_per_visited_node() {
    let (_document, planned) =
        planned_relation(ReferenceRelationFixtureV2::SelfJoinInner).expect("self join");
    let (nodes, _effects) = relational_policy(&planned).expect("relational policy");
    assert_eq!(
        u32::try_from(planned.explain().families().len()).expect("a small family list"),
        nodes
    );

    // The over-limit chain is refused BEFORE any note is assembled, so the
    // explain note can never describe a tree the budget walk did not visit.
    let carrier = quanta_gqlang_v2_sema::test_support::deep_relation_chain_document_v2(
        quanta_gqlang_v2_sema::test_support::SMALL_BUDGET_MAX_SEMANTIC_STEPS,
        quanta_gqlang_v2_sema::test_support::ReferenceBudgetV2::Standard,
    )
    .expect("the chain authors");
    let input =
        quanta_gqlang_v2_algebra::lower_relation_item(carrier.document(), carrier.item_id())
            .map(quanta_gqlang_v2_planning::SemanticPlanInputV2::Relational)
            .expect("the chain lowers");
    let over =
        quanta_gqlang_v2_sema::test_support::forged_small_receipt_relation_chain_document_v2(
            quanta_gqlang_v2_sema::test_support::SMALL_BUDGET_MAX_SEMANTIC_STEPS,
        )
        .expect("the small-receipt chain");
    match plan(&input, over.document()) {
        Err(quanta_gqlang_v2_planning::PlanningDiagnosticV1::SemanticStepBudgetExceeded {
            limit,
            ..
        }) => assert_eq!(
            limit,
            quanta_gqlang_v2_sema::test_support::SMALL_BUDGET_MAX_SEMANTIC_STEPS
        ),
        other => panic!("expected the exact budget refusal, got {other:?}"),
    }
}

/// A deep filter chain lists every filter before its scan, in charge order: the
/// family list IS the traversal's visit order, so a note can never re-order or
/// drop a charged node.
#[test]
fn a_deep_filter_chain_lists_every_filter_before_its_scan() {
    let depth = quanta_gqlang_v2_sema::test_support::SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;
    let carrier = quanta_gqlang_v2_sema::test_support::deep_relation_chain_document_v2(
        depth,
        quanta_gqlang_v2_sema::test_support::ReferenceBudgetV2::Standard,
    )
    .expect("the chain authors");
    let input =
        quanta_gqlang_v2_algebra::lower_relation_item(carrier.document(), carrier.item_id())
            .map(quanta_gqlang_v2_planning::SemanticPlanInputV2::Relational)
            .expect("the chain lowers");
    let document =
        quanta_gqlang_v2_sema::test_support::forged_small_receipt_relation_chain_document_v2(depth)
            .expect("the small-receipt chain");
    let planned = plan(&input, document.document()).expect("the exact limit plans");

    let mut expected =
        vec![AlgebraFamilyV2::Filter; usize::try_from(depth).expect("a small depth")];
    expected.push(AlgebraFamilyV2::Scan);
    assert_eq!(planned.explain().families(), expected.as_slice());
    let (nodes, _effects) = relational_policy(&planned).expect("relational policy");
    assert_eq!(
        u32::try_from(planned.explain().families().len()).expect("a small family list"),
        nodes
    );
}
