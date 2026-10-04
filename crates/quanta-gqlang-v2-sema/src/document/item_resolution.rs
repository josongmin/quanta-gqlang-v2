//! Closed graph, presentation, CHECK, and catalog item resolution.

use super::{
    document_failure, location, module_failure, BTreeMap, BindResultV1, BinderKindV1,
    ClosedFrontendAuthorityV2, DocumentCatalogFamilyReferenceInputV2,
    DocumentCatalogReferenceFamilyV2, DocumentCatalogReferenceInputV2, DocumentCatalogValidatorV2,
    ExtendedSourceRegistryViewV2, FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2,
    FrontendStageV2, FrontendTransactionErrorV2, GraphDirectionV2, GraphReferenceRoleV2,
    GraphWitnessBoundsV2, HirNodeIdV1, ModuleGraphViewV1, NamespaceKindV2,
    PredeclaredDocumentAssetV2, ResolvedCatalogReferenceV2, ResolvedCheckV2,
    ResolvedCompositionBindingV2, ResolvedExplainTargetV2, ResolvedGraphQueryV2,
    ResolvedGraphReferenceV2, ResolvedGraphTargetV2, ResolvedGraphUseV2, ResolvedGraphV2,
    ResolvedPresentationKindV2, ResolvedPresentationV2, ResolvedTargetV1, SemanticItemIdV2,
    SemanticStepCounterV2, SourceIdV2,
};

pub(super) fn resolve_graph_target_use(
    owner: HirNodeIdV1,
    reference: &quanta_gqlang_v2_hir::GraphRefV1,
    required: crate::GraphReferenceKindV2,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    steps: &mut SemanticStepCounterV2,
) -> Result<
    (
        ResolvedGraphTargetV2,
        quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
    ),
    FrontendFailureV2,
> {
    steps.charge(source_id, reference.span)?;
    let location = location(source_id, reference.span);
    let checked = extended
        .combined()
        .try_check_location(location)
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                reference.span,
                error.to_string(),
            )
        })?;
    if let Some(target) = bound.target_of_use(owner, reference.span, &reference.name) {
        let ResolvedTargetV1::Local(binding) = target else {
            return Err(document_failure(
                "sema.invalid_composition_graph_target",
                source_id,
                reference.span,
                "resolved graph reference is not a local Composition binding",
            ));
        };
        if binding.kind != BinderKindV1::CompositionLet {
            return Err(document_failure(
                "sema.invalid_composition_graph_target",
                source_id,
                reference.span,
                "resolved graph reference is not a Composition binding",
            ));
        }
        let local = composition_targets
            .get(&binding.identity())
            .ok_or_else(|| {
                document_failure(
                    "sema.invalid_composition_graph_target",
                    source_id,
                    reference.span,
                    "Composition graph target is not a prior sealed producer",
                )
            })?;
        if local.binding().target_node() != binding.node
            || local.binding().ordinal() != binding.identity().1
        {
            return Err(document_failure(
                "sema.document_validation_failed",
                source_id,
                reference.span,
                "Composition graph target differs from Binder binding proof",
            ));
        }
        let target = local.try_graph_target(required).map_err(|error| {
            document_failure(
                "sema.invalid_composition_graph_target",
                source_id,
                reference.span,
                error,
            )
        })?;
        return Ok((target, checked));
    }
    let target = authority
        .graph_references()
        .resolve(&reference.name, required, location)?;
    Ok((ResolvedGraphTargetV2::host(target), checked))
}

pub(super) fn resolve_graph_item(
    graph: &quanta_gqlang_v2_hir::GraphQueryHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedGraphQueryV2, FrontendFailureV2> {
    let mut references = Vec::new();
    let endpoints = &graph.endpoints;
    let constraints = &graph.constraints;
    let singular = [
        (GraphReferenceRoleV2::Source, endpoints.source.as_ref()),
        (GraphReferenceRoleV2::Sink, endpoints.sink.as_ref()),
        (GraphReferenceRoleV2::Anchor, endpoints.anchor.as_ref()),
        (
            GraphReferenceRoleV2::Reference,
            endpoints.reference.as_ref(),
        ),
        (GraphReferenceRoleV2::Pointee, endpoints.pointee.as_ref()),
        (GraphReferenceRoleV2::Callable, endpoints.callable.as_ref()),
        (GraphReferenceRoleV2::Scope, constraints.scope.as_ref()),
        (GraphReferenceRoleV2::Window, constraints.window.as_ref()),
        (
            GraphReferenceRoleV2::Frontier,
            constraints.frontier.as_ref(),
        ),
    ];
    let repeated = constraints
        .via
        .iter()
        .map(|reference| (GraphReferenceRoleV2::Via, reference))
        .chain(
            constraints
                .through
                .iter()
                .map(|reference| (GraphReferenceRoleV2::Through, reference)),
        )
        .chain(
            constraints
                .sanitizers
                .iter()
                .map(|reference| (GraphReferenceRoleV2::Sanitizer, reference)),
        );
    for (role, reference) in singular
        .into_iter()
        .filter_map(|(role, reference)| reference.map(|reference| (role, reference)))
        .chain(repeated)
    {
        let expected = match role {
            GraphReferenceRoleV2::Source
            | GraphReferenceRoleV2::Sink
            | GraphReferenceRoleV2::Anchor
            | GraphReferenceRoleV2::Reference
            | GraphReferenceRoleV2::Pointee
            | GraphReferenceRoleV2::Via
            | GraphReferenceRoleV2::Through => crate::GraphReferenceKindV2::Entity,
            GraphReferenceRoleV2::Callable => crate::GraphReferenceKindV2::Callable,
            GraphReferenceRoleV2::Sanitizer => crate::GraphReferenceKindV2::Sanitizer,
            GraphReferenceRoleV2::Scope => crate::GraphReferenceKindV2::Scope,
            GraphReferenceRoleV2::Window => crate::GraphReferenceKindV2::Window,
            GraphReferenceRoleV2::Frontier => crate::GraphReferenceKindV2::Frontier,
        };
        let (target, checked) = resolve_graph_target_use(
            graph.id,
            reference,
            expected,
            source_id,
            authority,
            bound,
            composition_targets,
            extended,
            steps,
        )?;
        references.push(
            ResolvedGraphReferenceV2::try_new(role, target, checked).map_err(|error| {
                document_failure("sema.invalid_item_shape", source_id, reference.span, error)
            })?,
        );
    }
    let direction = constraints
        .direction
        .as_ref()
        .map(|direction| GraphDirectionV2::try_from_keyword(&direction.keyword))
        .transpose()
        .map_err(|error| {
            document_failure("sema.invalid_item_shape", source_id, graph.span, error)
        })?;
    if constraints
        .depth
        .is_some_and(|depth| depth > authority.budget().max_witness_depth())
    {
        return Err(document_failure(
            "frontend.resource_limit_exceeded",
            source_id,
            graph.span,
            "FIND depth exceeds the frontend witness-depth budget",
        ));
    }
    let graph_location = extended
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
    ResolvedGraphQueryV2::try_new(
        SemanticItemIdV2::new(source_id, graph.id),
        graph_location,
        graph.kind,
        references,
        constraints.depth,
        direction,
        constraints.summary_facets.clone(),
        GraphWitnessBoundsV2::new(
            authority.budget().max_witness_depth(),
            authority.budget().max_witnesses(),
        ),
    )
    .map_err(|error| document_failure("sema.invalid_item_shape", source_id, graph.span, error))
}

#[expect(
    clippy::too_many_lines,
    reason = "legacy presentation resolution awaits phase extraction"
)]
pub(super) fn resolve_presentation_item(
    presentation: &quanta_gqlang_v2_hir::PresentationHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedPresentationV2, FrontendTransactionErrorV2> {
    let graph = |reference: &quanta_gqlang_v2_hir::GraphRefV1, kind| {
        let location = location(source_id, reference.span);
        let target = authority
            .graph_references()
            .resolve(&reference.name, kind, location)?;
        let checked = extended
            .combined()
            .try_check_location(location)
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    reference.span,
                    error.to_string(),
                )
            })?;
        Ok::<_, FrontendFailureV2>(ResolvedGraphUseV2::new(
            ResolvedGraphTargetV2::host(target),
            checked,
        ))
    };
    let kind = match &presentation.kind {
        quanta_gqlang_v2_hir::PresentationKindV1::Show { category } => {
            ResolvedPresentationKindV2::Show {
                category: *category,
            }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Describe { kind, path } => {
            steps.charge(source_id, path.span)?;
            let location = location(source_id, path.span);
            let target = authority
                .graph_references()
                .resolve_entity_subtype(&path.name, kind, location)?;
            let checked = extended
                .combined()
                .try_check_location(location)
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        path.span,
                        error.to_string(),
                    )
                })?;
            let target = ResolvedGraphUseV2::new(ResolvedGraphTargetV2::host(target), checked);
            ResolvedPresentationKindV2::Describe { target }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Sample {
            query_text,
            mode,
            limit,
            scope,
        } => {
            let scope = scope
                .as_ref()
                .map(|reference| {
                    steps.charge(source_id, reference.span)?;
                    graph(reference, crate::GraphReferenceKindV2::Scope)
                })
                .transpose()?;
            ResolvedPresentationKindV2::Sample {
                query_text: query_text.clone(),
                mode: *mode,
                limit: *limit,
                scope,
            }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Compare {
            baseline_ref,
            candidate_ref,
            metric,
        } => {
            steps.charge(source_id, baseline_ref.span)?;
            steps.charge(source_id, candidate_ref.span)?;
            ResolvedPresentationKindV2::Compare {
                baseline: graph(baseline_ref, crate::GraphReferenceKindV2::Snapshot)?,
                candidate: graph(candidate_ref, crate::GraphReferenceKindV2::Snapshot)?,
                metric: *metric,
            }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Export {
            source_ref,
            format,
            profile,
        } => {
            steps.charge(source_id, source_ref.span)?;
            let source = graph(source_ref, crate::GraphReferenceKindV2::ExportSource)?;
            let profile = profile
                .as_ref()
                .map(|reference| {
                    steps.charge(source_id, reference.span)?;
                    let checked = extended
                        .combined()
                        .try_check_location(location(source_id, reference.span))
                        .map_err(|error| {
                            FrontendTransactionErrorV2::Document(document_failure(
                                "sema.document_validation_failed",
                                source_id,
                                reference.span,
                                error.to_string(),
                            ))
                        })?;
                    let proof = validator
                        .resolve_reference(DocumentCatalogReferenceInputV2 {
                            module_id: None,
                            local_name: reference.name.clone(),
                            expected_kind: NamespaceKindV2::Profile,
                            location: checked,
                        })
                        .map_err(module_failure)?;
                    Ok::<_, FrontendTransactionErrorV2>(ResolvedCatalogReferenceV2::from_proof(
                        &proof,
                    ))
                })
                .transpose()?;
            ResolvedPresentationKindV2::Export {
                source,
                format: *format,
                profile,
            }
        }
        quanta_gqlang_v2_hir::PresentationKindV1::Explain {
            target_ref,
            evidence,
            truth,
        } => {
            steps.charge(source_id, target_ref.span)?;
            let raw = location(source_id, target_ref.span);
            let checked = extended
                .combined()
                .try_check_location(raw)
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        target_ref.span,
                        error.to_string(),
                    )
                })?;
            let rule = validator
                .probe_reference(DocumentCatalogReferenceInputV2 {
                    module_id: None,
                    local_name: target_ref.name.clone(),
                    expected_kind: NamespaceKindV2::Rule,
                    location: checked.clone(),
                })
                .map_err(module_failure)?;
            let entity = authority.graph_references().probe(
                &target_ref.name,
                crate::GraphReferenceKindV2::Entity,
                raw,
            )?;
            let target = match (rule, entity) {
                (Some(rule), None) => {
                    ResolvedExplainTargetV2::Rule(ResolvedCatalogReferenceV2::from_proof(&rule))
                }
                (None, Some(entity)) => ResolvedExplainTargetV2::Entity(ResolvedGraphUseV2::new(
                    ResolvedGraphTargetV2::host(entity),
                    checked,
                )),
                (Some(rule), Some(entity)) => {
                    return Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
                        FrontendStageV2::Resolution,
                        "sema.ambiguous_reference",
                        FrontendSeverityV2::Error,
                        raw,
                        vec![
                            rule.location().location(),
                            entity.declaration_location().location(),
                        ],
                        "EXPLAIN target resolves in both Rule and Entity namespaces",
                    ))
                    .into())
                }
                (None, None) => {
                    return Err(document_failure(
                        "sema.unresolved_reference",
                        source_id,
                        target_ref.span,
                        "EXPLAIN target is absent from Rule and Entity authorities",
                    )
                    .into())
                }
            };
            ResolvedPresentationKindV2::Explain {
                target,
                evidence: *evidence,
                truth: *truth,
            }
        }
    };
    ResolvedPresentationV2::try_new(
        SemanticItemIdV2::new(source_id, presentation.id),
        extended
            .combined()
            .try_check_location(location(source_id, presentation.span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    presentation.span,
                    error.to_string(),
                )
            })?,
        kind,
    )
    .map_err(|error| {
        document_failure(
            "sema.invalid_item_shape",
            source_id,
            presentation.span,
            error,
        )
        .into()
    })
}

pub(super) fn resolve_composition_explain(
    explain: &quanta_gqlang_v2_hir::ExplainHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedPresentationV2, FrontendTransactionErrorV2> {
    let raw = location(source_id, explain.target_ref.span);
    let target = if bound
        .target_of_use(
            explain.id,
            explain.target_ref.span,
            &explain.target_ref.name,
        )
        .is_some()
    {
        let (target, location) = resolve_graph_target_use(
            explain.id,
            &explain.target_ref,
            crate::GraphReferenceKindV2::Entity,
            source_id,
            authority,
            bound,
            composition_targets,
            extended,
            steps,
        )?;
        ResolvedExplainTargetV2::Entity(ResolvedGraphUseV2::new(target, location))
    } else {
        steps.charge(source_id, explain.target_ref.span)?;
        let checked = extended
            .combined()
            .try_check_location(raw)
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    explain.target_ref.span,
                    error.to_string(),
                )
            })?;
        let rule = validator
            .probe_reference(DocumentCatalogReferenceInputV2 {
                module_id: None,
                local_name: explain.target_ref.name.clone(),
                expected_kind: NamespaceKindV2::Rule,
                location: checked.clone(),
            })
            .map_err(module_failure)?;
        let entity = authority.graph_references().probe(
            &explain.target_ref.name,
            crate::GraphReferenceKindV2::Entity,
            raw,
        )?;
        match (rule, entity) {
            (Some(rule), None) => {
                ResolvedExplainTargetV2::Rule(ResolvedCatalogReferenceV2::from_proof(&rule))
            }
            (None, Some(entity)) => ResolvedExplainTargetV2::Entity(ResolvedGraphUseV2::new(
                ResolvedGraphTargetV2::host(entity),
                checked,
            )),
            (Some(rule), Some(entity)) => {
                return Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
                    FrontendStageV2::Resolution,
                    "sema.ambiguous_reference",
                    FrontendSeverityV2::Error,
                    raw,
                    vec![
                        rule.location().location(),
                        entity.declaration_location().location(),
                    ],
                    "EXPLAIN target resolves in both Rule and Entity namespaces",
                ))
                .into())
            }
            (None, None) => {
                return Err(document_failure(
                    "sema.unresolved_reference",
                    source_id,
                    explain.target_ref.span,
                    "EXPLAIN target is absent from Rule and Entity authorities",
                )
                .into())
            }
        }
    };
    ResolvedPresentationV2::try_new(
        SemanticItemIdV2::new(source_id, explain.id),
        extended
            .combined()
            .try_check_location(location(source_id, explain.span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    explain.span,
                    error.to_string(),
                )
            })?,
        ResolvedPresentationKindV2::Explain {
            target,
            evidence: explain.evidence,
            truth: explain.truth,
        },
    )
    .map_err(|error| {
        document_failure("sema.invalid_item_shape", source_id, explain.span, error).into()
    })
}

pub(super) fn resolve_check_item(
    check: &quanta_gqlang_v2_hir::LegalityHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    composition_targets: &BTreeMap<(HirNodeIdV1, usize), ResolvedCompositionBindingV2>,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
) -> Result<ResolvedCheckV2, FrontendTransactionErrorV2> {
    steps.charge(source_id, check.rule_ref.span)?;
    let rule_location = location(source_id, check.rule_ref.span);
    let checked_rule = extended
        .combined()
        .try_check_location(rule_location)
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                check.rule_ref.span,
                error.to_string(),
            )
        })?;
    let rule = validator
        .resolve_reference(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: check.rule_ref.name.clone(),
            expected_kind: NamespaceKindV2::Rule,
            location: checked_rule,
        })
        .map_err(module_failure)?;
    let resolve_optional = |reference: Option<&quanta_gqlang_v2_hir::GraphRefV1>,
                            kind,
                            steps: &mut SemanticStepCounterV2| {
        reference
            .map(|reference| {
                resolve_graph_target_use(
                    check.id,
                    reference,
                    kind,
                    source_id,
                    authority,
                    bound,
                    composition_targets,
                    extended,
                    steps,
                )
                .map(|(target, checked)| ResolvedGraphUseV2::new(target, checked))
            })
            .transpose()
    };
    let scope = resolve_optional(
        check.scope.as_ref(),
        crate::GraphReferenceKindV2::Scope,
        steps,
    )?;
    let pattern = resolve_optional(
        check.pattern.as_ref(),
        crate::GraphReferenceKindV2::Pattern,
        steps,
    )?;
    let nested = check
        .semantic_confirm
        .as_ref()
        .map(|graph| {
            resolve_graph_item(
                graph,
                source_id,
                authority,
                bound,
                composition_targets,
                extended,
                steps,
            )
        })
        .transpose()?;
    let expected_nested = check
        .semantic_confirm
        .as_ref()
        .map(|graph| SemanticItemIdV2::new(source_id, graph.id));
    ResolvedCheckV2::try_new(
        SemanticItemIdV2::new(source_id, check.id),
        extended
            .combined()
            .try_check_location(location(source_id, check.span))
            .map_err(|error| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    check.span,
                    error.to_string(),
                )
            })?,
        expected_nested,
        ResolvedCatalogReferenceV2::from_proof(&rule),
        scope,
        pattern,
        nested,
        check.confirm,
        check.emit,
        check.mode,
    )
    .map_err(|error| {
        document_failure("sema.invalid_item_shape", source_id, check.span, error).into()
    })
}

/// Projection of the modules-owned frozen graph into binder's read-only
/// imported-target interface; no precedence, search, or ID is recomputed.
pub(super) struct ModuleImportAdapterV2<'a>(pub(super) &'a ResolvedGraphV2);

impl ModuleGraphViewV1 for ModuleImportAdapterV2<'_> {
    fn imported_targets(&self, name: &str) -> Vec<String> {
        self.0
            .visible_bindings()
            .iter()
            .filter(|binding| {
                binding.namespace_kind() == NamespaceKindV2::Value && binding.local_name() == name
            })
            .map(|binding| {
                format!(
                    "{}::{}",
                    binding.resolved_asset_id().module_id().as_str(),
                    binding.resolved_asset_id().local_name()
                )
            })
            .collect()
    }
}

pub(super) fn resolve_catalog_family_reference(
    reference: &quanta_gqlang_v2_hir::GraphRefV1,
    family: DocumentCatalogReferenceFamilyV2,
    source: &PredeclaredDocumentAssetV2,
    source_id: SourceIdV2,
    extended: &ExtendedSourceRegistryViewV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
    edges: &mut Vec<DocumentCatalogFamilyReferenceInputV2>,
) -> Result<ResolvedCatalogReferenceV2, FrontendTransactionErrorV2> {
    steps.charge(source_id, reference.span)?;
    let checked = extended
        .combined()
        .try_check_location(location(source_id, reference.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                reference.span,
                error.to_string(),
            )
        })?;
    let target = validator
        .resolve_reference(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: reference.name.clone(),
            expected_kind: family.target_kind(),
            location: checked.clone(),
        })
        .map_err(module_failure)?;
    edges.push(DocumentCatalogFamilyReferenceInputV2 {
        family,
        source: source.clone(),
        target: target.clone(),
        location: checked,
    });
    Ok(ResolvedCatalogReferenceV2::from_proof(&target))
}
