//! Negative case B04_pipeline_module_only (class B: module-path leak).
//!
//! The bare module path is a leak even without naming an item inside it.
//!
//! Expected: E0432, "no `pipeline` in the root".
use quanta_gqlang_v2_frontend_contract::pipeline;

fn main() {}
