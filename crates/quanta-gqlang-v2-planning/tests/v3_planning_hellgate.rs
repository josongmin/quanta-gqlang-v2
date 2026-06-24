#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed shapes and panic on unexpected variants"
)]
//! v3 planning hellgate: fail-closed unsupported construct MIXES (ticket 07,
//! decisions `D10` / `D16`).
//!
//! The planner consumes already-normalized algebra/logic, so what it rejects is a
//! COMBINATION of otherwise-legal constructs that has no coherent
//! truth/effect/result-kind classification. Each test asserts the ACTUAL typed
//! diagnostic code / message / `unsupported_reason_code`, not merely that an
//! error occurred.

use quanta_gqlang_v2_algebra::{
    AlgebraNodeV1, AlgebraOpV1, ColumnProvenanceV1, ColumnV1, ProjectV1, ScanV1, SortDirectionV1,
    SortKeyV1, SortV1,
};
use quanta_gqlang_v2_hir::{HirNodeIdV1, LiteralV1, SpanV1, ValueExprHirV1, ValueExprKindV1};
use quanta_gqlang_v2_logic::{
    BodyLiteralV1, DedupIdentityV1, RecursiveProgramV1, RelationShapeV1, RuleDefV1,
    WitnessContractV1, WitnessRequestV1, WitnessTieBreakV1,
};
use quanta_gqlang_v2_planning::{
    produce_plan, EffectClassV1, PlanRequestV1, PlanningDiagnosticV1, RecursionShapeV1,
    RequestedResultKindV1, TruthLaneV1, UnsupportedMixReasonV1, WitnessSelectionPolicyV1,
    WitnessShapeV1,
};
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

fn scan(name: &str, cardinality: CardinalityV1) -> AlgebraNodeV1 {
    AlgebraNodeV1::new(
        id(1),
        span(),
        vec![col("x")],
        cardinality,
        AlgebraOpV1::Scan(ScanV1 {
            relation: name.to_owned(),
            alias: None,
        }),
    )
}

fn fixpoint_proof() -> RecursionShapeV1 {
    let program = RecursiveProgramV1::from_rules(vec![
        RuleDefV1::new("reach", vec![BodyLiteralV1::positive("edge")], span()),
        RuleDefV1::new(
            "reach",
            vec![
                BodyLiteralV1::positive("edge"),
                BodyLiteralV1::positive("reach"),
            ],
            span(),
        ),
    ]);
    RecursionShapeV1::Fixpoint(program.stratify().expect("positive recursion admits"))
}

fn bounded_witness() -> WitnessShapeV1 {
    let contract = WitnessContractV1::bind(&WitnessRequestV1 {
        shape: RelationShapeV1::Acyclic,
        dedup: DedupIdentityV1::Endpoints,
        max_witnesses: Some(4),
        max_path_length: None,
        tie_break: WitnessTieBreakV1::Lexical,
        span: span(),
    })
    .expect("bounded witness");
    WitnessShapeV1 {
        contract,
        selection: WitnessSelectionPolicyV1::AllWitness,
    }
}

/// proof/explain result + export effect -> rejects with explain_with_export.
#[test]
fn explain_with_export_rejects() {
    let request = PlanRequestV1 {
        root: scan("findings", CardinalityV1::Set),
        recursion: RecursionShapeV1::None,
        witness: None,
        result_kind: RequestedResultKindV1::ProofExplain,
        effect: EffectClassV1::ExportOnly,
        truth_lane: TruthLaneV1::Deterministic,
    };
    let err = produce_plan(&request).expect_err("explain+export must reject");
    assert_eq!(
        err,
        PlanningDiagnosticV1::UnsupportedMix {
            reason: UnsupportedMixReasonV1::ExplainWithExport,
            span: span(),
        }
    );
    assert_eq!(err.code(), "planning.unsupported_mix.explain_with_export");
    assert_eq!(err.unsupported_reason_code(), "explain_with_export");
    assert_eq!(
        err.message(),
        "unsupported construct mix: a proof/explain result cannot also export candidates"
    );
}

/// export effect + proof-confirmed lane -> export_with_proof_only_lane.
#[test]
fn export_with_proof_only_lane_rejects() {
    let request = PlanRequestV1 {
        root: scan("candidates", CardinalityV1::Set),
        recursion: RecursionShapeV1::None,
        witness: None,
        result_kind: RequestedResultKindV1::Table,
        effect: EffectClassV1::ExportOnly,
        truth_lane: TruthLaneV1::ProofConfirmed,
    };
    let err = produce_plan(&request).expect_err("export+proof-only lane must reject");
    assert_eq!(
        err.code(),
        "planning.unsupported_mix.export_with_proof_only_lane"
    );
    assert_eq!(err.unsupported_reason_code(), "export_with_proof_only_lane");
    assert_eq!(
        err.message(),
        "unsupported construct mix: export effect cannot combine with a proof/evidence-only lane"
    );
}

/// mutation-forbidden effect + witness materialization -> mutation_with_witness.
#[test]
fn mutation_with_witness_rejects() {
    let request = PlanRequestV1 {
        root: scan("graph", CardinalityV1::Set),
        recursion: RecursionShapeV1::None,
        witness: Some(bounded_witness()),
        result_kind: RequestedResultKindV1::PathWitness,
        effect: EffectClassV1::MutationForbidden,
        truth_lane: TruthLaneV1::Deterministic,
    };
    let err = produce_plan(&request).expect_err("mutation+witness must reject");
    assert_eq!(err.code(), "planning.unsupported_mix.mutation_with_witness");
    assert_eq!(err.unsupported_reason_code(), "mutation_with_witness");
    assert_eq!(
        err.message(),
        "unsupported construct mix: mutation effect cannot combine with witness/path \
         materialization"
    );
}

/// recursive closure + explicit total-order (sort) at the root ->
/// ordered_recursive_closure.
#[test]
fn ordered_recursive_closure_rejects() {
    let sorted = AlgebraNodeV1::new(
        id(2),
        span(),
        vec![col("x")],
        CardinalityV1::Sequence,
        AlgebraOpV1::Sort(SortV1 {
            input: Box::new(scan("reach", CardinalityV1::Set)),
            keys: vec![SortKeyV1 {
                expr: lit(20),
                direction: SortDirectionV1::Ascending,
            }],
        }),
    );
    let request = PlanRequestV1 {
        root: sorted,
        recursion: fixpoint_proof(),
        witness: None,
        result_kind: RequestedResultKindV1::Table,
        effect: EffectClassV1::ReadOnly,
        truth_lane: TruthLaneV1::Deterministic,
    };
    let err = produce_plan(&request).expect_err("ordered recursive closure must reject");
    assert_eq!(
        err.code(),
        "planning.unsupported_mix.ordered_recursive_closure"
    );
    assert_eq!(err.unsupported_reason_code(), "ordered_recursive_closure");
    assert_eq!(
        err.message(),
        "unsupported construct mix: a recursive closure result has no total-order rendering \
         contract"
    );
}

/// recursive closure whose ORDERED (Sequence) result is wrapped by a non-Sort
/// root operator (e.g. `... | sort by id | project {id}`) must still reject:
/// the ordered-recursive incoherence is a property of the result CARDINALITY,
/// not of whether a Sort happens to be the immediate root operator.
#[test]
fn ordered_recursive_closure_under_project_rejects() {
    // Mirrors `from mytable | sort by id | project {id}` over a recursive rule:
    // the Sort makes the result a Sequence, then a Project re-shapes columns
    // while PRESERVING that ordered cardinality. The root op is Project, not Sort.
    let sorted = AlgebraNodeV1::new(
        id(2),
        span(),
        vec![col("x")],
        CardinalityV1::Sequence,
        AlgebraOpV1::Sort(SortV1 {
            input: Box::new(scan("reach", CardinalityV1::Set)),
            keys: vec![SortKeyV1 {
                expr: lit(20),
                direction: SortDirectionV1::Ascending,
            }],
        }),
    );
    let projected = AlgebraNodeV1::new(
        id(3),
        span(),
        vec![col("x")],
        CardinalityV1::Sequence,
        AlgebraOpV1::Project(ProjectV1 {
            input: Box::new(sorted),
            items: vec![lit(30)],
        }),
    );
    let request = PlanRequestV1 {
        root: projected,
        recursion: fixpoint_proof(),
        witness: None,
        result_kind: RequestedResultKindV1::Table,
        effect: EffectClassV1::ReadOnly,
        truth_lane: TruthLaneV1::Deterministic,
    };
    let err = produce_plan(&request)
        .expect_err("ordered recursive closure under a project root must reject");
    assert_eq!(
        err.code(),
        "planning.unsupported_mix.ordered_recursive_closure"
    );
    assert_eq!(err.unsupported_reason_code(), "ordered_recursive_closure");
}

/// path_witness result + missing witness contract/selection ->
/// path_witness_without_witness.
#[test]
fn pathwitness_without_witness_rejects() {
    let request = PlanRequestV1 {
        root: scan("graph", CardinalityV1::Set),
        recursion: RecursionShapeV1::None,
        witness: None,
        result_kind: RequestedResultKindV1::PathWitness,
        effect: EffectClassV1::ReadOnly,
        truth_lane: TruthLaneV1::Deterministic,
    };
    let err = produce_plan(&request).expect_err("path_witness without witness must reject");
    assert_eq!(
        err.code(),
        "planning.unsupported_mix.path_witness_without_witness"
    );
    assert_eq!(
        err.unsupported_reason_code(),
        "path_witness_without_witness"
    );
    assert_eq!(
        err.message(),
        "unsupported construct mix: a path-witness result requires an explicit witness contract \
         and selection policy"
    );
}
