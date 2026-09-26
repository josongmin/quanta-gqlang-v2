//! Negative case D07_read_unresolved_code (class D: privacy probe on a curated root).
//!
//! `field `code` of struct `UnresolvedAnalysisFactV2` is private` — the curated
//! root is nameable, but its fields stay sealed, so a sibling repository cannot
//! read, write, or fabricate one from unverified parts.
//!
//! Expected: E0616, "field `code` of struct `UnresolvedAnalysisFactV2` is private".
use quanta_gqlang_v2_frontend_contract::UnresolvedAnalysisFactV2;

fn probe(fact: &UnresolvedAnalysisFactV2) -> &'static str {
    fact.code
}

fn main() {
    let _ = probe as fn(&UnresolvedAnalysisFactV2) -> &'static str;
}
