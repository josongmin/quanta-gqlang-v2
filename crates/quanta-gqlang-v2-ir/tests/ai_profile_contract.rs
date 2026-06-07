use quanta_gqlang_v2_ir::ai_plan::{
    AiExplorationPlanV1, AiObjectiveV1, DiscoveryPlanV1, ExplorationBudgetV1, ExportArtifactKindV1,
    ExportPlanV1, NarrowingPlanV1, NarrowingSignalV1, ProofEvidenceV1, ProofPlanV1,
    ProofTruthModeV1, ReconfirmationPlanV1,
};
use quanta_gqlang_v2_ir::working_set::{
    ReconfirmationTargetV1, WorkingSetBasisV1, WorkingSetItemV1, WorkingSetKindV1,
    WorkingSetReasonV1, WorkingSetV1,
};

#[test]
fn working_set_exposes_kind_basis_and_reasons_v1() {
    let working_set = WorkingSetV1 {
        working_set_id: "ws.catalog.candidates".to_string(),
        kind: WorkingSetKindV1::CandidateSet,
        basis: WorkingSetBasisV1::AiDiscovery,
        reasons: vec![
            WorkingSetReasonV1::QueryMatch {
                query: "SELECT id FROM catalog.files".to_string(),
            },
            WorkingSetReasonV1::ReconfirmRequired {
                target: ReconfirmationTargetV1::CiGate,
            },
        ],
        items: vec![WorkingSetItemV1 {
            public_ref: "catalog.files::id".to_string(),
            basis: WorkingSetBasisV1::AiNarrowing,
            reasons: vec![WorkingSetReasonV1::CapabilityRequirement {
                capability: "working_set_projection".to_string(),
            }],
        }],
    };

    assert_eq!(working_set.validate_v1(), Ok(()));
}

#[test]
fn ai_profile_requires_budget_subplans_and_reconfirmation_v1() {
    let valid = AiExplorationPlanV1 {
        objective: AiObjectiveV1::FindCandidateSet,
        discovery_plan: Some(DiscoveryPlanV1 {
            seed_query: "SAMPLE code WHERE text ~= \"Command::new\"".to_string(),
            target_kind: WorkingSetKindV1::CandidateSet,
            basis: WorkingSetBasisV1::AiDiscovery,
        }),
        narrowing_plan: Some(NarrowingPlanV1 {
            source_kind: WorkingSetKindV1::CandidateSet,
            narrowed_kind: WorkingSetKindV1::EvidenceSet,
            basis: WorkingSetBasisV1::AiNarrowing,
            required_signals: vec![
                NarrowingSignalV1::CapabilityCoverage,
                NarrowingSignalV1::TruthGap,
            ],
        }),
        proof_plan: Some(ProofPlanV1 {
            target_kind: WorkingSetKindV1::EvidenceSet,
            truth_mode: ProofTruthModeV1::ReconfirmRequired,
            evidence_requirements: vec![ProofEvidenceV1::Trace, ProofEvidenceV1::Citation],
        }),
        export_plan: Some(ExportPlanV1 {
            target_kind: WorkingSetKindV1::ExportSet,
            artifact: ExportArtifactKindV1::CandidateSet,
            requires_candidate_set_projection: true,
        }),
        budget: ExplorationBudgetV1 {
            max_candidates: 32,
            max_round_trips: 4,
            max_reconfirmations: 1,
        },
        reconfirmation: ReconfirmationPlanV1 {
            target: ReconfirmationTargetV1::Lint,
            replay_same_intent: true,
            export_candidate_set: true,
        },
    };
    assert_eq!(valid.validate_v1(), Ok(()));

    let invalid = AiExplorationPlanV1 {
        objective: AiObjectiveV1::BuildProof,
        discovery_plan: None,
        narrowing_plan: None,
        proof_plan: Some(ProofPlanV1 {
            target_kind: WorkingSetKindV1::EvidenceSet,
            truth_mode: ProofTruthModeV1::EvidenceOnly,
            evidence_requirements: vec![ProofEvidenceV1::Counterexample],
        }),
        export_plan: None,
        budget: ExplorationBudgetV1 {
            max_candidates: 0,
            max_round_trips: 1,
            max_reconfirmations: 1,
        },
        reconfirmation: ReconfirmationPlanV1 {
            target: ReconfirmationTargetV1::CiGate,
            replay_same_intent: false,
            export_candidate_set: false,
        },
    };
    let err = invalid
        .validate_v1()
        .expect_err("invalid ai plan must fail closed");
    assert_eq!(err.code, "ai_budget_missing_candidates");
}
