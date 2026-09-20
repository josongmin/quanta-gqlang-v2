#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]
#![allow(
    dead_code,
    reason = "shared stdio helpers are used by a subset of the integration-test crates that include this module"
)]

use std::io::{Cursor, Write};
use std::process::{Command, Output, Stdio};

use quanta_gqlang_v2_diagnostics_stdio::run_stdio_session_v1;

/// Drive the read-only stdio session entirely in-process over an exact byte
/// buffer. This imposes **no** implicit framing, so callers can pin EOF,
/// glued-frame, and
/// incomplete-frame behavior byte-for-byte. Returns `(exit status, stdout)`.
pub(crate) fn run_session_bytes_v1(input: &[u8]) -> (u8, String) {
    let mut reader = Cursor::new(input.to_vec());
    let mut output = Vec::new();
    let status = run_stdio_session_v1(&mut reader, &mut output).expect("stdio session runs");
    (status, String::from_utf8(output).expect("stdout utf-8"))
}

/// Feed exact bytes to the **spawned binary** front door, close stdin, and
/// collect the full process result. Models a real client that may omit a
/// trailing newline, glue several frames together, or truncate mid-object.
pub(crate) fn run_binary_bytes_v1(input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-diagnostics-stdio"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("binary should spawn");
    {
        let mut stdin = child.stdin.take().expect("stdin pipe");
        stdin.write_all(input).expect("write stdin");
    } // drop closes stdin -> the session observes EOF
    child.wait_with_output().expect("collect binary output")
}

/// Send a single newline-delimited request to the spawned stdio front door and
/// return `(exit code, trimmed single-line response)`.
pub(crate) fn run_binary_line_v1(request: &str) -> (Option<i32>, String) {
    let mut framed = String::from(request);
    framed.push('\n');
    let output = run_binary_bytes_v1(framed.as_bytes());
    (
        output.status.code(),
        binary_stdout_v1(&output).trim().to_string(),
    )
}

pub(crate) fn binary_stdout_v1(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout utf-8")
}

pub(crate) fn binary_stderr_v1(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr utf-8")
}

/// Split a newline-delimited response transcript into its non-empty JSON lines.
pub(crate) fn response_lines_v1(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect()
}
