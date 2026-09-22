//! Negative case A18_parse_document (class A: owner symbol through the facade root).
//!
//! `parse_document` is a surface owner parser entry point. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `parse_document` in the root".
use quanta_gqlang_v2_frontend_contract::parse_document;

fn main() {}
