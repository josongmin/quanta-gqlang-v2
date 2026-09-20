//! Checked pack manifests, dependency graphs, and canonical fingerprints.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::budget::ModuleBudgetSessionV2;
use crate::graph::{excerpt, is_segment, loc_key};
use crate::version::{VersionRangeV2, VersionV2};
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, SourceRegistryIdentityV2, SourceRegistryViewV2};
use sha2::{Digest, Sha256};

/// Canonical name-only pack identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackIdV2(String);

impl PackIdV2 {
    pub fn try_new(
        value: &str,
        location: CheckedLocatedSpanV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if !is_pack_name(value) {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_pack_id",
                format!("invalid pack id `{}`", excerpt(value)),
                &location,
            ));
        }
        Ok(Self(value.to_owned()))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn is_pack_name(value: &str) -> bool {
    let mut segments = value.split(['.', '-']);
    let Some(first) = segments.next() else {
        return false;
    };
    is_pack_segment(first, true) && segments.all(|segment| is_pack_segment(segment, false))
}
fn is_pack_segment(value: &str, first: bool) -> bool {
    if value.is_empty() {
        return false;
    }
    let mut bytes = value.bytes();
    let Some(head) = bytes.next() else {
        return false;
    };
    let head_ok = if first {
        matches!(head, b'a'..=b'z' | b'A'..=b'Z' | b'_')
    } else {
        head.is_ascii_alphanumeric() || head == b'_'
    };
    head_ok && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContentDigestV2([u8; 32]);
impl ContentDigestV2 {
    pub fn try_parse(
        value: &str,
        location: CheckedLocatedSpanV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if value.len() != 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
        {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_content_digest",
                "content digest must be 64 lowercase hexadecimal bytes",
                &location,
            ));
        }
        let mut bytes = [0u8; 32];
        for (index, chunk) in value.as_bytes().chunks_exact(2).enumerate() {
            let Some(high) = chunk.first().copied() else {
                return Err(ModuleDiagnosticV2::new(
                    "modules.invalid_content_digest",
                    "content digest contains an incomplete byte",
                    &location,
                ));
            };
            let Some(low) = chunk.get(1).copied() else {
                return Err(ModuleDiagnosticV2::new(
                    "modules.invalid_content_digest",
                    "content digest contains an incomplete byte",
                    &location,
                ));
            };
            let Some(slot) = bytes.get_mut(index) else {
                return Err(ModuleDiagnosticV2::new(
                    "modules.invalid_content_digest",
                    "content digest exceeds SHA-256 width",
                    &location,
                ));
            };
            *slot = (hex(high) << 4) | hex(low);
        }
        Ok(Self(bytes))
    }
    #[must_use]
    pub const fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0
            .iter()
            .fold(String::with_capacity(64), |mut text, byte| {
                if write!(text, "{byte:02x}").is_err() {
                    return text;
                }
                text
            })
    }
}
fn hex(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => 0,
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackCoordinateV2 {
    pack_id: PackIdV2,
    version: VersionV2,
}
impl PackCoordinateV2 {
    #[must_use]
    pub const fn pack_id(&self) -> &PackIdV2 {
        &self.pack_id
    }
    #[must_use]
    pub const fn version(&self) -> VersionV2 {
        self.version
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedDependencyInputV2 {
    pub pack_id: PackIdV2,
    pub version_range: VersionRangeV2,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedExportInputV2 {
    pub name: String,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedSuiteMemberInputV2 {
    pub name: String,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedSuiteInputV2 {
    pub name: String,
    pub members: Vec<LocatedSuiteMemberInputV2>,
    pub location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackManifestInputV2 {
    pub pack_id: PackIdV2,
    pub version: VersionV2,
    pub content_digest: ContentDigestV2,
    pub language_range: VersionRangeV2,
    pub dependencies: Vec<LocatedDependencyInputV2>,
    pub exports: Vec<LocatedExportInputV2>,
    pub suites: Vec<LocatedSuiteInputV2>,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DependencyV2 {
    pack_id: PackIdV2,
    range: VersionRangeV2,
    location: CheckedLocatedSpanV2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct SuiteV2 {
    members: BTreeMap<String, CheckedLocatedSpanV2>,
    location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackManifestV2 {
    coordinate: PackCoordinateV2,
    content_digest: ContentDigestV2,
    language_range: VersionRangeV2,
    dependencies: BTreeMap<PackIdV2, DependencyV2>,
    exports: BTreeMap<String, CheckedLocatedSpanV2>,
    suites: BTreeMap<String, SuiteV2>,
    location: CheckedLocatedSpanV2,
    registry_identity: SourceRegistryIdentityV2,
}

impl PackManifestV2 {
    pub(crate) fn try_new(
        input: PackManifestInputV2,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        check_registry(&input.location, source_registry)?;
        session.charge_node()?;
        session.charge_string(input.pack_id.as_str())?;
        let mut dependencies: BTreeMap<PackIdV2, DependencyV2> = BTreeMap::new();
        for dependency in input.dependencies {
            session.charge_edge()?;
            session.charge_string(dependency.pack_id.as_str())?;
            check_registry(&dependency.location, source_registry)?;
            check_registry(dependency.version_range.location(), source_registry)?;
            if let Some(original) = dependencies.get(&dependency.pack_id) {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_pack_dependency",
                    "duplicate pack dependency",
                    &dependency.location,
                    &original.location,
                ));
            }
            dependencies.insert(
                dependency.pack_id.clone(),
                DependencyV2 {
                    pack_id: dependency.pack_id,
                    range: dependency.version_range,
                    location: dependency.location,
                },
            );
        }
        let mut exports: BTreeMap<String, CheckedLocatedSpanV2> = BTreeMap::new();
        for export in input.exports {
            session.charge_edge()?;
            session.charge_string(&export.name)?;
            check_registry(&export.location, source_registry)?;
            if !is_segment(&export.name) {
                return Err(ModuleDiagnosticV2::new(
                    "modules.invalid_export",
                    "invalid pack export",
                    &export.location,
                ));
            }
            match exports.get_mut(&export.name) {
                Some(location)
                    if loc_key(export.location.location()) < loc_key(location.location()) =>
                {
                    *location = export.location
                }
                Some(_) => {}
                None => {
                    exports.insert(export.name, export.location);
                }
            }
        }
        let mut suites: BTreeMap<String, SuiteV2> = BTreeMap::new();
        for suite in input.suites {
            session.charge_node()?;
            session.charge_string(&suite.name)?;
            check_registry(&suite.location, source_registry)?;
            if !is_segment(&suite.name) {
                return Err(ModuleDiagnosticV2::new(
                    "modules.invalid_suite",
                    "invalid suite name",
                    &suite.location,
                ));
            }
            if let Some(original) = suites.get(&suite.name) {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_suite",
                    "duplicate suite declaration",
                    &suite.location,
                    &original.location,
                ));
            }
            let mut members: BTreeMap<String, CheckedLocatedSpanV2> = BTreeMap::new();
            for member in suite.members {
                session.charge_edge()?;
                session.charge_string(&member.name)?;
                check_registry(&member.location, source_registry)?;
                if !is_segment(&member.name) {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.invalid_suite_member",
                        "invalid suite member",
                        &member.location,
                    ));
                }
                match members.get_mut(&member.name) {
                    Some(location)
                        if loc_key(member.location.location()) < loc_key(location.location()) =>
                    {
                        *location = member.location
                    }
                    Some(_) => {}
                    None => {
                        members.insert(member.name, member.location);
                    }
                }
            }
            suites.insert(
                suite.name,
                SuiteV2 {
                    members,
                    location: suite.location,
                },
            );
        }
        Ok(Self {
            coordinate: PackCoordinateV2 {
                pack_id: input.pack_id,
                version: input.version,
            },
            content_digest: input.content_digest,
            language_range: input.language_range,
            dependencies,
            exports,
            suites,
            location: input.location,
            registry_identity: source_registry.identity(),
        })
    }
    #[must_use]
    pub const fn coordinate(&self) -> &PackCoordinateV2 {
        &self.coordinate
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

fn check_registry(
    location: &CheckedLocatedSpanV2,
    source_registry: &SourceRegistryViewV2,
) -> Result<(), ModuleDiagnosticV2> {
    if location.registry_identity() != &source_registry.identity() {
        return Err(ModuleDiagnosticV2::new(
            "modules.source_registry_mismatch",
            "location belongs to another source registry",
            location,
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct PackGraphV2 {
    manifests: BTreeMap<PackIdV2, PackManifestV2>,
    registry_identity: SourceRegistryIdentityV2,
}
impl PackGraphV2 {
    pub(crate) fn try_new(
        manifests: Vec<PackManifestV2>,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        let Some(identity) = manifests
            .first()
            .map(|manifest| manifest.registry_identity.clone())
        else {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.internal_empty_pack_graph",
                "empty pack graph must be constructed with registry context",
            ));
        };
        let mut map: BTreeMap<PackIdV2, PackManifestV2> = BTreeMap::new();
        for manifest in manifests {
            session.charge_node()?;
            if manifest.registry_identity != identity {
                return Err(ModuleDiagnosticV2::new(
                    "modules.source_registry_mismatch",
                    "manifest belongs to another source registry",
                    &manifest.location,
                ));
            }
            if let Some(original) = map.get(manifest.coordinate.pack_id()) {
                return Err(ModuleDiagnosticV2::conflict(
                    "modules.duplicate_pack",
                    "duplicate pack name",
                    &manifest.location,
                    &original.location,
                ));
            }
            map.insert(manifest.coordinate.pack_id.clone(), manifest);
        }
        Ok(Self {
            manifests: map,
            registry_identity: identity,
        })
    }

    pub(crate) fn empty(source_registry: &SourceRegistryViewV2) -> Self {
        Self {
            manifests: BTreeMap::new(),
            registry_identity: source_registry.identity(),
        }
    }

    pub(crate) fn validate(
        self,
        language_version: VersionV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<ValidatedPackGraphV2, ModuleDiagnosticV2> {
        for manifest in self.manifests.values() {
            session.charge_step()?;
            if !manifest.language_range.contains(language_version) {
                return Err(ModuleDiagnosticV2::new(
                    "modules.compat_mismatch",
                    "pack language range excludes frontend version",
                    &manifest.location,
                ));
            }
            for dependency in manifest.dependencies.values() {
                session.charge_step()?;
                let Some(target) = self.manifests.get(&dependency.pack_id) else {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.unknown_pack",
                        "pack dependency is absent",
                        &dependency.location,
                    ));
                };
                if !dependency.range.contains(target.coordinate.version) {
                    return Err(ModuleDiagnosticV2::conflict(
                        "modules.dependency_unsatisfied",
                        "pack dependency version is outside requested range",
                        &dependency.location,
                        &target.location,
                    ));
                }
            }
        }
        if let Some(witness) = pack_cycle(&self.manifests, session)? {
            let Some(first) = witness.first() else {
                return Err(ModuleDiagnosticV2::without_location(
                    "modules.pack_dependency_cycle",
                    "empty pack dependency cycle witness",
                ));
            };
            return Err(ModuleDiagnosticV2::new(
                "modules.pack_dependency_cycle",
                "pack dependency cycle",
                &first.2,
            )
            .with_witness(witness.iter().map(|item| item.2.location()).collect()));
        }
        let mut owners: BTreeMap<&str, (&PackIdV2, &CheckedLocatedSpanV2)> = BTreeMap::new();
        for manifest in self.manifests.values() {
            for (export, location) in &manifest.exports {
                if let Some((owner, original)) =
                    owners.insert(export, (&manifest.coordinate.pack_id, location))
                {
                    return Err(ModuleDiagnosticV2::conflict(
                        "modules.overlapping_pack_export",
                        format!(
                            "pack export `{}` overlaps `{}`",
                            excerpt(export),
                            excerpt(owner.as_str())
                        ),
                        location,
                        original,
                    ));
                }
            }
        }
        let fingerprint = graph_fingerprint(&self.manifests)?;
        Ok(ValidatedPackGraphV2 {
            manifests: self.manifests,
            fingerprint,
            registry_identity: self.registry_identity,
        })
    }
}

type PackCycleEdgeV2 = (PackIdV2, PackIdV2, CheckedLocatedSpanV2);

fn pack_cycle(
    manifests: &BTreeMap<PackIdV2, PackManifestV2>,
    session: &mut ModuleBudgetSessionV2,
) -> Result<Option<Vec<PackCycleEdgeV2>>, ModuleDiagnosticV2> {
    let mut cycles = Vec::new();
    for root in manifests.keys() {
        let mut stack = vec![(root.clone(), Vec::new(), BTreeSet::from([root.clone()]))];
        while let Some((node, path, seen)) = stack.pop() {
            let Some(manifest) = manifests.get(&node) else {
                continue;
            };
            for dependency in manifest.dependencies.values().rev() {
                session.charge_step()?;
                let edge = (
                    node.clone(),
                    dependency.pack_id.clone(),
                    dependency.location.clone(),
                );
                if seen.contains(&dependency.pack_id) {
                    let position = path
                        .iter()
                        .position(|item: &PackCycleEdgeV2| item.0 == dependency.pack_id)
                        .unwrap_or(0);
                    let mut cycle = path.get(position..).map_or_else(Vec::new, <[_]>::to_vec);
                    cycle.push(edge);
                    cycles.push(canonical_pack_cycle(cycle));
                } else if !seen.contains(&dependency.pack_id) {
                    let mut next_path = path.clone();
                    next_path.push(edge);
                    let mut next_seen = seen.clone();
                    next_seen.insert(dependency.pack_id.clone());
                    stack.push((dependency.pack_id.clone(), next_path, next_seen));
                }
            }
        }
    }
    cycles.sort_by_key(|left| pack_cycle_key(left));
    Ok(cycles.into_iter().next())
}
fn canonical_pack_cycle(cycle: Vec<PackCycleEdgeV2>) -> Vec<PackCycleEdgeV2> {
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
        .min_by_key(|candidate| pack_cycle_key(candidate))
        .unwrap_or_default()
}
fn pack_cycle_key(cycle: &[PackCycleEdgeV2]) -> Vec<(PackIdV2, PackIdV2, (u32, usize, usize))> {
    cycle
        .iter()
        .map(|(a, b, l)| (a.clone(), b.clone(), loc_key(l.location())))
        .collect()
}

fn graph_fingerprint(
    manifests: &BTreeMap<PackIdV2, PackManifestV2>,
) -> Result<[u8; 32], ModuleDiagnosticV2> {
    let mut hasher = Sha256::new();
    hasher.update(b"qglang.pack-graph.v2\0");
    put_len(&mut hasher, manifests.len())?;
    for manifest in manifests.values() {
        put_text(&mut hasher, "pack")?;
        put_text(&mut hasher, manifest.coordinate.pack_id.as_str())?;
        put_version(&mut hasher, manifest.coordinate.version);
        hasher.update(manifest.content_digest.bytes());
        put_version(&mut hasher, manifest.language_range.lower());
        put_version(&mut hasher, manifest.language_range.upper());
        put_len(&mut hasher, manifest.dependencies.len())?;
        for dependency in manifest.dependencies.values() {
            put_text(&mut hasher, "dependency")?;
            put_text(&mut hasher, dependency.pack_id.as_str())?;
            put_version(&mut hasher, dependency.range.lower());
            put_version(&mut hasher, dependency.range.upper());
        }
        put_len(&mut hasher, manifest.exports.len())?;
        for export in manifest.exports.keys() {
            put_text(&mut hasher, "export")?;
            put_text(&mut hasher, export)?;
        }
        put_len(&mut hasher, manifest.suites.len())?;
        for (name, suite) in &manifest.suites {
            put_text(&mut hasher, "suite")?;
            put_text(&mut hasher, name)?;
            put_len(&mut hasher, suite.members.len())?;
            for member in suite.members.keys() {
                put_text(&mut hasher, "suite-member")?;
                put_text(&mut hasher, member)?;
            }
        }
    }
    Ok(hasher.finalize().into())
}
fn put_u64(hasher: &mut Sha256, value: u64) {
    hasher.update(value.to_be_bytes());
}
fn put_len(hasher: &mut Sha256, value: usize) -> Result<(), ModuleDiagnosticV2> {
    let value = u64::try_from(value).map_err(|_conversion_error| {
        ModuleDiagnosticV2::without_location(
            "modules.resource_exhausted",
            "authority collection length exceeds canonical u64 encoding",
        )
    })?;
    put_u64(hasher, value);
    Ok(())
}
fn put_text(hasher: &mut Sha256, value: &str) -> Result<(), ModuleDiagnosticV2> {
    put_len(hasher, value.len())?;
    hasher.update(value.as_bytes());
    Ok(())
}
fn put_version(hasher: &mut Sha256, value: VersionV2) {
    put_u64(hasher, value.major());
    put_u64(hasher, value.minor());
    put_u64(hasher, value.patch());
}

#[derive(Clone, Debug)]
pub struct ValidatedPackManifestV2(PackManifestV2);
impl ValidatedPackManifestV2 {
    #[must_use]
    pub const fn coordinate(&self) -> &PackCoordinateV2 {
        &self.0.coordinate
    }
    #[must_use]
    pub const fn content_digest(&self) -> ContentDigestV2 {
        self.0.content_digest
    }
}

#[derive(Clone, Debug)]
pub struct ValidatedPackGraphV2 {
    manifests: BTreeMap<PackIdV2, PackManifestV2>,
    fingerprint: [u8; 32],
    registry_identity: SourceRegistryIdentityV2,
}
impl ValidatedPackGraphV2 {
    #[must_use]
    pub const fn fingerprint(&self) -> &[u8; 32] {
        &self.fingerprint
    }
    #[must_use]
    pub const fn registry_identity(&self) -> &SourceRegistryIdentityV2 {
        &self.registry_identity
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }
    #[must_use]
    pub(crate) fn raw_manifest(&self, id: &PackIdV2) -> Option<&PackManifestV2> {
        self.manifests.get(id)
    }
    pub(crate) fn manifests(&self) -> impl Iterator<Item = (&PackIdV2, &PackManifestV2)> {
        self.manifests.iter()
    }
}

impl ValidatedPackManifestV2 {
    pub(crate) fn from_raw(manifest: PackManifestV2) -> Self {
        Self(manifest)
    }
}
