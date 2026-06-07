use crate::ai::exploration::{AiExplorationPlanV1, AiSemaErrorV1};
use crate::profile::ExecutionProfileClassV1;
use crate::TypedRootV1;

pub fn validate_root_v1(_root: &TypedRootV1) -> Result<(), String> {
    Ok(())
}

pub fn validate_ai_plan_v1(
    profile: ExecutionProfileClassV1,
    plan: &AiExplorationPlanV1,
) -> Result<(), AiSemaErrorV1> {
    if !matches!(profile, ExecutionProfileClassV1::Ai) {
        return Err(AiSemaErrorV1 {
            code: "AI_PROFILE_REQUIRED",
            message: format!(
                "AI exploration plan requires ai profile at sema time, got {:?}.",
                profile
            ),
        });
    }

    plan.validate_v1()
}
