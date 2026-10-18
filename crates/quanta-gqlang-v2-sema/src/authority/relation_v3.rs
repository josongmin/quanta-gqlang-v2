//! Seal existing catalog inputs, including exact Host instance identity.

use super::{
    configuration_failure, AuthorityBudgetProfileV2, FrontendAuthorityErrorV2,
    FrontendRelationInputV2, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1,
    RelationCatalogV2, RelationSourceInputV2, SourceRegistryViewV2,
};

pub(super) fn seal_relations(
    host_sources: &SourceRegistryViewV2,
    relation_declarations: Vec<FrontendRelationInputV2>,
    profile: AuthorityBudgetProfileV2,
) -> Result<RelationCatalogV2, FrontendAuthorityErrorV2> {
    let mut relation_inputs = Vec::with_capacity(relation_declarations.len());
    for relation in relation_declarations {
        let location = relation.location.check(host_sources)?;
        let namespace =
            ProvenanceNamespaceV2::try_new(&relation.provenance_namespace, location.clone())
                .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        let mut fields = Vec::with_capacity(relation.fields.len());
        for field in relation.fields {
            let field_location = field.location.check(host_sources)?;
            let value_type = field.owner_value_type_v2()?;
            fields.push(
                RecordFieldTypeV1::try_new(&field.name, value_type, field_location)
                    .map_err(|error| configuration_failure(error.code(), error.to_string()))?,
            );
        }
        let row = RecordTypeV1::try_new(fields)
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        relation_inputs.push(RelationSourceInputV2 {
            host_binding_identity: relation.host_binding_identity,
            authority: relation.authority.owner(),
            provenance_namespace: namespace,
            qualified_name: relation.qualified_name,
            lookup_name: relation.lookup_name,
            row,
            cardinality: relation.cardinality.owner(),
            provenance: relation.provenance,
            location,
        });
    }
    let relations = RelationCatalogV2::try_new(relation_inputs, host_sources, profile.clone())
        .map_err(|error| configuration_failure(error.code(), error.to_string()))?;

    Ok(relations)
}
