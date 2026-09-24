#![expect(
    clippy::unwrap_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{
    check_document_v2, CallArgHirV1, CallArgsHirV1, HirBuilderV1, HirDocumentV1, LocatedSpanV2,
    LogicalBinderV1, LogicalQuantifierKindV1, NonEmptyBindersV1, ProgramItemHirV1, RecordFieldV1,
    RelationExistsPolarityV1, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_types::*;

fn source() -> SourceIdV2 {
    SourceIdV2::from_raw(71)
}
fn span(start: usize, end: usize) -> SpanV1 {
    SpanV1::new(start, end)
}
fn key(node: quanta_gqlang_v2_hir::HirNodeIdV1) -> SourceQualifiedNodeIdV2 {
    SourceQualifiedNodeIdV2 {
        source_id: source(),
        node_id: node,
    }
}
fn binding(node: quanta_gqlang_v2_hir::HirNodeIdV1) -> RuleBindingIdV2 {
    RuleBindingIdV2::new(source(), node, 0)
}
fn registry() -> quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2 {
    SourceRegistryViewV2::try_new(vec![])
        .unwrap()
        .try_extend_document(SourceRegistrationV2 {
            source_id: source(),
            source_len: 256,
        })
        .unwrap()
}
fn solve(
    doc: &HirDocumentV1,
    predicates: Vec<RulePredicateInputV2>,
    symbols: Vec<RuleSymbolInputV2>,
    calls: Vec<RuleCallInputV2>,
    budget: RuleTypeBudgetV2,
) -> Result<RuleTypeProofV2, TypeDiagnosticV2> {
    RuleTypeSolverV2::try_closed().unwrap().solve(
        check_document_v2(doc).unwrap(),
        &registry(),
        predicates,
        symbols,
        calls,
        budget,
    )
}
fn roomy() -> RuleTypeBudgetV2 {
    RuleTypeBudgetV2::try_new(128, 128, 100_000).unwrap()
}
fn host_key(local_name: &str) -> ResolvedHostPredicateKeyV2 {
    let len = ResolvedHostPredicateKeyV2::checked_canonical_len(
        "host::module",
        HostCallableNamespaceV2::Predicate,
        local_name,
    )
    .unwrap();
    ResolvedHostPredicateKeyV2::try_from_catalog_parts(
        "host::module",
        HostCallableNamespaceV2::Predicate,
        local_name,
        len,
    )
    .unwrap()
}

#[test]
fn literal_operator_anchor_infers_unary_rule_head_from_canonical_registry() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_ref = head.id;
    let body_ref = b.symbol("x", span(14, 15));
    let body_ref_id = body_ref.id;
    let body = b.compare("=", body_ref, b.int("1", span(18, 19)), span(14, 19));
    let rule = b.rule("r", span(5, 6), head, body, span(0, 20));
    let rule_id = rule.id;
    let predicate = ResolvedPredicateIdV2::try_new("r").unwrap();
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let proof = solve(
        &doc,
        vec![RulePredicateInputV2 {
            rule_node: key(rule_id),
            predicate_id: predicate.clone(),
            head_binding: binding(rule_id),
        }],
        vec![
            RuleSymbolInputV2 {
                reference: key(head_ref),
                binding: binding(rule_id),
            },
            RuleSymbolInputV2 {
                reference: key(body_ref_id),
                binding: binding(rule_id),
            },
        ],
        vec![],
        roomy(),
    )
    .unwrap();
    assert_eq!(
        proof.binding_type(binding(rule_id)),
        Some(&ValueTypeV1::int())
    );
    assert_eq!(proof.predicate_type(&predicate), Some(&ValueTypeV1::int()));
    assert!(proof.usage().steps > 0);
}

#[test]
fn anchored_self_recursion_solves_without_guessing_a_default() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let anchored = b.symbol("x", span(14, 15));
    let anchor_id = anchored.id;
    let compared = b.compare("=", anchored, b.int("1", span(18, 19)), span(14, 19));
    let recursive_arg = b.symbol("x", span(26, 27));
    let recursive_id = recursive_arg.id;
    let call = b.call(
        "r",
        span(24, 25),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(recursive_arg)]).unwrap(),
        span(24, 28),
    );
    let call_id = call.id;
    let body = b.and(compared, b.bool_value(call, span(24, 28)), span(14, 28));
    let rule = b.rule("r", span(5, 6), head, body, span(0, 29));
    let rule_id = rule.id;
    let predicate = ResolvedPredicateIdV2::try_new("r").unwrap();
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let proof = solve(
        &doc,
        vec![RulePredicateInputV2 {
            rule_node: key(rule_id),
            predicate_id: predicate.clone(),
            head_binding: binding(rule_id),
        }],
        [head_id, anchor_id, recursive_id]
            .into_iter()
            .map(|id| RuleSymbolInputV2 {
                reference: key(id),
                binding: binding(rule_id),
            })
            .collect(),
        vec![RuleCallInputV2 {
            call: key(call_id),
            target: RuleCallableTargetV2::SourcePredicate(predicate.clone()),
        }],
        roomy(),
    )
    .unwrap();
    assert_eq!(proof.predicate_type(&predicate), Some(&ValueTypeV1::int()));
    let callable = proof
        .try_source_predicate_callable(&predicate)
        .unwrap()
        .unwrap();
    assert_eq!(callable.id(), &predicate);
    assert_eq!(callable.params(), &[ValueTypeV1::int()]);
    assert_eq!(callable.result(), &ValueTypeV1::bool());
    assert_eq!(callable.effects().aggregation, AggregationClassV1::Scalar);
    assert_eq!(callable.effects().purity, PurityV1::Impure);
    assert!(proof
        .try_source_predicate_callable(&ResolvedPredicateIdV2::try_new("missing").unwrap())
        .unwrap()
        .is_none());
}

#[test]
fn unanchored_self_recursion_is_not_closed_success() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let arg = b.symbol("x", span(17, 18));
    let arg_id = arg.id;
    let call = b.call(
        "r",
        span(15, 16),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
        span(15, 19),
    );
    let call_id = call.id;
    let rule = b.rule(
        "r",
        span(5, 6),
        head,
        b.bool_value(call, span(15, 19)),
        span(0, 20),
    );
    let rule_id = rule.id;
    let predicate = ResolvedPredicateIdV2::try_new("r").unwrap();
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let predicates = vec![RulePredicateInputV2 {
        rule_node: key(rule_id),
        predicate_id: predicate.clone(),
        head_binding: binding(rule_id),
    }];
    let symbols = [head_id, arg_id]
        .into_iter()
        .map(|id| RuleSymbolInputV2 {
            reference: key(id),
            binding: binding(rule_id),
        })
        .collect::<Vec<_>>();
    let calls = vec![RuleCallInputV2 {
        call: key(call_id),
        target: RuleCallableTargetV2::SourcePredicate(predicate.clone()),
    }];
    let analysis = RuleTypeSolverV2::try_closed()
        .unwrap()
        .analyze_open(
            check_document_v2(&doc).unwrap(),
            &registry(),
            predicates.clone(),
            symbols.clone(),
            calls.clone(),
            roomy(),
        )
        .unwrap();
    assert!(!analysis.is_complete());
    assert_eq!(
        analysis.binding_state(binding(rule_id)),
        Some(&ExpressionTypeStateV2::Unknown(
            UnknownTypeCauseV2::UnanchoredRuleVariable
        ))
    );
    assert_eq!(
        analysis.predicate_state(&predicate),
        Some(&ExpressionTypeStateV2::Unknown(
            UnknownTypeCauseV2::UnanchoredRuleVariable
        ))
    );
    assert_eq!(
        analysis.diagnostics().first().unwrap().code(),
        "types.unanchored_recursive_rule"
    );
    let error = solve(&doc, predicates, symbols, calls, roomy()).unwrap_err();
    assert_eq!(error.code(), "types.unanchored_recursive_rule");
}

#[test]
fn conflicting_anchors_and_missing_exact_resolution_reject() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let left = b.symbol("x", span(14, 15));
    let left_id = left.id;
    let right = b.symbol("x", span(24, 25));
    let right_id = right.id;
    let first = b.compare("=", left, b.int("1", span(18, 19)), span(14, 19));
    let second = b.compare("=", right, b.string("s", span(28, 31)), span(24, 31));
    let rule = b.rule(
        "r",
        span(5, 6),
        head,
        b.and(first, second, span(14, 31)),
        span(0, 32),
    );
    let rule_id = rule.id;
    let predicates = vec![RulePredicateInputV2 {
        rule_node: key(rule_id),
        predicate_id: ResolvedPredicateIdV2::try_new("r").unwrap(),
        head_binding: binding(rule_id),
    }];
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let resolutions = [head_id, left_id, right_id]
        .into_iter()
        .map(|id| RuleSymbolInputV2 {
            reference: key(id),
            binding: binding(rule_id),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        solve(
            &doc,
            predicates.clone(),
            resolutions.iter().take(2).cloned().collect(),
            vec![],
            roomy()
        )
        .unwrap_err()
        .code(),
        "types.invalid_rule_input"
    );
    assert_eq!(
        solve(&doc, predicates, resolutions, vec![], roomy())
            .unwrap_err()
            .code(),
        "types.contradictory_rule_type"
    );
}

#[test]
fn budget_exact_limit_succeeds_and_one_under_fails() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let body_ref = b.symbol("x", span(14, 15));
    let body_id = body_ref.id;
    let body = b.compare("=", body_ref, b.int("1", span(18, 19)), span(14, 19));
    let rule = b.rule("r", span(5, 6), head, body, span(0, 20));
    let rule_id = rule.id;
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let predicates = vec![RulePredicateInputV2 {
        rule_node: key(rule_id),
        predicate_id: ResolvedPredicateIdV2::try_new("r").unwrap(),
        head_binding: binding(rule_id),
    }];
    let symbols = [head_id, body_id]
        .into_iter()
        .map(|id| RuleSymbolInputV2 {
            reference: key(id),
            binding: binding(rule_id),
        })
        .collect::<Vec<_>>();
    let usage = solve(&doc, predicates.clone(), symbols.clone(), vec![], roomy())
        .unwrap()
        .usage();
    let exact = RuleTypeBudgetV2::try_new(usage.nodes, usage.edges, usage.steps).unwrap();
    assert_eq!(
        solve(&doc, predicates.clone(), symbols.clone(), vec![], exact)
            .unwrap()
            .usage(),
        usage
    );
    for budget in [
        RuleTypeBudgetV2::try_new(usage.nodes - 1, usage.edges, usage.steps).unwrap(),
        RuleTypeBudgetV2::try_new(usage.nodes, usage.edges - 1, usage.steps).unwrap(),
        RuleTypeBudgetV2::try_new(usage.nodes, usage.edges, usage.steps - 1).unwrap(),
    ] {
        assert_eq!(
            solve(&doc, predicates.clone(), symbols.clone(), vec![], budget)
                .unwrap_err()
                .code(),
            "types.rule_type_resource_limit"
        );
    }
}

#[test]
fn quantifier_binding_is_inferred_by_exact_owner_and_ordinal() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let head_body = b.symbol("x", span(14, 15));
    let head_body_id = head_body.id;
    let head_anchor = b.compare("=", head_body, b.int("1", span(18, 19)), span(14, 19));
    let quantified = b.symbol("y", span(35, 36));
    let quantified_id = quantified.id;
    let quant_body = b.compare("=", quantified, b.string("a", span(39, 42)), span(35, 42));
    let quant = b.logical_quantifier(
        LogicalQuantifierKindV1::Exists,
        NonEmptyBindersV1::try_new(vec![LogicalBinderV1 {
            name: "y".into(),
            span: span(31, 32),
        }])
        .unwrap(),
        quant_body,
        span(24, 42),
    );
    let quant_id = quant.id;
    let rule = b.rule(
        "r",
        span(5, 6),
        head,
        b.and(head_anchor, quant, span(14, 42)),
        span(0, 43),
    );
    let rule_id = rule.id;
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let proof = solve(
        &doc,
        vec![RulePredicateInputV2 {
            rule_node: key(rule_id),
            predicate_id: ResolvedPredicateIdV2::try_new("r").unwrap(),
            head_binding: binding(rule_id),
        }],
        vec![
            RuleSymbolInputV2 {
                reference: key(head_id),
                binding: binding(rule_id),
            },
            RuleSymbolInputV2 {
                reference: key(head_body_id),
                binding: binding(rule_id),
            },
            RuleSymbolInputV2 {
                reference: key(quantified_id),
                binding: binding(quant_id),
            },
        ],
        vec![],
        roomy(),
    )
    .unwrap();
    assert_eq!(
        proof.binding_type(binding(rule_id)),
        Some(&ValueTypeV1::int())
    );
    assert_eq!(
        proof.binding_type(binding(quant_id)),
        Some(&ValueTypeV1::string())
    );
}

#[test]
fn nearest_quantifier_shadow_rejects_forged_outer_same_name() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let outer = b.symbol("x", span(14, 15));
    let outer_id = outer.id;
    let outer_anchor = b.compare("=", outer, b.int("1", span(18, 19)), span(14, 19));
    let inner = b.symbol("x", span(35, 36));
    let inner_id = inner.id;
    let inner_anchor = b.compare("=", inner, b.string("a", span(39, 42)), span(35, 42));
    let quant = b.logical_quantifier(
        LogicalQuantifierKindV1::Exists,
        NonEmptyBindersV1::try_new(vec![LogicalBinderV1 {
            name: "x".into(),
            span: span(31, 32),
        }])
        .unwrap(),
        inner_anchor,
        span(24, 42),
    );
    let quant_id = quant.id;
    let rule = b.rule(
        "r",
        span(5, 6),
        head,
        b.and(outer_anchor, quant, span(14, 42)),
        span(0, 43),
    );
    let rule_id = rule.id;
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let predicates = vec![RulePredicateInputV2 {
        rule_node: key(rule_id),
        predicate_id: ResolvedPredicateIdV2::try_new("r").unwrap(),
        head_binding: binding(rule_id),
    }];
    let mut symbols = vec![
        RuleSymbolInputV2 {
            reference: key(head_id),
            binding: binding(rule_id),
        },
        RuleSymbolInputV2 {
            reference: key(outer_id),
            binding: binding(rule_id),
        },
        RuleSymbolInputV2 {
            reference: key(inner_id),
            binding: binding(quant_id),
        },
    ];
    let proof = solve(&doc, predicates.clone(), symbols.clone(), vec![], roomy()).unwrap();
    assert_eq!(
        proof.binding_type(binding(rule_id)),
        Some(&ValueTypeV1::int())
    );
    assert_eq!(
        proof.binding_type(binding(quant_id)),
        Some(&ValueTypeV1::string())
    );
    symbols.last_mut().unwrap().binding = binding(rule_id);
    assert_eq!(
        solve(&doc, predicates, symbols, vec![], roomy())
            .unwrap_err()
            .code(),
        "types.invalid_rule_input"
    );
}

#[test]
fn source_rule_call_arity_rejects_before_constraint_solve() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let call = b.call(
        "r",
        span(15, 16),
        CallArgsHirV1::try_new(vec![]).unwrap(),
        span(15, 18),
    );
    let call_id = call.id;
    let rule = b.rule(
        "r",
        span(5, 6),
        head,
        b.bool_value(call, span(15, 18)),
        span(0, 19),
    );
    let rule_id = rule.id;
    let predicate = ResolvedPredicateIdV2::try_new("r").unwrap();
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    assert_eq!(
        solve(
            &doc,
            vec![RulePredicateInputV2 {
                rule_node: key(rule_id),
                predicate_id: predicate.clone(),
                head_binding: binding(rule_id)
            }],
            vec![RuleSymbolInputV2 {
                reference: key(head_id),
                binding: binding(rule_id)
            }],
            vec![RuleCallInputV2 {
                call: key(call_id),
                target: RuleCallableTargetV2::SourcePredicate(predicate)
            }],
            roomy()
        )
        .unwrap_err()
        .code(),
        "types.rule_predicate_arity"
    );
}

fn mutual_fixture(
    anchored: bool,
) -> (
    HirDocumentV1,
    Vec<RulePredicateInputV2>,
    Vec<RuleSymbolInputV2>,
    Vec<RuleCallInputV2>,
) {
    let b = HirBuilderV1::new();
    let r_head = b.symbol("x", span(9, 10));
    let r_head_id = r_head.id;
    let r_arg = b.symbol("x", span(32, 33));
    let r_arg_id = r_arg.id;
    let s_call = b.call(
        "s",
        span(30, 31),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(r_arg)]).unwrap(),
        span(30, 34),
    );
    let s_call_id = s_call.id;
    let r_call_bool = b.bool_value(s_call, span(30, 34));
    let (r_body, r_anchor_id) = if anchored {
        let anchor = b.symbol("x", span(15, 16));
        let anchor_id = anchor.id;
        let compared = b.compare("=", anchor, b.int("1", span(19, 20)), span(15, 20));
        (b.and(compared, r_call_bool, span(15, 34)), Some(anchor_id))
    } else {
        (r_call_bool, None)
    };
    let r = b.rule("r", span(5, 6), r_head, r_body, span(0, 40));
    let r_id = r.id;

    let s_head = b.symbol("y", span(79, 80));
    let s_head_id = s_head.id;
    let s_arg = b.symbol("y", span(98, 99));
    let s_arg_id = s_arg.id;
    let r_call = b.call(
        "r",
        span(96, 97),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(s_arg)]).unwrap(),
        span(96, 100),
    );
    let r_call_id = r_call.id;
    let s = b.rule(
        "s",
        span(75, 76),
        s_head,
        b.bool_value(r_call, span(96, 100)),
        span(70, 110),
    );
    let s_id = s.id;
    let r_pred = ResolvedPredicateIdV2::try_new("r").unwrap();
    let s_pred = ResolvedPredicateIdV2::try_new("s").unwrap();
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![
            ProgramItemHirV1::Rule(Box::new(r)),
            ProgramItemHirV1::Rule(Box::new(s)),
        ],
    };
    let predicates = vec![
        RulePredicateInputV2 {
            rule_node: key(r_id),
            predicate_id: r_pred.clone(),
            head_binding: binding(r_id),
        },
        RulePredicateInputV2 {
            rule_node: key(s_id),
            predicate_id: s_pred.clone(),
            head_binding: binding(s_id),
        },
    ];
    let mut symbols = vec![
        RuleSymbolInputV2 {
            reference: key(r_head_id),
            binding: binding(r_id),
        },
        RuleSymbolInputV2 {
            reference: key(r_arg_id),
            binding: binding(r_id),
        },
        RuleSymbolInputV2 {
            reference: key(s_head_id),
            binding: binding(s_id),
        },
        RuleSymbolInputV2 {
            reference: key(s_arg_id),
            binding: binding(s_id),
        },
    ];
    if let Some(id) = r_anchor_id {
        symbols.push(RuleSymbolInputV2 {
            reference: key(id),
            binding: binding(r_id),
        });
    }
    let calls = vec![
        RuleCallInputV2 {
            call: key(s_call_id),
            target: RuleCallableTargetV2::SourcePredicate(s_pred),
        },
        RuleCallInputV2 {
            call: key(r_call_id),
            target: RuleCallableTargetV2::SourcePredicate(r_pred),
        },
    ];
    (doc, predicates, symbols, calls)
}

#[test]
fn anchored_mutual_recursion_is_unique_and_input_permutation_stable() {
    let (doc, mut predicates, mut symbols, mut calls) = mutual_fixture(true);
    let first = solve(
        &doc,
        predicates.clone(),
        symbols.clone(),
        calls.clone(),
        roomy(),
    )
    .unwrap();
    predicates.reverse();
    symbols.reverse();
    calls.reverse();
    let permuted = solve(&doc, predicates, symbols, calls, roomy()).unwrap();
    for name in ["r", "s"] {
        let id = ResolvedPredicateIdV2::try_new(name).unwrap();
        assert_eq!(first.predicate_type(&id), Some(&ValueTypeV1::int()));
        assert_eq!(permuted.predicate_type(&id), first.predicate_type(&id));
    }
    assert_eq!(first.usage(), permuted.usage());
}

/// Three unary predicates form one equality cycle. A single integer comparison
/// on any head must constrain all three; removing it leaves no unique closed
/// assignment.
fn three_rule_cycle_fixture(
    anchor_head: Option<usize>,
) -> (
    HirDocumentV1,
    Vec<RulePredicateInputV2>,
    Vec<RuleSymbolInputV2>,
    Vec<RuleCallInputV2>,
) {
    let b = HirBuilderV1::new();
    let mut items = Vec::new();
    let mut predicates = Vec::new();
    let mut symbols = Vec::new();
    let mut calls = Vec::new();
    for (index, (name, target)) in [("a", "b"), ("b", "c"), ("c", "a")].into_iter().enumerate() {
        let start = index * 70;
        let head = b.symbol("x", span(start + 9, start + 10));
        let head_id = head.id;
        let arg = b.symbol("x", span(start + 32, start + 33));
        let arg_id = arg.id;
        let call = b.call(
            target,
            span(start + 30, start + 31),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
            span(start + 30, start + 34),
        );
        let call_id = call.id;
        let call_body = b.bool_value(call, span(start + 30, start + 34));
        let (body, anchor_id) = if anchor_head == Some(index) {
            let anchor = b.symbol("x", span(start + 15, start + 16));
            let anchor_id = anchor.id;
            let compared = b.compare(
                "=",
                anchor,
                b.int("1", span(start + 19, start + 20)),
                span(start + 15, start + 20),
            );
            (
                b.and(compared, call_body, span(start + 15, start + 34)),
                Some(anchor_id),
            )
        } else {
            (call_body, None)
        };
        let rule = b.rule(
            name,
            span(start + 5, start + 6),
            head,
            body,
            span(start, start + 40),
        );
        let rule_id = rule.id;
        items.push(ProgramItemHirV1::Rule(Box::new(rule)));
        predicates.push(RulePredicateInputV2 {
            rule_node: key(rule_id),
            predicate_id: ResolvedPredicateIdV2::try_new(name).unwrap(),
            head_binding: binding(rule_id),
        });
        for reference in [Some(head_id), Some(arg_id), anchor_id]
            .into_iter()
            .flatten()
        {
            symbols.push(RuleSymbolInputV2 {
                reference: key(reference),
                binding: binding(rule_id),
            });
        }
        calls.push(RuleCallInputV2 {
            call: key(call_id),
            target: RuleCallableTargetV2::SourcePredicate(
                ResolvedPredicateIdV2::try_new(target).unwrap(),
            ),
        });
    }
    (
        HirDocumentV1 {
            source_id: source(),
            source_len: 256,
            items,
        },
        predicates,
        symbols,
        calls,
    )
}

#[test]
fn three_rule_cycle_anchor_position_and_input_permutations_are_stable() {
    let orders = [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ];
    for anchor_head in [Some(0), Some(1), Some(2), None] {
        let anchored = anchor_head.is_some();
        let (document, predicates, symbols, calls) = three_rule_cycle_fixture(anchor_head);
        assert_eq!(document.items.len(), 3);
        let mut successes = 0;
        let mut rejections = 0;
        for (case, order) in orders.into_iter().enumerate() {
            let mut permuted = document.clone();
            permuted.items = order
                .into_iter()
                .map(|index| document.items.get(index).unwrap().clone())
                .collect();
            let mut predicates = predicates.clone();
            let mut symbols = symbols.clone();
            let mut calls = calls.clone();
            predicates.rotate_left(case % 3);
            let symbol_count = symbols.len();
            symbols.rotate_left(case % symbol_count);
            calls.rotate_left(case % 3);
            if case % 2 == 1 {
                predicates.reverse();
                symbols.reverse();
                calls.reverse();
            }
            let result = solve(&permuted, predicates, symbols, calls, roomy());
            if anchored {
                assert!(
                    result.is_ok(),
                    "anchor {anchor_head:?}, order {order:?}: {result:?}"
                );
                let proof = result.unwrap();
                for name in ["a", "b", "c"] {
                    let id = ResolvedPredicateIdV2::try_new(name).unwrap();
                    assert_eq!(
                        proof.predicate_type(&id),
                        Some(&ValueTypeV1::int()),
                        "anchor {anchor_head:?}, order {order:?}: {name}"
                    );
                }
                successes += 1;
            } else {
                assert_eq!(
                    result.unwrap_err().code(),
                    "types.unanchored_recursive_rule",
                    "anchor {anchor_head:?}, order {order:?}"
                );
                rejections += 1;
            }
        }
        assert_eq!(successes, if anchored { 6 } else { 0 });
        assert_eq!(rejections, if anchored { 0 } else { 6 });
    }
}

#[test]
fn unanchored_mutual_recursion_rejects_as_recursive_not_default() {
    let (doc, predicates, symbols, calls) = mutual_fixture(false);
    assert_eq!(
        solve(&doc, predicates, symbols, calls, roomy())
            .unwrap_err()
            .code(),
        "types.unanchored_recursive_rule"
    );
}

#[test]
fn exact_rule_resolution_inputs_reject_missing_duplicate_foreign_and_wrong_binding() {
    let (doc, predicates, symbols, calls) = mutual_fixture(true);
    assert_eq!(
        solve(
            &doc,
            predicates.clone(),
            symbols.iter().skip(1).cloned().collect(),
            calls.clone(),
            roomy()
        )
        .unwrap_err()
        .code(),
        "types.invalid_rule_input"
    );
    let mut duplicated = symbols.clone();
    duplicated.push(symbols.first().unwrap().clone());
    assert_eq!(
        solve(&doc, predicates.clone(), duplicated, calls.clone(), roomy())
            .unwrap_err()
            .code(),
        "types.invalid_rule_input"
    );
    let mut foreign = symbols.clone();
    foreign.first_mut().unwrap().reference.source_id = SourceIdV2::from_raw(72);
    assert_eq!(
        solve(&doc, predicates.clone(), foreign, calls.clone(), roomy())
            .unwrap_err()
            .code(),
        "types.invalid_rule_input"
    );
    let mut wrong_binding = symbols.clone();
    let other_rule = predicates
        .iter()
        .find(|p| p.predicate_id.as_str() == "s")
        .unwrap()
        .rule_node
        .node_id;
    wrong_binding.first_mut().unwrap().binding = binding(other_rule);
    assert_eq!(
        solve(
            &doc,
            predicates.clone(),
            wrong_binding,
            calls.clone(),
            roomy()
        )
        .unwrap_err()
        .code(),
        "types.invalid_rule_input"
    );
    let mut extra = symbols.clone();
    extra.push(RuleSymbolInputV2 {
        reference: key(quanta_gqlang_v2_hir::HirNodeIdV1::from_raw(99_999)),
        binding: symbols.first().unwrap().binding,
    });
    assert_eq!(
        solve(&doc, predicates.clone(), extra, calls.clone(), roomy())
            .unwrap_err()
            .code(),
        "types.invalid_rule_input"
    );
    let mut duplicate_predicates = predicates.clone();
    duplicate_predicates.push(predicates.first().unwrap().clone());
    assert_eq!(
        solve(
            &doc,
            duplicate_predicates,
            symbols.clone(),
            calls.clone(),
            roomy()
        )
        .unwrap_err()
        .code(),
        "types.invalid_rule_input"
    );
    let mut unknown_call = calls.clone();
    unknown_call.first_mut().unwrap().target =
        RuleCallableTargetV2::SourcePredicate(ResolvedPredicateIdV2::try_new("missing").unwrap());
    assert_eq!(
        solve(&doc, predicates, symbols, unknown_call, roomy())
            .unwrap_err()
            .code(),
        "types.unknown_rule_callable"
    );
}

#[test]
fn cross_call_target_transplant_rejects_against_authored_callee() {
    let (doc, predicates, symbols, mut calls) = mutual_fixture(true);
    let first_target = calls.first().unwrap().target.clone();
    let second_target = calls.get(1).unwrap().target.clone();
    calls.first_mut().unwrap().target = second_target;
    calls.get_mut(1).unwrap().target = first_target;
    assert_eq!(
        solve(&doc, predicates, symbols, calls, roomy())
            .unwrap_err()
            .code(),
        "types.unknown_rule_callable"
    );
}

#[test]
fn authoritative_host_signature_anchors_and_unknown_signature_rejects() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let arg = b.symbol("x", span(20, 21));
    let arg_id = arg.id;
    let call = b.call(
        "host",
        span(15, 19),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
        span(15, 22),
    );
    let call_id = call.id;
    let rule = b.rule(
        "r",
        span(5, 6),
        head,
        b.bool_value(call, span(15, 22)),
        span(0, 23),
    );
    let rule_id = rule.id;
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let predicates = vec![RulePredicateInputV2 {
        rule_node: key(rule_id),
        predicate_id: ResolvedPredicateIdV2::try_new("r").unwrap(),
        head_binding: binding(rule_id),
    }];
    let symbols = [head_id, arg_id]
        .into_iter()
        .map(|id| RuleSymbolInputV2 {
            reference: key(id),
            binding: binding(rule_id),
        })
        .collect::<Vec<_>>();
    let signature = ResolvedHostPredicateCallableV2::try_new(
        host_key("host"),
        vec![ValueTypeV1::int()],
        ValueTypeV1::bool(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .unwrap();
    let proof = solve(
        &doc,
        predicates.clone(),
        symbols.clone(),
        vec![RuleCallInputV2 {
            call: key(call_id),
            target: RuleCallableTargetV2::HostPredicate(signature),
        }],
        roomy(),
    )
    .unwrap();
    assert_eq!(
        proof.binding_type(binding(rule_id)),
        Some(&ValueTypeV1::int())
    );
    let nested_type = ValueTypeV1::try_list(ValueTypeV1::int()).unwrap();
    let nested_signature = ResolvedHostPredicateCallableV2::try_new(
        host_key("host"),
        vec![nested_type.clone()],
        ValueTypeV1::bool(),
        ExpressionEffectsV2::pure_scalar(),
    )
    .unwrap();
    let nested_calls = vec![RuleCallInputV2 {
        call: key(call_id),
        target: RuleCallableTargetV2::HostPredicate(nested_signature),
    }];
    let nested_proof = solve(
        &doc,
        predicates.clone(),
        symbols.clone(),
        nested_calls.clone(),
        roomy(),
    )
    .unwrap();
    assert_eq!(
        nested_proof.binding_type(binding(rule_id)),
        Some(&nested_type)
    );
    assert!(nested_proof.usage().steps > proof.usage().steps);
    assert_eq!(
        solve(
            &doc,
            predicates.clone(),
            symbols.clone(),
            nested_calls,
            RuleTypeBudgetV2::try_new(128, 128, proof.usage().steps).unwrap()
        )
        .unwrap_err()
        .code(),
        "types.rule_type_resource_limit"
    );
    assert_eq!(
        solve(
            &doc,
            predicates,
            symbols,
            vec![RuleCallInputV2 {
                call: key(call_id),
                target: RuleCallableTargetV2::Unknown
            }],
            roomy()
        )
        .unwrap_err()
        .code(),
        "types.unknown_rule_callable"
    );
}

#[test]
fn unanchored_equality_overloads_remain_ambiguous_without_recursion() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let left = b.symbol("x", span(14, 15));
    let left_id = left.id;
    let right = b.symbol("x", span(18, 19));
    let right_id = right.id;
    let rule = b.rule(
        "r",
        span(5, 6),
        head,
        b.compare("=", left, right, span(14, 19)),
        span(0, 20),
    );
    let rule_id = rule.id;
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    assert_eq!(
        solve(
            &doc,
            vec![RulePredicateInputV2 {
                rule_node: key(rule_id),
                predicate_id: ResolvedPredicateIdV2::try_new("r").unwrap(),
                head_binding: binding(rule_id)
            }],
            [head_id, left_id, right_id]
                .into_iter()
                .map(|id| RuleSymbolInputV2 {
                    reference: key(id),
                    binding: binding(rule_id)
                })
                .collect(),
            vec![],
            roomy()
        )
        .unwrap_err()
        .code(),
        "types.ambiguous_rule_type"
    );
}

#[test]
fn record_and_list_constructor_constraints_match_host_signature_without_sema_inference() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let anchor = b.symbol("x", span(14, 15));
    let anchor_id = anchor.id;
    let compared = b.compare("=", anchor, b.int("1", span(18, 19)), span(14, 19));
    let list = b.list(
        vec![b.int("2", span(38, 39)), b.int("3", span(40, 41))],
        span(37, 42),
    );
    let record = b.record(
        vec![RecordFieldV1 {
            name: "xs".into(),
            name_span: span(33, 35),
            value: list,
        }],
        span(32, 43),
    );
    let call = b.call(
        "host",
        span(27, 31),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(record)]).unwrap(),
        span(27, 44),
    );
    let call_id = call.id;
    let body = b.and(compared, b.bool_value(call, span(27, 44)), span(14, 44));
    let rule = b.rule("r", span(5, 6), head, body, span(0, 45));
    let rule_id = rule.id;
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let predicates = vec![RulePredicateInputV2 {
        rule_node: key(rule_id),
        predicate_id: ResolvedPredicateIdV2::try_new("r").unwrap(),
        head_binding: binding(rule_id),
    }];
    let symbols = [head_id, anchor_id]
        .into_iter()
        .map(|id| RuleSymbolInputV2 {
            reference: key(id),
            binding: binding(rule_id),
        })
        .collect::<Vec<_>>();
    for (element, success) in [(ValueTypeV1::int(), true), (ValueTypeV1::string(), false)] {
        let list_type = ValueTypeV1::try_list(element).unwrap();
        let field_location = registry()
            .try_check_location(LocatedSpanV2 {
                source_id: source(),
                span: span(33, 35),
            })
            .unwrap();
        let record_type = ValueTypeV1::try_record(
            RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
                "xs",
                list_type,
                field_location,
            )
            .unwrap()])
            .unwrap(),
        )
        .unwrap();
        let signature = ResolvedHostPredicateCallableV2::try_new(
            host_key("host"),
            vec![record_type],
            ValueTypeV1::bool(),
            ExpressionEffectsV2::pure_scalar(),
        )
        .unwrap();
        let result = solve(
            &doc,
            predicates.clone(),
            symbols.clone(),
            vec![RuleCallInputV2 {
                call: key(call_id),
                target: RuleCallableTargetV2::HostPredicate(signature),
            }],
            roomy(),
        );
        if success {
            assert_eq!(
                result.unwrap().binding_type(binding(rule_id)),
                Some(&ValueTypeV1::int())
            );
        } else {
            assert_eq!(result.unwrap_err().code(), "types.contradictory_rule_type");
        }
    }
}

#[test]
fn constructed_record_and_list_from_variable_use_the_same_closed_builtin_domain() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let anchored = b.symbol("x", span(14, 15));
    let anchored_id = anchored.id;
    let compared = b.compare("=", anchored, b.int("1", span(18, 19)), span(14, 19));
    let record_value = b.symbol("x", span(39, 40));
    let record_ref = record_value.id;
    let record = b.record(
        vec![RecordFieldV1 {
            name: "v".into(),
            name_span: span(36, 37),
            value: record_value,
        }],
        span(35, 41),
    );
    let record_call = b.call(
        "is_null",
        span(27, 34),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(record)]).unwrap(),
        span(27, 42),
    );
    let record_call_id = record_call.id;
    let list_value = b.symbol("x", span(58, 59));
    let list_ref = list_value.id;
    let list = b.list(vec![list_value], span(57, 60));
    let list_call = b.call(
        "is_null",
        span(49, 56),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(list)]).unwrap(),
        span(49, 61),
    );
    let list_call_id = list_call.id;
    let body = b.and(
        b.and(
            compared,
            b.bool_value(record_call, span(27, 42)),
            span(14, 42),
        ),
        b.bool_value(list_call, span(49, 61)),
        span(14, 61),
    );
    let rule = b.rule("r", span(5, 6), head, body, span(0, 62));
    let rule_id = rule.id;
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let family = BuiltinRegistryV2::try_closed()
        .unwrap()
        .family_id("is_null")
        .unwrap();
    let predicates = vec![RulePredicateInputV2 {
        rule_node: key(rule_id),
        predicate_id: ResolvedPredicateIdV2::try_new("r").unwrap(),
        head_binding: binding(rule_id),
    }];
    let symbols = [head_id, anchored_id, record_ref, list_ref]
        .into_iter()
        .map(|id| RuleSymbolInputV2 {
            reference: key(id),
            binding: binding(rule_id),
        })
        .collect::<Vec<_>>();
    let calls = vec![
        RuleCallInputV2 {
            call: key(record_call_id),
            target: RuleCallableTargetV2::Builtin(family),
        },
        RuleCallInputV2 {
            call: key(list_call_id),
            target: RuleCallableTargetV2::Builtin(family),
        },
    ];
    let proof = solve(
        &doc,
        predicates.clone(),
        symbols.clone(),
        calls.clone(),
        RuleTypeBudgetV2::try_new(128, 128, 100_000).unwrap(),
    )
    .unwrap();
    assert_eq!(
        proof.binding_type(binding(rule_id)),
        Some(&ValueTypeV1::int())
    );
    let usage = proof.usage();
    assert_eq!((usage.nodes, usage.edges, usage.steps), (15, 20, 8684));
    let exact = RuleTypeBudgetV2::try_new(usage.nodes, usage.edges, usage.steps).unwrap();
    assert_eq!(
        solve(
            &doc,
            predicates.clone(),
            symbols.clone(),
            calls.clone(),
            exact
        )
        .unwrap()
        .usage(),
        usage
    );
    for one_under in [
        RuleTypeBudgetV2::try_new(usage.nodes - 1, usage.edges, usage.steps).unwrap(),
        RuleTypeBudgetV2::try_new(usage.nodes, usage.edges - 1, usage.steps).unwrap(),
        RuleTypeBudgetV2::try_new(usage.nodes, usage.edges, usage.steps - 1).unwrap(),
    ] {
        assert_eq!(
            solve(
                &doc,
                predicates.clone(),
                symbols.clone(),
                calls.clone(),
                one_under
            )
            .unwrap_err()
            .code(),
            "types.rule_type_resource_limit"
        );
    }
    assert_eq!(
        solve(
            &doc,
            predicates.clone(),
            symbols.clone(),
            calls.clone(),
            RuleTypeBudgetV2::try_new(128, 128, 1).unwrap()
        )
        .unwrap_err()
        .code(),
        "types.rule_type_resource_limit"
    );
    let mut transplanted = calls;
    transplanted.first_mut().unwrap().target = RuleCallableTargetV2::Builtin(
        BuiltinRegistryV2::try_closed()
            .unwrap()
            .family_id("length")
            .unwrap(),
    );
    assert_eq!(
        solve(
            &doc,
            predicates,
            symbols,
            transplanted,
            RuleTypeBudgetV2::try_new(128, 128, 100_000).unwrap()
        )
        .unwrap_err()
        .code(),
        "types.unknown_rule_callable"
    );
}

#[test]
fn relation_existence_in_rule_has_exact_defense_in_depth_diagnostic() {
    let b = HirBuilderV1::new();
    let head = b.symbol("x", span(9, 10));
    let head_id = head.id;
    let body = b.relation_exists(
        RelationExistsPolarityV1::Exists,
        b.relation_source("data", span(26, 30), None, span(26, 30)),
        span(13, 31),
    );
    let rule = b.rule("r", span(5, 6), head, body, span(0, 32));
    let rule_id = rule.id;
    let doc = HirDocumentV1 {
        source_id: source(),
        source_len: 256,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    };
    let error = solve(
        &doc,
        vec![RulePredicateInputV2 {
            rule_node: key(rule_id),
            predicate_id: ResolvedPredicateIdV2::try_new("r").unwrap(),
            head_binding: binding(rule_id),
        }],
        vec![RuleSymbolInputV2 {
            reference: key(head_id),
            binding: binding(rule_id),
        }],
        vec![],
        roomy(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "types.relation_exists_in_rule");
}

#[test]
fn iterative_expected_prepass_rejects_one_over_depth_without_recursing() {
    for (wrappers, expected_code) in [
        (63, "types.ambiguous_rule_type"),
        (64, "types.rule_type_resource_limit"),
    ] {
        let b = HirBuilderV1::new();
        let head = b.symbol("x", span(9, 10));
        let head_id = head.id;
        let mut body = b.bool_literal(true, span(15, 16));
        for _ in 0..wrappers {
            body = b.paren_bool(body, span(15, 16));
        }
        let rule = b.rule("r", span(5, 6), head, body, span(0, 17));
        let rule_id = rule.id;
        let doc = HirDocumentV1 {
            source_id: source(),
            source_len: 256,
            items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
        };
        let error = solve(
            &doc,
            vec![RulePredicateInputV2 {
                rule_node: key(rule_id),
                predicate_id: ResolvedPredicateIdV2::try_new("r").unwrap(),
                head_binding: binding(rule_id),
            }],
            vec![RuleSymbolInputV2 {
                reference: key(head_id),
                binding: binding(rule_id),
            }],
            vec![],
            RuleTypeBudgetV2::try_new(512, 512, 1_000_000).unwrap(),
        )
        .unwrap_err();
        assert_eq!(error.code(), expected_code, "{error:?}");
        if wrappers == 64 {
            assert!(matches!(
                error,
                TypeDiagnosticV2::RuleTypeResourceLimit {
                    dimension: "depth",
                    ..
                }
            ));
        }
    }
}
