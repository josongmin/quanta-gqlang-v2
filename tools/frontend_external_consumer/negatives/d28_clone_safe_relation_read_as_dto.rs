//! Cloning the safe relation read cannot synthesize an unowned relation DTO.
use quanta_gqlang_v2_frontend_contract::{
    ExecutionRelationV3, SharedAdmittedScalarExecutionDocumentV3,
};

fn clone_dto(value: &SharedAdmittedScalarExecutionDocumentV3) -> ExecutionRelationV3 {
    value.relation_read_v3().unwrap().clone()
}

fn main() {
    let _ = clone_dto as fn(&SharedAdmittedScalarExecutionDocumentV3) -> ExecutionRelationV3;
}
