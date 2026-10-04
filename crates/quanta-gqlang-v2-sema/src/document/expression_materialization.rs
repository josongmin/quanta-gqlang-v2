//! Typed expression and predicate materialization.

use super::{
    capture_vector_witness, document_failure, location, materialize_relation,
    validate_capture_vector, AggregateContextV2, AggregatePositionV2, BTreeMap, BindResultV1,
    BinderKindV1, BoolExprHirV1, BoolExprKindV1, ClosedFrontendAuthorityV2, ExpressionTypeStateV2,
    ExtendedSourceRegistryViewV2, FrontendFailureV2, HirNodeIdV1, RelationOutputContractV2,
    ResolvedBindingKindV2, ResolvedTargetV1, ResolvedValueBindingV2, SemanticStepCounterV2,
    SourceIdV2, SourceQualifiedNodeIdV2, TypedBoolExprV2, TypedCallArgumentV2,
    TypedCorrelationEdgeV2, TypedExprShapeV2, TypedExprV2, TypedExpressionFactsV2,
    TypedPathSegmentV2, TypedRecordFieldV2, TypedRelationExprV2, ValueExprHirV1, ValueExprKindV1,
    ViewMaterializationInputsV2,
};

#[expect(
    clippy::too_many_lines,
    reason = "legacy expression materialization awaits phase extraction"
)]
pub(super) fn materialize_declaration_expr(
    node: &ValueExprHirV1,
    source_id: SourceIdV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &ExtendedSourceRegistryViewV2,
    output: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    steps: &mut SemanticStepCounterV2,
    position: AggregatePositionV2,
) -> Result<TypedExprV2, FrontendFailureV2> {
    steps.charge(source_id, node.span)?;
    let key_of = |id| SourceQualifiedNodeIdV2 {
        source_id,
        node_id: id,
    };
    let mut child = |child: &ValueExprHirV1, output: &mut BTreeMap<_, _>| {
        materialize_declaration_expr(
            child, source_id, bound, facts, extended, output, steps, position,
        )
        .map(|typed| typed.key())
    };
    let key = key_of(node.id);
    let fact = facts.get(key).ok_or_else(|| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            node.span,
            "types owner omitted expression fact",
        )
    })?;
    if !matches!(fact.state, ExpressionTypeStateV2::Known(_)) {
        return Err(document_failure(
            "sema.unknown_type_in_closed_compile",
            source_id,
            node.span,
            "closed expression cannot retain unknown type",
        ));
    }
    // The types owner records the authored `-2^127` expression as the checked
    // `i128::MIN` value on the unary node. Its positive magnitude child is only
    // a parsing/type-checking sentinel and must not become executable IR: doing
    // so would encode a second, overflowing negation of an already-minimum
    // value. Preserve the authored root identity while materializing the
    // authoritative checked literal fact.
    let shape = match &node.kind {
        ValueExprKindV1::Unary(unary) if fact.literal.is_some() => {
            // Retain a typed proof for every checked HIR node. The sentinel remains
            // in the semantic document but is unreachable from the folded root, so
            // execution closure projection excludes it.
            let _ = child(&unary.operand, output)?;
            TypedExprShapeV2::Literal(fact.literal.clone().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted folded unary literal fact",
                )
            })?)
        }
        ValueExprKindV1::Literal(_) => {
            TypedExprShapeV2::Literal(fact.literal.clone().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted literal fact",
                )
            })?)
        }
        ValueExprKindV1::Symbol(_) => {
            let (owner, ordinal, kind, name) = match bound.target_of(node.id) {
                Some(ResolvedTargetV1::Declaration {
                    name, node: owner, ..
                }) => (*owner, 0, ResolvedBindingKindV2::Declaration, name.clone()),
                Some(ResolvedTargetV1::Local(binding)) => {
                    let (owner, ordinal) = binding.identity();
                    let kind = match binding.kind {
                        BinderKindV1::Alias => ResolvedBindingKindV2::Alias,
                        BinderKindV1::Param => ResolvedBindingKindV2::Parameter,
                        BinderKindV1::Let => ResolvedBindingKindV2::Local,
                        BinderKindV1::Quantified => ResolvedBindingKindV2::Quantified,
                        BinderKindV1::CompositionLet => ResolvedBindingKindV2::Composition,
                    };
                    (owner, ordinal, kind, binding.name.clone())
                }
                _ => {
                    return Err(document_failure(
                        "sema.unresolved_binding_type",
                        source_id,
                        node.span,
                        "binding lacks a closed canonical identity",
                    ))
                }
            };
            let binding = ResolvedValueBindingV2::try_new(source_id, owner, ordinal, kind, name)
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        error,
                    )
                })?;
            TypedExprShapeV2::Binding(binding)
        }
        ValueExprKindV1::Path(path) => {
            let base = child(&path.base, output)?;
            let mut segments = Vec::new();
            for segment in path.segments.as_slice() {
                let raw = location(source_id, segment.span);
                let checked = extended
                    .combined()
                    .try_check_location(raw)
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            segment.span,
                            error.to_string(),
                        )
                    })?;
                segments.push(TypedPathSegmentV2::new(segment.name.clone(), checked));
            }
            TypedExprShapeV2::Path { base, segments }
        }
        ValueExprKindV1::Unary(unary) => TypedExprShapeV2::Unary {
            operator: fact.operator_id.ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted unary operator",
                )
            })?,
            operand: child(&unary.operand, output)?,
        },
        ValueExprKindV1::Binary(binary) => TypedExprShapeV2::Binary {
            operator: fact.operator_id.ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted binary operator",
                )
            })?,
            lhs: child(&binary.lhs, output)?,
            rhs: child(&binary.rhs, output)?,
        },
        ValueExprKindV1::Call(call) => {
            let mut arguments = Vec::new();
            for arg in call.args().as_slice() {
                arguments.push(match arg {
                    quanta_gqlang_v2_hir::CallArgHirV1::Expr(expr) => {
                        TypedCallArgumentV2::Expression(child(expr, output)?)
                    }
                    quanta_gqlang_v2_hir::CallArgHirV1::Star { .. } => TypedCallArgumentV2::Star,
                });
            }
            let target = fact.resolved_call().cloned().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted callable target",
                )
            })?;
            TypedExprShapeV2::Call { target, arguments }
        }
        ValueExprKindV1::Paren(inner) => TypedExprShapeV2::Paren {
            inner: child(inner, output)?,
        },
        ValueExprKindV1::Record(record) => {
            let mut fields = Vec::new();
            for field in &record.fields {
                let value = child(&field.value, output)?;
                let raw = location(source_id, field.name_span);
                let checked = extended
                    .combined()
                    .try_check_location(raw)
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            field.name_span,
                            error.to_string(),
                        )
                    })?;
                fields.push(
                    TypedRecordFieldV2::try_new(field.name.clone(), checked, value).map_err(
                        |error| {
                            document_failure(
                                "sema.document_validation_failed",
                                source_id,
                                field.name_span,
                                error,
                            )
                        },
                    )?,
                );
            }
            TypedExprShapeV2::Record(fields)
        }
        ValueExprKindV1::List(list) => {
            let mut elements = Vec::new();
            for element in &list.elements {
                elements.push(child(element, output)?);
            }
            TypedExprShapeV2::List(elements)
        }
    };
    let location = location(source_id, node.span);
    let star = matches!(&shape, TypedExprShapeV2::Call { arguments, .. } if arguments.iter().any(|arg| matches!(arg, TypedCallArgumentV2::Star)));
    let context = AggregateContextV2::try_new(position, fact.effects.aggregation, star, location)?;
    let checked = extended
        .combined()
        .try_check_location(location)
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                node.span,
                error.to_string(),
            )
        })?;
    let typed = TypedExprV2::try_from_fact(key, fact, context, shape, checked)?;
    if output.insert(key, typed.clone()).is_some() {
        return Err(document_failure(
            "sema.document_validation_failed",
            source_id,
            node.span,
            "duplicate typed expression identity",
        ));
    }
    Ok(typed)
}

#[expect(
    clippy::too_many_lines,
    reason = "legacy boolean materialization awaits phase extraction"
)]
pub(super) fn materialize_bool(
    node: &BoolExprHirV1,
    source_id: SourceIdV2,
    authority: &ClosedFrontendAuthorityV2,
    bound: &BindResultV1,
    facts: &TypedExpressionFactsV2,
    extended: &ExtendedSourceRegistryViewV2,
    composition_outputs: &BTreeMap<(HirNodeIdV1, usize), RelationOutputContractV2>,
    views: &ViewMaterializationInputsV2<'_>,
    output: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedExprV2>,
    nested_relations: &mut BTreeMap<SourceQualifiedNodeIdV2, TypedRelationExprV2>,
    steps: &mut SemanticStepCounterV2,
    position: AggregatePositionV2,
) -> Result<TypedBoolExprV2, FrontendFailureV2> {
    steps.charge(source_id, node.span)?;
    let key = |id| SourceQualifiedNodeIdV2 {
        source_id,
        node_id: id,
    };
    let fact = facts.get(key(node.id)).ok_or_else(|| {
        document_failure(
            "sema.document_validation_failed",
            source_id,
            node.span,
            "types owner omitted rule boolean fact",
        )
    })?;
    let shape = match &node.kind {
        BoolExprKindV1::Literal(_) => {
            TypedExprShapeV2::Literal(fact.literal.clone().ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted boolean literal fact",
                )
            })?)
        }
        BoolExprKindV1::Value(value) => {
            let value = materialize_declaration_expr(
                value, source_id, bound, facts, extended, output, steps, position,
            )?;
            TypedExprShapeV2::BoolValue { value: value.key() }
        }
        BoolExprKindV1::Compare(compare) => {
            let lhs = materialize_declaration_expr(
                &compare.lhs,
                source_id,
                bound,
                facts,
                extended,
                output,
                steps,
                position,
            )?;
            let rhs = materialize_declaration_expr(
                &compare.rhs,
                source_id,
                bound,
                facts,
                extended,
                output,
                steps,
                position,
            )?;
            TypedExprShapeV2::Compare {
                operator: fact.operator_id.ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        "types owner omitted comparison operator",
                    )
                })?,
                lhs: lhs.key(),
                rhs: rhs.key(),
            }
        }
        BoolExprKindV1::And(binary) | BoolExprKindV1::Or(binary) => {
            let lhs = materialize_bool(
                &binary.lhs,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
                position,
            )?;
            let rhs = materialize_bool(
                &binary.rhs,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
                position,
            )?;
            let operator = fact.operator_id.ok_or_else(|| {
                document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "types owner omitted logical operator",
                )
            })?;
            if matches!(node.kind, BoolExprKindV1::And(_)) {
                TypedExprShapeV2::And {
                    operator,
                    lhs: lhs.expression().key(),
                    rhs: rhs.expression().key(),
                }
            } else {
                TypedExprShapeV2::Or {
                    operator,
                    lhs: lhs.expression().key(),
                    rhs: rhs.expression().key(),
                }
            }
        }
        BoolExprKindV1::Not(inner) => {
            let operand = materialize_bool(
                inner,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
                position,
            )?;
            TypedExprShapeV2::Not {
                operator: fact.operator_id.ok_or_else(|| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        "types owner omitted negation operator",
                    )
                })?,
                operand: operand.expression().key(),
            }
        }
        BoolExprKindV1::Paren(inner) => {
            let inner = materialize_bool(
                inner,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
                position,
            )?;
            TypedExprShapeV2::Paren {
                inner: inner.expression().key(),
            }
        }
        BoolExprKindV1::LogicalQuantifier(quantifier) => {
            let body = materialize_bool(
                &quantifier.body,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
                position,
            )?;
            let binders = quantifier
                .binders
                .as_slice()
                .iter()
                .enumerate()
                .map(|(ordinal, binder)| {
                    ResolvedValueBindingV2::try_new(
                        source_id,
                        node.id,
                        ordinal,
                        ResolvedBindingKindV2::Quantified,
                        binder.name.clone(),
                    )
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            binder.span,
                            error,
                        )
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            TypedExprShapeV2::LogicalQuantifier {
                kind: quantifier.kind.into(),
                binders,
                body: body.expression().key(),
            }
        }
        BoolExprKindV1::RelationExists(exists) => {
            if position == AggregatePositionV2::RuleBody {
                return Err(document_failure(
                    "sema.invalid_item_shape",
                    source_id,
                    node.span,
                    "relational EXISTS is not admitted inside a source rule body",
                ));
            }
            if !bound.has_capture_set(node.id) {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "Binder omitted the explicit relational EXISTS capture set",
                ));
            }
            let relation = materialize_relation(
                &exists.relation,
                source_id,
                authority,
                bound,
                facts,
                extended,
                composition_outputs,
                views,
                output,
                nested_relations,
                steps,
            )?;
            let relation_root = SourceQualifiedNodeIdV2 {
                source_id,
                node_id: exists.relation.id,
            };
            if relation.key() != relation_root {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    exists.relation.span,
                    "nested relational EXISTS root differs from checked HIR identity",
                ));
            }
            if nested_relations.insert(relation_root, relation).is_some() {
                return Err(document_failure(
                    "sema.document_validation_failed",
                    source_id,
                    node.span,
                    "duplicate nested relational EXISTS root",
                ));
            }
            let witness = capture_vector_witness(bound, node.id, exists.relation.id);
            let edges = validate_capture_vector(node.id, exists.relation.id, &witness, bound)
                .map_err(|message| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        message,
                    )
                })?;
            let mut correlations = Vec::new();
            for edge in edges {
                let Some(resolution) = bound
                    .resolutions()
                    .iter()
                    .find(|resolution| resolution.reference == edge.reference)
                else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        node.span,
                        "Binder capture reference lacks its exact resolution proof",
                    ));
                };
                let ResolvedTargetV1::Local(binding) = &resolution.target else {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        resolution.span,
                        "Binder capture target is not a local binding",
                    ));
                };
                if binding.identity() != (edge.owner, edge.ordinal) {
                    return Err(document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        resolution.span,
                        "Binder capture owner or ordinal differs from its resolution proof",
                    ));
                }
                let kind = match binding.kind {
                    BinderKindV1::Alias => ResolvedBindingKindV2::Alias,
                    BinderKindV1::Param => ResolvedBindingKindV2::Parameter,
                    BinderKindV1::Let => ResolvedBindingKindV2::Local,
                    BinderKindV1::Quantified => ResolvedBindingKindV2::Quantified,
                    BinderKindV1::CompositionLet => ResolvedBindingKindV2::Composition,
                };
                let binding = ResolvedValueBindingV2::try_new(
                    source_id,
                    edge.owner,
                    edge.ordinal,
                    kind,
                    binding.name.clone(),
                )
                .map_err(|error| {
                    document_failure(
                        "sema.document_validation_failed",
                        source_id,
                        resolution.span,
                        error,
                    )
                })?;
                let location = extended
                    .combined()
                    .try_check_location(location(source_id, resolution.span))
                    .map_err(|error| {
                        document_failure(
                            "sema.document_validation_failed",
                            source_id,
                            resolution.span,
                            error.to_string(),
                        )
                    })?;
                correlations.push(TypedCorrelationEdgeV2::new(
                    binding,
                    SourceQualifiedNodeIdV2 {
                        source_id,
                        node_id: edge.reference,
                    },
                    location,
                ));
            }
            TypedExprShapeV2::RelationExists {
                polarity: exists.polarity.into(),
                relation_root,
                correlations,
            }
        }
    };
    let raw = location(source_id, node.span);
    let context = AggregateContextV2::try_new(position, fact.effects.aggregation, false, raw)?;
    let checked = extended
        .combined()
        .try_check_location(raw)
        .map_err(|error| {
            document_failure(
                "sema.document_validation_failed",
                source_id,
                node.span,
                error.to_string(),
            )
        })?;
    let typed = TypedExprV2::try_from_fact(key(node.id), fact, context, shape, checked)?;
    let bool_fact = TypedBoolExprV2::try_new(typed.clone(), raw)?;
    if output.insert(key(node.id), typed).is_some() {
        return Err(document_failure(
            "sema.document_validation_failed",
            source_id,
            node.span,
            "duplicate typed boolean identity",
        ));
    }
    Ok(bool_fact)
}
