use super::*;

#[test]
fn canonical_ids_versions_and_digests_reject_alias_spellings() {
    let sources = registry(&[(1, 1024)]);
    for bad in ["", "a::::b", "a/b", "-a", "é"] {
        assert_eq!(
            ModuleIdV2::try_new(bad, checked(&sources, 1, 0))
                .unwrap_err()
                .code(),
            "modules.invalid_module_id"
        );
    }
    for good in ["secure-pack", "my.pack", "pack_1"] {
        assert_eq!(
            PackIdV2::try_new(good, checked(&sources, 1, 1))
                .unwrap()
                .as_str(),
            good
        );
    }
    for bad in ["-pack", "pack..x", "a::b", "A/pack"] {
        assert_eq!(
            PackIdV2::try_new(bad, checked(&sources, 1, 2))
                .unwrap_err()
                .code(),
            "modules.invalid_pack_id"
        );
    }
    assert_eq!(
        VersionV2::try_parse("01.0.0", checked(&sources, 1, 3))
            .unwrap_err()
            .code(),
        "modules.invalid_version"
    );
    assert_eq!(
        ContentDigestV2::try_parse(&"A".repeat(64), checked(&sources, 1, 4))
            .unwrap_err()
            .code(),
        "modules.invalid_content_digest"
    );
}

#[test]
fn one_session_budget_charges_duplicate_attempt_before_dedup() {
    let sources = registry(&[(1, 1024)]);
    let budget = AuthorityBudgetProfileV2::try_new(1, 8, 32, 1024, 128).unwrap();
    let id = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(budget, sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: id.clone(),
            location: checked(&sources, 1, 1),
        })
        .unwrap();
    let error = builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: id,
            location: checked(&sources, 1, 2),
        })
        .unwrap_err();
    assert_resource(&error, "nodes", 1, 2);
}

#[test]
fn checked_locations_cannot_cross_registry_identity() {
    let host = registry(&[(1, 8)]);
    let substitute = registry(&[(1, 9)]);
    let id = module_id(&host, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), host).unwrap();
    let error = builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: id,
            location: checked(&substitute, 1, 0),
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.source_registry_mismatch");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(0, 1));
    assert_eq!(
        substitute
            .try_check_location(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(1),
                span: SpanV1::new(0, 10),
            })
            .unwrap_err()
            .code(),
        "hir.source_registry.location_out_of_bounds"
    );
}

#[test]
fn explicit_module_current_selection_and_typed_namespaces_resolve() {
    let sources = registry(&[(1, 1024)]);
    let app = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 1),
        })
        .unwrap();
    for (kind, at) in [(NamespaceKindV2::Value, 2), (NamespaceKindV2::Type, 3)] {
        builder
            .try_declare(ModuleDeclarationInputV2 {
                owner_module_id: app.clone(),
                local_name: "same".to_owned(),
                namespace_kind: kind,
                visibility: VisibilityV2::Public,
                location: checked(&sources, 1, at),
            })
            .unwrap();
    }
    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 4),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let value_binding = parts
        .resolved_graph()
        .visible_bindings()
        .get(NamespaceKindV2::Value, "same")
        .unwrap();
    assert_eq!(value_binding.namespace_kind(), NamespaceKindV2::Value);
    assert_eq!(value_binding.local_name(), "same");
    assert_eq!(
        value_binding.declaration_origin().module_id().as_str(),
        "app"
    );
    let type_binding = parts
        .resolved_graph()
        .visible_bindings()
        .get(NamespaceKindV2::Type, "same")
        .unwrap();
    assert_eq!(type_binding.namespace_kind(), NamespaceKindV2::Type);
    assert_eq!(type_binding.local_name(), "same");
    assert_eq!(
        type_binding.declaration_origin().module_id().as_str(),
        "app"
    );
}

#[test]
fn unknown_current_module_uses_configuration_location() {
    let sources = registry(&[(1, 1024)]);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    let app = module_id(&sources, "app", 0);
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app,
            location: checked(&sources, 1, 1),
        })
        .unwrap();
    let error = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: module_id(&sources, "missing", 2),
                location: checked(&sources, 1, 77),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap_err();
    assert_eq!(error.code(), "modules.unknown_module");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(77, 78));
}

#[test]
fn disconnected_cycle_rejects_whole_graph_with_located_witness() {
    let sources = registry(&[(1, 1024)]);
    let app = module_id(&sources, "app", 0);
    let a = module_id(&sources, "a", 1);
    let b = module_id(&sources, "b", 2);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for (id, at) in [(&app, 10), (&a, 11), (&b, 12)] {
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: id.clone(),
                location: checked(&sources, 1, at),
            })
            .unwrap();
    }
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: a.clone(),
            local_name: "x".to_owned(),
            target: asset_ref(&sources, &b, NamespaceKindV2::Value, "x", 20),
            location: checked(&sources, 1, 21),
        })
        .unwrap();
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: b,
            local_name: "x".to_owned(),
            target: asset_ref(&sources, &a, NamespaceKindV2::Value, "x", 22),
            location: checked(&sources, 1, 23),
        })
        .unwrap();
    let error = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 30),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap_err();
    assert_eq!(error.code(), "modules.import_cycle");
    assert_eq!(
        error.edge_witness().to_vec(),
        [
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(1),
                span: SpanV1::new(21, 22),
            },
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(1),
                span: SpanV1::new(23, 24),
            },
        ]
    );
}

#[test]
fn competing_cycles_choose_same_canonical_witness_under_permutation() {
    fn run(reverse: bool) -> Vec<LocatedSpanV2> {
        let sources = registry(&[(1, 128)]);
        let app = module_id(&sources, "app", 0);
        let a = module_id(&sources, "a", 1);
        let b = module_id(&sources, "b", 2);
        let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
        for (id, at) in [(&app, 3), (&a, 4), (&b, 5)] {
            builder
                .try_add_module(ModuleNodeInputV2 {
                    module_id: id.clone(),
                    location: checked(&sources, 1, at),
                })
                .unwrap();
        }
        let mut edges = vec![(a, 20), (b, 10)];
        if reverse {
            edges.reverse();
        }
        for (owner, at) in edges {
            builder
                .try_import(ModuleImportInputV2 {
                    owner_module_id: owner.clone(),
                    local_name: "x".into(),
                    target: asset_ref(&sources, &owner, NamespaceKindV2::Value, "x", at + 30),
                    location: checked(&sources, 1, at),
                })
                .unwrap();
        }
        builder
            .finish(
                CurrentModuleSelectionInputV2 {
                    module_id: app,
                    location: checked(&sources, 1, 6),
                },
                VersionV2::new(1, 0, 0),
            )
            .unwrap_err()
            .edge_witness()
            .to_vec()
    }
    let forward = run(false);
    let reverse = run(true);
    assert_eq!(forward, reverse);
    assert_eq!(forward.len(), 1);
    // Module identity is the primary sort key, so cycle `a` wins over lower-span `b`.
    assert_eq!(forward.first().unwrap().span, SpanV1::new(20, 21));
}

#[test]
fn three_origin_ambiguity_retains_all_candidates_independent_of_input_order() {
    fn run(order: &[usize]) -> Vec<String> {
        let sources = registry(&[(1, 2048)]);
        let importer = module_id(&sources, "importer", 0);
        let origins = [
            module_id(&sources, "a", 1),
            module_id(&sources, "b", 2),
            module_id(&sources, "c", 3),
        ];
        let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: importer.clone(),
                location: checked(&sources, 1, 10),
            })
            .unwrap();
        for (index, origin) in origins.iter().enumerate() {
            builder
                .try_add_module(ModuleNodeInputV2 {
                    module_id: origin.clone(),
                    location: checked(&sources, 1, 20 + index),
                })
                .unwrap();
            builder
                .try_declare(ModuleDeclarationInputV2 {
                    owner_module_id: origin.clone(),
                    local_name: "x".to_owned(),
                    namespace_kind: NamespaceKindV2::Value,
                    visibility: VisibilityV2::Public,
                    location: checked(&sources, 1, 30 + index),
                })
                .unwrap();
        }
        for index in order {
            builder
                .try_reexport(ModuleReexportInputV2 {
                    owner_module_id: importer.clone(),
                    exported_name: "x".to_owned(),
                    target: asset_ref(
                        &sources,
                        origins.get(*index).unwrap(),
                        NamespaceKindV2::Value,
                        "x",
                        40 + *index,
                    ),
                    location: checked(&sources, 1, 50 + *index),
                })
                .unwrap();
        }
        let error = builder
            .finish(
                CurrentModuleSelectionInputV2 {
                    module_id: importer,
                    location: checked(&sources, 1, 60),
                },
                VersionV2::new(1, 0, 0),
            )
            .unwrap_err();
        assert_eq!(error.code(), "modules.ambiguous_import");
        error
            .ambiguous_candidates()
            .iter()
            .map(|candidate| candidate.origin().module_id().as_str().to_owned())
            .collect()
    }
    assert_eq!(run(&[0, 1, 2]), vec!["a", "b", "c"]);
    assert_eq!(run(&[2, 0, 1]), vec!["a", "b", "c"]);
}

#[test]
fn ambiguity_primary_tracks_first_origin_not_first_target_edge() {
    let sources = registry(&[(1, 512)]);
    let app = module_id(&sources, "app", 0);
    let a = module_id(&sources, "a", 1);
    let b = module_id(&sources, "b", 2);
    let y = module_id(&sources, "y", 3);
    let z = module_id(&sources, "z", 4);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for (id, at) in [(&app, 10), (&a, 11), (&b, 12), (&y, 13), (&z, 14)] {
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: id.clone(),
                location: checked(&sources, 1, at),
            })
            .unwrap();
    }
    for (owner, at) in [(&a, 20), (&b, 21)] {
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
    // Target order is y,z while origin order is a,b: y resolves to b and z to a.
    builder
        .try_reexport(ModuleReexportInputV2 {
            owner_module_id: y.clone(),
            exported_name: "x".into(),
            target: asset_ref(&sources, &b, NamespaceKindV2::Value, "x", 30),
            location: checked(&sources, 1, 70),
        })
        .unwrap();
    builder
        .try_reexport(ModuleReexportInputV2 {
            owner_module_id: z.clone(),
            exported_name: "x".into(),
            target: asset_ref(&sources, &a, NamespaceKindV2::Value, "x", 31),
            location: checked(&sources, 1, 80),
        })
        .unwrap();
    for (target, at) in [(&y, 40), (&z, 41)] {
        builder
            .try_reexport(ModuleReexportInputV2 {
                owner_module_id: app.clone(),
                exported_name: "x".into(),
                target: asset_ref(&sources, target, NamespaceKindV2::Value, "x", at),
                location: checked(&sources, 1, at + 10),
            })
            .unwrap();
    }
    let error = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 90),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap_err();
    assert_eq!(error.code(), "modules.ambiguous_import");
    assert_eq!(
        error
            .ambiguous_candidates()
            .first()
            .unwrap()
            .origin()
            .module_id()
            .as_str(),
        "a"
    );
    assert_eq!(error.primary().unwrap().span, SpanV1::new(80, 81));
}

#[test]
fn acyclic_diamond_collapses_identical_origin() {
    let sources = registry(&[(1, 512)]);
    let app = module_id(&sources, "app", 0);
    let left = module_id(&sources, "left", 1);
    let right = module_id(&sources, "right", 2);
    let origin = module_id(&sources, "origin", 3);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for (id, at) in [(&app, 10), (&left, 11), (&right, 12), (&origin, 13)] {
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: id.clone(),
                location: checked(&sources, 1, at),
            })
            .unwrap();
    }
    builder
        .try_declare(ModuleDeclarationInputV2 {
            owner_module_id: origin.clone(),
            local_name: "x".into(),
            namespace_kind: NamespaceKindV2::Value,
            visibility: VisibilityV2::Public,
            location: checked(&sources, 1, 20),
        })
        .unwrap();
    for (owner, at) in [(&left, 30), (&right, 31)] {
        builder
            .try_reexport(ModuleReexportInputV2 {
                owner_module_id: owner.clone(),
                exported_name: "x".into(),
                target: asset_ref(&sources, &origin, NamespaceKindV2::Value, "x", at),
                location: checked(&sources, 1, at + 10),
            })
            .unwrap();
    }
    for (target, at) in [(&left, 50), (&right, 51)] {
        builder
            .try_reexport(ModuleReexportInputV2 {
                owner_module_id: app.clone(),
                exported_name: "x".into(),
                target: asset_ref(&sources, target, NamespaceKindV2::Value, "x", at),
                location: checked(&sources, 1, at + 10),
            })
            .unwrap();
    }
    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 70),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    assert_eq!(
        parts
            .resolved_graph()
            .visible_bindings()
            .get(NamespaceKindV2::Value, "x")
            .unwrap()
            .declaration_origin()
            .module_id()
            .as_str(),
        "origin"
    );
}

#[test]
fn external_catalog_validates_payload_and_dependency_kinds() {
    let sources = registry(&[(1, 2048)]);
    let app = module_id(&sources, "app", 0);
    let ext = module_id(&sources, "ext", 1);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 2),
        })
        .unwrap();
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "ext".to_owned(),
            local_name: "base".to_owned(),
            kind_token: "value".to_owned(),
            typed_payload: ExternalCatalogPayloadInputV2::Value(ValueTypeV1::int()),
            provenance: "host".to_owned(),
            location: checked(&sources, 1, 3),
            dependencies: vec![],
        })
        .unwrap();
    let row = RecordTypeV1::try_new(vec![]).unwrap();
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "ext".to_owned(),
            local_name: "view".to_owned(),
            kind_token: "view".to_owned(),
            typed_payload: ExternalCatalogPayloadInputV2::View(
                RelationTypeV1::try_new(row, CardinalityV1::Set).unwrap(),
            ),
            provenance: "host".to_owned(),
            location: checked(&sources, 1, 4),
            dependencies: vec![ExternalCatalogDependencyInputV2 {
                target: asset_ref(&sources, &ext, NamespaceKindV2::Value, "base", 5),
                expected_kind: NamespaceKindV2::Value,
                location: checked(&sources, 1, 6),
            }],
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
    assert_eq!(parts.external_catalog().iter().count(), 2);
    let view = parts
        .external_catalog()
        .iter()
        .find(|asset| asset.id().local_name() == "view")
        .unwrap();
    assert_eq!(view.dependencies().len(), 1);

    let mut bad = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    let app = module_id(&sources, "app", 8);
    bad.try_add_module(ModuleNodeInputV2 {
        module_id: app.clone(),
        location: checked(&sources, 1, 9),
    })
    .unwrap();
    let error = bad
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "ext".to_owned(),
            local_name: "bad".to_owned(),
            kind_token: "relation".to_owned(),
            typed_payload: ExternalCatalogPayloadInputV2::Value(ValueTypeV1::int()),
            provenance: "host".to_owned(),
            location: checked(&sources, 1, 10),
            dependencies: vec![],
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.catalog_payload_kind_mismatch");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(10, 11));

    // A dependency whose authored reference kind differs from its
    // `expected_kind` is rejected at external-catalog ingress before resolution.
    let mut wrong_kind = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    let app = module_id(&sources, "app", 11);
    wrong_kind
        .try_add_module(ModuleNodeInputV2 {
            module_id: app,
            location: checked(&sources, 1, 12),
        })
        .unwrap();
    let error = wrong_kind
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "ext".into(),
            local_name: "view".into(),
            kind_token: "view".into(),
            typed_payload: ExternalCatalogPayloadInputV2::View(
                RelationTypeV1::try_new(RecordTypeV1::try_new(vec![]).unwrap(), CardinalityV1::Set)
                    .unwrap(),
            ),
            provenance: "host".into(),
            location: checked(&sources, 1, 14),
            dependencies: vec![ExternalCatalogDependencyInputV2 {
                target: asset_ref(&sources, &ext, NamespaceKindV2::Value, "base", 15),
                expected_kind: NamespaceKindV2::View,
                location: checked(&sources, 1, 16),
            }],
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.catalog_dependency_kind_conflict");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(16, 17));
    assert!(error.secondary().is_none());
}

#[test]
fn external_catalog_rejects_duplicate_and_unresolved_assets() {
    let sources = registry(&[(1, 256)]);
    let app = module_id(&sources, "app", 0);
    let external = module_id(&sources, "host", 1);
    let mut duplicate = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    for (value_type, at) in [(ValueTypeV1::int(), 2), (ValueTypeV1::string(), 3)] {
        let result = duplicate.try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "host".into(),
            local_name: "x".into(),
            kind_token: "value".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Value(value_type),
            provenance: "host".into(),
            location: checked(&sources, 1, at),
            dependencies: vec![],
        });
        if at == 2 {
            result.unwrap();
        } else {
            let error = result.unwrap_err();
            assert_eq!(error.code(), "modules.duplicate_external_catalog_asset");
            assert_eq!(error.primary().unwrap().span, SpanV1::new(3, 4));
            assert_eq!(error.secondary().unwrap().span, SpanV1::new(2, 3));
        }
    }

    let relation =
        RelationTypeV1::try_new(RecordTypeV1::try_new(vec![]).unwrap(), CardinalityV1::Set)
            .unwrap();
    let mut unresolved = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    unresolved
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 4),
        })
        .unwrap();
    unresolved
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "host".into(),
            local_name: "view".into(),
            kind_token: "view".into(),
            typed_payload: ExternalCatalogPayloadInputV2::View(relation),
            provenance: "host".into(),
            location: checked(&sources, 1, 5),
            dependencies: vec![ExternalCatalogDependencyInputV2 {
                target: asset_ref(&sources, &external, NamespaceKindV2::Value, "missing", 6),
                expected_kind: NamespaceKindV2::Value,
                location: checked(&sources, 1, 7),
            }],
        })
        .unwrap();
    let error = unresolved
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 8),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap_err();
    assert_eq!(error.code(), "modules.unknown_catalog_reference");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(7, 8));
}

#[test]
fn external_catalog_cycle_witness_excludes_acyclic_tail_and_is_permutation_stable() {
    fn run(reverse: bool) -> Vec<LocatedSpanV2> {
        let sources = registry(&[(1, 512)]);
        let app = module_id(&sources, "app", 0);
        let external = module_id(&sources, "ext", 1);
        let relation =
            RelationTypeV1::try_new(RecordTypeV1::try_new(vec![]).unwrap(), CardinalityV1::Set)
                .unwrap();
        let asset = |name: &str, target: &str, edge_at, asset_at| ExternalCatalogAssetInputV2 {
            module_name: "ext".into(),
            local_name: name.into(),
            kind_token: "view".into(),
            typed_payload: ExternalCatalogPayloadInputV2::View(relation.clone()),
            provenance: "host".into(),
            location: checked(&sources, 1, asset_at),
            dependencies: vec![ExternalCatalogDependencyInputV2 {
                target: asset_ref(
                    &sources,
                    &external,
                    NamespaceKindV2::View,
                    target,
                    edge_at + 100,
                ),
                expected_kind: NamespaceKindV2::View,
                location: checked(&sources, 1, edge_at),
            }],
        };
        let mut inputs = vec![
            asset("x", "a", 10, 30),
            asset("a", "b", 20, 31),
            asset("b", "a", 21, 32),
        ];
        if reverse {
            inputs.reverse();
        }
        let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: app.clone(),
                location: checked(&sources, 1, 2),
            })
            .unwrap();
        for input in inputs {
            builder.try_add_external(input).unwrap();
        }
        let error = builder
            .finish(
                CurrentModuleSelectionInputV2 {
                    module_id: app,
                    location: checked(&sources, 1, 3),
                },
                VersionV2::new(1, 0, 0),
            )
            .unwrap_err();
        assert_eq!(error.code(), "modules.catalog_dependency_cycle");
        error.edge_witness().to_vec()
    }
    let forward = run(false);
    let reverse = run(true);
    assert_eq!(forward, reverse);
    assert_eq!(
        forward,
        [
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(1),
                span: SpanV1::new(21, 22),
            },
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(1),
                span: SpanV1::new(20, 21),
            },
        ]
    );
    assert!(!forward
        .iter()
        .any(|location| location.span == SpanV1::new(10, 11)));
}

#[test]
fn external_catalog_accepts_exact_closed_ten_kind_payload_matrix() {
    let sources = registry(&[(1, 4096)]);
    let app = module_id(&sources, "app", 0);
    let row = RecordTypeV1::try_new(vec![]).unwrap();
    let relation = RelationTypeV1::try_new(row, CardinalityV1::Set).unwrap();
    let pack = PackIdV2::try_new("catalog-pack", checked(&sources, 1, 1)).unwrap();
    let signature = |kind| SignatureInputV2 {
        parameters: vec![],
        result: ValueTypeV1::bool(),
        callable_kind: kind,
        effects: ExpressionEffectsV2::pure_scalar(),
        location: checked(&sources, 1, 2),
    };
    let rows = vec![
        (
            "value",
            ExternalCatalogPayloadInputV2::Value(ValueTypeV1::int()),
        ),
        (
            "relation",
            ExternalCatalogPayloadInputV2::Relation(relation.clone()),
        ),
        (
            "predicate",
            ExternalCatalogPayloadInputV2::Predicate(signature(CallableKindV2::Predicate)),
        ),
        (
            "type",
            ExternalCatalogPayloadInputV2::Type(ValueTypeV1::string()),
        ),
        ("view", ExternalCatalogPayloadInputV2::View(relation)),
        ("model", ExternalCatalogPayloadInputV2::Model(pack.clone())),
        (
            "pack",
            ExternalCatalogPayloadInputV2::Pack(ExternalPackContractV2::new(
                pack,
                CatalogPackActivationV2::Manual,
            )),
        ),
        (
            "profile",
            ExternalCatalogPayloadInputV2::Profile(ProfileAdmissionV2::CertifiedOnly),
        ),
        (
            "gate",
            ExternalCatalogPayloadInputV2::Gate(GateContractV2::new(GateBindingV2::All)),
        ),
        (
            "rule",
            ExternalCatalogPayloadInputV2::Rule(signature(CallableKindV2::Rule)),
        ),
    ];
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 2),
        })
        .unwrap();
    for (index, (tag, payload)) in rows.into_iter().enumerate() {
        builder
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: "host".to_owned(),
                local_name: format!("asset_{index}"),
                kind_token: tag.to_owned(),
                typed_payload: payload,
                provenance: "host".to_owned(),
                location: checked(&sources, 1, 10 + index),
                dependencies: vec![],
            })
            .unwrap();
    }
    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 30),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    assert_eq!(parts.external_catalog().iter().count(), 10);
    let expected = [
        ("asset_0", NamespaceKindV2::Value),
        ("asset_1", NamespaceKindV2::Relation),
        ("asset_2", NamespaceKindV2::Predicate),
        ("asset_3", NamespaceKindV2::Type),
        ("asset_4", NamespaceKindV2::View),
        ("asset_5", NamespaceKindV2::Model),
        ("asset_6", NamespaceKindV2::Pack),
        ("asset_7", NamespaceKindV2::Profile),
        ("asset_8", NamespaceKindV2::Gate),
        ("asset_9", NamespaceKindV2::Rule),
    ];
    for (name, kind) in expected {
        let asset = parts
            .external_catalog()
            .iter()
            .find(|asset| asset.id().local_name() == name)
            .unwrap();
        assert_eq!(asset.payload().namespace_kind(), kind, "{name}");
    }
}

#[test]
fn external_signature_rejects_foreign_nested_parameter_location() {
    let sources = registry(&[(1, 256)]);
    let foreign = registry(&[(1, 257)]);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    let error = builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "host".into(),
            local_name: "predicate".into(),
            kind_token: "predicate".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Predicate(SignatureInputV2 {
                parameters: vec![LocatedParameterInputV2 {
                    name: "value".into(),
                    value_type: ValueTypeV1::int(),
                    location: checked(&foreign, 1, 1),
                }],
                result: ValueTypeV1::bool(),
                callable_kind: CallableKindV2::Predicate,
                effects: ExpressionEffectsV2::pure_scalar(),
                location: checked(&sources, 1, 2),
            }),
            provenance: "host".into(),
            location: checked(&sources, 1, 3),
            dependencies: vec![],
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.source_registry_mismatch");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(1, 2));
}

#[test]
fn external_signature_pre_admission_has_exact_node_and_byte_boundaries() {
    let sources = registry(&[(1, 64)]);
    let input = |sources: &SourceRegistryViewV2| ExternalCatalogAssetInputV2 {
        module_name: "host".into(),
        local_name: "pred".into(),
        kind_token: "predicate".into(),
        typed_payload: ExternalCatalogPayloadInputV2::Predicate(SignatureInputV2 {
            parameters: vec![LocatedParameterInputV2 {
                name: "x".into(),
                value_type: ValueTypeV1::int(),
                location: checked(sources, 1, 1),
            }],
            result: ValueTypeV1::bool(),
            callable_kind: CallableKindV2::Predicate,
            effects: ExpressionEffectsV2::pure_scalar(),
            location: checked(sources, 1, 2),
        }),
        provenance: "p".into(),
        location: checked(sources, 1, 3),
        dependencies: vec![],
    };
    let exact = AuthorityBudgetProfileV2::try_new(3, 1, 1, 19, 9).unwrap();
    ModuleAuthorityBuilderV2::try_new(exact, sources.clone())
        .unwrap()
        .try_add_external(input(&sources))
        .unwrap();
    for (budget, dimension, limit, observed) in [
        (
            AuthorityBudgetProfileV2::try_new(2, 1, 1, 19, 9).unwrap(),
            "nodes",
            2,
            3,
        ),
        (
            AuthorityBudgetProfileV2::try_new(3, 1, 1, 18, 9).unwrap(),
            "authority_bytes",
            18,
            19,
        ),
    ] {
        let error = ModuleAuthorityBuilderV2::try_new(budget, sources.clone())
            .unwrap()
            .try_add_external(input(&sources))
            .unwrap_err();
        assert_resource(&error, dimension, limit, observed);
    }
}

#[test]
fn external_signature_preserves_arity_and_parameter_order() {
    let sources = registry(&[(1, 128)]);
    let app = module_id(&sources, "app", 0);
    let external = module_id(&sources, "host", 1);
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
                        name: "right".into(),
                        value_type: ValueTypeV1::int(),
                        location: checked(&sources, 1, 3),
                    },
                    LocatedParameterInputV2 {
                        name: "left".into(),
                        value_type: ValueTypeV1::string(),
                        location: checked(&sources, 1, 4),
                    },
                ],
                result: ValueTypeV1::bool(),
                callable_kind: CallableKindV2::Predicate,
                effects: ExpressionEffectsV2::pure_scalar(),
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
    let reference = asset_ref(&sources, &external, NamespaceKindV2::Predicate, "pred", 8);
    let id = parts
        .external_catalog()
        .iter()
        .find(|asset| {
            asset.id().module_id() == &reference.module_id
                && asset.id().local_name() == reference.local_name
        })
        .unwrap();
    assert!(matches!(
        id.payload(),
        ExternalCatalogPayloadV2::Predicate(_)
    ));
    let ExternalCatalogPayloadV2::Predicate(signature) = id.payload() else {
        return;
    };
    assert_eq!(
        signature
            .parameters()
            .iter()
            .map(|parameter| (parameter.name(), parameter.value_type().clone()))
            .collect::<Vec<_>>(),
        vec![
            ("right", ValueTypeV1::int()),
            ("left", ValueTypeV1::string())
        ]
    );
}

#[test]
fn external_dependencies_are_charged_per_submitted_occurrence() {
    let sources = registry(&[(1, 64)]);
    let host = module_id(&sources, "host", 0);
    let dependencies = ["x", "y"]
        .into_iter()
        .enumerate()
        .map(|(index, name)| ExternalCatalogDependencyInputV2 {
            target: asset_ref(&sources, &host, NamespaceKindV2::Value, name, 1 + index),
            expected_kind: NamespaceKindV2::Value,
            location: checked(&sources, 1, 3 + index),
        })
        .collect();
    let error = ModuleAuthorityBuilderV2::try_new(
        AuthorityBudgetProfileV2::try_new(1, 1, 1, 128, 16).unwrap(),
        sources.clone(),
    )
    .unwrap()
    .try_add_external(ExternalCatalogAssetInputV2 {
        module_name: "host".into(),
        local_name: "v".into(),
        kind_token: "value".into(),
        typed_payload: ExternalCatalogPayloadInputV2::Value(ValueTypeV1::int()),
        provenance: "p".into(),
        location: checked(&sources, 1, 5),
        dependencies,
    })
    .unwrap_err();
    assert_resource(&error, "edges", 1, 2);
}

#[test]
fn multibyte_identifier_diagnostic_is_utf8_safe_and_bounded_before_formatting() {
    let sources = registry(&[(1, 8)]);
    let invalid = "é".repeat(10_000);
    let rendered = ModuleIdV2::try_new(&invalid, checked(&sources, 1, 0))
        .unwrap_err()
        .to_string();
    // 64 multibyte chars (128 bytes) survive; the excerpt is byte-bounded and
    // cut on a char boundary, so the exact rendering is a stable oracle.
    let expected = format!(
        "modules.invalid_module_id: invalid module id `{}`",
        "é".repeat(64)
    );
    assert_eq!(rendered, expected);
}

#[test]
fn document_profile_and_require_certified_are_validated_in_combined_registry() {
    let host = registry(&[(1, 1024)]);
    let parts = empty_authority(&host);
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 1024,
        })
        .unwrap();
    let app = module_id(&host, "app", 0);
    let doc_loc = |at| {
        extended
            .try_check_location(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(at, at + 1),
            })
            .unwrap()
    };
    let profile_ref = CatalogAssetRefInputV2 {
        module_id: app.clone(),
        namespace_kind: NamespaceKindV2::Profile,
        local_name: "strict".to_owned(),
        location: doc_loc(2),
    };
    let asset_location = doc_loc(1);
    let admission_location = doc_loc(3);
    let require_location = doc_loc(4);
    let mut validator = DocumentCatalogValidatorV2::try_new(extended, &parts).unwrap();
    let error = validator
        .validate(DocumentCatalogInputV2 {
            assets: vec![DocumentCatalogAssetInputV2 {
                owner_module_id: app,
                local_name: "strict".to_owned(),
                namespace_kind: NamespaceKindV2::Profile,
                location: asset_location,
            }],
            profile_admissions: vec![ProfileAdmissionInputV2 {
                profile: profile_ref,
                admission: ProfileAdmissionV2::Allowlist,
                location: admission_location,
            }],
            activations: vec![],
            require_certified_directives: vec![RequireCertifiedInputV2 {
                location: require_location,
            }],
            dependency_edges: vec![],
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.certified_required_violation");
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(9),
            span: SpanV1::new(4, 5),
        }
    );
    assert!(error.secondary().is_none());
}

#[test]
fn document_view_can_depend_on_external_base_asset() {
    let host = registry(&[(1, 256)]);
    let app = module_id(&host, "app", 0);
    let external = module_id(&host, "host", 1);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), host.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&host, 1, 2),
        })
        .unwrap();
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "host".into(),
            local_name: "base".into(),
            kind_token: "value".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Value(ValueTypeV1::int()),
            provenance: "host".into(),
            location: checked(&host, 1, 3),
            dependencies: vec![],
        })
        .unwrap();
    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app.clone(),
                location: checked(&host, 1, 4),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 64,
        })
        .unwrap();
    let doc_loc = |at| {
        extended
            .try_check_location(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(at, at + 1),
            })
            .unwrap()
    };
    let input = DocumentCatalogInputV2 {
        assets: vec![DocumentCatalogAssetInputV2 {
            owner_module_id: app.clone(),
            local_name: "derived".into(),
            namespace_kind: NamespaceKindV2::View,
            location: doc_loc(1),
        }],
        profile_admissions: vec![],
        activations: vec![],
        require_certified_directives: vec![],
        dependency_edges: vec![DocumentCatalogDependencyInputV2 {
            source: CatalogAssetRefInputV2 {
                module_id: app,
                namespace_kind: NamespaceKindV2::View,
                local_name: "derived".into(),
                location: doc_loc(2),
            },
            target: CatalogAssetRefInputV2 {
                module_id: external,
                namespace_kind: NamespaceKindV2::Value,
                local_name: "base".into(),
                location: doc_loc(3),
            },
            expected_target_kind: NamespaceKindV2::Value,
            location: doc_loc(4),
        }],
    };
    let authority = DocumentCatalogValidatorV2::try_new(extended, &parts)
        .unwrap()
        .validate(input)
        .unwrap();
    assert_eq!(authority.assets().count(), 1);
    assert_eq!(authority.dependencies().len(), 1);
    let edge = authority.dependencies().first().unwrap();
    assert_eq!(edge.source_asset_id().local_name(), "derived");
    assert_eq!(
        edge.source_asset_id().namespace_kind(),
        NamespaceKindV2::View
    );
    assert_eq!(edge.target_asset_id().local_name(), "base");
    assert_eq!(edge.target_asset_id().module_id().as_str(), "host");
    assert_eq!(
        edge.target_asset_id().namespace_kind(),
        NamespaceKindV2::Value
    );
    assert!(edge.family().is_none());
}

#[test]
fn require_certified_traverses_activated_external_pack_contracts() {
    fn validate(
        admission: ProfileAdmissionV2,
    ) -> Result<DocumentCatalogAuthorityV2, ModuleDiagnosticV2> {
        let zeros = "0".repeat(64);
        let lock = format!("secure-pack 1.0.0 {zeros}\n");
        let host = registry(&[(1, 4096), (2, lock.len())]);
        let app = module_id(&host, "app", 0);
        let external_module = module_id(&host, "host", 1);
        let pack = PackIdV2::try_new("secure-pack", checked(&host, 1, 2)).unwrap();
        let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), host.clone()).unwrap();
        builder.try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&host, 1, 3),
        })?;
        builder.try_add_manifest(PackManifestInputV2 {
            pack_id: pack.clone(),
            version: VersionV2::new(1, 0, 0),
            content_digest: ContentDigestV2::try_parse(&zeros, checked(&host, 1, 4)).unwrap(),
            language_range: VersionRangeV2::try_new(
                VersionV2::new(1, 0, 0),
                VersionV2::new(2, 0, 0),
                checked(&host, 1, 5),
            )
            .unwrap(),
            dependencies: vec![],
            exports: vec![],
            suites: vec![],
            location: checked(&host, 1, 6),
        })?;
        builder.try_set_lockfile(
            LockReferenceInputV2 {
                path: "packs/security.lock".into(),
                location: checked(&host, 1, 7),
            },
            SourceIdV2::from_raw(2),
            &lock,
        )?;
        builder.try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "host".into(),
            local_name: "policy".into(),
            kind_token: "profile".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Profile(admission),
            provenance: "host".into(),
            location: checked(&host, 1, 8),
            dependencies: vec![],
        })?;
        builder.try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "host".into(),
            local_name: "security".into(),
            kind_token: "pack".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Pack(ExternalPackContractV2::new(
                pack.clone(),
                CatalogPackActivationV2::Manual,
            )),
            provenance: "host".into(),
            location: checked(&host, 1, 9),
            dependencies: vec![ExternalCatalogDependencyInputV2 {
                target: asset_ref(
                    &host,
                    &external_module,
                    NamespaceKindV2::Profile,
                    "policy",
                    10,
                ),
                expected_kind: NamespaceKindV2::Profile,
                location: checked(&host, 1, 11),
            }],
        })?;
        let parts = builder.finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&host, 1, 12),
            },
            VersionV2::new(1, 1, 0),
        )?;
        let extended = host
            .try_extend_document(SourceRegistrationV2 {
                source_id: SourceIdV2::from_raw(9),
                source_len: 64,
            })
            .unwrap();
        let doc_loc = |at| {
            extended
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(9),
                    span: SpanV1::new(at, at + 1),
                })
                .unwrap()
        };
        let input = DocumentCatalogInputV2 {
            assets: vec![],
            profile_admissions: vec![],
            activations: vec![PackActivationInputV2 {
                pack_id: pack,
                activation: UsePackActivationV2::Session,
                lock_reference: LockReferenceInputV2 {
                    path: "packs/security.lock".into(),
                    location: doc_loc(1),
                },
                location: doc_loc(2),
            }],
            require_certified_directives: vec![RequireCertifiedInputV2 {
                location: doc_loc(3),
            }],
            dependency_edges: vec![],
        };
        DocumentCatalogValidatorV2::try_new(extended, &parts)?.validate(input)
    }

    let error = validate(ProfileAdmissionV2::Allowlist).unwrap_err();
    assert_eq!(error.code(), "modules.certified_required_violation");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(3, 4));
    let authority = validate(ProfileAdmissionV2::CertifiedOnly).unwrap();
    assert_eq!(authority.activations().iter().count(), 1);
}

#[test]
fn pack_lock_digest_version_and_reference_are_sealed() {
    let parts = pack_authority(|text| text).unwrap();
    assert_ne!(parts.pack_authority().graph_fingerprint(), &[0; 32]);
    assert_eq!(
        hex_bytes(parts.pack_authority().graph_fingerprint()),
        "81bf694761f3fc1427f21e43656ee4ddaa3b6f06d6c1d8d1b882287e29c5a9cb"
    );
    let error = pack_authority(|text| text.replacen("1.0.0", "1.0.1", 1)).unwrap_err();
    assert_eq!(error.code(), "modules.lock_mismatch");
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(2),
            span: SpanV1::new(0, 82),
        }
    );
    assert_eq!(
        error.secondary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(6, 7),
        }
    );
    let error = pack_authority(|text| {
        let mut bytes = text.into_bytes();
        let index = bytes.len() - 2;
        *bytes.get_mut(index).unwrap() = b'1';
        String::from_utf8(bytes).unwrap()
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.lock_mismatch");
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(2),
            span: SpanV1::new(0, 82),
        }
    );
    assert_eq!(
        error.secondary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(6, 7),
        }
    );
    let error = pack_authority(|text| text.replacen(' ', "  ", 1)).unwrap_err();
    assert_eq!(error.code(), "modules.invalid_lock_syntax");
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(2),
            span: SpanV1::new(0, 83),
        }
    );
    assert!(error.secondary().is_none());
}

#[test]
fn lock_reference_registry_is_rejected_at_builder_ingress() {
    let host = registry(&[(1, 16), (2, 0)]);
    let foreign = registry(&[(1, 17), (2, 0)]);
    let error = ModuleAuthorityBuilderV2::try_new(profile(), host)
        .unwrap()
        .try_set_lockfile(
            LockReferenceInputV2 {
                path: "packs.lock".into(),
                location: checked(&foreign, 1, 0),
            },
            SourceIdV2::from_raw(2),
            "",
        )
        .unwrap_err();
    assert_eq!(error.code(), "modules.source_registry_mismatch");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(0, 1));
}

#[test]
fn lock_parser_applies_per_field_string_limit_to_precharged_raw_text() {
    let zeros = "0".repeat(64);
    let lock = format!("a 1.0.0 {zeros}\n");
    let sources = registry(&[(1, 128), (2, lock.len())]);
    let app = module_id(&sources, "app", 0);
    let budget = AuthorityBudgetProfileV2::try_new(8, 8, 32, 256, 63).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(budget, sources.clone()).unwrap();
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
                path: "a.lock".into(),
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
    assert_resource(&error, "authority_string_bytes", 63, 64);
}

fn hex_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut text, byte| {
            write!(text, "{byte:02x}").expect("writing to a String cannot fail");
            text
        })
}

#[test]
fn pack_activation_and_model_extension_are_observable() {
    let zeros = "0".repeat(64);
    let lock = format!("secure-pack 1.0.0 {zeros}\n");
    let host = registry(&[(1, 4096), (2, lock.len())]);
    let app = module_id(&host, "app", 0);
    let pack = PackIdV2::try_new("secure-pack", checked(&host, 1, 1)).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), host.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&host, 1, 2),
        })
        .unwrap();
    builder
        .try_add_manifest(PackManifestInputV2 {
            pack_id: pack.clone(),
            version: VersionV2::new(1, 0, 0),
            content_digest: ContentDigestV2::try_parse(&zeros, checked(&host, 1, 3)).unwrap(),
            language_range: VersionRangeV2::try_new(
                VersionV2::new(1, 0, 0),
                VersionV2::new(2, 0, 0),
                checked(&host, 1, 4),
            )
            .unwrap(),
            dependencies: vec![],
            exports: vec![],
            suites: vec![],
            location: checked(&host, 1, 5),
        })
        .unwrap();
    builder
        .try_set_lockfile(
            LockReferenceInputV2 {
                path: "packs/security.lock".to_owned(),
                location: checked(&host, 1, 6),
            },
            SourceIdV2::from_raw(2),
            &lock,
        )
        .unwrap();
    builder
        .try_set_model_schema(ModelSchemaInputV2 {
            schema_version: VersionV2::new(1, 0, 0),
            attributes: vec![ModelAttributeInputV2 {
                attribute: "taint_kind".to_owned(),
                allowed_values: vec![LocatedModelValueInputV2 {
                    value: "source".to_owned(),
                    location: checked(&host, 1, 7),
                }],
                location: checked(&host, 1, 8),
            }],
            location: checked(&host, 1, 9),
        })
        .unwrap();
    builder
        .try_add_model_pack(ModelPackInputV2 {
            pack_id: pack.clone(),
            model_version: VersionV2::new(1, 0, 0),
            facts: vec![ModelFactInputV2 {
                target: "framework::read".to_owned(),
                attribute: "taint_kind".to_owned(),
                value: "source".to_owned(),
                location: checked(&host, 1, 10),
            }],
            location: checked(&host, 1, 11),
        })
        .unwrap();
    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&host, 1, 12),
            },
            VersionV2::new(1, 1, 0),
        )
        .unwrap();
    let composed = parts.composed_model().unwrap();
    let facts = composed.facts().collect::<Vec<_>>();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts.first().unwrap().target(), "framework::read");
    assert_eq!(facts.first().unwrap().attribute(), "taint_kind");
    assert_eq!(facts.first().unwrap().value(), "source");
    assert_eq!(facts.first().unwrap().location().span, SpanV1::new(10, 11));
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 100,
        })
        .unwrap();
    let doc_loc = |at| {
        extended
            .try_check_location(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(at, at + 1),
            })
            .unwrap()
    };
    let reference_location = doc_loc(1);
    let activation_location = doc_loc(2);
    let mut validator = DocumentCatalogValidatorV2::try_new(extended, &parts).unwrap();
    let authority = validator
        .validate(DocumentCatalogInputV2 {
            assets: vec![],
            profile_admissions: vec![],
            activations: vec![PackActivationInputV2 {
                pack_id: pack,
                activation: UsePackActivationV2::Session,
                lock_reference: LockReferenceInputV2 {
                    path: "packs/security.lock".to_owned(),
                    location: reference_location,
                },
                location: activation_location,
            }],
            require_certified_directives: vec![],
            dependency_edges: vec![],
        })
        .unwrap();
    assert_eq!(authority.activations().iter().count(), 1);
    let activation = authority.activations().iter().next().unwrap();
    assert_eq!(activation.pack_id().as_str(), "secure-pack");
    assert_eq!(activation.activation(), UsePackActivationV2::Session);
}

macro_rules! namespace_tag_rail {
    ($name:ident, $variant:ident, $tag:literal) => {
        #[test]
        fn $name() {
            assert_eq!(NamespaceKindV2::$variant.tag(), $tag);
        }
    };
}

namespace_tag_rail!(namespace_value_tag, Value, "value");
namespace_tag_rail!(namespace_relation_tag, Relation, "relation");
namespace_tag_rail!(namespace_predicate_tag, Predicate, "predicate");
namespace_tag_rail!(namespace_type_tag, Type, "type");
namespace_tag_rail!(namespace_view_tag, View, "view");
namespace_tag_rail!(namespace_model_tag, Model, "model");
namespace_tag_rail!(namespace_pack_tag, Pack, "pack");
namespace_tag_rail!(namespace_profile_tag, Profile, "profile");
namespace_tag_rail!(namespace_gate_tag, Gate, "gate");
namespace_tag_rail!(namespace_rule_tag, Rule, "rule");

macro_rules! admission_tag_rail {
    ($name:ident, $variant:ident, $tag:literal) => {
        #[test]
        fn $name() {
            assert_eq!(ProfileAdmissionV2::$variant.tag(), $tag);
        }
    };
}

admission_tag_rail!(admission_allowlist_tag, Allowlist, "allowlist");
admission_tag_rail!(
    admission_review_required_tag,
    ReviewRequired,
    "review-required"
);
admission_tag_rail!(
    admission_certified_only_tag,
    CertifiedOnly,
    "certified-only"
);

#[test]
fn gate_binding_tags_are_closed() {
    assert_eq!(GateBindingV2::All.tag(), "all");
    assert_eq!(GateBindingV2::Any.tag(), "any");
}

#[test]
fn callable_kind_tags_are_closed() {
    assert_eq!(CallableKindV2::Predicate.tag(), "predicate");
    assert_eq!(CallableKindV2::Rule.tag(), "rule");
}

macro_rules! version_accept_rail {
    ($name:ident, $text:literal, $expected:expr) => {
        #[test]
        fn $name() {
            let sources = registry(&[(1, 32)]);
            assert_eq!(
                VersionV2::try_parse($text, checked(&sources, 1, 0)).unwrap(),
                $expected
            );
        }
    };
}

macro_rules! version_reject_rail {
    ($name:ident, $text:literal) => {
        #[test]
        fn $name() {
            let sources = registry(&[(1, 32)]);
            let error = VersionV2::try_parse($text, checked(&sources, 1, 0)).unwrap_err();
            assert_eq!(error.code(), "modules.invalid_version");
            assert_eq!(error.primary().unwrap().span, SpanV1::new(0, 1));
        }
    };
}

version_accept_rail!(version_zero_is_canonical, "0.0.0", VersionV2::new(0, 0, 0));
version_accept_rail!(
    version_numeric_order_fixture_is_canonical,
    "1.10.0",
    VersionV2::new(1, 10, 0)
);
version_reject_rail!(version_leading_zero_rejects, "1.01.0");
version_reject_rail!(version_missing_component_rejects, "1.0");
version_reject_rail!(version_extra_component_rejects, "1.0.0.0");
version_reject_rail!(version_sign_rejects, "+1.0.0");
version_reject_rail!(version_whitespace_rejects, "1.0.0 ");

#[test]
fn version_overflow_rejects_in_every_component() {
    let sources = registry(&[(1, 64)]);
    let overflow = "18446744073709551616";
    for text in [
        format!("{overflow}.0.0"),
        format!("0.{overflow}.0"),
        format!("0.0.{overflow}"),
    ] {
        let error = VersionV2::try_parse(&text, checked(&sources, 1, 7)).unwrap_err();
        assert_eq!(error.code(), "modules.invalid_version", "{text}");
        assert_eq!(error.primary().unwrap().span, SpanV1::new(7, 8));
        assert_eq!(
            error.to_string(),
            "modules.invalid_version: version component exceeds u64"
        );
    }
}

#[test]
fn version_numeric_order_and_display_are_canonical() {
    let sources = registry(&[(1, 8)]);
    let one_ten = VersionV2::try_parse("1.10.0", checked(&sources, 1, 0)).unwrap();
    let two_zero = VersionV2::try_parse("2.0.0", checked(&sources, 1, 1)).unwrap();
    assert!(one_ten < two_zero);
    assert_eq!(one_ten.to_string(), "1.10.0");
    assert_eq!(two_zero.to_string(), "2.0.0");
}

#[test]
fn version_range_is_half_open_and_requires_progress() {
    let sources = registry(&[(1, 32)]);
    let range = VersionRangeV2::try_new(
        VersionV2::new(1, 2, 0),
        VersionV2::new(2, 0, 0),
        checked(&sources, 1, 0),
    )
    .unwrap();
    assert!(range.contains(VersionV2::new(1, 2, 0)));
    assert!(!range.contains(VersionV2::new(2, 0, 0)));
    assert_eq!(
        VersionRangeV2::try_new(
            VersionV2::new(2, 0, 0),
            VersionV2::new(2, 0, 0),
            checked(&sources, 1, 1),
        )
        .unwrap_err()
        .code(),
        "modules.invalid_version_range"
    );
}
