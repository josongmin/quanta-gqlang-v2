//! Negative case D06_read_checked_span_location (class D: privacy probe on a curated root).
//!
//! `field `location` of struct `CheckedLocatedSpanV2` is private` — the curated
//! root is nameable, but its fields stay sealed, so a sibling repository cannot
//! read, write, or fabricate one from unverified parts.
//!
//! Expected: E0616, "field `location` of struct `CheckedLocatedSpanV2` is private".
use quanta_gqlang_v2_frontend_contract::{CheckedLocatedSpanV2, LocatedSpanV2};

fn probe(span: &CheckedLocatedSpanV2) -> LocatedSpanV2 {
    span.location
}

fn main() {
    let _ = probe as fn(&CheckedLocatedSpanV2) -> LocatedSpanV2;
}
