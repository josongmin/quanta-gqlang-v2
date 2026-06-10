use quanta_gqlang_v2_ir::explain_ir::{
    ExplainEvidenceModeIrV1, ExplainShapeIrV1, ExplainTruthExpectationIrV1,
};
use quanta_gqlang_v2_ir::find_ir::{FindExecutionPlanIrV1, FindFlowWindowIrV1};
use quanta_gqlang_v2_ir::frontend::{
    explain_find_subject_v1, explain_find_witness_v1, slice_direction_label_v1,
    summary_facet_label_v1, WitnessBoundV1, WitnessDetailV1, WitnessStepV1,
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

impl ExplainViewV1 {
    pub fn render_v1(&self) -> String {
        let mut lines = vec![
            format!("explain.subject={}", self.subject),
            format!("explain.evidence={}", self.evidence),
            format!("explain.truth={}", self.truth),
            format!("explain.degraded_count={}", self.degraded_badges.len()),
        ];

        for (index, badge) in self.degraded_badges.iter().enumerate() {
            lines.push(format!("explain.degraded[{index}].code={}", badge.code));
            lines.push(format!("explain.degraded[{index}].label={}", badge.label));
        }

        if let Some(path_witness) = &self.path_witness {
            lines.push(format!("explain.path_witness.kind={}", path_witness.kind));
            lines.push(format!(
                "explain.path_witness.step_count={}",
                path_witness.steps.len()
            ));
            for (index, step) in path_witness.steps.iter().enumerate() {
                lines.push(format!(
                    "explain.path_witness.step[{index}].label={}",
                    step.label
                ));
                lines.push(format!(
                    "explain.path_witness.step[{index}].detail={}",
                    step.detail
                ));
            }
            if let Some(note) = &path_witness.note {
                lines.push(format!("explain.path_witness.note={note}"));
            }
        }

        if let Some(why_no_result) = &self.why_no_result {
            lines.push(format!("explain.why_no_result={why_no_result}"));
        }

        lines.join("\n")
    }
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
        subject: explain_find_subject_v1(plan),
        evidence: "trace",
        truth: "inspect",
        degraded_badges: degraded_badges_v1(degraded_reasons),
        path_witness: Some(render_path_witness_v1(plan)),
        why_no_result: why_no_result.map(ToOwned::to_owned),
    }
}

/// Adapt the shared, presentation-neutral `WitnessModelV1` (the single 7-variant
/// walk lives in `ir`) into the CLI's own `PathWitnessViewV1`. The CLI is the
/// crate that wants index-suffixed intermediate labels (`via[0]`) and a `note`,
/// so it layers those on here; the neutral model carries the structural facts.
pub fn render_path_witness_v1(plan: &FindExecutionPlanIrV1) -> PathWitnessViewV1 {
    let model = explain_find_witness_v1(plan);
    PathWitnessViewV1 {
        kind: model.kind,
        steps: index_witness_steps_v1(&model.steps),
        note: model.bound.as_ref().map(witness_note_v1),
    }
}

/// Reattach the CLI's per-intermediate index suffix (e.g. `via` -> `via[0]`),
/// keeping the leading `source` and trailing `sink` (and the fixed-label slice /
/// points-to / summary steps) unsuffixed exactly as before.
fn index_witness_steps_v1(steps: &[WitnessStepV1]) -> Vec<PathWitnessStepV1> {
    let mut intermediate_index = 0usize;
    steps
        .iter()
        .map(|step| {
            let detail = witness_detail_v1(&step.detail);
            let label = if matches!(step.label.as_str(), "source" | "sink")
                || is_fixed_label_v1(&step.label)
            {
                step.label.clone()
            } else {
                let label = format!("{}[{intermediate_index}]", step.label);
                intermediate_index += 1;
                label
            };
            PathWitnessStepV1 { label, detail }
        })
        .collect()
}

/// The non-path-sequence step labels carry no index suffix in the CLI surface.
fn is_fixed_label_v1(label: &str) -> bool {
    matches!(
        label,
        "anchor" | "direction" | "frontier" | "reference" | "pointee" | "callable" | "facets"
    )
}

fn witness_detail_v1(detail: &WitnessDetailV1) -> String {
    match detail {
        WitnessDetailV1::Text(text) => text.clone(),
        WitnessDetailV1::Direction(direction) => slice_direction_label_v1(*direction).to_string(),
        WitnessDetailV1::Facets(facets) => facets
            .iter()
            .map(summary_facet_label_v1)
            .collect::<Vec<_>>()
            .join(","),
    }
}

fn witness_note_v1(bound: &WitnessBoundV1) -> String {
    match bound {
        WitnessBoundV1::MaxHops(depth) => format!("bounded by max_hops={depth}"),
        WitnessBoundV1::Window(window) => format!("window={}", flow_window_label_v1(*window)),
        WitnessBoundV1::Scope(scope) => format!("scope={scope}"),
        WitnessBoundV1::ExpansionDepth(depth) => format!("bounded by expansion_depth={depth}"),
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
                Some(DegradedReasonV1::CompositionMaterializationUnavailable) => {
                    "Composition materialization unavailable"
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
