#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_sema::admissibility::validate_ai_plan_v1;
use quanta_gqlang_v2_sema::ai::exploration::{
    AiExplorationPlanV1, AiExportTargetV1, AiObjectiveV1, CandidateBasisV1, DiscoveryBasisV1,
    DiscoveryPlanV1, ExplorationBudgetV1, ExportPlanV1, NarrowingPlanV1, ProofPlanV1,
    ReconfirmationRequirementV1, WorkingSetBasisV1,
};
use quanta_gqlang_v2_sema::profile::ExecutionProfileClassV1;

fn valid_budget_v1() -> ExplorationBudgetV1 {
    ExplorationBudgetV1 {
        max_candidates: 25,
        max_working_set_items: 10,
        max_reconfirmation_runs: 2,
    }
}

fn valid_reconfirmation_v1() -> ReconfirmationRequirementV1 {
    ReconfirmationRequirementV1 {
        replay_profile: ExecutionProfileClassV1::Lint,
        same_intent_replay: true,
    }
}

#[test]
fn ai_find_candidates_plan_accepts_typed_candidate_basis_v1() {
    let plan = AiExplorationPlanV1 {
        objective: AiObjectiveV1::FindCandidates,
        discovery_plan: Some(DiscoveryPlanV1 {
            basis: DiscoveryBasisV1::Candidate(CandidateBasisV1::RankedSelect),
        }),
        narrowing_plan: None,
        proof_plan: None,
        export_plan: Some(ExportPlanV1 {
            working_set_basis: WorkingSetBasisV1::CandidateSet,
            target: AiExportTargetV1::CandidateSet,
        }),
        budget: valid_budget_v1(),
        reconfirmation: valid_reconfirmation_v1(),
    };

    assert_eq!(
        validate_ai_plan_v1(ExecutionProfileClassV1::Ai, &plan),
        Ok(())
    );
}

#[test]
fn ai_narrow_candidates_plan_exposes_working_set_basis_v1() {
    let plan = AiExplorationPlanV1 {
        objective: AiObjectiveV1::NarrowCandidates,
        discovery_plan: Some(DiscoveryPlanV1 {
            basis: DiscoveryBasisV1::Candidate(CandidateBasisV1::ShallowFind),
        }),
        narrowing_plan: Some(NarrowingPlanV1 {
            candidate_basis: CandidateBasisV1::ShallowFind,
            working_set_basis: WorkingSetBasisV1::NarrowedCandidateSet,
        }),
        proof_plan: None,
        export_plan: None,
        budget: valid_budget_v1(),
        reconfirmation: valid_reconfirmation_v1(),
    };

    assert_eq!(
        plan.narrowing_plan
            .as_ref()
            .map(|plan| plan.working_set_basis),
        Some(WorkingSetBasisV1::NarrowedCandidateSet)
    );
    assert_eq!(
        validate_ai_plan_v1(ExecutionProfileClassV1::Ai, &plan),
        Ok(())
    );
}

#[test]
fn ai_plan_rejects_zero_budget_v1() {
    let plan = AiExplorationPlanV1 {
        objective: AiObjectiveV1::FindCandidates,
        discovery_plan: Some(DiscoveryPlanV1 {
            basis: DiscoveryBasisV1::Candidate(CandidateBasisV1::Sample),
        }),
        narrowing_plan: None,
        proof_plan: None,
        export_plan: None,
        budget: ExplorationBudgetV1 {
            max_candidates: 0,
            max_working_set_items: 4,
            max_reconfirmation_runs: 1,
        },
        reconfirmation: valid_reconfirmation_v1(),
    };

    let err = validate_ai_plan_v1(ExecutionProfileClassV1::Ai, &plan).unwrap_err();
    assert_eq!(err.code, "AI_EXPLORATION_BUDGET_MAX_CANDIDATES_ZERO");
}

#[test]
fn ai_plan_rejects_non_reconfirm_profile_v1() {
    let plan = AiExplorationPlanV1 {
        objective: AiObjectiveV1::PrepareExport,
        discovery_plan: None,
        narrowing_plan: None,
        proof_plan: None,
        export_plan: Some(ExportPlanV1 {
            working_set_basis: WorkingSetBasisV1::CandidateSet,
            target: AiExportTargetV1::CandidateSet,
        }),
        budget: valid_budget_v1(),
        reconfirmation: ReconfirmationRequirementV1 {
            replay_profile: ExecutionProfileClassV1::Ai,
            same_intent_replay: true,
        },
    };

    let err = validate_ai_plan_v1(ExecutionProfileClassV1::Ai, &plan).unwrap_err();
    assert_eq!(err.code, "AI_RECONFIRMATION_PROFILE_INVALID");
}

#[test]
fn ai_plan_rejects_final_policy_truth_direct_emit_v1() {
    let plan = AiExplorationPlanV1 {
        objective: AiObjectiveV1::PrepareExport,
        discovery_plan: None,
        narrowing_plan: None,
        proof_plan: Some(ProofPlanV1 {
            working_set_basis: WorkingSetBasisV1::ReconfirmedCandidateSet,
        }),
        export_plan: Some(ExportPlanV1 {
            working_set_basis: WorkingSetBasisV1::ReconfirmedCandidateSet,
            target: AiExportTargetV1::FinalPolicyTruthDirect,
        }),
        budget: valid_budget_v1(),
        reconfirmation: ReconfirmationRequirementV1 {
            replay_profile: ExecutionProfileClassV1::CiGate,
            same_intent_replay: true,
        },
    };

    let err = validate_ai_plan_v1(ExecutionProfileClassV1::Ai, &plan).unwrap_err();
    assert_eq!(err.code, "AI_FINAL_POLICY_TRUTH_DIRECT_EMIT_FORBIDDEN");
}

#[test]
fn ai_plan_rejects_non_ai_profile_at_admissibility_boundary_v1() {
    let plan = AiExplorationPlanV1 {
        objective: AiObjectiveV1::UnderstandSchema,
        discovery_plan: Some(DiscoveryPlanV1 {
            basis: DiscoveryBasisV1::SchemaIntrospection,
        }),
        narrowing_plan: None,
        proof_plan: None,
        export_plan: None,
        budget: valid_budget_v1(),
        reconfirmation: valid_reconfirmation_v1(),
    };

    let err = validate_ai_plan_v1(ExecutionProfileClassV1::Query, &plan).unwrap_err();
    assert_eq!(err.code, "AI_PROFILE_REQUIRED");
}
