use super::*;

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
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(9),
            span: SpanV1::new(1, 2),
        }
    );
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
    assert_eq!(duplicate_asset.primary().unwrap().span, SpanV1::new(2, 3));
    assert_eq!(duplicate_asset.secondary().unwrap().span, SpanV1::new(1, 2));

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
    assert_eq!(duplicate_profile.primary().unwrap().span, SpanV1::new(3, 4));
    assert_eq!(
        duplicate_profile.secondary().unwrap().span,
        SpanV1::new(3, 4)
    );

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
    assert_eq!(
        error.edge_witness().to_vec(),
        [LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(9),
            span: SpanV1::new(3, 4),
        }]
    );
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
    assert_eq!(
        forward,
        [
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(21, 22),
            },
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: SpanV1::new(20, 21),
            },
        ]
    );
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
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(9),
            span: SpanV1::new(1, 2),
        }
    );
    assert_eq!(
        error.secondary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(7, 8),
        }
    );
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
    assert_eq!(
        error.primary().unwrap(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(9),
            span: SpanV1::new(3, 4),
        }
    );
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
    assert_eq!(error.primary().unwrap().span, SpanV1::new(4, 5));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(2, 3));
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
    assert_eq!(mutations.len(), cases.len());
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

    let accepted = run(2).unwrap();
    assert_eq!(accepted.resolved_graph().current_module().as_str(), "app");
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
    assert_eq!(facts.first().unwrap().target(), "framework::read");
    assert_eq!(facts.first().unwrap().attribute(), "kind");
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
    assert_eq!(parts.composed_model().unwrap().facts().count(), 1);
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
    assert_eq!(error.primary().unwrap().span, SpanV1::new(31, 32));
    assert_eq!(error.secondary().unwrap().span, SpanV1::new(30, 31));
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
    let error = finish_model_facts(&[("unknown", 20)]).unwrap_err();
    assert_eq!(error.code(), "modules.invalid_model_data");
    assert_eq!(error.primary().unwrap().span, SpanV1::new(20, 21));
}
