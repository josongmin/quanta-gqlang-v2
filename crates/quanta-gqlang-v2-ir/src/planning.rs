use crate::ai_plan::{AiExplorationPlanErrorV1, AiExplorationPlanV1};
use crate::working_set::WorkingSetKindV1;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PublicPlanRootKindV1 {
    Select,
    Find,
    Check,
    Explain,
    Retrieval,
    Interactive,
    Declaration,
    Meta,
    Composition,
}

impl PublicPlanRootKindV1 {
    #[must_use]
    pub fn requires_planning_metadata_v1(self) -> bool {
        matches!(
            self,
            Self::Find | Self::Check | Self::Explain | Self::Retrieval | Self::Interactive
        )
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CapabilityNameV1 {
    Search,
    Resolve,
    Relation,
    Path,
    TaintAnalysis,
    PointsTo,
    Summary,
    Explain,
    PatternLint,
    EntityIndex,
    CatalogIntrospection,
    Sampling,
    WorkingSetProjection,
    Metrics,
    DiagnosticsIngest,
    Command,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CapabilityReasonV1 {
    RootExecution,
    WorkingSetProjection,
    TruthReconfirmation,
    ExportProjection,
    ExplanationEvidence,
    CatalogRead,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapabilityRequirementV1 {
    pub capability: CapabilityNameV1,
    pub reason: CapabilityReasonV1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TruthExpectationV1 {
    Observational,
    CandidateSet,
    ReconfirmRequired,
    ExplainEvidence,
    FinalPolicyTruth,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum BridgeModeV1 {
    DirectRead,
    RetrievalBridge,
    ExplainBridge,
    PolicyBridge,
    WorkingSetBridge,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExportModeV1 {
    None,
    CandidateSet,
    Report,
    Sarif,
    Json,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicPlanV1 {
    pub root: PublicPlanRootKindV1,
    pub required_capabilities: Vec<CapabilityRequirementV1>,
    pub truth_expectation: TruthExpectationV1,
    pub bridge_mode: BridgeModeV1,
    pub export_mode: ExportModeV1,
    pub working_set_kind: Option<WorkingSetKindV1>,
    pub ai_plan: Option<AiExplorationPlanV1>,
}

impl PublicPlanV1 {
    pub fn validate_v1(&self) -> Result<(), PublicPlanValidationErrorV1> {
        if self.required_capabilities.is_empty() {
            return Err(PublicPlanValidationErrorV1::new_v1(
                "planning_missing_capabilities",
                "Public planning IR must declare at least one required capability",
            ));
        }

        if !matches!(self.export_mode, ExportModeV1::None) && self.working_set_kind.is_none() {
            return Err(PublicPlanValidationErrorV1::new_v1(
                "planning_export_missing_working_set_kind",
                "Export-capable public plans must name the public working-set kind they export",
            ));
        }

        if let Some(ai_plan) = &self.ai_plan {
            ai_plan.validate_v1()?;
            if matches!(self.truth_expectation, TruthExpectationV1::FinalPolicyTruth) {
                return Err(PublicPlanValidationErrorV1::new_v1(
                    "planning_ai_final_truth_forbidden",
                    "AI planning IR cannot emit final policy truth directly",
                ));
            }
            if !matches!(self.bridge_mode, BridgeModeV1::WorkingSetBridge) {
                return Err(PublicPlanValidationErrorV1::new_v1(
                    "planning_ai_requires_working_set_bridge",
                    "AI planning IR must use the working-set bridge mode",
                ));
            }
            if !matches!(
                self.truth_expectation,
                TruthExpectationV1::ReconfirmRequired
            ) {
                return Err(PublicPlanValidationErrorV1::new_v1(
                    "planning_ai_requires_reconfirmation_truth",
                    "AI planning IR must require reconfirmation before truth promotion",
                ));
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicPlanValidationErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl PublicPlanValidationErrorV1 {
    fn new_v1(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<AiExplorationPlanErrorV1> for PublicPlanValidationErrorV1 {
    fn from(value: AiExplorationPlanErrorV1) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}
