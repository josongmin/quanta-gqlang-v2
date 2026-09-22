//! Negative case A24_graph_witness_bounds_v2 (class A: owner symbol through the facade root).
//!
//! `GraphWitnessBoundsV2` is a sema owner policy type (W4-C D02). It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `GraphWitnessBoundsV2` in the root".
use quanta_gqlang_v2_frontend_contract::GraphWitnessBoundsV2;

fn main() {}
