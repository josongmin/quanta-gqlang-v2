//! Negative case A03_produce_plan (class A: owner symbol through the facade root).
//!
//! `produce_plan` is a deleted V1 planning entry point. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `produce_plan` in the root".
use quanta_gqlang_v2_frontend_contract::produce_plan;

fn main() {}
