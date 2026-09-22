//! Negative case A07_frontend_context_v1 (class A: owner symbol through the facade root).
//!
//! `FrontendContextV1` is a deleted V1 pipeline context. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `FrontendContextV1` in the root".
use quanta_gqlang_v2_frontend_contract::FrontendContextV1;

fn main() {}
