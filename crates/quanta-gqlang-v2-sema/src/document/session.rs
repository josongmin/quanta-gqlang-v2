//! Request-owned semantic resolution staging.

use super::{
    attach_source_catalog_assets_with_request_owner_v3,
    collect_catalog_asset_inputs_with_request_owner_v3, CatalogAssetInputsRequestErrorV3,
    ClosedFrontendAuthorityV2, DeclaredDocumentV2, ExtendedSourceRegistryViewV2,
    FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3, SourceCatalogAssetsV3,
    SourceCatalogAttachErrorV3,
};
use quanta_gqlang_v2_modules::{
    DocumentCatalogValidatorV2, DocumentPredeclareRequestErrorV3, DocumentValidatorRequestErrorV3,
};

/// A Modules validator retained by the semantic owner for this request.
/// The pipeline can carry this session but cannot construct or inspect its
/// validator, and the issued storage permit remains with the pipeline owner.
pub struct ResolvingDocumentSessionV3<'authority> {
    pub(super) validator: DocumentCatalogValidatorV2<'authority>,
    pub(super) authority: &'authority ClosedFrontendAuthorityV2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticRequestPredeclareErrorV3 {
    Inputs(CatalogAssetInputsRequestErrorV3),
    Modules(DocumentPredeclareRequestErrorV3),
    Attach(SourceCatalogAttachErrorV3),
}

/// Begin semantic resolution with the exact declared source and request root.
pub fn begin_resolving_document_session_with_request_owner_v3<'authority>(
    declared: &DeclaredDocumentV2,
    extended: &ExtendedSourceRegistryViewV2,
    authority: &'authority ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    permit: &mut FrontendRetainedStoragePermitV3,
) -> Result<ResolvingDocumentSessionV3<'authority>, DocumentValidatorRequestErrorV3> {
    let validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
        extended.clone(),
        authority.modules(),
        root,
        declared.checked().source_id(),
        source,
        permit,
    )?;
    Ok(ResolvingDocumentSessionV3 {
        validator,
        authority,
    })
}

/// Issue catalog proofs under the same Modules validator and request permit.
pub fn predeclare_source_catalog_with_request_owner_v3(
    declared: &DeclaredDocumentV2,
    extended: &ExtendedSourceRegistryViewV2,
    session: &mut ResolvingDocumentSessionV3<'_>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    permit: &mut FrontendRetainedStoragePermitV3,
) -> Result<SourceCatalogAssetsV3, SemanticRequestPredeclareErrorV3> {
    let inputs = collect_catalog_asset_inputs_with_request_owner_v3(
        declared,
        extended,
        session.authority,
        root,
        source,
        permit,
    )
    .map_err(SemanticRequestPredeclareErrorV3::Inputs)?;
    let assets = session
        .validator
        .predeclare_assets_with_request_owner_v3(
            inputs,
            root,
            declared.checked().source_id(),
            source,
            permit,
        )
        .map_err(SemanticRequestPredeclareErrorV3::Modules)?;
    attach_source_catalog_assets_with_request_owner_v3(declared, assets, root, source, permit)
        .map_err(SemanticRequestPredeclareErrorV3::Attach)
}
