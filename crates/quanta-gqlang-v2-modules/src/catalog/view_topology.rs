//! Sealed local View dependency proof and dependency-first order.

use std::{cmp::Reverse, collections::BinaryHeap, sync::Arc};

use super::{
    ensure_registry, DocumentCatalogDependencyInputV2, DocumentCatalogDependencyV2,
    ExternalCatalogAssetV2, ExternalCatalogV2, PredeclaredDocumentAssetsV2,
};
use crate::budget::ModuleBudgetSessionV2;
use crate::graph::{loc_key, CatalogAssetIdV2, NamespaceKindV2};
use crate::manifest::cycle_with_legacy_steps;
use crate::session_identity::DocumentSessionIdentityV2;
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
    state: Arc<DocumentViewTopologyStateV2>,
    session_key: Arc<DocumentSessionIdentityV2>,
}

impl DocumentViewTopologyProofV2 {
    /// Local View IDs in the only materialization order admitted by Modules.
    pub fn dependency_first_order(&self) -> impl ExactSizeIterator<Item = &CatalogAssetIdV2> {
        self.state.dependency_first_order.iter()
    }

    pub(crate) fn state(&self) -> &DocumentViewTopologyStateV2 {
        &self.state
    }

    pub(crate) fn shared_state(&self) -> Arc<DocumentViewTopologyStateV2> {
        self.state.clone()
    }

    pub(crate) fn session_identity(&self) -> &Arc<DocumentSessionIdentityV2> {
        &self.session_key
    }

    pub(crate) fn belongs_to(&self, key: &Arc<DocumentSessionIdentityV2>) -> bool {
        Arc::ptr_eq(&self.session_key, key)
    }

    pub(crate) fn from_state(
        state: DocumentViewTopologyStateV2,
        session_key: Arc<DocumentSessionIdentityV2>,
    ) -> Self {
        Self {
            state: Arc::new(state),
            session_key,
        }
    }
}

/// Logical Vec and Arc capacity admitted before `try_new` can construct a
/// dependency, cycle scratch, diagnostic, or order. Counts come only from
/// the already-owned edge input and predeclared local catalog inventory.
pub(crate) struct TopologyRequestDemandV3 {
    pub(crate) retained_bytes: usize,
    pub(crate) scratch_bytes: usize,
}

pub(crate) fn request_topology_bytes_v3(
    inputs: &[DocumentCatalogDependencyInputV2],
    predeclared: &PredeclaredDocumentAssetsV2,
) -> Option<TopologyRequestDemandV3> {
    fn add<T>(total: &mut usize, count: usize) -> Option<()> {
        *total = total.checked_add(count.checked_mul(std::mem::size_of::<T>())?)?;
        Some(())
    }
    type BorrowedKey<'a> = (&'a CatalogAssetIdV2, (u32, usize, usize));
    let edges = inputs.len();
    let assets = predeclared.assets.len();
    let mut views = 0_usize;
    let mut copied_id_bytes = 0_usize;
    for (id, kind, _) in predeclared.iter_borrowed() {
        if kind == NamespaceKindV2::View {
            views = views.checked_add(1)?;
            copied_id_bytes = copied_id_bytes
                .checked_add(id.module_id().as_str().len())?
                .checked_add(id.local_name().len())?;
        }
    }
    // A filtered edge contributes at most one root and two indexed nodes.
    let cycle_nodes = edges.checked_mul(3)?;
    let mut retained = 0_usize;
    let mut scratch = 0_usize;
    add::<DocumentCatalogDependencyV2>(&mut retained, edges)?;
    add::<&DocumentCatalogDependencyV2>(&mut scratch, edges)?; // filtered
    add::<&CatalogAssetIdV2>(&mut scratch, edges)?; // roots
    add::<(
        &'static CatalogAssetIdV2,
        &'static CatalogAssetIdV2,
        BorrowedKey<'static>,
    )>(&mut scratch, edges)?;
    scratch = scratch.checked_add(crate::manifest::cycle_scratch_bytes_v3::<
        &'static CatalogAssetIdV2,
        BorrowedKey<'static>,
    >(edges, edges)?)?;
    add::<CheckedLocatedSpanV2>(&mut scratch, cycle_nodes)?; // cycle locations
    add::<quanta_gqlang_v2_hir::LocatedSpanV2>(&mut scratch, cycle_nodes)?; // diagnostic witness
    add::<&CatalogAssetIdV2>(&mut scratch, assets)?; // all local asset slots inspected
    add::<usize>(&mut scratch, views)?; // unresolved View dependency counts
    add::<(usize, usize)>(&mut scratch, edges)?; // dependents
    add::<Reverse<usize>>(&mut scratch, views)?; // ready View min-heap
    add::<CatalogAssetIdV2>(&mut retained, views)?; // retained order
    retained = retained.checked_add(copied_id_bytes)?;
    let state_layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<DocumentViewTopologyStateV2>())
        .ok()?
        .0
        .pad_to_align()
        .size();
    retained = retained.checked_add(state_layout)?;
    let shared_permit_layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<
            quanta_gqlang_v2_hir::FrontendRetainedStoragePermitV3,
        >())
        .ok()?
        .0
        .pad_to_align()
        .size();
    retained = retained.checked_add(shared_permit_layout)?;
    let longest_authored_name = inputs
        .iter()
        .flat_map(|edge| [&edge.source.local_name, &edge.target.local_name])
        .map(|name| name.len().min(128))
        .max()
        .unwrap_or(0);
    scratch = scratch.checked_add(crate::diagnostic::topology_diagnostic_fixed_bytes_v3(
        longest_authored_name,
    ))?;
    Some(TopologyRequestDemandV3 {
        retained_bytes: retained,
        scratch_bytes: scratch,
    })
}
impl DocumentViewTopologyProofV2 {
    pub(crate) fn try_new(
        inputs: Vec<DocumentCatalogDependencyInputV2>,
        predeclared: &PredeclaredDocumentAssetsV2,
        visible: &crate::ResolvedGraphV2,
        external: &ExternalCatalogV2,
        source_registry: &SourceRegistryViewV2,
        session: &mut ModuleBudgetSessionV2,
        session_key: Arc<DocumentSessionIdentityV2>,
    ) -> Result<Self, ModuleDiagnosticV2> {
        if !predeclared.belongs_to(&session_key) {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.catalog_reference_session_mismatch",
                "predeclaration belongs to another document validator",
            ));
        }
        let mut dependencies: Vec<DocumentCatalogDependencyV2> = Vec::with_capacity(inputs.len());
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
            let resolved = DocumentCatalogDependencyV2 {
                source_asset_id,
                target_asset_id,
                expected_target_kind: input.expected_target_kind,
                family: None,
                location: input.location,
            };
            match dependencies.binary_search_by(|existing| {
                (&existing.source_asset_id, &existing.target_asset_id)
                    .cmp(&(&resolved.source_asset_id, &resolved.target_asset_id))
            }) {
                Ok(index) => {
                    let Some(existing) = dependencies.get_mut(index) else {
                        return Err(ModuleDiagnosticV2::without_location(
                            "modules.internal_dependency_index_mismatch",
                            "sorted dependency index is outside the sealed edge set",
                        ));
                    };
                    if existing.expected_target_kind != resolved.expected_target_kind
                        || existing.family != resolved.family
                    {
                        return Err(ModuleDiagnosticV2::new(
                            "modules.catalog_dependency_kind_conflict",
                            "duplicate edge disagrees on expected kind",
                            &resolved.location,
                        ));
                    }
                    if loc_key(resolved.location.location()) < loc_key(existing.location.location())
                    {
                        *existing = resolved;
                    }
                }
                Err(index) => dependencies.insert(index, resolved),
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
                dependencies,
                dependency_first_order,
            },
            session_key,
        ))
    }
}

type PredeclaredAssetSlotV2 = (
    CatalogAssetIdV2,
    NamespaceKindV2,
    CheckedLocatedSpanV2,
    usize,
);

fn local_kind(assets: &[PredeclaredAssetSlotV2], id: &CatalogAssetIdV2) -> Option<NamespaceKindV2> {
    assets
        .binary_search_by(|(candidate, _, _, _)| candidate.cmp(id))
        .ok()
        .and_then(|index| assets.get(index))
        .map(|(_, kind, _, _)| *kind)
}

fn document_cycle(
    edges: &[DocumentCatalogDependencyV2],
    assets: &[PredeclaredAssetSlotV2],
    session: &mut ModuleBudgetSessionV2,
) -> Result<Option<Vec<CheckedLocatedSpanV2>>, ModuleDiagnosticV2> {
    let mut filtered = Vec::with_capacity(edges.len());
    for edge in edges {
        if local_kind(assets, &edge.source_asset_id) == Some(NamespaceKindV2::View)
            && local_kind(assets, &edge.target_asset_id) == Some(NamespaceKindV2::View)
        {
            filtered.push(edge);
        }
    }
    let mut roots = filtered
        .iter()
        .map(|edge| &edge.source_asset_id)
        .collect::<Vec<_>>();
    roots.sort_unstable();
    roots.dedup();
    let keyed = filtered
        .iter()
        .map(|edge| {
            (
                &edge.source_asset_id,
                &edge.target_asset_id,
                (&edge.target_asset_id, loc_key(edge.location.location())),
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
    edges: &[DocumentCatalogDependencyV2],
    assets: &[PredeclaredAssetSlotV2],
    session: &mut ModuleBudgetSessionV2,
) -> Result<Vec<CatalogAssetIdV2>, ModuleDiagnosticV2> {
    let mut views = Vec::with_capacity(assets.len());
    for (id, kind, _, _) in assets {
        if *kind == NamespaceKindV2::View {
            views.push(id);
        }
    }
    let mut unresolved_dependencies = vec![0_usize; views.len()];
    let mut dependents = Vec::<(usize, usize)>::with_capacity(edges.len());
    for edge in edges {
        let source = views.binary_search(&&edge.source_asset_id);
        let target = views.binary_search(&&edge.target_asset_id);
        if let (Ok(source), Ok(target)) = (source, target) {
            session.charge_step()?;
            let Some(count) = unresolved_dependencies.get_mut(source) else {
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
            dependents.push((target, source));
        }
    }
    dependents.sort_unstable();
    // Unlike the private cycle preflight FIFO, this order is sealed output:
    // always select the smallest ready View ID, without shifting a sorted Vec.
    let mut ready = BinaryHeap::with_capacity(views.len());
    for (index, count) in unresolved_dependencies.iter().enumerate() {
        if *count == 0 {
            ready.push(Reverse(index));
        }
    }
    let mut order = Vec::with_capacity(views.len());
    while let Some(Reverse(index)) = ready.pop() {
        session.charge_step()?;
        let Some(view) = views.get(index) else {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.view_topology_proof_mismatch",
                "ready View index is outside the local View set",
            ));
        };
        order.push((**view).clone());
        let first = dependents.partition_point(|(target, _)| *target < index);
        let Some(pending_dependents) = dependents.get(first..) else {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.view_topology_proof_mismatch",
                "ready View index is outside the dependency set",
            ));
        };
        for (_, dependent) in pending_dependents
            .iter()
            .take_while(|(target, _)| *target == index)
        {
            session.charge_step()?;
            let Some(count) = unresolved_dependencies.get_mut(*dependent) else {
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
                ready.push(Reverse(*dependent));
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
