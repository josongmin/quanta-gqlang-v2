//! Projection from sealed semantic and planning owners into execution DTOs.

use super::{
    project_call_target, project_cardinality, project_exists_polarity, project_join_mode,
    project_logical_quantifier, project_null_order, project_operator, project_sort_direction,
    project_value_type, CheckContractV2, CheckedAlgebraNodeV2, CompareMetricV1,
    CompositionGraphCapabilityV2, ConfirmModeV1, DedupIdentityV1, DetectionModeV1, EmitModeV1,
    EvidenceModeV1, ExecutionBindingV3, ExecutionCatalogIdentityV3, ExecutionCheckV3,
    ExecutionClosureFailureV3, ExecutionCompareMetricV3, ExecutionCompositionBindingV3,
    ExecutionCompositionBodyV3, ExecutionCompositionProducerV3, ExecutionCompositionV3,
    ExecutionConfirmModeV3, ExecutionCorrelationV3, ExecutionDetectionModeV3, ExecutionEmitModeV3,
    ExecutionEvidenceModeV3, ExecutionExistentialTermV3, ExecutionExplainTargetV3,
    ExecutionExportFormatV3, ExecutionExpressionShapeV3, ExecutionExpressionV3, ExecutionFamilyV3,
    ExecutionFieldV3, ExecutionFiniteFloatV3, ExecutionGraphCapabilityV3,
    ExecutionGraphDedupIdentityV3, ExecutionGraphDirectionV3, ExecutionGraphKindV3,
    ExecutionGraphQueryV3, ExecutionGraphReferenceKindV3, ExecutionGraphReferenceRoleV3,
    ExecutionGraphReferenceV3, ExecutionGraphRelationShapeV3, ExecutionGraphTargetV3,
    ExecutionGraphWitnessTieBreakV3, ExecutionItemV3, ExecutionLiteralV3, ExecutionNestedGraphV3,
    ExecutionOrderKeyV3, ExecutionPresentationKindV3, ExecutionRelationAuthorityV3,
    ExecutionRelationV3, ExecutionRelationalOperatorV3, ExecutionSampleModeV3, ExecutionSchemaV3,
    ExecutionShowCategoryV3, ExecutionSourceKindV3, ExecutionSourceV3, ExecutionTruthModeV3,
    ExecutionValueTypeV3, ExportFormatV1, ExpressionTypeStateV2, GraphDirectionV2,
    GraphQueryContractV2, GraphQueryKindV1, GraphReferenceKindV2, GraphReferenceRoleV2,
    LoweredRelationItemV2, PlanPolicyV2, PresentationContractV2, RelationAuthorityV2,
    RelationOutputContractV2, RelationShapeV1, ResolvedCompositionBodyV2,
    ResolvedCompositionProducerV2, ResolvedCompositionV2, ResolvedExplainTargetV2,
    ResolvedGraphTargetV2, ResolvedGraphUseV2, ResolvedPresentationKindV2,
    ResolvedRelationSourceKindV2, SampleModeV1, SemanticDocumentV2, SemanticItemIdV2,
    SemanticPlanV2, ShowCategoryV1, SourceIdV2, SourceQualifiedNodeIdV2, TruthModeV1,
    TypedCallArgumentV2, TypedExprShapeV2, TypedExprV2, TypedLiteralFactV2, TypedLiteralValueV2,
    WitnessTieBreakV1,
};

pub fn project_relation(
    document: &SemanticDocumentV2,
    lowered: &LoweredRelationItemV2,
) -> Result<ExecutionRelationV3, ExecutionClosureFailureV3> {
    project_relation_root(document, lowered.item_id(), lowered.root())
}

pub fn project_relation_root(
    document: &SemanticDocumentV2,
    item_id: SemanticItemIdV2,
    root: &CheckedAlgebraNodeV2,
) -> Result<ExecutionRelationV3, ExecutionClosureFailureV3> {
    let projected = project_node(root);
    let mut roots = Vec::new();
    collect_operator_roots(&projected, &mut roots);
    let closed = close_expression_ids(item_id.source_id(), &roots, &|key| {
        document
            .expression(key)
            .map(|expr| shape_children(expr.shape()))
    })?;
    let mut table = Vec::with_capacity(closed.len());
    for key in closed {
        // `close_expression_ids` resolved every key through this same table.
        let Some(expr) = document.expression(key) else {
            return Err(ExecutionClosureFailureV3::Dangling);
        };
        table.push(project_expression(expr)?);
    }
    Ok(ExecutionRelationV3 {
        item_id,
        root: projected,
        expressions: table.into_boxed_slice(),
    })
}

/// Every expression identity the operator tree references directly: predicate
/// and projection keys, correlation references, schema partition keys, and
/// order-key expressions, recursively through children.
fn collect_operator_roots(
    node: &ExecutionRelationalOperatorV3,
    roots: &mut Vec<SourceQualifiedNodeIdV2>,
) {
    roots.extend_from_slice(&node.predicates);
    roots.extend_from_slice(&node.expressions);
    for edge in &node.correlations {
        roots.push(edge.reference);
    }
    for term in &node.existential_terms {
        roots.push(term.expression);
        roots.extend(term.correlations.iter().map(|edge| edge.reference));
    }
    roots.extend_from_slice(&node.schema.partition_keys);
    for key in &node.schema.order {
        roots.push(key.expression);
    }
    for child in &node.children {
        collect_operator_roots(child, roots);
    }
}

/// Child expression identities of one checked shape. Binding shapes carry no
/// expression children; the `RelationExists` relation root is a relation key,
/// not an expression identity, so only its correlation references close.
fn shape_children(shape: &TypedExprShapeV2) -> Vec<SourceQualifiedNodeIdV2> {
    match shape {
        TypedExprShapeV2::Literal(_) | TypedExprShapeV2::Binding(_) => Vec::new(),
        TypedExprShapeV2::Path { base, .. } => vec![*base],
        TypedExprShapeV2::Unary { operand, .. } | TypedExprShapeV2::Not { operand, .. } => {
            vec![*operand]
        }
        TypedExprShapeV2::Binary { lhs, rhs, .. }
        | TypedExprShapeV2::Compare { lhs, rhs, .. }
        | TypedExprShapeV2::And { lhs, rhs, .. }
        | TypedExprShapeV2::Or { lhs, rhs, .. } => vec![*lhs, *rhs],
        TypedExprShapeV2::Call { arguments, .. } => arguments
            .iter()
            .filter_map(|argument| match argument {
                TypedCallArgumentV2::Expression(id) => Some(*id),
                TypedCallArgumentV2::Star => None,
            })
            .collect(),
        TypedExprShapeV2::Paren { inner } | TypedExprShapeV2::BoolValue { value: inner } => {
            vec![*inner]
        }
        TypedExprShapeV2::Record(fields) => fields.iter().map(|field| field.value()).collect(),
        TypedExprShapeV2::List(items) => items.to_vec(),
        TypedExprShapeV2::LogicalQuantifier { body, .. } => vec![*body],
        TypedExprShapeV2::RelationExists { correlations, .. } => {
            correlations.iter().map(|edge| edge.reference()).collect()
        }
    }
}

/// Transitively close expression identities over a caller-supplied child
/// lookup. The result is deduplicated and sorted in stable identity order, so
/// unrelated table rows never affect an item's contract. A dangling identity
/// or an identity from another source fails closed.
pub(super) fn close_expression_ids(
    source_id: SourceIdV2,
    roots: &[SourceQualifiedNodeIdV2],
    children_of: &dyn Fn(SourceQualifiedNodeIdV2) -> Option<Vec<SourceQualifiedNodeIdV2>>,
) -> Result<Vec<SourceQualifiedNodeIdV2>, ExecutionClosureFailureV3> {
    let mut closed = std::collections::BTreeSet::new();
    let mut stack: Vec<SourceQualifiedNodeIdV2> = roots.to_vec();
    while let Some(key) = stack.pop() {
        if key.source_id != source_id {
            return Err(ExecutionClosureFailureV3::Foreign);
        }
        if !closed.insert(key) {
            continue;
        }
        let Some(children) = children_of(key) else {
            return Err(ExecutionClosureFailureV3::Dangling);
        };
        stack.extend(children);
    }
    Ok(closed.into_iter().collect())
}

/// Exactly one checked execution payload for one Composition producer.
///
/// This tagged projection prevents relation/graph payload arrays from drifting
/// out of alignment with the sealed binding order.
pub enum ExecutionCompositionProducerProjectionV3 {
    Relation(ExecutionRelationV3),
    Sample,
    Graph {
        item_id: SemanticItemIdV2,
        query: ExecutionGraphQueryV3,
    },
}

/// Exactly one checked execution payload for the Composition body.
pub enum ExecutionCompositionBodyProjectionV3 {
    Relation(ExecutionRelationV3),
    Graph {
        item_id: SemanticItemIdV2,
        query: ExecutionGraphQueryV3,
    },
    Check,
    Explain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionCompositionProjectionFailureV3 {
    Producer,
    Body,
}

pub fn project_composition(
    composition: &ResolvedCompositionV2,
    producer_projections: Vec<ExecutionCompositionProducerProjectionV3>,
    body_projection: ExecutionCompositionBodyProjectionV3,
) -> Result<ExecutionCompositionV3, ExecutionCompositionProjectionFailureV3> {
    if producer_projections.len() != composition.bindings().len() {
        return Err(ExecutionCompositionProjectionFailureV3::Producer);
    }
    let bindings = composition
        .bindings()
        .iter()
        .zip(producer_projections)
        .map(|(entry, projection)| {
            let (family, relation, producer) = match (entry.producer(), projection) {
                (
                    ResolvedCompositionProducerV2::Query(_),
                    ExecutionCompositionProducerProjectionV3::Relation(relation),
                ) if relation.item_id() == entry.producer().item_id() => (
                    ExecutionFamilyV3::Relation,
                    Some(relation),
                    ExecutionCompositionProducerV3::Relation,
                ),
                (
                    ResolvedCompositionProducerV2::Sample { presentation, .. },
                    ExecutionCompositionProducerProjectionV3::Sample,
                ) => {
                    let ResolvedPresentationKindV2::Sample {
                        query_text,
                        mode,
                        limit,
                        scope,
                    } = presentation.kind()
                    else {
                        return Err(ExecutionCompositionProjectionFailureV3::Producer);
                    };
                    (
                        ExecutionFamilyV3::Presentation,
                        None,
                        ExecutionCompositionProducerV3::Sample {
                            query_text: query_text.clone(),
                            mode: project_sample_mode(*mode),
                            limit: *limit,
                            scope: scope.as_ref().map(project_graph_use),
                        },
                    )
                }
                (
                    ResolvedCompositionProducerV2::Graph { .. },
                    ExecutionCompositionProducerProjectionV3::Graph { item_id, query },
                ) if item_id == entry.producer().item_id() => (
                    ExecutionFamilyV3::Graph,
                    None,
                    ExecutionCompositionProducerV3::Graph { query },
                ),
                _ => return Err(ExecutionCompositionProjectionFailureV3::Producer),
            };
            Ok(ExecutionCompositionBindingV3 {
                binding: project_binding(entry.binding()),
                producer_item_id: entry.producer().item_id(),
                producer_family: family,
                schema: entry.relation_output().map(project_schema),
                relation,
                producer,
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    let (body_item_id, body_family, body_relation, body) =
        match (composition.body(), body_projection) {
            (
                ResolvedCompositionBodyV2::Query(body),
                ExecutionCompositionBodyProjectionV3::Relation(relation),
            ) if relation.item_id() == body.item_id() => (
                body.item_id(),
                ExecutionFamilyV3::Relation,
                Some(relation),
                ExecutionCompositionBodyV3::Relation,
            ),
            (
                ResolvedCompositionBodyV2::Graph(body),
                ExecutionCompositionBodyProjectionV3::Graph { item_id, query },
            ) if item_id == body.item_id() => (
                body.item_id(),
                ExecutionFamilyV3::Graph,
                None,
                ExecutionCompositionBodyV3::Graph(query),
            ),
            (
                ResolvedCompositionBodyV2::Check(body),
                ExecutionCompositionBodyProjectionV3::Check,
            ) => (
                body.item_id(),
                ExecutionFamilyV3::Check,
                None,
                ExecutionCompositionBodyV3::Check,
            ),
            (
                ResolvedCompositionBodyV2::Explain(body),
                ExecutionCompositionBodyProjectionV3::Explain,
            ) => (
                body.item_id(),
                ExecutionFamilyV3::Presentation,
                None,
                ExecutionCompositionBodyV3::Explain,
            ),
            _ => return Err(ExecutionCompositionProjectionFailureV3::Body),
        };
    Ok(ExecutionCompositionV3 {
        bindings,
        with_binding: project_binding(composition.with_target()),
        body_item_id,
        body_family,
        body_relation,
        body,
    })
}

pub fn project_graph_query(contract: &GraphQueryContractV2) -> ExecutionGraphQueryV3 {
    ExecutionGraphQueryV3 {
        kind: project_graph_kind(contract.kind()),
        references: contract
            .references()
            .iter()
            .map(|reference| ExecutionGraphReferenceV3 {
                role: project_graph_reference_role(reference.role()),
                target: project_graph_target(reference.target()),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        depth: contract.depth(),
        direction: contract.direction().map(project_graph_direction),
        summary_facets: contract.summary_facets().to_vec().into_boxed_slice(),
        relation_shape: project_graph_relation_shape(contract.shape()),
        dedup_identity: project_graph_dedup_identity(contract.dedup()),
        max_witnesses: contract.max_witnesses(),
        max_path_length: contract.witness().max_path_length(),
        witness_tie_break: project_graph_witness_tie_break(contract.tie_break()),
    }
}

pub fn project_check(contract: &CheckContractV2) -> ExecutionCheckV3 {
    let policy = contract.policy();
    ExecutionCheckV3 {
        rule: project_catalog_identity(policy.rule()),
        scope: policy.scope().map(project_graph_use),
        pattern: policy.pattern().map(project_graph_use),
        nested_graph: contract.nested_graph().map(|graph| ExecutionNestedGraphV3 {
            item_id: graph.item_id(),
            query: project_graph_query(graph),
        }),
        confirm: match policy.confirm() {
            ConfirmModeV1::Skip => ExecutionConfirmModeV3::Skip,
            ConfirmModeV1::Prompt => ExecutionConfirmModeV3::Prompt,
            ConfirmModeV1::Require => ExecutionConfirmModeV3::Require,
        },
        emit: match policy.emit() {
            EmitModeV1::Findings => ExecutionEmitModeV3::Findings,
            EmitModeV1::Sarif => ExecutionEmitModeV3::Sarif,
        },
        mode: match policy.mode() {
            DetectionModeV1::PatternOnly => ExecutionDetectionModeV3::PatternOnly,
            DetectionModeV1::SemanticOnly => ExecutionDetectionModeV3::SemanticOnly,
            DetectionModeV1::Hybrid => ExecutionDetectionModeV3::Hybrid,
        },
    }
}

fn project_catalog_identity(
    reference: &quanta_gqlang_v2_sema::ResolvedCatalogReferenceV2,
) -> ExecutionCatalogIdentityV3 {
    ExecutionCatalogIdentityV3 {
        module: reference.id().module_id().as_str().to_owned(),
        name: reference.id().local_name().to_owned(),
    }
}

pub fn project_presentation(contract: &PresentationContractV2) -> ExecutionPresentationKindV3 {
    match contract.kind() {
        ResolvedPresentationKindV2::Explain {
            target,
            evidence,
            truth,
        } => ExecutionPresentationKindV3::Explain {
            target: match target {
                ResolvedExplainTargetV2::Rule(rule) => {
                    ExecutionExplainTargetV3::Rule(project_catalog_identity(rule))
                }
                ResolvedExplainTargetV2::Entity(entity) => {
                    ExecutionExplainTargetV3::Entity(project_graph_use(entity))
                }
            },
            evidence: match evidence {
                EvidenceModeV1::Auto => ExecutionEvidenceModeV3::Auto,
                EvidenceModeV1::Trace => ExecutionEvidenceModeV3::Trace,
                EvidenceModeV1::Citation => ExecutionEvidenceModeV3::Citation,
                EvidenceModeV1::Counterexample => ExecutionEvidenceModeV3::Counterexample,
            },
            truth: match truth {
                TruthModeV1::Inspect => ExecutionTruthModeV3::Inspect,
                TruthModeV1::Confirm => ExecutionTruthModeV3::Confirm,
                TruthModeV1::Refute => ExecutionTruthModeV3::Refute,
            },
        },
        ResolvedPresentationKindV2::Show { category } => project_show_category_v2(*category),
        ResolvedPresentationKindV2::Describe { target } => ExecutionPresentationKindV3::Describe {
            target: project_graph_use(target),
        },
        ResolvedPresentationKindV2::Sample {
            query_text,
            mode,
            limit,
            scope,
        } => ExecutionPresentationKindV3::Sample {
            query_text: query_text.clone(),
            mode: project_sample_mode(*mode),
            limit: *limit,
            scope: scope.as_ref().map(project_graph_use),
        },
        ResolvedPresentationKindV2::Compare {
            baseline,
            candidate,
            metric,
        } => ExecutionPresentationKindV3::Compare {
            baseline: project_graph_use(baseline),
            candidate: project_graph_use(candidate),
            metric: match metric {
                CompareMetricV1::Exact => ExecutionCompareMetricV3::Exact,
                CompareMetricV1::Structural => ExecutionCompareMetricV3::Structural,
                CompareMetricV1::Semantic => ExecutionCompareMetricV3::Semantic,
            },
        },
        ResolvedPresentationKindV2::Export {
            source,
            format,
            profile,
        } => ExecutionPresentationKindV3::Export {
            source: project_graph_use(source),
            format: match format {
                ExportFormatV1::Json => ExecutionExportFormatV3::Json,
                ExportFormatV1::Csv => ExecutionExportFormatV3::Csv,
                ExportFormatV1::Sarif => ExecutionExportFormatV3::Sarif,
                ExportFormatV1::Markdown => ExecutionExportFormatV3::Markdown,
                ExportFormatV1::GraphMl => ExecutionExportFormatV3::GraphMl,
            },
            profile: profile.as_ref().map(project_catalog_identity),
        },
    }
}

fn project_show_category_v2(category: ShowCategoryV1) -> ExecutionPresentationKindV3 {
    ExecutionPresentationKindV3::Show {
        category: match category {
            ShowCategoryV1::Capabilities => ExecutionShowCategoryV3::Capabilities,
            ShowCategoryV1::Models => ExecutionShowCategoryV3::Models,
            ShowCategoryV1::Views => ExecutionShowCategoryV3::Views,
            ShowCategoryV1::Rules => ExecutionShowCategoryV3::Rules,
            ShowCategoryV1::Packs => ExecutionShowCategoryV3::Packs,
            ShowCategoryV1::Profiles => ExecutionShowCategoryV3::Profiles,
            ShowCategoryV1::Gates => ExecutionShowCategoryV3::Gates,
        },
    }
}

/// Project one checked SHOW plan without allocating a generic execution
/// payload. The caller admits its document item vector before invoking this
/// producer; category and plan policy must agree exactly.
pub(crate) fn project_selected_show_item_v2(
    resolved: &quanta_gqlang_v2_sema::ResolvedPresentationV2,
    plan: &SemanticPlanV2,
) -> Result<ExecutionItemV3, &'static str> {
    let ResolvedPresentationKindV2::Show { category } = resolved.kind() else {
        return Err("execution.selected_show_wrong_semantic_family");
    };
    let PlanPolicyV2::Presentation(ResolvedPresentationKindV2::Show {
        category: planned_category,
    }) = plan.policy()
    else {
        return Err("execution.selected_show_wrong_plan_policy");
    };
    if category != planned_category
        || plan.explain().root_location() != resolved.location()
        || plan.authority_lane() != quanta_gqlang_v2_planning::AuthorityLaneV2::FrontendChecked
        || plan.effect() != quanta_gqlang_v2_planning::EffectClassV2::ReadOnly
        || plan.result().result_kind() != quanta_gqlang_v2_planning::ResultKindV2::Table
        || plan.evidence() != quanta_gqlang_v2_planning::EvidenceRequirementV2::None
    {
        return Err("execution.selected_show_plan_semantic_mismatch");
    }
    Ok(ExecutionItemV3::new(
        resolved.item_id(),
        ExecutionFamilyV3::Presentation,
        None,
        None,
        None,
        Some(project_show_category_v2(*category)),
        None,
        plan,
    ))
}

const fn project_graph_relation_shape(shape: RelationShapeV1) -> ExecutionGraphRelationShapeV3 {
    match shape {
        RelationShapeV1::Acyclic => ExecutionGraphRelationShapeV3::Acyclic,
        RelationShapeV1::Cyclic => ExecutionGraphRelationShapeV3::Cyclic,
    }
}

const fn project_graph_dedup_identity(identity: DedupIdentityV1) -> ExecutionGraphDedupIdentityV3 {
    match identity {
        DedupIdentityV1::Endpoints => ExecutionGraphDedupIdentityV3::Endpoints,
        DedupIdentityV1::NodeSequence => ExecutionGraphDedupIdentityV3::NodeSequence,
        DedupIdentityV1::NodeSet => ExecutionGraphDedupIdentityV3::NodeSet,
    }
}

const fn project_graph_witness_tie_break(
    tie_break: WitnessTieBreakV1,
) -> ExecutionGraphWitnessTieBreakV3 {
    match tie_break {
        WitnessTieBreakV1::ShortestThenLexical => {
            ExecutionGraphWitnessTieBreakV3::ShortestThenLexical
        }
        WitnessTieBreakV1::Lexical => ExecutionGraphWitnessTieBreakV3::Lexical,
    }
}

fn project_graph_use(graph_use: &ResolvedGraphUseV2) -> ExecutionGraphTargetV3 {
    project_graph_target(graph_use.target())
}

fn project_graph_target(target: &ResolvedGraphTargetV2) -> ExecutionGraphTargetV3 {
    match target {
        ResolvedGraphTargetV2::Host(selector) => ExecutionGraphTargetV3::Host {
            kind: project_graph_reference_kind(selector.kind()),
            spelling: selector.spelling().to_owned(),
            entity_subtype: selector.entity_subtype().map(str::to_owned),
        },
        ResolvedGraphTargetV2::Composition(target) => ExecutionGraphTargetV3::Composition {
            binding: project_binding(target.source().binding()),
            capability: match target.capability() {
                CompositionGraphCapabilityV2::Entity => ExecutionGraphCapabilityV3::Entity,
                CompositionGraphCapabilityV2::Scope => ExecutionGraphCapabilityV3::Scope,
            },
        },
    }
}

const fn project_graph_reference_role(role: GraphReferenceRoleV2) -> ExecutionGraphReferenceRoleV3 {
    match role {
        GraphReferenceRoleV2::Source => ExecutionGraphReferenceRoleV3::Source,
        GraphReferenceRoleV2::Sink => ExecutionGraphReferenceRoleV3::Sink,
        GraphReferenceRoleV2::Anchor => ExecutionGraphReferenceRoleV3::Anchor,
        GraphReferenceRoleV2::Reference => ExecutionGraphReferenceRoleV3::Reference,
        GraphReferenceRoleV2::Pointee => ExecutionGraphReferenceRoleV3::Pointee,
        GraphReferenceRoleV2::Callable => ExecutionGraphReferenceRoleV3::Callable,
        GraphReferenceRoleV2::Via => ExecutionGraphReferenceRoleV3::Via,
        GraphReferenceRoleV2::Through => ExecutionGraphReferenceRoleV3::Through,
        GraphReferenceRoleV2::Sanitizer => ExecutionGraphReferenceRoleV3::Sanitizer,
        GraphReferenceRoleV2::Scope => ExecutionGraphReferenceRoleV3::Scope,
        GraphReferenceRoleV2::Window => ExecutionGraphReferenceRoleV3::Window,
        GraphReferenceRoleV2::Frontier => ExecutionGraphReferenceRoleV3::Frontier,
    }
}

const fn project_graph_reference_kind(kind: GraphReferenceKindV2) -> ExecutionGraphReferenceKindV3 {
    match kind {
        GraphReferenceKindV2::Entity => ExecutionGraphReferenceKindV3::Entity,
        GraphReferenceKindV2::Callable => ExecutionGraphReferenceKindV3::Callable,
        GraphReferenceKindV2::Scope => ExecutionGraphReferenceKindV3::Scope,
        GraphReferenceKindV2::Window => ExecutionGraphReferenceKindV3::Window,
        GraphReferenceKindV2::Frontier => ExecutionGraphReferenceKindV3::Frontier,
        GraphReferenceKindV2::Sanitizer => ExecutionGraphReferenceKindV3::Sanitizer,
        GraphReferenceKindV2::Pattern => ExecutionGraphReferenceKindV3::Pattern,
        GraphReferenceKindV2::Snapshot => ExecutionGraphReferenceKindV3::Snapshot,
        GraphReferenceKindV2::ExportSource => ExecutionGraphReferenceKindV3::ExportSource,
    }
}

const fn project_graph_direction(direction: GraphDirectionV2) -> ExecutionGraphDirectionV3 {
    match direction {
        GraphDirectionV2::Forward => ExecutionGraphDirectionV3::Forward,
        GraphDirectionV2::Backward => ExecutionGraphDirectionV3::Backward,
        GraphDirectionV2::Both => ExecutionGraphDirectionV3::Both,
        GraphDirectionV2::Undirected => ExecutionGraphDirectionV3::Undirected,
    }
}

const fn project_sample_mode(mode: SampleModeV1) -> ExecutionSampleModeV3 {
    match mode {
        SampleModeV1::Lexical => ExecutionSampleModeV3::Lexical,
        SampleModeV1::Semantic => ExecutionSampleModeV3::Semantic,
        SampleModeV1::Hybrid => ExecutionSampleModeV3::Hybrid,
    }
}
const fn project_graph_kind(kind: GraphQueryKindV1) -> ExecutionGraphKindV3 {
    match kind {
        GraphQueryKindV1::Reachability => ExecutionGraphKindV3::Reachability,
        GraphQueryKindV1::Flow => ExecutionGraphKindV3::Flow,
        GraphQueryKindV1::Slice => ExecutionGraphKindV3::Slice,
        GraphQueryKindV1::Taint => ExecutionGraphKindV3::Taint,
        GraphQueryKindV1::Path => ExecutionGraphKindV3::Path,
        GraphQueryKindV1::PointsTo => ExecutionGraphKindV3::PointsTo,
        GraphQueryKindV1::Summary => ExecutionGraphKindV3::Summary,
    }
}

fn project_node(node: &CheckedAlgebraNodeV2) -> ExecutionRelationalOperatorV3 {
    let source = node.scan_source().map(project_source);
    let predicates = node
        .predicate()
        .into_iter()
        .map(|p| p.expression().key())
        .chain(node.filter_conjuncts().iter().map(|p| p.expression().key()))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let expressions = node
        .expressions()
        .iter()
        .map(TypedExprV2::key)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let correlations = node
        .correlations()
        .into_iter()
        .flat_map(|set| set.iter())
        .map(|edge| ExecutionCorrelationV3 {
            binding: project_binding(edge.binding()),
            reference: edge.reference(),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let existential_terms = node
        .existential_terms()
        .iter()
        .map(|term| ExecutionExistentialTermV3 {
            expression: term.expression(),
            polarity: project_exists_polarity(term.polarity()),
            correlations: term
                .correlations()
                .iter()
                .map(|edge| ExecutionCorrelationV3 {
                    binding: project_binding(edge.binding()),
                    reference: edge.reference(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let partition_input = node
        .partition_input()
        .iter()
        .map(|entry| project_binding(entry.binding()))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let children = node
        .children()
        .into_iter()
        .map(project_node)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    ExecutionRelationalOperatorV3 {
        family: node.family(),
        join_mode: node.join_mode().map(project_join_mode),
        limit: node.limit_count(),
        stage: node.stage_key(),
        source,
        schema: project_schema(node.output()),
        predicates,
        expressions,
        correlations,
        existential_terms,
        partition_input,
        children,
    }
}

pub(super) fn project_expression(
    expr: &TypedExprV2,
) -> Result<ExecutionExpressionV3, ExecutionClosureFailureV3> {
    let value_type = match expr.type_state() {
        ExpressionTypeStateV2::Known(value) => project_value_type(value),
        ExpressionTypeStateV2::Unknown(_) => ExecutionValueTypeV3::Unknown,
    };
    Ok(ExecutionExpressionV3 {
        id: expr.key(),
        value_type,
        shape: project_expression_shape(expr.shape())?,
    })
}

pub(super) fn project_expression_shape(
    shape: &TypedExprShapeV2,
) -> Result<ExecutionExpressionShapeV3, ExecutionClosureFailureV3> {
    Ok(match shape {
        TypedExprShapeV2::Literal(fact) => ExecutionExpressionShapeV3::Literal(match fact {
            TypedLiteralFactV2::DeferredI128MinMagnitude => ExecutionLiteralV3::Int(i128::MIN),
            TypedLiteralFactV2::Value(value) => match value {
                TypedLiteralValueV2::Int(value) => ExecutionLiteralV3::Int(*value),
                TypedLiteralValueV2::Float(value) => {
                    ExecutionLiteralV3::Float(ExecutionFiniteFloatV3(value.get().to_bits()))
                }
                TypedLiteralValueV2::Bool(value) => ExecutionLiteralV3::Bool(*value),
                TypedLiteralValueV2::String(value) => ExecutionLiteralV3::String(value.clone()),
                TypedLiteralValueV2::Null => ExecutionLiteralV3::Null,
            },
        }),
        TypedExprShapeV2::Binding(binding) => {
            ExecutionExpressionShapeV3::Binding(project_binding(binding))
        }
        TypedExprShapeV2::Path { base, segments } => ExecutionExpressionShapeV3::Path {
            base: *base,
            segments: segments
                .iter()
                .map(|s| s.name().to_owned())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        },
        TypedExprShapeV2::Unary { operator, operand } => ExecutionExpressionShapeV3::Unary {
            operator: project_operator(*operator)?,
            operand: *operand,
        },
        TypedExprShapeV2::Binary { operator, lhs, rhs } => ExecutionExpressionShapeV3::Binary {
            operator: project_operator(*operator)?,
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::Call { target, arguments } => ExecutionExpressionShapeV3::Call {
            target: project_call_target(target)?,
            arguments: arguments
                .iter()
                .map(|arg| match arg {
                    TypedCallArgumentV2::Expression(id) => Some(*id),
                    TypedCallArgumentV2::Star => None,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        },
        TypedExprShapeV2::Paren { inner } => ExecutionExpressionShapeV3::Paren(*inner),
        TypedExprShapeV2::Record(fields) => ExecutionExpressionShapeV3::Record(
            fields
                .iter()
                .map(|field| (field.name().to_owned(), field.value()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        TypedExprShapeV2::List(items) => {
            ExecutionExpressionShapeV3::List(items.clone().into_boxed_slice())
        }
        TypedExprShapeV2::BoolValue { value } => ExecutionExpressionShapeV3::BoolValue(*value),
        TypedExprShapeV2::Compare { operator, lhs, rhs } => ExecutionExpressionShapeV3::Compare {
            operator: project_operator(*operator)?,
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::And { operator, lhs, rhs } => ExecutionExpressionShapeV3::And {
            operator: project_operator(*operator)?,
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::Or { operator, lhs, rhs } => ExecutionExpressionShapeV3::Or {
            operator: project_operator(*operator)?,
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::Not { operator, operand } => ExecutionExpressionShapeV3::Not {
            operator: project_operator(*operator)?,
            operand: *operand,
        },
        TypedExprShapeV2::LogicalQuantifier {
            kind,
            binders,
            body,
        } => ExecutionExpressionShapeV3::LogicalQuantifier {
            kind: project_logical_quantifier(*kind),
            binders: binders
                .iter()
                .map(project_binding)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            body: *body,
        },
        TypedExprShapeV2::RelationExists {
            polarity,
            relation_root,
            correlations,
        } => ExecutionExpressionShapeV3::RelationExists {
            polarity: project_exists_polarity(*polarity),
            relation_root: *relation_root,
            correlations: correlations
                .iter()
                .map(|edge| ExecutionCorrelationV3 {
                    binding: project_binding(edge.binding()),
                    reference: edge.reference(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        },
    })
}

pub(super) fn project_schema(output: &RelationOutputContractV2) -> ExecutionSchemaV3 {
    let fields = output
        .row()
        .fields()
        .iter()
        .map(|field| ExecutionFieldV3 {
            name: field.name().to_owned(),
            value_type: project_value_type(field.value_type()),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let cardinality = project_cardinality(output.cardinality());
    let order = output
        .ordering()
        .iter()
        .map(|key| ExecutionOrderKeyV3 {
            expression: key.expression(),
            direction: project_sort_direction(key.direction()),
            null_order: project_null_order(key.null_order()),
            value_type: project_value_type(key.value_type()),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    ExecutionSchemaV3 {
        fields,
        cardinality,
        partition_keys: output.partition_keys().to_vec().into_boxed_slice(),
        order,
    }
}

pub(super) fn project_source(source: &ResolvedRelationSourceKindV2) -> ExecutionSourceV3 {
    match source {
        ResolvedRelationSourceKindV2::Relation(id) => ExecutionSourceV3 {
            host_binding_identity: id.host_binding_identity(),
            authority: match id.authority() {
                RelationAuthorityV2::Host => ExecutionRelationAuthorityV3::Host,
                RelationAuthorityV2::CanonicalWorkspaceFiles => {
                    ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles
                }
            },
            kind: ExecutionSourceKindV3::Relation,
            namespace: Some(id.namespace().as_str().to_owned()),
            name: id.qualified_name().to_owned(),
            composition_binding: None,
        },
        ResolvedRelationSourceKindV2::View(id) => ExecutionSourceV3 {
            host_binding_identity: None,
            authority: ExecutionRelationAuthorityV3::CatalogView,
            kind: ExecutionSourceKindV3::View,
            namespace: Some(id.module_id().as_str().to_owned()),
            name: id.local_name().to_owned(),
            composition_binding: None,
        },
        ResolvedRelationSourceKindV2::Composition(id) => ExecutionSourceV3 {
            host_binding_identity: None,
            authority: ExecutionRelationAuthorityV3::CompositionBinding,
            kind: ExecutionSourceKindV3::Composition,
            namespace: None,
            name: id.binding().canonical_name().to_owned(),
            composition_binding: Some(project_binding(id.binding())),
        },
    }
}

fn project_binding(binding: &quanta_gqlang_v2_sema::ResolvedValueBindingV2) -> ExecutionBindingV3 {
    ExecutionBindingV3 {
        source_id: binding.source_id(),
        owner_node: binding.target_node().raw(),
        ordinal: binding.ordinal(),
        name: binding.canonical_name().to_owned(),
    }
}
