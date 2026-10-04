//! Source-ordered semantic item materialization after type and View proofs.

use super::super::{
    classify_rule_body_calls, document_failure, location, materialize_bool,
    materialize_declaration_expr, materialize_query, materialize_query_composition, module_failure,
    resolve_check_item, resolve_graph_item, resolve_presentation_item, AggregatePositionV2,
    BTreeMap, BindResultV1, BodyResolutionV2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    DeclaredDocumentV2, DocumentCatalogFinalInputV2, DocumentCatalogValidatorV2,
    ExpressionTypeStateV2, FrontendTransactionErrorV2, InlineViewPlanV2, LockReferenceInputV2,
    PackActivationInputV2, PackIdV2, ProgramItemHirV1, RequireCertifiedInputV2,
    ResolvedDeclarationV2, ResolvedMetaKindV2, ResolvedMetaV2, ResolvedRuleV2,
    RulePredicateInputV2, SemanticItemIdV2, SemanticItemV2, SemanticStepCounterV2,
    SourceCatalogAssetsV3, SourceQualifiedNodeIdV2, TypedExprV2, TypedExpressionFactsV2,
    TypedRelationExprV2, UsePackActivationV2, ViewMaterializationInputsV2,
};
use super::catalog::resolve_catalog_declaration;
use super::inline_views::InlineViewBodiesV2;
use super::prepare::PreparedBodyTypesV2;
use quanta_gqlang_v2_types::RuleTypeProofV2;

pub(super) fn materialize_source_order(
    declared: &DeclaredDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    predeclared: &SourceCatalogAssetsV3,
    inline_views: &mut InlineViewPlanV2,
    validator: &mut DocumentCatalogValidatorV2<'_>,
    steps: &mut SemanticStepCounterV2,
    prepared: PreparedBodyTypesV2,
    mut inline: InlineViewBodiesV2,
) -> Result<BodyResolutionV2, FrontendTransactionErrorV2> {
    let checked = declared.checked();
    let bound = &prepared.bound;
    let facts = &prepared.facts;
    let mut items = Vec::new();
    let mut expressions = inline.expressions;
    let mut nested_relations = inline.nested_relations;
    let mut certified = Vec::new();
    let mut activations = Vec::new();
    let mut family_references = Vec::new();
    let mut profile_admissions = Vec::new();
    let available_views = ViewMaterializationInputsV2 {
        source_targets: &inline_views.source_targets,
        outputs: &inline.outputs,
    };
    let empty_composition_graph_targets = BTreeMap::new();
    for item in &checked.checked().document().items {
        steps.charge(checked.source_id(), item.span())?;
        match item {
            ProgramItemHirV1::Command(command) => items.push(resolve_rename_command(
                command, checked, authority, extended, steps,
            )?),
            ProgramItemHirV1::Query(query) => {
                let typed = materialize_query(
                    query,
                    checked.source_id(),
                    authority,
                    bound,
                    facts,
                    extended,
                    &BTreeMap::new(),
                    &available_views,
                    &mut expressions,
                    &mut nested_relations,
                    steps,
                )?;
                items.push(SemanticItemV2::Relation(typed));
            }
            ProgramItemHirV1::GraphQuery(graph) => {
                items.push(SemanticItemV2::Graph(resolve_graph_item(
                    graph,
                    checked.source_id(),
                    authority,
                    bound,
                    &empty_composition_graph_targets,
                    extended,
                    steps,
                )?))
            }
            ProgramItemHirV1::Presentation(presentation) => {
                items.push(SemanticItemV2::Presentation(resolve_presentation_item(
                    presentation,
                    checked.source_id(),
                    authority,
                    extended,
                    validator,
                    steps,
                )?))
            }
            ProgramItemHirV1::Legality(check) => {
                items.push(SemanticItemV2::Check(Box::new(resolve_check_item(
                    check,
                    checked.source_id(),
                    authority,
                    bound,
                    &empty_composition_graph_targets,
                    extended,
                    validator,
                    steps,
                )?)))
            }
            ProgramItemHirV1::CatalogDecl(decl) => {
                items.push(SemanticItemV2::Catalog(resolve_catalog_declaration(
                    decl,
                    checked,
                    predeclared,
                    &mut inline_views.direct_references,
                    &mut inline.views,
                    extended,
                    validator,
                    steps,
                    &mut family_references,
                    &mut profile_admissions,
                )?))
            }
            ProgramItemHirV1::Decl(decl) => items.push(resolve_declaration(
                decl,
                checked,
                bound,
                facts,
                extended,
                &mut expressions,
                steps,
            )?),
            ProgramItemHirV1::Rule(rule) => items.push(resolve_rule(
                rule,
                checked,
                authority,
                bound,
                facts,
                extended,
                &available_views,
                &prepared.predicates,
                &prepared.rule_proof,
                &mut expressions,
                &mut nested_relations,
                steps,
            )?),
            ProgramItemHirV1::Meta(meta)
                if matches!(
                    meta.kind,
                    quanta_gqlang_v2_hir::MetaKindV1::RequireCertified
                ) =>
            {
                items.push(resolve_require_certified(
                    meta,
                    checked,
                    extended,
                    &mut certified,
                )?)
            }
            ProgramItemHirV1::Meta(meta) => {
                items.push(resolve_use_pack(meta, checked, extended, &mut activations)?)
            }
            ProgramItemHirV1::Composition(composition) => {
                let typed = materialize_query_composition(
                    composition,
                    checked.source_id(),
                    authority,
                    bound,
                    facts,
                    extended,
                    validator,
                    &available_views,
                    &mut expressions,
                    &mut nested_relations,
                    steps,
                )?;
                items.push(SemanticItemV2::Composition(typed));
            }
        }
    }
    Ok(BodyResolutionV2 {
        items,
        expressions,
        nested_relations,
        type_fact_count: facts.len(),
        catalog_input: DocumentCatalogFinalInputV2 {
            profile_admissions,
            activations,
            require_certified_directives: certified,
            family_references,
        },
    })
}

fn resolve_rename_command(
    command: &quanta_gqlang_v2_hir::RenameCommandHirV3,
    checked: &CheckedSyntaxDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    steps: &mut SemanticStepCounterV2,
) -> Result<SemanticItemV2, FrontendTransactionErrorV2> {
    let source_id = checked.source_id();
    steps.charge(source_id, command.target.span)?;
    let Some(target) = authority.symbol_target(&command.target.name) else {
        return Err(document_failure(
            "sema.unknown_symbol_target",
            source_id,
            command.target.span,
            "rename target is absent from the closed symbol authority",
        )
        .into());
    };
    let valid_phase_token = match command.phase {
        quanta_gqlang_v2_hir::RenamePhaseV3::Preview => command.preview_token.is_none(),
        quanta_gqlang_v2_hir::RenamePhaseV3::Apply => command
            .preview_token
            .as_ref()
            .is_some_and(|value| !value.is_empty()),
    };
    if command.new_name.is_empty()
        || command.new_name.chars().any(char::is_control)
        || command.expected_snapshot.is_empty()
        || !valid_phase_token
    {
        return Err(document_failure(
            "sema.invalid_rename_command",
            source_id,
            command.span,
            "rename intent has an invalid name, snapshot, or phase/token combination",
        )
        .into());
    }
    let location = extended
        .combined()
        .try_check_location(location(source_id, command.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                command.span,
                error.to_string(),
            )
        })?;
    Ok(SemanticItemV2::Command(
        crate::ResolvedRenameCommandV3::new(
            SemanticItemIdV2::new(source_id, command.id),
            location,
            target.clone(),
            command.new_name.clone(),
            command.phase,
            command.expected_snapshot.clone(),
            command.preview_token.clone(),
        ),
    ))
}

fn resolve_declaration(
    decl: &quanta_gqlang_v2_hir::DeclHirV1,
    checked: &CheckedSyntaxDocumentV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    expressions: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    steps: &mut SemanticStepCounterV2,
) -> Result<SemanticItemV2, FrontendTransactionErrorV2> {
    let typed = materialize_declaration_expr(
        &decl.body,
        checked.source_id(),
        bound,
        facts,
        extended,
        expressions,
        steps,
        AggregatePositionV2::Scalar,
    )?;
    let declaration = ResolvedDeclarationV2::try_new(
        SemanticItemIdV2::new(checked.source_id(), decl.id),
        decl.name.clone(),
        typed,
    )
    .map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            checked.source_id(),
            decl.span,
            error,
        )
    })?;
    Ok(SemanticItemV2::Declaration(declaration))
}

fn resolve_rule(
    rule: &quanta_gqlang_v2_hir::RuleHirV1,
    checked: &CheckedSyntaxDocumentV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    available_views: &ViewMaterializationInputsV2<'_>,
    predicates: &[RulePredicateInputV2],
    rule_proof: &Option<RuleTypeProofV2>,
    expressions: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &mut SemanticStepCounterV2,
) -> Result<SemanticItemV2, FrontendTransactionErrorV2> {
    let Some(predicate) = predicates
        .iter()
        .find(|predicate| predicate.rule_node.node_id == rule.id)
    else {
        return Err(document_failure(
            "sema.document_validation_failed",
            checked.source_id(),
            rule.span,
            "rule lacks a types-owned predicate predeclaration",
        )
        .into());
    };
    let head = materialize_declaration_expr(
        &rule.head,
        checked.source_id(),
        bound,
        facts,
        extended,
        expressions,
        steps,
        AggregatePositionV2::RuleBody,
    )?;
    let body = materialize_bool(
        &rule.body,
        checked.source_id(),
        authority,
        bound,
        facts,
        extended,
        &BTreeMap::new(),
        available_views,
        expressions,
        nested_relations,
        steps,
        AggregatePositionV2::RuleBody,
    )?;
    let Some(proof) = &rule_proof else {
        return Err(document_failure(
            "sema.document_validation_failed",
            checked.source_id(),
            rule.span,
            "rule type proof is absent from the document transaction",
        )
        .into());
    };
    if !matches!(head.type_state(), ExpressionTypeStateV2::Known(ty)
                    if proof.predicate_type(&predicate.predicate_id) == Some(ty))
    {
        return Err(document_failure(
            "sema.document_validation_failed",
            checked.source_id(),
            rule.head.span,
            "rule head type disagrees with the predicate signature proof",
        )
        .into());
    }
    let calls = classify_rule_body_calls(
        body.expression().key(),
        |key| expressions.get(&key),
        |key| nested_relations.get(&key),
    )
    .map_err(|error| {
        document_failure(
            "sema.invalid_item_shape",
            checked.source_id(),
            error
                .location()
                .map_or(rule.body.span, |location| location.location().span),
            error.reason(),
        )
    })?;
    let typed = ResolvedRuleV2::try_new(
        SemanticItemIdV2::new(checked.source_id(), rule.id),
        head.key(),
        body.expression().key(),
        predicate.predicate_id.clone(),
        head,
        body,
        calls,
    )
    .map_err(|error| {
        document_failure(
            "sema.document_validation_failed",
            checked.source_id(),
            rule.span,
            error,
        )
    })?;
    Ok(SemanticItemV2::Rule(typed))
}

fn resolve_require_certified(
    meta: &quanta_gqlang_v2_hir::MetaHirV1,
    checked: &CheckedSyntaxDocumentV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    certified: &mut Vec<RequireCertifiedInputV2>,
) -> Result<SemanticItemV2, FrontendTransactionErrorV2> {
    let location = quanta_gqlang_v2_hir::LocatedSpanV2 {
        source_id: checked.source_id(),
        span: meta.span,
    };
    let checked_location = extended
        .combined()
        .try_check_location(location)
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                meta.span,
                error.to_string(),
            )
        })?;
    certified.push(RequireCertifiedInputV2 {
        location: checked_location.clone(),
    });
    Ok(SemanticItemV2::Meta(ResolvedMetaV2::new(
        SemanticItemIdV2::new(checked.source_id(), meta.id),
        checked_location,
        ResolvedMetaKindV2::RequireCertified,
    )))
}

fn resolve_use_pack(
    meta: &quanta_gqlang_v2_hir::MetaHirV1,
    checked: &CheckedSyntaxDocumentV2,
    extended: &quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    activations: &mut Vec<PackActivationInputV2>,
) -> Result<SemanticItemV2, FrontendTransactionErrorV2> {
    let quanta_gqlang_v2_hir::MetaKindV1::UsePack(use_pack) = &meta.kind else {
        unreachable!("REQUIRE CERTIFIED is handled by the preceding arm");
    };
    let checked_location = extended
        .combined()
        .try_check_location(location(checked.source_id(), meta.span))
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                checked.source_id(),
                meta.span,
                error.to_string(),
            )
        })?;
    let pack =
        PackIdV2::try_new(&use_pack.pack_id, checked_location.clone()).map_err(module_failure)?;
    let activation = match use_pack.activation {
        quanta_gqlang_v2_hir::UsePackActivationV1::Session => UsePackActivationV2::Session,
        quanta_gqlang_v2_hir::UsePackActivationV1::Workspace => UsePackActivationV2::Workspace,
    };
    activations.push(PackActivationInputV2 {
        pack_id: pack.clone(),
        activation,
        lock_reference: LockReferenceInputV2 {
            path: use_pack.lockfile.clone(),
            location: checked_location.clone(),
        },
        location: checked_location.clone(),
    });
    Ok(SemanticItemV2::Meta(ResolvedMetaV2::new(
        SemanticItemIdV2::new(checked.source_id(), meta.id),
        checked_location,
        ResolvedMetaKindV2::UsePack {
            pack,
            activation: use_pack.activation,
        },
    )))
}
