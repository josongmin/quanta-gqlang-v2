//! Negative case A10_compile_program_structural (class A: owner symbol through the facade root).
//!
//! `compile_program_structural` is a deleted structural-authority entry point. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `compile_program_structural` in the root".
use quanta_gqlang_v2_frontend_contract::compile_program_structural;

fn main() {}
