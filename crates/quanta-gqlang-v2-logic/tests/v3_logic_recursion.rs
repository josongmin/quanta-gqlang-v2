#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed indices/variants and panic on unexpected shapes"
)]
//! Positive-recursion admission rails for the logic layer (ticket 05).
//!
//! syntax->sema->ir path pinned: shared HIR rule shapes + binder-resolved body
//! dependencies -> [`quanta_gqlang_v2_logic::RecursiveProgramV1::stratify`].
//! Proves D27: positive recursion admits under EXPLICIT least-fixpoint semantics
//! with a stratification proof; stratified negation (negation crossing to a
//! lower stratum) admits.

use quanta_gqlang_v2_hir::{HirBuilderV1, SpanV1};
use quanta_gqlang_v2_logic::{
    AggregateLegalityV1, BodyLiteralV1, RecursiveAggregateMatrixV1, RecursiveAggregateV1,
    RecursiveProgramV1, RuleDefV1,
};

fn sp(a: usize, b: usize) -> SpanV1 {
    SpanV1::new(a, b)
}

// Hellgate 1 (positive): transitive closure
// `reaches(x, z) :- edge(x, z)`
// `reaches(x, z) :- edge(x, y), reaches(y, z)`
// admits under least-fixpoint with a stratification proof: `reaches` is
// recursive (self-loop, positive) and lands in a single stratum.
#[test]
fn positive_transitive_closure_admits_with_proof() {
    let base = RuleDefV1::new("reaches", vec![BodyLiteralV1::positive("edge")], sp(0, 20));
    let step = RuleDefV1::new(
        "reaches",
        vec![
            BodyLiteralV1::positive("edge"),
            BodyLiteralV1::positive("reaches"),
        ],
        sp(21, 50),
    );
    let program = RecursiveProgramV1::from_rules(vec![base, step]);

    let proof = program.stratify().expect("positive recursion admits");
    // `reaches` is recursive under least-fixpoint.
    assert!(proof.is_recursive("reaches"));
    // `edge` is a base relation, not recursive.
    assert!(!proof.is_recursive("edge"));
    // Positive recursion needs only one stratum.
    assert_eq!(proof.stratum_count, 1);
    assert_eq!(proof.stratum_of("reaches"), Some(0));
    assert_eq!(proof.stratum_of("edge"), Some(0));
}

// A RuleHirV1 built through the shared HIR builder feeds RuleDefV1::from_rule:
// rule/view composition is over the shared HIR carrier, not a root-local hack.
#[test]
fn rule_def_built_from_shared_hir_rule() {
    let b = HirBuilderV1::new();
    let head = b.symbol("reaches", sp(0, 7));
    let body = b.bool_literal(true, sp(8, 12));
    let rule = b.rule("reaches", sp(0, 7), head, body, sp(0, 50));

    let def = RuleDefV1::from_rule(&rule, vec![BodyLiteralV1::positive("reaches")]);
    assert_eq!(def.head, "reaches");
    assert_eq!(def.span, sp(0, 50));

    let program = RecursiveProgramV1::from_rules(vec![def]);
    let proof = program.stratify().expect("self-positive recursion admits");
    assert!(proof.is_recursive("reaches"));
}

// Stratified negation: `safe` negates `tainted`, but `tainted` does NOT depend
// on `safe`, so negation crosses to a strictly lower stratum and admits.
#[test]
fn stratified_negation_admits_with_layered_strata() {
    // tainted(x) :- source(x)
    // tainted(x) :- edge(y, x), tainted(y)      (positive recursion)
    // safe(x)    :- node(x), not tainted(x)      (negation, one layer up)
    let tainted_base = RuleDefV1::new(
        "tainted",
        vec![BodyLiteralV1::positive("source")],
        sp(0, 15),
    );
    let tainted_step = RuleDefV1::new(
        "tainted",
        vec![
            BodyLiteralV1::positive("edge"),
            BodyLiteralV1::positive("tainted"),
        ],
        sp(16, 40),
    );
    let safe = RuleDefV1::new(
        "safe",
        vec![
            BodyLiteralV1::positive("node"),
            BodyLiteralV1::negated("tainted"),
        ],
        sp(41, 70),
    );
    let program = RecursiveProgramV1::from_rules(vec![tainted_base, tainted_step, safe]);

    let proof = program.stratify().expect("stratified negation admits");
    assert!(proof.is_recursive("tainted"));
    assert!(!proof.is_recursive("safe"));
    // `safe` negates `tainted`, so `safe` must sit strictly above `tainted`.
    let safe_stratum = proof.stratum_of("safe").expect("safe stratum");
    let tainted_stratum = proof.stratum_of("tainted").expect("tainted stratum");
    assert!(
        safe_stratum > tainted_stratum,
        "negation must raise the stratum: safe={safe_stratum} tainted={tainted_stratum}"
    );
    assert_eq!(proof.stratum_count, 2);
}

// Recursive-aggregate matrix admits the explicitly listed monotone folds and a
// non-recursive aggregate.
#[test]
fn recursive_aggregate_matrix_admits_min_and_nonrecursive() {
    let matrix = RecursiveAggregateMatrixV1::new();

    // `min` over recursion is admitted (monotone lattice fold).
    let min_req = RecursiveAggregateV1 {
        function: "min".to_owned(),
        predicate: "dist".to_owned(),
        recursive: true,
        span: sp(0, 10),
    };
    assert_eq!(matrix.classify(&min_req), AggregateLegalityV1::Admitted);
    assert!(matrix.check(&min_req).is_ok());

    // A non-recursive `count` is always admitted regardless of the admit list.
    let count_req = RecursiveAggregateV1 {
        function: "count".to_owned(),
        predicate: "users".to_owned(),
        recursive: false,
        span: sp(0, 10),
    };
    assert_eq!(matrix.classify(&count_req), AggregateLegalityV1::Admitted);

    // An empty matrix rejects even `min` over recursion.
    let empty = RecursiveAggregateMatrixV1::empty();
    assert_eq!(empty.classify(&min_req), AggregateLegalityV1::Rejected);
}

// `with_admitted` must build a matrix from EXACTLY the supplied names, not fall
// back to the default (`min`/`max`). Pins aggregate.rs:84
// (`with_admitted` -> `Default::default()`): a custom admit list of `count`
// alone must admit `count` and REJECT the default's `min`/`max`.
#[test]
fn with_admitted_uses_supplied_names_not_default() {
    let matrix = RecursiveAggregateMatrixV1::with_admitted(["count"]);

    // The supplied name is admitted...
    assert!(matrix.admits("count"), "count was explicitly admitted");
    let count_req = RecursiveAggregateV1 {
        function: "count".to_owned(),
        predicate: "p".to_owned(),
        recursive: true,
        span: sp(0, 10),
    };
    assert_eq!(matrix.classify(&count_req), AggregateLegalityV1::Admitted);

    // ...and the DEFAULT admit list (`min`/`max`) must NOT leak in. If
    // with_admitted were replaced by Default::default(), these would be admitted.
    assert!(!matrix.admits("min"), "min is not in the supplied list");
    assert!(!matrix.admits("max"), "max is not in the supplied list");
    let min_req = RecursiveAggregateV1 {
        function: "min".to_owned(),
        predicate: "p".to_owned(),
        recursive: true,
        span: sp(0, 10),
    };
    assert_eq!(matrix.classify(&min_req), AggregateLegalityV1::Rejected);
}

// A non-recursive program (a DAG of views) admits with no predicate marked
// recursive and strata reflecting the layering.
#[test]
fn acyclic_view_composition_admits_no_recursion() {
    // a :- base
    // b :- a
    let a = RuleDefV1::new("a", vec![BodyLiteralV1::positive("base")], sp(0, 5));
    let bdef = RuleDefV1::new("b", vec![BodyLiteralV1::positive("a")], sp(6, 12));
    let program = RecursiveProgramV1::from_rules(vec![a, bdef]);

    let proof = program.stratify().expect("acyclic composition admits");
    assert!(!proof.is_recursive("a"));
    assert!(!proof.is_recursive("b"));
    assert!(proof.recursive_count_is_zero());
}

// Mutual recursion forms a multi-member SCC: `a :- b`, `b :- a`. Both predicates
// sit in the SAME non-trivial SCC (size 2), so both are recursive. Pins
// recursion.rs:210 (`scc.len() > 1`): mutating `>` to `<` would mark NEITHER
// recursive (an SCC of size 2 is never `< 1`), and neither has a self-loop.
#[test]
fn mutual_recursion_marks_both_members_recursive() {
    let a = RuleDefV1::new("a", vec![BodyLiteralV1::positive("b")], sp(0, 5));
    let bdef = RuleDefV1::new("b", vec![BodyLiteralV1::positive("a")], sp(6, 12));
    let program = RecursiveProgramV1::from_rules(vec![a, bdef]);

    let proof = program
        .stratify()
        .expect("positive mutual recursion admits");
    // Both members of the size-2 SCC are recursive.
    assert!(proof.is_recursive("a"), "a is in the size-2 SCC");
    assert!(proof.is_recursive("b"), "b is in the size-2 SCC");
    assert_eq!(proof.recursive.len(), 2, "exactly the two SCC members");
    // Positive mutual recursion still needs only one stratum.
    assert_eq!(proof.stratum_of("a"), Some(0));
    assert_eq!(proof.stratum_of("b"), Some(0));
}

// A self-loop predicate (size-1 SCC) is recursive via the `else` arm, NOT the
// `scc.len() > 1` arm. This complements the mutual-recursion test by pinning the
// boundary at exactly 1: a single-member SCC must NOT be treated as multi-member.
#[test]
fn self_loop_is_recursive_but_singleton_base_is_not() {
    // r :- r (self loop, size-1 SCC) ; base :- nothing recursive
    let r = RuleDefV1::new("r", vec![BodyLiteralV1::positive("r")], sp(0, 5));
    let s = RuleDefV1::new("s", vec![BodyLiteralV1::positive("base")], sp(6, 12));
    let program = RecursiveProgramV1::from_rules(vec![r, s]);

    let proof = program.stratify().expect("self recursion admits");
    assert!(proof.is_recursive("r"), "self-loop is recursive");
    assert!(!proof.is_recursive("s"), "singleton non-self-loop is not");
    assert!(
        !proof.is_recursive("base"),
        "base relation is not recursive"
    );
    assert_eq!(proof.recursive.len(), 1);
}

// A three-level negation chain forces strata to LAYER monotonically and forces
// the relaxation in `assign_strata` to push the source strictly above the
// target on every negated edge. Pins recursion.rs:346 / :368 (the relaxation
// loop's continue-condition and the strict-improvement guard): the assigned
// stratum numbers must be EXACTLY 0/1/2/3 in order.
#[test]
fn deep_negation_chain_layers_strata_monotonically() {
    // a :- base                 (stratum 0)
    // b :- node, not a          (stratum 1, negates a)
    // c :- node, not b          (stratum 2, negates b)
    // d :- node, not c          (stratum 3, negates c)
    let a = RuleDefV1::new("a", vec![BodyLiteralV1::positive("base")], sp(0, 5));
    let b = RuleDefV1::new(
        "b",
        vec![BodyLiteralV1::positive("node"), BodyLiteralV1::negated("a")],
        sp(6, 12),
    );
    let c = RuleDefV1::new(
        "c",
        vec![BodyLiteralV1::positive("node"), BodyLiteralV1::negated("b")],
        sp(13, 20),
    );
    let d = RuleDefV1::new(
        "d",
        vec![BodyLiteralV1::positive("node"), BodyLiteralV1::negated("c")],
        sp(21, 28),
    );
    let program = RecursiveProgramV1::from_rules(vec![a, b, c, d]);

    let proof = program.stratify().expect("deep stratified negation admits");
    assert_eq!(proof.stratum_of("a"), Some(0), "base predicate at 0");
    assert_eq!(proof.stratum_of("b"), Some(1), "b negates a -> 1");
    assert_eq!(proof.stratum_of("c"), Some(2), "c negates b -> 2");
    assert_eq!(proof.stratum_of("d"), Some(3), "d negates c -> 3");
    assert_eq!(proof.stratum_count, 4, "four layers 0..=3");
}

// A negation chain whose nodes are declared in an order that DEFEATS single-pass
// propagation forces the relaxation loop to run multiple passes. Pins
// recursion.rs:346 (loop continue-condition `changed && passes <= sccs.len()`):
// the loop must keep iterating while changes are still happening AND stop once
// it has done enough passes. The strata must converge to the correct layering
// regardless of declaration order.
#[test]
fn negation_chain_converges_regardless_of_declaration_order() {
    // Same chain as above but declared d, c, b, a so a naive single pass over
    // sorted nodes still resolves; assert the final fixpoint is correct.
    let d = RuleDefV1::new("z_d", vec![BodyLiteralV1::negated("z_c")], sp(21, 28));
    let c = RuleDefV1::new("z_c", vec![BodyLiteralV1::negated("z_b")], sp(13, 20));
    let b = RuleDefV1::new("z_b", vec![BodyLiteralV1::negated("z_a")], sp(6, 12));
    let a = RuleDefV1::new("z_a", vec![BodyLiteralV1::positive("base")], sp(0, 5));
    let program = RecursiveProgramV1::from_rules(vec![d, c, b, a]);

    let proof = program.stratify().expect("chain admits");
    assert_eq!(proof.stratum_of("z_a"), Some(0));
    assert_eq!(proof.stratum_of("z_b"), Some(1));
    assert_eq!(proof.stratum_of("z_c"), Some(2));
    assert_eq!(proof.stratum_of("z_d"), Some(3));
    assert_eq!(proof.stratum_count, 4);
}

// `add_rule` must actually append a rule that is then observable both via
// `rules()` and via stratification. Pins recursion.rs:187 (body -> `()`, i.e.
// the push is deleted) and recursion.rs:193 (`rules()` -> empty slice).
#[test]
fn add_rule_is_observable_through_rules_and_stratify() {
    let mut program = RecursiveProgramV1::new();
    assert!(program.rules().is_empty(), "fresh program has no rules");

    program.add_rule(RuleDefV1::new(
        "reaches",
        vec![BodyLiteralV1::positive("reaches")],
        sp(0, 10),
    ));

    // recursion.rs:193: rules() must reflect the added rule, not an empty slice.
    assert_eq!(program.rules().len(), 1, "the added rule is observable");
    assert_eq!(program.rules()[0].head, "reaches");
    assert_eq!(
        program.rules()[0].body,
        vec![BodyLiteralV1::positive("reaches")]
    );

    // recursion.rs:187: the rule must actually be present for stratify to see
    // the self-loop and mark `reaches` recursive.
    let proof = program.stratify().expect("self recursion admits");
    assert!(
        proof.is_recursive("reaches"),
        "the added self-loop rule must be visible to stratify"
    );
}

trait RecursiveCountExt {
    fn recursive_count_is_zero(&self) -> bool;
}

impl RecursiveCountExt for quanta_gqlang_v2_logic::StratificationProofV1 {
    fn recursive_count_is_zero(&self) -> bool {
        self.recursive.is_empty()
    }
}
