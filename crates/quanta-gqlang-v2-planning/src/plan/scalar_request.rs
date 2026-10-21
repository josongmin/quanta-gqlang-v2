//! Compact request-owned plan for the checked Project(Scan) family.

use super::{classified_result_and_cost, determinism_of, relational_facts_from_shape};
use crate::capability::CapabilityV2;
use crate::cost::{CardinalityClassV2, SemanticCostV2};
use crate::effect::{
    AuthorityLaneV2, EffectClassV2, EvaluationDeterminismV2, EvidenceRequirementV2,
};
use crate::policy::{EffectSummaryV2, PlanPolicyV2};
use crate::result_kind::{ResultContractV2, ResultKindV2};
use crate::PlanningDiagnosticV1;
use quanta_gqlang_v2_algebra::{
    AdmittedOneLetAlgebraV3, AdmittedScalarAlgebraV3, AdmittedSourceAlgebraV3, AlgebraFamilyV2,
};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3,
    FrontendStorageRefusalV3, ScalarSourceProjectReadV3, SemanticItemIdV2,
};
use quanta_gqlang_v2_types::{CardinalityV1, ExpressionEffectsV2};

/// The original Algebra owner and a fixed-size relational classification move
/// together. No BTree capability set or family Vec is cloned merely to publish
/// the finite Project(Scan) observations.
pub struct AdmittedScalarPlanV3<'authority> {
    algebra: AdmittedScalarAlgebraV3<'authority>,
    result: ResultContractV2,
    cost: SemanticCostV2,
    effects: EffectSummaryV2,
    policy: PlanPolicyV2,
    root_location: CheckedLocatedSpanV2,
}

/// The Composition policy retains its exact same-request Binder binding in
/// Algebra and the canonical relational policies of both checked Query roots.
pub struct AdmittedOneLetPlanV3<'authority> {
    policy: PlanPolicyV2,
    root_location: CheckedLocatedSpanV2,
    _policy_storage: FrontendRetainedStoragePermitV3,
    algebra: AdmittedOneLetAlgebraV3<'authority>,
    result: ResultContractV2,
    cost: SemanticCostV2,
    effects: EffectSummaryV2,
}

pub enum AdmittedSourcePlanV3<'authority> {
    Scalar(AdmittedScalarPlanV3<'authority>),
    OneLet(AdmittedOneLetPlanV3<'authority>),
}

impl AdmittedOneLetPlanV3<'_> {
    #[must_use]
    pub fn algebra_v3(&self) -> &AdmittedOneLetAlgebraV3<'_> {
        &self.algebra
    }

    #[must_use]
    pub fn policy_v3(&self) -> &PlanPolicyV2 {
        &self.policy
    }

    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        self.algebra.semantic_v3().authored_item_id_v3()
    }

    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.algebra.validate_owner_v3(root, source)?;
        self._policy_storage
            .validate_owner_v3(root, self.item_id_v3().source_id(), source)
    }
}

impl AdmittedSourcePlanV3<'_> {
    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        match self {
            Self::Scalar(value) => value.item_id_v3(),
            Self::OneLet(value) => value.item_id_v3(),
        }
    }

    #[must_use]
    pub fn result_kind_v3(&self) -> ResultKindV2 {
        self.result_v3().result_kind()
    }

    #[must_use]
    pub fn result_v3(&self) -> &ResultContractV2 {
        match self {
            Self::Scalar(value) => value.result_v3(),
            Self::OneLet(value) => &value.result,
        }
    }

    #[must_use]
    pub const fn authority_lane_v3(&self) -> AuthorityLaneV2 {
        AuthorityLaneV2::FrontendChecked
    }

    #[must_use]
    pub const fn evidence_v3(&self) -> EvidenceRequirementV2 {
        EvidenceRequirementV2::None
    }

    #[must_use]
    pub const fn effect_v3(&self) -> EffectClassV2 {
        EffectClassV2::ReadOnly
    }

    #[must_use]
    pub fn evaluation_determinism_v3(&self) -> EvaluationDeterminismV2 {
        determinism_of(self.effect_summary_v3())
    }

    #[must_use]
    pub fn effect_summary_v3(&self) -> EffectSummaryV2 {
        match self {
            Self::Scalar(value) => value.effect_summary_v3(),
            Self::OneLet(value) => value.effects,
        }
    }

    #[must_use]
    pub fn semantic_cost_v3(&self) -> SemanticCostV2 {
        match self {
            Self::Scalar(value) => value.semantic_cost_v3(),
            Self::OneLet(value) => value.cost,
        }
    }

    /// The one-LET root retains the same checked Composition policy as the
    /// ordinary plan, including its producer output and exact WITH binding.
    #[must_use]
    pub fn policy_v3(&self) -> &PlanPolicyV2 {
        match self {
            Self::Scalar(value) => value.policy_v3(),
            Self::OneLet(value) => value.policy_v3(),
        }
    }

    #[must_use]
    pub const fn scan_capability_v3(&self) -> CapabilityV2 {
        CapabilityV2::RelationScan
    }

    #[must_use]
    pub const fn capability_count_v3(&self) -> usize {
        1
    }

    #[must_use]
    pub const fn explain_families_v3(&self) -> &'static [AlgebraFamilyV2] {
        const SCALAR: &[AlgebraFamilyV2] = &[AlgebraFamilyV2::Project, AlgebraFamilyV2::Scan];
        match self {
            Self::Scalar(_) => SCALAR,
            // The canonical Composition plan is its body's plan. The LET
            // producer is retained as authority, not appended to Explain.
            Self::OneLet(_) => SCALAR,
        }
    }

    #[must_use]
    pub fn explain_family_count_v3(&self) -> usize {
        self.explain_families_v3().len()
    }

    #[must_use]
    pub fn root_location_v3(&self) -> &CheckedLocatedSpanV2 {
        match self {
            Self::Scalar(value) => value.root_location_v3(),
            Self::OneLet(value) => &value.root_location,
        }
    }

    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        match self {
            Self::Scalar(value) => value.validate_owner_v3(root, source),
            Self::OneLet(value) => value.validate_owner_v3(root, source),
        }
    }
}

impl AdmittedScalarPlanV3<'_> {
    #[must_use]
    pub fn scalar_semantic_read_v3(&self) -> Option<ScalarSourceProjectReadV3<'_, '_>> {
        self.algebra.scalar_semantic_read_v3()
    }

    #[must_use]
    pub fn item_id_v3(&self) -> SemanticItemIdV2 {
        self.algebra.item_id_v3()
    }

    #[must_use]
    pub fn result_kind_v3(&self) -> ResultKindV2 {
        self.result.result_kind()
    }

    #[must_use]
    pub fn result_v3(&self) -> &ResultContractV2 {
        &self.result
    }

    #[must_use]
    pub const fn authority_lane_v3(&self) -> AuthorityLaneV2 {
        AuthorityLaneV2::FrontendChecked
    }

    #[must_use]
    pub const fn evidence_v3(&self) -> EvidenceRequirementV2 {
        EvidenceRequirementV2::None
    }

    #[must_use]
    pub const fn effect_v3(&self) -> EffectClassV2 {
        EffectClassV2::ReadOnly
    }

    #[must_use]
    pub fn evaluation_determinism_v3(&self) -> EvaluationDeterminismV2 {
        determinism_of(self.effects)
    }

    #[must_use]
    pub const fn effect_summary_v3(&self) -> EffectSummaryV2 {
        self.effects
    }

    #[must_use]
    pub const fn semantic_cost_v3(&self) -> SemanticCostV2 {
        self.cost
    }

    #[must_use]
    pub const fn policy_node_count_v3(&self) -> usize {
        2
    }

    #[must_use]
    pub const fn policy_v3(&self) -> &PlanPolicyV2 {
        &self.policy
    }

    #[must_use]
    pub const fn capability_count_v3(&self) -> usize {
        1
    }

    #[must_use]
    pub const fn scan_capability_v3(&self) -> CapabilityV2 {
        CapabilityV2::RelationScan
    }

    #[must_use]
    pub const fn explain_family_count_v3(&self) -> usize {
        2
    }

    #[must_use]
    pub const fn explain_families_v3(&self) -> [AlgebraFamilyV2; 2] {
        [AlgebraFamilyV2::Project, AlgebraFamilyV2::Scan]
    }

    #[must_use]
    pub const fn root_location_v3(&self) -> &CheckedLocatedSpanV2 {
        &self.root_location
    }

    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.algebra.validate_owner_v3(root, source)
    }
}

#[derive(Debug)]
pub enum ScalarPlanningRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    UnsupportedShape,
    DemandOverflow,
    Allocation,
    SemanticSteps(PlanningDiagnosticV1),
    Composition(PlanningDiagnosticV1),
}

fn derive_scalar_plan_parts_v3(
    cardinality: CardinalityV1,
    project_effects: ExpressionEffectsV2,
    scan_effects: ExpressionEffectsV2,
    expression_effects: impl IntoIterator<Item = Option<ExpressionEffectsV2>>,
) -> Option<(
    ResultContractV2,
    SemanticCostV2,
    EffectSummaryV2,
    PlanPolicyV2,
)> {
    let mut joined = ExpressionEffectsV2::pure_scalar().join(project_effects);
    for expression in expression_effects {
        joined = joined.join(expression?);
    }
    joined = joined.join(scan_effects);
    let effects = EffectSummaryV2::new(
        joined.purity,
        joined.determinism,
        joined.monotonicity,
        joined.aggregation,
    );
    let facts =
        relational_facts_from_shape(cardinality, CardinalityClassV2::LinearInInput, 2, effects);
    let (result, cost) = classified_result_and_cost(&facts);
    Some((result, cost, effects, facts.policy))
}

/// Consume the checked scalar Algebra owner. The finite classification reads
/// Project then Scan in the ordinary walk order and retains the same typed
/// relational result, effect fold, and semantic cost with no new heap birth.
pub fn plan_scalar_source_project_with_request_owner_v3<'authority>(
    algebra: AdmittedScalarAlgebraV3<'authority>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
) -> Result<AdmittedScalarPlanV3<'authority>, ScalarPlanningRequestErrorV3> {
    algebra
        .validate_owner_v3(root, source)
        .map_err(ScalarPlanningRequestErrorV3::Storage)?;
    let (result, cost, effects, policy, root_location) = {
        let read = algebra
            .scalar_read_v3()
            .ok_or(ScalarPlanningRequestErrorV3::UnsupportedShape)?;
        let limit = read.budget_receipt_v3().max_semantic_steps();
        if limit < 2 {
            return Err(ScalarPlanningRequestErrorV3::SemanticSteps(
                PlanningDiagnosticV1::SemanticStepBudgetExceeded {
                    limit,
                    span: read.scan_location_v3().clone(),
                },
            ));
        }
        let (result, cost, effects, policy) = derive_scalar_plan_parts_v3(
            read.project_cardinality_v3(),
            read.project_effects_v3(),
            read.scan_effects_v3(),
            read.project_expression_effects_v3(),
        )
        .ok_or(ScalarPlanningRequestErrorV3::UnsupportedShape)?;
        (
            result,
            cost,
            effects,
            policy,
            read.authored_item_location_v3().clone(),
        )
    };
    Ok(AdmittedScalarPlanV3 {
        algebra,
        result,
        cost,
        effects,
        policy,
        root_location,
    })
}

/// Classify a genuine one-LET Composition from the same Sema/Algebra owner.
/// Both relational policies use the canonical scalar classifier; root and
/// ordered binding identity use the ordinary Composition retention validator.
pub fn plan_source_algebra_with_request_owner_v3<'authority>(
    algebra: AdmittedSourceAlgebraV3<'authority>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
) -> Result<AdmittedSourcePlanV3<'authority>, ScalarPlanningRequestErrorV3> {
    match algebra {
        AdmittedSourceAlgebraV3::Scalar(value) => {
            plan_scalar_source_project_with_request_owner_v3(value, root, source)
                .map(AdmittedSourcePlanV3::Scalar)
        }
        AdmittedSourceAlgebraV3::OneLet(value) => {
            value
                .validate_owner_v3(root, source)
                .map_err(ScalarPlanningRequestErrorV3::Storage)?;
            let semantic = value.semantic_v3();
            let reads = value
                .source_project_reads_v3()
                .ok_or(ScalarPlanningRequestErrorV3::UnsupportedShape)?;
            let limit = reads[1].budget_receipt_v3().max_semantic_steps();
            if limit < 6 {
                return Err(ScalarPlanningRequestErrorV3::SemanticSteps(
                    PlanningDiagnosticV1::SemanticStepBudgetExceeded {
                        limit,
                        span: reads[0].scan_location_v3().clone(),
                    },
                ));
            }
            let binding = semantic
                .with_binding_v3()
                .ok_or(ScalarPlanningRequestErrorV3::UnsupportedShape)?;
            crate::composition::validate_composition_retention_fields_v3(
                semantic.authored_item_id_v3(),
                std::iter::once((
                    0,
                    semantic.producer_item_id_v3(),
                    semantic.binding_name_location_v3(),
                )),
                binding,
                crate::composition::CompositionBodyIdentityV2::new(
                    crate::composition::CompositionBodyFamilyV2::Query,
                    semantic.body_item_id_v3(),
                ),
            )
            .map_err(|reason| {
                ScalarPlanningRequestErrorV3::Composition(
                    PlanningDiagnosticV1::InvalidComposition {
                        reason,
                        span: semantic.authored_item_location_v3().clone(),
                    },
                )
            })?;
            let producer = &reads[0];
            let body = &reads[1];
            let (result, cost, effects, body_policy) = derive_scalar_plan_parts_v3(
                body.project_output_v3().cardinality(),
                body.project_output_v3()
                    .effects()
                    .join(producer.project_output_v3().effects()),
                body.scan_output_v3()
                    .effects()
                    .join(producer.scan_output_v3().effects()),
                producer
                    .project_expression_keys_v3()
                    .map(|key| {
                        producer
                            .expression_v3(key)
                            .map(quanta_gqlang_v2_sema::TypedExprV2::effects)
                    })
                    .chain(body.project_expression_keys_v3().map(|key| {
                        body.expression_v3(key)
                            .map(quanta_gqlang_v2_sema::TypedExprV2::effects)
                    })),
            )
            .ok_or(ScalarPlanningRequestErrorV3::UnsupportedShape)?;
            let root_location = semantic.authored_item_location_v3().clone();
            let producer_output = semantic.producer_output_v3();
            let demand = crate::composition::onelet_request_policy_storage_upper_v3(
                producer_output,
                binding,
            )
            .map_err(|error| match error {
                crate::composition::OneLetPolicyStorageDemandErrorV3::UnsupportedShape => {
                    ScalarPlanningRequestErrorV3::UnsupportedShape
                }
                crate::composition::OneLetPolicyStorageDemandErrorV3::DemandOverflow => {
                    ScalarPlanningRequestErrorV3::DemandOverflow
                }
            })?;
            let policy_storage = root
                .reserve_v3(semantic.authored_item_id_v3().source_id(), source, demand)
                .map_err(ScalarPlanningRequestErrorV3::Storage)?;
            let mut producers = Vec::new();
            producers
                .try_reserve_exact(1)
                .map_err(|_allocation| ScalarPlanningRequestErrorV3::Allocation)?;
            if producers.capacity() != 1 {
                return Err(ScalarPlanningRequestErrorV3::Allocation);
            }
            let invalid_composition = |reason| {
                ScalarPlanningRequestErrorV3::Composition(
                    PlanningDiagnosticV1::InvalidComposition {
                        reason,
                        span: root_location.clone(),
                    },
                )
            };
            producers.push(
                crate::composition::CheckedCompositionProducerV2::try_new(
                    0,
                    semantic.producer_item_id_v3(),
                    semantic.binding_name_location_v3().clone(),
                    Some(producer_output.clone()),
                )
                .map_err(invalid_composition)?,
            );
            let retention = crate::composition::CompositionRetentionV2::try_new(
                semantic.authored_item_id_v3(),
                root_location.clone(),
                producers,
                binding.clone(),
                crate::composition::CompositionBodyIdentityV2::new(
                    crate::composition::CompositionBodyFamilyV2::Query,
                    semantic.body_item_id_v3(),
                ),
            )
            .map_err(invalid_composition)?;
            let retained_body = crate::composition::CompositionBodyPolicyV2::try_new(body_policy)
                .map_err(invalid_composition)?;
            let policy = PlanPolicyV2::Composition(
                crate::composition::CompositionPolicyV2::try_new(retention, retained_body)
                    .map_err(invalid_composition)?,
            );
            Ok(AdmittedSourcePlanV3::OneLet(AdmittedOneLetPlanV3 {
                algebra: value,
                result,
                cost,
                effects,
                policy,
                _policy_storage: policy_storage,
                root_location,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{plan, PlanPolicyV2, SemanticPlanInputV2};
    use quanta_gqlang_v2_algebra::lower_relation_item;
    use quanta_gqlang_v2_sema::test_support::{
        reference_relation_document_v2, ReferenceRelationFixtureV2,
    };
    use quanta_gqlang_v2_sema::SemanticItemV2;

    #[test]
    #[expect(
        clippy::expect_used,
        clippy::panic,
        reason = "sealed V2 fixture is the independent oracle"
    )]
    fn finite_scalar_summary_matches_ordinary_v2_planning() {
        let fixture =
            reference_relation_document_v2(ReferenceRelationFixtureV2::ProjectMultipleExpressions)
                .expect("sealed project fixture");
        let lowered = lower_relation_item(fixture.document(), fixture.item_id())
            .expect("ordinary algebra lowers");
        let root = lowered.root();
        assert_eq!(root.family(), AlgebraFamilyV2::Project);
        let children = root.children();
        let scan = children.first().expect("one checked input");
        assert_eq!(scan.family(), AlgebraFamilyV2::Scan);
        let (result, cost, effects, policy) = derive_scalar_plan_parts_v3(
            root.output().cardinality(),
            root.output().effects(),
            scan.output().effects(),
            root.expressions()
                .iter()
                .map(|expression| Some(expression.effects())),
        )
        .expect("all expression facts exist");
        let root_location = root.location().clone();
        let expected = plan(
            &SemanticPlanInputV2::Relational(Box::new(lowered)),
            fixture.document(),
        )
        .expect("ordinary plan");
        assert_eq!(&result, expected.result());
        assert_eq!(&cost, expected.semantic_cost());
        assert_eq!(&policy, expected.policy());
        assert_eq!(determinism_of(effects), expected.evaluation_determinism());
        assert_eq!(expected.authority_lane(), AuthorityLaneV2::FrontendChecked);
        assert_eq!(expected.effect(), EffectClassV2::ReadOnly);
        assert_eq!(expected.evidence(), EvidenceRequirementV2::None);
        assert_eq!(
            expected.capabilities().iter().copied().collect::<Vec<_>>(),
            [CapabilityV2::RelationScan]
        );
        assert_eq!(
            expected.explain().families(),
            &[AlgebraFamilyV2::Project, AlgebraFamilyV2::Scan]
        );
        assert_ne!(expected.explain().root_location(), &root_location);
        let Some(SemanticItemV2::Relation(authored)) = fixture.document().item(fixture.item_id())
        else {
            panic!("authored fixture item is a relation");
        };
        assert_eq!(expected.explain().root_location(), authored.location());
        let PlanPolicyV2::Relational {
            nodes,
            effects: policy,
        } = expected.policy()
        else {
            panic!("ordinary scalar plan retains relational policy");
        };
        assert_eq!(*nodes, 2);
        assert_eq!(*policy, effects);
    }
}
