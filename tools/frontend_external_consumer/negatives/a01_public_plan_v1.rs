//! Negative case A01_public_plan_v1 (class A: owner symbol through the facade root).
//!
//! `PublicPlanV1` is a deleted V1 planning root. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `PublicPlanV1` in the root".
use quanta_gqlang_v2_frontend_contract::PublicPlanV1;

fn main() {}
