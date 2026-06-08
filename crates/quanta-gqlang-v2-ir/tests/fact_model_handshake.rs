#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_ir::entity_schema::{
    PublicEntityFieldSchemaV1, PublicEntityRefV1, PublicEntitySchemaV1,
    PublicFactSchemaAvailabilityV1, PublicFactSchemaSourceKindV1,
    PublicFactSchemaSourceProvenanceV1, PublicFieldArityV1, PublicFieldTypeV1, PublicScalarTypeV1,
};
use quanta_gqlang_v2_ir::relation_schema::{
    PublicRelationDirectionV1, PublicRelationEndpointCardinalityV1, PublicRelationEndpointSchemaV1,
    PublicRelationFieldSchemaV1, PublicRelationRefV1, PublicRelationSchemaV1,
};

#[test]
fn fact_model_handshake_keeps_public_entity_and_relation_surface_host_agnostic_v1() {
    let entity = PublicEntitySchemaV1 {
        entity: PublicEntityRefV1 {
            path: vec!["catalog".to_string(), "deployments".to_string()],
        },
        fields: vec![PublicEntityFieldSchemaV1 {
            name: "environment".to_string(),
            value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::String),
            arity: PublicFieldArityV1::Required,
            provenance: declared_source_v1("model::catalog.deployments"),
            availability: PublicFactSchemaAvailabilityV1::Available,
        }],
        provenance: declared_source_v1("model::catalog.deployments"),
        availability: PublicFactSchemaAvailabilityV1::Available,
    };
    assert_eq!(entity.validate_v1(), Ok(()));

    let relation = PublicRelationSchemaV1 {
        relation: PublicRelationRefV1 {
            path: vec![
                "catalog".to_string(),
                "deployment_owns_artifact".to_string(),
            ],
        },
        source: PublicRelationEndpointSchemaV1 {
            entity: PublicEntityRefV1 {
                path: vec!["catalog".to_string(), "deployments".to_string()],
            },
            role: "deployment".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::One,
        },
        target: PublicRelationEndpointSchemaV1 {
            entity: PublicEntityRefV1 {
                path: vec!["catalog".to_string(), "artifacts".to_string()],
            },
            role: "artifact".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::Many,
        },
        direction: PublicRelationDirectionV1::Directed,
        fields: vec![PublicRelationFieldSchemaV1 {
            name: "released_at".to_string(),
            value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::Timestamp),
            arity: PublicFieldArityV1::Required,
            provenance: observed_source_v1("catalog_snapshot::deployment_artifact"),
            availability: PublicFactSchemaAvailabilityV1::Available,
        }],
        provenance: observed_source_v1("catalog_snapshot::deployment_artifact"),
        availability: PublicFactSchemaAvailabilityV1::Available,
    };
    assert_eq!(relation.validate_v1(), Ok(()));
}

#[test]
fn fact_model_handshake_rejects_runtime_only_rowset_leaks_fail_closed_v1() {
    let entity = PublicEntitySchemaV1 {
        entity: PublicEntityRefV1 {
            path: vec!["catalog".to_string(), "artifacts".to_string()],
        },
        fields: vec![PublicEntityFieldSchemaV1 {
            name: "rowset_id".to_string(),
            value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::String),
            arity: PublicFieldArityV1::Optional,
            provenance: declared_source_v1("model::catalog.artifacts"),
            availability: PublicFactSchemaAvailabilityV1::Available,
        }],
        provenance: declared_source_v1("model::catalog.artifacts"),
        availability: PublicFactSchemaAvailabilityV1::Available,
    };
    let entity_err = entity
        .validate_v1()
        .expect_err("entity field rowset markers must fail closed");
    assert_eq!(entity_err.code, "entity_field_name_runtime_local");

    let relation = PublicRelationSchemaV1 {
        relation: PublicRelationRefV1 {
            path: vec!["catalog".to_string(), "artifact_has_finding".to_string()],
        },
        source: PublicRelationEndpointSchemaV1 {
            entity: PublicEntityRefV1 {
                path: vec!["catalog".to_string(), "artifacts".to_string()],
            },
            role: "artifact".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::One,
        },
        target: PublicRelationEndpointSchemaV1 {
            entity: PublicEntityRefV1 {
                path: vec!["catalog".to_string(), "findings".to_string()],
            },
            role: "finding".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::Many,
        },
        direction: PublicRelationDirectionV1::Directed,
        fields: vec![PublicRelationFieldSchemaV1 {
            name: "linked_at".to_string(),
            value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::Timestamp),
            arity: PublicFieldArityV1::Required,
            provenance: observed_source_v1("runtime::rowset::artifact_has_finding"),
            availability: PublicFactSchemaAvailabilityV1::Available,
        }],
        provenance: observed_source_v1("catalog_snapshot::artifact_has_finding"),
        availability: PublicFactSchemaAvailabilityV1::Available,
    };
    let relation_err = relation
        .validate_v1()
        .expect_err("runtime-local rowset provenance must fail closed");
    assert_eq!(relation_err.code, "fact_schema_source_ref_runtime_local");
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
