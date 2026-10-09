//! Compatibility imports; DTOs and encoding are owned by execution-contract.

pub use quanta_gqlang_v2_execution_contract::{
    execution_operator_descriptor_v3, ExecutionAuthorityV3, ExecutionBindingV3,
    ExecutionCallTargetV3, ExecutionCardinalityV3, ExecutionCatalogIdentityV3, ExecutionCheckV3,
    ExecutionCommandPhaseV3, ExecutionCompareMetricV3, ExecutionCompositionBindingV3,
    ExecutionCompositionBodyV3, ExecutionCompositionProducerV3, ExecutionCompositionV3,
    ExecutionConfirmModeV3, ExecutionCorrelationV3, ExecutionDetectionModeV3,
    ExecutionDeterminismV3, ExecutionDigestV3, ExecutionDocumentV3, ExecutionEffectV3,
    ExecutionEmitModeV3, ExecutionEvidenceModeV3, ExecutionEvidenceV3, ExecutionExistentialTermV3,
    ExecutionExistsPolarityV3, ExecutionExplainTargetV3, ExecutionExportFormatV3,
    ExecutionExpressionShapeV3, ExecutionExpressionV3, ExecutionFamilyV3, ExecutionFieldV3,
    ExecutionFiniteFloatV3, ExecutionGraphCapabilityV3, ExecutionGraphDedupIdentityV3,
    ExecutionGraphDirectionV3, ExecutionGraphKindV3, ExecutionGraphQueryV3,
    ExecutionGraphReferenceKindV3, ExecutionGraphReferenceRoleV3, ExecutionGraphReferenceV3,
    ExecutionGraphRelationShapeV3, ExecutionGraphTargetV3, ExecutionGraphWitnessTieBreakV3,
    ExecutionItemV3, ExecutionJoinModeV3, ExecutionLiteralV3, ExecutionLogicalQuantifierV3,
    ExecutionNestedGraphV3, ExecutionNullOrderV3, ExecutionOperatorDescriptorErrorV3,
    ExecutionOperatorDescriptorV3, ExecutionOperatorKindV3, ExecutionOperatorV3,
    ExecutionOrderKeyV3, ExecutionPlanV3, ExecutionPolicyFamilyV3, ExecutionPresentationKindV3,
    ExecutionRecordFieldV3, ExecutionRelationAuthorityV3, ExecutionRelationV3,
    ExecutionRelationalOperatorV3, ExecutionRenameCommandV3, ExecutionSampleModeV3,
    ExecutionScalarTypeV3, ExecutionSchemaV3, ExecutionShowCategoryV3, ExecutionSortDirectionV3,
    ExecutionSourceKindV3, ExecutionSourceV3, ExecutionSymbolTargetV3, ExecutionTruthModeV3,
    ExecutionUnionVariantV3, ExecutionValueTypeV3,
};

pub(crate) use quanta_gqlang_v2_execution_contract::producer::{
    project_check, project_composition, project_graph_query, project_presentation,
    project_relation, project_relation_root, ExecutionCompositionBodyProjectionV3,
    ExecutionCompositionProducerProjectionV3, ExecutionCompositionProjectionFailureV3,
    ExecutionProjectionV3,
};
pub(crate) use quanta_gqlang_v2_execution_contract::ExecutionClosureFailureV3;
