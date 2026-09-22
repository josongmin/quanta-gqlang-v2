//! Negative case A13_semantic_plan_input_v2 (class A: owner symbol through the facade root).
//!
//! `SemanticPlanInputV2` is a planning-internal sealed input (owner type). It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `SemanticPlanInputV2` in the root".
use quanta_gqlang_v2_frontend_contract::SemanticPlanInputV2;

fn main() {}
