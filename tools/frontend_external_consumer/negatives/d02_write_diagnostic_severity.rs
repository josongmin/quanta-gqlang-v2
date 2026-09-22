//! Negative case D02_write_diagnostic_severity (class D: privacy probe on a curated root).
//!
//! `field `severity` of struct `FrontendDiagnosticV2` is private` — the curated
//! root is nameable, but its fields stay sealed, so a sibling repository cannot
//! read, write, or fabricate one from unverified parts.
//!
//! Expected: E0616, "field `severity` of struct `FrontendDiagnosticV2` is private".
use quanta_gqlang_v2_frontend_contract::{FrontendDiagnosticV2, FrontendSeverityV2};

fn probe(diagnostic: &mut FrontendDiagnosticV2) {
    diagnostic.severity = FrontendSeverityV2::Note;
}

fn main() {
    let _ = probe as fn(&mut FrontendDiagnosticV2);
}
