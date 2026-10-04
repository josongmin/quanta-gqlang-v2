//! A sibling cannot extract the cloneable DTO or its request permits.
use quanta_gqlang_v2_frontend_contract::AdmittedScalarExecutionDocumentV3;

fn probe(document: &AdmittedScalarExecutionDocumentV3) {
    let _ = &document.owner;
}

fn main() {
    let _ = probe as fn(&AdmittedScalarExecutionDocumentV3);
}
