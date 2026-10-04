//! Borrowed scalar facts cannot outlive the owner retaining their storage.
use quanta_gqlang_v2_frontend_contract::{
    AdmittedCanonicalFilesScalarReadV3, SharedAdmittedScalarExecutionDocumentV3,
};

fn detach(
    value: SharedAdmittedScalarExecutionDocumentV3,
) -> AdmittedCanonicalFilesScalarReadV3<'static> {
    value.canonical_files_scalar_read_v3().unwrap()
}

fn main() {
    let _ = detach
        as fn(
            SharedAdmittedScalarExecutionDocumentV3,
        ) -> AdmittedCanonicalFilesScalarReadV3<'static>;
}
