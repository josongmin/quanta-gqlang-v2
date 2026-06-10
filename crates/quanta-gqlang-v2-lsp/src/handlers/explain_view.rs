use quanta_gqlang_v2_ir::explain_ir::ExplainShapeIrV1;
use quanta_gqlang_v2_ir::find_ir::FindExecutionPlanIrV1;
use quanta_gqlang_v2_ir::frontend::{
    analyze_document_v1, explain_find_subject_v1, explain_find_witness_v1, parse_document_v1,
    SemaDiagnosticPageV1, SyntaxDiagnosticPageV1, WitnessDetailV1, NO_RESULT_REASON_V1,
};
use quanta_gqlang_v2_ir::lower_document_to_ir_v1;
use quanta_gqlang_v2_ir::root_ir::DegradedReasonV1;
use quanta_gqlang_v2_ir::root_ir::PublicIrRootV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExplainHintCardV1 {
    pub(crate) title: String,
    pub(crate) badges: Vec<String>,
    pub(crate) witness_preview: Vec<String>,
    pub(crate) why_no_result: Option<String>,
}

pub fn explain_hint_from_shape_v1(
    shape: &ExplainShapeIrV1,
    degraded_reasons: &[DegradedReasonV1],
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
    degraded_reasons: &[DegradedReasonV1],
    why_no_result: Option<&str>,
) -> ExplainHintCardV1 {
    ExplainHintCardV1 {
        title: format!("WITNESS {}", explain_find_subject_v1(plan)),
        badges: badges_v1(degraded_reasons),
        witness_preview: witness_preview_v1(plan),
        why_no_result: why_no_result.map(ToOwned::to_owned),
    }
}

pub fn handle_explain_v1(query: &str) -> Result<ExplainHintCardV1, String> {
    let doc = parse_document_v1(query).map_err(render_parse_error_v1)?;
    let typed = analyze_document_v1(&doc).map_err(render_sema_error_v1)?;
    let ir_roots = lower_document_to_ir_v1(&typed).map_err(|err| err.message)?;

    let Some(first_root) = ir_roots.first() else {
        return Err("explain front door requires a non-empty document".to_string());
    };
    // First-statement contract: only the FIRST root's degraded reason is rendered
    // (the explained subject is the first root), never a cross-root aggregate.
    // Typed end-to-end — no code-string round-trip.
    let degraded_reasons: Vec<DegradedReasonV1> = DegradedReasonV1::for_ir_root_v1(first_root)
        .into_iter()
        .collect();

    match first_root {
        PublicIrRootV1::Explain(spec) => Ok(explain_hint_from_shape_v1(
            spec.shape(),
            &degraded_reasons,
            Some(NO_RESULT_REASON_V1),
        )),
        PublicIrRootV1::Find(plan) => Ok(explain_hint_from_find_v1(
            &plan.execution_plan().map_err(|err| err.message)?,
            &degraded_reasons,
            Some(NO_RESULT_REASON_V1),
        )),
        // Closed front door: every unsupported root is listed explicitly so a
        // future IR root variant is a compile error here, not a silent reject.
        PublicIrRootV1::Select(_)
        | PublicIrRootV1::Check(_)
        | PublicIrRootV1::Retrieval(_)
        | PublicIrRootV1::Interactive(_)
        | PublicIrRootV1::Declaration(_)
        | PublicIrRootV1::Meta(_)
        | PublicIrRootV1::Composition(_) => {
            Err("explain front door only supports EXPLAIN or FIND roots".to_string())
        }
    }
}

fn badges_v1(reasons: &[DegradedReasonV1]) -> Vec<String> {
    // Typed end-to-end: an exhaustive match over the closed DegradedReasonV1
    // enum, with no `from_code` round-trip and no unreachable `None` fallback
    // arm. A new variant is a compile error here, never a silent `degraded:<raw>`.
    reasons
        .iter()
        .map(|reason| match reason {
            DegradedReasonV1::RuntimeHostRoutingUnavailable => "degraded:routing".to_string(),
            DegradedReasonV1::SemanticConfirmLaneUnavailable => {
                "degraded:semantic-confirm".to_string()
            }
            DegradedReasonV1::TruthEvidenceMaterializationUnavailable => {
                "degraded:truth-materialization".to_string()
            }
            DegradedReasonV1::RetrievalResultMaterializationUnavailable => {
                "degraded:retrieval-materialization".to_string()
            }
            DegradedReasonV1::CatalogAndInteractiveHostTruthUnavailable => {
                "degraded:catalog-truth".to_string()
            }
            DegradedReasonV1::CompositionMaterializationUnavailable => {
                "degraded:composition-materialization".to_string()
            }
        })
        .collect()
}

/// Adapt the shared, presentation-neutral `WitnessModelV1` (the single 7-variant
/// walk lives in `ir`) into the LSP's own `label=detail` preview lines. The LSP
/// renders the slice direction with its enum's debug casing and the summary
/// facets as a count — both intentionally diverging from the CLI surface — so it
/// formats those from the neutral structural detail here rather than re-walking
/// the plan. (The neutral step labels are already the bare stems the LSP wants,
/// without the CLI's `[index]` suffix.)
fn witness_preview_v1(plan: &FindExecutionPlanIrV1) -> Vec<String> {
    explain_find_witness_v1(plan)
        .steps
        .into_iter()
        .map(|step| match step.detail {
            WitnessDetailV1::Text(text) => format!("{}={text}", step.label),
            WitnessDetailV1::Direction(direction) => format!("{}={direction:?}", step.label),
            WitnessDetailV1::Facets(facets) => format!("{}={}", step.label, facets.len()),
        })
        .collect()
}

fn render_parse_error_v1(page: SyntaxDiagnosticPageV1) -> String {
    page.diagnostics
        .into_iter()
        .map(|diag| format!("{}: {}", diag.code, diag.message))
        .collect::<Vec<_>>()
        .join("; ")
}

fn render_sema_error_v1(page: SemaDiagnosticPageV1) -> String {
    page.to_string()
}
