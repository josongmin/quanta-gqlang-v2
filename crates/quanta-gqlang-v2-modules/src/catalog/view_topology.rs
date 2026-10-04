//! Sealed local View dependency proof and dependency-first order.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::{
    ensure_registry, DocumentCatalogDependencyInputV2, DocumentCatalogDependencyV2,
    ExternalCatalogAssetV2, ExternalCatalogV2, PredeclaredDocumentAssetsV2,
};
use crate::budget::ModuleBudgetSessionV2;
use crate::graph::{loc_key, CatalogAssetIdV2, NamespaceKindV2};
use crate::manifest::cycle_with_legacy_steps;
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::{CheckedLocatedSpanV2, SourceRegistryViewV2};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentViewTopologyStateV2 {
    pub(crate) dependencies: Vec<DocumentCatalogDependencyV2>,
    pub(crate) dependency_first_order: Vec<CatalogAssetIdV2>,
}

/// Opaque proof that every direct document View dependency was resolved and
/// that the local View graph is acyclic. The order is dependency-first and is
/// stable under authored asset/edge permutation.
#[derive(Debug)]
pub struct DocumentViewTopologyProofV2 {
    state: DocumentViewTopologyStateV2,
    session_key: Arc<()>,
}

impl DocumentViewTopologyProofV2 {
    /// Local View IDs in the only materialization order admitted by Modules.
    pub fn dependency_first_order(&self) -> impl ExactSizeIterator<Item = &CatalogAssetIdV2> {
        self.state.dependency_first_order.iter()
    }

    pub(crate) fn state(&self) -> &DocumentViewTopologyStateV2 {
        &self.state
    }

    pub(crate) fn belongs_to(&self, key: &Arc<()>) -> bool {
        Arc::ptr_eq(&self.session_key, key)
    }

    pub(crate) fn from_state(state: DocumentViewTopologyStateV2, session_key: Arc<()>) -> Self {
        Self { state, session_key }
    }
}
impl DocumentViewTopologyProofV2 {
    pub(crate) fn try_new(
        inputs: Vec<DocumentCatalogDependencyInputV2>,
        predeclared: &PredeclaredDocumentAssetsV2,
        visible: &crate::ResolvedGraphV2,
        external: &ExternalCatalogV2,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
        session_key: Arc<()>,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if !predeclared.belongs_to(&session_key) {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.catalog_reference_session_mismatch",
                "predeclaration belongs to another document validator",
            ));
        }
        let mut dependencies: BTreeMap<
            (CatalogAssetIdV2, CatalogAssetIdV2),
            DocumentCatalogDependencyV2,
        > = BTreeMap::new();
        for input in inputs {
            session.charge_edge()?;
            session.charge_string(input.source.module_id.as_str())?;
            session.charge_string(&input.source.local_name)?;
            session.charge_string(input.target.module_id.as_str())?;
            session.charge_string(&input.target.local_name)?;
            ensure_registry(&input.location, source_registry)?;
            ensure_registry(&input.source.location, source_registry)?;
            ensure_registry(&input.target.location, source_registry)?;
            if input.target.namespace_kind != input.expected_target_kind {
                return Err(ModuleDiagnosticV2::new(
                    "modules.catalog_reference_kind_mismatch",
                    "target reference kind differs from expected kind",
                    &input.location,
                ));
            }
            let source_asset_id = input.source.derive()?;
            let target_asset_id = input.target.derive()?;
            if predeclared.get(&source_asset_id) != Some(NamespaceKindV2::View) {
                return Err(ModuleDiagnosticV2::new(
                    "modules.catalog_dependency_source_wrong_kind",
                    "document dependency source must be a local View",
                    &input.location,
                ));
            }
            let local_target = predeclared.get(&target_asset_id);
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
                    &input.location,
                ));
            };
            if actual != input.expected_target_kind {
                let target_location = if let Some(location) = predeclared.location(&target_asset_id)
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
                                &input.location,
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
                    &input.location,
                    &target_location,
                ));
            }
            let key = (source_asset_id.clone(), target_asset_id.clone());
            let resolved = DocumentCatalogDependencyV2 {
                source_asset_id,
                target_asset_id,
                expected_target_kind: input.expected_target_kind,
                family: None,
                location: input.location,
            };
            match dependencies.get_mut(&key) {
                Some(existing)
                    if existing.expected_target_kind != resolved.expected_target_kind
                        || existing.family != resolved.family =>
                {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.catalog_dependency_kind_conflict",
                        "duplicate edge disagrees on expected kind",
                        &resolved.location,
                    ));
                }
                Some(existing)
                    if loc_key(resolved.location.location())
                        < loc_key(existing.location.location()) =>
                {
                    *existing = resolved;
                }
                Some(_) => {}
                None => {
                    dependencies.insert(key, resolved);
                }
            }
        }
        if let Some(cycle) = document_cycle(&dependencies, &predeclared.assets, session)? {
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
        let dependency_first_order =
            dependency_first_view_order(&dependencies, &predeclared.assets, session)?;
        Ok(Self::from_state(
            DocumentViewTopologyStateV2 {
                dependencies: dependencies.into_values().collect(),
                dependency_first_order,
            },
            session_key,
        ))
    }
}

fn document_cycle(
    edges: &BTreeMap<(CatalogAssetIdV2, CatalogAssetIdV2), DocumentCatalogDependencyV2>,
    assets: &BTreeMap<CatalogAssetIdV2, NamespaceKindV2>,
    session: &mut ModuleBudgetSessionV2,
) -> Result<Option<Vec<CheckedLocatedSpanV2>>, ModuleDiagnosticV2> {
    let filtered = edges
        .values()
        .filter(|edge| {
            assets.get(&edge.source_asset_id) == Some(&NamespaceKindV2::View)
                && assets.get(&edge.target_asset_id) == Some(&NamespaceKindV2::View)
        })
        .collect::<Vec<_>>();
    let roots = filtered
        .iter()
        .map(|edge| edge.source_asset_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let keyed = filtered
        .iter()
        .map(|edge| {
            (
                edge.source_asset_id.clone(),
                edge.target_asset_id.clone(),
                (
                    edge.target_asset_id.clone(),
                    loc_key(edge.location.location()),
                ),
            )
        })
        .collect::<Vec<_>>();
    let Some(indices) = cycle_with_legacy_steps(&keyed, &roots, session)? else {
        return Ok(None);
    };
    let mut witness = Vec::with_capacity(indices.len());
    for index in indices {
        let Some(edge) = filtered.get(index) else {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.internal_cycle_witness_mismatch",
                "document cycle witness index is outside the validated edge set",
            ));
        };
        witness.push(edge.location.clone());
    }
    Ok(Some(witness))
}

fn dependency_first_view_order(
    edges: &BTreeMap<(CatalogAssetIdV2, CatalogAssetIdV2), DocumentCatalogDependencyV2>,
    assets: &BTreeMap<CatalogAssetIdV2, NamespaceKindV2>,
    session: &mut ModuleBudgetSessionV2,
) -> Result<Vec<CatalogAssetIdV2>, ModuleDiagnosticV2> {
    let views = assets
        .iter()
        .filter_map(|(id, kind)| (*kind == NamespaceKindV2::View).then_some(id.clone()))
        .collect::<BTreeSet<_>>();
    let mut unresolved_dependencies = views
        .iter()
        .cloned()
        .map(|id| (id, 0_usize))
        .collect::<BTreeMap<_, _>>();
    let mut dependents: BTreeMap<CatalogAssetIdV2, BTreeSet<CatalogAssetIdV2>> = BTreeMap::new();
    for edge in edges.values() {
        if views.contains(&edge.source_asset_id) && views.contains(&edge.target_asset_id) {
            session.charge_step()?;
            let Some(count) = unresolved_dependencies.get_mut(&edge.source_asset_id) else {
                return Err(ModuleDiagnosticV2::without_location(
                    "modules.view_topology_proof_mismatch",
                    "resolved View edge has an unknown local source",
                ));
            };
            *count = count.checked_add(1).ok_or_else(|| {
                ModuleDiagnosticV2::without_location(
                    "modules.resource_exhausted",
                    "View dependency count overflowed",
                )
            })?;
            dependents
                .entry(edge.target_asset_id.clone())
                .or_default()
                .insert(edge.source_asset_id.clone());
        }
    }
    let mut ready = unresolved_dependencies
        .iter()
        .filter_map(|(id, count)| (*count == 0).then_some(id.clone()))
        .collect::<BTreeSet<_>>();
    let mut order = Vec::with_capacity(views.len());
    while let Some(id) = ready.pop_first() {
        session.charge_step()?;
        order.push(id.clone());
        for dependent in dependents.get(&id).into_iter().flatten() {
            session.charge_step()?;
            let Some(count) = unresolved_dependencies.get_mut(dependent) else {
                return Err(ModuleDiagnosticV2::without_location(
                    "modules.view_topology_proof_mismatch",
                    "resolved View edge has an unknown local target",
                ));
            };
            *count = count.checked_sub(1).ok_or_else(|| {
                ModuleDiagnosticV2::without_location(
                    "modules.view_topology_proof_mismatch",
                    "View dependency count underflowed",
                )
            })?;
            if *count == 0 {
                ready.insert(dependent.clone());
            }
        }
    }
    if order.len() != views.len() {
        return Err(ModuleDiagnosticV2::without_location(
            "modules.view_topology_proof_mismatch",
            "acyclic View graph did not produce a complete topological order",
        ));
    }
    Ok(order)
}
