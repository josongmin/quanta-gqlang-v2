//! Nested safe relation reads cannot outlive the request-owned shared document.
use quanta_gqlang_v2_frontend_contract::{
    ExecutionRelationReadV3, SharedAdmittedScalarExecutionDocumentV3,
};

fn detach(value: SharedAdmittedScalarExecutionDocumentV3) -> ExecutionRelationReadV3<'static> {
    value.relation_read_v3().unwrap()
}

fn main() {
    let _ =
        detach as fn(SharedAdmittedScalarExecutionDocumentV3) -> ExecutionRelationReadV3<'static>;
}
