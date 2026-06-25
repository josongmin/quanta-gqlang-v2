#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed shapes and panic on unexpected variants"
)]
//! Cardinality-class / recursion-risk / witness-risk derivation edge cases
//! (ticket 07, `D24`/`D26`/`D32`) plus the closed keyword sets of every cost,
//! effect, truth-lane, and result-kind class. These drive the uncovered
//! `classify_node` / `worst_class` / `recursion_risk` / `witness_risk` arms of
//! `plan.rs` and the `keyword()` accessors of the typed cost vocabulary.

use quanta_gqlang_v2_algebra::{
    AlgebraNodeV1, AlgebraOpV1, AntiJoinV1, ApplyV1, ColumnProvenanceV1, ColumnV1, FilterV1,
    JoinV1, LimitV1, ScanV1, SemiJoinV1, UnionV1,
};
use quanta_gqlang_v2_hir::{BoolExprHirV1, BoolExprKindV1, HirNodeIdV1, SpanV1};
use quanta_gqlang_v2_logic::{
    BodyLiteralV1, DedupIdentityV1, RecursiveProgramV1, RelationShapeV1, RuleDefV1,
    WitnessContractV1, WitnessRequestV1, WitnessTieBreakV1,
};
use quanta_gqlang_v2_planning::{
    produce_plan, CapabilityV1, CardinalityClassV1, EffectClassV1, EvidenceRequirementV1,
    OrderingContractV1, PlanRequestV1, RecursionRiskClassV1, RecursionShapeV1, RenderRequirementV1,
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

fn pred() -> Box<BoolExprHirV1> {
    Box::new(BoolExprHirV1::new(
        id(99),
        span(),
        BoolExprKindV1::Literal(true),
    ))
}

fn node(card: CardinalityV1, op: AlgebraOpV1) -> AlgebraNodeV1 {
    AlgebraNodeV1::new(id(1), span(), vec![col("x")], card, op)
}

fn scan(name: &str, card: CardinalityV1) -> AlgebraNodeV1 {
    node(
        card,
        AlgebraOpV1::Scan(ScanV1 {
            relation: name.to_owned(),
            alias: None,
        }),
    )
}

fn request(root: AlgebraNodeV1) -> PlanRequestV1 {
    PlanRequestV1 {
        root,
        recursion: RecursionShapeV1::None,
        witness: None,
        result_kind: RequestedResultKindV1::Table,
        effect: EffectClassV1::ReadOnly,
        truth_lane: TruthLaneV1::Deterministic,
    }
}

fn fixpoint_proof() -> quanta_gqlang_v2_logic::StratificationProofV1 {
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
    program.stratify().expect("positive recursion admits")
}

fn acyclic_witness(selection: WitnessSelectionPolicyV1) -> WitnessShapeV1 {
    let contract = WitnessContractV1::bind(&WitnessRequestV1 {
        shape: RelationShapeV1::Acyclic,
        dedup: DedupIdentityV1::Endpoints,
        max_witnesses: Some(4),
        max_path_length: None,
        tie_break: WitnessTieBreakV1::Lexical,
        span: span(),
    })
    .expect("bounded acyclic witness");
    WitnessShapeV1 {
        contract,
        selection,
    }
}

// ---- cardinality class -----------------------------------------------------

/// A `project` root is transparent: it forwards the underlying input's class.
#[test]
fn project_forwards_input_class() {
    use quanta_gqlang_v2_algebra::ProjectV1;
    use quanta_gqlang_v2_hir::{LiteralV1, ValueExprHirV1, ValueExprKindV1};
    let projected = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Project(ProjectV1 {
            input: Box::new(scan("users", CardinalityV1::Bag)),
            items: vec![ValueExprHirV1::new(
                id(2),
                span(),
                ValueExprKindV1::Literal(LiteralV1::Int("1".to_owned())),
            )],
        }),
    );
    let plan = produce_plan(&request(projected)).expect("project plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::LinearInInput
    );
}

/// A union whose right arm is a recursive-shaped closure-ranked multiplicative
/// join keeps the worst rank. (This drives the upper `rank` arms.)
#[test]
fn union_takes_multiplicative_over_bounded() {
    let bounded = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Limit(LimitV1 {
            input: Box::new(scan("a", CardinalityV1::Bag)),
            count: 3,
            offset: 0,
        }),
    );
    let multiplicative = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Join(JoinV1 {
            left: Box::new(scan("b", CardinalityV1::Bag)),
            right: Box::new(scan("c", CardinalityV1::Bag)),
            predicate: None,
        }),
    );
    let unioned = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Union(UnionV1 {
            left: Box::new(bounded),
            right: Box::new(multiplicative),
            distinct: false,
        }),
    );
    let plan = produce_plan(&request(unioned)).expect("union plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::Multiplicative
    );
}

// ---- result-kind / render / evidence arms via produce_plan -----------------

/// An `Alert` result-kind maps to alert / source-location render and (under a
/// non-final lane) a source-location evidence requirement.
#[test]
fn alert_result_kind_maps_to_source_location() {
    let mut req = request(scan("findings", CardinalityV1::Set));
    req.result_kind = RequestedResultKindV1::Alert;
    let plan = produce_plan(&req).expect("alert plans");
    assert_eq!(plan.result_kind, ResultKindV1::Alert);
    assert_eq!(plan.render_requirement, RenderRequirementV1::SourceLocation);
    assert_eq!(plan.evidence, EvidenceRequirementV1::SourceLocation);
}

/// A `ProofExplain` result-kind maps to proof-explain / provenance-tree render
/// and a proof-tree evidence requirement even under a deterministic lane.
#[test]
fn proof_explain_result_kind_maps_to_provenance_tree() {
    let mut req = request(scan("findings", CardinalityV1::Set));
    req.result_kind = RequestedResultKindV1::ProofExplain;
    let plan = produce_plan(&req).expect("proof-explain plans");
    assert_eq!(plan.result_kind, ResultKindV1::ProofExplain);
    assert_eq!(plan.render_requirement, RenderRequirementV1::ProvenanceTree);
    assert_eq!(plan.evidence, EvidenceRequirementV1::ProofTree);
}

/// A bare scan is linear in its single input.
#[test]
fn scan_root_is_linear_in_input() {
    let plan = produce_plan(&request(scan("users", CardinalityV1::Bag))).expect("scan plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::LinearInInput
    );
}

/// A `limit` root bounds the result regardless of the input class beneath it.
#[test]
fn limit_root_is_bounded() {
    let limited = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Limit(LimitV1 {
            input: Box::new(scan("users", CardinalityV1::Bag)),
            count: 10,
            offset: 0,
        }),
    );
    let plan = produce_plan(&request(limited)).expect("limit plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::Bounded
    );
}

/// A `filter` is transparent: it forwards the input's class (linear).
#[test]
fn filter_forwards_input_class() {
    let filtered = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Filter(FilterV1 {
            input: Box::new(scan("users", CardinalityV1::Bag)),
            predicate: pred(),
        }),
    );
    let plan = produce_plan(&request(filtered)).expect("filter plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::LinearInInput
    );
}

/// A join can multiply input sizes.
#[test]
fn join_root_is_multiplicative() {
    let joined = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Join(JoinV1 {
            left: Box::new(scan("a", CardinalityV1::Bag)),
            right: Box::new(scan("b", CardinalityV1::Bag)),
            predicate: Some(pred()),
        }),
    );
    let plan = produce_plan(&request(joined)).expect("join plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::Multiplicative
    );
}

/// An apply (correlated) is also multiplicative.
#[test]
fn apply_root_is_multiplicative() {
    let applied = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Apply(ApplyV1 {
            left: Box::new(scan("a", CardinalityV1::Bag)),
            right: Box::new(scan("b", CardinalityV1::Bag)),
            correlation: vec![],
        }),
    );
    let plan = produce_plan(&request(applied)).expect("apply plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::Multiplicative
    );
}

/// A semijoin's output is bounded by its LEFT arm: it forwards the left class.
#[test]
fn semijoin_forwards_left_class() {
    let semi = node(
        CardinalityV1::Bag,
        AlgebraOpV1::SemiJoin(SemiJoinV1 {
            left: Box::new(scan("a", CardinalityV1::Bag)),
            // Right arm is a join (multiplicative) but contributes no columns,
            // so the semijoin must NOT inherit the right arm's multiplicativity.
            right: Box::new(node(
                CardinalityV1::Bag,
                AlgebraOpV1::Join(JoinV1 {
                    left: Box::new(scan("b", CardinalityV1::Bag)),
                    right: Box::new(scan("c", CardinalityV1::Bag)),
                    predicate: None,
                }),
            )),
            predicate: None,
        }),
    );
    let plan = produce_plan(&request(semi)).expect("semijoin plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::LinearInInput
    );
}

/// An antijoin's output is likewise bounded by its left arm.
#[test]
fn antijoin_forwards_left_class() {
    let anti = node(
        CardinalityV1::Bag,
        AlgebraOpV1::AntiJoin(AntiJoinV1 {
            left: Box::new(scan("a", CardinalityV1::Bag)),
            right: Box::new(node(
                CardinalityV1::Bag,
                AlgebraOpV1::Join(JoinV1 {
                    left: Box::new(scan("b", CardinalityV1::Bag)),
                    right: Box::new(scan("c", CardinalityV1::Bag)),
                    predicate: None,
                }),
            )),
            predicate: None,
        }),
    );
    let plan = produce_plan(&request(anti)).expect("antijoin plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::LinearInInput
    );
}

/// A union takes the WORST (largest-risk) class of its two arms: a linear arm
/// unioned with a multiplicative arm is multiplicative.
#[test]
fn union_takes_worst_arm_class() {
    let multiplicative_arm = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Join(JoinV1 {
            left: Box::new(scan("b", CardinalityV1::Bag)),
            right: Box::new(scan("c", CardinalityV1::Bag)),
            predicate: None,
        }),
    );
    let unioned = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Union(UnionV1 {
            left: Box::new(scan("a", CardinalityV1::Bag)),
            right: Box::new(multiplicative_arm),
            distinct: false,
        }),
    );
    let plan = produce_plan(&request(unioned)).expect("union plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::Multiplicative
    );
}

/// A union of two linear arms stays linear: `worst_class` returns the LEFT arm
/// when the ranks are equal (the `>=` branch).
#[test]
fn union_of_equal_arms_stays_linear() {
    let unioned = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Union(UnionV1 {
            left: Box::new(scan("a", CardinalityV1::Bag)),
            right: Box::new(scan("b", CardinalityV1::Bag)),
            distinct: false,
        }),
    );
    let plan = produce_plan(&request(unioned)).expect("union plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::LinearInInput
    );
}

/// A union whose RIGHT arm is the larger-risk one exercises the `worst_class`
/// `else` branch (right > left).
#[test]
fn union_with_heavier_right_arm_takes_right() {
    let bounded_left = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Limit(LimitV1 {
            input: Box::new(scan("a", CardinalityV1::Bag)),
            count: 5,
            offset: 0,
        }),
    );
    let unioned = node(
        CardinalityV1::Bag,
        AlgebraOpV1::Union(UnionV1 {
            left: Box::new(bounded_left),
            // linear ranks higher than bounded
            right: Box::new(scan("b", CardinalityV1::Bag)),
            distinct: false,
        }),
    );
    let plan = produce_plan(&request(unioned)).expect("union plans");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::LinearInInput
    );
}

/// A fixpoint recursion forces the `Closure` class regardless of the root op.
#[test]
fn fixpoint_recursion_is_closure_class() {
    let mut req = request(scan("reach", CardinalityV1::Set));
    req.recursion = RecursionShapeV1::Fixpoint(fixpoint_proof());
    let plan = produce_plan(&req).expect("recursive plan");
    assert_eq!(
        plan.cost.estimated_cardinality_class,
        CardinalityClassV1::Closure
    );
    assert!(plan.capabilities.contains(&CapabilityV1::RecursiveClosure));
}

// ---- recursion risk --------------------------------------------------------

/// No recursion -> recursion risk `None`.
#[test]
fn no_recursion_is_risk_none() {
    let plan = produce_plan(&request(scan("users", CardinalityV1::Bag))).expect("plans");
    assert_eq!(plan.cost.recursion_risk_class, RecursionRiskClassV1::None);
}

/// A fixpoint WITHOUT a witness is a bounded-fixpoint risk (terminates by
/// stratification, no path bound needed).
#[test]
fn fixpoint_without_witness_is_bounded_fixpoint() {
    let mut req = request(scan("reach", CardinalityV1::Set));
    req.recursion = RecursionShapeV1::Fixpoint(fixpoint_proof());
    let plan = produce_plan(&req).expect("recursive plan");
    assert_eq!(
        plan.cost.recursion_risk_class,
        RecursionRiskClassV1::BoundedFixpoint
    );
}

/// A fixpoint WITH a witness becomes bound-dependent: termination now leans on
/// the explicit witness/path bound.
#[test]
fn fixpoint_with_witness_is_bound_dependent() {
    let mut req = request(scan("reach", CardinalityV1::Set));
    req.recursion = RecursionShapeV1::Fixpoint(fixpoint_proof());
    req.witness = Some(acyclic_witness(WitnessSelectionPolicyV1::AllWitness));
    let plan = produce_plan(&req).expect("recursive witness plan");
    assert_eq!(
        plan.cost.recursion_risk_class,
        RecursionRiskClassV1::BoundDependent
    );
}

// ---- witness explosion risk ------------------------------------------------

/// No witness -> witness explosion risk `None`.
#[test]
fn no_witness_is_explosion_none() {
    let plan = produce_plan(&request(scan("users", CardinalityV1::Bag))).expect("plans");
    assert_eq!(
        plan.cost.witness_explosion_risk_class,
        WitnessExplosionRiskClassV1::None
    );
}

/// An acyclic witness with a canonical-one selection -> single-witness risk.
#[test]
fn acyclic_canonical_one_is_single_witness() {
    let mut req = request(scan("paths", CardinalityV1::Set));
    req.witness = Some(acyclic_witness(WitnessSelectionPolicyV1::CanonicalOne));
    req.result_kind = RequestedResultKindV1::PathWitness;
    let plan = produce_plan(&req).expect("witness plan");
    assert_eq!(
        plan.cost.witness_explosion_risk_class,
        WitnessExplosionRiskClassV1::SingleWitness
    );
}

/// An acyclic witness with a top-k selection -> bounded-set risk.
#[test]
fn acyclic_topk_is_bounded_set() {
    let mut req = request(scan("paths", CardinalityV1::Set));
    req.witness = Some(acyclic_witness(WitnessSelectionPolicyV1::TopK { k: 3 }));
    req.result_kind = RequestedResultKindV1::PathWitness;
    let plan = produce_plan(&req).expect("witness plan");
    assert_eq!(
        plan.cost.witness_explosion_risk_class,
        WitnessExplosionRiskClassV1::BoundedSet
    );
    assert_eq!(
        plan.witness_selection,
        Some(WitnessSelectionPolicyV1::TopK { k: 3 })
    );
}

/// An acyclic witness with an all-witness selection -> bounded-set risk.
#[test]
fn acyclic_all_witness_is_bounded_set() {
    let mut req = request(scan("paths", CardinalityV1::Set));
    req.witness = Some(acyclic_witness(WitnessSelectionPolicyV1::AllWitness));
    req.result_kind = RequestedResultKindV1::PathWitness;
    let plan = produce_plan(&req).expect("witness plan");
    assert_eq!(
        plan.cost.witness_explosion_risk_class,
        WitnessExplosionRiskClassV1::BoundedSet
    );
}

/// A cyclic witness -> cap-bounded risk regardless of the selection policy
/// (its termination is capped by the explicit path-length bound).
#[test]
fn cyclic_witness_is_cap_bounded() {
    let contract = WitnessContractV1::bind(&WitnessRequestV1 {
        shape: RelationShapeV1::Cyclic,
        dedup: DedupIdentityV1::NodeSequence,
        max_witnesses: Some(8),
        max_path_length: Some(16),
        tie_break: WitnessTieBreakV1::ShortestThenLexical,
        span: span(),
    })
    .expect("bounded cyclic witness");
    let mut req = request(scan("graph", CardinalityV1::Set));
    req.witness = Some(WitnessShapeV1 {
        contract,
        selection: WitnessSelectionPolicyV1::CanonicalOne,
    });
    req.result_kind = RequestedResultKindV1::PathWitness;
    let plan = produce_plan(&req).expect("cyclic witness plan");
    assert_eq!(
        plan.cost.witness_explosion_risk_class,
        WitnessExplosionRiskClassV1::CapBounded
    );
}

// ---- ordering contract -----------------------------------------------------

/// A bag result (no order) is unordered, like a set.
#[test]
fn bag_result_is_unordered() {
    let plan = produce_plan(&request(scan("users", CardinalityV1::Bag))).expect("plans");
    assert_eq!(plan.cost.ordering_contract, OrderingContractV1::Unordered);
    assert!(!plan.multiplicity_order.eliminates_duplicates());
}

// ---- closed keyword sets ---------------------------------------------------

#[test]
fn cardinality_class_keywords_are_stable() {
    assert_eq!(
        CardinalityClassV1::ScalarOrSingleton.keyword(),
        "scalar_or_singleton"
    );
    assert_eq!(CardinalityClassV1::Bounded.keyword(), "bounded");
    assert_eq!(
        CardinalityClassV1::LinearInInput.keyword(),
        "linear_in_input"
    );
    assert_eq!(
        CardinalityClassV1::Multiplicative.keyword(),
        "multiplicative"
    );
    assert_eq!(CardinalityClassV1::Closure.keyword(), "closure");
}

#[test]
fn recursion_risk_keywords_are_stable() {
    assert_eq!(RecursionRiskClassV1::None.keyword(), "none");
    assert_eq!(
        RecursionRiskClassV1::BoundedFixpoint.keyword(),
        "bounded_fixpoint"
    );
    assert_eq!(
        RecursionRiskClassV1::BoundDependent.keyword(),
        "bound_dependent"
    );
}

#[test]
fn witness_explosion_keywords_are_stable() {
    assert_eq!(WitnessExplosionRiskClassV1::None.keyword(), "none");
    assert_eq!(
        WitnessExplosionRiskClassV1::SingleWitness.keyword(),
        "single_witness"
    );
    assert_eq!(
        WitnessExplosionRiskClassV1::BoundedSet.keyword(),
        "bounded_set"
    );
    assert_eq!(
        WitnessExplosionRiskClassV1::CapBounded.keyword(),
        "cap_bounded"
    );
}

#[test]
fn ordering_contract_keywords_are_stable() {
    assert_eq!(OrderingContractV1::Unordered.keyword(), "unordered");
    assert_eq!(OrderingContractV1::Ordered.keyword(), "ordered");
}

#[test]
fn truth_lane_keywords_are_stable() {
    assert_eq!(TruthLaneV1::Deterministic.keyword(), "deterministic");
    assert_eq!(TruthLaneV1::Candidate.keyword(), "candidate");
    assert_eq!(TruthLaneV1::ProofConfirmed.keyword(), "proof_confirmed");
}

#[test]
fn evidence_requirement_keywords_are_stable() {
    assert_eq!(EvidenceRequirementV1::None.keyword(), "none");
    assert_eq!(
        EvidenceRequirementV1::SourceLocation.keyword(),
        "source_location"
    );
    assert_eq!(
        EvidenceRequirementV1::WitnessPayload.keyword(),
        "witness_payload"
    );
    assert_eq!(EvidenceRequirementV1::ProofTree.keyword(), "proof_tree");
}

#[test]
fn effect_class_keywords_are_stable() {
    assert_eq!(EffectClassV1::ReadOnly.keyword(), "read_only");
    assert_eq!(
        EffectClassV1::ProofEvidenceOnly.keyword(),
        "proof_evidence_only"
    );
    assert_eq!(EffectClassV1::ExportOnly.keyword(), "export_only");
    assert_eq!(
        EffectClassV1::MutationForbidden.keyword(),
        "mutation_forbidden"
    );
}

#[test]
fn result_kind_keywords_are_stable() {
    assert_eq!(ResultKindV1::Table.keyword(), "table");
    assert_eq!(ResultKindV1::Alert.keyword(), "alert");
    assert_eq!(ResultKindV1::PathWitness.keyword(), "path_witness");
    assert_eq!(ResultKindV1::ProofExplain.keyword(), "proof_explain");
}

#[test]
fn witness_selection_keywords_are_stable() {
    assert_eq!(
        WitnessSelectionPolicyV1::CanonicalOne.keyword(),
        "canonical_one"
    );
    assert_eq!(WitnessSelectionPolicyV1::TopK { k: 7 }.keyword(), "top_k");
    assert_eq!(
        WitnessSelectionPolicyV1::AllWitness.keyword(),
        "all_witness"
    );
}

#[test]
fn render_requirement_keywords_are_stable() {
    assert_eq!(RenderRequirementV1::RowsOnly.keyword(), "rows_only");
    assert_eq!(
        RenderRequirementV1::SourceLocation.keyword(),
        "source_location"
    );
    assert_eq!(
        RenderRequirementV1::WitnessPayload.keyword(),
        "witness_payload"
    );
    assert_eq!(
        RenderRequirementV1::ProvenanceTree.keyword(),
        "provenance_tree"
    );
}
