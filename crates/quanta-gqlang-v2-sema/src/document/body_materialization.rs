//! Request-owned scalar inline View semantic body.

use super::{
    AdmittedSourceBodyTypesV3, ExpressionEffectsV2, FrontendDiagnosticV2, FrontendFailureV2,
    FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3,
    FrontendStorageRefusalV3, OutputColumnContractV2, ProgramItemHirV1, SemanticStepCounterV2,
    SourceCatalogAssetsV3, SourceIdV2, SourceQualifiedNodeIdV2, TypeFactsReaderV3, TypedExprV2,
    TypedPathSegmentV2, TypedRelationExprV2, TypedRelationV2, ValueExprKindV1,
};
use quanta_gqlang_v2_modules::{
    AdmittedDocumentViewTopologyProofV3, DocumentCatalogSessionWitnessV3,
    DocumentTopologyRequestErrorV3,
};

struct ScalarBodyBuffersV3 {
    expressions: Vec<(SourceQualifiedNodeIdV2, TypedExprV2)>,
    source_columns: Vec<OutputColumnContractV2>,
    columns: Vec<OutputColumnContractV2>,
    keys: Vec<SourceQualifiedNodeIdV2>,
    effects: Vec<ExpressionEffectsV2>,
}

fn materialize_scalar_projection_v3(
    query: &quanta_gqlang_v2_hir::QueryHirV1,
    project: &quanta_gqlang_v2_hir::ProjectionV1,
    scan: &quanta_gqlang_v2_hir::RelationSourceV1,
    typed: &AdmittedSourceBodyTypesV3<'_>,
    buffers: &mut ScalarBodyBuffersV3,
) -> Result<TypedRelationV2, FrontendFailureV2> {
    let source_id = typed.source_id_v3();
    let authority = typed._authority;
    let extended = &typed._extended;
    let bound = typed._bound.evidence_v3();
    let facts = &typed.facts;
    let mut steps = SemanticStepCounterV2::new(authority.budget());
    let checked = |span| {
        extended
            .combined()
            .try_check_location(super::location(source_id, span))
            .map_err(|error| {
                super::document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    span,
                    error.to_string(),
                )
            })
    };
    // The ordinary relation walk charges Project before visiting its Source.
    steps.charge(source_id, query.span)?;
    steps.charge(source_id, query.body.span)?;
    steps.charge(source_id, project.input.span)?;
    if bound.relation_output(query.body.id).is_none()
        || bound.relation_output(project.input.id).is_none()
    {
        return Err(super::document_failure(
            "sema.document_validation_failed",
            source_id,
            query.body.span,
            "binder omitted relation output frame",
        ));
    }
    let source_key = SourceQualifiedNodeIdV2 {
        source_id,
        node_id: project.input.id,
    };
    let input = super::relation_materialization::materialize_host_source_v3(
        &project.input,
        checked(project.input.span)?,
        source_key,
        scan,
        authority,
        source_id,
        true,
        std::mem::take(&mut buffers.source_columns),
    )?;
    let frame = bound.relation_output(query.body.id).ok_or_else(|| {
        super::document_failure(
            "sema.document_validation_failed",
            source_id,
            query.body.span,
            "binder omitted projection output frame",
        )
    })?;
    if frame.len() != project.items().len() {
        return Err(super::document_failure(
            "sema.document_validation_failed",
            source_id,
            query.body.span,
            "binder projection output differs from authored item count",
        ));
    }
    for (ordinal, (item, binding)) in project.items().iter().zip(frame).enumerate() {
        if binding.identity() != (query.body.id, ordinal) {
            return Err(super::document_failure(
                "sema.document_validation_failed",
                source_id,
                item.value.span,
                "projection output does not retain its Binder-owned ordinal",
            ));
        }
        let value = super::materialize_declaration_expr(
            &item.value,
            source_id,
            bound,
            facts,
            extended,
            &mut buffers.expressions,
            &mut steps,
            super::AggregatePositionV2::Projection,
        )?;
        if !matches!(value.type_state(), super::ExpressionTypeStateV2::Known(_)) {
            return Err(super::document_failure(
                "sema.unknown_type_in_closed_compile",
                source_id,
                item.value.span,
                "projection field has no concrete value type",
            ));
        }
        let name_span = item
            .alias
            .as_ref()
            .map_or(item.value.span, |alias| alias.span);
        buffers.columns.push(OutputColumnContractV2::new(
            binding.name.clone(),
            super::OutputColumnOriginV2::Expression(value.key()),
            checked(name_span)?,
        ));
        buffers.keys.push(value.key());
        buffers.effects.push(value.effects());
    }
    let project_key = SourceQualifiedNodeIdV2 {
        source_id,
        node_id: query.body.id,
    };
    let row = facts.relation_row(project_key).cloned().ok_or_else(|| {
        super::document_failure(
            "sema.document_validation_failed",
            source_id,
            query.body.span,
            "types owner omitted projection row fact",
        )
    })?;
    let root = super::relation_materialization::seal_scalar_project_v3(
        &query.body,
        checked(query.body.span)?,
        project_key,
        super::relation_materialization::ScalarProjectPartsV3 {
            input,
            row,
            columns: std::mem::take(&mut buffers.columns),
            keys: std::mem::take(&mut buffers.keys),
            effects: std::mem::take(&mut buffers.effects),
        },
        source_id,
    )?;
    let output = root.output().clone();
    TypedRelationV2::try_new(
        super::SemanticItemIdV2::new(source_id, query.id),
        checked(query.span)?,
        project_key,
        root,
        output,
    )
    .map_err(|error| {
        super::document_failure(
            "sema.document_validation_failed",
            source_id,
            query.span,
            error,
        )
    })
}

/// A topology proof tied to the same source root as the sealed scalar facts.
pub struct AdmittedSourceViewTopologyV3 {
    pub(super) proof: AdmittedDocumentViewTopologyProofV3,
    pub(super) storage: FrontendRetainedStoragePermitV3,
}

impl AdmittedSourceViewTopologyV3 {
    #[must_use]
    pub fn dependency_first_view_count_v3(&self) -> usize {
        self.proof.dependency_first_order().len()
    }
}

pub fn topologize_source_inline_view_with_request_owner_v3(
    session: &mut super::ResolvingDocumentSessionV3<'_>,
    typed: &AdmittedSourceBodyTypesV3<'_>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    parent: &FrontendRetainedStoragePermitV3,
) -> Result<AdmittedSourceViewTopologyV3, DocumentTopologyRequestErrorV3> {
    let source_id = typed.source_id_v3();
    parent
        .validate_owner_v3(root, source_id, source)
        .map_err(DocumentTopologyRequestErrorV3::Storage)?;
    typed
        .facts
        .validate_owner_v3(root, source_id, source)
        .map_err(DocumentTopologyRequestErrorV3::Storage)?;
    if !std::ptr::eq(typed._authority, session.authority) {
        return Err(DocumentTopologyRequestErrorV3::Phase);
    }
    let storage = root
        .reserve_v3(source_id, source, 0)
        .map_err(DocumentTopologyRequestErrorV3::Storage)?;
    let proof = session
        .validator
        .validate_view_topology_with_request_owner_v3(
            Vec::new(),
            root,
            source_id,
            source,
            parent,
        )?;
    Ok(AdmittedSourceViewTopologyV3 { proof, storage })
}

fn request_body_push_vec_peak_bytes_v3<T>(items: usize) -> Option<usize> {
    let width = std::mem::size_of::<T>();
    if items == 0 || width == 0 {
        return Some(0);
    }
    let minimum = if width == 1 {
        8
    } else if width <= 1024 {
        4
    } else {
        1
    };
    let mut capacity = 0_usize;
    let mut peak = 0_usize;
    while capacity < items {
        let next = capacity
            .checked_mul(2)?
            .max(capacity.checked_add(1)?)
            .max(minimum);
        peak = peak.max(capacity.checked_add(next)?);
        capacity = next;
    }
    peak.checked_mul(width)
}

fn request_exact_vec_upper_bytes_v3<T>(items: usize) -> Option<usize> {
    if items == 0 || std::mem::size_of::<T>() == 0 {
        return Some(0);
    }
    items
        .checked_next_power_of_two()?
        .checked_mul(std::mem::size_of::<T>())
}

fn reserve_scalar_body_vec_v3<T>(
    items: usize,
) -> Result<Vec<T>, SourceBodyMaterializationRequestErrorV3> {
    let billed = request_exact_vec_upper_bytes_v3::<T>(items)
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(items)
        .map_err(|_error| SourceBodyMaterializationRequestErrorV3::FallibleAllocation)?;
    let actual = values
        .capacity()
        .checked_mul(std::mem::size_of::<T>())
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    if actual > billed {
        return Err(SourceBodyMaterializationRequestErrorV3::FallibleAllocation);
    }
    Ok(values)
}

fn scalar_projection_body_storage_upper_v3(
    schema_fields: usize,
    schema_name_bytes: usize,
    projection_fields: usize,
    projection_name_bytes: usize,
    path_name_bytes: usize,
    alias_bytes: usize,
    host_id_bytes: usize,
    source_bytes: usize,
) -> Option<usize> {
    let value_nodes = projection_fields.checked_mul(2)?;
    let vectors = [
        request_exact_vec_upper_bytes_v3::<(SourceQualifiedNodeIdV2, TypedExprV2)>(value_nodes)?,
        request_exact_vec_upper_bytes_v3::<OutputColumnContractV2>(schema_fields)?,
        request_exact_vec_upper_bytes_v3::<OutputColumnContractV2>(projection_fields)?,
        request_exact_vec_upper_bytes_v3::<OutputColumnContractV2>(projection_fields)?,
        request_exact_vec_upper_bytes_v3::<SourceQualifiedNodeIdV2>(projection_fields)?,
        request_exact_vec_upper_bytes_v3::<ExpressionEffectsV2>(projection_fields)?,
        std::mem::size_of::<TypedRelationExprV2>(), // `vec![input]`
        // Each Path push has a four-slot first buffer and `typed.clone()`
        // makes a one-slot copy retained by the expression table.
        projection_fields.checked_mul(
            request_body_push_vec_peak_bytes_v3::<TypedPathSegmentV2>(1)?
                .checked_add(std::mem::size_of::<TypedPathSegmentV2>())?,
        )?,
        std::mem::size_of::<FrontendDiagnosticV2>(), // one failure Vec
    ]
    .into_iter()
    .try_fold(0_usize, usize::checked_add)?;
    let strings = host_id_bytes
        .checked_mul(schema_fields.checked_add(4)?)?
        .checked_add(schema_name_bytes.checked_mul(2)?)?
        .checked_add(projection_name_bytes.checked_mul(2)?)?
        .checked_add(alias_bytes.checked_mul(projection_fields.checked_mul(2)?)?)?
        .checked_add(path_name_bytes.checked_mul(2)?)?
        // The actual diagnostic constructor truncates at this published bound.
        // Source/host names cover the sole temporary registry error rendering.
        .checked_add(crate::diagnostic::MAX_MESSAGE_BYTES)?
        .checked_add(source_bytes)?
        .checked_add(host_id_bytes)?;
    [vectors, strings]
        .into_iter()
        .try_fold(0_usize, usize::checked_add)
}

/// One materialized scalar Source -> Project item and every producer it depends on.
pub struct AdmittedSourceInlineViewBodyV3<'authority> {
    pub(super) _types: AdmittedSourceBodyTypesV3<'authority>,
    pub(super) relation: TypedRelationV2,
    pub(super) expressions: Vec<(SourceQualifiedNodeIdV2, TypedExprV2)>,
    pub(super) catalog_session: DocumentCatalogSessionWitnessV3,
    pub(super) _storage: FrontendRetainedStoragePermitV3,
}

impl AdmittedSourceInlineViewBodyV3<'_> {
    #[must_use]
    pub fn source_id_v3(&self) -> SourceIdV2 {
        self._types.source_id_v3()
    }

    #[must_use]
    pub fn output_field_count_v3(&self) -> usize {
        self.relation.output().columns().len()
    }

    #[must_use]
    pub fn expression_count_v3(&self) -> usize {
        self.expressions.len()
    }

    #[must_use]
    pub fn output_row_v3(&self) -> &quanta_gqlang_v2_types::RecordTypeV1 {
        self.relation.output().row()
    }

    #[must_use]
    pub fn output_columns_v3(&self) -> &[OutputColumnContractV2] {
        self.relation.output().columns()
    }

    pub fn expression_keys_v3(
        &self,
    ) -> impl ExactSizeIterator<Item = SourceQualifiedNodeIdV2> + '_ {
        self.expressions.iter().map(|(key, _value)| *key)
    }

    pub fn output_field_names_v3(&self) -> impl ExactSizeIterator<Item = &str> {
        self.relation
            .output()
            .columns()
            .iter()
            .map(|column| column.name())
    }
}

pub enum SourceBodyMaterializationRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    DemandOverflow,
    UnsupportedShape,
    SemanticStepLimit,
    FallibleAllocation,
    Diagnostic(AdmittedSourceBodyMaterializationDiagnosticV3),
}

pub struct AdmittedSourceBodyMaterializationDiagnosticV3 {
    failure: FrontendFailureV2,
    _storage: FrontendRetainedStoragePermitV3,
}

impl std::fmt::Debug for SourceBodyMaterializationRequestErrorV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(reason) => formatter.debug_tuple("Storage").field(reason).finish(),
            Self::DemandOverflow => formatter.write_str("DemandOverflow"),
            Self::UnsupportedShape => formatter.write_str("UnsupportedShape"),
            Self::SemanticStepLimit => formatter.write_str("SemanticStepLimit"),
            Self::FallibleAllocation => formatter.write_str("FallibleAllocation"),
            Self::Diagnostic(value) => formatter
                .debug_tuple("Diagnostic")
                .field(
                    &value
                        .failure
                        .diagnostics()
                        .first()
                        .map(FrontendDiagnosticV2::code),
                )
                .finish(),
        }
    }
}

/// Continue the same sealed Type/Binder source into the existing Sema body
/// producer. A View needs its exact local asset; a bare Query needs an empty
/// catalog and topology from the same Modules session.
pub fn materialize_source_inline_view_with_request_owner_v3<'authority>(
    typed: AdmittedSourceBodyTypesV3<'authority>,
    assets: &SourceCatalogAssetsV3,
    topology: &AdmittedSourceViewTopologyV3,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    parent: &FrontendRetainedStoragePermitV3,
) -> Result<AdmittedSourceInlineViewBodyV3<'authority>, SourceBodyMaterializationRequestErrorV3> {
    let authority = typed._authority;
    let source_id = typed.source_id_v3();
    parent
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceBodyMaterializationRequestErrorV3::Storage)?;
    typed
        .facts
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceBodyMaterializationRequestErrorV3::Storage)?;
    topology
        .storage
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceBodyMaterializationRequestErrorV3::Storage)?;
    let syntax = typed._declared.checked().checked().document();
    let query = match syntax.items.as_slice() {
        [ProgramItemHirV1::Query(query)]
            if assets.is_empty() && topology.proof.dependency_first_order().len() == 0 =>
        {
            query.as_ref()
        }
        [ProgramItemHirV1::CatalogDecl(view)] => {
            let Some(asset) = assets.get(&view.id) else {
                return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape);
            };
            if assets.len() != 1
                || topology.proof.dependency_first_order().next() != Some(asset.id())
                || topology.proof.dependency_first_order().len() != 1
            {
                return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape);
            }
            let quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                quanta_gqlang_v2_hir::ViewBodyV1::Inline(query),
            ) = &view.payload
            else {
                return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape);
            };
            query
        }
        _ => return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape),
    };
    let quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) = &query.body.kind else {
        return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape);
    };
    let quanta_gqlang_v2_hir::RelationExprKindV1::Source(scan) = &project.input.kind else {
        return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape);
    };
    let Some(host) = authority.relations().get(&scan.name) else {
        return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape);
    };
    let Some(alias) = &scan.alias else {
        return Err(SourceBodyMaterializationRequestErrorV3::UnsupportedShape);
    };
    let steps = project
        .items()
        .len()
        .checked_mul(2)
        .and_then(|n| n.checked_add(3))
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let max_steps = usize::try_from(authority.budget().max_semantic_steps())
        .map_err(|_error| SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    if steps > max_steps {
        return Err(SourceBodyMaterializationRequestErrorV3::SemanticStepLimit);
    }
    let schema = host.relation_type().row().fields();
    let schema_names = schema
        .iter()
        .try_fold(0_usize, |n, field| n.checked_add(field.name().len()))
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let projection_names = project
        .items()
        .iter()
        .try_fold(0_usize, |n, item| {
            let name = item.alias.as_ref().map_or_else(
                || match &item.value.kind {
                    ValueExprKindV1::Path(path) => path
                        .segments
                        .as_slice()
                        .first()
                        .map_or("", |field| field.name.as_str()),
                    ValueExprKindV1::Literal(_)
                    | ValueExprKindV1::Symbol(_)
                    | ValueExprKindV1::Unary(_)
                    | ValueExprKindV1::Binary(_)
                    | ValueExprKindV1::Call(_)
                    | ValueExprKindV1::Paren(_)
                    | ValueExprKindV1::Record(_)
                    | ValueExprKindV1::List(_) => "",
                },
                |alias| alias.name.as_str(),
            );
            n.checked_add(name.len())
        })
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let path_names = project
        .items()
        .iter()
        .try_fold(0_usize, |n, item| {
            let ValueExprKindV1::Path(path) = &item.value.kind else {
                return None;
            };
            let [field] = path.segments.as_slice() else {
                return None;
            };
            n.checked_add(field.name.len())
        })
        .ok_or(SourceBodyMaterializationRequestErrorV3::UnsupportedShape)?;
    let host_id = host
        .id()
        .namespace()
        .as_str()
        .len()
        .checked_add(host.id().qualified_name().len())
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let bytes = scalar_projection_body_storage_upper_v3(
        schema.len(),
        schema_names,
        project.items().len(),
        projection_names,
        path_names,
        alias.name.len(),
        host_id,
        source.len(),
    )
    .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let storage = root
        .reserve_v3(source_id, source, bytes)
        .map_err(SourceBodyMaterializationRequestErrorV3::Storage)?;
    let item_count = project.items().len();
    let expression_count = item_count
        .checked_mul(2)
        .ok_or(SourceBodyMaterializationRequestErrorV3::DemandOverflow)?;
    let expressions = reserve_scalar_body_vec_v3(expression_count)?;
    let source_columns = reserve_scalar_body_vec_v3(schema.len())?;
    let columns = reserve_scalar_body_vec_v3(item_count)?;
    let keys = reserve_scalar_body_vec_v3(item_count)?;
    let effects = reserve_scalar_body_vec_v3(item_count)?;
    let mut buffers = ScalarBodyBuffersV3 {
        expressions,
        source_columns,
        columns,
        keys,
        effects,
    };
    let relation =
        match materialize_scalar_projection_v3(query, project, scan, &typed, &mut buffers) {
            Ok(value) => value,
            Err(failure) => {
                return Err(SourceBodyMaterializationRequestErrorV3::Diagnostic(
                    AdmittedSourceBodyMaterializationDiagnosticV3 {
                        failure,
                        _storage: storage,
                    },
                ));
            }
        };
    buffers
        .expressions
        .sort_unstable_by_key(|(key, _value)| *key);
    Ok(AdmittedSourceInlineViewBodyV3 {
        _types: typed,
        relation,
        expressions: buffers.expressions,
        catalog_session: topology.proof.retain_session_v3(),
        _storage: storage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admitted_scalar_vec_capacity_fits_prebirth_limit() {
        fn check<T>(items: usize) {
            let mut storage: Vec<T> = Vec::new();
            assert!(storage.try_reserve_exact(items).is_ok());
            let actual = storage.capacity() * std::mem::size_of::<T>();
            assert!(
                request_exact_vec_upper_bytes_v3::<T>(items).is_some_and(|bound| bound >= actual)
            );
        }
        for items in [1, 9] {
            check::<(SourceQualifiedNodeIdV2, TypedExprV2)>(items * 2);
            check::<OutputColumnContractV2>(items);
            check::<SourceQualifiedNodeIdV2>(items);
            check::<ExpressionEffectsV2>(items);
        }
    }

    #[test]
    fn admitted_scalar_name_copies_fit_source_byte_lengths() {
        // These are the names and authored source lengths used by the one and
        // nine column request fixtures. The storage bound counts each name
        // copy by its source byte length, so allocator capacity must not grow.
        for name in [
            "r",
            "id",
            "id0",
            "id1",
            "id2",
            "id3",
            "id4",
            "id5",
            "id6",
            "id7",
            "id8",
            "host",
            "records",
            "view active | select=from records r | project r.id as id",
        ] {
            let owned = name.to_owned();
            let cloned = owned.clone();
            assert!(owned.capacity() <= name.len());
            assert!(cloned.capacity() <= name.len());
        }
    }
}
