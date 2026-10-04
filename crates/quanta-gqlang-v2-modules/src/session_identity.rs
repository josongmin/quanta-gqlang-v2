//! Document-session identity and storage custody shared by public proofs.

use quanta_gqlang_v2_hir::{
    FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3,
    FrontendStorageRefusalV3, SourceIdV2,
};

/// Pointer identity, not value equality, establishes document provenance.
pub(crate) struct DocumentSessionIdentityV2 {
    _storage: Option<FrontendRetainedStoragePermitV3>,
}

impl DocumentSessionIdentityV2 {
    pub(crate) const fn unowned() -> Self {
        Self { _storage: None }
    }

    pub(crate) const fn owned(storage: FrontendRetainedStoragePermitV3) -> Self {
        Self {
            _storage: Some(storage),
        }
    }

    pub(crate) fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self._storage
            .as_ref()
            .ok_or(FrontendStorageRefusalV3::ForeignRequestRoot)?
            .validate_owner_v3(root, source_id, source)
    }
}

impl std::fmt::Debug for DocumentSessionIdentityV2 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("DocumentSessionIdentityV2")
    }
}

// Arc<()> used to compare equal by value. Preserve that semantic equality;
// provenance checks use Arc::ptr_eq and do not use this implementation.
impl PartialEq for DocumentSessionIdentityV2 {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl Eq for DocumentSessionIdentityV2 {}

pub(crate) struct PredeclaredRequestCustodyV3 {
    _storage: FrontendRetainedStoragePermitV3,
}

impl PredeclaredRequestCustodyV3 {
    pub(crate) const fn new(storage: FrontendRetainedStoragePermitV3) -> Self {
        Self { _storage: storage }
    }
}

impl std::fmt::Debug for PredeclaredRequestCustodyV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("PredeclaredRequestCustodyV3")
    }
}

pub(crate) fn arc_layout_bytes_v3<T>() -> Option<usize> {
    std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
        .extend(std::alloc::Layout::new::<T>())
        .ok()
        .map(|(layout, _)| layout.pad_to_align().size())
}
