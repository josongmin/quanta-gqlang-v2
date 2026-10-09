#![expect(clippy::unwrap_used, reason = "owner-local request custody assertions")]

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use quanta_gqlang_v2_hir::{
    FrontendRequestMaterializationRootV3, FrontendRequestStoragePermitV3,
    FrontendRequestStorageSinkV3, FrontendStorageRefusalV3, LocatedSpanV2, SourceIdV2,
    SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_modules::{
    CatalogAssetRefInputV2, CurrentModuleSelectionInputV2, DocumentCatalogAssetInputV2,
    DocumentCatalogDependencyInputV2, DocumentCatalogFamilyReferenceInputV3,
    DocumentCatalogFinalInputV3, DocumentCatalogReferenceFamilyV2, DocumentCatalogReferenceInputV2,
    DocumentCatalogRequestErrorV3, DocumentCatalogValidatorV2, DocumentTopologyRequestErrorV3,
    ModuleAuthorityBuilderV2, ModuleIdV2, ModuleNodeInputV2, NamespaceKindV2,
    RequireCertifiedInputV2, VersionV2,
};
use quanta_gqlang_v2_types::AuthorityBudgetProfileV2;

struct TestLedger {
    live: AtomicU64,
    limit: AtomicU64,
}

struct TestCase {
    host: SourceRegistryViewV2,
    parts: quanta_gqlang_v2_modules::ValidatedModuleAuthorityPartsV2,
    module: ModuleIdV2,
    extended: quanta_gqlang_v2_hir::ExtendedSourceRegistryViewV2,
    location: quanta_gqlang_v2_hir::CheckedLocatedSpanV2,
    source: String,
    document_id: SourceIdV2,
}

impl TestCase {
    fn new() -> Self {
        let host_id = SourceIdV2::from_raw(31);
        let document_id = SourceIdV2::from_raw(32);
        let source = String::from("create view first; create view second");
        let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
            source_id: host_id,
            source_len: 64,
        }])
        .unwrap();
        let host_location = host
            .try_check_location(LocatedSpanV2 {
                source_id: host_id,
                span: SpanV1::new(0, 1),
            })
            .unwrap();
        let module = ModuleIdV2::try_new("app", host_location.clone()).unwrap();
        let mut builder = ModuleAuthorityBuilderV2::try_new(
            AuthorityBudgetProfileV2::try_new(16, 16, 1024, 4096, 256).unwrap(),
            host.clone(),
        )
        .unwrap();
        builder
            .try_add_module(ModuleNodeInputV2 {
                module_id: module.clone(),
                location: host_location.clone(),
            })
            .unwrap();
        let parts = builder
            .finish(
                CurrentModuleSelectionInputV2 {
                    module_id: module.clone(),
                    location: host_location,
                },
                VersionV2::new(1, 0, 0),
            )
            .unwrap();
        let extended = host
            .try_extend_document(SourceRegistrationV2 {
                source_id: document_id,
                source_len: source.len(),
            })
            .unwrap();
        let location = extended
            .combined()
            .try_check_location(LocatedSpanV2 {
                source_id: document_id,
                span: SpanV1::new(0, 1),
            })
            .unwrap();
        Self {
            host,
            parts,
            module,
            extended,
            location,
            source,
            document_id,
        }
    }

    fn edges(&self) -> Vec<DocumentCatalogDependencyInputV2> {
        vec![DocumentCatalogDependencyInputV2 {
            source: CatalogAssetRefInputV2 {
                module_id: self.module.clone(),
                namespace_kind: NamespaceKindV2::View,
                local_name: "first".to_owned(),
                location: self.location.clone(),
            },
            target: CatalogAssetRefInputV2 {
                module_id: self.module.clone(),
                namespace_kind: NamespaceKindV2::View,
                local_name: "second".to_owned(),
                location: self.location.clone(),
            },
            expected_target_kind: NamespaceKindV2::View,
            location: self.location.clone(),
        }]
    }

    fn assets(&self) -> Vec<DocumentCatalogAssetInputV2> {
        ["first", "second"]
            .into_iter()
            .map(|name| DocumentCatalogAssetInputV2 {
                owner_module_id: self.module.clone(),
                local_name: name.to_owned(),
                namespace_kind: NamespaceKindV2::View,
                location: self.location.clone(),
            })
            .collect()
    }

    fn final_input(&self) -> DocumentCatalogFinalInputV3 {
        DocumentCatalogFinalInputV3 {
            profile_admissions: Vec::new(),
            activations: Vec::new(),
            require_certified_directives: vec![RequireCertifiedInputV2 {
                location: self.location.clone(),
            }],
            family_references: Vec::new(),
        }
    }
}

#[test]
fn session_witness_rejects_equal_catalog_from_another_validator() {
    let case = TestCase::new();
    let ledger = Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(u64::MAX),
    });
    let sink = TestSink(Arc::clone(&ledger));
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        case.document_id,
        &case.source,
        &sink,
    );
    let mut parent = root.reserve_v3(case.document_id, &case.source, 0).unwrap();
    let mut issue_topology = || {
        let mut validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
            case.extended.clone(),
            &case.parts,
            &root,
            case.document_id,
            &case.source,
            &mut parent,
        )
        .unwrap();
        let declarations = validator
            .predeclare_assets_with_request_owner_v3(
                case.assets(),
                &root,
                case.document_id,
                &case.source,
                &mut parent,
            )
            .unwrap();
        let topology = validator
            .validate_view_topology_with_request_owner_v3(
                case.edges(),
                &root,
                case.document_id,
                &case.source,
                &parent,
            )
            .unwrap();
        assert!(topology
            .validate_owner_v3(&root, case.document_id, &case.source)
            .is_ok());
        let witness = topology.retain_session_v3();
        (validator, declarations, topology, witness)
    };
    let (mut first_validator, first_declarations, first_topology, first_witness) = issue_topology();
    let (mut second_validator, second_declarations, second_topology, second_witness) =
        issue_topology();
    assert!(first_witness.matches_topology_v3(&first_topology));
    assert!(second_witness.matches_topology_v3(&second_topology));
    assert!(!first_witness.matches_topology_v3(&second_topology));
    assert!(!second_witness.matches_topology_v3(&first_topology));
    let finish = |validator: &mut DocumentCatalogValidatorV2<'_>, topology| {
        let catalog = validator
            .validate_predeclared_with_request_owner_v3(
                topology,
                case.final_input(),
                &root,
                case.document_id,
                &case.source,
                &parent,
            )
            .unwrap();
        assert!(catalog
            .validate_owner_v3(&root, case.document_id, &case.source)
            .is_ok());
        assert!(catalog.has_source_registry_v3(&case.extended.combined().identity()));
        assert!(!catalog.has_source_registry_v3(&case.host.identity()));
        catalog
    };
    let first_catalog = finish(&mut first_validator, first_topology);
    let second_catalog = finish(&mut second_validator, second_topology);
    drop(first_declarations);
    drop(second_declarations);
    drop(first_validator);
    drop(second_validator);
    assert_eq!(first_catalog.asset_count(), second_catalog.asset_count());
    assert_eq!(
        first_catalog.dependency_count(),
        second_catalog.dependency_count()
    );
    assert!(first_witness.matches_catalog_v3(&first_catalog));
    assert!(second_witness.matches_catalog_v3(&second_catalog));
    assert!(!first_witness.matches_catalog_v3(&second_catalog));
    assert!(!second_witness.matches_catalog_v3(&first_catalog));
    drop(first_catalog);
    drop(second_catalog);
    drop(parent);
    drop(second_witness);
    assert!(ledger.live.load(Ordering::SeqCst) > 0);
    drop(first_witness);
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
}

#[test]
fn topology_owned_proof_and_foreign_root() {
    let case = TestCase::new();
    let document_id = case.document_id;
    let source = &case.source;
    let parts = &case.parts;
    let extended = case.extended.clone();
    let ledger = Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    });
    let sink = TestSink(ledger.clone());
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(document_id, source, &sink);
    let mut permit = root.reserve_v3(document_id, source, 1).unwrap();
    // A raw predeclaration under the same registry and root is not an issued
    // request stage. It must not be upgraded by a later topology call.
    let mut raw_validator = DocumentCatalogValidatorV2::try_new(extended.clone(), parts).unwrap();
    raw_validator.predeclare_assets(case.assets()).unwrap();
    assert!(matches!(
        raw_validator.validate_view_topology_with_request_owner_v3(
            case.edges(),
            &root,
            document_id,
            source,
            &permit,
        ),
        Err(DocumentTopologyRequestErrorV3::Phase)
    ));
    let mut validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
        extended.clone(),
        parts,
        &root,
        document_id,
        source,
        &mut permit,
    )
    .unwrap();
    let proofs = validator
        .predeclare_assets_with_request_owner_v3(
            case.assets(),
            &root,
            document_id,
            source,
            &mut permit,
        )
        .unwrap();
    let foreign_sink = TestSink(Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    }));
    let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        document_id,
        source,
        &foreign_sink,
    );
    assert!(matches!(
        validator.validate_view_topology_with_request_owner_v3(
            case.edges(),
            &foreign_root,
            document_id,
            source,
            &permit,
        ),
        Err(DocumentTopologyRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    let foreign_permit = foreign_root.reserve_v3(document_id, source, 0).unwrap();
    assert!(matches!(
        validator.validate_view_topology_with_request_owner_v3(
            case.edges(),
            &foreign_root,
            document_id,
            source,
            &foreign_permit,
        ),
        Err(DocumentTopologyRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    drop(foreign_permit);
    let topology = validator
        .validate_view_topology_with_request_owner_v3(
            case.edges(),
            &root,
            document_id,
            source,
            &permit,
        )
        .unwrap();
    assert_eq!(
        topology
            .dependency_first_order()
            .map(|id| id.local_name())
            .collect::<Vec<_>>(),
        vec!["second", "first"]
    );
    drop(proofs);
    drop(validator);
    drop(permit);
    assert!(ledger.live.load(Ordering::SeqCst) > 0);
    drop(topology);
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
}

#[test]
fn topology_budget_refusal() {
    let case = TestCase::new();
    let document_id = case.document_id;
    let source = &case.source;
    let parts = &case.parts;
    let extended = case.extended.clone();
    let short_ledger = Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    });
    let short_sink = TestSink(short_ledger.clone());
    let short_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(document_id, source, &short_sink);
    let mut short_permit = short_root.reserve_v3(document_id, source, 1).unwrap();
    let mut short_validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
        extended,
        parts,
        &short_root,
        document_id,
        source,
        &mut short_permit,
    )
    .unwrap();
    let short_proofs = short_validator
        .predeclare_assets_with_request_owner_v3(
            case.assets(),
            &short_root,
            document_id,
            source,
            &mut short_permit,
        )
        .unwrap();
    short_ledger.limit.store(
        short_ledger.live.load(Ordering::SeqCst) + 1,
        Ordering::SeqCst,
    );
    assert!(matches!(
        short_validator.validate_view_topology_with_request_owner_v3(
            case.edges(),
            &short_root,
            document_id,
            source,
            &short_permit,
        ),
        Err(DocumentTopologyRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    drop(short_proofs);
    drop(short_validator);
    drop(short_permit);
    assert_eq!(short_ledger.live.load(Ordering::SeqCst), 0);
}

#[test]
fn topology_cycle_diagnostic_retains_storage() {
    let case = TestCase::new();
    let document_id = case.document_id;
    let source = &case.source;
    let parts = &case.parts;
    let host = &case.host;
    let cycle_ledger = Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    });
    let cycle_sink = TestSink(cycle_ledger.clone());
    let cycle_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(document_id, source, &cycle_sink);
    let mut cycle_permit = cycle_root.reserve_v3(document_id, source, 1).unwrap();
    let mut cycle_validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
        host.try_extend_document(SourceRegistrationV2 {
            source_id: document_id,
            source_len: source.len(),
        })
        .unwrap(),
        parts,
        &cycle_root,
        document_id,
        source,
        &mut cycle_permit,
    )
    .unwrap();
    let cycle_proofs = cycle_validator
        .predeclare_assets_with_request_owner_v3(
            case.assets(),
            &cycle_root,
            document_id,
            source,
            &mut cycle_permit,
        )
        .unwrap();
    let mut cyclic = case.edges();
    cyclic.get_mut(0).unwrap().target.local_name = "first".to_owned();
    let outcome = cycle_validator.validate_view_topology_with_request_owner_v3(
        cyclic,
        &cycle_root,
        document_id,
        source,
        &cycle_permit,
    );
    assert!(matches!(
        &outcome,
        Err(DocumentTopologyRequestErrorV3::Domain(_))
    ));
    let Err(DocumentTopologyRequestErrorV3::Domain(diagnostic)) = outcome else {
        return;
    };
    assert_eq!(diagnostic.reason_code(), "modules.view_inline_cycle");
    drop(cycle_proofs);
    drop(cycle_validator);
    drop(cycle_permit);
    assert!(cycle_ledger.live.load(Ordering::SeqCst) > 0);
    drop(diagnostic);
    assert_eq!(cycle_ledger.live.load(Ordering::SeqCst), 0);
}

#[test]
fn final_catalog_owned_proof() {
    let case = TestCase::new();
    let document_id = case.document_id;
    let source = &case.source;
    let parts = &case.parts;
    let host = &case.host;
    let module = case.module.clone();
    let location = case.location.clone();
    let final_ledger = Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    });
    let final_sink = TestSink(final_ledger.clone());
    let final_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(document_id, source, &final_sink);
    let mut final_permit = final_root.reserve_v3(document_id, source, 1).unwrap();
    let mut final_validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
        host.try_extend_document(SourceRegistrationV2 {
            source_id: document_id,
            source_len: source.len(),
        })
        .unwrap(),
        parts,
        &final_root,
        document_id,
        source,
        &mut final_permit,
    )
    .unwrap();
    let mut final_assets = case.assets();
    final_assets.push(DocumentCatalogAssetInputV2 {
        owner_module_id: module.clone(),
        local_name: "query".to_owned(),
        namespace_kind: NamespaceKindV2::Query,
        location: location.clone(),
    });
    let final_proofs = final_validator
        .predeclare_assets_with_request_owner_v3(
            final_assets,
            &final_root,
            document_id,
            source,
            &mut final_permit,
        )
        .unwrap();
    let final_topology = final_validator
        .validate_view_topology_with_request_owner_v3(
            case.edges(),
            &final_root,
            document_id,
            source,
            &final_permit,
        )
        .unwrap();
    let target = final_validator
        .resolve_reference_if_present_with_request_owner_v3(
            DocumentCatalogReferenceInputV2 {
                module_id: None,
                local_name: "query".to_owned(),
                expected_kind: NamespaceKindV2::Query,
                location: location.clone(),
            },
            &final_root,
            document_id,
            source,
            &mut final_permit,
        )
        .unwrap()
        .unwrap();
    let source_proof = final_proofs
        .into_iter()
        .find(|proof| proof.id().local_name() == "first")
        .unwrap();
    let mut issued_input = case.final_input();
    issued_input
        .family_references
        .push(DocumentCatalogFamilyReferenceInputV3 {
            family: DocumentCatalogReferenceFamilyV2::ViewQuery,
            source: source_proof,
            target,
            location: location.clone(),
        });
    let catalog = final_validator
        .validate_predeclared_with_request_owner_v3(
            final_topology,
            issued_input,
            &final_root,
            document_id,
            source,
            &final_permit,
        )
        .unwrap();
    assert_eq!(catalog.asset_count(), 3);
    assert_eq!(catalog.dependency_count(), 2);
    drop(final_validator);
    drop(final_permit);
    assert!(final_ledger.live.load(Ordering::SeqCst) > 0);
    drop(catalog);
    assert_eq!(final_ledger.live.load(Ordering::SeqCst), 0);
}

#[test]
fn final_catalog_budget_refusal() {
    let case = TestCase::new();
    let document_id = case.document_id;
    let source = &case.source;
    let parts = &case.parts;
    let host = &case.host;
    let refusal_ledger = Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    });
    let refusal_sink = TestSink(refusal_ledger.clone());
    let refusal_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        document_id,
        source,
        &refusal_sink,
    );
    let mut refusal_permit = refusal_root.reserve_v3(document_id, source, 1).unwrap();
    let mut refusal_validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
        host.try_extend_document(SourceRegistrationV2 {
            source_id: document_id,
            source_len: source.len(),
        })
        .unwrap(),
        parts,
        &refusal_root,
        document_id,
        source,
        &mut refusal_permit,
    )
    .unwrap();
    let refusal_proofs = refusal_validator
        .predeclare_assets_with_request_owner_v3(
            case.assets(),
            &refusal_root,
            document_id,
            source,
            &mut refusal_permit,
        )
        .unwrap();
    let refusal_topology = refusal_validator
        .validate_view_topology_with_request_owner_v3(
            case.edges(),
            &refusal_root,
            document_id,
            source,
            &refusal_permit,
        )
        .unwrap();
    refusal_ledger.limit.store(
        refusal_ledger.live.load(Ordering::SeqCst) + 1,
        Ordering::SeqCst,
    );
    assert!(matches!(
        refusal_validator.validate_predeclared_with_request_owner_v3(
            refusal_topology,
            case.final_input(),
            &refusal_root,
            document_id,
            source,
            &refusal_permit,
        ),
        Err(DocumentCatalogRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    drop(refusal_proofs);
    drop(refusal_validator);
    drop(refusal_permit);
    assert_eq!(refusal_ledger.live.load(Ordering::SeqCst), 0);
}

#[test]
fn final_catalog_foreign_root_refusal() {
    let case = TestCase::new();
    let document_id = case.document_id;
    let source = &case.source;
    let parts = &case.parts;
    let host = &case.host;
    let foreign_sink = TestSink(Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    }));
    let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        document_id,
        source,
        &foreign_sink,
    );
    let foreign_final_ledger = Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    });
    let foreign_final_sink = TestSink(foreign_final_ledger.clone());
    let own_final_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        document_id,
        source,
        &foreign_final_sink,
    );
    let mut own_final_permit = own_final_root.reserve_v3(document_id, source, 1).unwrap();
    let mut foreign_final_validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
        host.try_extend_document(SourceRegistrationV2 {
            source_id: document_id,
            source_len: source.len(),
        })
        .unwrap(),
        parts,
        &own_final_root,
        document_id,
        source,
        &mut own_final_permit,
    )
    .unwrap();
    let foreign_final_proofs = foreign_final_validator
        .predeclare_assets_with_request_owner_v3(
            case.assets(),
            &own_final_root,
            document_id,
            source,
            &mut own_final_permit,
        )
        .unwrap();
    let foreign_final_topology = foreign_final_validator
        .validate_view_topology_with_request_owner_v3(
            case.edges(),
            &own_final_root,
            document_id,
            source,
            &own_final_permit,
        )
        .unwrap();
    let foreign_permit = foreign_root.reserve_v3(document_id, source, 0).unwrap();
    assert!(matches!(
        foreign_final_validator.validate_predeclared_with_request_owner_v3(
            foreign_final_topology,
            case.final_input(),
            &foreign_root,
            document_id,
            source,
            &foreign_permit,
        ),
        Err(DocumentCatalogRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    drop(foreign_permit);
    drop(foreign_final_proofs);
    drop(foreign_final_validator);
    drop(own_final_permit);
    assert_eq!(foreign_final_ledger.live.load(Ordering::SeqCst), 0);
}

struct TestSink(Arc<TestLedger>);

/// Observe the actual sink boundary, independently of production demand helpers.
struct PeakSink {
    inner: TestSink,
    peak: AtomicU64,
}

impl FrontendRequestStorageSinkV3 for PeakSink {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        let requested = self.inner.0.live.load(Ordering::SeqCst)
            + logical_bytes
            + u64::try_from(std::mem::size_of::<TestCharge>()).unwrap();
        self.peak.fetch_max(requested, Ordering::SeqCst);
        self.inner.reserve_v3(logical_bytes)
    }

    fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
        self.inner.owns_permit_v3(permit)
    }
}

#[test]
fn topology_acyclic_and_scc_witness_admit_exact_peak_and_refuse_one_byte_short() {
    for cyclic in [false, true] {
        let mut observed_peak = 0;
        // First observe the reservation peak, then replay at exact/one-short.
        // This tests admission/custody, not measured allocator peak bytes.
        for attempt in 0..3 {
            let case = TestCase::new();
            let ledger = Arc::new(TestLedger {
                live: AtomicU64::new(0),
                limit: AtomicU64::new(u64::MAX),
            });
            let sink = PeakSink {
                inner: TestSink(Arc::clone(&ledger)),
                peak: AtomicU64::new(0),
            };
            let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
                case.document_id,
                &case.source,
                &sink,
            );
            let mut permit = root.reserve_v3(case.document_id, &case.source, 0).unwrap();
            let mut validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
                case.extended.clone(),
                &case.parts,
                &root,
                case.document_id,
                &case.source,
                &mut permit,
            )
            .unwrap();
            let proofs = validator
                .predeclare_assets_with_request_owner_v3(
                    case.assets(),
                    &root,
                    case.document_id,
                    &case.source,
                    &mut permit,
                )
                .unwrap();
            let before = ledger.live.load(Ordering::SeqCst);
            sink.peak.store(before, Ordering::SeqCst);
            if attempt != 0 {
                ledger
                    .limit
                    .store(observed_peak - u64::from(attempt == 2), Ordering::SeqCst);
            }
            let mut edges = case.edges();
            if cyclic {
                edges.get_mut(0).unwrap().target.local_name = "first".to_owned();
            }
            let result = validator.validate_view_topology_with_request_owner_v3(
                edges,
                &root,
                case.document_id,
                &case.source,
                &permit,
            );
            if attempt == 0 {
                observed_peak = sink.peak.load(Ordering::SeqCst);
                assert!(observed_peak > before);
            }
            if attempt == 2 {
                assert!(matches!(
                    result,
                    Err(DocumentTopologyRequestErrorV3::Storage(
                        FrontendStorageRefusalV3::BudgetRejected
                    ))
                ));
                assert_eq!(ledger.live.load(Ordering::SeqCst), before);
            } else if cyclic {
                assert!(matches!(&result,
                    Err(DocumentTopologyRequestErrorV3::Domain(diagnostic))
                        if diagnostic.reason_code() == "modules.view_inline_cycle"
                ));
            } else {
                assert!(result.is_ok());
            }
            drop(result);
            drop(proofs);
            drop(validator);
            drop(permit);
            assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
        }
    }
}

struct TestCharge {
    ledger: Arc<TestLedger>,
    bytes: u64,
}

impl Drop for TestCharge {
    fn drop(&mut self) {
        self.ledger.live.fetch_sub(self.bytes, Ordering::SeqCst);
    }
}

impl FrontendRequestStorageSinkV3 for TestSink {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        let bytes = logical_bytes
            .checked_add(u64::try_from(std::mem::size_of::<TestCharge>()).unwrap())
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        loop {
            let live = self.0.live.load(Ordering::SeqCst);
            let next = live
                .checked_add(bytes)
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            if next > self.0.limit.load(Ordering::SeqCst) {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            if self
                .0
                .live
                .compare_exchange(live, next, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(Box::new(TestCharge {
                    ledger: self.0.clone(),
                    bytes,
                }));
            }
        }
    }

    fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
        permit
            .as_any_v3()
            .downcast_ref::<TestCharge>()
            .is_some_and(|charge| Arc::ptr_eq(&charge.ledger, &self.0))
    }
}
