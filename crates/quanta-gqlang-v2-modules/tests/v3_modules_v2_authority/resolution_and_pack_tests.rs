use super::*;

fn finish_graph_case(
    configure: impl FnOnce(
        &SourceRegistryViewV2,
        &ModuleIdV2,
        &ModuleIdV2,
        &ModuleIdV2,
        &mut ModuleAuthorityBuilderV2,
    ),
) -> Result<ValidatedModuleAuthorityPartsV2, ModuleDiagnosticV2> {
    let sources = registry(&[(1, 4096)]);
    let app = module_id(&sources, "app", 0);
    let left = module_id(&sources, "left", 1);
    let right = module_id(&sources, "right", 2);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for (id, at) in [(&app, 10), (&left, 11), (&right, 12)] {
        builder.try_add_module(ModuleNodeInputV2 {
            module_id: id.clone(),
            location: checked(&sources, 1, at),
        })?;
    }
    configure(&sources, &app, &left, &right, &mut builder);
    builder.finish(
        CurrentModuleSelectionInputV2 {
            module_id: app,
            location: checked(&sources, 1, 99),
        },
        VersionV2::new(1, 0, 0),
    )
}

#[test]
fn local_declaration_shadows_direct_import() {
    let parts = finish_graph_case(|sources, app, left, _, builder| {
        builder
            .try_declare(ModuleDeclarationInputV2 {
                owner_module_id: left.clone(),
                local_name: "x".into(),
                namespace_kind: NamespaceKindV2::Value,
                visibility: VisibilityV2::Public,
                location: checked(sources, 1, 20),
            })
            .unwrap();
        builder
            .try_import(ModuleImportInputV2 {
                owner_module_id: app.clone(),
                local_name: "x".into(),
                target: asset_ref(sources, left, NamespaceKindV2::Value, "x", 21),
                location: checked(sources, 1, 22),
            })
            .unwrap();
        builder
            .try_declare(ModuleDeclarationInputV2 {
                owner_module_id: app.clone(),
                local_name: "x".into(),
                namespace_kind: NamespaceKindV2::Value,
                visibility: VisibilityV2::Private,
                location: checked(sources, 1, 23),
            })
            .unwrap();
    })
    .unwrap();
    let binding = parts
        .resolved_graph()
        .visible_bindings()
        .get(NamespaceKindV2::Value, "x")
        .unwrap();
    assert_eq!(binding.declaration_origin().module_id().as_str(), "app");
    assert_eq!(binding.visibility(), VisibilityV2::Private);
}

#[test]
fn direct_import_precedes_reexport() {
    let parts = finish_graph_case(|sources, app, left, right, builder| {
        for (owner, at) in [(left, 20), (right, 21)] {
            builder
                .try_declare(ModuleDeclarationInputV2 {
                    owner_module_id: owner.clone(),
                    local_name: "x".into(),
                    namespace_kind: NamespaceKindV2::Value,
                    visibility: VisibilityV2::Public,
                    location: checked(sources, 1, at),
                })
                .unwrap();
        }
        builder
            .try_reexport(ModuleReexportInputV2 {
                owner_module_id: app.clone(),
                exported_name: "x".into(),
                target: asset_ref(sources, right, NamespaceKindV2::Value, "x", 22),
                location: checked(sources, 1, 23),
            })
            .unwrap();
        builder
            .try_import(ModuleImportInputV2 {
                owner_module_id: app.clone(),
                local_name: "x".into(),
                target: asset_ref(sources, left, NamespaceKindV2::Value, "x", 24),
                location: checked(sources, 1, 25),
            })
            .unwrap();
    })
    .unwrap();
    let binding = parts
        .resolved_graph()
        .visible_bindings()
        .get(NamespaceKindV2::Value, "x")
        .unwrap();
    assert_eq!(binding.declaration_origin().module_id().as_str(), "left");
}

#[test]
fn higher_priority_binding_shadows_ambiguous_reexports() {
    for (direct, local, expected_module) in [(true, false, "left"), (false, true, "app")] {
        let parts = finish_graph_case(|sources, app, left, right, builder| {
            for (owner, at) in [(left, 20), (right, 21)] {
                builder
                    .try_declare(ModuleDeclarationInputV2 {
                        owner_module_id: owner.clone(),
                        local_name: "x".into(),
                        namespace_kind: NamespaceKindV2::Value,
                        visibility: VisibilityV2::Public,
                        location: checked(sources, 1, at),
                    })
                    .unwrap();
                builder
                    .try_reexport(ModuleReexportInputV2 {
                        owner_module_id: app.clone(),
                        exported_name: "x".into(),
                        target: asset_ref(sources, owner, NamespaceKindV2::Value, "x", at + 2),
                        location: checked(sources, 1, at + 4),
                    })
                    .unwrap();
            }
            if direct {
                builder
                    .try_import(ModuleImportInputV2 {
                        owner_module_id: app.clone(),
                        local_name: "x".into(),
                        target: asset_ref(sources, left, NamespaceKindV2::Value, "x", 30),
                        location: checked(sources, 1, 31),
                    })
                    .unwrap();
            }
            if local {
                builder
                    .try_declare(ModuleDeclarationInputV2 {
                        owner_module_id: app.clone(),
                        local_name: "x".into(),
                        namespace_kind: NamespaceKindV2::Value,
                        visibility: VisibilityV2::Private,
                        location: checked(sources, 1, 32),
                    })
                    .unwrap();
            }
        })
        .expect("a shadowed lower-priority ambiguity must not reject");
        let binding = parts
            .resolved_graph()
            .visible_bindings()
            .get(NamespaceKindV2::Value, "x")
            .unwrap();
        assert_eq!(
            binding.declaration_origin().module_id().as_str(),
            expected_module
        );
    }
}

#[test]
fn direct_import_is_local_only_and_explicit_reexport_controls_downstream_origin() {
    let sources = registry(&[(1, 4096)]);
    let app = module_id(&sources, "app", 0);
    let left = module_id(&sources, "left", 1);
    let right = module_id(&sources, "right", 2);
    let consumer = module_id(&sources, "consumer", 3);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for (id, at) in [(&app, 10), (&left, 11), (&right, 12), (&consumer, 13)] {
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: id.clone(),
                location: checked(&sources, 1, at),
            })
            .unwrap();
    }
    for (owner, at) in [(&left, 20), (&right, 21)] {
        builder
            .try_declare(ModuleDeclarationInputV2 {
                owner_module_id: owner.clone(),
                local_name: "x".into(),
                namespace_kind: NamespaceKindV2::Value,
                visibility: VisibilityV2::Public,
                location: checked(&sources, 1, at),
            })
            .unwrap();
    }
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: app.clone(),
            local_name: "x".into(),
            target: asset_ref(&sources, &left, NamespaceKindV2::Value, "x", 22),
            location: checked(&sources, 1, 23),
        })
        .unwrap();
    builder
        .try_reexport(ModuleReexportInputV2 {
            owner_module_id: app.clone(),
            exported_name: "x".into(),
            target: asset_ref(&sources, &right, NamespaceKindV2::Value, "x", 24),
            location: checked(&sources, 1, 25),
        })
        .unwrap();
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: consumer.clone(),
            local_name: "x".into(),
            target: asset_ref(&sources, &app, NamespaceKindV2::Value, "x", 26),
            location: checked(&sources, 1, 27),
        })
        .unwrap();

    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: consumer,
                location: checked(&sources, 1, 99),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let app_binding = parts
        .resolved_graph()
        .module_bindings(&app)
        .unwrap()
        .find(|binding| {
            binding.local_name() == "x" && binding.namespace_kind() == NamespaceKindV2::Value
        })
        .unwrap();
    assert_eq!(
        app_binding.declaration_origin().module_id().as_str(),
        "left"
    );
    let consumer_binding = parts
        .resolved_graph()
        .visible_bindings()
        .get(NamespaceKindV2::Value, "x")
        .unwrap();
    assert_eq!(
        consumer_binding.declaration_origin().module_id().as_str(),
        "right"
    );
}

#[test]
fn local_direct_import_does_not_suppress_downstream_reexport_ambiguity() {
    let sources = registry(&[(1, 4096)]);
    let app = module_id(&sources, "app", 0);
    let left = module_id(&sources, "left", 1);
    let right = module_id(&sources, "right", 2);
    let consumer = module_id(&sources, "consumer", 3);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for (id, at) in [(&app, 10), (&left, 11), (&right, 12), (&consumer, 13)] {
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: id.clone(),
                location: checked(&sources, 1, at),
            })
            .unwrap();
    }
    for (owner, at) in [(&left, 20), (&right, 21)] {
        builder
            .try_declare(ModuleDeclarationInputV2 {
                owner_module_id: owner.clone(),
                local_name: "x".into(),
                namespace_kind: NamespaceKindV2::Value,
                visibility: VisibilityV2::Public,
                location: checked(&sources, 1, at),
            })
            .unwrap();
        builder
            .try_reexport(ModuleReexportInputV2 {
                owner_module_id: app.clone(),
                exported_name: "x".into(),
                target: asset_ref(&sources, owner, NamespaceKindV2::Value, "x", at + 2),
                location: checked(&sources, 1, at + 4),
            })
            .unwrap();
    }
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: app.clone(),
            local_name: "x".into(),
            target: asset_ref(&sources, &left, NamespaceKindV2::Value, "x", 30),
            location: checked(&sources, 1, 31),
        })
        .unwrap();
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: consumer.clone(),
            local_name: "x".into(),
            target: asset_ref(&sources, &app, NamespaceKindV2::Value, "x", 32),
            location: checked(&sources, 1, 33),
        })
        .unwrap();

    let error = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: consumer,
                location: checked(&sources, 1, 99),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap_err();
    assert_eq!(error.code(), "modules.ambiguous_import");
    assert_eq!(
        error
            .ambiguous_candidates()
            .iter()
            .map(|candidate| candidate.origin().module_id().as_str())
            .collect::<Vec<_>>(),
        vec!["left", "right"]
    );
}

fn finish_declaration_reexport_collision(
    visibility: VisibilityV2,
) -> Result<ValidatedModuleAuthorityPartsV2, ModuleDiagnosticV2> {
    let sources = registry(&[(1, 4096)]);
    let app = module_id(&sources, "app", 0);
    let left = module_id(&sources, "left", 1);
    let consumer = module_id(&sources, "consumer", 2);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for (id, at) in [(&app, 10), (&left, 11), (&consumer, 12)] {
        builder.try_add_module(ModuleNodeInputV2 {
            module_id: id.clone(),
            location: checked(&sources, 1, at),
        })?;
    }
    builder.try_declare(ModuleDeclarationInputV2 {
        owner_module_id: app.clone(),
        local_name: "x".into(),
        namespace_kind: NamespaceKindV2::Value,
        visibility,
        location: checked(&sources, 1, 20),
    })?;
    builder.try_declare(ModuleDeclarationInputV2 {
        owner_module_id: left.clone(),
        local_name: "x".into(),
        namespace_kind: NamespaceKindV2::Value,
        visibility: VisibilityV2::Public,
        location: checked(&sources, 1, 21),
    })?;
    builder.try_reexport(ModuleReexportInputV2 {
        owner_module_id: app.clone(),
        exported_name: "x".into(),
        target: asset_ref(&sources, &left, NamespaceKindV2::Value, "x", 22),
        location: checked(&sources, 1, 23),
    })?;
    builder.try_import(ModuleImportInputV2 {
        owner_module_id: consumer.clone(),
        local_name: "x".into(),
        target: asset_ref(&sources, &app, NamespaceKindV2::Value, "x", 24),
        location: checked(&sources, 1, 25),
    })?;
    builder.finish(
        CurrentModuleSelectionInputV2 {
            module_id: consumer,
            location: checked(&sources, 1, 99),
        },
        VersionV2::new(1, 0, 0),
    )
}

#[test]
fn public_local_declaration_controls_downstream_name_before_reexport() {
    let parts = finish_declaration_reexport_collision(VisibilityV2::Public).unwrap();
    let binding = parts
        .resolved_graph()
        .visible_bindings()
        .get(NamespaceKindV2::Value, "x")
        .unwrap();
    assert_eq!(binding.declaration_origin().module_id().as_str(), "app");
}

#[test]
fn private_local_declaration_blocks_downstream_name_before_reexport() {
    let error = finish_declaration_reexport_collision(VisibilityV2::Private).unwrap_err();
    assert_eq!(error.code(), "modules.not_exported");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(25, 26));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(20, 21));
}

#[test]
fn private_declaration_is_not_importable() {
    let error = finish_graph_case(|sources, app, left, _, builder| {
        builder
            .try_declare(ModuleDeclarationInputV2 {
                owner_module_id: left.clone(),
                local_name: "x".into(),
                namespace_kind: NamespaceKindV2::Value,
                visibility: VisibilityV2::Private,
                location: checked(sources, 1, 20),
            })
            .unwrap();
        builder
            .try_import(ModuleImportInputV2 {
                owner_module_id: app.clone(),
                local_name: "x".into(),
                target: asset_ref(sources, left, NamespaceKindV2::Value, "x", 21),
                location: checked(sources, 1, 22),
            })
            .unwrap();
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.not_exported");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(22, 23));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(20, 21));
}

#[test]
fn absent_declaration_is_not_exported() {
    let error = finish_graph_case(|sources, app, left, _, builder| {
        builder
            .try_import(ModuleImportInputV2 {
                owner_module_id: app.clone(),
                local_name: "missing".into(),
                target: asset_ref(sources, left, NamespaceKindV2::Value, "missing", 21),
                location: checked(sources, 1, 22),
            })
            .unwrap();
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.not_exported");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(22, 23));
    assert!(error.secondary().is_none());
}

#[test]
fn duplicate_module_preserves_both_locations() {
    let sources = registry(&[(1, 128)]);
    let app = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 1),
        })
        .unwrap();
    let error = builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app,
            location: checked(&sources, 1, 2),
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.duplicate_module");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(2, 3));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(1, 2));
}

#[test]
fn duplicate_declaration_rejects_before_resolution() {
    let error = finish_graph_case(|sources, app, _, _, builder| {
        for at in [20, 21] {
            let result = builder.try_declare(ModuleDeclarationInputV2 {
                owner_module_id: app.clone(),
                local_name: "x".into(),
                namespace_kind: NamespaceKindV2::Value,
                visibility: VisibilityV2::Public,
                location: checked(sources, 1, at),
            });
            if at == 20 {
                result.unwrap();
            } else {
                assert_eq!(result.unwrap_err().code(), "modules.duplicate_declaration");
            }
        }
    });
    assert!(error.is_ok());
}

#[test]
fn duplicate_direct_import_rejects_even_when_target_matches() {
    let sources = registry(&[(1, 512)]);
    let app = module_id(&sources, "app", 0);
    let dep = module_id(&sources, "dep", 1);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for (id, at) in [(&app, 2), (&dep, 3)] {
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: id.clone(),
                location: checked(&sources, 1, at),
            })
            .unwrap();
    }
    for at in [4, 5] {
        let result = builder.try_import(ModuleImportInputV2 {
            owner_module_id: app.clone(),
            local_name: "x".into(),
            target: asset_ref(&sources, &dep, NamespaceKindV2::Value, "x", at + 10),
            location: checked(&sources, 1, at),
        });
        if at == 4 {
            result.unwrap();
        } else {
            assert_eq!(result.unwrap_err().code(), "modules.duplicate_import");
        }
    }
}

#[test]
fn duplicate_direct_import_rejects_different_target() {
    let sources = registry(&[(1, 128)]);
    let app = module_id(&sources, "app", 0);
    let left = module_id(&sources, "left", 1);
    let right = module_id(&sources, "right", 2);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for (id, at) in [(&app, 3), (&left, 4), (&right, 5)] {
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: id.clone(),
                location: checked(&sources, 1, at),
            })
            .unwrap();
    }
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: app.clone(),
            local_name: "x".into(),
            target: asset_ref(&sources, &left, NamespaceKindV2::Value, "x", 6),
            location: checked(&sources, 1, 7),
        })
        .unwrap();
    let error = builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: app,
            local_name: "x".into(),
            target: asset_ref(&sources, &right, NamespaceKindV2::Value, "x", 8),
            location: checked(&sources, 1, 9),
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.duplicate_import");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(9, 10));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(7, 8));
}

#[test]
fn identical_reexport_is_idempotent_and_keeps_earliest_edge() {
    let parts = finish_graph_case(|sources, app, left, _, builder| {
        builder
            .try_declare(ModuleDeclarationInputV2 {
                owner_module_id: left.clone(),
                local_name: "x".into(),
                namespace_kind: NamespaceKindV2::Value,
                visibility: VisibilityV2::Public,
                location: checked(sources, 1, 20),
            })
            .unwrap();
        for at in [40, 30] {
            builder
                .try_reexport(ModuleReexportInputV2 {
                    owner_module_id: app.clone(),
                    exported_name: "x".into(),
                    target: asset_ref(sources, left, NamespaceKindV2::Value, "x", at + 20),
                    location: checked(sources, 1, at),
                })
                .unwrap();
        }
    })
    .unwrap();
    let binding = parts
        .resolved_graph()
        .visible_bindings()
        .get(NamespaceKindV2::Value, "x")
        .unwrap();
    assert_eq!(binding.edge_witness().len(), 1);
    assert_eq!(
        binding.edge_witness().first().unwrap().location.span,
        SpanV1::new(30, 31)
    );
}

#[test]
fn self_import_cycle_rejects_with_one_edge_witness() {
    let sources = registry(&[(1, 128)]);
    let app = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 1),
        })
        .unwrap();
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: app.clone(),
            local_name: "x".into(),
            target: asset_ref(&sources, &app, NamespaceKindV2::Value, "x", 2),
            location: checked(&sources, 1, 3),
        })
        .unwrap();
    let error = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 4),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap_err();
    assert_eq!(error.code(), "modules.import_cycle");
    assert_eq!(
        error.edge_witness().to_vec(),
        [LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(3, 4),
        }]
    );
}

#[test]
fn nonempty_pack_graph_requires_lockfile() {
    let error = finish_pack_inputs(None, |sources| {
        vec![manifest_input(
            sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![],
            vec![],
            vec![],
            10,
        )]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.missing_lockfile");
    assert!(error.primary().is_none());
}

#[test]
fn empty_pack_graph_rejects_unexpected_lockfile() {
    let error = finish_pack_inputs(Some(""), |_| vec![]).unwrap_err();
    assert_eq!(error.code(), "modules.unexpected_lockfile");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(2, 3));
}

#[test]
fn builder_rejects_duplicate_lockfile_and_model_pack_at_ingress() {
    let sources = registry(&[(1, 128), (2, 0)]);
    let mut lock_builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for at in [1, 2] {
        let result = lock_builder.try_set_lockfile(
            LockReferenceInputV2 {
                path: "packs.lock".into(),
                location: checked(&sources, 1, at),
            },
            SourceIdV2::from_raw(2),
            "",
        );
        if at == 1 {
            result.unwrap();
        } else {
            let error = result.unwrap_err();
            assert_eq!(error.code(), "modules.duplicate_lockfile_input");
            assert_eq!(error.primary().unwrap().span, SpanV1::new(2, 3));
            assert_eq!(error.secondary().unwrap().span, SpanV1::new(1, 2));
        }
    }

    let pack = PackIdV2::try_new("model-pack", checked(&sources, 1, 3)).unwrap();
    let mut model_builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for at in [4, 5] {
        let result = model_builder.try_add_model_pack(ModelPackInputV2 {
            pack_id: pack.clone(),
            model_version: VersionV2::new(1, 0, 0),
            facts: vec![],
            location: checked(&sources, 1, at),
        });
        if at == 4 {
            result.unwrap();
        } else {
            let error = result.unwrap_err();
            assert_eq!(error.code(), "modules.duplicate_model_pack");
            assert_eq!(error.primary().unwrap().span, SpanV1::new(5, 6));
            assert_eq!(error.secondary().unwrap().span, SpanV1::new(4, 5));
        }
    }
}

#[test]
fn duplicate_pack_rejects_before_graph_validation() {
    let lock = format!("a 1.0.0 {}\n", "0".repeat(64));
    let error = finish_pack_inputs(Some(&lock), |sources| {
        vec![
            manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![],
                vec![],
                10,
            ),
            manifest_input(
                sources,
                "a",
                VersionV2::new(1, 1, 0),
                vec![],
                vec![],
                vec![],
                20,
            ),
        ]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.duplicate_pack");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(23, 24));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(13, 14));
}

#[test]
fn compatibility_mismatch_rejects() {
    let lock = format!("a 1.0.0 {}\n", "0".repeat(64));
    let error = finish_pack_inputs(Some(&lock), |sources| {
        let mut input = manifest_input(
            sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![],
            vec![],
            vec![],
            10,
        );
        input.language_range = VersionRangeV2::try_new(
            VersionV2::new(2, 0, 0),
            VersionV2::new(3, 0, 0),
            checked(sources, 1, 20),
        )
        .unwrap();
        vec![input]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.compat_mismatch");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(13, 14));
}

#[test]
fn unknown_pack_dependency_rejects_at_edge() {
    let lock = format!("a 1.0.0 {}\n", "0".repeat(64));
    let error = finish_pack_inputs(Some(&lock), |sources| {
        let missing = PackIdV2::try_new("missing", checked(sources, 1, 20)).unwrap();
        vec![manifest_input(
            sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![LocatedDependencyInputV2 {
                pack_id: missing,
                version_range: VersionRangeV2::try_new(
                    VersionV2::new(1, 0, 0),
                    VersionV2::new(2, 0, 0),
                    checked(sources, 1, 21),
                )
                .unwrap(),
                location: checked(sources, 1, 22),
            }],
            vec![],
            vec![],
            10,
        )]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.unknown_pack");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(22, 23));
    assert!(error.secondary().is_none());
}

#[test]
fn unsatisfied_pack_dependency_retains_target_location() {
    let lock = format!("a 1.0.0 {}\nb 2.0.0 {}\n", "0".repeat(64), "0".repeat(64));
    let error = finish_pack_inputs(Some(&lock), |sources| {
        let b = PackIdV2::try_new("b", checked(sources, 1, 30)).unwrap();
        vec![
            manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![LocatedDependencyInputV2 {
                    pack_id: b,
                    version_range: VersionRangeV2::try_new(
                        VersionV2::new(1, 0, 0),
                        VersionV2::new(2, 0, 0),
                        checked(sources, 1, 31),
                    )
                    .unwrap(),
                    location: checked(sources, 1, 32),
                }],
                vec![],
                vec![],
                10,
            ),
            manifest_input(
                sources,
                "b",
                VersionV2::new(2, 0, 0),
                vec![],
                vec![],
                vec![],
                40,
            ),
        ]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.dependency_unsatisfied");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(32, 33));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(43, 44));
}

#[test]
fn pack_self_dependency_cycle_rejects() {
    let lock = format!("a 1.0.0 {}\n", "0".repeat(64));
    let error = finish_pack_inputs(Some(&lock), |sources| {
        let a = PackIdV2::try_new("a", checked(sources, 1, 20)).unwrap();
        vec![manifest_input(
            sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![LocatedDependencyInputV2 {
                pack_id: a,
                version_range: VersionRangeV2::try_new(
                    VersionV2::new(1, 0, 0),
                    VersionV2::new(2, 0, 0),
                    checked(sources, 1, 21),
                )
                .unwrap(),
                location: checked(sources, 1, 22),
            }],
            vec![],
            vec![],
            10,
        )]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.pack_dependency_cycle");
    assert_eq!(
        error.edge_witness().to_vec(),
        [LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(22, 23),
        }]
    );
}

#[test]
fn overlapping_pack_export_rejects_with_both_locations() {
    let lock = format!("a 1.0.0 {}\nb 1.0.0 {}\n", "0".repeat(64), "0".repeat(64));
    let error = finish_pack_inputs(Some(&lock), |sources| {
        vec![
            manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![LocatedExportInputV2 {
                    name: "scan".into(),
                    location: checked(sources, 1, 20),
                }],
                vec![],
                10,
            ),
            manifest_input(
                sources,
                "b",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![LocatedExportInputV2 {
                    name: "scan".into(),
                    location: checked(sources, 1, 30),
                }],
                vec![],
                40,
            ),
        ]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.overlapping_pack_export");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(30, 31));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(20, 21));
}

#[test]
fn duplicate_dependency_rejects_even_when_ranges_match() {
    let lock = format!("a 1.0.0 {}\nb 1.0.0 {}\n", "0".repeat(64), "0".repeat(64));
    let error = finish_pack_inputs(Some(&lock), |sources| {
        let b = PackIdV2::try_new("b", checked(sources, 1, 20)).unwrap();
        let dep = |at| LocatedDependencyInputV2 {
            pack_id: b.clone(),
            version_range: VersionRangeV2::try_new(
                VersionV2::new(1, 0, 0),
                VersionV2::new(2, 0, 0),
                checked(sources, 1, at + 1),
            )
            .unwrap(),
            location: checked(sources, 1, at),
        };
        vec![
            manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![dep(30), dep(40)],
                vec![],
                vec![],
                10,
            ),
            manifest_input(
                sources,
                "b",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![],
                vec![],
                50,
            ),
        ]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.duplicate_pack_dependency");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(40, 41));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(30, 31));
}

#[test]
fn duplicate_dependency_rejects_different_ranges() {
    let lock = format!("a 1.0.0 {}\nb 1.0.0 {}\n", "0".repeat(64), "0".repeat(64));
    let error = finish_pack_inputs(Some(&lock), |sources| {
        let b = PackIdV2::try_new("b", checked(sources, 1, 20)).unwrap();
        let dep = |upper, at| LocatedDependencyInputV2 {
            pack_id: b.clone(),
            version_range: VersionRangeV2::try_new(
                VersionV2::new(1, 0, 0),
                upper,
                checked(sources, 1, at + 1),
            )
            .unwrap(),
            location: checked(sources, 1, at),
        };
        vec![
            manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![
                    dep(VersionV2::new(2, 0, 0), 30),
                    dep(VersionV2::new(3, 0, 0), 40),
                ],
                vec![],
                vec![],
                10,
            ),
            manifest_input(
                sources,
                "b",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![],
                vec![],
                50,
            ),
        ]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.duplicate_pack_dependency");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(40, 41));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(30, 31));
}

#[test]
fn duplicate_suite_declaration_rejects() {
    let lock = format!("a 1.0.0 {}\n", "0".repeat(64));
    let error = finish_pack_inputs(Some(&lock), |sources| {
        vec![manifest_input(
            sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![],
            vec![],
            vec![
                LocatedSuiteInputV2 {
                    name: "core".into(),
                    members: vec![],
                    location: checked(sources, 1, 20),
                },
                LocatedSuiteInputV2 {
                    name: "core".into(),
                    members: vec![],
                    location: checked(sources, 1, 21),
                },
            ],
            10,
        )]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.duplicate_suite");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(21, 22));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(20, 21));
}

#[test]
fn duplicate_export_and_suite_member_are_semantically_idempotent() {
    let lock = format!("a 1.0.0 {}\n", "0".repeat(64));
    let build = |duplicate: bool| {
        finish_pack_inputs(Some(&lock), |sources| {
            let mut exports = vec![LocatedExportInputV2 {
                name: "scan".into(),
                location: checked(sources, 1, 20),
            }];
            let mut members = vec![LocatedSuiteMemberInputV2 {
                name: "scan".into(),
                location: checked(sources, 1, 21),
            }];
            if duplicate {
                exports.push(LocatedExportInputV2 {
                    name: "scan".into(),
                    location: checked(sources, 1, 22),
                });
                members.push(LocatedSuiteMemberInputV2 {
                    name: "scan".into(),
                    location: checked(sources, 1, 23),
                });
            }
            vec![manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![],
                exports,
                vec![LocatedSuiteInputV2 {
                    name: "core".into(),
                    members,
                    location: checked(sources, 1, 24),
                }],
                10,
            )]
        })
        .unwrap()
    };
    assert_eq!(
        build(false).pack_authority().graph_fingerprint(),
        build(true).pack_authority().graph_fingerprint()
    );
}

#[test]
fn duplicate_lock_pin_rejects() {
    let zeros = "0".repeat(64);
    let lock = format!("a 1.0.0 {zeros}\na 1.0.0 {zeros}\n");
    let error = finish_pack_inputs(Some(&lock), |sources| {
        vec![manifest_input(
            sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![],
            vec![],
            vec![],
            10,
        )]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.duplicate_lock_pin");
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(2),
            span: SpanV1::new(73, 145),
        }
    );
    assert_eq!(
        error.secondary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(2),
            span: SpanV1::new(0, 72),
        }
    );
}

#[test]
fn canonical_lock_roundtrip_is_byte_exact() {
    let zeros = "0".repeat(64);
    let lock = format!("a 1.0.0 {zeros}\nb 1.2.0 {zeros}\n");
    let parts = finish_pack_inputs(Some(&lock), |sources| {
        vec![
            manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![],
                vec![],
                10,
            ),
            manifest_input(
                sources,
                "b",
                VersionV2::new(1, 2, 0),
                vec![],
                vec![],
                vec![],
                20,
            ),
        ]
    })
    .unwrap();
    assert_eq!(parts.pack_authority().verified_lock().to_text(), lock);
}

#[test]
fn lock_blank_line_is_rejected_with_that_line_as_primary() {
    let zeros = "0".repeat(64);
    let first = format!("a 1.0.0 {zeros}\n");
    let lock = format!("{first}\n");
    let error = finish_pack_inputs(Some(&lock), |sources| {
        vec![manifest_input(
            sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![],
            vec![],
            vec![],
            10,
        )]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.invalid_lock_syntax");
    assert_eq!(
        error.primary().unwrap().span,
        SpanV1::new(first.len(), first.len())
    );
}

#[test]
fn lock_version_drift_is_rejected_exactly() {
    let lock = format!("a 1.0.1 {}\n", "0".repeat(64));
    let error = finish_pack_inputs(Some(&lock), |sources| {
        vec![manifest_input(
            sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![],
            vec![],
            vec![],
            10,
        )]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.lock_mismatch");
    assert_eq!(
        error.primary().unwrap().span,
        SpanV1::new(0, lock.len() - 1)
    );
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(13, 14));
}

#[test]
fn lock_missing_and_stale_pins_are_both_rejected() {
    let zeros = "0".repeat(64);
    let missing = format!("a 1.0.0 {zeros}\n");
    let missing_error = finish_pack_inputs(Some(&missing), |sources| {
        vec![
            manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![],
                vec![],
                10,
            ),
            manifest_input(
                sources,
                "b",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![],
                vec![],
                20,
            ),
        ]
    })
    .unwrap_err();
    assert_eq!(missing_error.code(), "modules.lock_mismatch");
    assert_eq!(
        missing_error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(23, 24),
        }
    );
    assert!(missing_error.secondary().is_none());

    let stale = format!("a 1.0.0 {zeros}\nb 1.0.0 {zeros}\n");
    let stale_error = finish_pack_inputs(Some(&stale), |sources| {
        vec![manifest_input(
            sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![],
            vec![],
            vec![],
            10,
        )]
    })
    .unwrap_err();
    assert_eq!(stale_error.code(), "modules.lock_mismatch");
    assert_eq!(
        stale_error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(2),
            span: SpanV1::new(73, 145),
        }
    );
    assert!(stale_error.secondary().is_none());
}

#[test]
fn lock_without_final_newline_rejects_canonicality() {
    let lock = format!("a 1.0.0 {}", "0".repeat(64));
    let expected_eof = lock.len();
    let error = finish_pack_inputs(Some(&lock), |sources| {
        vec![manifest_input(
            sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![],
            vec![],
            vec![],
            10,
        )]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.invalid_lock_syntax");
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(2),
            span: SpanV1::new(expected_eof, expected_eof),
        }
    );
}

#[test]
fn lock_reference_path_reject_matrix_is_closed() {
    let invalid = [
        "",
        "packs/security",
        "/packs/security.lock",
        "packs\\security.lock",
        "packs/../security.lock",
        "./security.lock",
        "packs/./security.lock",
        "packs//security.lock",
        "packs/security.lock\0suffix.lock",
    ];
    for path in invalid {
        let sources = registry(&[(1, 64), (2, 0)]);
        let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
        let error = builder
            .try_set_lockfile(
                LockReferenceInputV2 {
                    path: path.to_owned(),
                    location: checked(&sources, 1, 4),
                },
                SourceIdV2::from_raw(2),
                "",
            )
            .unwrap_err();
        assert_eq!(error.code(), "modules.invalid_lockfile_path", "{path:?}");
        assert_eq!(error.primary().unwrap().span, SpanV1::new(4, 5));
    }

    let sources = registry(&[(1, 64), (2, 0)]);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_set_lockfile(
            LockReferenceInputV2 {
                path: "packs/nested/security.lock".into(),
                location: checked(&sources, 1, 5),
            },
            SourceIdV2::from_raw(2),
            "",
        )
        .unwrap();
}

#[test]
fn lock_line_grammar_rejects_every_noncanonical_separator_and_field_shape() {
    let zeros = "0".repeat(64);
    let cases = [
        format!("a 1.0.0 {zeros}\r\n"),
        format!("a\t1.0.0\t{zeros}\n"),
        format!(" a 1.0.0 {zeros}\n"),
        format!("a 1.0.0 {zeros} \n"),
        format!("a 1.0.0  {zeros}\n"),
        "a 1.0.0\n".to_owned(),
        "a\n".to_owned(),
        format!("a{}1.0.0 {zeros}\n", " ".repeat(131_072)),
        format!("a extra 1.0.0 {zeros}\n"),
    ];
    for lock in cases {
        let error = finish_pack_inputs(Some(&lock), |sources| {
            vec![manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![],
                vec![],
                10,
            )]
        })
        .unwrap_err();
        assert_eq!(error.code(), "modules.invalid_lock_syntax", "{lock:?}");
        assert_eq!(
            error.primary(),
            Some(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(2),
                span: SpanV1::new(0, lock.len() - 1),
            })
        );
        assert!(error.secondary().is_none());
    }
}

#[test]
fn lock_source_length_mismatch_has_checked_source_span() {
    let zeros = "0".repeat(64);
    let lock = format!("a 1.0.0 {zeros}\n");
    let registered_len = lock.len() + 1;
    let sources = registry(&[(1, 256), (2, registered_len)]);
    let app = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 1),
        })
        .unwrap();
    builder
        .try_add_manifest(manifest_input(
            &sources,
            "a",
            VersionV2::new(1, 0, 0),
            vec![],
            vec![],
            vec![],
            10,
        ))
        .unwrap();
    builder
        .try_set_lockfile(
            LockReferenceInputV2 {
                path: "packs.lock".into(),
                location: checked(&sources, 1, 2),
            },
            SourceIdV2::from_raw(2),
            &lock,
        )
        .unwrap();
    let error = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 3),
            },
            VersionV2::new(1, 1, 0),
        )
        .unwrap_err();
    assert_eq!(error.code(), "modules.lock_source_length_mismatch");
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(2),
            span: SpanV1::new(0, registered_len),
        }
    );
}

#[test]
fn noncanonical_lock_order_points_at_first_out_of_order_line() {
    let zeros = "0".repeat(64);
    let first_line = format!("b 1.0.0 {zeros}\n");
    let lock = format!("{first_line}a 1.0.0 {zeros}\n");
    let error = finish_pack_inputs(Some(&lock), |sources| {
        vec![
            manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![],
                vec![],
                10,
            ),
            manifest_input(
                sources,
                "b",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![],
                vec![],
                20,
            ),
        ]
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.invalid_lock_syntax");
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(2),
            span: SpanV1::new(0, first_line.len() - 1),
        }
    );
}

#[test]
fn lock_order_all_permutations_preserve_first_mismatch_and_canonical_bytes() {
    // Literal byte ordering, independent of the lock serializer and PackId Ord.
    let canonical = ["A", "a", "a-b", "a.b"];
    let digest = "0123456789abcdef".repeat(4);
    let mut checked_permutations = 0;
    for encoded in 0..256usize {
        let indices = [
            encoded & 3,
            (encoded >> 2) & 3,
            (encoded >> 4) & 3,
            encoded >> 6,
        ];
        if indices
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != 4
        {
            continue;
        }
        checked_permutations += 1;
        let names = indices.map(|index| canonical[index]);
        let lines = names.map(|name| format!("{name} 1.0.0 {digest}\n"));
        let lock = lines.concat();
        let result = finish_pack_inputs(Some(&lock), |sources| {
            canonical
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    let mut input = manifest_input(
                        sources,
                        name,
                        VersionV2::new(1, 0, 0),
                        vec![],
                        vec![],
                        vec![],
                        10 + 10 * index,
                    );
                    input.content_digest =
                        ContentDigestV2::try_parse(&digest, checked(sources, 1, 100)).unwrap();
                    input
                })
                .collect()
        });
        if let Some(first) = names
            .iter()
            .zip(canonical)
            .position(|(actual, expected)| *actual != expected)
        {
            let error = result.unwrap_err();
            assert_eq!(error.code(), "modules.invalid_lock_syntax");
            let start: usize = lines.iter().take(first).map(String::len).sum();
            assert_eq!(
                error.primary(),
                Some(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(2),
                    span: SpanV1::new(start, start + lines[first].len() - 1),
                })
            );
            assert!(error.secondary().is_none());
        } else {
            assert_eq!(
                result.unwrap().pack_authority().verified_lock().to_text(),
                lock
            );
        }
    }
    assert_eq!(checked_permutations, 24);
}

#[test]
fn lock_later_field_and_duplicate_errors_precede_unsorted_order() {
    let digest = "0".repeat(64);
    let prefix = format!("b 1.0.0 {digest}\na 1.0.0 {digest}\n");
    for (tail, code) in [
        (format!("c 01.0.0 {digest}\n"), "modules.invalid_version"),
        (
            format!("c 18446744073709551616.0.0 {digest}\n"),
            "modules.invalid_version",
        ),
        (
            format!("c 1.0.0 {}\n", "A".repeat(64)),
            "modules.invalid_content_digest",
        ),
        (
            format!("c 1.0.0 {digest} extra\n"),
            "modules.invalid_lock_syntax",
        ),
        (format!("a 1.0.0 {digest}\n"), "modules.duplicate_lock_pin"),
    ] {
        let lock = format!("{prefix}{tail}");
        let error = finish_pack_inputs(Some(&lock), |sources| {
            ["a", "b"]
                .iter()
                .enumerate()
                .map(|(index, name)| {
                    manifest_input(
                        sources,
                        name,
                        VersionV2::new(1, 0, 0),
                        vec![],
                        vec![],
                        vec![],
                        10 + 10 * index,
                    )
                })
                .collect()
        })
        .unwrap_err();
        assert_eq!(error.code(), code);
        assert_eq!(
            error.primary(),
            Some(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(2),
                span: SpanV1::new(prefix.len(), lock.len() - 1),
            })
        );
        if code == "modules.duplicate_lock_pin" {
            let first_line_len = prefix.len() / 2;
            assert_eq!(
                error.secondary(),
                Some(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(2),
                    span: SpanV1::new(first_line_len, prefix.len() - 1),
                })
            );
        } else {
            assert!(error.secondary().is_none());
        }
    }
}

#[test]
fn pack_fingerprint_is_permutation_stable() {
    let lock = format!("a 1.0.0 {}\nb 1.0.0 {}\n", "0".repeat(64), "0".repeat(64));
    let build = |reverse| {
        finish_pack_inputs(Some(&lock), |sources| {
            let a = manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![LocatedExportInputV2 {
                    name: "a_export".into(),
                    location: checked(sources, 1, 20),
                }],
                vec![],
                10,
            );
            let b = manifest_input(
                sources,
                "b",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![LocatedExportInputV2 {
                    name: "b_export".into(),
                    location: checked(sources, 1, 30),
                }],
                vec![],
                40,
            );
            if reverse {
                vec![b, a]
            } else {
                vec![a, b]
            }
        })
        .unwrap()
    };
    assert_eq!(
        build(false).pack_authority().graph_fingerprint(),
        build(true).pack_authority().graph_fingerprint()
    );
}

#[test]
fn external_catalog_unknown_kind_token_rejects() {
    let sources = registry(&[(1, 256)]);
    let app = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 1),
        })
        .unwrap();
    let error = builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "host".into(),
            local_name: "x".into(),
            kind_token: "unknown".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Value(ValueTypeV1::int()),
            provenance: "host".into(),
            location: checked(&sources, 1, 2),
            dependencies: vec![],
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.unknown_catalog_kind");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(2, 3));
}

#[test]
fn duplicate_signature_parameter_rejects_with_both_locations() {
    let sources = registry(&[(1, 256)]);
    let app = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 1),
        })
        .unwrap();
    let error = builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "host".into(),
            local_name: "pred".into(),
            kind_token: "predicate".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Predicate(SignatureInputV2 {
                parameters: vec![
                    LocatedParameterInputV2 {
                        name: "x".into(),
                        value_type: ValueTypeV1::int(),
                        location: checked(&sources, 1, 2),
                    },
                    LocatedParameterInputV2 {
                        name: "x".into(),
                        value_type: ValueTypeV1::string(),
                        location: checked(&sources, 1, 3),
                    },
                ],
                result: ValueTypeV1::bool(),
                callable_kind: CallableKindV2::Predicate,
                effects: ExpressionEffectsV2::pure_scalar(),
                location: checked(&sources, 1, 4),
            }),
            provenance: "host".into(),
            location: checked(&sources, 1, 4),
            dependencies: vec![],
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.duplicate_signature_parameter");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(3, 4));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(2, 3));
}

#[test]
fn signature_parameter_names_use_the_closed_identifier_grammar() {
    for name in ["", "a.b", "a/b", "1arg", "arg name", "δ"] {
        let sources = registry(&[(1, 256)]);
        let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
        let error = builder
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: "host".into(),
                local_name: "pred".into(),
                kind_token: "predicate".into(),
                typed_payload: ExternalCatalogPayloadInputV2::Predicate(SignatureInputV2 {
                    parameters: vec![LocatedParameterInputV2 {
                        name: name.into(),
                        value_type: ValueTypeV1::int(),
                        location: checked(&sources, 1, 12),
                    }],
                    result: ValueTypeV1::bool(),
                    callable_kind: CallableKindV2::Predicate,
                    effects: ExpressionEffectsV2::pure_scalar(),
                    location: checked(&sources, 1, 11),
                }),
                provenance: "host".into(),
                location: checked(&sources, 1, 10),
                dependencies: vec![],
            })
            .unwrap_err();
        assert_eq!(
            error.code(),
            "modules.invalid_signature_parameter",
            "{name:?}"
        );
        assert_eq!(error.primary().unwrap().span, SpanV1::new(12, 13));
    }
}

#[test]
fn public_signature_projection_preserves_every_callable_fact() {
    // Every axis differs from pure_scalar; this is authored metadata, not a
    // default that either signature construction or projection may recreate.
    let effects = ExpressionEffectsV2 {
        purity: PurityV1::Impure,
        determinism: DeterminismV1::NonDeterministic,
        monotonicity: MonotonicityV1::Antitone,
        aggregation: AggregationClassV1::Aggregate,
        cost: CostClassV1::Linear,
    };
    let sources = registry(&[(1, 256)]);
    let app = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 2),
        })
        .unwrap();
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "host".into(),
            local_name: "pred".into(),
            kind_token: "predicate".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Predicate(SignatureInputV2 {
                parameters: vec![
                    LocatedParameterInputV2 {
                        name: "left-hand".into(),
                        value_type: ValueTypeV1::int(),
                        location: checked(&sources, 1, 3),
                    },
                    LocatedParameterInputV2 {
                        name: "_right2".into(),
                        value_type: ValueTypeV1::string(),
                        location: checked(&sources, 1, 4),
                    },
                ],
                result: ValueTypeV1::bool(),
                callable_kind: CallableKindV2::Predicate,
                effects,
                location: checked(&sources, 1, 5),
            }),
            provenance: "host".into(),
            location: checked(&sources, 1, 6),
            dependencies: vec![],
        })
        .unwrap();
    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 7),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let asset = parts.external_catalog().iter().next().unwrap();
    assert!(matches!(
        asset.payload(),
        ExternalCatalogPayloadV2::Predicate(_)
    ));
    let ExternalCatalogPayloadV2::Predicate(signature) = asset.payload() else {
        return;
    };
    assert_eq!(signature.effects(), effects);
    assert_eq!(
        signature
            .parameters()
            .iter()
            .map(SignatureParameterV2::name)
            .collect::<Vec<_>>(),
        ["left-hand", "_right2"]
    );
    let predicate = ResolvedPredicateIdV2::try_new("projected").unwrap();
    let projected = signature
        .try_as_resolved_predicate(predicate.clone())
        .unwrap();
    assert_eq!(projected.id(), &predicate);
    assert_eq!(
        projected.params(),
        &[ValueTypeV1::int(), ValueTypeV1::string()]
    );
    assert_eq!(projected.result(), &ValueTypeV1::bool());
    assert_eq!(projected.effects(), effects);
}
