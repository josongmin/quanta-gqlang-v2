use crate::working_set::{ReconfirmationTargetV1, WorkingSetBasisV1, WorkingSetKindV1};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AiObjectiveV1 {
    DiagnoseNoResult,
    FindCandidateSet,
    NarrowCandidateSet,
    BuildProof,
    ExportCandidateSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplorationBudgetV1 {
    pub max_candidates: u32,
    pub max_round_trips: u32,
    pub max_reconfirmations: u32,
}

impl ExplorationBudgetV1 {
    pub fn validate_v1(&self) -> Result<(), AiExplorationPlanErrorV1> {
        if self.max_candidates == 0 {
            return Err(AiExplorationPlanErrorV1::new_v1(
                "ai_budget_missing_candidates",
                "AI exploration budget must allocate at least one candidate",
            ));
        }
        if self.max_round_trips == 0 {
            return Err(AiExplorationPlanErrorV1::new_v1(
                "ai_budget_missing_round_trips",
                "AI exploration budget must allow at least one discovery round trip",
            ));
        }
        if self.max_reconfirmations == 0 {
            return Err(AiExplorationPlanErrorV1::new_v1(
                "ai_budget_missing_reconfirmations",
                "AI exploration budget must reserve at least one reconfirmation replay",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryPlanV1 {
    pub seed_query: String,
    pub target_kind: WorkingSetKindV1,
    pub basis: WorkingSetBasisV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NarrowingPlanV1 {
    pub source_kind: WorkingSetKindV1,
    pub narrowed_kind: WorkingSetKindV1,
    pub basis: WorkingSetBasisV1,
    pub required_signals: Vec<NarrowingSignalV1>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NarrowingSignalV1 {
    Ranking,
    CapabilityCoverage,
    TruthGap,
    HumanLabel,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofPlanV1 {
    pub target_kind: WorkingSetKindV1,
    pub truth_mode: ProofTruthModeV1,
    pub evidence_requirements: Vec<ProofEvidenceV1>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProofTruthModeV1 {
    EvidenceOnly,
    ReconfirmRequired,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProofEvidenceV1 {
    Trace,
    Citation,
    Counterexample,
    Diff,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportPlanV1 {
    pub target_kind: WorkingSetKindV1,
    pub artifact: ExportArtifactKindV1,
    pub requires_candidate_set_projection: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExportArtifactKindV1 {
    CandidateSet,
    MarkdownReport,
    Json,
    Sarif,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ReconfirmationPlanV1 {
    pub target: ReconfirmationTargetV1,
    pub replay_same_intent: bool,
    pub export_candidate_set: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiExplorationPlanV1 {
    pub objective: AiObjectiveV1,
    pub discovery_plan: Option<DiscoveryPlanV1>,
    pub narrowing_plan: Option<NarrowingPlanV1>,
    pub proof_plan: Option<ProofPlanV1>,
    pub export_plan: Option<ExportPlanV1>,
    pub budget: ExplorationBudgetV1,
    pub reconfirmation: ReconfirmationPlanV1,
}

impl AiExplorationPlanV1 {
    pub fn validate_v1(&self) -> Result<(), AiExplorationPlanErrorV1> {
        self.budget.validate_v1()?;

        if self.discovery_plan.is_none()
            && self.narrowing_plan.is_none()
            && self.proof_plan.is_none()
            && self.export_plan.is_none()
        {
            return Err(AiExplorationPlanErrorV1::new_v1(
                "ai_plan_missing_subplans",
                "AI exploration plan must carry at least one discovery, narrowing, proof, or export subplan",
            ));
        }

        if !self.reconfirmation.replay_same_intent {
            return Err(AiExplorationPlanErrorV1::new_v1(
                "ai_plan_reconfirmation_not_same_intent",
                "AI exploration plan must replay the same intent through ci_gate or lint",
            ));
        }

        if !self.reconfirmation.export_candidate_set {
            return Err(AiExplorationPlanErrorV1::new_v1(
                "ai_plan_missing_candidate_set_export",
                "AI exploration plan must export a candidate_set before reconfirmation",
            ));
        }

        if let Some(discovery) = &self.discovery_plan {
            if discovery.seed_query.trim().is_empty() {
                return Err(AiExplorationPlanErrorV1::new_v1(
                    "ai_discovery_missing_seed_query",
                    "AI discovery plan requires a non-empty seed query",
                ));
            }
        }

        if let Some(narrowing) = &self.narrowing_plan {
            if narrowing.required_signals.is_empty() {
                return Err(AiExplorationPlanErrorV1::new_v1(
                    "ai_narrowing_missing_signals",
                    "AI narrowing plan must name at least one narrowing signal",
                ));
            }
        }

        if let Some(proof) = &self.proof_plan {
            if proof.evidence_requirements.is_empty() {
                return Err(AiExplorationPlanErrorV1::new_v1(
                    "ai_proof_missing_evidence",
                    "AI proof plan must require at least one evidence mode",
                ));
            }
            if !matches!(proof.truth_mode, ProofTruthModeV1::ReconfirmRequired) {
                return Err(AiExplorationPlanErrorV1::new_v1(
                    "ai_proof_truth_mode_open",
                    "AI proof plan must require reconfirmation before policy truth",
                ));
            }
        }

        if let Some(export) = &self.export_plan {
            if !export.requires_candidate_set_projection {
                return Err(AiExplorationPlanErrorV1::new_v1(
                    "ai_export_missing_candidate_projection",
                    "AI export plan must project a candidate_set instead of runtime-local artifacts",
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiExplorationPlanErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl AiExplorationPlanErrorV1 {
    fn new_v1(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
