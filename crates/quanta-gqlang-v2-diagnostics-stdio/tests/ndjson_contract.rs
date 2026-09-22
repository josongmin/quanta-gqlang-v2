#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "wire contract tests inspect exact response positions"
)]

use std::io::Cursor;

use quanta_gqlang_v2_diagnostics_stdio::run_stdio_session_v2;
use serde_json::Value;

fn run(bytes: &[u8]) -> (u8, Vec<Value>) {
    let mut input = Cursor::new(bytes);
    let mut output = Vec::new();
    let status = run_stdio_session_v2(&mut input, &mut output).expect("stdio session");
    let values = output
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).expect("response json"))
        .collect();
    (status, values)
}

fn valid(id: u32) -> String {
    format!(r#"{{"id":{id},"method":"v3_diagnostics","source_id":7,"source":"show capabilities"}}"#)
}

#[test]
fn valid_request_echoes_id_and_reports_honest_incomplete_analysis() {
    let request = format!("{}\n", valid(41));
    let (status, responses) = run(request.as_bytes());
    assert_eq!(status, 0);
    assert_eq!(responses.len(), 1);
    assert_eq!(responses[0]["id"], 41);
    assert_eq!(responses[0]["ok"], true);
    assert_eq!(responses[0]["method"], "v3_diagnostics");
    assert_eq!(responses[0]["analysis_status"], "incomplete");
    assert!(responses[0]["diagnostics"]
        .as_array()
        .is_some_and(|items| !items.is_empty()));
}

#[test]
fn malformed_invalid_utf8_and_duplicate_lines_do_not_poison_following_requests() {
    let mut stream = Vec::new();
    stream.extend_from_slice(b"{not json}\n");
    stream.extend_from_slice(b"\xff\n");
    stream.extend_from_slice(
        br#"{"id":1,"id":2,"method":"v3_diagnostics","source_id":1,"source":"x"}"#,
    );
    stream.push(b'\n');
    stream.extend_from_slice(valid(9).as_bytes());
    stream.push(b'\n');

    let (status, responses) = run(&stream);
    assert_eq!(status, 1);
    assert_eq!(responses.len(), 4);
    for response in &responses[..3] {
        assert_eq!(response["ok"], false);
        assert_eq!(response["id"], Value::Null);
        assert_eq!(response["error"]["code"], "invalid_request");
    }
    assert_eq!(responses[3]["id"], 9);
    assert_eq!(responses[3]["ok"], true);
}

#[test]
fn protocol_is_one_object_per_line_not_brace_balanced_framing() {
    let glued = format!("{}{}\n{}\n", valid(1), valid(2), valid(3));
    let (status, responses) = run(glued.as_bytes());
    assert_eq!(status, 1);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["ok"], false);
    assert_eq!(responses[1]["id"], 3);
}

#[test]
fn response_is_byte_deterministic() {
    let request = format!("{}\n", valid(5));
    let mut first_input = Cursor::new(request.as_bytes());
    let mut first = Vec::new();
    let mut second_input = Cursor::new(request.as_bytes());
    let mut second = Vec::new();
    assert_eq!(
        run_stdio_session_v2(&mut first_input, &mut first).expect("first"),
        0
    );
    assert_eq!(
        run_stdio_session_v2(&mut second_input, &mut second).expect("second"),
        0
    );
    assert_eq!(first, second);
}
