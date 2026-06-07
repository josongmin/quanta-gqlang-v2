use crate::handlers::hover::analyze_query_v1;
pub use crate::handlers::hover::DiagnosticItemV1;

pub fn handle_diagnostics(query: &str) -> Result<Vec<DiagnosticItemV1>, String> {
    match analyze_query_v1(query) {
        Ok(_) => Ok(Vec::new()),
        Err(err) => Ok(vec![err.diagnostic_item_v1()]),
    }
}
