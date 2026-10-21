//! A nested read must not expose a cloneable owned operator tree.
use quanta_gqlang_v2_frontend_contract::{
    ExecutionRelationalOperatorV3, SharedAdmittedScalarExecutionDocumentV3,
};

fn detach(document: &SharedAdmittedScalarExecutionDocumentV3) -> ExecutionRelationalOperatorV3 {
    document
        .relation_read_v3()
        .expect("scalar relation")
        .root()
        .clone()
}

fn main() {}
