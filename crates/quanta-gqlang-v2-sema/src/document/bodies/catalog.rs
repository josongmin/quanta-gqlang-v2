//! Catalog payload resolution in source order.

use super::super::{
    document_failure, resolve_catalog_family_reference, BTreeMap, CatalogAssetRefInputV2,
    CheckedSyntaxDocumentV2, DocumentCatalogReferenceFamilyV2, DocumentCatalogValidatorV2,
    FrontendTransactionErrorV2, HirNodeIdV1, InlineViewPlanV2, NamespaceKindV2,
    PredeclaredDocumentAssetV2, ProfileAdmissionInputV2, ProfileAdmissionV2, ResolvedCatalogItemV2,
    ResolvedCatalogPayloadV2, SemanticItemIdV2, SemanticStepCounterV2, TypedRelationV2,
};
use quanta_gqlang_v2_modules::{CatalogAssetIdV2, DocumentCatalogFamilyReferenceInputV2};

pub(super) fn resolve_catalog_declaration(
    decl: &quanta_gqlang_v2_hir::CatalogDeclHirV1,
    checked: &CheckedSyntaxDocumentV2,
    predeclared: &BTreeMap<HirNodeIdV1, PredeclaredDocumentAssetV2>,
    inline_views: &InlineViewPlanV2,
    materialized_inline_views: &BTreeMap<CatalogAssetIdV2, TypedRelationV2>,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
    family_references: &mut Vec<DocumentCatalogFamilyReferenceInputV2>,
    profile_admissions: &mut Vec<ProfileAdmissionInputV2>,
) -> Result<ResolvedCatalogItemV2, FrontendTransactionErrorV2> {
    let Some(asset) = predeclared.get(&decl.id) else {
        return Err(document_failure(
            "sema.document_validation_failed",
            checked.source_id(),
            decl.span,
            "catalog declaration lacks modules-owned predeclaration",
        )
        .into());
    };
    let mut resolve = |reference, family| {
        resolve_catalog_family_reference(
            reference,
            family,
            asset,
            checked.source_id(),
            extended,
            validator,
            steps,
            family_references,
        )
    };
    let (payload, provider_metadata) = match &decl.payload {
        quanta_gqlang_v2_hir::CatalogPayloadV1::View(
            quanta_gqlang_v2_hir::ViewBodyV1::QueryRef(reference),
        ) => (
            ResolvedCatalogPayloadV2::View {
                query_ref: Some(resolve(
                    reference,
                    DocumentCatalogReferenceFamilyV2::ViewQuery,
                )?),
            },
            None,
        ),
        quanta_gqlang_v2_hir::CatalogPayloadV1::View(quanta_gqlang_v2_hir::ViewBodyV1::Inline(
            _,
        )) => (ResolvedCatalogPayloadV2::View { query_ref: None }, None),
        quanta_gqlang_v2_hir::CatalogPayloadV1::Model { provider, source } => (
            ResolvedCatalogPayloadV2::Model {
                source: resolve(source, DocumentCatalogReferenceFamilyV2::ModelSource)?,
            },
            Some(provider.clone()),
        ),
        quanta_gqlang_v2_hir::CatalogPayloadV1::Pack { source, activation } => (
            ResolvedCatalogPayloadV2::Pack {
                source: resolve(source, DocumentCatalogReferenceFamilyV2::PackSource)?,
                activation: *activation,
            },
            None,
        ),
        quanta_gqlang_v2_hir::CatalogPayloadV1::Profile {
            capabilities,
            admission,
        } => {
            let capabilities = capabilities
                .iter()
                .map(|reference| {
                    resolve(
                        reference,
                        DocumentCatalogReferenceFamilyV2::ProfileCapability,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let admission_proof = match admission {
                quanta_gqlang_v2_hir::AdmissionV1::Allowlist => ProfileAdmissionV2::Allowlist,
                quanta_gqlang_v2_hir::AdmissionV1::ReviewRequired => {
                    ProfileAdmissionV2::ReviewRequired
                }
                quanta_gqlang_v2_hir::AdmissionV1::CertifiedOnly => {
                    ProfileAdmissionV2::CertifiedOnly
                }
            };
            profile_admissions.push(ProfileAdmissionInputV2 {
                profile: CatalogAssetRefInputV2 {
                    module_id: asset.id().module_id().clone(),
                    namespace_kind: NamespaceKindV2::Profile,
                    local_name: decl.name.clone(),
                    location: asset.location().clone(),
                },
                admission: admission_proof,
                location: asset.location().clone(),
            });
            (
                ResolvedCatalogPayloadV2::Profile {
                    capabilities,
                    admission: *admission,
                },
                None,
            )
        }
        quanta_gqlang_v2_hir::CatalogPayloadV1::Gate { checks, binding } => {
            let checks = checks
                .iter()
                .map(|reference| resolve(reference, DocumentCatalogReferenceFamilyV2::GateCheck))
                .collect::<Result<Vec<_>, _>>()?;
            (
                ResolvedCatalogPayloadV2::Gate {
                    checks,
                    binding: *binding,
                },
                None,
            )
        }
        quanta_gqlang_v2_hir::CatalogPayloadV1::RuleBinding { check_ref } => (
            ResolvedCatalogPayloadV2::RuleBinding {
                check_ref: resolve(
                    check_ref,
                    DocumentCatalogReferenceFamilyV2::RuleBindingCheck,
                )?,
            },
            None,
        ),
    };
    let (expected_inline_view_id, inline_view) = match &decl.payload {
        quanta_gqlang_v2_hir::CatalogPayloadV1::View(quanta_gqlang_v2_hir::ViewBodyV1::Inline(
            query,
        )) => {
            let view = materialized_inline_views
                .get(asset.id())
                .cloned()
                .ok_or_else(|| {
                    document_failure(
                        "sema.view_materialization_order",
                        checked.source_id(),
                        decl.span,
                        "inline View was absent from dependency-first materialization cache",
                    )
                })?;
            (
                Some(SemanticItemIdV2::new(checked.source_id(), query.id)),
                Some(view),
            )
        }
        quanta_gqlang_v2_hir::CatalogPayloadV1::View(
            quanta_gqlang_v2_hir::ViewBodyV1::QueryRef(_),
        )
        | quanta_gqlang_v2_hir::CatalogPayloadV1::Model { .. }
        | quanta_gqlang_v2_hir::CatalogPayloadV1::Pack { .. }
        | quanta_gqlang_v2_hir::CatalogPayloadV1::Profile { .. }
        | quanta_gqlang_v2_hir::CatalogPayloadV1::Gate { .. }
        | quanta_gqlang_v2_hir::CatalogPayloadV1::RuleBinding { .. } => (None, None),
    };
    let catalog = ResolvedCatalogItemV2::try_new(
        SemanticItemIdV2::new(checked.source_id(), decl.id),
        asset.id().clone(),
        provider_metadata,
        expected_inline_view_id,
        inline_view,
        inline_views
            .direct_references
            .get(&decl.id)
            .cloned()
            .unwrap_or_default(),
        payload,
    )
    .map_err(|error| {
        document_failure(
            "sema.invalid_item_shape",
            checked.source_id(),
            decl.span,
            error,
        )
    })?;
    Ok(catalog)
}
