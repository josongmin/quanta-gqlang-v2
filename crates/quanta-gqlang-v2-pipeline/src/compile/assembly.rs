//! Exact-key assembly of one sealed semantic document.

use super::{
    algebra_transaction_error, assembly_transaction_error, canonicalize_diagnostics,
    enforce_diagnostic_limit, logic_transaction_error, planning_transaction_error, project_check,
    project_composition, project_graph_query, project_presentation, project_relation,
    project_relation_root, AlgebraSessionV2, AnalysisDocumentV2, AnalysisItemV2, BTreeMap,
    BTreeSet, CatalogDeclarationKindV2, CheckContractV2, CheckedCompositionPlanInputV2,
    CompiledItemV2, CompiledRuleProgramV2, CompositionBodyPlanInputV2,
    CompositionRelationalBodyPlanInputV2, DocumentAssemblyCodeV2, ExecutionCheckV3,
    ExecutionClosureFailureV3, ExecutionCompositionBodyProjectionV3,
    ExecutionCompositionProducerProjectionV3, ExecutionCompositionProjectionFailureV3,
    ExecutionCompositionV3, ExecutionFamilyV3, ExecutionGraphQueryV3, ExecutionItemV3,
    ExecutionPresentationKindV3, ExecutionRelationV3, FrontendFailureV2,
    FrontendTransactionErrorV2, GraphQueryContractV2, LocatedSpanV2, NoPlanReasonV2, PlanOutcomeV2,
    PlannedItemV2, PlanningDiagnosticV1, PlanningSessionV2, PresentationContractV2,
    ResolvedCatalogItemV2, ResolvedCatalogPayloadV2, ResolvedCheckV2, ResolvedCompositionBodyV2,
    ResolvedCompositionProducerV2, ResolvedCompositionV2, ResolvedMetaKindV2,
    RuleProgramContractV2, RuleProgramMemberV2, SemanticDocumentV2, SemanticItemIdV2,
    SemanticItemV2, SemanticPlanInputV2, SemanticPlanV2, SourceIdV2, SpanV1,
};

/// The completed run of every applicable stage over one sealed document.
pub(super) struct DocumentRunV2 {
    pub(super) items: Vec<ItemRunV2>,
    pub(super) rule_program: Option<CompiledRuleProgramV2>,
    pub(super) rule_blocked: Option<FrontendFailureV2>,
}

type CompiledPartsV2 = (
    Vec<CompiledItemV2>,
    Option<CompiledRuleProgramV2>,
    Vec<ExecutionItemV3>,
);

/// Boxed planned payload: the plan plus its two execution projections dwarf the
/// other outcomes, so the box keeps every `ItemRunV2` variant pointer-sized.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlannedRunV2 {
    pub(super) planned: Box<PlannedItemV2>,
    pub(super) relation: Option<ExecutionRelationV3>,
    pub(super) graph: Option<ExecutionGraphQueryV3>,
    pub(super) check: Option<ExecutionCheckV3>,
    pub(super) presentation: Option<ExecutionPresentationKindV3>,
    pub(super) composition: Option<ExecutionCompositionV3>,
}

/// One item's stage outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ItemRunV2 {
    /// Every applicable stage completed.
    Planned(Box<PlannedRunV2>),
    /// The item is a checked non-plan declaration.
    NoPlan {
        item_id: SemanticItemIdV2,
        reason: NoPlanReasonV2,
    },
    /// A stage failed; the owner failure guarantees non-empty ordered diagnostics.
    Blocked {
        item_id: SemanticItemIdV2,
        failure: FrontendFailureV2,
    },
}

impl DocumentRunV2 {
    /// The exact compiled item list, failing closed on the first blocked stage.
    pub(super) fn into_compiled(self) -> Result<CompiledPartsV2, FrontendTransactionErrorV2> {
        if let Some(failure) = self.rule_blocked {
            return Err(FrontendTransactionErrorV2::Document(failure.deduplicated()));
        }
        let mut items = Vec::with_capacity(self.items.len());
        let mut execution_items = Vec::new();
        for item in self.items {
            match item {
                ItemRunV2::Planned(run) => {
                    if let Some((family, plan)) = execution_identity(run.planned.as_ref()) {
                        execution_items.push(ExecutionItemV3::new(
                            run.planned.item_id(),
                            family,
                            run.relation,
                            run.graph,
                            run.check,
                            run.presentation,
                            run.composition,
                            plan,
                        ));
                    }
                    items.push(CompiledItemV2::Planned(run.planned));
                }
                ItemRunV2::NoPlan { item_id, reason } => {
                    items.push(CompiledItemV2::NoPlan { item_id, reason });
                }
                ItemRunV2::Blocked { failure, .. } => {
                    return Err(FrontendTransactionErrorV2::Document(failure.deduplicated()));
                }
            }
        }
        Ok((items, self.rule_program, execution_items))
    }

    /// The exact analysis item list; blocked stages stay explicit.
    pub(super) fn into_analysis(
        self,
        source_id: SourceIdV2,
        max_diagnostics: u32,
    ) -> Result<AnalysisDocumentV2, FrontendTransactionErrorV2> {
        // Count canonical diagnostics before a blocked rule-program diagnostic
        // is cloned onto every authored rule member.
        let mut diagnostics = self
            .rule_blocked
            .as_ref()
            .map(|failure| failure.diagnostics().to_vec())
            .unwrap_or_default();
        for item in &self.items {
            if let ItemRunV2::Blocked { failure, .. } = item {
                diagnostics.extend(failure.diagnostics().iter().cloned());
            }
        }
        enforce_diagnostic_limit(
            source_id,
            canonicalize_diagnostics(diagnostics).len(),
            max_diagnostics,
        )?;
        let rule_blocked = self.rule_blocked;
        let mut order = Vec::with_capacity(self.items.len());
        let items = self
            .items
            .into_iter()
            .map(|item| match item {
                ItemRunV2::Planned(run) => {
                    let planned = run.planned;
                    if let PlannedItemV2::RuleMember(member) = planned.as_ref() {
                        order.push(member.member_item_id());
                        match &rule_blocked {
                            Some(failure) => {
                                AnalysisItemV2::BlockedByDiagnostics(failure.diagnostics().into())
                            }
                            None => AnalysisItemV2::Complete(planned),
                        }
                    } else {
                        order.push(planned.item_id());
                        AnalysisItemV2::Complete(planned)
                    }
                }
                ItemRunV2::NoPlan { item_id, reason } => {
                    order.push(item_id);
                    AnalysisItemV2::NotApplicable(reason)
                }
                ItemRunV2::Blocked { item_id, failure } => {
                    order.push(item_id);
                    AnalysisItemV2::BlockedByDiagnostics(failure.diagnostics().into())
                }
            })
            .collect::<Vec<_>>();
        Ok(AnalysisDocumentV2 {
            source_id,
            order: order.into_boxed_slice(),
            items: items.into_boxed_slice(),
            rule_program: self.rule_program,
            unresolved: Box::new([]),
        })
    }
}

/// Run every applicable stage of one sealed document.
///
/// Only membership inconsistency is a hard error here; every owner stage
/// failure is collected so the analysis path can report it per item while the
/// compile path fails closed on the same collection.
pub(super) fn run_document(
    document: &SemanticDocumentV2,
) -> Result<DocumentRunV2, FrontendTransactionErrorV2> {
    let membership = DocumentMembershipV2::of(document)?;
    let rule_members = membership.rule_member_ids();
    let rule_program_id = membership.rule_program_id();
    let mut algebra = AlgebraSessionV2::new(document);
    let mut planning = PlanningSessionV2::new(document);

    let (rule_program, rule_blocked) = match rule_program_id {
        Some(program_id) => {
            match compile_rule_program(document, &mut planning, program_id, &rule_members) {
                Ok(program) => (Some(program), None),
                Err(FrontendTransactionErrorV2::Document(failure)) => (None, Some(failure)),
                Err(error @ FrontendTransactionErrorV2::Authority(_)) => return Err(error),
            }
        }
        None => (None, None),
    };

    let mut items = Vec::with_capacity(membership.order().len());
    let mut produced_plan: BTreeSet<SemanticItemIdV2> = BTreeSet::new();
    let mut produced_member: BTreeSet<SemanticItemIdV2> = BTreeSet::new();
    let mut produced_no_plan: BTreeSet<SemanticItemIdV2> = BTreeSet::new();
    let mut blocked_stage = false;
    for id in membership.order() {
        let location = membership.location(*id);
        match run_item(document, &membership, &mut algebra, &mut planning, *id) {
            Ok(ItemRunV2::Planned(run)) => {
                if matches!(run.planned.as_ref(), PlannedItemV2::RuleMember(_)) {
                    record_produced_identity(
                        &mut produced_member,
                        *id,
                        location,
                        "one authored identity produced two results",
                    )?;
                } else {
                    record_produced_identity(
                        &mut produced_plan,
                        *id,
                        location,
                        "one authored identity produced two results",
                    )?;
                }
                items.push(ItemRunV2::Planned(run));
            }
            Ok(ItemRunV2::NoPlan { item_id, reason }) => {
                record_produced_identity(
                    &mut produced_no_plan,
                    *id,
                    location,
                    "one authored identity produced two non-plan results",
                )?;
                items.push(ItemRunV2::NoPlan { item_id, reason });
            }
            Ok(blocked @ ItemRunV2::Blocked { .. }) => {
                blocked_stage = true;
                items.push(blocked);
            }
            Err(FrontendTransactionErrorV2::Document(failure)) => {
                blocked_stage = true;
                items.push(ItemRunV2::Blocked {
                    item_id: *id,
                    failure,
                });
            }
            Err(error @ FrontendTransactionErrorV2::Authority(_)) => return Err(error),
        }
    }

    // A blocked owner stage is reported first: the produced key sets are
    // incomplete by construction, so comparing them would mask the real stage
    // failure with a pipeline membership code.
    if blocked_stage || rule_blocked.is_some() {
        return Ok(DocumentRunV2 {
            items,
            rule_program,
            rule_blocked,
        });
    }

    validate_produced_membership(
        document.source_id(),
        &membership,
        &produced_plan,
        &produced_member,
        &produced_no_plan,
    )?;

    Ok(DocumentRunV2 {
        items,
        rule_program,
        rule_blocked,
    })
}

pub(super) fn record_produced_identity(
    produced: &mut BTreeSet<SemanticItemIdV2>,
    id: SemanticItemIdV2,
    location: LocatedSpanV2,
    message: &'static str,
) -> Result<(), FrontendTransactionErrorV2> {
    if produced.insert(id) {
        Ok(())
    } else {
        Err(assembly(
            location,
            DocumentAssemblyCodeV2::DuplicateItemResult,
            message,
        ))
    }
}

pub(super) fn validate_produced_membership(
    source_id: SourceIdV2,
    membership: &DocumentMembershipV2<'_>,
    produced_plan: &BTreeSet<SemanticItemIdV2>,
    produced_member: &BTreeSet<SemanticItemIdV2>,
    produced_no_plan: &BTreeSet<SemanticItemIdV2>,
) -> Result<(), FrontendTransactionErrorV2> {
    let expected_plan = membership.plan_required_ids();
    let expected_member: BTreeSet<SemanticItemIdV2> =
        membership.rule_member_ids().into_iter().collect();
    let expected_no_plan = membership.no_plan_ids();
    if produced_member != &expected_member {
        let anchor = expected_member
            .iter()
            .chain(produced_member.iter())
            .find(|id| expected_member.contains(id) != produced_member.contains(id))
            .copied();
        let location =
            anchor.map_or_else(|| zero_location(source_id), |id| membership.location(id));
        return Err(assembly(
            location,
            DocumentAssemblyCodeV2::RuleMembershipMismatch,
            "the produced rule members disagree with the authored rule set",
        ));
    }
    for id in &expected_plan {
        if !produced_plan.contains(id) {
            return Err(assembly(
                membership.location(*id),
                DocumentAssemblyCodeV2::MissingItemResult,
                "an applicable authored item produced no plan result",
            ));
        }
    }
    for id in produced_plan {
        if !expected_plan.contains(id) {
            return Err(assembly(
                membership.location(*id),
                DocumentAssemblyCodeV2::ExtraItemResult,
                "a produced plan has no applicable authored identity",
            ));
        }
    }
    for id in &expected_no_plan {
        if !produced_no_plan.contains(id) {
            return Err(assembly(
                membership.location(*id),
                DocumentAssemblyCodeV2::MissingItemResult,
                "a non-executable declaration produced no checked non-plan reason",
            ));
        }
    }
    for id in produced_no_plan {
        if !expected_no_plan.contains(id) {
            return Err(assembly(
                membership.location(*id),
                DocumentAssemblyCodeV2::ExtraItemResult,
                "a produced non-plan result has no authored declaration identity",
            ));
        }
    }
    Ok(())
}

/// Run the applicable stages for one authored item.
fn run_item(
    document: &SemanticDocumentV2,
    membership: &DocumentMembershipV2<'_>,
    algebra: &mut AlgebraSessionV2<'_>,
    planning: &mut PlanningSessionV2<'_>,
    id: SemanticItemIdV2,
) -> Result<ItemRunV2, FrontendTransactionErrorV2> {
    let Some(item) = membership.item(id) else {
        return Err(assembly(
            membership.location(id),
            DocumentAssemblyCodeV2::MissingItemResult,
            "an authored item has no produced semantic result",
        ));
    };
    match item {
        SemanticItemV2::Command(command) => {
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Command(command.clone()),
            )?;
            Ok(planned_run(PlannedItemV2::Command { item_id: id, plan }))
        }
        SemanticItemV2::Relation(_) => {
            let lowered = algebra
                .lower_relation_item(id)
                .map_err(|error| algebra_error(document, id, &error))?;
            let relation = project_relation(document, &lowered)
                .map_err(|failure| closure_error(document, id, failure))?;
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Relational(Box::new(lowered)),
            )?;
            Ok(ItemRunV2::Planned(Box::new(PlannedRunV2 {
                planned: Box::new(PlannedItemV2::Relation { item_id: id, plan }),
                relation: Some(relation),
                graph: None,
                check: None,
                presentation: None,
                composition: None,
            })))
        }
        SemanticItemV2::Graph(graph) => {
            let contract = GraphQueryContractV2::from_resolved(graph)
                .map_err(|error| logic_transaction_error(&error))?;
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Graph(contract.clone()),
            )?;
            Ok(ItemRunV2::Planned(Box::new(PlannedRunV2 {
                planned: Box::new(PlannedItemV2::Graph { item_id: id, plan }),
                relation: None,
                graph: Some(project_graph_query(&contract)),
                check: None,
                presentation: None,
                composition: None,
            })))
        }
        SemanticItemV2::Check(check) => {
            let contract = check_contract(check)?;
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Check(contract.clone()),
            )?;
            Ok(ItemRunV2::Planned(Box::new(PlannedRunV2 {
                planned: Box::new(PlannedItemV2::Check { item_id: id, plan }),
                relation: None,
                graph: None,
                check: Some(project_check(&contract)),
                presentation: None,
                composition: None,
            })))
        }
        SemanticItemV2::Presentation(presentation) => {
            let contract = PresentationContractV2::from_resolved(presentation)
                .map_err(|error| planning_transaction_error(&error))?;
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Presentation(contract.clone()),
            )?;
            Ok(ItemRunV2::Planned(Box::new(PlannedRunV2 {
                planned: Box::new(PlannedItemV2::Presentation { item_id: id, plan }),
                relation: None,
                graph: None,
                check: None,
                presentation: Some(project_presentation(&contract)),
                composition: None,
            })))
        }
        SemanticItemV2::Composition(composition) => {
            let mut producer_projections = Vec::with_capacity(composition.bindings().len());
            for binding in composition.bindings() {
                let projection = match binding.producer() {
                    ResolvedCompositionProducerV2::Query(query) => {
                        let root = algebra
                            .lower_nested_relation(composition.item_id(), query.root())
                            .map_err(|error| algebra_error(document, id, &error))?;
                        ExecutionCompositionProducerProjectionV3::Relation(
                            project_relation_root(document, query.item_id(), &root)
                                .map_err(|failure| closure_error(document, id, failure))?,
                        )
                    }
                    ResolvedCompositionProducerV2::Sample { .. } => {
                        ExecutionCompositionProducerProjectionV3::Sample
                    }
                    ResolvedCompositionProducerV2::Graph { graph, .. } => {
                        let contract = GraphQueryContractV2::from_resolved(graph)
                            .map_err(|error| logic_transaction_error(&error))?;
                        ExecutionCompositionProducerProjectionV3::Graph {
                            item_id: contract.item_id(),
                            query: project_graph_query(&contract),
                        }
                    }
                };
                producer_projections.push(projection);
            }
            let (body, body_projection) = composition_body_input(document, algebra, composition)?;
            let input = CheckedCompositionPlanInputV2::new(composition, body)
                .map_err(|error| planning_transaction_error(&error))?;
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Composition(Box::new(input)),
            )?;
            let execution = project_composition(composition, producer_projections, body_projection)
                .map_err(|failure| composition_projection_error(document, id, failure))?;
            Ok(ItemRunV2::Planned(Box::new(PlannedRunV2 {
                planned: Box::new(PlannedItemV2::Composition { item_id: id, plan }),
                relation: None,
                graph: None,
                check: None,
                presentation: None,
                composition: Some(execution),
            })))
        }
        SemanticItemV2::Meta(meta) => match meta.kind() {
            ResolvedMetaKindV2::RequireCertified => {
                let plan = planning_plan(
                    document,
                    planning,
                    id,
                    &SemanticPlanInputV2::MetaRequireCertified(meta.clone()),
                )?;
                Ok(planned_run(PlannedItemV2::MetaRequireCertified {
                    item_id: id,
                    plan,
                }))
            }
            ResolvedMetaKindV2::UsePack { .. } => Ok(ItemRunV2::NoPlan {
                item_id: id,
                reason: NoPlanReasonV2::MetaActivation,
            }),
        },
        SemanticItemV2::Rule(_) => match membership.rule_program_id() {
            Some(program_id) => Ok(planned_run(PlannedItemV2::RuleMember(
                RuleProgramMemberV2::new(program_id, id),
            ))),
            None => Err(assembly(
                membership.location(id),
                DocumentAssemblyCodeV2::RuleMembershipMismatch,
                "a source rule has no document rule program",
            )),
        },
        SemanticItemV2::Catalog(catalog) => {
            if let Some(view) = catalog.inline_view() {
                algebra
                    .lower_nested_relation(id, view.root())
                    .map_err(|error| algebra_error(document, id, &error))?;
            }
            Ok(ItemRunV2::NoPlan {
                item_id: id,
                reason: NoPlanReasonV2::CatalogDeclaration {
                    kind: catalog_declaration_kind(catalog),
                },
            })
        }
        SemanticItemV2::Declaration(_) => Ok(ItemRunV2::NoPlan {
            item_id: id,
            reason: NoPlanReasonV2::ValueDeclaration,
        }),
    }
}

/// Build and plan the one document-wide rule program.
///
/// Stratification is NOT repeated here: the program's sole ingress
/// ([`RuleProgramContractV2::from_document`]) already charged the document's
/// rule-graph allowance once and retained the proof, and this DTO carries that
/// exact proof.
fn compile_rule_program(
    document: &SemanticDocumentV2,
    planning: &mut PlanningSessionV2<'_>,
    program_id: SemanticItemIdV2,
    members: &[SemanticItemIdV2],
) -> Result<CompiledRuleProgramV2, FrontendTransactionErrorV2> {
    let contract = RuleProgramContractV2::from_document(document)
        .map_err(|error| logic_transaction_error(&error))?;
    let stratification = contract.stratification().clone();
    let plan = planning_plan(
        document,
        planning,
        program_id,
        &SemanticPlanInputV2::Rule(contract),
    )?;
    Ok(CompiledRuleProgramV2 {
        program_id,
        members: members
            .iter()
            .map(|member| RuleProgramMemberV2::new(program_id, *member))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        plan,
        stratification,
    })
}

/// Build one CHECK contract with its identity-checked nested graph contract.
fn check_contract(check: &ResolvedCheckV2) -> Result<CheckContractV2, FrontendTransactionErrorV2> {
    let graph = match check.nested_graph() {
        Some(nested) => Some(
            GraphQueryContractV2::from_resolved(nested)
                .map_err(|error| logic_transaction_error(&error))?,
        ),
        None => None,
    };
    CheckContractV2::from_resolved(check, graph).map_err(|error| planning_transaction_error(&error))
}

/// Build one composition body plan input.
fn composition_body_input(
    document: &SemanticDocumentV2,
    algebra: &mut AlgebraSessionV2<'_>,
    composition: &ResolvedCompositionV2,
) -> Result<
    (
        CompositionBodyPlanInputV2,
        ExecutionCompositionBodyProjectionV3,
    ),
    FrontendTransactionErrorV2,
> {
    let owner = composition.item_id();
    match composition.body() {
        ResolvedCompositionBodyV2::Query(body) => {
            let root = algebra
                .lower_nested_relation(owner, body.root())
                .map_err(|error| algebra_error(document, owner, &error))?;
            let execution = project_relation_root(document, body.item_id(), &root)
                .map_err(|failure| closure_error(document, owner, failure))?;
            let body = CompositionRelationalBodyPlanInputV2::try_new(body.item_id(), root)
                .map_err(|reason| {
                    planning_transaction_error(&PlanningDiagnosticV1::InvalidComposition {
                        reason,
                        span: composition.location().clone(),
                    })
                })?;
            Ok((
                CompositionBodyPlanInputV2::Relational(body),
                ExecutionCompositionBodyProjectionV3::Relation(execution),
            ))
        }
        ResolvedCompositionBodyV2::Graph(graph) => {
            let contract = GraphQueryContractV2::from_resolved(graph)
                .map_err(|error| logic_transaction_error(&error))?;
            let execution = project_graph_query(&contract);
            let item_id = contract.item_id();
            Ok((
                CompositionBodyPlanInputV2::Graph(contract),
                ExecutionCompositionBodyProjectionV3::Graph {
                    item_id,
                    query: execution,
                },
            ))
        }
        ResolvedCompositionBodyV2::Check(check) => Ok((
            CompositionBodyPlanInputV2::Check(check_contract(check)?),
            ExecutionCompositionBodyProjectionV3::Check,
        )),
        ResolvedCompositionBodyV2::Explain(explain) => Ok((
            CompositionBodyPlanInputV2::Explain(
                PresentationContractV2::from_resolved(explain)
                    .map_err(|error| planning_transaction_error(&error))?,
            ),
            ExecutionCompositionBodyProjectionV3::Explain,
        )),
    }
}

fn planned_run(planned: PlannedItemV2) -> ItemRunV2 {
    ItemRunV2::Planned(Box::new(PlannedRunV2 {
        planned: Box::new(planned),
        relation: None,
        graph: None,
        check: None,
        presentation: None,
        composition: None,
    }))
}

fn execution_identity(planned: &PlannedItemV2) -> Option<(ExecutionFamilyV3, &SemanticPlanV2)> {
    match planned {
        PlannedItemV2::Command { plan, .. } => Some((ExecutionFamilyV3::Command, plan)),
        PlannedItemV2::Relation { plan, .. } => Some((ExecutionFamilyV3::Relation, plan)),
        PlannedItemV2::Graph { plan, .. } => Some((ExecutionFamilyV3::Graph, plan)),
        PlannedItemV2::Check { plan, .. } => Some((ExecutionFamilyV3::Check, plan)),
        PlannedItemV2::Presentation { plan, .. } => Some((ExecutionFamilyV3::Presentation, plan)),
        PlannedItemV2::Composition { plan, .. } => Some((ExecutionFamilyV3::Composition, plan)),
        PlannedItemV2::MetaRequireCertified { plan, .. } => {
            Some((ExecutionFamilyV3::MetaRequireCertified, plan))
        }
        PlannedItemV2::RuleMember(_) => None,
    }
}

/// Map one resolved catalog payload to its closed non-plan reason kind.
fn catalog_declaration_kind(catalog: &ResolvedCatalogItemV2) -> CatalogDeclarationKindV2 {
    match catalog.payload() {
        ResolvedCatalogPayloadV2::View { .. } => CatalogDeclarationKindV2::View,
        ResolvedCatalogPayloadV2::Model { .. } => CatalogDeclarationKindV2::Model,
        ResolvedCatalogPayloadV2::Pack { .. } => CatalogDeclarationKindV2::Pack,
        ResolvedCatalogPayloadV2::Profile { .. } => CatalogDeclarationKindV2::Profile,
        ResolvedCatalogPayloadV2::Gate { .. } => CatalogDeclarationKindV2::Gate,
        ResolvedCatalogPayloadV2::RuleBinding { .. } => CatalogDeclarationKindV2::RuleBinding,
    }
}

/// Plan one input and require the plan-required outcome.
fn planning_plan(
    document: &SemanticDocumentV2,
    planning: &mut PlanningSessionV2<'_>,
    id: SemanticItemIdV2,
    input: &SemanticPlanInputV2,
) -> Result<SemanticPlanV2, FrontendTransactionErrorV2> {
    match planning.plan_outcome(input) {
        Ok(PlanOutcomeV2::Plan(plan)) => Ok(*plan),
        Ok(PlanOutcomeV2::NoPlan(_reason)) => Err(assembly(
            item_location_of(document, id),
            DocumentAssemblyCodeV2::PlanOutcomeMismatch,
            "a plan-required item returned a checked non-plan outcome",
        )),
        Err(error) => Err(planning_transaction_error(&error)),
    }
}

/// Project one algebra diagnostic with an explicit same-document fallback.
fn algebra_error(
    document: &SemanticDocumentV2,
    id: SemanticItemIdV2,
    error: &quanta_gqlang_v2_algebra::AlgebraDiagnosticV2,
) -> FrontendTransactionErrorV2 {
    algebra_transaction_error(error, item_location_of(document, id))
}

/// Map an execution expression-closure failure to a stable assembly error.
pub(super) fn closure_error(
    document: &SemanticDocumentV2,
    id: SemanticItemIdV2,
    failure: ExecutionClosureFailureV3,
) -> FrontendTransactionErrorV2 {
    let location = item_location_of(document, id);
    match failure {
        ExecutionClosureFailureV3::Dangling => assembly(
            location,
            DocumentAssemblyCodeV2::DanglingExecutionReference,
            "an executable item references an expression the sealed document does not carry",
        ),
        ExecutionClosureFailureV3::UnregisteredSymbol => assembly(
            location,
            DocumentAssemblyCodeV2::UnregisteredExecutionSymbol,
            "an executable expression's closed identity has no registered symbol",
        ),
        ExecutionClosureFailureV3::Foreign => assembly(
            location,
            DocumentAssemblyCodeV2::ForeignItemResult,
            "an executable item references an expression from another source",
        ),
    }
}

/// Map an impossible producer/body projection drift to a typed pipeline
/// diagnostic instead of truncating a binding or aborting the process.
fn composition_projection_error(
    document: &SemanticDocumentV2,
    id: SemanticItemIdV2,
    failure: ExecutionCompositionProjectionFailureV3,
) -> FrontendTransactionErrorV2 {
    let (code, message) = match failure {
        ExecutionCompositionProjectionFailureV3::Producer => (
            DocumentAssemblyCodeV2::CompositionProducerMismatch,
            "a composition producer and its checked execution projection disagree",
        ),
        ExecutionCompositionProjectionFailureV3::Body => (
            DocumentAssemblyCodeV2::CompositionBodyMismatch,
            "a composition body and its checked execution projection disagree",
        ),
    };
    assembly(item_location_of(document, id), code, message)
}

/// Mint one pipeline-owned assembly failure.
fn assembly(
    location: LocatedSpanV2,
    code: DocumentAssemblyCodeV2,
    message: &'static str,
) -> FrontendTransactionErrorV2 {
    assembly_transaction_error(location, code, message)
}

/// The only location a fully empty document can offer.
pub(super) fn zero_location(source_id: SourceIdV2) -> LocatedSpanV2 {
    LocatedSpanV2 {
        source_id,
        span: SpanV1::point(0),
    }
}

/// The authored location of one semantic item.
///
/// A catalog declaration retains its checked location through its inline VIEW
/// where it has one, and otherwise through the checked
/// [`quanta_gqlang_v2_sema::ResolvedCatalogReferenceV2`] its payload resolved;
/// both are document-owned facts, so a declaration is never anchored on a
/// fabricated byte 0.
fn item_location(item: &SemanticItemV2) -> Option<LocatedSpanV2> {
    match item {
        SemanticItemV2::Command(command) => Some(command.location().location()),
        SemanticItemV2::Relation(relation) => Some(relation.location().location()),
        SemanticItemV2::Graph(graph) => Some(graph.location().location()),
        SemanticItemV2::Check(check) => Some(check.rule().location().location()),
        SemanticItemV2::Presentation(presentation) => Some(presentation.location().location()),
        SemanticItemV2::Composition(composition) => Some(composition.location().location()),
        SemanticItemV2::Meta(meta) => Some(meta.location().location()),
        SemanticItemV2::Catalog(catalog) => catalog_location(catalog),
        SemanticItemV2::Declaration(declaration) => Some(declaration.body().location().location()),
        SemanticItemV2::Rule(rule) => Some(rule.body().expression().location().location()),
    }
}

/// The checked location of one catalog declaration, from the facts it retains.
///
/// A View alias/inline body anchors on the inline VIEW's own typed relation; a
/// declaration without one anchors on its payload's exact checked reference (the
/// queried/source/check reference the declaration resolved), falling back to the
/// first canonical inline-View dependency. Only a catalog item that resolves no
/// reference at all has no location of its own.
fn catalog_location(catalog: &ResolvedCatalogItemV2) -> Option<LocatedSpanV2> {
    if let Some(view) = catalog.inline_view() {
        return Some(view.location().location());
    }
    let payload_reference = match catalog.payload() {
        ResolvedCatalogPayloadV2::View { query_ref } => query_ref.as_ref(),
        ResolvedCatalogPayloadV2::Model { source } => Some(source),
        ResolvedCatalogPayloadV2::Pack { source, .. } => Some(source),
        ResolvedCatalogPayloadV2::Profile { capabilities, .. } => capabilities.first(),
        ResolvedCatalogPayloadV2::Gate { checks, .. } => checks.first(),
        ResolvedCatalogPayloadV2::RuleBinding { check_ref } => Some(check_ref),
    };
    payload_reference
        .or_else(|| catalog.inline_view_dependencies().first())
        .map(|reference| reference.location().location())
}

/// A located anchor for one authored identity.
pub(super) fn item_location_of(
    document: &SemanticDocumentV2,
    id: SemanticItemIdV2,
) -> LocatedSpanV2 {
    document
        .item(id)
        .and_then(item_location)
        .unwrap_or_else(|| zero_location(document.source_id()))
}

// ---------------------------------------------------------------------------
// Exact-key membership
// ---------------------------------------------------------------------------

/// The authored membership of one sealed document, verified exactly once.
pub(super) struct DocumentMembershipV2<'a> {
    items: BTreeMap<SemanticItemIdV2, &'a SemanticItemV2>,
    order: Vec<SemanticItemIdV2>,
    rule_program_id: Option<SemanticItemIdV2>,
}

impl<'a> DocumentMembershipV2<'a> {
    /// Verify duplicate, missing, extra, foreign, and wrong-kind membership.
    pub(super) fn of(document: &'a SemanticDocumentV2) -> Result<Self, FrontendTransactionErrorV2> {
        let mut items: BTreeMap<SemanticItemIdV2, &'a SemanticItemV2> = BTreeMap::new();
        for (key, item) in document.items() {
            if key.source_id() != document.source_id() || item.item_id() != *key {
                return Err(assembly(
                    item_location(item).unwrap_or_else(|| zero_location(document.source_id())),
                    DocumentAssemblyCodeV2::WrongKindItemResult,
                    "a produced item is registered under a foreign or mismatched identity",
                ));
            }
            if items.insert(*key, item).is_some() {
                return Err(assembly(
                    item_location(item).unwrap_or_else(|| zero_location(document.source_id())),
                    DocumentAssemblyCodeV2::DuplicateItemResult,
                    "a produced item identity was registered twice",
                ));
            }
        }
        let mut order = Vec::with_capacity(document.order().len());
        let mut seen = BTreeSet::new();
        for id in document.order() {
            if id.source_id() != document.source_id() {
                return Err(assembly(
                    zero_location(document.source_id()),
                    DocumentAssemblyCodeV2::ForeignItemResult,
                    "the authored order names an item of another source",
                ));
            }
            if !seen.insert(*id) {
                return Err(assembly(
                    items
                        .get(id)
                        .and_then(|item| item_location(item))
                        .unwrap_or_else(|| zero_location(document.source_id())),
                    DocumentAssemblyCodeV2::DuplicateItemResult,
                    "the authored order names one item twice",
                ));
            }
            if !items.contains_key(id) {
                return Err(assembly(
                    zero_location(document.source_id()),
                    DocumentAssemblyCodeV2::MissingItemResult,
                    "an authored item has no produced semantic result",
                ));
            }
            order.push(*id);
        }
        if let Some(extra) = items.keys().find(|id| !seen.contains(id)) {
            let location = items
                .get(extra)
                .and_then(|item| item_location(item))
                .unwrap_or_else(|| zero_location(document.source_id()));
            return Err(assembly(
                location,
                DocumentAssemblyCodeV2::ExtraItemResult,
                "a produced semantic item was not authored in document order",
            ));
        }
        let rule_program_id = order.iter().copied().find(|id| {
            items
                .get(id)
                .is_some_and(|item| matches!(item, SemanticItemV2::Rule(_)))
        });
        Ok(Self {
            items,
            order,
            rule_program_id,
        })
    }

    /// Authored source order.
    fn order(&self) -> &[SemanticItemIdV2] {
        &self.order
    }

    /// Exact authored item by identity, present for every verified order id.
    fn item(&self, id: SemanticItemIdV2) -> Option<&'a SemanticItemV2> {
        self.items.get(&id).copied()
    }

    /// A located anchor for one authored identity.
    fn location(&self, id: SemanticItemIdV2) -> LocatedSpanV2 {
        self.item(id)
            .and_then(item_location)
            .unwrap_or_else(|| zero_location(id.source_id()))
    }

    /// The stable document rule program identity (first rule in order).
    pub(super) fn rule_program_id(&self) -> Option<SemanticItemIdV2> {
        self.rule_program_id
    }

    /// Authored rule identities in source order.
    pub(super) fn rule_member_ids(&self) -> Vec<SemanticItemIdV2> {
        self.order
            .iter()
            .copied()
            .filter(|id| {
                self.item(*id)
                    .is_some_and(|item| matches!(item, SemanticItemV2::Rule(_)))
            })
            .collect()
    }

    /// Identities that must carry one semantic plan.
    fn plan_required_ids(&self) -> BTreeSet<SemanticItemIdV2> {
        self.order
            .iter()
            .copied()
            .filter(|id| {
                self.item(*id).is_some_and(|item| {
                    matches!(
                        item,
                        SemanticItemV2::Relation(_)
                            | SemanticItemV2::Command(_)
                            | SemanticItemV2::Graph(_)
                            | SemanticItemV2::Check(_)
                            | SemanticItemV2::Presentation(_)
                            | SemanticItemV2::Composition(_)
                    ) || matches!(
                        item,
                        SemanticItemV2::Meta(meta)
                            if matches!(meta.kind(), ResolvedMetaKindV2::RequireCertified)
                    )
                })
            })
            .collect()
    }

    /// Identities that must carry one checked non-plan reason.
    fn no_plan_ids(&self) -> BTreeSet<SemanticItemIdV2> {
        self.order
            .iter()
            .copied()
            .filter(|id| {
                self.item(*id).is_some_and(|item| {
                    matches!(
                        item,
                        SemanticItemV2::Catalog(_) | SemanticItemV2::Declaration(_)
                    ) || matches!(
                        item,
                        SemanticItemV2::Meta(meta)
                            if matches!(meta.kind(), ResolvedMetaKindV2::UsePack { .. })
                    )
                })
            })
            .collect()
    }
}
