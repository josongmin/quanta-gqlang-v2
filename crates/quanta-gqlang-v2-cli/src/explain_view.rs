use quanta_gqlang_v2_ir::explain_ir::{
    ExplainEvidenceModeIrV1, ExplainShapeIrV1, ExplainTruthExpectationIrV1,
};
use quanta_gqlang_v2_ir::find_ir::{
    FindExecutionPlanIrV1, FindFlowWindowIrV1, FindSliceDirectionIrV1, FindSummaryFacetIrV1,
};
use quanta_gqlang_v2_ir::root_ir::DegradedReasonV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DegradedBadgeV1 {
    pub code: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathWitnessStepV1 {
    pub label: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathWitnessViewV1 {
    pub kind: &'static str,
    pub steps: Vec<PathWitnessStepV1>,
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainViewV1 {
    pub subject: String,
    pub evidence: &'static str,
    pub truth: &'static str,
    pub degraded_badges: Vec<DegradedBadgeV1>,
    pub path_witness: Option<PathWitnessViewV1>,
    pub why_no_result: Option<String>,
}

pub fn build_explain_view_v1(
    shape: &ExplainShapeIrV1,
    degraded_reasons: &[String],
    why_no_result: Option<&str>,
) -> ExplainViewV1 {
    ExplainViewV1 {
        subject: shape.target.clone(),
        evidence: explain_evidence_label_v1(shape.evidence),
        truth: explain_truth_label_v1(shape.truth_expectation),
        degraded_badges: degraded_badges_v1(degraded_reasons),
        path_witness: None,
        why_no_result: why_no_result.map(ToOwned::to_owned),
    }
}

pub fn build_find_explain_view_v1(
    plan: &FindExecutionPlanIrV1,
    degraded_reasons: &[String],
    why_no_result: Option<&str>,
) -> ExplainViewV1 {
    ExplainViewV1 {
        subject: find_subject_v1(plan),
        evidence: "trace",
        truth: "inspect",
        degraded_badges: degraded_badges_v1(degraded_reasons),
        path_witness: Some(render_path_witness_v1(plan)),
        why_no_result: why_no_result.map(ToOwned::to_owned),
    }
}

pub fn render_path_witness_v1(plan: &FindExecutionPlanIrV1) -> PathWitnessViewV1 {
    match plan {
        FindExecutionPlanIrV1::Reachability(spec) => PathWitnessViewV1 {
            kind: "reachability",
            steps: path_sequence_v1(&spec.source, &spec.required_waypoints, &spec.sink, "via"),
            note: spec
                .max_hops
                .map(|depth| format!("bounded by max_hops={depth}")),
        },
        FindExecutionPlanIrV1::Flow(spec) => PathWitnessViewV1 {
            kind: "flow",
            steps: path_sequence_v1(&spec.source, &spec.stages, &spec.sink, "through"),
            note: Some(format!("window={}", flow_window_label_v1(spec.window))),
        },
        FindExecutionPlanIrV1::Slice(spec) => PathWitnessViewV1 {
            kind: "slice",
            steps: vec![
                PathWitnessStepV1 {
                    label: "anchor".to_string(),
                    detail: spec.anchor.clone(),
                },
                PathWitnessStepV1 {
                    label: "direction".to_string(),
                    detail: slice_direction_label_v1(spec.direction).to_string(),
                },
                PathWitnessStepV1 {
                    label: "frontier".to_string(),
                    detail: spec.frontier.clone().unwrap_or_else(|| "auto".to_string()),
                },
            ],
            note: None,
        },
        FindExecutionPlanIrV1::Taint(spec) => PathWitnessViewV1 {
            kind: "taint",
            steps: path_sequence_v1(&spec.source, &spec.blocked_by, &spec.sink, "sanitizer"),
            note: spec
                .max_hops
                .map(|depth| format!("bounded by max_hops={depth}")),
        },
        FindExecutionPlanIrV1::Path(spec) => PathWitnessViewV1 {
            kind: "path",
            steps: path_sequence_v1(
                &spec.source,
                &spec.required_waypoints,
                &spec.sink,
                "through",
            ),
            note: spec
                .max_hops
                .map(|depth| format!("bounded by max_hops={depth}")),
        },
        FindExecutionPlanIrV1::PointsTo(spec) => PathWitnessViewV1 {
            kind: "points_to",
            steps: vec![
                PathWitnessStepV1 {
                    label: "reference".to_string(),
                    detail: spec.reference.clone(),
                },
                PathWitnessStepV1 {
                    label: "pointee".to_string(),
                    detail: spec.expected_pointee.clone(),
                },
            ],
            note: spec.scope.as_ref().map(|scope| format!("scope={scope}")),
        },
        FindExecutionPlanIrV1::Summary(spec) => PathWitnessViewV1 {
            kind: "summary",
            steps: vec![
                PathWitnessStepV1 {
                    label: "callable".to_string(),
                    detail: spec.callable.clone(),
                },
                PathWitnessStepV1 {
                    label: "facets".to_string(),
                    detail: spec
                        .requested_facets
                        .iter()
                        .map(summary_facet_label_v1)
                        .collect::<Vec<_>>()
                        .join(","),
                },
            ],
            note: spec
                .expansion_depth
                .map(|depth| format!("bounded by expansion_depth={depth}")),
        },
    }
}

pub fn degraded_badges_v1(reasons: &[String]) -> Vec<DegradedBadgeV1> {
    reasons
        .iter()
        .map(|reason| {
            // Label keys off the shared DegradedReasonV1 authority (enum, not raw
            // string) so a code rename cannot silently desync this presenter.
            let label = match DegradedReasonV1::from_code_v1(reason) {
                Some(DegradedReasonV1::RuntimeHostRoutingUnavailable) => "Routing unavailable",
                Some(DegradedReasonV1::SemanticConfirmLaneUnavailable) => {
                    "Semantic confirm unavailable"
                }
                Some(DegradedReasonV1::TruthEvidenceMaterializationUnavailable) => {
                    "Truth materialization unavailable"
                }
                Some(DegradedReasonV1::RetrievalResultMaterializationUnavailable) => {
                    "Retrieval materialization unavailable"
                }
                Some(DegradedReasonV1::CatalogAndInteractiveHostTruthUnavailable) => {
                    "Catalog host truth unavailable"
                }
                None => "Degraded",
            };
            DegradedBadgeV1 {
                code: reason.clone(),
                label: label.to_string(),
            }
        })
        .collect()
}

fn find_subject_v1(plan: &FindExecutionPlanIrV1) -> String {
    match plan {
        FindExecutionPlanIrV1::Reachability(spec) => format!("{} -> {}", spec.source, spec.sink),
        FindExecutionPlanIrV1::Flow(spec) => format!("{} -> {}", spec.source, spec.sink),
        FindExecutionPlanIrV1::Slice(spec) => spec.anchor.clone(),
        FindExecutionPlanIrV1::Taint(spec) => format!("{} -> {}", spec.source, spec.sink),
        FindExecutionPlanIrV1::Path(spec) => format!("{} -> {}", spec.source, spec.sink),
        FindExecutionPlanIrV1::PointsTo(spec) => spec.reference.clone(),
        FindExecutionPlanIrV1::Summary(spec) => spec.callable.clone(),
    }
}

fn path_sequence_v1(
    source: &str,
    intermediates: &[String],
    sink: &str,
    middle_label: &str,
) -> Vec<PathWitnessStepV1> {
    let mut steps = vec![PathWitnessStepV1 {
        label: "source".to_string(),
        detail: source.to_string(),
    }];
    steps.extend(
        intermediates
            .iter()
            .enumerate()
            .map(|(index, item)| PathWitnessStepV1 {
                label: format!("{middle_label}[{index}]"),
                detail: item.clone(),
            }),
    );
    steps.push(PathWitnessStepV1 {
        label: "sink".to_string(),
        detail: sink.to_string(),
    });
    steps
}

fn explain_evidence_label_v1(mode: ExplainEvidenceModeIrV1) -> &'static str {
    match mode {
        ExplainEvidenceModeIrV1::Auto => "auto",
        ExplainEvidenceModeIrV1::Trace => "trace",
        ExplainEvidenceModeIrV1::Citation => "citation",
        ExplainEvidenceModeIrV1::Counterexample => "counterexample",
    }
}

fn explain_truth_label_v1(expectation: ExplainTruthExpectationIrV1) -> &'static str {
    match expectation {
        ExplainTruthExpectationIrV1::Inspect => "inspect",
        ExplainTruthExpectationIrV1::Confirm => "confirm",
        ExplainTruthExpectationIrV1::Refute => "refute",
    }
}

fn flow_window_label_v1(window: FindFlowWindowIrV1) -> &'static str {
    match window {
        FindFlowWindowIrV1::Node => "node",
        FindFlowWindowIrV1::Block => "block",
        FindFlowWindowIrV1::Function => "function",
    }
}

fn slice_direction_label_v1(direction: FindSliceDirectionIrV1) -> &'static str {
    match direction {
        FindSliceDirectionIrV1::Forward => "forward",
        FindSliceDirectionIrV1::Backward => "backward",
        FindSliceDirectionIrV1::Bidirectional => "bidirectional",
    }
}

fn summary_facet_label_v1(facet: &FindSummaryFacetIrV1) -> &'static str {
    match facet {
        FindSummaryFacetIrV1::Parameters => "parameters",
        FindSummaryFacetIrV1::Returns => "returns",
        FindSummaryFacetIrV1::Reads => "reads",
        FindSummaryFacetIrV1::Writes => "writes",
        FindSummaryFacetIrV1::Effects => "effects",
    }
}
