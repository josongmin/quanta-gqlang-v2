use crate::profile::ExecutionProfileClassV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiSemaErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl AiSemaErrorV1 {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AiExplorationPlanV1 {
    pub objective: AiObjectiveV1,
    pub discovery_plan: Option<DiscoveryPlanV1>,
    pub narrowing_plan: Option<NarrowingPlanV1>,
    pub proof_plan: Option<ProofPlanV1>,
    pub export_plan: Option<ExportPlanV1>,
    pub budget: ExplorationBudgetV1,
    pub reconfirmation: ReconfirmationRequirementV1,
}

impl AiExplorationPlanV1 {
    pub fn validate_v1(&self) -> Result<(), AiSemaErrorV1> {
        self.budget.validate_v1()?;
        self.reconfirmation.validate_v1()?;

        match self.objective {
            AiObjectiveV1::UnderstandSchema => {
                let discovery = self
                    .discovery_plan
                    .as_ref()
                    .ok_or_else(|| missing_plan_v1("AI_UNDERSTAND_SCHEMA_REQUIRES_DISCOVERY"))?;
                if !matches!(discovery.basis, DiscoveryBasisV1::SchemaIntrospection) {
                    return Err(AiSemaErrorV1::new(
                        "AI_UNDERSTAND_SCHEMA_BASIS_INVALID",
                        "UnderstandSchema requires SHOW/DESCRIBE-style schema introspection.",
                    ));
                }
            }
            AiObjectiveV1::FindCandidates => {
                let discovery = self
                    .discovery_plan
                    .as_ref()
                    .ok_or_else(|| missing_plan_v1("AI_FIND_CANDIDATES_REQUIRES_DISCOVERY"))?;
                match discovery.basis {
                    DiscoveryBasisV1::Candidate(_) => {}
                    DiscoveryBasisV1::SchemaIntrospection => {
                        return Err(AiSemaErrorV1::new(
                            "AI_FIND_CANDIDATES_BASIS_INVALID",
                            "FindCandidates requires a candidate basis, not schema introspection.",
                        ));
                    }
                }
            }
            AiObjectiveV1::NarrowCandidates => {
                let discovery = self
                    .discovery_plan
                    .as_ref()
                    .ok_or_else(|| missing_plan_v1("AI_NARROW_CANDIDATES_REQUIRES_DISCOVERY"))?;
                if !matches!(discovery.basis, DiscoveryBasisV1::Candidate(_)) {
                    return Err(AiSemaErrorV1::new(
                        "AI_NARROW_CANDIDATES_BASIS_INVALID",
                        "NarrowCandidates requires a candidate discovery basis.",
                    ));
                }
                self.narrowing_plan
                    .as_ref()
                    .ok_or_else(|| missing_plan_v1("AI_NARROW_CANDIDATES_REQUIRES_WORKING_SET"))?;
            }
            AiObjectiveV1::ProveOrDisprove => {
                let proof_plan = self
                    .proof_plan
                    .as_ref()
                    .ok_or_else(|| missing_plan_v1("AI_PROVE_OR_DISPROVE_REQUIRES_PROOF"))?;
                if matches!(
                    proof_plan.working_set_basis,
                    WorkingSetBasisV1::CandidateSet
                ) {
                    return Err(AiSemaErrorV1::new(
                        "AI_PROVE_OR_DISPROVE_WORKING_SET_TOO_SHALLOW",
                        "ProveOrDisprove requires a narrowed or reconfirmed working set.",
                    ));
                }
            }
            AiObjectiveV1::PrepareExport => {
                let export_plan = self
                    .export_plan
                    .as_ref()
                    .ok_or_else(|| missing_plan_v1("AI_PREPARE_EXPORT_REQUIRES_EXPORT"))?;
                export_plan.validate_v1()?;
            }
        }

        if let Some(export_plan) = &self.export_plan {
            export_plan.validate_v1()?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiObjectiveV1 {
    UnderstandSchema,
    FindCandidates,
    NarrowCandidates,
    ProveOrDisprove,
    PrepareExport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CandidateBasisV1 {
    Sample,
    RankedSelect,
    ShallowFind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiscoveryBasisV1 {
    SchemaIntrospection,
    Candidate(CandidateBasisV1),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkingSetBasisV1 {
    CandidateSet,
    NarrowedCandidateSet,
    ReconfirmedCandidateSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveryPlanV1 {
    pub basis: DiscoveryBasisV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NarrowingPlanV1 {
    pub candidate_basis: CandidateBasisV1,
    pub working_set_basis: WorkingSetBasisV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofPlanV1 {
    pub working_set_basis: WorkingSetBasisV1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiExportTargetV1 {
    CandidateSet,
    FinalPolicyTruthDirect,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportPlanV1 {
    pub working_set_basis: WorkingSetBasisV1,
    pub target: AiExportTargetV1,
}

impl ExportPlanV1 {
    fn validate_v1(&self) -> Result<(), AiSemaErrorV1> {
        if matches!(self.target, AiExportTargetV1::FinalPolicyTruthDirect) {
            return Err(AiSemaErrorV1::new(
                "AI_FINAL_POLICY_TRUTH_DIRECT_EMIT_FORBIDDEN",
                "AI exploration may export candidate_set only; final policy truth requires reconfirmation in lint or ci_gate.",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplorationBudgetV1 {
    pub max_candidates: u32,
    pub max_working_set_items: u32,
    pub max_reconfirmation_runs: u32,
}

impl ExplorationBudgetV1 {
    fn validate_v1(&self) -> Result<(), AiSemaErrorV1> {
        if self.max_candidates == 0 {
            return Err(AiSemaErrorV1::new(
                "AI_EXPLORATION_BUDGET_MAX_CANDIDATES_ZERO",
                "AI exploration requires a positive max_candidates budget.",
            ));
        }
        if self.max_working_set_items == 0 {
            return Err(AiSemaErrorV1::new(
                "AI_EXPLORATION_BUDGET_WORKING_SET_ZERO",
                "AI exploration requires a positive max_working_set_items budget.",
            ));
        }
        if self.max_reconfirmation_runs == 0 {
            return Err(AiSemaErrorV1::new(
                "AI_EXPLORATION_BUDGET_RECONFIRMATION_ZERO",
                "AI exploration requires a positive max_reconfirmation_runs budget.",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconfirmationRequirementV1 {
    pub replay_profile: ExecutionProfileClassV1,
    pub same_intent_replay: bool,
}

impl ReconfirmationRequirementV1 {
    fn validate_v1(&self) -> Result<(), AiSemaErrorV1> {
        if !self.replay_profile.supports_ai_reconfirmation_v1() {
            return Err(AiSemaErrorV1::new(
                "AI_RECONFIRMATION_PROFILE_INVALID",
                format!(
                    "AI exploration must reconfirm through lint or ci_gate, got {:?}.",
                    self.replay_profile
                ),
            ));
        }
        if !self.same_intent_replay {
            return Err(AiSemaErrorV1::new(
                "AI_RECONFIRMATION_REPLAY_REQUIRED",
                "AI exploration requires same-intent replay before lint or ci_gate truth.",
            ));
        }
        Ok(())
    }
}

fn missing_plan_v1(code: &'static str) -> AiSemaErrorV1 {
    AiSemaErrorV1::new(code, "Required AI plan phase is missing.")
}
