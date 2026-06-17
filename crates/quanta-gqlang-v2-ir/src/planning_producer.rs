//! E06C / E05E — the PRODUCTION planning-IR producer.
//!
//! Before this module the planning IR (`PublicPlanV1` / `LoweringReadyIrV1`) had
//! no production constructor — only tests built it, so the `Some(ai_plan)`
//! validation branch was unreachable from real code. This module is the single
//! production producer: it maps every lowered root to its planning metadata and
//! supplies a `LoweringReadyIrV1` whose validation actually runs.

use crate::ai_plan::{
    AiExplorationPlanV1, AiObjectiveV1, DiscoveryPlanV1, ExplorationBudgetV1, ReconfirmationPlanV1,
};
use crate::lowering_ready::{LoweringReadyIrErrorV1, LoweringReadyIrV1};
use crate::planning::{
    BridgeModeV1, CapabilityNameV1, CapabilityReasonV1, CapabilityRequirementV1, ExportModeV1,
    PublicPlanRootKindV1, PublicPlanV1, TruthExpectationV1,
};
use crate::root_ir::PublicIrRootV1;
use crate::working_set::{ReconfirmationTargetV1, WorkingSetBasisV1, WorkingSetKindV1};

/// The capability a root kind requires (None for non-query-plane roots).
fn capability_for_kind_v1(
    kind: PublicPlanRootKindV1,
) -> Option<(CapabilityNameV1, CapabilityReasonV1)> {
    use PublicPlanRootKindV1 as K;
    Some(match kind {
        K::Select | K::Retrieval => {
            (CapabilityNameV1::Search, CapabilityReasonV1::RootExecution)
        }
        K::Find => (CapabilityNameV1::Path, CapabilityReasonV1::RootExecution),
        K::Check => (
            CapabilityNameV1::PatternLint,
            CapabilityReasonV1::RootExecution,
        ),
        K::Explain => (
            CapabilityNameV1::Explain,
            CapabilityReasonV1::ExplanationEvidence,
        ),
        K::Interactive => (
            CapabilityNameV1::CatalogIntrospection,
            CapabilityReasonV1::CatalogRead,
        ),
        // Composition is served through its WITH body. It must not invent a
        // second top-level planning lane here; semantica ingress lowers the
        // body with `None` planning and attaches only the working-set ref.
        K::Composition | K::Declaration | K::Meta => return None,
    })
}

/// The truth/bridge/export lane a root kind runs in.
fn lane_for_kind_v1(
    kind: PublicPlanRootKindV1,
) -> (
    TruthExpectationV1,
    BridgeModeV1,
    ExportModeV1,
    Option<WorkingSetKindV1>,
) {
    use PublicPlanRootKindV1 as K;
    match kind {
        K::Check => (
            TruthExpectationV1::FinalPolicyTruth,
            BridgeModeV1::PolicyBridge,
            ExportModeV1::None,
            None,
        ),
        K::Explain => (
            TruthExpectationV1::ExplainEvidence,
            BridgeModeV1::ExplainBridge,
            ExportModeV1::None,
            None,
        ),
        K::Retrieval => (
            TruthExpectationV1::CandidateSet,
            BridgeModeV1::RetrievalBridge,
            ExportModeV1::None,
            None,
        ),
        K::Select | K::Find | K::Interactive | K::Composition | K::Declaration | K::Meta => {
            (
                TruthExpectationV1::Observational,
                BridgeModeV1::DirectRead,
                ExportModeV1::None,
                None,
            )
        }
    }
}

/// Produce the planning IR for a lowered root. `None` for declaration/meta
/// command roots and for composition, whose planning authority stays delegated
/// to its served WITH body instead of inventing a second top-level plan.
#[must_use]
pub fn plan_for_root_v1(root: &PublicIrRootV1) -> Option<PublicPlanV1> {
    let kind = root.plan_root_kind_v1();
    let (capability, reason) = capability_for_kind_v1(kind)?;
    let (truth_expectation, bridge_mode, export_mode, working_set_kind) = lane_for_kind_v1(kind);
    Some(PublicPlanV1 {
        root: kind,
        required_capabilities: vec![CapabilityRequirementV1 { capability, reason }],
        truth_expectation,
        bridge_mode,
        export_mode,
        working_set_kind,
        ai_plan: None,
    })
}

/// Produce an AI-lane plan for a Retrieval root (E05E). This is the first
/// production path that constructs a `Some(ai_plan)` plan, so the AI validation
/// branch (final-truth forbidden, working-set bridge required, reconfirmation
/// discipline) is now reachable from real code. `None` for non-Retrieval roots.
#[must_use]
pub fn plan_for_ai_retrieval_v1(root: &PublicIrRootV1) -> Option<PublicPlanV1> {
    let PublicIrRootV1::Retrieval(retrieval) = root else {
        return None;
    };
    let ai_plan = AiExplorationPlanV1 {
        objective: AiObjectiveV1::FindCandidateSet,
        discovery_plan: Some(DiscoveryPlanV1 {
            seed_query: retrieval.query.clone(),
            target_kind: WorkingSetKindV1::CandidateSet,
            basis: WorkingSetBasisV1::AiDiscovery,
        }),
        narrowing_plan: None,
        proof_plan: None,
        export_plan: None,
        budget: ExplorationBudgetV1 {
            max_candidates: 20,
            max_round_trips: 3,
            max_reconfirmations: 1,
        },
        reconfirmation: ReconfirmationPlanV1 {
            target: ReconfirmationTargetV1::CiGate,
            replay_same_intent: true,
            export_candidate_set: true,
        },
    };
    Some(PublicPlanV1 {
        root: PublicPlanRootKindV1::Retrieval,
        required_capabilities: vec![
            CapabilityRequirementV1 {
                capability: CapabilityNameV1::Sampling,
                reason: CapabilityReasonV1::RootExecution,
            },
            CapabilityRequirementV1 {
                capability: CapabilityNameV1::WorkingSetProjection,
                reason: CapabilityReasonV1::WorkingSetProjection,
            },
        ],
        // Ranking/AI never emits final policy truth and must reconfirm.
        truth_expectation: TruthExpectationV1::ReconfirmRequired,
        bridge_mode: BridgeModeV1::WorkingSetBridge,
        export_mode: ExportModeV1::CandidateSet,
        working_set_kind: Some(WorkingSetKindV1::CandidateSet),
        ai_plan: Some(ai_plan),
    })
}

/// Build a validated `LoweringReadyIrV1` — supplies the plan for roots that
/// require planning metadata so the lowering-ready validation passes. This is
/// the first production path that exercises that validation.
pub fn lowering_ready_for_root_v1(
    root: PublicIrRootV1,
) -> Result<LoweringReadyIrV1, LoweringReadyIrErrorV1> {
    let plan = plan_for_root_v1(&root);
    LoweringReadyIrV1::new_v1(root, plan)
}
