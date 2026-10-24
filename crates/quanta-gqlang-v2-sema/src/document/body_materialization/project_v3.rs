//! Existing request scalar Project seal, shared by direct and filtered scans.

use super::{
    AdmittedSourceBodyTypesV3, FrontendFailureV2, OutputColumnContractV2, ScalarBodyBuffersV3,
    SemanticStepCounterV2, SourceQualifiedNodeIdV2, TypeFactsReaderV3, TypedRelationExprV2,
    TypedRelationV2,
};

pub(super) fn seal_request_project_v3(
    query: &quanta_gqlang_v2_hir::QueryHirV1,
    project: &quanta_gqlang_v2_hir::ProjectionV1,
    input: TypedRelationExprV2,
    typed: &AdmittedSourceBodyTypesV3<'_>,
    buffers: &mut ScalarBodyBuffersV3,
    steps: &mut SemanticStepCounterV2,
) -> Result<TypedRelationV2, FrontendFailureV2> {
    let source_id = typed.source_id_v3();
    let bound = typed._bound.evidence_v3();
    let facts = &typed.facts;
    let extended = &typed._extended;
    let checked = |span| {
        extended
            .combined()
            .try_check_location(super::super::location(source_id, span))
            .map_err(|_error| {
                super::super::document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    span,
                    "request Project location has no checked source",
                )
            })
    };
    let frame = bound.relation_output(query.body.id).ok_or_else(|| {
        super::super::document_failure(
            "sema.document_validation_failed",
            source_id,
            query.body.span,
            "binder omitted projection output frame",
        )
    })?;
    if frame.len() != project.items().len() {
        return Err(super::super::document_failure(
            "sema.document_validation_failed",
            source_id,
            query.body.span,
            "binder projection output differs from authored item count",
        ));
    }
    for (ordinal, (item, binding)) in project.items().iter().zip(frame).enumerate() {
        if binding.identity() != (query.body.id, ordinal) {
            return Err(super::super::document_failure(
                "sema.document_validation_failed",
                source_id,
                item.value.span,
                "projection output does not retain its Binder-owned ordinal",
            ));
        }
        let value = super::super::materialize_declaration_expr(
            &item.value,
            source_id,
            bound,
            facts,
            extended,
            &mut buffers.expressions,
            steps,
            super::super::AggregatePositionV2::Projection,
        )?;
        if !matches!(
            value.type_state(),
            super::super::ExpressionTypeStateV2::Known(_)
        ) {
            return Err(super::super::document_failure(
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
            super::super::OutputColumnOriginV2::Expression(value.key()),
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
        super::super::document_failure(
            "sema.document_validation_failed",
            source_id,
            query.body.span,
            "types owner omitted projection row fact",
        )
    })?;
    let root = super::super::relation_materialization::seal_scalar_project_v3(
        &query.body,
        checked(query.body.span)?,
        project_key,
        super::super::relation_materialization::ScalarProjectPartsV3 {
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
        super::super::SemanticItemIdV2::new(source_id, query.id),
        checked(query.span)?,
        project_key,
        root,
        output,
    )
    .map_err(|error| {
        super::super::document_failure(
            "sema.document_validation_failed",
            source_id,
            query.span,
            error,
        )
    })
}
