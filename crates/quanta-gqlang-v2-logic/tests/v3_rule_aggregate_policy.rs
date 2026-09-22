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

//! Recursive-aggregate policy rails for the logic layer (ticket 06 / SS-06B).
//!
//! `SS-06B` step 6 replaces the V1 name-keyed admit/reject matrix with one
//! fixed policy carrier. These rails prove the replacement is total: every
//! aggregate spelling rejects over recursion, the V1 `min`/`max` admission is
//! dead, an unspelled projection renders the stable placeholder, and a
//! non-recursive aggregate stays outside Logic's legality authority.

use quanta_gqlang_v2_logic::{
    stratify, BodyLiteralV1, BodyPolarityV1, LogicDiagnosticV1, RecursiveProgramV1,
    RejectAllRecursiveAggregatesV2, RuleDefV1,
};
use quanta_gqlang_v2_sema::test_support::reference_checked_span_v2;
use quanta_gqlang_v2_sema::{CheckedLocatedSpanV2, DownstreamBudgetReceiptV2, FrontendBudgetV2};
use quanta_gqlang_v2_types::ResolvedPredicateIdV2;

/// Every aggregate spelling this packet recognizes in source text. None of them
/// is a legality proof for a recursive SCC.
const EVERY_SPELLING: [&str; 8] = [
    "count", "sum", "avg", "min", "max", "any", "collect", "fold",
];

/// Distinct registry-proven checked locations in canonical ascending order.
///
/// `count` locations are minted at consecutive byte ranges `[40 + 4i, 44 + 4i)`
/// of the sema reference source through the W3-D support minter, so ordering by
/// `(source, start, end)` is exactly index order and this crate's tests never
/// name an HIR type.
fn checked_locations(count: usize) -> Vec<CheckedLocatedSpanV2> {
    (0..count)
        .map(|index| {
            reference_checked_span_v2(40 + 4 * index, 44 + 4 * index)
                .expect("fixture checked location is in bounds")
        })
        .collect()
}

fn receipt(nodes: u32, edges: u32, steps: u32) -> DownstreamBudgetReceiptV2 {
    FrontendBudgetV2::try_new(
        1_024, 4_096, 1_024, 256, 8, 256, 64, 64, 1_000, 64, 128, 1_000, 4_096, 256, nodes, edges,
        steps, 64, 64, 64, 65_536,
    )
    .expect("explicit budget is well formed")
    .downstream_receipt()
}

fn generous() -> DownstreamBudgetReceiptV2 {
    receipt(4_096, 40_000, 4_000_000)
}

fn pred(name: &str) -> ResolvedPredicateIdV2 {
    ResolvedPredicateIdV2::try_new(name).expect("canonical predicate identity")
}

fn seed(head: &str, location: &CheckedLocatedSpanV2) -> RuleDefV1 {
    RuleDefV1::new(pred(head), Vec::new(), location.clone())
}

fn self_aggregate(function: &str, location: &CheckedLocatedSpanV2) -> RecursiveProgramV1 {
    RecursiveProgramV1::from_rules(vec![RuleDefV1::new(
        pred("rank"),
        vec![BodyLiteralV1::aggregated(
            pred("rank"),
            function,
            location.clone(),
        )],
        location.clone(),
    )])
}

// The policy carrier rejects every spelling at the exact supplied identity and
// location; there is no admitted arm to reach.
#[test]
fn reject_all_policy_rejects_every_spelling() {
    let l = checked_locations(4);
    // The carrier is a unit policy: it has exactly one value, no admit list,
    // and no extension point.
    let policy = RejectAllRecursiveAggregatesV2::new();

    for spelling in EVERY_SPELLING {
        let error = policy
            .check(spelling, &pred("reaches"), &l[1])
            .expect_err("every recursive aggregate spelling rejects");
        assert_eq!(error.code(), "logic.recursive_aggregate");
        assert_eq!(
            error.message(),
            format!(
                "recursive aggregate `{spelling}` over recursive predicate `reaches` is not admitted"
            )
        );
        assert_eq!(error.span(), &l[1]);
        match error {
            LogicDiagnosticV1::RecursiveAggregate {
                function,
                predicate,
                span,
            } => {
                assert_eq!(function, spelling);
                assert_eq!(predicate, pred("reaches"));
                assert_eq!(span, l[1]);
            }
            other => panic!("expected RecursiveAggregate, got {other:?}"),
        }
    }
}

// The same totality holds through the real SCC path: an aggregate that folds a
// recursive predicate rejects for every spelling, and the rendered function is
// the spelling that was projected.
#[test]
fn recursive_aggregate_rejects_for_every_spelling_through_scc() {
    let l = checked_locations(4);
    for spelling in EVERY_SPELLING {
        let program = self_aggregate(spelling, &l[1]);
        let error = stratify(&program, &generous())
            .expect_err("an aggregate over its own recursion rejects");
        assert_eq!(error.code(), "logic.recursive_aggregate");
        match error {
            LogicDiagnosticV1::RecursiveAggregate {
                function,
                predicate,
                span,
            } => {
                assert_eq!(function, spelling, "the projected spelling is retained");
                assert_eq!(predicate, pred("rank"));
                assert_eq!(span, l[1]);
            }
            other => panic!("expected RecursiveAggregate, got {other:?}"),
        }
    }
}

// Oracle kill: V1 asserted that `min` (and `max`) over recursion ADMITS as a
// monotone lattice fold. This packet requires the opposite, so both spellings
// must reject through the same fixed policy.
#[test]
fn v1_min_max_recursion_admission_is_dead() {
    let l = checked_locations(4);
    for spelling in ["min", "max"] {
        let program = self_aggregate(spelling, &l[1]);
        let error = stratify(&program, &generous())
            .expect_err("the V1 min/max recursion admission is deleted");
        assert_eq!(error.code(), "logic.recursive_aggregate");
        assert_eq!(error.span(), &l[1]);
    }
}

// An aggregated reference projected without a spelling renders the stable
// placeholder instead of guessing a name from the predicate identity.
#[test]
fn unspelled_recursive_aggregate_renders_stable_placeholder() {
    assert_eq!(
        RejectAllRecursiveAggregatesV2::UNSPECIFIED_FUNCTION,
        "<aggregate>"
    );
    let l = checked_locations(4);
    let program = RecursiveProgramV1::from_rules(vec![RuleDefV1::new(
        pred("rank"),
        vec![BodyLiteralV1::new(
            pred("rank"),
            BodyPolarityV1::Aggregated,
            l[2].clone(),
        )],
        l[0].clone(),
    )]);

    let literal = program
        .rules()
        .first()
        .and_then(|rule| rule.body().first())
        .expect("the fixture rule carries one body reference");
    assert_eq!(literal.aggregate_function(), None);
    assert_eq!(literal.polarity(), BodyPolarityV1::Aggregated);

    let error = stratify(&program, &generous()).expect_err("unspelled recursion rejects");
    assert_eq!(
        error.message(),
        "recursive aggregate `<aggregate>` over recursive predicate `rank` is not admitted"
    );
    match error {
        LogicDiagnosticV1::RecursiveAggregate { function, .. } => {
            assert_eq!(
                function,
                RejectAllRecursiveAggregatesV2::UNSPECIFIED_FUNCTION
            );
        }
        other => panic!("expected RecursiveAggregate, got {other:?}"),
    }
}

// A non-recursive aggregate gains no Logic-side legality decision: `t` is not
// recursive, so Logic admits and simply layers `s` strictly above `t`. The
// nonrecursive verdict stays Sema's.
#[test]
fn nonrecursive_aggregate_stays_outside_logic_legality() {
    let l = checked_locations(6);
    let program = RecursiveProgramV1::from_rules(vec![
        seed("base", &l[0]),
        RuleDefV1::new(
            pred("t"),
            vec![BodyLiteralV1::new(
                pred("base"),
                BodyPolarityV1::Positive,
                l[2].clone(),
            )],
            l[1].clone(),
        ),
        RuleDefV1::new(
            pred("s"),
            vec![BodyLiteralV1::aggregated(pred("t"), "count", l[4].clone())],
            l[3].clone(),
        ),
    ]);

    let proof = stratify(&program, &generous()).expect("a non-recursive aggregate is not ours");
    assert!(
        proof.recursive().is_empty(),
        "no predicate recurses, so no aggregate is recursive"
    );
    let t = proof.stratum_of(&pred("t")).expect("t stratum");
    let s = proof.stratum_of(&pred("s")).expect("s stratum");
    assert!(s > t, "aggregation layers like negation: s={s} t={t}");
    assert_eq!(proof.stratum_count(), 2);
}
