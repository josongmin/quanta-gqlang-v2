#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]
#![allow(
    dead_code,
    reason = "shared test helpers are used by a subset of the integration-test crates that include this module"
)]

use std::process::{Command, Output};

/// Run the shipped CLI binary front door with an arbitrary argv vector, leaving
/// argument shaping to the caller so usage, help, and per-verb contracts can be
/// exercised end-to-end against the real binary.
pub(crate) fn run_cli_argv_v1(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-cli"))
        .args(args)
        .output()
        .expect("spawn qglang cli")
}

pub(crate) fn cli_stdout_v1(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf-8")
}

pub(crate) fn cli_stderr_v1(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr utf-8")
}
