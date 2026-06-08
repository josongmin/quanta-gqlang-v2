use crate::handlers::read_only_analysis::analyze_statements_v1;
pub use crate::handlers::read_only_analysis::DiagnosticItemV1;

pub fn handle_diagnostics(query: &str) -> Result<Vec<DiagnosticItemV1>, String> {
    // Report a diagnostic for every malformed statement in the document, not just
    // the first, mirroring the parser's per-statement error recovery.
    Ok(analyze_statements_v1(query)
        .into_iter()
        .filter_map(|result| result.err().map(|err| err.diagnostic_item_v1()))
        .collect())
}
