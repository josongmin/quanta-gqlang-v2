#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

mod support;

use support::{
    assert_cli_fmt_round_trip_v1, assert_read_only_cli_binary_round_trip_v1,
    BinaryFrontDoorExpectationV1, SHIPPED_EXAMPLE_DOC_PATHS_V1,
};

#[test]
fn shipped_examples_round_trip_through_cli_binary_front_door_v1() {
    let cases = [
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/select/basic.qglang",
            syntax_kind: "select",
            typed_kind: "select",
            ir_kind: "select",
            degraded_reason: None,
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/find/taint.qglang",
            syntax_kind: "find",
            typed_kind: "find",
            ir_kind: "find",
            degraded_reason: Some("runtime-host-routing-unavailable"),
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/check/hybrid_rule.qglang",
            syntax_kind: "check",
            typed_kind: "check",
            ir_kind: "check",
            degraded_reason: Some("semantic-confirm-lane-unavailable"),
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/explain/why_no_result.qglang",
            syntax_kind: "explain",
            typed_kind: "explain",
            ir_kind: "explain",
            degraded_reason: Some("truth-evidence-materialization-unavailable"),
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/show/capabilities.qglang",
            syntax_kind: "show",
            typed_kind: "show_describe",
            ir_kind: "interactive",
            degraded_reason: Some("catalog-and-interactive-host-truth-unavailable"),
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/describe/rule_detail.qglang",
            syntax_kind: "describe",
            typed_kind: "show_describe",
            ir_kind: "interactive",
            degraded_reason: Some("catalog-and-interactive-host-truth-unavailable"),
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/sample/retrieval.qglang",
            syntax_kind: "sample",
            typed_kind: "sample_compare_export",
            ir_kind: "retrieval",
            degraded_reason: Some("retrieval-result-materialization-unavailable"),
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/export/sarif.qglang",
            syntax_kind: "export",
            typed_kind: "sample_compare_export",
            ir_kind: "interactive",
            degraded_reason: Some("catalog-and-interactive-host-truth-unavailable"),
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/compare/semantic_delta.qglang",
            syntax_kind: "compare",
            typed_kind: "sample_compare_export",
            ir_kind: "interactive",
            degraded_reason: Some("catalog-and-interactive-host-truth-unavailable"),
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/view/catalog_users.qglang",
            syntax_kind: "view",
            typed_kind: "declaration",
            ir_kind: "declaration",
            degraded_reason: None,
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/rule/shell_exec.qglang",
            syntax_kind: "rule",
            typed_kind: "declaration",
            ir_kind: "declaration",
            degraded_reason: None,
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/model/embeddings_code.qglang",
            syntax_kind: "model",
            typed_kind: "declaration",
            ir_kind: "declaration",
            degraded_reason: None,
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/pack/security_default.qglang",
            syntax_kind: "pack",
            typed_kind: "declaration",
            ir_kind: "declaration",
            degraded_reason: None,
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/profile/strict_review.qglang",
            syntax_kind: "profile",
            typed_kind: "declaration",
            ir_kind: "declaration",
            degraded_reason: None,
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/gate/release_gate.qglang",
            syntax_kind: "gate",
            typed_kind: "declaration",
            ir_kind: "declaration",
            degraded_reason: None,
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/use_pack/secure_pack.qglang",
            syntax_kind: "meta",
            typed_kind: "meta",
            ir_kind: "meta",
            degraded_reason: None,
        },
        BinaryFrontDoorExpectationV1 {
            example_relative: "examples/require_certified/strict_only.qglang",
            syntax_kind: "meta",
            typed_kind: "meta",
            ir_kind: "meta",
            degraded_reason: None,
        },
    ];

    for expectation in &cases {
        assert_read_only_cli_binary_round_trip_v1(expectation);
    }
}

#[test]
fn shipped_examples_are_canonically_formatted_on_the_cli_front_door_v1() {
    for example_relative in SHIPPED_EXAMPLE_DOC_PATHS_V1 {
        assert_cli_fmt_round_trip_v1(example_relative);
    }
}
