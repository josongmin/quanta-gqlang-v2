#![expect(
    clippy::expect_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use std::process::{Command, Output, Stdio};
use std::time::Duration;

#[path = "../../../../tools/test_support/process.rs"]
pub(crate) mod process;

/// Run the shipped CLI binary front door with an arbitrary argv vector, leaving
/// argument shaping to the caller so usage, help, and per-verb contracts can be
/// exercised end-to-end against the real binary.
pub(crate) fn run_cli_argv_v1(args: &[&str]) -> Output {
    let child = Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-cli"))
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn qglang cli");
    process::wait_bounded(child, None, Duration::from_secs(10)).expect("bounded qglang cli")
}

pub(crate) fn cli_stdout_v1(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf-8")
}

pub(crate) fn cli_stderr_v1(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr utf-8")
}
