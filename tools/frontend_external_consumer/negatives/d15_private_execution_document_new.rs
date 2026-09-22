//! Negative case D15_private_execution_document_new (class D: V3 seal privacy).
//!
//! A sibling consumer can inspect an issued execution document but cannot
//! mint one from selected items and caller-supplied source bytes.
//!
//! Expected: E0624, "associated function `new` is private".
use quanta_gqlang_v2_frontend_contract::{ExecutionDocumentV3, SourceIdV2};

fn main() {
    let _ = ExecutionDocumentV3::new(SourceIdV2::from_raw(9), b"forged", Vec::new());
}
