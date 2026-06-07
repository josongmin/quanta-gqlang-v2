#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionProfileClassV1 {
    Query,
    Ide,
    Ai,
    Lint,
    CiGate,
}

impl ExecutionProfileClassV1 {
    pub const fn supports_ai_reconfirmation_v1(self) -> bool {
        matches!(self, Self::Lint | Self::CiGate)
    }
}
