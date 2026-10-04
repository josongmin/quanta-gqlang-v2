//! Request-owned final catalog retained and scratch capacity census.

use super::{
    reachable_profile_slots_v3, DocumentCatalogDependencyV2, DocumentCatalogFinalInputV2,
    ExternalCatalogAssetV2, ExternalCatalogV2, ProfileAdmissionV2,
};
use crate::catalog::view_topology::DocumentViewTopologyProofV2;
use crate::graph::CatalogAssetIdV2;
use quanta_gqlang_v2_hir::CheckedLocatedSpanV2;

pub(crate) struct FinalCatalogRequestDemandV3 {
    pub(crate) retained_bytes: usize,
    pub(crate) scratch_bytes: usize,
}

/// Source-indexed final catalog capacity before activation, profile,
/// dependency, reachable-set, or rich diagnostic construction.
pub(crate) fn request_final_catalog_bytes_v3(
    input: &DocumentCatalogFinalInputV2,
    topology: &DocumentViewTopologyProofV2,
    external: &ExternalCatalogV2,
) -> Option<FinalCatalogRequestDemandV3> {
    fn add<T>(total: &mut usize, count: usize) -> Option<()> {
        *total = total.checked_add(count.checked_mul(std::mem::size_of::<T>())?)?;
        Some(())
    }
    let dependency_slots = topology
        .state()
        .dependencies
        .len()
        .checked_add(input.family_references.len())?;
    let mut retained = 0_usize;
    let mut scratch = 0_usize;
    add::<(crate::ValidatedPackActivationV2, CheckedLocatedSpanV2)>(
        &mut retained,
        input.activations.len(),
    )?;
    add::<(CatalogAssetIdV2, ProfileAdmissionV2, CheckedLocatedSpanV2)>(
        &mut retained,
        input.profile_admissions.len(),
    )?;
    add::<DocumentCatalogDependencyV2>(&mut retained, dependency_slots)?;
    for activation in &input.activations {
        retained = retained.checked_add(activation.pack_id.as_str().len())?;
    }
    for profile in &input.profile_admissions {
        retained = retained
            .checked_add(profile.profile.module_id.as_str().len())?
            .checked_add(profile.profile.local_name.len())?;
    }
    for dependency in &topology.state().dependencies {
        for id in [&dependency.source_asset_id, &dependency.target_asset_id] {
            retained = retained
                .checked_add(id.module_id().as_str().len())?
                .checked_add(id.local_name().len())?;
        }
    }
    for reference in &input.family_references {
        for id in [reference.source.id(), reference.target.id()] {
            retained = retained
                .checked_add(id.module_id().as_str().len())?
                .checked_add(id.local_name().len())?;
        }
    }
    // The final output shares the predeclared Arc. Its V3 custody retains the
    // original charge through the last final-catalog reader, so no duplicate
    // allocation or charge is introduced here.
    if !input.require_certified_directives.is_empty() {
        let (queue_slots, profile_slots) = reachable_profile_slots_v3(
            dependency_slots,
            external,
            input
                .activations
                .iter()
                .map(|activation| &activation.pack_id),
        )?;
        add::<&CatalogAssetIdV2>(&mut scratch, queue_slots)?; // queue
        add::<&CatalogAssetIdV2>(&mut scratch, queue_slots)?; // seen
        add::<&ExternalCatalogAssetV2>(&mut scratch, profile_slots)?;
    }
    let longest_excerpt = input
        .activations
        .iter()
        .map(|activation| activation.lock_reference.path.len())
        .chain(
            input
                .profile_admissions
                .iter()
                .map(|profile| profile.profile.local_name.len()),
        )
        .chain(external.iter().map(|asset| asset.id.local_name().len()))
        .map(|length| length.min(128))
        .max()
        .unwrap_or(0);
    scratch = scratch.checked_add(crate::diagnostic::final_catalog_diagnostic_bytes_v3(
        longest_excerpt,
    ))?;
    Some(FinalCatalogRequestDemandV3 {
        retained_bytes: retained,
        scratch_bytes: scratch,
    })
}
