pub mod commands;
pub mod doctor;
pub mod explain_view;

/// The read-only shell's single "why no result" label, shared by the headless and
/// doctor producers so the wording cannot desync between them (mirrors the
/// `DegradedReasonV1` consolidation). Presentation-only — it asserts no runtime
/// truth. Single-sourced in `ir` and re-exported here so `crate::NO_RESULT_REASON_V1`
/// keeps resolving for the CLI code and tests that reference it.
pub use quanta_gqlang_v2_ir::frontend::NO_RESULT_REASON_V1;
