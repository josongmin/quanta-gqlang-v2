use super::query;
use crate::explain_view::{self, ExplainViewV1};
use quanta_gqlang_v2_ir::root_ir::PublicIrRootV1;

pub fn run(input: &str) -> Result<ExplainViewV1, String> {
    let doc = query::parse_document_v1(input).map_err(query::render_syntax_page_v1)?;
    let typed = query::analyze_document_v1(&doc).map_err(query::render_sema_page_v1)?;
    let ir_roots = query::lower_document_to_ir_v1(&typed).map_err(query::render_ir_error_v1)?;
    let summary = query::build_query_summary_v1(&doc, &typed, &ir_roots);
    let degraded_reasons = summary
        .roots
        .iter()
        .filter_map(|root| {
            root.degraded_reason
                .map(|reason| reason.code_v1().to_string())
        })
        .collect::<Vec<_>>();

    let Some(first_root) = ir_roots.first() else {
        return Err("explain front door requires a non-empty document".to_string());
    };

    match first_root {
        PublicIrRootV1::Explain(spec) => {
            let shape = spec.shape().map_err(|err| err.message)?;
            Ok(explain_view::build_explain_view_v1(
                &shape,
                &degraded_reasons,
                Some(crate::NO_RESULT_REASON_V1),
            ))
        }
        PublicIrRootV1::Find(plan) => {
            let plan = plan.execution_plan().map_err(|err| err.message)?;
            Ok(explain_view::build_find_explain_view_v1(
                &plan,
                &degraded_reasons,
                Some(crate::NO_RESULT_REASON_V1),
            ))
        }
        _ => Err("explain front door only supports EXPLAIN or FIND roots".to_string()),
    }
}
