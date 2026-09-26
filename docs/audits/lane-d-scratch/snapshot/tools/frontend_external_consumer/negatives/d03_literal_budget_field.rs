//! Negative case D03_literal_budget_field (class D: privacy probe on a curated root).
//!
//! `field `max_source_bytes` of struct `FrontendBudgetV2` is private` — the curated
//! root is nameable, but its fields stay sealed, so a sibling repository cannot
//! read, write, or fabricate one from unverified parts.
//!
//! Expected: E0451, "field `max_source_bytes` of struct `FrontendBudgetV2` is private".
use quanta_gqlang_v2_frontend_contract::FrontendBudgetV2;

fn probe() -> FrontendBudgetV2 {
    FrontendBudgetV2 {
        max_source_bytes: std::num::NonZeroU32::MIN,
    }
}

fn main() {
    let _ = probe as fn() -> FrontendBudgetV2;
}
