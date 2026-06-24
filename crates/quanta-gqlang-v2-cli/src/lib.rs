pub mod commands;
pub mod v3_plan;

/// The complete set of CLI subcommand tokens, in usage order. This is the single
/// list the doc gate checks the README and the `--help` usage string against, so
/// a shipped command can never go undocumented. A binary test pins that each
/// token is a real, accepted command.
///
/// HARD CUTOVER: the CLI is v3-ONLY. `query` / `fmt` are the v3 path; `v3-query`
/// / `v3-fmt` are retained as explicit aliases for the same behavior.
pub const CLI_COMMAND_TOKENS_V1: [&str; 4] = ["query", "fmt", "v3-query", "v3-fmt"];
