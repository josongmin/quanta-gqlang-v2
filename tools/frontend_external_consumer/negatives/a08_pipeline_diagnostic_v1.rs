//! Negative case A08_pipeline_diagnostic_v1 (class A: owner symbol through the facade root).
//!
//! `PipelineDiagnosticV1` is a deleted V1 stage diagnostic. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `PipelineDiagnosticV1` in the root".
use quanta_gqlang_v2_frontend_contract::PipelineDiagnosticV1;

fn main() {}
