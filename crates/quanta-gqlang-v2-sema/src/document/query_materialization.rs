//! Query and composition materialization from sealed owner facts.

use super::{
    document_failure, location, materialize_relation, resolve_check_item,
    resolve_composition_explain, resolve_graph_item, resolve_graph_target_use, BTreeMap,
    BindResultV1, BinderEvidenceReaderV3, BinderKindV1, ClosedFrontendAuthorityV2,
    DocumentCatalogValidatorV2, ExpressionEffectsV2, ExtendedSourceRegistryViewV2,
    FrontendFailureV2, FrontendTransactionErrorV2, HirNodeIdV1, OutputColumnContractV2,
    OutputColumnOriginV2, OutputProvenanceV2, RelationOutputContractV2, ResolvedBindingKindV2,
    ResolvedCompositionBindingV2, ResolvedCompositionBodyV2, ResolvedCompositionProducerV2,
    ResolvedCompositionV2, ResolvedGraphQueryV2, ResolvedGraphUseV2, ResolvedPresentationKindV2,
    ResolvedPresentationV2, ResolvedTargetV1, ResolvedValueBindingV2, ScalarTyV1, SemanticItemIdV2,
    SemanticStepCounterV2, SourceIdV2, SourceQualifiedNodeIdV2, TypeFactsReaderV3, TypedExprV2,
    TypedRelationExprV2, TypedRelationV2, ValueTypeV1, ViewMaterializationInputsV2,
};

pub(super) fn materialize_query(
    query: &quanta_gqlang_v2_hir::QueryHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &dyn BinderEvidenceReaderV3,
    facts: &dyn TypeFactsReaderV3,
    extended: &ExtendedSourceRegistryViewV2,
    composition_outputs: &BTreeMap<(HirNodeIdV1, usize), RelationOutputContractV2>,
    views: &ViewMaterializationInputsV2<'_>,
    expressions: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &mut SemanticStepCounterV2,
) -> Result<TypedRelationV2, FrontendFailureV2> {
    if query.result_shape.is_some() {
        return Err(document_failure(
            "sema.document_validation_failed",
            source_id,
            query.span,
            "authored query result shapes are not supported by closed compilation",
        ));
    }
    let item_location = extended
        .combined()
        .try_check_location(location(source_id, query.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                query.span,
                error.to_string(),
            )
        })?;
    let root = materialize_relation(
        &query.body,
        source_id,
        authority,
        bound,
        facts,
        extended,
        composition_outputs,
        views,
        expressions,
        nested_relations,
        steps,
    )?;
    let output = root.output().clone();
    TypedRelationV2::try_new(
        SemanticItemIdV2::new(source_id, query.id),
        item_location,
        SourceQualifiedNodeIdV2 {
            source_id,
            node_id: query.body.id,
        },
        root,
        output,
    )
    .map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            query.span,
            error,
        )
    })
}

fn materialize_composition_sample(
    sample: &quanta_gqlang_v2_hir::SampleHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    steps: &mut SemanticStepCounterV2,
) -> Result<(ResolvedPresentationV2, RelationOutputContractV2), FrontendFailureV2> {
    let scope = sample
        .scope
        .as_ref()
        .map(|reference| {
            resolve_graph_target_use(
                sample.id,
                reference,
                crate::GraphReferenceKindV2::Scope,
                source_id,
                authority,
                bound,
                composition_targets,
                extended,
                steps,
            )
            .map(|(target, location)| ResolvedGraphUseV2::new(target, location))
        })
        .transpose()?;
    let location = extended
        .combined()
        .try_check_location(location(source_id, sample.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                sample.span,
                error.to_string(),
            )
        })?;
    let presentation = ResolvedPresentationV2::try_new(
        SemanticItemIdV2::new(source_id, sample.id),
        location.clone(),
        ResolvedPresentationKindV2::Sample {
            query_text: sample.query_text.clone(),
            mode: sample.mode,
            limit: sample.limit,
            scope,
        },
    )
    .map_err(|error| document_failure("sema.invalid_item_shape", source_id, sample.span, error))?;
    let field = quanta_gqlang_v2_types::RecordFieldTypeV1::try_new(
        "path",
        ValueTypeV1::scalar(ScalarTyV1::RepoPath),
        location.clone(),
    )
    .map_err(|error| {
        document_failure(
            "sema.invalid_item_shape",
            source_id,
            sample.span,
            error.to_string(),
        )
    })?;
    let row = quanta_gqlang_v2_types::RecordTypeV1::try_new(vec![field]).map_err(|error| {
        document_failure(
            "sema.invalid_item_shape",
            source_id,
            sample.span,
            error.to_string(),
        )
    })?;
    let output = RelationOutputContractV2::try_new(
        row,
        OutputProvenanceV2::default(),
        vec![OutputColumnContractV2::new(
            "path".to_owned(),
            OutputColumnOriginV2::Generated(presentation.item_id()),
            location,
        )],
        quanta_gqlang_v2_types::CardinalityV1::Set,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(|error| document_failure("sema.invalid_item_shape", source_id, sample.span, error))?;
    Ok((presentation, output))
}

fn materialize_composition_graph(
    graph: &quanta_gqlang_v2_hir::GraphQueryHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    steps: &mut SemanticStepCounterV2,
) -> Result<(ResolvedGraphQueryV2, Option<RelationOutputContractV2>), FrontendFailureV2> {
    let resolved = resolve_graph_item(
        graph,
        source_id,
        authority,
        bound,
        composition_targets,
        extended,
        steps,
    )?;
    if !matches!(
        graph.kind,
        quanta_gqlang_v2_hir::GraphQueryKindV1::Path
            | quanta_gqlang_v2_hir::GraphQueryKindV1::Reachability
    ) {
        return Ok((resolved, None));
    }
    let location = extended
        .combined()
        .try_check_location(location(source_id, graph.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                graph.span,
                error.to_string(),
            )
        })?;
    let row = quanta_gqlang_v2_types::RecordTypeV1::try_new(vec![
        quanta_gqlang_v2_types::RecordFieldTypeV1::try_new(
            "function",
            ValueTypeV1::scalar(ScalarTyV1::EntityRef),
            location.clone(),
        )
        .map_err(|error| {
            document_failure(
                "sema.invalid_item_shape",
                source_id,
                graph.span,
                error.to_string(),
            )
        })?,
    ])
    .map_err(|error| {
        document_failure(
            "sema.invalid_item_shape",
            source_id,
            graph.span,
            error.to_string(),
        )
    })?;
    let output = RelationOutputContractV2::try_new(
        row,
        OutputProvenanceV2::default(),
        vec![OutputColumnContractV2::new(
            "function".to_owned(),
            OutputColumnOriginV2::Generated(resolved.item_id()),
            location,
        )],
        quanta_gqlang_v2_types::CardinalityV1::Set,
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(|error| document_failure("sema.invalid_item_shape", source_id, graph.span, error))?;
    Ok((resolved, Some(output)))
}

type CompositionBindingRowsV2 = (
    Vec<ResolvedCompositionBindingV2>,
    BTreeMap<(HirNodeIdV1, usize), RelationOutputContractV2>,
);

fn materialize_composition_bindings(
    composition: &quanta_gqlang_v2_hir::CompositionHirV1,
    item_id: SemanticItemIdV2,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    facts: &dyn TypeFactsReaderV3,
    extended: &ExtendedSourceRegistryViewV2,
    views: &ViewMaterializationInputsV2<'_>,
    expressions: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &mut SemanticStepCounterV2,
) -> Result<CompositionBindingRowsV2, FrontendTransactionErrorV2> {
    let mut outputs = BTreeMap::new();
    let mut bindings: Vec<ResolvedCompositionBindingV2> =
        Vec::with_capacity(composition.bindings.len());
    for (ordinal, authored) in composition.bindings.iter().enumerate() {
        let proof = bound
            .composition_binding(composition.id, ordinal)
            .ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    authored.name_span,
                    "Binder omitted exact Composition binding proof",
                )
            })?;
        if proof.identity() != (composition.id, ordinal)
            || proof.kind != BinderKindV1::CompositionLet
            || proof.name != authored.name
            || proof.span != authored.name_span
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                authored.name_span,
                "Binder Composition binding proof differs from checked HIR",
            )
            .into());
        }
        let binding = ResolvedValueBindingV2::try_new(
            source_id,
            proof.node,
            proof.identity().1,
            ResolvedBindingKindV2::Composition,
            proof.name.clone(),
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                authored.name_span,
                error,
            )
        })?;
        let name_location = extended
            .combined()
            .try_check_location(location(source_id, authored.name_span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    authored.name_span,
                    error.to_string(),
                )
            })?;
        let composition_graph_targets: BTreeMap<
            (HirNodeIdV1, usize),
            ResolvedCompositionBindingV2,
        > = bindings
            .iter()
            .map(|binding| {
                (
                    (binding.binding().target_node(), binding.binding().ordinal()),
                    binding.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let (producer, output, producer_id) = match &authored.producer {
            quanta_gqlang_v2_hir::CompositionProducerV1::Query(query) => {
                let typed = materialize_query(
                    query,
                    source_id,
                    authority,
                    bound,
                    facts,
                    extended,
                    &outputs,
                    views,
                    expressions,
                    nested_relations,
                    steps,
                )?;
                let output = typed.output().clone();
                let producer_id = typed.item_id();
                (
                    ResolvedCompositionProducerV2::Query(typed),
                    Some(output),
                    producer_id,
                )
            }
            quanta_gqlang_v2_hir::CompositionProducerV1::Sample(sample) => {
                let (presentation, output) = materialize_composition_sample(
                    sample,
                    source_id,
                    authority,
                    bound,
                    &composition_graph_targets,
                    extended,
                    steps,
                )?;
                let producer_id = presentation.item_id();
                let producer =
                    ResolvedCompositionProducerV2::try_sample(presentation, output.clone())
                        .map_err(|error| {
                            document_failure(
                                "sema.invalid_item_shape",
                                source_id,
                                sample.span,
                                error,
                            )
                        })?;
                (producer, Some(output), producer_id)
            }
            quanta_gqlang_v2_hir::CompositionProducerV1::GraphQuery(graph) => {
                let graph_span = graph.span;
                let (graph, output) = materialize_composition_graph(
                    graph,
                    source_id,
                    authority,
                    bound,
                    &composition_graph_targets,
                    extended,
                    steps,
                )?;
                let producer_id = graph.item_id();
                let producer = ResolvedCompositionProducerV2::try_graph(graph, output.clone())
                    .map_err(|error| {
                        document_failure("sema.invalid_item_shape", source_id, graph_span, error)
                    })?;
                (producer, output, producer_id)
            }
        };
        let resolved = ResolvedCompositionBindingV2::try_new(
            item_id,
            ordinal,
            producer_id,
            binding,
            name_location,
            producer,
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                authored.name_span,
                error,
            )
        })?;
        if let Some(output) = output {
            if outputs.insert((composition.id, ordinal), output).is_some() {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    authored.name_span,
                    "duplicate Composition producer identity",
                )
                .into());
            }
        }
        bindings.push(resolved);
    }
    Ok((bindings, outputs))
}

pub(super) fn materialize_query_composition(
    composition: &quanta_gqlang_v2_hir::CompositionHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    facts: &dyn TypeFactsReaderV3,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    views: &ViewMaterializationInputsV2<'_>,
    expressions: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedCompositionV2, FrontendTransactionErrorV2> {
    let item_id = SemanticItemIdV2::new(source_id, composition.id);
    let item_location = extended
        .combined()
        .try_check_location(location(source_id, composition.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                composition.span,
                error.to_string(),
            )
        })?;
    let (bindings, outputs) = materialize_composition_bindings(
        composition,
        item_id,
        source_id,
        authority,
        bound,
        facts,
        extended,
        views,
        expressions,
        nested_relations,
        steps,
    )?;
    let composition_graph_targets = bindings
        .iter()
        .map(|binding| {
            (
                (binding.binding().target_node(), binding.binding().ordinal()),
                binding.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let (body, expected_body_id) = match &composition.body {
        quanta_gqlang_v2_hir::CompositionBodyV1::Query(query) => {
            let body = materialize_query(
                query,
                source_id,
                authority,
                bound,
                facts,
                extended,
                &outputs,
                views,
                expressions,
                nested_relations,
                steps,
            )?;
            let expected_body_id = body.item_id();
            (
                ResolvedCompositionBodyV2::Query(Box::new(body)),
                expected_body_id,
            )
        }
        quanta_gqlang_v2_hir::CompositionBodyV1::GraphQuery(graph) => {
            let body = resolve_graph_item(
                graph,
                source_id,
                authority,
                bound,
                &composition_graph_targets,
                extended,
                steps,
            )?;
            let expected_body_id = body.item_id();
            (ResolvedCompositionBodyV2::Graph(body), expected_body_id)
        }
        quanta_gqlang_v2_hir::CompositionBodyV1::Legality(check) => {
            let body = resolve_check_item(
                check,
                source_id,
                authority,
                bound,
                &composition_graph_targets,
                extended,
                validator,
                steps,
            )?;
            let expected_body_id = body.item_id();
            (
                ResolvedCompositionBodyV2::Check(Box::new(body)),
                expected_body_id,
            )
        }
        quanta_gqlang_v2_hir::CompositionBodyV1::Explain(explain) => {
            let body = resolve_composition_explain(
                explain,
                source_id,
                authority,
                bound,
                &composition_graph_targets,
                extended,
                validator,
                steps,
            )?;
            let expected_body_id = body.item_id();
            (ResolvedCompositionBodyV2::Explain(body), expected_body_id)
        }
    };
    let with_proof = bound
        .target_of_use(
            composition.id,
            composition.with_name_span,
            &composition.with_name,
        )
        .ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                composition.with_name_span,
                "Binder omitted exact Composition WITH target",
            )
        })?;
    let ResolvedTargetV1::Local(with_proof) = with_proof else {
        return Err(document_failure(
            "sema.invalid_item_shape",
            source_id,
            composition.with_name_span,
            "Composition WITH target is not local",
        )
        .into());
    };
    if with_proof.kind != BinderKindV1::CompositionLet {
        return Err(document_failure(
            "sema.invalid_item_shape",
            source_id,
            composition.with_name_span,
            "Composition WITH target is not a LET binding",
        )
        .into());
    }
    let with_target = ResolvedValueBindingV2::try_new(
        source_id,
        with_proof.node,
        with_proof.identity().1,
        ResolvedBindingKindV2::Composition,
        with_proof.name.clone(),
    )
    .map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            composition.with_name_span,
            error,
        )
    })?;
    let with_location = extended
        .combined()
        .try_check_location(location(source_id, composition.with_name_span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                composition.with_name_span,
                error.to_string(),
            )
        })?;
    ResolvedCompositionV2::try_new(
        item_id,
        item_location,
        bindings,
        expected_body_id,
        body,
        with_target,
        with_location,
    )
    .map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            composition.span,
            error,
        )
    })
    .map_err(Into::into)
}
