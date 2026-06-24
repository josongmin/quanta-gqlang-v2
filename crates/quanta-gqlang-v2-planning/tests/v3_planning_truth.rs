#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed shapes and panic on unexpected variants"
)]
//! v3 planning truth/evidence/effect + result-kind + cost rails (ticket 07).
//!
//! Pins the syntax->sema->ir path at the PLANNING seam: a NORMALIZED algebra
//! (ticket 04) / logic (ticket 05) shape lowers into a host-agnostic
//! [`PublicPlanV1`]. These rails assert the POSITIVE classifications — every one
//! of the five fixed cost fields is populated with a typed class value, result
//! kinds and witness-selection policies are explicit, and multiplicity/order is
//! derived from the algebra cardinality (`D6`/`D20`/`D26`/`D32`).

use quanta_gqlang_v2_algebra::{
    AggregateExprV1, AggregateV1, AlgebraNodeV1, AlgebraOpV1, ColumnProvenanceV1, ColumnV1,
    GroupV1, ScanV1, SortDirectionV1, SortKeyV1, SortV1,
};
use quanta_gqlang_v2_hir::{HirNodeIdV1, LiteralV1, SpanV1, ValueExprHirV1, ValueExprKindV1};
use quanta_gqlang_v2_logic::{
    DedupIdentityV1, RelationShapeV1, WitnessContractV1, WitnessRequestV1, WitnessTieBreakV1,
};
use quanta_gqlang_v2_planning::{
    produce_plan, CardinalityClassV1, EffectClassV1, EvidenceRequirementV1, OrderingContractV1,
    PlanRequestV1, RecursionRiskClassV1, RecursionShapeV1, RenderRequirementV1,
    RequestedResultKindV1, ResultKindV1, TruthLaneV1, WitnessExplosionRiskClassV1,
    WitnessSelectionPolicyV1, WitnessShapeV1,
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

fn lit_expr(n: u32) -> ValueExprHirV1 {
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

/// group + aggregate + sort -> all five cost fields are populated with typed
/// class values, result-kind table, ordering is semantic (sequence).
#[test]
fn group_aggregate_sort_populates_all_five_cost_fields() {
    let base = scan("users", CardinalityV1::Bag);
    let group = AlgebraNodeV1::new(
        id(2),
        span(),
        vec![col("k")],
        CardinalityV1::Set,
        AlgebraOpV1::Group(GroupV1 {
            input: Box::new(base),
            keys: vec![lit_expr(10)],
        }),
    );
    let aggregate = AlgebraNodeV1::new(
        id(3),
        span(),
        vec![col("k"), col("n")],
        CardinalityV1::Set,
        AlgebraOpV1::Aggregate(AggregateV1 {
            input: Box::new(group),
            aggregates: vec![AggregateExprV1 {
                function: "count".to_owned(),
                origin: id(11),
                args: vec![],
                distinct: false,
            }],
        }),
    );
    let sort = AlgebraNodeV1::new(
        id(4),
        span(),
        vec![col("k"), col("n")],
        CardinalityV1::Sequence,
        AlgebraOpV1::Sort(SortV1 {
            input: Box::new(aggregate),
            keys: vec![SortKeyV1 {
                expr: lit_expr(12),
                direction: SortDirectionV1::Descending,
            }],
        }),
    );

    let request = PlanRequestV1 {
        root: sort,
        recursion: RecursionShapeV1::None,
        witness: None,
        result_kind: RequestedResultKindV1::Table,
        effect: EffectClassV1::ReadOnly,
        truth_lane: TruthLaneV1::Deterministic,
    };

    let plan = produce_plan(&request).expect("group+aggregate+sort plans");

    // All five fixed cost fields carry typed class values.
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::LinearInInput
    );
    assert_eq!(plan.cost.recursion_risk_class, RecursionRiskClassV1::None);
    assert_eq!(
        plan.cost.witness_explosion_risk_class,
        WitnessExplosionRiskClassV1::None
    );
    assert_eq!(plan.cost.ordering_contract, OrderingContractV1::Ordered);
    assert_eq!(plan.cost.unsupported_reason_code, None);

    // Result kind + ordering contract.
    assert_eq!(plan.result_kind, ResultKindV1::Table);
    assert_eq!(plan.render_requirement, RenderRequirementV1::RowsOnly);
    assert!(plan.multiplicity_order.is_ordered());

    // Capabilities derived from the normalized shape.
    use quanta_gqlang_v2_planning::CapabilityV1;
    assert!(plan.capabilities.contains(&CapabilityV1::Grouping));
    assert!(plan.capabilities.contains(&CapabilityV1::Aggregation));
    assert!(plan.capabilities.contains(&CapabilityV1::Ordering));
    assert!(plan.capabilities.contains(&CapabilityV1::RelationScan));
}

/// A SCALAR aggregate (`select count(*) from users` with NO grouping keys)
/// collapses all input rows to exactly one output row. Its estimated cardinality
/// class must be `ScalarOrSingleton`, NOT the `LinearInInput` of the underlying
/// scan. The scalar shape is an `Aggregate` over a `Group` with EMPTY keys
/// (`D24`: frontend-owned cost signals must be accurate).
#[test]
fn scalar_aggregate_with_no_grouping_keys_is_scalar_or_singleton() {
    let base = scan("users", CardinalityV1::Bag);
    let group = AlgebraNodeV1::new(
        id(2),
        span(),
        vec![],
        CardinalityV1::Set,
        AlgebraOpV1::Group(GroupV1 {
            input: Box::new(base),
            keys: vec![],
        }),
    );
    let aggregate = AlgebraNodeV1::new(
        id(3),
        span(),
        vec![col("n")],
        CardinalityV1::Set,
        AlgebraOpV1::Aggregate(AggregateV1 {
            input: Box::new(group),
            aggregates: vec![AggregateExprV1 {
                function: "count".to_owned(),
                origin: id(11),
                args: vec![],
                distinct: false,
            }],
        }),
    );

    let request = PlanRequestV1 {
        root: aggregate,
        recursion: RecursionShapeV1::None,
        witness: None,
        result_kind: RequestedResultKindV1::Table,
        effect: EffectClassV1::ReadOnly,
        truth_lane: TruthLaneV1::Deterministic,
    };

    let plan = produce_plan(&request).expect("scalar aggregate plans");

    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::ScalarOrSingleton
    );
}

/// A recursive/witness logic shape -> result-kind path/witness with an EXPLICIT
/// witness-selection policy and a closure cardinality class.
#[test]
fn recursive_witness_shape_is_path_witness_with_explicit_selection() {
    // Build an admitted positive-recursion proof from the logic crate.
    use quanta_gqlang_v2_logic::{BodyLiteralV1, RecursiveProgramV1, RuleDefV1};
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
    let proof = program.stratify().expect("positive recursion admits");

    let witness = WitnessContractV1::bind(&WitnessRequestV1 {
        shape: RelationShapeV1::Cyclic,
        dedup: DedupIdentityV1::NodeSequence,
        max_witnesses: Some(8),
        max_path_length: Some(16),
        tie_break: WitnessTieBreakV1::ShortestThenLexical,
        span: span(),
    })
    .expect("bounded witness contract");

    let request = PlanRequestV1 {
        root: scan("reach", CardinalityV1::Set),
        recursion: RecursionShapeV1::Fixpoint(proof),
        witness: Some(WitnessShapeV1 {
            contract: witness,
            selection: WitnessSelectionPolicyV1::CanonicalOne,
        }),
        result_kind: RequestedResultKindV1::PathWitness,
        effect: EffectClassV1::ReadOnly,
        truth_lane: TruthLaneV1::Deterministic,
    };

    let plan = produce_plan(&request).expect("recursive witness plans");

    assert_eq!(plan.result_kind, ResultKindV1::PathWitness);
    assert_eq!(plan.render_requirement, RenderRequirementV1::WitnessPayload);
    assert_eq!(plan.evidence, EvidenceRequirementV1::WitnessPayload);
    assert_eq!(
        plan.witness_selection,
        Some(WitnessSelectionPolicyV1::CanonicalOne)
    );

    // Recursion + cyclic witness drive the risk classes.
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::Closure
    );
    assert_eq!(
        plan.cost.recursion_risk_class,
        RecursionRiskClassV1::BoundDependent
    );
    assert_eq!(
        plan.cost.witness_explosion_risk_class,
        WitnessExplosionRiskClassV1::CapBounded
    );

    use quanta_gqlang_v2_planning::CapabilityV1;
    assert!(plan.capabilities.contains(&CapabilityV1::RecursiveClosure));
    assert!(plan
        .capabilities
        .contains(&CapabilityV1::WitnessMaterialization));
}

/// A proof-confirmed truth lane forces a proof-tree evidence requirement and is
/// the ONLY lane that may carry final policy truth.
#[test]
fn proof_confirmed_lane_requires_proof_tree_and_is_final_capable() {
    let request = PlanRequestV1 {
        root: scan("findings", CardinalityV1::Set),
        recursion: RecursionShapeV1::None,
        witness: None,
        result_kind: RequestedResultKindV1::ProofExplain,
        effect: EffectClassV1::ProofEvidenceOnly,
        truth_lane: TruthLaneV1::ProofConfirmed,
    };
    let plan = produce_plan(&request).expect("proof-confirmed plans");
    assert_eq!(plan.evidence, EvidenceRequirementV1::ProofTree);
    assert!(plan.truth_lane.is_final_policy_capable());
    assert!(!TruthLaneV1::Candidate.is_final_policy_capable());
    assert!(!TruthLaneV1::Deterministic.is_final_policy_capable());
}

/// An unordered (set) result keeps an unordered contract: the language does not
/// pretend an order exists (`D26`).
#[test]
fn unordered_result_stays_unordered() {
    let request = PlanRequestV1 {
        root: scan("users", CardinalityV1::Set),
        recursion: RecursionShapeV1::None,
        witness: None,
        result_kind: RequestedResultKindV1::Table,
        effect: EffectClassV1::ReadOnly,
        truth_lane: TruthLaneV1::Deterministic,
    };
    let plan = produce_plan(&request).expect("set table plans");
    assert_eq!(plan.cost.ordering_contract, OrderingContractV1::Unordered);
    assert!(!plan.multiplicity_order.is_ordered());
    assert!(plan.multiplicity_order.eliminates_duplicates());
}
