//! Request-owned scalar inline View type facts.

use super::{
    AdmittedSourceBinderResolutionV3, BinderEvidenceReaderV3, BinderKindV1,
    ClosedFrontendAuthorityV2, DeclaredDocumentV2, ExtendedSourceRegistryViewV2,
    FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3,
    FrontendStorageRefusalV3, ProgramItemHirV1, ResolvedTargetV1, ResolvingDocumentSessionV3,
    SourceIdV2, SourceQualifiedNodeIdV2, TypeDiagnosticV2, TypeFactsReaderV3, ValueExprKindV1,
};

#[path = "body_types/composition.rs"]
mod composition;
pub use composition::type_one_let_composition_body_with_request_owner_v3;
pub(in crate::document) use composition::{
    output_name as one_let_output_name_v3, query_parts as one_let_query_parts_v3,
};

/// Types facts for one source-backed inline View. The Binder result, builtin
/// inventory, and Types reservation cannot be detached from the facts.
pub struct AdmittedSourceBodyTypesV3<'authority> {
    pub(super) _authority: &'authority ClosedFrontendAuthorityV2,
    pub(super) _declared: DeclaredDocumentV2,
    pub(super) _extended: ExtendedSourceRegistryViewV2,
    pub(super) _bound: AdmittedSourceBinderResolutionV3<'authority>,
    pub(super) facts: quanta_gqlang_v2_types::AdmittedScalarProjectionFactsV3,
}

impl AdmittedSourceBodyTypesV3<'_> {
    #[must_use]
    pub fn is_one_let_composition_v3(&self) -> bool {
        self._declared.is_one_let_composition_v3()
    }

    #[must_use]
    pub fn source_id_v3(&self) -> SourceIdV2 {
        self._declared.checked().source_id()
    }

    #[must_use]
    pub fn typed_fact_count_v3(&self) -> usize {
        self.facts.len()
    }

    #[must_use]
    pub fn relation_row_count_v3(&self) -> usize {
        self.facts.relation_row_count()
    }
}

pub enum SourceBodyTypesRequestErrorV3 {
    Storage(FrontendStorageRefusalV3),
    DemandOverflow,
    FallibleAllocation,
    SourceMismatch,
    UnsupportedShape,
    Diagnostic(AdmittedSourceBodyTypeDiagnosticV3),
}

pub struct AdmittedSourceBodyTypeDiagnosticV3 {
    diagnostic: Vec<TypeDiagnosticV2>,
    _storage: FrontendRetainedStoragePermitV3,
}

impl std::fmt::Debug for SourceBodyTypesRequestErrorV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Storage(reason) => formatter.debug_tuple("Storage").field(reason).finish(),
            Self::DemandOverflow => formatter.write_str("DemandOverflow"),
            Self::FallibleAllocation => formatter.write_str("FallibleAllocation"),
            Self::SourceMismatch => formatter.write_str("SourceMismatch"),
            Self::UnsupportedShape => formatter.write_str("UnsupportedShape"),
            Self::Diagnostic(value) => formatter.debug_tuple("Diagnostic").field(value).finish(),
        }
    }
}

impl std::fmt::Debug for AdmittedSourceBodyTypeDiagnosticV3 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_tuple("Types")
            .field(&self.diagnostic)
            .finish()
    }
}

fn scalar_source_project_shape_v3<'a>(
    declared: &'a DeclaredDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    session: &ResolvingDocumentSessionV3<'_>,
    reader: &dyn BinderEvidenceReaderV3,
    source: &str,
) -> Result<(&'a quanta_gqlang_v2_hir::QueryHirV1, usize), SourceBodyTypesRequestErrorV3> {
    let syntax = declared.checked().checked().document();
    let query = match syntax.items.as_slice() {
        [ProgramItemHirV1::Query(query)] => query.as_ref(),
        [ProgramItemHirV1::CatalogDecl(view)] => {
            let quanta_gqlang_v2_hir::CatalogPayloadV1::View(
                quanta_gqlang_v2_hir::ViewBodyV1::Inline(query),
            ) = &view.payload
            else {
                return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
            };
            query
        }
        _ => return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape),
    };
    if query.result_shape.is_some() {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    let quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) = &query.body.kind else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let (scan_root, filter) = match &project.input.kind {
        quanta_gqlang_v2_hir::RelationExprKindV1::Source(_) => (&*project.input, None),
        quanta_gqlang_v2_hir::RelationExprKindV1::Filter(filter) => (&*filter.input, Some(filter)),
        quanta_gqlang_v2_hir::RelationExprKindV1::Project(_)
        | quanta_gqlang_v2_hir::RelationExprKindV1::Join(_)
        | quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(_)
        | quanta_gqlang_v2_hir::RelationExprKindV1::OrderBy(_)
        | quanta_gqlang_v2_hir::RelationExprKindV1::Limit(_) => {
            return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape)
        }
    };
    let quanta_gqlang_v2_hir::RelationExprKindV1::Source(scan) = &scan_root.kind else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let Some(alias) = &scan.alias else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    if session
        .validator
        .has_current_catalog_spelling_v3(&scan.name)
    {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    let Some(host) = authority.relations().get(&scan.name) else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let schema = host.relation_type().row().fields();
    if schema.is_empty()
        || schema.iter().any(|field| {
            let value = field.value_type();
            value.as_type().as_scalar().is_none()
                && !value.as_optional().is_some_and(|optional| {
                    optional.inner() == &quanta_gqlang_v2_types::ValueTypeV1::string()
                })
                && !value.as_collection().is_some_and(|collection| {
                    collection.cardinality() == quanta_gqlang_v2_types::CardinalityV1::Sequence
                        && collection.element() == &quanta_gqlang_v2_types::ValueTypeV1::string()
                })
        })
    {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    let Some([source_binding]) = reader.relation_output(scan_root.id) else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    if source_binding.identity() != (scan_root.id, 0)
        || source_binding.kind != BinderKindV1::Alias
        || source_binding.name != alias.name
    {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    if filter.is_some_and(|_| {
        !matches!(reader.relation_output(project.input.id), Some([binding])
            if binding.identity() == (scan_root.id, 0)
                && binding.kind == BinderKindV1::Alias
                && binding.name == alias.name)
    }) {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    let items = project.items();
    if items.is_empty() {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    let Some(frame) = reader.relation_output(query.body.id) else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    if frame.len() != items.len() {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    let mut projection_name_bytes = 0_usize;
    for (ordinal, (item, binding)) in items.iter().zip(frame).enumerate() {
        let ValueExprKindV1::Path(path) = &item.value.kind else {
            return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
        };
        let [field] = path.segments.as_slice() else {
            return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
        };
        let ValueExprKindV1::Symbol(base) = &path.base.kind else {
            return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
        };
        let Some(ResolvedTargetV1::Local(target)) = reader.target_of(path.base.id) else {
            return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
        };
        if base.name != alias.name
            || target.identity() != (scan_root.id, 0)
            || target.kind != BinderKindV1::Alias
            || schema
                .iter()
                .all(|candidate| candidate.name() != field.name)
            || binding.identity() != (query.body.id, ordinal)
            || binding.kind != BinderKindV1::Alias
            || binding.name
                != item
                    .alias
                    .as_ref()
                    .map_or(field.name.as_str(), |alias| &alias.name)
        {
            return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
        }
        projection_name_bytes = projection_name_bytes
            .checked_add(binding.name.len())
            .ok_or(SourceBodyTypesRequestErrorV3::DemandOverflow)?;
    }
    let schema_name_bytes = schema
        .iter()
        .try_fold(0_usize, |total, field| {
            total.checked_add(field.name().len())
        })
        .ok_or(SourceBodyTypesRequestErrorV3::DemandOverflow)?;
    let base_bytes = quanta_gqlang_v2_types::scalar_projection_type_storage_upper_v3(
        source.len(),
        schema.len(),
        schema_name_bytes,
        items.len(),
        projection_name_bytes,
    )
    .ok_or(SourceBodyTypesRequestErrorV3::DemandOverflow)?;
    let bytes = if let Some(filter) = filter {
        let quanta_gqlang_v2_hir::BoolExprKindV1::Compare(compare) = &filter.predicate.kind else {
            return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
        };
        let literal_bytes = match &compare.rhs.kind {
            ValueExprKindV1::Literal(quanta_gqlang_v2_hir::LiteralV1::Str(value)) => value.len(),
            ValueExprKindV1::Literal(
                quanta_gqlang_v2_hir::LiteralV1::Int(_) | quanta_gqlang_v2_hir::LiteralV1::Bool(_),
            ) => 0,
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Symbol(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Call(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => {
                return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape)
            }
        };
        quanta_gqlang_v2_types::scalar_filter_type_storage_upper_v3(
            base_bytes,
            items.len(),
            literal_bytes,
        )
        .ok_or(SourceBodyTypesRequestErrorV3::DemandOverflow)?
    } else {
        base_bytes
    };
    Ok((query, bytes))
}

/// Produce actual Types facts from the sealed Binder owner. This first source
/// family is deliberately finite: one bare Query or inline View over an authored
/// host Source -> optional single Filter -> Project of scalar one-segment paths. Other families stop before
/// the first Types allocation and cannot become a generic compile success.
pub fn type_source_inline_view_with_request_owner_v3<'authority>(
    declared: DeclaredDocumentV2,
    extended: &ExtendedSourceRegistryViewV2,
    session: &ResolvingDocumentSessionV3<'authority>,
    builtins: quanta_gqlang_v2_types::AdmittedBuiltinRegistryV3,
    bound: AdmittedSourceBinderResolutionV3<'authority>,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source: &str,
    parent: &FrontendRetainedStoragePermitV3,
) -> Result<AdmittedSourceBodyTypesV3<'authority>, SourceBodyTypesRequestErrorV3> {
    let authority = bound.authority;
    let source_id = declared.checked().source_id();
    parent
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceBodyTypesRequestErrorV3::Storage)?;
    session
        .validator
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceBodyTypesRequestErrorV3::Storage)?;
    bound
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceBodyTypesRequestErrorV3::Storage)?;
    builtins
        .validate_owner_v3(root, source_id, source)
        .map_err(SourceBodyTypesRequestErrorV3::Storage)?;
    if declared.checked().source_len() != source.len() {
        return Err(SourceBodyTypesRequestErrorV3::SourceMismatch);
    }
    if !std::ptr::eq(authority, session.authority) {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    let reader = bound.evidence_v3();
    let (query, bytes) =
        scalar_source_project_shape_v3(&declared, authority, session, reader, source)?;
    let storage = root
        .reserve_v3(source_id, source, bytes)
        .map_err(SourceBodyTypesRequestErrorV3::Storage)?;

    let quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) = &query.body.kind else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let scan_root = match &project.input.kind {
        quanta_gqlang_v2_hir::RelationExprKindV1::Source(_) => &*project.input,
        quanta_gqlang_v2_hir::RelationExprKindV1::Filter(filter) => &*filter.input,
        quanta_gqlang_v2_hir::RelationExprKindV1::Project(_)
        | quanta_gqlang_v2_hir::RelationExprKindV1::Join(_)
        | quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(_)
        | quanta_gqlang_v2_hir::RelationExprKindV1::OrderBy(_)
        | quanta_gqlang_v2_hir::RelationExprKindV1::Limit(_) => {
            return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape)
        }
    };
    let quanta_gqlang_v2_hir::RelationExprKindV1::Source(scan) = &scan_root.kind else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let Some(host) = authority.relations().get(&scan.name) else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let facts = match quanta_gqlang_v2_types::check_scalar_projection_with_request_owner_v3(
        &declared.checked().checked().as_checked(),
        extended,
        host.relation_type().row(),
        builtins,
        storage,
        root,
        source_id,
        source,
    ) {
        Ok(value) => value,
        Err(quanta_gqlang_v2_types::ScalarProjectionCheckRequestErrorV3::UnsupportedShape) => {
            return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
        }
        Err(quanta_gqlang_v2_types::ScalarProjectionCheckRequestErrorV3::Storage(error)) => {
            return Err(SourceBodyTypesRequestErrorV3::Storage(error));
        }
        Err(quanta_gqlang_v2_types::ScalarProjectionCheckRequestErrorV3::DemandOverflow) => {
            return Err(SourceBodyTypesRequestErrorV3::DemandOverflow);
        }
        Err(quanta_gqlang_v2_types::ScalarProjectionCheckRequestErrorV3::FallibleAllocation) => {
            return Err(SourceBodyTypesRequestErrorV3::FallibleAllocation);
        }
        Err(quanta_gqlang_v2_types::ScalarProjectionCheckRequestErrorV3::Diagnostic {
            diagnostics,
            _storage,
        }) => {
            return Err(SourceBodyTypesRequestErrorV3::Diagnostic(
                AdmittedSourceBodyTypeDiagnosticV3 {
                    diagnostic: diagnostics,
                    _storage,
                },
            ));
        }
    };
    if facts
        .relation_row(SourceQualifiedNodeIdV2 {
            source_id,
            node_id: query.body.id,
        })
        .is_none()
    {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    Ok(AdmittedSourceBodyTypesV3 {
        _authority: authority,
        _declared: declared,
        _extended: extended.clone(),
        _bound: bound,
        facts,
    })
}
