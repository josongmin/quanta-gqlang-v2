//! The finite one-LET Composition Binder lane. Public scalar compilation still
//! admits only a Query root; this owner only seals lexical Binder evidence.

use super::{Binder, BinderDeclarationRequestErrorV3, RequestProjectDemandV3};
use crate::module_graph::ModuleGraphViewV1;
use crate::resolve::{BindDiagnosticV1, BindingV1, CaptureV1, ResolutionV1};
use quanta_gqlang_v2_hir::{
    CompositionBodyV1, CompositionHirV1, CompositionProducerV1,
    FrontendRequestMaterializationRootV3, HirNodeIdV1, ProjectionItemV1, QueryHirV1,
    RelationExprKindV1, RelationSourceV1, SourceIdV2, ValueExprKindV1,
};

struct QueryPartsV3<'a> {
    query: &'a QueryHirV1,
    scan: &'a RelationSourceV1,
    item: &'a ProjectionItemV1,
    demand: RequestProjectDemandV3,
}

struct QueryScratchV3 {
    query_id: HirNodeIdV1,
    alias_output: Vec<BindingV1>,
    project_input: Vec<BindingV1>,
    project_output: Vec<BindingV1>,
    project_index: Vec<(usize, usize, quanta_gqlang_v2_hir::SpanV1)>,
}

fn exact_vec<T>(count: usize) -> Result<Vec<T>, BinderDeclarationRequestErrorV3> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
    if values.capacity() != count {
        return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
    }
    Ok(values)
}

impl<G: ModuleGraphViewV1> Binder<G> {
    fn composition_query_parts_v3<'a>(
        &self,
        query: &'a QueryHirV1,
    ) -> Result<QueryPartsV3<'a>, BinderDeclarationRequestErrorV3> {
        if query.result_shape.is_some() {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        }
        let RelationExprKindV1::Project(project) = &query.body.kind else {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        };
        let RelationExprKindV1::Source(scan) = &project.input.kind else {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        };
        let [item] = project.items() else {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        };
        let ValueExprKindV1::Path(path) = &item.value.kind else {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        };
        let ValueExprKindV1::Symbol(base) = &path.base.kind else {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        };
        if path.segments.len() != 1
            || scan.alias.as_ref().map(|alias| &alias.name) != Some(&base.name)
        {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        }
        let demand = self
            .request_project_demand_v3(query)?
            .ok_or(BinderDeclarationRequestErrorV3::UnsupportedShape)?;
        if demand.width != 1 || demand.references != 1 {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        }
        Ok(QueryPartsV3 {
            query,
            scan,
            item,
            demand,
        })
    }

    fn composition_storage_bytes_v3(
        authored_name: &str,
        producer: &QueryPartsV3<'_>,
        body: &QueryPartsV3<'_>,
    ) -> Result<usize, BinderDeclarationRequestErrorV3> {
        let overflow = || BinderDeclarationRequestErrorV3::DemandOverflow;
        let output_name = Self::borrowed_projection_output_name_v3;
        let name_bytes = authored_name
            .len()
            .checked_mul(8)
            .and_then(|n| n.checked_add(producer.scan.alias.as_ref()?.name.len()))
            .and_then(|n| n.checked_add(body.scan.alias.as_ref()?.name.len()))
            .and_then(|n| n.checked_add(output_name(producer.item).len()))
            .and_then(|n| n.checked_add(output_name(body.item).len()))
            .and_then(|n| n.checked_add(producer.demand.reference_names))
            .and_then(|n| n.checked_add(producer.demand.local_binding_names))
            .and_then(|n| n.checked_add(body.demand.reference_names))
            .and_then(|n| n.checked_add(body.demand.local_binding_names))
            .ok_or_else(overflow)?;
        let relation_slots = 4_usize
            .checked_mul(std::mem::size_of::<(HirNodeIdV1, Vec<BindingV1>)>())
            .ok_or_else(overflow)?;
        let bindings = 5_usize
            .checked_mul(std::mem::size_of::<BindingV1>())
            .ok_or_else(overflow)?;
        let resolutions = 4_usize
            .checked_mul(std::mem::size_of::<ResolutionV1>())
            .ok_or_else(overflow)?;
        let diagnostics = 4_usize
            .checked_mul(std::mem::size_of::<BindDiagnosticV1>())
            .ok_or_else(overflow)?;
        producer
            .demand
            .scratch_bytes
            .checked_add(body.demand.scratch_bytes)
            .and_then(|n| n.checked_add(name_bytes))
            .and_then(|n| n.checked_add(relation_slots))
            .and_then(|n| n.checked_add(bindings))
            .and_then(|n| n.checked_add(resolutions))
            .and_then(|n| n.checked_add(diagnostics))
            .and_then(|n| n.checked_add(crate::scope::LexicalScopeV1::two_frame_storage_bytes_v3()))
            .ok_or_else(overflow)
    }

    fn composition_scratch_v3(
        parts: &QueryPartsV3<'_>,
    ) -> Result<QueryScratchV3, BinderDeclarationRequestErrorV3> {
        Ok(QueryScratchV3 {
            query_id: parts.query.id,
            alias_output: exact_vec(1)?,
            project_input: exact_vec(1)?,
            project_output: exact_vec(1)?,
            project_index: exact_vec(1)?,
        })
    }

    fn resolve_composition_query_v3(
        &mut self,
        parts: &QueryPartsV3<'_>,
        scratch: QueryScratchV3,
    ) -> Result<(), BinderDeclarationRequestErrorV3> {
        if scratch.query_id != parts.query.id
            || self.request_alias_output.is_some()
            || self.request_project_input.is_some()
            || self.request_project_output.is_some()
            || self.request_project_index.is_some()
        {
            return Err(BinderDeclarationRequestErrorV3::Phase);
        }
        self.request_alias_output = Some(scratch.alias_output);
        self.request_project_input = Some(scratch.project_input);
        self.request_project_output = Some(scratch.project_output);
        self.request_project_index = Some(scratch.project_index);
        self.resolve_query(parts.query);
        if self.request_alias_output.is_some()
            || self.request_project_input.is_some()
            || self.request_project_output.is_some()
            || self.request_project_index.is_some()
        {
            return Err(BinderDeclarationRequestErrorV3::Phase);
        }
        Ok(())
    }

    pub(super) fn resolve_one_let_composition_with_request_owner_v3(
        &mut self,
        composition: &CompositionHirV1,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), BinderDeclarationRequestErrorV3> {
        let [authored] = composition.bindings.as_slice() else {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        };
        let CompositionProducerV1::Query(producer_query) = &authored.producer else {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        };
        let CompositionBodyV1::Query(body_query) = &composition.body else {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        };
        let producer = self.composition_query_parts_v3(producer_query)?;
        let body = self.composition_query_parts_v3(body_query)?;
        if !self.declarations.is_empty()
            || composition.with_name != authored.name
            || producer.scan.name == authored.name
            || body.scan.name != authored.name
            || producer
                .scan
                .alias
                .as_ref()
                .is_some_and(|alias| alias.name == authored.name)
            || body
                .scan
                .alias
                .as_ref()
                .is_some_and(|alias| alias.name == authored.name)
            || Self::borrowed_projection_output_name_v3(producer.item) == authored.name
            || Self::borrowed_projection_output_name_v3(body.item) == authored.name
        {
            return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
        }
        let bytes = Self::composition_storage_bytes_v3(&authored.name, &producer, &body)?;
        self.request_resolution_storage = Some(
            root.reserve_v3(source_id, source, bytes)
                .map_err(BinderDeclarationRequestErrorV3::Storage)?,
        );
        self.scope
            .try_reserve_two_frames_v3()
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        if !self.scope.two_frame_capacity_v3() {
            return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
        }
        self.result
            .resolutions
            .try_reserve_exact(4)
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        self.result
            .diagnostics
            .try_reserve_exact(4)
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        self.result
            .request_relation_outputs
            .try_reserve_exact(4)
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        if self.result.resolutions.capacity() != 4
            || self.result.diagnostics.capacity() != 4
            || self.result.request_relation_outputs.capacity() != 4
        {
            return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
        }
        let mut composition_bindings = exact_vec(1)?;
        let producer_scratch = Self::composition_scratch_v3(&producer)?;
        let body_scratch = Self::composition_scratch_v3(&body)?;
        if !self.scope.push_prepared_frame_v3(
            CaptureV1::NonCapturing,
            std::mem::take(&mut composition_bindings),
        ) {
            return Err(BinderDeclarationRequestErrorV3::Phase);
        }
        self.resolve_composition_query_v3(&producer, producer_scratch)?;
        let introduced = BindingV1::new_indexed(
            authored.name.clone(),
            crate::scope::BinderKindV1::CompositionLet,
            authored.name_span,
            composition.id,
            0,
        );
        self.result.request_composition_binding = Some(((composition.id, 0), introduced.clone()));
        self.introduce(introduced);
        self.resolve_composition_query_v3(&body, body_scratch)?;
        self.resolve_with(
            &composition.with_name,
            composition.id,
            composition.with_name_span,
        );
        self.scope.pop();
        if !self.result.diagnostics.is_empty()
            || self.result.resolutions.len() != 4
            || self.result.request_relation_outputs.len() != 4
            || self.result.request_composition_binding.is_none()
        {
            return Err(BinderDeclarationRequestErrorV3::Phase);
        }
        self.request_resolution_complete = true;
        Ok(())
    }
}
