#![expect(
    clippy::expect_used,
    reason = "independent canonical descriptor assertions"
)]

use quanta_gqlang_v2_frontend_contract::{
    execution_operator_descriptor_v3, ExecutionOperatorKindV3 as Kind,
    ExecutionScalarTypeV3 as Scalar, ExecutionValueTypeV3 as Value,
};

#[test]
fn canonical_operator_descriptors_expose_semantics_and_exact_domains() {
    let cases = [
        (7, Kind::Not, vec![Scalar::Bool], Scalar::Bool),
        (
            115,
            Kind::And,
            vec![Scalar::Bool, Scalar::Bool],
            Scalar::Bool,
        ),
        (
            116,
            Kind::Or,
            vec![Scalar::Bool, Scalar::Bool],
            Scalar::Bool,
        ),
        (
            117,
            Kind::Xor,
            vec![Scalar::Bool, Scalar::Bool],
            Scalar::Bool,
        ),
        (
            303,
            Kind::Equal,
            vec![Scalar::String, Scalar::String],
            Scalar::Bool,
        ),
        (
            327,
            Kind::NotEqual,
            vec![Scalar::RepoPath, Scalar::RepoPath],
            Scalar::Bool,
        ),
        (
            202,
            Kind::Less,
            vec![Scalar::String, Scalar::String],
            Scalar::Bool,
        ),
        (
            232,
            Kind::GreaterEqual,
            vec![Scalar::String, Scalar::String],
            Scalar::Bool,
        ),
        (101, Kind::Add, vec![Scalar::Int, Scalar::Int], Scalar::Int),
    ];
    for (id, kind, operands, result) in cases {
        let descriptor = execution_operator_descriptor_v3(id)
            .expect("canonical registry")
            .expect("canonical overload");
        assert_eq!(descriptor.kind(), kind);
        assert_eq!(
            descriptor.operand_types(),
            operands.into_iter().map(Value::Scalar).collect::<Vec<_>>()
        );
        assert_eq!(descriptor.result_type(), &Value::Scalar(result));
        assert_eq!(
            execution_operator_descriptor_v3(id).expect("repeat lookup"),
            Some(descriptor)
        );
    }
}

#[test]
fn unissued_operator_ids_do_not_gain_descriptors() {
    for id in [0, 8, 119, 204, 313, 333, u32::MAX] {
        assert_eq!(execution_operator_descriptor_v3(id), Ok(None));
    }
}
