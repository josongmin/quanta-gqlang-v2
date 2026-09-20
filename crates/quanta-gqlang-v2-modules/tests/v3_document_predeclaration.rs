#![allow(clippy::unwrap_used, reason = "owner-local authority assertions")]

use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2,
    SpanV1,
};
use quanta_gqlang_v2_modules::*;
use quanta_gqlang_v2_types::AuthorityBudgetProfileV2;

fn registry() -> SourceRegistryViewV2 {
    SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1),
        source_len: 512,
    }])
    .unwrap()
}

fn checked(registry: &SourceRegistryViewV2, source: u32, at: usize) -> CheckedLocatedSpanV2 {
    registry
        .try_check_location(LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(source),
            span: SpanV1::new(at, at + 1),
        })
        .unwrap()
}

fn profile(nodes: usize, edges: usize) -> AuthorityBudgetProfileV2 {
    AuthorityBudgetProfileV2::try_new(nodes, edges, 10_000, 100_000, 1_024).unwrap()
}

fn authority(
    host: &SourceRegistryViewV2,
    budget: AuthorityBudgetProfileV2,
    external: &[(&str, &str, ExternalCatalogPayloadInputV2)],
) -> (ValidatedModuleAuthorityPartsV2, ModuleIdV2, ModuleIdV2) {
    let app = ModuleIdV2::try_new("app", checked(host, 1, 0)).unwrap();
    let ext = ModuleIdV2::try_new("ext", checked(host, 1, 1)).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(budget, host.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(host, 1, 2),
        })
        .unwrap();
    for (index, (kind, name, payload)) in external.iter().enumerate() {
        builder
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: "ext".into(),
                local_name: (*name).into(),
                kind_token: (*kind).into(),
                typed_payload: payload.clone(),
                provenance: "test-host".into(),
                location: checked(host, 1, index + 10),
                dependencies: Vec::new(),
            })
            .unwrap();
    }
    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app.clone(),
                location: checked(host, 1, 3),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    (parts, app, ext)
}

fn extended(host: &SourceRegistryViewV2) -> quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2 {
    host.try_extend_document(SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(9),
        source_len: 512,
    })
    .unwrap()
}

fn local(
    registry: &SourceRegistryViewV2,
    app: &ModuleIdV2,
    kind: NamespaceKindV2,
    name: &str,
    at: usize,
) -> DocumentCatalogAssetInputV2 {
    DocumentCatalogAssetInputV2 {
        owner_module_id: app.clone(),
        local_name: name.into(),
        namespace_kind: kind,
        location: checked(registry, 9, at),
    }
}

fn final_input() -> DocumentCatalogFinalInputV2 {
    DocumentCatalogFinalInputV2 {
        profile_admissions: Vec::new(),
        activations: Vec::new(),
        require_certified_directives: Vec::new(),
        dependency_edges: Vec::new(),
        family_references: Vec::new(),
    }
}

fn host_signature(host: &SourceRegistryViewV2, kind: CallableKindV2) -> SignatureInputV2 {
    SignatureInputV2 {
        parameters: vec![LocatedParameterInputV2 {
            name: "x".into(),
            value_type: quanta_gqlang_v2_types::ValueTypeV1::int(),
            location: checked(host, 1, 8),
        }],
        result: quanta_gqlang_v2_types::ValueTypeV1::bool(),
        callable_kind: kind,
        effects: quanta_gqlang_v2_types::ExpressionEffectsV2::pure_scalar(),
        location: checked(host, 1, 9),
    }
}

#[test]
fn all_six_family_references_resolve_exact_host_kinds_and_forward_paths() {
    let host = registry();
    let targets = [
        (
            DocumentCatalogReferenceFamilyV2::ViewQuery,
            "query",
            "queries.catalog.users",
            ExternalCatalogPayloadInputV2::Query,
        ),
        (
            DocumentCatalogReferenceFamilyV2::ModelSource,
            "model_source",
            "models/code",
            ExternalCatalogPayloadInputV2::ModelSource,
        ),
        (
            DocumentCatalogReferenceFamilyV2::PackSource,
            "pack_source",
            "registry::security",
            ExternalCatalogPayloadInputV2::PackSource,
        ),
        (
            DocumentCatalogReferenceFamilyV2::ProfileCapability,
            "capability",
            "graph.read",
            ExternalCatalogPayloadInputV2::Capability,
        ),
        (
            DocumentCatalogReferenceFamilyV2::GateCheck,
            "check",
            "checks.authz",
            ExternalCatalogPayloadInputV2::Check,
        ),
        (
            DocumentCatalogReferenceFamilyV2::RuleBindingCheck,
            "check",
            "checks/shell-exec",
            ExternalCatalogPayloadInputV2::Check,
        ),
    ];
    let external = targets
        .iter()
        .map(|(_, kind, name, payload)| (*kind, *name, payload.clone()))
        .collect::<Vec<_>>();
    let (parts, app, ext) = authority(&host, profile(100, 100), &external);
    let registry = extended(&host);
    let source_names = [
        "view.a",
        "model.a",
        "pack-a",
        "profile.a",
        "gate.a",
        "ruledef.a",
    ];
    let assets = targets
        .iter()
        .enumerate()
        .map(|(index, (family, _, _, _))| {
            local(
                registry.combined(),
                &app,
                family.source_kind(),
                source_names.get(index).unwrap(),
                index + 20,
            )
        })
        .collect();
    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    let predeclared = validator.predeclare_assets(assets).unwrap();
    let mut input = final_input();
    for (index, (family, _, target_name, _)) in targets.iter().enumerate() {
        let source = predeclared
            .iter()
            .find(|asset| asset.kind() == family.source_kind())
            .unwrap()
            .clone();
        let target = validator
            .resolve_reference(DocumentCatalogReferenceInputV2 {
                module_id: Some(ext.clone()),
                local_name: (*target_name).into(),
                expected_kind: family.target_kind(),
                location: checked(registry.combined(), 9, index + 40),
            })
            .unwrap();
        assert_eq!(target.kind(), family.target_kind());
        assert_eq!(target.id().local_name(), *target_name);
        input
            .family_references
            .push(DocumentCatalogFamilyReferenceInputV2 {
                family: *family,
                source,
                target,
                location: checked(registry.combined(), 9, index + 60),
            });
    }
    input.profile_admissions.push(ProfileAdmissionInputV2 {
        profile: CatalogAssetRefInputV2 {
            module_id: app,
            namespace_kind: NamespaceKindV2::Profile,
            local_name: "profile.a".into(),
            location: checked(registry.combined(), 9, 23),
        },
        admission: ProfileAdmissionV2::CertifiedOnly,
        location: checked(registry.combined(), 9, 70),
    });
    let proof = validator.validate_predeclared(input).unwrap();
    assert_eq!(proof.assets().count(), 6);
    assert_eq!(proof.dependencies().len(), 6);
    assert!(proof
        .dependencies()
        .iter()
        .all(|edge| edge.family().is_some()));
    assert_eq!(
        validator
            .validate_predeclared(final_input())
            .unwrap_err()
            .code(),
        "modules.document_catalog_phase_error"
    );
}

#[test]
fn each_family_rejects_wrong_kind_and_unknown_target() {
    let host = registry();
    let (parts, app, ext) = authority(
        &host,
        profile(100, 100),
        &[(
            "value",
            "wrong.target",
            ExternalCatalogPayloadInputV2::Value(quanta_gqlang_v2_types::ValueTypeV1::int()),
        )],
    );
    let registry = extended(&host);
    for (index, family) in [
        DocumentCatalogReferenceFamilyV2::ViewQuery,
        DocumentCatalogReferenceFamilyV2::ModelSource,
        DocumentCatalogReferenceFamilyV2::PackSource,
        DocumentCatalogReferenceFamilyV2::ProfileCapability,
        DocumentCatalogReferenceFamilyV2::GateCheck,
        DocumentCatalogReferenceFamilyV2::RuleBindingCheck,
    ]
    .into_iter()
    .enumerate()
    {
        let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
        validator
            .predeclare_assets(vec![local(
                registry.combined(),
                &app,
                family.source_kind(),
                "local.owner",
                20 + index,
            )])
            .unwrap();
        let wrong = validator
            .resolve_reference(DocumentCatalogReferenceInputV2 {
                module_id: Some(ext.clone()),
                local_name: "wrong.target".into(),
                expected_kind: family.target_kind(),
                location: checked(registry.combined(), 9, 40 + index),
            })
            .unwrap_err();
        assert_eq!(wrong.code(), "modules.catalog_reference_kind_mismatch");
        assert_eq!(wrong.primary().unwrap().span.start, 40 + index);
        let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
        validator.predeclare_assets(Vec::new()).unwrap();
        assert_eq!(
            validator
                .resolve_reference(DocumentCatalogReferenceInputV2 {
                    module_id: Some(ext.clone()),
                    local_name: "missing.target".into(),
                    expected_kind: family.target_kind(),
                    location: checked(registry.combined(), 9, 60 + index),
                })
                .unwrap_err()
                .code(),
            "modules.unknown_catalog_reference"
        );
    }
}

#[test]
fn predeclaration_rejects_duplicate_and_preserves_catalog_path_spelling() {
    let host = registry();
    let (parts, app, _) = authority(&host, profile(100, 100), &[]);
    let registry = extended(&host);
    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    let error = validator
        .predeclare_assets(vec![
            local(
                registry.combined(),
                &app,
                NamespaceKindV2::View,
                "catalog.users",
                20,
            ),
            local(
                registry.combined(),
                &app,
                NamespaceKindV2::View,
                "catalog.users",
                21,
            ),
        ])
        .unwrap_err();
    assert_eq!(error.code(), "modules.duplicate_document_catalog_asset");
    assert_eq!(error.primary().unwrap().span.start, 21);
    assert_eq!(error.secondary().unwrap().span.start, 20);
    for name in [
        "catalog.users",
        "models/code",
        "repo::auth",
        "secure-pack",
        "9",
        "a:b",
    ] {
        let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
        let assets = validator
            .predeclare_assets(vec![local(
                registry.combined(),
                &app,
                NamespaceKindV2::View,
                name,
                30,
            )])
            .unwrap();
        assert_eq!(assets.first().unwrap().id().local_name(), name);
    }
    for name in [".bad", "a..b", "a//b", "a::", "a-", "a b", "é"] {
        let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
        assert_eq!(
            validator
                .predeclare_assets(vec![local(
                    registry.combined(),
                    &app,
                    NamespaceKindV2::View,
                    name,
                    31
                )])
                .unwrap_err()
                .code(),
            "modules.invalid_catalog_name"
        );
    }
}

#[test]
fn one_session_budget_charges_predeclaration_and_final_family_edge() {
    let host = registry();
    for (limit, expected_error) in [(2, None), (1, Some(("edges", 2)))] {
        let (parts, app, _) = authority(&host, profile(10, limit), &[]);
        let registry = extended(&host);
        let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
        let assets = validator
            .predeclare_assets(vec![
                local(
                    registry.combined(),
                    &app,
                    NamespaceKindV2::Profile,
                    "profile.a",
                    20,
                ),
                local(
                    registry.combined(),
                    &app,
                    NamespaceKindV2::Capability,
                    "graph.read",
                    21,
                ),
            ])
            .unwrap();
        let source = assets
            .iter()
            .find(|asset| asset.kind() == NamespaceKindV2::Profile)
            .unwrap()
            .clone();
        let target = validator
            .resolve_reference(DocumentCatalogReferenceInputV2 {
                module_id: None,
                local_name: "graph.read".into(),
                expected_kind: NamespaceKindV2::Capability,
                location: checked(registry.combined(), 9, 30),
            })
            .unwrap();
        let mut input = final_input();
        input.profile_admissions.push(ProfileAdmissionInputV2 {
            profile: CatalogAssetRefInputV2 {
                module_id: app,
                namespace_kind: NamespaceKindV2::Profile,
                local_name: "profile.a".into(),
                location: checked(registry.combined(), 9, 20),
            },
            admission: ProfileAdmissionV2::CertifiedOnly,
            location: checked(registry.combined(), 9, 31),
        });
        input
            .family_references
            .push(DocumentCatalogFamilyReferenceInputV2 {
                family: DocumentCatalogReferenceFamilyV2::ProfileCapability,
                source,
                target,
                location: checked(registry.combined(), 9, 32),
            });
        match expected_error {
            None => assert_eq!(
                validator
                    .validate_predeclared(input)
                    .unwrap()
                    .dependencies()
                    .len(),
                1
            ),
            Some((dimension, observed)) => {
                let error = validator.validate_predeclared(input).unwrap_err();
                let resource = error.resource_exhaustion().unwrap();
                assert_eq!(resource.dimension(), dimension);
                assert_eq!(resource.limit(), limit);
                assert_eq!(resource.observed(), observed);
            }
        }
    }
}

#[test]
fn imported_path_is_visible_and_local_collision_is_ambiguous() {
    let host = registry();
    let app = ModuleIdV2::try_new("app", checked(&host, 1, 0)).unwrap();
    let lib = ModuleIdV2::try_new("lib", checked(&host, 1, 1)).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(100, 100), host.clone()).unwrap();
    for (at, module) in [(2, app.clone()), (3, lib.clone())] {
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: module,
                location: checked(&host, 1, at),
            })
            .unwrap();
    }
    builder
        .try_declare(ModuleDeclarationInputV2 {
            owner_module_id: lib.clone(),
            local_name: "queries.catalog.users".into(),
            namespace_kind: NamespaceKindV2::Query,
            visibility: VisibilityV2::Public,
            location: checked(&host, 1, 4),
        })
        .unwrap();
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: app.clone(),
            local_name: "queries.catalog.users".into(),
            target: CatalogAssetRefInputV2 {
                module_id: lib.clone(),
                namespace_kind: NamespaceKindV2::Query,
                local_name: "queries.catalog.users".into(),
                location: checked(&host, 1, 5),
            },
            location: checked(&host, 1, 6),
        })
        .unwrap();
    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app.clone(),
                location: checked(&host, 1, 7),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let registry = extended(&host);
    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    validator.predeclare_assets(Vec::new()).unwrap();
    let imported = validator
        .resolve_reference(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: "queries.catalog.users".into(),
            expected_kind: NamespaceKindV2::Query,
            location: checked(registry.combined(), 9, 20),
        })
        .unwrap();
    assert_eq!(
        imported.origin(),
        DocumentCatalogReferenceOriginV2::VisibleHost
    );
    assert_eq!(imported.id().module_id(), &lib);

    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    validator
        .predeclare_assets(vec![local(
            registry.combined(),
            &app,
            NamespaceKindV2::Query,
            "queries.catalog.users",
            21,
        )])
        .unwrap();
    let error = validator
        .resolve_reference(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: "queries.catalog.users".into(),
            expected_kind: NamespaceKindV2::Query,
            location: checked(registry.combined(), 9, 22),
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.ambiguous_catalog_reference");
    assert_eq!(error.primary().unwrap().span.start, 22);
}

#[test]
fn local_document_identity_cannot_duplicate_visible_or_external_host_declaration() {
    let host = registry();
    let app = ModuleIdV2::try_new("app", checked(&host, 1, 0)).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(100, 100), host.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&host, 1, 1),
        })
        .unwrap();
    builder
        .try_declare(ModuleDeclarationInputV2 {
            owner_module_id: app.clone(),
            local_name: "profile.local".into(),
            namespace_kind: NamespaceKindV2::Profile,
            visibility: VisibilityV2::Public,
            location: checked(&host, 1, 2),
        })
        .unwrap();
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "app".into(),
            local_name: "checks.host".into(),
            kind_token: "check".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Check,
            provenance: "test-host".into(),
            location: checked(&host, 1, 3),
            dependencies: Vec::new(),
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
    let registry = extended(&host);
    for (kind, name, at, host_at) in [
        (NamespaceKindV2::Profile, "profile.local", 20, 2),
        (NamespaceKindV2::Check, "checks.host", 21, 3),
    ] {
        let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
        let error = validator
            .predeclare_assets(vec![local(registry.combined(), &app, kind, name, at)])
            .unwrap_err();
        assert_eq!(error.code(), "modules.duplicate_document_catalog_asset");
        assert_eq!(error.primary().unwrap().span.start, at);
        assert_eq!(error.secondary().unwrap().span.start, host_at);
    }
}

#[test]
fn document_predeclaration_cannot_mint_asset_in_foreign_module() {
    let host = registry();
    let (parts, _, foreign) = authority(&host, profile(100, 100), &[]);
    let registry = extended(&host);
    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    let error = validator
        .predeclare_assets(vec![local(
            registry.combined(),
            &foreign,
            NamespaceKindV2::View,
            "view.foreign",
            20,
        )])
        .unwrap_err();
    assert_eq!(error.code(), "modules.document_asset_wrong_module");
    assert_eq!(error.primary().unwrap().span.start, 20);
}

#[test]
fn optional_rule_probe_preserves_session_on_absence_in_both_orders() {
    let host = registry();
    let signature = SignatureInputV2 {
        parameters: Vec::new(),
        result: quanta_gqlang_v2_types::ValueTypeV1::bool(),
        callable_kind: CallableKindV2::Rule,
        effects: quanta_gqlang_v2_types::ExpressionEffectsV2::pure_scalar(),
        location: checked(&host, 1, 8),
    };
    let (parts, _, ext) = authority(
        &host,
        profile(100, 100),
        &[
            (
                "rule",
                "rule.present",
                ExternalCatalogPayloadInputV2::Rule(signature),
            ),
            ("check", "rule.wrong", ExternalCatalogPayloadInputV2::Check),
        ],
    );
    let registry = extended(&host);
    for names in [
        ["rule.absent", "rule.present"],
        ["rule.present", "rule.absent"],
    ] {
        let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
        validator.predeclare_assets(Vec::new()).unwrap();
        for (index, name) in names.into_iter().enumerate() {
            let result = validator
                .probe_reference(DocumentCatalogReferenceInputV2 {
                    module_id: Some(ext.clone()),
                    local_name: name.into(),
                    expected_kind: NamespaceKindV2::Rule,
                    location: checked(registry.combined(), 9, 20 + index),
                })
                .unwrap();
            assert_eq!(result.is_some(), name == "rule.present");
        }
        validator.validate_predeclared(final_input()).unwrap();
    }
    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    validator.predeclare_assets(Vec::new()).unwrap();
    let error = validator
        .probe_reference(DocumentCatalogReferenceInputV2 {
            module_id: Some(ext),
            local_name: "rule.wrong".into(),
            expected_kind: NamespaceKindV2::Rule,
            location: checked(registry.combined(), 9, 30),
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.catalog_reference_kind_mismatch");
    assert_eq!(
        validator
            .validate_predeclared(final_input())
            .unwrap_err()
            .code(),
        "modules.document_catalog_phase_error"
    );
}

#[test]
fn repeated_host_identity_requires_the_same_declaration_origin() {
    let host = registry();
    let app = ModuleIdV2::try_new("app", checked(&host, 1, 0)).unwrap();
    for (external_at, expected_error) in
        [(2, None), (3, Some("modules.duplicate_host_catalog_asset"))]
    {
        let mut builder =
            ModuleAuthorityBuilderV2::try_new(profile(100, 100), host.clone()).unwrap();
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: app.clone(),
                location: checked(&host, 1, 1),
            })
            .unwrap();
        builder
            .try_declare(ModuleDeclarationInputV2 {
                owner_module_id: app.clone(),
                local_name: "checks.authz".into(),
                namespace_kind: NamespaceKindV2::Check,
                visibility: VisibilityV2::Public,
                location: checked(&host, 1, 2),
            })
            .unwrap();
        builder
            .try_add_external(ExternalCatalogAssetInputV2 {
                module_name: "app".into(),
                local_name: "checks.authz".into(),
                kind_token: "check".into(),
                typed_payload: ExternalCatalogPayloadInputV2::Check,
                provenance: "test-host".into(),
                location: checked(&host, 1, external_at),
                dependencies: Vec::new(),
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
        let registry = extended(&host);
        let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
        validator.predeclare_assets(Vec::new()).unwrap();
        let result = validator.resolve_reference(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: "checks.authz".into(),
            expected_kind: NamespaceKindV2::Check,
            location: checked(registry.combined(), 9, 20),
        });
        if let Some(code) = expected_error {
            let error = result.unwrap_err();
            assert_eq!(error.code(), code);
            assert_eq!(error.primary().unwrap().span.start, external_at);
            assert_eq!(error.secondary().unwrap().span.start, 2);
        } else {
            let reference = result.unwrap();
            assert_eq!(
                reference.origin(),
                DocumentCatalogReferenceOriginV2::VisibleHost
            );
        }
    }
}

#[test]
fn view_cycle_remains_view_only_and_foreign_registry_rejects() {
    let host = registry();
    let (parts, app, _) = authority(&host, profile(100, 100), &[]);
    let registry = extended(&host);
    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    validator
        .predeclare_assets(vec![
            local(
                registry.combined(),
                &app,
                NamespaceKindV2::View,
                "view.a",
                20,
            ),
            local(
                registry.combined(),
                &app,
                NamespaceKindV2::View,
                "view.b",
                21,
            ),
        ])
        .unwrap();
    let mut input = final_input();
    for (from, to, at) in [("view.a", "view.b", 30), ("view.b", "view.a", 31)] {
        input
            .dependency_edges
            .push(DocumentCatalogDependencyInputV2 {
                source: CatalogAssetRefInputV2 {
                    module_id: app.clone(),
                    namespace_kind: NamespaceKindV2::View,
                    local_name: from.into(),
                    location: checked(registry.combined(), 9, at),
                },
                target: CatalogAssetRefInputV2 {
                    module_id: app.clone(),
                    namespace_kind: NamespaceKindV2::View,
                    local_name: to.into(),
                    location: checked(registry.combined(), 9, at),
                },
                expected_target_kind: NamespaceKindV2::View,
                location: checked(registry.combined(), 9, at),
            });
    }
    let error = validator.validate_predeclared(input).unwrap_err();
    assert_eq!(error.code(), "modules.view_inline_cycle");
    assert_eq!(error.edge_witness().len(), 2);

    let other = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(10),
            source_len: 512,
        })
        .unwrap();
    let mut validator = DocumentCatalogValidatorV2::try_new(registry, &parts).unwrap();
    let error = validator
        .predeclare_assets(vec![DocumentCatalogAssetInputV2 {
            owner_module_id: app,
            local_name: "view.a".into(),
            namespace_kind: NamespaceKindV2::View,
            location: checked(other.combined(), 10, 20),
        }])
        .unwrap_err();
    assert_eq!(error.code(), "modules.source_registry_mismatch");
}

#[test]
fn same_spelling_cross_kind_is_ambiguous_before_expected_kind_selection() {
    let host = registry();
    let (parts, app, _) = authority(&host, profile(100, 100), &[]);
    let registry = extended(&host);
    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    validator
        .predeclare_assets(vec![
            local(
                registry.combined(),
                &app,
                NamespaceKindV2::Query,
                "same.path",
                20,
            ),
            local(
                registry.combined(),
                &app,
                NamespaceKindV2::Check,
                "same.path",
                21,
            ),
        ])
        .unwrap();
    let error = validator
        .resolve_reference(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: "same.path".into(),
            expected_kind: NamespaceKindV2::Query,
            location: checked(registry.combined(), 9, 22),
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.ambiguous_catalog_reference");
    assert_eq!(error.ambiguous_candidates().len(), 2);
    assert_eq!(error.primary().unwrap().span.start, 22);
}

#[test]
fn final_family_refs_must_come_from_the_same_validator_session() {
    let host = registry();
    let (parts, app, _) = authority(&host, profile(100, 100), &[]);
    let registry = extended(&host);
    let assets = || {
        vec![
            local(
                registry.combined(),
                &app,
                NamespaceKindV2::View,
                "view.a",
                20,
            ),
            local(
                registry.combined(),
                &app,
                NamespaceKindV2::Query,
                "query.a",
                21,
            ),
        ]
    };
    let reference = || DocumentCatalogReferenceInputV2 {
        module_id: None,
        local_name: "query.a".into(),
        expected_kind: NamespaceKindV2::Query,
        location: checked(registry.combined(), 9, 30),
    };
    let mut first = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    let first_assets = first.predeclare_assets(assets()).unwrap();
    let first_source = first_assets
        .iter()
        .find(|asset| asset.kind() == NamespaceKindV2::View)
        .unwrap()
        .clone();
    let first_target = first.resolve_reference(reference()).unwrap();

    let mut second = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    let second_assets = second.predeclare_assets(assets()).unwrap();
    let second_target = second.resolve_reference(reference()).unwrap();
    let mut input = final_input();
    input
        .family_references
        .push(DocumentCatalogFamilyReferenceInputV2 {
            family: DocumentCatalogReferenceFamilyV2::ViewQuery,
            source: first_source,
            target: second_target,
            location: checked(registry.combined(), 9, 40),
        });
    assert_eq!(
        second.validate_predeclared(input).unwrap_err().code(),
        "modules.catalog_reference_session_mismatch"
    );

    let mut third = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    let third_assets = third.predeclare_assets(assets()).unwrap();
    let third_source = third_assets
        .iter()
        .find(|asset| asset.kind() == NamespaceKindV2::View)
        .unwrap()
        .clone();
    let mut input = final_input();
    input
        .family_references
        .push(DocumentCatalogFamilyReferenceInputV2 {
            family: DocumentCatalogReferenceFamilyV2::ViewQuery,
            source: third_source,
            target: first_target,
            location: checked(registry.combined(), 9, 41),
        });
    assert_eq!(
        third.validate_predeclared(input).unwrap_err().code(),
        "modules.catalog_reference_session_mismatch"
    );
    assert_eq!(second_assets.len(), 2);
}

#[test]
fn forward_local_query_reference_seals_after_predeclaration() {
    let host = registry();
    let (parts, app, _) = authority(&host, profile(100, 100), &[]);
    let registry = extended(&host);
    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    // The View is authored first; its target arrives later in the input list.
    let assets = validator
        .predeclare_assets(vec![
            local(
                registry.combined(),
                &app,
                NamespaceKindV2::View,
                "view.a",
                20,
            ),
            local(
                registry.combined(),
                &app,
                NamespaceKindV2::Query,
                "query.later",
                21,
            ),
        ])
        .unwrap();
    let source = assets
        .iter()
        .find(|asset| asset.kind() == NamespaceKindV2::View)
        .unwrap()
        .clone();
    let target = validator
        .resolve_reference(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: "query.later".into(),
            expected_kind: NamespaceKindV2::Query,
            location: checked(registry.combined(), 9, 30),
        })
        .unwrap();
    assert_eq!(target.origin(), DocumentCatalogReferenceOriginV2::Local);
    let mut input = final_input();
    input
        .family_references
        .push(DocumentCatalogFamilyReferenceInputV2 {
            family: DocumentCatalogReferenceFamilyV2::ViewQuery,
            source,
            target,
            location: checked(registry.combined(), 9, 30),
        });
    let proof = validator.validate_predeclared(input).unwrap();
    assert_eq!(proof.dependencies().len(), 1);
    assert_eq!(
        proof
            .dependencies()
            .first()
            .unwrap()
            .target_asset_id()
            .local_name(),
        "query.later"
    );
}

#[test]
fn host_predicate_proof_preserves_exact_catalog_id_signature_and_session() {
    use quanta_gqlang_v2_types::{HostCallableNamespaceV2, ResolvedHostPredicateKeyV2};

    let host = registry();
    let authored = "policy/checks::is-safe.v2";
    let (parts, _, ext) = authority(
        &host,
        profile(100, 100),
        &[(
            "predicate",
            authored,
            ExternalCatalogPayloadInputV2::Predicate(host_signature(
                &host,
                CallableKindV2::Predicate,
            )),
        )],
    );
    let registry = extended(&host);
    let call_location = checked(registry.combined(), 9, 41);
    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    validator.predeclare_assets(Vec::new()).unwrap();
    let proof = validator
        .resolve_host_predicate_call(DocumentCatalogReferenceInputV2 {
            module_id: Some(ext.clone()),
            local_name: authored.into(),
            expected_kind: NamespaceKindV2::Predicate,
            location: call_location.clone(),
        })
        .unwrap();
    assert_eq!(proof.id().module_id(), &ext);
    assert_eq!(proof.id().local_name(), authored);
    assert_eq!(proof.origin(), DocumentCatalogReferenceOriginV2::External);
    assert_eq!(proof.location(), &call_location);
    assert_eq!(proof.signature().parameters().len(), 1);
    assert_eq!(
        proof.callable().params(),
        &[quanta_gqlang_v2_types::ValueTypeV1::int()]
    );
    let encoded_len = ResolvedHostPredicateKeyV2::checked_canonical_len(
        ext.as_str(),
        HostCallableNamespaceV2::Predicate,
        authored,
    )
    .unwrap();
    let exact_key = ResolvedHostPredicateKeyV2::try_from_catalog_parts(
        ext.as_str(),
        HostCallableNamespaceV2::Predicate,
        authored,
        encoded_len,
    )
    .unwrap();
    assert_eq!(proof.key(), &exact_key);
    validator
        .validate_host_predicate_proof(&proof, proof.key(), &call_location)
        .unwrap();

    let wrong_location = checked(registry.combined(), 9, 42);
    let error = validator
        .validate_host_predicate_proof(&proof, proof.key(), &wrong_location)
        .unwrap_err();
    assert_eq!(error.code(), "modules.host_predicate_proof_mismatch");

    let mut wrong_target_validator =
        DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    wrong_target_validator
        .predeclare_assets(Vec::new())
        .unwrap();
    let wrong_target_proof = wrong_target_validator
        .resolve_host_predicate_call(DocumentCatalogReferenceInputV2 {
            module_id: Some(ext.clone()),
            local_name: authored.into(),
            expected_kind: NamespaceKindV2::Predicate,
            location: call_location.clone(),
        })
        .unwrap();
    let wrong_target_name = "policy/checks::another-safe.v2";
    let wrong_target_len = ResolvedHostPredicateKeyV2::checked_canonical_len(
        ext.as_str(),
        HostCallableNamespaceV2::Predicate,
        wrong_target_name,
    )
    .unwrap();
    let wrong_target_key = ResolvedHostPredicateKeyV2::try_from_catalog_parts(
        ext.as_str(),
        HostCallableNamespaceV2::Predicate,
        wrong_target_name,
        wrong_target_len,
    )
    .unwrap();
    let error = wrong_target_validator
        .validate_host_predicate_proof(&wrong_target_proof, &wrong_target_key, &call_location)
        .unwrap_err();
    assert_eq!(error.code(), "modules.host_predicate_proof_mismatch");

    let mut other = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    other.predeclare_assets(Vec::new()).unwrap();
    let error = other
        .validate_host_predicate_proof(&proof, proof.key(), &call_location)
        .unwrap_err();
    assert_eq!(error.code(), "modules.source_registry_mismatch");
}

#[test]
fn host_predicate_rejects_wrong_kind_ambiguity_and_missing_signature() {
    let host = registry();
    let (parts, _, ext) = authority(
        &host,
        profile(100, 100),
        &[
            (
                "rule",
                "same.name",
                ExternalCatalogPayloadInputV2::Rule(host_signature(&host, CallableKindV2::Rule)),
            ),
            (
                "predicate",
                "ambiguous.name",
                ExternalCatalogPayloadInputV2::Predicate(host_signature(
                    &host,
                    CallableKindV2::Predicate,
                )),
            ),
            (
                "rule",
                "ambiguous.name",
                ExternalCatalogPayloadInputV2::Rule(host_signature(&host, CallableKindV2::Rule)),
            ),
        ],
    );
    let registry = extended(&host);
    for (name, code) in [
        ("same.name", "modules.catalog_reference_kind_mismatch"),
        ("ambiguous.name", "modules.ambiguous_catalog_reference"),
    ] {
        let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
        validator.predeclare_assets(Vec::new()).unwrap();
        let error = validator
            .resolve_host_predicate_call(DocumentCatalogReferenceInputV2 {
                module_id: Some(ext.clone()),
                local_name: name.into(),
                expected_kind: NamespaceKindV2::Predicate,
                location: checked(registry.combined(), 9, 50),
            })
            .unwrap_err();
        assert_eq!(error.code(), code);
    }

    let app = ModuleIdV2::try_new("app", checked(&host, 1, 0)).unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(100, 100), host.clone()).unwrap();
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: app.clone(),
            location: checked(&host, 1, 1),
        })
        .unwrap();
    builder
        .try_declare(ModuleDeclarationInputV2 {
            owner_module_id: app.clone(),
            local_name: "host-only".into(),
            namespace_kind: NamespaceKindV2::Predicate,
            visibility: VisibilityV2::Public,
            location: checked(&host, 1, 2),
        })
        .unwrap();
    let visible_only = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&host, 1, 3),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let mut validator =
        DocumentCatalogValidatorV2::try_new(registry.clone(), &visible_only).unwrap();
    validator.predeclare_assets(Vec::new()).unwrap();
    let error = validator
        .resolve_host_predicate_call(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: "host-only".into(),
            expected_kind: NamespaceKindV2::Predicate,
            location: checked(registry.combined(), 9, 51),
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.missing_host_predicate_signature");
}

#[test]
fn imported_predicate_alias_uses_underlying_external_id_and_unproven_alias_rejects() {
    let host = registry();
    let app = ModuleIdV2::try_new("app", checked(&host, 1, 0)).unwrap();
    let ext = ModuleIdV2::try_new("ext", checked(&host, 1, 1)).unwrap();
    let actual = "policy/checks::is-safe.v2";
    let alias = "safe_check";
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile(100, 100), host.clone()).unwrap();
    for (at, module) in [(2, app.clone()), (3, ext.clone())] {
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: module,
                location: checked(&host, 1, at),
            })
            .unwrap();
    }
    builder
        .try_declare(ModuleDeclarationInputV2 {
            owner_module_id: ext.clone(),
            local_name: actual.into(),
            namespace_kind: NamespaceKindV2::Predicate,
            visibility: VisibilityV2::Public,
            location: checked(&host, 1, 4),
        })
        .unwrap();
    builder
        .try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "ext".into(),
            local_name: actual.into(),
            kind_token: "predicate".into(),
            typed_payload: ExternalCatalogPayloadInputV2::Predicate(host_signature(
                &host,
                CallableKindV2::Predicate,
            )),
            provenance: "test-host".into(),
            location: checked(&host, 1, 4),
            dependencies: Vec::new(),
        })
        .unwrap();
    builder
        .try_import(ModuleImportInputV2 {
            owner_module_id: app.clone(),
            local_name: alias.into(),
            target: CatalogAssetRefInputV2 {
                module_id: ext.clone(),
                namespace_kind: NamespaceKindV2::Predicate,
                local_name: actual.into(),
                location: checked(&host, 1, 5),
            },
            location: checked(&host, 1, 6),
        })
        .unwrap();
    let parts = builder
        .finish(
            CurrentModuleSelectionInputV2 {
                module_id: app,
                location: checked(&host, 1, 7),
            },
            VersionV2::new(1, 0, 0),
        )
        .unwrap();
    let registry = extended(&host);
    let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
    validator.predeclare_assets(Vec::new()).unwrap();
    let proof = validator
        .resolve_host_predicate_call(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: alias.into(),
            expected_kind: NamespaceKindV2::Predicate,
            location: checked(registry.combined(), 9, 60),
        })
        .unwrap();
    assert_eq!(
        proof.origin(),
        DocumentCatalogReferenceOriginV2::VisibleHost
    );
    assert_eq!(proof.id().module_id(), &ext);
    assert_eq!(proof.id().local_name(), actual);
    let checked_call = checked(registry.combined(), 9, 60);
    validator
        .validate_host_predicate_proof(&proof, proof.key(), &checked_call)
        .unwrap();
    let error = validator
        .resolve_host_predicate_call(DocumentCatalogReferenceInputV2 {
            module_id: None,
            local_name: "unimported".into(),
            expected_kind: NamespaceKindV2::Predicate,
            location: checked(registry.combined(), 9, 61),
        })
        .unwrap_err();
    assert_eq!(error.code(), "modules.unknown_catalog_reference");
}

#[test]
fn long_host_predicate_key_charges_exact_encoded_bytes_before_allocation() {
    use quanta_gqlang_v2_types::{HostCallableNamespaceV2, ResolvedHostPredicateKeyV2};

    let host = registry();
    let local_name = format!("{}.b/c::d-e", "a".repeat(300));
    let encoded_len = ResolvedHostPredicateKeyV2::checked_canonical_len(
        "ext",
        HostCallableNamespaceV2::Predicate,
        &local_name,
    )
    .unwrap();
    // The document session charges the authored reference, the two exact
    // catalog components, then the canonical byte encoding before allocation.
    let exact = local_name.len() + "ext".len() + local_name.len() + encoded_len;
    for (limit, accepted) in [(exact, true), (exact - 1, false)] {
        let budget = AuthorityBudgetProfileV2::try_new(100, 100, 10_000, limit, 1_024).unwrap();
        let (parts, _, ext) = authority(
            &host,
            budget,
            &[(
                "predicate",
                &local_name,
                ExternalCatalogPayloadInputV2::Predicate(host_signature(
                    &host,
                    CallableKindV2::Predicate,
                )),
            )],
        );
        let registry = extended(&host);
        let mut validator = DocumentCatalogValidatorV2::try_new(registry.clone(), &parts).unwrap();
        validator.predeclare_assets(Vec::new()).unwrap();
        let result = validator.resolve_host_predicate_call(DocumentCatalogReferenceInputV2 {
            module_id: Some(ext),
            local_name: local_name.clone(),
            expected_kind: NamespaceKindV2::Predicate,
            location: checked(registry.combined(), 9, 70),
        });
        if accepted {
            let proof = result.unwrap();
            assert_eq!(proof.key().canonical_bytes().len(), encoded_len);
            assert_eq!(proof.id().local_name(), local_name);
        } else {
            let error = result.unwrap_err();
            assert_eq!(error.code(), "modules.resource_exhausted");
            let exhaustion = error.resource_exhaustion().unwrap();
            assert_eq!(exhaustion.dimension(), "authority_bytes");
            assert_eq!(exhaustion.limit(), limit);
            assert_eq!(exhaustion.observed(), exact);
        }
    }
}
