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
pub use crate::scalar_request::{
    onelet_composition_projection_demand_v3, scalar_relation_projection_demand_v3,
    ScalarProjectionFailureV3,
};
use crate::{
    ExecutionCheckV3, ExecutionCompositionBindingV3, ExecutionCompositionBodyV3,
    ExecutionCompositionProducerV3, ExecutionCompositionV3, ExecutionDocumentV3, ExecutionFamilyV3,
    ExecutionGraphQueryV3, ExecutionItemV3, ExecutionPresentationKindV3, ExecutionRelationV3,
    SemanticItemIdV2, SourceIdV2,
};
use quanta_gqlang_v2_planning::{AdmittedScalarPlanV3, AdmittedSourcePlanV3, SemanticPlanV2};
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

#[repr(transparent)]
pub struct ProjectedOneLetItemV3(ExecutionItemV3);

pub enum ProjectedSourceItemV3 {
    Scalar(ProjectedScalarItemV3),
    OneLet(ProjectedOneLetItemV3),
}

impl ProjectedSourceItemV3 {
    pub fn item_id(&self) -> SemanticItemIdV2 {
        match self {
            Self::Scalar(value) => value.0.item_id(),
            Self::OneLet(value) => value.0.item_id(),
        }
    }

    pub fn family(&self) -> ExecutionFamilyV3 {
        match self {
            Self::Scalar(value) => value.0.family(),
            Self::OneLet(value) => value.0.family(),
        }
    }

    pub fn relation(&self) -> Option<&ExecutionRelationV3> {
        match self {
            Self::Scalar(value) => value.0.relation(),
            Self::OneLet(value) => value.0.relation(),
        }
    }
}

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

/// Invoke only after reserving onelet_composition_projection_demand_v3 under
/// the same request root as both retained Query relations and Binder facts.
pub fn project_onelet_item_v3(
    source_plan: &AdmittedSourcePlanV3<'_>,
) -> Result<ProjectedOneLetItemV3, ScalarProjectionFailureV3> {
    let AdmittedSourcePlanV3::OneLet(plan) = source_plan else {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    };
    let semantic = plan.algebra_v3().semantic_v3();
    let reads = plan
        .algebra_v3()
        .source_project_reads_v3()
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    let binding = semantic
        .with_binding_v3()
        .ok_or(ScalarProjectionFailureV3::UnsupportedShape)?;
    if reads[0].item_id_v3() != semantic.producer_item_id_v3()
        || reads[1].item_id_v3() != semantic.body_item_id_v3()
    {
        return Err(ScalarProjectionFailureV3::UnsupportedShape);
    }
    let producer = crate::scalar_request::project_scalar_relation_v3(&reads[0])?;
    let body = crate::scalar_request::project_scalar_relation_v3(&reads[1])?;
    let binding_projection = crate::scalar_request::bounded::composition_binding(binding)?;
    let with_binding = crate::scalar_request::bounded::composition_binding(binding)?;
    let schema = crate::scalar_request::bounded::schema(semantic.producer_output_v3())?;
    let mut bindings = crate::scalar_request::bounded::slots::<ExecutionCompositionBindingV3>(1)?;
    bindings.push(ExecutionCompositionBindingV3 {
        binding: binding_projection,
        producer_item_id: semantic.producer_item_id_v3(),
        producer_family: ExecutionFamilyV3::Relation,
        schema: Some(schema),
        relation: Some(producer),
        producer: ExecutionCompositionProducerV3::Relation,
    });
    let composition = ExecutionCompositionV3 {
        bindings: bindings.into_boxed_slice(),
        with_binding,
        body_item_id: semantic.body_item_id_v3(),
        body_family: ExecutionFamilyV3::Relation,
        body_relation: Some(body),
        body: ExecutionCompositionBodyV3::Relation,
    };
    Ok(ProjectedOneLetItemV3(ExecutionItemV3 {
        item_id: semantic.authored_item_id_v3(),
        family: ExecutionFamilyV3::Composition,
        relation: None,
        graph: None,
        check: None,
        presentation: None,
        composition: Some(composition),
        command: None,
        plan: crate::vocabulary::project_source_plan(source_plan),
    }))
}

/// One admitted item slot, independent of the selected source family.
pub fn seal_source_item_v3(
    item: ProjectedSourceItemV3,
    source: &[u8],
) -> Result<ExecutionDocumentV3, ScalarProjectionFailureV3> {
    let item = match item {
        ProjectedSourceItemV3::Scalar(value) => value.0,
        ProjectedSourceItemV3::OneLet(value) => value.0,
    };
    let mut items = crate::scalar_request::bounded::slots(1)?;
    let source_id = item.item_id().source_id();
    items.push(item);
    Ok(ExecutionDocumentV3::new(source_id, source, items))
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
