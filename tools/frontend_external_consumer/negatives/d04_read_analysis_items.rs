//! Negative case D04_read_analysis_items (class D: privacy probe on a curated root).
//!
//! `field `items` of struct `AnalysisDocumentV2` is private` — the curated
//! root is nameable, but its fields stay sealed, so a sibling repository cannot
//! read, write, or fabricate one from unverified parts.
//!
//! Expected: E0616, "field `items` of struct `AnalysisDocumentV2` is private".
use quanta_gqlang_v2_frontend_contract::AnalysisDocumentV2;

fn probe(document: &AnalysisDocumentV2) -> usize {
    document.items.len()
}

fn main() {
    let _ = probe as fn(&AnalysisDocumentV2) -> usize;
}
