//! Checked pack manifests, dependency graphs, and canonical fingerprints.

use std::collections::BTreeMap;
#[cfg(test)]
use std::collections::BTreeSet;
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
        session.charge_node()?;
        session.charge_string(input.pack_id.as_str())?;
        check_registry(&input.location, source_registry)?;
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
    let roots = manifests.keys().cloned().collect::<Vec<_>>();
    let edges = manifests
        .iter()
        .flat_map(|(source, manifest)| {
            manifest.dependencies.values().map(move |dependency| {
                (
                    source.clone(),
                    dependency.pack_id.clone(),
                    (
                        source.clone(),
                        dependency.pack_id.clone(),
                        loc_key(dependency.location.location()),
                    ),
                    dependency.location.clone(),
                )
            })
        })
        .collect::<Vec<_>>();
    let keyed = edges
        .iter()
        .map(|(source, target, key, _)| (source.clone(), target.clone(), key.clone()))
        .collect::<Vec<_>>();
    let Some(indices) = cycle_with_legacy_steps(&keyed, &roots, session)? else {
        return Ok(None);
    };
    let mut witness = Vec::with_capacity(indices.len());
    for index in indices {
        let Some((source, target, _, location)) = edges.get(index) else {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.internal_cycle_witness_mismatch",
                "pack cycle witness index is outside the validated edge set",
            ));
        };
        witness.push((source.clone(), target.clone(), location.clone()));
    }
    Ok(Some(witness))
}

/// Preserves the former per-simple-path edge charge, then obtains the minimum
/// canonical witness through the graph's polynomial SCC/path search.
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "node and edge indices are derived from the complete local ID set above"
)]
pub(crate) fn cycle_with_legacy_steps<Id: Ord + Clone, Key: Ord + Clone>(
    edges: &[(Id, Id, Key)],
    roots: &[Id],
    session: &mut ModuleBudgetSessionV2,
) -> Result<Option<Vec<usize>>, ModuleDiagnosticV2> {
    let node_slots = roots
        .len()
        .checked_add(edges.len().checked_mul(2).ok_or_else(|| {
            ModuleDiagnosticV2::without_location(
                "modules.resource_exhausted",
                "cycle node capacity overflowed",
            )
        })?)
        .ok_or_else(|| {
            ModuleDiagnosticV2::without_location(
                "modules.resource_exhausted",
                "cycle node capacity overflowed",
            )
        })?;
    let mut nodes = Vec::with_capacity(node_slots);
    nodes.extend(roots.iter().cloned());
    for (source, target, _) in edges {
        nodes.push(source.clone());
        nodes.push(target.clone());
    }
    nodes.sort_unstable();
    nodes.dedup();
    let mut outbound = vec![0_usize; nodes.len()];
    for (source, _, _) in edges {
        let source_index = nodes.binary_search(source).expect("source was indexed");
        outbound[source_index] += 1;
    }
    let mut adjacency = Vec::with_capacity(nodes.len());
    let mut adjacency_ids = Vec::with_capacity(nodes.len());
    for count in outbound {
        adjacency.push(Vec::with_capacity(count));
        adjacency_ids.push(Vec::with_capacity(count));
    }
    for (index, (source, target, key)) in edges.iter().enumerate() {
        let source_index = nodes.binary_search(source).expect("source was indexed");
        let target_index = nodes.binary_search(target).expect("target was indexed");
        adjacency[source_index].push((
            target_index,
            CycleEdgeLabel {
                key: key.clone(),
                index,
            },
        ));
        adjacency_ids[source_index].push(target_index);
    }
    let root_indices = roots
        .iter()
        .map(|root| nodes.binary_search(root).expect("root was indexed"))
        .collect::<Vec<_>>();
    if let Some((steps, overflowed)) =
        acyclic_cycle_walk_steps_indexed(&adjacency_ids, &root_indices)
    {
        session.charge_steps(steps)?;
        if overflowed {
            session.charge_step()?;
        }
        return Ok(None);
    }

    // The charge-only walk uses one active bit per node and one frame per
    // path vertex. No cycle path is copied for a charged edge.
    let mut active = vec![false; nodes.len()];
    for &root_index in &root_indices {
        active[root_index] = true;
        let mut stack = Vec::with_capacity(nodes.len());
        stack.push((root_index, 0_usize));
        while let Some((node, next_index)) = stack.last_mut() {
            let Some((target, _)) = adjacency[*node].get(*next_index) else {
                active[*node] = false;
                stack.pop();
                continue;
            };
            let target = *target;
            *next_index += 1;
            session.charge_step()?;
            if !active[target] {
                active[target] = true;
                stack.push((target, 0));
            }
        }
    }
    Ok(crate::graph::minimal_cycle_witness(&adjacency)
        .map(|witness| witness.into_iter().map(|label| label.index).collect()))
}

/// Source-count upper capacity of the indexed cycle walk. This models both
/// DAG and cyclic branches because the branch is selected after admission.
/// The request caller instantiates Id and Key with borrowed catalog IDs, so
/// their Clone operations never allocate nested strings.
pub(crate) fn cycle_scratch_bytes_v3<Id, Key>(edges: usize, roots: usize) -> Option<usize> {
    fn add<T>(total: &mut usize, count: usize) -> Option<()> {
        *total = total.checked_add(count.checked_mul(std::mem::size_of::<T>())?)?;
        Some(())
    }
    let nodes = roots.checked_add(edges.checked_mul(2)?)?;
    let mut total = 0_usize;
    add::<Id>(&mut total, nodes)?;
    add::<usize>(&mut total, nodes)?; // outbound counts
    add::<Vec<(usize, CycleEdgeLabel<Key>)>>(&mut total, nodes)?;
    add::<(usize, CycleEdgeLabel<Key>)>(&mut total, edges)?;
    add::<Vec<usize>>(&mut total, nodes)?;
    add::<usize>(&mut total, edges)?;
    add::<usize>(&mut total, roots)?; // root indices
    add::<usize>(&mut total, nodes)?; // indegree
    add::<usize>(&mut total, nodes)?; // ways
    add::<usize>(&mut total, nodes)?; // ready
    add::<bool>(&mut total, nodes)?; // cyclic active set
    add::<(usize, usize)>(&mut total, nodes)?; // cyclic DFS stack
    total = total.checked_add(crate::graph::minimal_cycle_scratch_bytes_v3::<
        CycleEdgeLabel<Key>,
    >(nodes, edges)?)?;
    add::<usize>(&mut total, nodes)?; // returned witness indices
    Some(total)
}

#[derive(Clone)]
struct CycleEdgeLabel<Key> {
    key: Key,
    index: usize,
}

impl<Key: Ord> PartialEq for CycleEdgeLabel<Key> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}
impl<Key: Ord> Eq for CycleEdgeLabel<Key> {}
impl<Key: Ord> PartialOrd for CycleEdgeLabel<Key> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<Key: Ord> Ord for CycleEdgeLabel<Key> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

/// Counts the edge visits of the legacy all-root simple-path walk on a DAG.
/// This keeps the public step budget exact without enumerating its paths.
fn acyclic_cycle_walk_steps_indexed(
    adjacency: &[Vec<usize>],
    roots: &[usize],
) -> Option<(usize, bool)> {
    let mut indegree = vec![0_usize; adjacency.len()];
    for targets in adjacency {
        for &target in targets {
            *indegree.get_mut(target)? += 1;
        }
    }
    let mut ways = vec![0_usize; adjacency.len()];
    for &root in roots {
        *ways.get_mut(root)? += 1;
    }
    let mut ready = Vec::with_capacity(indegree.len());
    for (index, count) in indegree.iter().enumerate() {
        if *count == 0 {
            ready.push(index);
        }
    }
    let mut steps = 0_usize;
    let mut overflowed = false;
    let mut visited = 0_usize;
    while !ready.is_empty() {
        let node = ready.remove(0);
        visited += 1;
        let visits = *ways.get(node)?;
        let targets = adjacency.get(node)?;
        let edge_visits = visits.checked_mul(targets.len()).unwrap_or_else(|| {
            overflowed = true;
            usize::MAX
        });
        steps = steps.checked_add(edge_visits).unwrap_or_else(|| {
            overflowed = true;
            usize::MAX
        });
        for &target in targets {
            let target_ways = ways.get_mut(target)?;
            *target_ways = target_ways.checked_add(visits).unwrap_or_else(|| {
                overflowed = true;
                usize::MAX
            });
            let count = indegree.get_mut(target)?;
            *count -= 1;
            if *count == 0 {
                match ready.binary_search(&target) {
                    Ok(_) => {}
                    Err(index) => ready.insert(index, target),
                }
            }
        }
    }
    (visited == indegree.len()).then_some((steps, overflowed))
}

/// Compatibility test oracle for the historical map-based step count. The
/// production cycle walker above uses source-indexed vectors.
#[cfg(test)]
pub(crate) fn acyclic_cycle_walk_steps<Id: Ord + Clone>(
    adjacency: &BTreeMap<Id, Vec<Id>>,
    roots: &[Id],
) -> Option<(usize, bool)> {
    let mut indegree = BTreeMap::<Id, usize>::new();
    for (source, targets) in adjacency {
        indegree.entry(source.clone()).or_default();
        for target in targets {
            *indegree.entry(target.clone()).or_default() += 1;
        }
    }
    let mut ways = indegree
        .keys()
        .cloned()
        .map(|id| (id, 0_usize))
        .collect::<BTreeMap<_, _>>();
    for root in roots {
        *ways.entry(root.clone()).or_default() += 1;
    }
    let mut ready = indegree
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(id.clone()))
        .collect::<BTreeSet<_>>();
    let mut steps = 0_usize;
    let mut overflowed = false;
    let mut visited = 0_usize;
    while let Some(node) = ready.pop_first() {
        visited += 1;
        let visits = *ways.get(&node)?;
        if let Some(targets) = adjacency.get(&node) {
            let edge_visits = visits.checked_mul(targets.len()).unwrap_or_else(|| {
                overflowed = true;
                usize::MAX
            });
            steps = steps.checked_add(edge_visits).unwrap_or_else(|| {
                overflowed = true;
                usize::MAX
            });
            for target in targets {
                let target_ways = ways.get_mut(target)?;
                *target_ways = target_ways.checked_add(visits).unwrap_or_else(|| {
                    overflowed = true;
                    usize::MAX
                });
                let count = indegree.get_mut(target)?;
                *count -= 1;
                if *count == 0 {
                    ready.insert(target.clone());
                }
            }
        }
    }
    (visited == indegree.len()).then_some((steps, overflowed))
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "bounded exhaustive oracle and exact-budget assertions"
)]
mod cycle_walk_tests {
    use super::{acyclic_cycle_walk_steps, cycle_with_legacy_steps};
    use crate::budget::ModuleBudgetV2;
    use quanta_gqlang_v2_types::AuthorityBudgetProfileV2;
    use std::collections::{BTreeMap, BTreeSet};

    fn session(step_limit: usize) -> crate::budget::ModuleBudgetSessionV2 {
        ModuleBudgetV2::from_profile(
            AuthorityBudgetProfileV2::try_new(100, 100, step_limit, 1_024, 128).unwrap(),
        )
        .begin_session_v2()
    }

    fn legacy_steps(adjacency: &BTreeMap<usize, Vec<usize>>, roots: &[usize]) -> usize {
        let mut steps = 0;
        for root in roots {
            let mut stack = vec![(*root, BTreeSet::from([*root]))];
            while let Some((node, seen)) = stack.pop() {
                for target in adjacency.get(&node).into_iter().flatten() {
                    steps += 1;
                    if !seen.contains(target) {
                        let mut next_seen = seen.clone();
                        next_seen.insert(*target);
                        stack.push((*target, next_seen));
                    }
                }
            }
        }
        steps
    }

    #[test]
    fn dag_fast_path_matches_legacy_steps_for_all_five_node_forward_graphs() {
        let pairs = (0..5)
            .flat_map(|source| ((source + 1)..5).map(move |target| (source, target)))
            .collect::<Vec<_>>();
        for mask in 0..(1_usize << pairs.len()) {
            let mut adjacency = (0..5)
                .map(|node| (node, Vec::new()))
                .collect::<BTreeMap<_, _>>();
            for (index, &(source, target)) in pairs.iter().enumerate() {
                if mask & (1 << index) != 0 {
                    adjacency.get_mut(&source).unwrap().push(target);
                }
            }
            for roots in [&[0, 1, 2, 3, 4][..], &[0, 2][..]] {
                assert_eq!(
                    acyclic_cycle_walk_steps(&adjacency, roots),
                    Some((legacy_steps(&adjacency, roots), false)),
                    "mask={mask}, roots={roots:?}"
                );
            }
        }
    }

    #[test]
    fn dag_fast_path_is_bounded_on_long_chain_and_branching_graph() {
        let chain = (0..5_000)
            .map(|node| {
                (
                    node,
                    (node + 1 < 5_000).then_some(node + 1).into_iter().collect(),
                )
            })
            .collect::<BTreeMap<_, Vec<_>>>();
        let roots = (0..5_000).collect::<Vec<_>>();
        assert_eq!(
            acyclic_cycle_walk_steps(&chain, &roots),
            Some((12_497_500, false))
        );

        let branching = (0..90)
            .map(|node| {
                (
                    node,
                    [node + 1, node + 2]
                        .into_iter()
                        .filter(|target| *target < 90)
                        .collect(),
                )
            })
            .collect::<BTreeMap<_, Vec<_>>>();
        assert_eq!(
            acyclic_cycle_walk_steps(&branching, &(0..90).collect::<Vec<_>>()),
            Some((usize::MAX, true))
        );
        let keyed = branching
            .iter()
            .flat_map(|(&source, targets)| {
                targets
                    .iter()
                    .map(move |&target| (source, target, (source, target)))
            })
            .collect::<Vec<_>>();
        let error = cycle_with_legacy_steps(
            &keyed,
            &(0..90).collect::<Vec<_>>(),
            &mut session(usize::MAX),
        )
        .unwrap_err();
        assert_eq!(
            error
                .resource_exhaustion()
                .map(|payload| payload.dimension()),
            Some("steps")
        );
    }

    #[test]
    fn cycle_fast_path_defers_to_witness_search() {
        let adjacency = BTreeMap::from([(0, vec![1]), (1, vec![2]), (2, vec![0])]);
        assert_eq!(acyclic_cycle_walk_steps(&adjacency, &[0, 1, 2]), None);
    }

    #[test]
    fn cyclic_legacy_budget_is_exact_and_tail_does_not_enter_self_cycle() {
        let edges = vec![(0, 1, (1, 10)), (1, 1, (1, 20))];
        assert_eq!(
            cycle_with_legacy_steps(&edges, &[0, 1], &mut session(3)).unwrap(),
            Some(vec![1])
        );
        let error = cycle_with_legacy_steps(&edges, &[0, 1], &mut session(2)).unwrap_err();
        let payload = error.resource_exhaustion().unwrap();
        assert_eq!(
            (payload.dimension(), payload.limit(), payload.observed()),
            ("steps", 2, 3)
        );
    }

    #[test]
    fn tied_catalog_first_label_uses_smallest_suffix() {
        // Both closing edges have the same catalog key (target, location).
        // The edge from vertex 2 is deliberately listed first.
        let edges = vec![
            (2, 0, (0, 0)),
            (0, 2, (2, 1)),
            (1, 0, (0, 0)),
            (0, 1, (1, 1)),
        ];
        assert_eq!(
            cycle_with_legacy_steps(&edges, &[0, 1, 2], &mut session(100)).unwrap(),
            Some(vec![2, 3])
        );
    }

    #[test]
    fn cyclic_witness_matches_exhaustive_reference_on_all_three_node_graphs() {
        fn reference(edges: &[(usize, usize, (usize, usize))]) -> Option<Vec<(usize, usize)>> {
            let mut best: Option<Vec<(usize, usize)>> = None;
            for root in 0..3 {
                let mut stack = vec![(root, Vec::<usize>::new(), BTreeSet::from([root]))];
                while let Some((node, path, seen)) = stack.pop() {
                    for (index, edge) in edges.iter().enumerate() {
                        if edge.0 != node {
                            continue;
                        }
                        if seen.contains(&edge.1) {
                            let start = path
                                .iter()
                                .position(|&index| edges[index].0 == edge.1)
                                .unwrap_or(path.len());
                            let mut cycle = path[start..]
                                .iter()
                                .map(|&index| edges[index].2)
                                .collect::<Vec<_>>();
                            cycle.push(edge.2);
                            let offset = cycle
                                .iter()
                                .enumerate()
                                .min_by_key(|(_, key)| *key)
                                .unwrap()
                                .0;
                            cycle.rotate_left(offset);
                            if best.as_ref().map_or(true, |current| cycle < *current) {
                                best = Some(cycle);
                            }
                        } else {
                            let mut next_path = path.clone();
                            next_path.push(index);
                            let mut next_seen = seen.clone();
                            next_seen.insert(edge.1);
                            stack.push((edge.1, next_path, next_seen));
                        }
                    }
                }
            }
            best
        }

        let pairs = (0..3)
            .flat_map(|source| (0..3).map(move |target| (source, target)))
            .collect::<Vec<_>>();
        for mask in 0..(1_usize << pairs.len()) {
            let edges = pairs
                .iter()
                .enumerate()
                .filter_map(|(index, &(source, target))| {
                    (mask & (1 << index) != 0).then_some((source, target, (source, target)))
                })
                .collect::<Vec<_>>();
            let observed = cycle_with_legacy_steps(&edges, &[0, 1, 2], &mut session(100))
                .unwrap()
                .map(|indices| indices.into_iter().map(|index| edges[index].2).collect());
            assert_eq!(observed, reference(&edges), "mask={mask}");
        }
    }
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
