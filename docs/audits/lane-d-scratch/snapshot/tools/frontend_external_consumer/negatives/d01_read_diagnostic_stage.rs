//! Negative case D01_read_diagnostic_stage (class D: privacy probe on a curated root).
//!
//! `field `stage` of struct `FrontendDiagnosticV2` is private` — the curated
//! root is nameable, but its fields stay sealed, so a sibling repository cannot
//! read, write, or fabricate one from unverified parts.
//!
//! Expected: E0616, "field `stage` of struct `FrontendDiagnosticV2` is private".
use quanta_gqlang_v2_frontend_contract::{FrontendDiagnosticV2, FrontendStageV2};

fn probe(diagnostic: &FrontendDiagnosticV2) -> FrontendStageV2 {
    diagnostic.stage
}

fn main() {
    let _ = probe as fn(&FrontendDiagnosticV2) -> FrontendStageV2;
}
