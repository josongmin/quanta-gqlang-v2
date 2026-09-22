//! Negative case B03_diagnostic_module_path (class B: module-path leak).
//!
//! Pipeline's private `diagnostic` module must not leak through the facade.
//!
//! Expected: E0432, "no `diagnostic` in the root".
use quanta_gqlang_v2_frontend_contract::diagnostic::DocumentAssemblyCodeV2;

fn main() {}
