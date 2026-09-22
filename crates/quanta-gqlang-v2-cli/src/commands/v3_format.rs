//! Canonical source formatter through the public frontend contract.

use quanta_gqlang_v2_frontend_contract::{
    format_source_document_v2, FrontendBudgetV2, FrontendFailureV2, SourceIdV2,
};

use crate::v3_plan::render_failure_v2;

/// Format one source document under the canonical bounded profile.
pub fn run(input: &str) -> Result<String, String> {
    let budget = FrontendBudgetV2::standard_v2().map_err(|error| {
        format!(
            "frontend.error.code={}\nfrontend.error.message={}",
            error.code(),
            error.message()
        )
    })?;
    format_source_document_v2(SourceIdV2::from_raw(0), input, &budget)
        .map_err(|failure| render_failure_v2(&failure))
}

#[must_use]
pub fn render_error(error: &FrontendFailureV2) -> String {
    render_failure_v2(error)
}
