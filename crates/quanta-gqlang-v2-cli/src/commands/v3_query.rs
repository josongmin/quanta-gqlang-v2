//! Open-analysis CLI query front door.

use quanta_gqlang_v2_frontend_contract::{
    analyze_source_document_v2, AnalysisStatusV2, FrontendTransactionErrorV2,
    OpenAnalysisAuthorityV2, SourceIdV2,
};

use crate::v3_plan::{render_analysis_document_v2, render_transaction_error_v2};

/// Query output and its process status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryOutputV2 {
    pub body: String,
    pub exit_code: u8,
}

/// Analyze source under explicit partial authority. The CLI never fabricates
/// host catalogs and therefore reports unresolved authority as `incomplete`.
pub fn run(input: &str) -> Result<QueryOutputV2, String> {
    let authority = OpenAnalysisAuthorityV2::try_standard_v2().map_err(|error| {
        format!(
            "frontend.error.code={}\nfrontend.error.message={}",
            error.code(),
            error.message()
        )
    })?;
    let document = analyze_source_document_v2(SourceIdV2::from_raw(0), input, &authority)
        .map_err(|error| render_transaction_error_v2(&error))?;
    let exit_code = match document.status() {
        AnalysisStatusV2::Complete => 0,
        AnalysisStatusV2::Incomplete | AnalysisStatusV2::Error => 1,
    };
    Ok(QueryOutputV2 {
        body: render_analysis_document_v2(&document),
        exit_code,
    })
}

/// Render the public transaction failure without reaching into owner crates.
#[must_use]
pub fn render_error(error: &FrontendTransactionErrorV2) -> String {
    render_transaction_error_v2(error)
}
