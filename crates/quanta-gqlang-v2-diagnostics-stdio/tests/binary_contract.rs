#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "binary contract tests require successful process plumbing"
)]

use std::process::{Command, Output, Stdio};
use std::time::Duration;

#[path = "../../../tools/test_support/process.rs"]
mod process;

fn run(input: &[u8]) -> Output {
    let child = Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-diagnostics-stdio"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn diagnostics stdio");
    process::wait_bounded(child, Some(input.to_vec()), Duration::from_secs(10))
        .expect("bounded diagnostics stdio")
}

#[test]
fn protocol_failures_use_stdout_and_make_the_process_fail() {
    let output = run(b"{not json}\n");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stderr.is_empty());
    let response: serde_json::Value =
        serde_json::from_slice(output.stdout.strip_suffix(b"\n").expect("newline"))
            .expect("response json");
    assert_eq!(response["id"], serde_json::Value::Null);
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["code"], "invalid_request");
}

#[test]
fn semantic_diagnostics_are_a_successful_protocol_response() {
    let output = run(
        br#"{"id":3,"method":"v3_diagnostics","source_id":1,"source":"totally not a qglang root"}
"#,
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let response: serde_json::Value =
        serde_json::from_slice(output.stdout.strip_suffix(b"\n").expect("newline"))
            .expect("response json");
    assert_eq!(response["id"], 3);
    assert_eq!(response["ok"], true);
    assert_eq!(response["analysis_status"], "error");
    let diagnostics = response["diagnostics"]
        .as_array()
        .expect("diagnostics array");
    assert_eq!(diagnostics.len(), 1, "unexpected diagnostics: {response}");
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic["stage"], "integrity");
    assert_eq!(diagnostic["code"], "surface.parse.unexpected_token");
    assert_eq!(diagnostic["severity"], "error");
    assert_eq!(
        diagnostic["message"],
        "expected a top-level item ('let', 'rule', 'from', or a v3 family keyword)"
    );
    assert_eq!(diagnostic["source_id"], 1);
    assert_eq!(diagnostic["span_start"], 0);
    assert_eq!(diagnostic["span_end"], 7);
}

#[cfg(unix)]
#[test]
fn broken_stdout_is_reported_as_an_io_error_on_stderr() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-diagnostics-stdio"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn diagnostics stdio");
    drop(child.stdout.take().expect("child stdout"));
    let output = process::wait_bounded(
        child,
        Some(
            br#"{"id":3,"method":"v3_diagnostics","source_id":1,"source":"show capabilities"}
"#
            .to_vec(),
        ),
        Duration::from_secs(10),
    )
    .expect("bounded diagnostics stdio");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let response: serde_json::Value =
        serde_json::from_slice(output.stderr.strip_suffix(b"\n").expect("newline"))
            .expect("stderr response json");
    assert_eq!(response["id"], serde_json::Value::Null);
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["code"], "io_error");
}
