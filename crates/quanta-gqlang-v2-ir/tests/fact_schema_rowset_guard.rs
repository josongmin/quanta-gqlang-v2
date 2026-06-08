#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! The runtime-rowset transport guard must:
//!   1. catch the `rowset` marker on EVERY public identifier — field names,
//!      entity/relation path segments, and endpoint roles — not just field names
//!      and provenance, and
//!   2. use token boundaries so benign words such as `escrow_set` / `borrow_set`
//!      (which collapsed to `escrowset` / `borrowset` under the old
//!      separator-stripping `contains("rowset")`) are accepted.

use quanta_gqlang_v2_ir::entity_schema::{
    PublicEntityFieldSchemaV1, PublicEntityRefV1, PublicEntitySchemaV1,
    PublicFactSchemaAvailabilityV1, PublicFactSchemaSourceKindV1,
    PublicFactSchemaSourceProvenanceV1, PublicFieldArityV1, PublicFieldTypeV1, PublicScalarTypeV1,
};
use quanta_gqlang_v2_ir::relation_schema::{
    PublicRelationDirectionV1, PublicRelationEndpointCardinalityV1, PublicRelationEndpointSchemaV1,
    PublicRelationRefV1, PublicRelationSchemaV1,
};

fn provenance() -> PublicFactSchemaSourceProvenanceV1 {
    PublicFactSchemaSourceProvenanceV1 {
        kind: PublicFactSchemaSourceKindV1::Declared,
        source_ref: "model::catalog.demo".to_string(),
    }
}

fn field(name: &str) -> PublicEntityFieldSchemaV1 {
    PublicEntityFieldSchemaV1 {
        name: name.to_string(),
        value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::Integer),
        arity: PublicFieldArityV1::Required,
        provenance: provenance(),
        availability: PublicFactSchemaAvailabilityV1::Available,
    }
}

fn entity(path: &[&str], fields: Vec<PublicEntityFieldSchemaV1>) -> PublicEntitySchemaV1 {
    PublicEntitySchemaV1 {
        entity: PublicEntityRefV1 {
            path: path.iter().map(|segment| (*segment).to_string()).collect(),
        },
        fields,
        provenance: provenance(),
        availability: PublicFactSchemaAvailabilityV1::Available,
    }
}

fn endpoint(role: &str) -> PublicRelationEndpointSchemaV1 {
    PublicRelationEndpointSchemaV1 {
        entity: PublicEntityRefV1 {
            path: vec!["catalog".to_string(), "node".to_string()],
        },
        role: role.to_string(),
        cardinality: PublicRelationEndpointCardinalityV1::One,
    }
}

fn relation(path: &[&str], source: PublicRelationEndpointSchemaV1) -> PublicRelationSchemaV1 {
    PublicRelationSchemaV1 {
        relation: PublicRelationRefV1 {
            path: path.iter().map(|segment| (*segment).to_string()).collect(),
        },
        source,
        target: endpoint("artifact"),
        direction: PublicRelationDirectionV1::Directed,
        fields: Vec::new(),
        provenance: provenance(),
        availability: PublicFactSchemaAvailabilityV1::Available,
    }
}

#[test]
fn benign_set_suffixed_field_names_are_accepted() {
    for name in ["escrow_set", "borrow_set", "narrow_set", "arrow_set"] {
        let schema = entity(&["catalog", "demo"], vec![field(name)]);
        assert_eq!(schema.validate_v1(), Ok(()), "{name} must be accepted");
    }
}

#[test]
fn rowset_field_name_is_still_rejected() {
    let schema = entity(&["catalog", "demo"], vec![field("rowset_id")]);
    let result = schema.validate_v1();
    assert!(
        matches!(&result, Err(error) if error.code == "entity_field_name_runtime_local"),
        "got {result:?}"
    );
}

#[test]
fn rowset_entity_path_segment_is_rejected() {
    let schema = entity(&["catalog", "rowset_view"], vec![field("id")]);
    let result = schema.validate_v1();
    assert!(
        matches!(&result, Err(error) if error.code == "entity_path_runtime_local"),
        "got {result:?}"
    );
}

#[test]
fn rowset_entity_ref_field_target_is_rejected() {
    let mut bad_field = field("owner");
    bad_field.value_type = PublicFieldTypeV1::EntityRef(PublicEntityRefV1 {
        path: vec!["catalog".to_string(), "rowset_owner".to_string()],
    });
    let schema = entity(&["catalog", "demo"], vec![bad_field]);
    let result = schema.validate_v1();
    assert!(
        matches!(&result, Err(error) if error.code == "entity_path_runtime_local"),
        "got {result:?}"
    );
}

#[test]
fn rowset_endpoint_role_is_rejected() {
    let schema = relation(&["catalog", "edge"], endpoint("rowset"));
    let result = schema.validate_v1();
    assert!(
        matches!(&result, Err(error) if error.code == "relation_endpoint_role_runtime_local"),
        "got {result:?}"
    );
}

#[test]
fn rowset_relation_path_segment_is_rejected() {
    let schema = relation(&["catalog", "rowset_edge"], endpoint("source"));
    let result = schema.validate_v1();
    assert!(
        matches!(&result, Err(error) if error.code == "relation_path_runtime_local"),
        "got {result:?}"
    );
}
