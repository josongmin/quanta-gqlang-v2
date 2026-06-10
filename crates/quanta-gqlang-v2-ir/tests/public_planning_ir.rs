#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::ai_plan::{
    AiExplorationPlanV1, AiObjectiveV1, DiscoveryPlanV1, ExplorationBudgetV1, ReconfirmationPlanV1,
};
use quanta_gqlang_v2_ir::find_ir::FindIrV1;
use quanta_gqlang_v2_ir::lowering_ready::LoweringReadyIrV1;
use quanta_gqlang_v2_ir::planning::{
    BridgeModeV1, CapabilityNameV1, CapabilityReasonV1, CapabilityRequirementV1, ExportModeV1,
    PublicPlanRootKindV1, PublicPlanV1, TruthExpectationV1,
};
use quanta_gqlang_v2_ir::root_ir::PublicIrRootV1;
use quanta_gqlang_v2_ir::select_ir::SelectIrV1;
use quanta_gqlang_v2_ir::working_set::{
    ReconfirmationTargetV1, WorkingSetBasisV1, WorkingSetKindV1,
};

#[test]
fn lowering_ready_ir_preserves_public_planning_metadata_v1() {
    let ai_plan = AiExplorationPlanV1 {
        objective: AiObjectiveV1::FindCandidateSet,
        discovery_plan: Some(DiscoveryPlanV1 {
            seed_query: "FIND taint source=web sink=db".to_string(),
            target_kind: WorkingSetKindV1::CandidateSet,
            basis: WorkingSetBasisV1::AiDiscovery,
        }),
        narrowing_plan: None,
        proof_plan: None,
        export_plan: None,
        budget: ExplorationBudgetV1 {
            max_candidates: 12,
            max_round_trips: 2,
            max_reconfirmations: 1,
        },
        reconfirmation: ReconfirmationPlanV1 {
            target: ReconfirmationTargetV1::CiGate,
            replay_same_intent: true,
            export_candidate_set: true,
        },
    };
    let plan = PublicPlanV1 {
        root: PublicPlanRootKindV1::Find,
        required_capabilities: vec![
            CapabilityRequirementV1 {
                capability: CapabilityNameV1::TaintAnalysis,
                reason: CapabilityReasonV1::RootExecution,
            },
            CapabilityRequirementV1 {
                capability: CapabilityNameV1::WorkingSetProjection,
                reason: CapabilityReasonV1::TruthReconfirmation,
            },
        ],
        truth_expectation: TruthExpectationV1::ReconfirmRequired,
        bridge_mode: BridgeModeV1::WorkingSetBridge,
        export_mode: ExportModeV1::CandidateSet,
        working_set_kind: Some(WorkingSetKindV1::CandidateSet),
        ai_plan: Some(ai_plan),
    };
    let ready = LoweringReadyIrV1::new_v1(
        PublicIrRootV1::Find(FindIrV1::from_typed_v1("taint", "web -> db").expect("valid find")),
        Some(plan.clone()),
    )
    .expect("planning metadata should validate");

    let normalized = ready.normalized_v1().expect("normalization");
    assert_eq!(normalized.planning, Some(plan));
    assert_eq!(normalized.root_kind_v1(), PublicPlanRootKindV1::Find);
}

#[test]
fn roots_requiring_planning_metadata_fail_closed_when_absent_v1() {
    let find_err = LoweringReadyIrV1::new_v1(
        PublicIrRootV1::Find(FindIrV1::from_typed_v1("path", "entry -> sink").expect("valid find")),
        None,
    )
    .expect_err("FIND should require planning metadata");
    assert_eq!(find_err.code, "missing_planning_metadata");

    let select_ready = LoweringReadyIrV1::new_v1(
        PublicIrRootV1::Select(SelectIrV1 {
            from_entity: "SELECT id FROM catalog.files".to_string(),
        }),
        None,
    )
    .expect("SELECT may remain planning-free");
    assert_eq!(select_ready.root_kind_v1(), PublicPlanRootKindV1::Select);
}
