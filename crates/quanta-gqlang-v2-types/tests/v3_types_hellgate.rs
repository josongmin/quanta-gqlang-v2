#![expect(
    clippy::unwrap_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{
    check_document_v2, CallArgHirV1, CallArgsHirV1, CheckedHirDocumentV2, DeclHirV1, HirBuilderV1,
    HirDocumentV1, HirNodeIdV1, LogicalBinderV1, LogicalQuantifierKindV1, NonEmptyBindersV1,
    ProgramItemHirV1, RelationExistsPolarityV1, RuleHirV1, SourceIdV2, SourceRegistrationV2,
    SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_types::*;

fn resolved_type_inputs(
    checked: CheckedHirDocumentV2<'_>,
    entries: Vec<ResolvedTypeInputEntryV2>,
    contexts: Vec<ContextualExpectedTypeInputV2>,
) -> Result<ResolvedTypeInputsV2<'_>, TypeDiagnosticV2> {
    let registry = SourceRegistryViewV2::try_new(Vec::new())
        .unwrap()
        .try_extend_document(SourceRegistrationV2 {
            source_id: checked.document().source_id,
            source_len: checked.document().source_len,
        })
        .unwrap();
    ResolvedTypeInputsV2::try_new(checked, &registry, entries, contexts, Vec::new())
}

fn resolved_type_inputs_with_rows(
    checked: CheckedHirDocumentV2<'_>,
    entries: Vec<ResolvedTypeInputEntryV2>,
    contexts: Vec<ContextualExpectedTypeInputV2>,
    rows: Vec<DeferredRelationRowInputV2>,
) -> Result<ResolvedTypeInputsV2<'_>, TypeDiagnosticV2> {
    let registry = SourceRegistryViewV2::try_new(Vec::new())
        .unwrap()
        .try_extend_document(SourceRegistrationV2 {
            source_id: checked.document().source_id,
            source_len: checked.document().source_len,
        })
        .unwrap();
    ResolvedTypeInputsV2::try_new(checked, &registry, entries, contexts, rows)
}

fn checked_document(body: quanta_gqlang_v2_hir::ValueExprHirV1) -> HirDocumentV1 {
    HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9),
        source_len: 100,
        items: vec![ProgramItemHirV1::Decl(Box::new(DeclHirV1::new(
            HirNodeIdV1::from_raw(10_000),
            SpanV1::new(0, 90),
            "x".into(),
            SpanV1::new(0, 1),
            body,
        )))],
    }
}

fn checked_rule(
    head: quanta_gqlang_v2_hir::ValueExprHirV1,
    body: quanta_gqlang_v2_hir::BoolExprHirV1,
) -> HirDocumentV1 {
    HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9),
        source_len: 100,
        items: vec![ProgramItemHirV1::Rule(Box::new(RuleHirV1::new(
            HirNodeIdV1::from_raw(10_000),
            SpanV1::new(0, 90),
            "r".into(),
            SpanV1::new(0, 1),
            head,
            body,
        )))],
    }
}

#[test]
fn resolved_inputs_reject_missing_duplicate_foreign_and_extra_entries() {
    let builder = HirBuilderV1::new();
    let symbol = builder.symbol("name", SpanV1::new(1, 5));
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: symbol.id,
    };
    let doc = checked_document(symbol);
    let checked = check_document_v2(&doc).unwrap();
    assert_eq!(
        resolved_type_inputs(checked, vec![], vec![]).unwrap_err(),
        TypeDiagnosticV2::MissingNodeFact {
            node_id: key.node_id,
            location: quanta_gqlang_v2_hir::LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(1, 5),
            },
        }
    );
    let entry = ResolvedTypeInputEntryV2 {
        key,
        value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
            ValueTypeV1::string(),
        )),
    };
    assert_eq!(
        resolved_type_inputs(checked, vec![entry.clone(), entry.clone()], vec![]).unwrap_err(),
        TypeDiagnosticV2::DuplicateNodeFact {
            node_id: key.node_id,
            location: quanta_gqlang_v2_hir::LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(1, 5),
            },
        }
    );
    let extra = ResolvedTypeInputEntryV2 {
        key: SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: HirNodeIdV1::from_raw(777),
        },
        value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
            ValueTypeV1::string(),
        )),
    };
    assert_eq!(
        resolved_type_inputs(checked, vec![entry, extra], vec![]).unwrap_err(),
        TypeDiagnosticV2::ExtraNodeFact {
            node_id: HirNodeIdV1::from_raw(777),
        }
    );
    let foreign = ResolvedTypeInputEntryV2 {
        key: SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(10),
            node_id: key.node_id,
        },
        value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
            ValueTypeV1::string(),
        )),
    };
    assert_eq!(
        resolved_type_inputs(checked, vec![foreign], vec![]).unwrap_err(),
        TypeDiagnosticV2::ForeignDocumentFact {
            node_id: key.node_id,
            location: quanta_gqlang_v2_hir::LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(10),
                span: SpanV1::point(0),
            },
        }
    );
}

#[test]
fn kernel_selects_stable_operator_and_builtin_ids() {
    let builder = HirBuilderV1::new();
    let lhs = builder.symbol("a", SpanV1::new(0, 1));
    let lhs_id = lhs.id;
    let rhs = builder.int("1", SpanV1::new(4, 5));
    let rhs_id = rhs.id;
    let sum = builder.binary("+", lhs, rhs, SpanV1::new(0, 5));
    let sum_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: sum.id,
    };
    let doc = checked_document(sum);
    let entry = ResolvedTypeInputEntryV2 {
        key: SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: lhs_id,
        },
        value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(ValueTypeV1::int())),
    };
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![entry], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    let sum_fact = facts.get(sum_key).unwrap();
    assert_eq!(sum_fact.operator_id.map(OperatorIdV2::raw), Some(101));
    assert_eq!(
        sum_fact.operator_id.and_then(OperatorIdV2::symbol),
        Some("+")
    );
    assert_eq!(
        facts
            .get(SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(9),
                node_id: rhs_id,
            })
            .unwrap()
            .literal,
        Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Int(1))),
        "the integer literal operand must retain its typed literal fact"
    );
    assert_eq!(sum_fact.literal, None);

    let builder = HirBuilderV1::new();
    let arg = builder.int("1", SpanV1::new(6, 7));
    let call = builder.call(
        "count",
        SpanV1::new(0, 5),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
        SpanV1::new(0, 8),
    );
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let doc = checked_document(call);
    let registry = BuiltinRegistryV2::try_closed().unwrap();
    let input = ResolvedTypeInputEntryV2 {
        key,
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
            registry.family_id("count").unwrap(),
        )),
    };
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert!(matches!(
        facts.get(key).unwrap().resolved_call(),
        Some(ResolvedCallTargetV2::Builtin(id)) if id.raw() == 1 && id.name() == Some("count")
    ));
}

#[test]
fn resolved_inputs_reject_extra_and_wrong_kind_before_kernel_walk() {
    let builder = HirBuilderV1::new();
    let symbol = builder.symbol("x", SpanV1::new(0, 1));
    let symbol_id = symbol.id;
    let doc = checked_document(symbol);
    let checked = check_document_v2(&doc).unwrap();
    let registry = BuiltinRegistryV2::try_closed().unwrap();
    let wrong = ResolvedTypeInputEntryV2 {
        key: SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: symbol_id,
        },
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
            registry.family_id("count").unwrap(),
        )),
    };
    assert_eq!(
        resolved_type_inputs(checked, vec![wrong], vec![])
            .unwrap_err()
            .code(),
        "types.wrong_resolved_input_kind"
    );
    let extra = ResolvedTypeInputEntryV2 {
        key: SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: HirNodeIdV1::from_raw(777),
        },
        value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(ValueTypeV1::int())),
    };
    assert_eq!(
        resolved_type_inputs(checked, vec![extra], vec![])
            .unwrap_err()
            .code(),
        "types.extra_resolved_input"
    );
}

#[test]
fn star_is_accepted_only_for_explicit_count_star_signature() {
    let registry = BuiltinRegistryV2::try_closed().unwrap();
    for (name, expected_ok) in [("count", true), ("sum", false), ("starts_with", false)] {
        let builder = HirBuilderV1::new();
        let call = builder.call(
            name,
            SpanV1::new(0, name.len()),
            CallArgsHirV1::try_new(vec![CallArgHirV1::star(SpanV1::new(
                name.len() + 1,
                name.len() + 2,
            ))])
            .unwrap(),
            SpanV1::new(0, name.len() + 3),
        );
        let key = SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: call.id,
        };
        let doc = checked_document(call);
        let input = ResolvedTypeInputEntryV2 {
            key,
            value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
                registry.family_id(name).unwrap(),
            )),
        };
        let inputs =
            resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
        let result = ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs);
        if expected_ok {
            assert!(matches!(
                result.unwrap().get(key).unwrap().resolved_call(),
                Some(ResolvedCallTargetV2::Builtin(id)) if id.raw() == 2 && id.name() == Some("count")
            ));
        } else {
            assert_eq!(result.unwrap_err().code(), "types.invalid_star_argument");
        }
    }

    let builder = HirBuilderV1::new();
    let call = builder.call(
        "p",
        SpanV1::new(0, 1),
        CallArgsHirV1::try_new(vec![CallArgHirV1::star(SpanV1::new(2, 3))]).unwrap(),
        SpanV1::new(0, 4),
    );
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let callable = ResolvedPredicateCallableV2::try_new(
        ResolvedPredicateIdV2::try_new("pkg::p").unwrap(),
        vec![ValueTypeV1::int()],
        ValueTypeV1::bool(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .unwrap();
    let input = ResolvedTypeInputEntryV2 {
        key,
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Predicate(callable)),
    };
    let doc = checked_document(call);
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    assert_eq!(
        ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs)
            .unwrap_err()
            .code(),
        "types.invalid_star_argument"
    );
}

#[test]
fn unknown_and_bad_operator_reject_distinctly() {
    for operator in ["~", "not"] {
        let builder = HirBuilderV1::new();
        let operand = builder.string("x", SpanV1::new(1, 4));
        let unary = builder.unary(operator, operand, SpanV1::new(0, 4));
        let unary_id = unary.id;
        let doc = checked_document(unary);
        let inputs =
            resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
        let error = ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs)
            .unwrap_err();
        let location = quanta_gqlang_v2_hir::LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(9),
            span: SpanV1::new(0, 4),
        };
        let expected = if operator == "~" {
            TypeDiagnosticV2::UnknownOperator {
                node_id: unary_id,
                symbol: "~".into(),
                location,
            }
        } else {
            TypeDiagnosticV2::BadOverload {
                node_id: unary_id,
                name: "not".into(),
                operands: vec![ValueTypeV1::string()],
                location,
            }
        };
        assert_eq!(error, expected, "wrong diagnostic for {operator}");
    }
}

#[test]
fn unused_contextual_input_rejects_during_kernel_finalization() {
    let builder = HirBuilderV1::new();
    let list = builder.list(vec![], SpanV1::new(0, 2));
    let list_id = list.id;
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: list.id,
    };
    let doc = checked_document(list);
    let context = ContextualExpectedTypeInputV2 {
        key,
        expected_type: ValueTypeV1::int(),
        reason: ContextualTypeReasonV2::ExpectedBinding,
    };
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![context]).unwrap();
    assert_eq!(
        ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs)
            .unwrap_err(),
        TypeDiagnosticV2::UnusedContextualInput {
            node_id: list_id,
            location: quanta_gqlang_v2_hir::LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(0, 2),
            },
        }
    );
}

#[test]
fn callable_parameters_contextualize_empty_lists_without_external_context_facts() {
    let builder = HirBuilderV1::new();
    let list = builder.list(vec![], SpanV1::new(5, 7));
    let list_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: list.id,
    };
    let call = builder.call(
        "size",
        SpanV1::new(0, 4),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(list)]).unwrap(),
        SpanV1::new(0, 8),
    );
    let call_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let registry = BuiltinRegistryV2::try_closed().unwrap();
    let input = ResolvedTypeInputEntryV2 {
        key: call_key,
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
            registry.family_id("size").unwrap(),
        )),
    };
    let doc = checked_document(call);
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(
        facts.get(list_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::try_list(ValueTypeV1::int()).unwrap())
    );
    assert_eq!(
        facts.get(call_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::int())
    );

    let builder = HirBuilderV1::new();
    let list = builder.list(vec![], SpanV1::new(5, 7));
    let list_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: list.id,
    };
    let call = builder.call(
        "p",
        SpanV1::new(0, 1),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(list)]).unwrap(),
        SpanV1::new(0, 8),
    );
    let call_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let callable = ResolvedPredicateCallableV2::try_new(
        ResolvedPredicateIdV2::try_new("pkg::p").unwrap(),
        vec![ValueTypeV1::try_list(ValueTypeV1::string()).unwrap()],
        ValueTypeV1::bool(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .unwrap();
    let input = ResolvedTypeInputEntryV2 {
        key: call_key,
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Predicate(callable)),
    };
    let doc = checked_document(call);
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(
        facts.get(list_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::try_list(ValueTypeV1::string()).unwrap())
    );
}

#[test]
fn unknown_operands_select_only_unique_safe_operator_ids() {
    for (operator, expected_state, expected_id) in [
        (
            "not",
            ExpressionTypeStateV2::Known(ValueTypeV1::bool()),
            Some(7u32),
        ),
        (
            "+",
            ExpressionTypeStateV2::Unknown(UnknownTypeCauseV2::UnresolvedEditorBinding),
            None,
        ),
    ] {
        let builder = HirBuilderV1::new();
        let symbol = builder.symbol("x", SpanV1::new(1, 2));
        let symbol_id = symbol.id;
        let unary = builder.unary(operator, symbol, SpanV1::new(0, 2));
        let unary_key = SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: unary.id,
        };
        let input = ResolvedTypeInputEntryV2 {
            key: SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(9),
                node_id: symbol_id,
            },
            value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Unknown(
                UnknownTypeCauseV2::UnresolvedEditorBinding,
            )),
        };
        let doc = checked_document(unary);
        let inputs =
            resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
        let facts = ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs)
            .unwrap();
        let fact = facts.get(unary_key).unwrap();
        assert_eq!(fact.state, expected_state);
        assert_eq!(fact.operator_id.map(OperatorIdV2::raw), expected_id);
    }
}

#[test]
fn parent_nodes_conservatively_compose_call_effects() {
    let builder = HirBuilderV1::new();
    let call = builder.call(
        "now",
        SpanV1::new(1, 4),
        CallArgsHirV1::try_new(vec![]).unwrap(),
        SpanV1::new(1, 6),
    );
    let call_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let parent = builder.paren_value(call, SpanV1::new(0, 7));
    let parent_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: parent.id,
    };
    let builtins = BuiltinRegistryV2::try_closed().unwrap();
    let input = ResolvedTypeInputEntryV2 {
        key: call_key,
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
            builtins.family_id("now").unwrap(),
        )),
    };
    let doc = checked_document(parent);
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    let effects = facts.get(parent_key).unwrap().effects;
    assert_eq!(
        effects,
        ExpressionEffectsV2 {
            purity: PurityV1::Impure,
            determinism: DeterminismV1::NonDeterministic,
            monotonicity: MonotonicityV1::NonMonotone,
            aggregation: AggregationClassV1::Scalar,
            cost: CostClassV1::Constant,
        }
    );
}

#[test]
fn unresolved_callable_effects_are_fail_closed() {
    let builder = HirBuilderV1::new();
    let call = builder.call(
        "unresolved",
        SpanV1::new(0, 10),
        CallArgsHirV1::try_new(vec![]).unwrap(),
        SpanV1::new(0, 12),
    );
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let input = ResolvedTypeInputEntryV2 {
        key,
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Unknown(
            UnknownTypeCauseV2::UnresolvedEditorCallable,
        )),
    };
    let doc = checked_document(call);
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    let fact = facts.get(key).unwrap();
    assert_eq!(
        fact.state,
        ExpressionTypeStateV2::Unknown(UnknownTypeCauseV2::UnresolvedEditorCallable)
    );
    assert_eq!(fact.resolved_call(), None);
    assert_eq!(fact.effects.purity, PurityV1::Impure);
    assert_eq!(fact.effects.determinism, DeterminismV1::NonDeterministic);
    assert_eq!(fact.effects.monotonicity, MonotonicityV1::NonMonotone);
    assert_eq!(fact.effects.aggregation, AggregationClassV1::Aggregate);
    assert_eq!(fact.effects.cost, CostClassV1::SuperLinear);
}

#[test]
fn predicate_calls_are_checked_once_and_retain_opaque_identity_and_effects() {
    let builder = HirBuilderV1::new();
    let arg = builder.int("1", SpanV1::new(5, 6));
    let call = builder.call(
        "p",
        SpanV1::new(0, 1),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
        SpanV1::new(0, 7),
    );
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let predicate_id = ResolvedPredicateIdV2::try_new("pkg::p").unwrap();
    let effects = ExpressionEffectsV2 {
        purity: PurityV1::Impure,
        determinism: DeterminismV1::NonDeterministic,
        monotonicity: MonotonicityV1::NonMonotone,
        aggregation: AggregationClassV1::Scalar,
        cost: CostClassV1::Linear,
    };
    let callable = ResolvedPredicateCallableV2::try_new(
        predicate_id.clone(),
        vec![ValueTypeV1::int()],
        ValueTypeV1::bool(),
        effects,
    )
    .unwrap();
    let input = ResolvedTypeInputEntryV2 {
        key,
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Predicate(callable)),
    };
    let doc = checked_document(call);
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(
        facts.len(),
        2,
        "one fact per call node and per literal argument, with no re-typing"
    );
    let fact = facts.get(key).unwrap();
    assert_eq!(
        fact.state,
        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    );
    assert_eq!(fact.effects, effects);
    assert!(
        matches!(fact.resolved_call(), Some(ResolvedCallTargetV2::Predicate(id)) if id == &predicate_id)
    );
}

#[test]
fn host_predicate_key_is_bounded_exact_and_disjoint_from_source_ids() {
    let local = format!("p.{}", "x".repeat(300));
    let len = ResolvedHostPredicateKeyV2::checked_canonical_len(
        "pkg::inner",
        HostCallableNamespaceV2::Predicate,
        &local,
    )
    .unwrap();
    assert!(len > 256);
    assert_eq!(
        ResolvedHostPredicateKeyV2::try_from_catalog_parts(
            "pkg::inner",
            HostCallableNamespaceV2::Predicate,
            &local,
            len - 1
        )
        .unwrap_err()
        .code(),
        "types.host_predicate_identity_limit"
    );
    let exact = ResolvedHostPredicateKeyV2::try_from_catalog_parts(
        "pkg::inner",
        HostCallableNamespaceV2::Predicate,
        &local,
        len,
    )
    .unwrap();
    assert_eq!(exact.canonical_bytes().len(), len);
    let mut expected = Vec::new();
    expected.extend_from_slice(b"gqlang-host-predicate-v2\0");
    expected.extend_from_slice(&10u64.to_be_bytes());
    expected.extend_from_slice(b"pkg::inner");
    expected.push(1);
    expected.extend_from_slice(&u64::try_from(local.len()).unwrap().to_be_bytes());
    expected.extend_from_slice(local.as_bytes());
    assert_eq!(
        exact.canonical_bytes(),
        expected.as_slice(),
        "the host key encoding must pin prefix, big-endian lengths, kind tag, and payload"
    );
    assert_eq!(
        ResolvedPredicateIdV2::try_new(&local).unwrap_err().code(),
        "types.invalid_resolved_identity"
    );
    let other_split = ResolvedHostPredicateKeyV2::try_from_catalog_parts(
        "pkg",
        HostCallableNamespaceV2::Predicate,
        &format!("inner::{local}"),
        len + 8,
    )
    .unwrap();
    assert_ne!(exact, other_split);
    let other_kind = ResolvedHostPredicateKeyV2::try_from_catalog_parts(
        "pkg::inner",
        HostCallableNamespaceV2::Rule,
        &local,
        len,
    )
    .unwrap();
    assert_ne!(exact, other_kind);
    let mut expected_rule = Vec::new();
    expected_rule.extend_from_slice(b"gqlang-host-predicate-v2\0");
    expected_rule.extend_from_slice(&10u64.to_be_bytes());
    expected_rule.extend_from_slice(b"pkg::inner");
    expected_rule.push(2);
    expected_rule.extend_from_slice(&u64::try_from(local.len()).unwrap().to_be_bytes());
    expected_rule.extend_from_slice(local.as_bytes());
    assert_eq!(other_kind.canonical_bytes(), expected_rule.as_slice());
    assert_eq!(
        ResolvedHostPredicateKeyV2::try_from_catalog_parts(
            "",
            HostCallableNamespaceV2::Predicate,
            &local,
            len
        )
        .unwrap_err()
        .code(),
        "types.invalid_resolved_identity"
    );
}

#[test]
fn host_predicate_call_retains_exact_host_target_and_effects() {
    let builder = HirBuilderV1::new();
    let call = builder.call(
        "p",
        SpanV1::new(0, 1),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
            builder.int("1", SpanV1::new(5, 6)),
        )])
        .unwrap(),
        SpanV1::new(0, 7),
    );
    let node = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let len = ResolvedHostPredicateKeyV2::checked_canonical_len(
        "pkg::module",
        HostCallableNamespaceV2::Predicate,
        "p.with/slash",
    )
    .unwrap();
    let key = ResolvedHostPredicateKeyV2::try_from_catalog_parts(
        "pkg::module",
        HostCallableNamespaceV2::Predicate,
        "p.with/slash",
        len,
    )
    .unwrap();
    let effects = ExpressionEffectsV2 {
        purity: PurityV1::Impure,
        determinism: DeterminismV1::NonDeterministic,
        monotonicity: MonotonicityV1::NonMonotone,
        aggregation: AggregationClassV1::Scalar,
        cost: CostClassV1::Linear,
    };
    let callable = ResolvedHostPredicateCallableV2::try_new(
        key.clone(),
        vec![ValueTypeV1::int()],
        ValueTypeV1::bool(),
        effects,
    )
    .unwrap();
    let doc = checked_document(call);
    let inputs = resolved_type_inputs(
        check_document_v2(&doc).unwrap(),
        vec![ResolvedTypeInputEntryV2 {
            key: node,
            value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::HostPredicate(
                callable,
            )),
        }],
        vec![],
    )
    .unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    let fact = facts.get(node).unwrap();
    assert_eq!(
        fact.state,
        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    );
    assert_eq!(fact.effects, effects);
    assert!(
        matches!(fact.resolved_call(), Some(ResolvedCallTargetV2::HostPredicate(found)) if found == &key)
    );
}

#[test]
fn logical_quantifier_retains_complete_nested_predicate_effects() {
    let builder = HirBuilderV1::new();
    let call = builder.call(
        "p",
        SpanV1::new(10, 11),
        CallArgsHirV1::try_new(vec![]).unwrap(),
        SpanV1::new(10, 13),
    );
    let call_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let body = builder.bool_value(call, SpanV1::new(10, 13));
    let quantifier = builder.logical_quantifier(
        LogicalQuantifierKindV1::Exists,
        NonEmptyBindersV1::try_new(vec![LogicalBinderV1 {
            name: "x".to_owned(),
            span: SpanV1::new(5, 6),
        }])
        .unwrap(),
        body,
        SpanV1::new(0, 13),
    );
    let quantifier_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: quantifier.id,
    };
    let effects = ExpressionEffectsV2 {
        purity: PurityV1::Impure,
        determinism: DeterminismV1::NonDeterministic,
        monotonicity: MonotonicityV1::NonMonotone,
        aggregation: AggregationClassV1::Scalar,
        cost: CostClassV1::SuperLinear,
    };
    let predicate_id = ResolvedPredicateIdV2::try_new("pkg::p").unwrap();
    let callable = ResolvedPredicateCallableV2::try_new(
        predicate_id.clone(),
        vec![],
        ValueTypeV1::bool(),
        effects,
    )
    .unwrap();
    let input = ResolvedTypeInputEntryV2 {
        key: call_key,
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Predicate(callable)),
    };
    let doc = checked_rule(builder.int("1", SpanV1::new(20, 21)), quantifier);
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    let call_fact = facts.get(call_key).unwrap();
    assert_eq!(call_fact.effects, effects);
    assert_eq!(
        call_fact.state,
        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    );
    assert!(
        matches!(call_fact.resolved_call(), Some(ResolvedCallTargetV2::Predicate(id)) if id == &predicate_id)
    );
    assert_eq!(facts.get(quantifier_key).unwrap().effects, effects);
    assert_eq!(
        facts.get(quantifier_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    );
}

#[test]
fn relation_exists_requires_resolved_authority_and_types_nested_expressions() {
    let builder = HirBuilderV1::new();
    let nested_symbol = builder.symbol("active", SpanV1::new(20, 26));
    let nested_id = nested_symbol.id;
    let predicate = builder.bool_value(nested_symbol, SpanV1::new(20, 26));
    let source = builder.relation_source("users", SpanV1::new(10, 15), None, SpanV1::new(10, 15));
    let source_root = source.id;
    let relation = builder.relation_filter(source, predicate, SpanV1::new(10, 26));
    let relation_root = relation.id;
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        relation,
        SpanV1::new(5, 27),
    );
    let exists_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: exists.id,
    };
    let doc = checked_rule(builder.int("1", SpanV1::new(0, 1)), exists);
    let checked = check_document_v2(&doc).unwrap();
    assert_eq!(
        resolved_type_inputs_with_rows(
            checked,
            vec![ResolvedTypeInputEntryV2 {
                key: SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(9),
                    node_id: nested_id
                },
                value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
                    ValueTypeV1::bool()
                )),
            }],
            vec![],
            vec![DeferredRelationRowInputV2::known_source(
                SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(9),
                    node_id: source_root,
                },
                0,
                Vec::new(),
            )],
        )
        .unwrap_err()
        .code(),
        "types.missing_resolved_input"
    );

    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(9),
        source_len: 100,
    }])
    .unwrap();
    let namespace = ProvenanceNamespaceV2::try_new(
        "host",
        registry
            .try_check_location(quanta_gqlang_v2_hir::LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(10, 15),
            })
            .unwrap(),
    )
    .unwrap();
    let catalog = RelationCatalogV2::try_new(
        vec![RelationSourceInputV2 {
            authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
            provenance_namespace: namespace,
            qualified_name: "pkg::users".into(),
            lookup_name: "users".into(),
            row: RecordTypeV1::try_new(vec![]).unwrap(),
            cardinality: CardinalityV1::Set,
            provenance: "host".into(),
            location: registry
                .try_check_location(quanta_gqlang_v2_hir::LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(9),
                    span: SpanV1::new(10, 15),
                })
                .unwrap(),
        }],
        &registry,
        AuthorityBudgetProfileV2::try_new(128, 256, 1024, 64 * 1024, 1024).unwrap(),
    )
    .unwrap();
    let relation_source = catalog.get("users").unwrap();
    assert_eq!(relation_source.lookup_name(), "users");
    assert_eq!(
        relation_source.id().authority(),
        quanta_gqlang_v2_types::RelationAuthorityV2::Host
    );
    assert_eq!(relation_source.id().qualified_name(), "pkg::users");
    assert_eq!(relation_source.id().namespace().as_str(), "host");
    assert_eq!(relation_source.provenance(), "host");
    assert_eq!(
        relation_source.relation_type(),
        &RelationTypeV1::try_new(RecordTypeV1::try_new(vec![]).unwrap(), CardinalityV1::Set)
            .unwrap()
    );
    let capture = ResolvedRelationCaptureInputV2::new(
        SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: nested_id,
        },
        SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: HirNodeIdV1::from_raw(10_000),
        },
        0,
    );
    let duplicate_capture_error = ResolvedRelationPredicateInputV2::try_resolved(
        SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: relation_root,
        },
        vec![capture.clone(), capture.clone()],
    )
    .unwrap_err();
    assert_eq!(
        duplicate_capture_error.code(),
        "types.invalid_relation_predicate_input"
    );

    let reject_predicate = |predicate| {
        resolved_type_inputs_with_rows(
            checked,
            vec![
                ResolvedTypeInputEntryV2 {
                    key: SourceQualifiedNodeIdV2 {
                        source_id: SourceIdV2::from_raw(9),
                        node_id: nested_id,
                    },
                    value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
                        ValueTypeV1::bool(),
                    )),
                },
                ResolvedTypeInputEntryV2 {
                    key: exists_key,
                    value: ResolvedTypeInputValueV2::RelationPredicate(predicate),
                },
            ],
            vec![],
            vec![DeferredRelationRowInputV2::known_source(
                SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(9),
                    node_id: source_root,
                },
                0,
                Vec::new(),
            )],
        )
        .unwrap_err()
        .code()
    };
    assert_eq!(
        reject_predicate(
            ResolvedRelationPredicateInputV2::try_resolved(
                SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(9),
                    node_id: source_root,
                },
                vec![capture.clone()],
            )
            .unwrap(),
        ),
        "types.invalid_relation_predicate_input"
    );
    assert_eq!(
        reject_predicate(
            ResolvedRelationPredicateInputV2::try_resolved(
                SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(9),
                    node_id: relation_root,
                },
                vec![ResolvedRelationCaptureInputV2::new(
                    SourceQualifiedNodeIdV2 {
                        source_id: SourceIdV2::from_raw(9),
                        node_id: nested_id,
                    },
                    SourceQualifiedNodeIdV2 {
                        source_id: SourceIdV2::from_raw(9),
                        node_id: nested_id,
                    },
                    0,
                )],
            )
            .unwrap(),
        ),
        "types.invalid_relation_predicate_input"
    );
    assert_eq!(
        ResolvedRelationPredicateInputV2::try_resolved(
            SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(9),
                node_id: relation_root,
            },
            vec![ResolvedRelationCaptureInputV2::new(
                SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(77),
                    node_id: nested_id,
                },
                SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(9),
                    node_id: HirNodeIdV1::from_raw(10_000),
                },
                0,
            )],
        )
        .unwrap_err()
        .code(),
        "types.invalid_relation_predicate_input"
    );
    let entries = vec![
        ResolvedTypeInputEntryV2 {
            key: SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(9),
                node_id: nested_id,
            },
            value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
                ValueTypeV1::bool(),
            )),
        },
        ResolvedTypeInputEntryV2 {
            key: exists_key,
            value: ResolvedTypeInputValueV2::RelationPredicate(
                ResolvedRelationPredicateInputV2::try_resolved(
                    SourceQualifiedNodeIdV2 {
                        source_id: SourceIdV2::from_raw(9),
                        node_id: relation_root,
                    },
                    vec![capture],
                )
                .unwrap(),
            ),
        },
    ];
    let inputs = resolved_type_inputs_with_rows(
        checked,
        entries,
        vec![],
        vec![DeferredRelationRowInputV2::known_source(
            SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(9),
                node_id: source_root,
            },
            0,
            Vec::new(),
        )],
    )
    .unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(
        facts.len(),
        4,
        "every value and bool node must have one fact"
    );
    assert_eq!(
        facts.relation_row_count(),
        2,
        "the supplied known source row and its filtered copy must be materialized"
    );
    assert!(facts
        .relation_row(SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: source_root,
        })
        .unwrap()
        .fields()
        .is_empty());
    assert_eq!(
        facts
            .get(SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(9),
                node_id: nested_id,
            })
            .unwrap()
            .state,
        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    );
    let fact = facts.get(exists_key).unwrap();
    assert_eq!(
        fact.state,
        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    );
    assert_eq!(fact.effects.purity, PurityV1::Impure);
    assert_eq!(fact.effects.determinism, DeterminismV1::NonDeterministic);
    assert_eq!(fact.effects.monotonicity, MonotonicityV1::NonMonotone);
    assert_eq!(fact.effects.aggregation, AggregationClassV1::Scalar);
    assert_eq!(fact.effects.cost, CostClassV1::SuperLinear);

    let unknown_inputs = resolved_type_inputs_with_rows(
        checked,
        vec![
            ResolvedTypeInputEntryV2 {
                key: SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(9),
                    node_id: nested_id,
                },
                value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
                    ValueTypeV1::bool(),
                )),
            },
            ResolvedTypeInputEntryV2 {
                key: exists_key,
                value: ResolvedTypeInputValueV2::RelationPredicate(
                    ResolvedRelationPredicateInputV2::unknown(
                        UnknownTypeCauseV2::UnresolvedEditorBinding,
                    ),
                ),
            },
        ],
        vec![],
        vec![DeferredRelationRowInputV2::known_source(
            SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(9),
                node_id: source_root,
            },
            0,
            Vec::new(),
        )],
    )
    .unwrap();
    let unknown_facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&unknown_inputs)
        .unwrap();
    let unknown = unknown_facts.get(exists_key).unwrap();
    assert_eq!(
        unknown.state,
        ExpressionTypeStateV2::Unknown(UnknownTypeCauseV2::UnresolvedEditorBinding)
    );
    assert_eq!(unknown.effects.purity, PurityV1::Impure);
    assert_eq!(unknown.effects.determinism, DeterminismV1::NonDeterministic);
    assert_eq!(unknown.effects.monotonicity, MonotonicityV1::NonMonotone);
    assert_eq!(unknown.effects.aggregation, AggregationClassV1::Aggregate);
    assert_eq!(unknown.effects.cost, CostClassV1::SuperLinear);
}

#[test]
fn relation_exists_captures_use_hir_order_and_types_owned_effects() {
    let builder = HirBuilderV1::new();
    let first = builder.symbol("outer_first", SpanV1::new(20, 31));
    let first_id = first.id;
    let second = builder.symbol("outer_second", SpanV1::new(34, 46));
    let second_id = second.id;
    let predicate = builder.compare("=", first, second, SpanV1::new(20, 46));
    let source = builder.relation_source("users", SpanV1::new(10, 15), None, SpanV1::new(10, 15));
    let source_root = source.id;
    let relation = builder.relation_filter(source, predicate, SpanV1::new(10, 46));
    let relation_root = relation.id;
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        relation,
        SpanV1::new(5, 47),
    );
    let exists_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: exists.id,
    };
    let doc = checked_rule(builder.int("1", SpanV1::new(0, 1)), exists);
    let checked = check_document_v2(&doc).unwrap();
    let owner = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: HirNodeIdV1::from_raw(10_000),
    };
    let capture = |reference| {
        ResolvedRelationCaptureInputV2::new(
            SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(9),
                node_id: reference,
            },
            owner,
            0,
        )
    };
    let entries = |captures| {
        vec![
            ResolvedTypeInputEntryV2 {
                key: SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(9),
                    node_id: first_id,
                },
                value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
                    ValueTypeV1::bool(),
                )),
            },
            ResolvedTypeInputEntryV2 {
                key: SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(9),
                    node_id: second_id,
                },
                value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
                    ValueTypeV1::bool(),
                )),
            },
            ResolvedTypeInputEntryV2 {
                key: exists_key,
                value: ResolvedTypeInputValueV2::RelationPredicate(
                    ResolvedRelationPredicateInputV2::try_resolved(
                        SourceQualifiedNodeIdV2 {
                            source_id: SourceIdV2::from_raw(9),
                            node_id: relation_root,
                        },
                        captures,
                    )
                    .unwrap(),
                ),
            },
        ]
    };
    let rows = || {
        vec![DeferredRelationRowInputV2::known_source(
            SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(9),
                node_id: source_root,
            },
            0,
            Vec::new(),
        )]
    };

    // Both captures are real relation-subtree references and are supplied in
    // HIR traversal order. Types does not need Binder to reject a permutation.
    let ordered = resolved_type_inputs_with_rows(
        checked,
        entries(vec![capture(first_id), capture(second_id)]),
        vec![],
        rows(),
    )
    .unwrap();
    let ordered_facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&ordered)
        .unwrap();
    let exists_fact = ordered_facts.get(exists_key).unwrap();
    assert_eq!(
        exists_fact.state,
        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    );
    assert_eq!(
        exists_fact.effects,
        ExpressionEffectsV2 {
            purity: PurityV1::Impure,
            determinism: DeterminismV1::NonDeterministic,
            monotonicity: MonotonicityV1::NonMonotone,
            aggregation: AggregationClassV1::Scalar,
            cost: CostClassV1::SuperLinear,
        }
    );
    assert_eq!(
        resolved_type_inputs_with_rows(
            checked,
            entries(vec![capture(second_id), capture(first_id)]),
            vec![],
            rows(),
        )
        .unwrap_err()
        .code(),
        "types.invalid_relation_predicate_input"
    );
}

#[test]
fn comparison_and_boolean_parents_emit_complete_bool_facts() {
    let builder = HirBuilderV1::new();
    let compare = builder.compare(
        "<",
        builder.int("1", SpanV1::new(5, 6)),
        builder.int("2", SpanV1::new(9, 10)),
        SpanV1::new(5, 10),
    );
    let compare_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: compare.id,
    };
    let parent = builder.not(compare, SpanV1::new(1, 10));
    let parent_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: parent.id,
    };
    let doc = checked_rule(builder.int("1", SpanV1::new(0, 1)), parent);
    let inputs = resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(
        facts.get(compare_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    );
    assert_eq!(
        facts
            .get(compare_key)
            .unwrap()
            .operator_id
            .map(OperatorIdV2::raw),
        Some(200)
    );
    assert_eq!(
        facts.get(compare_key).unwrap().effects,
        ExpressionEffectsV2 {
            purity: PurityV1::Pure,
            determinism: DeterminismV1::Deterministic,
            monotonicity: MonotonicityV1::NonMonotone,
            aggregation: AggregationClassV1::Scalar,
            cost: CostClassV1::Constant,
        }
    );
    assert_eq!(
        facts.get(parent_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    );
    assert_eq!(
        facts
            .get(parent_key)
            .unwrap()
            .operator_id
            .map(OperatorIdV2::raw),
        Some(7)
    );
    assert_eq!(
        facts.get(parent_key).unwrap().effects,
        ExpressionEffectsV2 {
            purity: PurityV1::Pure,
            determinism: DeterminismV1::Deterministic,
            monotonicity: MonotonicityV1::NonMonotone,
            aggregation: AggregationClassV1::Scalar,
            cost: CostClassV1::Constant,
        }
    );
}

#[test]
fn builtin_bad_arity_rejects_without_fabricating_a_target() {
    let builder = HirBuilderV1::new();
    let call = builder.call(
        "size",
        SpanV1::new(0, 4),
        CallArgsHirV1::try_new(vec![]).unwrap(),
        SpanV1::new(0, 6),
    );
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let registry = BuiltinRegistryV2::try_closed().unwrap();
    let input = ResolvedTypeInputEntryV2 {
        key,
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
            registry.family_id("size").unwrap(),
        )),
    };
    let doc = checked_document(call);
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    assert_eq!(
        ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs)
            .unwrap_err()
            .code(),
        "types.bad_overload"
    );
}

#[test]
fn ambiguous_unknown_builtin_domain_keeps_conservative_effects_without_id() {
    let builder = HirBuilderV1::new();
    let symbol = builder.symbol("x", SpanV1::new(4, 5));
    let symbol_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: symbol.id,
    };
    let call = builder.call(
        "sum",
        SpanV1::new(0, 3),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(symbol)]).unwrap(),
        SpanV1::new(0, 6),
    );
    let call_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: call.id,
    };
    let registry = BuiltinRegistryV2::try_closed().unwrap();
    let entries = vec![
        ResolvedTypeInputEntryV2 {
            key: symbol_key,
            value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Unknown(
                UnknownTypeCauseV2::UnresolvedEditorBinding,
            )),
        },
        ResolvedTypeInputEntryV2 {
            key: call_key,
            value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
                registry.family_id("sum").unwrap(),
            )),
        },
    ];
    let doc = checked_document(call);
    let inputs = resolved_type_inputs(check_document_v2(&doc).unwrap(), entries, vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    let fact = facts.get(call_key).unwrap();
    assert_eq!(
        fact.state,
        ExpressionTypeStateV2::Unknown(UnknownTypeCauseV2::UnresolvedEditorBinding)
    );
    assert!(fact.resolved_call().is_none());
    assert_eq!(
        fact.effects,
        ExpressionEffectsV2 {
            purity: PurityV1::Pure,
            determinism: DeterminismV1::Deterministic,
            monotonicity: MonotonicityV1::Monotone,
            aggregation: AggregationClassV1::Aggregate,
            cost: CostClassV1::Linear,
        }
    );
}

#[test]
fn matching_antitone_callable_effects_preserve_antitone_in_parent_fact() {
    let b = HirBuilderV1::new();
    let inner = b.call(
        "inner",
        SpanV1::new(6, 11),
        CallArgsHirV1::try_new(vec![]).unwrap(),
        SpanV1::new(6, 13),
    );
    let inner_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: inner.id,
    };
    let outer = b.call(
        "outer",
        SpanV1::new(0, 5),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(inner)]).unwrap(),
        SpanV1::new(0, 14),
    );
    let outer_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9),
        node_id: outer.id,
    };
    let effects = ExpressionEffectsV2 {
        purity: PurityV1::Impure,
        determinism: DeterminismV1::NonDeterministic,
        monotonicity: MonotonicityV1::Antitone,
        aggregation: AggregationClassV1::Scalar,
        cost: CostClassV1::Linear,
    };
    let callable = |name: &str, params: Vec<ValueTypeV1>| {
        ResolvedCallableInputV2::Predicate(
            ResolvedPredicateCallableV2::try_new(
                ResolvedPredicateIdV2::try_new(name).unwrap(),
                params,
                ValueTypeV1::bool(),
                effects,
            )
            .unwrap(),
        )
    };
    let doc = checked_document(outer);
    let inputs = resolved_type_inputs(
        check_document_v2(&doc).unwrap(),
        vec![
            ResolvedTypeInputEntryV2 {
                key: inner_key,
                value: ResolvedTypeInputValueV2::Callable(callable("inner", vec![])),
            },
            ResolvedTypeInputEntryV2 {
                key: outer_key,
                value: ResolvedTypeInputValueV2::Callable(callable(
                    "outer",
                    vec![ValueTypeV1::bool()],
                )),
            },
        ],
        vec![],
    )
    .unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(facts.get(inner_key).unwrap().effects, effects);
    assert_eq!(facts.get(outer_key).unwrap().effects, effects);
    assert_eq!(
        facts.get(outer_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::bool())
    );
}

#[test]
fn boolean_hir_literal_facts_preserve_both_authored_values() {
    for value in [false, true] {
        let builder = HirBuilderV1::new();
        let body = builder.bool_literal(value, SpanV1::new(10, 15));
        let key = SourceQualifiedNodeIdV2 { source_id: SourceIdV2::from_raw(9), node_id: body.id };
        let doc = checked_rule(builder.int("1", SpanV1::new(4, 5)), body);
        let inputs = resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
        let facts = ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs).unwrap();
        let fact = facts.get(key).unwrap();
        assert_eq!(fact.state, ExpressionTypeStateV2::Known(ValueTypeV1::bool()));
        assert_eq!(fact.literal, Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Bool(value))));
    }
}
