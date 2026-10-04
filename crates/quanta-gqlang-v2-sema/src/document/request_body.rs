//! Request-owned builtin preparation and Binder declaration boundaries.

use super::{
    Binder, BinderEvidenceReaderV3, ClosedFrontendAuthorityV2, DeclaredDocumentV2,
    FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3,
    FrontendStorageRefusalV3, ModuleImportAdapterV2, SourceIdV2,
};
use quanta_gqlang_v2_binder::BinderDeclarationRequestErrorV3;
use quanta_gqlang_v2_types::{
    AdmittedBuiltinRegistryV3, BuiltinRegistryRequestErrorV3, BuiltinRegistryV2,
};

/// Prepare the finite builtin inventory under the declared source's request.
pub fn begin_prepare_document_body_types_with_request_owner_v3(
    declared: &DeclaredDocumentV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    permit: &FrontendRetainedStoragePermitV3,
) -> Result<AdmittedBuiltinRegistryV3, BuiltinRegistryRequestErrorV3> {
    let source_id = declared.checked().source_id();
    if declared.checked().source_len() != source.len() {
        return Err(BuiltinRegistryRequestErrorV3::SourceMismatch);
    }
    BuiltinRegistryV2::try_closed_with_request_owner_v3(root, source_id, source, permit)
}

/// Binder declarations with their independently retained request storage.
pub struct AdmittedBinderDeclarationsV3<'authority> {
    binder:
        quanta_gqlang_v2_binder::AdmittedBinderDeclarationsV3<ModuleImportAdapterV2<'authority>>,
    authority: &'authority ClosedFrontendAuthorityV2,
}

/// Source-issued Binder result for the supported checked source subset.
/// The cloneable raw Binder result remains inside its admitted owner.
pub struct AdmittedSourceBinderResolutionV3<'authority> {
    bound: quanta_gqlang_v2_binder::AdmittedBinderResolutionV3,
    pub(super) authority: &'authority ClosedFrontendAuthorityV2,
}

impl AdmittedSourceBinderResolutionV3<'_> {
    pub(super) fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.bound.validate_owner_v3(root, source_id, source)
    }
    #[must_use]
    pub fn resolved_reference_count_v3(&self) -> usize {
        self.bound.resolved_reference_count_v3()
    }

    #[must_use]
    pub fn relation_output_count_v3(&self) -> usize {
        self.bound.relation_output_count_v3()
    }

    pub(super) fn evidence_v3(&self) -> &dyn BinderEvidenceReaderV3 {
        &self.bound
    }
}

impl AdmittedBinderDeclarationsV3<'_> {
    #[must_use]
    pub fn declared_name_count_v3(&self) -> usize {
        self.binder.declared_name_count_v3()
    }
}

/// Start Binder from the exact checked HIR held by the request stage.
pub fn begin_bind_document_with_request_owner_v3<'authority>(
    declared: &DeclaredDocumentV2,
    authority: &'authority ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    parent: &FrontendRetainedStoragePermitV3,
) -> Result<AdmittedBinderDeclarationsV3<'authority>, BinderDeclarationRequestErrorV3> {
    let source_id = declared.checked().source_id();
    if declared.checked().source_len() != source.len() {
        return Err(BinderDeclarationRequestErrorV3::SourceMismatch);
    }
    let graph = ModuleImportAdapterV2(authority.modules().resolved_graph());
    let binder = Binder::new(graph).declare_checked_program_with_request_owner_v3(
        declared.checked().checked(),
        root,
        source_id,
        source,
        parent,
    )?;
    Ok(AdmittedBinderDeclarationsV3 { binder, authority })
}

/// Continue Binder over the original checked source and request root. Unsupported
/// calls and richer relation or rule forms retain Binder's typed refusal.
pub fn resolve_source_document_binder_with_request_owner_v3<'authority>(
    declared: &DeclaredDocumentV2,
    binder: AdmittedBinderDeclarationsV3<'authority>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    parent: &FrontendRetainedStoragePermitV3,
) -> Result<AdmittedSourceBinderResolutionV3<'authority>, BinderDeclarationRequestErrorV3> {
    let source_id = declared.checked().source_id();
    if declared.checked().source_len() != source.len() {
        return Err(BinderDeclarationRequestErrorV3::SourceMismatch);
    }
    let AdmittedBinderDeclarationsV3 { binder, authority } = binder;
    let bound = binder.resolve_checked_source_bindings_with_request_owner_v3(
        declared.checked().checked(),
        root,
        source_id,
        source,
        parent,
    )?;
    Ok(AdmittedSourceBinderResolutionV3 { bound, authority })
}
