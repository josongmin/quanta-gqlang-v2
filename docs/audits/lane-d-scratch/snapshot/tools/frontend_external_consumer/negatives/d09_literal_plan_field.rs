//! Negative case D09_literal_plan_field (class D: privacy probe on a curated root).
//!
//! `field `authority_lane` of struct `SemanticPlanV2` is private` — the curated
//! root is nameable, but its fields stay sealed, so a sibling repository cannot
//! read, write, or fabricate one from unverified parts.
//!
//! Expected: E0451, "field `authority_lane` of struct `SemanticPlanV2` is private".
use quanta_gqlang_v2_frontend_contract::SemanticPlanV2;

fn probe() -> SemanticPlanV2 {
    SemanticPlanV2 {
        authority_lane: panic!("unreachable: the field is private"),
    }
}

fn main() {
    let _ = probe as fn() -> SemanticPlanV2;
}
