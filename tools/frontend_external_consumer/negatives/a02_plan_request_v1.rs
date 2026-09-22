//! Negative case A02_plan_request_v1 (class A: owner symbol through the facade root).
//!
//! `PlanRequestV1` is a deleted V1 planning request DTO. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `PlanRequestV1` in the root".
use quanta_gqlang_v2_frontend_contract::PlanRequestV1;

fn main() {}
