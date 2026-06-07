mod support;

use support::{assert_read_only_cli_library_round_trip_v1, ExampleRoundTripExpectationV1};

#[test]
fn declaration_a_examples_round_trip_through_read_only_cli_library_v1() {
    let cases = [
        ExampleRoundTripExpectationV1 {
            example_relative: "examples/compare/semantic_delta.qglang",
            syntax_kind: "compare",
            syntax_summary: "payload=baseline=baseline::nightly-2026-06-07 ;; candidate=result::catalog-users ;; metric=semantic",
            typed_kind: "sample_compare_export",
            typed_summary_fragments: &[
                "Compare {",
                "nightly-2026-06-07",
                "catalog-users",
                "metric=semantic",
            ],
            ir_kind: "interactive",
            ir_summary: "compare baseline=baseline::nightly-2026-06-07 candidate=result::catalog-users metric=semantic",
            degraded_reason: Some("catalog-and-interactive-host-truth-unavailable"),
        },
        ExampleRoundTripExpectationV1 {
            example_relative: "examples/view/catalog_users.qglang",
            syntax_kind: "view",
            syntax_summary: "carrier=catalog.users ;; query-ref=queries/catalog/users",
            typed_kind: "declaration",
            typed_summary_fragments: &[
                "View(",
                "catalog.users",
                "QueryRef(\"queries/catalog/users\")",
            ],
            ir_kind: "declaration",
            ir_summary: "view name=catalog.users query-ref=queries/catalog/users",
            degraded_reason: None,
        },
        ExampleRoundTripExpectationV1 {
            example_relative: "examples/rule/shell_exec.qglang",
            syntax_kind: "rule",
            syntax_summary: "carrier=shell.exec ;; check-ref=checks/shell/exec",
            typed_kind: "declaration",
            typed_summary_fragments: &["Rule(", "shell.exec", "checks/shell/exec"],
            ir_kind: "declaration",
            ir_summary: "rule name=shell.exec check-ref=checks/shell/exec",
            degraded_reason: None,
        },
    ];

    for expectation in &cases {
        assert_read_only_cli_library_round_trip_v1(expectation);
    }
}
