use super::*;

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
fn export_profile_charges_one_step_at_the_exact_authored_use() {
    let budget = |steps| {
        let standard = FrontendBudgetV2::standard_v2().unwrap();
        FrontendBudgetV2::try_new(
            standard.max_source_bytes(),
            standard.max_frame_bytes(),
            standard.max_tokens(),
            standard.max_document_items(),
            standard.max_expression_depth(),
            standard.max_relation_depth(),
            standard.max_expression_nodes(),
            standard.max_collection_width(),
            standard.max_projection_width(),
            steps,
            standard.max_module_graph_nodes(),
            standard.max_module_graph_edges(),
            standard.max_module_steps(),
            standard.max_authority_bytes(),
            standard.max_authority_string_bytes(),
            standard.max_rule_graph_nodes(),
            standard.max_rule_graph_edges(),
            standard.max_rule_steps(),
            standard.max_witness_depth(),
            standard.max_witnesses(),
            standard.max_diagnostics(),
            standard.max_response_bytes(),
        )
        .unwrap()
    };
    // Profile item + capability, EXPORT item + source, optional profile use.
    // Keep the declaration first so the final charge is the profile use.
    const WITHOUT_PROFILE_STEPS: u32 = 4;
    const WITH_PROFILE_STEPS: u32 = 5;
    let (_, exact) = host_with_budget(budget(WITH_PROFILE_STEPS));
    let document =
        resolve_checked_document_v2(export_profile_document(Some("strict"), true, true), &exact)
            .expect("five authored semantic charges fit exactly");
    assert_eq!(
        document.budget_receipt().max_semantic_steps(),
        WITH_PROFILE_STEPS
    );

    let (_, under) = host_with_budget(budget(WITHOUT_PROFILE_STEPS));
    resolve_checked_document_v2(export_profile_document(None, true, true), &under)
        .expect("the identical document without the optional use fits four steps");
    let FrontendTransactionErrorV2::Document(failure) =
        resolve_checked_document_v2(export_profile_document(Some("strict"), true, true), &under)
            .expect_err("the optional use must consume its own fifth step")
    else {
        panic!("expected document resource diagnostic")
    };
    let [diagnostic] = failure.diagnostics() else {
        panic!("expected one resource diagnostic")
    };
    assert_eq!(diagnostic.code(), "frontend.resource_limit_exceeded");
    assert_eq!(
        diagnostic.primary(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(9),
            span: SpanV1::new(88, 94),
        }
    );
    assert!(diagnostic.secondary().is_empty());
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
        | quanta_gqlang_v2_sema::SemanticItemV2::Command(_)
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
