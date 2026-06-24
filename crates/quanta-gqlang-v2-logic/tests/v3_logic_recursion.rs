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

trait RecursiveCountExt {
    fn recursive_count_is_zero(&self) -> bool;
}

impl RecursiveCountExt for quanta_gqlang_v2_logic::StratificationProofV1 {
    fn recursive_count_is_zero(&self) -> bool {
        self.recursive.is_empty()
    }
}
