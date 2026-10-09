//! Canonical lockfile verification and validated pack activations.

use std::collections::BTreeMap;

use crate::budget::ModuleBudgetSessionV2;
use crate::graph::excerpt;
use crate::manifest::{ContentDigestV2, PackIdV2, ValidatedPackGraphV2, ValidatedPackManifestV2};
use crate::version::VersionV2;
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, LocatedSpanV2, SourceIdV2, SourceRegistryIdentityV2,
    SourceRegistryViewV2, SpanV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockReferenceInputV2 {
    pub path: String,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockReferenceV2 {
    path: String,
    location: CheckedLocatedSpanV2,
}
impl LockReferenceV2 {
    pub(crate) fn try_new(
        input: LockReferenceInputV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        session.charge_string(&input.path)?;
        Self::try_new_precharged(input)
    }
    pub(crate) fn try_new_precharged(
        input: LockReferenceInputV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if !canonical_lock_path(&input.path) {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_lockfile_path",
                format!("invalid lock reference `{}`", excerpt(&input.path)),
                &input.location,
            ));
        }
        Ok(Self {
            path: input.path,
            location: input.location,
        })
    }
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}
fn canonical_lock_path(path: &str) -> bool {
    !path.is_empty()
        && path.ends_with(".lock")
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.contains('\0')
        && path
            .split('/')
            .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LockPinV2 {
    pack_id: PackIdV2,
    version: VersionV2,
    content_digest: ContentDigestV2,
    location: CheckedLocatedSpanV2,
}
impl LockPinV2 {
    #[must_use]
    pub const fn pack_id(&self) -> &PackIdV2 {
        &self.pack_id
    }
    #[must_use]
    pub const fn version(&self) -> VersionV2 {
        self.version
    }
    #[must_use]
    pub const fn content_digest(&self) -> ContentDigestV2 {
        self.content_digest
    }
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }
}

fn canonical_lock_text(pins: &BTreeMap<PackIdV2, LockPinV2>) -> String {
    let mut output = String::new();
    for pin in pins.values() {
        output.push_str(pin.pack_id.as_str());
        output.push(' ');
        output.push_str(&pin.version.to_string());
        output.push(' ');
        output.push_str(&pin.content_digest.to_hex());
        output.push('\n');
    }
    output
}

#[derive(Clone, Debug)]
pub struct LockfileV2 {
    pins: BTreeMap<PackIdV2, LockPinV2>,
    source_id: SourceIdV2,
    registry_identity: SourceRegistryIdentityV2,
    reference: Option<LockReferenceV2>,
}
impl LockfileV2 {
    pub(crate) fn empty(source_id: SourceIdV2, source_registry: &SourceRegistryViewV2) -> Self {
        Self {
            pins: BTreeMap::new(),
            source_id,
            registry_identity: source_registry.identity(),
            reference: None,
        }
    }
    pub(crate) fn parse(
        source_id: SourceIdV2,
        text: &str,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        // Raw bytes were charged before mutation by ModuleAuthorityBuilderV2.
        if source_registry.source_len(source_id) != Some(text.len()) {
            let diagnostic = source_registry
                .source_len(source_id)
                .and_then(|registered_len| {
                    source_registry
                        .try_check_location(LocatedSpanV2 {
                            source_id,
                            span: SpanV1::new(0, registered_len),
                        })
                        .ok()
                        .map(|location| {
                            ModuleDiagnosticV2::new(
                                "modules.lock_source_length_mismatch",
                                "registered lock source length differs from submitted bytes",
                                &location,
                            )
                        })
                });
            return Err(diagnostic.unwrap_or_else(|| {
                ModuleDiagnosticV2::without_location(
                    "modules.lock_source_length_mismatch",
                    "registered lock source length differs from submitted bytes",
                )
            }));
        }
        if !text.is_empty() && !text.ends_with('\n') {
            let location = source_registry
                .try_check_location(LocatedSpanV2 {
                    source_id,
                    span: SpanV1::new(text.len(), text.len()),
                })
                .map_err(|_source_error| {
                    ModuleDiagnosticV2::without_location(
                        "modules.source_registry_mismatch",
                        "lock EOF location is outside registered source",
                    )
                })?;
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_lock_syntax",
                "lock text must end with LF",
                &location,
            ));
        }
        let mut pins: BTreeMap<PackIdV2, LockPinV2> = BTreeMap::new();
        let mut parsed_order = Vec::new();
        let mut offset = 0usize;
        for line_with_lf in text.split_inclusive('\n') {
            let line = &line_with_lf[..line_with_lf.len() - 1];
            let raw = LocatedSpanV2 {
                source_id,
                span: SpanV1::new(offset, offset + line.len()),
            };
            let location = source_registry
                .try_check_location(raw)
                .map_err(|_source_error| {
                    ModuleDiagnosticV2::without_location(
                        "modules.invalid_lock_syntax",
                        "lock line location is outside registered source",
                    )
                })?;
            if line.is_empty() || line.contains('\r') || line.contains('\t') {
                return Err(ModuleDiagnosticV2::new(
                    "modules.invalid_lock_syntax",
                    "lock line contains forbidden whitespace",
                    &location,
                ));
            }
            // Read at most four fields: malformed whitespace must not allocate
            // a slice vector proportional to the submitted line width.
            let mut fields = line.split(' ');
            let (pack_text, version_text, digest_text) =
                match (fields.next(), fields.next(), fields.next(), fields.next()) {
                    (Some(pack), Some(version), Some(digest), None)
                        if !pack.is_empty() && !version.is_empty() && !digest.is_empty() =>
                    {
                        (pack, version, digest)
                    }
                    _ => {
                        return Err(ModuleDiagnosticV2::new(
                            "modules.invalid_lock_syntax",
                            "lock line must contain exactly three single-space-separated fields",
                            &location,
                        ));
                    }
                };
            session.check_string_width(pack_text)?;
            session.check_string_width(version_text)?;
            session.check_string_width(digest_text)?;
            let pack_id = PackIdV2::try_new(pack_text, location.clone())?;
            let version = VersionV2::try_parse(version_text, location.clone())?;
            let content_digest = ContentDigestV2::try_parse(digest_text, location.clone())?;
            session.charge_node()?;
            parsed_order.push((pack_text, location.clone()));
            if let Some(original) = pins.get(&pack_id) {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_lock_pin",
                    "duplicate lock pin",
                    &location,
                    &original.location,
                ));
            }
            pins.insert(
                pack_id.clone(),
                LockPinV2 {
                    pack_id,
                    version,
                    content_digest,
                    location,
                },
            );
            offset += line_with_lf.len();
        }
        let lock = Self {
            pins,
            source_id,
            registry_identity: source_registry.identity(),
            reference: None,
        };
        // Each field parser already requires its canonical spelling. Unique
        // pins have the same count as parsed_order, so only ordering remains.
        // Compare after all lines validate to preserve later field/duplicate/
        // budget errors and the first mismatching line's primary location.
        if let Some(((_, primary), _)) = parsed_order
            .iter()
            .zip(lock.pins.keys())
            .find(|((actual, _), canonical)| *actual != canonical.as_str())
        {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_lock_syntax",
                "lock entries are not in canonical sorted order",
                primary,
            ));
        }
        Ok(lock)
    }

    pub(crate) fn with_reference(
        mut self,
        reference: LockReferenceV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if reference.location.registry_identity() != &self.registry_identity {
            return Err(ModuleDiagnosticV2::new(
                "modules.source_registry_mismatch",
                "lock reference belongs to another registry",
                &reference.location,
            ));
        }
        self.reference = Some(reference);
        Ok(self)
    }

    #[must_use]
    pub fn to_text(&self) -> String {
        canonical_lock_text(&self.pins)
    }

    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.source_id
    }

    pub(crate) fn verify_against(
        &self,
        graph: &ValidatedPackGraphV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<VerifiedLockSetV2, ModuleDiagnosticV2> {
        if &self.registry_identity != graph.registry_identity() {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.source_registry_mismatch",
                "lock and pack graph registry identities differ",
            ));
        }
        for (pack_id, manifest) in graph.manifests() {
            session.charge_step()?;
            let Some(pin) = self.pins.get(pack_id) else {
                return Err(ModuleDiagnosticV2::new(
                    "modules.lock_mismatch",
                    "manifest has no lock pin",
                    manifest.location(),
                ));
            };
            if pin.version != manifest.coordinate().version()
                || pin.content_digest != manifest.content_digest()
            {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.lock_mismatch",
                    "lock version or digest differs from manifest",
                    &pin.location,
                    manifest.location(),
                ));
            }
        }
        for pin in self.pins.values() {
            session.charge_step()?;
            if graph.raw_manifest(&pin.pack_id).is_none() {
                return Err(ModuleDiagnosticV2::new(
                    "modules.lock_mismatch",
                    "lock pins an absent manifest",
                    &pin.location,
                ));
            }
        }
        Ok(VerifiedLockSetV2 {
            pins: self.pins.clone(),
            fingerprint: *graph.fingerprint(),
            registry_identity: self.registry_identity.clone(),
            reference: self.reference.clone(),
        })
    }
}

#[derive(Clone, Debug)]
pub struct VerifiedLockSetV2 {
    pins: BTreeMap<PackIdV2, LockPinV2>,
    fingerprint: [u8; 32],
    registry_identity: SourceRegistryIdentityV2,
    reference: Option<LockReferenceV2>,
}
impl VerifiedLockSetV2 {
    #[must_use]
    pub const fn fingerprint(&self) -> &[u8; 32] {
        &self.fingerprint
    }
    #[must_use]
    pub const fn registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.registry_identity
    }
    #[must_use]
    pub const fn reference(&self) -> Option<&LockReferenceV2> {
        self.reference.as_ref()
    }
    #[must_use]
    pub fn pin(&self, id: &PackIdV2) -> Option<&LockPinV2> {
        self.pins.get(id)
    }
    #[must_use]
    pub fn to_text(&self) -> String {
        canonical_lock_text(&self.pins)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CatalogPackActivationV2 {
    Manual,
    Session,
    Workspace,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsePackActivationV2 {
    Session,
    Workspace,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackActivationInputV2 {
    pub pack_id: PackIdV2,
    pub activation: UsePackActivationV2,
    pub lock_reference: LockReferenceInputV2,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedPackActivationV2 {
    pack_id: PackIdV2,
    activation: UsePackActivationV2,
    location: LocatedSpanV2,
}
impl ValidatedPackActivationV2 {
    #[must_use]
    pub const fn pack_id(&self) -> &PackIdV2 {
        &self.pack_id
    }
    #[must_use]
    pub const fn activation(&self) -> UsePackActivationV2 {
        self.activation
    }
    #[must_use]
    pub const fn location(&self) -> LocatedSpanV2 {
        self.location
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidatedPackActivationsV2 {
    activations: Vec<(ValidatedPackActivationV2, CheckedLocatedSpanV2)>,
}
impl ValidatedPackActivationsV2 {
    fn empty() -> Self {
        Self {
            activations: Vec::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ValidatedPackActivationV2> {
        self.activations.iter().map(|(activation, _)| activation)
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedPackAuthorityV2 {
    manifests: BTreeMap<PackIdV2, ValidatedPackManifestV2>,
    graph_fingerprint: [u8; 32],
    lock: VerifiedLockSetV2,
    registry_identity: SourceRegistryIdentityV2,
}
impl ValidatedPackAuthorityV2 {
    pub(crate) fn try_new(
        graph: ValidatedPackGraphV2,
        lock: VerifiedLockSetV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if graph.fingerprint() != lock.fingerprint()
            || graph.registry_identity() != lock.registry_identity()
        {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.pack_lock_proof_mismatch",
                "pack graph and verified lock proof differ",
            ));
        }
        let manifests = graph
            .manifests()
            .map(|(id, manifest)| {
                (
                    id.clone(),
                    ValidatedPackManifestV2::from_raw(manifest.clone()),
                )
            })
            .collect();
        Ok(Self {
            manifests,
            graph_fingerprint: *graph.fingerprint(),
            registry_identity: graph.registry_identity().clone(),
            lock,
        })
    }
    #[must_use]
    pub fn manifest(&self, id: &PackIdV2) -> Option<&ValidatedPackManifestV2> {
        self.manifests.get(id)
    }
    #[must_use]
    pub const fn registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.registry_identity
    }
    #[must_use]
    pub const fn graph_fingerprint(&self) -> &[u8; 32] {
        &self.graph_fingerprint
    }
    #[must_use]
    pub const fn verified_lock(&self) -> &VerifiedLockSetV2 {
        &self.lock
    }
    #[expect(
        dead_code,
        reason = "pinned host-phase API; document callers use the combined-registry path"
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "binary_search returns an in-range duplicate index"
    )]
    pub(crate) fn validate_activations(
        &self,
        inputs: Vec<PackActivationInputV2>,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<ValidatedPackActivationsV2, ModuleDiagnosticV2> {
        if source_registry.identity() != self.registry_identity {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.source_registry_mismatch",
                "activation registry differs from pack authority",
            ));
        }
        if inputs.is_empty() {
            return Ok(ValidatedPackActivationsV2::empty());
        }
        let expected_reference = self.lock.reference().ok_or_else(|| {
            ModuleDiagnosticV2::without_location(
                "modules.missing_lockfile",
                "pack authority has no lock reference",
            )
        })?;
        let mut activations = Vec::with_capacity(inputs.len());
        for input in inputs {
            session.charge_edge()?;
            session.charge_string(input.pack_id.as_str())?;
            if input.location.registry_identity() != &self.registry_identity {
                return Err(ModuleDiagnosticV2::new(
                    "modules.source_registry_mismatch",
                    "activation location belongs to another registry",
                    &input.location,
                ));
            }
            let reference = LockReferenceV2::try_new(input.lock_reference, session)?;
            let insertion = match activations.binary_search_by(
                |(activation, _): &(ValidatedPackActivationV2, CheckedLocatedSpanV2)| {
                    activation.pack_id.cmp(&input.pack_id)
                },
            ) {
                Ok(index) => {
                    return Err(ModuleDiagnosticV2::conflict(
                        "modules.duplicate_pack_activation",
                        "duplicate pack activation",
                        &input.location,
                        &activations[index].1,
                    ));
                }
                Err(index) => index,
            };
            if reference.path() != expected_reference.path() {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.lock_reference_mismatch",
                    "activation lock reference differs from sealed authority",
                    reference.location(),
                    expected_reference.location(),
                ));
            }
            if !self.manifests.contains_key(&input.pack_id) {
                return Err(ModuleDiagnosticV2::new(
                    "modules.unknown_pack",
                    "activation names an unknown pack",
                    &input.location,
                ));
            }
            activations.insert(
                insertion,
                (
                    ValidatedPackActivationV2 {
                        pack_id: input.pack_id,
                        activation: input.activation,
                        location: input.location.location(),
                    },
                    input.location,
                ),
            );
        }
        Ok(ValidatedPackActivationsV2 { activations })
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "binary_search returns an in-range duplicate index"
    )]
    pub(crate) fn validate_document_activations(
        &self,
        inputs: Vec<PackActivationInputV2>,
        combined_identity: &SourceRegistryIdentityV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<ValidatedPackActivationsV2, ModuleDiagnosticV2> {
        if inputs.is_empty() {
            return Ok(ValidatedPackActivationsV2::empty());
        }
        let expected_reference = self.lock.reference().ok_or_else(|| {
            ModuleDiagnosticV2::without_location(
                "modules.missing_lockfile",
                "pack authority has no lock reference",
            )
        })?;
        let mut activations = Vec::with_capacity(inputs.len());
        for input in inputs {
            session.charge_edge()?;
            session.charge_string(input.pack_id.as_str())?;
            if input.location.registry_identity() != combined_identity
                || input.lock_reference.location.registry_identity() != combined_identity
            {
                return Err(ModuleDiagnosticV2::new(
                    "modules.source_registry_mismatch",
                    "activation location belongs to another document registry",
                    &input.location,
                ));
            }
            let reference = LockReferenceV2::try_new(input.lock_reference, session)?;
            let insertion = match activations.binary_search_by(
                |(activation, _): &(ValidatedPackActivationV2, CheckedLocatedSpanV2)| {
                    activation.pack_id.cmp(&input.pack_id)
                },
            ) {
                Ok(index) => {
                    return Err(ModuleDiagnosticV2::conflict(
                        "modules.duplicate_pack_activation",
                        "duplicate pack activation",
                        &input.location,
                        &activations[index].1,
                    ));
                }
                Err(index) => index,
            };
            if reference.path() != expected_reference.path() {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.lock_reference_mismatch",
                    "activation lock reference differs from sealed authority",
                    reference.location(),
                    expected_reference.location(),
                ));
            }
            if !self.manifests.contains_key(&input.pack_id) {
                return Err(ModuleDiagnosticV2::new(
                    "modules.unknown_pack",
                    "activation names an unknown pack",
                    &input.location,
                ));
            }
            activations.insert(
                insertion,
                (
                    ValidatedPackActivationV2 {
                        pack_id: input.pack_id,
                        activation: input.activation,
                        location: input.location.location(),
                    },
                    input.location,
                ),
            );
        }
        Ok(ValidatedPackActivationsV2 { activations })
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "owner assertions")]
mod tests {
    use super::*;
    use crate::{ModuleBudgetV2, PackGraphV2};
    use quanta_gqlang_v2_hir::SourceRegistrationV2;
    use quanta_gqlang_v2_types::AuthorityBudgetProfileV2;

    #[test]
    fn pack_lock_proof_mismatch_rejects_before_sealing() {
        let source_registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(1),
            source_len: 1,
        }])
        .expect("test registry must be valid");
        let budget = ModuleBudgetV2::from_profile(
            AuthorityBudgetProfileV2::try_new(8, 8, 32, 256, 64)
                .expect("test budget must be valid"),
        );
        let mut session = budget.begin_session_v2();
        let graph = PackGraphV2::empty(&source_registry)
            .validate(VersionV2::new(1, 0, 0), &mut session)
            .expect("empty graph must validate");
        let mismatched_lock = VerifiedLockSetV2 {
            pins: BTreeMap::new(),
            fingerprint: [0xff; 32],
            registry_identity: source_registry.identity(),
            reference: None,
        };
        let error = ValidatedPackAuthorityV2::try_new(graph, mismatched_lock)
            .expect_err("mismatched proofs must reject");
        assert_eq!(error.code(), "modules.pack_lock_proof_mismatch");
    }
}
