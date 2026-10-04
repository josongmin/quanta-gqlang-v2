//! Request-owned topology and final catalog transaction.

use std::sync::Arc;

use super::reference_request::AdmittedDocumentReferenceV3;
use super::{DocumentCatalogPhaseV2, DocumentCatalogRequestStageV3, DocumentCatalogValidatorV2};
use crate::catalog::{
    request_final_catalog_bytes_v3, request_topology_bytes_v3, DocumentCatalogAuthorityV2,
    DocumentCatalogFamilyReferenceInputV2, DocumentCatalogFinalInputV2,
    DocumentCatalogReferenceFamilyV2, DocumentViewTopologyProofV2, PredeclaredDocumentAssetV2,
    ProfileAdmissionInputV2, RequireCertifiedInputV2,
};
use crate::graph::CatalogAssetIdV2;
use crate::session_identity::DocumentSessionIdentityV2;
use crate::ModuleDiagnosticV2;
use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3,
    FrontendStorageRefusalV3, SourceIdV2, SourceRegistryIdentityV2,
};

/// Topology proof and the source-root charge shared with its validator. The
/// raw V2 proof remains private to this request API.
pub struct AdmittedDocumentViewTopologyProofV3 {
    proof: DocumentViewTopologyProofV2,
    _storage: Arc<FrontendRetainedStoragePermitV3>,
}

/// Retained link to the exact validator that issued a body topology. Value
/// equality of catalog facts cannot substitute for this session identity.
pub struct DocumentCatalogSessionWitnessV3 {
    session_key: Arc<DocumentSessionIdentityV2>,
    _topology_storage: Arc<FrontendRetainedStoragePermitV3>,
}

impl DocumentCatalogSessionWitnessV3 {
    #[must_use]
    pub fn matches_topology_v3(&self, topology: &AdmittedDocumentViewTopologyProofV3) -> bool {
        Arc::ptr_eq(&self.session_key, topology.proof.session_identity())
            && Arc::ptr_eq(&self._topology_storage, &topology._storage)
    }

    #[must_use]
    pub fn matches_catalog_v3(&self, catalog: &AdmittedDocumentCatalogAuthorityV3) -> bool {
        Arc::ptr_eq(&self.session_key, &catalog.session_key)
    }
}

impl AdmittedDocumentViewTopologyProofV3 {
    pub fn dependency_first_order(&self) -> impl ExactSizeIterator<Item = &CatalogAssetIdV2> {
        self.proof.dependency_first_order()
    }

    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self._storage.validate_owner_v3(root, source_id, source)?;
        self.proof
            .session_identity()
            .validate_owner_v3(root, source_id, source)
    }

    #[must_use]
    pub fn retain_session_v3(&self) -> DocumentCatalogSessionWitnessV3 {
        DocumentCatalogSessionWitnessV3 {
            session_key: Arc::clone(self.proof.session_identity()),
            _topology_storage: Arc::clone(&self._storage),
        }
    }
}

impl std::fmt::Debug for AdmittedDocumentViewTopologyProofV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdmittedDocumentViewTopologyProofV3")
            .field("view_count", &self.dependency_first_order().len())
            .finish_non_exhaustive()
    }
}

pub struct AdmittedDocumentTopologyDiagnosticV3 {
    diagnostic: ModuleDiagnosticV2,
    _storage: FrontendRetainedStoragePermitV3,
}

impl AdmittedDocumentTopologyDiagnosticV3 {
    #[must_use]
    pub fn reason_code(&self) -> &'static str {
        self.diagnostic.code()
    }
}

impl std::fmt::Debug for AdmittedDocumentTopologyDiagnosticV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdmittedDocumentTopologyDiagnosticV3")
            .field("reason_code", &self.reason_code())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum DocumentTopologyRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    Phase,
    DemandOverflow,
    Domain(AdmittedDocumentTopologyDiagnosticV3),
}

/// Final catalog facts with source-root storage retained through the last
/// public reader. The cloneable legacy V2 authority never leaves this owner.
pub struct AdmittedDocumentCatalogAuthorityV3 {
    value: DocumentCatalogAuthorityV2,
    _storage: FrontendRetainedStoragePermitV3,
    session_key: Arc<DocumentSessionIdentityV2>,
}

impl AdmittedDocumentCatalogAuthorityV3 {
    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self._storage.validate_owner_v3(root, source_id, source)?;
        self.session_key.validate_owner_v3(root, source_id, source)
    }

    #[must_use]
    pub fn has_source_registry_v3(&self, expected: &SourceRegistryIdentityV2) -> bool {
        self.value.source_registry_identity() == expected
    }

    #[must_use]
    pub fn asset_count(&self) -> usize {
        self.value.assets().count()
    }

    #[must_use]
    pub fn dependency_count(&self) -> usize {
        self.value.dependencies().len()
    }
}

impl std::fmt::Debug for AdmittedDocumentCatalogAuthorityV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdmittedDocumentCatalogAuthorityV3")
            .field("asset_count", &self.asset_count())
            .field("dependency_count", &self.dependency_count())
            .finish_non_exhaustive()
    }
}

pub struct AdmittedDocumentCatalogDiagnosticV3 {
    diagnostic: ModuleDiagnosticV2,
    _storage: FrontendRetainedStoragePermitV3,
}

impl AdmittedDocumentCatalogDiagnosticV3 {
    #[must_use]
    pub fn reason_code(&self) -> &'static str {
        self.diagnostic.code()
    }
}

impl std::fmt::Debug for AdmittedDocumentCatalogDiagnosticV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AdmittedDocumentCatalogDiagnosticV3")
            .field("reason_code", &self.reason_code())
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub enum DocumentCatalogRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    ForeignTopology,
    Phase,
    DemandOverflow,
    FallibleAllocation,
    Domain(AdmittedDocumentCatalogDiagnosticV3),
}

/// Request-issued family target. The target proof cannot be supplied through
/// the cloneable V2 result shape at this boundary.
pub struct DocumentCatalogFamilyReferenceInputV3 {
    pub family: DocumentCatalogReferenceFamilyV2,
    pub source: PredeclaredDocumentAssetV2,
    pub target: AdmittedDocumentReferenceV3,
    pub location: CheckedLocatedSpanV2,
}

pub struct DocumentCatalogFinalInputV3 {
    pub profile_admissions: Vec<ProfileAdmissionInputV2>,
    pub activations: Vec<crate::PackActivationInputV2>,
    pub require_certified_directives: Vec<RequireCertifiedInputV2>,
    pub family_references: Vec<DocumentCatalogFamilyReferenceInputV3>,
}

impl DocumentCatalogValidatorV2<'_> {
    /// Admit the complete indexed dependency/cycle/order transaction before
    /// its first Vec or Arc birth. The same issued permit remains alive while
    /// either the validator or returned proof retains the topology state.
    pub fn validate_view_topology_with_request_owner_v3(
        &mut self,
        dependency_edges: Vec<crate::DocumentCatalogDependencyInputV2>,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        permit: &FrontendRetainedStoragePermitV3,
    ) -> Result<AdmittedDocumentViewTopologyProofV3, DocumentTopologyRequestErrorV3> {
        permit
            .validate_owner_v3(root, source_id, source)
            .map_err(DocumentTopologyRequestErrorV3::Storage)?;
        if self.phase != DocumentCatalogPhaseV2::Predeclared
            || self.request_stage != DocumentCatalogRequestStageV3::Predeclared
        {
            return Err(DocumentTopologyRequestErrorV3::Phase);
        }
        self.session_key
            .validate_owner_v3(root, source_id, source)
            .map_err(DocumentTopologyRequestErrorV3::Storage)?;
        let demand = self
            .predeclared
            .as_ref()
            .and_then(|assets| request_topology_bytes_v3(&dependency_edges, assets))
            .ok_or_else(|| {
                self.phase = DocumentCatalogPhaseV2::Failed;
                DocumentTopologyRequestErrorV3::DemandOverflow
            })?;
        let retained = root
            .reserve_v3(source_id, source, demand.retained_bytes)
            .map_err(|error| {
                self.phase = DocumentCatalogPhaseV2::Failed;
                DocumentTopologyRequestErrorV3::Storage(error)
            })?;
        let scratch = root
            .reserve_v3(source_id, source, demand.scratch_bytes)
            .map_err(|error| {
                self.phase = DocumentCatalogPhaseV2::Failed;
                DocumentTopologyRequestErrorV3::Storage(error)
            })?;
        match self.validate_view_topology(dependency_edges) {
            Ok(proof) => {
                drop(scratch);
                let storage = Arc::new(retained);
                self._topology_storage = Some(storage.clone());
                self.request_stage = DocumentCatalogRequestStageV3::TopologyValidated;
                Ok(AdmittedDocumentViewTopologyProofV3 {
                    proof,
                    _storage: storage,
                })
            }
            Err(diagnostic) => Err(DocumentTopologyRequestErrorV3::Domain(
                AdmittedDocumentTopologyDiagnosticV3 {
                    diagnostic,
                    _storage: scratch,
                },
            )),
        }
    }

    /// Seal final catalog facts under the original request root. The complete
    /// retained output and largest producer/diagnostic scratch are admitted
    /// before activation validation or dependency cloning starts.
    pub fn validate_predeclared_with_request_owner_v3(
        &mut self,
        topology: AdmittedDocumentViewTopologyProofV3,
        input: DocumentCatalogFinalInputV3,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        permit: &FrontendRetainedStoragePermitV3,
    ) -> Result<AdmittedDocumentCatalogAuthorityV3, DocumentCatalogRequestErrorV3> {
        permit
            .validate_owner_v3(root, source_id, source)
            .map_err(DocumentCatalogRequestErrorV3::Storage)?;
        topology
            ._storage
            .validate_owner_v3(root, source_id, source)
            .map_err(DocumentCatalogRequestErrorV3::Storage)?;
        if self.phase != DocumentCatalogPhaseV2::TopologyValidated
            || self.request_stage != DocumentCatalogRequestStageV3::TopologyValidated
        {
            return Err(DocumentCatalogRequestErrorV3::Phase);
        }
        self.session_key
            .validate_owner_v3(root, source_id, source)
            .map_err(DocumentCatalogRequestErrorV3::Storage)?;
        if !self
            ._topology_storage
            .as_ref()
            .is_some_and(|stored| Arc::ptr_eq(stored, &topology._storage))
        {
            return Err(DocumentCatalogRequestErrorV3::ForeignTopology);
        }
        let DocumentCatalogFinalInputV3 {
            profile_admissions,
            activations,
            require_certified_directives,
            family_references,
        } = input;
        let family_slots = family_references.len();
        for reference in &family_references {
            reference
                .target
                ._permit
                .validate_owner_v3(root, source_id, source)
                .map_err(DocumentCatalogRequestErrorV3::Storage)?;
            if !reference.source.belongs_to(&self.session_key)
                || !reference.target.value.belongs_to(&self.session_key)
            {
                return Err(DocumentCatalogRequestErrorV3::Phase);
            }
        }
        let translation_bytes = family_slots
            .checked_mul(std::mem::size_of::<DocumentCatalogFamilyReferenceInputV2>())
            .and_then(|bytes| {
                family_slots
                    .checked_mul(std::mem::size_of::<FrontendRetainedStoragePermitV3>())
                    .and_then(|permits| bytes.checked_add(permits))
            })
            .ok_or(DocumentCatalogRequestErrorV3::DemandOverflow)?;
        let translation = (translation_bytes > 0)
            .then(|| root.reserve_v3(source_id, source, translation_bytes))
            .transpose()
            .map_err(DocumentCatalogRequestErrorV3::Storage)?;
        let mut raw_references = Vec::new();
        raw_references
            .try_reserve_exact(family_slots)
            .map_err(|_allocation_error| DocumentCatalogRequestErrorV3::FallibleAllocation)?;
        let mut reference_storage = Vec::new();
        reference_storage
            .try_reserve_exact(family_slots)
            .map_err(|_allocation_error| DocumentCatalogRequestErrorV3::FallibleAllocation)?;
        if raw_references.capacity() > family_slots || reference_storage.capacity() > family_slots {
            return Err(DocumentCatalogRequestErrorV3::FallibleAllocation);
        }
        for reference in family_references {
            let AdmittedDocumentReferenceV3 { value, _permit } = reference.target;
            raw_references.push(DocumentCatalogFamilyReferenceInputV2 {
                family: reference.family,
                source: reference.source,
                target: value,
                location: reference.location,
            });
            reference_storage.push(_permit);
        }
        let input = DocumentCatalogFinalInputV2 {
            profile_admissions,
            activations,
            require_certified_directives,
            family_references: raw_references,
        };
        let demand =
            request_final_catalog_bytes_v3(&input, &topology.proof, &self.parts.external_catalog)
                .ok_or_else(|| {
                self.phase = DocumentCatalogPhaseV2::Failed;
                DocumentCatalogRequestErrorV3::DemandOverflow
            })?;
        let retained = root
            .reserve_v3(source_id, source, demand.retained_bytes)
            .map_err(|error| {
                self.phase = DocumentCatalogPhaseV2::Failed;
                DocumentCatalogRequestErrorV3::Storage(error)
            })?;
        let scratch = root
            .reserve_v3(source_id, source, demand.scratch_bytes)
            .map_err(|error| {
                self.phase = DocumentCatalogPhaseV2::Failed;
                DocumentCatalogRequestErrorV3::Storage(error)
            })?;
        match self.validate_predeclared(topology.proof, input) {
            Ok(value) => {
                drop(scratch);
                drop(reference_storage);
                drop(translation);
                self.request_stage = DocumentCatalogRequestStageV3::Finished;
                Ok(AdmittedDocumentCatalogAuthorityV3 {
                    value,
                    _storage: retained,
                    session_key: Arc::clone(&self.session_key),
                })
            }
            Err(diagnostic) => Err(DocumentCatalogRequestErrorV3::Domain(
                AdmittedDocumentCatalogDiagnosticV3 {
                    diagnostic,
                    _storage: scratch,
                },
            )),
        }
    }
}
