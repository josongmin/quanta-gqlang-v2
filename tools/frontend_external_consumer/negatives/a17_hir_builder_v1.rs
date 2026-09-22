//! Negative case A17_hir_builder_v1 (class A: owner symbol through the facade root).
//!
//! `HirBuilderV1` is a deleted raw-HIR builder. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `HirBuilderV1` in the root".
use quanta_gqlang_v2_frontend_contract::HirBuilderV1;

fn main() {}
