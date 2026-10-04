//! Closed and open semantic authority plus the canonical frontend budget.

mod budget;
mod inputs;
mod open_analysis;

pub use budget::{DownstreamBudgetReceiptV2, FrontendBudgetV2};
pub use inputs::{
    FrontendAuthorityLocationV2, FrontendAuthoritySourceV2, FrontendCallableParameterInputV2,
    FrontendCatalogAssetInputV2, FrontendCatalogPayloadV2, FrontendGraphReferenceInputV2,
    FrontendLanguageVersionV2, FrontendLockfileInputV2, FrontendPackInputV2,
    FrontendRelationAuthorityV2, FrontendRelationCardinalityV2, FrontendRelationFieldInputV2,
    FrontendRelationInputV2, FrontendScalarTypeV2, FrontendSymbolTargetInputV3,
};
pub use open_analysis::{OpenAnalysisAuthorityInputV2, OpenAnalysisAuthorityV2};

use crate::diagnostic::{
    bounded_identifier, FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2,
    FrontendStageV2,
};
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, ExtendedSourceRegistryViewV2, LocatedSpanV2, SourceIdV2,
    SourceRegistrationV2, SourceRegistryIdentityV2, SourceRegistryViewV2,
};
use quanta_gqlang_v2_modules::{
    ContentDigestV2, CurrentModuleSelectionInputV2, ExternalCatalogAssetInputV2,
    LockReferenceInputV2, ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2, PackIdV2,
    PackManifestInputV2, ValidatedModuleAuthorityPartsV2, VersionRangeV2, VersionV2,
};
use quanta_gqlang_v2_types::{
    AuthorityBudgetProfileV2, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1,
    RelationCatalogV2, RelationSourceInputV2, ValueTypeV1,
};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Source-independent authority construction error. It never fabricates a
/// document ID or a byte span for host configuration failure.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontendAuthorityErrorV2 {
    code: &'static str,
    message: String,
    primary: Option<LocatedSpanV2>,
    secondary: Option<LocatedSpanV2>,
}

impl FrontendAuthorityErrorV2 {
    /// Stable machine-readable code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        self.code
    }
    /// Bounded explanation.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
    /// Checked host source location, if this authority error has one.
    #[must_use]
    pub const fn primary(&self) -> Option<LocatedSpanV2> {
        self.primary
    }
    /// Conflicting declaration location, if present.
    #[must_use]
    pub const fn secondary(&self) -> Option<LocatedSpanV2> {
        self.secondary
    }

    pub(crate) fn from_external(code: &'static str, message: impl AsRef<str>) -> Self {
        configuration_failure(code, message)
    }
}

/// Closed kind vocabulary for host-admitted graph selectors. These selectors
/// are opaque frontend identities, not modules assets or provider handles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GraphReferenceKindV2 {
    /// Graph entity/endpoint/waypoint.
    Entity,
    /// Callable endpoint.
    Callable,
    /// Analysis scope.
    Scope,
    /// Window selector.
    Window,
    /// Frontier selector.
    Frontier,
    /// Sanitizer selector.
    Sanitizer,
    /// CHECK pattern selector.
    Pattern,
    /// Comparison baseline/candidate snapshot.
    Snapshot,
    /// EXPORT source selector.
    ExportSource,
}

/// Deterministic identity in one sealed host graph-reference authority.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GraphReferenceIdV2 {
    kind: GraphReferenceKindV2,
    spelling: String,
    entity_subtype: Option<String>,
    location: CheckedLocatedSpanV2,
    catalog_identity: Arc<GraphReferenceCatalogIdentityV2>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct GraphReferenceCatalogIdentityV2 {
    registry: SourceRegistryIdentityV2,
    budget: AuthorityBudgetProfileV2,
    entries: Vec<(
        String,
        GraphReferenceKindV2,
        Option<String>,
        CheckedLocatedSpanV2,
    )>,
}

impl GraphReferenceIdV2 {
    /// Checked selector kind.
    #[must_use]
    pub const fn kind(&self) -> GraphReferenceKindV2 {
        self.kind
    }
    /// Exact host-admitted spelling; punctuation is never split/rewritten.
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.spelling
    }
    /// Host-admitted subtype for DESCRIBE Entity selectors.
    #[must_use]
    pub fn entity_subtype(&self) -> Option<&str> {
        self.entity_subtype.as_deref()
    }
    /// Exact checked host declaration that admitted this selector.
    #[must_use]
    pub const fn declaration_location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

/// One host supplied graph selector and its checked source location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphReferenceInputV2 {
    /// Exact name as matched in source.
    pub spelling: String,
    /// Semantic selector kind.
    pub kind: GraphReferenceKindV2,
    /// Exact entity subtype for DESCRIBE, absent for other selector kinds.
    pub entity_subtype: Option<String>,
    /// Host-registry-checked declaration location.
    pub location: CheckedLocatedSpanV2,
}

/// Immutable graph selector authority. No runtime/provider lookup occurs.
#[derive(Clone, Debug)]
pub struct GraphReferenceCatalogV2 {
    by_spelling: BTreeMap<String, (GraphReferenceIdV2, CheckedLocatedSpanV2)>,
    identity: Arc<GraphReferenceCatalogIdentityV2>,
    registry_identity: SourceRegistryIdentityV2,
    budget_profile: AuthorityBudgetProfileV2,
}

impl GraphReferenceCatalogV2 {
    /// Admit every selector in one non-resettable authority-byte session.
    pub fn try_new(
        inputs: Vec<GraphReferenceInputV2>,
        host_sources: &SourceRegistryViewV2,
        budget_profile: AuthorityBudgetProfileV2,
    ) -> Result<Self, FrontendAuthorityErrorV2> {
        let identity = host_sources.identity();
        let mut admitted: BTreeMap<
            String,
            (GraphReferenceKindV2, Option<String>, CheckedLocatedSpanV2),
        > = BTreeMap::new();
        let mut bytes = 0usize;
        for input in inputs {
            if input.location.registry_identity() != &identity {
                return Err(authority_failure_at(
                    "sema.source_registry_mismatch",
                    "graph selector location belongs to another host registry",
                    input.location.location(),
                    None,
                ));
            }
            if input.spelling.is_empty() {
                return Err(authority_failure_at(
                    "sema.invalid_graph_reference",
                    "graph selector spelling cannot be empty",
                    input.location.location(),
                    None,
                ));
            }
            if input.entity_subtype.as_deref().is_some_and(str::is_empty)
                || (input.entity_subtype.is_some() && input.kind != GraphReferenceKindV2::Entity)
            {
                return Err(authority_failure_at(
                    "sema.invalid_graph_reference",
                    "entity subtype must be nonempty and only an Entity may carry it",
                    input.location.location(),
                    None,
                ));
            }
            if input.spelling.len() > budget_profile.authority_string_bytes() {
                return Err(authority_failure_at(
                    "frontend.resource_limit_exceeded",
                    "graph selector exceeds authority_string_bytes",
                    input.location.location(),
                    None,
                ));
            }
            bytes = bytes.saturating_add(input.spelling.len());
            if let Some(subtype) = &input.entity_subtype {
                if subtype.len() > budget_profile.authority_string_bytes() {
                    return Err(authority_failure_at(
                        "frontend.resource_limit_exceeded",
                        "entity subtype exceeds authority_string_bytes",
                        input.location.location(),
                        None,
                    ));
                }
                bytes = bytes.saturating_add(subtype.len());
            }
            if bytes > budget_profile.authority_bytes() {
                return Err(authority_failure_at(
                    "frontend.resource_limit_exceeded",
                    "graph selectors exceed authority_bytes",
                    input.location.location(),
                    None,
                ));
            }
            if let Some((_, _, previous)) = admitted.get(&input.spelling) {
                return Err(authority_failure_at(
                    "sema.ambiguous_reference",
                    "graph selector spelling is duplicated",
                    input.location.location(),
                    Some(previous.location()),
                ));
            }
            admitted.insert(
                input.spelling,
                (input.kind, input.entity_subtype, input.location),
            );
        }
        let catalog_identity = Arc::new(GraphReferenceCatalogIdentityV2 {
            registry: identity.clone(),
            budget: budget_profile.clone(),
            entries: admitted
                .iter()
                .map(|(name, (kind, subtype, location))| {
                    (name.clone(), *kind, subtype.clone(), location.clone())
                })
                .collect(),
        });
        let by_spelling = admitted
            .into_iter()
            .map(|(spelling, (kind, entity_subtype, location))| {
                let id = GraphReferenceIdV2 {
                    kind,
                    spelling: spelling.clone(),
                    entity_subtype,
                    location: location.clone(),
                    catalog_identity: Arc::clone(&catalog_identity),
                };
                (spelling, (id, location))
            })
            .collect();
        Ok(Self {
            by_spelling,
            identity: catalog_identity,
            registry_identity: identity,
            budget_profile,
        })
    }

    /// Resolve exact spelling with an expected semantic kind. No punctuation
    /// splitting, prefix search, or fallback namespace is allowed.
    pub fn resolve(
        &self,
        spelling: &str,
        expected: GraphReferenceKindV2,
        location: LocatedSpanV2,
    ) -> Result<GraphReferenceIdV2, FrontendFailureV2> {
        self.probe(spelling, expected, location)?.ok_or_else(|| {
            FrontendFailureV2::one(FrontendDiagnosticV2::new(
                FrontendStageV2::Resolution,
                "sema.unresolved_reference",
                FrontendSeverityV2::Error,
                location,
                Vec::new(),
                format!("unknown graph selector `{}`", bounded_identifier(spelling)),
            ))
        })
    }

    /// Nonterminal exact candidate probe for a source form whose namespace is
    /// intentionally ambiguous until catalog and graph authorities are both
    /// queried (EXPLAIN). Wrong-kind is still a hard source diagnostic.
    pub fn probe(
        &self,
        spelling: &str,
        expected: GraphReferenceKindV2,
        location: LocatedSpanV2,
    ) -> Result<Option<GraphReferenceIdV2>, FrontendFailureV2> {
        match self.by_spelling.get(spelling) {
            Some((id, _)) if id.kind == expected => Ok(Some(id.clone())),
            Some((_, declaration)) => Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
                FrontendStageV2::Resolution,
                "sema.wrong_reference_kind",
                FrontendSeverityV2::Error,
                location,
                vec![declaration.location()],
                "graph selector has the wrong semantic kind",
            ))),
            None => Ok(None),
        }
    }

    /// Resolve a DESCRIBE path against an exact host-admitted entity subtype.
    pub fn resolve_entity_subtype(
        &self,
        spelling: &str,
        subtype: &str,
        location: LocatedSpanV2,
    ) -> Result<GraphReferenceIdV2, FrontendFailureV2> {
        let id = self.resolve(spelling, GraphReferenceKindV2::Entity, location)?;
        if id.entity_subtype() != Some(subtype) {
            return Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
                FrontendStageV2::Resolution,
                "sema.wrong_reference_kind",
                FrontendSeverityV2::Error,
                location,
                Vec::new(),
                "DESCRIBE kind differs from the host-admitted entity subtype",
            )));
        }
        Ok(id)
    }

    /// Exact host registry identity.
    #[must_use]
    pub const fn source_registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.registry_identity
    }
    /// Exact budget profile used at admission.
    #[must_use]
    pub const fn budget_profile(&self) -> &AuthorityBudgetProfileV2 {
        &self.budget_profile
    }
    /// Checked selector count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_spelling.len()
    }
    /// Whether no selectors were admitted by this explicit host proof.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_spelling.is_empty()
    }
    /// Verify an opaque selector was minted by this exact admitted catalog,
    /// including all records, host registry and budget profile.
    #[must_use]
    pub fn admits(&self, id: &GraphReferenceIdV2) -> bool {
        self.identity == id.catalog_identity
            && self
                .by_spelling
                .get(id.spelling())
                .is_some_and(|(owned, _)| owned == id)
    }
}

/// Complete immutable authority required for closed semantic compilation.
#[derive(Debug)]
pub struct ClosedFrontendAuthorityV2 {
    modules: ValidatedModuleAuthorityPartsV2,
    relations: RelationCatalogV2,
    graph_references: GraphReferenceCatalogV2,
    symbol_targets: BTreeMap<Arc<str>, ResolvedSymbolTargetV3>,
    budget: FrontendBudgetV2,
}

/// A host-issued, exact symbol target. Source spelling only selects this sealed
/// record; runtime must still revalidate it against the admitted snapshot.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSymbolTargetV3 {
    selector: Arc<str>,
    repo_path: Arc<str>,
    symbol_id: Arc<str>,
    start_byte: u64,
    end_byte: u64,
    location: CheckedLocatedSpanV2,
}

impl ResolvedSymbolTargetV3 {
    #[must_use]
    pub fn selector(&self) -> &str {
        &self.selector
    }
    #[must_use]
    pub fn repo_path(&self) -> &str {
        &self.repo_path
    }
    #[must_use]
    pub fn symbol_id(&self) -> &str {
        &self.symbol_id
    }
    /// Shared selector storage for cross-crate immutable projections.
    #[doc(hidden)]
    #[must_use]
    pub fn clone_selector_storage_v3(&self) -> Arc<str> {
        Arc::clone(&self.selector)
    }
    /// Shared repository path storage for cross-crate immutable projections.
    #[doc(hidden)]
    #[must_use]
    pub fn clone_repo_path_storage_v3(&self) -> Arc<str> {
        Arc::clone(&self.repo_path)
    }
    /// Shared symbol ID storage for cross-crate immutable projections.
    #[doc(hidden)]
    #[must_use]
    pub fn clone_symbol_id_storage_v3(&self) -> Arc<str> {
        Arc::clone(&self.symbol_id)
    }
    #[must_use]
    pub const fn start_byte(&self) -> u64 {
        self.start_byte
    }
    #[must_use]
    pub const fn end_byte(&self) -> u64 {
        self.end_byte
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

/// Curated external transaction for constructing closed frontend authority.
///
/// It accepts only host-agnostic declarations. Owner proofs and checked
/// locations are minted internally in one registry/budget session and cannot
/// be supplied or forged by an external caller.
pub struct ClosedFrontendAuthorityBuilderV2 {
    budget: FrontendBudgetV2,
    sources: Vec<FrontendAuthoritySourceV2>,
    current_module: String,
    current_module_location: FrontendAuthorityLocationV2,
    language_version: FrontendLanguageVersionV2,
    packs: Vec<FrontendPackInputV2>,
    lockfiles: Vec<FrontendLockfileInputV2>,
    relations: Vec<FrontendRelationInputV2>,
    graph_references: Vec<FrontendGraphReferenceInputV2>,
    symbol_targets: Vec<FrontendSymbolTargetInputV3>,
    catalog_assets: Vec<FrontendCatalogAssetInputV2>,
}

impl ClosedFrontendAuthorityBuilderV2 {
    /// Start an explicit authority transaction. No empty/default source or
    /// module identity is invented.
    pub fn try_new(
        budget: FrontendBudgetV2,
        sources: Vec<FrontendAuthoritySourceV2>,
        current_module: String,
        current_module_location: FrontendAuthorityLocationV2,
        language_version: FrontendLanguageVersionV2,
    ) -> Result<Self, FrontendAuthorityErrorV2> {
        if sources.is_empty() {
            return Err(configuration_failure(
                "sema.empty_authority_sources",
                "closed authority requires at least one registered host source",
            ));
        }
        if current_module.is_empty() {
            return Err(configuration_failure(
                "sema.invalid_current_module",
                "current module cannot be empty",
            ));
        }
        Ok(Self {
            budget,
            sources,
            current_module,
            current_module_location,
            language_version,
            packs: Vec::new(),
            lockfiles: Vec::new(),
            relations: Vec::new(),
            graph_references: Vec::new(),
            symbol_targets: Vec::new(),
            catalog_assets: Vec::new(),
        })
    }

    /// Add one relation declaration. Validation remains atomic until finish.
    pub fn add_relation(&mut self, relation: FrontendRelationInputV2) {
        self.relations.push(relation);
    }

    /// Add one graph selector declaration. Validation remains atomic until finish.
    pub fn add_graph_reference(&mut self, reference: FrontendGraphReferenceInputV2) {
        self.graph_references.push(reference);
    }

    /// Add an exact host-issued code symbol. Duplicates reject at `finish`.
    pub fn add_symbol_target(&mut self, target: FrontendSymbolTargetInputV3) {
        self.symbol_targets.push(target);
    }

    /// Add one opaque external catalog asset. Validation remains atomic until
    /// `finish`; no caller can mint owner proof types.
    pub fn add_catalog_asset(&mut self, asset: FrontendCatalogAssetInputV2) {
        self.catalog_assets.push(asset);
    }

    /// Add one dependency-free pack manifest. Validation remains atomic until finish.
    pub fn add_pack(&mut self, pack: FrontendPackInputV2) {
        self.packs.push(pack);
    }

    /// Add one canonical lockfile. Exactly one is required for a nonempty pack graph.
    pub fn add_lockfile(&mut self, lockfile: FrontendLockfileInputV2) {
        self.lockfiles.push(lockfile);
    }

    /// Validate every input and seal one closed authority.
    pub fn finish(self) -> Result<ClosedFrontendAuthorityV2, FrontendAuthorityErrorV2> {
        let host_sources = SourceRegistryViewV2::try_new(
            self.sources
                .into_iter()
                .map(|source| SourceRegistrationV2 {
                    source_id: source.source_id,
                    source_len: source.source_len,
                })
                .collect(),
        )
        .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        let (modules, profile) = seal_modules(
            &host_sources,
            &self.budget,
            self.current_module,
            self.current_module_location,
            self.language_version,
            self.packs,
            self.lockfiles,
            self.catalog_assets,
        )?;
        let relations = seal_relations(&host_sources, self.relations, profile.clone())?;
        let graph_references =
            seal_graph_references(&host_sources, self.graph_references, profile)?;
        let max_symbol_bytes = self.budget.max_authority_bytes();
        let max_symbol_string = self.budget.max_authority_string_bytes();
        let mut authority =
            ClosedFrontendAuthorityV2::try_new(modules, relations, graph_references, self.budget)?;
        seal_symbol_targets(
            &host_sources,
            max_symbol_bytes,
            max_symbol_string,
            self.symbol_targets,
            &mut authority,
        )?;
        Ok(authority)
    }
}

fn seal_modules(
    host_sources: &SourceRegistryViewV2,
    budget: &FrontendBudgetV2,
    current_module: String,
    current_module_location: FrontendAuthorityLocationV2,
    language_version: FrontendLanguageVersionV2,
    packs: Vec<FrontendPackInputV2>,
    lockfiles: Vec<FrontendLockfileInputV2>,
    catalog_assets: Vec<FrontendCatalogAssetInputV2>,
) -> Result<(ValidatedModuleAuthorityPartsV2, AuthorityBudgetProfileV2), FrontendAuthorityErrorV2> {
    let module_location = current_module_location.check(host_sources)?;
    let profile = budget.authority_profile()?;
    let module = ModuleIdV2::try_new(&current_module, module_location.clone())
        .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
    let mut modules = ModuleAuthorityBuilderV2::try_new(profile.clone(), (*host_sources).clone())
        .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
    modules
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: module_location.clone(),
        })
        .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
    for pack in packs {
        let location = pack.location.check(host_sources)?;
        let pack_id = PackIdV2::try_new(&pack.pack_id, location.clone())
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        let content_digest = ContentDigestV2::try_parse(&pack.content_digest, location.clone())
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        let language_range = VersionRangeV2::try_new(
            pack.language_lower.owner(),
            pack.language_upper.owner(),
            location.clone(),
        )
        .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        modules
            .try_add_manifest(PackManifestInputV2 {
                pack_id,
                version: pack.version.owner(),
                content_digest,
                language_range,
                dependencies: Vec::new(),
                exports: Vec::new(),
                suites: Vec::new(),
                location,
            })
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
    }
    for lockfile in lockfiles {
        modules
            .try_set_lockfile(
                LockReferenceInputV2 {
                    path: lockfile.path,
                    location: lockfile.location.check(host_sources)?,
                },
                lockfile.source_id,
                &lockfile.text,
            )
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
    }
    for asset in catalog_assets {
        let location = asset.location.check(host_sources)?;
        let kind_token = asset.payload.tag().to_owned();
        let typed_payload = asset.payload.owner_payload(host_sources, &location)?;
        modules
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: asset.module_name,
                local_name: asset.local_name,
                kind_token,
                typed_payload,
                provenance: asset.provenance,
                location,
                dependencies: Vec::new(),
            })
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
    }
    let modules = modules
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location: module_location,
            },
            VersionV2::new(
                language_version.major,
                language_version.minor,
                language_version.patch,
            ),
        )
        .map_err(|error| configuration_failure(error.code(), error.to_string()))?;

    Ok((modules, profile))
}

fn seal_relations(
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
            let mut value_type = ValueTypeV1::scalar(field.scalar_type.owner());
            if field.optional {
                value_type = ValueTypeV1::try_optional(value_type)
                    .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
            }
            fields.push(
                RecordFieldTypeV1::try_new(&field.name, value_type, field_location)
                    .map_err(|error| configuration_failure(error.code(), error.to_string()))?,
            );
        }
        let row = RecordTypeV1::try_new(fields)
            .map_err(|error| configuration_failure(error.code(), error.to_string()))?;
        relation_inputs.push(RelationSourceInputV2 {
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

fn seal_graph_references(
    host_sources: &SourceRegistryViewV2,
    graph_declarations: Vec<FrontendGraphReferenceInputV2>,
    profile: AuthorityBudgetProfileV2,
) -> Result<GraphReferenceCatalogV2, FrontendAuthorityErrorV2> {
    let graph_inputs = graph_declarations
        .into_iter()
        .map(|reference| {
            Ok(GraphReferenceInputV2 {
                spelling: reference.spelling,
                kind: reference.kind,
                entity_subtype: reference.entity_subtype,
                location: reference.location.check(host_sources)?,
            })
        })
        .collect::<Result<Vec<_>, FrontendAuthorityErrorV2>>()?;
    let graph_references = GraphReferenceCatalogV2::try_new(graph_inputs, host_sources, profile)?;
    Ok(graph_references)
}

fn seal_symbol_targets(
    host_sources: &SourceRegistryViewV2,
    max_symbol_bytes_limit: u32,
    max_symbol_string_limit: u32,
    symbol_targets: Vec<FrontendSymbolTargetInputV3>,
    authority: &mut ClosedFrontendAuthorityV2,
) -> Result<(), FrontendAuthorityErrorV2> {
    let max_symbol_bytes = usize::try_from(max_symbol_bytes_limit).unwrap_or(usize::MAX);
    let max_symbol_string = usize::try_from(max_symbol_string_limit).unwrap_or(usize::MAX);
    let mut symbol_bytes = 0usize;
    for target in symbol_targets {
        if target.selector.is_empty()
            || target.repo_path.is_empty()
            || target.symbol_id.is_empty()
            || target.start_byte >= target.end_byte
        {
            return Err(configuration_failure(
                "sema.invalid_symbol_target",
                "symbol target requires nonempty selector/path/id and a nonempty byte span",
            ));
        }
        let strings = [&target.selector, &target.repo_path, &target.symbol_id];
        if strings.iter().any(|value| value.len() > max_symbol_string) {
            return Err(configuration_failure(
                "sema.symbol_target_budget_exceeded",
                "symbol target string exceeds authority limit",
            ));
        }
        let entry_bytes = strings
            .iter()
            .try_fold(16usize, |acc, value| acc.checked_add(value.len()))
            .ok_or_else(|| {
                configuration_failure(
                    "sema.symbol_target_budget_exceeded",
                    "symbol target size overflow",
                )
            })?;
        symbol_bytes = symbol_bytes.checked_add(entry_bytes).ok_or_else(|| {
            configuration_failure(
                "sema.symbol_target_budget_exceeded",
                "symbol target size overflow",
            )
        })?;
        if symbol_bytes > max_symbol_bytes {
            return Err(configuration_failure(
                "sema.symbol_target_budget_exceeded",
                "symbol target catalog exceeds authority limit",
            ));
        }
        let location = target.location.check(host_sources)?;
        let selector: Arc<str> = Arc::from(target.selector);
        let resolved = ResolvedSymbolTargetV3 {
            selector: Arc::clone(&selector),
            repo_path: Arc::from(target.repo_path),
            symbol_id: Arc::from(target.symbol_id),
            start_byte: target.start_byte,
            end_byte: target.end_byte,
            location,
        };
        if authority
            .symbol_targets
            .insert(selector, resolved)
            .is_some()
        {
            return Err(configuration_failure(
                "sema.duplicate_symbol_target",
                "duplicate symbol target selector",
            ));
        }
    }
    Ok(())
}

impl ClosedFrontendAuthorityV2 {
    /// Seal authority proofs only when registry identity and budget profiles
    /// are exactly the ones selected by the frontend budget.
    pub fn try_new(
        modules: ValidatedModuleAuthorityPartsV2,
        relations: RelationCatalogV2,
        graph_references: GraphReferenceCatalogV2,
        budget: FrontendBudgetV2,
    ) -> Result<Self, FrontendAuthorityErrorV2> {
        let projected = budget.authority_profile()?;
        if modules.source_registry().identity() != *relations.source_registry_identity() {
            return Err(configuration_failure(
                "sema.source_registry_mismatch",
                "module and relation authority use different source registries",
            ));
        }
        if modules.source_registry().identity() != *graph_references.source_registry_identity() {
            return Err(configuration_failure(
                "sema.source_registry_mismatch",
                "graph references and modules use different host source registries",
            ));
        }
        if modules.budget_profile() != &projected
            || relations.budget_profile() != &projected
            || graph_references.budget_profile() != &projected
        {
            return Err(configuration_failure(
                "sema.authority_budget_mismatch",
                "authority proof budget differs from frontend budget projection",
            ));
        }
        Ok(Self {
            modules,
            relations,
            graph_references,
            symbol_targets: BTreeMap::new(),
            budget,
        })
    }

    /// Sealed module authority.
    #[must_use]
    pub const fn modules(&self) -> &ValidatedModuleAuthorityPartsV2 {
        &self.modules
    }
    /// Sealed relation authority.
    #[must_use]
    pub const fn relations(&self) -> &RelationCatalogV2 {
        &self.relations
    }
    /// Host-admitted graph selector authority.
    #[must_use]
    pub const fn graph_references(&self) -> &GraphReferenceCatalogV2 {
        &self.graph_references
    }
    /// Resolve a source spelling to an exact host-issued symbol coordinate.
    #[must_use]
    pub fn symbol_target(&self, selector: &str) -> Option<&ResolvedSymbolTargetV3> {
        self.symbol_targets.get(selector)
    }
    /// Frontend budget.
    #[must_use]
    pub const fn budget(&self) -> &FrontendBudgetV2 {
        &self.budget
    }
    pub(crate) fn try_extend_document(
        &self,
        source_id: SourceIdV2,
        source_len: usize,
    ) -> Result<ExtendedSourceRegistryViewV2, quanta_gqlang_v2_hir::SourceRegistryDiagnosticV2>
    {
        self.modules
            .source_registry()
            .try_extend_document(SourceRegistrationV2 {
                source_id,
                source_len,
            })
    }
}

fn configuration_failure(code: &'static str, message: impl AsRef<str>) -> FrontendAuthorityErrorV2 {
    let value = message.as_ref();
    let mut end = value.len().min(1_024);
    while !value.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    FrontendAuthorityErrorV2 {
        code,
        message: value[..end].to_owned(),
        primary: None,
        secondary: None,
    }
}

fn authority_failure_at(
    code: &'static str,
    message: &'static str,
    primary: LocatedSpanV2,
    secondary: Option<LocatedSpanV2>,
) -> FrontendAuthorityErrorV2 {
    FrontendAuthorityErrorV2 {
        code,
        message: message.to_owned(),
        primary: Some(primary),
        secondary,
    }
}
