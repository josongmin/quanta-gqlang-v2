//! The FROZEN module/import graph the binder consumes (a `ticket 11` boundary).
//!
//! Cites `D30`: module/import graph semantics — re-export precedence, module
//! cycles, and import ordering — are owned by ticket 11. The binder MUST NOT
//! invent that authority. It only *consumes* an already-resolved frozen graph
//! and resolves names against it.
//!
//! Because ticket 11 is not built yet, this module defines the CONSUMED
//! interface as a concrete value type ([`ModuleGraphV1`]) plus a trait
//! ([`ModuleGraphViewV1`]) so ticket 11 can later supply a real graph by
//! implementing the trait without the binder changing. The binder treats the
//! graph as immutable input: it never mutates, re-orders, or precedence-ranks
//! entries.
//!
//! Determinism contract (so resolution does not depend on textual import
//! order): a lookup either returns exactly one fully-qualified target, or — if
//! the frozen graph already records more than one distinct target for a name —
//! the binder reports an ambiguous-import reject (`D28`). The graph view is
//! responsible only for reporting the distinct candidate set; it does NOT pick
//! a winner.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

/// Read-only view of a frozen module/import graph.
///
/// Ticket 11 owns building one of these. The binder depends only on this
/// surface so that import-precedence and cycle authority stay out of lane.
pub trait ModuleGraphViewV1 {
    /// Resolve an imported short name to the set of distinct fully-qualified
    /// targets the frozen graph records for it.
    ///
    /// Contract:
    /// - an empty set means the name is not imported (the binder treats this as
    ///   "not an import"; it may still resolve as a local declaration),
    /// - a one-element set is an unambiguous import,
    /// - a two-or-more-element set is an ambiguous import the binder rejects
    ///   deterministically (`D28`); the order returned here is irrelevant
    ///   because the binder sorts candidates before reporting.
    fn imported_targets(&self, name: &str) -> Vec<String>;
}

/// A concrete frozen module-graph snapshot.
///
/// This is a plain, immutable mapping from imported short name to the set of
/// distinct fully-qualified targets. It is intentionally inert: it encodes no
/// precedence and performs no last-wins collapse, so an entry with multiple
/// targets stays ambiguous and is rejected rather than silently resolved.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleGraphV1 {
    entries: BTreeMap<String, BTreeSet<String>>,
}

impl ModuleGraphV1 {
    /// An empty frozen graph (nothing imported).
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Record that `name` is imported and resolves to fully-qualified `target`.
    ///
    /// Adding a second distinct `target` for the same `name` does NOT overwrite
    /// the first: both are retained so the lookup stays ambiguous. This encodes
    /// the `D28` rule that conflicting imports fail closed instead of last-wins.
    /// Re-adding an identical `(name, target)` is idempotent.
    #[must_use]
    pub fn with_import(mut self, name: impl Into<String>, target: impl Into<String>) -> Self {
        let _ = self
            .entries
            .entry(name.into())
            .or_default()
            .insert(target.into());
        self
    }

    /// Number of imported short names recorded.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the graph records any imports.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl ModuleGraphViewV1 for ModuleGraphV1 {
    fn imported_targets(&self, name: &str) -> Vec<String> {
        self.entries
            .get(name)
            .map(|targets| targets.iter().cloned().collect())
            .unwrap_or_default()
    }
}
