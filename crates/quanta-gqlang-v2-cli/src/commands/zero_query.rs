//! E12A — the CLI `zero-query` verb. Renders the read-only starter scaffold from
//! the shared ir catalog plane (`build_zero_query_response_v1`), the exact same
//! builder the LSP zero-query method consumes — so the CLI and the editor surface
//! can never drift in what they suggest.

use quanta_gqlang_v2_ir::catalog::zero_query_surface::build_zero_query_response_v1;

/// Run the nullary zero-query verb and render its deterministic text report.
pub fn run() -> Result<String, String> {
    build_zero_query_response_v1().map(|response| response.render_v1())
}
