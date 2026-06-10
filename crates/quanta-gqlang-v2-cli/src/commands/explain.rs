use super::query;
use crate::explain_view::{self, ExplainViewV1};
use quanta_gqlang_v2_ir::root_ir::{DegradedReasonV1, PublicIrRootV1};

pub fn run(input: &str) -> Result<ExplainViewV1, String> {
    let doc = query::parse_document_v1(input).map_err(query::render_syntax_page_v1)?;
    let typed = query::analyze_document_v1(&doc).map_err(query::render_sema_page_v1)?;
    let ir_roots = query::lower_document_to_ir_v1(&typed).map_err(query::render_ir_error_v1)?;
    let summary = query::build_query_summary_v1(&doc, &typed, &ir_roots);
    // First-statement contract: explain renders the FIRST root's view with ONLY
    // that root's degraded reason. Aggregating every root's degradation onto the
    // first subject would attribute a later root's lane gap to the explained
    // subject (the cross-root misattribution this fixes). Typed end-to-end — no
    // code-string round-trip.
    let degraded_reasons: Vec<DegradedReasonV1> = summary
        .roots
        .first()
        .and_then(|root| root.degraded_reason)
        .into_iter()
        .collect();

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
