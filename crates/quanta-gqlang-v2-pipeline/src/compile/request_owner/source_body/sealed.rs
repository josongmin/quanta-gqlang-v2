//! Catalog, normalized algebra and planning over the same retained scalar request.

use super::MaterializedSourceBodyRequestSourceV3;
use quanta_gqlang_v2_algebra::{
    lower_source_semantic_with_request_owner_v3, AdmittedSourceAlgebraV3, AlgebraFamilyV2,
    ScalarAlgebraRequestErrorV3,
};
use quanta_gqlang_v2_planning::{
    plan_source_algebra_with_request_owner_v3, AdmittedSourcePlanV3, ResultKindV2,
    ScalarPlanningRequestErrorV3,
};
use quanta_gqlang_v2_sema::{
    seal_source_one_let_catalog_with_request_owner_v3,
    seal_source_scalar_catalog_with_request_owner_v3, AdmittedScalarSemanticKindV3,
    AdmittedSourceBodyV3, AdmittedSourceSemanticDocumentV3, SemanticItemIdV2,
};
use quanta_gqlang_v2_surface::{FrontendRequestMaterializationRootV3, FrontendRequestOwnedValueV3};

#[path = "sealed/execution.rs"]
mod execution;
pub use execution::{
    project_source_body_with_request_owner_v3,
    seal_scalar_execution_document_with_request_owner_v3,
    seal_source_execution_document_with_request_owner_v3, AdmittedBindingIdentityReadV3,
    AdmittedCanonicalFilesScalarReadV3, AdmittedOneLetBindingReadV3,
    AdmittedOneLetCompositionReadV3, AdmittedOneLetExecutionDocumentV3, AdmittedProjectScanReadV3,
    AdmittedScalarExecutionDocumentV3, AdmittedSourceExecutionDocumentV3,
    ProjectedSourceBodyRequestSourceV3, ScalarExecutionRequestErrorV3,
    SharedAdmittedOneLetExecutionDocumentV3, SharedAdmittedScalarExecutionDocumentV3,
};

pub use quanta_gqlang_v2_sema::SourceCatalogSealErrorV3;

/// The exact source body and final catalog sealed by its issuing Modules session.
pub struct CatalogSealedSourceBodyRequestSourceV3<'authority> {
    owner: FrontendRequestOwnedValueV3<AdmittedSourceSemanticDocumentV3<'authority>>,
}

impl CatalogSealedSourceBodyRequestSourceV3<'_> {
    #[must_use]
    pub fn authored_item_id_v3(&self) -> SemanticItemIdV2 {
        self.owner.value_v3().authored_item_id_v3()
    }

    #[must_use]
    pub fn semantic_kind_v3(&self) -> AdmittedScalarSemanticKindV3 {
        self.owner.value_v3().kind_v3()
    }

    #[must_use]
    pub fn catalog_asset_count_v3(&self) -> usize {
        self.owner.value_v3().catalog_asset_count_v3()
    }

    #[must_use]
    pub fn catalog_dependency_count_v3(&self) -> usize {
        self.owner.value_v3().catalog_dependency_count_v3()
    }
}

/// Move-only normalization with all preceding source and semantic custody.
pub struct LoweredSourceBodyRequestSourceV3<'authority> {
    owner: FrontendRequestOwnedValueV3<AdmittedSourceAlgebraV3<'authority>>,
}

impl LoweredSourceBodyRequestSourceV3<'_> {
    #[must_use]
    pub fn root_family_v3(&self) -> AlgebraFamilyV2 {
        self.owner.value_v3().root_family_v3()
    }

    #[must_use]
    pub fn output_field_count_v3(&self) -> usize {
        self.owner.value_v3().output_field_count_v3()
    }
}

/// Checked finite plan whose borrowed observations retain the original source.
pub struct PlannedSourceBodyRequestSourceV3<'authority> {
    owner: FrontendRequestOwnedValueV3<AdmittedSourcePlanV3<'authority>>,
}

impl PlannedSourceBodyRequestSourceV3<'_> {
    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        self.owner.value_v3().item_id_v3()
    }

    #[must_use]
    pub fn result_kind_v3(&self) -> ResultKindV2 {
        self.owner.value_v3().result_kind_v3()
    }

    #[must_use]
    pub fn result_v3(&self) -> &quanta_gqlang_v2_planning::ResultContractV2 {
        self.owner.value_v3().result_v3()
    }

    #[must_use]
    pub fn authority_lane_v3(&self) -> quanta_gqlang_v2_planning::AuthorityLaneV2 {
        self.owner.value_v3().authority_lane_v3()
    }

    #[must_use]
    pub fn evaluation_determinism_v3(&self) -> quanta_gqlang_v2_planning::EvaluationDeterminismV2 {
        self.owner.value_v3().evaluation_determinism_v3()
    }

    #[must_use]
    pub fn evidence_v3(&self) -> quanta_gqlang_v2_planning::EvidenceRequirementV2 {
        self.owner.value_v3().evidence_v3()
    }

    #[must_use]
    pub fn effect_v3(&self) -> quanta_gqlang_v2_planning::EffectClassV2 {
        self.owner.value_v3().effect_v3()
    }

    #[must_use]
    pub fn effect_summary_v3(&self) -> quanta_gqlang_v2_planning::EffectSummaryV2 {
        self.owner.value_v3().effect_summary_v3()
    }

    #[must_use]
    pub fn semantic_cost_v3(&self) -> quanta_gqlang_v2_planning::SemanticCostV2 {
        self.owner.value_v3().semantic_cost_v3()
    }

    #[must_use]
    pub fn policy_v3(&self) -> &quanta_gqlang_v2_planning::PlanPolicyV2 {
        self.owner.value_v3().policy_v3()
    }

    #[must_use]
    pub fn scan_capability_v3(&self) -> quanta_gqlang_v2_planning::CapabilityV2 {
        self.owner.value_v3().scan_capability_v3()
    }

    #[must_use]
    pub fn explain_families_v3(&self) -> &'static [AlgebraFamilyV2] {
        self.owner.value_v3().explain_families_v3()
    }

    #[must_use]
    pub fn root_location_v3(&self) -> &quanta_gqlang_v2_sema::CheckedLocatedSpanV2 {
        self.owner.value_v3().root_location_v3()
    }

    #[must_use]
    pub fn capability_count_v3(&self) -> usize {
        self.owner.value_v3().capability_count_v3()
    }

    #[must_use]
    pub fn explain_family_count_v3(&self) -> usize {
        self.owner.value_v3().explain_family_count_v3()
    }
}

/// Sema owns final catalog validation and same-session semantic assembly.
pub fn seal_source_catalog_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    materialized: MaterializedSourceBodyRequestSourceV3<'authority>,
) -> Result<CatalogSealedSourceBodyRequestSourceV3<'authority>, SourceCatalogSealErrorV3> {
    let owner = materialized.owner.try_map_v3(
        |(_extended, session, _assets, topology, body), parent| match body {
            AdmittedSourceBodyV3::Scalar(body) => seal_source_scalar_catalog_with_request_owner_v3(
                session, topology, body, root, source, parent,
            )
            .map(AdmittedSourceSemanticDocumentV3::Scalar),
            AdmittedSourceBodyV3::OneLet(body) => {
                seal_source_one_let_catalog_with_request_owner_v3(
                    session, topology, body, root, source, parent,
                )
                .map(AdmittedSourceSemanticDocumentV3::OneLet)
            }
        },
    )?;
    Ok(CatalogSealedSourceBodyRequestSourceV3 { owner })
}

/// Validate the normalized scalar algebra through its sealed semantic producer.
pub fn lower_source_body_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    sealed: CatalogSealedSourceBodyRequestSourceV3<'authority>,
) -> Result<LoweredSourceBodyRequestSourceV3<'authority>, ScalarAlgebraRequestErrorV3> {
    let owner = sealed.owner.try_map_v3(|semantic, _parent| {
        lower_source_semantic_with_request_owner_v3(semantic, root, source)
    })?;
    Ok(LoweredSourceBodyRequestSourceV3 { owner })
}

/// Classify the bounded source plan from the admitted normalized algebra view.
pub fn plan_source_body_with_request_owner_v3<'authority>(
    source: &str,
    root: &FrontendRequestMaterializationRootV3<'_>,
    lowered: LoweredSourceBodyRequestSourceV3<'authority>,
) -> Result<PlannedSourceBodyRequestSourceV3<'authority>, ScalarPlanningRequestErrorV3> {
    let owner = lowered.owner.try_map_v3(|algebra, _parent| {
        plan_source_algebra_with_request_owner_v3(algebra, root, source)
    })?;
    Ok(PlannedSourceBodyRequestSourceV3 { owner })
}
