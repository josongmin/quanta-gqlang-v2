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
    for id in [r#""text""#, "0", "-1", "1.25"] {
        let line = format!(
            r#"{{"id":{id},"method":"v3_diagnostics","source_id":1,"source":"show capabilities"}}"#
        );
        let response = dispatch(&line).expect("valid id");
        assert_eq!(response["ok"], true, "id {id}");
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
        r#"{"method":"v3_diagnostics","source_id":1,"source":"x"}"#,
        r#"{"id":1,"source_id":1,"source":"x"}"#,
        r#"{"id":1,"method":"v3_diagnostics","source":"x"}"#,
        r#"{"id":1,"method":"v3_diagnostics","source_id":1}"#,
        r#"{"id":1,"method":7,"source_id":1,"source":"x"}"#,
        r#"{"id":1,"method":"v3_diagnostics","source_id":-1,"source":"x"}"#,
        r#"{"id":1,"method":"v3_diagnostics","source_id":4294967296,"source":"x"}"#,
        r#"{"id":1,"method":"v3_diagnostics","source_id":1,"source":[]}"#,
        r#"{"id":1,"method":"v3_diagnostics","source_id":1,"source":[],"extra":0}"#,
        r#"{"id":1,"method":"v3_diagnostics","source_id":1,"source":"x","extra":0}"#,
    ];
    for line in cases {
        let error = dispatch(line).expect_err("schema-invalid request");
        assert_eq!(error.0, "invalid_request", "request {line}");
        assert!(!error.1.is_empty());
    }
}
