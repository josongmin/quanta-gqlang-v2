//! Negative case A23_lower_relation_item (class A: owner symbol through the facade root).
//!
//! `lower_relation_item` is a algebra owner lowering entry point. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `lower_relation_item` in the root".
use quanta_gqlang_v2_frontend_contract::lower_relation_item;

fn main() {}
