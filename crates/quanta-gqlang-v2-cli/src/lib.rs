pub mod commands;
pub mod doctor;
pub mod explain_view;

/// The complete set of CLI subcommand tokens, in usage order. This is the single
/// list the doc gate checks the README and the `--help` usage string against, so
/// a shipped command can never go undocumented (the `explain` omission the audit
/// caught). A binary test pins that each token is a real, accepted command.
pub const CLI_COMMAND_TOKENS_V1: [&str; 7] = [
    "parse", "sema", "query", "fmt", "explain", "doctor", "headless",
];

/// The read-only shell's single "why no result" label, shared by the headless and
/// doctor producers so the wording cannot desync between them (mirrors the
/// `DegradedReasonV1` consolidation). Presentation-only — it asserts no runtime
/// truth. Single-sourced in `ir` and re-exported here so `crate::NO_RESULT_REASON_V1`
/// keeps resolving for the CLI code and tests that reference it.
pub use quanta_gqlang_v2_ir::frontend::NO_RESULT_REASON_V1;
