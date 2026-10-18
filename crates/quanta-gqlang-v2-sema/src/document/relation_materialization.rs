//! Typed relation node construction from the checked relation walk.

use super::{
    document_failure, location, materialize_bool, materialize_declaration_expr,
    take_materialized_relation, AggregatePositionV2, AggregationClassV1, BTreeMap,
    BinderEvidenceReaderV3, BinderKindV1, ClosedFrontendAuthorityV2, ExpressionEffectsV2,
    ExpressionTypeStateV2, ExtendedSourceRegistryViewV2, FrontendFailureV2, HirNodeIdV1,
    NullOrderV2, OrderKeyContractV2, OutputColumnContractV2, OutputColumnOriginV2,
    OutputProvenanceV2, PartitionInputBindingV2, RelationOutputContractV2, ResolvedBindingKindV2,
    ResolvedCompositionSourceIdV2, ResolvedScanSourceV2, ResolvedTargetV1, ResolvedValueBindingV2,
    SemanticStepCounterV2, SourceIdV2, SourceQualifiedNodeIdV2, TypeFactsReaderV3, TypedExprV2,
    TypedRelationExprV2, ViewMaterializationInputsV2, ViewSourceTargetV2,
};

pub(super) fn materialize_host_source_v3(
    relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
    checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
    key: SourceQualifiedNodeIdV2,
    source: &quanta_gqlang_v2_hir::RelationSourceV1,
    authority: &ClosedFrontendAuthorityV2,
    source_id: SourceIdV2,
    request_single: bool,
    mut columns: Vec<OutputColumnContractV2>,
) -> Result<TypedRelationExprV2, FrontendFailureV2> {
    let host = authority.relations().get(&source.name).ok_or_else(|| {
        document_failure(
            "sema.unresolved_reference",
            source_id,
            relation.span,
            "relation source is absent from the sealed host catalog",
        )
    })?;
    let scan = ResolvedScanSourceV2::Relation(host.id().clone());
    for field in host.relation_type().row().fields() {
        columns.push(OutputColumnContractV2::new(
            field.name().to_owned(),
            OutputColumnOriginV2::SourceField {
                source: scan.clone(),
                scan_stage: key,
                field_name: field.name().to_owned(),
            },
            host.location().clone(),
        ));
    }
    let output = RelationOutputContractV2::try_new(
        host.relation_type().row().clone(),
        if request_single {
            OutputProvenanceV2::from_single_source_v3(scan.clone())
        } else {
            OutputProvenanceV2::new([scan.clone()])
        },
        columns,
        host.relation_type().cardinality(),
        Vec::new(),
        Vec::new(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            relation.span,
            error,
        )
    })?;
    TypedRelationExprV2::scan(key, checked, scan, output).map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            relation.span,
            error,
        )
    })
}

pub(super) struct ScalarProjectPartsV3 {
    pub input: TypedRelationExprV2,
    pub row: quanta_gqlang_v2_types::RecordTypeV1,
    pub columns: Vec<OutputColumnContractV2>,
    pub keys: Vec<SourceQualifiedNodeIdV2>,
    pub effects: Vec<ExpressionEffectsV2>,
}

pub(super) fn seal_scalar_project_v3(
    relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
    checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
    key: SourceQualifiedNodeIdV2,
    parts: ScalarProjectPartsV3,
    source_id: SourceIdV2,
) -> Result<TypedRelationExprV2, FrontendFailureV2> {
    let ScalarProjectPartsV3 {
        input,
        row,
        columns,
        keys,
        effects,
    } = parts;
    let previous = input.output();
    let output = RelationOutputContractV2::try_new(
        row,
        previous.provenance().clone(),
        columns,
        previous.cardinality(),
        Vec::new(),
        Vec::new(),
        previous.effects().compose(effects),
    )
    .map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            relation.span,
            error,
        )
    })?;
    TypedRelationExprV2::try_project(key, checked, input, keys, output).map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            relation.span,
            error,
        )
    })
}

struct RelationMaterializer<'a, 'view> {
    source_id: SourceIdV2,
    authority: &'a ClosedFrontendAuthorityV2,
    bound: &'a dyn BinderEvidenceReaderV3,
    facts: &'a dyn TypeFactsReaderV3,
    extended: &'a ExtendedSourceRegistryViewV2,
    composition_outputs: &'a BTreeMap<(HirNodeIdV1, usize), RelationOutputContractV2>,
    views: &'a ViewMaterializationInputsV2<'view>,
    expressions: &'a mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &'a mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &'a mut SemanticStepCounterV2,
    completed: &'a mut BTreeMap<HirNodeIdV1, TypedRelationExprV2>,
}

struct ProjectedColumns {
    columns: Vec<OutputColumnContractV2>,
    keys: Vec<SourceQualifiedNodeIdV2>,
    effects: Vec<ExpressionEffectsV2>,
    aggregate_classes: Vec<AggregationClassV1>,
}

pub(super) fn materialize_relation_node(
    relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
    checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
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
    completed: &mut BTreeMap<HirNodeIdV1, TypedRelationExprV2>,
) -> Result<TypedRelationExprV2, FrontendFailureV2> {
    RelationMaterializer {
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
        completed,
    }
    .materialize(relation, checked)
}

impl RelationMaterializer<'_, '_> {
    fn materialize(
        &mut self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
    ) -> Result<TypedRelationExprV2, FrontendFailureV2> {
        let key = SourceQualifiedNodeIdV2 {
            source_id: self.source_id,
            node_id: relation.id,
        };
        match &relation.kind {
            quanta_gqlang_v2_hir::RelationExprKindV1::Source(source) => {
                self.materialize_source(relation, checked, key, source)
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::Filter(filter) => {
                self.materialize_filter(relation, checked, key, filter)
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) => {
                self.materialize_project(relation, checked, key, project)
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::Join(join) => {
                self.materialize_join(relation, checked, key, join)
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::Limit(limit) => {
                let input =
                    take_materialized_relation(self.completed, &limit.input, self.source_id)?;
                TypedRelationExprV2::try_limit(key, checked, input, limit.count).map_err(
                    |message| {
                        document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            relation.span,
                            message,
                        )
                    },
                )
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::OrderBy(order) => {
                self.materialize_orderby(relation, checked, key, order)
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(group) => {
                self.materialize_groupby(relation, checked, key, group)
            }
        }
    }
    fn materialize_source(
        &self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
        key: SourceQualifiedNodeIdV2,
        source: &quanta_gqlang_v2_hir::RelationSourceV1,
    ) -> Result<TypedRelationExprV2, FrontendFailureV2> {
        if let Some(view) =
            super::InlineViewPlanV2::source_target_in(self.views.source_targets, relation.id)
        {
            return self.materialize_view_source(relation, checked, key, source, view);
        }
        if let Some(target) = self
            .bound
            .target_of_use(relation.id, source.name_span, &source.name)
        {
            return self.materialize_composition_source(relation, checked, key, source, target);
        }
        self.materialize_host_source(relation, checked, key, source)
    }

    fn materialize_view_source(
        &self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
        key: SourceQualifiedNodeIdV2,
        source: &quanta_gqlang_v2_hir::RelationSourceV1,
        view: &ViewSourceTargetV2,
    ) -> Result<TypedRelationExprV2, FrontendFailureV2> {
        let source_id = self.source_id;
        let facts = self.facts;
        let extended = self.extended;
        let views = self.views;
        let (scan, expected_row, cardinality, partition_keys, ordering, effects) = match view {
            ViewSourceTargetV2::Inline(view) => {
                let previous = views.outputs.get(view).ok_or_else(|| {
                    document_failure(
                        "sema.view_materialization_order",
                        source_id,
                        source.name_span,
                        "View source was not available in Modules dependency-first order",
                    )
                })?;
                (
                    ResolvedScanSourceV2::View(view.clone()),
                    previous.row(),
                    previous.cardinality(),
                    previous.partition_keys().to_vec(),
                    previous.ordering().to_vec(),
                    previous.effects(),
                )
            }
            ViewSourceTargetV2::External {
                asset,
                relation_type,
            } => (
                ResolvedScanSourceV2::View(asset.clone()),
                relation_type.row(),
                relation_type.cardinality(),
                Vec::new(),
                Vec::new(),
                ExpressionEffectsV2::pure_scalar(),
            ),
        };
        let checked_row = facts.relation_row(key).ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "types owner omitted View scan row fact",
            )
        })?;
        if checked_row != expected_row {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "View scan row differs from the sealed typed View contract",
            ));
        }
        let use_location = extended
            .combined()
            .try_check_location(location(source_id, source.name_span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    source.name_span,
                    error.to_string(),
                )
            })?;
        let columns = expected_row
            .fields()
            .iter()
            .map(|field| {
                OutputColumnContractV2::new(
                    field.name().to_owned(),
                    OutputColumnOriginV2::SourceField {
                        source: scan.clone(),
                        scan_stage: key,
                        field_name: field.name().to_owned(),
                    },
                    use_location.clone(),
                )
            })
            .collect();
        let output = RelationOutputContractV2::try_new(
            checked_row.clone(),
            OutputProvenanceV2::new([scan.clone()]),
            columns,
            cardinality,
            partition_keys,
            ordering,
            effects,
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })?;
        TypedRelationExprV2::scan(key, checked, scan, output).map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })
    }

    fn materialize_composition_source(
        &self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
        key: SourceQualifiedNodeIdV2,
        source: &quanta_gqlang_v2_hir::RelationSourceV1,
        target: &ResolvedTargetV1,
    ) -> Result<TypedRelationExprV2, FrontendFailureV2> {
        let source_id = self.source_id;
        let facts = self.facts;
        let extended = self.extended;
        let composition_outputs = self.composition_outputs;
        let ResolvedTargetV1::Local(binding) = target else {
            return Err(document_failure(
                "sema.invalid_relation_source_kind",
                source_id,
                source.name_span,
                "resolved relation source is not a local Composition binding",
            ));
        };
        if binding.kind != BinderKindV1::CompositionLet {
            return Err(document_failure(
                "sema.invalid_relation_source_kind",
                source_id,
                source.name_span,
                "resolved local source is not a Composition relation binding",
            ));
        }
        let identity = binding.identity();
        let previous = composition_outputs.get(&identity).ok_or_else(|| {
            document_failure(
                "sema.non_relation_composition_source",
                source_id,
                source.name_span,
                "Composition binding has no prior sealed relation output",
            )
        })?;
        let checked_row = facts.relation_row(key).ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "types owner omitted Composition-local scan row fact",
            )
        })?;
        if checked_row != previous.row() {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "Composition-local scan row differs from the prior sealed producer",
            ));
        }
        let composition_binding = ResolvedValueBindingV2::try_new(
            source_id,
            identity.0,
            identity.1,
            ResolvedBindingKindV2::Composition,
            binding.name.clone(),
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                source.name_span,
                error,
            )
        })?;
        let scan = ResolvedScanSourceV2::Composition(
            ResolvedCompositionSourceIdV2::try_from_binding(composition_binding).map_err(
                |error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        source.name_span,
                        error,
                    )
                },
            )?,
        );
        let use_location = extended
            .combined()
            .try_check_location(location(source_id, source.name_span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    source.name_span,
                    error.to_string(),
                )
            })?;
        let columns = previous
            .row()
            .fields()
            .iter()
            .map(|field| {
                OutputColumnContractV2::new(
                    field.name().to_owned(),
                    OutputColumnOriginV2::SourceField {
                        source: scan.clone(),
                        scan_stage: key,
                        field_name: field.name().to_owned(),
                    },
                    use_location.clone(),
                )
            })
            .collect();
        let output = RelationOutputContractV2::try_new(
            checked_row.clone(),
            OutputProvenanceV2::new([scan.clone()]),
            columns,
            previous.cardinality(),
            previous.partition_keys().to_vec(),
            previous.ordering().to_vec(),
            previous.effects(),
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })?;
        TypedRelationExprV2::scan(key, checked, scan, output).map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })
    }

    fn materialize_host_source(
        &self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
        key: SourceQualifiedNodeIdV2,
        source: &quanta_gqlang_v2_hir::RelationSourceV1,
    ) -> Result<TypedRelationExprV2, FrontendFailureV2> {
        materialize_host_source_v3(
            relation,
            checked,
            key,
            source,
            self.authority,
            self.source_id,
            false,
            Vec::new(),
        )
    }
    fn materialize_filter(
        &mut self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
        key: SourceQualifiedNodeIdV2,
        filter: &quanta_gqlang_v2_hir::RelationFilterV1,
    ) -> Result<TypedRelationExprV2, FrontendFailureV2> {
        let source_id = self.source_id;
        let authority = self.authority;
        let bound = self.bound;
        let facts = self.facts;
        let extended = self.extended;
        let composition_outputs = self.composition_outputs;
        let views = self.views;
        let expressions = &mut *self.expressions;
        let nested_relations = &mut *self.nested_relations;
        let steps = &mut *self.steps;
        let completed = &mut *self.completed;
        let input = take_materialized_relation(completed, &filter.input, source_id)?;
        let predicate = materialize_bool(
            &filter.predicate,
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
            AggregatePositionV2::Predicate,
        )?;
        let previous = input.output();
        let output = RelationOutputContractV2::try_new(
            previous.row().clone(),
            previous.provenance().clone(),
            previous.columns().to_vec(),
            previous.cardinality(),
            previous.partition_keys().to_vec(),
            previous.ordering().to_vec(),
            previous
                .effects()
                .compose([predicate.expression().effects()]),
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })?;
        TypedRelationExprV2::filter(key, checked, input, predicate, output).map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })
    }
    fn materialize_project(
        &mut self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
        key: SourceQualifiedNodeIdV2,
        project: &quanta_gqlang_v2_hir::ProjectionV1,
    ) -> Result<TypedRelationExprV2, FrontendFailureV2> {
        let source_id = self.source_id;
        let input = take_materialized_relation(self.completed, &project.input, source_id)?;
        let ProjectedColumns {
            columns,
            keys,
            effects,
            aggregate_classes,
        } = self.materialize_project_columns(relation, project)?;
        let facts = self.facts;
        let steps = &mut *self.steps;
        let row = facts.relation_row(key).cloned().ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "types owner omitted projection row fact",
            )
        })?;
        let previous = input.output();
        // One aggregate classification over the complete projection
        // expression vector, via each expression's sealed
        // `AggregateContextV2`. Mixed and illegal shapes reject here,
        // before any aggregate-ready stage is constructed.
        let aggregate = aggregate_classes.contains(&AggregationClassV1::Aggregate);
        if !aggregate {
            return seal_scalar_project_v3(
                relation,
                checked,
                key,
                ScalarProjectPartsV3 {
                    input,
                    row,
                    columns,
                    keys,
                    effects,
                },
                source_id,
            );
        }
        let composed = previous.effects().compose(effects);
        if let TypedRelationExprV2::Group(_) = &input {
            // Explicit authored Group child: preserve its exact
            // partition-key IDs and partition-input proof. No second
            // Group is synthesized.
            let partition_keys = input.output().partition_keys().to_vec();
            let output = RelationOutputContractV2::try_aggregate(
                row,
                previous.provenance().clone(),
                columns,
                partition_keys.clone(),
                composed,
                &partition_keys,
            )
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })?;
            return TypedRelationExprV2::try_aggregate_project(key, checked, input, keys, output)
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        relation.span,
                        error,
                    )
                });
        }
        // Scalar (global) aggregate over a non-Group input. Every column
        // must be aggregate-composed; a plain column cannot coexist with
        // a global fold.
        if let Some(plain) = aggregate_classes
            .iter()
            .position(|class| *class != AggregationClassV1::Aggregate)
        {
            return Err(document_failure(
                "sema.mixed_aggregate_projection",
                source_id,
                project
                    .items()
                    .get(plain)
                    .map_or(relation.span, |item| item.value.span),
                "global aggregate projection cannot retain a non-aggregated column",
            ));
        }
        // One private Sema-owned empty-partition Group wraps the pre-fold
        // input; the aggregate Project consumes it directly.
        steps.charge(source_id, relation.span)?;
        let group =
            TypedRelationExprV2::implicit_group(key, checked.clone(), input).map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            })?;
        let output = RelationOutputContractV2::try_aggregate(
            row,
            group.output().provenance().clone(),
            columns,
            Vec::new(),
            composed,
            &[],
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })?;
        TypedRelationExprV2::try_aggregate_project(key, checked, group, keys, output).map_err(
            |error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            },
        )
    }
    fn materialize_project_columns(
        &mut self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        project: &quanta_gqlang_v2_hir::ProjectionV1,
    ) -> Result<ProjectedColumns, FrontendFailureV2> {
        let source_id = self.source_id;
        let bound = self.bound;
        let facts = self.facts;
        let extended = self.extended;
        let expressions = &mut *self.expressions;
        let steps = &mut *self.steps;
        let frame = bound.relation_output(relation.id).ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "binder omitted projection output frame",
            )
        })?;
        if frame.len() != project.items().len() {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "binder projection output differs from authored item count",
            ));
        }
        let mut columns = Vec::new();
        let mut keys = Vec::new();
        let mut effects = Vec::new();
        let mut aggregate_classes = Vec::new();
        for (ordinal, (item, binding)) in project.items().iter().zip(frame).enumerate() {
            if binding.identity() != (relation.id, ordinal) {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    item.value.span,
                    "projection output does not retain its Binder-owned ordinal",
                ));
            }
            let value = materialize_declaration_expr(
                &item.value,
                source_id,
                bound,
                facts,
                extended,
                expressions,
                steps,
                AggregatePositionV2::Projection,
            )?;
            let ExpressionTypeStateV2::Known(_) = value.type_state() else {
                return Err(document_failure(
                    "sema.unknown_type_in_closed_compile",
                    source_id,
                    item.value.span,
                    "projection field has no concrete value type",
                ));
            };
            let name_span = item
                .alias
                .as_ref()
                .map_or(item.value.span, |alias| alias.span);
            let field_location = extended
                .combined()
                .try_check_location(location(source_id, name_span))
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        name_span,
                        error.to_string(),
                    )
                })?;
            columns.push(OutputColumnContractV2::new(
                binding.name.clone(),
                OutputColumnOriginV2::Expression(value.key()),
                field_location,
            ));
            aggregate_classes.push(value.aggregate_context().aggregation());
            keys.push(value.key());
            effects.push(value.effects());
        }
        Ok(ProjectedColumns {
            columns,
            keys,
            effects,
            aggregate_classes,
        })
    }

    fn materialize_join(
        &mut self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
        key: SourceQualifiedNodeIdV2,
        join: &quanta_gqlang_v2_hir::JoinHirV1,
    ) -> Result<TypedRelationExprV2, FrontendFailureV2> {
        let source_id = self.source_id;
        let authority = self.authority;
        let bound = self.bound;
        let facts = self.facts;
        let extended = self.extended;
        let composition_outputs = self.composition_outputs;
        let views = self.views;
        let expressions = &mut *self.expressions;
        let nested_relations = &mut *self.nested_relations;
        let steps = &mut *self.steps;
        let completed = &mut *self.completed;
        let (left_node, right_node, predicate_node) = match join {
            quanta_gqlang_v2_hir::JoinHirV1::Inner { left, right, on } => (left, right, Some(on)),
            quanta_gqlang_v2_hir::JoinHirV1::Cross { left, right } => (left, right, None),
        };
        let left = take_materialized_relation(completed, left_node, source_id)?;
        let right = take_materialized_relation(completed, right_node, source_id)?;
        let predicate = predicate_node
            .map(|node| {
                materialize_bool(
                    node,
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
                    AggregatePositionV2::Predicate,
                )
            })
            .transpose()?;
        let left_output = left.output();
        let right_output = right.output();
        let mut columns = Vec::new();
        for (field, column) in left_output
            .row()
            .fields()
            .iter()
            .zip(left_output.columns())
            .chain(
                right_output
                    .row()
                    .fields()
                    .iter()
                    .zip(right_output.columns()),
            )
        {
            let _ = field;
            columns.push(column.clone());
        }
        let row = facts.relation_row(key).cloned().ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "types owner omitted join row fact",
            )
        })?;
        let provenance = OutputProvenanceV2::new(
            left_output
                .provenance()
                .iter()
                .cloned()
                .chain(right_output.provenance().iter().cloned()),
        );
        let effects = ExpressionEffectsV2::pure_scalar().compose([
            left_output.effects(),
            right_output.effects(),
            predicate
                .as_ref()
                .map_or(ExpressionEffectsV2::pure_scalar(), |on| {
                    on.expression().effects()
                }),
        ]);
        let output = RelationOutputContractV2::try_new(
            row,
            provenance,
            columns,
            quanta_gqlang_v2_types::CardinalityV1::Bag,
            Vec::new(),
            Vec::new(),
            effects,
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })?;
        match predicate {
            Some(on) => TypedRelationExprV2::join(key, checked, left, right, on, output),
            None => TypedRelationExprV2::cross(key, checked, left, right, output),
        }
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })
    }
    fn materialize_orderby(
        &mut self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
        key: SourceQualifiedNodeIdV2,
        order: &quanta_gqlang_v2_hir::OrderByHirV1,
    ) -> Result<TypedRelationExprV2, FrontendFailureV2> {
        let source_id = self.source_id;
        let bound = self.bound;
        let facts = self.facts;
        let extended = self.extended;
        let expressions = &mut *self.expressions;
        let steps = &mut *self.steps;
        let completed = &mut *self.completed;
        let input = take_materialized_relation(completed, &order.input, source_id)?;
        let mut keys = Vec::new();
        let mut ordering = Vec::new();
        let mut effects = Vec::new();
        for item in order.keys() {
            let value = materialize_declaration_expr(
                &item.value,
                source_id,
                bound,
                facts,
                extended,
                expressions,
                steps,
                AggregatePositionV2::OrderKey,
            )?;
            if value.effects().aggregation == AggregationClassV1::Aggregate {
                return Err(document_failure(
                    "sema.illegal_aggregate_context",
                    source_id,
                    item.value.span,
                    "ORDER BY cannot introduce an aggregate outside the grouped output",
                ));
            }
            let ExpressionTypeStateV2::Known(ty) = value.type_state() else {
                return Err(document_failure(
                    "sema.unknown_type_in_closed_compile",
                    source_id,
                    item.value.span,
                    "order key has no concrete value type",
                ));
            };
            keys.push(value.key());
            ordering.push(OrderKeyContractV2::new(
                value.key(),
                item.direction.into(),
                NullOrderV2::Last,
                ty.clone(),
            ));
            effects.push(value.effects());
        }
        let previous = input.output();
        let output = RelationOutputContractV2::try_new(
            previous.row().clone(),
            previous.provenance().clone(),
            previous.columns().to_vec(),
            quanta_gqlang_v2_types::CardinalityV1::Sequence,
            previous.partition_keys().to_vec(),
            ordering,
            previous.effects().compose(effects),
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })?;
        TypedRelationExprV2::try_sort(key, checked, input, keys, output).map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })
    }
    fn materialize_groupby(
        &mut self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        checked: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
        key: SourceQualifiedNodeIdV2,
        group: &quanta_gqlang_v2_hir::GroupByHirV1,
    ) -> Result<TypedRelationExprV2, FrontendFailureV2> {
        let source_id = self.source_id;
        let bound = self.bound;
        let facts = self.facts;
        let extended = self.extended;
        let expressions = &mut *self.expressions;
        let steps = &mut *self.steps;
        let completed = &mut *self.completed;
        let input = take_materialized_relation(completed, &group.input, source_id)?;
        let frame = bound.relation_output(relation.id).ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "binder omitted group key output frame",
            )
        })?;
        if frame.len() != group.keys().len()
            || bound.partition_input(relation.id) != bound.relation_output(group.input.id)
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "group output and separately sealed partition input disagree with Binder",
            ));
        }
        let mut columns = Vec::new();
        let mut keys = Vec::new();
        let mut effects = Vec::new();
        for (ordinal, (key_expr, binding)) in group.keys().iter().zip(frame).enumerate() {
            if binding.identity() != (relation.id, ordinal) {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    key_expr.span,
                    "group key output lost its Binder-owned ordinal",
                ));
            }
            let value = materialize_declaration_expr(
                key_expr,
                source_id,
                bound,
                facts,
                extended,
                expressions,
                steps,
                AggregatePositionV2::GroupKey,
            )?;
            let ExpressionTypeStateV2::Known(_) = value.type_state() else {
                return Err(document_failure(
                    "sema.unknown_type_in_closed_compile",
                    source_id,
                    key_expr.span,
                    "group key has no concrete value type",
                ));
            };
            let field_location = extended
                .combined()
                .try_check_location(location(source_id, binding.span))
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        binding.span,
                        error.to_string(),
                    )
                })?;
            columns.push(OutputColumnContractV2::new(
                binding.name.clone(),
                OutputColumnOriginV2::Expression(value.key()),
                field_location,
            ));
            keys.push(value.key());
            effects.push(value.effects());
        }
        let row = facts.relation_row(key).cloned().ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "types owner omitted group row fact",
            )
        })?;
        let output = RelationOutputContractV2::try_new(
            row,
            input.output().provenance().clone(),
            columns,
            quanta_gqlang_v2_types::CardinalityV1::Set,
            keys.clone(),
            Vec::new(),
            input.output().effects().compose(effects),
        )
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                error,
            )
        })?;
        let partition_input = bound.partition_input(relation.id).ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                relation.span,
                "group has no Binder partition input",
            )
        })?;
        let partition_input = partition_input
            .iter()
            .map(|binding| {
                let (owner, ordinal) = binding.identity();
                let identity = ResolvedValueBindingV2::try_new(
                    source_id,
                    owner,
                    ordinal,
                    ResolvedBindingKindV2::Alias,
                    binding.name.clone(),
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        binding.span,
                        error,
                    )
                })?;
                let span = extended
                    .combined()
                    .try_check_location(location(source_id, binding.span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            binding.span,
                            error.to_string(),
                        )
                    })?;
                PartitionInputBindingV2::try_new(identity, span).map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        binding.span,
                        error,
                    )
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        TypedRelationExprV2::try_group(key, checked, input, keys, partition_input, output).map_err(
            |error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    relation.span,
                    error,
                )
            },
        )
    }
}
