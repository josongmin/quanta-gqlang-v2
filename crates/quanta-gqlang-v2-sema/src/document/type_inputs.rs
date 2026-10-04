//! Closed type inputs collected from bound document syntax.

use super::{
    document_failure, location, AggregationClassV1, BTreeMap, BTreeSet, BindResultV1,
    BinderEvidenceReaderV3, BinderKindV1, BoolExprHirV1, BoolExprKindV1, BuiltinEvidenceReaderV3,
    BuiltinRegistryV2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    DeferredRelationRowFieldInputV2, DeferredRelationRowInputV2, ExtendedSourceRegistryViewV2,
    FrontendFailureV2, HirNodeIdV1, HirVisitorV1, InlineViewPlanV2, ProgramItemHirV1,
    RelationRowFieldSourceV2, ResolvedCallTargetV2, ResolvedCallableInputV2,
    ResolvedRelationCaptureInputV2, ResolvedRelationPredicateInputV2, ResolvedTargetV1,
    ResolvedTypeInputEntryV2, ResolvedTypeInputValueV2, RuleBindingIdV2, RuleSymbolInputV2,
    ScalarTyV1, SourceIdV2, SourceQualifiedNodeIdV2, SpanV1, TypedExpressionFactsV2,
    ValueExprHirV1, ValueExprKindV1, ValueTypeV1, ViewSourceTargetV2,
};

pub(super) struct DeclarationTypeInputsV2<'a> {
    pub(super) source_id: SourceIdV2,
    pub(super) bound: &'a dyn BinderEvidenceReaderV3,
    pub(super) declaration_bodies: &'a BTreeMap<HirNodeIdV1, HirNodeIdV1>,
    pub(super) builtins: &'a dyn BuiltinEvidenceReaderV3,
    pub(super) entries: Vec<ResolvedTypeInputEntryV2>,
    pub(super) error: Option<FrontendFailureV2>,
}

pub(super) struct RuleSymbolInputsV2<'a> {
    pub(super) source_id: SourceIdV2,
    pub(super) bound: &'a dyn BinderEvidenceReaderV3,
    pub(super) symbols: Vec<RuleSymbolInputV2>,
    pub(super) calls: Vec<(SourceQualifiedNodeIdV2, String, SpanV1)>,
    pub(super) error: Option<FrontendFailureV2>,
}

impl HirVisitorV1 for RuleSymbolInputsV2<'_> {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        match &node.kind {
            ValueExprKindV1::Symbol(_) => match self.bound.target_of(node.id) {
                Some(ResolvedTargetV1::Local(binding)) => {
                    let (owner, ordinal) = binding.identity();
                    self.symbols.push(RuleSymbolInputV2 {
                        reference: SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: node.id,
                        },
                        binding: RuleBindingIdV2::new(self.source_id, owner, ordinal),
                    });
                }
                _ if self.error.is_none() => {
                    self.error = Some(document_failure(
                        "sema.unresolved_binding_type",
                        self.source_id,
                        node.span,
                        "rule symbol is not an exact local Binder binding",
                    ))
                }
                _ => {}
            },
            ValueExprKindV1::Call(call) => self.calls.push((
                SourceQualifiedNodeIdV2 {
                    source_id: self.source_id,
                    node_id: node.id,
                },
                call.callee.name.clone(),
                node.span,
            )),
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => {}
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
}

/// Read-only witness of one Binder capture edge. This is comparison data for
/// the private cross-owner equality validator; it is never a public forgery
/// API and confers no authority by itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CaptureEdgeWitnessV2 {
    pub(super) reference: HirNodeIdV1,
    pub(super) owner: HirNodeIdV1,
    pub(super) ordinal: usize,
}

/// Independently comparable witness of one relational-EXISTS capture vector,
/// including the nested relation root it belongs to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CaptureVectorWitnessV2 {
    pub(super) relation_root: HirNodeIdV1,
    pub(super) edges: Vec<CaptureEdgeWitnessV2>,
}

/// Project one Binder capture set into the read-only witness compared by
/// [`validate_capture_vector`]. Binder remains the sole lexical authority.
pub(super) fn capture_vector_witness(
    bound: &dyn BinderEvidenceReaderV3,
    exists: HirNodeIdV1,
    relation_root: HirNodeIdV1,
) -> CaptureVectorWitnessV2 {
    CaptureVectorWitnessV2 {
        relation_root,
        edges: bound
            .capture_edges(exists)
            .iter()
            .map(|&(reference, owner, ordinal)| CaptureEdgeWitnessV2 {
                reference,
                owner,
                ordinal,
            })
            .collect(),
    }
}

/// Private cross-owner capture-vector equality validator. Sema alone performs
/// this comparison: the witness must equal Binder's complete ordered edge
/// vector for the exact EXISTS node, every edge must resolve to the exact
/// local `(owner, ordinal)` Binder binding, and the witnessed nested relation
/// root must equal the checked HIR root. Dropped, extra, duplicated,
/// reordered, transplanted, foreign, wrong-root, and omitted-capture-set
/// witnesses all reject with a distinct fail-closed reason.
pub(super) fn validate_capture_vector(
    exists_node: HirNodeIdV1,
    hir_relation_root: HirNodeIdV1,
    witness: &CaptureVectorWitnessV2,
    bound: &dyn BinderEvidenceReaderV3,
) -> Result<Vec<CaptureEdgeWitnessV2>, &'static str> {
    if !bound.has_capture_set(exists_node) {
        return Err("Binder omitted the explicit relational EXISTS capture set");
    }
    if witness.relation_root != hir_relation_root {
        return Err("capture witness nested relation root differs from checked HIR");
    }
    let mut references = BTreeSet::new();
    for edge in &witness.edges {
        if !references.insert(edge.reference) {
            return Err("duplicate capture reference in witness");
        }
    }
    let binder_edges = bound.capture_edges(exists_node);
    if binder_edges.len() != witness.edges.len() {
        return Err("capture vector dropped or added an edge");
    }
    for (&(binder_reference, binder_owner, binder_ordinal), edge) in
        binder_edges.iter().zip(&witness.edges)
    {
        if edge.reference != binder_reference {
            return Err("capture edge reference differs from Binder proof");
        }
        if edge.owner != binder_owner {
            return Err("capture edge owner differs from Binder proof");
        }
        if edge.ordinal != binder_ordinal {
            return Err("capture edge ordinal differs from Binder proof");
        }
        let Some(ResolvedTargetV1::Local(binding)) = bound.target_of(edge.reference) else {
            return Err("capture reference has no exact local resolution");
        };
        if binding.identity() != (edge.owner, edge.ordinal) {
            return Err("capture resolution identity disagrees with the witness");
        }
    }
    Ok(witness.edges.clone())
}

pub(super) struct QueryTypeInputsV2<'a> {
    pub(super) source_id: SourceIdV2,
    pub(super) authority: &'a ClosedFrontendAuthorityV2,
    pub(super) bound: &'a dyn BinderEvidenceReaderV3,
    pub(super) builtins: &'a dyn BuiltinEvidenceReaderV3,
    pub(super) extended: &'a ExtendedSourceRegistryViewV2,
    pub(super) view_source_targets: &'a InlineViewPlanV2,
    pub(super) view_relation_roots: &'a InlineViewPlanV2,
    pub(super) owners: BTreeMap<(HirNodeIdV1, usize), ResolvedTypeInputValueV2>,
    pub(super) entries: Vec<ResolvedTypeInputEntryV2>,
    pub(super) rows: Vec<DeferredRelationRowInputV2>,
    pub(super) composition_rows: BTreeMap<(HirNodeIdV1, usize), CompositionRelationRowV2>,
    pub(super) error: Option<FrontendFailureV2>,
}

/// The only relation row forms a prior Composition producer can expose to a
/// later authored `FROM` source. Generated rows are known at that consuming
/// source; they are never forged as a non-HIR relation root.
pub(super) enum CompositionRelationRowV2 {
    PriorRelation(SourceQualifiedNodeIdV2),
    Generated {
        field_name: &'static str,
        scalar: ScalarTyV1,
    },
    Absent,
}

impl QueryTypeInputsV2<'_> {
    fn visit_relation_source(
        &mut self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        source: &quanta_gqlang_v2_hir::RelationSourceV1,
    ) -> bool {
        let root = SourceQualifiedNodeIdV2 {
            source_id: self.source_id,
            node_id: relation.id,
        };
        if let Some(view) = self.view_source_targets.source_target_for(relation.id) {
            match view {
                ViewSourceTargetV2::Inline(view) => {
                    let Some(target_root) = self.view_relation_roots.relation_root_for(view) else {
                        if self.error.is_none() {
                            self.error = Some(document_failure(
                                "sema.unmaterialized_view_source",
                                self.source_id,
                                source.name_span,
                                "resolved View source has no local inline relation root",
                            ));
                        }
                        return false;
                    };
                    self.rows.push(DeferredRelationRowInputV2::prior_relation(
                        root,
                        target_root,
                    ));
                }
                ViewSourceTargetV2::External { relation_type, .. } => {
                    let checked = match self
                        .extended
                        .combined()
                        .try_check_location(location(self.source_id, source.name_span))
                    {
                        Ok(checked) => checked,
                        Err(error) => {
                            if self.error.is_none() {
                                self.error = Some(document_failure(
                                    "sema.document_validation_failed",
                                    self.source_id,
                                    source.name_span,
                                    error.to_string(),
                                ));
                            }
                            return false;
                        }
                    };
                    let fields = relation_type
                        .row()
                        .fields()
                        .iter()
                        .enumerate()
                        .map(|(ordinal, field)| {
                            DeferredRelationRowFieldInputV2::new(
                                ordinal,
                                field.name(),
                                checked.clone(),
                                RelationRowFieldSourceV2::Known(field.value_type().clone()),
                            )
                        })
                        .collect::<Vec<_>>();
                    self.rows.push(DeferredRelationRowInputV2::known_source(
                        root,
                        fields.len(),
                        fields,
                    ));
                }
            }
            if source.alias.is_some() {
                self.owners.insert(
                    (relation.id, 0),
                    ResolvedTypeInputValueV2::BindingFromRelationRow(root),
                );
            }
            return true;
        }
        if let Some(target) = self
            .bound
            .target_of_use(relation.id, source.name_span, &source.name)
        {
            match target {
                ResolvedTargetV1::Local(binding)
                    if binding.kind == BinderKindV1::CompositionLet =>
                {
                    let Some(producer_row) = self.composition_rows.get(&binding.identity()) else {
                        if self.error.is_none() {
                            self.error = Some(document_failure(
                                "sema.non_relation_composition_source",
                                self.source_id,
                                source.name_span,
                                "Composition binding has no relation row",
                            ));
                        }
                        return false;
                    };
                    match producer_row {
                        CompositionRelationRowV2::PriorRelation(producer_root) => self.rows.push(
                            DeferredRelationRowInputV2::prior_relation(root, *producer_root),
                        ),
                        CompositionRelationRowV2::Generated { field_name, scalar } => {
                            let checked = match self
                                .extended
                                .combined()
                                .try_check_location(location(self.source_id, source.name_span))
                            {
                                Ok(checked) => checked,
                                Err(error) => {
                                    if self.error.is_none() {
                                        self.error = Some(document_failure(
                                            "sema.document_validation_failed",
                                            self.source_id,
                                            source.name_span,
                                            error.to_string(),
                                        ));
                                    }
                                    return false;
                                }
                            };
                            self.rows.push(DeferredRelationRowInputV2::known_source(
                                root,
                                1,
                                vec![DeferredRelationRowFieldInputV2::new(
                                    0,
                                    *field_name,
                                    checked,
                                    RelationRowFieldSourceV2::Known(ValueTypeV1::scalar(
                                        scalar.clone(),
                                    )),
                                )],
                            ));
                        }
                        CompositionRelationRowV2::Absent => {
                            if self.error.is_none() {
                                self.error = Some(document_failure(
                                    "sema.non_relation_composition_source",
                                    self.source_id,
                                    source.name_span,
                                    "Composition binding has no relation row",
                                ));
                            }
                            return false;
                        }
                    }
                    if source.alias.is_some() {
                        self.owners.insert(
                            (relation.id, 0),
                            ResolvedTypeInputValueV2::BindingFromRelationRow(root),
                        );
                    }
                    return true;
                }
                ResolvedTargetV1::Local(_)
                | ResolvedTargetV1::Declaration { .. }
                | ResolvedTargetV1::Import { .. } => {
                    if self.error.is_none() {
                        self.error = Some(document_failure(
                            "sema.invalid_relation_source_kind",
                            self.source_id,
                            source.name_span,
                            "resolved source is not a Composition relation binding",
                        ));
                    }
                    return false;
                }
            }
        }
        self.visit_host_relation_source(relation, source, root)
    }

    fn visit_host_relation_source(
        &mut self,
        relation: &quanta_gqlang_v2_hir::RelationExprHirV1,
        source: &quanta_gqlang_v2_hir::RelationSourceV1,
        root: SourceQualifiedNodeIdV2,
    ) -> bool {
        let Some(host) = self.authority.relations().get(&source.name) else {
            if self.error.is_none() {
                self.error = Some(document_failure(
                    "sema.unresolved_reference",
                    self.source_id,
                    source.name_span,
                    "host relation source is absent",
                ));
            }
            return false;
        };
        let checked = match self
            .extended
            .combined()
            .try_check_location(location(self.source_id, source.name_span))
        {
            Ok(checked) => checked,
            Err(error) => {
                if self.error.is_none() {
                    self.error = Some(document_failure(
                        "sema.document_validation_failed",
                        self.source_id,
                        source.name_span,
                        error.to_string(),
                    ));
                }
                return false;
            }
        };
        let fields = host
            .relation_type()
            .row()
            .fields()
            .iter()
            .enumerate()
            .map(|(ordinal, field)| {
                DeferredRelationRowFieldInputV2::new(
                    ordinal,
                    field.name(),
                    checked.clone(),
                    RelationRowFieldSourceV2::Known(field.value_type().clone()),
                )
            })
            .collect::<Vec<_>>();
        self.rows.push(DeferredRelationRowInputV2::known_source(
            root,
            fields.len(),
            fields,
        ));
        if source.alias.is_some() {
            self.owners.insert(
                (relation.id, 0),
                ResolvedTypeInputValueV2::BindingFromRelationRow(root),
            );
        }
        true
    }
}

impl HirVisitorV1 for QueryTypeInputsV2<'_> {
    fn visit_relation(&mut self, relation: &quanta_gqlang_v2_hir::RelationExprHirV1) {
        match &relation.kind {
            quanta_gqlang_v2_hir::RelationExprKindV1::Source(source) => {
                if !self.visit_relation_source(relation, source) {
                    return;
                }
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) => {
                let Some(frame) = self.bound.relation_output(relation.id) else {
                    if self.error.is_none() {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            relation.span,
                            "Binder omitted projection output",
                        ));
                    }
                    return;
                };
                let mut fields = Vec::new();
                for (ordinal, (item, binding)) in project.items().iter().zip(frame).enumerate() {
                    self.owners.insert(
                        (relation.id, ordinal),
                        ResolvedTypeInputValueV2::BindingFromExpression(SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: item.value.id,
                        }),
                    );
                    let name_span = item
                        .alias
                        .as_ref()
                        .map_or(item.value.span, |alias| alias.span);
                    match self
                        .extended
                        .combined()
                        .try_check_location(location(self.source_id, name_span))
                    {
                        Ok(checked) => fields.push(DeferredRelationRowFieldInputV2::new(
                            ordinal,
                            binding.name.clone(),
                            checked,
                            RelationRowFieldSourceV2::PriorExpression(SourceQualifiedNodeIdV2 {
                                source_id: self.source_id,
                                node_id: item.value.id,
                            }),
                        )),
                        Err(error) => {
                            if self.error.is_none() {
                                self.error = Some(document_failure(
                                    "sema.document_validation_failed",
                                    self.source_id,
                                    name_span,
                                    error.to_string(),
                                ));
                            }
                        }
                    }
                }
                self.rows
                    .push(DeferredRelationRowInputV2::expression_fields(
                        SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: relation.id,
                        },
                        frame.len(),
                        fields,
                    ));
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(group) => {
                let Some(frame) = self.bound.relation_output(relation.id) else {
                    if self.error.is_none() {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            relation.span,
                            "Binder omitted group output",
                        ));
                    }
                    return;
                };
                let mut fields = Vec::new();
                for (ordinal, (key, binding)) in group.keys().iter().zip(frame).enumerate() {
                    self.owners.insert(
                        (relation.id, ordinal),
                        ResolvedTypeInputValueV2::BindingFromExpression(SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: key.id,
                        }),
                    );
                    match self
                        .extended
                        .combined()
                        .try_check_location(location(self.source_id, binding.span))
                    {
                        Ok(checked) => fields.push(DeferredRelationRowFieldInputV2::new(
                            ordinal,
                            binding.name.clone(),
                            checked,
                            RelationRowFieldSourceV2::PriorExpression(SourceQualifiedNodeIdV2 {
                                source_id: self.source_id,
                                node_id: key.id,
                            }),
                        )),
                        Err(error) => {
                            if self.error.is_none() {
                                self.error = Some(document_failure(
                                    "sema.document_validation_failed",
                                    self.source_id,
                                    binding.span,
                                    error.to_string(),
                                ));
                            }
                        }
                    }
                }
                self.rows
                    .push(DeferredRelationRowInputV2::expression_fields(
                        SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: relation.id,
                        },
                        frame.len(),
                        fields,
                    ));
            }
            quanta_gqlang_v2_hir::RelationExprKindV1::Filter(_)
            | quanta_gqlang_v2_hir::RelationExprKindV1::Join(_)
            | quanta_gqlang_v2_hir::RelationExprKindV1::OrderBy(_) => {}
        }
        quanta_gqlang_v2_hir::walk_relation(self, relation);
    }

    fn visit_composition(&mut self, node: &quanta_gqlang_v2_hir::CompositionHirV1) {
        for (ordinal, binding) in node.bindings.iter().enumerate() {
            let Some(proof) = self.bound.composition_binding(node.id, ordinal) else {
                if self.error.is_none() {
                    self.error = Some(document_failure(
                        "sema.document_validation_failed",
                        self.source_id,
                        binding.name_span,
                        "Binder omitted exact Composition binding proof",
                    ));
                }
                return;
            };
            if proof.kind != BinderKindV1::CompositionLet
                || proof.identity() != (node.id, ordinal)
                || proof.name != binding.name
                || proof.span != binding.name_span
            {
                if self.error.is_none() {
                    self.error = Some(document_failure(
                        "sema.document_validation_failed",
                        self.source_id,
                        binding.name_span,
                        "Binder Composition binding proof differs from checked HIR",
                    ));
                }
                return;
            }
            let identity = proof.identity();
            match &binding.producer {
                quanta_gqlang_v2_hir::CompositionProducerV1::Query(query) => {
                    self.visit_query(query);
                    if self
                        .composition_rows
                        .insert(
                            identity,
                            CompositionRelationRowV2::PriorRelation(SourceQualifiedNodeIdV2 {
                                source_id: self.source_id,
                                node_id: query.body.id,
                            }),
                        )
                        .is_some()
                        && self.error.is_none()
                    {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            binding.name_span,
                            "duplicate Composition binding identity",
                        ));
                    }
                }
                quanta_gqlang_v2_hir::CompositionProducerV1::Sample(_) => {
                    if self
                        .composition_rows
                        .insert(
                            identity,
                            CompositionRelationRowV2::Generated {
                                field_name: "path",
                                scalar: ScalarTyV1::RepoPath,
                            },
                        )
                        .is_some()
                        && self.error.is_none()
                    {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            binding.name_span,
                            "duplicate Composition binding identity",
                        ));
                    }
                }
                quanta_gqlang_v2_hir::CompositionProducerV1::GraphQuery(graph) => {
                    let row = if matches!(
                        graph.kind,
                        quanta_gqlang_v2_hir::GraphQueryKindV1::Path
                            | quanta_gqlang_v2_hir::GraphQueryKindV1::Reachability
                    ) {
                        CompositionRelationRowV2::Generated {
                            field_name: "function",
                            scalar: ScalarTyV1::EntityRef,
                        }
                    } else {
                        CompositionRelationRowV2::Absent
                    };
                    if self.composition_rows.insert(identity, row).is_some() && self.error.is_none()
                    {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            binding.name_span,
                            "duplicate Composition binding identity",
                        ));
                    }
                }
            }
        }
        match &node.body {
            quanta_gqlang_v2_hir::CompositionBodyV1::Query(query) => self.visit_query(query),
            quanta_gqlang_v2_hir::CompositionBodyV1::GraphQuery(_)
            | quanta_gqlang_v2_hir::CompositionBodyV1::Legality(_)
            | quanta_gqlang_v2_hir::CompositionBodyV1::Explain(_) => {}
        }
    }

    fn visit_bool(&mut self, node: &BoolExprHirV1) {
        if let BoolExprKindV1::RelationExists(exists) = &node.kind {
            if !self.bound.has_capture_set(node.id) {
                if self.error.is_none() {
                    self.error = Some(document_failure(
                        "sema.document_validation_failed",
                        self.source_id,
                        node.span,
                        "Binder omitted explicit relational EXISTS capture set",
                    ));
                }
                return;
            }
            let witness = capture_vector_witness(self.bound, node.id, exists.relation.id);
            let edges = validate_capture_vector(node.id, exists.relation.id, &witness, self.bound)
                .map_err(|message| {
                    if self.error.is_none() {
                        self.error = Some(document_failure(
                            "sema.document_validation_failed",
                            self.source_id,
                            node.span,
                            message,
                        ));
                    }
                });
            let Ok(edges) = edges else {
                return;
            };
            let captures = edges
                .into_iter()
                .map(|edge| {
                    ResolvedRelationCaptureInputV2::new(
                        SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: edge.reference,
                        },
                        SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: edge.owner,
                        },
                        edge.ordinal,
                    )
                })
                .collect::<Vec<_>>();
            match ResolvedRelationPredicateInputV2::try_resolved(
                SourceQualifiedNodeIdV2 {
                    source_id: self.source_id,
                    node_id: exists.relation.id,
                },
                captures,
            ) {
                Ok(input) => self.entries.push(ResolvedTypeInputEntryV2 {
                    key: SourceQualifiedNodeIdV2 {
                        source_id: self.source_id,
                        node_id: node.id,
                    },
                    value: ResolvedTypeInputValueV2::RelationPredicate(input),
                }),
                Err(error) => {
                    if self.error.is_none() {
                        self.error = Some(document_failure(
                            error.code(),
                            self.source_id,
                            node.span,
                            error.to_string(),
                        ));
                    }
                }
            }
        }
        quanta_gqlang_v2_hir::walk_bool(self, node);
    }

    fn visit_value(&mut self, node: &ValueExprHirV1) {
        let entry = match &node.kind {
            ValueExprKindV1::Symbol(_) => match self.bound.target_of(node.id) {
                Some(ResolvedTargetV1::Local(binding)) => {
                    self.owners.get(&binding.identity()).cloned()
                }
                Some(ResolvedTargetV1::Declaration { .. }) => None,
                _ => None,
            },
            ValueExprKindV1::Call(call) => self
                .builtins
                .family_id(&call.callee.name)
                .map(|id| ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(id))),
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => {
                quanta_gqlang_v2_hir::walk_value(self, node);
                return;
            }
        };
        if let Some(value) = entry {
            self.entries.push(ResolvedTypeInputEntryV2 {
                key: SourceQualifiedNodeIdV2 {
                    source_id: self.source_id,
                    node_id: node.id,
                },
                value,
            });
        } else if self.error.is_none() {
            self.error = Some(document_failure(
                "sema.unresolved_binding_type",
                self.source_id,
                node.span,
                "query reference has no exact Binder/host or prior-stage type authority",
            ));
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
}

/// Independently reconstructed HIR path for one tentative Binder partition
/// reference. This is a structural witness, not a second name resolver or a
/// callable-effect proof.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PartitionCandidateShapeV2 {
    pub(super) span: SpanV1,
    pub(super) direct_group: HirNodeIdV1,
    pub(super) call_arguments: Vec<HirNodeIdV1>,
}

struct PartitionCandidateShapeCollectorV2 {
    targets: BTreeSet<HirNodeIdV1>,
    direct_group: Option<HirNodeIdV1>,
    call_arguments: Vec<HirNodeIdV1>,
    witnesses: BTreeMap<HirNodeIdV1, PartitionCandidateShapeV2>,
    duplicate: Option<SpanV1>,
}

impl HirVisitorV1 for PartitionCandidateShapeCollectorV2 {
    fn visit_relation(&mut self, relation: &quanta_gqlang_v2_hir::RelationExprHirV1) {
        if let quanta_gqlang_v2_hir::RelationExprKindV1::Project(project) = &relation.kind {
            self.visit_relation(&project.input);
            let previous = self.direct_group;
            self.direct_group = matches!(
                project.input.kind,
                quanta_gqlang_v2_hir::RelationExprKindV1::GroupBy(_)
            )
            .then_some(project.input.id);
            for item in project.items() {
                self.visit_value(&item.value);
            }
            self.direct_group = previous;
        } else {
            quanta_gqlang_v2_hir::walk_relation(self, relation);
        }
    }

    fn visit_value(&mut self, node: &ValueExprHirV1) {
        match &node.kind {
            ValueExprKindV1::Symbol(_) if self.targets.contains(&node.id) => {
                if let Some(direct_group) = self.direct_group {
                    let shape = PartitionCandidateShapeV2 {
                        span: node.span,
                        direct_group,
                        call_arguments: self.call_arguments.clone(),
                    };
                    if self.witnesses.insert(node.id, shape).is_some() {
                        self.duplicate = Some(node.span);
                    }
                }
            }
            ValueExprKindV1::Call(_) => {
                self.call_arguments.push(node.id);
                // HIR's call walker visits only expression arguments, never
                // the callee spelling or a sibling outside this call.
                quanta_gqlang_v2_hir::walk_value(self, node);
                self.call_arguments.pop();
            }
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Symbol(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => quanta_gqlang_v2_hir::walk_value(self, node),
        }
    }
}

pub(super) fn collect_partition_candidate_shapes(
    checked: &CheckedSyntaxDocumentV2,
    bound: &BindResultV1,
) -> Result<BTreeMap<HirNodeIdV1, PartitionCandidateShapeV2>, FrontendFailureV2> {
    let targets = bound
        .partition_candidates()
        .map(|candidate| candidate.reference())
        .collect::<BTreeSet<_>>();
    if targets.is_empty() {
        return Ok(BTreeMap::new());
    }
    let mut collector = PartitionCandidateShapeCollectorV2 {
        targets,
        direct_group: None,
        call_arguments: Vec::new(),
        witnesses: BTreeMap::new(),
        duplicate: None,
    };
    for item in &checked.checked().document().items {
        if let ProgramItemHirV1::Query(query) = item {
            collector.visit_relation(&query.body);
        }
    }
    if let Some(span) = collector.duplicate {
        return Err(document_failure(
            "sema.document_validation_failed",
            checked.source_id(),
            span,
            "partition candidate reference occurs more than once in checked HIR",
        ));
    }
    Ok(collector.witnesses)
}

struct NestedAggregateCallGuardV2<'a> {
    source_id: SourceIdV2,
    facts: &'a TypedExpressionFactsV2,
    builtins: &'a BuiltinRegistryV2,
    aggregate_depth: u32,
    violation: Option<SpanV1>,
}

impl HirVisitorV1 for NestedAggregateCallGuardV2<'_> {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        if self.violation.is_some() {
            return;
        }
        let aggregate_call = if matches!(node.kind, ValueExprKindV1::Call(_)) {
            let key = SourceQualifiedNodeIdV2 {
                source_id: self.source_id,
                node_id: node.id,
            };
            self.facts
                .get(key)
                .and_then(quanta_gqlang_v2_types::TypedExpressionFactV2::resolved_call)
                .and_then(|target| match target {
                    ResolvedCallTargetV2::Builtin(id) => self.builtins.effects_of_resolved(*id),
                    ResolvedCallTargetV2::Predicate(_) | ResolvedCallTargetV2::HostPredicate(_) => {
                        None
                    }
                })
                .is_some_and(|effects| effects.aggregation == AggregationClassV1::Aggregate)
        } else {
            false
        };
        if aggregate_call && self.aggregate_depth > 0 {
            self.violation = Some(node.span);
            return;
        }
        if aggregate_call {
            self.aggregate_depth += 1;
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
        if aggregate_call {
            self.aggregate_depth -= 1;
        }
    }
}

pub(super) fn reject_nested_aggregate_calls(
    checked: &CheckedSyntaxDocumentV2,
    facts: &TypedExpressionFactsV2,
    builtins: &BuiltinRegistryV2,
) -> Result<(), FrontendFailureV2> {
    let mut guard = NestedAggregateCallGuardV2 {
        source_id: checked.source_id(),
        facts,
        builtins,
        aggregate_depth: 0,
        violation: None,
    };
    for item in &checked.checked().document().items {
        guard.visit_item(item);
        if let Some(span) = guard.violation {
            return Err(document_failure(
                "sema.illegal_aggregate_context",
                checked.source_id(),
                span,
                "aggregate call cannot be nested inside another aggregate call",
            ));
        }
    }
    Ok(())
}

pub(super) fn check_partition_candidate_shape(
    witnesses: &BTreeMap<HirNodeIdV1, PartitionCandidateShapeV2>,
    reference: HirNodeIdV1,
    span: SpanV1,
    group: HirNodeIdV1,
    call_stack: &[HirNodeIdV1],
) -> Result<(), &'static str> {
    let Some(shape) = witnesses.get(&reference) else {
        return Err("partition candidate is absent from a direct Project-over-Group expression");
    };
    if shape.span != span || shape.direct_group != group || shape.call_arguments != call_stack {
        return Err("partition candidate differs from the checked HIR reference, group, or call-argument path");
    }
    Ok(())
}

impl HirVisitorV1 for DeclarationTypeInputsV2<'_> {
    fn visit_value(&mut self, node: &ValueExprHirV1) {
        let value = match &node.kind {
            ValueExprKindV1::Symbol(_) => match self.bound.target_of(node.id) {
                Some(ResolvedTargetV1::Declaration { node: target, .. }) => {
                    self.declaration_bodies.get(target).copied().map(|body| {
                        ResolvedTypeInputValueV2::BindingFromDeclaration(SourceQualifiedNodeIdV2 {
                            source_id: self.source_id,
                            node_id: body,
                        })
                    })
                }
                _ => None,
            },
            ValueExprKindV1::Call(call) => self
                .builtins
                .family_id(&call.callee.name)
                .map(|id| ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(id))),
            ValueExprKindV1::Literal(_)
            | ValueExprKindV1::Path(_)
            | ValueExprKindV1::Unary(_)
            | ValueExprKindV1::Binary(_)
            | ValueExprKindV1::Paren(_)
            | ValueExprKindV1::Record(_)
            | ValueExprKindV1::List(_) => None,
        };
        if matches!(
            node.kind,
            ValueExprKindV1::Symbol(_) | ValueExprKindV1::Call(_)
        ) {
            if let Some(value) = value {
                self.entries.push(ResolvedTypeInputEntryV2 {
                    key: SourceQualifiedNodeIdV2 {
                        source_id: self.source_id,
                        node_id: node.id,
                    },
                    value,
                });
            } else if self.error.is_none() {
                let code = if matches!(node.kind, ValueExprKindV1::Call(_)) {
                    "types.unknown_builtin"
                } else {
                    "sema.unresolved_binding_type"
                };
                self.error = Some(document_failure(
                    code,
                    self.source_id,
                    node.span,
                    "no closed authoritative type for value reference",
                ));
            }
        }
        quanta_gqlang_v2_hir::walk_value(self, node);
    }
}
