//! Dependency-first inline View materialization from the Modules topology proof.

use super::super::{
    document_failure, inline_query_for_decl, materialize_query, BTreeMap, BindResultV1,
    ClosedFrontendAuthorityV2, DeclaredDocumentV2, DocumentViewTopologyProofV2,
    FrontendTransactionErrorV2, InlineViewPlanV2, RelationOutputContractV2, SemanticStepCounterV2,
    SourceQualifiedNodeIdV2, SpanV1, TypedExprV2, TypedExpressionFactsV2, TypedRelationExprV2,
    TypedRelationV2, ViewMaterializationInputsV2,
};
use quanta_gqlang_v2_modules::CatalogAssetIdV2;

pub(super) struct InlineViewBodiesV2 {
    pub(super) views: BTreeMap<CatalogAssetIdV2, TypedRelationV2>,
    pub(super) outputs: BTreeMap<CatalogAssetIdV2, RelationOutputContractV2>,
    pub(super) expressions: BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    pub(super) nested_relations: BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
}

pub(super) fn materialize_inline_view_bodies(
    declared: &DeclaredDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    inline_views: &InlineViewPlanV2,
    topology: &DocumentViewTopologyProofV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    steps: &mut SemanticStepCounterV2,
) -> Result<InlineViewBodiesV2, FrontendTransactionErrorV2> {
    let checked = declared.checked();
    let syntax = checked.checked().document();
    let mut expressions = BTreeMap::new();
    let mut nested_relations = BTreeMap::new();
    let mut materialized_inline_views = BTreeMap::new();
    let mut inline_view_outputs = BTreeMap::new();
    for asset in topology.dependency_first_order() {
        let Some(declaration) = inline_views.declaration_for(asset) else {
            continue;
        };
        let query = inline_query_for_decl(syntax, declaration).ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                SpanV1::point(0),
                "topology-selected inline View lost its exact HIR query",
            )
        })?;
        let available_views = ViewMaterializationInputsV2 {
            source_targets: &inline_views.source_targets,
            outputs: &inline_view_outputs,
        };
        let typed = materialize_query(
            query,
            checked.source_id(),
            authority,
            bound,
            facts,
            extended,
            &BTreeMap::new(),
            &available_views,
            &mut expressions,
            &mut nested_relations,
            steps,
        )?;
        if inline_view_outputs
            .insert(asset.clone(), typed.output().clone())
            .is_some()
            || materialized_inline_views
                .insert(asset.clone(), typed)
                .is_some()
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                SpanV1::point(0),
                "duplicate dependency-first inline View materialization",
            )
            .into());
        }
    }
    Ok(InlineViewBodiesV2 {
        views: materialized_inline_views,
        outputs: inline_view_outputs,
        expressions,
        nested_relations,
    })
}
