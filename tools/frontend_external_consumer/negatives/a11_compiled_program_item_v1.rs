//! Negative case A11_compiled_program_item_v1 (class A: owner symbol through the facade root).
//!
//! `compiled_program_item_v1` is a deleted V1 program-item accessor. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `compiled_program_item_v1` in the root".
use quanta_gqlang_v2_frontend_contract::compiled_program_item_v1;

fn main() {}
