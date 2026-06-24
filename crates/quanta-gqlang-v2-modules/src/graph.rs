//! The canonical module / import graph and its deterministic resolution rules
//! (deliverables 1, 10; `D30`).
//!
//! This module owns the frozen authority the binder (ticket 02) consumes. It
//! pins, by construction and by test, the four rules ticket 11 must freeze:
//!
//! 1. import shadowing: a module's own (local) declaration of a name shadows any
//!    imported name of the same short name. Shadowing is structural and
//!    deterministic, never import-order-sensitive (`D30`).
//! 2. re-export precedence: when a module both directly imports a name and
//!    re-exports a name from a transitive module, the DIRECT import takes
//!    precedence over the re-export. Two re-exports of distinct targets do NOT
//!    silently pick a winner — they collapse to an ambiguous set.
//! 3. ambiguous import: any short name that resolves to more than one distinct
//!    fully-qualified target after rules 1-2 is a hard fail-closed reject
//!    ([`ModuleDiagnosticV1::AmbiguousImport`]) — no last-wins, no import-order
//!    tiebreak (`D28`).
//! 4. cycle policy: a module dependency cycle is an EXPLICIT typed reject
//!    ([`ModuleDiagnosticV1::ImportCycle`]); there is no silent accept and no
//!    assumed future cycle semantics (`D30`, ticket reject rule 2).
//!
//! Determinism note: all candidate sets are sorted and de-duplicated before any
//! decision, so resolution never depends on the order edges were added.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::diagnostic::ModuleDiagnosticV1;
use crate::visibility::VisibilityV1;

/// A fully-qualified target: `module::symbol`.
fn qualify(module: &str, symbol: &str) -> String {
    format!("{module}::{symbol}")
}

/// One module's declarations, direct imports, and re-exports.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct ModuleNode {
    /// Local declarations: short name -> visibility.
    declarations: BTreeMap<String, VisibilityV1>,
    /// Direct imports: local short name -> (source module, source symbol).
    direct_imports: BTreeMap<String, (String, String)>,
    /// Re-exports: re-exported short name -> set of (source module, source
    /// symbol) it forwards. A set (not a single value) so two distinct
    /// re-export sources stay ambiguous instead of last-wins.
    reexports: BTreeMap<String, BTreeSet<(String, String)>>,
}

/// The module/import graph builder + resolver.
///
/// Build a graph with [`ModuleGraphBuilderV1`], then call
/// [`ModuleGraphBuilderV1::resolve`] to freeze it into a [`ResolvedGraphV1`].
/// Resolution applies the four canonical rules and fails closed on the first
/// deterministically-ordered violation.
#[derive(Clone, Debug, Default)]
pub struct ModuleGraphBuilderV1 {
    modules: BTreeMap<String, ModuleNode>,
}

impl ModuleGraphBuilderV1 {
    /// An empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
        }
    }

    /// Declare a module with no contents (idempotent).
    #[must_use]
    pub fn module(mut self, name: impl Into<String>) -> Self {
        let _ = self.modules.entry(name.into()).or_default();
        self
    }

    /// Add a local declaration `symbol` with `visibility` to `module`.
    #[must_use]
    pub fn declare(
        mut self,
        module: impl Into<String>,
        symbol: impl Into<String>,
        visibility: VisibilityV1,
    ) -> Self {
        let _ = self
            .modules
            .entry(module.into())
            .or_default()
            .declarations
            .insert(symbol.into(), visibility);
        self
    }

    /// Add a direct import into `module`: bind local short name `local` to
    /// `from_module::from_symbol`.
    #[must_use]
    pub fn import(
        mut self,
        module: impl Into<String>,
        local: impl Into<String>,
        from_module: impl Into<String>,
        from_symbol: impl Into<String>,
    ) -> Self {
        let _ = self
            .modules
            .entry(module.into())
            .or_default()
            .direct_imports
            .insert(local.into(), (from_module.into(), from_symbol.into()));
        self
    }

    /// Add a re-export into `module`: expose short name `local` forwarding to
    /// `from_module::from_symbol`. Adding a second distinct source for the same
    /// `local` retains both (so it stays ambiguous, never last-wins).
    #[must_use]
    pub fn reexport(
        mut self,
        module: impl Into<String>,
        local: impl Into<String>,
        from_module: impl Into<String>,
        from_symbol: impl Into<String>,
    ) -> Self {
        let _ = self
            .modules
            .entry(module.into())
            .or_default()
            .reexports
            .entry(local.into())
            .or_default()
            .insert((from_module.into(), from_symbol.into()));
        self
    }

    /// Detect a module dependency cycle over import + re-export edges.
    ///
    /// Returns the canonicalized cycle (rotated to start at the lexically
    /// smallest member, closed by repeating that member) if any cycle exists.
    /// Deterministic: modules and their out-edges are visited in sorted order.
    fn detect_cycle(&self) -> Option<Vec<String>> {
        // States: 0 = unvisited, 1 = on stack, 2 = done. Keyed by owned name to
        // avoid borrowing the modules map while the DFS mutates its own stack.
        let mut state: BTreeMap<String, u8> = BTreeMap::new();
        for name in self.modules.keys() {
            let _ = state.insert(name.clone(), 0u8);
        }
        // Iterative DFS carrying the path, visiting roots in sorted order.
        for root in self.modules.keys() {
            if state.get(root).copied().unwrap_or(2) != 0 {
                continue;
            }
            let mut path: Vec<String> = Vec::new();
            // Stack of (node, next-child-index into its sorted out-edges).
            let mut stack: Vec<(String, usize)> = vec![(root.clone(), 0)];
            let _ = state.insert(root.clone(), 1);
            path.push(root.clone());
            while let Some((node, idx)) = stack.last().cloned() {
                let edges = self.out_edges(&node);
                if let Some(next) = edges.get(idx) {
                    if let Some(top) = stack.last_mut() {
                        top.1 += 1;
                    }
                    match state.get(next).copied().unwrap_or(2) {
                        0 => {
                            let _ = state.insert(next.clone(), 1);
                            stack.push((next.clone(), 0));
                            path.push(next.clone());
                        }
                        1 => {
                            // Back edge: `next` is on the current path -> cycle.
                            return Some(canonical_cycle(&path, next));
                        }
                        _ => {}
                    }
                } else {
                    let _ = state.insert(node, 2);
                    let _ = stack.pop();
                    let _ = path.pop();
                }
            }
        }
        None
    }

    /// Sorted out-edge target module names for `node` (imports + re-exports).
    fn out_edges(&self, node: &str) -> Vec<String> {
        let mut targets: BTreeSet<String> = BTreeSet::new();
        if let Some(m) = self.modules.get(node) {
            for (src, _) in m.direct_imports.values() {
                let _ = targets.insert(src.clone());
            }
            for sources in m.reexports.values() {
                for (src, _) in sources {
                    let _ = targets.insert(src.clone());
                }
            }
        }
        targets.into_iter().collect()
    }

    /// Resolve `src_mod::src_sym` to the fully-qualified name of the module that
    /// actually DECLARES the symbol, following re-export chains.
    ///
    /// A module exports `sym` if it declares it Public, OR re-exports it (which
    /// is followed transitively to the declaring origin). A plain import is NOT
    /// re-exposed — only declarations and re-exports cross the module boundary
    /// (visibility rule, deliverable 4). The cycle policy already ran, so this
    /// chain is acyclic and terminates. `importer` is carried only for the
    /// unknown-module diagnostic.
    fn resolve_export_origin(
        &self,
        importer: &str,
        src_mod: &str,
        src_sym: &str,
    ) -> Result<String, ModuleDiagnosticV1> {
        let target =
            self.modules
                .get(src_mod)
                .ok_or_else(|| ModuleDiagnosticV1::UnknownModule {
                    importer: importer.to_string(),
                    missing: src_mod.to_string(),
                })?;

        // A Public local declaration is the origin.
        if let Some(vis) = target.declarations.get(src_sym) {
            if vis.is_importable() {
                return Ok(qualify(src_mod, src_sym));
            }
        }

        // Otherwise it may be re-exported: follow the (unique) re-export source
        // to its origin. A re-export with two distinct sources is ambiguous.
        if let Some(sources) = target.reexports.get(src_sym) {
            let mut origins: BTreeSet<String> = BTreeSet::new();
            for (next_mod, next_sym) in sources {
                origins.insert(self.resolve_export_origin(src_mod, next_mod, next_sym)?);
            }
            let mut origins: Vec<String> = origins.into_iter().collect();
            if origins.len() > 1 {
                return Err(ModuleDiagnosticV1::AmbiguousImport {
                    name: src_sym.to_string(),
                    candidates: origins,
                });
            }
            if let Some(origin) = origins.pop() {
                return Ok(origin);
            }
        }

        Err(ModuleDiagnosticV1::NotExported {
            module: src_mod.to_string(),
            symbol: src_sym.to_string(),
        })
    }

    /// Resolve `module`'s direct imports + re-exports into the distinct
    /// fully-qualified target set per local name, applying shadowing and
    /// re-export precedence. Returns a map suitable for ambiguity checking.
    fn resolve_module_names(
        &self,
        module: &str,
        node: &ModuleNode,
    ) -> Result<BTreeMap<String, BTreeSet<String>>, ModuleDiagnosticV1> {
        let mut resolved: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

        // Rule 2 (lower precedence first): re-exports. Each re-exported source is
        // followed to its declaring ORIGIN so transitive re-export chains resolve
        // to one fully-qualified name.
        for (local, sources) in &node.reexports {
            let bucket = resolved.entry(local.clone()).or_default();
            for (src_mod, src_sym) in sources {
                let _ = bucket.insert(self.resolve_export_origin(module, src_mod, src_sym)?);
            }
        }

        // Rule 2 (higher precedence): a DIRECT import overrides any re-export of
        // the same local name — the direct import REPLACES the re-export bucket
        // rather than unioning with it.
        for (local, (src_mod, src_sym)) in &node.direct_imports {
            let q = self.resolve_export_origin(module, src_mod, src_sym)?;
            let mut bucket = BTreeSet::new();
            let _ = bucket.insert(q);
            let _ = resolved.insert(local.clone(), bucket);
        }

        // Rule 1: a LOCAL declaration shadows any imported/re-exported name.
        // Shadowing fully removes the imported candidates for that short name.
        for local in node.declarations.keys() {
            let mut bucket = BTreeSet::new();
            let _ = bucket.insert(qualify(module, local));
            let _ = resolved.insert(local.clone(), bucket);
        }

        Ok(resolved)
    }

    /// Freeze the graph into a [`ResolvedGraphV1`], failing closed on the first
    /// deterministically-ordered violation: cycle first, then per-module
    /// (sorted) unknown-module / not-exported / ambiguous-import.
    pub fn resolve(&self) -> Result<ResolvedGraphV1, ModuleDiagnosticV1> {
        // Rule 4: cycle policy is checked before name resolution.
        if let Some(cycle) = self.detect_cycle() {
            return Err(ModuleDiagnosticV1::ImportCycle { cycle });
        }

        let mut per_module: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        for (module, node) in &self.modules {
            let resolved = self.resolve_module_names(module, node)?;
            let mut frozen: BTreeMap<String, String> = BTreeMap::new();
            for (local, targets) in resolved {
                // Rule 3: ambiguity is a hard reject; candidates sorted by
                // BTreeSet iteration order.
                let mut candidates: Vec<String> = targets.into_iter().collect();
                if candidates.len() > 1 {
                    return Err(ModuleDiagnosticV1::AmbiguousImport {
                        name: local,
                        candidates,
                    });
                }
                if let Some(target) = candidates.pop() {
                    let _ = frozen.insert(local, target);
                }
            }
            let _ = per_module.insert(module.clone(), frozen);
        }

        Ok(ResolvedGraphV1 { per_module })
    }
}

/// Rotate `path` to start at the lexically smallest member of the cycle that
/// begins at `back` (the back-edge target), and close it by repeating that
/// member. The returned vector is the canonical, order-independent cycle.
fn canonical_cycle(path: &[String], back: &str) -> Vec<String> {
    // The cycle is the suffix of `path` starting at `back`.
    let start = path.iter().position(|n| n == back).unwrap_or(0);
    let cycle_slice: Vec<&str> = path
        .get(start..)
        .map(|s| s.iter().map(String::as_str).collect())
        .unwrap_or_default();
    if cycle_slice.is_empty() {
        return Vec::new();
    }
    // Rotate to the lexically smallest member for determinism.
    let min_pos = cycle_slice
        .iter()
        .enumerate()
        .min_by(|a, b| a.1.cmp(b.1))
        .map(|(i, _)| i)
        .unwrap_or(0);
    let mut rotated: Vec<String> = Vec::with_capacity(cycle_slice.len() + 1);
    for offset in 0..cycle_slice.len() {
        let idx = (min_pos + offset) % cycle_slice.len();
        if let Some(name) = cycle_slice.get(idx) {
            rotated.push((*name).to_string());
        }
    }
    // Close the cycle by repeating the start.
    if let Some(first) = rotated.first().cloned() {
        rotated.push(first);
    }
    rotated
}

/// A frozen, resolved module graph: every module's local short name maps to
/// exactly one fully-qualified target.
///
/// This is the value the binder consumes. It is inert: it encodes the already-
/// decided resolution and exposes no precedence or re-ranking surface, so the
/// binder cannot invent import-order authority (`D30`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedGraphV1 {
    per_module: BTreeMap<String, BTreeMap<String, String>>,
}

impl ResolvedGraphV1 {
    /// Resolve `name` within `module` to its single fully-qualified target, if
    /// the module imports/declares it.
    #[must_use]
    pub fn lookup(&self, module: &str, name: &str) -> Option<&str> {
        self.per_module
            .get(module)
            .and_then(|m| m.get(name))
            .map(String::as_str)
    }

    /// All `(local name, target)` bindings visible in `module`, sorted.
    #[must_use]
    pub fn bindings(&self, module: &str) -> Vec<(String, String)> {
        self.per_module
            .get(module)
            .map(|m| {
                m.iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    /// Number of modules in the frozen graph.
    #[must_use]
    pub fn module_count(&self) -> usize {
        self.per_module.len()
    }

    /// Project this resolved graph into the flat binder-consumed view for one
    /// module: short name -> distinct fully-qualified targets.
    ///
    /// This is the integration seam with the binder's `ModuleGraphViewV1`
    /// (ticket 02). Because resolution already collapsed each name to exactly
    /// one target (or rejected on ambiguity), every returned set has at most one
    /// element — the binder will therefore never re-derive precedence.
    #[must_use]
    pub fn binder_view(&self, module: &str) -> BTreeMap<String, Vec<String>> {
        self.per_module
            .get(module)
            .map(|m| {
                m.iter()
                    .map(|(local, target)| (local.clone(), vec![target.clone()]))
                    .collect()
            })
            .unwrap_or_default()
    }
}
