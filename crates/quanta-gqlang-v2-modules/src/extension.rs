//! Checked, declarative model schema and model-pack composition.

use std::collections::BTreeMap;

use crate::budget::ModuleBudgetSessionV2;
use crate::graph::{excerpt, is_qualified_name, is_segment, loc_key};
use crate::lockfile::ValidatedPackAuthorityV2;
use crate::manifest::PackIdV2;
use crate::version::VersionV2;
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, LocatedSpanV2, SourceRegistryIdentityV2, SourceRegistryViewV2,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedModelValueInputV2 {
    pub value: String,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelAttributeInputV2 {
    pub attribute: String,
    pub allowed_values: Vec<LocatedModelValueInputV2>,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelSchemaInputV2 {
    pub schema_version: VersionV2,
    pub attributes: Vec<ModelAttributeInputV2>,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelFactInputV2 {
    pub target: String,
    pub attribute: String,
    pub value: String,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPackInputV2 {
    pub pack_id: PackIdV2,
    pub model_version: VersionV2,
    pub facts: Vec<ModelFactInputV2>,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug)]
pub(crate) struct PrecheckedModelPackInputV2 {
    input: ModelPackInputV2,
    registry_identity: SourceRegistryIdentityV2,
}

impl PrecheckedModelPackInputV2 {
    pub(crate) fn try_admit(
        input: ModelPackInputV2,
        registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        session.charge_node()?;
        session.charge_string(input.pack_id.as_str())?;
        for fact in &input.facts {
            session.charge_edge()?;
            session.charge_string(&fact.target)?;
            session.charge_string(&fact.attribute)?;
            session.charge_string(&fact.value)?;
        }
        check_location(&input.location, registry)?;
        for fact in &input.facts {
            check_location(&fact.location, registry)?;
        }
        Ok(Self {
            input,
            registry_identity: registry.identity(),
        })
    }

    pub(crate) const fn pack_id(&self) -> &PackIdV2 {
        &self.input.pack_id
    }

    pub(crate) const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.input.location
    }
}

#[derive(Clone, Debug)]
pub struct ModelSchemaV2 {
    version: VersionV2,
    allowed: BTreeMap<String, BTreeMap<String, CheckedLocatedSpanV2>>,
    location: CheckedLocatedSpanV2,
    registry_identity: SourceRegistryIdentityV2,
}
impl ModelSchemaV2 {
    pub(crate) fn validate_and_charge_input(
        input: &ModelSchemaInputV2,
        registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        session.charge_node()?;
        check_location(&input.location, registry)?;
        for attribute in &input.attributes {
            session.charge_node()?;
            session.charge_string(&attribute.attribute)?;
            check_location(&attribute.location, registry)?;
            for value in &attribute.allowed_values {
                session.charge_edge()?;
                session.charge_string(&value.value)?;
                check_location(&value.location, registry)?;
            }
        }
        Ok(())
    }

    pub(crate) fn try_new(
        input: ModelSchemaInputV2,
        registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        Self::validate_and_charge_input(&input, registry, session)?;
        let mut allowed: BTreeMap<String, BTreeMap<String, CheckedLocatedSpanV2>> = BTreeMap::new();
        let mut attr_locations: BTreeMap<String, CheckedLocatedSpanV2> = BTreeMap::new();
        for attribute in input.attributes {
            if !is_segment(&attribute.attribute) {
                return Err(ModuleDiagnosticV2::new(
                    "modules.invalid_model_data",
                    "invalid model attribute",
                    &attribute.location,
                ));
            }
            if let Some(original) = attr_locations.get(&attribute.attribute) {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_model_attribute",
                    "duplicate model attribute",
                    &attribute.location,
                    original,
                ));
            }
            let mut values: BTreeMap<String, CheckedLocatedSpanV2> = BTreeMap::new();
            for value in attribute.allowed_values {
                if value.value.is_empty() {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.invalid_model_data",
                        "empty model value",
                        &value.location,
                    ));
                }
                match values.get_mut(&value.value) {
                    Some(existing)
                        if loc_key(value.location.location()) < loc_key(existing.location()) =>
                    {
                        *existing = value.location
                    }
                    Some(_) => {}
                    None => {
                        values.insert(value.value, value.location);
                    }
                }
            }
            attr_locations.insert(attribute.attribute.clone(), attribute.location);
            allowed.insert(attribute.attribute, values);
        }
        Ok(Self {
            version: input.schema_version,
            allowed,
            location: input.location,
            registry_identity: registry.identity(),
        })
    }
    #[must_use]
    pub const fn version(&self) -> VersionV2 {
        self.version
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    #[must_use]
    pub const fn source_registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.registry_identity
    }
}

#[derive(Clone, Debug)]
struct ModelFactV2 {
    target: String,
    attribute: String,
    value: String,
    location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug)]
pub(crate) struct ModelPackV2 {
    pack_id: PackIdV2,
    facts: Vec<ModelFactV2>,
}
impl ModelPackV2 {
    pub(crate) fn try_new_prechecked(
        input: PrecheckedModelPackInputV2,
        schema: &ModelSchemaV2,
        packs: &ValidatedPackAuthorityV2,
        registry: &SourceRegistryViewV2,
        _session: &mut ModuleBudgetSessionV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if input.registry_identity != registry.identity() {
            return Err(ModuleDiagnosticV2::new(
                "modules.source_registry_mismatch",
                "prechecked model pack belongs to another source registry",
                &input.input.location,
            ));
        }
        let input = input.input;
        if input.model_version != schema.version {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_model_data",
                "model pack version differs from schema version",
                &input.location,
            ));
        }
        if packs.manifest(&input.pack_id).is_none() {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_model_data",
                "model pack id is absent from validated pack authority",
                &input.location,
            ));
        }
        let mut facts: BTreeMap<(String, String, String), ModelFactV2> = BTreeMap::new();
        for fact in input.facts {
            if !is_qualified_name(&fact.target)
                || !is_segment(&fact.attribute)
                || fact.value.is_empty()
            {
                return Err(ModuleDiagnosticV2::new(
                    "modules.invalid_model_data",
                    "invalid model fact spelling",
                    &fact.location,
                ));
            }
            match schema.allowed.get(&fact.attribute) {
                Some(values) if values.contains_key(&fact.value) => {}
                _ => {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.invalid_model_data",
                        "model fact is outside schema",
                        &fact.location,
                    ))
                }
            }
            let key = (
                fact.target.clone(),
                fact.attribute.clone(),
                fact.value.clone(),
            );
            match facts.get_mut(&key) {
                Some(existing)
                    if loc_key(fact.location.location())
                        < loc_key(existing.location.location()) =>
                {
                    existing.location = fact.location
                }
                Some(_) => {}
                None => {
                    facts.insert(
                        key,
                        ModelFactV2 {
                            target: fact.target,
                            attribute: fact.attribute,
                            value: fact.value,
                            location: fact.location,
                        },
                    );
                }
            }
        }
        Ok(Self {
            pack_id: input.pack_id,
            facts: facts.into_values().collect(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ComposedModelFactV2 {
    target: String,
    attribute: String,
    value: String,
    asserting_packs: Vec<PackIdV2>,
    location: CheckedLocatedSpanV2,
}
impl ComposedModelFactV2 {
    #[must_use]
    pub fn target(&self) -> &str {
        &self.target
    }
    #[must_use]
    pub fn attribute(&self) -> &str {
        &self.attribute
    }
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
    #[must_use]
    pub fn asserting_packs(&self) -> &[PackIdV2] {
        &self.asserting_packs
    }
    #[must_use]
    pub const fn location(&self) -> LocatedSpanV2 {
        self.location.location()
    }
}
#[derive(Clone, Debug)]
pub struct ComposedModelV2 {
    facts: Vec<ComposedModelFactV2>,
    registry_identity: SourceRegistryIdentityV2,
}
impl ComposedModelV2 {
    pub(crate) fn compose(
        mut packs: Vec<ModelPackV2>,
        registry_identity: SourceRegistryIdentityV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        packs.sort_by(|a, b| a.pack_id.cmp(&b.pack_id));
        let mut merged: BTreeMap<(String, String), ComposedModelFactV2> = BTreeMap::new();
        for pack in packs {
            for fact in pack.facts {
                let key = (fact.target.clone(), fact.attribute.clone());
                match merged.get_mut(&key) {
                    Some(existing) if existing.value == fact.value => {
                        if !existing.asserting_packs.contains(&pack.pack_id) {
                            existing.asserting_packs.push(pack.pack_id.clone());
                            existing.asserting_packs.sort();
                        }
                        if loc_key(fact.location.location()) < loc_key(existing.location.location())
                        {
                            existing.location = fact.location;
                        }
                    }
                    Some(existing) => {
                        let existing_pack = existing
                            .asserting_packs
                            .first()
                            .map_or("<unknown>", PackIdV2::as_str);
                        return Err(ModuleDiagnosticV2::conflict(
                            "modules.model_conflict",
                            format!(
                                "model conflict for {}::{} between packs {} and {}",
                                excerpt(&fact.target),
                                excerpt(&fact.attribute),
                                excerpt(existing_pack),
                                excerpt(pack.pack_id.as_str())
                            ),
                            &fact.location,
                            &existing.location,
                        ));
                    }
                    None => {
                        merged.insert(
                            key,
                            ComposedModelFactV2 {
                                target: fact.target,
                                attribute: fact.attribute,
                                value: fact.value,
                                asserting_packs: vec![pack.pack_id.clone()],
                                location: fact.location,
                            },
                        );
                    }
                }
            }
        }
        Ok(Self {
            facts: merged.into_values().collect(),
            registry_identity,
        })
    }
    pub fn facts(&self) -> impl Iterator<Item = &ComposedModelFactV2> {
        self.facts.iter()
    }
    #[must_use]
    pub const fn source_registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.registry_identity
    }
}

fn check_location(
    location: &CheckedLocatedSpanV2,
    registry: &SourceRegistryViewV2,
) -> Result<(), ModuleDiagnosticV2> {
    if location.registry_identity() != &registry.identity() {
        Err(ModuleDiagnosticV2::new(
            "modules.source_registry_mismatch",
            "model location belongs to another source registry",
            location,
        ))
    } else {
        Ok(())
    }
}
