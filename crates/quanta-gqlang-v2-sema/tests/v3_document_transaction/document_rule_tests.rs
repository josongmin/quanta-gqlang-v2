use super::*;

#[test]
fn checked_syntax_rejects_document_wide_duplicate_ids_with_exact_spans() {
    let source = SourceIdV2::from_raw(9);
    let item = ProgramItemHirV1::Meta(Box::new(MetaHirV1::new(
        HirNodeIdV1::from_raw(4),
        SpanV1::new(1, 2),
        MetaKindV1::RequireCertified,
    )));
    let duplicate = ProgramItemHirV1::Meta(Box::new(MetaHirV1::new(
        HirNodeIdV1::from_raw(4),
        SpanV1::new(3, 4),
        MetaKindV1::RequireCertified,
    )));
    let error = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: source,
        source_len: 5,
        items: vec![item, duplicate],
    })
    .unwrap_err();
    let diag = error.diagnostics().first().unwrap();
    assert_eq!(diag.code(), "hir.integrity.duplicate_node_id");
    assert_eq!(
        diag.primary(),
        LocatedSpanV2 {
            source_id: source,
            span: SpanV1::new(3, 4)
        }
    );
    assert_eq!(
        diag.secondary(),
        &[LocatedSpanV2 {
            source_id: source,
            span: SpanV1::new(1, 2)
        }]
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn source_collision_is_rejected_before_body_resolution() {
    let (_host, authority) = host();
    let checked = CheckedSyntaxDocumentV2::try_from_hir(meta_document(1, 8)).unwrap();
    let error = resolve_checked_document_v2(checked, &authority).unwrap_err();
    let FrontendTransactionErrorV2::Document(error) = error else {
        panic!("source collision must be document-local");
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "hir.source_registry.duplicate_source_id"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().source_id,
        SourceIdV2::from_raw(1)
    );
}

#[test]
fn declared_item_set_retains_authored_identity_and_checked_kind() {
    let (host, authority) = host();
    let checked = CheckedSyntaxDocumentV2::try_from_hir(meta_document(9, 8)).unwrap();
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 17,
        })
        .unwrap();
    let declared = DeclaredDocumentV2::try_new(checked, authority.budget(), &extended).unwrap();
    let id = SemanticItemIdV2::new(SourceIdV2::from_raw(9), HirNodeIdV1::from_raw(8));
    assert_eq!(declared.order(), &[id]);
    assert_eq!(
        declared.expected().collect::<Vec<_>>(),
        vec![(id, SpanV1::new(0, 17))]
    );
}

#[test]
fn itemless_documents_retain_the_checked_original_source_anchor() {
    let (host, authority) = host();
    for (source, source_len) in [(9, 0), (23, 37)] {
        let source_id = SourceIdV2::from_raw(source);
        let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len,
            items: Vec::new(),
        })
        .unwrap();
        let document = resolve_checked_document_v2(checked, &authority).unwrap();
        let expected_registry = host
            .try_extend_document(SourceRegistrationV2 {
                source_id,
                source_len,
            })
            .unwrap();
        assert!(document.order().is_empty());
        assert_eq!(document.items().count(), 0);
        assert_eq!(
            document.source_anchor().location(),
            LocatedSpanV2 {
                source_id,
                span: SpanV1::point(0),
            }
        );
        assert_eq!(
            document.source_anchor().registry_identity(),
            &expected_registry.identity()
        );
    }
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn nonempty_certification_directive_is_validated_by_modules_in_one_transaction() {
    let (_host, authority) = host();
    let checked = CheckedSyntaxDocumentV2::try_from_hir(meta_document(9, 8)).unwrap();
    let document = resolve_checked_document_v2(checked, &authority).unwrap();
    let id = SemanticItemIdV2::new(SourceIdV2::from_raw(9), HirNodeIdV1::from_raw(8));
    assert_eq!(document.order(), &[id]);
    let quanta_gqlang_v2_sema::SemanticItemV2::Meta(meta) = document.item(id).unwrap() else {
        panic!("REQUIRE CERTIFIED must retain its typed meta item");
    };
    assert!(matches!(
        meta.kind(),
        quanta_gqlang_v2_sema::ResolvedMetaKindV2::RequireCertified
    ));
    assert_eq!(
        document.catalog().require_certified_location(),
        Some(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(9),
            span: SpanV1::new(0, 17),
        })
    );
}

#[test]
fn certification_checks_the_complete_profile_set_in_both_authored_orders() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    for profile_first in [true, false] {
        for (admission, should_accept) in [
            (AdmissionV1::CertifiedOnly, true),
            (AdmissionV1::Allowlist, false),
        ] {
            let profile_start = if profile_first { 0 } else { 40 };
            let meta_start = if profile_first { 40 } else { 0 };
            let profile = ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                HirNodeIdV1::from_raw(if profile_first { 1 } else { 2 }),
                SpanV1::new(profile_start, profile_start + 35),
                "strict",
                SpanV1::new(profile_start + 8, profile_start + 14),
                CatalogPayloadV1::Profile {
                    capabilities: vec![GraphRefV1 {
                        name: "capability1".into(),
                        span: SpanV1::new(profile_start + 16, profile_start + 27),
                    }],
                    admission,
                },
            )));
            let require = ProgramItemHirV1::Meta(Box::new(MetaHirV1::new(
                HirNodeIdV1::from_raw(if profile_first { 2 } else { 1 }),
                SpanV1::new(meta_start, meta_start + 17),
                MetaKindV1::RequireCertified,
            )));
            let items = if profile_first {
                vec![profile, require]
            } else {
                vec![require, profile]
            };
            let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
                source_id,
                source_len: 80,
                items,
            })
            .expect("both authored orders are valid syntax documents");
            let result = resolve_checked_document_v2(checked, &authority);
            if should_accept {
                let document = result.expect("certified-only profile must satisfy directive");
                assert_eq!(document.order().len(), 2);
                assert_eq!(
                    document.catalog().require_certified_location(),
                    Some(LocatedSpanV2 {
                        source_id,
                        span: SpanV1::new(meta_start, meta_start + 17),
                    })
                );
            } else {
                let FrontendTransactionErrorV2::Document(failure) =
                    result.expect_err("allowlist profile must fail certification")
                else {
                    panic!("certification must fail in the document transaction");
                };
                let [diagnostic] = failure.diagnostics() else {
                    panic!("certification must produce one exact diagnostic");
                };
                assert_eq!(diagnostic.code(), "modules.certified_required_violation");
                assert_eq!(diagnostic.primary().source_id, source_id);
                assert_eq!(
                    diagnostic.primary().span,
                    SpanV1::new(meta_start, meta_start + 17)
                );
            }
        }
    }
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn literal_declaration_uses_binder_then_one_types_kernel_then_typed_document() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let body = ValueExprHirV1::new(
        HirNodeIdV1::from_raw(9),
        SpanV1::new(8, 10),
        ValueExprKindV1::Literal(LiteralV1::Int("42".into())),
    );
    let decl = DeclHirV1::new(
        HirNodeIdV1::from_raw(8),
        SpanV1::new(0, 10),
        "answer".into(),
        SpanV1::new(0, 6),
        body,
    );
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id,
        source_len: 10,
        items: vec![ProgramItemHirV1::Decl(Box::new(decl))],
    })
    .unwrap();
    let document = resolve_checked_document_v2(checked, &authority).unwrap();
    let id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(8));
    let quanta_gqlang_v2_sema::SemanticItemV2::Declaration(typed) = document.item(id).unwrap()
    else {
        panic!("expected typed declaration")
    };
    assert_eq!(typed.name(), "answer");
    assert_eq!(typed.body().key().node_id, HirNodeIdV1::from_raw(9));
    assert!(matches!(
        typed.body().shape(),
        quanta_gqlang_v2_sema::TypedExprShapeV2::Literal(_)
    ));
    assert_eq!(
        typed.body().type_state(),
        &quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(
            quanta_gqlang_v2_types::ValueTypeV1::int()
        )
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn type_diagnostic_preserves_the_inner_expression_span() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let body_span = SpanV1::new(8, 47);
    let body = ValueExprHirV1::new(
        HirNodeIdV1::from_raw(9),
        body_span,
        ValueExprKindV1::Literal(LiteralV1::Int(
            "170141183460469231731687303715884105728".into(),
        )),
    );
    let decl = DeclHirV1::new(
        HirNodeIdV1::from_raw(8),
        SpanV1::new(0, 47),
        "answer".into(),
        SpanV1::new(0, 6),
        body,
    );
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id,
        source_len: 47,
        items: vec![ProgramItemHirV1::Decl(Box::new(decl))],
    })
    .unwrap();
    let FrontendTransactionErrorV2::Document(failure) =
        resolve_checked_document_v2(checked, &authority).unwrap_err()
    else {
        panic!("numeric overflow has an exact source span");
    };
    assert_eq!(
        failure.diagnostics().first().unwrap().code(),
        "types.integer_out_of_range"
    );
    assert_eq!(
        failure.diagnostics().first().unwrap().primary(),
        LocatedSpanV2 {
            source_id,
            span: body_span
        }
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn forward_declaration_dependency_uses_types_owned_lazy_path_and_preserves_binding_id() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let first = DeclHirV1::new(
        HirNodeIdV1::from_raw(2),
        SpanV1::new(0, 9),
        "a".into(),
        SpanV1::new(4, 5),
        ValueExprHirV1::new(
            HirNodeIdV1::from_raw(3),
            SpanV1::new(8, 9),
            ValueExprKindV1::Symbol(SymbolRefV1 { name: "b".into() }),
        ),
    );
    let second = DeclHirV1::new(
        HirNodeIdV1::from_raw(4),
        SpanV1::new(10, 19),
        "b".into(),
        SpanV1::new(14, 15),
        ValueExprHirV1::new(
            HirNodeIdV1::from_raw(5),
            SpanV1::new(18, 19),
            ValueExprKindV1::Literal(LiteralV1::Int("1".into())),
        ),
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 19,
            items: vec![
                ProgramItemHirV1::Decl(Box::new(first)),
                ProgramItemHirV1::Decl(Box::new(second)),
            ],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let first_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: HirNodeIdV1::from_raw(3),
    };
    let typed = document.expression(first_key).unwrap();
    assert_eq!(
        typed.type_state(),
        &quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(
            quanta_gqlang_v2_types::ValueTypeV1::int()
        )
    );
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = typed.shape() else {
        panic!("expected declaration binding")
    };
    assert_eq!(
        (
            binding.target_node(),
            binding.ordinal(),
            binding.canonical_name()
        ),
        (HirNodeIdV1::from_raw(4), 0, "b")
    );
    assert_eq!(document.expressions().count(), 2);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn rule_relational_exists_is_rejected_at_exact_bool_span_before_rule_solver() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let relation =
        builder.relation_source("records", SpanV1::new(27, 34), None, SpanV1::new(27, 34));
    let body = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        relation,
        SpanV1::new(14, 35),
    );
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 35));
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9),
        source_len: 35,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    })
    .unwrap();
    let FrontendTransactionErrorV2::Document(error) =
        resolve_checked_document_v2(checked, &authority).unwrap_err()
    else {
        panic!("rule shape failure must be source anchored")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.invalid_item_shape"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(14, 35)
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn unary_rule_uses_binder_identity_and_types_solver_concrete_head_proof() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let head_id = head.id;
    let body_ref = builder.symbol("x", SpanV1::new(14, 15));
    let body_ref_id = body_ref.id;
    let body = builder.compare(
        "=",
        body_ref,
        builder.int("1", SpanV1::new(18, 19)),
        SpanV1::new(14, 19),
    );
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 20));
    let rule_id = rule.id;
    let source_id = SourceIdV2::from_raw(9);
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 20,
            items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Rule(rule) = document
        .item(SemanticItemIdV2::new(source_id, rule_id))
        .unwrap()
    else {
        panic!("source rule must retain typed body");
    };
    assert_eq!(rule.predicate_id().as_str(), "r");
    assert_eq!(
        document
            .rules()
            .map(|rule| rule.predicate_id().as_str())
            .collect::<Vec<_>>(),
        vec!["r"]
    );
    assert_eq!(
        rule.head_term().type_state(),
        &quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(ValueTypeV1::int())
    );
    let head = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: head_id,
        })
        .unwrap();
    let body_ref = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: body_ref_id,
        })
        .unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(head_binding) = head.shape() else {
        panic!("typed rule head binding expected")
    };
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(body_binding) = body_ref.shape() else {
        panic!("typed rule body binding expected")
    };
    assert_eq!(head_binding, body_binding);
    assert_eq!(
        (head_binding.target_node(), head_binding.ordinal()),
        (rule_id, 0)
    );
    assert_eq!(document.expressions().count(), 4);
}

#[test]
fn built_in_rule_call_retains_typed_call_but_not_a_source_scc_edge() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let anchor = builder.compare(
        "=",
        builder.symbol("x", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)),
        SpanV1::new(14, 19),
    );
    let call = builder.call(
        "is_null",
        SpanV1::new(24, 31),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
            builder.symbol("x", SpanV1::new(32, 33)),
        )])
        .unwrap(),
        SpanV1::new(24, 34),
    );
    let call_id = call.id;
    let body = builder.and(
        anchor,
        builder.bool_value(call, SpanV1::new(24, 34)),
        SpanV1::new(14, 34),
    );
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 35));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 35,
            items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let call = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(9),
            node_id: call_id,
        })
        .unwrap();
    assert!(matches!(
        call.shape(),
        quanta_gqlang_v2_sema::TypedExprShapeV2::Call {
            target: quanta_gqlang_v2_types::ResolvedCallTargetV2::Builtin(_),
            ..
        }
    ));
    assert!(document.rules().next().unwrap().calls().is_empty());
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn host_predicate_call_uses_modules_session_proof_and_never_becomes_local_scc_edge() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let anchor = builder.compare(
        "=",
        builder.symbol("x", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)),
        SpanV1::new(14, 19),
    );
    let call = builder.call(
        "host_pred",
        SpanV1::new(24, 33),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
            builder.symbol("x", SpanV1::new(34, 35)),
        )])
        .unwrap(),
        SpanV1::new(24, 36),
    );
    let call_id = call.id;
    let body = builder.and(
        anchor,
        builder.bool_value(call, SpanV1::new(24, 36)),
        SpanV1::new(14, 36),
    );
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 37));
    let source_id = SourceIdV2::from_raw(9);
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 37,
            items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let call = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: call_id,
        })
        .unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Call {
        target: quanta_gqlang_v2_types::ResolvedCallTargetV2::HostPredicate(key),
        ..
    } = call.shape()
    else {
        panic!("host call must retain a distinct exact catalog key")
    };
    assert!(!key.canonical_bytes().is_empty());
    assert_eq!(
        call.location().location(),
        LocatedSpanV2 {
            source_id,
            span: SpanV1::new(24, 36)
        }
    );
    assert!(document.rules().next().unwrap().calls().is_empty());
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn source_rule_and_host_predicate_name_collision_rejects_at_call_span() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(16, 17));
    let anchor = builder.compare(
        "=",
        builder.symbol("x", SpanV1::new(25, 26)),
        builder.int("1", SpanV1::new(29, 30)),
        SpanV1::new(25, 30),
    );
    let call = builder.call(
        "host_pred",
        SpanV1::new(35, 44),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
            builder.symbol("x", SpanV1::new(45, 46)),
        )])
        .unwrap(),
        SpanV1::new(35, 47),
    );
    let body = builder.and(
        anchor,
        builder.bool_value(call, SpanV1::new(35, 47)),
        SpanV1::new(25, 47),
    );
    let rule = builder.rule(
        "host_pred",
        SpanV1::new(5, 14),
        head,
        body,
        SpanV1::new(0, 48),
    );
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 48,
            items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("dual source and host callable must reject in source")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.ambiguous_callable"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(35, 47)
    );
}

#[test]
fn anchored_source_rule_call_is_a_typed_exact_scc_dependency() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let anchor = builder.compare(
        "=",
        builder.symbol("x", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)),
        SpanV1::new(14, 19),
    );
    let call = builder.call(
        "r",
        SpanV1::new(24, 25),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
            builder.symbol("x", SpanV1::new(26, 27)),
        )])
        .unwrap(),
        SpanV1::new(24, 28),
    );
    let call_id = call.id;
    let body = builder.and(
        anchor,
        builder.bool_value(call, SpanV1::new(24, 28)),
        SpanV1::new(14, 28),
    );
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 29));
    let source_id = SourceIdV2::from_raw(9);
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 29,
            items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let calls = document.rules().next().unwrap().calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls.first().unwrap().call().node_id, call_id);
    assert_eq!(
        calls.first().unwrap().location().location(),
        LocatedSpanV2 {
            source_id,
            span: SpanV1::new(24, 28)
        }
    );
    assert_eq!(
        calls.first().unwrap().polarity(),
        quanta_gqlang_v2_sema::RuleCallPolarityV2::Positive
    );
    assert_eq!(
        calls.first().unwrap().target(),
        &quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(
            quanta_gqlang_v2_types::ResolvedPredicateIdV2::try_new("r").unwrap()
        )
    );
}

#[test]
fn boolean_and_value_parens_preserve_direct_rule_literal_position() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let anchor = builder.compare(
        "=",
        builder.symbol("x", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)),
        SpanV1::new(14, 19),
    );
    let call = builder.call(
        "r",
        SpanV1::new(25, 26),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
            builder.symbol("x", SpanV1::new(27, 28)),
        )])
        .unwrap(),
        SpanV1::new(25, 29),
    );
    let value_grouped = builder.paren_value(call, SpanV1::new(24, 30));
    let literal = builder.bool_value(value_grouped, SpanV1::new(24, 30));
    let grouped = builder.paren_bool(literal, SpanV1::new(23, 31));
    let disjunction = builder.or(
        grouped,
        builder.bool_literal(false, SpanV1::new(34, 39)),
        SpanV1::new(23, 39),
    );
    let body = builder.and(anchor, disjunction, SpanV1::new(14, 39));
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 40));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 40,
            items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let calls = document.rules().next().unwrap().calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls.first().unwrap().polarity(),
        quanta_gqlang_v2_sema::RuleCallPolarityV2::Positive
    );
    assert_eq!(
        calls.first().unwrap().location().location().span,
        SpanV1::new(25, 29)
    );
}

#[test]
fn source_rule_call_inside_scalar_comparison_is_not_a_literal() {
    let (_host, authority) = host();
    for value in [false, true] {
        let builder = HirBuilderV1::new();
        let head = builder.symbol("x", SpanV1::new(9, 10));
        let anchor = builder.compare(
            "=",
            builder.symbol("x", SpanV1::new(14, 15)),
            builder.int("1", SpanV1::new(18, 19)),
            SpanV1::new(14, 19),
        );
        let call = builder.call(
            "r",
            SpanV1::new(24, 25),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
                builder.symbol("x", SpanV1::new(26, 27)),
            )])
            .unwrap(),
            SpanV1::new(24, 28),
        );
        let scalar_comparison = builder.compare(
            "=",
            call,
            builder.bool_lit(value, SpanV1::new(31, 36)),
            SpanV1::new(24, 36),
        );
        let comparison = if value {
            builder.paren_bool(scalar_comparison, SpanV1::new(23, 37))
        } else {
            scalar_comparison
        };
        let end = if value { 38 } else { 37 };
        let body = builder.and(anchor, comparison, SpanV1::new(14, end - 1));
        let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, end));
        let result = resolve_checked_document_v2(
            CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
                source_id: SourceIdV2::from_raw(9),
                source_len: end,
                items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
            })
            .unwrap(),
            &authority,
        );
        let FrontendTransactionErrorV2::Document(error) = result.unwrap_err() else {
            panic!("scalar comparison with {value} must reject in sema");
        };
        let diagnostic = error.diagnostics().first().unwrap();
        assert_eq!(diagnostic.code(), "sema.invalid_item_shape");
        assert_eq!(diagnostic.primary().span, SpanV1::new(24, 28));
    }
}

#[test]
fn source_rule_dependency_inside_scalar_wrappers_rejects_at_call() {
    let (_host, authority) = host();
    for aggregated in [false, true] {
        let builder = HirBuilderV1::new();
        let head = builder.symbol("x", SpanV1::new(9, 10));
        let anchor = builder.compare(
            "=",
            builder.symbol("x", SpanV1::new(14, 15)),
            builder.int("1", SpanV1::new(18, 19)),
            SpanV1::new(14, 19),
        );
        let recursive = builder.call(
            "r",
            SpanV1::new(40, 41),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
                builder.symbol("x", SpanV1::new(42, 43)),
            )])
            .unwrap(),
            SpanV1::new(40, 44),
        );
        let predicate = if aggregated {
            let count = builder.call(
                "count",
                SpanV1::new(34, 39),
                CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(recursive)]).unwrap(),
                SpanV1::new(34, 45),
            );
            builder.compare(
                ">",
                count,
                builder.int("0", SpanV1::new(48, 49)),
                SpanV1::new(34, 49),
            )
        } else {
            let count = builder.call(
                "count",
                SpanV1::new(24, 29),
                CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
                    builder.int("1", SpanV1::new(30, 31)),
                )])
                .unwrap(),
                SpanV1::new(24, 32),
            );
            let record = builder.record(
                vec![
                    RecordFieldV1 {
                        name: "aggregate".into(),
                        name_span: SpanV1::new(22, 23),
                        value: count,
                    },
                    RecordFieldV1 {
                        name: "recursive".into(),
                        name_span: SpanV1::new(36, 37),
                        value: recursive,
                    },
                ],
                SpanV1::new(22, 45),
            );
            let wrapper = builder.call(
                "is_null",
                SpanV1::new(20, 27),
                CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(record)]).unwrap(),
                SpanV1::new(20, 46),
            );
            builder.bool_value(wrapper, SpanV1::new(20, 46))
        };
        let body = builder.and(anchor, predicate, SpanV1::new(14, 49));
        let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 50));
        let result = resolve_checked_document_v2(
            CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
                source_id: SourceIdV2::from_raw(9),
                source_len: 50,
                items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
            })
            .unwrap(),
            &authority,
        );
        let FrontendTransactionErrorV2::Document(error) = result.unwrap_err() else {
            panic!("source call under a scalar wrapper must reject in sema");
        };
        let diagnostic = error.diagnostics().first().unwrap();
        assert_eq!(diagnostic.code(), "sema.invalid_item_shape");
        assert_eq!(diagnostic.primary().span, SpanV1::new(40, 44));
    }
}

#[test]
fn negated_source_dependency_retains_exact_polarity_for_logic_cycle_rejection() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let anchor = builder.compare(
        "=",
        builder.symbol("x", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)),
        SpanV1::new(14, 19),
    );
    let call = builder.call(
        "r",
        SpanV1::new(28, 29),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
            builder.symbol("x", SpanV1::new(30, 31)),
        )])
        .unwrap(),
        SpanV1::new(28, 32),
    );
    let body = builder.and(
        anchor,
        builder.not(
            builder.bool_value(call, SpanV1::new(28, 32)),
            SpanV1::new(24, 32),
        ),
        SpanV1::new(14, 32),
    );
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 33));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 33,
            items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let calls = document.rules().next().unwrap().calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls.first().unwrap().polarity(),
        quanta_gqlang_v2_sema::RuleCallPolarityV2::Negated
    );
    assert_eq!(
        calls.first().unwrap().location().location().span,
        SpanV1::new(28, 32)
    );
}

#[test]
fn two_authored_rules_retain_mutual_source_dependency_edges() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let make_rule = |name: &str, callee: &str, start: usize| {
        let head = builder.symbol("x", SpanV1::new(start + 9, start + 10));
        let anchor = builder.compare(
            "=",
            builder.symbol("x", SpanV1::new(start + 14, start + 15)),
            builder.int("1", SpanV1::new(start + 18, start + 19)),
            SpanV1::new(start + 14, start + 19),
        );
        let call = builder.call(
            callee,
            SpanV1::new(start + 24, start + 25),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(
                builder.symbol("x", SpanV1::new(start + 26, start + 27)),
            )])
            .unwrap(),
            SpanV1::new(start + 24, start + 28),
        );
        let body = builder.and(
            anchor,
            builder.bool_value(call, SpanV1::new(start + 24, start + 28)),
            SpanV1::new(start + 14, start + 28),
        );
        ProgramItemHirV1::Rule(Box::new(builder.rule(
            name,
            SpanV1::new(start + 5, start + 6),
            head,
            body,
            SpanV1::new(start, start + 29),
        )))
    };
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 59,
            items: vec![make_rule("r", "s", 0), make_rule("s", "r", 30)],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let edges = document
        .rules()
        .map(|rule| {
            (
                rule.predicate_id().as_str().to_owned(),
                rule.calls()
                    .iter()
                    .map(|call| call.target().clone())
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        edges,
        vec![
            (
                "r".to_owned(),
                vec![quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(
                    quanta_gqlang_v2_types::ResolvedPredicateIdV2::try_new("s").unwrap()
                )]
            ),
            (
                "s".to_owned(),
                vec![quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(
                    quanta_gqlang_v2_types::ResolvedPredicateIdV2::try_new("r").unwrap()
                )]
            ),
        ]
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn unanchored_rule_head_has_no_fabricated_concrete_type() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let body = builder.compare(
        "=",
        builder.symbol("x", SpanV1::new(14, 15)),
        builder.symbol("x", SpanV1::new(18, 19)),
        SpanV1::new(14, 19),
    );
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 20));
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9),
        source_len: 20,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    })
    .unwrap();
    let FrontendTransactionErrorV2::Document(error) =
        resolve_checked_document_v2(checked, &authority).unwrap_err()
    else {
        panic!("unanchored rule is not a closed semantic document");
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "types.ambiguous_rule_type"
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn nonsymbol_rule_head_rejects_before_binder_and_type_solver() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.int("1", SpanV1::new(9, 10));
    let body = builder.compare(
        "=",
        builder.int("1", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)),
        SpanV1::new(14, 19),
    );
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 20));
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9),
        source_len: 20,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    })
    .unwrap();
    let FrontendTransactionErrorV2::Document(error) =
        resolve_checked_document_v2(checked, &authority).unwrap_err()
    else {
        panic!("non-symbol head cannot be a unary rule");
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.invalid_item_shape"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(9, 10)
    );
}
