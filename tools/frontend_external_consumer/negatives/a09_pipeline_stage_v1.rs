//! Negative case A09_pipeline_stage_v1 (class A: owner symbol through the facade root).
//!
//! `PipelineStageV1` is a deleted V1 stage tag. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `PipelineStageV1` in the root".
use quanta_gqlang_v2_frontend_contract::PipelineStageV1;

fn main() {}
