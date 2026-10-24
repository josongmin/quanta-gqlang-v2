//! Source-bound Binder declaration and resolution custody.

use super::{
    BindDiagnosticV1, BindResultV1, Binder, BinderDeclarationRequestErrorV3,
    BinderEvidenceReaderV3, BindingV1, CatalogPayloadV1, CheckedHirDocumentOwnedV2, DeclHirV1,
    DeclSymbolV1, FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3,
    HirNodeIdV1, LexicalScopeV1, ModuleGraphViewV1, ProgramItemHirV1, QueryHirV1,
    RelationExprKindV1, ResolutionV1, ResolvedTargetV1, SourceIdV2, SpanV1, ValueExprHirV1,
    ValueExprKindV1, ViewBodyV1,
};
use quanta_gqlang_v2_hir::{BoolExprHirV1, BoolExprKindV1, ProjectionV1, RelationSourceV1};

#[cfg(test)]
#[path = "request_tests.rs"]
mod request_tests;

#[path = "request_owner/composition.rs"]
mod composition;

struct RequestProjectDemandV3 {
    width: usize,
    output_names: usize,
    reference_names: usize,
    local_binding_names: usize,
    references: usize,
    scratch_bytes: usize,
    filtered: bool,
}

/// The request path admits a single direct Filter over a host Source. Its
/// predicate has one alias-qualified scalar field and a scalar literal;
/// richer boolean trees require their own bounded resolution census.
fn request_project_source_v3(
    project: &ProjectionV1,
) -> Result<(&RelationSourceV1, Option<&BoolExprHirV1>), BinderDeclarationRequestErrorV3> {
    match &project.input.kind {
        RelationExprKindV1::Source(source) => Ok((source, None)),
        RelationExprKindV1::Filter(filter) => {
            let RelationExprKindV1::Source(source) = &filter.input.kind else {
                return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
            };
            let Some(alias) = &source.alias else {
                return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
            };
            let BoolExprKindV1::Compare(compare) = &filter.predicate.kind else {
                return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
            };
            let ValueExprKindV1::Path(path) = &compare.lhs.kind else {
                return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
            };
            let [field] = path.segments.as_slice() else {
                return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
            };
            let ValueExprKindV1::Symbol(base) = &path.base.kind else {
                return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
            };
            if field.name.is_empty()
                || base.name != alias.name
                || !matches!(
                    &compare.rhs.kind,
                    ValueExprKindV1::Literal(
                        quanta_gqlang_v2_hir::LiteralV1::Int(_)
                            | quanta_gqlang_v2_hir::LiteralV1::Str(_)
                            | quanta_gqlang_v2_hir::LiteralV1::Bool(_)
                    )
                )
            {
                return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
            }
            Ok((source, Some(&filter.predicate)))
        }
        RelationExprKindV1::Project(_)
        | RelationExprKindV1::Join(_)
        | RelationExprKindV1::GroupBy(_)
        | RelationExprKindV1::OrderBy(_)
        | RelationExprKindV1::Limit(_) => Err(BinderDeclarationRequestErrorV3::UnsupportedShape),
    }
}

struct RequestResolutionDemandV3 {
    declarations: usize,
    bare_relations: usize,
    project_diagnostic_slots: usize,
    references: usize,
    retained_bytes: usize,
}

/// Immutable request result with its original-root declaration and resolution
/// reservations. The raw cloneable Binder result cannot be moved out alone.
pub struct AdmittedBinderResolutionV3 {
    bound: BindResultV1,
    _declarations: Option<FrontendRetainedStoragePermitV3>,
    _resolution: Option<FrontendRetainedStoragePermitV3>,
}

impl BinderEvidenceReaderV3 for AdmittedBinderResolutionV3 {
    fn resolution_of(&self, reference: HirNodeIdV1) -> Option<&ResolutionV1> {
        self.bound
            .resolutions()
            .iter()
            .find(|entry| entry.reference == reference)
    }
    fn target_of(&self, reference: HirNodeIdV1) -> Option<&ResolvedTargetV1> {
        self.bound.target_of(reference)
    }

    fn target_of_use(
        &self,
        reference: HirNodeIdV1,
        span: SpanV1,
        name: &str,
    ) -> Option<&ResolvedTargetV1> {
        self.bound.target_of_use(reference, span, name)
    }

    fn relation_output(&self, relation: HirNodeIdV1) -> Option<&[BindingV1]> {
        self.bound.relation_output(relation)
    }

    fn partition_input(&self, group: HirNodeIdV1) -> Option<&[BindingV1]> {
        self.bound.partition_input(group)
    }

    fn composition_binding(&self, owner: HirNodeIdV1, ordinal: usize) -> Option<&BindingV1> {
        self.bound.composition_binding(owner, ordinal)
    }

    fn has_capture_set(&self, relation_exists: HirNodeIdV1) -> bool {
        self.bound.has_capture_set(relation_exists)
    }

    fn capture_edges(&self, relation_exists: HirNodeIdV1) -> &[(HirNodeIdV1, HirNodeIdV1, usize)] {
        self.bound.capture_edges(relation_exists)
    }
}

impl AdmittedBinderResolutionV3 {
    /// Authenticate every independently retained Binder reservation.
    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), super::FrontendStorageRefusalV3> {
        if self._declarations.is_none() && self._resolution.is_none() {
            return Err(super::FrontendStorageRefusalV3::ForeignRequestRoot);
        }
        for storage in [&self._declarations, &self._resolution]
            .into_iter()
            .flatten()
        {
            storage.validate_owner_v3(root, source_id, source)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn resolved_reference_count_v3(&self) -> usize {
        self.bound.resolutions().len()
    }

    #[must_use]
    pub fn relation_output_count_v3(&self) -> usize {
        self.bound.relation_outputs().len()
    }
}

/// Consuming declaration owner. The raw Binder cannot be recovered while its
/// source-issued declaration table is live.
pub struct AdmittedBinderDeclarationsV3<G: ModuleGraphViewV1> {
    binder: Binder<G>,
    _storage: Option<FrontendRetainedStoragePermitV3>,
}

impl<G: ModuleGraphViewV1> AdmittedBinderDeclarationsV3<G> {
    #[must_use]
    pub fn declared_name_count_v3(&self) -> usize {
        self.binder.declared_name_count_v3()
    }

    pub fn resolve_checked_source_bindings_with_request_owner_v3(
        self,
        checked: &CheckedHirDocumentOwnedV2,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        parent: &FrontendRetainedStoragePermitV3,
    ) -> Result<AdmittedBinderResolutionV3, BinderDeclarationRequestErrorV3> {
        let Self { binder, _storage } = self;
        let declaration_storage = _storage.ok_or(BinderDeclarationRequestErrorV3::Phase)?;
        declaration_storage
            .validate_owner_v3(root, source_id, source)
            .map_err(BinderDeclarationRequestErrorV3::Storage)?;
        let mut resolved = binder.resolve_checked_source_bindings_with_request_owner_v3(
            checked, root, source_id, source, parent,
        )?;
        resolved._declarations = Some(declaration_storage);
        Ok(resolved)
    }
}

fn request_query_item_v3(item: &ProgramItemHirV1) -> Option<&QueryHirV1> {
    match item {
        ProgramItemHirV1::Query(query) => Some(query),
        ProgramItemHirV1::CatalogDecl(decl) => {
            if let CatalogPayloadV1::View(ViewBodyV1::Inline(query)) = &decl.payload {
                Some(query)
            } else {
                None
            }
        }
        ProgramItemHirV1::Decl(_)
        | ProgramItemHirV1::Rule(_)
        | ProgramItemHirV1::GraphQuery(_)
        | ProgramItemHirV1::Legality(_)
        | ProgramItemHirV1::Presentation(_)
        | ProgramItemHirV1::Composition(_)
        | ProgramItemHirV1::Meta(_)
        | ProgramItemHirV1::Command(_) => None,
    }
}

impl<G: ModuleGraphViewV1> Binder<G> {
    fn request_value_resolution_demand_v3(
        &self,
        value: &ValueExprHirV1,
    ) -> Result<(usize, usize), BinderDeclarationRequestErrorV3> {
        fn combine(
            left: (usize, usize),
            right: (usize, usize),
        ) -> Result<(usize, usize), BinderDeclarationRequestErrorV3> {
            Ok((
                left.0
                    .checked_add(right.0)
                    .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?,
                left.1
                    .checked_add(right.1)
                    .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?,
            ))
        }
        match &value.kind {
            ValueExprKindV1::Literal(_) => Ok((0, 0)),
            ValueExprKindV1::Symbol(symbol) => {
                if self
                    .declarations
                    .binary_search_by(|(name, _)| name.as_str().cmp(symbol.name.as_str()))
                    .is_err()
                {
                    return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
                }
                Ok((1, symbol.name.len()))
            }
            ValueExprKindV1::Path(path) => self.request_value_resolution_demand_v3(&path.base),
            ValueExprKindV1::Unary(unary) => {
                self.request_value_resolution_demand_v3(&unary.operand)
            }
            ValueExprKindV1::Binary(binary) => combine(
                self.request_value_resolution_demand_v3(&binary.lhs)?,
                self.request_value_resolution_demand_v3(&binary.rhs)?,
            ),
            ValueExprKindV1::Paren(inner) => self.request_value_resolution_demand_v3(inner),
            ValueExprKindV1::Record(record) => {
                record.fields.iter().try_fold((0, 0), |current, field| {
                    combine(
                        current,
                        self.request_value_resolution_demand_v3(&field.value)?,
                    )
                })
            }
            ValueExprKindV1::List(list) => {
                list.elements.iter().try_fold((0, 0), |current, element| {
                    combine(current, self.request_value_resolution_demand_v3(element)?)
                })
            }
            // Calls push an active-call stack and may invoke imported symbols;
            // those allocations need a separate source/authority census.
            ValueExprKindV1::Call(_) => Err(BinderDeclarationRequestErrorV3::UnsupportedShape),
        }
    }

    /// Resolve value declarations, bare sources, and source-backed projects
    /// through the existing Binder traversal. Inline View queries consume the
    /// same checked source ordinal and request root as top-level queries.
    /// The checked source proves every reference is a source declaration and
    /// counts its two name copies and one result slot before their first birth.
    /// Calls and richer relation/rule families stop before allocation until
    /// their distinct scope, graph, and side-table demands are source indexed.
    fn resolve_checked_source_bindings_with_request_owner_v3(
        mut self,
        checked: &CheckedHirDocumentOwnedV2,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        parent: &FrontendRetainedStoragePermitV3,
    ) -> Result<AdmittedBinderResolutionV3, BinderDeclarationRequestErrorV3> {
        self.resolve_checked_source_bindings_inner_v3(checked, root, source_id, source, parent)?;
        self.finish_checked_request_bindings_v3()
    }

    fn resolve_checked_source_bindings_inner_v3(
        &mut self,
        checked: &CheckedHirDocumentOwnedV2,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        parent: &FrontendRetainedStoragePermitV3,
    ) -> Result<(), BinderDeclarationRequestErrorV3> {
        parent
            .validate_owner_v3(root, source_id, source)
            .map_err(BinderDeclarationRequestErrorV3::Storage)?;
        if !self.request_declaration_complete || self.request_resolution_attempted {
            return Err(BinderDeclarationRequestErrorV3::Phase);
        }
        self.request_resolution_attempted = true;
        let document = checked.document();
        if document.source_id != source_id || document.source_len != source.len() {
            return Err(BinderDeclarationRequestErrorV3::SourceMismatch);
        }
        if !self.scope.is_unborn_v3()
            || !self.result.diagnostics.is_empty()
            || !self.result.resolutions.is_empty()
            || !self.result.relation_outputs.is_empty()
            || !self.result.partition_inputs.is_empty()
            || !self.result.partition_candidates.is_empty()
            || !self.result.capture_edges.is_empty()
            || !self.result.composition_bindings.is_empty()
            || self.result.request_composition_binding.is_some()
            || !self.capture_owners.is_empty()
            || !self.relation_partition.is_empty()
            || self.active_partition.is_some()
            || !self.active_calls.is_empty()
            || !self.forbidden_composition_sources.is_empty()
            || self.request_alias_output.is_some()
            || self.request_project_input.is_some()
            || self.request_project_output.is_some()
            || self.request_project_index.is_some()
            || self.request_resolution_storage.is_some()
        {
            return Err(BinderDeclarationRequestErrorV3::Phase);
        }
        if let [ProgramItemHirV1::Composition(composition)] = document.items.as_slice() {
            return self.resolve_one_let_composition_with_request_owner_v3(
                composition,
                root,
                source_id,
                source,
            );
        }
        let demand = self.request_resolution_demand_v3(checked)?;
        self.request_resolution_storage = (demand.retained_bytes > 0)
            .then(|| root.reserve_v3(source_id, source, demand.retained_bytes))
            .transpose()
            .map_err(BinderDeclarationRequestErrorV3::Storage)?;
        if demand.declarations > 0 || demand.bare_relations > 0 {
            self.scope
                .try_reserve_one_frame_v3()
                .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
            if !self.scope.one_frame_capacity_v3() {
                return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
            }
        }
        self.result
            .resolutions
            .try_reserve_exact(demand.references)
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        self.result
            .diagnostics
            .try_reserve_exact(demand.project_diagnostic_slots)
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        self.result
            .request_relation_outputs
            .try_reserve_exact(demand.bare_relations)
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        if self.result.resolutions.capacity() > demand.references
            || self.result.diagnostics.capacity() > demand.project_diagnostic_slots
            || self.result.request_relation_outputs.capacity() > demand.bare_relations
        {
            return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
        }
        for item in &document.items {
            if let ProgramItemHirV1::Decl(decl) = item {
                self.resolve_decl(decl, &[]);
            }
        }
        for item in &document.items {
            if let Some(query) = request_query_item_v3(item) {
                let source = match &query.body.kind {
                    RelationExprKindV1::Source(source) => source,
                    RelationExprKindV1::Project(project) => {
                        request_project_source_v3(project)
                            .map_err(|_shape_error| BinderDeclarationRequestErrorV3::Phase)?
                            .0
                    }
                    RelationExprKindV1::Filter(_)
                    | RelationExprKindV1::Join(_)
                    | RelationExprKindV1::GroupBy(_)
                    | RelationExprKindV1::OrderBy(_)
                    | RelationExprKindV1::Limit(_) => {
                        return Err(BinderDeclarationRequestErrorV3::Phase)
                    }
                };
                if source.alias.is_some() {
                    let mut output = Vec::new();
                    output.try_reserve_exact(1).map_err(|_allocation| {
                        BinderDeclarationRequestErrorV3::FallibleAllocation
                    })?;
                    if output.capacity() != 1 {
                        return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
                    }
                    self.request_alias_output = Some(output);
                }
                if let RelationExprKindV1::Project(project) = &query.body.kind {
                    let width = project.items().len();
                    let mut input = Vec::new();
                    input.try_reserve_exact(width).map_err(|_allocation| {
                        BinderDeclarationRequestErrorV3::FallibleAllocation
                    })?;
                    let mut output = Vec::new();
                    output.try_reserve_exact(width).map_err(|_allocation| {
                        BinderDeclarationRequestErrorV3::FallibleAllocation
                    })?;
                    if input.capacity() != width || output.capacity() != width {
                        return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
                    }
                    let mut index = Vec::new();
                    index.try_reserve_exact(width).map_err(|_allocation| {
                        BinderDeclarationRequestErrorV3::FallibleAllocation
                    })?;
                    if index.capacity() != width {
                        return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
                    }
                    self.request_project_input = Some(input);
                    self.request_project_output = Some(output);
                    self.request_project_index = Some(index);
                }
                self.resolve_query(query);
                if self.request_alias_output.is_some()
                    || self.request_project_input.is_some()
                    || self.request_project_output.is_some()
                    || self.request_project_index.is_some()
                {
                    return Err(BinderDeclarationRequestErrorV3::Phase);
                }
            }
        }
        if self.result.resolutions.len() != demand.references
            || self.result.request_relation_outputs.len() != demand.bare_relations
            || !self.result.diagnostics.is_empty()
        {
            return Err(BinderDeclarationRequestErrorV3::Phase);
        }
        self.request_resolution_complete = true;
        Ok(())
    }

    fn request_resolution_demand_v3(
        &self,
        checked: &CheckedHirDocumentOwnedV2,
    ) -> Result<RequestResolutionDemandV3, BinderDeclarationRequestErrorV3> {
        let document = checked.document();
        let mut declarations = 0_usize;
        let mut bare_relations = 0_usize;
        let mut aliased_relations = 0_usize;
        let mut alias_name_bytes = 0_usize;
        let mut alias_scratch_bytes = 0_usize;
        let mut project_bindings = 0_usize;
        let mut project_output_names = 0_usize;
        let mut project_reference_names = 0_usize;
        let mut project_local_binding_names = 0_usize;
        let mut project_diagnostic_slots = 0_usize;
        let mut references = 0_usize;
        let mut reference_name_bytes = 0_usize;
        for item in &document.items {
            if let Some(query) = request_query_item_v3(item) {
                if query.result_shape.is_some() {
                    return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
                }
                let project = self.request_project_demand_v3(query)?;
                let source = match &query.body.kind {
                    RelationExprKindV1::Source(source) => source,
                    RelationExprKindV1::Project(project) => request_project_source_v3(project)?.0,
                    RelationExprKindV1::Filter(_)
                    | RelationExprKindV1::Join(_)
                    | RelationExprKindV1::GroupBy(_)
                    | RelationExprKindV1::OrderBy(_)
                    | RelationExprKindV1::Limit(_) => {
                        return Err(BinderDeclarationRequestErrorV3::UnsupportedShape)
                    }
                };
                bare_relations = bare_relations
                    .checked_add(
                        project
                            .as_ref()
                            .map_or(1, |value| if value.filtered { 3 } else { 2 }),
                    )
                    .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                if let Some(alias) = &source.alias {
                    aliased_relations = aliased_relations
                        .checked_add(1)
                        .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    alias_name_bytes = alias_name_bytes
                        .checked_add(alias.name.len())
                        .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    alias_scratch_bytes = alias_scratch_bytes.max(
                        std::mem::size_of::<BindingV1>()
                            .checked_add(alias.name.len())
                            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?,
                    );
                }
                if let Some(project) = project {
                    if project.filtered {
                        // Filter retains the Source alias frame and copies it
                        // once into its predicate scope before Project enters.
                        aliased_relations = aliased_relations
                            .checked_add(2)
                            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                        let extra_names = source
                            .alias
                            .as_ref()
                            .map_or(Some(0), |alias| alias.name.len().checked_mul(2))
                            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                        alias_name_bytes = alias_name_bytes
                            .checked_add(extra_names)
                            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    }
                    project_bindings = project_bindings
                        .checked_add(project.width)
                        .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    project_output_names =
                        project_output_names
                            .checked_add(project.output_names)
                            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    project_reference_names = project_reference_names
                        .checked_add(project.reference_names)
                        .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    project_local_binding_names = project_local_binding_names
                        .checked_add(project.local_binding_names)
                        .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    // Each output may both shadow the source alias and
                    // duplicate an earlier projection name.
                    project_diagnostic_slots = project_diagnostic_slots
                        .checked_add(
                            project
                                .width
                                .checked_mul(2)
                                .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?,
                        )
                        .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    references = references
                        .checked_add(project.references)
                        .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    alias_scratch_bytes = alias_scratch_bytes.max(project.scratch_bytes);
                }
                continue;
            }
            match item {
                ProgramItemHirV1::Decl(decl) => {
                    let Ok(index) = self
                        .declarations
                        .binary_search_by(|(name, _)| name.as_str().cmp(decl.name.as_str()))
                    else {
                        return Err(BinderDeclarationRequestErrorV3::Phase);
                    };
                    let (_, symbol) = self
                        .declarations
                        .get(index)
                        .ok_or(BinderDeclarationRequestErrorV3::Phase)?;
                    if symbol.node != decl.id || symbol.span != decl.name_span {
                        return Err(BinderDeclarationRequestErrorV3::Phase);
                    }
                    declarations = declarations
                        .checked_add(1)
                        .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    let (count, bytes) = self.request_value_resolution_demand_v3(&decl.body)?;
                    references = references
                        .checked_add(count)
                        .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                    reference_name_bytes = reference_name_bytes
                        .checked_add(bytes)
                        .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                }
                ProgramItemHirV1::CatalogDecl(_) => {}
                ProgramItemHirV1::Meta(_)
                | ProgramItemHirV1::GraphQuery(_)
                | ProgramItemHirV1::Presentation(_)
                | ProgramItemHirV1::Legality(_)
                | ProgramItemHirV1::Command(_) => {}
                ProgramItemHirV1::Query(_)
                | ProgramItemHirV1::Rule(_)
                | ProgramItemHirV1::Composition(_) => {
                    return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
                }
            }
        }
        if declarations != self.declarations.len() {
            return Err(BinderDeclarationRequestErrorV3::Phase);
        }
        let frame_bytes = if declarations > 0 || bare_relations > 0 {
            LexicalScopeV1::one_frame_storage_bytes_v3()
        } else {
            0
        };
        let result_bytes = references
            .checked_mul(std::mem::size_of::<ResolutionV1>())
            .and_then(|bytes| {
                reference_name_bytes
                    .checked_mul(2)
                    .and_then(|names| bytes.checked_add(names))
            })
            .and_then(|bytes| bytes.checked_add(project_reference_names))
            .and_then(|bytes| bytes.checked_add(project_local_binding_names))
            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
        let relation_bytes = bare_relations
            .checked_mul(std::mem::size_of::<(HirNodeIdV1, Vec<BindingV1>)>())
            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
        let retained_bytes = frame_bytes
            .checked_add(result_bytes)
            .and_then(|bytes| bytes.checked_add(relation_bytes))
            .and_then(|bytes| {
                aliased_relations
                    .checked_mul(std::mem::size_of::<BindingV1>())
                    .and_then(|bindings| bytes.checked_add(bindings))
            })
            .and_then(|bytes| bytes.checked_add(alias_name_bytes))
            .and_then(|bytes| {
                project_bindings
                    .checked_mul(std::mem::size_of::<BindingV1>())
                    .and_then(|bindings| bytes.checked_add(bindings))
            })
            .and_then(|bytes| bytes.checked_add(project_output_names))
            .and_then(|bytes| {
                project_diagnostic_slots
                    .checked_mul(std::mem::size_of::<BindDiagnosticV1>())
                    .and_then(|diagnostics| bytes.checked_add(diagnostics))
            })
            .and_then(|bytes| bytes.checked_add(alias_scratch_bytes))
            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
        Ok(RequestResolutionDemandV3 {
            declarations,
            bare_relations,
            project_diagnostic_slots,
            references,
            retained_bytes,
        })
    }

    /// Consume the checked request traversal without the raw finish path's
    /// resolution pointer-sort Vec. Every source HIR node is unique and visited
    /// once in this restricted source traversal, so duplicate-use
    /// diagnostics cannot be produced here.
    fn finish_checked_request_bindings_v3(
        mut self,
    ) -> Result<AdmittedBinderResolutionV3, BinderDeclarationRequestErrorV3> {
        if !self.request_resolution_complete || !self.result.diagnostics.is_empty() {
            return Err(BinderDeclarationRequestErrorV3::Phase);
        }
        self.result
            .request_relation_outputs
            .sort_unstable_by_key(|entry| entry.0);
        Ok(AdmittedBinderResolutionV3 {
            bound: std::mem::take(&mut self.result),
            _declarations: None,
            _resolution: self.request_resolution_storage.take(),
        })
    }

    /// Issue the checked source's declaration table under its existing root.
    /// Scratch ordering and retained names/diagnostics are admitted before
    /// their respective first Vec or String birth.
    pub fn declare_checked_program_with_request_owner_v3(
        mut self,
        checked: &CheckedHirDocumentOwnedV2,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        parent: &FrontendRetainedStoragePermitV3,
    ) -> Result<AdmittedBinderDeclarationsV3<G>, BinderDeclarationRequestErrorV3> {
        let storage =
            self.declare_checked_program_inner_v3(checked, root, source_id, source, parent)?;
        Ok(AdmittedBinderDeclarationsV3 {
            binder: self,
            _storage: storage,
        })
    }

    fn declare_checked_program_inner_v3(
        &mut self,
        checked: &CheckedHirDocumentOwnedV2,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        parent: &FrontendRetainedStoragePermitV3,
    ) -> Result<Option<FrontendRetainedStoragePermitV3>, BinderDeclarationRequestErrorV3> {
        parent
            .validate_owner_v3(root, source_id, source)
            .map_err(BinderDeclarationRequestErrorV3::Storage)?;
        if self.request_declaration_attempted
            || !self.declarations.is_empty()
            || !self.result.diagnostics.is_empty()
            || self.request_declaration_storage.is_some()
        {
            return Err(BinderDeclarationRequestErrorV3::Phase);
        }
        self.request_declaration_attempted = true;
        let document = checked.document();
        if document.source_id != source_id || document.source_len != source.len() {
            return Err(BinderDeclarationRequestErrorV3::SourceMismatch);
        }
        let slots = document
            .items
            .iter()
            .filter(|item| matches!(item, ProgramItemHirV1::Decl(_)))
            .count();
        let scratch_bytes = slots
            .checked_mul(std::mem::size_of::<&DeclHirV1>())
            .and_then(|bytes| bytes.checked_mul(2))
            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
        let scratch = (scratch_bytes > 0)
            .then(|| root.reserve_v3(source_id, source, scratch_bytes))
            .transpose()
            .map_err(BinderDeclarationRequestErrorV3::Storage)?;
        let mut ordered = Vec::new();
        ordered
            .try_reserve_exact(slots)
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        if ordered.capacity() > slots {
            return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
        }
        ordered.extend(document.items.iter().filter_map(|item| match item {
            ProgramItemHirV1::Decl(decl) => Some(decl.as_ref()),
            ProgramItemHirV1::Query(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::CatalogDecl(_)
            | ProgramItemHirV1::Composition(_)
            | ProgramItemHirV1::Meta(_)
            | ProgramItemHirV1::Command(_) => None,
        }));
        ordered.sort_unstable_by_key(|decl| {
            (
                decl.name_span.start,
                decl.name_span.end,
                decl.name.as_str(),
                decl.id,
            )
        });
        let mut by_name = Vec::new();
        by_name
            .try_reserve_exact(slots)
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        if by_name.capacity() != slots {
            return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
        }
        by_name.extend(ordered.iter().copied());
        by_name.sort_unstable_by(|left, right| {
            (
                left.name.as_str(),
                left.name_span.start,
                left.name_span.end,
                left.id,
            )
                .cmp(&(
                    right.name.as_str(),
                    right.name_span.start,
                    right.name_span.end,
                    right.id,
                ))
        });
        let mut unique = 0_usize;
        let mut duplicate = 0_usize;
        let mut name_bytes = 0_usize;
        let mut previous_name: Option<&str> = None;
        for decl in &by_name {
            name_bytes = name_bytes
                .checked_add(decl.name.len())
                .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
            if previous_name == Some(decl.name.as_str()) {
                duplicate += 1;
            } else {
                unique += 1;
            }
            previous_name = Some(decl.name.as_str());
        }
        let retained_bytes = unique
            .checked_mul(std::mem::size_of::<(String, DeclSymbolV1)>())
            .and_then(|bytes| {
                duplicate
                    .checked_mul(std::mem::size_of::<BindDiagnosticV1>())
                    .and_then(|diagnostics| bytes.checked_add(diagnostics))
            })
            .and_then(|bytes| bytes.checked_add(name_bytes))
            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
        // Keep a concrete root-issued carrier even when the document has no
        // declarations. Resolution must authenticate this stage independently
        // of the caller's parent permit.
        self.request_declaration_storage = Some(
            root.reserve_v3(source_id, source, retained_bytes)
                .map_err(BinderDeclarationRequestErrorV3::Storage)?,
        );
        self.declarations
            .try_reserve_exact(unique)
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        self.result
            .diagnostics
            .try_reserve_exact(duplicate)
            .map_err(|_allocation| BinderDeclarationRequestErrorV3::FallibleAllocation)?;
        if self.declarations.capacity() > unique || self.result.diagnostics.capacity() > duplicate {
            return Err(BinderDeclarationRequestErrorV3::FallibleAllocation);
        }
        let mut previous_name: Option<&str> = None;
        for decl in &by_name {
            if previous_name != Some(decl.name.as_str()) {
                self.declarations.push((
                    decl.name.clone(),
                    DeclSymbolV1 {
                        node: decl.id,
                        span: decl.name_span,
                    },
                ));
            }
            previous_name = Some(decl.name.as_str());
        }
        for decl in ordered {
            let first = by_name.partition_point(|candidate| candidate.name < decl.name);
            let first_decl = by_name
                .get(first)
                .ok_or(BinderDeclarationRequestErrorV3::Phase)?;
            if first_decl.id != decl.id {
                self.result.diagnostics.push(BindDiagnosticV1::Duplicate {
                    name: decl.name.clone(),
                    first_span: first_decl.name_span,
                    second_span: decl.name_span,
                });
            }
        }
        drop(scratch);
        self.request_declaration_complete = true;
        Ok(self.request_declaration_storage.take())
    }

    fn borrowed_projection_output_name_v3(item: &quanta_gqlang_v2_hir::ProjectionItemV1) -> &str {
        if let Some(alias) = &item.alias {
            return &alias.name;
        }
        match &item.value.kind {
            ValueExprKindV1::Symbol(symbol) => &symbol.name,
            ValueExprKindV1::Path(path) => path
                .segments
                .as_slice()
                .last()
                .map_or("col", |segment| segment.name.as_str()),
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Call(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => "col",
        }
    }

    fn request_project_demand_v3(
        &self,
        query: &QueryHirV1,
    ) -> Result<Option<RequestProjectDemandV3>, BinderDeclarationRequestErrorV3> {
        let RelationExprKindV1::Project(project) = &query.body.kind else {
            return Ok(None);
        };
        let (source, predicate) = request_project_source_v3(project)?;
        let width = project.items().len();
        let mut output_names = 0_usize;
        let mut reference_names = 0_usize;
        let mut local_binding_names = 0_usize;
        let mut references = 0_usize;
        for item in project.items() {
            let output = Self::borrowed_projection_output_name_v3(item);
            output_names = output_names
                .checked_add(output.len())
                .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
            let symbol = match &item.value.kind {
                ValueExprKindV1::Literal(_) => None,
                ValueExprKindV1::Symbol(symbol) => Some(symbol),
                ValueExprKindV1::Path(path) => match &path.base.kind {
                    ValueExprKindV1::Symbol(symbol) => Some(symbol),
                    ValueExprKindV1::Literal(_)
                    | ValueExprKindV1::Path(_)
                    | ValueExprKindV1::Unary(_)
                    | ValueExprKindV1::Binary(_)
                    | ValueExprKindV1::Call(_)
                    | ValueExprKindV1::Paren(_)
                    | ValueExprKindV1::Record(_)
                    | ValueExprKindV1::List(_) => {
                        return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
                    }
                },
                ValueExprKindV1::Unary(_)
                | ValueExprKindV1::Binary(_)
                | ValueExprKindV1::Call(_)
                | ValueExprKindV1::Paren(_)
                | ValueExprKindV1::Record(_)
                | ValueExprKindV1::List(_) => {
                    return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
                }
            };
            if let Some(symbol) = symbol {
                let Some(alias) = &source.alias else {
                    return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
                };
                if symbol.name != alias.name {
                    return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
                }
                references = references
                    .checked_add(1)
                    .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                reference_names = reference_names
                    .checked_add(symbol.name.len())
                    .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
                local_binding_names = local_binding_names
                    .checked_add(alias.name.len())
                    .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
            }
        }
        let binding_slots = width
            .checked_mul(std::mem::size_of::<BindingV1>())
            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
        let duplicate_index = width
            .checked_mul(std::mem::size_of::<(usize, usize, SpanV1)>())
            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
        let alias_frame = source
            .alias
            .as_ref()
            .map_or(Some(0), |alias| {
                std::mem::size_of::<BindingV1>().checked_add(alias.name.len())
            })
            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
        // The source alias frame remains in scope while projection input,
        // output, and borrowed-name index vectors are live. An output name
        // lives in one Binding and may be cloned into one duplicate diagnostic.
        let scratch_bytes = alias_frame
            .checked_add(if predicate.is_some() { alias_frame } else { 0 })
            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?
            .checked_add(
                binding_slots
                    .checked_mul(2)
                    .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?,
            )
            .and_then(|bytes| bytes.checked_add(duplicate_index))
            .and_then(|bytes| {
                output_names
                    .checked_mul(2)
                    .and_then(|names| bytes.checked_add(names))
            })
            .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
        if let Some(predicate) = predicate {
            let BoolExprKindV1::Compare(compare) = &predicate.kind else {
                return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
            };
            let ValueExprKindV1::Path(path) = &compare.lhs.kind else {
                return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
            };
            let ValueExprKindV1::Symbol(base) = &path.base.kind else {
                return Err(BinderDeclarationRequestErrorV3::UnsupportedShape);
            };
            references = references
                .checked_add(1)
                .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
            reference_names = reference_names
                .checked_add(base.name.len())
                .ok_or(BinderDeclarationRequestErrorV3::DemandOverflow)?;
        }
        Ok(Some(RequestProjectDemandV3 {
            width,
            output_names,
            reference_names,
            local_binding_names,
            references,
            scratch_bytes,
            filtered: predicate.is_some(),
        }))
    }
}
