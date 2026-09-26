//! Negative case D05_read_plan_authority_lane (class D: privacy probe on a curated root).
//!
//! `field `authority_lane` of struct `SemanticPlanV2` is private` — the curated
//! root is nameable, but its fields stay sealed, so a sibling repository cannot
//! read, write, or fabricate one from unverified parts.
//!
//! Expected: E0616, "field `authority_lane` of struct `SemanticPlanV2` is private".
use quanta_gqlang_v2_frontend_contract::SemanticPlanV2;

fn probe(plan: &SemanticPlanV2) -> usize {
    std::mem::size_of_val(&plan.authority_lane)
}

fn main() {
    let _ = probe as fn(&SemanticPlanV2) -> usize;
}
