//! The host-agnostic public plan and its producer (ticket 07).
//!
//! [`PublicPlanV1`] is the single assembled planning output keyed off the
//! NORMALIZED algebra/logic shape (`D6`/`D14`/`D24`): capability requirements,
//! truth lane, evidence requirement, effect class, result kind, render
//! requirement, witness-selection policy, multiplicity/order contract, the fixed
//! semantic-cost schema, and a logical explain note. It is NOT a runtime DTO or a
//! physical plan — those stay downstream (`D24`).
//!
//! The producer [`produce_plan`] consumes a [`PlanRequestV1`] describing the
//! normalized inputs and the requested result-kind/effect. It derives everything
//! mechanically from the normalized shape and FAILS CLOSED with a
//! [`PlanningDiagnosticV1`] for an unsupported construct mix (`D10`/`D16`).

use std::collections::BTreeSet;

use quanta_gqlang_v2_algebra::{AlgebraNodeV1, AlgebraOpV1};
use quanta_gqlang_v2_hir::SpanV1;
use quanta_gqlang_v2_logic::{StratificationProofV1, WitnessContractV1};
use quanta_gqlang_v2_types::CardinalityV1;

use crate::capability::{derive_algebra_capabilities, CapabilityV1};
use crate::cost::{
    CardinalityClassV1, OrderingContractV1, RecursionRiskClassV1, SemanticCostV1,
    WitnessExplosionRiskClassV1,
};
use crate::diagnostic::{PlanningDiagnosticV1, UnsupportedMixReasonV1};
use crate::effect::{EffectClassV1, EvidenceRequirementV1, TruthLaneV1};
use crate::result_kind::{
    MultiplicityOrderContractV1, RenderRequirementV1, ResultKindV1, WitnessSelectionPolicyV1,
};

/// The recursion shape a request carries, if any. Derived upstream by ticket 05
/// (the proof is the admission artifact); the planner only classifies risk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecursionShapeV1 {
    /// No recursion.
    None,
    /// Positive recursion admitted under an explicit least-fixpoint proof.
    Fixpoint(StratificationProofV1),
}

/// The witness shape a request carries, if any.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WitnessShapeV1 {
    /// The bounded witness contract from ticket 05 (its existence proves the
    /// witness family is admissible).
    pub contract: WitnessContractV1,
    /// The explicit witness-selection policy (canonical-one / top-k / all).
    pub selection: WitnessSelectionPolicyV1,
}

/// The result-kind the caller requests for a plan. The producer validates it
/// against the normalized shape and effect, and derives render/evidence
/// requirements from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequestedResultKindV1 {
    /// A plain relational table.
    Table,
    /// An alert that must carry source locations.
    Alert,
    /// A path/witness result (requires a witness shape on the request).
    PathWitness,
    /// A proof/explain result (provenance without execution).
    ProofExplain,
}

/// A planning request: the normalized inputs plus the requested
/// result-kind/effect. The producer derives everything else.
#[derive(Clone, Debug, PartialEq)]
pub struct PlanRequestV1 {
    /// The normalized algebra root the result is rendered from.
    pub root: AlgebraNodeV1,
    /// The recursion shape, if the program is recursive.
    pub recursion: RecursionShapeV1,
    /// The witness shape, if witness/path materialization is requested.
    pub witness: Option<WitnessShapeV1>,
    /// The requested result-kind.
    pub result_kind: RequestedResultKindV1,
    /// The requested effect class.
    pub effect: EffectClassV1,
    /// The truth lane requested for the result.
    pub truth_lane: TruthLaneV1,
}

/// The assembled, host-agnostic public plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicPlanV1 {
    /// The host-agnostic capability requirements, derived from normalized shape.
    pub capabilities: BTreeSet<CapabilityV1>,
    /// The truth lane the result lives in.
    pub truth_lane: TruthLaneV1,
    /// The evidence the result must carry for its lane.
    pub evidence: EvidenceRequirementV1,
    /// The permitted effect class.
    pub effect: EffectClassV1,
    /// The kind of result rendered.
    pub result_kind: ResultKindV1,
    /// The per-row render-payload requirement.
    pub render_requirement: RenderRequirementV1,
    /// The witness-selection policy, when witnesses are materialized.
    pub witness_selection: Option<WitnessSelectionPolicyV1>,
    /// The multiplicity/order contract of the rendered result.
    pub multiplicity_order: MultiplicityOrderContractV1,
    /// The fixed semantic-cost schema.
    pub cost: SemanticCostV1,
    /// A logical explain note over the normalized semantics (provenance, not a
    /// runtime physical plan).
    pub explain: crate::explain::ExplainNoteV1,
}

/// Produce a host-agnostic [`PublicPlanV1`] from a normalized request, or reject
/// an unsupported construct mix with a typed [`PlanningDiagnosticV1`].
///
/// Rejection is deterministic and fail-closed (`D10`/`D16`): the mix checks run
/// first, in a fixed order, before any plan is assembled.
pub fn produce_plan(request: &PlanRequestV1) -> Result<PublicPlanV1, PlanningDiagnosticV1> {
    let span = request.root.span;
    reject_unsupported_mix(request, span)?;

    let mut capabilities = derive_algebra_capabilities(&request.root);
    if let RecursionShapeV1::Fixpoint(_) = &request.recursion {
        capabilities.insert(CapabilityV1::RecursiveClosure);
    }
    if request.witness.is_some() {
        capabilities.insert(CapabilityV1::WitnessMaterialization);
    }

    let result_kind = map_result_kind(request.result_kind);
    let render_requirement = render_requirement_for(result_kind);
    let evidence = evidence_for(result_kind, request.truth_lane);
    let multiplicity_order =
        MultiplicityOrderContractV1::from_cardinality(request.root.cardinality);

    let cost = SemanticCostV1 {
        estimated_cardinality_class: cardinality_class(&request.root, &request.recursion),
        recursion_risk_class: recursion_risk(&request.recursion, request.witness.as_ref()),
        witness_explosion_risk_class: witness_risk(request.witness.as_ref()),
        ordering_contract: ordering_contract(request.root.cardinality),
        unsupported_reason_code: None,
    };

    let explain = crate::explain::ExplainNoteV1::from_root(&request.root, &request.recursion);

    Ok(PublicPlanV1 {
        capabilities,
        truth_lane: request.truth_lane,
        evidence,
        effect: request.effect,
        result_kind,
        render_requirement,
        witness_selection: request.witness.map(|w| w.selection),
        multiplicity_order,
        cost,
        explain,
    })
}

/// Run the fixed-order unsupported-mix checks. Each rejection carries the stable
/// `unsupported_reason_code` (`D16`). The checks are mutually exclusive and run
/// in a fixed order so the surfaced reason is deterministic.
fn reject_unsupported_mix(
    request: &PlanRequestV1,
    span: SpanV1,
) -> Result<(), PlanningDiagnosticV1> {
    // 1. A path/witness result is incoherent without an admitted witness shape:
    //    the render/evidence contract demands witness payloads and an explicit
    //    selection policy.
    if request.result_kind == RequestedResultKindV1::PathWitness && request.witness.is_none() {
        return Err(PlanningDiagnosticV1::UnsupportedMix {
            reason: UnsupportedMixReasonV1::PathWitnessWithoutWitness,
            span,
        });
    }
    // 2. A proof/explain result cannot also export candidates: explain is
    //    read-only provenance, export materializes a candidate set.
    if request.result_kind == RequestedResultKindV1::ProofExplain
        && request.effect == EffectClassV1::ExportOnly
    {
        return Err(PlanningDiagnosticV1::UnsupportedMix {
            reason: UnsupportedMixReasonV1::ExplainWithExport,
            span,
        });
    }
    // 3. An export effect cannot combine with a proof/evidence-only lane: the
    //    lane forbids materialization beyond evidence while export demands it.
    if request.effect == EffectClassV1::ExportOnly
        && request.truth_lane == TruthLaneV1::ProofConfirmed
    {
        return Err(PlanningDiagnosticV1::UnsupportedMix {
            reason: UnsupportedMixReasonV1::ExportWithProofOnlyLane,
            span,
        });
    }
    // 4. Mutation is forbidden; pairing it with witness materialization rejects.
    if request.effect == EffectClassV1::MutationForbidden && request.witness.is_some() {
        return Err(PlanningDiagnosticV1::UnsupportedMix {
            reason: UnsupportedMixReasonV1::MutationWithWitness,
            span,
        });
    }
    // 5. A recursive closure has no total-order rendering contract. The
    //    incoherence is a property of the RESULT cardinality: any recursive
    //    program whose root renders an ordered (`Sequence`) result is invalid,
    //    regardless of which operator sits at the immediate root. Keying off the
    //    cardinality (rather than requiring the root op to be a `Sort`) catches
    //    cases where the ordering is established lower in the tree and a column
    //    re-shape (e.g. `Project`) preserves it at the root.
    if matches!(request.recursion, RecursionShapeV1::Fixpoint(_))
        && request.root.cardinality == CardinalityV1::Sequence
    {
        return Err(PlanningDiagnosticV1::UnsupportedMix {
            reason: UnsupportedMixReasonV1::OrderedRecursiveClosure,
            span,
        });
    }
    Ok(())
}

fn map_result_kind(kind: RequestedResultKindV1) -> ResultKindV1 {
    match kind {
        RequestedResultKindV1::Table => ResultKindV1::Table,
        RequestedResultKindV1::Alert => ResultKindV1::Alert,
        RequestedResultKindV1::PathWitness => ResultKindV1::PathWitness,
        RequestedResultKindV1::ProofExplain => ResultKindV1::ProofExplain,
    }
}

fn render_requirement_for(kind: ResultKindV1) -> RenderRequirementV1 {
    match kind {
        ResultKindV1::Table => RenderRequirementV1::RowsOnly,
        ResultKindV1::Alert => RenderRequirementV1::SourceLocation,
        ResultKindV1::PathWitness => RenderRequirementV1::WitnessPayload,
        ResultKindV1::ProofExplain => RenderRequirementV1::ProvenanceTree,
    }
}

/// Evidence requirement is the stronger of (a) what the result kind demands and
/// (b) what the truth lane demands. A proof-confirmed lane always demands a proof
/// tree; otherwise the result kind decides.
fn evidence_for(kind: ResultKindV1, lane: TruthLaneV1) -> EvidenceRequirementV1 {
    if lane.is_final_policy_capable() {
        return EvidenceRequirementV1::ProofTree;
    }
    match kind {
        ResultKindV1::Table => EvidenceRequirementV1::None,
        ResultKindV1::Alert => EvidenceRequirementV1::SourceLocation,
        ResultKindV1::PathWitness => EvidenceRequirementV1::WitnessPayload,
        ResultKindV1::ProofExplain => EvidenceRequirementV1::ProofTree,
    }
}

/// Derive the coarse cardinality class from the normalized root and recursion.
fn cardinality_class(root: &AlgebraNodeV1, recursion: &RecursionShapeV1) -> CardinalityClassV1 {
    if matches!(recursion, RecursionShapeV1::Fixpoint(_)) {
        return CardinalityClassV1::Closure;
    }
    classify_node(root)
}

fn classify_node(node: &AlgebraNodeV1) -> CardinalityClassV1 {
    match &node.op {
        AlgebraOpV1::Scan(_) => CardinalityClassV1::LinearInInput,
        AlgebraOpV1::Limit(_) => CardinalityClassV1::Bounded,
        AlgebraOpV1::Join(_) | AlgebraOpV1::Apply(_) => CardinalityClassV1::Multiplicative,
        AlgebraOpV1::Aggregate(aggregate) => {
            // An aggregate's output cardinality is governed by its GROUPING
            // partition, not by the size of the input it folds. The structural
            // invariant (enforced by `AlgebraNodeV1::new`'s `debug_assert`, see
            // algebra `node.rs`) guarantees the direct input is always a `Group`.
            // A GROUPED aggregate (non-empty keys) emits one row per distinct key:
            // linear in the number of groups, at most linear in the input. A
            // SCALAR aggregate has NO partition keys (the canonical empty-keyed
            // `Group` for `count(*) from t`), so it folds the WHOLE input into
            // exactly ONE row: `ScalarOrSingleton` (`D26`). We classify off the
            // ABSENCE of a non-empty grouping partition rather than asserting the
            // empty-keyed `Group` structure, so the result stays correct even in
            // release builds where the `debug_assert` is compiled out.
            if let AlgebraOpV1::Group(group) = &aggregate.input.op {
                if !group.keys.is_empty() {
                    return classify_node(&aggregate.input);
                }
            }
            CardinalityClassV1::ScalarOrSingleton
        }
        AlgebraOpV1::Group(group) => classify_node(&group.input),
        AlgebraOpV1::Project(project) => classify_node(&project.input),
        AlgebraOpV1::Filter(filter) => classify_node(&filter.input),
        AlgebraOpV1::Sort(sort) => classify_node(&sort.input),
        AlgebraOpV1::Union(union) => {
            // A union is at least as large as either arm; if either multiplies,
            // the union does too.
            let left = classify_node(&union.left);
            let right = classify_node(&union.right);
            worst_class(left, right)
        }
        AlgebraOpV1::SemiJoin(semi) => classify_node(&semi.left),
        AlgebraOpV1::AntiJoin(anti) => classify_node(&anti.left),
    }
}

/// Pick the larger-risk cardinality class of two.
fn worst_class(a: CardinalityClassV1, b: CardinalityClassV1) -> CardinalityClassV1 {
    if rank(a) >= rank(b) {
        a
    } else {
        b
    }
}

const fn rank(class: CardinalityClassV1) -> u8 {
    match class {
        CardinalityClassV1::ScalarOrSingleton => 0,
        CardinalityClassV1::Bounded => 1,
        CardinalityClassV1::LinearInInput => 2,
        CardinalityClassV1::Multiplicative => 3,
        CardinalityClassV1::Closure => 4,
    }
}

fn recursion_risk(
    recursion: &RecursionShapeV1,
    witness: Option<&WitnessShapeV1>,
) -> RecursionRiskClassV1 {
    match recursion {
        RecursionShapeV1::None => RecursionRiskClassV1::None,
        RecursionShapeV1::Fixpoint(_) => {
            if witness.is_some() {
                RecursionRiskClassV1::BoundDependent
            } else {
                RecursionRiskClassV1::BoundedFixpoint
            }
        }
    }
}

fn witness_risk(witness: Option<&WitnessShapeV1>) -> WitnessExplosionRiskClassV1 {
    use quanta_gqlang_v2_logic::RelationShapeV1;
    match witness {
        None => WitnessExplosionRiskClassV1::None,
        Some(shape) => {
            if shape.contract.shape == RelationShapeV1::Cyclic {
                WitnessExplosionRiskClassV1::CapBounded
            } else {
                match shape.selection {
                    WitnessSelectionPolicyV1::CanonicalOne => {
                        WitnessExplosionRiskClassV1::SingleWitness
                    }
                    WitnessSelectionPolicyV1::TopK { .. }
                    | WitnessSelectionPolicyV1::AllWitness => {
                        WitnessExplosionRiskClassV1::BoundedSet
                    }
                }
            }
        }
    }
}

fn ordering_contract(cardinality: CardinalityV1) -> OrderingContractV1 {
    match cardinality {
        CardinalityV1::Sequence => OrderingContractV1::Ordered,
        CardinalityV1::Set | CardinalityV1::Bag => OrderingContractV1::Unordered,
    }
}
