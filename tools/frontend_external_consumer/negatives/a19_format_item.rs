//! Negative case A19_format_item (class A: owner symbol through the facade root).
//!
//! `format_item` is a surface owner formatter helper. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `format_item` in the root".
use quanta_gqlang_v2_frontend_contract::format_item;

fn main() {}
