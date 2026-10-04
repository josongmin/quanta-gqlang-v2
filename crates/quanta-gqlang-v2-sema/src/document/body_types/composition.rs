//! Exact one-LET Type continuation; semantic materialization remains separate.

use super::{
    AdmittedSourceBinderResolutionV3, AdmittedSourceBodyTypeDiagnosticV3,
    AdmittedSourceBodyTypesV3, BinderEvidenceReaderV3, BinderKindV1, DeclaredDocumentV2,
    ExtendedSourceRegistryViewV2, FrontendRequestMaterializationRootV3,
    FrontendRetainedStoragePermitV3, ProgramItemHirV1, ResolvedTargetV1,
    ResolvingDocumentSessionV3, SourceBodyTypesRequestErrorV3, SourceQualifiedNodeIdV2,
    TypeFactsReaderV3, ValueExprKindV1,
};
use quanta_gqlang_v2_hir::{
    CompositionBodyV1, CompositionProducerV1, ProjectionItemV1, QueryHirV1, RelationExprHirV1,
    RelationExprKindV1, RelationSourceV1,
};

struct QueryPartsV3<'a> {
    scan_root: &'a RelationExprHirV1,
    project_root: &'a RelationExprHirV1,
    scan: &'a RelationSourceV1,
    item: &'a ProjectionItemV1,
    base: &'a quanta_gqlang_v2_hir::ValueExprHirV1,
    field: &'a quanta_gqlang_v2_hir::PathSegmentV1,
}

fn query_parts(query: &QueryHirV1) -> Option<QueryPartsV3<'_>> {
    if query.result_shape.is_some() {
        return None;
    }
    let RelationExprKindV1::Project(project) = &query.body.kind else {
        return None;
    };
    let RelationExprKindV1::Source(scan) = &project.input.kind else {
        return None;
    };
    let [item] = project.items() else { return None };
    let ValueExprKindV1::Path(path) = &item.value.kind else {
        return None;
    };
    let ValueExprKindV1::Symbol(base) = &path.base.kind else {
        return None;
    };
    let [field] = path.segments.as_slice() else {
        return None;
    };
    if scan.alias.as_ref()?.name != base.name {
        return None;
    }
    Some(QueryPartsV3 {
        scan_root: &project.input,
        project_root: &query.body,
        scan,
        item,
        base: &path.base,
        field,
    })
}

fn output_name(item: &ProjectionItemV1) -> Option<&str> {
    let ValueExprKindV1::Path(path) = &item.value.kind else {
        return None;
    };
    let [segment] = path.segments.as_slice() else {
        return None;
    };
    Some(
        item.alias
            .as_ref()
            .map_or(segment.name.as_str(), |alias| alias.name.as_str()),
    )
}

fn validate_query_bindings(
    reader: &dyn BinderEvidenceReaderV3,
    parts: &QueryPartsV3<'_>,
) -> Result<(), SourceBodyTypesRequestErrorV3> {
    let Some([alias]) = reader.relation_output(parts.scan_root.id) else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let Some([output]) = reader.relation_output(parts.project_root.id) else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let Some(ResolvedTargetV1::Local(base_target)) = reader.target_of(parts.base.id) else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    if alias.identity() != (parts.scan_root.id, 0)
        || alias.kind != BinderKindV1::Alias
        || !parts
            .scan
            .alias
            .as_ref()
            .is_some_and(|source_alias| alias.name == source_alias.name)
        || base_target != alias
        || output.identity() != (parts.project_root.id, 0)
        || output.kind != BinderKindV1::Alias
        || output_name(parts.item) != Some(output.name.as_str())
    {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    Ok(())
}

/// Continue the exact Binder LET identity into two scalar query type rows.
/// This emits Type facts only; the later scalar materializer still rejects a
/// Composition root and public scalar compile accepts only a Query.
pub fn type_one_let_composition_body_with_request_owner_v3<'authority>(
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
    if declared.checked().source_len() != source.len()
        || extended.source_len(source_id) != Some(source.len())
    {
        return Err(SourceBodyTypesRequestErrorV3::SourceMismatch);
    }
    if !std::ptr::eq(authority, session.authority) {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    let [ProgramItemHirV1::Composition(composition)] =
        declared.checked().checked().document().items.as_slice()
    else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let [authored] = composition.bindings.as_slice() else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let CompositionProducerV1::Query(producer_query) = &authored.producer else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let CompositionBodyV1::Query(body_query) = &composition.body else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let producer =
        query_parts(producer_query).ok_or(SourceBodyTypesRequestErrorV3::UnsupportedShape)?;
    let body = query_parts(body_query).ok_or(SourceBodyTypesRequestErrorV3::UnsupportedShape)?;
    let reader = bound.evidence_v3();
    let Some(let_proof) = reader.composition_binding(composition.id, 0) else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let Some(ResolvedTargetV1::Local(body_source_target)) =
        reader.target_of_use(body.scan_root.id, body.scan.name_span, &body.scan.name)
    else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    if composition.with_name != authored.name
        || body.scan.name != authored.name
        || let_proof.kind != BinderKindV1::CompositionLet
        || let_proof.identity() != (composition.id, 0)
        || let_proof.name != authored.name
        || let_proof.span != authored.name_span
        || body_source_target != let_proof
        || session
            .validator
            .has_current_catalog_spelling_v3(&producer.scan.name)
    {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    validate_query_bindings(reader, &producer)?;
    validate_query_bindings(reader, &body)?;
    let Some(host) = authority.relations().get(&producer.scan.name) else {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    };
    let schema = host.relation_type().row().fields();
    if schema.is_empty()
        || schema
            .iter()
            .any(|field| field.value_type().as_type().as_scalar().is_none())
        || schema
            .iter()
            .all(|field| field.name() != producer.field.name)
        || output_name(producer.item) != Some(body.field.name.as_str())
    {
        return Err(SourceBodyTypesRequestErrorV3::UnsupportedShape);
    }
    let producer_name =
        output_name(producer.item).ok_or(SourceBodyTypesRequestErrorV3::UnsupportedShape)?;
    let body_name =
        output_name(body.item).ok_or(SourceBodyTypesRequestErrorV3::UnsupportedShape)?;
    let bytes = quanta_gqlang_v2_types::one_let_composition_type_storage_upper_v3(
        schema.len(),
        producer_name.len(),
        body_name.len(),
    )
    .ok_or(SourceBodyTypesRequestErrorV3::DemandOverflow)?;
    let storage = root
        .reserve_v3(source_id, source, bytes)
        .map_err(SourceBodyTypesRequestErrorV3::Storage)?;
    let facts = match quanta_gqlang_v2_types::check_one_let_composition_with_request_owner_v3(
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
            node_id: body.project_root.id,
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
