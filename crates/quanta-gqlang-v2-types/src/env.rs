use crate::{CardinalityV1, RecordTypeV1, RelationTypeV1, TypeAuthorityDiagnosticV2};
use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, SourceRegistryIdentityV2, SourceRegistryViewV2};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProvenanceNamespaceV2(String);
impl ProvenanceNamespaceV2 {
    pub fn try_new(value: impl Into<String>) -> Result<Self, &'static str> {
        let value = value.into();
        if !canonical_segment(&value) { return Err("invalid provenance namespace"); }
        Ok(Self(value))
    }
    #[must_use] pub fn as_str(&self) -> &str { &self.0 }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationSourceIdV2 { namespace: ProvenanceNamespaceV2, qualified_name: String }
impl RelationSourceIdV2 {
    fn derive(namespace: ProvenanceNamespaceV2, qualified_name: &str) -> Result<Self, &'static str> {
        if qualified_name.split("::").any(|part| !canonical_segment(part)) { return Err("invalid qualified relation name"); }
        Ok(Self { namespace, qualified_name: qualified_name.to_owned() })
    }
    #[must_use] pub fn namespace(&self) -> &ProvenanceNamespaceV2 { &self.namespace }
    #[must_use] pub fn qualified_name(&self) -> &str { &self.qualified_name }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationSourceInputV2 {
    pub provenance_namespace: ProvenanceNamespaceV2,
    pub qualified_name: String,
    pub lookup_name: String,
    pub row: RecordTypeV1,
    pub cardinality: CardinalityV1,
    pub provenance: String,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationSourceV2 {
    id: RelationSourceIdV2,
    lookup_name: String,
    relation_type: RelationTypeV1,
    provenance: String,
    location: CheckedLocatedSpanV2,
}
impl RelationSourceV2 {
    #[must_use] pub fn id(&self) -> &RelationSourceIdV2 { &self.id }
    #[must_use] pub fn lookup_name(&self) -> &str { &self.lookup_name }
    #[must_use] pub const fn relation_type(&self) -> &RelationTypeV1 { &self.relation_type }
    #[must_use] pub fn provenance(&self) -> &str { &self.provenance }
    #[must_use] pub const fn location(&self) -> &CheckedLocatedSpanV2 { &self.location }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationCatalogV2 { by_name: BTreeMap<String, RelationSourceV2>, source_registry_identity: SourceRegistryIdentityV2 }
impl RelationCatalogV2 {
    pub fn try_new(
        sources: Vec<RelationSourceInputV2>, source_registry: &SourceRegistryViewV2,
    ) -> Result<Self, TypeAuthorityDiagnosticV2> {
        let mut by_name = BTreeMap::new();
        let registry_identity = source_registry.identity();
        let mut by_id: BTreeMap<RelationSourceIdV2, CheckedLocatedSpanV2> = BTreeMap::new();
        for input in sources {
            if input.location.registry_identity() != &registry_identity {
                return Err(TypeAuthorityDiagnosticV2::SourceRegistryMismatch {
                    location: input.location.location(),
                });
            }
            let raw_location = input.location.location();
            let id = RelationSourceIdV2::derive(input.provenance_namespace, &input.qualified_name)
                .map_err(|_name_error| TypeAuthorityDiagnosticV2::InvalidCanonicalName { name: input.qualified_name.clone(), location: raw_location })?;
            if !canonical_segment(&input.lookup_name) { return Err(TypeAuthorityDiagnosticV2::InvalidCanonicalName { name: input.lookup_name, location: raw_location }); }
            if let Some(original) = by_name.get(&input.lookup_name).map(|existing: &RelationSourceV2| existing.location.location()) {
                return Err(TypeAuthorityDiagnosticV2::DuplicateRelationSource { name: input.lookup_name, primary: raw_location, secondary: original });
            }
            if let Some(original) = by_id.insert(id.clone(), input.location.clone()) {
                return Err(TypeAuthorityDiagnosticV2::DuplicateRelationSourceId { id: format!("{}::{}", id.namespace.as_str(), id.qualified_name), primary: raw_location, secondary: original.location() });
            }
            let relation_type = RelationTypeV1::try_new(input.row, input.cardinality)
                .map_err(TypeAuthorityDiagnosticV2::InvalidRelationType)?;
            let source = RelationSourceV2 { id, lookup_name: input.lookup_name.clone(), relation_type, provenance: input.provenance, location: input.location };
            let _ = by_name.insert(input.lookup_name, source);
        }
        Ok(Self { by_name, source_registry_identity: registry_identity })
    }
    #[must_use] pub fn get(&self, name: &str) -> Option<&RelationSourceV2> { self.by_name.get(name) }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &RelationSourceV2)> { self.by_name.iter() }
    #[must_use] pub fn source_registry_identity(&self) -> &SourceRegistryIdentityV2 { &self.source_registry_identity }
}

fn canonical_segment(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else { return false; };
    (first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}
