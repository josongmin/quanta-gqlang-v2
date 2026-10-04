#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

const OWNER_LOCAL_AUTHORITY_CHAIN_LEN: usize = 16;
const STACK_SAFETY_AUTHORITY_CHAIN_LEN: usize = 256;
const STACK_SAFETY_CHILD: &str = "QGLANG_MODULES_STACK_SAFETY_CHILD";

#[path = "../../../tools/test_support/process.rs"]
mod process;

fn chain_steps(nodes: usize) -> usize {
    nodes * (nodes - 1) / 2 + nodes - 1
}

fn seal_module_chain(
    depth: usize,
    reverse: bool,
    max_steps: usize,
) -> Result<Vec<String>, ModuleDiagnosticV2> {
    let sources = registry(&[(1, depth * 8 + 32)]);
    let ids = (0..depth)
        .map(|index| module_id(&sources, &format!("m{index:04}"), index * 4))
        .collect::<Vec<_>>();
    let mut order = (0..depth).collect::<Vec<_>>();
    if reverse {
        order.reverse();
    }
    let mut builder = ModuleAuthorityBuilderV2::try_new(
        AuthorityBudgetProfileV2::try_new(
            depth * 2,
            depth - 1,
            max_steps,
            16 * 1024 * 1024,
            64 * 1024,
        )
        .unwrap(),
        sources.clone(),
    )?;
    for &index in &order {
        builder.try_add_module(ModuleNodeInputV2 {
            module_id: ids.get(index).expect("generated module index").clone(),
            location: checked(&sources, 1, index * 4),
        })?;
    }
    for &index in &order {
        builder.try_declare(ModuleDeclarationInputV2 {
            owner_module_id: ids.get(index).expect("generated module index").clone(),
            local_name: format!("v{index:04}"),
            namespace_kind: NamespaceKindV2::Value,
            visibility: VisibilityV2::Public,
            location: checked(&sources, 1, index * 4 + 1),
        })?;
    }
    let mut edges = (0..depth - 1).collect::<Vec<_>>();
    if reverse {
        edges.reverse();
    }
    for index in edges {
        builder.try_import(ModuleImportInputV2 {
            owner_module_id: ids.get(index).expect("generated module index").clone(),
            local_name: format!("v{:04}", index + 1),
            target: asset_ref(
                &sources,
                ids.get(index + 1).expect("generated target module index"),
                NamespaceKindV2::Value,
                &format!("v{:04}", index + 1),
                index * 4 + 2,
            ),
            location: checked(&sources, 1, index * 4 + 3),
        })?;
    }
    let parts = builder.finish(
        CurrentModuleSelectionInputV2 {
            module_id: ids.first().expect("non-empty deep chain").clone(),
            location: checked(&sources, 1, depth * 4),
        },
        VersionV2::new(1, 0, 0),
    )?;
    Ok(ids
        .iter()
        .flat_map(|id| {
            parts
                .resolved_graph()
                .module_bindings(id)
                .into_iter()
                .flatten()
                .map(|binding| {
                    format!(
                        "{}:{}:{}:{}",
                        id.as_str(),
                        binding.local_name(),
                        binding.resolved_asset_id().local_name(),
                        binding.edge_witness().len()
                    )
                })
        })
        .collect())
}

fn seal_catalog_chain(
    depth: usize,
    reverse: bool,
    max_steps: usize,
) -> Result<Vec<String>, ModuleDiagnosticV2> {
    let sources = registry(&[(1, depth * 6 + 32)]);
    let app = module_id(&sources, "app", 0);
    let external = module_id(&sources, "host", 1);
    let relation =
        RelationTypeV1::try_new(RecordTypeV1::try_new(vec![]).unwrap(), CardinalityV1::Set)
            .unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(
        AuthorityBudgetProfileV2::try_new(
            depth + 1,
            depth - 1,
            max_steps,
            16 * 1024 * 1024,
            64 * 1024,
        )
        .unwrap(),
        sources.clone(),
    )?;
    builder.try_add_module(ModuleNodeInputV2 {
        module_id: app.clone(),
        location: checked(&sources, 1, 2),
    })?;
    let mut order = (0..depth).collect::<Vec<_>>();
    if reverse {
        order.reverse();
    }
    for index in order {
        let dependencies = (index + 1 < depth)
            .then(|| ExternalCatalogDependencyInputV2 {
                target: asset_ref(
                    &sources,
                    &external,
                    NamespaceKindV2::View,
                    &format!("a{:04}", index + 1),
                    index * 6 + 3,
                ),
                expected_kind: NamespaceKindV2::View,
                location: checked(&sources, 1, index * 6 + 4),
            })
            .into_iter()
            .collect();
        builder.try_add_external(ExternalCatalogAssetInputV2 {
            module_name: "host".into(),
            local_name: format!("a{index:04}"),
            kind_token: "view".into(),
            typed_payload: ExternalCatalogPayloadInputV2::View(relation.clone()),
            provenance: "host".into(),
            location: checked(&sources, 1, index * 6 + 5),
            dependencies,
        })?;
    }
    let parts = builder.finish(
        CurrentModuleSelectionInputV2 {
            module_id: app,
            location: checked(&sources, 1, depth * 6),
        },
        VersionV2::new(1, 0, 0),
    )?;
    Ok(parts
        .external_catalog()
        .iter()
        .map(|asset| format!("{}:{}", asset.id().local_name(), asset.dependencies().len()))
        .collect())
}

fn seal_pack_chain(
    depth: usize,
    reverse: bool,
    max_steps: usize,
) -> Result<[u8; 32], ModuleDiagnosticV2> {
    let zeros = "0".repeat(64);
    let mut lock = String::new();
    for index in 0..depth {
        writeln!(&mut lock, "p{index:04} 1.0.0 {zeros}").unwrap();
    }
    let sources = registry(&[(1, depth * 6 + 32), (2, lock.len())]);
    let app = module_id(&sources, "app", 0);
    let mut builder = ModuleAuthorityBuilderV2::try_new(
        AuthorityBudgetProfileV2::try_new(
            depth * 3 + 1,
            depth - 1,
            max_steps,
            16 * 1024 * 1024,
            64 * 1024,
        )
        .unwrap(),
        sources.clone(),
    )?;
    builder.try_add_module(ModuleNodeInputV2 {
        module_id: app.clone(),
        location: checked(&sources, 1, 1),
    })?;
    let mut order = (0..depth).collect::<Vec<_>>();
    if reverse {
        order.reverse();
    }
    for index in order {
        let dependencies = if index + 1 < depth {
            let dependency_name = format!("p{:04}", index + 1);
            vec![LocatedDependencyInputV2 {
                pack_id: PackIdV2::try_new(&dependency_name, checked(&sources, 1, index * 6 + 2))
                    .unwrap(),
                version_range: VersionRangeV2::try_new(
                    VersionV2::new(1, 0, 0),
                    VersionV2::new(2, 0, 0),
                    checked(&sources, 1, index * 6 + 3),
                )
                .unwrap(),
                location: checked(&sources, 1, index * 6 + 4),
            }]
        } else {
            vec![]
        };
        builder.try_add_manifest(manifest_input(
            &sources,
            &format!("p{index:04}"),
            VersionV2::new(1, 0, 0),
            dependencies,
            vec![],
            vec![],
            index * 6 + 2,
        ))?;
    }
    builder.try_set_lockfile(
        LockReferenceInputV2 {
            path: "packs.lock".into(),
            location: checked(&sources, 1, depth * 6),
        },
        SourceIdV2::from_raw(2),
        &lock,
    )?;
    let parts = builder.finish(
        CurrentModuleSelectionInputV2 {
            module_id: app,
            location: checked(&sources, 1, depth * 6 + 1),
        },
        VersionV2::new(1, 0, 0),
    )?;
    Ok(*parts.pack_authority().graph_fingerprint())
}

#[test]
fn module_catalog_pack_graphs_are_deterministic_and_budget_exact() {
    let depth = OWNER_LOCAL_AUTHORITY_CHAIN_LEN;
    let exact = chain_steps(depth);

    let module_forward = seal_module_chain(depth, false, exact).unwrap();
    let module_reverse = seal_module_chain(depth, true, exact).unwrap();
    assert_eq!(module_forward, module_reverse);
    assert_eq!(module_forward.len(), depth * 2 - 1);
    assert_resource(
        &seal_module_chain(depth, false, exact - 1).unwrap_err(),
        "steps",
        exact - 1,
        exact,
    );

    let catalog_forward = seal_catalog_chain(depth, false, exact).unwrap();
    let catalog_reverse = seal_catalog_chain(depth, true, exact).unwrap();
    assert_eq!(catalog_forward, catalog_reverse);
    assert_eq!(catalog_forward.len(), depth);
    assert_resource(
        &seal_catalog_chain(depth, false, exact - 1).unwrap_err(),
        "steps",
        exact - 1,
        exact,
    );

    let pack_exact = exact + depth * 3;
    let pack_forward = seal_pack_chain(depth, false, pack_exact).unwrap();
    let pack_reverse = seal_pack_chain(depth, true, pack_exact).unwrap();
    assert_eq!(pack_forward, pack_reverse);
    assert_resource(
        &seal_pack_chain(depth, false, pack_exact - 1).unwrap_err(),
        "steps",
        pack_exact - 1,
        pack_exact,
    );
}

#[test]
fn deep_module_catalog_pack_graphs_are_stack_safe() {
    if std::env::var_os(STACK_SAFETY_CHILD).is_some() {
        std::thread::Builder::new()
            .name("deep-modules-owner".into())
            .stack_size(128 * 1024)
            .spawn(|| {
                let depth = STACK_SAFETY_AUTHORITY_CHAIN_LEN;
                let exact = chain_steps(depth);

                let modules = seal_module_chain(depth, false, exact).unwrap();
                assert_eq!(modules.len(), depth * 2 - 1);

                let catalog = seal_catalog_chain(depth, false, exact).unwrap();
                assert_eq!(catalog.len(), depth);

                let pack_exact = exact + depth * 3;
                assert_ne!(seal_pack_chain(depth, false, pack_exact).unwrap(), [0; 32]);
            })
            .expect("deep modules thread starts")
            .join()
            .expect("deep modules thread completes without stack overflow");
        return;
    }

    let mut command = std::process::Command::new(std::env::current_exe().expect("test binary"));
    command
        .args([
            "--exact",
            "deep_module_catalog_pack_graphs_are_stack_safe",
            "--nocapture",
        ])
        .env(STACK_SAFETY_CHILD, "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = command.spawn().expect("deep modules child process starts");
    let output = process::wait_bounded(child, None, std::time::Duration::from_secs(10))
        .expect("deep modules child process finishes within watchdog");
    assert!(
        output.status.success(),
        "deep modules child status: {}; stderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured;"),
        "deep modules child selected zero or multiple tests: {stdout}"
    );
}

use std::fmt::Write as _;

use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, LocatedSpanV2, SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2,
    SpanV1,
};
use quanta_gqlang_v2_modules::*;
use quanta_gqlang_v2_types::{
    AggregationClassV1, AuthorityBudgetProfileV2, CardinalityV1, CostClassV1, DeterminismV1,
    ExpressionEffectsV2, MonotonicityV1, PurityV1, RecordTypeV1, RelationTypeV1,
    ResolvedPredicateIdV2, ValueTypeV1,
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

#[path = "v3_modules_v2_authority/catalog_and_activation_tests.rs"]
mod catalog_and_activation_tests;

#[path = "v3_modules_v2_authority/resolution_and_pack_tests.rs"]
mod resolution_and_pack_tests;

#[path = "v3_modules_v2_authority/document_and_model_tests.rs"]
mod document_and_model_tests;
