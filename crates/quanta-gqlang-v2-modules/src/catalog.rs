//! Checked external and document catalog authority.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::budget::ModuleBudgetSessionV2;
use crate::graph::{
    excerpt, loc_key, CatalogAssetIdV2, CatalogAssetRefInputV2, ModuleIdV2, NamespaceKindV2,
};
use crate::lockfile::{CatalogPackActivationV2, ValidatedPackAuthorityV2};
use crate::manifest::{cycle_with_legacy_steps, PackIdV2};
use crate::session_identity::DocumentSessionIdentityV2;
use crate::session_identity::PredeclaredRequestCustodyV3;
use crate::signature::{CallableKindV2, SignatureInputV2, SignatureV2};

mod family_merge;
mod predeclaration;
mod request_final_demand;
mod view_topology;

use crate::ModuleDiagnosticV2;
use family_merge::merge_family_references;
pub use predeclaration::DocumentPredeclareRequestErrorV3;
pub(crate) use predeclaration::PredeclaredDocumentAssetsV2;
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, LocatedSpanV2, SourceRegistryIdentityV2, SourceRegistryViewV2,
};
use quanta_gqlang_v2_types::{
    RelationTypeV1, ResolvedHostPredicateCallableV2, ResolvedHostPredicateKeyV2, ValueTypeV1,
};
pub(crate) use request_final_demand::request_final_catalog_bytes_v3;
pub use view_topology::DocumentViewTopologyProofV2;
pub(crate) use view_topology::{request_topology_bytes_v3, DocumentViewTopologyStateV2};

// Checked external and document catalog authority -------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ProfileAdmissionV2 {
    Allowlist,
    ReviewRequired,
    CertifiedOnly,
}
impl ProfileAdmissionV2 {
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Allowlist => "allowlist",
            Self::ReviewRequired => "review-required",
            Self::CertifiedOnly => "certified-only",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GateBindingV2 {
    All,
    Any,
}
impl GateBindingV2 {
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Any => "any",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateContractV2 {
    binding: GateBindingV2,
}
impl GateContractV2 {
    #[must_use]
    pub const fn new(binding: GateBindingV2) -> Self {
        Self { binding }
    }
    #[must_use]
    pub const fn binding(&self) -> GateBindingV2 {
        self.binding
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalPackContractV2 {
    pack_id: PackIdV2,
    activation: CatalogPackActivationV2,
}
impl ExternalPackContractV2 {
    #[must_use]
    pub const fn new(pack_id: PackIdV2, activation: CatalogPackActivationV2) -> Self {
        Self {
            pack_id,
            activation,
        }
    }
    #[must_use]
    pub const fn pack_id(&self) -> &PackIdV2 {
        &self.pack_id
    }
    #[must_use]
    pub const fn activation(&self) -> CatalogPackActivationV2 {
        self.activation
    }
}

// Checked and input payloads share the namespace vocabulary. Keep this mapping
// in one place while retaining their separate signature validation and types.
macro_rules! catalog_payload_namespace_kind {
    ($payload:expr) => {
        match $payload {
            Self::Value(_) => NamespaceKindV2::Value,
            Self::Relation(_) => NamespaceKindV2::Relation,
            Self::Predicate(_) => NamespaceKindV2::Predicate,
            Self::Type(_) => NamespaceKindV2::Type,
            Self::Query => NamespaceKindV2::Query,
            Self::Check => NamespaceKindV2::Check,
            Self::Capability => NamespaceKindV2::Capability,
            Self::ModelSource => NamespaceKindV2::ModelSource,
            Self::PackSource => NamespaceKindV2::PackSource,
            Self::View(_) => NamespaceKindV2::View,
            Self::Model(_) => NamespaceKindV2::Model,
            Self::Pack(_) => NamespaceKindV2::Pack,
            Self::Profile(_) => NamespaceKindV2::Profile,
            Self::Gate(_) => NamespaceKindV2::Gate,
            Self::Rule(_) => NamespaceKindV2::Rule,
        }
    };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalCatalogPayloadV2 {
    Value(ValueTypeV1),
    Relation(RelationTypeV1),
    Predicate(SignatureV2),
    Type(ValueTypeV1),
    Query,
    Check,
    Capability,
    ModelSource,
    PackSource,
    View(RelationTypeV1),
    Model(PackIdV2),
    Pack(ExternalPackContractV2),
    Profile(ProfileAdmissionV2),
    Gate(GateContractV2),
    Rule(SignatureV2),
}
impl ExternalCatalogPayloadV2 {
    #[must_use]
    pub const fn namespace_kind(&self) -> NamespaceKindV2 {
        catalog_payload_namespace_kind!(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalCatalogPayloadInputV2 {
    Value(ValueTypeV1),
    Relation(RelationTypeV1),
    Predicate(SignatureInputV2),
    Type(ValueTypeV1),
    Query,
    Check,
    Capability,
    ModelSource,
    PackSource,
    View(RelationTypeV1),
    Model(PackIdV2),
    Pack(ExternalPackContractV2),
    Profile(ProfileAdmissionV2),
    Gate(GateContractV2),
    Rule(SignatureInputV2),
}
impl ExternalCatalogPayloadInputV2 {
    fn namespace_kind(&self) -> NamespaceKindV2 {
        catalog_payload_namespace_kind!(self)
    }
    fn try_into_checked(
        self,
        location: &CheckedLocatedSpanV2,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<ExternalCatalogPayloadV2, ModuleDiagnosticV2> {
        match self {
            Self::Value(value) => Ok(ExternalCatalogPayloadV2::Value(value)),
            Self::Relation(value) => Ok(ExternalCatalogPayloadV2::Relation(value)),
            Self::Query => Ok(ExternalCatalogPayloadV2::Query),
            Self::Check => Ok(ExternalCatalogPayloadV2::Check),
            Self::Capability => Ok(ExternalCatalogPayloadV2::Capability),
            Self::ModelSource => Ok(ExternalCatalogPayloadV2::ModelSource),
            Self::PackSource => Ok(ExternalCatalogPayloadV2::PackSource),
            Self::Predicate(input) => {
                let signature = SignatureV2::try_new(input, source_registry, session)?;
                if signature.callable_kind() != CallableKindV2::Predicate {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.catalog_payload_kind_mismatch",
                        "Predicate payload requires predicate callable kind",
                        location,
                    ));
                }
                Ok(ExternalCatalogPayloadV2::Predicate(signature))
            }
            Self::Type(value) => Ok(ExternalCatalogPayloadV2::Type(value)),
            Self::View(value) => Ok(ExternalCatalogPayloadV2::View(value)),
            Self::Model(value) => {
                session.charge_string(value.as_str())?;
                Ok(ExternalCatalogPayloadV2::Model(value))
            }
            Self::Pack(value) => {
                session.charge_string(value.pack_id().as_str())?;
                Ok(ExternalCatalogPayloadV2::Pack(value))
            }
            Self::Profile(value) => Ok(ExternalCatalogPayloadV2::Profile(value)),
            Self::Gate(value) => Ok(ExternalCatalogPayloadV2::Gate(value)),
            Self::Rule(input) => {
                let signature = SignatureV2::try_new(input, source_registry, session)?;
                if signature.callable_kind() != CallableKindV2::Rule {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.catalog_payload_kind_mismatch",
                        "Rule payload requires rule callable kind",
                        location,
                    ));
                }
                Ok(ExternalCatalogPayloadV2::Rule(signature))
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalCatalogDependencyInputV2 {
    pub target: CatalogAssetRefInputV2,
    pub expected_kind: NamespaceKindV2,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalCatalogAssetInputV2 {
    pub module_name: String,
    pub local_name: String,
    pub kind_token: String,
    pub typed_payload: ExternalCatalogPayloadInputV2,
    pub provenance: String,
    pub location: CheckedLocatedSpanV2,
    pub dependencies: Vec<ExternalCatalogDependencyInputV2>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalCatalogDependencyV2 {
    target_asset_id: CatalogAssetIdV2,
    expected_kind: NamespaceKindV2,
    location: LocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExternalCatalogAssetV2 {
    id: CatalogAssetIdV2,
    kind: NamespaceKindV2,
    typed_payload: ExternalCatalogPayloadV2,
    provenance: String,
    location: LocatedSpanV2,
    dependencies: Vec<ExternalCatalogDependencyV2>,
}

#[derive(Clone, Debug)]
pub(crate) struct PrecheckedExternalCatalogAssetV2 {
    asset: ExternalCatalogAssetV2,
    location: CheckedLocatedSpanV2,
    registry_identity: SourceRegistryIdentityV2,
}

impl PrecheckedExternalCatalogAssetV2 {
    pub(crate) fn try_admit(
        input: ExternalCatalogAssetInputV2,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        session.charge_node()?;
        session.charge_string(&input.module_name)?;
        session.charge_string(&input.local_name)?;
        session.charge_string(&input.kind_token)?;
        session.charge_string(&input.provenance)?;
        ensure_registry(&input.location, source_registry)?;

        for dependency in &input.dependencies {
            session.charge_edge()?;
            session.charge_string(dependency.target.module_id.as_str())?;
            session.charge_string(&dependency.target.local_name)?;
            ensure_registry(&dependency.location, source_registry)?;
            ensure_registry(&dependency.target.location, source_registry)?;
        }

        let module_id = ModuleIdV2::try_new(&input.module_name, input.location.clone())?;
        let kind = NamespaceKindV2::try_from_tag(&input.kind_token, &input.location)?;
        if kind != input.typed_payload.namespace_kind() {
            return Err(ModuleDiagnosticV2::new(
                "modules.catalog_payload_kind_mismatch",
                "catalog kind token does not match typed payload",
                &input.location,
            ));
        }
        let typed_payload =
            input
                .typed_payload
                .try_into_checked(&input.location, source_registry, session)?;
        let id = CatalogAssetIdV2::derive(module_id, kind, &input.local_name, &input.location)?;
        let mut dependencies: BTreeMap<CatalogAssetIdV2, ExternalCatalogDependencyV2> =
            BTreeMap::new();
        for dependency in input.dependencies {
            if dependency.target.namespace_kind != dependency.expected_kind {
                return Err(ModuleDiagnosticV2::new(
                    "modules.catalog_dependency_kind_conflict",
                    "dependency reference kind differs from expected kind",
                    &dependency.location,
                ));
            }
            let target_asset_id = dependency.target.derive()?;
            match dependencies.get_mut(&target_asset_id) {
                Some(existing) if existing.expected_kind != dependency.expected_kind => {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.catalog_dependency_kind_conflict",
                        "same catalog target declared with different expected kinds",
                        &dependency.location,
                    ));
                }
                Some(existing)
                    if loc_key(dependency.location.location()) < loc_key(existing.location) =>
                {
                    existing.location = dependency.location.location();
                }
                Some(_) => {}
                None => {
                    dependencies.insert(
                        target_asset_id.clone(),
                        ExternalCatalogDependencyV2 {
                            target_asset_id,
                            expected_kind: dependency.expected_kind,
                            location: dependency.location.location(),
                        },
                    );
                }
            }
        }
        Ok(Self {
            asset: ExternalCatalogAssetV2 {
                id,
                kind,
                typed_payload,
                provenance: input.provenance,
                location: input.location.location(),
                dependencies: dependencies.into_values().collect(),
            },
            location: input.location,
            registry_identity: source_registry.identity(),
        })
    }

    pub(crate) const fn id(&self) -> &CatalogAssetIdV2 {
        &self.asset.id
    }

    pub(crate) const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}
impl ExternalCatalogAssetV2 {
    #[must_use]
    pub const fn id(&self) -> &CatalogAssetIdV2 {
        &self.id
    }
    #[must_use]
    pub const fn kind(&self) -> NamespaceKindV2 {
        self.kind
    }
    #[must_use]
    pub const fn payload(&self) -> &ExternalCatalogPayloadV2 {
        &self.typed_payload
    }
    #[must_use]
    pub fn provenance(&self) -> &str {
        &self.provenance
    }
    #[must_use]
    pub const fn location(&self) -> LocatedSpanV2 {
        self.location
    }
    #[must_use]
    pub fn dependencies(&self) -> &[ExternalCatalogDependencyV2] {
        &self.dependencies
    }
}

#[derive(Clone, Debug)]
pub struct ExternalCatalogV2 {
    assets: BTreeMap<CatalogAssetIdV2, ExternalCatalogAssetV2>,
    registry_identity: SourceRegistryIdentityV2,
}
impl ExternalCatalogV2 {
    pub(crate) fn try_new(
        inputs: Vec<PrecheckedExternalCatalogAssetV2>,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        let mut assets: BTreeMap<CatalogAssetIdV2, ExternalCatalogAssetV2> = BTreeMap::new();
        let mut original_locations: BTreeMap<CatalogAssetIdV2, CheckedLocatedSpanV2> =
            BTreeMap::new();
        for input in inputs {
            if input.registry_identity != source_registry.identity() {
                return Err(ModuleDiagnosticV2::new(
                    "modules.source_registry_mismatch",
                    "prechecked external asset belongs to another registry",
                    &input.location,
                ));
            }
            let id = input.asset.id.clone();
            if let Some(original) = original_locations.get(&id) {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_external_catalog_asset",
                    "duplicate external catalog asset",
                    &input.location,
                    original,
                ));
            }
            original_locations.insert(id.clone(), input.location);
            assets.insert(id, input.asset);
        }
        for asset in assets.values() {
            for dependency in &asset.dependencies {
                session.charge_step()?;
                let Some(target) = assets.get(&dependency.target_asset_id) else {
                    let edge_location = source_registry
                        .try_check_location(dependency.location)
                        .map_err(|_source_error| {
                            ModuleDiagnosticV2::without_location(
                                "modules.source_registry_mismatch",
                                "catalog edge location lost registry proof",
                            )
                        })?;
                    return Err(ModuleDiagnosticV2::new(
                        "modules.unknown_catalog_reference",
                        format!(
                            "external dependency target `{}` is absent",
                            excerpt(dependency.target_asset_id.local_name())
                        ),
                        &edge_location,
                    ));
                };
                if target.kind != dependency.expected_kind {
                    let edge_location = source_registry
                        .try_check_location(dependency.location)
                        .map_err(|_source_error| {
                            ModuleDiagnosticV2::without_location(
                                "modules.source_registry_mismatch",
                                "catalog edge location lost registry proof",
                            )
                        })?;
                    let target_location = source_registry
                        .try_check_location(target.location)
                        .map_err(|_source_error| {
                            ModuleDiagnosticV2::without_location(
                                "modules.source_registry_mismatch",
                                "catalog target location lost registry proof",
                            )
                        })?;
                    return Err(ModuleDiagnosticV2::conflict(
                        "modules.catalog_reference_kind_mismatch",
                        "external dependency target has wrong kind",
                        &edge_location,
                        &target_location,
                    ));
                }
            }
        }
        if let Some(cycle) = catalog_cycle(&assets, session)? {
            let Some(first_location) = cycle.first().copied() else {
                return Err(ModuleDiagnosticV2::without_location(
                    "modules.catalog_dependency_cycle",
                    "empty external catalog cycle witness",
                ));
            };
            let first =
                source_registry
                    .try_check_location(first_location)
                    .map_err(|_source_error| {
                        ModuleDiagnosticV2::without_location(
                            "modules.source_registry_mismatch",
                            "cycle location lost registry proof",
                        )
                    })?;
            return Err(ModuleDiagnosticV2::new(
                "modules.catalog_dependency_cycle",
                "external catalog dependency cycle",
                &first,
            )
            .with_witness(cycle));
        }
        Ok(Self {
            assets,
            registry_identity: source_registry.identity(),
        })
    }
    #[must_use]
    pub fn asset(&self, id: &CatalogAssetIdV2) -> Option<&ExternalCatalogAssetV2> {
        self.assets.get(id)
    }
    pub fn iter(&self) -> impl Iterator<Item = &ExternalCatalogAssetV2> {
        self.assets.values()
    }
    #[must_use]
    pub const fn source_registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.registry_identity
    }
}
fn ensure_registry(
    location: &CheckedLocatedSpanV2,
    registry: &SourceRegistryViewV2,
) -> Result<(), ModuleDiagnosticV2> {
    if location.registry_identity() != &registry.identity() {
        Err(ModuleDiagnosticV2::new(
            "modules.source_registry_mismatch",
            "location belongs to another source registry",
            location,
        ))
    } else {
        Ok(())
    }
}

fn catalog_cycle(
    assets: &BTreeMap<CatalogAssetIdV2, ExternalCatalogAssetV2>,
    session: &mut ModuleBudgetSessionV2,
) -> Result<Option<Vec<LocatedSpanV2>>, ModuleDiagnosticV2> {
    let edges = assets
        .iter()
        .flat_map(|(source, asset)| {
            asset.dependencies.iter().map(move |edge| {
                (
                    source.clone(),
                    edge.target_asset_id.clone(),
                    (edge.target_asset_id.clone(), loc_key(edge.location)),
                    edge.location,
                )
            })
        })
        .collect::<Vec<_>>();
    let roots = assets.keys().cloned().collect::<Vec<_>>();
    let keyed = edges
        .iter()
        .map(|(source, target, key, _)| (source.clone(), target.clone(), key.clone()))
        .collect::<Vec<_>>();
    let Some(indices) = cycle_with_legacy_steps(&keyed, &roots, session)? else {
        return Ok(None);
    };
    let mut witness = Vec::with_capacity(indices.len());
    for index in indices {
        let Some(edge) = edges.get(index) else {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.internal_cycle_witness_mismatch",
                "catalog cycle witness index is outside the validated edge set",
            ));
        };
        witness.push(edge.3);
    }
    Ok(Some(witness))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentCatalogAssetInputV2 {
    pub owner_module_id: ModuleIdV2,
    pub local_name: String,
    pub namespace_kind: NamespaceKindV2,
    pub location: CheckedLocatedSpanV2,
}
/// One local asset sealed during document predeclaration. Its ID is derived by
/// modules, not supplied by the semantic consumer.
#[derive(Clone, Debug)]
pub struct PredeclaredDocumentAssetV2 {
    id: CatalogAssetIdV2,
    kind: NamespaceKindV2,
    location: CheckedLocatedSpanV2,
    session_key: Arc<DocumentSessionIdentityV2>,
    input_ordinal: usize,
    _request_custody: Option<Arc<PredeclaredRequestCustodyV3>>,
}
impl PartialEq for PredeclaredDocumentAssetV2 {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.kind == other.kind
            && self.location == other.location
            && self.session_key == other.session_key
            && self.input_ordinal == other.input_ordinal
    }
}
impl Eq for PredeclaredDocumentAssetV2 {}
impl PredeclaredDocumentAssetV2 {
    #[must_use]
    pub const fn id(&self) -> &CatalogAssetIdV2 {
        &self.id
    }
    #[must_use]
    pub const fn kind(&self) -> NamespaceKindV2 {
        self.kind
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    /// Position in the authored input sequence, retained across ID sorting.
    #[must_use]
    pub const fn input_ordinal(&self) -> usize {
        self.input_ordinal
    }
    pub(crate) fn belongs_to(&self, key: &Arc<DocumentSessionIdentityV2>) -> bool {
        Arc::ptr_eq(&self.session_key, key)
    }
}

/// A catalog name within an explicitly selected module, or the current
/// module's visible namespace when `module_id` is absent. `local_name` retains
/// the exact source path spelling, including dots, slashes, and `::`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentCatalogReferenceInputV2 {
    pub module_id: Option<ModuleIdV2>,
    pub local_name: String,
    pub expected_kind: NamespaceKindV2,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentCatalogReferenceOriginV2 {
    Local,
    VisibleHost,
    External,
}

/// Modules-owned exact ID and kind proof for one document reference.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedDocumentCatalogReferenceV2 {
    id: CatalogAssetIdV2,
    kind: NamespaceKindV2,
    origin: DocumentCatalogReferenceOriginV2,
    location: CheckedLocatedSpanV2,
    session_key: Arc<DocumentSessionIdentityV2>,
}
impl ResolvedDocumentCatalogReferenceV2 {
    pub(crate) fn new(
        id: CatalogAssetIdV2,
        kind: NamespaceKindV2,
        origin: DocumentCatalogReferenceOriginV2,
        location: CheckedLocatedSpanV2,
        session_key: Arc<DocumentSessionIdentityV2>,
    ) -> Self {
        Self {
            id,
            kind,
            origin,
            location,
            session_key,
        }
    }
    #[must_use]
    pub const fn id(&self) -> &CatalogAssetIdV2 {
        &self.id
    }
    #[must_use]
    pub const fn kind(&self) -> NamespaceKindV2 {
        self.kind
    }
    #[must_use]
    pub const fn origin(&self) -> DocumentCatalogReferenceOriginV2 {
        self.origin
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
    pub(crate) fn belongs_to(&self, key: &Arc<DocumentSessionIdentityV2>) -> bool {
        Arc::ptr_eq(&self.session_key, key)
    }
}

/// Exact host Predicate callable admitted by the same document validator
/// session that resolved its catalog reference. The neutral types callable is
/// usable only through this modules-owned catalog/signature proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedHostPredicateProofV2 {
    reference: ResolvedDocumentCatalogReferenceV2,
    signature: SignatureV2,
    callable: ResolvedHostPredicateCallableV2,
}

impl ResolvedHostPredicateProofV2 {
    pub(crate) fn new(
        reference: ResolvedDocumentCatalogReferenceV2,
        signature: SignatureV2,
        callable: ResolvedHostPredicateCallableV2,
    ) -> Self {
        Self {
            reference,
            signature,
            callable,
        }
    }

    /// Underlying exact catalog identity, never the imported alias spelling.
    #[must_use]
    pub const fn id(&self) -> &CatalogAssetIdV2 {
        self.reference.id()
    }

    /// The checked source call location.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        self.reference.location()
    }

    /// How the host Predicate was made visible to this document.
    #[must_use]
    pub const fn origin(&self) -> DocumentCatalogReferenceOriginV2 {
        self.reference.origin()
    }

    /// Exact host-catalog signature, including authored parameter order.
    #[must_use]
    pub const fn signature(&self) -> &SignatureV2 {
        &self.signature
    }

    /// Types-owned callable projected from the exact checked signature.
    #[must_use]
    pub const fn callable(&self) -> &ResolvedHostPredicateCallableV2 {
        &self.callable
    }

    /// Injective host callable key carrying full module, namespace and path.
    #[must_use]
    pub fn key(&self) -> &ResolvedHostPredicateKeyV2 {
        self.callable.key()
    }

    pub(crate) fn belongs_to(&self, key: &Arc<DocumentSessionIdentityV2>) -> bool {
        self.reference.belongs_to(key)
    }
}

/// Exact source/target kind policy for the six catalog payload references.
/// The inert target kinds do not imply a runtime registry or provider lookup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentCatalogReferenceFamilyV2 {
    ViewQuery,
    ModelSource,
    PackSource,
    ProfileCapability,
    GateCheck,
    RuleBindingCheck,
}
impl DocumentCatalogReferenceFamilyV2 {
    #[must_use]
    pub const fn source_kind(self) -> NamespaceKindV2 {
        match self {
            Self::ViewQuery => NamespaceKindV2::View,
            Self::ModelSource => NamespaceKindV2::Model,
            Self::PackSource => NamespaceKindV2::Pack,
            Self::ProfileCapability => NamespaceKindV2::Profile,
            Self::GateCheck => NamespaceKindV2::Gate,
            Self::RuleBindingCheck => NamespaceKindV2::Rule,
        }
    }
    #[must_use]
    pub const fn target_kind(self) -> NamespaceKindV2 {
        match self {
            Self::ViewQuery => NamespaceKindV2::Query,
            Self::ModelSource => NamespaceKindV2::ModelSource,
            Self::PackSource => NamespaceKindV2::PackSource,
            Self::ProfileCapability => NamespaceKindV2::Capability,
            Self::GateCheck | Self::RuleBindingCheck => NamespaceKindV2::Check,
        }
    }
}

/// A fully pre-resolved family edge; final validation checks its provenance
/// against the same validator's predeclared map and sealed host catalog.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentCatalogFamilyReferenceInputV2 {
    pub family: DocumentCatalogReferenceFamilyV2,
    pub source: PredeclaredDocumentAssetV2,
    pub target: ResolvedDocumentCatalogReferenceV2,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfileAdmissionInputV2 {
    pub profile: CatalogAssetRefInputV2,
    pub admission: ProfileAdmissionV2,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentCatalogDependencyInputV2 {
    pub source: CatalogAssetRefInputV2,
    pub target: CatalogAssetRefInputV2,
    pub expected_target_kind: NamespaceKindV2,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentCatalogDependencyV2 {
    source_asset_id: CatalogAssetIdV2,
    target_asset_id: CatalogAssetIdV2,
    expected_target_kind: NamespaceKindV2,
    family: Option<DocumentCatalogReferenceFamilyV2>,
    location: CheckedLocatedSpanV2,
}
impl DocumentCatalogDependencyV2 {
    #[must_use]
    pub const fn source_asset_id(&self) -> &CatalogAssetIdV2 {
        &self.source_asset_id
    }
    #[must_use]
    pub const fn target_asset_id(&self) -> &CatalogAssetIdV2 {
        &self.target_asset_id
    }
    #[must_use]
    pub const fn expected_target_kind(&self) -> NamespaceKindV2 {
        self.expected_target_kind
    }
    #[must_use]
    pub const fn family(&self) -> Option<DocumentCatalogReferenceFamilyV2> {
        self.family
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequireCertifiedInputV2 {
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentCatalogInputV2 {
    pub assets: Vec<DocumentCatalogAssetInputV2>,
    pub profile_admissions: Vec<ProfileAdmissionInputV2>,
    pub activations: Vec<crate::PackActivationInputV2>,
    pub require_certified_directives: Vec<RequireCertifiedInputV2>,
    pub dependency_edges: Vec<DocumentCatalogDependencyInputV2>,
}

/// Post-predeclaration input. It deliberately has no asset list: local IDs
/// were already minted and charged by the validator's single session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentCatalogFinalInputV2 {
    pub profile_admissions: Vec<ProfileAdmissionInputV2>,
    pub activations: Vec<crate::PackActivationInputV2>,
    pub require_certified_directives: Vec<RequireCertifiedInputV2>,
    pub family_references: Vec<DocumentCatalogFamilyReferenceInputV2>,
}

#[derive(Clone, Debug)]
pub struct DocumentCatalogAuthorityV2 {
    assets: Arc<PredeclaredDocumentAssetsV2>,
    profiles: Vec<(CatalogAssetIdV2, ProfileAdmissionV2, CheckedLocatedSpanV2)>,
    dependencies: Vec<DocumentCatalogDependencyV2>,
    activations: crate::ValidatedPackActivationsV2,
    require_certified: Option<LocatedSpanV2>,
    registry_identity: SourceRegistryIdentityV2,
}
impl DocumentCatalogAuthorityV2 {
    pub(crate) fn try_new_predeclared(
        input: DocumentCatalogFinalInputV2,
        predeclared: &Arc<PredeclaredDocumentAssetsV2>,
        topology: &DocumentViewTopologyProofV2,
        visible: &crate::ResolvedGraphV2,
        external: &ExternalCatalogV2,
        packs: &ValidatedPackAuthorityV2,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
        session_key: &Arc<DocumentSessionIdentityV2>,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if !predeclared.belongs_to(session_key) {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.catalog_reference_session_mismatch",
                "predeclaration belongs to another document validator",
            ));
        }
        if !topology.belongs_to(session_key) {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.catalog_reference_session_mismatch",
                "View topology proof belongs to another document validator",
            ));
        }
        let DocumentCatalogFinalInputV2 {
            profile_admissions,
            activations,
            require_certified_directives,
            family_references,
        } = input;
        // Activation authority participates in REQUIRE CERTIFIED reachability,
        // so validate and seal it before certification closure.
        let activations = packs.validate_document_activations(
            activations,
            &source_registry.identity(),
            session,
        )?;
        let context = CatalogSealContext {
            predeclared,
            topology,
            visible,
            external,
            source_registry,
            session_key,
        };
        let profiles = admit_profiles(profile_admissions, &context, session)?;
        let dependency_edges = merge_family_references(family_references, &context, session)?;
        let mut require_location: Option<CheckedLocatedSpanV2> = None;
        for directive in require_certified_directives {
            session.charge_edge()?;
            ensure_registry(&directive.location, source_registry)?;
            if require_location.as_ref().map_or(true, |current| {
                loc_key(directive.location.location()) < loc_key(current.location())
            }) {
                require_location = Some(directive.location);
            }
        }
        if let Some(require) = &require_location {
            for (id, admission, _) in &profiles {
                if *admission != ProfileAdmissionV2::CertifiedOnly {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.certified_required_violation",
                        format!(
                            "profile `{}` is not certified-only",
                            excerpt(id.local_name())
                        ),
                        require,
                    ));
                }
            }
            let reachable = reachable_external_profiles(&dependency_edges, external, &activations)?;
            for asset in reachable {
                if let ExternalCatalogPayloadV2::Profile(admission) = asset.payload() {
                    if *admission != ProfileAdmissionV2::CertifiedOnly {
                        return Err(ModuleDiagnosticV2::new(
                            "modules.certified_required_violation",
                            format!(
                                "external profile `{}` is not certified-only",
                                excerpt(asset.id.local_name())
                            ),
                            require,
                        ));
                    }
                }
            }
        }
        Ok(Self {
            assets: predeclared.clone(),
            profiles,
            dependencies: dependency_edges,
            activations,
            require_certified: require_location.map(|location| location.location()),
            registry_identity: source_registry.identity(),
        })
    }
    pub fn assets(&self) -> impl Iterator<Item = (&CatalogAssetIdV2, NamespaceKindV2)> {
        self.assets.iter_borrowed().map(|(id, kind, _)| (id, kind))
    }
    #[must_use]
    pub const fn activations(&self) -> &crate::ValidatedPackActivationsV2 {
        &self.activations
    }
    pub fn profiles(&self) -> impl Iterator<Item = (&CatalogAssetIdV2, ProfileAdmissionV2)> {
        self.profiles
            .iter()
            .map(|(id, admission, _)| (id, *admission))
    }
    /// Exact validated dependency and family-reference facts in stable ID order.
    #[must_use]
    pub fn dependencies(&self) -> &[DocumentCatalogDependencyV2] {
        &self.dependencies
    }
    #[must_use]
    pub const fn require_certified_location(&self) -> Option<LocatedSpanV2> {
        self.require_certified
    }
    #[must_use]
    pub const fn source_registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.registry_identity
    }
}

struct CatalogSealContext<'a> {
    predeclared: &'a PredeclaredDocumentAssetsV2,
    topology: &'a DocumentViewTopologyProofV2,
    visible: &'a crate::ResolvedGraphV2,
    external: &'a ExternalCatalogV2,
    source_registry: &'a SourceRegistryViewV2,
    session_key: &'a Arc<DocumentSessionIdentityV2>,
}

fn admit_profiles(
    profile_admissions: Vec<ProfileAdmissionInputV2>,
    context: &CatalogSealContext<'_>,
    session: &mut ModuleBudgetSessionV2,
) -> Result<Vec<(CatalogAssetIdV2, ProfileAdmissionV2, CheckedLocatedSpanV2)>, ModuleDiagnosticV2> {
    let source_registry = context.source_registry;
    let mut profiles: Vec<(CatalogAssetIdV2, ProfileAdmissionV2, CheckedLocatedSpanV2)> =
        Vec::new();
    for profile in profile_admissions {
        session.charge_edge()?;
        session.charge_string(profile.profile.module_id.as_str())?;
        session.charge_string(&profile.profile.local_name)?;
        ensure_registry(&profile.location, source_registry)?;
        ensure_registry(&profile.profile.location, source_registry)?;
        let profile_asset_id = profile.profile.derive()?;
        match context.predeclared.get(&profile_asset_id) {
            Some(NamespaceKindV2::Profile) => {}
            Some(_) => {
                return Err(ModuleDiagnosticV2::new(
                    "modules.profile_admission_wrong_kind",
                    "profile admission target is not a Profile",
                    &profile.location,
                ))
            }
            None => {
                return Err(ModuleDiagnosticV2::new(
                    "modules.unknown_profile_admission",
                    "profile admission target is absent",
                    &profile.location,
                ))
            }
        }
        match profiles.binary_search_by(|(id, _, _)| id.cmp(&profile_asset_id)) {
            Ok(index) => {
                let Some((_, _, original)) = profiles.get(index) else {
                    return Err(ModuleDiagnosticV2::without_location(
                        "modules.internal_profile_index_mismatch",
                        "sorted profile index is outside the admitted profile set",
                    ));
                };
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_profile_admission",
                    "duplicate profile admission",
                    &profile.location,
                    original,
                ));
            }
            Err(index) => profiles.insert(
                index,
                (profile_asset_id, profile.admission, profile.location),
            ),
        }
    }
    for (id, kind, _) in context.predeclared.iter_borrowed() {
        if kind == NamespaceKindV2::Profile
            && profiles
                .binary_search_by(|(candidate, _, _)| candidate.cmp(id))
                .is_err()
        {
            let Some(location) = context.predeclared.location(id) else {
                return Err(ModuleDiagnosticV2::without_location(
                    "modules.missing_profile_admission",
                    "profile location missing from document catalog",
                ));
            };
            return Err(ModuleDiagnosticV2::new(
                "modules.missing_profile_admission",
                "local Profile requires exactly one admission",
                location,
            ));
        }
    }
    Ok(profiles)
}

fn reachable_external_profiles<'a>(
    edges: &'a [DocumentCatalogDependencyV2],
    external: &'a ExternalCatalogV2,
    activations: &crate::ValidatedPackActivationsV2,
) -> Result<Vec<&'a ExternalCatalogAssetV2>, ModuleDiagnosticV2> {
    let (queue_slots, profile_slots) = reachable_profile_slots_v3(
        edges.len(),
        external,
        activations.iter().map(|activation| activation.pack_id()),
    )
    .ok_or_else(|| {
        ModuleDiagnosticV2::without_location(
            "modules.resource_exhausted",
            "reachable profile capacity overflowed",
        )
    })?;
    let mut queue = Vec::with_capacity(queue_slots);
    queue.extend(edges.iter().map(|edge| &edge.target_asset_id));
    for activation in activations.iter() {
        queue.extend(external.iter().filter_map(|asset| match asset.payload() {
            ExternalCatalogPayloadV2::Pack(contract)
                if contract.pack_id() == activation.pack_id() =>
            {
                Some(asset.id())
            }
            ExternalCatalogPayloadV2::Pack(_)
            | ExternalCatalogPayloadV2::Value(_)
            | ExternalCatalogPayloadV2::Relation(_)
            | ExternalCatalogPayloadV2::Predicate(_)
            | ExternalCatalogPayloadV2::Type(_)
            | ExternalCatalogPayloadV2::Query
            | ExternalCatalogPayloadV2::Check
            | ExternalCatalogPayloadV2::Capability
            | ExternalCatalogPayloadV2::ModelSource
            | ExternalCatalogPayloadV2::PackSource
            | ExternalCatalogPayloadV2::View(_)
            | ExternalCatalogPayloadV2::Model(_)
            | ExternalCatalogPayloadV2::Profile(_)
            | ExternalCatalogPayloadV2::Gate(_)
            | ExternalCatalogPayloadV2::Rule(_) => None,
        }));
    }
    // Every reachable ID is either an initial target or an external edge
    // target. Index all candidates once, then only flip a visited bit.
    let mut seen = Vec::with_capacity(queue_slots);
    seen.extend(queue.iter().copied().map(|id| (id, false)));
    seen.extend(external.iter().flat_map(|asset| {
        asset
            .dependencies
            .iter()
            .map(|edge| (&edge.target_asset_id, false))
    }));
    seen.sort_unstable_by(|left, right| left.0.cmp(right.0));
    seen.dedup_by(|left, right| left.0 == right.0);
    let mut profiles = Vec::with_capacity(profile_slots);
    while let Some(id) = queue.pop() {
        let index = seen
            .binary_search_by(|candidate| candidate.0.cmp(id))
            .map_err(|_index_error| {
                ModuleDiagnosticV2::without_location(
                    "modules.internal_dependency_index_mismatch",
                    "reachable target is outside the sealed candidate index",
                )
            })?;
        let Some(entry) = seen.get_mut(index) else {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.internal_dependency_index_mismatch",
                "visited index is outside the sealed candidate set",
            ));
        };
        if entry.1 {
            continue;
        }
        entry.1 = true;
        if let Some(asset) = external.asset(id) {
            if asset.kind == NamespaceKindV2::Profile {
                profiles.push(asset);
            }
            queue.extend(asset.dependencies.iter().map(|edge| &edge.target_asset_id));
        }
    }
    // External catalog IDs are unique; unstable ordering preserves the exact
    // ID order without allocating stable-sort scratch.
    profiles.sort_unstable_by(|left, right| left.id.cmp(&right.id));
    Ok(profiles)
}

pub(super) fn reachable_profile_slots_v3<'a>(
    dependency_count: usize,
    external: &ExternalCatalogV2,
    activations: impl Iterator<Item = &'a PackIdV2>,
) -> Option<(usize, usize)> {
    let mut queue_slots = dependency_count;
    for activation in activations {
        let matching_packs = external
            .iter()
            .filter(|asset| {
                matches!(
                    asset.payload(),
                    ExternalCatalogPayloadV2::Pack(contract) if contract.pack_id() == activation
                )
            })
            .count();
        queue_slots = queue_slots.checked_add(matching_packs)?;
    }
    let mut profile_slots = 0_usize;
    for asset in external.iter() {
        queue_slots = queue_slots.checked_add(asset.dependencies.len())?;
        if asset.kind == NamespaceKindV2::Profile {
            profile_slots = profile_slots.checked_add(1)?;
        }
    }
    Some((queue_slots, profile_slots))
}
