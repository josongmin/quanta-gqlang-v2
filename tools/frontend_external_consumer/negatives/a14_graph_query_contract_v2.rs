//! Negative case A14_graph_query_contract_v2 (class A: owner symbol through the facade root).
//!
//! `GraphQueryContractV2` is a graph owner contract type. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `GraphQueryContractV2` in the root".
use quanta_gqlang_v2_frontend_contract::GraphQueryContractV2;

fn main() {}
