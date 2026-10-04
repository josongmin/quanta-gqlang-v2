//! Request-owned catalog reference lookup and retained diagnostic proof.

use super::{DocumentCatalogPhaseV2, DocumentCatalogRequestStageV3, DocumentCatalogValidatorV2};
use crate::catalog::{
    DocumentCatalogReferenceInputV2, DocumentCatalogReferenceOriginV2,
    ResolvedDocumentCatalogReferenceV2,
};
use crate::graph::{CatalogAssetIdV2, NamespaceKindV2};
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3,
    FrontendStorageRefusalV3, SourceIdV2,
};

pub(super) type ReferenceCandidateV3<'a> = (
    &'a CatalogAssetIdV2,
    NamespaceKindV2,
    DocumentCatalogReferenceOriginV2,
    CheckedLocatedSpanV2,
    &'a [crate::LocatedEdgeV2],
);

/// Selected catalog proof with an independently retained source-root charge.
/// The cloneable V2 proof cannot leave this carrier.
pub struct AdmittedDocumentReferenceV3 {
    pub(super) value: ResolvedDocumentCatalogReferenceV2,
    pub(super) _permit: FrontendRetainedStoragePermitV3,
}

impl AdmittedDocumentReferenceV3 {
    #[must_use]
    pub fn id(&self) -> &CatalogAssetIdV2 {
        self.value.id()
    }

    #[must_use]
    pub fn kind(&self) -> NamespaceKindV2 {
        self.value.kind()
    }

    #[must_use]
    pub fn origin(&self) -> DocumentCatalogReferenceOriginV2 {
        self.value.origin()
    }

    #[must_use]
    pub fn location(&self) -> &CheckedLocatedSpanV2 {
        self.value.location()
    }
}

impl std::fmt::Debug for AdmittedDocumentReferenceV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdmittedDocumentReferenceV3")
            .field("id", self.id())
            .field("kind", &self.kind())
            .field("origin", &self.origin())
            .finish_non_exhaustive()
    }
}

pub struct AdmittedDocumentReferenceDiagnosticV3 {
    diagnostic: ModuleDiagnosticV2,
    _scratch: FrontendRetainedStoragePermitV3,
}

impl AdmittedDocumentReferenceDiagnosticV3 {
    #[must_use]
    pub fn reason_code(&self) -> &'static str {
        self.diagnostic.code()
    }
}

impl std::fmt::Debug for AdmittedDocumentReferenceDiagnosticV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdmittedDocumentReferenceDiagnosticV3")
            .field("reason_code", &self.reason_code())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum DocumentReferenceRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    Phase,
    SourceMismatch,
    DemandOverflow,
    FallibleAllocation,
    Resource {
        dimension: &'static str,
        limit: usize,
        observed: usize,
    },
    Domain(AdmittedDocumentReferenceDiagnosticV3),
}

impl<'a> DocumentCatalogValidatorV2<'a> {
    /// Resolve a request reference using the validator's original retained
    /// source root. The upper bound follows the exact matching local, visible,
    /// and external inventories, including the largest diagnostic branch.
    pub fn resolve_reference_if_present_with_request_owner_v3(
        &mut self,
        input: DocumentCatalogReferenceInputV2,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        permit: &mut FrontendRetainedStoragePermitV3,
    ) -> Result<Option<AdmittedDocumentReferenceV3>, DocumentReferenceRequestErrorV3> {
        permit
            .validate_owner_v3(root, source_id, source)
            .map_err(DocumentReferenceRequestErrorV3::Storage)?;
        if !self.references_are_open()
            || self.predeclared.is_none()
            || !matches!(
                self.request_stage,
                DocumentCatalogRequestStageV3::Predeclared
                    | DocumentCatalogRequestStageV3::TopologyValidated
            )
        {
            return Err(DocumentReferenceRequestErrorV3::Phase);
        }
        self.session_key
            .validate_owner_v3(root, source_id, source)
            .map_err(DocumentReferenceRequestErrorV3::Storage)?;
        if input.location.location().source_id != source_id {
            return Err(DocumentReferenceRequestErrorV3::SourceMismatch);
        }
        let prior_phase = self.phase;
        self.phase = DocumentCatalogPhaseV2::Failed;
        let predeclared = self
            .predeclared
            .as_ref()
            .ok_or(DocumentReferenceRequestErrorV3::Phase)?;
        let visible = self.parts.resolved_graph.visible_bindings();
        let current_module = self.parts.resolved_graph.current_module();
        let selected = input.module_id.as_ref().unwrap_or(current_module);
        let visible_count = if selected == current_module {
            visible.iter().count()
        } else {
            0
        };
        let steps = predeclared
            .iter_borrowed()
            .count()
            .checked_add(visible_count)
            .and_then(|count| count.checked_add(self.parts.external_catalog.iter().count()))
            .and_then(|count| count.checked_add(1))
            .ok_or(DocumentReferenceRequestErrorV3::DemandOverflow)?;
        self.session
            .preflight_reference_v3(&input.local_name, steps)
            .map_err(|error| DocumentReferenceRequestErrorV3::Resource {
                dimension: error.dimension,
                limit: error.limit,
                observed: error.observed,
            })?;
        let mut slots = 0_usize;
        let mut id_bytes = 0_usize;
        let mut witness_slots = 0_usize;
        let mut witness_bytes = 0_usize;
        let mut add = |id: &CatalogAssetIdV2, witness: &[crate::LocatedEdgeV2]| {
            slots = slots.checked_add(1)?;
            id_bytes = id_bytes
                .checked_add(id.module_id().as_str().len())?
                .checked_add(id.local_name().len())?;
            witness_slots = witness_slots.checked_add(witness.len())?;
            for edge in witness {
                witness_bytes = witness_bytes
                    .checked_add(edge.from.as_str().len())?
                    .checked_add(edge.to.as_str().len())?;
            }
            Some(())
        };
        for (id, _, _) in predeclared.iter_borrowed() {
            if id.module_id() == selected && id.local_name() == input.local_name {
                add(id, &[]).ok_or(DocumentReferenceRequestErrorV3::DemandOverflow)?;
            }
        }
        if selected == current_module {
            for binding in visible.iter() {
                if binding.local_name() == input.local_name {
                    add(binding.resolved_asset_id(), binding.edge_witness())
                        .ok_or(DocumentReferenceRequestErrorV3::DemandOverflow)?;
                }
            }
        }
        for asset in self.parts.external_catalog.iter() {
            if asset.id().module_id() == selected && asset.id().local_name() == input.local_name {
                add(asset.id(), &[]).ok_or(DocumentReferenceRequestErrorV3::DemandOverflow)?;
            }
        }
        let scratch_bytes = slots
            .checked_mul(std::mem::size_of::<ReferenceCandidateV3<'static>>())
            .and_then(|count| {
                slots
                    .checked_mul(std::mem::size_of::<crate::AmbiguousCandidateV2>())
                    .and_then(|ambiguous| count.checked_add(ambiguous))
            })
            .and_then(|count| count.checked_add(id_bytes))
            .and_then(|count| {
                witness_slots
                    .checked_mul(std::mem::size_of::<crate::LocatedEdgeV2>())
                    .and_then(|witness| count.checked_add(witness))
            })
            .and_then(|count| count.checked_add(witness_bytes))
            .and_then(|count| {
                count.checked_add(crate::diagnostic::reference_diagnostic_fixed_bytes_v3())
            })
            .ok_or(DocumentReferenceRequestErrorV3::DemandOverflow)?;
        let selected_permit = root
            .reserve_v3(source_id, source, id_bytes)
            .map_err(DocumentReferenceRequestErrorV3::Storage)?;
        let scratch = root
            .reserve_v3(source_id, source, scratch_bytes)
            .map_err(DocumentReferenceRequestErrorV3::Storage)?;
        let mut candidates = Vec::new();
        candidates
            .try_reserve_exact(slots)
            .map_err(|_allocation_error| DocumentReferenceRequestErrorV3::FallibleAllocation)?;
        if candidates.capacity() > slots {
            return Err(DocumentReferenceRequestErrorV3::FallibleAllocation);
        }
        match self.resolve_reference_inner(input, candidates) {
            Ok(reference) => {
                drop(scratch);
                self.phase = prior_phase;
                Ok(reference.map(|value| AdmittedDocumentReferenceV3 {
                    value,
                    _permit: selected_permit,
                }))
            }
            Err(diagnostic) => Err(DocumentReferenceRequestErrorV3::Domain(
                AdmittedDocumentReferenceDiagnosticV3 {
                    diagnostic,
                    _scratch: scratch,
                },
            )),
        }
    }
}
