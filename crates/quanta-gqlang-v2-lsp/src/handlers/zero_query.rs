//! E12A — the zero-query response surface now lives in the ir catalog plane
//! (`quanta_gqlang_v2_ir::catalog::zero_query_surface`). This handler is a thin
//! re-export so the LSP-side `crate::handlers::zero_query::*` path stays stable;
//! the unchanged LSP zero-query tests prove the move was lossless.

pub use quanta_gqlang_v2_ir::catalog::zero_query_surface::*;
