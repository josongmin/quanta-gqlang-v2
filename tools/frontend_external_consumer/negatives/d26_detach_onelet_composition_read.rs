//! Borrowed one-LET facts cannot outlive the owner retaining their storage.
use quanta_gqlang_v2_frontend_contract::{
    AdmittedOneLetCompositionReadV3, SharedAdmittedOneLetExecutionDocumentV3,
};

fn detach(
    value: SharedAdmittedOneLetExecutionDocumentV3,
) -> AdmittedOneLetCompositionReadV3<'static> {
    value.composition_read_v3().unwrap()
}

fn main() {
    let _ = detach
        as fn(SharedAdmittedOneLetExecutionDocumentV3) -> AdmittedOneLetCompositionReadV3<'static>;
}
