//! The move-only carrier cannot deep-clone an admitted execution payload.
use quanta_gqlang_v2_frontend_contract::AdmittedScalarExecutionDocumentV3;

fn probe(document: AdmittedScalarExecutionDocumentV3) {
    let _ = document.clone();
}

fn main() {
    let _ = probe as fn(AdmittedScalarExecutionDocumentV3);
}
