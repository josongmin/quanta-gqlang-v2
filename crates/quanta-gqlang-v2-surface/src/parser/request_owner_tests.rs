#![expect(
    clippy::expect_used,
    reason = "unit tests assert exact request-root storage outcomes"
)]

use super::*;
use crate::materialization::{
    FrontendRequestStoragePermitV3, FrontendRequestStorageSinkV3, FrontendRetainedStoragePermitV3,
};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct TestSink {
    live: Arc<AtomicU64>,
    limit: u64,
}

struct TestPermit {
    live: Arc<AtomicU64>,
    bytes: u64,
}

impl Drop for TestPermit {
    fn drop(&mut self) {
        self.live.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

impl FrontendRequestStorageSinkV3 for TestSink {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        let carrier_bytes = u64::try_from(std::mem::size_of::<TestPermit>())
            .map_err(|_overflow| FrontendStorageRefusalV3::DemandOverflow)?;
        let bytes = logical_bytes
            .checked_add(carrier_bytes)
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        let mut live = self.live.load(Ordering::Acquire);
        loop {
            let next = live
                .checked_add(bytes)
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            if next > self.limit {
                return Err(FrontendStorageRefusalV3::BudgetRejected);
            }
            match self
                .live
                .compare_exchange_weak(live, next, Ordering::AcqRel, Ordering::Acquire)
            {
                Ok(_) => break,
                Err(observed) => live = observed,
            }
        }
        Ok(Box::new(TestPermit {
            live: Arc::clone(&self.live),
            bytes,
        }))
    }

    fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
        permit
            .as_any_v3()
            .downcast_ref::<TestPermit>()
            .is_some_and(|permit| Arc::ptr_eq(&self.live, &permit.live))
    }
}

struct PeakSink<'a> {
    inner: &'a TestSink,
    peak: Arc<AtomicU64>,
}

impl FrontendRequestStorageSinkV3 for PeakSink<'_> {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        let permit = self.inner.reserve_v3(logical_bytes)?;
        self.peak
            .fetch_max(self.inner.live.load(Ordering::Acquire), Ordering::AcqRel);
        Ok(permit)
    }

    fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
        self.inner.owns_permit_v3(permit)
    }
}

#[test]
fn request_root_preserves_raw_hir_and_malformed_diagnostic_v3() {
    let source_id = SourceIdV2::from_raw(111);
    let budget = ParseBudgetV2::standard_v2();
    for source in [
        "show capabilities",
        "let x = f(1, 2)",
        "let x = [1, 2]",
        "let a = from users u let b = from users v from users u with a",
        "from users u | project u.id as id, u.name as name => { id: u.id, tags: [ u.t1 ] }",
    ] {
        let source = String::from(source);
        let expected =
            parse_source_document_v2(source_id, &source, &budget).expect("raw source parses");
        let live = Arc::new(AtomicU64::new(0));
        let sink = TestSink {
            live: Arc::clone(&live),
            limit: u64::MAX,
        };
        let admitted = {
            let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
                source_id, &source, &sink,
            );
            parse_source_document_with_request_owner_v3(source_id, &source, &budget, &root)
                .expect("request source parses")
        };
        assert_eq!(&expected.hir, admitted.value_v3(), "{source}");
        assert!(live.load(Ordering::Acquire) > 0);
        drop(admitted);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }

    let malformed = String::from("let x = [1,]");
    let expected = parse_source_document_v2(source_id, &malformed, &budget)
        .expect_err("raw malformed list rejects");
    let live = Arc::new(AtomicU64::new(0));
    let sink = TestSink {
        live: Arc::clone(&live),
        limit: u64::MAX,
    };
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &malformed, &sink);
    let observed =
        parse_source_document_with_request_owner_v3(source_id, &malformed, &budget, &root)
            .err()
            .expect("request malformed list rejects");
    assert_eq!(observed, expected);
    assert_eq!(live.load(Ordering::Acquire), 0);
}

#[test]
fn request_composition_scratch_releases_before_owner_drop_v3() {
    let source_id = SourceIdV2::from_raw(113);
    let source = String::from("let a = from users u let b = from users v from users u with a");
    let budget = ParseBudgetV2::standard_v2();
    let live = Arc::new(AtomicU64::new(0));
    let peak = Arc::new(AtomicU64::new(0));
    let backing = TestSink {
        live: Arc::clone(&live),
        limit: u64::MAX,
    };
    let sink = PeakSink {
        inner: &backing,
        peak: Arc::clone(&peak),
    };
    let owner = {
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
        parse_source_document_with_request_owner_v3(source_id, &source, &budget, &root)
            .expect("source composition parses with request root")
    };
    let observed_peak = peak.load(Ordering::Acquire);
    assert!(observed_peak > live.load(Ordering::Acquire));
    drop(owner);
    assert_eq!(live.load(Ordering::Acquire), 0);

    let short_live = Arc::new(AtomicU64::new(0));
    let short_backing = TestSink {
        live: Arc::clone(&short_live),
        limit: observed_peak - 1,
    };
    let short_sink = PeakSink {
        inner: &short_backing,
        peak: Arc::new(AtomicU64::new(0)),
    };
    let root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &short_sink);
    assert!(
        parse_source_document_with_request_owner_v3(source_id, &source, &budget, &root,).is_err()
    );
    assert_eq!(short_live.load(Ordering::Acquire), 0);
}

#[test]
fn partial_request_root_show_ledger_short_foreign_and_last_drop_v3() {
    let source_id = SourceIdV2::from_raw(109);
    let source = String::from("show capabilities");
    let budget = ParseBudgetV2::standard_v2();
    let lexical = crate::lexer::census_lexical_materialization_v1(&source, &budget)
        .expect("lexical census")
        .logical_bytes();
    let parser = std::mem::size_of::<Vec<SurfaceItemV1>>() + std::mem::size_of::<SurfaceItemV1>();
    let hir =
        std::mem::size_of::<Vec<ProgramItemHirV1>>() + std::mem::size_of::<ProgramItemHirV1>();
    let show = std::mem::size_of::<quanta_gqlang_v2_hir::PresentationHirV1>();
    let links = 3 * std::mem::size_of::<FrontendRetainedStoragePermitV3>();
    let carriers = 4 * std::mem::size_of::<TestPermit>();
    let exact = u64::try_from(lexical + parser + hir + show + links + carriers)
        .expect("exact source-derived storage fits u64");

    for limit in [exact, exact - 1] {
        let live = Arc::new(AtomicU64::new(0));
        let sink = TestSink {
            live: Arc::clone(&live),
            limit,
        };
        let result = {
            let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
                source_id, &source, &sink,
            );
            let foreign = String::from("show capabilities");
            assert!(parse_source_document_with_request_owner_v3(
                source_id, &foreign, &budget, &root
            )
            .is_err());
            assert_eq!(live.load(Ordering::Acquire), 0);
            parse_source_document_with_request_owner_v3(source_id, &source, &budget, &root)
        };
        if limit == exact {
            let owner = result.expect("exact SHOW parse");
            assert_eq!(owner.value_v3().items.len(), 1);
            assert_eq!(live.load(Ordering::Acquire), exact);
            drop(owner);
        } else {
            assert!(result.is_err(), "one-byte-short root must refuse");
        }
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}

#[test]
fn partial_request_root_query_box_is_admitted_before_lift_v3() {
    let source_id = SourceIdV2::from_raw(110);
    let source = String::from("from records");
    let budget = ParseBudgetV2::standard_v2();
    let lexical = crate::lexer::census_lexical_materialization_v1(&source, &budget)
        .expect("lexical census")
        .logical_bytes();
    let parser = std::mem::size_of::<Vec<SurfaceItemV1>>() + std::mem::size_of::<SurfaceItemV1>();
    let hir =
        std::mem::size_of::<Vec<ProgramItemHirV1>>() + std::mem::size_of::<ProgramItemHirV1>();
    let box_bytes = std::mem::size_of::<QueryHirV1>();
    let depth = quanta_gqlang_v2_hir::depth_stack_storage_bytes_v3(2)
        .expect("source and query HIR nodes have finite depth frames");
    let final_live = lexical
        + parser
        + hir
        + box_bytes
        + 3 * std::mem::size_of::<FrontendRetainedStoragePermitV3>()
        + 4 * std::mem::size_of::<TestPermit>();
    let depth_peak = lexical
        + parser
        + std::mem::size_of::<FrontendRetainedStoragePermitV3>()
        + 2 * std::mem::size_of::<TestPermit>()
        + depth
        + std::mem::size_of::<TestPermit>();
    let exact =
        u64::try_from(final_live.max(depth_peak)).expect("source-derived query peak fits u64");
    let final_live = u64::try_from(final_live).expect("retained query storage fits u64");

    for limit in [exact, exact - 1] {
        let live = Arc::new(AtomicU64::new(0));
        let sink = TestSink {
            live: Arc::clone(&live),
            limit,
        };
        let result = {
            let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
                source_id, &source, &sink,
            );
            parse_source_document_with_request_owner_v3(source_id, &source, &budget, &root)
        };
        if limit == exact {
            let owner = result.expect("exact query parse");
            assert_eq!(owner.source_issued_node_slots_v3(), 2);
            assert!(matches!(
                owner.value_v3().items.as_slice(),
                [ProgramItemHirV1::Query(_)]
            ));
            assert_eq!(live.load(Ordering::Acquire), final_live);
            drop(owner);
        } else {
            assert!(result.is_err(), "one-byte-short query owner must refuse");
        }
        assert_eq!(live.load(Ordering::Acquire), 0);
    }

    // Refuse at the depth scratch's first reservation, before its Vec is
    // constructed. Earlier source/HIR allocations return with the owner.
    let depth_short = u64::try_from(depth_peak - 1).expect("depth short limit fits u64");
    let live = Arc::new(AtomicU64::new(0));
    let sink = TestSink {
        live: Arc::clone(&live),
        limit: depth_short,
    };
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
    assert!(
        parse_source_document_with_request_owner_v3(source_id, &source, &budget, &root).is_err()
    );
    assert_eq!(live.load(Ordering::Acquire), 0);

    // Matching bytes and source id cannot transfer the source-bound root
    // to another allocation. Refusal occurs before the lexical permit.
    let foreign = String::from("from records");
    assert!(!std::ptr::eq(source.as_ptr(), foreign.as_ptr()));
    let foreign_live = Arc::new(AtomicU64::new(0));
    let foreign_sink = TestSink {
        live: Arc::clone(&foreign_live),
        limit: exact,
    };
    let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
        source_id,
        &source,
        &foreign_sink,
    );
    assert!(parse_source_document_with_request_owner_v3(
        source_id,
        &foreign,
        &budget,
        &foreign_root,
    )
    .is_err());
    assert_eq!(foreign_live.load(Ordering::Acquire), 0);
}

#[test]
fn request_root_refuses_projection_vec_before_first_item_v3() {
    let source_id = SourceIdV2::from_raw(112);
    let source = String::from("from records | project a, b");
    let budget = ParseBudgetV2::standard_v2();
    let lexical = crate::lexer::census_lexical_materialization_v1(&source, &budget)
        .expect("lexical census")
        .logical_bytes();
    let parser = std::mem::size_of::<Vec<SurfaceItemV1>>() + std::mem::size_of::<SurfaceItemV1>();
    let already_live = lexical
        + parser
        + std::mem::size_of::<FrontendRetainedStoragePermitV3>()
        + 2 * std::mem::size_of::<TestPermit>();
    let projection_birth = 2 * std::mem::size_of::<quanta_gqlang_v2_hir::ProjectionItemV1>()
        + std::mem::size_of::<FrontendRetainedStoragePermitV3>()
        + std::mem::size_of::<TestPermit>();
    let limit = u64::try_from(already_live + projection_birth - 1)
        .expect("projection short limit fits u64");
    let live = Arc::new(AtomicU64::new(0));
    let sink = TestSink {
        live: Arc::clone(&live),
        limit,
    };
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, &source, &sink);
    assert!(
        parse_source_document_with_request_owner_v3(source_id, &source, &budget, &root).is_err()
    );
    assert_eq!(live.load(Ordering::Acquire), 0);
}
