#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "boundary tests inspect exact response positions"
)]

use std::io::{Cursor, Read};

use quanta_gqlang_v2_diagnostics_stdio::{
    run_stdio_session_with_budget_v2, server::DiagnosticsServerV2,
};
use quanta_gqlang_v2_frontend_contract::FrontendBudgetV2;
use serde_json::Value;

fn budget(source: u32, frame: u32, diagnostics: u32, response: u32) -> FrontendBudgetV2 {
    FrontendBudgetV2::try_new(
        source,
        frame,
        1_024,
        64,
        32,
        32,
        2_048,
        64,
        64,
        10_000,
        128,
        256,
        10_000,
        4_096,
        1_024,
        128,
        256,
        10_000,
        64,
        128,
        diagnostics,
        response,
    )
    .expect("test budget")
}

fn run(input: &[u8], budget: FrontendBudgetV2) -> (u8, Vec<Value>) {
    let mut reader = Cursor::new(input);
    let mut writer = Vec::new();
    let status = run_stdio_session_with_budget_v2(&mut reader, &mut writer, budget)
        .expect("bounded session");
    let responses = writer
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).expect("response json"))
        .collect();
    (status, responses)
}

fn padded_object(bytes: usize) -> Vec<u8> {
    let prefix = br#"{"id":1,"method":"v3_diagnostics","source_id":1,"source":"x","pad":""#;
    let suffix = br#""}"#;
    assert!(bytes >= prefix.len() + suffix.len());
    let mut output = prefix.to_vec();
    output.resize(bytes - suffix.len(), b'a');
    output.extend_from_slice(suffix);
    output
}

struct OneByteReader {
    bytes: Vec<u8>,
    cursor: usize,
}

impl Read for OneByteReader {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        if self.cursor == self.bytes.len() {
            return Ok(0);
        }
        output[0] = self.bytes[self.cursor];
        self.cursor += 1;
        Ok(1)
    }
}

#[test]
fn frame_exact_limit_is_parsed_and_limit_plus_one_is_rejected_then_recovers() {
    let mut input = padded_object(255);
    input.push(b'\n');
    input.extend_from_slice(&padded_object(256));
    input.push(b'\n');
    input.extend_from_slice(&padded_object(257));
    input.push(b'\n');
    input.extend_from_slice(br#"{"id":3,"method":"v3_diagnostics","source_id":1,"source":"x"}"#);
    input.push(b'\n');
    let (status, responses) = run(&input, budget(64, 256, 16, 4_096));
    assert_eq!(status, 1);
    assert_eq!(responses.len(), 4);
    assert_eq!(responses[0]["error"]["code"], "invalid_request");
    assert_eq!(responses[1]["error"]["code"], "invalid_request");
    assert_eq!(responses[2]["error"]["code"], "frame_limit_exceeded");
    assert_eq!(responses[3]["id"], 3);
}

#[test]
fn source_exact_limit_is_admitted_and_plus_one_is_rejected() {
    let under = " ".repeat(31);
    let exact = " ".repeat(32);
    let over = " ".repeat(33);
    let input = format!(
        "{{\"id\":0,\"method\":\"v3_diagnostics\",\"source_id\":1,\"source\":{}}}\n{{\"id\":1,\"method\":\"v3_diagnostics\",\"source_id\":1,\"source\":{}}}\n{{\"id\":2,\"method\":\"v3_diagnostics\",\"source_id\":1,\"source\":{}}}\n",
        serde_json::to_string(&under).expect("under json"),
        serde_json::to_string(&exact).expect("exact json"),
        serde_json::to_string(&over).expect("over json")
    );
    let (status, responses) = run(input.as_bytes(), budget(32, 256, 16, 4_096));
    assert_eq!(status, 0);
    assert_eq!(responses[0]["ok"], true);
    assert_eq!(responses[1]["ok"], true);
    assert_eq!(responses[2]["error"]["code"], "source_limit_exceeded");
}

#[test]
fn diagnostic_and_response_caps_fail_closed() {
    let request =
        br#"{"id":1,"method":"v3_diagnostics","source_id":1,"source":"show capabilities"}"#;
    let exact_diagnostic_server = DiagnosticsServerV2::try_with_budget(budget(64, 256, 3, 4_096))
        .expect("exact diagnostic server");
    let exact_diagnostic = exact_diagnostic_server
        .dispatch(request)
        .expect("exact diagnostic cap admits response");
    let exact_diagnostic: Value =
        serde_json::from_slice(&exact_diagnostic).expect("exact diagnostic json");
    assert_eq!(
        exact_diagnostic["diagnostics"]
            .as_array()
            .expect("diagnostic array")
            .len(),
        3
    );

    let diagnostic_server =
        DiagnosticsServerV2::try_with_budget(budget(64, 256, 2, 4_096)).expect("diagnostic server");
    let diagnostic = diagnostic_server
        .dispatch(request)
        .expect("bounded error response");
    let diagnostic: Value = serde_json::from_slice(&diagnostic).expect("diagnostic json");
    assert_eq!(diagnostic["error"]["code"], "diagnostic_limit_exceeded");

    let full_response = exact_diagnostic_server
        .dispatch(request)
        .expect("measure full response");
    let exact_response_bytes = u32::try_from(full_response.len()).expect("small response");
    let exact_response_server =
        DiagnosticsServerV2::try_with_budget(budget(64, 256, 16, exact_response_bytes))
            .expect("exact response server");
    assert_eq!(
        exact_response_server
            .dispatch(request)
            .expect("exact response cap"),
        full_response
    );
    let response_server =
        DiagnosticsServerV2::try_with_budget(budget(64, 256, 16, exact_response_bytes - 1))
            .expect("one-under response server");
    let response_error = response_server
        .dispatch(request)
        .expect_err("response must exceed cap");
    assert_eq!(response_error.code, "response_limit_exceeded");

    let invalid = DiagnosticsServerV2::try_with_budget(budget(64, 256, 16, 110))
        .expect_err("transport must reserve space for its bounded fallback");
    assert_eq!(invalid.code, "configuration_error");
}

#[test]
fn overflow_spanning_many_reads_discards_only_that_frame_and_recovers() {
    let mut input = padded_object(257);
    input.push(b'\n');
    input.extend_from_slice(br#"{"id":9,"method":"v3_diagnostics","source_id":1,"source":"x"}"#);
    let mut reader = OneByteReader {
        bytes: input,
        cursor: 0,
    };
    let mut writer = Vec::new();
    let status =
        run_stdio_session_with_budget_v2(&mut reader, &mut writer, budget(64, 256, 16, 4_096))
            .expect("bounded session");
    let responses: Vec<Value> = writer
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).expect("response json"))
        .collect();
    assert_eq!(status, 1);
    assert_eq!(responses.len(), 2);
    assert_eq!(responses[0]["error"]["code"], "frame_limit_exceeded");
    assert_eq!(responses[1]["id"], 9);
    assert_eq!(responses[1]["ok"], true);
}

#[test]
fn response_fallback_has_an_exact_admission_boundary_and_is_emitted() {
    const FALLBACK: &[u8] = br#"{"id":null,"ok":false,"error":{"code":"response_limit_exceeded","message":"response exceeds configured limit"}}"#;
    let exact = u32::try_from(FALLBACK.len()).expect("small fallback");
    let server = DiagnosticsServerV2::try_with_budget(budget(64, 256, 16, exact))
        .expect("exact fallback budget");
    let detailed = server.render_transport_error(
        quanta_gqlang_v2_diagnostics_stdio::server::ProtocolErrorV2::new(
            "response_limit_exceeded",
            "a deliberately verbose response-limit error that cannot fit in the fallback budget",
        ),
    );
    assert_eq!(detailed, FALLBACK);
    assert_eq!(
        detailed.len(),
        usize::try_from(exact).expect("usize fallback")
    );

    let request =
        br#"{"id":1,"method":"v3_diagnostics","source_id":1,"source":"show capabilities"}"#;
    let mut input = Cursor::new(request);
    let mut output = Vec::new();
    let status =
        run_stdio_session_with_budget_v2(&mut input, &mut output, budget(64, 256, 16, exact))
            .expect("fallback session");
    assert_eq!(status, 1);
    let mut expected = FALLBACK.to_vec();
    expected.push(b'\n');
    assert_eq!(output, expected);

    let below = DiagnosticsServerV2::try_with_budget(budget(64, 256, 16, exact - 1))
        .expect_err("one byte below fallback must reject");
    assert_eq!(below.code, "configuration_error");
}
