//! The old callback must not return a cloned relation DTO outside the owner.
use quanta_gqlang_v2_frontend_contract::{
    ExecutionRelationV3, SharedAdmittedScalarExecutionDocumentV3,
};

fn detach(value: &SharedAdmittedScalarExecutionDocumentV3) -> ExecutionRelationV3 {
    value
        .with_sealed_relation_v3(|relation| relation.clone())
        .unwrap()
}

fn main() {
    let _ = detach as fn(&SharedAdmittedScalarExecutionDocumentV3) -> ExecutionRelationV3;
}
