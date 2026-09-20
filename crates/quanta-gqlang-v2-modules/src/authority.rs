//! Session-owning public authority builders.

use std::collections::BTreeMap;

use quanta_gqlang_v2_hir::{ExtendedSourceRegistryViewV2, SourceIdV2, SourceRegistryViewV2};

use crate::budget::ModuleBudgetSessionV2;
use crate::catalog::{
    DocumentCatalogAuthorityV2, DocumentCatalogInputV2, ExternalCatalogAssetInputV2,
    ExternalCatalogV2, PrecheckedExternalCatalogAssetV2,
};
use crate::extension::{
    ComposedModelV2, ModelPackInputV2, ModelPackV2, ModelSchemaInputV2, ModelSchemaV2,
    PrecheckedModelPackInputV2,
};
use crate::graph::{
    CurrentModuleSelectionInputV2, ModuleDeclarationInputV2, ModuleGraphBuilderV2,
    ModuleImportInputV2, ModuleNodeInputV2, ModuleReexportInputV2, ResolvedGraphV2,
};
use crate::lockfile::{
    LockReferenceInputV2, LockReferenceV2, LockfileV2, ValidatedPackAuthorityV2,
};
use crate::manifest::{PackGraphV2, PackIdV2, PackManifestInputV2, PackManifestV2};
use crate::{ModuleBudgetV2, ModuleDiagnosticV2, VersionV2};
use quanta_gqlang_v2_types::AuthorityBudgetProfileV2;

#[derive(Clone, Debug)]
struct PendingLockV2 {
    reference: LockReferenceV2,
    source_id: SourceIdV2,
    text: String,
}

/// Sole public constructor for one closed host-side module authority.
pub struct ModuleAuthorityBuilderV2 {
    source_registry: SourceRegistryViewV2,
    budget_profile: AuthorityBudgetProfileV2,
    session: ModuleBudgetSessionV2,
    graph: ModuleGraphBuilderV2,
    external_inputs: Vec<PrecheckedExternalCatalogAssetV2>,
    external_locations:
        BTreeMap<crate::CatalogAssetIdV2, quanta_gqlang_v2_hir::CheckedLocatedSpanV2>,
    manifests: Vec<PackManifestV2>,
    lock: Option<PendingLockV2>,
    model_schema: Option<ModelSchemaV2>,
    model_schema_location: Option<quanta_gqlang_v2_hir::CheckedLocatedSpanV2>,
    model_pack_inputs: Vec<PrecheckedModelPackInputV2>,
    model_pack_locations: BTreeMap<PackIdV2, quanta_gqlang_v2_hir::CheckedLocatedSpanV2>,
}

impl ModuleAuthorityBuilderV2 {
    pub fn try_new(
        budget_profile: AuthorityBudgetProfileV2,
        source_registry: SourceRegistryViewV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        let mut graph = ModuleGraphBuilderV2::new();
        graph.bind_registry(source_registry.identity());
        let budget = ModuleBudgetV2::from_profile(budget_profile.clone());
        Ok(Self {
            source_registry,
            budget_profile,
            session: budget.begin_session_v2(),
            graph,
            external_inputs: Vec::new(),
            external_locations: BTreeMap::new(),
            manifests: Vec::new(),
            lock: None,
            model_schema: None,
            model_schema_location: None,
            model_pack_inputs: Vec::new(),
            model_pack_locations: BTreeMap::new(),
        })
    }

    pub fn try_add_module(&mut self, input: ModuleNodeInputV2) -> Result<(), ModuleDiagnosticV2> {
        self.graph.try_add_module(input, &mut self.session)
    }

    pub fn try_declare(
        &mut self,
        input: ModuleDeclarationInputV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        self.graph.try_declare(input, &mut self.session)
    }

    pub fn try_import(&mut self, input: ModuleImportInputV2) -> Result<(), ModuleDiagnosticV2> {
        self.graph.try_import(input, &mut self.session)
    }

    pub fn try_reexport(&mut self, input: ModuleReexportInputV2) -> Result<(), ModuleDiagnosticV2> {
        self.graph.try_reexport(input, &mut self.session)
    }

    pub fn try_add_manifest(
        &mut self,
        input: PackManifestInputV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        let manifest = PackManifestV2::try_new(input, &self.source_registry, &mut self.session)?;
        if let Some(original) = self
            .manifests
            .iter()
            .find(|existing| existing.coordinate().pack_id() == manifest.coordinate().pack_id())
        {
            return Err(ModuleDiagnosticV2::conflict(
                "modules.duplicate_pack",
                "duplicate pack name",
                manifest.location(),
                original.location(),
            ));
        }
        self.manifests.push(manifest);
        Ok(())
    }

    pub fn try_set_lockfile(
        &mut self,
        reference: LockReferenceInputV2,
        source_id: SourceIdV2,
        text: &str,
    ) -> Result<(), ModuleDiagnosticV2> {
        // Attempted replacements are charged before the duplicate is rejected.
        self.session.charge_string(&reference.path)?;
        self.session.charge_bytes(text.len())?;
        if reference.location.registry_identity() != &self.source_registry.identity() {
            return Err(ModuleDiagnosticV2::new(
                "modules.source_registry_mismatch",
                "lock reference belongs to another source registry",
                &reference.location,
            ));
        }
        let reference = LockReferenceV2::try_new_precharged(reference)?;
        if let Some(original) = &self.lock {
            return Err(ModuleDiagnosticV2::conflict(
                "modules.duplicate_lockfile_input",
                "lockfile was already supplied",
                reference.location(),
                original.reference.location(),
            ));
        }
        self.lock = Some(PendingLockV2 {
            reference,
            source_id,
            text: text.to_owned(),
        });
        Ok(())
    }

    pub fn try_add_external(
        &mut self,
        input: ExternalCatalogAssetInputV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        let input = PrecheckedExternalCatalogAssetV2::try_admit(
            input,
            &self.source_registry,
            &mut self.session,
        )?;
        if let Some(original) = self.external_locations.get(input.id()) {
            return Err(ModuleDiagnosticV2::conflict(
                "modules.duplicate_external_catalog_asset",
                "duplicate external catalog asset",
                input.location(),
                original,
            ));
        }
        self.external_locations
            .insert(input.id().clone(), input.location().clone());
        self.external_inputs.push(input);
        Ok(())
    }

    pub fn try_set_model_schema(
        &mut self,
        input: ModelSchemaInputV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        if let Some(original) = &self.model_schema_location {
            ModelSchemaV2::validate_and_charge_input(
                &input,
                &self.source_registry,
                &mut self.session,
            )?;
            return Err(ModuleDiagnosticV2::conflict(
                "modules.duplicate_model_schema",
                "model schema was already supplied",
                &input.location,
                original,
            ));
        }
        let location = input.location.clone();
        self.model_schema = Some(ModelSchemaV2::try_new(
            input,
            &self.source_registry,
            &mut self.session,
        )?);
        self.model_schema_location = Some(location);
        Ok(())
    }

    pub fn try_add_model_pack(
        &mut self,
        input: ModelPackInputV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        let input =
            PrecheckedModelPackInputV2::try_admit(input, &self.source_registry, &mut self.session)?;
        if let Some(original) = self.model_pack_locations.get(input.pack_id()) {
            return Err(ModuleDiagnosticV2::conflict(
                "modules.duplicate_model_pack",
                "duplicate model pack input",
                input.location(),
                original,
            ));
        }
        self.model_pack_locations
            .insert(input.pack_id().clone(), input.location().clone());
        self.model_pack_inputs.push(input);
        Ok(())
    }

    pub fn finish(
        mut self,
        current_module: CurrentModuleSelectionInputV2,
        language_version: VersionV2,
    ) -> Result<ValidatedModuleAuthorityPartsV2, ModuleDiagnosticV2> {
        let resolved_graph = self.graph.resolve(current_module, &mut self.session)?;
        let external_catalog = ExternalCatalogV2::try_new(
            self.external_inputs,
            &self.source_registry,
            &mut self.session,
        )?;
        let pack_graph = if self.manifests.is_empty() {
            PackGraphV2::empty(&self.source_registry)
        } else {
            PackGraphV2::try_new(self.manifests, &mut self.session)?
        };
        let validated_graph = pack_graph.validate(language_version, &mut self.session)?;
        let verified_lock = if validated_graph.is_empty() {
            if let Some(lock) = self.lock {
                return Err(ModuleDiagnosticV2::new(
                    "modules.unexpected_lockfile",
                    "an empty pack graph must not submit a lockfile",
                    lock.reference.location(),
                ));
            }
            LockfileV2::empty(SourceIdV2::from_raw(0), &self.source_registry)
                .verify_against(&validated_graph, &mut self.session)?
        } else {
            let lock = self.lock.ok_or_else(|| {
                ModuleDiagnosticV2::without_location(
                    "modules.missing_lockfile",
                    "a nonempty pack graph requires exactly one lockfile",
                )
            })?;
            LockfileV2::parse(
                lock.source_id,
                &lock.text,
                &self.source_registry,
                &mut self.session,
            )?
            .with_reference(lock.reference)?
            .verify_against(&validated_graph, &mut self.session)?
        };
        let pack_authority = ValidatedPackAuthorityV2::try_new(validated_graph, verified_lock)?;
        let composed_model = match (self.model_schema, self.model_pack_inputs.is_empty()) {
            (None, true) => None,
            (None, false) => {
                let Some(first) = self.model_pack_inputs.first() else {
                    return Err(ModuleDiagnosticV2::without_location(
                        "modules.invalid_model_data",
                        "model packs require a model schema",
                    ));
                };
                return Err(ModuleDiagnosticV2::new(
                    "modules.invalid_model_data",
                    "model packs require a model schema",
                    first.location(),
                ));
            }
            (Some(schema), _) => {
                let mut packs = Vec::new();
                for input in self.model_pack_inputs {
                    packs.push(ModelPackV2::try_new_prechecked(
                        input,
                        &schema,
                        &pack_authority,
                        &self.source_registry,
                        &mut self.session,
                    )?);
                }
                Some(ComposedModelV2::compose(
                    packs,
                    self.source_registry.identity(),
                )?)
            }
        };
        Ok(ValidatedModuleAuthorityPartsV2 {
            resolved_graph,
            external_catalog,
            pack_authority,
            composed_model,
            source_registry: self.source_registry,
            budget_profile: self.budget_profile,
        })
    }
}

/// Sealed, registry-consistent output of one authority transaction.
#[derive(Debug)]
pub struct ValidatedModuleAuthorityPartsV2 {
    resolved_graph: ResolvedGraphV2,
    external_catalog: ExternalCatalogV2,
    pack_authority: ValidatedPackAuthorityV2,
    composed_model: Option<ComposedModelV2>,
    source_registry: SourceRegistryViewV2,
    budget_profile: AuthorityBudgetProfileV2,
}
impl ValidatedModuleAuthorityPartsV2 {
    #[must_use]
    pub const fn resolved_graph(&self) -> &ResolvedGraphV2 {
        &self.resolved_graph
    }
    #[must_use]
    pub const fn external_catalog(&self) -> &ExternalCatalogV2 {
        &self.external_catalog
    }
    #[must_use]
    pub const fn pack_authority(&self) -> &ValidatedPackAuthorityV2 {
        &self.pack_authority
    }
    #[must_use]
    pub const fn composed_model(&self) -> Option<&ComposedModelV2> {
        self.composed_model.as_ref()
    }
    #[must_use]
    pub const fn source_registry(&self) -> &SourceRegistryViewV2 {
        &self.source_registry
    }
    #[must_use]
    pub const fn budget_profile(&self) -> &AuthorityBudgetProfileV2 {
        &self.budget_profile
    }
}

/// Per-document validator with one non-resettable budget session.
pub struct DocumentCatalogValidatorV2<'a> {
    extended_registry: ExtendedSourceRegistryViewV2,
    parts: &'a ValidatedModuleAuthorityPartsV2,
    session: ModuleBudgetSessionV2,
}
impl<'a> DocumentCatalogValidatorV2<'a> {
    pub fn try_new(
        extended_registry: ExtendedSourceRegistryViewV2,
        parts: &'a ValidatedModuleAuthorityPartsV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if extended_registry.parent_identity() != &parts.source_registry.identity() {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.source_registry_mismatch",
                "document registry is not an extension of sealed host authority",
            ));
        }
        Ok(Self {
            extended_registry,
            parts,
            session: ModuleBudgetV2::from_profile(parts.budget_profile.clone()).begin_session_v2(),
        })
    }

    pub fn validate(
        &mut self,
        input: DocumentCatalogInputV2,
    ) -> Result<DocumentCatalogAuthorityV2, ModuleDiagnosticV2> {
        DocumentCatalogAuthorityV2::try_new(
            input,
            &self.parts.external_catalog,
            &self.parts.pack_authority,
            self.extended_registry.combined(),
            &mut self.session,
        )
    }
}
