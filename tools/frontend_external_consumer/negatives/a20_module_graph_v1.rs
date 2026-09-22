//! Negative case A20_module_graph_v1 (class A: owner symbol through the facade root).
//!
//! `ModuleGraphV1` is a binder/modules owner graph type. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `ModuleGraphV1` in the root".
use quanta_gqlang_v2_frontend_contract::ModuleGraphV1;

fn main() {}
