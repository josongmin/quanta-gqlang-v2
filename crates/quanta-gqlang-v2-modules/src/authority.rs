//! Session-owning public authority builders.

use std::collections::BTreeMap;
use std::sync::Arc;

use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, ExtendedSourceRegistryViewV2, SourceIdV2, SourceRegistryViewV2,
};

use crate::budget::ModuleBudgetSessionV2;
use crate::catalog::{
    DocumentCatalogAuthorityV2, DocumentCatalogFinalInputV2, DocumentCatalogInputV2,
    DocumentCatalogReferenceInputV2, DocumentCatalogReferenceOriginV2, ExternalCatalogAssetInputV2,
    ExternalCatalogPayloadV2, ExternalCatalogV2, PrecheckedExternalCatalogAssetV2,
    PredeclaredDocumentAssetV2, PredeclaredDocumentAssetsV2, ResolvedDocumentCatalogReferenceV2,
    ResolvedHostPredicateProofV2,
};
use crate::extension::{
    ComposedModelV2, ModelPackInputV2, ModelPackV2, ModelSchemaInputV2, ModelSchemaV2,
    PrecheckedModelPackInputV2,
};
use crate::graph::{
    is_catalog_local_name, CatalogAssetIdV2, CurrentModuleSelectionInputV2,
    ModuleDeclarationInputV2, ModuleGraphBuilderV2, ModuleImportInputV2, ModuleNodeInputV2,
    ModuleReexportInputV2, NamespaceKindV2, ResolvedGraphV2,
};
use crate::lockfile::{
    LockReferenceInputV2, LockReferenceV2, LockfileV2, ValidatedPackAuthorityV2,
};
use crate::manifest::{PackGraphV2, PackIdV2, PackManifestInputV2, PackManifestV2};
use crate::{ModuleBudgetV2, ModuleDiagnosticV2, VersionV2};
use quanta_gqlang_v2_types::{
    AuthorityBudgetProfileV2, HostCallableNamespaceV2, ResolvedHostPredicateKeyV2,
};

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
    predeclared: Option<PredeclaredDocumentAssetsV2>,
    phase: DocumentCatalogPhaseV2,
    session_key: Arc<()>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DocumentCatalogPhaseV2 {
    Empty,
    Predeclared,
    Finished,
    Failed,
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
            predeclared: None,
            phase: DocumentCatalogPhaseV2::Empty,
            session_key: Arc::new(()),
        })
    }

    /// Derive and charge every local document asset exactly once, before any
    /// body reference is resolved. The returned values cannot be fabricated by
    /// a caller and retain the checked combined-registry locations.
    pub fn predeclare_assets(
        &mut self,
        assets: Vec<crate::DocumentCatalogAssetInputV2>,
    ) -> Result<Vec<PredeclaredDocumentAssetV2>, ModuleDiagnosticV2> {
        if self.phase != DocumentCatalogPhaseV2::Empty {
            return Err(document_phase_error());
        }
        self.phase = DocumentCatalogPhaseV2::Failed;
        let predeclared = PredeclaredDocumentAssetsV2::try_new(
            assets,
            self.extended_registry.combined(),
            &mut self.session,
            self.session_key.clone(),
        )?;
        // A document declaration cannot reuse a sealed host identity. A later
        // resolver lookup keyed only by CatalogAssetId would otherwise merge
        // two distinct declaration origins and silently choose one payload.
        for asset in predeclared.iter() {
            self.session.charge_step()?;
            if asset.id().module_id() != self.parts.resolved_graph.current_module() {
                return Err(ModuleDiagnosticV2::new(
                    "modules.document_asset_wrong_module",
                    "document catalog asset must belong to the selected current module",
                    asset.location(),
                ));
            }
            let host_location = self
                .parts
                .external_catalog
                .asset(asset.id())
                .map(crate::ExternalCatalogAssetV2::location)
                .or_else(|| {
                    self.parts
                        .resolved_graph
                        .visible_bindings()
                        .get(asset.kind(), asset.id().local_name())
                        .filter(|binding| binding.resolved_asset_id() == asset.id())
                        .map(crate::ResolvedBindingV2::declaration_location)
                });
            if let Some(host_location) = host_location {
                let checked_host = self
                    .extended_registry
                    .combined()
                    .try_check_location(host_location)
                    .map_err(|_source_error| {
                        ModuleDiagnosticV2::without_location(
                            "modules.source_registry_mismatch",
                            "host catalog declaration location lost registry proof",
                        )
                    })?;
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_document_catalog_asset",
                    "document catalog asset duplicates a sealed host declaration",
                    asset.location(),
                    &checked_host,
                ));
            }
        }
        let output = predeclared.iter().collect();
        self.predeclared = Some(predeclared);
        self.phase = DocumentCatalogPhaseV2::Predeclared;
        Ok(output)
    }

    /// Resolve one exact catalog path against predeclared locals, the current
    /// module's sealed visible bindings, and the external catalog. Distinct
    /// same-spelling IDs reject before the expected kind is checked.
    pub fn resolve_reference(
        &mut self,
        input: DocumentCatalogReferenceInputV2,
    ) -> Result<ResolvedDocumentCatalogReferenceV2, ModuleDiagnosticV2> {
        if self.phase != DocumentCatalogPhaseV2::Predeclared {
            return Err(document_phase_error());
        }
        let location = input.location.clone();
        let result = self.resolve_reference_inner(input).and_then(|reference| {
            reference.ok_or_else(|| {
                ModuleDiagnosticV2::new(
                    "modules.unknown_catalog_reference",
                    "catalog reference is absent",
                    &location,
                )
            })
        });
        if result.is_err() {
            self.phase = DocumentCatalogPhaseV2::Failed;
        }
        result
    }

    /// Perform the same bounded typed lookup without terminalizing a truly
    /// absent catalog name. This supports source constructs that may also
    /// resolve against a separate graph selector authority. Wrong kind,
    /// ambiguity, registry mismatch and budget exhaustion remain hard errors.
    pub fn probe_reference(
        &mut self,
        input: DocumentCatalogReferenceInputV2,
    ) -> Result<Option<ResolvedDocumentCatalogReferenceV2>, ModuleDiagnosticV2> {
        if self.phase != DocumentCatalogPhaseV2::Predeclared {
            return Err(document_phase_error());
        }
        let result = self.resolve_reference_inner(input);
        if result.is_err() {
            self.phase = DocumentCatalogPhaseV2::Failed;
        }
        result
    }

    /// Resolve an authored host Predicate through the same catalog session and
    /// project its exact external typed signature into a types-owned callable.
    /// An imported alias retains the underlying CatalogAssetId; a visible
    /// declaration without matching external signature cannot certify a call.
    pub fn resolve_host_predicate_call(
        &mut self,
        input: DocumentCatalogReferenceInputV2,
    ) -> Result<ResolvedHostPredicateProofV2, ModuleDiagnosticV2> {
        if self.phase != DocumentCatalogPhaseV2::Predeclared {
            return Err(document_phase_error());
        }
        if input.expected_kind != NamespaceKindV2::Predicate {
            self.phase = DocumentCatalogPhaseV2::Failed;
            return Err(ModuleDiagnosticV2::new(
                "modules.catalog_reference_kind_mismatch",
                "host callable reference must request Predicate namespace",
                &input.location,
            ));
        }
        let reference = self.resolve_reference(input)?;
        let result = self.project_host_predicate(reference);
        if result.is_err() {
            self.phase = DocumentCatalogPhaseV2::Failed;
        }
        result
    }

    fn project_host_predicate(
        &mut self,
        reference: ResolvedDocumentCatalogReferenceV2,
    ) -> Result<ResolvedHostPredicateProofV2, ModuleDiagnosticV2> {
        self.session.charge_step()?;
        let Some(asset) = self.parts.external_catalog.asset(reference.id()) else {
            return Err(ModuleDiagnosticV2::new(
                "modules.missing_host_predicate_signature",
                "resolved Predicate lacks an exact external signature",
                reference.location(),
            ));
        };
        let ExternalCatalogPayloadV2::Predicate(signature) = asset.payload() else {
            return Err(ModuleDiagnosticV2::new(
                "modules.catalog_reference_kind_mismatch",
                "resolved host asset is not a Predicate signature",
                reference.location(),
            ));
        };
        if signature.callable_kind() != crate::CallableKindV2::Predicate {
            return Err(ModuleDiagnosticV2::new(
                "modules.catalog_payload_kind_mismatch",
                "host Predicate signature has another callable kind",
                reference.location(),
            ));
        }
        let module = reference.id().module_id().as_str();
        let local = reference.id().local_name();
        self.session.charge_string(module)?;
        self.session.charge_string(local)?;
        let encoded_len = ResolvedHostPredicateKeyV2::checked_canonical_len(
            module,
            HostCallableNamespaceV2::Predicate,
            local,
        )
        .map_err(|error| {
            ModuleDiagnosticV2::new(
                "modules.invalid_host_predicate_key",
                error.to_string(),
                reference.location(),
            )
        })?;
        self.session.charge_bytes(encoded_len)?;
        let key = ResolvedHostPredicateKeyV2::try_from_catalog_parts(
            module,
            HostCallableNamespaceV2::Predicate,
            local,
            encoded_len,
        )
        .map_err(|error| {
            ModuleDiagnosticV2::new(
                "modules.invalid_host_predicate_key",
                error.to_string(),
                reference.location(),
            )
        })?;
        let callable = signature.try_as_resolved_host_predicate(key, reference.location())?;
        Ok(ResolvedHostPredicateProofV2::new(
            reference,
            signature.clone(),
            callable,
        ))
    }

    /// Recheck a retained proof at semantic assembly against this exact
    /// validator session, checked call location and types-owned call target.
    pub fn validate_host_predicate_proof(
        &mut self,
        proof: &ResolvedHostPredicateProofV2,
        expected_key: &ResolvedHostPredicateKeyV2,
        expected_location: &CheckedLocatedSpanV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        if self.phase != DocumentCatalogPhaseV2::Predeclared {
            return Err(document_phase_error());
        }
        self.session.charge_step()?;
        if !proof.belongs_to(&self.session_key)
            || expected_location.registry_identity()
                != &self.extended_registry.combined().identity()
        {
            self.phase = DocumentCatalogPhaseV2::Failed;
            return Err(ModuleDiagnosticV2::new(
                "modules.source_registry_mismatch",
                "host predicate proof belongs to another document session",
                expected_location,
            ));
        }
        if proof.location() != expected_location || proof.key() != expected_key {
            self.phase = DocumentCatalogPhaseV2::Failed;
            return Err(ModuleDiagnosticV2::new(
                "modules.host_predicate_proof_mismatch",
                "host predicate call identity or location was transplanted",
                expected_location,
            ));
        }
        let exact = self
            .parts
            .external_catalog
            .asset(proof.id())
            .and_then(|asset| {
                if let ExternalCatalogPayloadV2::Predicate(signature) = asset.payload() {
                    Some(signature)
                } else {
                    None
                }
            });
        if exact != Some(proof.signature()) {
            self.phase = DocumentCatalogPhaseV2::Failed;
            return Err(ModuleDiagnosticV2::new(
                "modules.host_predicate_proof_mismatch",
                "host predicate proof no longer matches exact catalog signature",
                expected_location,
            ));
        }
        Ok(())
    }

    fn resolve_reference_inner(
        &mut self,
        input: DocumentCatalogReferenceInputV2,
    ) -> Result<Option<ResolvedDocumentCatalogReferenceV2>, ModuleDiagnosticV2> {
        let registry = self.extended_registry.combined();
        if input.location.registry_identity() != &registry.identity() {
            return Err(ModuleDiagnosticV2::new(
                "modules.source_registry_mismatch",
                "catalog reference uses another registry",
                &input.location,
            ));
        }
        self.session.charge_string(&input.local_name)?;
        self.session.charge_step()?;
        if !is_catalog_local_name(&input.local_name) {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_catalog_name",
                "invalid catalog reference path",
                &input.location,
            ));
        }
        let Some(predeclared) = &self.predeclared else {
            return Err(document_phase_error());
        };
        let current_module = self.parts.resolved_graph.current_module();
        let selected = input.module_id.as_ref().unwrap_or(current_module);
        let mut candidates: BTreeMap<
            CatalogAssetIdV2,
            (
                NamespaceKindV2,
                DocumentCatalogReferenceOriginV2,
                CheckedLocatedSpanV2,
                Vec<crate::LocatedEdgeV2>,
            ),
        > = BTreeMap::new();
        for asset in predeclared.iter() {
            self.session.charge_step()?;
            if asset.id().module_id() == selected && asset.id().local_name() == input.local_name {
                candidates.insert(
                    asset.id().clone(),
                    (
                        asset.kind(),
                        DocumentCatalogReferenceOriginV2::Local,
                        asset.location().clone(),
                        Vec::new(),
                    ),
                );
            }
        }
        if selected == current_module {
            for binding in self.parts.resolved_graph.visible_bindings().iter() {
                self.session.charge_step()?;
                if binding.local_name() == input.local_name {
                    let location = registry
                        .try_check_location(binding.declaration_location())
                        .map_err(|_source_error| {
                            ModuleDiagnosticV2::without_location(
                                "modules.source_registry_mismatch",
                                "visible declaration location lost registry proof",
                            )
                        })?;
                    candidates
                        .entry(binding.resolved_asset_id().clone())
                        .or_insert((
                            binding.namespace_kind(),
                            DocumentCatalogReferenceOriginV2::VisibleHost,
                            location,
                            binding.edge_witness().to_vec(),
                        ));
                }
            }
        }
        for asset in self.parts.external_catalog.iter() {
            self.session.charge_step()?;
            if asset.id().module_id() == selected && asset.id().local_name() == input.local_name {
                let location =
                    registry
                        .try_check_location(asset.location())
                        .map_err(|_source_error| {
                            ModuleDiagnosticV2::without_location(
                                "modules.source_registry_mismatch",
                                "external declaration location lost registry proof",
                            )
                        })?;
                match candidates.entry(asset.id().clone()) {
                    std::collections::btree_map::Entry::Occupied(existing) => {
                        if existing.get().2.location() != location.location() {
                            return Err(ModuleDiagnosticV2::conflict(
                                "modules.duplicate_host_catalog_asset",
                                "host catalog identity has distinct declaration origins",
                                &location,
                                &existing.get().2,
                            ));
                        }
                    }
                    std::collections::btree_map::Entry::Vacant(vacant) => {
                        vacant.insert((
                            asset.kind(),
                            DocumentCatalogReferenceOriginV2::External,
                            location,
                            Vec::new(),
                        ));
                    }
                }
            }
        }
        // A kind-directed lookup may not silently discard a different asset
        // with the same authored spelling. Distinct IDs are ambiguous even if
        // exactly one happens to have the requested kind.
        if candidates.len() > 1 {
            let detailed = candidates
                .iter()
                .map(|(id, (_, _, location, witness))| {
                    crate::AmbiguousCandidateV2::from_catalog_reference(
                        id.clone(),
                        location.location(),
                        witness.clone(),
                    )
                })
                .collect();
            return Err(ModuleDiagnosticV2::new(
                "modules.ambiguous_catalog_reference",
                "catalog name resolves to distinct assets",
                &input.location,
            )
            .with_ambiguous_candidates(detailed));
        }
        if let Some((id, (kind, origin, wrong, _))) = candidates.into_iter().next() {
            if kind == input.expected_kind {
                return Ok(Some(ResolvedDocumentCatalogReferenceV2::new(
                    id,
                    kind,
                    origin,
                    input.location,
                    self.session_key.clone(),
                )));
            }
            return Err(ModuleDiagnosticV2::conflict(
                "modules.catalog_reference_kind_mismatch",
                "catalog reference has wrong kind",
                &input.location,
                &wrong,
            ));
        }
        Ok(None)
    }

    /// Finish the predeclared document exactly once, without recharging or
    /// re-deriving its local asset IDs.
    pub fn validate_predeclared(
        &mut self,
        input: DocumentCatalogFinalInputV2,
    ) -> Result<DocumentCatalogAuthorityV2, ModuleDiagnosticV2> {
        if self.phase != DocumentCatalogPhaseV2::Predeclared {
            return Err(document_phase_error());
        }
        self.phase = DocumentCatalogPhaseV2::Failed;
        let Some(predeclared) = &self.predeclared else {
            return Err(document_phase_error());
        };
        let result = DocumentCatalogAuthorityV2::try_new_predeclared(
            input,
            predeclared,
            &self.parts.resolved_graph,
            &self.parts.external_catalog,
            &self.parts.pack_authority,
            self.extended_registry.combined(),
            &mut self.session,
            &self.session_key,
        )?;
        self.phase = DocumentCatalogPhaseV2::Finished;
        Ok(result)
    }

    pub fn validate(
        &mut self,
        input: DocumentCatalogInputV2,
    ) -> Result<DocumentCatalogAuthorityV2, ModuleDiagnosticV2> {
        if self.phase != DocumentCatalogPhaseV2::Empty {
            return Err(document_phase_error());
        }
        let _ = self.predeclare_assets(input.assets)?;
        self.validate_predeclared(DocumentCatalogFinalInputV2 {
            profile_admissions: input.profile_admissions,
            activations: input.activations,
            require_certified_directives: input.require_certified_directives,
            dependency_edges: input.dependency_edges,
            family_references: Vec::new(),
        })
    }
}

fn document_phase_error() -> ModuleDiagnosticV2 {
    ModuleDiagnosticV2::without_location(
        "modules.document_catalog_phase_error",
        "document catalog validator cannot repeat or skip a phase",
    )
}
