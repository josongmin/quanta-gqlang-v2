//! Request-owned scalar Query and View body stages delegated to semantic producers.

mod sealed;
pub use sealed::{
    lower_source_body_with_request_owner_v3, plan_source_body_with_request_owner_v3,
    project_source_body_with_request_owner_v3,
    seal_scalar_execution_document_with_request_owner_v3,
    seal_source_catalog_with_request_owner_v3, AdmittedCanonicalFilesScalarReadV3,
    AdmittedScalarExecutionDocumentV3, CatalogSealedSourceBodyRequestSourceV3,
    LoweredSourceBodyRequestSourceV3, PlannedSourceBodyRequestSourceV3,
    ProjectedSourceBodyRequestSourceV3, ScalarExecutionRequestErrorV3,
    SharedAdmittedScalarExecutionDocumentV3, SourceCatalogSealErrorV3,
};

use super::ResolvedSourceBinderRequestSourceV3;
use quanta_gqlang_v2_sema::{
    materialize_source_inline_view_with_request_owner_v3,
    topologize_source_inline_view_with_request_owner_v3,
    type_one_let_composition_body_with_request_owner_v3,
    type_source_inline_view_with_request_owner_v3, AdmittedSourceBodyTypesV3,
    AdmittedSourceInlineViewBodyV3, AdmittedSourceViewTopologyV3, ClosedFrontendAuthorityV2,
    ExtendedSourceRegistryViewV2, ResolvingDocumentSessionV3,
    SourceBodyMaterializationRequestErrorV3, SourceBodyTypesRequestErrorV3, SourceCatalogAssetsV3,
};
use quanta_gqlang_v2_surface::{FrontendRequestMaterializationRootV3, FrontendRequestOwnedValueV3};

pub use quanta_gqlang_v2_sema::DocumentTopologyRequestErrorV3 as SourceBodyTopologyRequestErrorV3;

/// Scalar projection Types facts with their original source and producer permits.
/// This carrier is a partial frontend stage, not a compiled execution document.
pub struct TypedSourceBodyRequestSourceV3<'authority> {
    authority: &'authority ClosedFrontendAuthorityV2,
    owner: FrontendRequestOwnedValueV3<(
        ExtendedSourceRegistryViewV2,
        ResolvingDocumentSessionV3<'authority>,
        SourceCatalogAssetsV3,
        AdmittedSourceBodyTypesV3<'authority>,
    )>,
}

impl TypedSourceBodyRequestSourceV3<'_> {
    #[must_use]
    pub fn typed_fact_count_v3(&self) -> usize {
        self.owner.value_v3().3.typed_fact_count_v3()
    }

    #[must_use]
    pub fn relation_row_count_v3(&self) -> usize {
        self.owner.value_v3().3.relation_row_count_v3()
    }
}

/// Semantic-owned topology issued under the same request as the typed View.
pub struct TopologizedSourceBodyRequestSourceV3<'authority> {
    authority: &'authority ClosedFrontendAuthorityV2,
    owner: FrontendRequestOwnedValueV3<(
        ExtendedSourceRegistryViewV2,
        ResolvingDocumentSessionV3<'authority>,
        SourceCatalogAssetsV3,
        AdmittedSourceBodyTypesV3<'authority>,
        AdmittedSourceViewTopologyV3,
    )>,
}

impl TopologizedSourceBodyRequestSourceV3<'_> {
    #[must_use]
    pub fn dependency_first_view_count_v3(&self) -> usize {
        self.owner.value_v3().4.dependency_first_view_count_v3()
    }
}

/// Materialized scalar body retained with every producing request stage.
/// The finite catalog/algebra/planning continuation consumes this carrier;
/// general diagnostics and serving execution still require their own admission.
pub struct MaterializedSourceBodyRequestSourceV3<'authority> {
    _authority: &'authority ClosedFrontendAuthorityV2,
    owner: FrontendRequestOwnedValueV3<(
        ExtendedSourceRegistryViewV2,
        ResolvingDocumentSessionV3<'authority>,
        SourceCatalogAssetsV3,
        AdmittedSourceViewTopologyV3,
        AdmittedSourceInlineViewBodyV3<'authority>,
    )>,
}

impl MaterializedSourceBodyRequestSourceV3<'_> {
    #[must_use]
    pub fn output_field_count_v3(&self) -> usize {
        self.owner.value_v3().4.output_field_count_v3()
    }

    #[must_use]
    pub fn expression_count_v3(&self) -> usize {
        self.owner.value_v3().4.expression_count_v3()
    }

    pub fn output_field_names_v3(&self) -> impl ExactSizeIterator<Item = &str> {
        self.owner.value_v3().4.output_field_names_v3()
    }

    #[must_use]
    pub fn output_row_v3(&self) -> &quanta_gqlang_v2_sema::RecordTypeV1 {
        self.owner.value_v3().4.output_row_v3()
    }

    #[must_use]
    pub fn output_columns_v3(&self) -> &[quanta_gqlang_v2_sema::OutputColumnContractV2] {
        self.owner.value_v3().4.output_columns_v3()
    }

    pub fn expression_keys_v3(
        &self,
    ) -> impl ExactSizeIterator<Item = quanta_gqlang_v2_sema::SourceQualifiedNodeIdV2> + '_ {
        self.owner.value_v3().4.expression_keys_v3()
    }
}

/// Produce Types facts from the source-issued Binder result. Sema owns the
/// finite supported subset and refuses other families before Types storage.
pub fn type_source_body_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    bound: ResolvedSourceBinderRequestSourceV3<'authority>,
) -> Result<TypedSourceBodyRequestSourceV3<'authority>, SourceBodyTypesRequestErrorV3> {
    let authority = bound._authority;
    let owner = bound.owner.try_map_v3(
        |(declared, extended, session, assets, builtins, bindings), permit| {
            let is_one_let = declared.is_one_let_composition_v3();
            let typed = if is_one_let {
                type_one_let_composition_body_with_request_owner_v3(
                    declared, &extended, &session, builtins, bindings, root, source, permit,
                )?
            } else {
                type_source_inline_view_with_request_owner_v3(
                    declared, &extended, &session, builtins, bindings, root, source, permit,
                )?
            };
            Ok((extended, session, assets, typed))
        },
    )?;
    Ok(TypedSourceBodyRequestSourceV3 { authority, owner })
}

/// Seal the exact supported View topology through the semantic session.
pub fn topologize_source_body_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    typed: TypedSourceBodyRequestSourceV3<'authority>,
) -> Result<TopologizedSourceBodyRequestSourceV3<'authority>, SourceBodyTopologyRequestErrorV3> {
    let authority = typed.authority;
    let owner = typed
        .owner
        .try_map_v3(|(extended, mut session, assets, facts), permit| {
            let topology = topologize_source_inline_view_with_request_owner_v3(
                &mut session,
                &facts,
                root,
                source,
                permit,
            )?;
            Ok((extended, session, assets, facts, topology))
        })?;
    Ok(TopologizedSourceBodyRequestSourceV3 { authority, owner })
}

/// Materialize a typed View only after the same request has sealed topology.
pub fn materialize_source_body_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    topologized: TopologizedSourceBodyRequestSourceV3<'authority>,
) -> Result<
    MaterializedSourceBodyRequestSourceV3<'authority>,
    SourceBodyMaterializationRequestErrorV3,
> {
    let authority = topologized.authority;
    let owner =
        topologized
            .owner
            .try_map_v3(|(extended, session, assets, facts, topology), permit| {
                let body = materialize_source_inline_view_with_request_owner_v3(
                    facts, &assets, &topology, root, source, permit,
                )?;
                Ok((extended, session, assets, topology, body))
            })?;
    Ok(MaterializedSourceBodyRequestSourceV3 {
        _authority: authority,
        owner,
    })
}

#[cfg(test)]
#[path = "source_body/session_tests.rs"]
mod session_tests;
