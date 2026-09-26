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
fn source_limit_counts_decoded_utf8_bytes_not_scalars_or_json_escape_bytes() {
    let source = "let x = \"한🙂\"";
    assert_eq!(source.len(), 17);
    assert_eq!(source.chars().count(), 12);
    let exact = format!("{source} ");
    let over = format!("{source}  ");
    assert_eq!(exact.len(), 18);
    assert_eq!(over.len(), 19);

    let mut input = Vec::new();
    for (id, text) in [(0, source), (1, exact.as_str()), (2, over.as_str())] {
        let request = serde_json::json!({
            "id": id, "method": "v3_diagnostics", "source_id": 7, "source": text,
        });
        input.extend_from_slice(&serde_json::to_vec(&request).expect("request json"));
        input.push(b'\n');
    }
    // Same decoded exact-limit source, with different physical JSON bytes.
    input.extend_from_slice(
        br#"{"id":3,"method":"v3_diagnostics","source_id":7,"source":"let x = \"\uD55C\uD83D\uDE42\" "}"#,
    );
    input.push(b'\n');
    input.extend_from_slice(br#"{"id":4,"method":"v3_diagnostics","source_id":7,"source":""}"#);
    let (status, responses) = run(&input, budget(18, 256, 16, 4_096));
    assert_eq!(status, 0, "source rejection is a serialized request error");
    assert_eq!(responses.len(), 5);
    for (id, response) in responses.iter().enumerate() {
        assert_eq!(response["id"], id);
        assert_eq!(response["ok"], id != 2);
    }
    assert_eq!(responses[2]["error"]["code"], "source_limit_exceeded");
    assert_eq!(
        responses[2]["error"]["message"],
        "source exceeds max_source_bytes 18"
    );
    let mut escaped = responses[3].clone();
    escaped["id"] = serde_json::json!(1);
    assert_eq!(
        responses[1], escaped,
        "JSON escaping preserves decoded input"
    );
}

#[test]
fn response_limit_counts_serialized_utf8_bytes_and_emits_bounded_fallback() {
    let request = r#"{"id":"응🙂","method":"한🙂한🙂","source_id":7,"source":""}"#.as_bytes();
    // Independent wire golden, including multibyte ID and method text.
    let expected = r#"{"id":"응🙂","ok":false,"error":{"code":"method_not_allowed","message":"method `한🙂한🙂` is not allowed"}}"#;
    assert_eq!(expected.len(), 116);
    assert_eq!(expected.chars().count(), 101);
    let exact =
        DiagnosticsServerV2::try_with_budget(budget(64, 256, 16, 116)).expect("exact byte budget");
    assert_eq!(
        exact.dispatch(request).expect("116 bytes admitted"),
        expected.as_bytes()
    );

    let below = DiagnosticsServerV2::try_with_budget(budget(64, 256, 16, 115))
        .expect("one-under byte budget can still hold fallback");
    let error = below.dispatch(request).expect_err("116 bytes exceed 115");
    assert_eq!(error.code, "response_limit_exceeded");
    assert_eq!(
        error.message,
        "serialized response exceeds max_response_bytes 115"
    );

    let mut input = Cursor::new(request);
    let mut output = Vec::new();
    let status =
        run_stdio_session_with_budget_v2(&mut input, &mut output, budget(64, 256, 16, 115))
            .expect("bounded fallback session");
    assert_eq!(status, 1);
    assert_eq!(
        output,
        b"{\"id\":null,\"ok\":false,\"error\":{\"code\":\"response_limit_exceeded\",\"message\":\"response exceeds configured limit\"}}\n"
    );
    assert!(output.len() <= 115, "fallback plus framing stays bounded");
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
fn overflowing_final_frame_without_newline_emits_one_error() {
    for prefix in [false, true] {
        let mut input = Vec::new();
        if prefix {
            input.extend_from_slice(
                br#"{"id":9,"method":"v3_diagnostics","source_id":1,"source":""}"#,
            );
            input.push(b'\n');
        }
        input.extend_from_slice(&padded_object(257));
        // Exact first overflow clears the buffered line, leaving only the flag
        // at EOF. No newline or further byte may be required to emit its error.
        for one_byte in [false, true] {
            let mut output = Vec::new();
            let limits = budget(64, 256, 16, 4_096);
            let status = if one_byte {
                let mut reader = OneByteReader {
                    bytes: input.clone(),
                    cursor: 0,
                };
                run_stdio_session_with_budget_v2(&mut reader, &mut output, limits)
            } else {
                let mut reader = Cursor::new(&input);
                run_stdio_session_with_budget_v2(&mut reader, &mut output, limits)
            }
            .expect("overflow at EOF session");
            assert_eq!(status, 1);
            let responses: Vec<Value> = output
                .split(|byte| *byte == b'\n')
                .filter(|line| !line.is_empty())
                .map(|line| serde_json::from_slice(line).expect("response json"))
                .collect();
            assert_eq!(responses.len(), if prefix { 2 } else { 1 });
            if prefix {
                assert_eq!(responses[0]["id"], 9);
                assert_eq!(responses[0]["ok"], true);
            }
            assert_eq!(
                responses.last().expect("overflow response"),
                &serde_json::json!({
                    "id": null, "ok": false,
                    "error": {
                        "code": "frame_limit_exceeded",
                        "message": "request exceeds max_frame_bytes 256",
                    },
                })
            );
            assert_eq!(output.last(), Some(&b'\n'));
        }
    }
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
