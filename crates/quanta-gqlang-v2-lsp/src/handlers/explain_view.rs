use quanta_gqlang_v2_ir::explain_ir::ExplainShapeIrV1;
use quanta_gqlang_v2_ir::find_ir::FindExecutionPlanIrV1;
use quanta_gqlang_v2_ir::root_ir::DegradedReasonV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainHintCardV1 {
    pub(crate) title: String,
    pub(crate) badges: Vec<String>,
    pub(crate) witness_preview: Vec<String>,
    pub(crate) why_no_result: Option<String>,
}

pub fn explain_hint_from_shape_v1(
    shape: &ExplainShapeIrV1,
    degraded_reasons: &[String],
    why_no_result: Option<&str>,
) -> ExplainHintCardV1 {
    ExplainHintCardV1 {
        title: format!("EXPLAIN {}", shape.target),
        badges: badges_v1(degraded_reasons),
        witness_preview: vec![
            format!("evidence={:?}", shape.evidence),
            format!("truth={:?}", shape.truth_expectation),
        ],
        why_no_result: why_no_result.map(ToOwned::to_owned),
    }
}

pub fn explain_hint_from_find_v1(
    plan: &FindExecutionPlanIrV1,
    degraded_reasons: &[String],
    why_no_result: Option<&str>,
) -> ExplainHintCardV1 {
    ExplainHintCardV1 {
        title: format!("WITNESS {}", find_title_v1(plan)),
        badges: badges_v1(degraded_reasons),
        witness_preview: witness_preview_v1(plan),
        why_no_result: why_no_result.map(ToOwned::to_owned),
    }
}

fn badges_v1(reasons: &[String]) -> Vec<String> {
    // Map through the shared DegradedReasonV1 authority rather than re-matching the
    // raw code strings: the badge label keys off the enum variant, so a code rename
    // in one place can never silently desync the LSP badge here (the match is
    // exhaustive over the enum).
    reasons
        .iter()
        .map(|reason| match DegradedReasonV1::from_code_v1(reason) {
            Some(DegradedReasonV1::RuntimeHostRoutingUnavailable) => "degraded:routing".to_string(),
            Some(DegradedReasonV1::SemanticConfirmLaneUnavailable) => {
                "degraded:semantic-confirm".to_string()
            }
            Some(DegradedReasonV1::TruthEvidenceMaterializationUnavailable) => {
                "degraded:truth-materialization".to_string()
            }
            Some(DegradedReasonV1::RetrievalResultMaterializationUnavailable) => {
                "degraded:retrieval-materialization".to_string()
            }
            Some(DegradedReasonV1::CatalogAndInteractiveHostTruthUnavailable) => {
                "degraded:catalog-truth".to_string()
            }
            None => format!("degraded:{reason}"),
        })
        .collect()
}

fn find_title_v1(plan: &FindExecutionPlanIrV1) -> String {
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

fn witness_preview_v1(plan: &FindExecutionPlanIrV1) -> Vec<String> {
    match plan {
        FindExecutionPlanIrV1::Reachability(spec) => {
            let mut preview = vec![format!("source={}", spec.source)];
            preview.extend(
                spec.required_waypoints
                    .iter()
                    .map(|via| format!("via={via}")),
            );
            preview.push(format!("sink={}", spec.sink));
            preview
        }
        FindExecutionPlanIrV1::Flow(spec) => {
            let mut preview = vec![format!("source={}", spec.source)];
            preview.extend(spec.stages.iter().map(|stage| format!("through={stage}")));
            preview.push(format!("sink={}", spec.sink));
            preview
        }
        FindExecutionPlanIrV1::Slice(spec) => vec![
            format!("anchor={}", spec.anchor),
            format!("direction={:?}", spec.direction),
            format!("frontier={}", spec.frontier.as_deref().unwrap_or("auto")),
        ],
        FindExecutionPlanIrV1::Taint(spec) => {
            let mut preview = vec![format!("source={}", spec.source)];
            preview.extend(
                spec.blocked_by
                    .iter()
                    .map(|item| format!("sanitizer={item}")),
            );
            preview.push(format!("sink={}", spec.sink));
            preview
        }
        FindExecutionPlanIrV1::Path(spec) => {
            let mut preview = vec![format!("source={}", spec.source)];
            preview.extend(
                spec.required_waypoints
                    .iter()
                    .map(|item| format!("through={item}")),
            );
            preview.push(format!("sink={}", spec.sink));
            preview
        }
        FindExecutionPlanIrV1::PointsTo(spec) => vec![
            format!("reference={}", spec.reference),
            format!("pointee={}", spec.expected_pointee),
        ],
        FindExecutionPlanIrV1::Summary(spec) => vec![
            format!("callable={}", spec.callable),
            format!("facets={}", spec.requested_facets.len()),
        ],
    }
}
