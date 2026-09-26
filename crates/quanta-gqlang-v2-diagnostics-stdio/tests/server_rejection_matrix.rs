#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "protocol tests inspect exact JSON response fields"
)]

use quanta_gqlang_v2_diagnostics_stdio::server::DiagnosticsServerV2;
use serde_json::Value;

fn dispatch(line: &str) -> Result<Value, (&'static str, String)> {
    DiagnosticsServerV2::try_standard()
        .expect("standard server")
        .dispatch(line.as_bytes())
        .map(|bytes| serde_json::from_slice(&bytes).expect("response json"))
        .map_err(|error| (error.code, error.message))
}

#[test]
fn method_rejection_retains_a_valid_request_id() {
    let response = dispatch(
        r#"{"id":"request-7","method":"shutdown","source_id":1,"source":"show capabilities"}"#,
    )
    .expect("bounded method error response");
    assert_eq!(response["id"], "request-7");
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["code"], "method_not_allowed");
}

/// The command plane is closed to exactly the advertised read-only allowlist:
/// every other method name (including retired protocol verbs) is rejected.
#[test]
fn only_the_advertised_read_only_method_is_admitted() {
    assert_eq!(
        quanta_gqlang_v2_diagnostics_stdio::server::READ_ONLY_METHOD_NAMES_V2,
        ["v3_diagnostics"]
    );
    for method in [
        "shutdown",
        "initialize",
        "hover",
        "completion",
        "symbols",
        "diagnostics",
        "zero_query",
    ] {
        let line =
            format!(r#"{{"id":1,"method":"{method}","source_id":1,"source":"show capabilities"}}"#);
        let response = dispatch(&line).expect("bounded method error response");
        assert_eq!(response["ok"], false, "method {method}");
        assert_eq!(
            response["error"]["code"], "method_not_allowed",
            "method {method}"
        );
    }
}

#[test]
fn only_string_and_number_ids_are_admitted() {
    for (id, expected) in [
        (r#""text""#, serde_json::json!("text")),
        ("0", serde_json::json!(0)),
        ("-1", serde_json::json!(-1)),
        ("1.25", serde_json::json!(1.25)),
    ] {
        let line = format!(
            r#"{{"id":{id},"method":"v3_diagnostics","source_id":1,"source":"show capabilities"}}"#
        );
        let response = dispatch(&line).expect("valid id");
        assert_eq!(response["ok"], true, "id {id}");
        assert_eq!(
            response["id"], expected,
            "id {id} must be preserved exactly"
        );
    }

    for id in ["null", "true", "false", "[]", "{}"] {
        let line = format!(
            r#"{{"id":{id},"method":"v3_diagnostics","source_id":1,"source":"show capabilities"}}"#
        );
        let error = dispatch(&line).expect_err("invalid id");
        assert_eq!(error.0, "invalid_id", "id {id}");
        assert_eq!(error.1, "id must be a JSON string or number");
    }
}

#[test]
fn required_fields_types_ranges_and_unknown_fields_fail_as_invalid_request() {
    let cases = [
        (r#"{"method":"v3_diagnostics","source_id":1,"source":"x"}"#, "missing field `id` at line 1 column 54"),
        (r#"{"id":1,"source_id":1,"source":"x"}"#, "missing field `method` at line 1 column 35"),
        (r#"{"id":1,"method":"v3_diagnostics","source":"x"}"#, "missing field `source_id` at line 1 column 47"),
        (r#"{"id":1,"method":"v3_diagnostics","source_id":1}"#, "missing field `source` at line 1 column 48"),
        (r#"{"id":1,"method":7,"source_id":1,"source":"x"}"#, "invalid type: integer `7`, expected a string at line 1 column 18"),
        (r#"{"id":1,"method":"v3_diagnostics","source_id":-1,"source":"x"}"#, "invalid value: integer `-1`, expected u32 at line 1 column 48"),
        (r#"{"id":1,"method":"v3_diagnostics","source_id":4294967296,"source":"x"}"#, "invalid value: integer `4294967296`, expected u32 at line 1 column 56"),
        (r#"{"id":1,"method":"v3_diagnostics","source_id":1,"source":[]}"#, "invalid type: sequence, expected a string at line 1 column 57"),
        (r#"{"id":1,"method":"v3_diagnostics","source_id":1,"source":[],"extra":0}"#, "invalid type: sequence, expected a string at line 1 column 57"),
        (r#"{"id":1,"method":"v3_diagnostics","source_id":1,"source":"x","extra":0}"#, "unknown field `extra`, expected one of `id`, `method`, `source_id`, `source` at line 1 column 68"),
    ];
    for (line, expected_message) in cases {
        let error = dispatch(line).expect_err("schema-invalid request");
        assert_eq!(error.0, "invalid_request", "request {line}");
        assert_eq!(error.1, expected_message, "request {line}");
    }
}
