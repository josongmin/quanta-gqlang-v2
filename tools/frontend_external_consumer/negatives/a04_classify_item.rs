//! Negative case A04_classify_item (class A: owner symbol through the facade root).
//!
//! `classify_item` is a deleted raw-HIR classifier. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `classify_item` in the root".
use quanta_gqlang_v2_frontend_contract::classify_item;

fn main() {}
