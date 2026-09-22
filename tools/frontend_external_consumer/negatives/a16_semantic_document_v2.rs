//! Negative case A16_semantic_document_v2 (class A: owner symbol through the facade root).
//!
//! `SemanticDocumentV2` is a sema owner sealed document. It must stay unreachable from
//! `quanta_gqlang_v2_frontend_contract`: the facade is a curated allowlist, not
//! a re-export of an owner root.
//!
//! Expected: E0432, "no `SemanticDocumentV2` in the root".
use quanta_gqlang_v2_frontend_contract::SemanticDocumentV2;

fn main() {}
