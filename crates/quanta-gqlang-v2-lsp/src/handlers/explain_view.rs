use quanta_gqlang_v2_ir::explain_ir::ExplainShapeIrV1;
use quanta_gqlang_v2_ir::find_ir::FindExecutionPlanIrV1;
use quanta_gqlang_v2_ir::frontend::{
    analyze_document_v1, parse_document_v1, SemaDiagnosticPageV1, SyntaxDiagnosticPageV1,
};
use quanta_gqlang_v2_ir::lower_document_to_ir_v1;
use quanta_gqlang_v2_ir::root_ir::DegradedReasonV1;
use quanta_gqlang_v2_ir::root_ir::PublicIrRootV1;

const NO_RESULT_REASON_V1: &str = "read-only-shell-has-no-execution-backend";

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

pub fn handle_explain_v1(query: &str) -> Result<ExplainHintCardV1, String> {
    let doc = parse_document_v1(query).map_err(render_parse_error_v1)?;
    let typed = analyze_document_v1(&doc).map_err(render_sema_error_v1)?;
    let ir_roots = lower_document_to_ir_v1(&typed).map_err(|err| err.message)?;
    let degraded_reasons = ir_roots
        .iter()
        .filter_map(|root| {
            DegradedReasonV1::for_ir_root_v1(root).map(|reason| reason.code_v1().to_string())
        })
        .collect::<Vec<_>>();

    let Some(first_root) = ir_roots.first() else {
        return Err("explain front door requires a non-empty document".to_string());
    };

    match first_root {
        PublicIrRootV1::Explain(spec) => Ok(explain_hint_from_shape_v1(
            &spec.shape().map_err(|err| err.message)?,
            &degraded_reasons,
            Some(NO_RESULT_REASON_V1),
        )),
        PublicIrRootV1::Find(plan) => Ok(explain_hint_from_find_v1(
            &plan.execution_plan().map_err(|err| err.message)?,
            &degraded_reasons,
            Some(NO_RESULT_REASON_V1),
        )),
        _ => Err("explain front door only supports EXPLAIN or FIND roots".to_string()),
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

fn render_parse_error_v1(page: SyntaxDiagnosticPageV1) -> String {
    page.diagnostics
        .into_iter()
        .map(|diag| format!("{}: {}", diag.code, diag.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn render_sema_error_v1(page: SemaDiagnosticPageV1) -> String {
    page.diagnostics.join("; ")
}
