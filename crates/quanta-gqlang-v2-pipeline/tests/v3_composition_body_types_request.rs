#![expect(
    clippy::expect_used,
    reason = "owner integration fixtures require exact success"
)]

use quanta_gqlang_v2_pipeline::{
    begin_bind_source_document_with_request_owner_v3,
    begin_prepare_source_body_with_request_owner_v3,
    begin_resolve_source_document_with_request_owner_v3,
    check_source_document_with_request_owner_v3, declare_source_document_with_request_owner_v3,
    parse_source_document_with_request_owner_v3, predeclare_source_catalog_with_request_owner_v3,
    resolve_source_bindings_with_request_owner_v3, type_source_body_with_request_owner_v3,
    ClosedFrontendAuthorityBuilderV2, ClosedFrontendAuthorityV2, FrontendAuthorityLocationV2,
    FrontendAuthoritySourceV2, FrontendBudgetV2, FrontendLanguageVersionV2,
    FrontendRelationInputV2, FrontendRequestMaterializationRootV3, FrontendRequestStoragePermitV3,
    FrontendRequestStorageSinkV3, FrontendStorageRefusalV3, SourceIdV2,
};
use quanta_gqlang_v2_sema::SourceBodyTypesRequestErrorV3;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

const SOURCE_ID: SourceIdV2 = SourceIdV2::from_raw(711);
const SOURCE: &str =
    "let p = from files f | project f.path as path from p x | project x.path as output with p";
const UNKNOWN_HOST_FIELD: &str =
    "let p = from files f | project f.missing as path from p x | project x.path as output with p";

#[derive(Default)]
struct Ledger {
    live: AtomicU64,
    peak: AtomicU64,
    limit: AtomicU64,
}

struct Sink(Arc<Ledger>);
struct Charge {
    ledger: Arc<Ledger>,
    bytes: u64,
}

impl Drop for Charge {
    fn drop(&mut self) {
        self.ledger.live.fetch_sub(self.bytes, Ordering::SeqCst);
    }
}

impl FrontendRequestStorageSinkV3 for Sink {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        let bytes = logical_bytes
            .checked_add(u64::try_from(std::mem::size_of::<Charge>()).expect("charge size"))
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        let mut live = self.0.live.load(Ordering::Acquire);
        loop {
            let next = live
                .checked_add(bytes)
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            if next > self.0.limit.load(Ordering::Acquire) {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            match self
                .0
                .live
                .compare_exchange_weak(live, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => {
                    self.0.peak.fetch_max(next, Ordering::AcqRel);
                    break;
                }
                Err(observed) => live = observed,
            }
        }
        Ok(Box::new(Charge {
            ledger: Arc::clone(&self.0),
            bytes,
        }))
    }
    fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
        permit
            .as_any_v3()
            .downcast_ref::<Charge>()
            .is_some_and(|charge| Arc::ptr_eq(&charge.ledger, &self.0))
    }
}

fn authority() -> ClosedFrontendAuthorityV2 {
    let location = FrontendAuthorityLocationV2 {
        source_id: SourceIdV2::from_raw(1),
        start: 0,
        end: 1,
    };
    let mut builder = ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("standard budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: location.source_id,
            source_len: 64,
        }],
        "app".to_owned(),
        location,
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("closed builder");
    builder.add_relation(FrontendRelationInputV2::canonical_workspace_files(
        location, location,
    ));
    builder.finish().expect("host files authority")
}

fn resolved<'a>(
    source: &str,
    authority: &'a ClosedFrontendAuthorityV2,
    root: &FrontendRequestMaterializationRootV3<'_>,
) -> quanta_gqlang_v2_pipeline::ResolvedSourceBinderRequestSourceV3<'a> {
    let parsed =
        parse_source_document_with_request_owner_v3(SOURCE_ID, source, authority.budget(), root)
            .expect("parsed source owner");
    let checked = check_source_document_with_request_owner_v3(SOURCE_ID, source, root, parsed)
        .expect("checked source owner");
    let declared = declare_source_document_with_request_owner_v3(source, authority, root, checked)
        .expect("declaration owner");
    let resolving = begin_resolve_source_document_with_request_owner_v3(source, root, declared)
        .expect("Modules session");
    let predeclared = predeclare_source_catalog_with_request_owner_v3(source, root, resolving)
        .expect("empty catalog");
    let preparing = begin_prepare_source_body_with_request_owner_v3(source, root, predeclared)
        .expect("admitted builtins");
    let declaring = begin_bind_source_document_with_request_owner_v3(source, root, preparing)
        .expect("Binder declarations");
    resolve_source_bindings_with_request_owner_v3(source, root, declaring)
        .expect("one-LET Binder evidence")
}

fn sink() -> Sink {
    let ledger = Arc::new(Ledger::default());
    ledger.limit.store(u64::MAX, Ordering::Release);
    Sink(ledger)
}

#[test]
fn one_let_type_stage_retains_four_exact_facts_and_rows() {
    let authority = authority();
    let sink = sink();
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(SOURCE_ID, SOURCE, &sink);
    let bound = resolved(SOURCE, &authority, &root);
    assert_eq!(bound.resolved_reference_count_v3(), 4);
    let typed = type_source_body_with_request_owner_v3(SOURCE, &root, bound)
        .expect("two real typed Query bodies");
    assert_eq!(typed.typed_fact_count_v3(), 4);
    assert_eq!(typed.relation_row_count_v3(), 4);
    drop(typed);
    assert_eq!(sink.0.live.load(Ordering::Acquire), 0);
}

#[test]
fn one_let_type_stage_refuses_short_foreign_and_other_source() {
    let authority = authority();
    let owner_sink = sink();
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(SOURCE_ID, SOURCE, &owner_sink);
    let bound = resolved(SOURCE, &authority, &root);
    let baseline = owner_sink.0.live.load(Ordering::Acquire);
    let typed = type_source_body_with_request_owner_v3(SOURCE, &root, bound)
        .expect("baseline type reservation");
    let type_bytes = owner_sink.0.live.load(Ordering::Acquire) - baseline;
    assert!(type_bytes > 0);
    drop(typed);
    assert_eq!(owner_sink.0.live.load(Ordering::Acquire), 0);

    let bound = resolved(SOURCE, &authority, &root);
    let before = owner_sink.0.live.load(Ordering::Acquire);
    owner_sink
        .0
        .limit
        .store(before + type_bytes - 1, Ordering::Release);
    assert!(matches!(
        type_source_body_with_request_owner_v3(SOURCE, &root, bound),
        Err(SourceBodyTypesRequestErrorV3::Storage(
            FrontendStorageRefusalV3::BudgetRejected
        ))
    ));
    assert_eq!(owner_sink.0.live.load(Ordering::Acquire), 0);
    owner_sink.0.limit.store(u64::MAX, Ordering::Release);

    let bound = resolved(SOURCE, &authority, &root);
    let foreign_sink = sink();
    let foreign_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(SOURCE_ID, SOURCE, &foreign_sink);
    let foreign_peak = foreign_sink.0.peak.load(Ordering::Acquire);
    assert!(matches!(
        type_source_body_with_request_owner_v3(SOURCE, &foreign_root, bound),
        Err(SourceBodyTypesRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignRequestRoot
        ))
    ));
    assert_eq!(foreign_sink.0.peak.load(Ordering::Acquire), foreign_peak);
    assert_eq!(owner_sink.0.live.load(Ordering::Acquire), 0);

    let bound = resolved(SOURCE, &authority, &root);
    let copied_source = SOURCE.to_owned();
    let previous_peak = owner_sink.0.peak.load(Ordering::Acquire);
    let result = type_source_body_with_request_owner_v3(&copied_source, &root, bound);
    assert!(matches!(
        result,
        Err(SourceBodyTypesRequestErrorV3::Storage(
            FrontendStorageRefusalV3::ForeignSource
        ))
    ));
    assert_eq!(owner_sink.0.peak.load(Ordering::Acquire), previous_peak);
    assert_eq!(owner_sink.0.live.load(Ordering::Acquire), 0);
}

#[test]
fn one_let_type_stage_refuses_unknown_host_field_before_debit() {
    let authority = authority();
    let owner_sink = sink();
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        SOURCE_ID,
        UNKNOWN_HOST_FIELD,
        &owner_sink,
    );
    let bound = resolved(UNKNOWN_HOST_FIELD, &authority, &root);
    let before_peak = owner_sink.0.peak.load(Ordering::Acquire);
    assert!(matches!(
        type_source_body_with_request_owner_v3(UNKNOWN_HOST_FIELD, &root, bound),
        Err(SourceBodyTypesRequestErrorV3::UnsupportedShape)
    ));
    assert_eq!(owner_sink.0.peak.load(Ordering::Acquire), before_peak);
    assert_eq!(owner_sink.0.live.load(Ordering::Acquire), 0);
}
