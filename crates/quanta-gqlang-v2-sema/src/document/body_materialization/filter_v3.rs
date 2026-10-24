//! Finite request-owned Filter typed-expression prebirth.

use super::{
    reserve_scalar_body_vec_v3, AdmittedSourceBodyTypesV3, FrontendFailureV2, ScalarBodyBuffersV3,
    SemanticStepCounterV2, SourceQualifiedNodeIdV2, TypeFactsReaderV3, TypedExprV2,
    TypedPathSegmentV2, TypedRelationExprV2, ValueExprKindV1,
};

pub(super) enum RequestFilterPrebirthErrorV3 {
    Allocation,
    Diagnostic(FrontendFailureV2),
}

impl From<FrontendFailureV2> for RequestFilterPrebirthErrorV3 {
    fn from(failure: FrontendFailureV2) -> Self {
        Self::Diagnostic(failure)
    }
}

fn request_filter_string_v3(value: &str) -> Result<String, RequestFilterPrebirthErrorV3> {
    let mut copied = String::new();
    copied
        .try_reserve_exact(value.len())
        .map_err(|_allocation| RequestFilterPrebirthErrorV3::Allocation)?;
    if copied.capacity() > value.len() {
        return Err(RequestFilterPrebirthErrorV3::Allocation);
    }
    copied.push_str(value);
    Ok(copied)
}

/// Build only the four checked Filter facts admitted by the request Type
/// producer. Strings and the path Vec are born fallibly before the relation.
pub(super) fn prepare_request_filter_predicate_v3(
    filter: &quanta_gqlang_v2_hir::RelationFilterV1,
    scan_id: super::super::HirNodeIdV1,
    typed: &AdmittedSourceBodyTypesV3<'_>,
    expressions: &mut Vec<(SourceQualifiedNodeIdV2, TypedExprV2)>,
) -> Result<super::super::TypedBoolExprV2, RequestFilterPrebirthErrorV3> {
    let source_id = typed.source_id_v3();
    let bad = |span: super::super::SpanV1, message: &'static str| {
        RequestFilterPrebirthErrorV3::Diagnostic(super::super::document_failure(
            "sema.document_validation_failed",
            source_id,
            span,
            message,
        ))
    };
    let quanta_gqlang_v2_hir::BoolExprKindV1::Compare(compare) = &filter.predicate.kind else {
        return Err(bad(
            filter.predicate.span,
            "request Filter predicate is not Compare",
        ));
    };
    let ValueExprKindV1::Path(path) = &compare.lhs.kind else {
        return Err(bad(
            compare.lhs.span,
            "request Filter left operand is not Path",
        ));
    };
    let [field] = path.segments.as_slice() else {
        return Err(bad(
            compare.lhs.span,
            "request Filter path has multiple segments",
        ));
    };
    let ValueExprKindV1::Symbol(base) = &path.base.kind else {
        return Err(bad(
            path.base.span,
            "request Filter path base is not Symbol",
        ));
    };
    let Some(super::super::ResolvedTargetV1::Local(binding)) =
        typed._bound.evidence_v3().target_of(path.base.id)
    else {
        return Err(bad(
            path.base.span,
            "request Filter base has no local binding",
        ));
    };
    if binding.identity() != (scan_id, 0)
        || binding.kind != super::super::BinderKindV1::Alias
        || binding.name != base.name
        || !expressions.is_empty()
        || expressions.capacity() < 4
    {
        return Err(bad(
            path.base.span,
            "request Filter binding or expression capacity differs from admitted source",
        ));
    }
    let facts = &typed.facts;
    let extended = &typed._extended;
    let key = |node_id| SourceQualifiedNodeIdV2 { source_id, node_id };
    let make = |node_id, span, shape| -> Result<TypedExprV2, RequestFilterPrebirthErrorV3> {
        let fact = facts
            .get(key(node_id))
            .ok_or_else(|| bad(span, "types owner omitted request Filter expression fact"))?;
        if !matches!(fact.state, super::super::ExpressionTypeStateV2::Known(_)) {
            return Err(bad(span, "request Filter expression has unknown type"));
        }
        let raw = super::super::location(source_id, span);
        let context = super::super::AggregateContextV2::try_new(
            super::super::AggregatePositionV2::Predicate,
            fact.effects.aggregation,
            false,
            raw,
        )?;
        let checked = extended
            .combined()
            .try_check_location(raw)
            .map_err(|_error| bad(span, "request Filter location has no checked source"))?;
        Ok(TypedExprV2::try_from_fact(
            key(node_id),
            fact,
            context,
            shape,
            checked,
        )?)
    };
    let resolved = super::super::ResolvedValueBindingV2::try_new(
        source_id,
        scan_id,
        0,
        super::super::ResolvedBindingKindV2::Alias,
        request_filter_string_v3(&binding.name)?,
    )
    .map_err(|error| bad(path.base.span, error))?;
    let typed_base = make(
        path.base.id,
        path.base.span,
        super::super::TypedExprShapeV2::Binding(resolved),
    )?;
    expressions.push((typed_base.key(), typed_base));

    let mut segments = reserve_scalar_body_vec_v3::<TypedPathSegmentV2>(1)
        .map_err(|_error| RequestFilterPrebirthErrorV3::Allocation)?;
    let checked_field = extended
        .combined()
        .try_check_location(super::super::location(source_id, field.span))
        .map_err(|_error| bad(field.span, "request Filter field has no checked source"))?;
    segments.push(TypedPathSegmentV2::new(
        request_filter_string_v3(&field.name)?,
        checked_field,
    ));
    let typed_path = make(
        compare.lhs.id,
        compare.lhs.span,
        super::super::TypedExprShapeV2::Path {
            base: key(path.base.id),
            segments,
        },
    )?;
    expressions.push((typed_path.key(), typed_path));

    let literal_key = key(compare.rhs.id);
    let literal_fact = facts.get(literal_key).ok_or_else(|| {
        bad(
            compare.rhs.span,
            "types owner omitted request Filter literal",
        )
    })?;
    let literal_shape = match (&compare.rhs.kind, literal_fact.literal.as_ref()) {
        (
            ValueExprKindV1::Literal(quanta_gqlang_v2_hir::LiteralV1::Int(text)),
            Some(quanta_gqlang_v2_types::TypedLiteralFactV2::Value(
                quanta_gqlang_v2_types::TypedLiteralValueV2::Int(value),
            )),
        ) if text.parse::<i128>() == Ok(*value) => super::super::TypedExprShapeV2::Literal(
            quanta_gqlang_v2_types::TypedLiteralFactV2::Value(
                quanta_gqlang_v2_types::TypedLiteralValueV2::Int(*value),
            ),
        ),
        (
            ValueExprKindV1::Literal(quanta_gqlang_v2_hir::LiteralV1::Bool(authored)),
            Some(quanta_gqlang_v2_types::TypedLiteralFactV2::Value(
                quanta_gqlang_v2_types::TypedLiteralValueV2::Bool(checked),
            )),
        ) if authored == checked => super::super::TypedExprShapeV2::Literal(
            quanta_gqlang_v2_types::TypedLiteralFactV2::Value(
                quanta_gqlang_v2_types::TypedLiteralValueV2::Bool(*checked),
            ),
        ),
        (
            ValueExprKindV1::Literal(quanta_gqlang_v2_hir::LiteralV1::Str(authored)),
            Some(quanta_gqlang_v2_types::TypedLiteralFactV2::Value(
                quanta_gqlang_v2_types::TypedLiteralValueV2::String(checked),
            )),
        ) if authored == checked => super::super::TypedExprShapeV2::Literal(
            quanta_gqlang_v2_types::TypedLiteralFactV2::Value(
                quanta_gqlang_v2_types::TypedLiteralValueV2::String(request_filter_string_v3(
                    checked,
                )?),
            ),
        ),
        _ => {
            return Err(bad(
                compare.rhs.span,
                "request Filter literal differs from checked Type fact",
            ))
        }
    };
    let typed_literal = make(compare.rhs.id, compare.rhs.span, literal_shape)?;
    expressions.push((typed_literal.key(), typed_literal));

    let predicate_key = key(filter.predicate.id);
    let predicate_fact = facts.get(predicate_key).ok_or_else(|| {
        bad(
            filter.predicate.span,
            "types owner omitted request Filter predicate",
        )
    })?;
    let operator = predicate_fact.operator_id.ok_or_else(|| {
        bad(
            filter.predicate.span,
            "types owner omitted request Filter comparison operator",
        )
    })?;
    let typed_predicate = make(
        filter.predicate.id,
        filter.predicate.span,
        super::super::TypedExprShapeV2::Compare {
            operator,
            lhs: key(compare.lhs.id),
            rhs: literal_key,
        },
    )?;
    let predicate = super::super::TypedBoolExprV2::try_new(
        typed_predicate.clone(),
        super::super::location(source_id, filter.predicate.span),
    )?;
    expressions.push((predicate_key, typed_predicate));
    Ok(predicate)
}

/// Attach the admitted predicate to the already materialized Scan.
pub(super) fn attach_request_filter_v3(
    project: &quanta_gqlang_v2_hir::ProjectionV1,
    filter: &quanta_gqlang_v2_hir::RelationFilterV1,
    input: TypedRelationExprV2,
    typed: &AdmittedSourceBodyTypesV3<'_>,
    buffers: &mut ScalarBodyBuffersV3,
    steps: &mut SemanticStepCounterV2,
) -> Result<TypedRelationExprV2, FrontendFailureV2> {
    let source_id = typed.source_id_v3();
    let facts = &typed.facts;
    let checked = |span| {
        typed
            ._extended
            .combined()
            .try_check_location(super::super::location(source_id, span))
            .map_err(|_error| {
                super::super::document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    span,
                    "request Filter location has no checked source",
                )
            })
    };
    let filtered = {
        let quanta_gqlang_v2_hir::BoolExprKindV1::Compare(compare) = &filter.predicate.kind else {
            return Err(super::super::document_failure(
                "sema.document_validation_failed",
                source_id,
                filter.predicate.span,
                "request Filter predicate differs from the checked Compare",
            ));
        };
        steps.charge(source_id, compare.lhs.span)?;
        let ValueExprKindV1::Path(path) = &compare.lhs.kind else {
            return Err(super::super::document_failure(
                "sema.document_validation_failed",
                source_id,
                compare.lhs.span,
                "request Filter left operand differs from checked Path",
            ));
        };
        steps.charge(source_id, path.base.span)?;
        steps.charge(source_id, compare.rhs.span)?;
        steps.charge(source_id, filter.predicate.span)?;
        let predicate_key = SourceQualifiedNodeIdV2 {
            source_id,
            node_id: filter.predicate.id,
        };
        let predicate = buffers.prepared_filter_predicate.take().ok_or_else(|| {
            super::super::document_failure(
                "sema.document_validation_failed",
                source_id,
                filter.predicate.span,
                "request Filter predicate has no admitted typed prebirth",
            )
        })?;
        if predicate.expression().key() != predicate_key {
            return Err(super::super::document_failure(
                "sema.document_validation_failed",
                source_id,
                filter.predicate.span,
                "request Filter predicate identity differs from checked source",
            ));
        }
        let previous = input.output();
        let filter_key = SourceQualifiedNodeIdV2 {
            source_id,
            node_id: project.input.id,
        };
        if facts.relation_row(filter_key) != Some(previous.row()) {
            return Err(super::super::document_failure(
                "sema.document_validation_failed",
                source_id,
                project.input.span,
                "types owner Filter row differs from its Source input",
            ));
        }
        let mut filter_columns = std::mem::take(&mut buffers.filter_columns);
        if !filter_columns.is_empty() || filter_columns.capacity() < previous.columns().len() {
            return Err(super::super::document_failure(
                "sema.document_validation_failed",
                source_id,
                project.input.span,
                "request Filter columns differ from admitted capacity",
            ));
        }
        filter_columns.extend(previous.columns().iter().cloned());
        let output = super::super::RelationOutputContractV2::try_new(
            previous.row().clone(),
            previous.provenance().clone(),
            filter_columns,
            previous.cardinality(),
            previous.partition_keys().to_vec(),
            previous.ordering().to_vec(),
            previous
                .effects()
                .compose([predicate.expression().effects()]),
        )
        .map_err(|error| {
            super::super::document_failure(
                "sema.document_validation_failed",
                source_id,
                project.input.span,
                error,
            )
        })?;
        TypedRelationExprV2::filter_with_request_input_v3(
            filter_key,
            checked(project.input.span)?,
            input,
            predicate,
            output,
            std::mem::take(&mut buffers.filter_inputs),
        )
        .map_err(|error| {
            super::super::document_failure(
                "sema.document_validation_failed",
                source_id,
                project.input.span,
                error,
            )
        })?
    };
    Ok(filtered)
}
