#![allow(clippy::unwrap_used, reason = "owner assertions")]

use std::fmt::Write as _;

use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2,
    SpanV1,
};
use quanta_gqlang_v2_modules::*;
use quanta_gqlang_v2_types::{
    AuthorityBudgetProfileV2, CardinalityV1, ExpressionEffectsV2, RecordTypeV1, RelationTypeV1,
    ValueTypeV1,
};

fn profile() -> AuthorityBudgetProfileV2 {
    AuthorityBudgetProfileV2::try_new(65_536, 262_144, 1_048_576, 16 * 1024 * 1024, 64 * 1024)
        .unwrap()
}

fn registry(entries: &[(u32, usize)]) -> SourceRegistryViewV2 {
    SourceRegistryViewV2::try_new(
        entries
            .iter()
            .map(|(id, len)| SourceRegistrationV2 {
                source_id: SourceIdV2::from_raw(*id),
                source_len: *len,
            })
            .collect(),
    )
    .unwrap()
}

fn checked(registry: &SourceRegistryViewV2, source: u32, start: usize) -> CheckedLocatedSpanV2 {
    registry
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(source),
            span: SpanV1::new(start, start + 1),
        })
        .unwrap()
}

fn module_id(registry: &SourceRegistryViewV2, value: &str, at: usize) -> ModuleIdV2 {
    ModuleIdV2::try_new(value, checked(registry, 1, at)).unwrap()
}

fn asset_ref(
    registry: &SourceRegistryViewV2,
    module: &ModuleIdV2,
    kind: NamespaceKindV2,
    name: &str,
    at: usize,
) -> CatalogAssetRefInputV2 {
    CatalogAssetRefInputV2 {
        module_id: module.clone(),
        namespace_kind: kind,
        local_name: name.to_owned(),
        location: checked(registry, 1, at),
    }
}

fn empty_authority(registry: &SourceRegistryViewV2) -> ValidatedModuleAuthorityPartsV2 {
    let current = module_id(registry, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), registry.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: current.clone(),
            location: checked(registry, 1, 1),
        })
        .unwrap();
    builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: current,
                location: checked(registry, 1, 2),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap()
}

fn document_case_error(
    make_input: impl FnOnce(&SourceRegistryViewV2, &ModuleIdV2) -> DocumentCatalogInputV2,
) -> ModuleDiagnosticV2 {
    let host = registry(&[(1, 128)]);
    let parts = empty_authority(&host);
    let app = module_id(&host, "app", 0);
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 128,
        })
        .unwrap();
    let input = make_input(extended.combined(), &app);
    DocumentCatalogValidatorV2::try_new(extended, &parts)
        .unwrap()
        .validate(input)
        .unwrap_err()
}

fn assert_resource(
    error: &ModuleDiagnosticV2,
    dimension: &'static str,
    limit: usize,
    observed: usize,
) {
    assert_eq!(error.code(), "modules.resource_exhausted");
    let payload = error.resource_exhaustion().unwrap();
    assert_eq!(payload.dimension(), dimension);
    assert_eq!(payload.limit(), limit);
    assert_eq!(payload.observed(), observed);
}

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
    assert!(parts
        .resolved_graph()
        .visible_bindings()
        .get(NamespaceKindV2::Value, "same")
        .is_some());
    assert!(parts
        .resolved_graph()
        .visible_bindings()
        .get(NamespaceKindV2::Type, "same")
        .is_some());
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
    assert_eq!(error.edge_witness().len(), 2);
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
            assert!(error.secondary().is_some());
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
    assert_eq!(forward.len(), 2);
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
            .map(SignatureParameterV2::name)
            .collect::<Vec<_>>(),
        vec!["right", "left"]
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
    assert!(rendered.len() < 256);
    assert!(rendered.starts_with("modules.invalid_module_id:"));
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
    assert_eq!(error.primary().unwrap().source_id.raw(), 9);
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

    assert_eq!(
        validate(ProfileAdmissionV2::Allowlist).unwrap_err().code(),
        "modules.certified_required_violation"
    );
    assert!(validate(ProfileAdmissionV2::CertifiedOnly).is_ok());
}

fn pack_authority(
    mutate_lock: impl FnOnce(String) -> String,
) -> Result<ValidatedModuleAuthorityPartsV2, ModuleDiagnosticV2> {
    let zeros = "0".repeat(64);
    let canonical_lock = format!("secure-pack 1.0.0 {zeros}\n");
    let lock = mutate_lock(canonical_lock);
    let sources = registry(&[(1, 4096), (2, lock.len())]);
    let app = module_id(&sources, "app", 0);
    let pack = PackIdV2::try_new("secure-pack", checked(&sources, 1, 1)).unwrap();
    let digest = ContentDigestV2::try_parse(&zeros, checked(&sources, 1, 2)).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder.try_add_module(ModuleNodeInputV2 {
        module_id: app.clone(),
        location: checked(&sources, 1, 3),
    })?;
    builder.try_add_manifest(PackManifestInputV2 {
        pack_id: pack,
        version: VersionV2::new(1, 0, 0),
        content_digest: digest,
        language_range: VersionRangeV2::try_new(
            VersionV2::new(1, 0, 0),
            VersionV2::new(2, 0, 0),
            checked(&sources, 1, 4),
        )
        .unwrap(),
        dependencies: vec![],
        exports: vec![LocatedExportInputV2 {
            name: "scan".to_owned(),
            location: checked(&sources, 1, 5),
        }],
        suites: vec![],
        location: checked(&sources, 1, 6),
    })?;
    builder.try_set_lockfile(
        LockReferenceInputV2 {
            path: "packs/security.lock".to_owned(),
            location: checked(&sources, 1, 7),
        },
        SourceIdV2::from_raw(2),
        &lock,
    )?;
    builder.finish(
        CurrentModuleSelectionInputV2 {
            module_id: app,
            location: checked(&sources, 1, 8),
        },
        VersionV2::new(1, 1, 0),
    )
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
    let error = pack_authority(|text| {
        let mut bytes = text.into_bytes();
        let index = bytes.len() - 2;
        *bytes.get_mut(index).unwrap() = b'1';
        String::from_utf8(bytes).unwrap()
    })
    .unwrap_err();
    assert_eq!(error.code(), "modules.lock_mismatch");
    let error = pack_authority(|text| text.replacen(' ', "  ", 1)).unwrap_err();
    assert_eq!(error.code(), "modules.invalid_lock_syntax");
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
            assert!(write!(text, "{byte:02x}").is_ok());
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
    assert_eq!(parts.composed_model().unwrap().facts().count(), 1);
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
    assert_eq!(
        authority.activations().iter().next().unwrap().activation(),
        UsePackActivationV2::Session
    );
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

macro_rules! version_parse_rail {
    ($name:ident, $text:literal, $ok:expr) => {
        #[test]
        fn $name() {
            let sources = registry(&[(1, 32)]);
            assert_eq!(
                VersionV2::try_parse($text, checked(&sources, 1, 0)).is_ok(),
                $ok
            );
        }
    };
}

version_parse_rail!(version_zero_is_canonical, "0.0.0", true);
version_parse_rail!(version_numeric_order_fixture_is_canonical, "1.10.0", true);
version_parse_rail!(version_leading_zero_rejects, "1.01.0", false);
version_parse_rail!(version_missing_component_rejects, "1.0", false);
version_parse_rail!(version_extra_component_rejects, "1.0.0.0", false);
version_parse_rail!(version_sign_rejects, "+1.0.0", false);
version_parse_rail!(version_whitespace_rejects, "1.0.0 ", false);

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
    assert!(error.secondary().is_some());
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
    assert!(error.secondary().is_some());
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
    assert_eq!(error.edge_witness().len(), 1);
}

fn manifest_input(
    sources: &SourceRegistryViewV2,
    name: &str,
    version: VersionV2,
    dependencies: Vec<LocatedDependencyInputV2>,
    exports: Vec<LocatedExportInputV2>,
    suites: Vec<LocatedSuiteInputV2>,
    at: usize,
) -> PackManifestInputV2 {
    PackManifestInputV2 {
        pack_id: PackIdV2::try_new(name, checked(sources, 1, at)).unwrap(),
        version,
        content_digest: ContentDigestV2::try_parse(&"0".repeat(64), checked(sources, 1, at + 1))
            .unwrap(),
        language_range: VersionRangeV2::try_new(
            VersionV2::new(1, 0, 0),
            VersionV2::new(2, 0, 0),
            checked(sources, 1, at + 2),
        )
        .unwrap(),
        dependencies,
        exports,
        suites,
        location: checked(sources, 1, at + 3),
    }
}

fn finish_pack_inputs(
    lock: Option<&str>,
    make_inputs: impl FnOnce(&SourceRegistryViewV2) -> Vec<PackManifestInputV2>,
) -> Result<ValidatedModuleAuthorityPartsV2, ModuleDiagnosticV2> {
    let lock_len = lock.map_or(1, str::len);
    let sources = registry(&[(1, 8192), (2, lock_len)]);
    let app = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder.try_add_module(ModuleNodeInputV2 {
        module_id: app.clone(),
        location: checked(&sources, 1, 1),
    })?;
    for input in make_inputs(&sources) {
        builder.try_add_manifest(input)?;
    }
    if let Some(text) = lock {
        builder.try_set_lockfile(
            LockReferenceInputV2 {
                path: "packs.lock".into(),
                location: checked(&sources, 1, 2),
            },
            SourceIdV2::from_raw(2),
            text,
        )?;
    }
    builder.finish(
        CurrentModuleSelectionInputV2 {
            module_id: app,
            location: checked(&sources, 1, 3),
        },
        VersionV2::new(1, 1, 0),
    )
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
}

#[test]
fn empty_pack_graph_rejects_unexpected_lockfile() {
    let error = finish_pack_inputs(Some(""), |_| vec![]).unwrap_err();
    assert_eq!(error.code(), "modules.unexpected_lockfile");
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
            assert!(error.secondary().is_some());
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
            assert!(error.secondary().is_some());
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
    assert!(error.secondary().is_some());
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
    assert!(error.secondary().is_some());
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
    assert_eq!(error.edge_witness().len(), 1);
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
    assert!(error.secondary().is_some());
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
    assert!(error.secondary().is_some());
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
    assert_eq!(
        finish_pack_inputs(Some(&lock), |sources| {
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
        .unwrap_err()
        .code(),
        "modules.lock_mismatch"
    );
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
    assert!(error.secondary().is_some());
}

#[test]
fn document_profile_requires_exactly_one_admission() {
    let host = registry(&[(1, 128)]);
    let parts = empty_authority(&host);
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 128,
        })
        .unwrap();
    let app = module_id(&host, "app", 0);
    let location = extended
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(9),
            span: SpanV1::new(1, 2),
        })
        .unwrap();
    let mut validator = DocumentCatalogValidatorV2::try_new(extended, &parts).unwrap();
    let error = validator
        .validate(DocumentCatalogInputV2 {
            assets: vec![DocumentCatalogAssetInputV2 {
                owner_module_id: app,
                local_name: "p".into(),
                namespace_kind: NamespaceKindV2::Profile,
                location,
            }],
            profile_admissions: vec![],
            activations: vec![],
            require_certified_directives: vec![],
            dependency_edges: vec![],
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.missing_profile_admission");
}

#[test]
fn document_catalog_duplicate_profile_and_reference_negative_matrix() {
    let duplicate_asset = document_case_error(|sources, app| DocumentCatalogInputV2 {
        assets: vec![
            DocumentCatalogAssetInputV2 {
                owner_module_id: app.clone(),
                local_name: "x".into(),
                namespace_kind: NamespaceKindV2::Value,
                location: checked(sources, 9, 1),
            },
            DocumentCatalogAssetInputV2 {
                owner_module_id: app.clone(),
                local_name: "x".into(),
                namespace_kind: NamespaceKindV2::Value,
                location: checked(sources, 9, 2),
            },
        ],
        profile_admissions: vec![],
        activations: vec![],
        require_certified_directives: vec![],
        dependency_edges: vec![],
    });
    assert_eq!(
        duplicate_asset.code(),
        "modules.duplicate_document_catalog_asset"
    );
    assert!(duplicate_asset.secondary().is_some());

    let wrong_profile_kind = document_case_error(|sources, app| DocumentCatalogInputV2 {
        assets: vec![DocumentCatalogAssetInputV2 {
            owner_module_id: app.clone(),
            local_name: "x".into(),
            namespace_kind: NamespaceKindV2::Value,
            location: checked(sources, 9, 1),
        }],
        profile_admissions: vec![ProfileAdmissionInputV2 {
            profile: CatalogAssetRefInputV2 {
                module_id: app.clone(),
                namespace_kind: NamespaceKindV2::Value,
                local_name: "x".into(),
                location: checked(sources, 9, 2),
            },
            admission: ProfileAdmissionV2::CertifiedOnly,
            location: checked(sources, 9, 3),
        }],
        activations: vec![],
        require_certified_directives: vec![],
        dependency_edges: vec![],
    });
    assert_eq!(
        wrong_profile_kind.code(),
        "modules.profile_admission_wrong_kind"
    );

    let unknown_profile = document_case_error(|sources, app| DocumentCatalogInputV2 {
        assets: vec![],
        profile_admissions: vec![ProfileAdmissionInputV2 {
            profile: CatalogAssetRefInputV2 {
                module_id: app.clone(),
                namespace_kind: NamespaceKindV2::Profile,
                local_name: "missing".into(),
                location: checked(sources, 9, 1),
            },
            admission: ProfileAdmissionV2::CertifiedOnly,
            location: checked(sources, 9, 2),
        }],
        activations: vec![],
        require_certified_directives: vec![],
        dependency_edges: vec![],
    });
    assert_eq!(unknown_profile.code(), "modules.unknown_profile_admission");

    let duplicate_profile = document_case_error(|sources, app| {
        let profile = || ProfileAdmissionInputV2 {
            profile: CatalogAssetRefInputV2 {
                module_id: app.clone(),
                namespace_kind: NamespaceKindV2::Profile,
                local_name: "p".into(),
                location: checked(sources, 9, 2),
            },
            admission: ProfileAdmissionV2::CertifiedOnly,
            location: checked(sources, 9, 3),
        };
        DocumentCatalogInputV2 {
            assets: vec![DocumentCatalogAssetInputV2 {
                owner_module_id: app.clone(),
                local_name: "p".into(),
                namespace_kind: NamespaceKindV2::Profile,
                location: checked(sources, 9, 1),
            }],
            profile_admissions: vec![profile(), profile()],
            activations: vec![],
            require_certified_directives: vec![],
            dependency_edges: vec![],
        }
    });
    assert_eq!(
        duplicate_profile.code(),
        "modules.duplicate_profile_admission"
    );
    assert!(duplicate_profile.secondary().is_some());

    let wrong_reference_kind = document_case_error(|sources, app| DocumentCatalogInputV2 {
        assets: vec![DocumentCatalogAssetInputV2 {
            owner_module_id: app.clone(),
            local_name: "v".into(),
            namespace_kind: NamespaceKindV2::View,
            location: checked(sources, 9, 1),
        }],
        profile_admissions: vec![],
        activations: vec![],
        require_certified_directives: vec![],
        dependency_edges: vec![DocumentCatalogDependencyInputV2 {
            source: CatalogAssetRefInputV2 {
                module_id: app.clone(),
                namespace_kind: NamespaceKindV2::View,
                local_name: "v".into(),
                location: checked(sources, 9, 2),
            },
            target: CatalogAssetRefInputV2 {
                module_id: app.clone(),
                namespace_kind: NamespaceKindV2::Value,
                local_name: "missing".into(),
                location: checked(sources, 9, 3),
            },
            expected_target_kind: NamespaceKindV2::Type,
            location: checked(sources, 9, 4),
        }],
    });
    assert_eq!(
        wrong_reference_kind.code(),
        "modules.catalog_reference_kind_mismatch"
    );

    let unknown_target = document_case_error(|sources, app| DocumentCatalogInputV2 {
        assets: vec![DocumentCatalogAssetInputV2 {
            owner_module_id: app.clone(),
            local_name: "v".into(),
            namespace_kind: NamespaceKindV2::View,
            location: checked(sources, 9, 1),
        }],
        profile_admissions: vec![],
        activations: vec![],
        require_certified_directives: vec![],
        dependency_edges: vec![DocumentCatalogDependencyInputV2 {
            source: CatalogAssetRefInputV2 {
                module_id: app.clone(),
                namespace_kind: NamespaceKindV2::View,
                local_name: "v".into(),
                location: checked(sources, 9, 2),
            },
            target: CatalogAssetRefInputV2 {
                module_id: app.clone(),
                namespace_kind: NamespaceKindV2::Value,
                local_name: "missing".into(),
                location: checked(sources, 9, 3),
            },
            expected_target_kind: NamespaceKindV2::Value,
            location: checked(sources, 9, 4),
        }],
    });
    assert_eq!(unknown_target.code(), "modules.unknown_catalog_reference");

    let wrong_source = document_case_error(|sources, app| DocumentCatalogInputV2 {
        assets: vec![DocumentCatalogAssetInputV2 {
            owner_module_id: app.clone(),
            local_name: "x".into(),
            namespace_kind: NamespaceKindV2::Value,
            location: checked(sources, 9, 1),
        }],
        profile_admissions: vec![],
        activations: vec![],
        require_certified_directives: vec![],
        dependency_edges: vec![DocumentCatalogDependencyInputV2 {
            source: CatalogAssetRefInputV2 {
                module_id: app.clone(),
                namespace_kind: NamespaceKindV2::Value,
                local_name: "x".into(),
                location: checked(sources, 9, 2),
            },
            target: CatalogAssetRefInputV2 {
                module_id: app.clone(),
                namespace_kind: NamespaceKindV2::Value,
                local_name: "x".into(),
                location: checked(sources, 9, 3),
            },
            expected_target_kind: NamespaceKindV2::Value,
            location: checked(sources, 9, 4),
        }],
    });
    assert_eq!(
        wrong_source.code(),
        "modules.catalog_dependency_source_wrong_kind"
    );
}

#[test]
fn document_view_dependency_self_cycle_rejects() {
    let host = registry(&[(1, 128)]);
    let parts = empty_authority(&host);
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 128,
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
    let reference = CatalogAssetRefInputV2 {
        module_id: app.clone(),
        namespace_kind: NamespaceKindV2::View,
        local_name: "v".into(),
        location: doc_loc(2),
    };
    let asset_location = doc_loc(1);
    let edge_location = doc_loc(3);
    let mut validator = DocumentCatalogValidatorV2::try_new(extended, &parts).unwrap();
    let error = validator
        .validate(DocumentCatalogInputV2 {
            assets: vec![DocumentCatalogAssetInputV2 {
                owner_module_id: app,
                local_name: "v".into(),
                namespace_kind: NamespaceKindV2::View,
                location: asset_location,
            }],
            profile_admissions: vec![],
            activations: vec![],
            require_certified_directives: vec![],
            dependency_edges: vec![DocumentCatalogDependencyInputV2 {
                source: reference.clone(),
                target: reference,
                expected_target_kind: NamespaceKindV2::View,
                location: edge_location,
            }],
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.view_inline_cycle");
    assert_eq!(error.edge_witness().len(), 1);
}

#[test]
fn document_cycle_witness_excludes_acyclic_tail_and_is_permutation_stable() {
    fn run(reverse: bool) -> Vec<LocatedSpanV2> {
        let host = registry(&[(1, 128)]);
        let parts = empty_authority(&host);
        let extended = host
            .try_extend_document(SourceRegistrationV2 {
                source_id: SourceIdV2::from_raw(9),
                source_len: 128,
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
        let reference = |name: &str, at| CatalogAssetRefInputV2 {
            module_id: app.clone(),
            namespace_kind: NamespaceKindV2::View,
            local_name: name.into(),
            location: doc_loc(at),
        };
        let mut edges = vec![
            DocumentCatalogDependencyInputV2 {
                source: reference("x", 30),
                target: reference("a", 31),
                expected_target_kind: NamespaceKindV2::View,
                location: doc_loc(10),
            },
            DocumentCatalogDependencyInputV2 {
                source: reference("a", 32),
                target: reference("b", 33),
                expected_target_kind: NamespaceKindV2::View,
                location: doc_loc(20),
            },
            DocumentCatalogDependencyInputV2 {
                source: reference("b", 34),
                target: reference("a", 35),
                expected_target_kind: NamespaceKindV2::View,
                location: doc_loc(21),
            },
        ];
        if reverse {
            edges.reverse();
        }
        let input = DocumentCatalogInputV2 {
            assets: ["x", "a", "b"]
                .into_iter()
                .enumerate()
                .map(|(index, name)| DocumentCatalogAssetInputV2 {
                    owner_module_id: app.clone(),
                    local_name: name.into(),
                    namespace_kind: NamespaceKindV2::View,
                    location: doc_loc(40 + index),
                })
                .collect(),
            profile_admissions: vec![],
            activations: vec![],
            require_certified_directives: vec![],
            dependency_edges: edges,
        };
        let error = DocumentCatalogValidatorV2::try_new(extended, &parts)
            .unwrap()
            .validate(input)
            .unwrap_err();
        assert_eq!(error.code(), "modules.view_inline_cycle");
        error.edge_witness().to_vec()
    }
    let forward = run(false);
    let reverse = run(true);
    assert_eq!(forward, reverse);
    assert_eq!(forward.len(), 2);
    assert!(!forward
        .iter()
        .any(|location| location.span == SpanV1::new(10, 11)));
}

#[test]
fn activation_lock_reference_must_match_sealed_reference() {
    let parts = pack_authority(|text| text).unwrap();
    let zeros = "0".repeat(64);
    let lock = format!("secure-pack 1.0.0 {zeros}\n");
    let host = registry(&[(1, 4096), (2, lock.len())]);
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 128,
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
    let pack = PackIdV2::try_new("secure-pack", checked(&host, 1, 1)).unwrap();
    let reference_location = doc_loc(1);
    let activation_location = doc_loc(2);
    let mut validator = DocumentCatalogValidatorV2::try_new(extended, &parts).unwrap();
    let error = validator
        .validate(DocumentCatalogInputV2 {
            assets: vec![],
            profile_admissions: vec![],
            require_certified_directives: vec![],
            dependency_edges: vec![],
            activations: vec![PackActivationInputV2 {
                pack_id: pack,
                activation: UsePackActivationV2::Workspace,
                lock_reference: LockReferenceInputV2 {
                    path: "other.lock".into(),
                    location: reference_location,
                },
                location: activation_location,
            }],
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.lock_reference_mismatch");
}

#[test]
fn duplicate_activation_validates_and_charges_attempted_lock_path_before_dedup() {
    let parts = pack_authority(|text| text).unwrap();
    let zeros = "0".repeat(64);
    let lock = format!("secure-pack 1.0.0 {zeros}\n");
    let host = registry(&[(1, 4096), (2, lock.len())]);
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 128,
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
    let pack = PackIdV2::try_new("secure-pack", checked(&host, 1, 1)).unwrap();
    let input = DocumentCatalogInputV2 {
        assets: vec![],
        profile_admissions: vec![],
        require_certified_directives: vec![],
        dependency_edges: vec![],
        activations: vec![
            PackActivationInputV2 {
                pack_id: pack.clone(),
                activation: UsePackActivationV2::Workspace,
                lock_reference: LockReferenceInputV2 {
                    path: "packs/security.lock".into(),
                    location: doc_loc(1),
                },
                location: doc_loc(2),
            },
            PackActivationInputV2 {
                pack_id: pack,
                activation: UsePackActivationV2::Session,
                lock_reference: LockReferenceInputV2 {
                    path: "not-a-lock".into(),
                    location: doc_loc(3),
                },
                location: doc_loc(4),
            },
        ],
    };
    let error = DocumentCatalogValidatorV2::try_new(extended, &parts)
        .unwrap()
        .validate(input)
        .unwrap_err();
    assert_eq!(error.code(), "modules.invalid_lockfile_path");
}

#[test]
fn duplicate_pack_activation_rejects_identical_valid_references() {
    let parts = pack_authority(|text| text).unwrap();
    let zeros = "0".repeat(64);
    let lock = format!("secure-pack 1.0.0 {zeros}\n");
    let host = registry(&[(1, 4096), (2, lock.len())]);
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 32,
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
    let pack = PackIdV2::try_new("secure-pack", checked(&host, 1, 1)).unwrap();
    let activation = |at| PackActivationInputV2 {
        pack_id: pack.clone(),
        activation: UsePackActivationV2::Session,
        lock_reference: LockReferenceInputV2 {
            path: "packs/security.lock".into(),
            location: doc_loc(at),
        },
        location: doc_loc(at + 1),
    };
    let input = DocumentCatalogInputV2 {
        assets: vec![],
        profile_admissions: vec![],
        require_certified_directives: vec![],
        dependency_edges: vec![],
        activations: vec![activation(1), activation(3)],
    };
    let error = DocumentCatalogValidatorV2::try_new(extended, &parts)
        .unwrap()
        .validate(input)
        .unwrap_err();
    assert_eq!(error.code(), "modules.duplicate_pack_activation");
    assert!(error.secondary().is_some());
}

#[test]
fn use_pack_rejects_unknown_pack_with_checked_activation_location() {
    let parts = pack_authority(|text| text).unwrap();
    let zeros = "0".repeat(64);
    let lock = format!("secure-pack 1.0.0 {zeros}\n");
    let host = registry(&[(1, 4096), (2, lock.len())]);
    let extended = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 32,
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
    let activation_location = doc_loc(2);
    let input = DocumentCatalogInputV2 {
        assets: vec![],
        profile_admissions: vec![],
        require_certified_directives: vec![],
        dependency_edges: vec![],
        activations: vec![PackActivationInputV2 {
            pack_id: PackIdV2::try_new("missing", checked(&host, 1, 1)).unwrap(),
            activation: UsePackActivationV2::Session,
            lock_reference: LockReferenceInputV2 {
                path: "packs/security.lock".into(),
                location: doc_loc(1),
            },
            location: activation_location.clone(),
        }],
    };
    let error = DocumentCatalogValidatorV2::try_new(extended, &parts)
        .unwrap()
        .validate(input)
        .unwrap_err();
    assert_eq!(error.code(), "modules.unknown_pack");
    assert_eq!(error.primary(), Some(activation_location.location()));
}

#[test]
fn authority_string_limit_is_enforced_at_public_ingress() {
    let sources = registry(&[(1, 64)]);
    let tiny = AuthorityBudgetProfileV2::try_new(4, 4, 16, 64, 2).unwrap();
    let app = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(tiny, sources.clone()).unwrap();
    let error = builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app,
            location: checked(&sources, 1, 1),
        })
        .unwrap_err();
    assert_resource(&error, "authority_string_bytes", 2, 3);
}

#[test]
fn authority_string_exact_limit_is_accepted() {
    let sources = registry(&[(1, 64)]);
    let exact = AuthorityBudgetProfileV2::try_new(4, 4, 16, 64, 1).unwrap();
    let app = module_id(&sources, "a", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(exact, sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 1),
        })
        .unwrap();
    assert_eq!(
        builder
            .finish(
                CurrentModuleSelectionInputV2 {
                    module_id: app,
                    location: checked(&sources, 1, 2)
                },
                VersionV2::new(1, 0, 0)
            )
            .unwrap()
            .resolved_graph()
            .current_module()
            .as_str(),
        "a"
    );
}

#[test]
fn fingerprint_changes_when_manifest_field_changes() {
    let zeros = "0".repeat(64);
    let lock = format!("a 1.0.0 {zeros}\n");
    let build = |export: &'static str| {
        finish_pack_inputs(Some(&lock), |sources| {
            vec![manifest_input(
                sources,
                "a",
                VersionV2::new(1, 0, 0),
                vec![],
                vec![LocatedExportInputV2 {
                    name: export.into(),
                    location: checked(sources, 1, 20),
                }],
                vec![],
                10,
            )]
        })
        .unwrap()
    };
    assert_ne!(
        build("x").pack_authority().graph_fingerprint(),
        build("y").pack_authority().graph_fingerprint()
    );
}

#[test]
fn fingerprint_mutation_matrix_covers_all_manifest_semantics() {
    #[derive(Clone)]
    struct FingerprintCase<'a> {
        name: &'a str,
        version: VersionV2,
        digest_text: &'a str,
        language_lower: VersionV2,
        language_upper: VersionV2,
        dependency_name: &'a str,
        dependency_lower: VersionV2,
        dependency_upper: VersionV2,
        export_name: &'a str,
        suite_name: &'a str,
        suite_member: &'a str,
    }

    fn build(case: &FingerprintCase<'_>) -> [u8; 32] {
        let zeros = "0".repeat(64);
        let mut pin_lines = [
            format!("{} {} {}\n", case.name, case.version, case.digest_text),
            format!("y 1.0.0 {zeros}\n"),
            format!("z 1.0.0 {zeros}\n"),
        ];
        pin_lines.sort();
        let lock = pin_lines.concat();
        let parts = finish_pack_inputs(Some(&lock), |sources| {
            let dependency =
                PackIdV2::try_new(case.dependency_name, checked(sources, 1, 30)).unwrap();
            let primary = PackManifestInputV2 {
                pack_id: PackIdV2::try_new(case.name, checked(sources, 1, 10)).unwrap(),
                version: case.version,
                content_digest: ContentDigestV2::try_parse(
                    case.digest_text,
                    checked(sources, 1, 11),
                )
                .unwrap(),
                language_range: VersionRangeV2::try_new(
                    case.language_lower,
                    case.language_upper,
                    checked(sources, 1, 12),
                )
                .unwrap(),
                dependencies: vec![LocatedDependencyInputV2 {
                    pack_id: dependency,
                    version_range: VersionRangeV2::try_new(
                        case.dependency_lower,
                        case.dependency_upper,
                        checked(sources, 1, 13),
                    )
                    .unwrap(),
                    location: checked(sources, 1, 14),
                }],
                exports: vec![LocatedExportInputV2 {
                    name: case.export_name.into(),
                    location: checked(sources, 1, 15),
                }],
                suites: vec![LocatedSuiteInputV2 {
                    name: case.suite_name.into(),
                    members: vec![LocatedSuiteMemberInputV2 {
                        name: case.suite_member.into(),
                        location: checked(sources, 1, 16),
                    }],
                    location: checked(sources, 1, 17),
                }],
                location: checked(sources, 1, 17),
            };
            vec![
                primary,
                manifest_input(
                    sources,
                    "y",
                    VersionV2::new(1, 0, 0),
                    vec![],
                    vec![],
                    vec![],
                    40,
                ),
                manifest_input(
                    sources,
                    "z",
                    VersionV2::new(1, 0, 0),
                    vec![],
                    vec![],
                    vec![],
                    40,
                ),
            ]
        })
        .unwrap();
        *parts.pack_authority().graph_fingerprint()
    }

    let zeros = "0".repeat(64);
    let ones = "1".repeat(64);
    let base_case = FingerprintCase {
        name: "a",
        version: VersionV2::new(1, 0, 0),
        digest_text: &zeros,
        language_lower: VersionV2::new(1, 0, 0),
        language_upper: VersionV2::new(2, 0, 0),
        dependency_name: "z",
        dependency_lower: VersionV2::new(1, 0, 0),
        dependency_upper: VersionV2::new(2, 0, 0),
        export_name: "x",
        suite_name: "core",
        suite_member: "scan",
    };
    let base = build(&base_case);
    macro_rules! mutated {
        ($field:ident = $value:expr) => {{
            let mut case = base_case.clone();
            case.$field = $value;
            case
        }};
    }
    let cases = vec![
        mutated!(name = "b"),
        mutated!(version = VersionV2::new(1, 1, 0)),
        mutated!(digest_text = &ones),
        mutated!(language_lower = VersionV2::new(0, 9, 0)),
        mutated!(language_upper = VersionV2::new(3, 0, 0)),
        mutated!(dependency_name = "y"),
        mutated!(dependency_lower = VersionV2::new(0, 9, 0)),
        mutated!(dependency_upper = VersionV2::new(3, 0, 0)),
        mutated!(export_name = "y"),
        mutated!(suite_name = "extended"),
        mutated!(suite_member = "join"),
    ];
    let mutations: Vec<_> = cases.iter().map(build).collect();
    assert!(mutations.iter().all(|fingerprint| fingerprint != &base));
}

#[test]
fn module_node_and_edge_budgets_accept_exact_limit_then_reject_one_over() {
    let sources = registry(&[(1, 256)]);
    let a = module_id(&sources, "a", 0);
    let b = module_id(&sources, "b", 1);
    let budget = AuthorityBudgetProfileV2::try_new(2, 1, 32, 1_024, 128).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(budget, sources.clone()).unwrap();
    for (module_id, at) in [(a.clone(), 2), (b.clone(), 3)] {
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id,
                location: checked(&sources, 1, at),
            })
            .unwrap();
    }
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: a.clone(),
            local_name: "x".into(),
            target: asset_ref(&sources, &b, NamespaceKindV2::Value, "x", 4),
            location: checked(&sources, 1, 5),
        })
        .unwrap();
    let error = builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: a,
            local_name: "y".into(),
            target: asset_ref(&sources, &b, NamespaceKindV2::Value, "y", 6),
            location: checked(&sources, 1, 7),
        })
        .unwrap_err();
    assert_resource(&error, "edges", 1, 2);
}

#[test]
fn module_node_budget_reports_exact_one_over_payload() {
    let sources = registry(&[(1, 32)]);
    let mut builder = ModuleAuthorityBuilderV2::try_new(
        AuthorityBudgetProfileV2::try_new(1, 1, 1, 32, 8).unwrap(),
        sources.clone(),
    )
    .unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: module_id(&sources, "a", 0),
            location: checked(&sources, 1, 1),
        })
        .unwrap();
    let error = builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: module_id(&sources, "b", 2),
            location: checked(&sources, 1, 3),
        })
        .unwrap_err();
    let payload = error.resource_exhaustion().unwrap();
    assert_eq!(payload.dimension(), "nodes");
    assert_eq!(payload.limit(), 1);
    assert_eq!(payload.observed(), 2);
}

#[test]
fn module_step_budget_accepts_exact_limit_and_reports_one_over() {
    fn run(step_limit: usize) -> Result<ValidatedModuleAuthorityPartsV2, ModuleDiagnosticV2> {
        let sources = registry(&[(1, 128)]);
        let app = module_id(&sources, "app", 0);
        let lib = module_id(&sources, "lib", 1);
        let mut builder = ModuleAuthorityBuilderV2::try_new(
            AuthorityBudgetProfileV2::try_new(3, 1, step_limit, 128, 16).unwrap(),
            sources.clone(),
        )?;
        for (id, at) in [(&app, 2), (&lib, 3)] {
            builder.try_add_module(ModuleNodeInputV2 {
                module_id: id.clone(),
                location: checked(&sources, 1, at),
            })?;
        }
        builder.try_declare(ModuleDeclarationInputV2 {
            owner_module_id: lib.clone(),
            local_name: "x".into(),
            namespace_kind: NamespaceKindV2::Value,
            visibility: VisibilityV2::Public,
            location: checked(&sources, 1, 4),
        })?;
        builder.try_import(ModuleImportInputV2 {
            owner_module_id: app.clone(),
            local_name: "x".into(),
            target: asset_ref(&sources, &lib, NamespaceKindV2::Value, "x", 5),
            location: checked(&sources, 1, 6),
        })?;
        builder.finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 7),
            },
            VersionV2::new(1, 0, 0),
        )
    }

    assert!(run(2).is_ok());
    let error = run(1).unwrap_err();
    let payload = error.resource_exhaustion().unwrap();
    assert_eq!(payload.dimension(), "steps");
    assert_eq!(payload.limit(), 1);
    assert_eq!(payload.observed(), 2);
}

#[test]
fn aggregate_authority_bytes_accept_exact_name_then_reject_one_over() {
    let sources = registry(&[(1, 64)]);
    let id = module_id(&sources, "app", 0);
    let exact = AuthorityBudgetProfileV2::try_new(1, 1, 1, 3, 3).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(exact, sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: id.clone(),
            location: checked(&sources, 1, 1),
        })
        .unwrap();

    let one_under = AuthorityBudgetProfileV2::try_new(1, 1, 1, 2, 3).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(one_under, sources.clone()).unwrap();
    let error = builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: id,
            location: checked(&sources, 1, 2),
        })
        .unwrap_err();
    assert_resource(&error, "authority_bytes", 2, 3);
}

#[test]
fn model_value_and_lock_bytes_have_exact_and_one_over_boundaries() {
    let sources = registry(&[(1, 64), (2, 1)]);
    let schema = |sources: &SourceRegistryViewV2| ModelSchemaInputV2 {
        schema_version: VersionV2::new(1, 0, 0),
        attributes: vec![ModelAttributeInputV2 {
            attribute: "a".into(),
            allowed_values: vec![LocatedModelValueInputV2 {
                value: "xy".into(),
                location: checked(sources, 1, 1),
            }],
            location: checked(sources, 1, 2),
        }],
        location: checked(sources, 1, 3),
    };

    let exact = AuthorityBudgetProfileV2::try_new(2, 1, 8, 3, 2).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(exact, sources.clone()).unwrap();
    builder.try_set_model_schema(schema(&sources)).unwrap();
    let one_under = AuthorityBudgetProfileV2::try_new(2, 1, 8, 2, 2).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(one_under, sources.clone()).unwrap();
    let error = builder.try_set_model_schema(schema(&sources)).unwrap_err();
    assert_resource(&error, "authority_bytes", 2, 3);

    let lock_input = |sources: &SourceRegistryViewV2| LockReferenceInputV2 {
        path: "l.lock".into(),
        location: checked(sources, 1, 4),
    };
    let exact = AuthorityBudgetProfileV2::try_new(1, 1, 8, 7, 6).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(exact, sources.clone()).unwrap();
    builder
        .try_set_lockfile(lock_input(&sources), SourceIdV2::from_raw(2), "x")
        .unwrap();
    let one_under = AuthorityBudgetProfileV2::try_new(1, 1, 8, 6, 6).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(one_under, sources.clone()).unwrap();
    let error = builder
        .try_set_lockfile(lock_input(&sources), SourceIdV2::from_raw(2), "x")
        .unwrap_err();
    assert_resource(&error, "authority_bytes", 6, 7);
}

#[test]
fn duplicate_model_schema_attempt_charges_nested_edges_and_strings_before_rejection() {
    let sources = registry(&[(1, 64)]);
    let schema = |sources: &SourceRegistryViewV2, at| ModelSchemaInputV2 {
        schema_version: VersionV2::new(1, 0, 0),
        attributes: vec![ModelAttributeInputV2 {
            attribute: "a".into(),
            allowed_values: vec![LocatedModelValueInputV2 {
                value: "x".into(),
                location: checked(sources, 1, at),
            }],
            location: checked(sources, 1, at + 1),
        }],
        location: checked(sources, 1, at + 2),
    };

    let exact = AuthorityBudgetProfileV2::try_new(4, 2, 8, 4, 1).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(exact, sources.clone()).unwrap();
    builder.try_set_model_schema(schema(&sources, 1)).unwrap();
    assert_eq!(
        builder
            .try_set_model_schema(schema(&sources, 10))
            .unwrap_err()
            .code(),
        "modules.duplicate_model_schema"
    );

    let one_under = AuthorityBudgetProfileV2::try_new(4, 2, 8, 3, 1).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(one_under, sources.clone()).unwrap();
    builder.try_set_model_schema(schema(&sources, 1)).unwrap();
    let error = builder
        .try_set_model_schema(schema(&sources, 10))
        .unwrap_err();
    assert_resource(&error, "authority_bytes", 3, 4);
}

#[test]
fn document_catalog_charges_repeated_owner_and_reference_module_ids() {
    fn validate_with_limit(limit: usize) -> Result<(), ModuleDiagnosticV2> {
        let host = registry(&[(1, 64)]);
        let budget = AuthorityBudgetProfileV2::try_new(8, 8, 32, limit, 4).unwrap();
        let app = module_id(&host, "app", 0);
        let mut builder = ModuleAuthorityBuilderV2::try_new(budget, host.clone()).unwrap();
        builder.try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&host, 1, 1),
        })?;
        let parts = builder.finish(
            CurrentModuleSelectionInputV2 {
                module_id: app.clone(),
                location: checked(&host, 1, 2),
            },
            VersionV2::new(1, 0, 0),
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
        let reference = |kind, name: &str, at| CatalogAssetRefInputV2 {
            module_id: app.clone(),
            namespace_kind: kind,
            local_name: name.into(),
            location: doc_loc(at),
        };
        let input = DocumentCatalogInputV2 {
            assets: vec![
                DocumentCatalogAssetInputV2 {
                    owner_module_id: app.clone(),
                    local_name: "v".into(),
                    namespace_kind: NamespaceKindV2::View,
                    location: doc_loc(1),
                },
                DocumentCatalogAssetInputV2 {
                    owner_module_id: app.clone(),
                    local_name: "p".into(),
                    namespace_kind: NamespaceKindV2::Profile,
                    location: doc_loc(2),
                },
            ],
            profile_admissions: vec![ProfileAdmissionInputV2 {
                profile: reference(NamespaceKindV2::Profile, "p", 3),
                admission: ProfileAdmissionV2::CertifiedOnly,
                location: doc_loc(4),
            }],
            activations: vec![],
            require_certified_directives: vec![],
            dependency_edges: vec![DocumentCatalogDependencyInputV2 {
                source: reference(NamespaceKindV2::View, "v", 5),
                target: reference(NamespaceKindV2::Profile, "p", 6),
                expected_target_kind: NamespaceKindV2::Profile,
                location: doc_loc(7),
            }],
        };
        DocumentCatalogValidatorV2::try_new(extended, &parts)?.validate(input)?;
        Ok(())
    }

    // Two assets (8), one profile reference (4), and two dependency refs (8).
    validate_with_limit(20).unwrap();
    let error = validate_with_limit(19).unwrap_err();
    assert_resource(&error, "authority_bytes", 19, 20);
}

fn finish_model_facts(
    facts: &[(&str, usize)],
) -> Result<ValidatedModuleAuthorityPartsV2, ModuleDiagnosticV2> {
    let zeros = "0".repeat(64);
    let lock = format!("model-pack 1.0.0 {zeros}\n");
    let sources = registry(&[(1, 1_024), (2, lock.len())]);
    let app = module_id(&sources, "app", 0);
    let pack = PackIdV2::try_new("model-pack", checked(&sources, 1, 1)).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder.try_add_module(ModuleNodeInputV2 {
        module_id: app.clone(),
        location: checked(&sources, 1, 2),
    })?;
    builder.try_add_manifest(PackManifestInputV2 {
        pack_id: pack.clone(),
        version: VersionV2::new(1, 0, 0),
        content_digest: ContentDigestV2::try_parse(&zeros, checked(&sources, 1, 3)).unwrap(),
        language_range: VersionRangeV2::try_new(
            VersionV2::new(1, 0, 0),
            VersionV2::new(2, 0, 0),
            checked(&sources, 1, 4),
        )
        .unwrap(),
        dependencies: vec![],
        exports: vec![],
        suites: vec![],
        location: checked(&sources, 1, 5),
    })?;
    builder.try_set_lockfile(
        LockReferenceInputV2 {
            path: "model.lock".into(),
            location: checked(&sources, 1, 6),
        },
        SourceIdV2::from_raw(2),
        &lock,
    )?;
    builder.try_set_model_schema(ModelSchemaInputV2 {
        schema_version: VersionV2::new(1, 0, 0),
        attributes: vec![ModelAttributeInputV2 {
            attribute: "kind".into(),
            allowed_values: vec![
                LocatedModelValueInputV2 {
                    value: "sink".into(),
                    location: checked(&sources, 1, 7),
                },
                LocatedModelValueInputV2 {
                    value: "source".into(),
                    location: checked(&sources, 1, 8),
                },
            ],
            location: checked(&sources, 1, 9),
        }],
        location: checked(&sources, 1, 10),
    })?;
    builder.try_add_model_pack(ModelPackInputV2 {
        pack_id: pack,
        model_version: VersionV2::new(1, 0, 0),
        facts: facts
            .iter()
            .map(|(value, at)| ModelFactInputV2 {
                target: "framework::read".into(),
                attribute: "kind".into(),
                value: (*value).to_owned(),
                location: checked(&sources, 1, *at),
            })
            .collect(),
        location: checked(&sources, 1, 11),
    })?;
    builder.finish(
        CurrentModuleSelectionInputV2 {
            module_id: app,
            location: checked(&sources, 1, 12),
        },
        VersionV2::new(1, 1, 0),
    )
}

#[test]
fn model_duplicate_fact_is_idempotent_and_observable() {
    let parts = finish_model_facts(&[("source", 20), ("source", 19)]).unwrap();
    let facts = parts.composed_model().unwrap().facts().collect::<Vec<_>>();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts.first().unwrap().value(), "source");
    assert_eq!(facts.first().unwrap().location().span, SpanV1::new(19, 20));
}

#[test]
fn agreeing_model_packs_merge_and_retain_sorted_provenance() {
    let zeros = "0".repeat(64);
    let lock = format!("a 1.0.0 {zeros}\nb 1.0.0 {zeros}\n");
    let sources = registry(&[(1, 1024), (2, lock.len())]);
    let app = module_id(&sources, "app", 0);
    let packs = [
        PackIdV2::try_new("a", checked(&sources, 1, 1)).unwrap(),
        PackIdV2::try_new("b", checked(&sources, 1, 2)).unwrap(),
    ];
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 3),
        })
        .unwrap();
    for (index, pack) in packs.iter().enumerate() {
        builder
            .try_add_manifest(PackManifestInputV2 {
                pack_id: pack.clone(),
                version: VersionV2::new(1, 0, 0),
                content_digest: ContentDigestV2::try_parse(
                    &zeros,
                    checked(&sources, 1, 10 + index),
                )
                .unwrap(),
                language_range: VersionRangeV2::try_new(
                    VersionV2::new(1, 0, 0),
                    VersionV2::new(2, 0, 0),
                    checked(&sources, 1, 12 + index),
                )
                .unwrap(),
                dependencies: vec![],
                exports: vec![],
                suites: vec![],
                location: checked(&sources, 1, 14 + index),
            })
            .unwrap();
    }
    builder
        .try_set_lockfile(
            LockReferenceInputV2 {
                path: "model.lock".into(),
                location: checked(&sources, 1, 20),
            },
            SourceIdV2::from_raw(2),
            &lock,
        )
        .unwrap();
    builder
        .try_set_model_schema(ModelSchemaInputV2 {
            schema_version: VersionV2::new(1, 0, 0),
            attributes: vec![ModelAttributeInputV2 {
                attribute: "kind".into(),
                allowed_values: vec![LocatedModelValueInputV2 {
                    value: "source".into(),
                    location: checked(&sources, 1, 21),
                }],
                location: checked(&sources, 1, 22),
            }],
            location: checked(&sources, 1, 23),
        })
        .unwrap();
    for (index, pack) in packs.iter().enumerate().rev() {
        builder
            .try_add_model_pack(ModelPackInputV2 {
                pack_id: pack.clone(),
                model_version: VersionV2::new(1, 0, 0),
                facts: vec![ModelFactInputV2 {
                    target: "framework::read".into(),
                    attribute: "kind".into(),
                    value: "source".into(),
                    location: checked(&sources, 1, 30 + index),
                }],
                location: checked(&sources, 1, 32 + index),
            })
            .unwrap();
    }
    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 40),
            },
            VersionV2::new(1, 1, 0),
        )
        .unwrap();
    let fact = parts.composed_model().unwrap().facts().next().unwrap();
    assert_eq!(
        fact.asserting_packs()
            .iter()
            .map(PackIdV2::as_str)
            .collect::<Vec<_>>(),
        vec!["a", "b"]
    );
    assert_eq!(fact.location().span, SpanV1::new(30, 31));
}

#[test]
fn conflicting_values_across_distinct_model_packs_reject_with_both_locations() {
    let zeros = "0".repeat(64);
    let lock = format!("a 1.0.0 {zeros}\nb 1.0.0 {zeros}\n");
    let sources = registry(&[(1, 1024), (2, lock.len())]);
    let app = module_id(&sources, "app", 0);
    let packs = [
        PackIdV2::try_new("a", checked(&sources, 1, 1)).unwrap(),
        PackIdV2::try_new("b", checked(&sources, 1, 2)).unwrap(),
    ];
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(), sources.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&sources, 1, 3),
        })
        .unwrap();
    for (index, pack) in packs.iter().enumerate() {
        builder
            .try_add_manifest(PackManifestInputV2 {
                pack_id: pack.clone(),
                version: VersionV2::new(1, 0, 0),
                content_digest: ContentDigestV2::try_parse(
                    &zeros,
                    checked(&sources, 1, 10 + index),
                )
                .unwrap(),
                language_range: VersionRangeV2::try_new(
                    VersionV2::new(1, 0, 0),
                    VersionV2::new(2, 0, 0),
                    checked(&sources, 1, 12 + index),
                )
                .unwrap(),
                dependencies: vec![],
                exports: vec![],
                suites: vec![],
                location: checked(&sources, 1, 14 + index),
            })
            .unwrap();
    }
    builder
        .try_set_lockfile(
            LockReferenceInputV2 {
                path: "model.lock".into(),
                location: checked(&sources, 1, 20),
            },
            SourceIdV2::from_raw(2),
            &lock,
        )
        .unwrap();
    builder
        .try_set_model_schema(ModelSchemaInputV2 {
            schema_version: VersionV2::new(1, 0, 0),
            attributes: vec![ModelAttributeInputV2 {
                attribute: "kind".into(),
                allowed_values: ["source", "sink"]
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| LocatedModelValueInputV2 {
                        value: value.into(),
                        location: checked(&sources, 1, 21 + index),
                    })
                    .collect(),
                location: checked(&sources, 1, 23),
            }],
            location: checked(&sources, 1, 24),
        })
        .unwrap();
    for (index, (pack, value)) in packs.iter().zip(["source", "sink"]).enumerate() {
        builder
            .try_add_model_pack(ModelPackInputV2 {
                pack_id: pack.clone(),
                model_version: VersionV2::new(1, 0, 0),
                facts: vec![ModelFactInputV2 {
                    target: "framework::read".into(),
                    attribute: "kind".into(),
                    value: value.into(),
                    location: checked(&sources, 1, 30 + index),
                }],
                location: checked(&sources, 1, 32 + index),
            })
            .unwrap();
    }
    let error = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&sources, 1, 40),
            },
            VersionV2::new(1, 1, 0),
        )
        .unwrap_err();
    assert_eq!(error.code(), "modules.model_conflict");
    assert!(error.primary().is_some());
    assert!(error.secondary().is_some());
}

#[test]
fn model_conflict_is_permutation_stable_and_retains_both_locations() {
    let forward = finish_model_facts(&[("source", 20), ("sink", 21)]).unwrap_err();
    let reverse = finish_model_facts(&[("sink", 21), ("source", 20)]).unwrap_err();
    assert_eq!(forward.code(), "modules.model_conflict");
    assert_eq!(reverse.code(), "modules.model_conflict");
    assert_eq!(forward.primary(), reverse.primary());
    assert_eq!(forward.secondary(), reverse.secondary());
    assert_eq!(forward.primary().unwrap().span, SpanV1::new(20, 21));
    assert_eq!(forward.secondary().unwrap().span, SpanV1::new(21, 22));
}

#[test]
fn model_value_outside_schema_rejects() {
    assert_eq!(
        finish_model_facts(&[("unknown", 20)]).unwrap_err().code(),
        "modules.invalid_model_data"
    );
}
