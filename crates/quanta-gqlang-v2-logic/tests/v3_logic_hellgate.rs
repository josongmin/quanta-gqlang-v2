#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed indices/variants and panic on unexpected shapes"
)]
//! Hellgate (fail-closed) rails for the logic / recursion layer (ticket 05).
//!
//! syntax->sema->ir path pinned: shared HIR rule shapes + binder-resolved body
//! dependencies -> logic stratification / aggregate / witness legality. Each
//! test asserts the ACTUAL typed [`quanta_gqlang_v2_logic::LogicDiagnosticV1`]
//! code + message, not merely that an error occurred. These rails fail RED
//! before the logic impl exists and prove the D10/D27/D31 "no silent downgrade"
//! contract.

use quanta_gqlang_v2_hir::SpanV1;
use quanta_gqlang_v2_logic::{
    BodyLiteralV1, DedupIdentityV1, LogicDiagnosticV1, RecursiveAggregateMatrixV1,
    RecursiveAggregateV1, RecursiveProgramV1, RelationShapeV1, RuleDefV1, UnboundedWitnessReasonV1,
    WitnessContractV1, WitnessRequestV1, WitnessTieBreakV1,
};

fn sp(a: usize, b: usize) -> SpanV1 {
    SpanV1::new(a, b)
}

// Hellgate 3: recursion through negation (unstratifiable) rejects with a typed
// diagnostic. `win(x) :- move(x, y), not win(y)` is the classic unstratifiable
// game: `win` depends negatively on itself.
#[test]
fn negation_cycle_rejects_with_typed_code() {
    let win = RuleDefV1::new(
        "win",
        vec![
            BodyLiteralV1::positive("move"),
            BodyLiteralV1::negated("win"),
        ],
        sp(0, 40),
    );
    let program = RecursiveProgramV1::from_rules(vec![win]);

    let err = program.stratify().expect_err("negation cycle must reject");
    assert_eq!(err.code(), "logic.negation_cycle");
    assert_eq!(
        err.message(),
        "recursion through negation is not stratifiable: cycle [win]"
    );
    assert_eq!(err.span(), sp(0, 40));
    match err {
        LogicDiagnosticV1::NegationCycle { cycle, .. } => {
            assert_eq!(cycle, vec!["win".to_owned()]);
        }
        other => panic!("expected NegationCycle, got {other:?}"),
    }
}

// Mutual negation cycle across two predicates also rejects, with a sorted cycle
// witness for determinism.
#[test]
fn mutual_negation_cycle_rejects_with_sorted_cycle() {
    let p = RuleDefV1::new("p", vec![BodyLiteralV1::negated("q")], sp(0, 10));
    let q = RuleDefV1::new("q", vec![BodyLiteralV1::positive("p")], sp(11, 20));
    let program = RecursiveProgramV1::from_rules(vec![p, q]);

    let err = program
        .stratify()
        .expect_err("mutual negation cycle rejects");
    assert_eq!(err.code(), "logic.negation_cycle");
    match err {
        LogicDiagnosticV1::NegationCycle { cycle, .. } => {
            assert_eq!(cycle, vec!["p".to_owned(), "q".to_owned()]);
        }
        other => panic!("expected NegationCycle, got {other:?}"),
    }
}

// Hellgate 4 (reject side): a recursive aggregate NOT on the admit matrix
// rejects with a typed diagnostic.
#[test]
fn recursive_aggregate_count_rejects_with_typed_code() {
    let matrix = RecursiveAggregateMatrixV1::new();
    let request = RecursiveAggregateV1 {
        function: "count".to_owned(),
        predicate: "reaches".to_owned(),
        recursive: true,
        span: sp(5, 25),
    };

    let err = matrix.check(&request).expect_err("recursive count rejects");
    assert_eq!(err.code(), "logic.recursive_aggregate");
    assert_eq!(
        err.message(),
        "recursive aggregate `count` over recursive predicate `reaches` is not admitted"
    );
    assert_eq!(err.span(), sp(5, 25));
}

// Hellgate 2: an unbounded witness family (no max cap) rejects rather than
// streaming infinitely.
#[test]
fn unbounded_witness_missing_cap_rejects_with_typed_code() {
    let request = WitnessRequestV1 {
        shape: RelationShapeV1::Cyclic,
        dedup: DedupIdentityV1::NodeSequence,
        max_witnesses: None,
        max_path_length: Some(8),
        tie_break: WitnessTieBreakV1::ShortestThenLexical,
        span: sp(3, 30),
    };

    let err = WitnessContractV1::bind(&request).expect_err("unbounded witness rejects");
    assert_eq!(err.code(), "logic.unbounded_witness.missing_max_witnesses");
    assert_eq!(
        err.message(),
        "unbounded witness materialization: a finite max-witness cap is required"
    );
    assert_eq!(err.span(), sp(3, 30));
    match err {
        LogicDiagnosticV1::UnboundedWitness { reason, .. } => {
            assert_eq!(reason, UnboundedWitnessReasonV1::MissingMaxWitnesses);
        }
        other => panic!("expected UnboundedWitness, got {other:?}"),
    }
}

// A cyclic relation with a cap but NO path-length bound is still unbounded:
// enumeration can revisit nodes forever. Rejects with the path-bound reason.
#[test]
fn cyclic_witness_without_path_bound_rejects() {
    let request = WitnessRequestV1 {
        shape: RelationShapeV1::Cyclic,
        dedup: DedupIdentityV1::NodeSequence,
        max_witnesses: Some(5),
        max_path_length: None,
        tie_break: WitnessTieBreakV1::ShortestThenLexical,
        span: sp(0, 12),
    };

    let err = WitnessContractV1::bind(&request).expect_err("cyclic without path bound rejects");
    assert_eq!(
        err.code(),
        "logic.unbounded_witness.missing_path_length_bound"
    );
    assert_eq!(
        err.message(),
        "unbounded witness materialization: a cyclic relation requires a path-length bound"
    );
}

// A zero max-witness cap is degenerate and rejects (it can never materialize a
// witness).
#[test]
fn zero_witness_cap_rejects() {
    let request = WitnessRequestV1 {
        shape: RelationShapeV1::Acyclic,
        dedup: DedupIdentityV1::Endpoints,
        max_witnesses: Some(0),
        max_path_length: None,
        tie_break: WitnessTieBreakV1::Lexical,
        span: sp(7, 9),
    };

    let err = WitnessContractV1::bind(&request).expect_err("zero cap rejects");
    assert_eq!(err.code(), "logic.unbounded_witness.zero_max_witnesses");
    assert_eq!(
        err.message(),
        "ill-formed witness bound: max-witness cap of zero can never materialize a witness"
    );
}

// An unknown source quantifier keyword rejects rather than silently lowering.
#[test]
fn unknown_quantifier_rejects() {
    use quanta_gqlang_v2_logic::ExistencePolarityV1;
    assert!(ExistencePolarityV1::from_keyword("some").is_none());
}

// A recursive aggregate inside its own recursion is non-monotone and rejects via
// the stratifier (distinct from the matrix-level reject).
#[test]
fn aggregated_self_recursion_rejects_non_monotone() {
    let rank = RuleDefV1::new("rank", vec![BodyLiteralV1::aggregated("rank")], sp(0, 33));
    let program = RecursiveProgramV1::from_rules(vec![rank]);

    let err = program
        .stratify()
        .expect_err("aggregated self-recursion rejects");
    assert_eq!(
        err.code(),
        "logic.non_monotone_recursion.aggregated_recursion"
    );
    assert_eq!(
        err.message(),
        "non-monotone recursion: predicate `rank` aggregates itself inside its own recursion"
    );
}
