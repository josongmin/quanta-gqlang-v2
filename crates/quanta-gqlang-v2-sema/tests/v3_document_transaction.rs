#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "owner assertions inspect exact transaction diagnostics and semantic variants"
)]

use quanta_gqlang_v2_hir::{
    ActivationV1, AdmissionV1, AliasV1, CallArgHirV1, CallArgsHirV1, CatalogDeclHirV1,
    CatalogPayloadV1, ConfirmModeV1, DeclHirV1, DetectionModeV1, EmitModeV1, ExportFormatV1,
    GateBindingV1, GraphConstraintsV1, GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1,
    GraphRefV1, HirBuilderV1, HirDocumentV1, HirNodeIdV1, LegalityHirV1, LiteralV1, LocatedSpanV2,
    MetaHirV1, MetaKindV1, OrderKeyHirV1, PathSegmentV1, PresentationHirV1, PresentationKindV1,
    ProgramItemHirV1, ProjectionItemV1, RecordFieldV1, RelationExistsPolarityV1, ShowCategoryV1,
    SortDirectionHirV1, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
    SymbolRefV1, ValueExprHirV1, ValueExprKindV1, ViewBodyV1,
};
use quanta_gqlang_v2_modules::{
    CallableKindV2, CurrentModuleSelectionInputV2, ExternalCatalogAssetInputV2,
    ExternalCatalogPayloadInputV2, LocatedParameterInputV2, ModuleAuthorityBuilderV2, ModuleIdV2,
    ModuleNodeInputV2, SignatureInputV2, VersionV2,
};
use quanta_gqlang_v2_sema::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    DeclaredDocumentV2, FrontendBudgetV2, FrontendTransactionErrorV2, GraphReferenceCatalogV2,
    GraphReferenceInputV2, GraphReferenceKindV2, SemanticItemIdV2,
};
use quanta_gqlang_v2_types::{
    CardinalityV1, ExpressionEffectsV2, ProvenanceNamespaceV2, RecordFieldTypeV1, RecordTypeV1,
    RelationCatalogV2, RelationSourceInputV2, RelationTypeV1, ValueTypeV1,
};

fn host() -> (SourceRegistryViewV2, ClosedFrontendAuthorityV2) {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 8,
    }])
    .unwrap();
    let location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(0, 1),
        })
        .unwrap();
    let module = ModuleIdV2::try_new("app", location.clone()).unwrap();
    let budget = FrontendBudgetV2::standard_v2().unwrap();
    let profile = budget.authority_profile().unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: module.clone(),
            location: location.clone(),
        })
        .unwrap();
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "app".into(),
            local_name: "check1".into(),
            kind_token: "check".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Check,
            provenance: "host".into(),
            location: host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span: SpanV1::new(3, 4),
                })
                .unwrap(),
            dependencies: Vec::new(),
        })
        .unwrap();
    for (kind_token, local_name, typed_payload) in [
        ("query", "query1", ExternalCatalogPayloadInputV2::Query),
        (
            "model_source",
            "model_source1",
            ExternalCatalogPayloadInputV2::ModelSource,
        ),
        (
            "pack_source",
            "pack_source1",
            ExternalCatalogPayloadInputV2::PackSource,
        ),
        (
            "capability",
            "capability1",
            ExternalCatalogPayloadInputV2::Capability,
        ),
    ] {
        builder
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: "app".into(),
                local_name: local_name.into(),
                kind_token: kind_token.into(),
                typed_payload,
                provenance: "host".into(),
                location: host
                    .try_check_location(LocatedSpanV2 {
                        source_id: SourceIdV2::from_raw(1),
                        span: SpanV1::new(3, 4),
                    })
                    .unwrap(),
                dependencies: Vec::new(),
            })
            .unwrap();
    }
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "app".into(),
            local_name: "host_view".into(),
            kind_token: "view".into(),
            typed_payload: ExternalCatalogPayloadInputV2::View(
                RelationTypeV1::try_new(
                    RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
                        "id",
                        ValueTypeV1::int(),
                        location.clone(),
                    )
                    .unwrap()])
                    .unwrap(),
                    CardinalityV1::Bag,
                )
                .unwrap(),
            ),
            provenance: "host".into(),
            location: location.clone(),
            dependencies: Vec::new(),
        })
        .unwrap();
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "app".into(),
            local_name: "host_pred".into(),
            kind_token: "predicate".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Predicate(SignatureInputV2 {
                parameters: vec![LocatedParameterInputV2 {
                    name: "value".into(),
                    value_type: ValueTypeV1::int(),
                    location: location.clone(),
                }],
                result: ValueTypeV1::bool(),
                callable_kind: CallableKindV2::Predicate,
                effects: ExpressionEffectsV2::pure_scalar(),
                location: location.clone(),
            }),
            provenance: "host".into(),
            location: location.clone(),
            dependencies: Vec::new(),
        })
        .unwrap();
    let modules = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module,
                location,
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let relation_location = host
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(4, 5),
        })
        .unwrap();
    let relations = RelationCatalogV2::try_new(
        vec![
            RelationSourceInputV2 {
                authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
                provenance_namespace: ProvenanceNamespaceV2::try_new(
                    "host",
                    relation_location.clone(),
                )
                .unwrap(),
                qualified_name: "records".into(),
                lookup_name: "records".into(),
                row: RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
                    "id",
                    ValueTypeV1::int(),
                    relation_location.clone(),
                )
                .unwrap()])
                .unwrap(),
                cardinality: CardinalityV1::Bag,
                provenance: "host".into(),
                location: relation_location.clone(),
            },
            RelationSourceInputV2 {
                authority: quanta_gqlang_v2_types::RelationAuthorityV2::Host,
                provenance_namespace: ProvenanceNamespaceV2::try_new(
                    "host",
                    relation_location.clone(),
                )
                .unwrap(),
                qualified_name: "records2".into(),
                lookup_name: "records2".into(),
                row: RecordTypeV1::try_new(vec![
                    RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), relation_location.clone())
                        .unwrap(),
                    RecordFieldTypeV1::try_new(
                        "value",
                        ValueTypeV1::int(),
                        relation_location.clone(),
                    )
                    .unwrap(),
                ])
                .unwrap(),
                cardinality: CardinalityV1::Bag,
                provenance: "host".into(),
                location: relation_location,
            },
        ],
        &host,
        profile,
    )
    .unwrap();
    let graph = GraphReferenceCatalogV2::try_new(
        [
            ("repo::src", SpanV1::new(1, 2), GraphReferenceKindV2::Entity),
            (
                "repo::sink",
                SpanV1::new(2, 3),
                GraphReferenceKindV2::Entity,
            ),
            (
                "result.latest",
                SpanV1::new(5, 6),
                GraphReferenceKindV2::ExportSource,
            ),
        ]
        .into_iter()
        .map(|(spelling, span, kind)| GraphReferenceInputV2 {
            spelling: spelling.into(),
            kind,
            entity_subtype: None,
            location: host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span,
                })
                .unwrap(),
        })
        .collect(),
        &host,
        budget.authority_profile().unwrap(),
    )
    .unwrap();
    (
        host,
        ClosedFrontendAuthorityV2::try_new(modules, relations, graph, budget).unwrap(),
    )
}

fn meta_document(source: u32, node: u32) -> HirDocumentV1 {
    HirDocumentV1 {
        source_id: SourceIdV2::from_raw(source),
        source_len: 17,
        items: vec![ProgramItemHirV1::Meta(Box::new(MetaHirV1::new(
            HirNodeIdV1::from_raw(node),
            SpanV1::new(0, 17),
            MetaKindV1::RequireCertified,
        )))],
    }
}

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
fn source_rule_dependency_polarity_uses_exact_ancestor_overload_not_composed_sibling_effects() {
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
        let recursive_id = recursive.id;
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
        let document = resolve_checked_document_v2(
            CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
                source_id: SourceIdV2::from_raw(9),
                source_len: 50,
                items: vec![ProgramItemHirV1::Rule(Box::new(rule))],
            })
            .unwrap(),
            &authority,
        )
        .unwrap();
        let calls = document.rules().next().unwrap().calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls.first().unwrap().call().node_id, recursive_id);
        assert_eq!(
            calls.first().unwrap().polarity(),
            if aggregated {
                quanta_gqlang_v2_sema::RuleCallPolarityV2::Aggregated
            } else {
                quanta_gqlang_v2_sema::RuleCallPolarityV2::Positive
            }
        );
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

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn graph_references_keep_source_roles_and_checked_use_locations() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let graph = GraphQueryHirV1::new(
        HirNodeIdV1::from_raw(2),
        SpanV1::new(0, 38),
        GraphQueryKindV1::Reachability,
        GraphEndpointsV1 {
            source: Some(GraphRefV1 {
                name: "repo::src".into(),
                span: SpanV1::new(10, 19),
            }),
            sink: Some(GraphRefV1 {
                name: "repo::sink".into(),
                span: SpanV1::new(25, 35),
            }),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1::default(),
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 38,
            items: vec![ProgramItemHirV1::GraphQuery(Box::new(graph))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(2));
    let quanta_gqlang_v2_sema::SemanticItemV2::Graph(graph) = document.item(id).unwrap() else {
        panic!("typed FIND expected")
    };
    let references = graph.references();
    assert_eq!(
        graph.location().location(),
        LocatedSpanV2 {
            source_id,
            span: SpanV1::new(0, 38)
        }
    );
    assert_eq!(
        graph.witness_bounds().max_depth(),
        authority.budget().max_witness_depth()
    );
    assert_eq!(
        graph.witness_bounds().max_witnesses(),
        authority.budget().max_witnesses()
    );
    assert_eq!(references.len(), 2);
    assert_eq!(
        references.first().unwrap().role(),
        quanta_gqlang_v2_sema::GraphReferenceRoleV2::Source
    );
    assert_eq!(
        references
            .first()
            .and_then(|reference| reference.target().host_selector())
            .map(quanta_gqlang_v2_sema::GraphReferenceIdV2::spelling),
        Some("repo::src")
    );
    assert_eq!(
        references.first().unwrap().use_location().location(),
        LocatedSpanV2 {
            source_id,
            span: SpanV1::new(10, 19)
        }
    );
    assert!(authority.graph_references().admits(
        references
            .get(1)
            .and_then(|reference| reference.target().host_selector())
            .expect("fixture reference remains host-owned"),
    ));
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn graph_depth_over_frontend_witness_budget_fails_at_query_span() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let graph = GraphQueryHirV1::new(
        HirNodeIdV1::from_raw(2),
        SpanV1::new(0, 8),
        GraphQueryKindV1::Path,
        GraphEndpointsV1::default(),
        GraphConstraintsV1 {
            depth: Some(authority.budget().max_witness_depth() + 1),
            ..GraphConstraintsV1::default()
        },
    );
    let checked = CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id,
        source_len: 8,
        items: vec![ProgramItemHirV1::GraphQuery(Box::new(graph))],
    })
    .unwrap();
    let FrontendTransactionErrorV2::Document(error) =
        resolve_checked_document_v2(checked, &authority).unwrap_err()
    else {
        panic!("over-budget graph depth must be source anchored");
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "frontend.resource_limit_exceeded"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary(),
        LocatedSpanV2 {
            source_id,
            span: SpanV1::new(0, 8)
        }
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn presentation_show_is_a_closed_source_item_without_phantom_reference() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let show = PresentationHirV1::new(
        HirNodeIdV1::from_raw(8),
        SpanV1::new(0, 10),
        PresentationKindV1::Show {
            category: ShowCategoryV1::Models,
        },
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 10,
            items: vec![ProgramItemHirV1::Presentation(Box::new(show))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(8));
    let quanta_gqlang_v2_sema::SemanticItemV2::Presentation(item) = document.item(id).unwrap()
    else {
        panic!("SHOW must remain presentation")
    };
    assert!(matches!(
        item.kind(),
        quanta_gqlang_v2_sema::ResolvedPresentationKindV2::Show {
            category: ShowCategoryV1::Models
        }
    ));
}

fn export_profile_document(
    profile_name: Option<&str>,
    declare_profile: bool,
    profile_first: bool,
) -> CheckedSyntaxDocumentV2 {
    let export = ProgramItemHirV1::Presentation(Box::new(PresentationHirV1::new(
        HirNodeIdV1::from_raw(11),
        SpanV1::new(40, 100),
        PresentationKindV1::Export {
            source_ref: GraphRefV1 {
                name: "result.latest".into(),
                span: SpanV1::new(54, 67),
            },
            format: ExportFormatV1::Sarif,
            profile: profile_name.map(|name| GraphRefV1 {
                name: name.into(),
                span: SpanV1::new(88, 88 + name.len()),
            }),
        },
    )));
    let mut items = vec![export];
    if declare_profile {
        let profile = ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
            HirNodeIdV1::from_raw(10),
            SpanV1::new(0, 35),
            "strict",
            SpanV1::new(8, 14),
            CatalogPayloadV1::Profile {
                capabilities: vec![GraphRefV1 {
                    name: "capability1".into(),
                    span: SpanV1::new(16, 27),
                }],
                admission: AdmissionV1::Allowlist,
            },
        )));
        if profile_first {
            items.insert(0, profile);
        } else {
            items.push(profile);
        }
    }
    CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
        source_id: SourceIdV2::from_raw(9),
        source_len: 100,
        items,
    })
    .expect("EXPORT fixture has distinct IDs and bounded authored spans")
}

#[test]
fn export_profile_resolves_exact_local_identity_and_use_in_both_orders() {
    use quanta_gqlang_v2_modules::NamespaceKindV2;
    use quanta_gqlang_v2_sema::{ResolvedPresentationKindV2, SemanticItemV2};

    let (_, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    for profile_first in [true, false] {
        let document = resolve_checked_document_v2(
            export_profile_document(Some("strict"), true, profile_first),
            &authority,
        )
        .expect("EXPORT admits the authored profile, including forward references");
        assert_eq!(document.order().len(), 2);
        let SemanticItemV2::Catalog(declaration) = document
            .item(SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(10)))
            .expect("authored profile retained")
        else {
            panic!("expected Profile declaration")
        };
        let SemanticItemV2::Presentation(export) = document
            .item(SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(11)))
            .expect("authored EXPORT retained")
        else {
            panic!("expected EXPORT")
        };
        let ResolvedPresentationKindV2::Export {
            source,
            format,
            profile: Some(profile),
        } = export.kind()
        else {
            panic!("EXPORT must retain its authored profile")
        };
        assert_eq!(*format, ExportFormatV1::Sarif);
        assert_eq!(profile.id(), declaration.asset_id());
        assert_eq!(profile.id().module_id().as_str(), "app");
        assert_eq!(profile.id().local_name(), "strict");
        assert_eq!(profile.kind(), NamespaceKindV2::Profile);
        assert_eq!(
            profile.location().location(),
            LocatedSpanV2 {
                source_id,
                span: SpanV1::new(88, 94)
            }
        );
        assert_eq!(
            source.location().location(),
            LocatedSpanV2 {
                source_id,
                span: SpanV1::new(54, 67)
            }
        );
        assert_eq!(
            source
                .target()
                .host_selector()
                .expect("host export source")
                .spelling(),
            "result.latest"
        );
    }
    let no_profile =
        resolve_checked_document_v2(export_profile_document(None, false, false), &authority)
            .expect("profile remains optional");
    assert_eq!(no_profile.order().len(), 1);
    let (_, SemanticItemV2::Presentation(export)) = no_profile.items().next().expect("one EXPORT")
    else {
        panic!("expected EXPORT")
    };
    assert!(matches!(
        export.kind(),
        ResolvedPresentationKindV2::Export { profile: None, .. }
    ));
}

#[test]
fn export_profile_rejects_missing_and_wrong_kind_at_the_authored_use() {
    let (_, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    for (name, code, secondary) in [
        ("missing", "modules.unknown_catalog_reference", Vec::new()),
        (
            "capability1",
            "modules.catalog_reference_kind_mismatch",
            vec![LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(1),
                span: SpanV1::new(3, 4),
            }],
        ),
    ] {
        let FrontendTransactionErrorV2::Document(failure) = resolve_checked_document_v2(
            export_profile_document(Some(name), false, false),
            &authority,
        )
        .expect_err("missing or non-Profile catalog target must reject") else {
            panic!("expected document diagnostic")
        };
        let [diagnostic] = failure.diagnostics() else {
            panic!("expected one exact diagnostic")
        };
        assert_eq!(diagnostic.code(), code, "{name}");
        assert_eq!(
            diagnostic.primary(),
            LocatedSpanV2 {
                source_id,
                span: SpanV1::new(88, 88 + name.len())
            }
        );
        assert_eq!(diagnostic.secondary(), secondary);
    }
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn catalog_rule_binding_and_check_share_modules_validated_exact_reference() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let binding = CatalogDeclHirV1::new(
        HirNodeIdV1::from_raw(8),
        SpanV1::new(0, 30),
        "rule1",
        SpanV1::new(5, 10),
        CatalogPayloadV1::RuleBinding {
            check_ref: GraphRefV1 {
                name: "check1".into(),
                span: SpanV1::new(21, 27),
            },
        },
    );
    let check = LegalityHirV1::new(
        HirNodeIdV1::from_raw(9),
        SpanV1::new(31, 60),
        GraphRefV1 {
            name: "rule1".into(),
            span: SpanV1::new(40, 45),
        },
        None,
        None,
        ConfirmModeV1::Skip,
        EmitModeV1::Findings,
        DetectionModeV1::PatternOnly,
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 60,
            items: vec![
                ProgramItemHirV1::CatalogDecl(Box::new(binding)),
                ProgramItemHirV1::Legality(Box::new(check)),
            ],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let catalog_id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(8));
    let check_id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(9));
    let quanta_gqlang_v2_sema::SemanticItemV2::Catalog(rule) = document.item(catalog_id).unwrap()
    else {
        panic!("catalog Rule binding expected")
    };
    let quanta_gqlang_v2_sema::SemanticItemV2::Check(check) = document.item(check_id).unwrap()
    else {
        panic!("typed CHECK expected")
    };
    assert_eq!(check.rule().id(), rule.asset_id());
    assert_eq!(document.catalog().dependencies().len(), 1);
    let edge = document.catalog().dependencies().first().unwrap();
    assert_eq!(edge.source_asset_id(), rule.asset_id());
    assert_eq!(
        edge.family(),
        Some(quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::RuleBindingCheck)
    );
}

#[test]
fn catalog_reference_families_use_one_modules_predeclaration_and_validation_session() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let cases = [
        (
            "view1",
            CatalogPayloadV1::View(ViewBodyV1::QueryRef(GraphRefV1 {
                name: "query1".into(),
                span: SpanV1::new(11, 17),
            })),
        ),
        (
            "model1",
            CatalogPayloadV1::Model {
                provider: "opaque-provider".into(),
                source: GraphRefV1 {
                    name: "model_source1".into(),
                    span: SpanV1::new(31, 44),
                },
            },
        ),
        (
            "pack1",
            CatalogPayloadV1::Pack {
                source: GraphRefV1 {
                    name: "pack_source1".into(),
                    span: SpanV1::new(51, 63),
                },
                activation: ActivationV1::Manual,
            },
        ),
        (
            "profile1",
            CatalogPayloadV1::Profile {
                capabilities: vec![GraphRefV1 {
                    name: "capability1".into(),
                    span: SpanV1::new(71, 82),
                }],
                admission: AdmissionV1::Allowlist,
            },
        ),
        (
            "gate1",
            CatalogPayloadV1::Gate {
                checks: vec![GraphRefV1 {
                    name: "check1".into(),
                    span: SpanV1::new(91, 97),
                }],
                binding: GateBindingV1::All,
            },
        ),
    ];
    let items = cases
        .into_iter()
        .enumerate()
        .map(|(index, (name, payload))| {
            let start = index * 20;
            ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                HirNodeIdV1::from_raw(u32::try_from(index + 1).unwrap()),
                SpanV1::new(start, start + 25),
                name,
                SpanV1::new(start + 1, start + 5),
                payload,
            )))
        })
        .collect();
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 105,
            items,
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    assert_eq!(document.order().len(), 5);
    assert_eq!(document.catalog().dependencies().len(), 5);
    assert!(document
        .items()
        .all(|(_, item)| matches!(item, quanta_gqlang_v2_sema::SemanticItemV2::Catalog(_))));
    let families = document
        .catalog()
        .dependencies()
        .iter()
        .map(|edge| edge.family().unwrap())
        .collect::<Vec<_>>();
    assert!(
        families.contains(&quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::ViewQuery)
    );
    assert!(
        families.contains(&quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::ModelSource)
    );
    assert!(
        families.contains(&quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::PackSource)
    );
    assert!(families
        .contains(&quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::ProfileCapability));
    assert!(
        families.contains(&quanta_gqlang_v2_modules::DocumentCatalogReferenceFamilyV2::GateCheck)
    );
}

#[test]
fn inline_view_materializes_through_the_canonical_relation_path() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let inline = builder.query(
        builder.relation_source("records", SpanV1::new(15, 22), None, SpanV1::new(15, 22)),
        None,
        SpanV1::new(8, 22),
    );
    let inline_id = inline.id;
    let outer = builder.query(
        builder.relation_source("view1", SpanV1::new(31, 36), None, SpanV1::new(31, 36)),
        None,
        SpanV1::new(24, 36),
    );
    let outer_id = outer.id;
    let view_id = HirNodeIdV1::from_raw(100);
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 64,
            items: vec![
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    view_id,
                    SpanV1::new(0, 22),
                    "view1",
                    SpanV1::new(3, 8),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(inline))),
                ))),
                ProgramItemHirV1::Query(Box::new(outer)),
            ],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let catalog_id = SemanticItemIdV2::new(source_id, view_id);
    let outer_id = SemanticItemIdV2::new(source_id, outer_id);
    let quanta_gqlang_v2_sema::SemanticItemV2::Catalog(view) = document.item(catalog_id).unwrap()
    else {
        panic!("inline catalog View expected")
    };
    let typed_inline = view.inline_view().expect("inline relation must be sealed");
    assert_eq!(
        typed_inline.item_id(),
        SemanticItemIdV2::new(source_id, inline_id)
    );
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(outer) = document.item(outer_id).unwrap()
    else {
        panic!("outer relation expected")
    };
    assert!(matches!(
        outer.root().source(),
        Some(quanta_gqlang_v2_sema::ResolvedRelationSourceKindV2::View(id)) if id == view.asset_id()
    ));
    assert!(document.catalog().dependencies().is_empty());
}

#[test]
fn inline_views_use_modules_dependency_order_not_authored_order_and_reject_cycles() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let derived = builder.query(
        builder.relation_source("base", SpanV1::new(15, 19), None, SpanV1::new(15, 19)),
        None,
        SpanV1::new(8, 19),
    );
    let base = builder.query(
        builder.relation_source("records", SpanV1::new(38, 45), None, SpanV1::new(38, 45)),
        None,
        SpanV1::new(31, 45),
    );
    let derived_id = HirNodeIdV1::from_raw(101);
    let base_id = HirNodeIdV1::from_raw(102);
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 80,
            items: vec![
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    derived_id,
                    SpanV1::new(0, 19),
                    "derived",
                    SpanV1::new(3, 10),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(derived))),
                ))),
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    base_id,
                    SpanV1::new(23, 45),
                    "base",
                    SpanV1::new(26, 30),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(base))),
                ))),
            ],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let derived = document
        .item(SemanticItemIdV2::new(source_id, derived_id))
        .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Catalog(derived) = derived else {
        panic!("derived catalog View expected")
    };
    let base = document
        .item(SemanticItemIdV2::new(source_id, base_id))
        .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Catalog(base) = base else {
        panic!("base catalog View expected")
    };
    assert!(matches!(
        derived.inline_view().unwrap().root().source(),
        Some(quanta_gqlang_v2_sema::ResolvedRelationSourceKindV2::View(id)) if id == base.asset_id()
    ));
    assert_eq!(document.catalog().dependencies().len(), 1);
    assert_eq!(
        document.catalog().dependencies().first().unwrap().family(),
        None
    );

    let left = builder.query(
        builder.relation_source("right", SpanV1::new(12, 17), None, SpanV1::new(12, 17)),
        None,
        SpanV1::new(6, 17),
    );
    let right = builder.query(
        builder.relation_source("left", SpanV1::new(31, 35), None, SpanV1::new(31, 35)),
        None,
        SpanV1::new(25, 35),
    );
    let cycle = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 64,
            items: vec![
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    HirNodeIdV1::from_raw(103),
                    SpanV1::new(0, 17),
                    "left",
                    SpanV1::new(1, 5),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(left))),
                ))),
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    HirNodeIdV1::from_raw(104),
                    SpanV1::new(20, 35),
                    "right",
                    SpanV1::new(21, 26),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(right))),
                ))),
            ],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err();
    let FrontendTransactionErrorV2::Document(cycle) = cycle else {
        panic!("cycle must be a source-anchored document failure")
    };
    assert_eq!(
        cycle.diagnostics().first().unwrap().code(),
        "modules.view_inline_cycle"
    );
}

#[test]
fn inline_view_repeated_direct_uses_canonicalize_and_query_ref_never_falls_back() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let base_query = builder.query(
        builder.relation_source("records", SpanV1::new(15, 22), None, SpanV1::new(15, 22)),
        None,
        SpanV1::new(8, 22),
    );
    let outer = builder.relation_source("base", SpanV1::new(38, 42), None, SpanV1::new(38, 42));
    let inner = builder.relation_source("base", SpanV1::new(51, 55), None, SpanV1::new(51, 55));
    let exists =
        builder.relation_exists(RelationExistsPolarityV1::Exists, inner, SpanV1::new(44, 55));
    let derived_query = builder.query(
        builder.relation_filter(outer, exists, SpanV1::new(38, 55)),
        None,
        SpanV1::new(31, 55),
    );
    let base_id = HirNodeIdV1::from_raw(105);
    let derived_id = HirNodeIdV1::from_raw(106);
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 96,
            items: vec![
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    base_id,
                    SpanV1::new(0, 22),
                    "base",
                    SpanV1::new(3, 7),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(base_query))),
                ))),
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    derived_id,
                    SpanV1::new(24, 55),
                    "derived",
                    SpanV1::new(27, 34),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(derived_query))),
                ))),
            ],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Catalog(derived) = document
        .item(SemanticItemIdV2::new(source_id, derived_id))
        .unwrap()
    else {
        panic!("derived inline View expected")
    };
    assert_eq!(derived.inline_view_dependencies().len(), 1);
    assert_eq!(document.catalog().dependencies().len(), 1);
    assert_eq!(
        document.catalog().dependencies().first().unwrap().family(),
        None
    );

    let query_ref_id = HirNodeIdV1::from_raw(107);
    let query_ref = CatalogDeclHirV1::new(
        query_ref_id,
        SpanV1::new(0, 20),
        "query_ref_view",
        SpanV1::new(3, 17),
        CatalogPayloadV1::View(ViewBodyV1::QueryRef(GraphRefV1 {
            name: "query1".into(),
            span: SpanV1::new(18, 19),
        })),
    );
    let consumer = builder.query(
        builder.relation_source(
            "query_ref_view",
            SpanV1::new(31, 45),
            None,
            SpanV1::new(31, 45),
        ),
        None,
        SpanV1::new(22, 45),
    );
    let error = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 64,
            items: vec![
                ProgramItemHirV1::CatalogDecl(Box::new(query_ref)),
                ProgramItemHirV1::Query(Box::new(consumer)),
            ],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err();
    let FrontendTransactionErrorV2::Document(error) = error else {
        panic!("unmaterialized local View must be a document failure")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.unmaterialized_view_source"
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "external typed View source must preserve its exact Modules asset identity"
)]
fn external_typed_view_materializes_as_a_closed_relation_source() {
    let (_host, authority) = host();
    let expected_asset = authority
        .modules()
        .external_catalog()
        .iter()
        .find(|asset| asset.id().local_name() == "host_view")
        .unwrap()
        .id()
        .clone();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "host_view",
        SpanV1::new(5, 14),
        Some(AliasV1 {
            name: "v".into(),
            span: SpanV1::new(15, 16),
        }),
        SpanV1::new(5, 16),
    );
    let scan_id = scan.id;
    let projection = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: builder
                    .try_path(
                        builder.symbol("v", SpanV1::new(27, 28)),
                        vec![PathSegmentV1 {
                            name: "id".into(),
                            span: SpanV1::new(29, 31),
                        }],
                        SpanV1::new(27, 31),
                    )
                    .unwrap(),
                alias: Some(AliasV1 {
                    name: "id".into(),
                    span: SpanV1::new(35, 37),
                }),
            }],
            SpanV1::new(5, 37),
        )
        .unwrap();
    let query = builder.query(projection, None, SpanV1::new(0, 37));
    let query_id = query.id;
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 48,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(query) = document
        .item(SemanticItemIdV2::new(source_id, query_id))
        .unwrap()
    else {
        panic!("typed external View query expected")
    };
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Project(_) = query.root() else {
        panic!("external View projection expected")
    };
    let input = query.root().inputs().next().unwrap();
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Scan(_) = input else {
        panic!("external View scan expected")
    };
    assert_eq!(input.key().node_id, scan_id);
    assert!(matches!(
        input.source().unwrap(),
        quanta_gqlang_v2_sema::ResolvedRelationSourceKindV2::View(id) if id == &expected_asset
    ));
    assert_eq!(input.output().row().fields().first().unwrap().name(), "id");
    assert_eq!(input.output().cardinality(), CardinalityV1::Bag);
}

#[test]
#[expect(
    clippy::panic,
    reason = "the owner oracle checks exact View asset identities across a topology diamond"
)]
fn inline_view_diamond_materializes_dependency_first_despite_reverse_authored_order() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();

    let base = builder.query(
        builder.relation_source(
            "records",
            SpanV1::new(170, 177),
            None,
            SpanV1::new(170, 177),
        ),
        None,
        SpanV1::new(165, 177),
    );
    let left = builder.query(
        builder.relation_source("base", SpanV1::new(115, 119), None, SpanV1::new(115, 119)),
        None,
        SpanV1::new(110, 119),
    );
    let right = builder.query(
        builder.relation_source("base", SpanV1::new(145, 149), None, SpanV1::new(145, 149)),
        None,
        SpanV1::new(140, 149),
    );
    let outer = builder.relation_source("left", SpanV1::new(25, 29), None, SpanV1::new(25, 29));
    let inner = builder.relation_source("right", SpanV1::new(48, 53), None, SpanV1::new(48, 53));
    let top = builder.query(
        builder.relation_filter(
            outer,
            builder.relation_exists(RelationExistsPolarityV1::Exists, inner, SpanV1::new(40, 53)),
            SpanV1::new(25, 53),
        ),
        None,
        SpanV1::new(20, 53),
    );
    let top_id = HirNodeIdV1::from_raw(201);
    let right_id = HirNodeIdV1::from_raw(202);
    let left_id = HirNodeIdV1::from_raw(203);
    let base_id = HirNodeIdV1::from_raw(204);
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 192,
            items: vec![
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    top_id,
                    SpanV1::new(0, 53),
                    "top",
                    SpanV1::new(1, 4),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(top))),
                ))),
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    right_id,
                    SpanV1::new(130, 149),
                    "right",
                    SpanV1::new(131, 136),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(right))),
                ))),
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    left_id,
                    SpanV1::new(100, 119),
                    "left",
                    SpanV1::new(101, 105),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(left))),
                ))),
                ProgramItemHirV1::CatalogDecl(Box::new(CatalogDeclHirV1::new(
                    base_id,
                    SpanV1::new(160, 177),
                    "base",
                    SpanV1::new(161, 165),
                    CatalogPayloadV1::View(ViewBodyV1::Inline(Box::new(base))),
                ))),
            ],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let catalog = |id| match document.item(SemanticItemIdV2::new(source_id, id)).unwrap() {
        quanta_gqlang_v2_sema::SemanticItemV2::Catalog(view) => view,
        quanta_gqlang_v2_sema::SemanticItemV2::Relation(_)
        | quanta_gqlang_v2_sema::SemanticItemV2::Rule(_)
        | quanta_gqlang_v2_sema::SemanticItemV2::Graph(_)
        | quanta_gqlang_v2_sema::SemanticItemV2::Check(_)
        | quanta_gqlang_v2_sema::SemanticItemV2::Presentation(_)
        | quanta_gqlang_v2_sema::SemanticItemV2::Declaration(_)
        | quanta_gqlang_v2_sema::SemanticItemV2::Composition(_)
        | quanta_gqlang_v2_sema::SemanticItemV2::Meta(_) => {
            panic!("inline View declaration must materialize as Catalog")
        }
    };
    let top = catalog(top_id);
    let right = catalog(right_id);
    let left = catalog(left_id);
    let base = catalog(base_id);
    assert_eq!(top.inline_view_dependencies().len(), 2);
    assert_eq!(left.inline_view_dependencies().len(), 1);
    assert_eq!(right.inline_view_dependencies().len(), 1);
    assert!(matches!(
        left.inline_view().unwrap().root().source(),
        Some(quanta_gqlang_v2_sema::ResolvedRelationSourceKindV2::View(id)) if id == base.asset_id()
    ));
    assert!(matches!(
        right.inline_view().unwrap().root().source(),
        Some(quanta_gqlang_v2_sema::ResolvedRelationSourceKindV2::View(id)) if id == base.asset_id()
    ));
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Filter(_) = top.inline_view().unwrap().root()
    else {
        panic!("top View must retain its filtered relation body")
    };
    let top_scan = top.inline_view().unwrap().root().inputs().next().unwrap();
    assert!(matches!(
        top_scan.source(),
        Some(quanta_gqlang_v2_sema::ResolvedRelationSourceKindV2::View(id)) if id == left.asset_id()
    ));
    assert_eq!(document.catalog().dependencies().len(), 4);
    assert!(document
        .catalog()
        .dependencies()
        .iter()
        .all(|edge| edge.family().is_none()));
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn query_scan_uses_binder_frame_and_types_owned_host_relation_contract() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let stage_node = scan.id;
    let query = quanta_gqlang_v2_hir::QueryHirV1::new(
        HirNodeIdV1::from_raw(20),
        SpanV1::new(0, 12),
        scan,
        None,
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 12,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(20));
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document.item(id).unwrap()
    else {
        panic!("typed relation expected")
    };
    assert_eq!(relation.root().key().node_id, stage_node);
    assert_eq!(
        relation.root().location().location(),
        LocatedSpanV2 {
            source_id,
            span: SpanV1::new(5, 12)
        }
    );
    assert_eq!(
        relation.location().location(),
        LocatedSpanV2 {
            source_id,
            span: SpanV1::new(0, 12)
        }
    );
    assert_eq!(relation.output().columns().len(), 1);
    assert_eq!(relation.output().columns().first().unwrap().name(), "id");
    assert_eq!(
        relation
            .output()
            .columns()
            .first()
            .unwrap()
            .location()
            .location(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(4, 5)
        }
    );
    let quanta_gqlang_v2_sema::OutputColumnOriginV2::SourceField {
        source,
        scan_stage,
        field_name,
    } = relation.output().columns().first().unwrap().origin()
    else {
        panic!("scan column must retain authoritative source field")
    };
    assert_eq!(source, relation.root().source().unwrap());
    assert_eq!(*scan_stage, relation.root().key());
    assert_eq!(field_name, "id");
    assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
    assert_eq!(relation.output().provenance().iter().count(), 1);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn query_filter_retains_sealed_predicate_and_inherited_scan_columns() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let scan_id = scan.id;
    let predicate = builder.compare(
        "=",
        builder.int("1", SpanV1::new(15, 16)),
        builder.int("1", SpanV1::new(19, 20)),
        SpanV1::new(15, 20),
    );
    let predicate_id = predicate.id;
    let filter = builder.relation_filter(scan, predicate, SpanV1::new(5, 20));
    let query = builder.query(filter, None, SpanV1::new(0, 20));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 20,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed filter relation expected");
    };
    assert!(matches!(
        relation.root(),
        quanta_gqlang_v2_sema::TypedRelationExprV2::Filter(_)
    ));
    assert_eq!(
        relation.root().inputs().next().unwrap().key().node_id,
        scan_id
    );
    assert_eq!(
        relation
            .root()
            .predicate()
            .unwrap()
            .expression()
            .key()
            .node_id,
        predicate_id
    );
    assert_eq!(
        relation.root().location().location().span,
        SpanV1::new(5, 20)
    );
    assert_eq!(
        relation.output().columns().first().unwrap().origin(),
        relation
            .root()
            .inputs()
            .next()
            .unwrap()
            .output()
            .columns()
            .first()
            .unwrap()
            .origin()
    );
    assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
    assert_eq!(document.expressions().count(), 3);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong nested relation or capture identity must fail this owner test"
)]
fn query_relation_exists_seals_nested_relation_and_exact_binder_capture() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let outer_scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let outer_scan_id = outer_scan.id;
    let inner_scan = builder.relation_source(
        "records2",
        SpanV1::new(25, 33),
        Some(AliasV1 {
            name: "f".into(),
            span: SpanV1::new(34, 35),
        }),
        SpanV1::new(25, 35),
    );
    let inner_scan_id = inner_scan.id;
    let inner_f = builder.symbol("f", SpanV1::new(38, 39));
    let inner_f_path = builder
        .try_path(
            inner_f,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(40, 42),
            }],
            SpanV1::new(38, 42),
        )
        .unwrap();
    let captured_r = builder.symbol("r", SpanV1::new(45, 46));
    let captured_r_id = captured_r.id;
    let captured_r_path = builder
        .try_path(
            captured_r,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(47, 49),
            }],
            SpanV1::new(45, 49),
        )
        .unwrap();
    let inner_compare = builder.compare("=", inner_f_path, captured_r_path, SpanV1::new(38, 49));
    let inner_compare_id = inner_compare.id;
    let inner_filter = builder.relation_filter(inner_scan, inner_compare, SpanV1::new(25, 49));
    let inner_filter_id = inner_filter.id;
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        inner_filter,
        SpanV1::new(20, 50),
    );
    let exists_id = exists.id;
    let outer_filter = builder.relation_filter(outer_scan, exists, SpanV1::new(5, 50));
    let query = builder.query(outer_filter, None, SpanV1::new(0, 50));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let exists_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: exists_id,
    };
    let inner_filter_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: inner_filter_id,
    };
    let exists_expression = document.expression(exists_key).unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::RelationExists {
        polarity,
        relation_root,
        correlations,
    } = exists_expression.shape()
    else {
        panic!("sealed relational EXISTS shape expected")
    };
    assert_eq!(
        (*polarity, *relation_root),
        (
            quanta_gqlang_v2_sema::RelationExistsPolarityV2::Exists,
            inner_filter_key
        )
    );
    let [correlation] = correlations.as_slice() else {
        panic!("one exact outer-alias capture expected")
    };
    assert_eq!(
        (
            correlation.binding().target_node(),
            correlation.binding().ordinal(),
            correlation.binding().kind(),
            correlation.binding().canonical_name(),
            correlation.reference().node_id,
            correlation.location().location().span,
        ),
        (
            outer_scan_id,
            0,
            quanta_gqlang_v2_sema::ResolvedBindingKindV2::Alias,
            "r",
            captured_r_id,
            SpanV1::new(45, 46),
        )
    );
    let nested = document.nested_relation(inner_filter_key).unwrap();
    assert!(matches!(
        nested,
        quanta_gqlang_v2_sema::TypedRelationExprV2::Filter(_)
    ));
    assert_eq!(nested.key(), inner_filter_key);
    assert_eq!(nested.inputs().next().unwrap().key().node_id, inner_scan_id);
    assert_eq!(
        nested.predicate().unwrap().expression().key().node_id,
        inner_compare_id
    );
    assert_eq!(
        document
            .nested_relations()
            .map(|(root, _)| *root)
            .collect::<Vec<_>>(),
        vec![inner_filter_key]
    );
    assert_eq!(document.items().count(), 1);
}

#[test]
#[expect(
    clippy::panic,
    reason = "multi-capture EXISTS must retain each exact reference rather than one binding summary"
)]
fn query_relation_exists_retains_all_outer_reference_captures() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let outer = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let outer_id = outer.id;
    let inner = builder.relation_source(
        "records2",
        SpanV1::new(25, 33),
        Some(AliasV1 {
            name: "f".into(),
            span: SpanV1::new(34, 35),
        }),
        SpanV1::new(25, 35),
    );
    let inner_id = inner.id;
    let path = |name: &str, name_span: SpanV1, field: &str, field_span: SpanV1| {
        builder
            .try_path(
                builder.symbol(name, name_span),
                vec![PathSegmentV1 {
                    name: field.into(),
                    span: field_span,
                }],
                SpanV1::new(name_span.start, field_span.end),
            )
            .unwrap()
    };
    let inner_id_path = path("f", SpanV1::new(38, 39), "id", SpanV1::new(40, 42));
    let captured_id = builder.symbol("r", SpanV1::new(45, 46));
    let captured_id_ref = captured_id.id;
    let captured_id_path = builder
        .try_path(
            captured_id,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(47, 49),
            }],
            SpanV1::new(45, 49),
        )
        .unwrap();
    let inner_value_path = path("f", SpanV1::new(54, 55), "value", SpanV1::new(56, 61));
    let captured_value = builder.symbol("r", SpanV1::new(64, 65));
    let captured_value_ref = captured_value.id;
    let captured_value_path = builder
        .try_path(
            captured_value,
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(66, 71),
            }],
            SpanV1::new(64, 71),
        )
        .unwrap();
    let predicate = builder.and(
        builder.compare("=", inner_id_path, captured_id_path, SpanV1::new(38, 49)),
        builder.compare(
            "=",
            inner_value_path,
            captured_value_path,
            SpanV1::new(54, 71),
        ),
        SpanV1::new(38, 71),
    );
    let inner_filter = builder.relation_filter(inner, predicate, SpanV1::new(25, 71));
    let inner_root = inner_filter.id;
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        inner_filter,
        SpanV1::new(20, 72),
    );
    let exists_id = exists.id;
    let query = builder.query(
        builder.relation_filter(outer, exists, SpanV1::new(5, 72)),
        None,
        SpanV1::new(0, 72),
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 80,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let exists_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: exists_id,
    };
    let quanta_gqlang_v2_sema::TypedExprShapeV2::RelationExists {
        relation_root,
        correlations,
        ..
    } = document.expression(exists_key).unwrap().shape()
    else {
        panic!("multi-capture EXISTS shape expected")
    };
    assert_eq!(relation_root.node_id, inner_root);
    assert_eq!(
        correlations
            .iter()
            .map(|edge| (
                edge.binding().target_node(),
                edge.binding().ordinal(),
                edge.reference().node_id,
                edge.location().location().span,
            ))
            .collect::<Vec<_>>(),
        vec![
            (outer_id, 0, captured_id_ref, SpanV1::new(45, 46)),
            (outer_id, 0, captured_value_ref, SpanV1::new(64, 65)),
        ],
    );
    let root = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: inner_root,
    };
    let nested = document.nested_relation(root).unwrap();
    assert_eq!(nested.key(), root);
    assert_eq!(nested.inputs().next().unwrap().key().node_id, inner_id);
}

#[test]
#[expect(
    clippy::panic,
    reason = "nested EXISTS must retain captures for every crossed correlation boundary"
)]
fn nested_relation_exists_retains_outer_and_inner_capture_sets() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let outer = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let outer_id = outer.id;
    let middle = builder.relation_source(
        "records2",
        SpanV1::new(25, 33),
        Some(AliasV1 {
            name: "f".into(),
            span: SpanV1::new(34, 35),
        }),
        SpanV1::new(25, 35),
    );
    let deepest = builder.relation_source(
        "records2",
        SpanV1::new(65, 73),
        Some(AliasV1 {
            name: "g".into(),
            span: SpanV1::new(74, 75),
        }),
        SpanV1::new(65, 75),
    );
    let path = |name: &str, name_span: SpanV1, field: &str, field_span: SpanV1| {
        builder
            .try_path(
                builder.symbol(name, name_span),
                vec![PathSegmentV1 {
                    name: field.into(),
                    span: field_span,
                }],
                SpanV1::new(name_span.start, field_span.end),
            )
            .unwrap()
    };
    let direct_outer = builder.symbol("r", SpanV1::new(45, 46));
    let direct_outer_ref = direct_outer.id;
    let direct = builder.compare(
        "=",
        path("f", SpanV1::new(38, 39), "id", SpanV1::new(40, 42)),
        builder
            .try_path(
                direct_outer,
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(47, 49),
                }],
                SpanV1::new(45, 49),
            )
            .unwrap(),
        SpanV1::new(38, 49),
    );
    let deep_outer = builder.symbol("r", SpanV1::new(84, 85));
    let deep_outer_ref = deep_outer.id;
    let deep = builder.compare(
        "=",
        path("g", SpanV1::new(78, 79), "value", SpanV1::new(80, 85)),
        builder
            .try_path(
                deep_outer,
                vec![PathSegmentV1 {
                    name: "value".into(),
                    span: SpanV1::new(86, 91),
                }],
                SpanV1::new(84, 91),
            )
            .unwrap(),
        SpanV1::new(78, 91),
    );
    let deepest_filter = builder.relation_filter(deepest, deep, SpanV1::new(65, 91));
    let deepest_root = deepest_filter.id;
    let nested_exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        deepest_filter,
        SpanV1::new(60, 92),
    );
    let nested_exists_id = nested_exists.id;
    let middle_filter = builder.relation_filter(
        middle,
        builder.and(direct, nested_exists, SpanV1::new(38, 92)),
        SpanV1::new(25, 92),
    );
    let middle_root = middle_filter.id;
    let outer_exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        middle_filter,
        SpanV1::new(20, 93),
    );
    let outer_exists_id = outer_exists.id;
    let query = builder.query(
        builder.relation_filter(outer, outer_exists, SpanV1::new(5, 93)),
        None,
        SpanV1::new(0, 93),
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 104,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let shape = |node_id| {
        let key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 { source_id, node_id };
        match document.expression(key).unwrap().shape() {
            quanta_gqlang_v2_sema::TypedExprShapeV2::RelationExists {
                relation_root,
                correlations,
                ..
            } => (*relation_root, correlations),
            quanta_gqlang_v2_sema::TypedExprShapeV2::Literal(_)
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(_)
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Path { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Unary { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Binary { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Call { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Record { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::List { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::BoolValue { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Compare { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::And { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Or { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Not { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Paren { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::LogicalQuantifier { .. } => {
                panic!("relational EXISTS shape expected")
            }
        }
    };
    let (outer_root, outer_captures) = shape(outer_exists_id);
    let (nested_root, nested_captures) = shape(nested_exists_id);
    assert_eq!(outer_root.node_id, middle_root);
    assert_eq!(nested_root.node_id, deepest_root);
    let expected_outer_references = if direct_outer_ref < deep_outer_ref {
        vec![direct_outer_ref, deep_outer_ref]
    } else {
        vec![deep_outer_ref, direct_outer_ref]
    };
    assert_eq!(
        outer_captures
            .iter()
            .map(|edge| edge.reference().node_id)
            .collect::<Vec<_>>(),
        expected_outer_references,
    );
    assert_eq!(
        nested_captures
            .iter()
            .map(|edge| edge.reference().node_id)
            .collect::<Vec<_>>(),
        vec![deep_outer_ref],
    );
    for capture in outer_captures.iter().chain(nested_captures.iter()) {
        assert_eq!(capture.binding().target_node(), outer_id);
        assert_eq!(capture.binding().ordinal(), 0);
        assert_eq!(capture.binding().canonical_name(), "r");
    }
    let roots = document
        .nested_relations()
        .map(|(root, _)| root.node_id)
        .collect::<Vec<_>>();
    let expected_roots = if middle_root < deepest_root {
        vec![middle_root, deepest_root]
    } else {
        vec![deepest_root, middle_root]
    };
    assert_eq!(roots, expected_roots);
}

#[test]
#[expect(
    clippy::panic,
    reason = "uncorrelated EXISTS must retain an explicit empty Binder capture set"
)]
fn query_not_exists_seals_an_uncorrelated_nested_relation() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let outer = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let inner = builder.relation_source("records", SpanV1::new(25, 32), None, SpanV1::new(25, 32));
    let inner_id = inner.id;
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::NotExists,
        inner,
        SpanV1::new(20, 33),
    );
    let exists_id = exists.id;
    let query = builder.query(
        builder.relation_filter(outer, exists, SpanV1::new(5, 33)),
        None,
        SpanV1::new(0, 33),
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 48,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let exists_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: exists_id,
    };
    let inner_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: inner_id,
    };
    let quanta_gqlang_v2_sema::TypedExprShapeV2::RelationExists {
        polarity,
        relation_root,
        correlations,
    } = document.expression(exists_key).unwrap().shape()
    else {
        panic!("sealed relational NOT EXISTS shape expected")
    };
    assert_eq!(
        (*polarity, *relation_root),
        (
            quanta_gqlang_v2_sema::RelationExistsPolarityV2::NotExists,
            inner_key
        )
    );
    assert!(correlations.is_empty());
    assert_eq!(document.nested_relations().count(), 1);
    assert_eq!(
        document.nested_relation(inner_key).unwrap().key(),
        inner_key
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn query_filter_alias_field_type_comes_from_exact_host_row_and_binder_owner() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let scan_id = scan.id;
    let symbol = builder.symbol("r", SpanV1::new(17, 18));
    let symbol_id = symbol.id;
    let path = builder
        .try_path(
            symbol,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(19, 21),
            }],
            SpanV1::new(17, 21),
        )
        .unwrap();
    let predicate = builder.compare(
        "=",
        path,
        builder.int("1", SpanV1::new(24, 25)),
        SpanV1::new(17, 25),
    );
    let filter = builder.relation_filter(scan, predicate, SpanV1::new(5, 25));
    let query = builder.query(filter, None, SpanV1::new(0, 25));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 25,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let expression = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: symbol_id,
        })
        .unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = expression.shape() else {
        panic!("exact row alias binding expected")
    };
    assert_eq!(
        (
            binding.target_node(),
            binding.ordinal(),
            binding.canonical_name()
        ),
        (scan_id, 0, "r")
    );
    let quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(ty) = expression.type_state() else {
        panic!("host row type required")
    };
    let row = ty.as_record().unwrap();
    assert_eq!(row.fields().len(), 1);
    assert_eq!(row.fields().first().unwrap().name(), "id");
    assert_eq!(
        row.fields().first().unwrap().value_type(),
        &ValueTypeV1::int()
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn query_projection_alias_is_a_prior_typed_expression_not_a_host_field_guess() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let path = builder
        .try_path(
            builder.symbol("r", SpanV1::new(22, 23)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(24, 26),
            }],
            SpanV1::new(22, 26),
        )
        .unwrap();
    let path_id = path.id;
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: path,
                alias: Some(AliasV1 {
                    name: "score".into(),
                    span: SpanV1::new(30, 35),
                }),
            }],
            SpanV1::new(5, 35),
        )
        .unwrap();
    let project_id = project.id;
    let output_ref = builder.symbol("score", SpanV1::new(44, 49));
    let output_ref_id = output_ref.id;
    let predicate = builder.compare(
        "=",
        output_ref,
        builder.int("1", SpanV1::new(52, 53)),
        SpanV1::new(44, 53),
    );
    let filter = builder.relation_filter(project, predicate, SpanV1::new(5, 53));
    let query = builder.query(filter, None, SpanV1::new(0, 53));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 53,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed query expected")
    };
    let project = relation.root().inputs().next().unwrap();
    assert!(matches!(
        project,
        quanta_gqlang_v2_sema::TypedRelationExprV2::Project(_)
    ));
    assert_eq!(project.expressions().first().unwrap().node_id, path_id);
    assert_eq!(project.output().columns().first().unwrap().name(), "score");
    assert_eq!(
        project.output().columns().first().unwrap().origin(),
        &quanta_gqlang_v2_sema::OutputColumnOriginV2::Expression(
            quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
                source_id,
                node_id: path_id
            }
        )
    );
    assert_eq!(
        project
            .output()
            .columns()
            .first()
            .unwrap()
            .location()
            .location()
            .span,
        SpanV1::new(30, 35)
    );
    let reference = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: output_ref_id,
        })
        .unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = reference.shape() else {
        panic!("projection frame binding required")
    };
    assert_eq!(
        (
            binding.target_node(),
            binding.ordinal(),
            binding.canonical_name()
        ),
        (project_id, 0, "score")
    );
    assert_eq!(
        reference.type_state(),
        &quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(ValueTypeV1::int())
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn self_join_projected_columns_retain_distinct_scan_occurrences_and_checked_on() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let left_scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "l".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let left_scan_id = left_scan.id;
    let left_value = builder
        .try_path(
            builder.symbol("l", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let left = builder
        .try_project(
            left_scan,
            vec![ProjectionItemV1 {
                value: left_value,
                alias: Some(AliasV1 {
                    name: "left_id".into(),
                    span: SpanV1::new(28, 35),
                }),
            }],
            SpanV1::new(5, 35),
        )
        .unwrap();
    let right_scan = builder.relation_source(
        "records",
        SpanV1::new(40, 47),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(48, 49),
        }),
        SpanV1::new(40, 49),
    );
    let right_scan_id = right_scan.id;
    let right_value = builder
        .try_path(
            builder.symbol("r", SpanV1::new(55, 56)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(57, 59),
            }],
            SpanV1::new(55, 59),
        )
        .unwrap();
    let right = builder
        .try_project(
            right_scan,
            vec![ProjectionItemV1 {
                value: right_value,
                alias: Some(AliasV1 {
                    name: "right_id".into(),
                    span: SpanV1::new(63, 71),
                }),
            }],
            SpanV1::new(40, 71),
        )
        .unwrap();
    let on = builder.compare(
        "=",
        builder.symbol("left_id", SpanV1::new(75, 82)),
        builder.symbol("right_id", SpanV1::new(85, 93)),
        SpanV1::new(75, 93),
    );
    let on_id = on.id;
    let join = builder.inner_join(left, right, on, SpanV1::new(5, 93));
    let query = builder.query(join, None, SpanV1::new(0, 93));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 93,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed self-join required")
    };
    assert!(matches!(
        relation.root(),
        quanta_gqlang_v2_sema::TypedRelationExprV2::Join(_)
    ));
    assert_eq!(
        relation.root().join_mode(),
        Some(quanta_gqlang_v2_sema::JoinModeV2::Inner)
    );
    assert_eq!(
        relation
            .root()
            .predicate()
            .unwrap()
            .expression()
            .key()
            .node_id,
        on_id
    );
    assert_eq!(
        relation
            .output()
            .row()
            .fields()
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
        vec!["left_id", "right_id"]
    );
    let sources = relation
        .root()
        .inputs()
        .map(|project| project.inputs().next().unwrap().key().node_id)
        .collect::<Vec<_>>();
    assert_eq!(sources, vec![left_scan_id, right_scan_id]);
    assert_ne!(sources.first().unwrap(), sources.get(1).unwrap());
    assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn ordered_projection_alias_and_null_key_retain_direction_null_policy_and_sequence() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: builder.int("3", SpanV1::new(20, 21)),
                alias: Some(AliasV1 {
                    name: "score".into(),
                    span: SpanV1::new(24, 29),
                }),
            }],
            SpanV1::new(5, 29),
        )
        .unwrap();
    let asc = builder.symbol("score", SpanV1::new(39, 44));
    let asc_id = asc.id;
    let desc = builder.null(SpanV1::new(51, 55));
    let desc_id = desc.id;
    let order = builder
        .order_by(
            project,
            vec![
                OrderKeyHirV1 {
                    value: asc,
                    direction: SortDirectionHirV1::Asc,
                },
                OrderKeyHirV1 {
                    value: desc,
                    direction: SortDirectionHirV1::Desc,
                },
            ],
            SpanV1::new(5, 55),
        )
        .unwrap();
    let query = builder.query(order, None, SpanV1::new(0, 55));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 55,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed sort required")
    };
    assert!(matches!(
        relation.root(),
        quanta_gqlang_v2_sema::TypedRelationExprV2::Sort(_)
    ));
    assert_eq!(relation.output().cardinality(), CardinalityV1::Sequence);
    assert_eq!(
        relation
            .root()
            .expressions()
            .iter()
            .map(|key| key.node_id)
            .collect::<Vec<_>>(),
        vec![asc_id, desc_id]
    );
    assert_eq!(
        relation
            .output()
            .ordering()
            .iter()
            .map(|key| (key.direction(), key.null_order()))
            .collect::<Vec<_>>(),
        vec![
            (
                quanta_gqlang_v2_sema::SortDirectionV2::Asc,
                quanta_gqlang_v2_sema::NullOrderV2::Last
            ),
            (
                quanta_gqlang_v2_sema::SortDirectionV2::Desc,
                quanta_gqlang_v2_sema::NullOrderV2::Last
            ),
        ]
    );
    assert_eq!(relation.output().columns().first().unwrap().name(), "score");
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn grouped_key_only_row_is_visible_to_sort_by_binder_owned_ordinal() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let key_id = key.id;
    let group = builder
        .group_by(scan, vec![key], SpanV1::new(5, 30))
        .unwrap();
    let group_id = group.id;
    let order_ref = builder.symbol("id", SpanV1::new(39, 41));
    let order_ref_id = order_ref.id;
    let order = builder
        .order_by(
            group,
            vec![OrderKeyHirV1 {
                value: order_ref,
                direction: SortDirectionHirV1::Asc,
            }],
            SpanV1::new(5, 41),
        )
        .unwrap();
    let query = builder.query(order, None, SpanV1::new(0, 41));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 41,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed group and sort required")
    };
    let group = relation.root().inputs().next().unwrap();
    assert!(matches!(
        group,
        quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_)
    ));
    assert_eq!(group.output().cardinality(), CardinalityV1::Set);
    assert_eq!(
        group
            .output()
            .row()
            .fields()
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
        vec!["id"]
    );
    assert_eq!(
        group.output().partition_keys().first().unwrap().node_id,
        key_id
    );
    assert_eq!(relation.output().cardinality(), CardinalityV1::Sequence);
    let reference = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: order_ref_id,
        })
        .unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = reference.shape() else {
        panic!("grouped key binding expected")
    };
    assert_eq!(
        (
            binding.target_node(),
            binding.ordinal(),
            binding.canonical_name()
        ),
        (group_id, 0, "id")
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn source_alias_disappears_from_public_group_output_frame() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![key], SpanV1::new(5, 30))
        .unwrap();
    let unavailable = builder.symbol("r", SpanV1::new(39, 40));
    let order = builder
        .order_by(
            group,
            vec![OrderKeyHirV1 {
                value: unavailable,
                direction: SortDirectionHirV1::Asc,
            }],
            SpanV1::new(5, 40),
        )
        .unwrap();
    let query = builder.query(order, None, SpanV1::new(0, 40));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 40,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("pre-group alias cannot escape group frame")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(39, 40)
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn post_group_filter_cannot_recover_pre_group_field_without_aggregate_authority() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![key], SpanV1::new(5, 30))
        .unwrap();
    let value = builder
        .try_path(
            builder.symbol("r", SpanV1::new(39, 40)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(41, 46),
            }],
            SpanV1::new(39, 46),
        )
        .unwrap();
    let predicate = builder.compare(
        "=",
        value,
        builder.int("1", SpanV1::new(49, 50)),
        SpanV1::new(39, 50),
    );
    let filter = builder.relation_filter(group, predicate, SpanV1::new(5, 50));
    let query = builder.query(filter, None, SpanV1::new(0, 50));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 50,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("post-group WHERE must see only public key row")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(39, 40)
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn grouped_aggregate_project_uses_sealed_partition_input_not_public_key_frame() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let scan_id = scan.id;
    let group_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .unwrap();
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(42, 43)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(44, 49),
            }],
            SpanV1::new(42, 49),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(38, 41),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(38, 50),
    );
    let sum_id = sum.id;
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value: sum,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(55, 60),
                }),
            }],
            SpanV1::new(5, 60),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 60));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 60,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed grouped aggregate expected")
    };
    let group = relation.root().inputs().next().unwrap();
    assert_eq!(group.partition_input().len(), 1);
    assert_eq!(
        (
            group
                .partition_input()
                .first()
                .unwrap()
                .binding()
                .target_node(),
            group.partition_input().first().unwrap().binding().ordinal()
        ),
        (scan_id, 0)
    );
    assert_eq!(
        group
            .output()
            .columns()
            .iter()
            .map(|column| column.name())
            .collect::<Vec<_>>(),
        vec!["id"]
    );
    assert_eq!(
        relation
            .output()
            .columns()
            .iter()
            .map(|column| column.name())
            .collect::<Vec<_>>(),
        vec!["total"]
    );
    assert_eq!(
        relation.root().expressions().first().unwrap().node_id,
        sum_id
    );
}

#[test]
fn partition_candidate_cannot_cross_an_intervening_sort_stage() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let group_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .unwrap();
    let sorted = builder
        .order_by(
            group,
            vec![OrderKeyHirV1 {
                value: builder.symbol("id", SpanV1::new(35, 37)),
                direction: SortDirectionHirV1::Asc,
            }],
            SpanV1::new(5, 37),
        )
        .unwrap();
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(49, 50)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(51, 56),
            }],
            SpanV1::new(49, 56),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(45, 48),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(45, 57),
    );
    let project = builder
        .try_project(
            sorted,
            vec![ProjectionItemV1 {
                value: sum,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(61, 66),
                }),
            }],
            SpanV1::new(5, 66),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 66));
    let result = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 66,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    );
    assert!(
        matches!(result, Err(FrontendTransactionErrorV2::Document(ref error))
        if error.diagnostics().first().is_some_and(|diagnostic|
            diagnostic.code() == "sema.document_validation_failed"
            && diagnostic.primary().span == SpanV1::new(49, 50))),
        "{result:?}"
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn scalar_over_aggregate_cannot_smuggle_group_partition_input() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let group_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .unwrap();
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(44, 45)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(46, 48),
            }],
            SpanV1::new(44, 48),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(40, 43),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(40, 49),
    );
    let outer = builder.call(
        "is_null",
        SpanV1::new(32, 39),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(sum)]).unwrap(),
        SpanV1::new(32, 50),
    );
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value: outer,
                alias: Some(AliasV1 {
                    name: "bad".into(),
                    span: SpanV1::new(55, 58),
                }),
            }],
            SpanV1::new(5, 58),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 58));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 58,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("composed aggregate effect cannot authorize scalar outer call")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.illegal_aggregate_context"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(44, 45)
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn scalar_call_cannot_authorize_a_group_partition_candidate() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![key], SpanV1::new(5, 30))
        .unwrap();
    let arg = builder
        .try_path(
            builder.symbol("r", SpanV1::new(42, 43)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(44, 49),
            }],
            SpanV1::new(42, 49),
        )
        .unwrap();
    let value = builder.call(
        "is_null",
        SpanV1::new(34, 41),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
        SpanV1::new(34, 50),
    );
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value,
                alias: Some(AliasV1 {
                    name: "bad".into(),
                    span: SpanV1::new(50, 53),
                }),
            }],
            SpanV1::new(5, 53),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 53));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 53,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("scalar call alone must not authorize partition-input reference")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.illegal_aggregate_context"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(42, 43)
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn plain_group_partition_field_cannot_be_projected_without_aggregate() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![key], SpanV1::new(5, 30))
        .unwrap();
    let value = builder
        .try_path(
            builder.symbol("r", SpanV1::new(42, 43)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(44, 49),
            }],
            SpanV1::new(42, 49),
        )
        .unwrap();
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value,
                alias: Some(AliasV1 {
                    name: "bad".into(),
                    span: SpanV1::new(52, 55),
                }),
            }],
            SpanV1::new(5, 55),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 55));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 55,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("plain group partition field is unavailable")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(42, 43)
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn nested_aggregate_cannot_authorize_group_partition_candidate() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![key], SpanV1::new(5, 30))
        .unwrap();
    let arg = builder
        .try_path(
            builder.symbol("r", SpanV1::new(44, 45)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(46, 48),
            }],
            SpanV1::new(44, 48),
        )
        .unwrap();
    let inner = builder.call(
        "sum",
        SpanV1::new(40, 43),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
        SpanV1::new(40, 49),
    );
    let outer = builder.call(
        "sum",
        SpanV1::new(32, 35),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(inner)]).unwrap(),
        SpanV1::new(32, 50),
    );
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value: outer,
                alias: Some(AliasV1 {
                    name: "bad".into(),
                    span: SpanV1::new(55, 58),
                }),
            }],
            SpanV1::new(5, 58),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 58));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 58,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("nested aggregates cannot consume partition input")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.illegal_aggregate_context"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(44, 45)
    );
}

#[test]
fn standalone_declaration_cannot_introduce_an_aggregate() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let argument = builder.int("1", SpanV1::new(14, 15));
    let aggregate = builder.call(
        "sum",
        SpanV1::new(10, 13),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(10, 16),
    );
    let decl = builder.decl("total", SpanV1::new(4, 9), aggregate, SpanV1::new(0, 16));
    let result = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 16,
            items: vec![ProgramItemHirV1::Decl(Box::new(decl))],
        })
        .unwrap(),
        &authority,
    );
    assert!(
        matches!(result, Err(FrontendTransactionErrorV2::Document(ref error))
        if error.diagnostics().first().is_some_and(|diagnostic|
            diagnostic.code() == "sema.illegal_aggregate_context"
            && diagnostic.primary().span == SpanV1::new(10, 16))),
        "{result:?}"
    );
}

#[test]
fn order_key_cannot_introduce_an_aggregate_inside_a_nested_expression() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let source = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let argument = builder.int("1", SpanV1::new(28, 29));
    let aggregate = builder.call(
        "sum",
        SpanV1::new(24, 27),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(24, 30),
    );
    let wrapped = builder.paren_value(aggregate, SpanV1::new(23, 31));
    let sorted = builder
        .order_by(
            source,
            vec![OrderKeyHirV1 {
                value: wrapped,
                direction: SortDirectionHirV1::Asc,
            }],
            SpanV1::new(5, 31),
        )
        .unwrap();
    let query = builder.query(sorted, None, SpanV1::new(0, 31));
    let result = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 31,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    );
    assert!(
        matches!(result, Err(FrontendTransactionErrorV2::Document(ref error))
        if error.diagnostics().first().is_some_and(|diagnostic|
            diagnostic.code() == "sema.illegal_aggregate_context"
            && diagnostic.primary().span == SpanV1::new(24, 30))),
        "{result:?}"
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn identical_host_relation_scans_preserve_distinct_authored_occurrence_ids() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let first = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "u".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let second = builder.relation_source(
        "records",
        SpanV1::new(25, 32),
        Some(AliasV1 {
            name: "v".into(),
            span: SpanV1::new(33, 34),
        }),
        SpanV1::new(25, 34),
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 34,
            items: vec![
                ProgramItemHirV1::Query(Box::new(quanta_gqlang_v2_hir::QueryHirV1::new(
                    HirNodeIdV1::from_raw(20),
                    SpanV1::new(0, 14),
                    first,
                    None,
                ))),
                ProgramItemHirV1::Query(Box::new(quanta_gqlang_v2_hir::QueryHirV1::new(
                    HirNodeIdV1::from_raw(21),
                    SpanV1::new(20, 34),
                    second,
                    None,
                ))),
            ],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let relation = |node| {
        let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document
            .item(SemanticItemIdV2::new(
                source_id,
                HirNodeIdV1::from_raw(node),
            ))
            .unwrap()
        else {
            panic!("typed scan expected");
        };
        relation
    };
    let first = relation(20);
    let second = relation(21);
    assert_eq!(first.root().source(), second.root().source());
    assert_ne!(first.root().key(), second.root().key());
    assert_ne!(
        first.output().columns().first().unwrap().origin(),
        second.output().columns().first().unwrap().origin()
    );
}

#[test]
fn diagnostics_sort_stage_severity_code_and_secondary_edges() {
    use quanta_gqlang_v2_sema::{
        FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2,
    };
    let source = SourceIdV2::from_raw(9);
    let a = LocatedSpanV2 {
        source_id: source,
        span: SpanV1::new(3, 4),
    };
    let b = LocatedSpanV2 {
        source_id: source,
        span: SpanV1::new(8, 9),
    };
    let later = FrontendDiagnosticV2::new(
        FrontendStageV2::Typing,
        "z",
        FrontendSeverityV2::Error,
        b,
        vec![b, a, a],
        "Ω".repeat(600),
    );
    assert_eq!(later.secondary(), &[a, b]);
    assert!(later.message().len() <= 1_024);
    assert!(later.message().is_char_boundary(later.message().len()));
    let first = FrontendDiagnosticV2::new(
        FrontendStageV2::Binding,
        "a",
        FrontendSeverityV2::Error,
        a,
        Vec::new(),
        "first",
    );
    let failure = FrontendFailureV2::try_new(vec![later, first]).unwrap();
    assert_eq!(
        failure
            .diagnostics()
            .iter()
            .map(FrontendDiagnosticV2::code)
            .collect::<Vec<_>>(),
        vec!["a", "z"]
    );
    assert!(FrontendFailureV2::try_new(Vec::new()).is_err());
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic shape must fail this owner test"
)]
fn scalar_count_star_projects_through_exactly_one_implicit_empty_group() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let count = builder.call(
        "count",
        SpanV1::new(24, 29),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Star {
            span: SpanV1::new(30, 31),
        }])
        .unwrap(),
        SpanV1::new(24, 31),
    );
    let count_id = count.id;
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: count,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(36, 41),
                }),
            }],
            SpanV1::new(16, 41),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 41));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("scalar aggregate relation expected");
    };
    // Exact Group(empty) -> Project(aggregate) shape with Set cardinality,
    // empty partition keys, retained star, and no caller option.
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Project(_) = relation.root() else {
        panic!("aggregate projection root expected");
    };
    let group = relation.root().inputs().next().unwrap();
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_) = group else {
        panic!("implicit empty-partition Group expected");
    };
    assert!(group.expressions().is_empty());
    assert!(group.partition_input().is_empty());
    assert_eq!(group.output().cardinality(), CardinalityV1::Set);
    assert!(group.output().partition_keys().is_empty());
    assert!(group.output().columns().is_empty());
    assert!(group.output().row().fields().is_empty());
    assert_eq!(relation.output().cardinality(), CardinalityV1::Set);
    assert!(relation.output().partition_keys().is_empty());
    assert!(relation.output().ordering().is_empty());
    let count = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: count_id,
        })
        .unwrap();
    assert_eq!(
        count.shape(),
        &quanta_gqlang_v2_sema::TypedExprShapeV2::Call {
            target: count.resolved_call().cloned().unwrap(),
            arguments: vec![quanta_gqlang_v2_sema::TypedCallArgumentV2::Star],
        }
    );
    assert!(count.aggregate_context().uses_star());
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic shape must fail this owner test"
)]
fn grouped_sum_output_retains_the_authored_group_partition_identity() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let scan_id = scan.id;
    let group_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(22, 23)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(24, 26),
            }],
            SpanV1::new(22, 26),
        )
        .unwrap();
    let group_key_id = group_key.id;
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .unwrap();
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(42, 43)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(44, 49),
            }],
            SpanV1::new(42, 49),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(38, 41),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(38, 50),
    );
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value: sum,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(55, 60),
                }),
            }],
            SpanV1::new(34, 60),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 60));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("grouped aggregate relation expected");
    };
    // Exactly one authored Group; the aggregate Project consumes it directly
    // and no second Group is synthesized.
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Project(_) = relation.root() else {
        panic!("aggregate projection root expected");
    };
    let group = relation.root().inputs().next().unwrap();
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_) = group else {
        panic!("the authored Group must be preserved");
    };
    let mut groups = 0usize;
    fn count_groups(stage: &quanta_gqlang_v2_sema::TypedRelationExprV2, groups: &mut usize) {
        if matches!(stage, quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_)) {
            *groups += 1;
        }
        for child in stage.inputs() {
            count_groups(child, groups);
        }
    }
    count_groups(relation.root(), &mut groups);
    assert_eq!(groups, 1, "exactly one authored Group may exist");
    // Exact Binder partition input is retained on the Group.
    assert_eq!(group.partition_input().len(), 1);
    let partition = group.partition_input().first().unwrap();
    assert_eq!(
        (
            partition.binding().target_node(),
            partition.binding().ordinal()
        ),
        (scan_id, 0)
    );
    // Aggregate output retains the exact authored partition identity.
    assert_eq!(
        relation.output().partition_keys(),
        &[quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: group_key_id,
        }]
    );
    assert_eq!(relation.output().cardinality(), CardinalityV1::Set);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic shape must fail this owner test"
)]
fn multiple_aggregates_and_key_plus_aggregate_keep_authored_order_and_aligned_origins() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let group_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(22, 23)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(24, 26),
            }],
            SpanV1::new(22, 26),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .unwrap();
    // Authored order: sum(r.value), then the key, then count(r.value).
    let sum_argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(44, 45)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(46, 51),
            }],
            SpanV1::new(44, 51),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(40, 43),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(sum_argument)]).unwrap(),
        SpanV1::new(40, 52),
    );
    let sum_id = sum.id;
    let key_ref = builder.symbol("id", SpanV1::new(56, 58));
    let key_ref_id = key_ref.id;
    let count_argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(68, 69)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(70, 75),
            }],
            SpanV1::new(68, 75),
        )
        .unwrap();
    let count = builder.call(
        "count",
        SpanV1::new(62, 67),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(count_argument)]).unwrap(),
        SpanV1::new(62, 76),
    );
    let count_id = count.id;
    let project = builder
        .try_project(
            group,
            vec![
                ProjectionItemV1 {
                    value: sum,
                    alias: Some(AliasV1 {
                        name: "total".into(),
                        span: SpanV1::new(80, 85),
                    }),
                },
                ProjectionItemV1 {
                    value: key_ref,
                    alias: None,
                },
                ProjectionItemV1 {
                    value: count,
                    alias: Some(AliasV1 {
                        name: "n".into(),
                        span: SpanV1::new(96, 97),
                    }),
                },
            ],
            SpanV1::new(36, 97),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 97));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 128,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("grouped aggregate relation expected");
    };
    // Stable authored expression order on the aggregate Project.
    assert_eq!(
        relation
            .root()
            .expressions()
            .iter()
            .map(|key| key.node_id)
            .collect::<Vec<_>>(),
        vec![sum_id, key_ref_id, count_id]
    );
    // Exact row-aligned expression origins, one per output column, in order.
    for (expression, column) in relation
        .root()
        .expressions()
        .iter()
        .zip(relation.output().columns())
    {
        assert_eq!(
            column.origin(),
            &quanta_gqlang_v2_sema::OutputColumnOriginV2::Expression(*expression)
        );
    }
    assert_eq!(relation.output().columns().len(), 3);
    assert_eq!(relation.output().row().fields().len(), 3);
    assert_eq!(relation.output().cardinality(), CardinalityV1::Set);
    assert_eq!(relation.output().partition_keys().len(), 1);
}

#[test]
fn mixed_star_argument_is_unrepresentable_in_checked_hir() {
    // `count(*, x)`: the HIR owner rejects a star mixed with an expression
    // argument at construction, so the illegal shape can never reach the
    // semantic transaction, let alone algebra.
    let error = CallArgsHirV1::try_new(vec![
        CallArgHirV1::Star {
            span: SpanV1::new(30, 31),
        },
        CallArgHirV1::Expr(HirBuilderV1::new().int("1", SpanV1::new(33, 34))),
    ])
    .unwrap_err();
    assert_eq!(error, "'*' must be the sole call argument");
}

#[test]
fn illegal_star_matrix_fails_before_algebra_at_the_authored_call_span() {
    for (name, args, call_span) in [
        (
            "sum",
            vec![CallArgHirV1::Star {
                span: SpanV1::new(30, 31),
            }],
            SpanV1::new(24, 31),
        ),
        (
            "min",
            vec![CallArgHirV1::Star {
                span: SpanV1::new(30, 31),
            }],
            SpanV1::new(24, 31),
        ),
        (
            "max",
            vec![CallArgHirV1::Star {
                span: SpanV1::new(30, 31),
            }],
            SpanV1::new(24, 31),
        ),
    ] {
        let (_host, authority) = host();
        let builder = HirBuilderV1::new();
        let scan = builder.relation_source(
            "records",
            SpanV1::new(5, 12),
            Some(AliasV1 {
                name: "r".into(),
                span: SpanV1::new(13, 14),
            }),
            SpanV1::new(5, 14),
        );
        let call = builder.call(
            name,
            SpanV1::new(24, 29),
            CallArgsHirV1::try_new(args).unwrap(),
            call_span,
        );
        let project = builder
            .try_project(
                scan,
                vec![ProjectionItemV1 {
                    value: call,
                    alias: None,
                }],
                SpanV1::new(16, call_span.end),
            )
            .unwrap();
        let query = builder.query(project, None, SpanV1::new(0, call_span.end));
        let result = resolve_checked_document_v2(
            CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
                source_id: SourceIdV2::from_raw(9),
                source_len: 64,
                items: vec![ProgramItemHirV1::Query(Box::new(query))],
            })
            .unwrap(),
            &authority,
        );
        let FrontendTransactionErrorV2::Document(error) = result.unwrap_err() else {
            panic!("{name} star must fail inside the document transaction");
        };
        let diagnostic = error.diagnostics().first().unwrap();
        assert_eq!(diagnostic.code(), "types.invalid_star_argument", "{name}");
        assert_eq!(
            diagnostic.primary(),
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: call_span,
            },
            "{name} star must fail at the exact authored call span"
        );
    }
}

#[test]
fn global_aggregate_projection_cannot_retain_a_plain_column() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let plain = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let count = builder.call(
        "count",
        SpanV1::new(27, 32),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Star {
            span: SpanV1::new(33, 34),
        }])
        .unwrap(),
        SpanV1::new(27, 34),
    );
    let project = builder
        .try_project(
            scan,
            vec![
                ProjectionItemV1 {
                    value: plain,
                    alias: None,
                },
                ProjectionItemV1 {
                    value: count,
                    alias: None,
                },
            ],
            SpanV1::new(16, 34),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 34));
    let result = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    );
    assert!(
        matches!(result, Err(FrontendTransactionErrorV2::Document(ref error))
        if error.diagnostics().first().is_some_and(|diagnostic|
            diagnostic.code() == "sema.mixed_aggregate_projection"
                && diagnostic.primary().span == SpanV1::new(20, 24))),
        "{result:?}"
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn global_nested_aggregate_is_rejected_at_the_inner_call() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(32, 33)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(34, 39),
            }],
            SpanV1::new(32, 39),
        )
        .unwrap();
    let inner_span = SpanV1::new(28, 40);
    let inner = builder.call(
        "sum",
        SpanV1::new(28, 31),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        inner_span,
    );
    let outer = builder.call(
        "sum",
        SpanV1::new(24, 27),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(inner)]).unwrap(),
        SpanV1::new(24, 41),
    );
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: outer,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(45, 50),
                }),
            }],
            SpanV1::new(16, 50),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 50));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("nested global aggregate must fail in sema")
    };
    let diagnostic = error.diagnostics().first().unwrap();
    assert_eq!(diagnostic.code(), "sema.illegal_aggregate_context");
    assert_eq!(diagnostic.primary().span, inner_span);
}

#[test]
fn global_single_sum_remains_admitted() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(28, 29)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(30, 35),
            }],
            SpanV1::new(28, 35),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(24, 27),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(24, 36),
    );
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: sum,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(40, 45),
                }),
            }],
            SpanV1::new(16, 45),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 45));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("global SUM must remain a relation")
    };
    assert!(matches!(
        relation.root(),
        quanta_gqlang_v2_sema::TypedRelationExprV2::Project(_)
    ));
    assert!(matches!(
        relation.root().inputs().next(),
        Some(quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_))
    ));
}
