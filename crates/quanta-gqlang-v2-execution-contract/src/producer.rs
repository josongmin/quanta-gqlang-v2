//! Internal producer bridge. The curated downstream facade does not export it.
//!
//! Compilation supplies sealed stage facts; this owner constructs private DTO
//! fields and seals its own item buffer. No constructor accepts a caller-built
//! ExecutionItem vector or supplied digest. Runtime admission remains separate.

pub use crate::projection::{
    project_check, project_composition, project_graph_query, project_presentation,
    project_relation, project_relation_root, ExecutionCompositionBodyProjectionV3,
    ExecutionCompositionProducerProjectionV3, ExecutionCompositionProjectionFailureV3,
};
pub use crate::scalar_request::{scalar_relation_projection_demand_v3, ScalarProjectionFailureV3};
use crate::{
    ExecutionCheckV3, ExecutionCompositionV3, ExecutionDocumentV3, ExecutionFamilyV3,
    ExecutionGraphQueryV3, ExecutionItemV3, ExecutionPresentationKindV3, ExecutionRelationV3,
    SemanticItemIdV2, SourceIdV2,
};
use quanta_gqlang_v2_planning::{AdmittedScalarPlanV3, SemanticPlanV2};
use quanta_gqlang_v2_sema::{ResolvedPresentationV2, ScalarSourceProjectReadV3};

/// Move-only, producer-owned buffer. Its layout is the original item Vec;
/// transferring ownership into the sealed document adds no intermediate buffer.
#[derive(Debug, Default)]
pub struct ExecutionProjectionV3 {
    items: Vec<ExecutionItemV3>,
}

impl ExecutionProjectionV3 {
    pub fn try_with_capacity(count: usize) -> Result<Self, ScalarProjectionFailureV3> {
        Ok(Self {
            items: crate::scalar_request::bounded::slots(count)?,
        })
    }

    pub fn push_planned(
        &mut self,
        item_id: SemanticItemIdV2,
        family: ExecutionFamilyV3,
        relation: Option<ExecutionRelationV3>,
        graph: Option<ExecutionGraphQueryV3>,
        check: Option<ExecutionCheckV3>,
        presentation: Option<ExecutionPresentationKindV3>,
        composition: Option<ExecutionCompositionV3>,
        plan: &SemanticPlanV2,
    ) {
        self.items.push(ExecutionItemV3::new(
            item_id,
            family,
            relation,
            graph,
            check,
            presentation,
            composition,
            plan,
        ));
    }

    /// Uses the same plan/semantic agreement check as the selected SHOW path.
    pub fn push_show(
        &mut self,
        resolved: &ResolvedPresentationV2,
        plan: &SemanticPlanV2,
    ) -> Result<(), &'static str> {
        self.items
            .push(crate::projection::project_selected_show_item_v2(
                resolved, plan,
            )?);
        Ok(())
    }

    pub fn finish(self, source_id: SourceIdV2, source: &[u8]) -> ExecutionDocumentV3 {
        ExecutionDocumentV3::new(source_id, source, self.items)
    }
}

/// One scalar projection issued from the admitted plan and semantic read.
/// The wrapper has the DTO's layout and cannot be created from a cloned DTO.
#[repr(transparent)]
pub struct ProjectedScalarItemV3(ExecutionItemV3);

impl ProjectedScalarItemV3 {
    pub fn item_id(&self) -> SemanticItemIdV2 {
        self.0.item_id()
    }

    pub fn family(&self) -> ExecutionFamilyV3 {
        self.0.family()
    }

    pub fn relation(&self) -> Option<&ExecutionRelationV3> {
        self.0.relation()
    }
}

/// Invoke only after the caller reserves scalar_relation_projection_demand_v3.
pub fn project_scalar_item_v3(
    read: &ScalarSourceProjectReadV3<'_, '_>,
    plan: &AdmittedScalarPlanV3<'_>,
) -> Result<ProjectedScalarItemV3, ScalarProjectionFailureV3> {
    let relation = crate::scalar_request::project_scalar_relation_v3(read)?;
    Ok(ProjectedScalarItemV3(
        ExecutionItemV3::new_scalar_relation_v3(read.item_id_v3(), relation, plan),
    ))
}

/// Invoke after reserving two item slots (Vec plus possible Box replacement).
pub fn seal_scalar_item_v3(
    item: ProjectedScalarItemV3,
    source: &[u8],
) -> Result<ExecutionDocumentV3, ScalarProjectionFailureV3> {
    let mut items = crate::scalar_request::bounded::slots(1)?;
    let source_id = item.item_id().source_id();
    items.push(item.0);
    Ok(ExecutionDocumentV3::new(source_id, source, items))
}
