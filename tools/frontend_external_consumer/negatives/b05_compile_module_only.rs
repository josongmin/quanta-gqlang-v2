//! Negative case B05_compile_module_only (class B: module-path leak).
//!
//! The bare module path is a leak even without naming an item inside it.
//!
//! Expected: E0432, "no `compile` in the root".
use quanta_gqlang_v2_frontend_contract::compile;

fn main() {}
