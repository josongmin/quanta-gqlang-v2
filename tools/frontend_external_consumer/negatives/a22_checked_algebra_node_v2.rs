//! Negative case A22_checked_algebra_node_v2 (class A: owner symbol through the facade root).
//!
//! `CheckedAlgebraNodeV2` is a algebra owner node type. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `CheckedAlgebraNodeV2` in the root".
use quanta_gqlang_v2_frontend_contract::CheckedAlgebraNodeV2;

fn main() {}
