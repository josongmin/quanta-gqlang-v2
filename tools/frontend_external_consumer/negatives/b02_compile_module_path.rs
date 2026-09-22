//! Negative case B02_compile_module_path (class B: module-path leak).
//!
//! Pipeline's private `compile` module must not leak through the facade.
//!
//! Expected: E0432, "no `compile` in the root".
use quanta_gqlang_v2_frontend_contract::compile::compile_source_document_v2;

fn main() {}
