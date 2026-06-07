mod support;

use support::{assert_read_only_cli_library_round_trip_v1, ExampleRoundTripExpectationV1};

#[test]
fn declaration_examples_b_round_trip_through_cli_library_surface_v1() {
    let cases = [
        ExampleRoundTripExpectationV1 {
            example_relative: "examples/model/embeddings_code.qglang",
            syntax_kind: "model",
            syntax_summary: "carrier=embeddings.code ;; provider=openai ;; source=models/code-search",
            typed_kind: "declaration",
            typed_summary_fragments: &[
                "Model(",
                "embeddings.code",
                "provider: \"openai\"",
                "source: \"models/code-search\"",
            ],
            ir_kind: "declaration",
            ir_summary: "model name=embeddings.code provider=openai source=models/code-search",
            degraded_reason: None,
        },
        ExampleRoundTripExpectationV1 {
            example_relative: "examples/pack/security_default.qglang",
            syntax_kind: "pack",
            syntax_summary: "carrier=security-default ;; source=registry://packs/security-default ;; activation=workspace",
            typed_kind: "declaration",
            typed_summary_fragments: &[
                "Pack(",
                "security-default",
                "registry://packs/security-default",
                "Workspace",
            ],
            ir_kind: "declaration",
            ir_summary: "pack name=security-default source=registry://packs/security-default activation=workspace",
            degraded_reason: None,
        },
        ExampleRoundTripExpectationV1 {
            example_relative: "examples/gate/release_gate.qglang",
            syntax_kind: "gate",
            syntax_summary: "carrier=release-gate ;; checks=owasp.sql,authz.path ;; binding=all",
            typed_kind: "declaration",
            typed_summary_fragments: &["Gate(", "release-gate", "owasp.sql", "authz.path", "All"],
            ir_kind: "declaration",
            ir_summary: "gate name=release-gate checks=[\"owasp.sql\", \"authz.path\"] binding=all",
            degraded_reason: None,
        },
    ];

    for expectation in &cases {
        assert_read_only_cli_library_round_trip_v1(expectation);
    }
}
