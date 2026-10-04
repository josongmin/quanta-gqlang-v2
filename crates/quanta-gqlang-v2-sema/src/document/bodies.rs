//! Authored document body resolution after declaration and topology sealing.

use super::{
    BTreeMap, BodyResolutionV2, ClosedFrontendAuthorityV2, DeclaredDocumentV2,
    DocumentCatalogValidatorV2, DocumentViewTopologyProofV2, FrontendTransactionErrorV2,
    HirNodeIdV1, InlineViewPlanV2, PredeclaredDocumentAssetV2, SemanticStepCounterV2,
};

mod catalog;
mod inline_views;
mod prepare;
mod source_order;

pub(super) fn resolve_document_bodies(
    declared: &DeclaredDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    predeclared: &BTreeMap<HirNodeIdV1, PredeclaredDocumentAssetV2>,
    inline_views: &InlineViewPlanV2,
    topology: &DocumentViewTopologyProofV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<BodyResolutionV2, FrontendTransactionErrorV2> {
    let prepared = prepare::prepare_document_body_types(
        declared,
        authority,
        extended,
        inline_views,
        topology,
        validator,
    )?;
    let inline = inline_views::materialize_inline_view_bodies(
        declared,
        authority,
        extended,
        inline_views,
        topology,
        &prepared.bound,
        &prepared.facts,
        steps,
    )?;
    source_order::materialize_source_order(
        declared,
        authority,
        extended,
        predeclared,
        inline_views,
        validator,
        steps,
        prepared,
        inline,
    )
}
