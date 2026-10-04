//! Sharing an owner does not expose its backing allocation or DTO.
use quanta_gqlang_v2_frontend_contract::SharedAdmittedScalarExecutionDocumentV3;

fn probe(document: &SharedAdmittedScalarExecutionDocumentV3) {
    let _ = &document.0;
}

fn main() {
    let _ = probe as fn(&SharedAdmittedScalarExecutionDocumentV3);
}
