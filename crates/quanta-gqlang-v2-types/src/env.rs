use crate::{CardinalityV1, RecordTypeV1, RelationTypeV1, TypeAuthorityDiagnosticV2};
use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, SourceRegistryIdentityV2, SourceRegistryViewV2};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuthorityBudgetProfileV2 {
    module_nodes: usize,
    module_edges: usize,
    module_steps: usize,
    authority_bytes: usize,
    authority_string_bytes: usize,
}

impl AuthorityBudgetProfileV2 {
    pub fn try_new(
        module_nodes: usize,
        module_edges: usize,
        module_steps: usize,
        authority_bytes: usize,
        authority_string_bytes: usize,
    ) -> Result<Self, TypeAuthorityDiagnosticV2> {
        if [
            module_nodes,
            module_edges,
            module_steps,
            authority_bytes,
            authority_string_bytes,
        ]
        .contains(&0)
        {
            return Err(TypeAuthorityDiagnosticV2::InvalidAuthorityBudget);
        }
        Ok(Self {
            module_nodes,
            module_edges,
            module_steps,
            authority_bytes,
            authority_string_bytes,
        })
    }

    #[must_use]
    pub const fn module_nodes(&self) -> usize {
        self.module_nodes
    }
    #[must_use]
    pub const fn module_edges(&self) -> usize {
        self.module_edges
    }
    #[must_use]
    pub const fn module_steps(&self) -> usize {
        self.module_steps
    }
    #[must_use]
    pub const fn authority_bytes(&self) -> usize {
        self.authority_bytes
    }
    #[must_use]
    pub const fn authority_string_bytes(&self) -> usize {
        self.authority_string_bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProvenanceNamespaceV2(String);
impl ProvenanceNamespaceV2 {
    pub fn try_new(
        value: &str,
        location: CheckedLocatedSpanV2,
    ) -> Result<Self, TypeAuthorityDiagnosticV2> {
        if !canonical_segment(value) {
            return Err(TypeAuthorityDiagnosticV2::InvalidCanonicalName {
                name: diagnostic_name(value),
                location: location.location(),
            });
        }
        Ok(Self(value.to_owned()))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelationSourceIdV2 {
    namespace: ProvenanceNamespaceV2,
    qualified_name: String,
}
impl RelationSourceIdV2 {
    fn derive(
        namespace: ProvenanceNamespaceV2,
        qualified_name: &str,
    ) -> Result<Self, &'static str> {
        if qualified_name.len() > crate::MAX_TYPE_NAME_BYTES
            || qualified_name
                .split("::")
                .any(|part| !canonical_segment(part))
        {
            return Err("invalid qualified relation name");
        }
        Ok(Self {
            namespace,
            qualified_name: qualified_name.to_owned(),
        })
    }
    #[must_use]
    pub fn namespace(&self) -> &ProvenanceNamespaceV2 {
        &self.namespace
    }
    #[must_use]
    pub fn qualified_name(&self) -> &str {
        &self.qualified_name
    }
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
    #[must_use]
    pub fn id(&self) -> &RelationSourceIdV2 {
        &self.id
    }
    #[must_use]
    pub fn lookup_name(&self) -> &str {
        &self.lookup_name
    }
    #[must_use]
    pub const fn relation_type(&self) -> &RelationTypeV1 {
        &self.relation_type
    }
    #[must_use]
    pub fn provenance(&self) -> &str {
        &self.provenance
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelationCatalogV2 {
    by_name: BTreeMap<String, RelationSourceV2>,
    source_registry_identity: SourceRegistryIdentityV2,
    budget_profile: AuthorityBudgetProfileV2,
}
impl RelationCatalogV2 {
    pub fn try_new(
        sources: Vec<RelationSourceInputV2>,
        source_registry: &SourceRegistryViewV2,
        budget_profile: AuthorityBudgetProfileV2,
    ) -> Result<Self, TypeAuthorityDiagnosticV2> {
        let mut by_name = BTreeMap::new();
        let registry_identity = source_registry.identity();
        let mut by_id: BTreeMap<RelationSourceIdV2, CheckedLocatedSpanV2> = BTreeMap::new();
        let mut authority_bytes = 0usize;
        for input in sources {
            for value in [
                input.provenance_namespace.as_str(),
                input.qualified_name.as_str(),
                input.lookup_name.as_str(),
                input.provenance.as_str(),
            ] {
                if value.len() > budget_profile.authority_string_bytes {
                    return Err(TypeAuthorityDiagnosticV2::ResourceExhausted {
                        dimension: "authority_string_bytes",
                        limit: budget_profile.authority_string_bytes,
                        observed: value.len(),
                    });
                }
                authority_bytes = authority_bytes.saturating_add(value.len());
                if authority_bytes > budget_profile.authority_bytes {
                    return Err(TypeAuthorityDiagnosticV2::ResourceExhausted {
                        dimension: "authority_bytes",
                        limit: budget_profile.authority_bytes,
                        observed: authority_bytes,
                    });
                }
            }
            if input.location.registry_identity() != &registry_identity {
                return Err(TypeAuthorityDiagnosticV2::SourceRegistryMismatch {
                    location: input.location.location(),
                });
            }
            let raw_location = input.location.location();
            let id = RelationSourceIdV2::derive(input.provenance_namespace, &input.qualified_name)
                .map_err(
                    |_name_error| TypeAuthorityDiagnosticV2::InvalidCanonicalName {
                        name: diagnostic_name(&input.qualified_name),
                        location: raw_location,
                    },
                )?;
            if !canonical_segment(&input.lookup_name) {
                return Err(TypeAuthorityDiagnosticV2::InvalidCanonicalName {
                    name: input.lookup_name,
                    location: raw_location,
                });
            }
            if let Some(original) = by_name
                .get(&input.lookup_name)
                .map(|existing: &RelationSourceV2| existing.location.location())
            {
                return Err(TypeAuthorityDiagnosticV2::DuplicateRelationSource {
                    name: input.lookup_name,
                    primary: raw_location,
                    secondary: original,
                });
            }
            if let Some(original) = by_id.insert(id.clone(), input.location.clone()) {
                return Err(TypeAuthorityDiagnosticV2::DuplicateRelationSourceId {
                    id: format!("{}::{}", id.namespace.as_str(), id.qualified_name),
                    primary: raw_location,
                    secondary: original.location(),
                });
            }
            let relation_type = RelationTypeV1::try_new(input.row, input.cardinality)
                .map_err(TypeAuthorityDiagnosticV2::InvalidRelationType)?;
            let source = RelationSourceV2 {
                id,
                lookup_name: input.lookup_name.clone(),
                relation_type,
                provenance: input.provenance,
                location: input.location,
            };
            let _ = by_name.insert(input.lookup_name, source);
        }
        Ok(Self {
            by_name,
            source_registry_identity: registry_identity,
            budget_profile,
        })
    }
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&RelationSourceV2> {
        self.by_name.get(name)
    }
    pub fn iter(&self) -> impl Iterator<Item = (&String, &RelationSourceV2)> {
        self.by_name.iter()
    }
    #[must_use]
    pub fn source_registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.source_registry_identity
    }
    #[must_use]
    pub const fn budget_profile(&self) -> &AuthorityBudgetProfileV2 {
        &self.budget_profile
    }
}

fn canonical_segment(value: &str) -> bool {
    if value.len() > crate::MAX_TYPE_NAME_BYTES {
        return false;
    }
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-')
}

fn diagnostic_name(value: &str) -> String {
    let mut end = value.len().min(128);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}
