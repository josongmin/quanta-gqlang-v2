//! E11A — the catalog-facing registry that makes the public fact/relation schema
//! surface reachable from DESCRIBE roots.
//!
//! The schema *types* live in `facts/{entity_schema,relation_schema}.rs`; before
//! this module they were a defined-but-unreferenced island (only IR tests touched
//! them). This registry holds the canonical published instances and exposes the
//! lookup the catalog-facing roots (CLI DESCRIBE summary, LSP DESCRIBE hover)
//! consume, so `DESCRIBE <entity|relation>` renders real schema fields instead of
//! echoing the raw target string.
//!
//! Entity/relation names mirror the `fact_model_handshake` fixture so the
//! published surface and the host-agnostic handshake never drift apart.

use std::sync::LazyLock;

use crate::entity_schema::{
    PublicEntityFieldSchemaV1, PublicEntityRefV1, PublicEntitySchemaV1,
    PublicFactSchemaAvailabilityV1, PublicFactSchemaSourceKindV1,
    PublicFactSchemaSourceProvenanceV1, PublicFieldArityV1, PublicFieldTypeV1, PublicScalarTypeV1,
};
use crate::relation_schema::{
    PublicRelationDirectionV1, PublicRelationEndpointCardinalityV1, PublicRelationEndpointSchemaV1,
    PublicRelationFieldSchemaV1, PublicRelationRefV1, PublicRelationSchemaV1,
};

fn declared_v1(source_ref: &str) -> PublicFactSchemaSourceProvenanceV1 {
    PublicFactSchemaSourceProvenanceV1 {
        kind: PublicFactSchemaSourceKindV1::Declared,
        source_ref: source_ref.to_string(),
    }
}

fn observed_v1(source_ref: &str) -> PublicFactSchemaSourceProvenanceV1 {
    PublicFactSchemaSourceProvenanceV1 {
        kind: PublicFactSchemaSourceKindV1::Observed,
        source_ref: source_ref.to_string(),
    }
}

fn entity_ref_v1(path: &[&str]) -> PublicEntityRefV1 {
    PublicEntityRefV1 {
        path: path.iter().map(|segment| (*segment).to_string()).collect(),
    }
}

fn scalar_field_v1(
    name: &str,
    scalar: PublicScalarTypeV1,
    arity: PublicFieldArityV1,
    provenance: PublicFactSchemaSourceProvenanceV1,
) -> PublicEntityFieldSchemaV1 {
    PublicEntityFieldSchemaV1 {
        name: name.to_string(),
        value_type: PublicFieldTypeV1::Scalar(scalar),
        arity,
        provenance,
        availability: PublicFactSchemaAvailabilityV1::Available,
    }
}

/// The canonical published entities. Each is `validate_v1`-clean (asserted by the
/// registry integrity test) so a malformed published schema fails the build, not
/// a DESCRIBE at runtime.
static PUBLIC_ENTITY_SCHEMAS_V1: LazyLock<Vec<PublicEntitySchemaV1>> = LazyLock::new(|| {
    vec![
        PublicEntitySchemaV1 {
            entity: entity_ref_v1(&["catalog", "deployments"]),
            fields: vec![
                scalar_field_v1(
                    "environment",
                    PublicScalarTypeV1::String,
                    PublicFieldArityV1::Required,
                    declared_v1("model::catalog.deployments"),
                ),
                scalar_field_v1(
                    "released_at",
                    PublicScalarTypeV1::Timestamp,
                    PublicFieldArityV1::Optional,
                    declared_v1("model::catalog.deployments"),
                ),
            ],
            provenance: declared_v1("model::catalog.deployments"),
            availability: PublicFactSchemaAvailabilityV1::Available,
        },
        PublicEntitySchemaV1 {
            entity: entity_ref_v1(&["catalog", "artifacts"]),
            fields: vec![
                scalar_field_v1(
                    "digest",
                    PublicScalarTypeV1::String,
                    PublicFieldArityV1::Required,
                    declared_v1("model::catalog.artifacts"),
                ),
                scalar_field_v1(
                    "size_bytes",
                    PublicScalarTypeV1::Integer,
                    PublicFieldArityV1::Optional,
                    declared_v1("model::catalog.artifacts"),
                ),
            ],
            provenance: declared_v1("model::catalog.artifacts"),
            availability: PublicFactSchemaAvailabilityV1::Available,
        },
    ]
});

/// The canonical published relations.
static PUBLIC_RELATION_SCHEMAS_V1: LazyLock<Vec<PublicRelationSchemaV1>> = LazyLock::new(|| {
    vec![PublicRelationSchemaV1 {
        relation: PublicRelationRefV1 {
            path: vec![
                "catalog".to_string(),
                "deployment_owns_artifact".to_string(),
            ],
        },
        source: PublicRelationEndpointSchemaV1 {
            entity: entity_ref_v1(&["catalog", "deployments"]),
            role: "deployment".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::One,
        },
        target: PublicRelationEndpointSchemaV1 {
            entity: entity_ref_v1(&["catalog", "artifacts"]),
            role: "artifact".to_string(),
            cardinality: PublicRelationEndpointCardinalityV1::Many,
        },
        direction: PublicRelationDirectionV1::Directed,
        fields: vec![PublicRelationFieldSchemaV1 {
            name: "released_at".to_string(),
            value_type: PublicFieldTypeV1::Scalar(PublicScalarTypeV1::Timestamp),
            arity: PublicFieldArityV1::Required,
            provenance: observed_v1("catalog_snapshot::deployment_artifact"),
            availability: PublicFactSchemaAvailabilityV1::Available,
        }],
        provenance: observed_v1("catalog_snapshot::deployment_artifact"),
        availability: PublicFactSchemaAvailabilityV1::Available,
    }]
});

/// Resolve a published entity schema by its canonical dotted name.
#[must_use]
pub fn entity_schema_for_v1(name: &str) -> Option<&'static PublicEntitySchemaV1> {
    PUBLIC_ENTITY_SCHEMAS_V1
        .iter()
        .find(|schema| schema.entity.canonical_name_v1() == name)
}

/// Resolve a published relation schema by its canonical dotted name.
#[must_use]
pub fn relation_schema_for_v1(name: &str) -> Option<&'static PublicRelationSchemaV1> {
    PUBLIC_RELATION_SCHEMAS_V1
        .iter()
        .find(|schema| schema.relation.canonical_name_v1() == name)
}

/// A host-agnostic label for a field's value type, for DESCRIBE rendering.
#[must_use]
pub fn field_type_label_v1(value_type: &PublicFieldTypeV1) -> String {
    match value_type {
        PublicFieldTypeV1::Scalar(scalar) => format!("{scalar:?}").to_lowercase(),
        PublicFieldTypeV1::EntityRef(entity) => format!("ref:{}", entity.canonical_name_v1()),
    }
}

/// Strip a DESCRIBE `<kind>::` prefix (e.g. `model::catalog.deployments`) down to
/// the bare canonical fact name (`catalog.deployments`). A target with no `::`
/// passes through unchanged so the direct canonical form also resolves.
fn canonical_fact_name_v1(target: &str) -> &str {
    target.split_once("::").map_or(target, |(_kind, name)| name)
}

/// The `(field-name, type-label)` rows a DESCRIBE root renders for `target`, or
/// `None` when `target` is not a published entity or relation (fail-closed: the
/// catalog never invents fields for an unknown target). `target` may be either a
/// bare canonical name or a DESCRIBE `<kind>::<name>` reference.
#[must_use]
pub fn describe_schema_rows_v1(target: &str) -> Option<Vec<(String, String)>> {
    let name = canonical_fact_name_v1(target);
    if let Some(entity) = entity_schema_for_v1(name) {
        return Some(
            entity
                .fields
                .iter()
                .map(|field| (field.name.clone(), field_type_label_v1(&field.value_type)))
                .collect(),
        );
    }
    if let Some(relation) = relation_schema_for_v1(name) {
        return Some(
            relation
                .fields
                .iter()
                .map(|field| (field.name.clone(), field_type_label_v1(&field.value_type)))
                .collect(),
        );
    }
    None
}

/// Every published entity name, for enumeration surfaces (SHOW, completion).
#[must_use]
pub fn published_entity_names_v1() -> Vec<String> {
    PUBLIC_ENTITY_SCHEMAS_V1
        .iter()
        .map(|schema| schema.entity.canonical_name_v1())
        .collect()
}
