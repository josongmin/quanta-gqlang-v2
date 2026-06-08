#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::entity_schema::{
    PublicEntityFieldSchemaV1, PublicEntityRefV1, PublicEntitySchemaV1,
    PublicFactSchemaAvailabilityV1, PublicFactSchemaDegradedReasonV1, PublicFactSchemaSourceKindV1,
    PublicFactSchemaSourceProvenanceV1, PublicFieldArityV1, PublicFieldTypeV1, PublicScalarTypeV1,
};
use quanta_gqlang_v2_ir::relation_schema::{
    PublicRelationDirectionV1, PublicRelationEndpointCardinalityV1, PublicRelationEndpointSchemaV1,
    PublicRelationFieldSchemaV1, PublicRelationRefV1, PublicRelationSchemaV1,
};

#[test]
fn entity_schema_exposes_public_fields_provenance_and_availability_v1() {
    let schema = PublicEntitySchemaV1 {
        entity: PublicEntityRefV1 {
            path: vec!["catalog".to_string(), "users".to_string()],
        },
        fields: vec![
            PublicEntityFieldSchemaV1 {
                name: "id".to_string(),
                value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::String),
                arity: PublicFieldArityV1::Required,
                provenance: declared_source_v1("model::catalog.users"),
                availability: PublicFactSchemaAvailabilityV1::Available,
            },
            PublicEntityFieldSchemaV1 {
                name: "manager".to_string(),
                value_type: PublicFieldTypeV1::EntityRef(PublicEntityRefV1 {
                    path: vec!["catalog".to_string(), "users".to_string()],
                }),
                arity: PublicFieldArityV1::Optional,
                provenance: declared_source_v1("model::catalog.users"),
                availability: PublicFactSchemaAvailabilityV1::Degraded {
                    reason: PublicFactSchemaDegradedReasonV1::PartialCoverage,
                },
            },
        ],
        provenance: declared_source_v1("model::catalog.users"),
        availability: PublicFactSchemaAvailabilityV1::Available,
    };

    assert_eq!(schema.validate_v1(), Ok(()));
    assert_eq!(schema.entity.canonical_name_v1(), "catalog.users");
    assert_eq!(schema.fields[1].name, "manager");
    assert_eq!(
        schema.fields[1].availability,
        PublicFactSchemaAvailabilityV1::Degraded {
            reason: PublicFactSchemaDegradedReasonV1::PartialCoverage,
        }
    );
}

#[test]
fn entity_schema_fails_closed_on_duplicate_field_names_v1() {
    let schema = PublicEntitySchemaV1 {
        entity: PublicEntityRefV1 {
            path: vec!["catalog".to_string(), "repos".to_string()],
        },
        fields: vec![
            PublicEntityFieldSchemaV1 {
                name: "name".to_string(),
                value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::String),
                arity: PublicFieldArityV1::Required,
                provenance: declared_source_v1("model::catalog.repos"),
                availability: PublicFactSchemaAvailabilityV1::Available,
            },
            PublicEntityFieldSchemaV1 {
                name: "name".to_string(),
                value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::String),
                arity: PublicFieldArityV1::Optional,
                provenance: declared_source_v1("model::catalog.repos"),
                availability: PublicFactSchemaAvailabilityV1::Available,
            },
        ],
        provenance: declared_source_v1("model::catalog.repos"),
        availability: PublicFactSchemaAvailabilityV1::Available,
    };

    let err = schema
        .validate_v1()
        .expect_err("duplicate field names must fail closed");
    assert_eq!(err.code, "entity_field_name_duplicate");
}

#[test]
fn relation_schema_exposes_direction_cardinality_and_relation_fields_v1() {
    let schema = PublicRelationSchemaV1 {
        relation: PublicRelationRefV1 {
            path: vec!["catalog".to_string(), "repo_owns_issue".to_string()],
        },
        source: PublicRelationEndpointSchemaV1 {
            entity: PublicEntityRefV1 {
                path: vec!["catalog".to_string(), "repos".to_string()],
            },
            role: "owner".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::One,
        },
        target: PublicRelationEndpointSchemaV1 {
            entity: PublicEntityRefV1 {
                path: vec!["catalog".to_string(), "issues".to_string()],
            },
            role: "owned".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::Many,
        },
        direction: PublicRelationDirectionV1::Directed,
        fields: vec![PublicRelationFieldSchemaV1 {
            name: "created_at".to_string(),
            value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::Timestamp),
            arity: PublicFieldArityV1::Required,
            provenance: observed_source_v1("catalog_snapshot::repo_issue_edge"),
            availability: PublicFactSchemaAvailabilityV1::Degraded {
                reason: PublicFactSchemaDegradedReasonV1::ExampleDerivedOnly,
            },
        }],
        provenance: observed_source_v1("catalog_snapshot::repo_issue_edge"),
        availability: PublicFactSchemaAvailabilityV1::Degraded {
            reason: PublicFactSchemaDegradedReasonV1::ExampleDerivedOnly,
        },
    };

    assert_eq!(schema.validate_v1(), Ok(()));
    assert_eq!(
        schema.relation.canonical_name_v1(),
        "catalog.repo_owns_issue"
    );
    assert_eq!(
        schema.source.cardinality,
        PublicRelationEndpointCardinalityV1::One
    );
    assert_eq!(
        schema.target.cardinality,
        PublicRelationEndpointCardinalityV1::Many
    );
    assert_eq!(schema.direction, PublicRelationDirectionV1::Directed);
}

#[test]
fn relation_schema_fails_closed_on_blank_endpoint_role_and_empty_source_ref_v1() {
    let blank_role = PublicRelationSchemaV1 {
        relation: PublicRelationRefV1 {
            path: vec!["catalog".to_string(), "repo_depends_on_repo".to_string()],
        },
        source: PublicRelationEndpointSchemaV1 {
            entity: PublicEntityRefV1 {
                path: vec!["catalog".to_string(), "repos".to_string()],
            },
            role: " ".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::Many,
        },
        target: PublicRelationEndpointSchemaV1 {
            entity: PublicEntityRefV1 {
                path: vec!["catalog".to_string(), "repos".to_string()],
            },
            role: "dependency".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::Many,
        },
        direction: PublicRelationDirectionV1::Bidirectional,
        fields: Vec::new(),
        provenance: observed_source_v1("catalog_snapshot::repo_dependency"),
        availability: PublicFactSchemaAvailabilityV1::Available,
    };

    let blank_role_err = blank_role
        .validate_v1()
        .expect_err("blank relation endpoint role must fail closed");
    assert_eq!(blank_role_err.code, "relation_endpoint_role");

    let empty_source_ref = PublicRelationSchemaV1 {
        relation: PublicRelationRefV1 {
            path: vec!["catalog".to_string(), "repo_has_label".to_string()],
        },
        source: PublicRelationEndpointSchemaV1 {
            entity: PublicEntityRefV1 {
                path: vec!["catalog".to_string(), "repos".to_string()],
            },
            role: "repo".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::One,
        },
        target: PublicRelationEndpointSchemaV1 {
            entity: PublicEntityRefV1 {
                path: vec!["catalog".to_string(), "labels".to_string()],
            },
            role: "label".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::Many,
        },
        direction: PublicRelationDirectionV1::Directed,
        fields: vec![PublicRelationFieldSchemaV1 {
            name: "applied_by".to_string(),
            value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::String),
            arity: PublicFieldArityV1::Optional,
            provenance: PublicFactSchemaSourceProvenanceV1 {
                kind: PublicFactSchemaSourceKindV1::Observed,
                source_ref: String::new(),
            },
            availability: PublicFactSchemaAvailabilityV1::Available,
        }],
        provenance: observed_source_v1("catalog_snapshot::repo_label"),
        availability: PublicFactSchemaAvailabilityV1::Available,
    };

    let source_ref_err = empty_source_ref
        .validate_v1()
        .expect_err("empty nested provenance source_ref must fail closed");
    assert_eq!(source_ref_err.code, "fact_schema_source_ref_empty");
}

fn declared_source_v1(source_ref: &str) -> PublicFactSchemaSourceProvenanceV1 {
    PublicFactSchemaSourceProvenanceV1 {
        kind: PublicFactSchemaSourceKindV1::Declared,
        source_ref: source_ref.to_string(),
    }
}

fn observed_source_v1(source_ref: &str) -> PublicFactSchemaSourceProvenanceV1 {
    PublicFactSchemaSourceProvenanceV1 {
        kind: PublicFactSchemaSourceKindV1::Observed,
        source_ref: source_ref.to_string(),
    }
}
