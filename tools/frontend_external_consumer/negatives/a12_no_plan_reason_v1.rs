//! Negative case A12_no_plan_reason_v1 (class A: owner symbol through the facade root).
//!
//! `NoPlanReasonV1` is a deleted V1 non-plan reason. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `NoPlanReasonV1` in the root".
use quanta_gqlang_v2_frontend_contract::NoPlanReasonV1;

fn main() {}
