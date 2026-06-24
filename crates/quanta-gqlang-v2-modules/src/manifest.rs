//! Pack manifest format (deliverable 3) and overlapping-pack conflict detection
//! (deliverable 9).
//!
//! A [`PackManifestV1`] declares a pack's identity, version, dependency ranges,
//! language/feature compatibility window, public exports, and suite membership.
//! A [`PackGraphV1`] is a set of manifests resolved together; resolving it
//! validates language compat and detects overlapping public exports across
//! packs, which fail closed (`D28`) rather than last-wins.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::diagnostic::ModuleDiagnosticV1;
use crate::version::{VersionRangeV1, VersionV1};

/// A pack's identity: a name plus a version.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PackIdV1 {
    /// The pack name (registry-unique).
    pub name: String,
    /// The pack version.
    pub version: VersionV1,
}

impl PackIdV1 {
    /// Construct a pack identity.
    #[must_use]
    pub fn new(name: impl Into<String>, version: VersionV1) -> Self {
        Self {
            name: name.into(),
            version,
        }
    }

    /// Render as `name@major.minor.patch`.
    #[must_use]
    pub fn render(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// A declared dependency on another pack by name + version range.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DependencyReqV1 {
    /// The dependency pack name.
    pub name: String,
    /// The accepted version range.
    pub range: VersionRangeV1,
}

impl DependencyReqV1 {
    /// Construct a dependency requirement.
    #[must_use]
    pub fn new(name: impl Into<String>, range: VersionRangeV1) -> Self {
        Self {
            name: name.into(),
            range,
        }
    }
}

/// A named suite: a deterministically-ordered set of query ids in this pack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuiteV1 {
    /// The suite name (unique within the pack).
    pub name: String,
    /// Member query ids, canonically ordered/de-duplicated.
    pub members: BTreeSet<String>,
}

impl SuiteV1 {
    /// Construct an empty suite.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            members: BTreeSet::new(),
        }
    }

    /// Add a member query id (idempotent).
    #[must_use]
    pub fn with_member(mut self, query_id: impl Into<String>) -> Self {
        let _ = self.members.insert(query_id.into());
        self
    }
}

/// A complete pack manifest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackManifestV1 {
    /// The pack identity.
    pub id: PackIdV1,
    /// The host language/feature compatibility window this pack requires.
    pub language_compat: VersionRangeV1,
    /// Declared dependencies on other packs.
    pub dependencies: Vec<DependencyReqV1>,
    /// Public symbols this pack exports (canonically ordered).
    pub exports: BTreeSet<String>,
    /// Named suites, keyed by suite name.
    pub suites: BTreeMap<String, SuiteV1>,
}

impl PackManifestV1 {
    /// Construct a manifest with the given identity and compat window.
    #[must_use]
    pub fn new(id: PackIdV1, language_compat: VersionRangeV1) -> Self {
        Self {
            id,
            language_compat,
            dependencies: Vec::new(),
            exports: BTreeSet::new(),
            suites: BTreeMap::new(),
        }
    }

    /// Add a dependency requirement (builder style).
    #[must_use]
    pub fn with_dependency(mut self, dep: DependencyReqV1) -> Self {
        self.dependencies.push(dep);
        self
    }

    /// Add a public export (idempotent).
    #[must_use]
    pub fn with_export(mut self, symbol: impl Into<String>) -> Self {
        let _ = self.exports.insert(symbol.into());
        self
    }

    /// Add a suite (replaces any existing suite of the same name).
    #[must_use]
    pub fn with_suite(mut self, suite: SuiteV1) -> Self {
        let _ = self.suites.insert(suite.name.clone(), suite);
        self
    }

    /// Validate this pack against a host language version. Fail-closed with
    /// [`ModuleDiagnosticV1::CompatMismatch`] when the host is outside the
    /// pack's compatibility window.
    pub fn check_language_compat(&self, host: VersionV1) -> Result<(), ModuleDiagnosticV1> {
        if self.language_compat.contains(host) {
            Ok(())
        } else {
            Err(ModuleDiagnosticV1::CompatMismatch {
                pack: self.id.render(),
                required: self.language_compat,
                actual: host,
            })
        }
    }
}

/// A set of manifests resolved together against one host language version.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackGraphV1 {
    manifests: BTreeMap<String, PackManifestV1>,
}

impl PackGraphV1 {
    /// An empty pack graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            manifests: BTreeMap::new(),
        }
    }

    /// Add a manifest, keyed by pack name, failing closed on a duplicate name.
    ///
    /// Per `D28` a pack-name conflict is a hard reject with a deterministic
    /// [`ModuleDiagnosticV1::DuplicatePackName`] diagnostic — never a silent
    /// last-wins overwrite. The duplicate is rejected BEFORE the graph mutates,
    /// so a rejected `add_manifest` leaves the graph state untouched.
    pub fn add_manifest(mut self, manifest: PackManifestV1) -> Result<Self, ModuleDiagnosticV1> {
        if let Some(existing) = self.manifests.get(&manifest.id.name) {
            return Err(ModuleDiagnosticV1::DuplicatePackName {
                name: manifest.id.name.clone(),
                existing: existing.id.version,
                incoming: manifest.id.version,
            });
        }
        let _ = self.manifests.insert(manifest.id.name.clone(), manifest);
        Ok(self)
    }

    /// Borrow a manifest by pack name.
    #[must_use]
    pub fn manifest(&self, name: &str) -> Option<&PackManifestV1> {
        self.manifests.get(name)
    }

    /// Number of packs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.manifests.len()
    }

    /// The canonical `pack name -> version` pinning this graph declares, in
    /// sorted order. This is the reproducible set a lockfile mirrors.
    #[must_use]
    pub fn pinned_versions(&self) -> BTreeMap<String, VersionV1> {
        self.manifests
            .iter()
            .map(|(name, manifest)| (name.clone(), manifest.id.version))
            .collect()
    }

    /// Whether the graph holds no packs.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.manifests.is_empty()
    }

    /// Validate every pack against `host` and detect overlapping public
    /// exports, failing closed on the first deterministically-ordered conflict.
    ///
    /// Order: (1) every pack's language compat, then (2) dependency
    /// satisfiability against the in-graph version, then (3) overlapping public
    /// exports. All keyed iteration is over sorted `BTreeMap`s.
    pub fn validate(&self, host: VersionV1) -> Result<(), ModuleDiagnosticV1> {
        // (1) language compat, sorted by pack name.
        for manifest in self.manifests.values() {
            manifest.check_language_compat(host)?;
        }

        // (2) dependency satisfiability against in-graph versions.
        for manifest in self.manifests.values() {
            for dep in &manifest.dependencies {
                let offered = self.manifests.get(&dep.name).map(|m| m.id.version);
                match offered {
                    Some(version) if dep.range.contains(version) => {}
                    Some(version) => {
                        return Err(ModuleDiagnosticV1::DependencyUnsatisfied {
                            pack: manifest.id.render(),
                            dependency: dep.name.clone(),
                            required: dep.range,
                            offered: version,
                        });
                    }
                    None => {
                        return Err(ModuleDiagnosticV1::UnknownModule {
                            importer: manifest.id.render(),
                            missing: dep.name.clone(),
                        });
                    }
                }
            }
        }

        // (3) overlapping public exports: build symbol -> packs map, reject any
        // symbol exported by more than one pack.
        let mut owners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for manifest in self.manifests.values() {
            for symbol in &manifest.exports {
                let _ = owners
                    .entry(symbol.clone())
                    .or_default()
                    .insert(manifest.id.render());
            }
        }
        for (symbol, packs) in owners {
            if packs.len() > 1 {
                return Err(ModuleDiagnosticV1::OverlappingPackExport {
                    symbol,
                    packs: packs.into_iter().collect(),
                });
            }
        }

        Ok(())
    }
}
