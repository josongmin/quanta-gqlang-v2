//! Lockfile model for reproducible, offline pack resolution (deliverable 8).
//!
//! A [`LockfileV1`] pins each pack name to one resolved version. Resolution must
//! be reproducible offline from manifest + lock data (ticket pinned semantics
//! 7): given a [`crate::manifest::PackGraphV1`] and a matching lock, the locked
//! version set is replayed deterministically and any drift between lock and
//! manifest fails closed with [`ModuleDiagnosticV1::LockMismatch`].
//!
//! The lock has a textual round-trip form ([`LockfileV1::to_text`] /
//! [`LockfileV1::parse`]) so a lock can be committed and reproduced byte-for-
//! byte: entries are emitted in canonical sorted order.

use std::collections::BTreeMap;

use crate::diagnostic::ModuleDiagnosticV1;
use crate::manifest::PackGraphV1;
use crate::version::VersionV1;

/// A reproducible lock: pack name -> resolved version.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LockfileV1 {
    entries: BTreeMap<String, VersionV1>,
}

impl LockfileV1 {
    /// An empty lock.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Pin `pack` to `version` (builder style; replaces any existing pin).
    #[must_use]
    pub fn with_pin(mut self, pack: impl Into<String>, version: VersionV1) -> Self {
        let _ = self.entries.insert(pack.into(), version);
        self
    }

    /// The locked version for `pack`, if pinned.
    #[must_use]
    pub fn locked_version(&self, pack: &str) -> Option<VersionV1> {
        self.entries.get(pack).copied()
    }

    /// Number of pinned packs.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the lock pins nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Derive a lock from a pack graph: pin every pack to its manifest version.
    /// The result is canonical and replayable.
    #[must_use]
    pub fn from_graph(graph: &PackGraphV1) -> Self {
        let mut lock = Self::new();
        for (name, version) in graph.pinned_versions() {
            lock = lock.with_pin(name, version);
        }
        lock
    }

    /// Serialize to canonical text, one `pack version` line per pin in sorted
    /// order, terminated by a newline. Round-trips through [`LockfileV1::parse`].
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for (pack, version) in &self.entries {
            out.push_str(pack);
            out.push(' ');
            out.push_str(&version.to_string());
            out.push('\n');
        }
        out
    }

    /// Parse canonical lock text. Returns `None` on any malformed line (wrong
    /// field count, unparseable version).
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut lock = Self::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut parts = trimmed.split_whitespace();
            let pack = parts.next()?;
            let version = VersionV1::parse(parts.next()?)?;
            if parts.next().is_some() {
                return None;
            }
            lock = lock.with_pin(pack, version);
        }
        Some(lock)
    }

    /// Verify this lock exactly reproduces `graph`: every manifest pack must be
    /// pinned to its manifest version and vice versa. Fail-closed with
    /// [`ModuleDiagnosticV1::LockMismatch`] on the first sorted discrepancy.
    pub fn verify_against(&self, graph: &PackGraphV1) -> Result<(), ModuleDiagnosticV1> {
        let manifest_versions: BTreeMap<String, VersionV1> = graph.pinned_versions();

        // Every manifest pack must be locked at the exact manifest version.
        for (name, version) in &manifest_versions {
            match self.entries.get(name) {
                Some(locked) if *locked == *version => {}
                Some(locked) => {
                    return Err(ModuleDiagnosticV1::LockMismatch {
                        pack: name.clone(),
                        detail: format!("locked {locked} but manifest declares {version}"),
                    });
                }
                None => {
                    return Err(ModuleDiagnosticV1::LockMismatch {
                        pack: name.clone(),
                        detail: format!("manifest declares {version} but lock has no pin"),
                    });
                }
            }
        }

        // Every locked pack must exist in the manifest set (no stale pins).
        for name in self.entries.keys() {
            if !manifest_versions.contains_key(name) {
                return Err(ModuleDiagnosticV1::LockMismatch {
                    pack: name.clone(),
                    detail: "lock pins a pack absent from the manifest graph".to_string(),
                });
            }
        }

        Ok(())
    }
}
