//! Sharing a one-LET owner does not expose its backing scalar owner or DTO.
use quanta_gqlang_v2_frontend_contract::SharedAdmittedOneLetExecutionDocumentV3;

fn probe(document: &SharedAdmittedOneLetExecutionDocumentV3) {
    let _ = &document.0;
}

fn main() {
    let _ = probe as fn(&SharedAdmittedOneLetExecutionDocumentV3);
}
