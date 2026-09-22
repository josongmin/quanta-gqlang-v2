#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! Positive-recursion admission rails for the logic layer (ticket 06 / SS-06B).
//!
//! syntax->sema->logic path pinned: the sealed document's resolved predicate
//! identities and checked locations -> [`stratify`]. Proves `D27`: positive
//! recursion admits under EXPLICIT least-fixpoint semantics with a
//! stratification proof; stratified negation (negation crossing to a lower
//! stratum) admits; the canonical SCC decomposition and strata are invariant
//! under rule/body-literal insertion permutation.
//!
//! Checked locations are extracted from a sealed sema reference document, so
//! this crate's tests never name an HIR type: the location proof comes from the
//! real semantic transaction, not a test-local spindle.

use quanta_gqlang_v2_logic::{
    stratify, BodyLiteralV1, BodyPolarityV1, LogicDiagnosticV1, RecursiveProgramV1, RuleDefV1,
    RuleGraphBudgetDimensionV1, StratificationProofV1,
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

/// A rule-graph budget with explicit rule dimensions and generous neighbours.
fn receipt(nodes: u32, edges: u32, steps: u32) -> DownstreamBudgetReceiptV2 {
    FrontendBudgetV2::try_new(
        1_024, 4_096, 1_024, 256, 8, 256, 64, 64, 1_000, 64, 128, 1_000, 4_096, 256, nodes, edges,
        steps, 64, 64, 64, 65_536,
    )
    .expect("explicit budget is well formed")
    .downstream_receipt()
}

/// A budget that admits every fixture in this file.
fn generous() -> DownstreamBudgetReceiptV2 {
    receipt(4_096, 40_000, 4_000_000)
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

fn seed(head: &str, location: &CheckedLocatedSpanV2) -> RuleDefV1 {
    RuleDefV1::new(pred(head), Vec::new(), location.clone())
}

fn rule(head: &str, body: Vec<BodyLiteralV1>, location: &CheckedLocatedSpanV2) -> RuleDefV1 {
    RuleDefV1::new(pred(head), body, location.clone())
}

/// The acyclic diamond `d :- b, c ; b :- a ; c :- a` with one seed vertex.
///
/// Distinct locations make the canonical orders observable: vertices ascend
/// `a, b, c, d` (rule locations `l[0], l[1], l[3], l[5]`) and edges ascend
/// `b->a, c->a, d->b, d->c` (literal locations `l[2], l[4], l[6], l[7]`).
fn diamond_program() -> (RecursiveProgramV1, Vec<CheckedLocatedSpanV2>) {
    let l = checked_locations(8);
    let program = RecursiveProgramV1::from_rules(vec![
        seed("a", &l[0]),
        rule("b", vec![positive("a", &l[2])], &l[1]),
        rule("c", vec![positive("a", &l[4])], &l[3]),
        rule("d", vec![positive("b", &l[6]), positive("c", &l[7])], &l[5]),
    ]);
    (program, l)
}

// Hellgate 1 (positive): transitive closure
// `reaches(x, z) :- edge(x, z)`
// `reaches(x, z) :- edge(x, y), reaches(y, z)`
// admits under least-fixpoint with a stratification proof: `reaches` is
// recursive (self-loop, positive) and lands in a single stratum.
#[test]
fn positive_transitive_closure_admits_with_proof() {
    let l = checked_locations(8);
    let program = RecursiveProgramV1::from_rules(vec![
        seed("edge", &l[0]),
        rule("reaches", vec![positive("edge", &l[2])], &l[1]),
        rule(
            "reaches",
            vec![positive("edge", &l[3]), positive("reaches", &l[4])],
            &l[5],
        ),
    ]);

    let proof = stratify(&program, &generous()).expect("positive recursion admits");
    assert!(proof.is_recursive(&pred("reaches")));
    assert!(!proof.is_recursive(&pred("edge")));
    assert_eq!(
        proof.recursive().len(),
        1,
        "exactly the self-recursive head"
    );
    assert_eq!(proof.stratum_count(), 1);
    assert_eq!(proof.stratum_of(&pred("reaches")), Some(0));
    assert_eq!(proof.stratum_of(&pred("edge")), Some(0));
    assert_eq!(
        proof.components(),
        vec![vec![pred("edge")], vec![pred("reaches")],],
        "the canonical decomposition lists singletons in predicate order"
    );
}

// A non-recursive dependency chain admits with no recursive predicate, so the
// proof never marks a merely-referenced predicate recursive.
#[test]
fn nonrecursive_dependency_admits_without_recursion() {
    let l = checked_locations(4);
    let program = RecursiveProgramV1::from_rules(vec![
        seed("base", &l[0]),
        rule("derived", vec![positive("base", &l[2])], &l[1]),
    ]);

    let proof = stratify(&program, &generous()).expect("a DAG admits");
    assert!(proof.recursive().is_empty(), "no SCC is recursive");
    assert_eq!(proof.stratum_of(&pred("derived")), Some(0));
    assert_eq!(proof.stratum_count(), 1);
}

// Stratified negation: `safe` negates `tainted`, but `tainted` does NOT depend
// on `safe`, so negation crosses to a strictly lower stratum and admits.
#[test]
fn stratified_negation_admits_with_layered_strata() {
    let l = checked_locations(12);
    // source(x) ; tainted(x) :- source(x) ; tainted(x) :- edge(y, x), tainted(y)
    // safe(x) :- node(x), not tainted(x)
    let program = RecursiveProgramV1::from_rules(vec![
        seed("source", &l[0]),
        seed("edge", &l[1]),
        seed("node", &l[2]),
        rule("tainted", vec![positive("source", &l[4])], &l[3]),
        rule(
            "tainted",
            vec![positive("edge", &l[5]), positive("tainted", &l[6])],
            &l[7],
        ),
        rule(
            "safe",
            vec![positive("node", &l[8]), negated("tainted", &l[9])],
            &l[10],
        ),
    ]);

    let proof = stratify(&program, &generous()).expect("stratified negation admits");
    assert!(proof.is_recursive(&pred("tainted")));
    assert!(!proof.is_recursive(&pred("safe")));
    let safe = proof.stratum_of(&pred("safe")).expect("safe stratum");
    let tainted = proof.stratum_of(&pred("tainted")).expect("tainted stratum");
    assert!(
        safe > tainted,
        "negation must raise the stratum: safe={safe} tainted={tainted}"
    );
    assert_eq!(safe, 1);
    assert_eq!(tainted, 0);
    assert_eq!(proof.stratum_count(), 2);
}

// Mutual recursion forms one multi-member SCC: `a :- b`, `b :- a`. Both
// predicates sit in the SAME non-trivial SCC, so both are recursive and share
// one stratum.
#[test]
fn mutual_recursion_forms_one_recursive_component() {
    let l = checked_locations(6);
    let program = RecursiveProgramV1::from_rules(vec![
        rule("a", vec![positive("b", &l[1])], &l[0]),
        rule("b", vec![positive("a", &l[3])], &l[2]),
    ]);

    let proof = stratify(&program, &generous()).expect("positive mutual recursion admits");
    assert!(proof.is_recursive(&pred("a")));
    assert!(proof.is_recursive(&pred("b")));
    assert_eq!(proof.recursive().len(), 2, "exactly the two SCC members");
    assert_eq!(
        proof.components(),
        vec![vec![pred("a"), pred("b")]],
        "one component carries both members in predicate order"
    );
    assert_eq!(proof.stratum_of(&pred("a")), Some(0));
    assert_eq!(proof.stratum_of(&pred("b")), Some(0));
    assert_eq!(proof.stratum_count(), 1);
}

// A self-loop predicate is recursive through its own component, NOT through a
// multi-member SCC; a singleton without a self-loop is not recursive.
#[test]
fn self_loop_is_recursive_but_singleton_base_is_not() {
    let l = checked_locations(6);
    let program = RecursiveProgramV1::from_rules(vec![
        rule("r", vec![positive("r", &l[1])], &l[0]),
        seed("base", &l[2]),
        rule("s", vec![positive("base", &l[4])], &l[3]),
    ]);

    let proof = stratify(&program, &generous()).expect("self recursion admits");
    assert!(proof.is_recursive(&pred("r")), "self-loop is recursive");
    assert!(
        !proof.is_recursive(&pred("s")),
        "singleton non-self-loop is not"
    );
    assert!(!proof.is_recursive(&pred("base")));
    assert_eq!(proof.recursive().len(), 1);
}

// A three-level negation chain forces strata to LAYER monotonically: the
// relaxation must push every source strictly above its negated target, and the
// assigned stratum numbers are EXACTLY 0/1/2/3 in dependency order.
#[test]
fn deep_negation_chain_layers_strata_monotonically() {
    let l = checked_locations(16);
    // a :- base ; b :- node, not a ; c :- node, not b ; d :- node, not c
    let program = RecursiveProgramV1::from_rules(vec![
        seed("base", &l[0]),
        seed("node", &l[1]),
        rule("a", vec![positive("base", &l[3])], &l[2]),
        rule(
            "b",
            vec![positive("node", &l[4]), negated("a", &l[5])],
            &l[6],
        ),
        rule(
            "c",
            vec![positive("node", &l[7]), negated("b", &l[8])],
            &l[9],
        ),
        rule(
            "d",
            vec![positive("node", &l[10]), negated("c", &l[11])],
            &l[12],
        ),
    ]);

    let proof = stratify(&program, &generous()).expect("deep stratified negation admits");
    assert_eq!(proof.stratum_of(&pred("a")), Some(0), "base predicate at 0");
    assert_eq!(proof.stratum_of(&pred("b")), Some(1), "b negates a -> 1");
    assert_eq!(proof.stratum_of(&pred("c")), Some(2), "c negates b -> 2");
    assert_eq!(proof.stratum_of(&pred("d")), Some(3), "d negates c -> 3");
    assert_eq!(proof.stratum_count(), 4, "four layers 0..=3");
    assert_eq!(proof.strata().len(), 6, "every member predicate is mapped");
}

// A negation chain whose canonical predicate order OPPOSES the dependency
// direction forces more than one relaxation pass: `s1` sorts before `s3` which
// sorts before `s5`, yet the negation dependency runs the other way. The strata
// must still converge to the correct layering.
#[test]
fn reversed_negation_chain_converges_by_relaxation() {
    let l = checked_locations(8);
    let program = RecursiveProgramV1::from_rules(vec![
        seed("base", &l[0]),
        rule("s1", vec![negated("s3", &l[2])], &l[1]),
        rule("s3", vec![negated("s5", &l[4])], &l[3]),
        rule("s5", vec![positive("base", &l[6])], &l[5]),
    ]);

    let proof = stratify(&program, &generous()).expect("reversed negation chain admits");
    assert_eq!(proof.stratum_of(&pred("s5")), Some(0));
    assert_eq!(proof.stratum_of(&pred("s3")), Some(1));
    assert_eq!(proof.stratum_of(&pred("s1")), Some(2));
    assert_eq!(proof.stratum_count(), 3);
}

// The positive acyclic diamond is non-recursive over stable IDs and admits in
// exactly one stratum with the canonical four-singleton decomposition.
#[test]
fn acyclic_diamond_admits_with_canonical_components() {
    let (program, _) = diamond_program();
    let proof = stratify(&program, &generous()).expect("the diamond admits");

    assert!(proof.recursive().is_empty(), "a DAG has no recursive SCC");
    assert_eq!(proof.stratum_count(), 1);
    assert_eq!(proof.strata().len(), 4);
    assert_eq!(
        proof.components(),
        vec![
            vec![pred("a")],
            vec![pred("b")],
            vec![pred("c")],
            vec![pred("d")],
        ]
    );
    for head in ["a", "b", "c", "d"] {
        assert_eq!(proof.stratum_of(&pred(head)), Some(0), "positive DAG");
    }
}

// The proof is invariant under rule insertion permutation AND under body
// literal permutation: the canonical decomposition, strata, and recursion set
// are the same object for every authored order.
#[test]
fn proof_is_invariant_under_rule_and_literal_permutation() {
    let (canonical, l) = diamond_program();
    let reference = stratify(&canonical, &generous()).expect("canonical order admits");

    // Rule order reversed.
    let reversed = RecursiveProgramV1::from_rules(vec![
        rule("d", vec![positive("b", &l[6]), positive("c", &l[7])], &l[5]),
        rule("c", vec![positive("a", &l[4])], &l[3]),
        rule("b", vec![positive("a", &l[2])], &l[1]),
        seed("a", &l[0]),
    ]);
    assert_eq!(
        stratify(&reversed, &generous()).expect("reversed order admits"),
        reference
    );

    // `d` body literals swapped (canonical edge order must be unaffected).
    let swapped = RecursiveProgramV1::from_rules(vec![
        seed("a", &l[0]),
        rule("b", vec![positive("a", &l[2])], &l[1]),
        rule("c", vec![positive("a", &l[4])], &l[3]),
        rule("d", vec![positive("c", &l[7]), positive("b", &l[6])], &l[5]),
    ]);
    assert_eq!(
        stratify(&swapped, &generous()).expect("swapped literals admit"),
        reference
    );
}

// An empty program admits with an empty proof and zero strata; the projection
// adapter, not this layer, owns the non-empty member requirement.
#[test]
fn empty_program_admits_with_zero_strata() {
    let program = RecursiveProgramV1::from_rules(Vec::new());
    let proof = stratify(&program, &receipt(1, 1, 1)).expect("empty program admits");
    assert!(proof.strata().is_empty());
    assert!(proof.recursive().is_empty());
    assert!(proof.components().is_empty());
    assert_eq!(proof.stratum_count(), 0);
}

// Node budget: the exact limit admits, one less rejects with the exact
// dimension, limit, and the refused vertex's defining-rule checked location.
#[test]
fn node_budget_exact_limit_admits_and_limit_minus_one_rejects() {
    let (program, l) = diamond_program();
    assert!(
        stratify(&program, &receipt(4, 40, 400)).is_ok(),
        "four vertices fit the exact node limit"
    );

    let error = stratify(&program, &receipt(3, 40, 400)).expect_err("three vertices do not");
    assert_eq!(error.code(), "logic.rule_graph_budget.nodes");
    assert_eq!(error.message(), "rule graph node budget of 3 exceeded");
    match error {
        LogicDiagnosticV1::RuleGraphBudgetExceeded {
            dimension,
            limit,
            span,
        } => {
            assert_eq!(dimension, RuleGraphBudgetDimensionV1::Nodes);
            assert_eq!(limit, 3);
            assert_eq!(span, l[5], "the refused fourth vertex is `d`");
        }
        other => panic!("expected a rule-graph budget rejection, got {other:?}"),
    }
}

// Edge budget: one charge per projected call edge in canonical order; the
// refused edge is the fourth canonical edge (`d -> c`).
#[test]
fn edge_budget_exact_limit_admits_and_limit_minus_one_rejects() {
    let (program, l) = diamond_program();
    assert!(
        stratify(&program, &receipt(40, 4, 400)).is_ok(),
        "four edges fit the exact edge limit"
    );

    let error = stratify(&program, &receipt(40, 3, 400)).expect_err("three edges do not");
    assert_eq!(error.code(), "logic.rule_graph_budget.edges");
    assert_eq!(error.message(), "rule graph edge budget of 3 exceeded");
    match error {
        LogicDiagnosticV1::RuleGraphBudgetExceeded {
            dimension,
            limit,
            span,
        } => {
            assert_eq!(dimension, RuleGraphBudgetDimensionV1::Edges);
            assert_eq!(limit, 3);
            assert_eq!(span, l[7], "the refused fourth canonical edge is `d -> c`");
        }
        other => panic!("expected a rule-graph budget rejection, got {other:?}"),
    }
}

// Step budget: pushes, SCC emissions, offending-edge scan, and relaxation are
// all charged. The diamond charges exactly 16 steps (4 pushes + 4 emissions +
// 4 scan + 4 relaxation); 15 rejects at the refused relaxation charge.
#[test]
fn step_budget_exact_limit_admits_and_limit_minus_one_rejects() {
    let (program, l) = diamond_program();
    assert!(
        stratify(&program, &receipt(40, 40, 16)).is_ok(),
        "sixteen steps fit the exact step limit"
    );

    let error = stratify(&program, &receipt(40, 40, 15)).expect_err("fifteen do not");
    assert_eq!(error.code(), "logic.rule_graph_budget.steps");
    assert_eq!(error.message(), "rule graph step budget of 15 exceeded");
    match error {
        LogicDiagnosticV1::RuleGraphBudgetExceeded {
            dimension,
            limit,
            span,
        } => {
            assert_eq!(dimension, RuleGraphBudgetDimensionV1::Steps);
            assert_eq!(limit, 15);
            assert_eq!(
                span, l[7],
                "the refused sixteenth step is the `d -> c` relaxation charge"
            );
        }
        other => panic!("expected a rule-graph budget rejection, got {other:?}"),
    }
}

// A 50k-deep chain is admitted at the exact `max_rule_graph_nodes` limit with
// no process-stack recursion. The stratifier runs on a deliberately small
// worker stack (512 KiB), so any recursive SCC descent over 50k nodes would
// overflow instead of passing.
#[test]
fn fifty_thousand_deep_chain_needs_no_process_stack() {
    const DEPTH: usize = 50_000;
    let l = checked_locations(2);
    let mut rules: Vec<RuleDefV1> = Vec::new();
    for index in 0..DEPTH {
        let head = format!("p{index:06}");
        let body = if index + 1 < DEPTH {
            vec![positive(&format!("p{:06}", index + 1), &l[1])]
        } else {
            Vec::new()
        };
        rules.push(rule(&head, body, &l[0]));
    }
    let chain = RecursiveProgramV1::from_rules(rules);

    let standard = FrontendBudgetV2::standard_v2()
        .expect("the pinned standard budget")
        .downstream_receipt();
    assert_eq!(
        standard.max_rule_graph_nodes(),
        50_000,
        "the admitted chain sits exactly on the sealed node limit"
    );

    let proof = std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .stack_size(512 * 1024)
            .spawn_scoped(scope, || stratify(&chain, &standard))
            .expect("small-stack worker thread");
        worker.join().expect("worker thread joins")
    })
    .expect("the 50k chain admits");
    assert_eq!(proof.components().len(), DEPTH);
    assert!(proof.recursive().is_empty());
    assert_eq!(proof.stratum_count(), 1);

    // exact/+1 on the sealed node dimension: 49_999 rejects at the last vertex.
    let error = stratify(&chain, &receipt(49_999, 200_000, 1_000_000))
        .expect_err("one vertex over the node budget rejects");
    match error {
        LogicDiagnosticV1::RuleGraphBudgetExceeded {
            dimension, limit, ..
        } => {
            assert_eq!(dimension, RuleGraphBudgetDimensionV1::Nodes);
            assert_eq!(limit, 49_999);
        }
        other => panic!("expected a node-budget rejection, got {other:?}"),
    }
}

// The chain's full step charge is 50_000 pushes + 50_000 SCC emissions +
// 49_999 scan edges + 49_999 relaxation edges = 199_998, so the exact step
// limit admits and one less rejects at the final relaxation charge.
#[test]
fn fifty_thousand_deep_chain_step_charge_is_exact_and_bounded() {
    const DEPTH: usize = 50_000;
    let l = checked_locations(2);
    let mut rules: Vec<RuleDefV1> = Vec::new();
    for index in 0..DEPTH {
        let head = format!("p{index:06}");
        let body = if index + 1 < DEPTH {
            vec![positive(&format!("p{:06}", index + 1), &l[1])]
        } else {
            Vec::new()
        };
        rules.push(rule(&head, body, &l[0]));
    }
    let chain = RecursiveProgramV1::from_rules(rules);

    assert!(
        stratify(&chain, &receipt(50_000, 49_999, 199_998)).is_ok(),
        "the exact step charge admits"
    );
    let error =
        stratify(&chain, &receipt(50_000, 49_999, 199_997)).expect_err("one step less rejects");
    match error {
        LogicDiagnosticV1::RuleGraphBudgetExceeded {
            dimension,
            limit,
            span,
        } => {
            assert_eq!(dimension, RuleGraphBudgetDimensionV1::Steps);
            assert_eq!(limit, 199_997);
            assert_eq!(
                span, l[1],
                "the refused final charge is the last relaxation edge"
            );
        }
        other => panic!("expected a step-budget rejection, got {other:?}"),
    }
    let proof: StratificationProofV1 = stratify(&chain, &receipt(50_000, 49_999, 199_998))
        .expect("the exact charge admits a second time");
    assert_eq!(proof.components().len(), DEPTH);
}
