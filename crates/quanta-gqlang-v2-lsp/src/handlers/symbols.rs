use crate::handlers::read_only_analysis::analyze_query_v1;
pub use crate::handlers::read_only_analysis::SymbolItemV1;

pub fn handle_symbols(query: &str) -> Result<Vec<SymbolItemV1>, String> {
    analyze_query_v1(query)
        .map(|summary| summary.symbols)
        .map_err(|err| err.render_v1())
}
