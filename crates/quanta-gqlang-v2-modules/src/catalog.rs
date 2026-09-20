//! Checked external and document catalog authority.

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::budget::ModuleBudgetSessionV2;
use crate::graph::{
    excerpt, loc_key, CatalogAssetIdV2, CatalogAssetRefInputV2, ModuleIdV2, NamespaceKindV2,
};
use crate::lockfile::{CatalogPackActivationV2, ValidatedPackAuthorityV2};
use crate::manifest::PackIdV2;
use crate::signature::{CallableKindV2, SignatureInputV2, SignatureV2};
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, LocatedSpanV2, SourceRegistryIdentityV2, SourceRegistryViewV2,
};
use quanta_gqlang_v2_types::{
    RelationTypeV1, ResolvedHostPredicateCallableV2, ResolvedHostPredicateKeyV2, ValueTypeV1,
};

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
        match self {
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
        match self {
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
    let mut found: Vec<Vec<(CatalogAssetIdV2, LocatedSpanV2)>> = Vec::new();
    for root in assets.keys() {
        let mut stack = vec![(
            root.clone(),
            Vec::new(),
            std::collections::BTreeSet::from([root.clone()]),
        )];
        while let Some((node, path, seen)) = stack.pop() {
            let Some(asset) = assets.get(&node) else {
                continue;
            };
            for edge in asset.dependencies.iter().rev() {
                session.charge_step()?;
                if seen.contains(&edge.target_asset_id) {
                    let position = path
                        .iter()
                        .position(|(id, _)| id == &edge.target_asset_id)
                        .map_or(0, |index| index + 1);
                    let mut cycle = path.get(position..).map_or_else(Vec::new, <[_]>::to_vec);
                    cycle.push((edge.target_asset_id.clone(), edge.location));
                    cycle = canonicalize_catalog_cycle(cycle);
                    found.push(cycle);
                } else if !seen.contains(&edge.target_asset_id) {
                    let mut next_path = path.clone();
                    next_path.push((edge.target_asset_id.clone(), edge.location));
                    let mut next_seen = seen.clone();
                    next_seen.insert(edge.target_asset_id.clone());
                    stack.push((edge.target_asset_id.clone(), next_path, next_seen));
                }
            }
        }
    }
    found.sort_by(|left, right| {
        left.iter()
            .map(|(id, loc)| (id.clone(), loc_key(*loc)))
            .collect::<Vec<_>>()
            .cmp(
                &right
                    .iter()
                    .map(|(id, loc)| (id.clone(), loc_key(*loc)))
                    .collect::<Vec<_>>(),
            )
    });
    Ok(found
        .into_iter()
        .next()
        .map(|cycle| cycle.into_iter().map(|(_, location)| location).collect()))
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PredeclaredDocumentAssetV2 {
    id: CatalogAssetIdV2,
    kind: NamespaceKindV2,
    location: CheckedLocatedSpanV2,
    session_key: Arc<()>,
}
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
    pub(crate) fn belongs_to(&self, key: &Arc<()>) -> bool {
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
    session_key: Arc<()>,
}
impl ResolvedDocumentCatalogReferenceV2 {
    pub(crate) fn new(
        id: CatalogAssetIdV2,
        kind: NamespaceKindV2,
        origin: DocumentCatalogReferenceOriginV2,
        location: CheckedLocatedSpanV2,
        session_key: Arc<()>,
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
    pub(crate) fn belongs_to(&self, key: &Arc<()>) -> bool {
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

    pub(crate) fn belongs_to(&self, key: &Arc<()>) -> bool {
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
    pub dependency_edges: Vec<DocumentCatalogDependencyInputV2>,
    pub family_references: Vec<DocumentCatalogFamilyReferenceInputV2>,
}

#[derive(Clone, Debug)]
pub(crate) struct PredeclaredDocumentAssetsV2 {
    assets: BTreeMap<CatalogAssetIdV2, NamespaceKindV2>,
    locations: BTreeMap<CatalogAssetIdV2, CheckedLocatedSpanV2>,
    session_key: Arc<()>,
}
impl PredeclaredDocumentAssetsV2 {
    pub(crate) fn try_new(
        inputs: Vec<DocumentCatalogAssetInputV2>,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
        session_key: Arc<()>,
    ) -> Result<Self, ModuleDiagnosticV2> {
        let mut assets = BTreeMap::new();
        let mut locations = BTreeMap::new();
        for asset in inputs {
            session.charge_node()?;
            session.charge_string(asset.owner_module_id.as_str())?;
            session.charge_string(&asset.local_name)?;
            ensure_registry(&asset.location, source_registry)?;
            let id = CatalogAssetIdV2::derive(
                asset.owner_module_id,
                asset.namespace_kind,
                &asset.local_name,
                &asset.location,
            )?;
            if let Some(original) = locations.get(&id) {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_document_catalog_asset",
                    "duplicate document catalog asset",
                    &asset.location,
                    original,
                ));
            }
            locations.insert(id.clone(), asset.location);
            assets.insert(id, asset.namespace_kind);
        }
        Ok(Self {
            assets,
            locations,
            session_key,
        })
    }
    pub(crate) fn get(&self, id: &CatalogAssetIdV2) -> Option<NamespaceKindV2> {
        self.assets.get(id).copied()
    }
    pub(crate) fn location(&self, id: &CatalogAssetIdV2) -> Option<&CheckedLocatedSpanV2> {
        self.locations.get(id)
    }
    pub(crate) fn iter(&self) -> impl Iterator<Item = PredeclaredDocumentAssetV2> + '_ {
        self.assets.iter().filter_map(|(id, kind)| {
            self.locations
                .get(id)
                .map(|location| PredeclaredDocumentAssetV2 {
                    id: id.clone(),
                    kind: *kind,
                    location: location.clone(),
                    session_key: self.session_key.clone(),
                })
        })
    }
    pub(crate) fn belongs_to(&self, key: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.session_key, key)
    }
}

#[derive(Clone, Debug)]
pub struct DocumentCatalogAuthorityV2 {
    assets: BTreeMap<CatalogAssetIdV2, NamespaceKindV2>,
    profiles: BTreeMap<CatalogAssetIdV2, ProfileAdmissionV2>,
    dependencies: Vec<DocumentCatalogDependencyV2>,
    activations: crate::ValidatedPackActivationsV2,
    require_certified: Option<LocatedSpanV2>,
    registry_identity: SourceRegistryIdentityV2,
}
impl DocumentCatalogAuthorityV2 {
    pub(crate) fn try_new_predeclared(
        input: DocumentCatalogFinalInputV2,
        predeclared: &PredeclaredDocumentAssetsV2,
        visible: &crate::ResolvedGraphV2,
        external: &ExternalCatalogV2,
        packs: &ValidatedPackAuthorityV2,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
        session_key: &Arc<()>,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if !predeclared.belongs_to(session_key) {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.catalog_reference_session_mismatch",
                "predeclaration belongs to another document validator",
            ));
        }
        // Activation authority participates in REQUIRE CERTIFIED reachability,
        // so validate and seal it before certification closure.
        let activations = packs.validate_document_activations(
            input.activations,
            &source_registry.identity(),
            session,
        )?;
        let assets = predeclared.assets.clone();
        let asset_locations = &predeclared.locations;
        let mut profiles = BTreeMap::new();
        let mut profile_locations: BTreeMap<CatalogAssetIdV2, CheckedLocatedSpanV2> =
            BTreeMap::new();
        for profile in input.profile_admissions {
            session.charge_edge()?;
            session.charge_string(profile.profile.module_id.as_str())?;
            session.charge_string(&profile.profile.local_name)?;
            ensure_registry(&profile.location, source_registry)?;
            ensure_registry(&profile.profile.location, source_registry)?;
            let profile_asset_id = profile.profile.derive()?;
            match assets.get(&profile_asset_id) {
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
            if let Some(original) = profile_locations.get(&profile_asset_id) {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_profile_admission",
                    "duplicate profile admission",
                    &profile.location,
                    original,
                ));
            }
            profile_locations.insert(profile_asset_id.clone(), profile.location);
            profiles.insert(profile_asset_id, profile.admission);
        }
        for (id, kind) in &assets {
            if *kind == NamespaceKindV2::Profile && !profiles.contains_key(id) {
                let Some(location) = asset_locations.get(id) else {
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
        let mut dependency_edges: BTreeMap<
            (CatalogAssetIdV2, CatalogAssetIdV2),
            DocumentCatalogDependencyV2,
        > = BTreeMap::new();
        for edge in input.dependency_edges {
            session.charge_edge()?;
            session.charge_string(edge.source.module_id.as_str())?;
            session.charge_string(&edge.source.local_name)?;
            session.charge_string(edge.target.module_id.as_str())?;
            session.charge_string(&edge.target.local_name)?;
            ensure_registry(&edge.location, source_registry)?;
            ensure_registry(&edge.source.location, source_registry)?;
            ensure_registry(&edge.target.location, source_registry)?;
            if edge.target.namespace_kind != edge.expected_target_kind {
                return Err(ModuleDiagnosticV2::new(
                    "modules.catalog_reference_kind_mismatch",
                    "target reference kind differs from expected kind",
                    &edge.location,
                ));
            }
            let source_asset_id = edge.source.derive()?;
            let target_asset_id = edge.target.derive()?;
            if assets.get(&source_asset_id) != Some(&NamespaceKindV2::View) {
                return Err(ModuleDiagnosticV2::new(
                    "modules.catalog_dependency_source_wrong_kind",
                    "document dependency source must be a local View",
                    &edge.location,
                ));
            }
            let local_target = assets.get(&target_asset_id).copied();
            let external_target = external.asset(&target_asset_id);
            let visible_target = visible
                .visible_bindings()
                .iter()
                .find(|binding| binding.resolved_asset_id() == &target_asset_id);
            let actual = local_target
                .or_else(|| external_target.map(ExternalCatalogAssetV2::kind))
                .or_else(|| visible_target.map(crate::ResolvedBindingV2::namespace_kind));
            let Some(actual) = actual else {
                return Err(ModuleDiagnosticV2::new(
                    "modules.unknown_catalog_reference",
                    "document dependency target is absent",
                    &edge.location,
                ));
            };
            if actual != edge.expected_target_kind {
                let target_location = if let Some(location) = asset_locations.get(&target_asset_id)
                {
                    location.clone()
                } else {
                    let target_location = external_target
                        .map(ExternalCatalogAssetV2::location)
                        .or_else(|| {
                            visible_target.map(crate::ResolvedBindingV2::declaration_location)
                        })
                        .ok_or_else(|| {
                            ModuleDiagnosticV2::new(
                                "modules.unknown_catalog_reference",
                                "document dependency target is absent",
                                &edge.location,
                            )
                        })?;
                    source_registry
                        .try_check_location(target_location)
                        .map_err(|_source_error| {
                            ModuleDiagnosticV2::without_location(
                                "modules.source_registry_mismatch",
                                "catalog target location lost registry proof",
                            )
                        })?
                };
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.catalog_reference_kind_mismatch",
                    "document dependency target has wrong kind",
                    &edge.location,
                    &target_location,
                ));
            }
            let key = (source_asset_id.clone(), target_asset_id.clone());
            let resolved_edge = DocumentCatalogDependencyV2 {
                source_asset_id,
                target_asset_id,
                expected_target_kind: edge.expected_target_kind,
                family: None,
                location: edge.location,
            };
            match dependency_edges.get_mut(&key) {
                Some(existing)
                    if existing.expected_target_kind != resolved_edge.expected_target_kind
                        || existing.family != resolved_edge.family =>
                {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.catalog_dependency_kind_conflict",
                        "duplicate edge disagrees on expected kind",
                        &resolved_edge.location,
                    ))
                }
                Some(existing)
                    if loc_key(resolved_edge.location.location())
                        < loc_key(existing.location.location()) =>
                {
                    *existing = resolved_edge
                }
                Some(_) => {}
                None => {
                    dependency_edges.insert(key, resolved_edge);
                }
            }
        }
        for reference in input.family_references {
            session.charge_edge()?;
            ensure_registry(&reference.location, source_registry)?;
            ensure_registry(&reference.source.location, source_registry)?;
            ensure_registry(&reference.target.location, source_registry)?;
            if !reference.source.belongs_to(session_key)
                || !reference.target.belongs_to(session_key)
            {
                return Err(ModuleDiagnosticV2::new(
                    "modules.catalog_reference_session_mismatch",
                    "family reference proof belongs to another document validator",
                    &reference.location,
                ));
            }
            let source_id = reference.source.id.clone();
            if predeclared.get(&source_id) != Some(reference.family.source_kind())
                || predeclared.location(&source_id) != Some(&reference.source.location)
                || reference.source.kind != reference.family.source_kind()
            {
                return Err(ModuleDiagnosticV2::new(
                    "modules.catalog_dependency_source_wrong_kind",
                    "family reference source is not the matching predeclared local asset",
                    &reference.location,
                ));
            }
            if reference.target.kind != reference.family.target_kind() {
                return Err(ModuleDiagnosticV2::new(
                    "modules.catalog_reference_kind_mismatch",
                    "family reference target kind differs from required kind",
                    &reference.location,
                ));
            }
            let target_id = reference.target.id.clone();
            let target_exists = predeclared.get(&target_id) == Some(reference.target.kind)
                || external
                    .asset(&target_id)
                    .is_some_and(|asset| asset.kind() == reference.target.kind)
                || visible.visible_bindings().iter().any(|binding| {
                    binding.resolved_asset_id() == &target_id
                        && binding.namespace_kind() == reference.target.kind
                });
            if !target_exists {
                return Err(ModuleDiagnosticV2::new(
                    "modules.unknown_catalog_reference",
                    "family reference target is absent from sealed document or host authority",
                    &reference.location,
                ));
            }
            let key = (source_id.clone(), target_id.clone());
            let edge = DocumentCatalogDependencyV2 {
                source_asset_id: source_id,
                target_asset_id: target_id,
                expected_target_kind: reference.family.target_kind(),
                family: Some(reference.family),
                location: reference.location,
            };
            match dependency_edges.get_mut(&key) {
                Some(existing)
                    if existing.expected_target_kind != edge.expected_target_kind
                        || existing.family != edge.family =>
                {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.catalog_dependency_kind_conflict",
                        "duplicate edge disagrees on expected kind",
                        &edge.location,
                    ));
                }
                Some(existing)
                    if loc_key(edge.location.location())
                        < loc_key(existing.location.location()) =>
                {
                    *existing = edge;
                }
                Some(_) => {}
                None => {
                    dependency_edges.insert(key, edge);
                }
            }
        }
        if let Some(cycle) = document_cycle(&dependency_edges, &assets, session)? {
            let witness = cycle.iter().map(|location| location.location()).collect();
            let Some(first) = cycle.first() else {
                return Err(ModuleDiagnosticV2::without_location(
                    "modules.view_inline_cycle",
                    "empty document View cycle witness",
                ));
            };
            return Err(ModuleDiagnosticV2::new(
                "modules.view_inline_cycle",
                "document View dependency cycle",
                first,
            )
            .with_witness(witness));
        }
        let mut require_location: Option<CheckedLocatedSpanV2> = None;
        for directive in input.require_certified_directives {
            session.charge_edge()?;
            ensure_registry(&directive.location, source_registry)?;
            if require_location.as_ref().map_or(true, |current| {
                loc_key(directive.location.location()) < loc_key(current.location())
            }) {
                require_location = Some(directive.location);
            }
        }
        if let Some(require) = &require_location {
            for (id, admission) in &profiles {
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
            let reachable = reachable_external_profiles(&dependency_edges, external, &activations);
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
            assets,
            profiles,
            dependencies: dependency_edges.into_values().collect(),
            activations,
            require_certified: require_location.map(|location| location.location()),
            registry_identity: source_registry.identity(),
        })
    }
    pub fn assets(&self) -> impl Iterator<Item = (&CatalogAssetIdV2, NamespaceKindV2)> {
        self.assets.iter().map(|(id, kind)| (id, *kind))
    }
    #[must_use]
    pub const fn activations(&self) -> &crate::ValidatedPackActivationsV2 {
        &self.activations
    }
    pub fn profiles(&self) -> impl Iterator<Item = (&CatalogAssetIdV2, ProfileAdmissionV2)> {
        self.profiles.iter().map(|(id, admission)| (id, *admission))
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

fn document_cycle(
    edges: &BTreeMap<(CatalogAssetIdV2, CatalogAssetIdV2), DocumentCatalogDependencyV2>,
    assets: &BTreeMap<CatalogAssetIdV2, NamespaceKindV2>,
    session: &mut ModuleBudgetSessionV2,
) -> Result<Option<Vec<CheckedLocatedSpanV2>>, ModuleDiagnosticV2> {
    let mut adjacency: BTreeMap<CatalogAssetIdV2, Vec<&DocumentCatalogDependencyV2>> =
        BTreeMap::new();
    for edge in edges.values() {
        if assets.get(&edge.source_asset_id) == Some(&NamespaceKindV2::View)
            && assets.get(&edge.target_asset_id) == Some(&NamespaceKindV2::View)
        {
            adjacency
                .entry(edge.source_asset_id.clone())
                .or_default()
                .push(edge);
        }
    }
    for outgoing in adjacency.values_mut() {
        outgoing.sort_by(|left, right| {
            left.target_asset_id
                .cmp(&right.target_asset_id)
                .then(loc_key(left.location.location()).cmp(&loc_key(right.location.location())))
        });
    }
    let mut cycles = Vec::new();
    for root in adjacency.keys() {
        let mut stack = vec![(
            root.clone(),
            Vec::<(CatalogAssetIdV2, CheckedLocatedSpanV2)>::new(),
            std::collections::BTreeSet::from([root.clone()]),
        )];
        while let Some((node, path, seen)) = stack.pop() {
            for edge in adjacency.get(&node).into_iter().flatten().rev() {
                session.charge_step()?;
                if seen.contains(&edge.target_asset_id) {
                    let position = path
                        .iter()
                        .position(|(id, _)| id == &edge.target_asset_id)
                        .map_or(0, |index| index + 1);
                    let mut cycle = path.get(position..).map_or_else(Vec::new, <[_]>::to_vec);
                    cycle.push((edge.target_asset_id.clone(), edge.location.clone()));
                    cycle = canonicalize_document_cycle(cycle);
                    cycles.push(cycle);
                } else {
                    let mut next_path = path.clone();
                    next_path.push((edge.target_asset_id.clone(), edge.location.clone()));
                    let mut next_seen = seen.clone();
                    next_seen.insert(edge.target_asset_id.clone());
                    stack.push((edge.target_asset_id.clone(), next_path, next_seen));
                }
            }
        }
    }
    cycles.sort_by(|left, right| {
        left.iter()
            .map(|(id, loc)| (id.clone(), loc_key(loc.location())))
            .collect::<Vec<_>>()
            .cmp(
                &right
                    .iter()
                    .map(|(id, loc)| (id.clone(), loc_key(loc.location())))
                    .collect::<Vec<_>>(),
            )
    });
    Ok(cycles
        .into_iter()
        .next()
        .map(|cycle| cycle.into_iter().map(|(_, location)| location).collect()))
}

fn canonicalize_catalog_cycle(
    cycle: Vec<(CatalogAssetIdV2, LocatedSpanV2)>,
) -> Vec<(CatalogAssetIdV2, LocatedSpanV2)> {
    (0..cycle.len())
        .map(|offset| {
            cycle
                .iter()
                .cycle()
                .skip(offset)
                .take(cycle.len())
                .cloned()
                .collect::<Vec<_>>()
        })
        .min_by_key(|rotation| {
            rotation
                .iter()
                .map(|(id, location)| (id.clone(), loc_key(*location)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn canonicalize_document_cycle(
    cycle: Vec<(CatalogAssetIdV2, CheckedLocatedSpanV2)>,
) -> Vec<(CatalogAssetIdV2, CheckedLocatedSpanV2)> {
    (0..cycle.len())
        .map(|offset| {
            cycle
                .iter()
                .cycle()
                .skip(offset)
                .take(cycle.len())
                .cloned()
                .collect::<Vec<_>>()
        })
        .min_by_key(|rotation| {
            rotation
                .iter()
                .map(|(id, location)| (id.clone(), loc_key(location.location())))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}
fn reachable_external_profiles<'a>(
    edges: &BTreeMap<(CatalogAssetIdV2, CatalogAssetIdV2), DocumentCatalogDependencyV2>,
    external: &'a ExternalCatalogV2,
    activations: &crate::ValidatedPackActivationsV2,
) -> Vec<&'a ExternalCatalogAssetV2> {
    let mut queue = edges
        .values()
        .map(|edge| edge.target_asset_id.clone())
        .collect::<Vec<_>>();
    for activation in activations.iter() {
        queue.extend(external.iter().filter_map(|asset| match asset.payload() {
            ExternalCatalogPayloadV2::Pack(contract)
                if contract.pack_id() == activation.pack_id() =>
            {
                Some(asset.id().clone())
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
    let mut seen = std::collections::BTreeSet::new();
    let mut profiles = Vec::new();
    while let Some(id) = queue.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Some(asset) = external.asset(&id) {
            if asset.kind == NamespaceKindV2::Profile {
                profiles.push(asset);
            }
            queue.extend(
                asset
                    .dependencies
                    .iter()
                    .map(|edge| edge.target_asset_id.clone()),
            );
        }
    }
    profiles.sort_by_key(|asset| asset.id.clone());
    profiles
}
