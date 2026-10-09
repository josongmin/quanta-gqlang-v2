//! Final catalog family-reference merge in authored diagnostic order.

use super::{
    ensure_registry, CatalogSealContext, DocumentCatalogDependencyV2,
    DocumentCatalogFamilyReferenceInputV2,
};
use crate::budget::ModuleBudgetSessionV2;
use crate::graph::loc_key;
use crate::ModuleDiagnosticV2;

pub(super) fn merge_family_references(
    family_references: Vec<DocumentCatalogFamilyReferenceInputV2>,
    context: &CatalogSealContext<'_>,
    session: &mut ModuleBudgetSessionV2,
) -> Result<Vec<DocumentCatalogDependencyV2>, ModuleDiagnosticV2> {
    let topology = context.topology;
    let predeclared = context.predeclared;
    let visible = context.visible;
    let external = context.external;
    let source_registry = context.source_registry;
    let session_key = context.session_key;
    let direct_edges = &topology.state().dependencies;
    let mut dependency_edges = Vec::with_capacity(direct_edges.len() + family_references.len());
    dependency_edges.extend_from_slice(direct_edges);
    // Keep authored validation order while looking up repeated identities in
    // logarithmic time. Direct topology edges precede family references.
    let mut edge_lookup: Vec<_> = direct_edges
        .iter()
        .enumerate()
        .map(|(index, edge)| (&edge.source_asset_id, &edge.target_asset_id, Some(index)))
        .chain(
            family_references
                .iter()
                .map(|reference| (reference.source.id(), reference.target.id(), None::<usize>)),
        )
        .collect();
    edge_lookup.sort_unstable_by(|left, right| {
        (left.0, left.1)
            .cmp(&(right.0, right.1))
            .then_with(|| right.2.cmp(&left.2))
    });
    for reference in &family_references {
        session.charge_edge()?;
        ensure_registry(&reference.location, source_registry)?;
        ensure_registry(&reference.source.location, source_registry)?;
        ensure_registry(&reference.target.location, source_registry)?;
        if !reference.source.belongs_to(session_key) || !reference.target.belongs_to(session_key) {
            return Err(ModuleDiagnosticV2::new(
                "modules.catalog_reference_session_mismatch",
                "family reference proof belongs to another document validator",
                &reference.location,
            ));
        }
        let source_id = reference.source.id.clone();
        if predeclared.get(&source_id) != Some(reference.family.source_kind())
            || predeclared.location(&source_id) != Some(&reference.source.location)
            || reference.source.kind != reference.family.source_kind()
        {
            return Err(ModuleDiagnosticV2::new(
                "modules.catalog_dependency_source_wrong_kind",
                "family reference source is not the matching predeclared local asset",
                &reference.location,
            ));
        }
        if reference.target.kind != reference.family.target_kind() {
            return Err(ModuleDiagnosticV2::new(
                "modules.catalog_reference_kind_mismatch",
                "family reference target kind differs from required kind",
                &reference.location,
            ));
        }
        let target_id = reference.target.id.clone();
        let target_exists = predeclared.get(&target_id) == Some(reference.target.kind)
            || external
                .asset(&target_id)
                .is_some_and(|asset| asset.kind() == reference.target.kind)
            || visible.visible_bindings().iter().any(|binding| {
                binding.resolved_asset_id() == &target_id
                    && binding.namespace_kind() == reference.target.kind
            });
        if !target_exists {
            return Err(ModuleDiagnosticV2::new(
                "modules.unknown_catalog_reference",
                "family reference target is absent from sealed document or host authority",
                &reference.location,
            ));
        }
        let edge = DocumentCatalogDependencyV2 {
            source_asset_id: source_id,
            target_asset_id: target_id,
            expected_target_kind: reference.family.target_kind(),
            family: Some(reference.family),
            location: reference.location.clone(),
        };
        let key = (&edge.source_asset_id, &edge.target_asset_id);
        let index = edge_lookup.partition_point(|existing| (existing.0, existing.1) < key);
        let Some(slot) = edge_lookup.get_mut(index) else {
            return Err(ModuleDiagnosticV2::without_location(
                "modules.internal_dependency_index_mismatch",
                "edge lookup is outside the sealed edge set",
            ));
        };
        match slot.2 {
            Some(index) => {
                let Some(existing) = dependency_edges.get_mut(index) else {
                    return Err(ModuleDiagnosticV2::without_location(
                        "modules.internal_dependency_index_mismatch",
                        "sorted dependency index is outside the sealed edge set",
                    ));
                };
                if existing.expected_target_kind != edge.expected_target_kind
                    || existing.family != edge.family
                {
                    return Err(ModuleDiagnosticV2::new(
                        "modules.catalog_dependency_kind_conflict",
                        "duplicate edge disagrees on expected kind",
                        &edge.location,
                    ));
                }
                if loc_key(edge.location.location()) < loc_key(existing.location.location()) {
                    *existing = edge;
                }
            }
            None => {
                slot.2 = Some(dependency_edges.len());
                dependency_edges.push(edge);
            }
        }
    }
    dependency_edges.sort_unstable_by(|left, right| {
        (&left.source_asset_id, &left.target_asset_id)
            .cmp(&(&right.source_asset_id, &right.target_asset_id))
    });
    Ok(dependency_edges)
}
