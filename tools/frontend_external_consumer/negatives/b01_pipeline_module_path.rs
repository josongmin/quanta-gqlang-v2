//! Negative case B01_pipeline_module_path (class B: module-path leak).
//!
//! The facade is a flat curated root: it must not expose a `pipeline` module.
//!
//! Expected: E0432, "no `pipeline` in the root".
use quanta_gqlang_v2_frontend_contract::pipeline::compile_source_document_v2;

fn main() {}
