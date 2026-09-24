//! Deterministic rendering of the public V2 analysis DTOs.

use quanta_gqlang_v2_frontend_contract::{
    AnalysisDocumentV2, AnalysisItemV2, FrontendDiagnosticV2, FrontendFailureV2,
    FrontendTransactionErrorV2, PlannedItemV2, SemanticPlanV2,
};

use crate::line_value::encode_line_value_v1;

/// Render one open-analysis document without re-deriving frontend semantics.
#[must_use]
pub fn render_analysis_document_v2(document: &AnalysisDocumentV2) -> String {
    let diagnostics = document.diagnostics();
    let mut lines = vec![
        format!("analysis.status={}", document.status().keyword()),
        format!("analysis.source_id={}", document.source_id().raw()),
        format!("analysis.item_count={}", document.items().len()),
        format!("analysis.unresolved_count={}", document.unresolved().len()),
        format!("analysis.diagnostic_count={}", diagnostics.len()),
    ];
    for (index, item) in document.items().iter().enumerate() {
        lines.extend(render_item_v2(index, item));
    }
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        lines.extend(render_diagnostic_v2("diagnostic", index, diagnostic));
    }
    lines.join("\n")
}

fn render_item_v2(index: usize, item: &AnalysisItemV2) -> Vec<String> {
    match item {
        AnalysisItemV2::Complete(planned) => {
            let mut lines = vec![
                format!("item[{index}].status=complete"),
                format!("item[{index}].kind={}", planned_kind_v2(planned)),
            ];
            if let Some(plan) = planned.plan() {
                lines.extend(render_plan_v2(index, plan));
            }
            lines
        }
        AnalysisItemV2::NotApplicable(reason) => vec![
            format!("item[{index}].status=not_applicable"),
            format!("item[{index}].no_plan_reason={}", reason.code()),
        ],
        AnalysisItemV2::BlockedByDiagnostics(diagnostics) => vec![
            format!("item[{index}].status=blocked"),
            format!("item[{index}].diagnostic_count={}", diagnostics.len()),
        ],
    }
}

fn planned_kind_v2(item: &PlannedItemV2) -> &'static str {
    match item {
        PlannedItemV2::Relation { .. } => "relation",
        PlannedItemV2::Graph { .. } => "graph",
        PlannedItemV2::Check { .. } => "check",
        PlannedItemV2::Presentation { .. } => "presentation",
        PlannedItemV2::Composition { .. } => "composition",
        PlannedItemV2::MetaRequireCertified { .. } => "meta_require_certified",
        PlannedItemV2::RuleMember(_) => "rule_member",
    }
}

fn render_plan_v2(index: usize, plan: &SemanticPlanV2) -> Vec<String> {
    let capabilities = plan
        .capabilities()
        .iter()
        .map(|capability| capability.keyword())
        .collect::<Vec<_>>()
        .join(",");
    let mut lines = vec![
        format!(
            "item[{index}].plan.authority_lane={}",
            plan.authority_lane().keyword()
        ),
        format!(
            "item[{index}].plan.determinism={}",
            plan.evaluation_determinism().keyword()
        ),
        format!("item[{index}].plan.evidence={}", plan.evidence().keyword()),
        format!("item[{index}].plan.effect={}", plan.effect().keyword()),
        format!(
            "item[{index}].plan.result_kind={}",
            plan.result().result_kind().keyword()
        ),
        format!("item[{index}].plan.capabilities={capabilities}"),
    ];
    for (name, value) in plan.semantic_cost().fields() {
        lines.push(format!("item[{index}].cost.{name}={value}"));
    }
    lines
}

/// Render one public transaction error.
#[must_use]
pub fn render_transaction_error_v2(error: &FrontendTransactionErrorV2) -> String {
    match error {
        FrontendTransactionErrorV2::Authority(error) => [
            "analysis.status=error".to_owned(),
            "frontend.error.kind=authority".to_owned(),
            format!("frontend.error.code={}", error.code()),
            format!(
                "frontend.error.message={}",
                encode_line_value_v1(error.message())
            ),
        ]
        .join("\n"),
        FrontendTransactionErrorV2::Document(failure) => render_failure_v2(failure),
    }
}

/// Render one ordered source failure.
#[must_use]
pub fn render_failure_v2(failure: &FrontendFailureV2) -> String {
    let mut lines = vec![
        "analysis.status=error".to_owned(),
        format!("analysis.diagnostic_count={}", failure.diagnostics().len()),
    ];
    for (index, diagnostic) in failure.diagnostics().iter().enumerate() {
        lines.extend(render_diagnostic_v2("diagnostic", index, diagnostic));
    }
    lines.join("\n")
}

fn render_diagnostic_v2(
    prefix: &str,
    index: usize,
    diagnostic: &FrontendDiagnosticV2,
) -> Vec<String> {
    let location = diagnostic.primary();
    vec![
        format!("{prefix}[{index}].stage={}", diagnostic.stage().keyword()),
        format!("{prefix}[{index}].code={}", diagnostic.code()),
        format!(
            "{prefix}[{index}].severity={}",
            diagnostic.severity().keyword()
        ),
        format!("{prefix}[{index}].source_id={}", location.source_id.raw()),
        format!(
            "{prefix}[{index}].span={}..{}",
            location.span.start, location.span.end
        ),
        format!(
            "{prefix}[{index}].message={}",
            encode_line_value_v1(diagnostic.message())
        ),
    ]
}
