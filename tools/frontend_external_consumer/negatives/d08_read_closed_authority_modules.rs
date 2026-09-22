//! Negative case D08_read_closed_authority_modules (class D: privacy probe on a curated root).
//!
//! `field `modules` of struct `ClosedFrontendAuthorityV2` is private` — the curated
//! root is nameable, but its fields stay sealed, so a sibling repository cannot
//! read, write, or fabricate one from unverified parts.
//!
//! Expected: E0616, "field `modules` of struct `ClosedFrontendAuthorityV2` is private".
use quanta_gqlang_v2_frontend_contract::ClosedFrontendAuthorityV2;

fn probe(authority: &ClosedFrontendAuthorityV2) -> usize {
    std::mem::size_of_val(&authority.modules)
}

fn main() {
    let _ = probe as fn(&ClosedFrontendAuthorityV2) -> usize;
}
