//! Opaque source-owned frontend stages; semantics remain with the semantic owner.

use super::{enforce_source_budget, project_parse_budget};
use crate::diagnostic::surface_transaction_error;
use quanta_gqlang_v2_sema::{
    begin_bind_document_with_request_owner_v3,
    begin_prepare_document_body_types_with_request_owner_v3,
    begin_resolving_document_session_with_request_owner_v3,
    declare_checked_document_with_request_owner_v3,
    predeclare_source_catalog_with_request_owner_v3 as predeclare_semantic_catalog_with_request_owner_v3,
    resolve_source_document_binder_with_request_owner_v3, AdmittedBinderDeclarationsV3,
    AdmittedBuiltinRegistryV3, AdmittedSourceBinderResolutionV3, BinderDeclarationRequestErrorV3,
    BuiltinRegistryRequestErrorV3, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    DeclaredDocumentV2, DeclaredRequestErrorV3, DocumentValidatorRequestErrorV3, FrontendBudgetV2,
    FrontendTransactionErrorV2, ResolvingDocumentSessionV3, SourceCatalogAssetsV3, SourceIdV2,
};
use quanta_gqlang_v2_surface::{
    FrontendRequestMaterializationRootV3, FrontendRequestOwnedValueV3, FrontendStorageRefusalV3,
};

#[path = "request_owner/scalar_compile.rs"]
mod scalar_compile;
mod source_body;
pub use scalar_compile::ScalarSourceCompileRequestErrorV3;
pub use scalar_compile::{
    compile_scalar_parsed_source_with_request_owner_v3, compile_scalar_source_with_request_owner_v3,
};
pub use source_body::{
    lower_source_body_with_request_owner_v3, materialize_source_body_with_request_owner_v3,
    plan_source_body_with_request_owner_v3, project_source_body_with_request_owner_v3,
    seal_scalar_execution_document_with_request_owner_v3,
    seal_source_catalog_with_request_owner_v3, topologize_source_body_with_request_owner_v3,
    type_source_body_with_request_owner_v3, AdmittedCanonicalFilesScalarReadV3,
    AdmittedScalarExecutionDocumentV3, CatalogSealedSourceBodyRequestSourceV3,
    LoweredSourceBodyRequestSourceV3, MaterializedSourceBodyRequestSourceV3,
    PlannedSourceBodyRequestSourceV3, ProjectedSourceBodyRequestSourceV3,
    ScalarExecutionRequestErrorV3, SharedAdmittedScalarExecutionDocumentV3,
    SourceBodyTopologyRequestErrorV3, SourceCatalogSealErrorV3,
    TopologizedSourceBodyRequestSourceV3, TypedSourceBodyRequestSourceV3,
};

/// Parser-issued surface and HIR storage retained on the host request root.
/// Diagnostics, semantic work, planning, and runtime execution require their
/// own producer admission before this can become a complete frontend owner.
pub struct ParsedRequestSourceV3 {
    owner: FrontendRequestOwnedValueV3<quanta_gqlang_v2_surface::HirDocumentV1>,
    budget: FrontendBudgetV2,
}

impl ParsedRequestSourceV3 {
    #[must_use]
    pub fn item_count_v3(&self) -> usize {
        self.owner.value_v3().items.len()
    }

    /// Inspect one parser-issued root while retaining the source allocation owner.
    pub fn root_family_v3(
        &self,
    ) -> Result<super::FrontendSourceRootFamilyV3, super::FrontendSourceRootClassificationErrorV3>
    {
        let items = self.owner.value_v3().items.as_slice();
        let [item] = items else {
            return Err(if items.is_empty() {
                super::FrontendSourceRootClassificationErrorV3::NoItem
            } else {
                super::FrontendSourceRootClassificationErrorV3::MultipleItems {
                    observed: items.len(),
                }
            });
        };
        Ok(super::classify_parsed_root_v3(item))
    }
}

/// HIR checked by the pipeline from its exact parser-issued owner. Private
/// construction prevents a public generic map from installing foreign HIR as
/// the pipeline's checked stage.
pub struct CheckedRequestSourceV3 {
    owner: FrontendRequestOwnedValueV3<CheckedSyntaxDocumentV2>,
}

impl CheckedRequestSourceV3 {
    #[must_use]
    pub fn node_count_v3(&self) -> usize {
        self.owner.value_v3().node_count()
    }
}

/// Declared syntax retained with the same parser-issued request permit.
pub struct DeclaredRequestSourceV3<'authority> {
    authority: &'authority ClosedFrontendAuthorityV2,
    owner: FrontendRequestOwnedValueV3<(
        DeclaredDocumentV2,
        quanta_gqlang_v2_sema::ExtendedSourceRegistryViewV2,
    )>,
}

/// The Modules resolution session is bound to the exact source and request
/// root. Later producers retain their own permits under that root.
pub struct ResolvingRequestSourceV3<'authority> {
    authority: &'authority ClosedFrontendAuthorityV2,
    owner: FrontendRequestOwnedValueV3<(
        DeclaredDocumentV2,
        quanta_gqlang_v2_sema::ExtendedSourceRegistryViewV2,
        ResolvingDocumentSessionV3<'authority>,
    )>,
}

/// Exact local catalog assets issued by Modules before body resolution.
pub struct PredeclaredRequestSourceV3<'authority> {
    _authority: &'authority ClosedFrontendAuthorityV2,
    owner: FrontendRequestOwnedValueV3<(
        DeclaredDocumentV2,
        quanta_gqlang_v2_sema::ExtendedSourceRegistryViewV2,
        ResolvingDocumentSessionV3<'authority>,
        SourceCatalogAssetsV3,
    )>,
}

impl PredeclaredRequestSourceV3<'_> {
    #[must_use]
    pub fn local_asset_count_v3(&self) -> usize {
        self.owner.value_v3().3.len()
    }
}

/// Builtin type inventory retained under the declared source's request root.
pub struct PreparingBodyRequestSourceV3<'authority> {
    _authority: &'authority ClosedFrontendAuthorityV2,
    owner: FrontendRequestOwnedValueV3<(
        DeclaredDocumentV2,
        quanta_gqlang_v2_sema::ExtendedSourceRegistryViewV2,
        ResolvingDocumentSessionV3<'authority>,
        SourceCatalogAssetsV3,
        AdmittedBuiltinRegistryV3,
    )>,
}

impl PreparingBodyRequestSourceV3<'_> {
    #[must_use]
    pub fn local_asset_count_v3(&self) -> usize {
        self.owner.value_v3().3.len()
    }

    #[must_use]
    pub fn builtin_family_v3(
        &self,
        name: &str,
    ) -> Option<quanta_gqlang_v2_sema::BuiltinFamilyIdV2> {
        self.owner.value_v3().4.family_id(name)
    }
}

/// Binder declaration table retained with the catalog and builtin stages.
pub struct DeclaringBinderRequestSourceV3<'authority> {
    _authority: &'authority ClosedFrontendAuthorityV2,
    owner: FrontendRequestOwnedValueV3<(
        DeclaredDocumentV2,
        quanta_gqlang_v2_sema::ExtendedSourceRegistryViewV2,
        ResolvingDocumentSessionV3<'authority>,
        SourceCatalogAssetsV3,
        AdmittedBuiltinRegistryV3,
        AdmittedBinderDeclarationsV3<'authority>,
    )>,
}

impl DeclaringBinderRequestSourceV3<'_> {
    #[must_use]
    pub fn declared_name_count_v3(&self) -> usize {
        self.owner.value_v3().5.declared_name_count_v3()
    }
}

/// Source bindings retained with the exact source, catalog, and builtin
/// stages. Type checking and planning still require their own admission.
pub struct ResolvedSourceBinderRequestSourceV3<'authority> {
    _authority: &'authority ClosedFrontendAuthorityV2,
    owner: FrontendRequestOwnedValueV3<(
        DeclaredDocumentV2,
        quanta_gqlang_v2_sema::ExtendedSourceRegistryViewV2,
        ResolvingDocumentSessionV3<'authority>,
        SourceCatalogAssetsV3,
        AdmittedBuiltinRegistryV3,
        AdmittedSourceBinderResolutionV3<'authority>,
    )>,
}

impl ResolvedSourceBinderRequestSourceV3<'_> {
    #[must_use]
    pub fn resolved_reference_count_v3(&self) -> usize {
        self.owner.value_v3().5.resolved_reference_count_v3()
    }

    #[must_use]
    pub fn relation_output_count_v3(&self) -> usize {
        self.owner.value_v3().5.relation_output_count_v3()
    }
}

/// Typed refusal from semantic-owned catalog predeclaration.
pub use quanta_gqlang_v2_sema::SemanticRequestPredeclareErrorV3 as FrontendRequestPredeclareErrorV3;

impl ResolvingRequestSourceV3<'_> {
    #[must_use]
    pub fn item_count_v3(&self) -> usize {
        self.owner.value_v3().0.order().len()
    }
}

impl DeclaredRequestSourceV3<'_> {
    #[must_use]
    pub fn item_count_v3(&self) -> usize {
        self.owner.value_v3().0.order().len()
    }
}

pub fn parse_source_document_with_request_owner_v3(
    source_id: SourceIdV2,
    source: &str,
    budget: &FrontendBudgetV2,
    root: &quanta_gqlang_v2_surface::FrontendRequestMaterializationRootV3<'_>,
) -> Result<ParsedRequestSourceV3, FrontendTransactionErrorV2> {
    enforce_source_budget(source_id, source, budget).map_err(FrontendTransactionErrorV2::from)?;
    let projected = project_parse_budget(source_id, source, budget)
        .map_err(FrontendTransactionErrorV2::from)?;
    let owner = quanta_gqlang_v2_surface::parse_source_document_with_request_owner_v3(
        source_id, source, &projected, root,
    )
    .map_err(|error| surface_transaction_error(&error))?;
    Ok(ParsedRequestSourceV3 {
        owner,
        budget: budget.clone(),
    })
}

/// Classify one parser-issued source root, then release its request storage.
pub fn classify_single_source_root_with_request_owner_v3(
    source_id: SourceIdV2,
    source: &str,
    budget: &FrontendBudgetV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
) -> Result<super::FrontendSourceRootFamilyV3, super::FrontendSourceRootClassificationErrorV3> {
    let parsed = parse_source_document_with_request_owner_v3(source_id, source, budget, root)
        .map_err(super::FrontendSourceRootClassificationErrorV3::Frontend)?;
    parsed.root_family_v3()
}

/// Allocation-free refusal from the request-owned HIR integrity stage.
#[derive(Debug)]
pub enum FrontendRequestCheckErrorV3 {
    Root(FrontendStorageRefusalV3),
    DemandOverflow,
    SourceMismatch,
    Integrity(quanta_gqlang_v2_sema::HirRequestIntegrityErrorV3),
}

/// Check parser-issued HIR identity and depth under the same request root.
/// This is a staging boundary; semantic and plan producers are not yet
/// admitted by this method.
pub fn check_source_document_with_request_owner_v3(
    source_id: SourceIdV2,
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    parsed: ParsedRequestSourceV3,
) -> Result<CheckedRequestSourceV3, FrontendRequestCheckErrorV3> {
    let slots = parsed.owner.source_issued_node_slots_v3();
    let bytes = CheckedSyntaxDocumentV2::request_integrity_scratch_bytes_v3(slots)
        .ok_or(FrontendRequestCheckErrorV3::DemandOverflow)?;
    let owner = parsed.owner.try_map_v3(|hir, permit| {
        permit
            .validate_owner_v3(root, source_id, source)
            .map_err(FrontendRequestCheckErrorV3::Root)?;
        if hir.source_id != source_id || hir.source_len != source.len() {
            return Err(FrontendRequestCheckErrorV3::SourceMismatch);
        }
        let scratch = if bytes == 0 {
            None
        } else {
            Some(
                root.reserve_v3(source_id, source, bytes)
                    .map_err(FrontendRequestCheckErrorV3::Root)?,
            )
        };
        let checked = CheckedSyntaxDocumentV2::try_from_hir_with_source_slots_v3(hir, slots)
            .map_err(FrontendRequestCheckErrorV3::Integrity);
        drop(scratch);
        checked
    })?;
    Ok(CheckedRequestSourceV3 { owner })
}

/// Predeclare one checked source under its retained parser/HIR permit. This
/// stage returns its registry extension with the declared document so later
/// semantic producers can use the same source identity.
pub fn declare_source_document_with_request_owner_v3<'authority>(
    source: &str,
    authority: &'authority ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
    checked: CheckedRequestSourceV3,
) -> Result<DeclaredRequestSourceV3<'authority>, DeclaredRequestErrorV3> {
    let slots = checked.owner.source_issued_node_slots_v3();
    let owner = checked.owner.try_map_v3(|value, permit| {
        declare_checked_document_with_request_owner_v3(
            value, authority, root, source, permit, slots,
        )
    })?;
    Ok(DeclaredRequestSourceV3 { authority, owner })
}

/// Begin the Modules-owned resolution transaction after source declaration.
/// The validator's session-key allocation is admitted before construction.
pub fn begin_resolve_source_document_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    declared: DeclaredRequestSourceV3<'authority>,
) -> Result<ResolvingRequestSourceV3<'authority>, DocumentValidatorRequestErrorV3> {
    let authority = declared.authority;
    let owner = declared.owner.try_map_v3(|(declared, extended), permit| {
        let session = begin_resolving_document_session_with_request_owner_v3(
            &declared, &extended, authority, root, source, permit,
        )?;
        Ok((declared, extended, session))
    })?;
    Ok(ResolvingRequestSourceV3 { authority, owner })
}

/// Issue local catalog IDs using borrowed checked HIR and request-root
/// custody. The returned IDs retain the Modules session provenance.
pub fn predeclare_source_catalog_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    resolving: ResolvingRequestSourceV3<'authority>,
) -> Result<PredeclaredRequestSourceV3<'authority>, FrontendRequestPredeclareErrorV3> {
    let authority = resolving.authority;
    let owner = resolving
        .owner
        .try_map_v3(|(declared, extended, mut session), permit| {
            let assets = predeclare_semantic_catalog_with_request_owner_v3(
                &declared,
                &extended,
                &mut session,
                root,
                source,
                permit,
            )?;
            Ok((declared, extended, session, assets))
        })?;
    Ok(PredeclaredRequestSourceV3 {
        _authority: authority,
        owner,
    })
}

/// Admit the finite builtin inventory before its retained allocation.
pub fn begin_prepare_source_body_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    predeclared: PredeclaredRequestSourceV3<'authority>,
) -> Result<PreparingBodyRequestSourceV3<'authority>, BuiltinRegistryRequestErrorV3> {
    let authority = predeclared._authority;
    let owner = predeclared
        .owner
        .try_map_v3(|(declared, extended, session, assets), permit| {
            let builtins = begin_prepare_document_body_types_with_request_owner_v3(
                &declared, root, source, permit,
            )?;
            Ok((declared, extended, session, assets, builtins))
        })?;
    Ok(PreparingBodyRequestSourceV3 {
        _authority: authority,
        owner,
    })
}

/// Admit Binder declarations from the retained checked source.
pub fn begin_bind_source_document_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    preparing: PreparingBodyRequestSourceV3<'authority>,
) -> Result<DeclaringBinderRequestSourceV3<'authority>, BinderDeclarationRequestErrorV3> {
    let authority = preparing._authority;
    let owner =
        preparing
            .owner
            .try_map_v3(|(declared, extended, session, assets, builtins), permit| {
                let binder = begin_bind_document_with_request_owner_v3(
                    &declared, authority, root, source, permit,
                )?;
                Ok((declared, extended, session, assets, builtins, binder))
            })?;
    Ok(DeclaringBinderRequestSourceV3 {
        _authority: authority,
        owner,
    })
}

/// Resolve the supported source bindings through their original request.
/// Unsupported calls and richer families refuse before new Binder storage.
pub fn resolve_source_bindings_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    declaring: DeclaringBinderRequestSourceV3<'authority>,
) -> Result<ResolvedSourceBinderRequestSourceV3<'authority>, BinderDeclarationRequestErrorV3> {
    let authority = declaring._authority;
    let owner = declaring.owner.try_map_v3(
        |(declared, extended, session, assets, builtins, binder), permit| {
            let bound = resolve_source_document_binder_with_request_owner_v3(
                &declared, binder, root, source, permit,
            )?;
            Ok((declared, extended, session, assets, builtins, bound))
        },
    )?;
    Ok(ResolvedSourceBinderRequestSourceV3 {
        _authority: authority,
        owner,
    })
}
