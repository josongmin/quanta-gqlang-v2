//! Negative case A05_compiled_item_v1 (class A: owner symbol through the facade root).
//!
//! `CompiledItemV1` is a deleted V1 compiled item. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `CompiledItemV1` in the root".
use quanta_gqlang_v2_frontend_contract::CompiledItemV1;

fn main() {}
