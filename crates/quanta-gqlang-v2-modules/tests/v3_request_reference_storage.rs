#![expect(clippy::unwrap_used, reason = "owner-local request custody assertions")]

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use quanta_gqlang_v2_hir::{
    CheckedLocatedSpanV2, FrontendRequestMaterializationRootV3, FrontendRequestStoragePermitV3,
    FrontendRequestStorageSinkV3, FrontendStorageRefusalV3, LocatedSpanV2, SourceIdV2,
    SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_modules::{
    AmbiguousCandidateV2, CatalogAssetIdV2, CatalogAssetRefInputV2, CurrentModuleSelectionInputV2,
    DocumentCatalogAssetInputV2, DocumentCatalogReferenceInputV2, DocumentCatalogReferenceOriginV2,
    DocumentCatalogValidatorV2, DocumentReferenceRequestErrorV3, LocatedEdgeV2,
    ModuleAuthorityBuilderV2, ModuleDeclarationInputV2, ModuleIdV2, ModuleNodeInputV2,
    ModuleReexportInputV2, NamespaceKindV2, ResourceExhaustionV2, VersionV2, VisibilityV2,
};
use quanta_gqlang_v2_types::AuthorityBudgetProfileV2;

struct TestLedger {
    live: AtomicU64,
    limit: AtomicU64,
}

struct TestSink(Arc<TestLedger>);

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

fn independent_reference_peak_delta(
    candidate_ids: &[(&str, &str)],
    witness_edges: &[(&str, &str)],
) -> u64 {
    type Candidate<'a> = (
        &'a CatalogAssetIdV2,
        NamespaceKindV2,
        DocumentCatalogReferenceOriginV2,
        CheckedLocatedSpanV2,
        &'a [LocatedEdgeV2],
    );
    type DiagnosticLayout = (
        String,
        Option<ResourceExhaustionV2>,
        Option<LocatedSpanV2>,
        Option<LocatedSpanV2>,
        Vec<LocatedSpanV2>,
        Vec<AmbiguousCandidateV2>,
    );
    let id_bytes = candidate_ids
        .iter()
        .map(|(module, local)| module.len() + local.len())
        .sum::<usize>();
    let witness_bytes = witness_edges
        .iter()
        .map(|(from, to)| from.len() + to.len())
        .sum::<usize>();
    // The resource diagnostic contains two bounded decimal usize values.
    let resource_message = "authority_string_bytes limit ".len()
        + usize::MAX.to_string().len()
        + " exceeded by observed value ".len()
        + usize::MAX.to_string().len();
    let scratch = candidate_ids.len()
        * (std::mem::size_of::<Candidate<'static>>() + std::mem::size_of::<AmbiguousCandidateV2>())
        + id_bytes
        + witness_edges.len() * std::mem::size_of::<LocatedEdgeV2>()
        + witness_bytes
        + std::mem::size_of::<DiagnosticLayout>()
        + 56.max(resource_message);
    u64::try_from(id_bytes + scratch + 2 * std::mem::size_of::<TestCharge>()).unwrap()
}

#[test]
fn request_reference_uses_issued_source_and_releases_last_permit() {
    let host_id = SourceIdV2::from_raw(1);
    let document_id = SourceIdV2::from_raw(9);
    let source = String::from("create view catalog.local");
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
    let ledger = Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    });
    let sink = TestSink(ledger.clone());
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(document_id, &source, &sink);
    let mut permit = root.reserve_v3(document_id, &source, 1).unwrap();
    let mut validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
        extended,
        &parts,
        &root,
        document_id,
        &source,
        &mut permit,
    )
    .unwrap();
    let proofs = validator
        .predeclare_assets_with_request_owner_v3(
            vec![DocumentCatalogAssetInputV2 {
                owner_module_id: module,
                local_name: "catalog.local".to_owned(),
                namespace_kind: NamespaceKindV2::View,
                location: location.clone(),
            }],
            &root,
            document_id,
            &source,
            &mut permit,
        )
        .unwrap();
    assert_eq!(proofs.len(), 1);
    let reference = || DocumentCatalogReferenceInputV2 {
        module_id: None,
        local_name: "catalog.local".to_owned(),
        expected_kind: NamespaceKindV2::View,
        location: location.clone(),
    };
    let foreign_sink = TestSink(Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    }));
    let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        document_id,
        &source,
        &foreign_sink,
    );
    let copied_source = source.clone();
    assert!(matches!(
        validator.resolve_reference_if_present_with_request_owner_v3(
            reference(),
            &root,
            document_id,
            &copied_source,
            &mut permit,
        ),
        Err(DocumentReferenceRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignSource
        ))
    ));
    assert!(matches!(
        validator.resolve_reference_if_present_with_request_owner_v3(
            reference(),
            &foreign_root,
            document_id,
            &source,
            &mut permit,
        ),
        Err(DocumentReferenceRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    let mut foreign_permit = foreign_root.reserve_v3(document_id, &source, 0).unwrap();
    assert!(matches!(
        validator.resolve_reference_if_present_with_request_owner_v3(
            reference(),
            &foreign_root,
            document_id,
            &source,
            &mut foreign_permit,
        ),
        Err(DocumentReferenceRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    drop(foreign_permit);
    let before_reference = ledger.live.load(Ordering::SeqCst);
    let exact_peak =
        before_reference + independent_reference_peak_delta(&[("app", "catalog.local")], &[]);
    ledger.limit.store(exact_peak, Ordering::SeqCst);
    let selected = validator
        .resolve_reference_if_present_with_request_owner_v3(
            reference(),
            &root,
            document_id,
            &source,
            &mut permit,
        )
        .unwrap()
        .unwrap();
    assert_eq!(selected.id().local_name(), "catalog.local");
    assert_eq!(
        ledger.live.load(Ordering::SeqCst),
        before_reference
            + u64::try_from(
                "app".len() + "catalog.local".len() + std::mem::size_of::<TestCharge>()
            )
            .unwrap()
    );
    ledger.limit.store(
        ledger.live.load(Ordering::SeqCst)
            + independent_reference_peak_delta(&[("app", "catalog.local")], &[])
            - 1,
        Ordering::SeqCst,
    );
    assert!(matches!(
        validator.resolve_reference_if_present_with_request_owner_v3(
            reference(),
            &root,
            document_id,
            &source,
            &mut permit,
        ),
        Err(DocumentReferenceRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    drop(proofs);
    drop(validator);
    drop(permit);
    assert!(ledger.live.load(Ordering::SeqCst) > 0);
    assert_eq!(selected.id().local_name(), "catalog.local");
    drop(selected);
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
}

#[test]
fn typed_reference_diagnostic_retains_scratch_until_last_drop() {
    let host_id = SourceIdV2::from_raw(1);
    let document_id = SourceIdV2::from_raw(9);
    let source = String::from("create view catalog.local");
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
    let left = ModuleIdV2::try_new("left", host_location.clone()).unwrap();
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
    builder
        .try_add_module(ModuleNodeInputV2 {
            module_id: left.clone(),
            location: host_location.clone(),
        })
        .unwrap();
    builder
        .try_declare(ModuleDeclarationInputV2 {
            owner_module_id: left.clone(),
            local_name: "catalog.local".to_owned(),
            namespace_kind: NamespaceKindV2::Value,
            visibility: VisibilityV2::Public,
            location: host_location.clone(),
        })
        .unwrap();
    builder
        .try_reexport(ModuleReexportInputV2 {
            owner_module_id: module.clone(),
            exported_name: "catalog.local".to_owned(),
            target: CatalogAssetRefInputV2 {
                module_id: left,
                namespace_kind: NamespaceKindV2::Value,
                local_name: "catalog.local".to_owned(),
                location: host_location.clone(),
            },
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
    let binding = parts
        .resolved_graph()
        .visible_bindings()
        .get(NamespaceKindV2::Value, "catalog.local")
        .unwrap();
    assert_eq!(binding.resolved_asset_id().module_id().as_str(), "left");
    assert_eq!(binding.edge_witness().len(), 1);
    let edge = binding.edge_witness().first().unwrap();
    assert_eq!(edge.from.as_str(), "app");
    assert_eq!(edge.to.as_str(), "left");
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
    let ledger = Arc::new(TestLedger {
        live: AtomicU64::new(0),
        limit: AtomicU64::new(1_000_000),
    });
    let sink = TestSink(ledger.clone());
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(document_id, &source, &sink);
    let mut permit = root.reserve_v3(document_id, &source, 0).unwrap();
    let mut validator = DocumentCatalogValidatorV2::try_new_with_request_owner_v3(
        extended,
        &parts,
        &root,
        document_id,
        &source,
        &mut permit,
    )
    .unwrap();
    let proofs = validator
        .predeclare_assets_with_request_owner_v3(
            vec![DocumentCatalogAssetInputV2 {
                owner_module_id: module,
                local_name: "catalog.local".to_owned(),
                namespace_kind: NamespaceKindV2::View,
                location: location.clone(),
            }],
            &root,
            document_id,
            &source,
            &mut permit,
        )
        .unwrap();
    assert_eq!(proofs.len(), 1);
    let before_reference = ledger.live.load(Ordering::SeqCst);
    let exact_peak = before_reference
        + independent_reference_peak_delta(
            &[("app", "catalog.local"), ("left", "catalog.local")],
            &[("app", "left")],
        );
    ledger.limit.store(exact_peak, Ordering::SeqCst);
    let error = validator
        .resolve_reference_if_present_with_request_owner_v3(
            DocumentCatalogReferenceInputV2 {
                module_id: None,
                local_name: "catalog.local".to_owned(),
                expected_kind: NamespaceKindV2::Profile,
                location,
            },
            &root,
            document_id,
            &source,
            &mut permit,
        )
        .unwrap_err();
    let DocumentReferenceRequestErrorV3::Domain(diagnostic) = error else {
        assert!(matches!(error, DocumentReferenceRequestErrorV3::Domain(_)));
        return;
    };
    assert_eq!(
        diagnostic.reason_code(),
        "modules.ambiguous_catalog_reference"
    );
    let selected_charge = u64::try_from(
        "app".len()
            + "catalog.local".len()
            + "left".len()
            + "catalog.local".len()
            + std::mem::size_of::<TestCharge>(),
    )
    .unwrap();
    assert_eq!(
        ledger.live.load(Ordering::SeqCst),
        exact_peak - selected_charge
    );
    drop(proofs);
    drop(validator);
    drop(permit);
    assert!(ledger.live.load(Ordering::SeqCst) > 0);
    drop(diagnostic);
    assert_eq!(ledger.live.load(Ordering::SeqCst), 0);
}
