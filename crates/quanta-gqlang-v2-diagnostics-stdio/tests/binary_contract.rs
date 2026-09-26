#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "binary contract tests require successful process plumbing"
)]

use std::io::Write;
use std::process::{Command, Output, Stdio};

fn run(input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_quanta-gqlang-v2-diagnostics-stdio"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn diagnostics stdio");
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(input)
        .expect("write request");
    child
        .wait_with_output()
        .expect("wait for diagnostics stdio")
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
    assert!(
        response["diagnostics"]
            .as_array()
            .is_some_and(|diagnostics| !diagnostics.is_empty()),
        "an error analysis must carry at least one diagnostic: {response}"
    );
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
    child
        .stdin
        .take()
        .expect("child stdin")
        .write_all(
            br#"{"id":3,"method":"v3_diagnostics","source_id":1,"source":"show capabilities"}
"#,
        )
        .expect("write request");
    let output = child
        .wait_with_output()
        .expect("wait for diagnostics stdio");
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let response: serde_json::Value =
        serde_json::from_slice(output.stderr.strip_suffix(b"\n").expect("newline"))
            .expect("stderr response json");
    assert_eq!(response["id"], serde_json::Value::Null);
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["code"], "io_error");
}
