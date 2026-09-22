//! Negative case A06_compiled_program_v1 (class A: owner symbol through the facade root).
//!
//! `CompiledProgramV1` is a deleted V1 compiled program. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `CompiledProgramV1` in the root".
use quanta_gqlang_v2_frontend_contract::CompiledProgramV1;

fn main() {}
