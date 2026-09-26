//! Negative case D12_read_budget_max_source_bytes (class D: privacy probe).
//!
//! `field `max_source_bytes` of struct `FrontendBudgetV2` is private` — the
//! budget root is readable only through its accessors; the sealed field itself
//! stays closed to a sibling repository.
//!
//! Expected: E0616, "field `max_source_bytes` of struct `FrontendBudgetV2` is private".
use quanta_gqlang_v2_frontend_contract::FrontendBudgetV2;

fn probe(budget: &FrontendBudgetV2) -> u32 {
    budget.max_source_bytes.get()
}

fn main() {
    let _ = probe as fn(&FrontendBudgetV2) -> u32;
}
