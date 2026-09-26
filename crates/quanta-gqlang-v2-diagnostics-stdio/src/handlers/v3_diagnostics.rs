//! Facade-only diagnostics analysis handler.

use quanta_gqlang_v2_frontend_contract::{
    analyze_source_document_v2, AnalysisStatusV2, FrontendDiagnosticV2, FrontendTransactionErrorV2,
    OpenAnalysisAuthorityV2, SourceIdV2, ANALYSIS_DIAGNOSTIC_LIMIT_CODE,
};
use serde::Serialize;

use crate::server::{ProtocolErrorV2, WireDiagnosticV2};

/// Bounded analysis payload returned to the transport.
#[derive(Debug, Serialize)]
pub struct AnalysisPayloadV2 {
    pub analysis_status: &'static str,
    pub diagnostics: Vec<WireDiagnosticV2>,
}

/// Analyze one source under the server's explicit open authority.
pub fn handle_v3_diagnostics_v2(
    source_id: u32,
    source: &str,
    authority: &OpenAnalysisAuthorityV2,
) -> Result<AnalysisPayloadV2, ProtocolErrorV2> {
    match analyze_source_document_v2(SourceIdV2::from_raw(source_id), source, authority) {
        Ok(document) => {
            let diagnostics = document.diagnostics();
            Ok(AnalysisPayloadV2 {
                analysis_status: document.status().keyword(),
                diagnostics: diagnostics.iter().map(WireDiagnosticV2::from).collect(),
            })
        }
        Err(FrontendTransactionErrorV2::Document(failure)) => {
            if let [diagnostic] = failure.diagnostics() {
                if diagnostic.code() == ANALYSIS_DIAGNOSTIC_LIMIT_CODE {
                    return Err(ProtocolErrorV2::new(
                        "diagnostic_limit_exceeded",
                        diagnostic.message(),
                    ));
                }
            }
            Ok(AnalysisPayloadV2 {
                analysis_status: AnalysisStatusV2::Error.keyword(),
                diagnostics: failure
                    .diagnostics()
                    .iter()
                    .map(WireDiagnosticV2::from)
                    .collect(),
            })
        }
        Err(FrontendTransactionErrorV2::Authority(error)) => Err(ProtocolErrorV2::new(
            "authority_error",
            format!("{}: {}", error.code(), error.message()),
        )),
    }
}

impl From<&FrontendDiagnosticV2> for WireDiagnosticV2 {
    fn from(diagnostic: &FrontendDiagnosticV2) -> Self {
        let primary = diagnostic.primary();
        Self {
            stage: diagnostic.stage().keyword(),
            code: diagnostic.code(),
            severity: diagnostic.severity().keyword(),
            message: diagnostic.message().to_owned(),
            source_id: primary.source_id.raw(),
            span_start: primary.span.start,
            span_end: primary.span.end,
        }
    }
}
