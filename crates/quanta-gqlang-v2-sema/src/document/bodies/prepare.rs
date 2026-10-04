//! Closed declaration, callable, and type preparation before materialization.

use super::super::{
    check_partition_candidate_shape, collect_partition_candidate_shapes, document_failure,
    inline_query_for_decl, location, module_failure, reject_nested_aggregate_calls, type_failure,
    AggregationClassV1, BTreeMap, BindResultV1, Binder, BuiltinRegistryV2, CheckedSyntaxDocumentV2,
    ClosedFrontendAuthorityV2, DeclarationTypeInputsV2, DeclaredDocumentV2,
    DocumentCatalogReferenceInputV2, DocumentCatalogValidatorV2, DocumentViewTopologyProofV2,
    ExpressionTypeKernelV2, ExpressionTypeStateV2, FrontendTransactionErrorV2, HirVisitorV1,
    InlineViewPlanV2, ModuleImportAdapterV2, NamespaceKindV2, ParamV1, ProgramItemHirV1,
    QueryTypeInputsV2, RelationPreEvaluationInputV2, ResolvedCallTargetV2, ResolvedCallableInputV2,
    ResolvedPredicateIdV2, ResolvedTargetV1, ResolvedTypeInputEntryV2, ResolvedTypeInputValueV2,
    ResolvedTypeInputsV2, RuleBindingIdV2, RuleCallInputV2, RuleCallableTargetV2,
    RulePredicateInputV2, RuleSymbolInputsV2, RuleTypeBudgetV2, RuleTypeSolverV2,
    SourceQualifiedNodeIdV2, SpanV1, ValueExprKindV1,
};
use quanta_gqlang_v2_hir::{DeclHirV1, RuleHirV1};
use quanta_gqlang_v2_modules::ResolvedHostPredicateProofV2;
use quanta_gqlang_v2_types::{RuleSymbolInputV2, RuleTypeProofV2, TypedExpressionFactsV2};

pub(super) struct PreparedBodyTypesV2 {
    pub(super) bound: BindResultV1,
    pub(super) facts: TypedExpressionFactsV2,
    pub(super) predicates: Vec<RulePredicateInputV2>,
    pub(super) rule_proof: Option<RuleTypeProofV2>,
}

struct BoundDocumentV2 {
    bound: BindResultV1,
    declarations: Vec<DeclHirV1>,
    rules: Vec<RuleHirV1>,
}

struct ResolvedRulesV2 {
    predicates: Vec<RulePredicateInputV2>,
    symbols: Vec<RuleSymbolInputV2>,
    calls: Vec<RuleCallInputV2>,
    host_call_proofs: Vec<(SourceQualifiedNodeIdV2, ResolvedHostPredicateProofV2)>,
    proof: Option<RuleTypeProofV2>,
}

pub(super) fn prepare_document_body_types(
    declared: &DeclaredDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    inline_views: &InlineViewPlanV2,
    topology: &DocumentViewTopologyProofV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
) -> Result<PreparedBodyTypesV2, FrontendTransactionErrorV2> {
    let checked = declared.checked();
    validate_item_families(checked)?;
    let binding = bind_document(checked, authority)?;
    let builtins = BuiltinRegistryV2::try_closed().map_err(type_failure)?;
    let rules =
        resolve_rule_callables(checked, authority, extended, validator, &binding, &builtins)?;
    let facts = check_document_types(
        checked,
        authority,
        extended,
        inline_views,
        topology,
        &binding,
        &rules,
        &builtins,
    )?;
    validate_partition_candidates(checked, &binding.bound, &facts, &builtins)?;
    reject_nested_aggregate_calls(checked, &facts, &builtins)?;
    validate_host_predicates(checked, validator, &facts, &rules.host_call_proofs)?;
    Ok(PreparedBodyTypesV2 {
        bound: binding.bound,
        facts,
        predicates: rules.predicates,
        rule_proof: rules.proof,
    })
}

fn validate_item_families(
    checked: &CheckedSyntaxDocumentV2,
) -> Result<(), FrontendTransactionErrorV2> {
    let syntax = checked.checked().document();
    struct RuleRelationExistsV2(Option<SpanV1>);
    impl HirVisitorV1 for RuleRelationExistsV2 {
        fn visit_bool(&mut self, node: &quanta_gqlang_v2_hir::BoolExprHirV1) {
            if matches!(
                node.kind,
                quanta_gqlang_v2_hir::BoolExprKindV1::RelationExists(_)
            ) {
                if self.0.is_none() {
                    self.0 = Some(node.span);
                }
            } else {
                quanta_gqlang_v2_hir::walk_bool(self, node);
            }
        }
    }
    for item in &syntax.items {
        if let ProgramItemHirV1::Rule(rule) = item {
            if !matches!(rule.head.kind, ValueExprKindV1::Symbol(_)) {
                return Err(document_failure(
                    "sema.invalid_item_shape",
                    checked.source_id(),
                    rule.head.span,
                    "source rules require one symbol head term",
                )
                .into());
            }
            let mut forbidden = RuleRelationExistsV2(None);
            forbidden.visit_bool(&rule.body);
            if let Some(span) = forbidden.0 {
                return Err(document_failure(
                    "sema.invalid_item_shape",
                    checked.source_id(),
                    span,
                    "relational EXISTS is not admitted inside a source rule body",
                )
                .into());
            }
        }
    }
    // Fail-closed allowlist of every current `ProgramItemHirV1` variant.
    // A future HIR family that is not listed here is explicit RED, not a
    // silent skip. All listed families have owner resolution paths below.
    if let Some(item) = syntax.items.iter().find(|item| {
        !matches!(
            item,
            ProgramItemHirV1::Query(_)
                | ProgramItemHirV1::Decl(_)
                | ProgramItemHirV1::Rule(_)
                | ProgramItemHirV1::Meta(_)
                | ProgramItemHirV1::GraphQuery(_)
                | ProgramItemHirV1::Presentation(_)
                | ProgramItemHirV1::Legality(_)
                | ProgramItemHirV1::CatalogDecl(_)
                | ProgramItemHirV1::Composition(_)
                | ProgramItemHirV1::Command(_)
        )
    }) {
        return Err(document_failure(
            "sema.document_validation_failed",
            checked.source_id(),
            item.span(),
            "HIR item family is not in the current semantic body allowlist",
        )
        .into());
    }
    Ok(())
}

fn bind_document(
    checked: &CheckedSyntaxDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
) -> Result<BoundDocumentV2, FrontendTransactionErrorV2> {
    let syntax = checked.checked().document();
    let graph = ModuleImportAdapterV2(authority.modules().resolved_graph());
    let mut binder = Binder::new(&graph);
    let declarations = syntax
        .items
        .iter()
        .filter_map(|item| match item {
            ProgramItemHirV1::Decl(decl) => Some((**decl).clone()),
            ProgramItemHirV1::Query(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::CatalogDecl(_)
            | ProgramItemHirV1::Composition(_)
            | ProgramItemHirV1::Meta(_)
            | ProgramItemHirV1::Command(_) => None,
        })
        .collect::<Vec<_>>();
    let rules = syntax
        .items
        .iter()
        .filter_map(|item| match item {
            ProgramItemHirV1::Rule(rule) => Some((**rule).clone()),
            ProgramItemHirV1::Query(_)
            | ProgramItemHirV1::Decl(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::CatalogDecl(_)
            | ProgramItemHirV1::Composition(_)
            | ProgramItemHirV1::Meta(_)
            | ProgramItemHirV1::Command(_) => None,
        })
        .collect::<Vec<_>>();
    binder.declare_program(&declarations, &rules);
    for decl in &declarations {
        binder.resolve_decl(decl, &[]);
    }
    for rule in &rules {
        let ValueExprKindV1::Symbol(head) = &rule.head.kind else {
            return Err(document_failure(
                "sema.invalid_item_shape",
                checked.source_id(),
                rule.head.span,
                "source rules require one symbol head term",
            )
            .into());
        };
        binder.resolve_rule(rule, &[ParamV1::new(&head.name, rule.head.span, rule.id)]);
    }
    for item in &syntax.items {
        match item {
            ProgramItemHirV1::Query(query) => binder.resolve_query(query),
            ProgramItemHirV1::Composition(composition) => binder.resolve_composition(composition),
            ProgramItemHirV1::CatalogDecl(decl) => binder.resolve_catalog_decl(decl),
            ProgramItemHirV1::Decl(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::Meta(_)
            | ProgramItemHirV1::Command(_) => {}
        }
    }
    let bound = binder.finish();
    if let Some(error) = bound.diagnostics().first() {
        return Err(document_failure(
            error.code(),
            checked.source_id(),
            error.primary_span(),
            error.to_string(),
        )
        .into());
    }
    Ok(BoundDocumentV2 {
        bound,
        declarations,
        rules,
    })
}

fn resolve_rule_callables(
    checked: &CheckedSyntaxDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    binding: &BoundDocumentV2,
    builtins: &BuiltinRegistryV2,
) -> Result<ResolvedRulesV2, FrontendTransactionErrorV2> {
    let bound = &binding.bound;
    let rules = &binding.rules;
    let mut rule_collector = RuleSymbolInputsV2 {
        source_id: checked.source_id(),
        bound,
        symbols: Vec::new(),
        calls: Vec::new(),
        error: None,
    };
    for rule in rules {
        rule_collector.visit_rule(rule);
    }
    if let Some(error) = rule_collector.error {
        return Err(error.into());
    }
    let predicates = rules
        .iter()
        .map(|rule| {
            let predicate_id = ResolvedPredicateIdV2::try_new(&rule.name).map_err(type_failure)?;
            Ok::<_, FrontendTransactionErrorV2>(RulePredicateInputV2 {
                rule_node: SourceQualifiedNodeIdV2 {
                    source_id: checked.source_id(),
                    node_id: rule.id,
                },
                predicate_id,
                head_binding: RuleBindingIdV2::new(checked.source_id(), rule.id, 0),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut rule_calls = Vec::new();
    let mut host_call_proofs = Vec::new();
    for (call, name, span) in &rule_collector.calls {
        let source = predicates
            .iter()
            .find(|candidate| candidate.predicate_id.as_str() == name);
        let builtin = builtins.family_id(name);
        if source.is_some() || builtin.is_some() {
            let checked_location = extended
                .combined()
                .try_check_location(location(checked.source_id(), *span))
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        checked.source_id(),
                        *span,
                        error.to_string(),
                    )
                })?;
            if validator
                .probe_reference(DocumentCatalogReferenceInputV2 {
                    module_id: None,
                    local_name: name.clone(),
                    expected_kind: NamespaceKindV2::Predicate,
                    location: checked_location,
                })
                .map_err(module_failure)?
                .is_some()
            {
                return Err(document_failure(
                    "sema.ambiguous_callable",
                    checked.source_id(),
                    *span,
                    "source predicate or builtin family conflicts with visible host Predicate",
                )
                .into());
            }
        }
        let target = match (source, builtin) {
            (Some(_), Some(_)) => {
                return Err(document_failure(
                    "sema.ambiguous_callable",
                    checked.source_id(),
                    *span,
                    "source predicate and builtin family share one call spelling",
                )
                .into())
            }
            (Some(predicate), None) => {
                RuleCallableTargetV2::SourcePredicate(predicate.predicate_id.clone())
            }
            (None, Some(family)) => RuleCallableTargetV2::Builtin(family),
            (None, None) => {
                let checked_location = extended
                    .combined()
                    .try_check_location(location(checked.source_id(), *span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            checked.source_id(),
                            *span,
                            error.to_string(),
                        )
                    })?;
                let proof = validator
                    .resolve_host_predicate_call(DocumentCatalogReferenceInputV2 {
                        module_id: None,
                        local_name: name.clone(),
                        expected_kind: NamespaceKindV2::Predicate,
                        location: checked_location,
                    })
                    .map_err(module_failure)?;
                let target = RuleCallableTargetV2::HostPredicate(proof.callable().clone());
                host_call_proofs.push((*call, proof));
                target
            }
        };
        rule_calls.push(RuleCallInputV2 {
            call: *call,
            target,
        });
    }
    let rule_proof = if rules.is_empty() {
        None
    } else {
        let solver = RuleTypeSolverV2::try_closed().map_err(type_failure)?;
        let budget = RuleTypeBudgetV2::try_new(
            authority.budget().max_rule_graph_nodes(),
            authority.budget().max_rule_graph_edges(),
            authority.budget().max_rule_steps(),
        )
        .map_err(type_failure)?;
        Some(
            solver
                .solve(
                    checked.checked().as_checked(),
                    extended,
                    predicates.clone(),
                    rule_collector.symbols.clone(),
                    rule_calls.clone(),
                    budget,
                )
                .map_err(type_failure)?,
        )
    };
    Ok(ResolvedRulesV2 {
        predicates,
        symbols: rule_collector.symbols,
        calls: rule_calls,
        host_call_proofs,
        proof: rule_proof,
    })
}

fn check_document_types(
    checked: &CheckedSyntaxDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    inline_views: &InlineViewPlanV2,
    topology: &DocumentViewTopologyProofV2,
    binding: &BoundDocumentV2,
    rules: &ResolvedRulesV2,
    builtins: &BuiltinRegistryV2,
) -> Result<TypedExpressionFactsV2, FrontendTransactionErrorV2> {
    let syntax = checked.checked().document();
    let bound = &binding.bound;
    let declarations = &binding.declarations;
    let rule_proof = &rules.proof;
    let rule_calls = &rules.calls;
    let declaration_bodies = declarations
        .iter()
        .map(|decl| (decl.id, decl.body.id))
        .collect::<BTreeMap<_, _>>();
    let mut collector = DeclarationTypeInputsV2 {
        source_id: checked.source_id(),
        bound,
        declaration_bodies: &declaration_bodies,
        builtins,
        entries: Vec::new(),
        error: None,
    };
    for decl in declarations {
        collector.visit_value(&decl.body);
    }
    if let Some(error) = collector.error {
        return Err(error.into());
    }
    let mut query_inputs = QueryTypeInputsV2 {
        source_id: checked.source_id(),
        authority,
        bound,
        builtins,
        extended,
        view_source_targets: &inline_views.source_targets,
        view_relation_roots: &inline_views.relation_roots,
        owners: BTreeMap::new(),
        entries: Vec::new(),
        rows: Vec::new(),
        composition_rows: BTreeMap::new(),
        error: None,
    };
    for asset in topology.dependency_first_order() {
        let Some(declaration) = inline_views.declarations.get(asset).copied() else {
            continue;
        };
        let query = inline_query_for_decl(syntax, declaration).ok_or_else(|| {
            document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                SpanV1::point(0),
                "topology-selected inline View lost its exact HIR query",
            )
        })?;
        query_inputs.visit_query(query);
    }
    for item in &syntax.items {
        match item {
            ProgramItemHirV1::Query(query) => query_inputs.visit_relation(&query.body),
            ProgramItemHirV1::Composition(composition) => {
                query_inputs.visit_composition(composition)
            }
            ProgramItemHirV1::Decl(_)
            | ProgramItemHirV1::Rule(_)
            | ProgramItemHirV1::GraphQuery(_)
            | ProgramItemHirV1::Legality(_)
            | ProgramItemHirV1::Presentation(_)
            | ProgramItemHirV1::CatalogDecl(_)
            | ProgramItemHirV1::Meta(_)
            | ProgramItemHirV1::Command(_) => {}
        }
    }
    if let Some(error) = query_inputs.error {
        return Err(error.into());
    }
    collector.entries.extend(query_inputs.entries);
    if let Some(proof) = &rule_proof {
        for symbol in &rules.symbols {
            let Some(ty) = proof.binding_type(symbol.binding) else {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    checked.source_id(),
                    SpanV1::point(0),
                    "rule solver omitted a Binder-identified concrete binding",
                )
                .into());
            };
            collector.entries.push(ResolvedTypeInputEntryV2 {
                key: symbol.reference,
                value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(ty.clone())),
            });
        }
        for call in rule_calls {
            let callable = match &call.target {
                RuleCallableTargetV2::Builtin(family) => ResolvedCallableInputV2::Builtin(*family),
                RuleCallableTargetV2::SourcePredicate(id) => {
                    let Some(signature) = proof
                        .try_source_predicate_callable(id)
                        .map_err(type_failure)?
                    else {
                        return Err(document_failure(
                            "sema.document_validation_failed",
                            checked.source_id(),
                            SpanV1::point(0),
                            "source predicate signature is absent from rule type proof",
                        )
                        .into());
                    };
                    ResolvedCallableInputV2::Predicate(signature)
                }
                RuleCallableTargetV2::HostPredicate(signature) => {
                    ResolvedCallableInputV2::HostPredicate(signature.clone())
                }
                RuleCallableTargetV2::Unknown => {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        checked.source_id(),
                        SpanV1::point(0),
                        "unknown rule callable cannot enter closed types inputs",
                    )
                    .into())
                }
            };
            collector.entries.push(ResolvedTypeInputEntryV2 {
                key: call.call,
                value: ResolvedTypeInputValueV2::Callable(callable),
            });
        }
    }
    let pre_evaluation_roots = topology
        .dependency_first_order()
        .filter_map(|asset| inline_views.relation_roots.get(asset).copied())
        .collect();
    let type_inputs = ResolvedTypeInputsV2::try_new_with_relation_pre_evaluation(
        checked.checked().as_checked(),
        extended,
        collector.entries,
        Vec::new(),
        query_inputs.rows,
        RelationPreEvaluationInputV2::new(pre_evaluation_roots),
    )
    .map_err(type_failure)?;
    let kernel = ExpressionTypeKernelV2::try_closed().map_err(type_failure)?;
    let facts = kernel.check_document(&type_inputs).map_err(type_failure)?;
    Ok(facts)
}

fn validate_partition_candidates(
    checked: &CheckedSyntaxDocumentV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    builtins: &BuiltinRegistryV2,
) -> Result<(), FrontendTransactionErrorV2> {
    let partition_witnesses = collect_partition_candidate_shapes(checked, bound)?;
    for candidate in bound.partition_candidates() {
        check_partition_candidate_shape(
            &partition_witnesses,
            candidate.reference(),
            candidate.span(),
            candidate.group(),
            candidate.call_stack(),
        )
        .map_err(|message| {
            document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                candidate.span(),
                message,
            )
        })?;
        let Some(partition) = bound.partition_input(candidate.group()) else {
            return Err(document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                candidate.span(),
                "deferred group binding has no exact partition input",
            )
            .into());
        };
        if !partition.contains(candidate.binding())
            || !matches!(bound.target_of(candidate.reference()),
            Some(ResolvedTargetV1::Local(binding)) if binding == candidate.binding())
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                candidate.span(),
                "deferred aggregate argument differs from Binder partition identity",
            )
            .into());
        }
        if candidate.call_stack().is_empty() {
            return Err(document_failure(
                "sema.illegal_aggregate_context",
                checked.source_id(),
                candidate.span(),
                "partition-input name cannot escape into an ordinary projection",
            )
            .into());
        }
        for (index, &call_node) in candidate.call_stack().iter().enumerate() {
            let key = SourceQualifiedNodeIdV2 {
                source_id: checked.source_id(),
                node_id: call_node,
            };
            let Some(ResolvedCallTargetV2::Builtin(id)) = facts
                .get(key)
                .and_then(quanta_gqlang_v2_types::TypedExpressionFactV2::resolved_call)
            else {
                return Err(document_failure(
                    "sema.illegal_aggregate_context",
                    checked.source_id(),
                    candidate.span(),
                    "partition-input argument is not enclosed by exact builtin overloads",
                )
                .into());
            };
            let Some(intrinsic) = builtins.effects_of_resolved(*id) else {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    checked.source_id(),
                    candidate.span(),
                    "types owner omitted resolved builtin overload effects",
                )
                .into());
            };
            if (index == 0 && intrinsic.aggregation != AggregationClassV1::Aggregate)
                || (index > 0 && intrinsic.aggregation != AggregationClassV1::Scalar)
            {
                return Err(document_failure("sema.illegal_aggregate_context", checked.source_id(), candidate.span(),
                    "partition input is available only under one outer aggregate, never scalar-over-aggregate or nested aggregate").into());
            }
        }
    }
    Ok(())
}

fn validate_host_predicates(
    checked: &CheckedSyntaxDocumentV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    facts: &TypedExpressionFactsV2,
    host_call_proofs: &[(SourceQualifiedNodeIdV2, ResolvedHostPredicateProofV2)],
) -> Result<(), FrontendTransactionErrorV2> {
    for (call, proof) in host_call_proofs {
        let target = facts
            .get(*call)
            .and_then(quanta_gqlang_v2_types::TypedExpressionFactV2::resolved_call);
        let Some(ResolvedCallTargetV2::HostPredicate(expected_key)) = target else {
            return Err(document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                proof.location().location().span,
                "types kernel omitted host predicate call target",
            )
            .into());
        };
        validator
            .validate_host_predicate_proof(proof, expected_key, proof.location())
            .map_err(module_failure)?;
    }
    Ok(())
}
