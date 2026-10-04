//! Checked module/import authority and deterministic visibility resolution.

use std::collections::BTreeMap;
use std::collections::VecDeque;

use crate::budget::ModuleBudgetSessionV2;
use crate::visibility::VisibilityV2;
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, LocatedSpanV2, SourceRegistryIdentityV2};

mod catalog_identity;
pub use catalog_identity::{CatalogAssetIdV2, CatalogAssetRefInputV2};

/// Canonical, collision-free qualified module identity.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleIdV2(String);

impl ModuleIdV2 {
    pub fn try_new(
        value: &str,
        location: CheckedLocatedSpanV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if !is_qualified_name(value) {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_module_id",
                format!("invalid module id `{}`", excerpt(value)),
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

/// Disjoint semantic namespaces. Stable tags are part of the public format.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NamespaceKindV2 {
    Value,
    Relation,
    Predicate,
    Type,
    Query,
    Check,
    Capability,
    ModelSource,
    PackSource,
    View,
    Model,
    Pack,
    Profile,
    Gate,
    Rule,
}

impl NamespaceKindV2 {
    #[must_use]
    pub const fn tag(self) -> &'static str {
        match self {
            Self::Value => "value",
            Self::Relation => "relation",
            Self::Predicate => "predicate",
            Self::Type => "type",
            Self::Query => "query",
            Self::Check => "check",
            Self::Capability => "capability",
            Self::ModelSource => "model_source",
            Self::PackSource => "pack_source",
            Self::View => "view",
            Self::Model => "model",
            Self::Pack => "pack",
            Self::Profile => "profile",
            Self::Gate => "gate",
            Self::Rule => "rule",
        }
    }

    pub(crate) fn try_from_tag(
        tag: &str,
        location: &CheckedLocatedSpanV2,
    ) -> Result<Self, ModuleDiagnosticV2> {
        match tag {
            "value" => Ok(Self::Value),
            "relation" => Ok(Self::Relation),
            "predicate" => Ok(Self::Predicate),
            "type" => Ok(Self::Type),
            "query" => Ok(Self::Query),
            "check" => Ok(Self::Check),
            "capability" => Ok(Self::Capability),
            "model_source" => Ok(Self::ModelSource),
            "pack_source" => Ok(Self::PackSource),
            "view" => Ok(Self::View),
            "model" => Ok(Self::Model),
            "pack" => Ok(Self::Pack),
            "profile" => Ok(Self::Profile),
            "gate" => Ok(Self::Gate),
            "rule" => Ok(Self::Rule),
            _ => Err(ModuleDiagnosticV2::new(
                "modules.unknown_catalog_kind",
                format!("unknown catalog kind `{}`", excerpt(tag)),
                location,
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleNodeInputV2 {
    pub module_id: ModuleIdV2,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CurrentModuleSelectionInputV2 {
    pub module_id: ModuleIdV2,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleDeclarationInputV2 {
    pub owner_module_id: ModuleIdV2,
    pub local_name: String,
    pub namespace_kind: NamespaceKindV2,
    pub visibility: VisibilityV2,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleImportInputV2 {
    pub owner_module_id: ModuleIdV2,
    pub local_name: String,
    pub target: CatalogAssetRefInputV2,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModuleReexportInputV2 {
    pub owner_module_id: ModuleIdV2,
    pub exported_name: String,
    pub target: CatalogAssetRefInputV2,
    pub location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedEdgeV2 {
    pub from: ModuleIdV2,
    pub to: ModuleIdV2,
    pub location: LocatedSpanV2,
}

impl Ord for LocatedEdgeV2 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.from
            .cmp(&other.from)
            .then(self.to.cmp(&other.to))
            .then(loc_key(self.location).cmp(&loc_key(other.location)))
    }
}
impl PartialOrd for LocatedEdgeV2 {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AmbiguousCandidateV2 {
    origin: CatalogAssetIdV2,
    declaration_location: LocatedSpanV2,
    witness: Vec<LocatedEdgeV2>,
}

impl AmbiguousCandidateV2 {
    pub(crate) fn from_catalog_reference(
        origin: CatalogAssetIdV2,
        declaration_location: LocatedSpanV2,
        witness: Vec<LocatedEdgeV2>,
    ) -> Self {
        Self {
            origin,
            declaration_location,
            witness,
        }
    }
    #[must_use]
    pub const fn origin(&self) -> &CatalogAssetIdV2 {
        &self.origin
    }
    #[must_use]
    pub const fn declaration_location(&self) -> LocatedSpanV2 {
        self.declaration_location
    }
    #[must_use]
    pub fn witness(&self) -> &[LocatedEdgeV2] {
        &self.witness
    }
}

#[derive(Clone, Debug)]
struct DeclarationV2 {
    id: CatalogAssetIdV2,
    visibility: VisibilityV2,
    location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug)]
struct EdgeV2 {
    target: CatalogAssetIdV2,
    location: CheckedLocatedSpanV2,
}

#[derive(Clone, Debug)]
struct ModuleNodeV2 {
    location: CheckedLocatedSpanV2,
    declarations: BTreeMap<(NamespaceKindV2, String), DeclarationV2>,
    imports: BTreeMap<(NamespaceKindV2, String), EdgeV2>,
    reexports: BTreeMap<(NamespaceKindV2, String), Vec<EdgeV2>>,
}

/// Low-level graph phase. Public callers use `ModuleAuthorityBuilderV2`.
#[derive(Clone, Debug)]
pub struct ModuleGraphBuilderV2 {
    modules: BTreeMap<ModuleIdV2, ModuleNodeV2>,
    registry_identity: Option<SourceRegistryIdentityV2>,
}

impl ModuleGraphBuilderV2 {
    #[must_use]
    pub(crate) fn new() -> Self {
        Self {
            modules: BTreeMap::new(),
            registry_identity: None,
        }
    }

    pub(crate) fn bind_registry(&mut self, identity: SourceRegistryIdentityV2) {
        self.registry_identity = Some(identity);
    }

    pub(crate) fn try_add_module(
        &mut self,
        input: ModuleNodeInputV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        session.charge_node()?;
        session.charge_string(input.module_id.as_str())?;
        self.check_location(&input.location)?;
        if let Some(original) = self.modules.get(&input.module_id) {
            return Err(ModuleDiagnosticV2::conflict(
                "modules.duplicate_module",
                format!("duplicate module `{}`", excerpt(input.module_id.as_str())),
                &input.location,
                &original.location,
            ));
        }
        self.modules.insert(
            input.module_id,
            ModuleNodeV2 {
                location: input.location,
                declarations: BTreeMap::new(),
                imports: BTreeMap::new(),
                reexports: BTreeMap::new(),
            },
        );
        Ok(())
    }

    pub(crate) fn try_declare(
        &mut self,
        input: ModuleDeclarationInputV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        session.charge_node()?;
        session.charge_string(input.owner_module_id.as_str())?;
        session.charge_string(&input.local_name)?;
        self.check_location(&input.location)?;
        let id = CatalogAssetIdV2::derive(
            input.owner_module_id.clone(),
            input.namespace_kind,
            &input.local_name,
            &input.location,
        )?;
        let node = self
            .modules
            .get_mut(&input.owner_module_id)
            .ok_or_else(|| {
                ModuleDiagnosticV2::new(
                    "modules.unknown_module",
                    "declaration owner is absent",
                    &input.location,
                )
            })?;
        let key = (input.namespace_kind, input.local_name);
        if let Some(original) = node.declarations.get(&key) {
            return Err(ModuleDiagnosticV2::conflict(
                "modules.duplicate_declaration",
                "duplicate module declaration",
                &input.location,
                &original.location,
            ));
        }
        node.declarations.insert(
            key,
            DeclarationV2 {
                id,
                visibility: input.visibility,
                location: input.location,
            },
        );
        Ok(())
    }

    pub(crate) fn try_import(
        &mut self,
        input: ModuleImportInputV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        session.charge_edge()?;
        session.charge_string(input.owner_module_id.as_str())?;
        session.charge_string(&input.local_name)?;
        self.check_location(&input.location)?;
        self.check_location(&input.target.location)?;
        session.charge_string(&input.target.local_name)?;
        session.charge_string(input.target.module_id.as_str())?;
        if !is_catalog_local_name(&input.local_name) {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_catalog_name",
                "invalid import alias",
                &input.location,
            ));
        }
        let target = input.target.derive()?;
        let node = self
            .modules
            .get_mut(&input.owner_module_id)
            .ok_or_else(|| {
                ModuleDiagnosticV2::new(
                    "modules.unknown_module",
                    "import owner is absent",
                    &input.location,
                )
            })?;
        let key = (target.namespace_kind, input.local_name);
        if let Some(original) = node.imports.get(&key) {
            return Err(ModuleDiagnosticV2::conflict(
                "modules.duplicate_import",
                "duplicate direct import alias",
                &input.location,
                &original.location,
            ));
        }
        node.imports.insert(
            key,
            EdgeV2 {
                target,
                location: input.location,
            },
        );
        Ok(())
    }

    pub(crate) fn try_reexport(
        &mut self,
        input: ModuleReexportInputV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        session.charge_edge()?;
        session.charge_string(input.owner_module_id.as_str())?;
        session.charge_string(&input.exported_name)?;
        self.check_location(&input.location)?;
        self.check_location(&input.target.location)?;
        session.charge_string(&input.target.local_name)?;
        session.charge_string(input.target.module_id.as_str())?;
        if !is_catalog_local_name(&input.exported_name) {
            return Err(ModuleDiagnosticV2::new(
                "modules.invalid_catalog_name",
                "invalid re-export name",
                &input.location,
            ));
        }
        let target = input.target.derive()?;
        let node = self
            .modules
            .get_mut(&input.owner_module_id)
            .ok_or_else(|| {
                ModuleDiagnosticV2::new(
                    "modules.unknown_module",
                    "re-export owner is absent",
                    &input.location,
                )
            })?;
        let key = (target.namespace_kind, input.exported_name);
        let edges = node.reexports.entry(key).or_default();
        if let Some(existing) = edges.iter_mut().find(|edge| edge.target == target) {
            if loc_key(input.location.location()) < loc_key(existing.location.location()) {
                existing.location = input.location;
            }
            return Ok(());
        }
        edges.push(EdgeV2 {
            target,
            location: input.location,
        });
        edges.sort_by(|left, right| {
            left.target
                .cmp(&right.target)
                .then(loc_key(left.location.location()).cmp(&loc_key(right.location.location())))
        });
        Ok(())
    }

    fn check_location(
        &mut self,
        location: &CheckedLocatedSpanV2,
    ) -> Result<(), ModuleDiagnosticV2> {
        match &self.registry_identity {
            None => self.registry_identity = Some(location.registry_identity().clone()),
            Some(identity) if identity == location.registry_identity() => {}
            Some(_) => {
                return Err(ModuleDiagnosticV2::new(
                    "modules.source_registry_mismatch",
                    "location belongs to another source registry",
                    location,
                ))
            }
        }
        Ok(())
    }

    pub(crate) fn resolve(
        &self,
        current: CurrentModuleSelectionInputV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<ResolvedGraphV2, ModuleDiagnosticV2> {
        session.charge_string(current.module_id.as_str())?;
        if self.registry_identity.as_ref() != Some(current.location.registry_identity()) {
            return Err(ModuleDiagnosticV2::new(
                "modules.source_registry_mismatch",
                "current-module location belongs to another registry",
                &current.location,
            ));
        }
        if !self.modules.contains_key(&current.module_id) {
            return Err(ModuleDiagnosticV2::new(
                "modules.unknown_module",
                format!(
                    "unknown current module `{}`",
                    excerpt(current.module_id.as_str())
                ),
                &current.location,
            ));
        }
        if let Some(cycle) = self.detect_cycle(session)? {
            let locations = cycle.iter().map(|edge| edge.location).collect::<Vec<_>>();
            let primary = cycle
                .first()
                .and_then(|edge| self.checked_edge_location(edge))
                .unwrap_or(&current.location);
            return Err(ModuleDiagnosticV2::new(
                "modules.import_cycle",
                "module dependency cycle",
                primary,
            )
            .with_witness(locations));
        }

        // Validate and resolve every component before selecting the current view.
        let mut all = BTreeMap::new();
        for module_id in self.modules.keys() {
            let bindings = self.resolve_module(module_id, session)?;
            all.insert(module_id.clone(), bindings);
        }
        let Some(visible) = all.get(&current.module_id).cloned() else {
            return Err(ModuleDiagnosticV2::new(
                "modules.internal_graph_invariant",
                "validated current module has no resolved binding table",
                &current.location,
            ));
        };
        Ok(ResolvedGraphV2 {
            current_module: current.module_id,
            visible: ResolvedModuleBindingsV2 { bindings: visible },
            all,
        })
    }

    fn checked_edge_location(&self, wanted: &LocatedEdgeV2) -> Option<&CheckedLocatedSpanV2> {
        let node = self.modules.get(&wanted.from)?;
        node.imports
            .values()
            .chain(node.reexports.values().flatten())
            .find(|edge| {
                edge.target.module_id == wanted.to && edge.location.location() == wanted.location
            })
            .map(|edge| &edge.location)
    }

    fn checked_declaration_location(
        &self,
        wanted: &CatalogAssetIdV2,
    ) -> Option<&CheckedLocatedSpanV2> {
        self.modules
            .get(&wanted.module_id)?
            .declarations
            .get(&(wanted.namespace_kind, wanted.local_name.clone()))
            .map(|declaration| &declaration.location)
    }

    fn ambiguity_primary<'a>(
        &'a self,
        candidates: &[AmbiguousCandidateV2],
    ) -> Option<&'a CheckedLocatedSpanV2> {
        let first = candidates.first()?;
        first
            .witness
            .last()
            .and_then(|edge| self.checked_edge_location(edge))
            .or_else(|| self.checked_declaration_location(&first.origin))
    }

    fn resolve_module(
        &self,
        module_id: &ModuleIdV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<BTreeMap<(NamespaceKindV2, String), ResolvedBindingV2>, ModuleDiagnosticV2> {
        let Some(node) = self.modules.get(module_id) else {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.unknown_module",
                "module disappeared during resolution",
            ));
        };
        let mut resolved = BTreeMap::new();
        for (key, edges) in &node.reexports {
            let mut candidates: BTreeMap<CatalogAssetIdV2, AmbiguousCandidateV2> = BTreeMap::new();
            for edge in edges {
                for candidate in self.resolve_origin(module_id, edge, session)? {
                    match candidates.get_mut(&candidate.origin) {
                        Some(existing) if candidate.witness < existing.witness => {
                            *existing = candidate
                        }
                        None => {
                            candidates.insert(candidate.origin.clone(), candidate);
                        }
                        _ => {}
                    }
                }
            }
            if candidates.len() > 1 {
                // Validate every re-export edge, but only the winning local
                // visible tier can make this module's name ambiguous. A local
                // declaration also owns the downstream export name (subject to
                // visibility); a direct import shadows only local lookup and is
                // absent from downstream origin resolution.
                if node.imports.contains_key(key) || node.declarations.contains_key(key) {
                    continue;
                }
                let values = candidates.into_values().collect::<Vec<_>>();
                let evidence = values
                    .iter()
                    .flat_map(|candidate| candidate.witness.iter().map(|edge| edge.location))
                    .collect();
                let Some(primary) = self.ambiguity_primary(&values) else {
                    return Err(ModuleDiagnosticV2::without_location(
                        "modules.ambiguous_import",
                        "ambiguous re-export has no checked witness",
                    ));
                };
                return Err(ModuleDiagnosticV2::new(
                    "modules.ambiguous_import",
                    format!("ambiguous binding `{}`", excerpt(&key.1)),
                    primary,
                )
                .with_witness(evidence)
                .with_ambiguous_candidates(values));
            }
            if let Some(candidate) = candidates.into_values().next() {
                resolved.insert(
                    key.clone(),
                    ResolvedBindingV2::from_candidate(
                        key.1.clone(),
                        candidate,
                        VisibilityV2::Public,
                    ),
                );
            }
        }
        // Direct imports replace re-export candidates only in the module's
        // local visible namespace. They never become transitive exports;
        // downstream resolution sees a local declaration first, then explicit
        // re-export edges.
        for (key, edge) in &node.imports {
            let mut candidates = self.resolve_origin(module_id, edge, session)?;
            if candidates.len() != 1 {
                let evidence = candidates
                    .iter()
                    .flat_map(|candidate| candidate.witness.iter().map(|item| item.location))
                    .collect();
                let diagnostic_candidates = candidates.clone();
                let primary = self
                    .ambiguity_primary(&diagnostic_candidates)
                    .unwrap_or(&edge.location);
                return Err(ModuleDiagnosticV2::new(
                    "modules.ambiguous_import",
                    format!("ambiguous binding `{}`", excerpt(&key.1)),
                    primary,
                )
                .with_witness(evidence)
                .with_ambiguous_candidates(diagnostic_candidates));
            }
            let Some(candidate) = candidates.pop() else {
                return Err(ModuleDiagnosticV2::new(
                    "modules.not_exported",
                    "target did not resolve to an exported declaration",
                    &edge.location,
                ));
            };
            resolved.insert(
                key.clone(),
                ResolvedBindingV2::from_candidate(key.1.clone(), candidate, VisibilityV2::Public),
            );
        }
        // Local declarations replace all imported forms.
        for (key, declaration) in &node.declarations {
            resolved.insert(
                key.clone(),
                ResolvedBindingV2 {
                    local_name: key.1.clone(),
                    resolved_asset_id: declaration.id.clone(),
                    namespace_kind: key.0,
                    visibility: declaration.visibility,
                    declaration_origin: declaration.id.clone(),
                    declaration_location: declaration.location.location(),
                    edge_witness: Vec::new(),
                },
            );
        }
        Ok(resolved)
    }

    fn resolve_origin(
        &self,
        importer: &ModuleIdV2,
        first_edge: &EdgeV2,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Vec<AmbiguousCandidateV2>, ModuleDiagnosticV2> {
        let first = LocatedEdgeV2 {
            from: importer.clone(),
            to: first_edge.target.module_id.clone(),
            location: first_edge.location.location(),
        };
        let mut stack = vec![(first_edge.target.clone(), vec![first])];
        let mut outcomes: BTreeMap<CatalogAssetIdV2, AmbiguousCandidateV2> = BTreeMap::new();
        while let Some((target, witness)) = stack.pop() {
            session.charge_step()?;
            let node = self.modules.get(&target.module_id).ok_or_else(|| {
                ModuleDiagnosticV2::new(
                    "modules.unknown_module",
                    format!("unknown module `{}`", excerpt(target.module_id.as_str())),
                    &first_edge.location,
                )
            })?;
            let key = (target.namespace_kind, target.local_name.clone());
            if let Some(declaration) = node.declarations.get(&key) {
                if !declaration.visibility.is_importable() && &target.module_id != importer {
                    return Err(ModuleDiagnosticV2::conflict(
                        "modules.not_exported",
                        "target declaration is private",
                        &first_edge.location,
                        &declaration.location,
                    ));
                }
                let candidate = AmbiguousCandidateV2 {
                    origin: declaration.id.clone(),
                    declaration_location: declaration.location.location(),
                    witness,
                };
                match outcomes.get_mut(&candidate.origin) {
                    Some(existing) if candidate.witness < existing.witness => *existing = candidate,
                    None => {
                        outcomes.insert(candidate.origin.clone(), candidate);
                    }
                    _ => {}
                }
                continue;
            }
            let Some(next_edges) = node.reexports.get(&key) else {
                return Err(ModuleDiagnosticV2::new(
                    "modules.not_exported",
                    "target symbol is absent or not exported",
                    &first_edge.location,
                ));
            };
            for next in next_edges.iter().rev() {
                let mut next_witness = witness.clone();
                next_witness.push(LocatedEdgeV2 {
                    from: target.module_id.clone(),
                    to: next.target.module_id.clone(),
                    location: next.location.location(),
                });
                stack.push((next.target.clone(), next_witness));
            }
        }
        Ok(outcomes.into_values().collect())
    }

    #[expect(
        clippy::indexing_slicing,
        reason = "indices are constructed from the same module-key snapshot"
    )]
    fn detect_cycle(
        &self,
        session: &mut ModuleBudgetSessionV2,
    ) -> Result<Option<Vec<LocatedEdgeV2>>, ModuleDiagnosticV2> {
        let nodes = self.modules.keys().collect::<Vec<_>>();
        let index = nodes
            .iter()
            .enumerate()
            .map(|(index, id)| ((*id).clone(), index))
            .collect::<BTreeMap<_, _>>();
        let graph = nodes
            .iter()
            .map(|id| {
                let node = &self.modules[*id];
                let mut edges = node
                    .imports
                    .values()
                    .chain(node.reexports.values().flatten())
                    .map(|edge| DependencyEdge {
                        target: index.get(&edge.target.module_id).copied(),
                        edge,
                    })
                    .collect::<Vec<_>>();
                edges.sort_by(|left, right| {
                    left.edge.target.cmp(&right.edge.target).then(
                        loc_key(left.edge.location.location())
                            .cmp(&loc_key(right.edge.location.location())),
                    )
                });
                edges
            })
            .collect::<Vec<_>>();

        if let Some(order) = topological_order(&graph) {
            // A root is visited once by itself and once for every path from
            // another root. This computes precisely the edge visits charged by
            // the former path-enumerating DFS, including edges to absent nodes.
            let mut visits = vec![1usize; nodes.len()];
            let mut steps = 0usize;
            let mut overflowed = false;
            for source in order {
                let edge_visits = visits[source]
                    .checked_mul(graph[source].len())
                    .unwrap_or_else(|| {
                        overflowed = true;
                        usize::MAX
                    });
                steps = steps.checked_add(edge_visits).unwrap_or_else(|| {
                    overflowed = true;
                    usize::MAX
                });
                for edge in &graph[source] {
                    if let Some(target) = edge.target {
                        visits[target] =
                            visits[target]
                                .checked_add(visits[source])
                                .unwrap_or_else(|| {
                                    overflowed = true;
                                    usize::MAX
                                });
                    }
                }
            }
            session.charge_steps(steps)?;
            if overflowed {
                // The true legacy count exceeds usize::MAX. Even a profile at
                // that numeric maximum must reject the next logical visit.
                session.charge_step()?;
            }
            return Ok(None);
        }

        // A cyclic graph still charges the legacy number of edge visits, so
        // callers observe the same resource limit before a cycle diagnostic.
        // Frames and membership are mutated in place instead of cloned for
        // every path; the budget bounds the number of loop iterations.
        let mut seen = vec![false; nodes.len()];
        for root in 0..nodes.len() {
            let mut stack = vec![(root, 0usize)];
            seen[root] = true;
            while let Some((node, next_edge)) = stack.last_mut() {
                if *next_edge == graph[*node].len() {
                    seen[*node] = false;
                    stack.pop();
                    continue;
                }
                let edge = &graph[*node][*next_edge];
                *next_edge += 1;
                session.charge_step()?;
                if let Some(target) = edge.target {
                    if !seen[target] {
                        seen[target] = true;
                        stack.push((target, 0));
                    }
                }
            }
        }

        Ok(minimal_cycle(&nodes, &graph))
    }
}

struct DependencyEdge<'a> {
    target: Option<usize>,
    edge: &'a EdgeV2,
}

#[expect(
    clippy::indexing_slicing,
    reason = "dependency targets are produced from this graph's node index"
)]
fn topological_order(graph: &[Vec<DependencyEdge<'_>>]) -> Option<Vec<usize>> {
    let mut indegree = vec![0usize; graph.len()];
    for edges in graph {
        for edge in edges {
            if let Some(target) = edge.target {
                indegree[target] += 1;
            }
        }
    }
    let mut ready = indegree
        .iter()
        .enumerate()
        .filter_map(|(node, count)| (*count == 0).then_some(node))
        .collect::<VecDeque<_>>();
    let mut order = Vec::with_capacity(graph.len());
    while let Some(node) = ready.pop_front() {
        order.push(node);
        for edge in &graph[node] {
            if let Some(target) = edge.target {
                indegree[target] -= 1;
                if indegree[target] == 0 {
                    ready.push_back(target);
                }
            }
        }
    }
    (order.len() == graph.len()).then_some(order)
}

#[expect(
    clippy::indexing_slicing,
    reason = "nodes and adjacency are built together with identical lengths"
)]
fn minimal_cycle(
    nodes: &[&ModuleIdV2],
    graph: &[Vec<DependencyEdge<'_>>],
) -> Option<Vec<LocatedEdgeV2>> {
    let labeled = graph
        .iter()
        .enumerate()
        .map(|(source, edges)| {
            edges
                .iter()
                .filter_map(|edge| {
                    edge.target
                        .map(|target| (target, located_edge(nodes[source], edge.edge)))
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    minimal_cycle_witness(&labeled)
}

/// Returns the lexicographically minimum canonical simple-cycle witness.
/// The indexed adjacency may be in any order; each label must carry the full
/// stable comparison key and the output payload for its edge. Equal labels on
/// edges leaving the same node must have the same target.
#[expect(
    clippy::indexing_slicing,
    reason = "all target indices are range-checked before indexed traversal"
)]
pub(crate) fn minimal_cycle_witness<K: Ord + Clone>(graph: &[Vec<(usize, K)>]) -> Option<Vec<K>> {
    if graph
        .iter()
        .flatten()
        .any(|(target, _)| *target >= graph.len())
    {
        return None;
    }
    // Kosaraju's two passes are iterative, including on narrow-stack hosts.
    let mut visited = vec![false; graph.len()];
    let mut finish = Vec::with_capacity(graph.len());
    for root in 0..graph.len() {
        if visited[root] {
            continue;
        }
        visited[root] = true;
        let mut stack = Vec::with_capacity(graph.len());
        stack.push((root, 0usize));
        while let Some((node, next_edge)) = stack.last_mut() {
            if *next_edge == graph[*node].len() {
                finish.push(*node);
                stack.pop();
                continue;
            }
            let target = graph[*node][*next_edge].0;
            *next_edge += 1;
            if !visited[target] {
                visited[target] = true;
                stack.push((target, 0));
            }
        }
    }
    let mut incoming = vec![0_usize; graph.len()];
    for edges in graph {
        for (target, _) in edges {
            incoming[*target] += 1;
        }
    }
    let mut reverse = Vec::with_capacity(graph.len());
    for count in incoming {
        reverse.push(Vec::with_capacity(count));
    }
    for (source, edges) in graph.iter().enumerate() {
        for edge in edges {
            reverse[edge.0].push(source);
        }
    }
    let mut component = vec![usize::MAX; graph.len()];
    let mut component_count = 0usize;
    for root in finish.into_iter().rev() {
        if component[root] != usize::MAX {
            continue;
        }
        let id = component_count;
        component_count += 1;
        let mut stack = Vec::with_capacity(graph.len());
        stack.push(root);
        component[root] = id;
        while let Some(node) = stack.pop() {
            for &next in &reverse[node] {
                if component[next] == usize::MAX {
                    component[next] = id;
                    stack.push(next);
                }
            }
        }
    }

    // The canonical rotation begins at the smallest labeled edge belonging to
    // any cycle. An edge belongs to a cycle exactly when its ends share an SCC.
    let mut first: Vec<(usize, usize)> = Vec::with_capacity(graph.iter().map(Vec::len).sum());
    for (source, edges) in graph.iter().enumerate() {
        for (edge_index, edge) in edges.iter().enumerate() {
            if component[source] != component[edge.0] {
                continue;
            }
            match first
                .first()
                .map(|&(best_source, best_index)| edge.1.cmp(&graph[best_source][best_index].1))
            {
                None | Some(std::cmp::Ordering::Less) => {
                    first.clear();
                    first.push((source, edge_index));
                }
                Some(std::cmp::Ordering::Equal) => first.push((source, edge_index)),
                Some(std::cmp::Ordering::Greater) => {}
            }
        }
    }
    if first.is_empty() {
        return None;
    }

    // Equal minimum labels can occur at different source nodes. Compare their
    // completed witnesses rather than depending on input adjacency order.
    let sorted = (0..graph.len())
        .map(|node| {
            let mut order = (0..graph[node].len()).collect::<Vec<_>>();
            // The index tie-break preserves the former stable input order
            // without allocating a stable-sort merge buffer.
            order.sort_unstable_by(|&left, &right| {
                graph[node][left]
                    .1
                    .cmp(&graph[node][right].1)
                    .then_with(|| left.cmp(&right))
            });
            order
        })
        .collect::<Vec<_>>();
    let mut best: Option<Vec<K>> = None;
    for (source, edge_index) in first {
        let Some(candidate) =
            cycle_from_edge(graph, &reverse, &component, &sorted, source, edge_index)
        else {
            continue;
        };
        if match &best {
            None => true,
            Some(current) => candidate < *current,
        } {
            best = Some(candidate);
        }
    }
    best
}

/// Sum of the live Vec capacities reachable through the indexed SCC witness
/// path. Each loop iteration drops its previous candidate and reachability
/// scratch, so one candidate and one reachability traversal cover the peak.
pub(crate) fn minimal_cycle_scratch_bytes_v3<K>(nodes: usize, edges: usize) -> Option<usize> {
    fn add<T>(total: &mut usize, count: usize) -> Option<()> {
        *total = total.checked_add(count.checked_mul(std::mem::size_of::<T>())?)?;
        Some(())
    }
    let mut total = 0_usize;
    add::<bool>(&mut total, nodes)?; // visited
    add::<usize>(&mut total, nodes)?; // finish
    add::<(usize, usize)>(&mut total, nodes)?; // DFS stack
    add::<usize>(&mut total, nodes)?; // incoming counts
    add::<Vec<usize>>(&mut total, nodes)?; // reverse headers
    add::<usize>(&mut total, edges)?; // reverse edges
    add::<usize>(&mut total, nodes)?; // SCC component
    add::<usize>(&mut total, nodes)?; // reverse DFS stack
    add::<(usize, usize)>(&mut total, edges)?; // equal-minimum edges
    add::<Vec<usize>>(&mut total, nodes)?; // sorted adjacency headers
    add::<usize>(&mut total, edges)?; // sorted adjacency indices
    add::<K>(&mut total, nodes)?; // retained best cycle
    add::<bool>(&mut total, nodes)?; // candidate seen
    add::<K>(&mut total, nodes)?; // candidate path
    add::<bool>(&mut total, nodes)?; // reaches_source
    add::<usize>(&mut total, nodes)?; // reachability pending
    Some(total)
}

#[expect(
    clippy::indexing_slicing,
    reason = "caller passes enumerated in-range nodes and edges after graph validation"
)]
fn cycle_from_edge<K: Ord + Clone>(
    graph: &[Vec<(usize, K)>],
    reverse: &[Vec<usize>],
    component: &[usize],
    sorted: &[Vec<usize>],
    source: usize,
    edge_index: usize,
) -> Option<Vec<K>> {
    let target = graph[source][edge_index].0;
    let first_edge = graph[source][edge_index].1.clone();
    if source == target {
        return Some(vec![first_edge]);
    }
    // Within one fixed first edge, reachability after removing the path prefix
    // identifies the lexicographically smallest simple completion.
    let mut seen = vec![false; graph.len()];
    seen[source] = true;
    seen[target] = true;
    let mut path = Vec::with_capacity(graph.len());
    path.push(first_edge);
    let mut current = target;
    while current != source {
        // Reachability after removing the path prefix answers whether an edge
        // can begin a simple completion. Recomputing it once per path vertex
        // gives polynomial work even for heavily branching cyclic graphs.
        let mut reaches_source = vec![false; graph.len()];
        reaches_source[source] = true;
        let mut pending = Vec::with_capacity(graph.len());
        pending.push(source);
        while let Some(node) = pending.pop() {
            for &previous in &reverse[node] {
                if component[previous] == component[source]
                    && !seen[previous]
                    && !reaches_source[previous]
                {
                    reaches_source[previous] = true;
                    pending.push(previous);
                }
            }
        }
        let choice = sorted[current].iter().find_map(|&edge_index| {
            let edge = &graph[current][edge_index];
            let next = edge.0;
            (component[next] == component[source] && reaches_source[next]).then_some(edge)
        })?;
        path.push(choice.1.clone());
        current = choice.0;
        seen[current] = true;
    }
    Some(path)
}

fn located_edge(source: &ModuleIdV2, edge: &EdgeV2) -> LocatedEdgeV2 {
    LocatedEdgeV2 {
        from: source.clone(),
        to: edge.target.module_id.clone(),
        location: edge.location.location(),
    }
}

/// One binding visible while resolving a specific module.
///
/// `visibility` is the resolved declaration's importability. A direct import
/// never becomes a downstream export. Downstream lookup resolves the target
/// module's own declaration first and otherwise follows explicit re-exports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedBindingV2 {
    local_name: String,
    resolved_asset_id: CatalogAssetIdV2,
    namespace_kind: NamespaceKindV2,
    visibility: VisibilityV2,
    declaration_origin: CatalogAssetIdV2,
    declaration_location: LocatedSpanV2,
    edge_witness: Vec<LocatedEdgeV2>,
}

impl ResolvedBindingV2 {
    fn from_candidate(
        local_name: String,
        candidate: AmbiguousCandidateV2,
        visibility: VisibilityV2,
    ) -> Self {
        Self {
            local_name,
            resolved_asset_id: candidate.origin.clone(),
            namespace_kind: candidate.origin.namespace_kind,
            visibility,
            declaration_origin: candidate.origin,
            declaration_location: candidate.declaration_location,
            edge_witness: candidate.witness,
        }
    }
    #[must_use]
    pub fn local_name(&self) -> &str {
        &self.local_name
    }
    #[must_use]
    pub const fn resolved_asset_id(&self) -> &CatalogAssetIdV2 {
        &self.resolved_asset_id
    }
    #[must_use]
    pub const fn namespace_kind(&self) -> NamespaceKindV2 {
        self.namespace_kind
    }
    #[must_use]
    pub const fn visibility(&self) -> VisibilityV2 {
        self.visibility
    }
    #[must_use]
    pub const fn declaration_origin(&self) -> &CatalogAssetIdV2 {
        &self.declaration_origin
    }
    #[must_use]
    pub const fn declaration_location(&self) -> LocatedSpanV2 {
        self.declaration_location
    }
    #[must_use]
    pub fn edge_witness(&self) -> &[LocatedEdgeV2] {
        &self.edge_witness
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedModuleBindingsV2 {
    bindings: BTreeMap<(NamespaceKindV2, String), ResolvedBindingV2>,
}
impl ResolvedModuleBindingsV2 {
    #[must_use]
    pub fn get(&self, kind: NamespaceKindV2, local_name: &str) -> Option<&ResolvedBindingV2> {
        self.bindings.get(&(kind, local_name.to_owned()))
    }
    pub fn iter(&self) -> impl Iterator<Item = &ResolvedBindingV2> {
        self.bindings.values()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedGraphV2 {
    current_module: ModuleIdV2,
    visible: ResolvedModuleBindingsV2,
    all: BTreeMap<ModuleIdV2, BTreeMap<(NamespaceKindV2, String), ResolvedBindingV2>>,
}
impl ResolvedGraphV2 {
    #[must_use]
    pub const fn current_module(&self) -> &ModuleIdV2 {
        &self.current_module
    }
    #[must_use]
    pub const fn visible_bindings(&self) -> &ResolvedModuleBindingsV2 {
        &self.visible
    }
    #[must_use]
    pub fn module_bindings(
        &self,
        module: &ModuleIdV2,
    ) -> Option<impl Iterator<Item = &ResolvedBindingV2>> {
        self.all.get(module).map(|bindings| bindings.values())
    }
}

pub(crate) fn is_segment(value: &str) -> bool {
    let mut bytes = value.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z' | b'A'..=b'Z' | b'_'))
        && bytes.all(|byte| matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'-'))
}

/// The source grammar's `GraphRefV1` path spelling, kept as one local asset
/// name under an independently selected module. In particular `::` is not a
/// module separator here. No case folding or separator rewriting is allowed.
pub(crate) fn is_catalog_local_name(value: &str) -> bool {
    fn segment(bytes: &[u8], mut at: usize, allow_digits: bool) -> Option<usize> {
        let first = *bytes.get(at)?;
        let ident = first.is_ascii_alphabetic() || first == b'_';
        if !(ident || allow_digits && first.is_ascii_digit()) {
            return None;
        }
        at += 1;
        while bytes.get(at).is_some_and(|byte| {
            if ident {
                byte.is_ascii_alphanumeric() || *byte == b'_'
            } else {
                byte.is_ascii_digit()
            }
        }) {
            at += 1;
        }
        Some(at)
    }

    let bytes = value.as_bytes();
    let Some(mut at) = segment(bytes, 0, true) else {
        return false;
    };
    while let Some(&separator) = bytes.get(at) {
        if !matches!(separator, b'.' | b'-' | b'/' | b':') {
            return false;
        }
        at += 1;
        if separator == b':' && bytes.get(at) == Some(&b':') {
            at += 1;
        }
        let allow_digits = matches!(separator, b'-' | b'/');
        let Some(next) = segment(bytes, at, allow_digits) else {
            return false;
        };
        at = next;
    }
    true
}

pub(crate) fn is_qualified_name(value: &str) -> bool {
    !value.is_empty() && value.split("::").all(is_segment)
}

pub(crate) fn excerpt(value: &str) -> String {
    let mut end = value.len().min(128);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

pub(crate) fn loc_key(location: LocatedSpanV2) -> (u32, usize, usize) {
    (
        location.source_id.raw(),
        location.span.start,
        location.span.end,
    )
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "bounded owner-local oracle and fixture construction assertions"
)]
#[path = "graph/cycle_tests.rs"]
mod cycle_tests;
