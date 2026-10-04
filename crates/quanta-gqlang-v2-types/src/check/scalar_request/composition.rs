//! Finite two-query Composition typing over a Binder-proven one-LET source.

use super::{
    key, known, reserve, AdmittedScalarProjectionFactsV3, ScalarProjectionCheckRequestErrorV3,
    ScalarProjectionRequestErrorV3,
};
use crate::builtin::ExpressionEffectsV2;
use crate::check::{ExpressionTypeKernelV2, TypedExpressionFactV2, TypedExpressionFactsV2};
use crate::ty::{RecordFieldTypeV1, RecordFieldV1, RecordTypeV1, ScalarRecordErrorV3, ValueTypeV1};
use quanta_gqlang_v2_hir::{
    CheckedHirDocumentV2, CompositionBodyV1, CompositionProducerV1, ExtendedSourceRegistryViewV2,
    FrontendRequestMaterializationRootV3, FrontendRetainedStoragePermitV3, LocatedSpanV2,
    ProgramItemHirV1, QueryHirV1, RelationExprKindV1, SourceIdV2, ValueExprKindV1,
};
use std::mem::size_of;

struct OneLetPartsV3<'a> {
    producer: &'a QueryHirV1,
    body: &'a QueryHirV1,
    producer_output_name: &'a str,
    body_output_name: &'a str,
}

fn query_parts(query: &QueryHirV1) -> Option<(&str, &str, &str)> {
    if query.result_shape.is_some() {
        return None;
    }
    let RelationExprKindV1::Project(project) = &query.body.kind else {
        return None;
    };
    let RelationExprKindV1::Source(scan) = &project.input.kind else {
        return None;
    };
    let [item] = project.items() else {
        return None;
    };
    let ValueExprKindV1::Path(path) = &item.value.kind else {
        return None;
    };
    let ValueExprKindV1::Symbol(base) = &path.base.kind else {
        return None;
    };
    let [segment] = path.segments.as_slice() else {
        return None;
    };
    if scan.alias.as_ref()?.name != base.name {
        return None;
    }
    Some((
        scan.name.as_str(),
        segment.name.as_str(),
        item.alias
            .as_ref()
            .map_or(segment.name.as_str(), |alias| alias.name.as_str()),
    ))
}

fn one_let_parts<'a>(
    checked: &'a CheckedHirDocumentV2<'_>,
    source_row: &RecordTypeV1,
) -> Option<OneLetPartsV3<'a>> {
    let [ProgramItemHirV1::Composition(composition)] = checked.document().items.as_slice() else {
        return None;
    };
    let [authored] = composition.bindings.as_slice() else {
        return None;
    };
    let CompositionProducerV1::Query(producer) = &authored.producer else {
        return None;
    };
    let CompositionBodyV1::Query(body) = &composition.body else {
        return None;
    };
    let (host_name, producer_field, producer_output_name) = query_parts(producer)?;
    let (body_source, body_field, body_output_name) = query_parts(body)?;
    if host_name == authored.name
        || body_source != authored.name
        || composition.with_name != authored.name
        || body_field != producer_output_name
        || source_row
            .field(producer_field)?
            .value_type()
            .as_type()
            .as_scalar()
            .is_none()
        || source_row.fields().is_empty()
        || source_row
            .fields()
            .iter()
            .any(|field| field.value_type().as_type().as_scalar().is_none())
    {
        return None;
    }
    Some(OneLetPartsV3 {
        producer,
        body,
        producer_output_name,
        body_output_name,
    })
}

/// A portable logical upper for two sequential one-field projects and their
/// four retained facts/rows. The producer row remains live while the body is
/// checked. Vec capacities are checked against power-of-two ceilings.
#[must_use]
pub fn one_let_composition_type_storage_upper_v3(
    schema_fields: usize,
    producer_output_name_bytes: usize,
    body_output_name_bytes: usize,
) -> Option<usize> {
    let width = |count: usize, slot: usize| count.checked_next_power_of_two()?.checked_mul(slot);
    let facts = width(
        4,
        size_of::<(crate::check::SourceQualifiedNodeIdV2, TypedExpressionFactV2)>(),
    )?;
    let rows = width(
        4,
        size_of::<(crate::check::SourceQualifiedNodeIdV2, RecordTypeV1)>(),
    )?;
    let schema_index = width(schema_fields, size_of::<&RecordFieldV1>())?;
    let body_index = width(1, size_of::<&RecordFieldV1>())?;
    let field_scratch = 2_usize.checked_mul(
        width(1, size_of::<RecordFieldTypeV1>())?
            .checked_add(width(1, size_of::<RecordFieldV1>())?)?
            .checked_add(width(1, size_of::<(usize, &RecordFieldTypeV1)>())?)?,
    )?;
    let arc_row = 2_usize.checked_mul(
        size_of::<RecordFieldV1>()
            .checked_add(size_of::<[usize; 2]>())?
            .checked_add(std::mem::align_of::<RecordFieldV1>())?,
    )?;
    let names = producer_output_name_bytes
        .checked_add(body_output_name_bytes)?
        .checked_mul(3)?;
    [
        facts,
        rows,
        schema_index,
        body_index,
        field_scratch,
        arc_row,
        names,
        size_of::<crate::TypeDiagnosticV2>(),
    ]
    .into_iter()
    .try_fold(0_usize, usize::checked_add)
}

fn checked_demand(
    checked: &CheckedHirDocumentV2<'_>,
    source_row: &RecordTypeV1,
    source_id: SourceIdV2,
    source: &str,
) -> Result<usize, ScalarProjectionCheckRequestErrorV3> {
    let syntax = checked.document();
    if syntax.source_id != source_id || syntax.source_len != source.len() {
        return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
    }
    let parts = one_let_parts(checked, source_row)
        .ok_or(ScalarProjectionCheckRequestErrorV3::UnsupportedShape)?;
    one_let_composition_type_storage_upper_v3(
        source_row.fields().len(),
        parts.producer_output_name.len(),
        parts.body_output_name.len(),
    )
    .ok_or(ScalarProjectionCheckRequestErrorV3::DemandOverflow)
}

/// Consume only a Binder-proven one-LET source's checked Type subset. Sema
/// authenticates the concrete LET/source identity before it calls this owner.
pub fn check_one_let_composition_with_request_owner_v3(
    checked: &CheckedHirDocumentV2<'_>,
    extended: &ExtendedSourceRegistryViewV2,
    source_row: &RecordTypeV1,
    builtins: crate::AdmittedBuiltinRegistryV3,
    storage: FrontendRetainedStoragePermitV3,
    root: &FrontendRequestMaterializationRootV3<'_>,
    source_id: SourceIdV2,
    source: &str,
) -> Result<AdmittedScalarProjectionFactsV3, ScalarProjectionCheckRequestErrorV3> {
    storage
        .validate_owner_v3(root, source_id, source)
        .map_err(ScalarProjectionCheckRequestErrorV3::Storage)?;
    builtins
        .validate_owner_v3(root, source_id, source)
        .map_err(ScalarProjectionCheckRequestErrorV3::Storage)?;
    let required = checked_demand(checked, source_row, source_id, source)?;
    storage
        .validate_minimum_reservation_v3(root, source_id, source, required)
        .map_err(ScalarProjectionCheckRequestErrorV3::Storage)?;
    let kernel = ExpressionTypeKernelV2::for_admitted_scalar_projection_v3(&builtins);
    let mut diagnostics = Vec::new();
    diagnostics
        .try_reserve_exact(1)
        .map_err(|_allocation| ScalarProjectionCheckRequestErrorV3::FallibleAllocation)?;
    if diagnostics.capacity() > 1 {
        return Err(ScalarProjectionCheckRequestErrorV3::FallibleAllocation);
    }
    #[cfg(test)]
    crate::ty::request_storage_observer::note_vec(&diagnostics);
    let facts = match check_two_queries(&kernel, checked, extended, source_row) {
        Ok(facts) => facts,
        Err(ScalarProjectionRequestErrorV3::UnsupportedShape) => {
            return Err(ScalarProjectionCheckRequestErrorV3::UnsupportedShape);
        }
        Err(ScalarProjectionRequestErrorV3::Storage) => {
            return Err(ScalarProjectionCheckRequestErrorV3::FallibleAllocation);
        }
        Err(ScalarProjectionRequestErrorV3::Diagnostic(diagnostic)) => {
            diagnostics.push(diagnostic);
            return Err(ScalarProjectionCheckRequestErrorV3::Diagnostic {
                diagnostics,
                _storage: storage,
            });
        }
    };
    Ok(AdmittedScalarProjectionFactsV3 {
        facts,
        builtins,
        storage,
    })
}

fn check_two_queries(
    kernel: &ExpressionTypeKernelV2,
    checked: &CheckedHirDocumentV2<'_>,
    extended: &ExtendedSourceRegistryViewV2,
    source_row: &RecordTypeV1,
) -> Result<TypedExpressionFactsV2, ScalarProjectionRequestErrorV3> {
    if !matches!(kernel.builtins, crate::check::KernelBuiltinsV2::Shared(_)) {
        return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
    }
    let parts = one_let_parts(checked, source_row)
        .ok_or(ScalarProjectionRequestErrorV3::UnsupportedShape)?;
    let syntax = checked.document();
    let mut facts = Vec::new();
    reserve(&mut facts, 4)?;
    let mut rows = Vec::new();
    reserve(&mut rows, 4)?;
    let producer_row = append_query(
        parts.producer,
        source_row,
        syntax.source_id,
        extended,
        &mut facts,
        &mut rows,
    )?;
    let _body_row = append_query(
        parts.body,
        &producer_row,
        syntax.source_id,
        extended,
        &mut facts,
        &mut rows,
    )?;
    facts.sort_unstable_by_key(|entry| entry.0);
    rows.sort_unstable_by_key(|entry| entry.0);
    Ok(TypedExpressionFactsV2 {
        facts: std::collections::BTreeMap::new(),
        relation_rows: std::collections::BTreeMap::new(),
        request_facts: Some(facts),
        request_rows: Some(rows),
    })
}

fn append_query(
    query: &QueryHirV1,
    source_row: &RecordTypeV1,
    source_id: SourceIdV2,
    extended: &ExtendedSourceRegistryViewV2,
    facts: &mut Vec<(crate::check::SourceQualifiedNodeIdV2, TypedExpressionFactV2)>,
    rows: &mut Vec<(crate::check::SourceQualifiedNodeIdV2, RecordTypeV1)>,
) -> Result<RecordTypeV1, ScalarProjectionRequestErrorV3> {
    let RelationExprKindV1::Project(project) = &query.body.kind else {
        return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
    };
    let RelationExprKindV1::Source(scan) = &project.input.kind else {
        return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
    };
    let [item] = project.items() else {
        return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
    };
    let ValueExprKindV1::Path(path) = &item.value.kind else {
        return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
    };
    let ValueExprKindV1::Symbol(base) = &path.base.kind else {
        return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
    };
    let [segment] = path.segments.as_slice() else {
        return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
    };
    if !scan
        .alias
        .as_ref()
        .is_some_and(|alias| alias.name == base.name)
    {
        return Err(ScalarProjectionRequestErrorV3::UnsupportedShape);
    }
    let mut source_index = Vec::new();
    reserve(&mut source_index, source_row.fields().len())?;
    source_index.extend(source_row.fields());
    source_index.sort_unstable_by(|left: &&RecordFieldV1, right| left.name().cmp(right.name()));
    let index = source_index
        .binary_search_by(|field| field.name().cmp(&segment.name))
        .map_err(|_missing| ScalarProjectionRequestErrorV3::UnsupportedShape)?;
    let field = source_index
        .get(index)
        .ok_or(ScalarProjectionRequestErrorV3::UnsupportedShape)?;
    let name = item
        .alias
        .as_ref()
        .map_or(segment.name.as_str(), |alias| alias.name.as_str());
    let span = item
        .alias
        .as_ref()
        .map_or(item.value.span, |alias| alias.span);
    let location = extended
        .combined()
        .try_check_location(LocatedSpanV2 { source_id, span })
        .map_err(|_source_error| ScalarProjectionRequestErrorV3::UnsupportedShape)?;
    let mut output_fields = Vec::new();
    reserve(&mut output_fields, 1)?;
    output_fields.push(
        RecordFieldTypeV1::try_new(name, field.value_type().clone(), location)
            .map_err(ScalarProjectionRequestErrorV3::Diagnostic)?,
    );
    let base_type = ValueTypeV1::try_record(source_row.clone())
        .map_err(ScalarProjectionRequestErrorV3::Diagnostic)?;
    let pure = ExpressionEffectsV2::pure_scalar();
    facts.push((key(source_id, path.base.id), known(base_type, pure)));
    facts.push((
        key(source_id, item.value.id),
        known(field.value_type().clone(), pure),
    ));
    let project_row =
        RecordTypeV1::try_new_scalar_request_v3(output_fields).map_err(|error| match error {
            ScalarRecordErrorV3::Storage => ScalarProjectionRequestErrorV3::Storage,
            ScalarRecordErrorV3::Diagnostic(diagnostic) => {
                ScalarProjectionRequestErrorV3::Diagnostic(diagnostic)
            }
        })?;
    rows.push((key(source_id, project.input.id), source_row.clone()));
    rows.push((key(source_id, query.body.id), project_row.clone()));
    Ok(project_row)
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "independent ordinary Type oracle")]

    use super::*;
    use crate::check::{
        DeferredRelationRowFieldInputV2, DeferredRelationRowInputV2, ExpressionTypeKernelV2,
        RelationRowFieldSourceV2, ResolvedTypeInputEntryV2, ResolvedTypeInputValueV2,
        ResolvedTypeInputsV2, TypeFactsReaderV3,
    };
    use quanta_gqlang_v2_hir::{
        AliasV1, CheckedHirDocumentOwnedV2, FrontendRequestStoragePermitV3,
        FrontendRequestStorageSinkV3, FrontendStorageRefusalV3, HirBuilderV1, HirDocumentV1,
        HirNodeIdV1, PathSegmentV1, ProjectionItemV1, SourceRegistrationV2, SourceRegistryViewV2,
        SpanV1,
    };

    struct Sink;
    struct Permit;
    impl FrontendRequestStorageSinkV3 for Sink {
        fn reserve_v3(
            &self,
            _bytes: u64,
        ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
            Ok(Box::new(Permit))
        }
        fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
            permit.as_any_v3().is::<Permit>()
        }
    }

    fn one_let_checked(source_id: SourceIdV2) -> (CheckedHirDocumentOwnedV2, [HirNodeIdV1; 8]) {
        let b = HirBuilderV1::new();
        let span = SpanV1::new(0, 90);
        let producer_scan = b.relation_source(
            "records",
            span,
            Some(AliasV1 {
                name: "p".into(),
                span,
            }),
            span,
        );
        let producer_base = b.symbol("p", span);
        let producer_path = b
            .try_path(
                producer_base.clone(),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span,
                }],
                span,
            )
            .unwrap();
        let producer_path_id = producer_path.id;
        let producer_project = b
            .try_project(
                producer_scan.clone(),
                vec![ProjectionItemV1 {
                    value: producer_path,
                    alias: Some(AliasV1 {
                        name: "id".into(),
                        span,
                    }),
                }],
                span,
            )
            .unwrap();
        let producer = b.query(producer_project.clone(), None, span);
        let body_scan = b.relation_source(
            "s",
            span,
            Some(AliasV1 {
                name: "q".into(),
                span,
            }),
            span,
        );
        let body_base = b.symbol("q", span);
        let body_path = b
            .try_path(
                body_base.clone(),
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span,
                }],
                span,
            )
            .unwrap();
        let body_path_id = body_path.id;
        let body_project = b
            .try_project(
                body_scan.clone(),
                vec![ProjectionItemV1 {
                    value: body_path,
                    alias: Some(AliasV1 {
                        name: "out".into(),
                        span,
                    }),
                }],
                span,
            )
            .unwrap();
        let body = b.query(body_project.clone(), None, span);
        let composition = b.composition(
            vec![b.composition_binding("s", span, producer.into())],
            "s",
            span,
            body.into(),
            span,
        );
        let ids = [
            producer_scan.id,
            producer_base.id,
            producer_path_id,
            producer_project.id,
            body_scan.id,
            body_base.id,
            body_path_id,
            body_project.id,
        ];
        let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
            source_id,
            source_len: 100,
            items: vec![ProgramItemHirV1::from(composition)],
        })
        .unwrap();
        (checked, ids)
    }

    #[test]
    fn one_let_facts_and_rows_match_ordinary_type_kernel_with_bounded_births() {
        let source = " ".repeat(100);
        let source_id = SourceIdV2::from_raw(844);
        let (
            checked,
            [producer_scan_id, producer_base_id, producer_path_id, producer_project_id, body_scan_id, body_base_id, body_path_id, body_project_id],
        ) = one_let_checked(source_id);
        let span = SpanV1::new(0, 90);
        let extended = SourceRegistryViewV2::try_new(Vec::new())
            .unwrap()
            .try_extend_document(SourceRegistrationV2 {
                source_id,
                source_len: source.len(),
            })
            .unwrap();
        let location = extended
            .combined()
            .try_check_location(LocatedSpanV2 { source_id, span })
            .unwrap();
        let row = RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
            "id",
            ValueTypeV1::int(),
            location.clone(),
        )
        .unwrap()])
        .unwrap();
        let key = |node_id| key(source_id, node_id);
        let values = vec![producer_base_id, body_base_id]
            .into_iter()
            .zip([producer_scan_id, body_scan_id])
            .map(|(base, scan)| ResolvedTypeInputEntryV2 {
                key: key(base),
                value: ResolvedTypeInputValueV2::BindingFromRelationRow(key(scan)),
            })
            .collect();
        let output = |name: &str, value| {
            vec![DeferredRelationRowFieldInputV2::new(
                0,
                name,
                location.clone(),
                RelationRowFieldSourceV2::PriorExpression(key(value)),
            )]
        };
        let rows = vec![
            DeferredRelationRowInputV2::known_source(
                key(producer_scan_id),
                1,
                vec![DeferredRelationRowFieldInputV2::new(
                    0,
                    "id",
                    location.clone(),
                    RelationRowFieldSourceV2::Known(ValueTypeV1::int()),
                )],
            ),
            DeferredRelationRowInputV2::expression_fields(
                key(producer_project_id),
                1,
                output("id", producer_path_id),
            ),
            DeferredRelationRowInputV2::prior_relation(key(body_scan_id), key(producer_project_id)),
            DeferredRelationRowInputV2::expression_fields(
                key(body_project_id),
                1,
                output("out", body_path_id),
            ),
        ];
        let inputs = ResolvedTypeInputsV2::try_new(
            checked.as_checked(),
            &extended,
            values,
            Vec::new(),
            rows,
        )
        .unwrap();
        let ordinary = ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs)
            .unwrap();

        let upper = one_let_composition_type_storage_upper_v3(1, 2, 3).unwrap();
        let sink = Sink;
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
        let parent = root.reserve_v3(source_id, &source, 1).unwrap();
        let builtins = crate::BuiltinRegistryV2::try_closed_with_request_owner_v3(
            &root, source_id, &source, &parent,
        )
        .unwrap();
        let storage = root.reserve_v3(source_id, &source, upper).unwrap();
        crate::ty::request_storage_observer::begin();
        let request = check_one_let_composition_with_request_owner_v3(
            &checked.as_checked(),
            &extended,
            &row,
            builtins,
            storage,
            &root,
            source_id,
            &source,
        )
        .unwrap();
        let gross_birth = crate::ty::request_storage_observer::finish();
        assert!(
            gross_birth > 0 && gross_birth <= upper,
            "observed={gross_birth} upper={upper}"
        );
        assert_eq!(request.len(), 4);
        assert_eq!(request.relation_row_count(), 4);
        for node_id in [
            producer_base_id,
            producer_path_id,
            body_base_id,
            body_path_id,
        ] {
            assert_eq!(request.get(key(node_id)), ordinary.get(key(node_id)));
        }
        for node_id in [
            producer_scan_id,
            producer_project_id,
            body_scan_id,
            body_project_id,
        ] {
            assert_eq!(
                request.relation_row(key(node_id)),
                ordinary.relation_row(key(node_id))
            );
        }
    }
}
