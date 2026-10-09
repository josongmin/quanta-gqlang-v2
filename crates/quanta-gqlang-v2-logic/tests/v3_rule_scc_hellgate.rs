#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Fail-closed SCC hellgate rails for the rule program (ticket 06 / SS-06B).
//!
//! Re-expresses the five `v3_logic_hellgate.rs` rule rows that lane W3-A owns —
//! negation-cycle reject, mutual negation, aggregate-over-recursion, aggregated
//! self-recursion, and the negation-over-aggregation precedence rail — against
//! stable [`ResolvedPredicateIdV2`](quanta_gqlang_v2_types::ResolvedPredicateIdV2)
//! vertices, and adds the canonical-witness rails: the cycle is an ACTUAL edge
//! path (never a sorted member list), the offending edge is the canonical
//! minimum under any insertion permutation, and a non-local target fails closed
//! without inserting a vertex.

use quanta_gqlang_v2_logic::{
    stratify, BodyLiteralV1, BodyPolarityV1, LogicDiagnosticV1, RecursiveProgramV1,
    RejectAllRecursiveAggregatesV2, RuleDefV1, RuleGraphBudgetDimensionV1,
    RuleProgramInvalidReasonV1,
};
use quanta_gqlang_v2_sema::test_support::reference_checked_span_v2;
use quanta_gqlang_v2_sema::{CheckedLocatedSpanV2, DownstreamBudgetReceiptV2, FrontendBudgetV2};
use quanta_gqlang_v2_types::ResolvedPredicateIdV2;

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
        1_024, 4_096, 1_024, 256, 8, 128, 256, 64, 64, 1_000, 64, 128, 1_000, 4_096, 256, nodes,
        edges, steps, 64, 64, 64, 65_536,
    )
    .expect("explicit budget is well formed")
    .downstream_receipt()
}

fn generous() -> DownstreamBudgetReceiptV2 {
    receipt(4_096, 40_000, 4_000_000)
}

#[test]
fn many_offending_edges_keep_canonical_minimum_and_scan_budget_before_rejection() {
    let l = checked_locations(5);
    let mut body = vec![negated("p", &l[1])];
    for _ in 0..4096 {
        body.push(positive("p", &l[3]));
        body.push(aggregated("p", "count", &l[2]));
    }
    // Canonical graph order puts the final aggregated literal after all
    // positives, negations and the earlier aggregate locations.
    body.push(aggregated("p", "max", &l[4]));
    let exact_steps = u32::try_from(body.len() + 2).expect("discovery + emission + scan");
    let program = RecursiveProgramV1::from_rules(vec![rule("p", body, &l[0])]);
    let error =
        stratify(&program, &receipt(1, 10_000, exact_steps - 1)).expect_err("last scan refuses");
    assert!(matches!(error, LogicDiagnosticV1::RuleGraphBudgetExceeded {
        dimension: RuleGraphBudgetDimensionV1::Steps, limit, span
    } if limit == exact_steps - 1 && span == l[4]));
    let error = stratify(&program, &receipt(1, 10_000, exact_steps)).expect_err("negation wins");
    assert!(
        matches!(error, LogicDiagnosticV1::NegationCycle { cycle, offending_edge, span }
        if cycle == vec![pred("p")] && offending_edge == l[1] && span == l[0])
    );
}

#[test]
fn equal_canonical_aggregate_keys_retain_first_payload() {
    let l = checked_locations(2);
    for functions in [["count", "max"], ["max", "count"]] {
        let program = RecursiveProgramV1::from_rules(vec![rule(
            "p",
            vec![
                aggregated("p", functions[0], &l[1]),
                aggregated("p", functions[1], &l[1]),
            ],
            &l[0],
        )]);
        let error = stratify(&program, &receipt(1, 2, 4)).expect_err("recursive aggregate");
        assert!(
            matches!(error, LogicDiagnosticV1::RecursiveAggregate { function, predicate, span }
            if function == functions[0] && predicate == pred("p") && span == l[1])
        );
    }
}

fn pred(name: &str) -> ResolvedPredicateIdV2 {
    ResolvedPredicateIdV2::try_new(name).expect("canonical predicate identity")
}

fn positive(target: &str, location: &CheckedLocatedSpanV2) -> BodyLiteralV1 {
    BodyLiteralV1::new(pred(target), BodyPolarityV1::Positive, location.clone())
}

fn negated(target: &str, location: &CheckedLocatedSpanV2) -> BodyLiteralV1 {
    BodyLiteralV1::new(pred(target), BodyPolarityV1::Negated, location.clone())
}

fn aggregated(target: &str, function: &str, location: &CheckedLocatedSpanV2) -> BodyLiteralV1 {
    BodyLiteralV1::aggregated(pred(target), function, location.clone())
}

fn seed(head: &str, location: &CheckedLocatedSpanV2) -> RuleDefV1 {
    RuleDefV1::new(pred(head), Vec::new(), location.clone())
}

fn rule(head: &str, body: Vec<BodyLiteralV1>, location: &CheckedLocatedSpanV2) -> RuleDefV1 {
    RuleDefV1::new(pred(head), body, location.clone())
}

#[test]
fn raw_rule_count_rejects_before_graph_allocation_at_cap_plus_one() {
    let l = checked_locations(3);
    let budget = FrontendBudgetV2::standard_v2()
        .expect("standard budget")
        .try_with_rule_program_rules(2)
        .expect("explicit raw-rule cap")
        .downstream_receipt();
    let exact = RecursiveProgramV1::from_rules(vec![seed("p", &l[0]), seed("p", &l[1])]);
    stratify(&exact, &budget).expect("two definitions at the exact cap admit");

    let over = RecursiveProgramV1::from_rules(vec![
        seed("p", &l[0]),
        seed("p", &l[1]),
        rule("p", vec![positive("missing", &l[2])], &l[2]),
    ]);
    let error = stratify(&over, &budget).expect_err("third rule rejects before graph work");
    assert_eq!(error.code(), "logic.rule_graph_budget.rules");
    assert_eq!(error.span(), &l[2]);
    assert!(matches!(
        error,
        LogicDiagnosticV1::RuleGraphBudgetExceeded {
            dimension: RuleGraphBudgetDimensionV1::Rules,
            limit: 2,
            ..
        }
    ));
}

#[test]
fn raw_body_literal_count_rejects_before_nonlocal_scan_at_cap_plus_one() {
    let l = checked_locations(3);
    let exact = RecursiveProgramV1::from_rules(vec![rule("p", vec![positive("p", &l[1])], &l[0])]);
    stratify(&exact, &receipt(1, 1, 64)).expect("exact body-literal cap admits");

    let over = RecursiveProgramV1::from_rules(vec![rule(
        "p",
        vec![positive("p", &l[1]), positive("missing", &l[2])],
        &l[0],
    )]);
    let error = stratify(&over, &receipt(1, 1, 64))
        .expect_err("over-budget raw body rejects before nonlocal scan");
    assert_eq!(error.code(), "logic.rule_graph_budget.edges");
    assert_eq!(error.span(), &l[2]);
    assert!(
        matches!(
            error,
            LogicDiagnosticV1::RuleGraphBudgetExceeded {
                dimension: RuleGraphBudgetDimensionV1::Edges,
                limit: 1,
                ..
            }
        ),
        "the body-literal cap reports the Edges dimension at the exact cap"
    );
}

// Hellgate 3: recursion through negation (unstratifiable) rejects with a typed
// diagnostic whose payload is the canonical offending edge and the actual
// return path. `win(x) :- move(x, y), not win(y)` is the classic
// unstratifiable game: `win` depends negatively on itself.
#[test]
fn negation_cycle_rejects_with_canonical_offending_edge() {
    let l = checked_locations(4);
    let program = RecursiveProgramV1::from_rules(vec![
        seed("move", &l[0]),
        rule(
            "win",
            vec![positive("move", &l[1]), negated("win", &l[2])],
            &l[3],
        ),
    ]);

    let error = stratify(&program, &generous()).expect_err("negation cycle must reject");
    assert_eq!(error.code(), "logic.negation_cycle");
    assert_eq!(
        error.message(),
        "recursion through negation is not stratifiable: cycle [win]"
    );
    assert_eq!(error.span(), &l[3], "the rule that closed the cycle");
    match error {
        LogicDiagnosticV1::NegationCycle {
            cycle,
            offending_edge,
            span,
        } => {
            assert_eq!(cycle, vec![pred("win")]);
            assert_eq!(offending_edge, l[2], "the negated call closes the cycle");
            assert_eq!(span, l[3]);
        }
        other => panic!("expected NegationCycle, got {other:?}"),
    }
}

// A mutual negation cycle rejects with the one negated edge as the canonical
// offending edge and a proven two-predicate return path.
#[test]
fn mutual_negation_cycle_reports_actual_return_path() {
    let l = checked_locations(4);
    let program = RecursiveProgramV1::from_rules(vec![
        rule("p", vec![negated("q", &l[1])], &l[0]),
        rule("q", vec![positive("p", &l[3])], &l[2]),
    ]);

    let error = stratify(&program, &generous()).expect_err("mutual negation rejects");
    assert_eq!(error.code(), "logic.negation_cycle");
    match error {
        LogicDiagnosticV1::NegationCycle {
            cycle,
            offending_edge,
            span,
        } => {
            assert_eq!(cycle, vec![pred("p"), pred("q")]);
            assert_eq!(offending_edge, l[1], "the negated `p -> q` call");
            assert_ne!(offending_edge, l[3], "the positive `q -> p` call is not it");
            assert_eq!(span, l[0]);
        }
        other => panic!("expected NegationCycle, got {other:?}"),
    }
}

// The rendered cycle is an ACTUAL edge path, not the sorted SCC member list.
// The component is `{a, b, c}` with the only negated edge `a -> c`; the return
// path is `c -> b -> a`, so the cycle is exactly `[a, c, b]` and provably not
// the sorted `[a, b, c]` (no `a -> b` edge exists).
#[test]
fn canonical_cycle_is_an_actual_edge_path_not_a_sorted_member_list() {
    let l = checked_locations(6);
    let program = RecursiveProgramV1::from_rules(vec![
        rule("a", vec![negated("c", &l[1])], &l[0]),
        rule("c", vec![positive("b", &l[3])], &l[2]),
        rule("b", vec![positive("a", &l[5])], &l[4]),
    ]);

    let error = stratify(&program, &generous()).expect_err("the negated cycle rejects");
    match error {
        LogicDiagnosticV1::NegationCycle {
            cycle,
            offending_edge,
            span,
        } => {
            assert_eq!(cycle, vec![pred("a"), pred("c"), pred("b")]);
            assert_ne!(
                cycle,
                vec![pred("a"), pred("b"), pred("c")],
                "a sorted member list is not a witness"
            );
            assert_eq!(offending_edge, l[1], "the canonical negated `a -> c` call");
            assert_eq!(span, l[0]);
        }
        other => panic!("expected NegationCycle, got {other:?}"),
    }
}

// Aggregate over recursion: the offending aggregated edge folds the recursive
// predicate `t`, and the rendered payload carries the aggregate spelling, the
// FOLDED predicate identity, and the exact aggregate-call location.
#[test]
fn aggregate_over_recursion_rejects_with_folded_predicate() {
    let l = checked_locations(4);
    let program = RecursiveProgramV1::from_rules(vec![
        rule("t", vec![positive("s", &l[1])], &l[0]),
        rule("s", vec![aggregated("t", "count", &l[3])], &l[2]),
    ]);

    let error = stratify(&program, &generous()).expect_err("aggregate recursion rejects");
    assert_eq!(error.code(), "logic.recursive_aggregate");
    assert_eq!(
        error.message(),
        "recursive aggregate `count` over recursive predicate `t` is not admitted"
    );
    assert_eq!(error.span(), &l[3], "the aggregate call's checked location");
    match error {
        LogicDiagnosticV1::RecursiveAggregate {
            function,
            predicate,
            span,
        } => {
            assert_eq!(function, "count");
            assert_eq!(predicate, pred("t"), "the folded predicate, not the head");
            assert_eq!(span, l[3]);
        }
        other => panic!("expected RecursiveAggregate, got {other:?}"),
    }
}

// Aggregated self-recursion rejects under the same fixed policy, and a
// projection that carried no spelling renders the stable placeholder rather
// than inventing one.
#[test]
fn aggregated_self_recursion_rejects_with_fixed_policy() {
    let l = checked_locations(4);
    let spelled = RecursiveProgramV1::from_rules(vec![rule(
        "rank",
        vec![aggregated("rank", "min", &l[1])],
        &l[0],
    )]);
    let error = stratify(&spelled, &generous()).expect_err("recursive min rejects");
    assert_eq!(error.code(), "logic.recursive_aggregate");
    match error {
        LogicDiagnosticV1::RecursiveAggregate {
            function,
            predicate,
            span,
        } => {
            assert_eq!(function, "min");
            assert_eq!(predicate, pred("rank"));
            assert_eq!(span, l[1]);
        }
        other => panic!("expected RecursiveAggregate, got {other:?}"),
    }

    let unspelled = RecursiveProgramV1::from_rules(vec![rule(
        "rank",
        vec![BodyLiteralV1::new(
            pred("rank"),
            BodyPolarityV1::Aggregated,
            l[2].clone(),
        )],
        &l[0],
    )]);
    let error = stratify(&unspelled, &generous()).expect_err("unspelled recursion rejects");
    match error {
        LogicDiagnosticV1::RecursiveAggregate { function, span, .. } => {
            assert_eq!(
                function,
                RejectAllRecursiveAggregatesV2::UNSPECIFIED_FUNCTION
            );
            assert_eq!(span, l[2]);
        }
        other => panic!("expected RecursiveAggregate, got {other:?}"),
    }
}

// PRECEDENCE rail: a rule that negates its own recursion is a negation cycle,
// so it rejects as `logic.negation_cycle` — never as the aggregate family. The
// second fixture puts the aggregated literal at the SMALLEST canonical location
// so the aggregate candidate would win a key-only selection; negation must
// still win, because precedence is owned by polarity, not by location.
#[test]
fn negation_beats_aggregation_regardless_of_edge_order() {
    let l = checked_locations(4);

    let negated_first = RecursiveProgramV1::from_rules(vec![rule(
        "p",
        vec![
            BodyLiteralV1::new(pred("p"), BodyPolarityV1::Negated, l[1].clone()),
            aggregated("p", "count", &l[2]),
        ],
        &l[0],
    )]);
    let error = stratify(&negated_first, &generous()).expect_err("negated self-recursion rejects");
    assert_eq!(error.code(), "logic.negation_cycle");
    assert_eq!(
        error.message(),
        "recursion through negation is not stratifiable: cycle [p]"
    );
    match error {
        LogicDiagnosticV1::NegationCycle {
            cycle,
            offending_edge,
            span,
        } => {
            assert_eq!(cycle, vec![pred("p")]);
            assert_eq!(offending_edge, l[1]);
            assert_eq!(span, l[0]);
        }
        other => panic!("expected NegationCycle, got {other:?}"),
    }

    let aggregated_first = RecursiveProgramV1::from_rules(vec![rule(
        "p",
        vec![
            aggregated("p", "count", &l[1]),
            BodyLiteralV1::new(pred("p"), BodyPolarityV1::Negated, l[2].clone()),
        ],
        &l[0],
    )]);
    let error = stratify(&aggregated_first, &generous())
        .expect_err("negated self-recursion still rejects as a negation cycle");
    assert_eq!(error.code(), "logic.negation_cycle");
    match error {
        LogicDiagnosticV1::NegationCycle {
            cycle,
            offending_edge,
            ..
        } => {
            assert_eq!(cycle, vec![pred("p")]);
            assert_eq!(
                offending_edge, l[2],
                "the negated edge wins even when the aggregate edge sorts first"
            );
        }
        other => panic!("expected NegationCycle, got {other:?}"),
    }
}

// The canonical offending edge is the global minimum across every recursive
// component and every insertion order. This program carries four non-positive
// recursive candidates: three self-loops (`a` negated, `b` negated, `c`
// aggregated at the smallest location) and the mutual `d <-> e` negation. The
// canonical answer is always `a`'s negated self-loop, in every permutation.
#[test]
fn canonical_offending_edge_is_permutation_invariant() {
    let l = checked_locations(12);
    let build = |order: &[u8]| {
        let rules = [
            rule("a", vec![negated("a", &l[2])], &l[1]),
            rule("b", vec![negated("b", &l[4])], &l[3]),
            rule("c", vec![aggregated("c", "count", &l[0])], &l[5]),
            rule("d", vec![negated("e", &l[6])], &l[7]),
            rule("e", vec![positive("d", &l[8])], &l[9]),
        ];
        let mut ordered: Vec<RuleDefV1> = Vec::new();
        for index in order {
            let rule = rules
                .get(usize::from(*index))
                .expect("permutation index names one of the five fixture rules");
            ordered.push(rule.clone());
        }
        assert_eq!(
            ordered.len(),
            rules.len(),
            "every permutation must place every fixture rule, not silently drop one"
        );
        RecursiveProgramV1::from_rules(ordered)
    };

    let reference = stratify(&build(&[0, 1, 2, 3, 4]), &generous())
        .expect_err("the canonical negation cycle rejects");
    assert_eq!(reference.code(), "logic.negation_cycle");
    match &reference {
        LogicDiagnosticV1::NegationCycle {
            cycle,
            offending_edge,
            span,
        } => {
            assert_eq!(cycle, &vec![pred("a")], "the smallest negated source wins");
            assert_eq!(offending_edge, &l[2]);
            assert_eq!(span, &l[1]);
        }
        other => panic!("expected NegationCycle, got {other:?}"),
    }

    fn permute(order: &mut [u8; 5], next: usize, orders: &mut Vec<[u8; 5]>) {
        if next == order.len() {
            orders.push(*order);
            return;
        }
        for index in next..order.len() {
            order.swap(next, index);
            permute(order, next + 1, orders);
            order.swap(next, index);
        }
    }
    let mut orders = Vec::new();
    permute(&mut [0, 1, 2, 3, 4], 0, &mut orders);
    assert_eq!(orders.len(), 120);
    assert_eq!(
        orders
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        120
    );
    for order in orders {
        let permuted = stratify(&build(&order), &generous())
            .expect_err("every permutation rejects identically");
        assert_eq!(
            permuted, reference,
            "the rendered witness is insertion-order invariant for {order:?}"
        );
    }
}

// A body target that is not a member predicate fails closed before any vertex
// is inserted or any budget is charged: the rejection survives the smallest
// possible budget. A host predicate that shares a spelling segment does not
// rescue the reference, because identity — not spelling — is the contract.
#[test]
fn nonlocal_target_fails_closed_without_inserting_a_vertex() {
    let l = checked_locations(4);
    let program =
        RecursiveProgramV1::from_rules(vec![rule("p", vec![positive("q", &l[1])], &l[0])]);

    let error = stratify(&program, &receipt(1, 1, 1)).expect_err("a non-local target rejects");
    assert_eq!(error.code(), "logic.rule_program.nonlocal_target");
    assert_eq!(
        error.message(),
        "rule call target is not a predicate of this document"
    );
    assert_eq!(error.span(), &l[1]);
    match error {
        LogicDiagnosticV1::RuleProgramInvalid { reason, span } => {
            assert_eq!(reason, RuleProgramInvalidReasonV1::NonLocalTarget);
            assert_eq!(span, l[1]);
        }
        other => panic!("expected RuleProgramInvalid, got {other:?}"),
    }

    let collision = RecursiveProgramV1::from_rules(vec![
        seed("host::q", &l[0]),
        rule("p", vec![positive("q", &l[2])], &l[3]),
    ]);
    let error = stratify(&collision, &receipt(1, 1, 1))
        .expect_err("a same-segment host predicate is still non-local");
    assert_eq!(error.code(), "logic.rule_program.nonlocal_target");
    assert_eq!(error.span(), &l[2]);
}

// Mutation case 2 as an executed data mutation: downgrading the aggregated
// recursion edge to a positive recursion edge flips the program from rejection
// to admission, proving the polarity — not the aggregate spelling — owns the
// decision.
#[test]
fn aggregate_downgraded_to_positive_admits() {
    let l = checked_locations(4);
    let aggregated_program = RecursiveProgramV1::from_rules(vec![rule(
        "rank",
        vec![aggregated("rank", "count", &l[1])],
        &l[0],
    )]);
    assert_eq!(
        stratify(&aggregated_program, &generous())
            .expect_err("aggregated recursion rejects")
            .code(),
        "logic.recursive_aggregate"
    );

    let downgraded =
        RecursiveProgramV1::from_rules(vec![rule("rank", vec![positive("rank", &l[1])], &l[0])]);
    let proof = stratify(&downgraded, &generous()).expect("positive self-recursion admits");
    assert!(proof.is_recursive(&pred("rank")));
    assert_eq!(proof.stratum_of(&pred("rank")), Some(0));
    assert_eq!(proof.stratum_count(), 1);
}
