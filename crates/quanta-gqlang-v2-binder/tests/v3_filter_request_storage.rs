#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "request-owner regression fixture"
)]

use quanta_gqlang_v2_binder::{Binder, BinderEvidenceReaderV3, BinderKindV1, ModuleGraphV1};
use quanta_gqlang_v2_hir::{
    AliasV1, CheckedHirDocumentOwnedV2, FrontendRequestMaterializationRootV3,
    FrontendRequestStoragePermitV3, FrontendRequestStorageSinkV3, FrontendStorageRefusalV3,
    HirBuilderV1, HirDocumentV1, PathSegmentV1, ProgramItemHirV1, ProjectionItemV1, SourceIdV2,
    SpanV1,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

const SOURCE: &str = "from records r | where r.id > 3 | project r.id as id";
const SOURCE_ID: SourceIdV2 = SourceIdV2::from_raw(441);

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
        self.ledger.live.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}
impl FrontendRequestStorageSinkV3 for Sink {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        let bytes = logical_bytes
            .checked_add(u64::try_from(std::mem::size_of::<Charge>()).unwrap())
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        let mut prior = self.0.live.load(Ordering::Acquire);
        loop {
            let next = prior
                .checked_add(bytes)
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            if next > self.0.limit.load(Ordering::Acquire) {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            match self.0.live.compare_exchange_weak(
                prior,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    self.0.peak.fetch_max(next, Ordering::AcqRel);
                    break;
                }
                Err(observed) => prior = observed,
            }
        }
        Ok(Box::new(Charge {
            ledger: self.0.clone(),
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

#[test]
fn filter_project_request_retains_alias_across_filter_and_refunds_on_last_drop() {
    let b = HirBuilderV1::new();
    let all = SpanV1::new(0, SOURCE.len());
    let scan = b.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        all,
    );
    let scan_id = scan.id;
    let predicate_base = b.symbol("r", SpanV1::new(23, 24));
    let predicate_id = predicate_base.id;
    let predicate_path = b
        .try_path(
            predicate_base,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(25, 27),
            }],
            SpanV1::new(23, 27),
        )
        .unwrap();
    let predicate = b.compare(
        ">",
        predicate_path,
        b.int("3", SpanV1::new(30, 31)),
        SpanV1::new(23, 31),
    );
    let filter = b.relation_filter(scan, predicate, all);
    let filter_id = filter.id;
    let project_base = b.symbol("r", SpanV1::new(42, 43));
    let project_id = project_base.id;
    let project_path = b
        .try_path(
            project_base,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(44, 46),
            }],
            SpanV1::new(42, 46),
        )
        .unwrap();
    let projected = b
        .try_project(
            filter,
            vec![ProjectionItemV1 {
                value: project_path,
                alias: Some(AliasV1 {
                    name: "id".into(),
                    span: SpanV1::new(50, 52),
                }),
            }],
            all,
        )
        .unwrap();
    let projected_id = projected.id;
    let query = b.query(projected, None, all);
    let mut ordinary = Binder::new(ModuleGraphV1::new());
    ordinary.resolve_query(&query);
    let ordinary = ordinary.finish();
    assert!(ordinary.is_ok());
    let checked = CheckedHirDocumentOwnedV2::try_new(HirDocumentV1 {
        source_id: SOURCE_ID,
        source_len: SOURCE.len(),
        items: vec![ProgramItemHirV1::Query(Box::new(query))],
    })
    .unwrap();
    let ledger = Arc::new(Ledger {
        live: AtomicU64::new(0),
        peak: AtomicU64::new(0),
        limit: AtomicU64::new(u64::MAX),
    });
    let sink = Sink(ledger.clone());
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(SOURCE_ID, SOURCE, &sink);
    let parent = root.reserve_v3(SOURCE_ID, SOURCE, 0).unwrap();
    let admitted = Binder::new(ModuleGraphV1::new())
        .declare_checked_program_with_request_owner_v3(&checked, &root, SOURCE_ID, SOURCE, &parent)
        .unwrap()
        .resolve_checked_source_bindings_with_request_owner_v3(
            &checked, &root, SOURCE_ID, SOURCE, &parent,
        )
        .expect("request Filter→Project Binder");
    for relation in [scan_id, filter_id, projected_id] {
        assert_eq!(
            admitted.relation_output(relation),
            ordinary.relation_output(relation)
        );
    }
    for reference in [predicate_id, project_id] {
        assert_eq!(
            admitted.resolution_of(reference),
            ordinary.resolution_of(reference)
        );
    }
    let scan_bindings = admitted.relation_output(scan_id).unwrap();
    assert_eq!(scan_bindings.len(), 1);
    let scan_binding = scan_bindings.first().unwrap();
    assert_eq!(scan_binding.name, "r");
    assert_eq!(scan_binding.kind, BinderKindV1::Alias);
    assert_eq!(
        admitted.relation_output(filter_id),
        admitted.relation_output(scan_id)
    );
    drop(parent);
    assert!(ledger.live.load(Ordering::Acquire) > 0);
    drop(admitted);
    assert_eq!(ledger.live.load(Ordering::Acquire), 0);
}
