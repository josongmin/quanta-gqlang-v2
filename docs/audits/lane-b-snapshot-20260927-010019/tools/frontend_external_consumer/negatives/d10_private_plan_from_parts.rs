//! Negative case D10_private_plan_from_parts (class D: private constructor).
//!
//! `SemanticPlanV2::from_parts` is deliberately private: the only way to obtain
//! a plan is to run a stage that derives one. The curated root is nameable; its
//! constructor is not. rustc suggests the call in a struct-literal diagnostic —
//! the suggestion does not make it reachable.
//!
//! Expected: E0624, "associated function `from_parts` is private".
use quanta_gqlang_v2_frontend_contract::SemanticPlanV2;

fn main() {
    let _ = SemanticPlanV2::from_parts;
}
