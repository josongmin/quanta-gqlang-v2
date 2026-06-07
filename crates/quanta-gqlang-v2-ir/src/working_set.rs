#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WorkingSetKindV1 {
    CandidateSet,
    EvidenceSet,
    CounterexampleSet,
    ExportSet,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum WorkingSetBasisV1 {
    SearchQuery,
    CompareDelta,
    ExplainTrace,
    AiDiscovery,
    AiNarrowing,
    ManualReview,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReconfirmationTargetV1 {
    CiGate,
    Lint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkingSetReasonV1 {
    QueryMatch { query: String },
    CapabilityRequirement { capability: String },
    TruthGap { expectation: String },
    ReconfirmRequired { target: ReconfirmationTargetV1 },
    UserFilter { clause: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkingSetV1 {
    pub working_set_id: String,
    pub kind: WorkingSetKindV1,
    pub basis: WorkingSetBasisV1,
    pub reasons: Vec<WorkingSetReasonV1>,
    pub items: Vec<WorkingSetItemV1>,
}

impl WorkingSetV1 {
    pub fn validate_v1(&self) -> Result<(), WorkingSetValidationErrorV1> {
        if self.working_set_id.trim().is_empty() {
            return Err(WorkingSetValidationErrorV1::new_v1(
                "working_set_missing_id",
                "Working set requires a stable public working_set_id",
            ));
        }
        if self.reasons.is_empty() {
            return Err(WorkingSetValidationErrorV1::new_v1(
                "working_set_missing_reasons",
                "Working set must expose at least one public reason",
            ));
        }
        if self.items.is_empty() {
            return Err(WorkingSetValidationErrorV1::new_v1(
                "working_set_missing_items",
                "Working set must carry at least one public item",
            ));
        }
        for item in &self.items {
            item.validate_v1()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkingSetItemV1 {
    pub public_ref: String,
    pub basis: WorkingSetBasisV1,
    pub reasons: Vec<WorkingSetReasonV1>,
}

impl WorkingSetItemV1 {
    fn validate_v1(&self) -> Result<(), WorkingSetValidationErrorV1> {
        if self.public_ref.trim().is_empty() {
            return Err(WorkingSetValidationErrorV1::new_v1(
                "working_set_item_missing_public_ref",
                "Working set item must carry a public reference instead of a runtime-local id",
            ));
        }
        if self.reasons.is_empty() {
            return Err(WorkingSetValidationErrorV1::new_v1(
                "working_set_item_missing_reasons",
                "Working set item must expose at least one public reason",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkingSetValidationErrorV1 {
    pub code: &'static str,
    pub message: String,
}

impl WorkingSetValidationErrorV1 {
    fn new_v1(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
