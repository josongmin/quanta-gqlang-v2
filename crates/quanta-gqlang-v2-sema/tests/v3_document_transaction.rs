#![expect(clippy::unwrap_used, reason = "owner assertions inspect exact transaction diagnostics")]

use quanta_gqlang_v2_hir::{
    ActivationV1, AdmissionV1, AliasV1, CallArgHirV1, CallArgsHirV1, CatalogDeclHirV1, CatalogPayloadV1, ConfirmModeV1, DeclHirV1, DetectionModeV1,
    EmitModeV1, GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1,
    GateBindingV1, GraphRefV1, HirBuilderV1, HirDocumentV1, HirNodeIdV1, LegalityHirV1, LiteralV1, LocatedSpanV2, MetaHirV1,
    MetaKindV1, OrderKeyHirV1, PathSegmentV1, PresentationHirV1, PresentationKindV1, ProgramItemHirV1, ProjectionItemV1,
    ShowCategoryV1, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2,
    RelationExistsPolarityV1, SortDirectionHirV1, SpanV1, SymbolRefV1, ValueExprHirV1, ValueExprKindV1, ViewBodyV1,
};
use quanta_gqlang_v2_modules::{
    CallableKindV2, CurrentModuleSelectionInputV2, ExternalCatalogAssetInputV2, ExternalCatalogPayloadInputV2,
    LocatedParameterInputV2, ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2, SignatureInputV2, VersionV2,
};
use quanta_gqlang_v2_sema::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    DeclaredDocumentV2, FrontendBudgetV2, FrontendTransactionErrorV2,
    GraphReferenceCatalogV2, GraphReferenceInputV2, GraphReferenceKindV2, SemanticItemIdV2,
};
use quanta_gqlang_v2_types::{CardinalityV1, ExpressionEffectsV2, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1, RelationCatalogV2, RelationSourceInputV2, ValueTypeV1};

fn host() -> (SourceRegistryViewV2, ClosedFrontendAuthorityV2) {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1), source_len: 8,
    }]).unwrap();
    let location = host.try_check_location(LocatedSpanV2 {
        source_id: SourceIdV2::from_raw(1), span: SpanV1::new(0, 1),
    }).unwrap();
    let module = ModuleIdV2::try_new("app", location.clone()).unwrap();
    let budget = FrontendBudgetV2::standard_v2().unwrap();
    let profile = budget.authority_profile().unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).unwrap();
    builder.try_add_module(ModuleNodeInputV2 { module_id: module.clone(), location: location.clone() }).unwrap();
    builder.try_add_external(ExternalCatalogAssetInputV2 {
        module_name: "app".into(), local_name: "check1".into(), kind_token: "check".into(),
        typed_payload: ExternalCatalogPayloadInputV2::Check, provenance: "host".into(),
        location: host.try_check_location(LocatedSpanV2 { source_id: SourceIdV2::from_raw(1), span: SpanV1::new(3, 4) }).unwrap(),
        dependencies: Vec::new(),
    }).unwrap();
    for (kind_token, local_name, typed_payload) in [
        ("query", "query1", ExternalCatalogPayloadInputV2::Query),
        ("model_source", "model_source1", ExternalCatalogPayloadInputV2::ModelSource),
        ("pack_source", "pack_source1", ExternalCatalogPayloadInputV2::PackSource),
        ("capability", "capability1", ExternalCatalogPayloadInputV2::Capability),
    ] {
        builder.try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "app".into(), local_name: local_name.into(), kind_token: kind_token.into(),
            typed_payload, provenance: "host".into(),
            location: host.try_check_location(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(1), span: SpanV1::new(3, 4),
            }).unwrap(),
            dependencies: Vec::new(),
        }).unwrap();
    }
    builder.try_add_external(ExternalCatalogAssetInputV2 {
        module_name: "app".into(), local_name: "host_pred".into(), kind_token: "predicate".into(),
        typed_payload: ExternalCatalogPayloadInputV2::Predicate(SignatureInputV2 {
            parameters: vec![LocatedParameterInputV2 { name: "value".into(), value_type: ValueTypeV1::int(), location: location.clone() }],
            result: ValueTypeV1::bool(), callable_kind: CallableKindV2::Predicate,
            effects: ExpressionEffectsV2::pure_scalar(), location: location.clone(),
        }), provenance: "host".into(), location: location.clone(), dependencies: Vec::new(),
    }).unwrap();
    let modules = builder.finish(CurrentModuleSelectionInputV2 { module_id: module, location }, VersionV2::new(1, 0, 0)).unwrap();
    let relation_location = host.try_check_location(LocatedSpanV2 { source_id: SourceIdV2::from_raw(1), span: SpanV1::new(4, 5) }).unwrap();
    let relations = RelationCatalogV2::try_new(vec![RelationSourceInputV2 {
        provenance_namespace: ProvenanceNamespaceV2::try_new("host", relation_location.clone()).unwrap(),
        qualified_name: "records".into(), lookup_name: "records".into(),
        row: RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), relation_location.clone()).unwrap()]).unwrap(), cardinality: CardinalityV1::Bag,
        provenance: "host".into(), location: relation_location,
    }], &host, profile).unwrap();
    let graph = GraphReferenceCatalogV2::try_new([
        ("repo::src", SpanV1::new(1, 2)), ("repo::sink", SpanV1::new(2, 3)),
    ].into_iter().map(|(spelling, span)| GraphReferenceInputV2 {
        spelling: spelling.into(), kind: GraphReferenceKindV2::Entity, entity_subtype: None,
        location: host.try_check_location(LocatedSpanV2 { source_id: SourceIdV2::from_raw(1), span }).unwrap(),
    }).collect(), &host, budget.authority_profile().unwrap()).unwrap();
    (host, ClosedFrontendAuthorityV2::try_new(modules, relations, graph, budget).unwrap())
}

fn meta_document(source: u32, node: u32) -> HirDocumentV1 {
    HirDocumentV1 {
        source_id: SourceIdV2::from_raw(source), source_len: 17,
        items: vec![ProgramItemHirV1::Meta(Box::new(MetaHirV1::new(
            HirNodeIdV1::from_raw(node), SpanV1::new(0, 17), MetaKindV1::RequireCertified,
        )))],
    }
}

#[test]
fn checked_syntax_rejects_document_wide_duplicate_ids_with_exact_spans() {
    let source = SourceIdV2::from_raw(9);
    let item = ProgramItemHirV1::Meta(Box::new(MetaHirV1::new(
        HirNodeIdV1::from_raw(4), SpanV1::new(1, 2), MetaKindV1::RequireCertified,
    )));
    let duplicate = ProgramItemHirV1::Meta(Box::new(MetaHirV1::new(
        HirNodeIdV1::from_raw(4), SpanV1::new(3, 4), MetaKindV1::RequireCertified,
    )));
    let error = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: source, source_len: 5, items: vec![item, duplicate],
    }).unwrap_err();
    let diag = &error.diagnostics()[0];
    assert_eq!(diag.code(), "hir.integrity.duplicate_node_id");
    assert_eq!(diag.primary(), LocatedSpanV2 { source_id: source, span: SpanV1::new(3, 4) });
    assert_eq!(diag.secondary(), &[LocatedSpanV2 { source_id: source, span: SpanV1::new(1, 2) }]);
}

#[test]
fn source_collision_is_rejected_before_body_resolution() {
    let (_host, authority) = host();
    let checked = CheckedSyntaxDocumentV2::try_from_hir(meta_document(1, 8)).unwrap();
    let error = resolve_checked_document_v2(checked, &authority).unwrap_err();
    let FrontendTransactionErrorV2::Document(error) = error else { panic!("source collision must be document-local"); };
    assert_eq!(error.diagnostics()[0].code(), "hir.source_registry.duplicate_source_id");
    assert_eq!(error.diagnostics()[0].primary().source_id, SourceIdV2::from_raw(1));
}

#[test]
fn declared_item_set_retains_authored_identity_and_checked_kind() {
    let (host, authority) = host();
    let checked = CheckedSyntaxDocumentV2::try_from_hir(meta_document(9, 8)).unwrap();
    let extended = host.try_extend_document(SourceRegistrationV2 { source_id: SourceIdV2::from_raw(9), source_len: 17 }).unwrap();
    let declared = DeclaredDocumentV2::try_new(checked, authority.budget(), &extended).unwrap();
    let id = SemanticItemIdV2::new(SourceIdV2::from_raw(9), HirNodeIdV1::from_raw(8));
    assert_eq!(declared.order(), &[id]);
    assert_eq!(declared.expected().collect::<Vec<_>>(), vec![(id, SpanV1::new(0, 17))]);
}

#[test]
fn nonempty_certification_directive_is_validated_by_modules_in_one_transaction() {
    let (_host, authority) = host();
    let checked = CheckedSyntaxDocumentV2::try_from_hir(meta_document(9, 8)).unwrap();
    let document = resolve_checked_document_v2(checked, &authority).unwrap();
    let id = SemanticItemIdV2::new(SourceIdV2::from_raw(9), HirNodeIdV1::from_raw(8));
    assert_eq!(document.order(), &[id]);
    let quanta_gqlang_v2_sema::SemanticItemV2::Meta(meta) = document.item(id).unwrap() else {
        panic!("REQUIRE CERTIFIED must retain its typed meta item");
    };
    assert!(matches!(meta.kind(), quanta_gqlang_v2_sema::ResolvedMetaKindV2::RequireCertified));
    assert_eq!(document.catalog().require_certified_location(), Some(LocatedSpanV2 {
        source_id: SourceIdV2::from_raw(9), span: SpanV1::new(0, 17),
    }));
}

#[test]
fn literal_declaration_uses_binder_then_one_types_kernel_then_typed_document() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let body = ValueExprHirV1::new(HirNodeIdV1::from_raw(9), SpanV1::new(8, 10), ValueExprKindV1::Literal(LiteralV1::Int("42".into())));
    let decl = DeclHirV1::new(HirNodeIdV1::from_raw(8), SpanV1::new(0, 10), "answer".into(), SpanV1::new(0, 6), body);
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 10, items: vec![ProgramItemHirV1::Decl(Box::new(decl))],
    }).unwrap();
    let document = resolve_checked_document_v2(checked, &authority).unwrap();
    let id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(8));
    let quanta_gqlang_v2_sema::SemanticItemV2::Declaration(typed) = document.item(id).unwrap() else { panic!("expected typed declaration") };
    assert_eq!(typed.name(), "answer");
    assert_eq!(typed.body().key().node_id, HirNodeIdV1::from_raw(9));
    assert!(matches!(typed.body().shape(), quanta_gqlang_v2_sema::TypedExprShapeV2::Literal(_)));
    assert_eq!(typed.body().type_state(), &quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(quanta_gqlang_v2_types::ValueTypeV1::int()));
}

#[test]
fn type_diagnostic_preserves_the_inner_expression_span() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let body_span = SpanV1::new(8, 47);
    let body = ValueExprHirV1::new(HirNodeIdV1::from_raw(9), body_span,
        ValueExprKindV1::Literal(LiteralV1::Int("170141183460469231731687303715884105728".into())));
    let decl = DeclHirV1::new(HirNodeIdV1::from_raw(8), SpanV1::new(0, 47),
        "answer".into(), SpanV1::new(0, 6), body);
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 47, items: vec![ProgramItemHirV1::Decl(Box::new(decl))],
    }).unwrap();
    let FrontendTransactionErrorV2::Document(failure) = resolve_checked_document_v2(checked, &authority).unwrap_err() else {
        panic!("numeric overflow has an exact source span");
    };
    assert_eq!(failure.diagnostics()[0].code(), "types.integer_out_of_range");
    assert_eq!(failure.diagnostics()[0].primary(), LocatedSpanV2 { source_id, span: body_span });
}

#[test]
fn forward_declaration_dependency_uses_types_owned_lazy_path_and_preserves_binding_id() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let first = DeclHirV1::new(
        HirNodeIdV1::from_raw(2), SpanV1::new(0, 9), "a".into(), SpanV1::new(4, 5),
        ValueExprHirV1::new(HirNodeIdV1::from_raw(3), SpanV1::new(8, 9), ValueExprKindV1::Symbol(SymbolRefV1 { name: "b".into() })),
    );
    let second = DeclHirV1::new(
        HirNodeIdV1::from_raw(4), SpanV1::new(10, 19), "b".into(), SpanV1::new(14, 15),
        ValueExprHirV1::new(HirNodeIdV1::from_raw(5), SpanV1::new(18, 19), ValueExprKindV1::Literal(LiteralV1::Int("1".into()))),
    );
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 19,
        items: vec![ProgramItemHirV1::Decl(Box::new(first)), ProgramItemHirV1::Decl(Box::new(second))],
    }).unwrap(), &authority).unwrap();
    let first_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 { source_id, node_id: HirNodeIdV1::from_raw(3) };
    let typed = document.expression(first_key).unwrap();
    assert_eq!(typed.type_state(), &quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(quanta_gqlang_v2_types::ValueTypeV1::int()));
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = typed.shape() else { panic!("expected declaration binding") };
    assert_eq!((binding.target_node(), binding.ordinal(), binding.canonical_name()), (HirNodeIdV1::from_raw(4), 0, "b"));
    assert_eq!(document.expressions().count(), 2);
}

#[test]
fn rule_relational_exists_is_rejected_at_exact_bool_span_before_rule_solver() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let relation = builder.relation_source("records", None, SpanV1::new(27, 34));
    let body = builder.relation_exists(RelationExistsPolarityV1::Exists, relation, SpanV1::new(14, 35));
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 35));
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9), source_len: 35,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    }).unwrap();
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(checked, &authority).unwrap_err() else { panic!("rule shape failure must be source anchored") };
    assert_eq!(error.diagnostics()[0].code(), "sema.invalid_item_shape");
    assert_eq!(error.diagnostics()[0].primary().span, SpanV1::new(14, 35));
}

#[test]
fn unary_rule_uses_binder_identity_and_types_solver_concrete_head_proof() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let head_id = head.id;
    let body_ref = builder.symbol("x", SpanV1::new(14, 15));
    let body_ref_id = body_ref.id;
    let body = builder.compare("=", body_ref, builder.int("1", SpanV1::new(18, 19)), SpanV1::new(14, 19));
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 20));
    let rule_id = rule.id;
    let source_id = SourceIdV2::from_raw(9);
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 20, items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    }).unwrap(), &authority).unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Rule(rule) = document.item(SemanticItemIdV2::new(source_id, rule_id)).unwrap() else {
        panic!("source rule must retain typed body");
    };
    assert_eq!(rule.predicate_id().as_str(), "r");
    assert_eq!(document.rules().map(|rule| rule.predicate_id().as_str()).collect::<Vec<_>>(), vec!["r"]);
    assert_eq!(rule.head_term().type_state(), &quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(ValueTypeV1::int()));
    let head = document.expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 { source_id, node_id: head_id }).unwrap();
    let body_ref = document.expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 { source_id, node_id: body_ref_id }).unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(head_binding) = head.shape() else { panic!("typed rule head binding expected") };
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(body_binding) = body_ref.shape() else { panic!("typed rule body binding expected") };
    assert_eq!(head_binding, body_binding);
    assert_eq!((head_binding.target_node(), head_binding.ordinal()), (rule_id, 0));
    assert_eq!(document.expressions().count(), 4);
}

#[test]
fn built_in_rule_call_retains_typed_call_but_not_a_source_scc_edge() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let anchor = builder.compare("=", builder.symbol("x", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)), SpanV1::new(14, 19));
    let call = builder.call("is_null", SpanV1::new(24, 31),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(builder.symbol("x", SpanV1::new(32, 33)))]).unwrap(),
        SpanV1::new(24, 34));
    let call_id = call.id;
    let body = builder.and(anchor, builder.bool_value(call, SpanV1::new(24, 34)), SpanV1::new(14, 34));
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 35));
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9), source_len: 35,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    }).unwrap(), &authority).unwrap();
    let call = document.expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(9), node_id: call_id,
    }).unwrap();
    assert!(matches!(call.shape(), quanta_gqlang_v2_sema::TypedExprShapeV2::Call { target: quanta_gqlang_v2_types::ResolvedCallTargetV2::Builtin(_), .. }));
    assert!(document.rules().next().unwrap().calls().is_empty());
}

#[test]
fn host_predicate_call_uses_modules_session_proof_and_never_becomes_local_scc_edge() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let anchor = builder.compare("=", builder.symbol("x", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)), SpanV1::new(14, 19));
    let call = builder.call("host_pred", SpanV1::new(24, 33),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(builder.symbol("x", SpanV1::new(34, 35)))]).unwrap(),
        SpanV1::new(24, 36));
    let call_id = call.id;
    let body = builder.and(anchor, builder.bool_value(call, SpanV1::new(24, 36)), SpanV1::new(14, 36));
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 37));
    let source_id = SourceIdV2::from_raw(9);
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 37, items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    }).unwrap(), &authority).unwrap();
    let call = document.expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 { source_id, node_id: call_id }).unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Call {
        target: quanta_gqlang_v2_types::ResolvedCallTargetV2::HostPredicate(key), ..
    } = call.shape() else { panic!("host call must retain a distinct exact catalog key") };
    assert!(!key.canonical_bytes().is_empty());
    assert_eq!(call.location().location(), LocatedSpanV2 { source_id, span: SpanV1::new(24, 36) });
    assert!(document.rules().next().unwrap().calls().is_empty());
}

#[test]
fn source_rule_and_host_predicate_name_collision_rejects_at_call_span() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(16, 17));
    let anchor = builder.compare("=", builder.symbol("x", SpanV1::new(25, 26)),
        builder.int("1", SpanV1::new(29, 30)), SpanV1::new(25, 30));
    let call = builder.call("host_pred", SpanV1::new(35, 44),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(builder.symbol("x", SpanV1::new(45, 46)))]).unwrap(),
        SpanV1::new(35, 47));
    let body = builder.and(anchor, builder.bool_value(call, SpanV1::new(35, 47)), SpanV1::new(25, 47));
    let rule = builder.rule("host_pred", SpanV1::new(5, 14), head, body, SpanV1::new(0, 48));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9), source_len: 48,
            items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
        }).unwrap(), &authority,
    ).unwrap_err() else { panic!("dual source and host callable must reject in source") };
    assert_eq!(error.diagnostics()[0].code(), "sema.ambiguous_callable");
    assert_eq!(error.diagnostics()[0].primary().span, SpanV1::new(35, 47));
}

#[test]
fn anchored_source_rule_call_is_a_typed_exact_scc_dependency() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let anchor = builder.compare("=", builder.symbol("x", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)), SpanV1::new(14, 19));
    let call = builder.call("r", SpanV1::new(24, 25),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(builder.symbol("x", SpanV1::new(26, 27)))]).unwrap(),
        SpanV1::new(24, 28));
    let call_id = call.id;
    let body = builder.and(anchor, builder.bool_value(call, SpanV1::new(24, 28)), SpanV1::new(14, 28));
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 29));
    let source_id = SourceIdV2::from_raw(9);
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 29, items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    }).unwrap(), &authority).unwrap();
    let calls = document.rules().next().unwrap().calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].call().node_id, call_id);
    assert_eq!(calls[0].location().location(), LocatedSpanV2 { source_id, span: SpanV1::new(24, 28) });
    assert_eq!(calls[0].polarity(), quanta_gqlang_v2_sema::RuleCallPolarityV2::Positive);
    assert_eq!(calls[0].target(), &quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(
        quanta_gqlang_v2_types::ResolvedPredicateIdV2::try_new("r").unwrap()));
}

#[test]
fn negated_source_dependency_retains_exact_polarity_for_logic_cycle_rejection() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let anchor = builder.compare("=", builder.symbol("x", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)), SpanV1::new(14, 19));
    let call = builder.call("r", SpanV1::new(28, 29),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(builder.symbol("x", SpanV1::new(30, 31)))]).unwrap(),
        SpanV1::new(28, 32));
    let body = builder.and(anchor, builder.not(builder.bool_value(call, SpanV1::new(28, 32)), SpanV1::new(24, 32)), SpanV1::new(14, 32));
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 33));
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9), source_len: 33,
        items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    }).unwrap(), &authority).unwrap();
    let calls = document.rules().next().unwrap().calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].polarity(), quanta_gqlang_v2_sema::RuleCallPolarityV2::Negated);
    assert_eq!(calls[0].location().location().span, SpanV1::new(28, 32));
}

#[test]
fn two_authored_rules_retain_mutual_source_dependency_edges() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let make_rule = |name: &str, callee: &str, start: usize| {
        let head = builder.symbol("x", SpanV1::new(start + 9, start + 10));
        let anchor = builder.compare("=", builder.symbol("x", SpanV1::new(start + 14, start + 15)),
            builder.int("1", SpanV1::new(start + 18, start + 19)), SpanV1::new(start + 14, start + 19));
        let call = builder.call(callee, SpanV1::new(start + 24, start + 25),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(builder.symbol("x", SpanV1::new(start + 26, start + 27)))]).unwrap(),
            SpanV1::new(start + 24, start + 28));
        let body = builder.and(anchor, builder.bool_value(call, SpanV1::new(start + 24, start + 28)), SpanV1::new(start + 14, start + 28));
        ProgramItemHirV1::Rule(Box::new(builder.rule(name, SpanV1::new(start + 5, start + 6), head, body, SpanV1::new(start, start + 29))))
    };
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9), source_len: 59,
        items: vec![make_rule("r", "s", 0), make_rule("s", "r", 30)],
    }).unwrap(), &authority).unwrap();
    let edges = document.rules().map(|rule| (
        rule.predicate_id().as_str().to_owned(), rule.calls().iter().map(|call| call.target().clone()).collect::<Vec<_>>(),
    )).collect::<Vec<_>>();
    assert_eq!(edges, vec![
        ("r".to_owned(), vec![quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(quanta_gqlang_v2_types::ResolvedPredicateIdV2::try_new("s").unwrap())]),
        ("s".to_owned(), vec![quanta_gqlang_v2_types::ResolvedCallTargetV2::Predicate(quanta_gqlang_v2_types::ResolvedPredicateIdV2::try_new("r").unwrap())]),
    ]);
}

#[test]
fn unanchored_rule_head_has_no_fabricated_concrete_type() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.symbol("x", SpanV1::new(9, 10));
    let body = builder.compare("=", builder.symbol("x", SpanV1::new(14, 15)),
        builder.symbol("x", SpanV1::new(18, 19)), SpanV1::new(14, 19));
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 20));
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9), source_len: 20, items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    }).unwrap();
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(checked, &authority).unwrap_err() else {
        panic!("unanchored rule is not a closed semantic document");
    };
    assert_eq!(error.diagnostics()[0].code(), "types.ambiguous_rule_type");
}

#[test]
fn nonsymbol_rule_head_rejects_before_binder_and_type_solver() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let head = builder.int("1", SpanV1::new(9, 10));
    let body = builder.compare("=", builder.int("1", SpanV1::new(14, 15)),
        builder.int("1", SpanV1::new(18, 19)), SpanV1::new(14, 19));
    let rule = builder.rule("r", SpanV1::new(5, 6), head, body, SpanV1::new(0, 20));
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9), source_len: 20, items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
    }).unwrap();
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(checked, &authority).unwrap_err() else {
        panic!("non-symbol head cannot be a unary rule");
    };
    assert_eq!(error.diagnostics()[0].code(), "sema.invalid_item_shape");
    assert_eq!(error.diagnostics()[0].primary().span, SpanV1::new(9, 10));
}

#[test]
fn graph_references_keep_source_roles_and_checked_use_locations() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let graph = GraphQueryHirV1::new(
        HirNodeIdV1::from_raw(2), SpanV1::new(0, 38), GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 { name: "repo::src".into(), span: SpanV1::new(10, 19) }),
            sink: Some(GraphRefV1 { name: "repo::sink".into(), span: SpanV1::new(25, 35) }),
            ..GraphEndpointsV1::default()
        }, GraphConstraintsV1::default(),
    );
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 38, items: vec![ProgramItemHirV1::GraphQuery(Box::new(graph))],
    }).unwrap(), &authority).unwrap();
    let id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(2));
    let quanta_gqlang_v2_sema::SemanticItemV2::Graph(graph) = document.item(id).unwrap() else { panic!("typed FIND expected") };
    let references = graph.references();
    assert_eq!(graph.location().location(), LocatedSpanV2 { source_id, span: SpanV1::new(0, 38) });
    assert_eq!(graph.witness_bounds().max_depth(), authority.budget().max_witness_depth());
    assert_eq!(graph.witness_bounds().max_witnesses(), authority.budget().max_witnesses());
    assert_eq!(references.len(), 2);
    assert_eq!(references[0].role(), quanta_gqlang_v2_sema::GraphReferenceRoleV2::Source);
    assert_eq!(references[0].target().spelling(), "repo::src");
    assert_eq!(references[0].use_location().location(), LocatedSpanV2 { source_id, span: SpanV1::new(10, 19) });
    assert!(authority.graph_references().admits(references[1].target()));
}

#[test]
fn graph_depth_over_frontend_witness_budget_fails_at_query_span() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let graph = GraphQueryHirV1::new(
        HirNodeIdV1::from_raw(2), SpanV1::new(0, 8), GraphQueryKindV1::Path,
        GraphEndpointsV1::default(), GraphConstraintsV1 {
            depth: Some(authority.budget().max_witness_depth() + 1),
            ..GraphConstraintsV1::default()
        },
    );
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 8, items: vec![ProgramItemHirV1::GraphQuery(Box::new(graph))],
    }).unwrap();
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(checked, &authority).unwrap_err() else {
        panic!("over-budget graph depth must be source anchored");
    };
    assert_eq!(error.diagnostics()[0].code(), "frontend.resource_limit_exceeded");
    assert_eq!(error.diagnostics()[0].primary(), LocatedSpanV2 { source_id, span: SpanV1::new(0, 8) });
}

#[test]
fn presentation_show_is_a_closed_source_item_without_phantom_reference() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let show = PresentationHirV1::new(HirNodeIdV1::from_raw(8), SpanV1::new(0, 10),
        PresentationKindV1::Show { category: ShowCategoryV1::Models });
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 10, items: vec![ProgramItemHirV1::Presentation(Box::new(show))],
    }).unwrap(), &authority).unwrap();
    let id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(8));
    let quanta_gqlang_v2_sema::SemanticItemV2::Presentation(item) = document.item(id).unwrap() else { panic!("SHOW must remain presentation") };
    assert!(matches!(item.kind(), quanta_gqlang_v2_sema::ResolvedPresentationKindV2::Show { category: ShowCategoryV1::Models }));
}

#[test]
fn catalog_rule_binding_and_check_share_modules_validated_exact_reference() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let binding = CatalogDeclHirV1::new(
        HirNodeIdV1::from_raw(8), SpanV1::new(0, 30), "rule1", SpanV1::new(5, 10),
        CatalogPayloadV1::RuleBinding { check_ref: GraphRefV1 { name: "check1".into(), span: SpanV1::new(21, 27) } },
    );
    let check = LegalityHirV1::new(
        HirNodeIdV1::from_raw(9), SpanV1::new(31, 60),
        GraphRefV1 { name: "rule1".into(), span: SpanV1::new(40, 45) }, None, None,
        ConfirmModeV1::Skip, EmitModeV1::Findings, DetectionModeV1::PatternOnly,
    );
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 60,
        items: vec![ProgramItemHirV1::CatalogDecl(Box::new(binding)), ProgramItemHirV1::Legality(Box::new(check))],
    }).unwrap(), &authority).unwrap();
    let catalog_id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(8));
    let check_id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(9));
    let quanta_gqlang_v2_sema::SemanticItemV2::Catalog(rule) = document.item(catalog_id).unwrap() else { panic!("catalog Rule binding expected") };
    let quanta_gqlang_v2_sema::SemanticItemV2::Check(check) = document.item(check_id).unwrap() else { panic!("typed CHECK expected") };
    assert_eq!(check.rule().id(), rule.asset_id());
    assert_eq!(document.catalog().dependencies().len(), 1);
    let edge = &document.catalog().dependencies()[0];
    assert_eq!(edge.source_asset_id(), rule.asset_id());
    assert_eq!(edge.family(), Some(quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::RuleBindingCheck));
}

#[test]
fn catalog_reference_families_use_one_modules_predeclaration_and_validation_session() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let cases = [
        ("view1", CatalogPayloadV1::View(ViewBodyV1::QueryRef(GraphRefV1 { name: "query1".into(), span: SpanV1::new(11, 17) }))),
        ("model1", CatalogPayloadV1::Model { provider: "opaque-provider".into(), source: GraphRefV1 { name: "model_source1".into(), span: SpanV1::new(31, 44) } }),
        ("pack1", CatalogPayloadV1::Pack { source: GraphRefV1 { name: "pack_source1".into(), span: SpanV1::new(51, 63) }, activation: ActivationV1::Manual }),
        ("profile1", CatalogPayloadV1::Profile { capabilities: vec![GraphRefV1 { name: "capability1".into(), span: SpanV1::new(71, 82) }], admission: AdmissionV1::Allowlist }),
        ("gate1", CatalogPayloadV1::Gate { checks: vec![GraphRefV1 { name: "check1".into(), span: SpanV1::new(91, 97) }], binding: GateBindingV1::All }),
    ];
    let items = cases.into_iter().enumerate().map(|(index, (name, payload))| {
        let start = index * 20;
        ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
            HirNodeIdV1::from_raw(index as u32 + 1), SpanV1::new(start, start + 25),
            name, SpanV1::new(start + 1, start + 5), payload,
        )))
    }).collect();
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 105, items,
    }).unwrap(), &authority).unwrap();
    assert_eq!(document.order().len(), 5);
    assert_eq!(document.catalog().dependencies().len(), 5);
    assert!(document.items().all(|(_, item)| matches!(item, quanta_gqlang_v2_sema::SemanticItemV2::Catalog(_))));
    let families = document.catalog().dependencies().iter().map(|edge| edge.family().unwrap()).collect::<Vec<_>>();
    assert!(families.contains(&quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::ViewQuery));
    assert!(families.contains(&quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::ModelSource));
    assert!(families.contains(&quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::PackSource));
    assert!(families.contains(&quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::ProfileCapability));
    assert!(families.contains(&quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::GateCheck));
}

#[test]
fn query_scan_uses_binder_frame_and_types_owned_host_relation_contract() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", None, SpanV1::new(5, 12));
    let stage_node = scan.id;
    let query = quanta_gqlang_v2_hir::QueryHirV1::new(HirNodeIdV1::from_raw(20), SpanV1::new(0, 12), scan, None);
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 12, items: vec![ProgramItemHirV1::Query(Box::new(query))],
    }).unwrap(), &authority).unwrap();
    let id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(20));
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document.item(id).unwrap() else { panic!("typed relation expected") };
    assert_eq!(relation.root().key().node_id, stage_node);
    assert_eq!(relation.root().location().location(), LocatedSpanV2 { source_id, span: SpanV1::new(5, 12) });
    assert_eq!(relation.location().location(), LocatedSpanV2 { source_id, span: SpanV1::new(0, 12) });
    assert_eq!(relation.output().columns().len(), 1);
    assert_eq!(relation.output().columns()[0].name(), "id");
    assert_eq!(relation.output().columns()[0].location().location(), LocatedSpanV2 { source_id: SourceIdV2::from_raw(1), span: SpanV1::new(4, 5) });
    let quanta_gqlang_v2_sema::OutputColumnOriginV2::SourceField { source, scan_stage, field_name } = relation.output().columns()[0].origin() else { panic!("scan column must retain authoritative source field") };
    assert_eq!(source, relation.root().source().unwrap());
    assert_eq!(*scan_stage, relation.root().key());
    assert_eq!(field_name, "id");
    assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
    assert_eq!(relation.output().provenance().iter().count(), 1);
}

#[test]
fn query_filter_retains_sealed_predicate_and_inherited_scan_columns() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", None, SpanV1::new(5, 12));
    let scan_id = scan.id;
    let predicate = builder.compare("=", builder.int("1", SpanV1::new(15, 16)),
        builder.int("1", SpanV1::new(19, 20)), SpanV1::new(15, 20));
    let predicate_id = predicate.id;
    let filter = builder.relation_filter(scan, predicate, SpanV1::new(5, 20));
    let query = builder.query(filter, None, SpanV1::new(0, 20));
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 20, items: vec![ProgramItemHirV1::Query(Box::new(query))],
    }).unwrap(), &authority).unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document.items().next().unwrap().1 else {
        panic!("typed filter relation expected");
    };
    assert!(matches!(relation.root(), quanta_gqlang_v2_sema::TypedRelationExprV2::Filter(_)));
    assert_eq!(relation.root().inputs().next().unwrap().key().node_id, scan_id);
    assert_eq!(relation.root().predicate().unwrap().expression().key().node_id, predicate_id);
    assert_eq!(relation.root().location().location().span, SpanV1::new(5, 20));
    assert_eq!(relation.output().columns()[0].origin(), relation.root().inputs().next().unwrap().output().columns()[0].origin());
    assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
    assert_eq!(document.expressions().count(), 3);
}

#[test]
fn query_filter_alias_field_type_comes_from_exact_host_row_and_binder_owner() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", Some(AliasV1 { name: "r".into(), span: SpanV1::new(13, 14) }), SpanV1::new(5, 14));
    let scan_id = scan.id;
    let symbol = builder.symbol("r", SpanV1::new(17, 18));
    let symbol_id = symbol.id;
    let path = builder.try_path(symbol, vec![PathSegmentV1 { name: "id".into(), span: SpanV1::new(19, 21) }], SpanV1::new(17, 21)).unwrap();
    let predicate = builder.compare("=", path, builder.int("1", SpanV1::new(24, 25)), SpanV1::new(17, 25));
    let filter = builder.relation_filter(scan, predicate, SpanV1::new(5, 25));
    let query = builder.query(filter, None, SpanV1::new(0, 25));
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 25, items: vec![ProgramItemHirV1::Query(Box::new(query))],
    }).unwrap(), &authority).unwrap();
    let expression = document.expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 { source_id, node_id: symbol_id }).unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = expression.shape() else { panic!("exact row alias binding expected") };
    assert_eq!((binding.target_node(), binding.ordinal(), binding.canonical_name()), (scan_id, 0, "r"));
    let quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(ty) = expression.type_state() else { panic!("host row type required") };
    let row = ty.as_record().unwrap();
    assert_eq!(row.fields().len(), 1);
    assert_eq!(row.fields()[0].name(), "id");
    assert_eq!(row.fields()[0].value_type(), &ValueTypeV1::int());
}

#[test]
fn query_projection_alias_is_a_prior_typed_expression_not_a_host_field_guess() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", Some(AliasV1 { name: "r".into(), span: SpanV1::new(13, 14) }), SpanV1::new(5, 14));
    let path = builder.try_path(builder.symbol("r", SpanV1::new(22, 23)),
        vec![PathSegmentV1 { name: "id".into(), span: SpanV1::new(24, 26) }], SpanV1::new(22, 26)).unwrap();
    let path_id = path.id;
    let project = builder.try_project(scan, vec![ProjectionItemV1 {
        value: path, alias: Some(AliasV1 { name: "score".into(), span: SpanV1::new(30, 35) }),
    }], SpanV1::new(5, 35)).unwrap();
    let project_id = project.id;
    let output_ref = builder.symbol("score", SpanV1::new(44, 49));
    let output_ref_id = output_ref.id;
    let predicate = builder.compare("=", output_ref, builder.int("1", SpanV1::new(52, 53)), SpanV1::new(44, 53));
    let filter = builder.relation_filter(project, predicate, SpanV1::new(5, 53));
    let query = builder.query(filter, None, SpanV1::new(0, 53));
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 53, items: vec![ProgramItemHirV1::Query(Box::new(query))],
    }).unwrap(), &authority).unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document.items().next().unwrap().1 else { panic!("typed query expected") };
    let project = relation.root().inputs().next().unwrap();
    assert!(matches!(project, quanta_gqlang_v2_sema::TypedRelationExprV2::Project(_)));
    assert_eq!(project.expressions()[0].node_id, path_id);
    assert_eq!(project.output().columns()[0].name(), "score");
    assert_eq!(project.output().columns()[0].origin(), &quanta_gqlang_v2_sema::OutputColumnOriginV2::Expression(
        quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 { source_id, node_id: path_id }));
    assert_eq!(project.output().columns()[0].location().location().span, SpanV1::new(30, 35));
    let reference = document.expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 { source_id, node_id: output_ref_id }).unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = reference.shape() else { panic!("projection frame binding required") };
    assert_eq!((binding.target_node(), binding.ordinal(), binding.canonical_name()), (project_id, 0, "score"));
    assert_eq!(reference.type_state(), &quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(ValueTypeV1::int()));
}

#[test]
fn self_join_projected_columns_retain_distinct_scan_occurrences_and_checked_on() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let left_scan = builder.relation_source("records", Some(AliasV1 { name: "l".into(), span: SpanV1::new(13, 14) }), SpanV1::new(5, 14));
    let left_scan_id = left_scan.id;
    let left_value = builder.try_path(builder.symbol("l", SpanV1::new(20, 21)),
        vec![PathSegmentV1 { name: "id".into(), span: SpanV1::new(22, 24) }], SpanV1::new(20, 24)).unwrap();
    let left = builder.try_project(left_scan, vec![ProjectionItemV1 {
        value: left_value, alias: Some(AliasV1 { name: "left_id".into(), span: SpanV1::new(28, 35) }),
    }], SpanV1::new(5, 35)).unwrap();
    let right_scan = builder.relation_source("records", Some(AliasV1 { name: "r".into(), span: SpanV1::new(48, 49) }), SpanV1::new(40, 49));
    let right_scan_id = right_scan.id;
    let right_value = builder.try_path(builder.symbol("r", SpanV1::new(55, 56)),
        vec![PathSegmentV1 { name: "id".into(), span: SpanV1::new(57, 59) }], SpanV1::new(55, 59)).unwrap();
    let right = builder.try_project(right_scan, vec![ProjectionItemV1 {
        value: right_value, alias: Some(AliasV1 { name: "right_id".into(), span: SpanV1::new(63, 71) }),
    }], SpanV1::new(40, 71)).unwrap();
    let on = builder.compare("=", builder.symbol("left_id", SpanV1::new(75, 82)),
        builder.symbol("right_id", SpanV1::new(85, 93)), SpanV1::new(75, 93));
    let on_id = on.id;
    let join = builder.inner_join(left, right, on, SpanV1::new(5, 93));
    let query = builder.query(join, None, SpanV1::new(0, 93));
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 93, items: vec![ProgramItemHirV1::Query(Box::new(query))],
    }).unwrap(), &authority).unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document.items().next().unwrap().1 else { panic!("typed self-join required") };
    assert!(matches!(relation.root(), quanta_gqlang_v2_sema::TypedRelationExprV2::Join(_)));
    assert_eq!(relation.root().join_mode(), Some(quanta_gqlang_v2_sema::JoinModeV2::Inner));
    assert_eq!(relation.root().predicate().unwrap().expression().key().node_id, on_id);
    assert_eq!(relation.output().row().fields().iter().map(|field| field.name()).collect::<Vec<_>>(), vec!["left_id", "right_id"]);
    let sources = relation.root().inputs().map(|project| project.inputs().next().unwrap().key().node_id).collect::<Vec<_>>();
    assert_eq!(sources, vec![left_scan_id, right_scan_id]);
    assert_ne!(sources[0], sources[1]);
    assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
}

#[test]
fn ordered_projection_alias_and_null_key_retain_direction_null_policy_and_sequence() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", None, SpanV1::new(5, 12));
    let project = builder.try_project(scan, vec![ProjectionItemV1 {
        value: builder.int("3", SpanV1::new(20, 21)),
        alias: Some(AliasV1 { name: "score".into(), span: SpanV1::new(24, 29) }),
    }], SpanV1::new(5, 29)).unwrap();
    let asc = builder.symbol("score", SpanV1::new(39, 44));
    let asc_id = asc.id;
    let desc = builder.null(SpanV1::new(51, 55));
    let desc_id = desc.id;
    let order = builder.order_by(project, vec![
        OrderKeyHirV1 { value: asc, direction: SortDirectionHirV1::Asc },
        OrderKeyHirV1 { value: desc, direction: SortDirectionHirV1::Desc },
    ], SpanV1::new(5, 55)).unwrap();
    let query = builder.query(order, None, SpanV1::new(0, 55));
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 55, items: vec![ProgramItemHirV1::Query(Box::new(query))],
    }).unwrap(), &authority).unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document.items().next().unwrap().1 else { panic!("typed sort required") };
    assert!(matches!(relation.root(), quanta_gqlang_v2_sema::TypedRelationExprV2::Sort(_)));
    assert_eq!(relation.output().cardinality(), CardinalityV1::Sequence);
    assert_eq!(relation.root().expressions().iter().map(|key| key.node_id).collect::<Vec<_>>(), vec![asc_id, desc_id]);
    assert_eq!(relation.output().ordering().iter().map(|key| (key.direction(), key.null_order())).collect::<Vec<_>>(), vec![
        (SortDirectionHirV1::Asc, quanta_gqlang_v2_sema::NullOrderV2::Last),
        (SortDirectionHirV1::Desc, quanta_gqlang_v2_sema::NullOrderV2::Last),
    ]);
    assert_eq!(relation.output().columns()[0].name(), "score");
}

#[test]
fn grouped_key_only_row_is_visible_to_sort_by_binder_owned_ordinal() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", Some(AliasV1 { name: "r".into(), span: SpanV1::new(13, 14) }), SpanV1::new(5, 14));
    let key = builder.try_path(builder.symbol("r", SpanV1::new(20, 21)),
        vec![PathSegmentV1 { name: "id".into(), span: SpanV1::new(22, 24) }], SpanV1::new(20, 24)).unwrap();
    let key_id = key.id;
    let group = builder.group_by(scan, vec![key], SpanV1::new(5, 30)).unwrap();
    let group_id = group.id;
    let order_ref = builder.symbol("id", SpanV1::new(39, 41));
    let order_ref_id = order_ref.id;
    let order = builder.order_by(group, vec![OrderKeyHirV1 { value: order_ref, direction: SortDirectionHirV1::Asc }],
        SpanV1::new(5, 41)).unwrap();
    let query = builder.query(order, None, SpanV1::new(0, 41));
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 41, items: vec![ProgramItemHirV1::Query(Box::new(query))],
    }).unwrap(), &authority).unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document.items().next().unwrap().1 else { panic!("typed group and sort required") };
    let group = relation.root().inputs().next().unwrap();
    assert!(matches!(group, quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_)));
    assert_eq!(group.output().cardinality(), CardinalityV1::Set);
    assert_eq!(group.output().row().fields().iter().map(|field| field.name()).collect::<Vec<_>>(), vec!["id"]);
    assert_eq!(group.output().partition_keys()[0].node_id, key_id);
    assert_eq!(relation.output().cardinality(), CardinalityV1::Sequence);
    let reference = document.expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 { source_id, node_id: order_ref_id }).unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = reference.shape() else { panic!("grouped key binding expected") };
    assert_eq!((binding.target_node(), binding.ordinal(), binding.canonical_name()), (group_id, 0, "id"));
}

#[test]
fn source_alias_disappears_from_public_group_output_frame() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", Some(AliasV1 { name: "r".into(), span: SpanV1::new(13, 14) }), SpanV1::new(5, 14));
    let key = builder.try_path(builder.symbol("r", SpanV1::new(20, 21)),
        vec![PathSegmentV1 { name: "id".into(), span: SpanV1::new(22, 24) }], SpanV1::new(20, 24)).unwrap();
    let group = builder.group_by(scan, vec![key], SpanV1::new(5, 30)).unwrap();
    let unavailable = builder.symbol("r", SpanV1::new(39, 40));
    let order = builder.order_by(group, vec![OrderKeyHirV1 { value: unavailable, direction: SortDirectionHirV1::Asc }],
        SpanV1::new(5, 40)).unwrap();
    let query = builder.query(order, None, SpanV1::new(0, 40));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9), source_len: 40,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        }).unwrap(), &authority,
    ).unwrap_err() else { panic!("pre-group alias cannot escape group frame") };
    assert_eq!(error.diagnostics()[0].primary().span, SpanV1::new(39, 40));
}

#[test]
fn identical_host_relation_scans_preserve_distinct_authored_occurrence_ids() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let first = builder.relation_source("records", Some(AliasV1 { name: "u".into(), span: SpanV1::new(13, 14) }), SpanV1::new(5, 14));
    let second = builder.relation_source("records", Some(AliasV1 { name: "v".into(), span: SpanV1::new(33, 34) }), SpanV1::new(25, 34));
    let document = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id, source_len: 34,
        items: vec![
            ProgramItemHirV1::Query(Box::new(quanta_gqlang_v2_hir::QueryHirV1::new(HirNodeIdV1::from_raw(20), SpanV1::new(0, 14), first, None))),
            ProgramItemHirV1::Query(Box::new(quanta_gqlang_v2_hir::QueryHirV1::new(HirNodeIdV1::from_raw(21), SpanV1::new(20, 34), second, None))),
        ],
    }).unwrap(), &authority).unwrap();
    let relation = |node| {
        let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document.item(SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(node))).unwrap() else {
            panic!("typed scan expected");
        };
        relation
    };
    let first = relation(20);
    let second = relation(21);
    assert_eq!(first.root().source(), second.root().source());
    assert_ne!(first.root().key(), second.root().key());
    assert_ne!(first.output().columns()[0].origin(), second.output().columns()[0].origin());
}

#[test]
fn diagnostics_sort_stage_severity_code_and_secondary_edges() {
    use quanta_gqlang_v2_sema::{FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2};
    let source = SourceIdV2::from_raw(9);
    let a = LocatedSpanV2 { source_id: source, span: SpanV1::new(3, 4) };
    let b = LocatedSpanV2 { source_id: source, span: SpanV1::new(8, 9) };
    let later = FrontendDiagnosticV2::new(FrontendStageV2::Typing, "z", FrontendSeverityV2::Error, b, vec![b, a, a], "Ω".repeat(600));
    assert_eq!(later.secondary(), &[a, b]);
    assert!(later.message().len() <= 1_024);
    assert!(later.message().is_char_boundary(later.message().len()));
    let first = FrontendDiagnosticV2::new(FrontendStageV2::Binding, "a", FrontendSeverityV2::Error, a, Vec::new(), "first");
    let failure = FrontendFailureV2::try_new(vec![later, first]).unwrap();
    assert_eq!(failure.diagnostics().iter().map(FrontendDiagnosticV2::code).collect::<Vec<_>>(), vec!["a", "z"]);
    assert!(FrontendFailureV2::try_new(Vec::new()).is_err());
}
