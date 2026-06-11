//! E12A — the starter-scaffold core now lives in the ir catalog plane
//! (`quanta_gqlang_v2_ir::catalog::starter_scaffold`) so every shell consumes one
//! source. This handler is a thin re-export to keep the LSP-side path stable; the
//! unchanged LSP tests prove the move was lossless.

pub use quanta_gqlang_v2_ir::catalog::starter_scaffold::*;
