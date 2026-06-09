pub mod commands;
pub mod doctor;
pub mod explain_view;

/// The read-only shell's single "why no result" label, shared by the headless and
/// doctor producers so the wording cannot desync between them (mirrors the
/// `DegradedReasonV1` consolidation). Presentation-only — it asserts no runtime truth.
pub const NO_RESULT_REASON_V1: &str = "read-only-shell-has-no-execution-backend";
