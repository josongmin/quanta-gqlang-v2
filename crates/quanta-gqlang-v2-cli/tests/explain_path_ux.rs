#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_cli::explain_view::{
    build_explain_view_v1, build_find_explain_view_v1, DegradedBadgeV1, ExplainViewV1,
    PathWitnessStepV1, PathWitnessViewV1,
};
use quanta_gqlang_v2_ir::explain_ir::{
    ExplainEvidenceModeIrV1, ExplainShapeIrV1, ExplainTruthExpectationIrV1,
};
use quanta_gqlang_v2_ir::find_ir::{
    FindExecutionPlanIrV1, FindFlowWindowIrV1, FlowPlanIrV1, ReachabilityPlanIrV1,
};
use quanta_gqlang_v2_cli::NO_RESULT_REASON_V1;

#[test]
fn explain_view_surfaces_evidence_truth_and_degraded_badges() {
    let view = build_explain_view_v1(
        &ExplainShapeIrV1 {
            target: "view::authz".to_string(),
            evidence: ExplainEvidenceModeIrV1::Counterexample,
            truth_expectation: ExplainTruthExpectationIrV1::Refute,
        },
        &[
            "truth-evidence-materialization-unavailable".to_string(),
            "runtime-host-routing-unavailable".to_string(),
        ],
        Some(NO_RESULT_REASON_V1),
    );

    assert_eq!(
        view,
        ExplainViewV1 {
            subject: "view::authz".to_string(),
            evidence: "counterexample",
            truth: "refute",
            degraded_badges: vec![
                DegradedBadgeV1 {
                    code: "truth-evidence-materialization-unavailable".to_string(),
                    label: "Truth materialization unavailable".to_string(),
                },
                DegradedBadgeV1 {
                    code: "runtime-host-routing-unavailable".to_string(),
                    label: "Routing unavailable".to_string(),
                },
            ],
            path_witness: None,
            why_no_result: Some(NO_RESULT_REASON_V1.to_string()),
        }
    );
}

#[test]
fn reachability_path_witness_preserves_source_via_sink_order() {
    let view = build_find_explain_view_v1(
        &FindExecutionPlanIrV1::Reachability(ReachabilityPlanIrV1 {
            source: "api.entry".to_string(),
            sink: "db.write".to_string(),
            required_waypoints: vec!["auth".to_string(), "repo".to_string()],
            max_hops: Some(3),
        }),
        &["runtime-host-routing-unavailable".to_string()],
        Some(NO_RESULT_REASON_V1),
    );

    assert_eq!(
        view.path_witness,
        Some(PathWitnessViewV1 {
            kind: "reachability",
            steps: vec![
                PathWitnessStepV1 {
                    label: "source".to_string(),
                    detail: "api.entry".to_string(),
                },
                PathWitnessStepV1 {
                    label: "via[0]".to_string(),
                    detail: "auth".to_string(),
                },
                PathWitnessStepV1 {
                    label: "via[1]".to_string(),
                    detail: "repo".to_string(),
                },
                PathWitnessStepV1 {
                    label: "sink".to_string(),
                    detail: "db.write".to_string(),
                },
            ],
            note: Some("bounded by max_hops=3".to_string()),
        })
    );
}

#[test]
fn flow_witness_stays_external_and_never_claims_runtime_truth() {
    let view = build_find_explain_view_v1(
        &FindExecutionPlanIrV1::Flow(FlowPlanIrV1 {
            source: "ingress".to_string(),
            sink: "egress".to_string(),
            stages: vec!["ssa".to_string(), "cfg".to_string()],
            window: FindFlowWindowIrV1::Function,
        }),
        &["semantic-confirm-lane-unavailable".to_string()],
        Some(NO_RESULT_REASON_V1),
    );

    assert_eq!(view.subject, "ingress -> egress");
    assert_eq!(view.evidence, "trace");
    assert_eq!(view.truth, "inspect");
    assert_eq!(
        view.why_no_result.as_deref(),
        Some(NO_RESULT_REASON_V1)
    );
    assert!(view
        .degraded_badges
        .iter()
        .any(|badge| badge.code == "semantic-confirm-lane-unavailable"));
}
