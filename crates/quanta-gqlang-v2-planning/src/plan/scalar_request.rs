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
use quanta_gqlang_v2_algebra::{AdmittedScalarAlgebraV3, AlgebraFamilyV2};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, FrontendRequestMaterializationRootV3, FrontendStorageRefusalV3,
    ScalarSourceProjectReadV3, SemanticItemIdV2,
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
