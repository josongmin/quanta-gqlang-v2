#![expect(clippy::expect_used, reason = "independent permit custody assertions")]

use quanta_gqlang_v2_hir::{
    FrontendRequestMaterializationRootV3, FrontendRequestStoragePermitV3,
    FrontendRequestStorageSinkV3, FrontendStorageRefusalV3, SourceIdV2,
};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

struct Sink(Arc<AtomicU64>);
struct Charge {
    ledger: Arc<AtomicU64>,
    bytes: u64,
}

impl Drop for Charge {
    fn drop(&mut self) {
        self.ledger.fetch_sub(self.bytes, Ordering::SeqCst);
    }
}

impl FrontendRequestStorageSinkV3 for Sink {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
        let bytes = logical_bytes
            .checked_add(
                u64::try_from(std::mem::size_of::<Charge>())
                    .map_err(|_size_overflow| FrontendStorageRefusalV3::DemandOverflow)?,
            )
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        self.0.fetch_add(bytes, Ordering::SeqCst);
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
fn minimum_is_current_stage_demand_without_ancestor_or_carrier_credit() {
    let source_id = SourceIdV2::from_raw(19);
    let source = "show modules";
    let ledger = Arc::new(AtomicU64::new(0));
    let sink = Sink(ledger.clone());
    let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &sink);
    let mut storage = root
        .reserve_v3(source_id, source, 100)
        .expect("first stage");
    assert_eq!(
        storage.validate_minimum_reservation_v3(&root, source_id, source, 100),
        Ok(())
    );
    storage
        .try_extend_v3(&root, source_id, source, 8)
        .expect("second stage");
    assert_eq!(
        storage.validate_minimum_reservation_v3(&root, source_id, source, 8),
        Ok(())
    );
    assert_eq!(
        storage.validate_minimum_reservation_v3(&root, source_id, source, 9),
        Err(FrontendStorageRefusalV3::BudgetRejected)
    );
    assert!(ledger.load(Ordering::SeqCst) > 108);
    drop(storage);
    assert_eq!(ledger.load(Ordering::SeqCst), 0);
}

#[test]
fn same_source_foreign_root_is_rejected_even_for_zero_demand() {
    let source_id = SourceIdV2::from_raw(20);
    let source = "show modules";
    let first_ledger = Arc::new(AtomicU64::new(0));
    let second_ledger = Arc::new(AtomicU64::new(0));
    let first = Sink(first_ledger.clone());
    let second = Sink(second_ledger.clone());
    let first_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &first);
    let second_root =
        FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &second);
    let storage = first_root
        .reserve_v3(source_id, source, 0)
        .expect("issued zero demand");
    assert_eq!(
        storage.validate_minimum_reservation_v3(&second_root, source_id, source, 0),
        Err(FrontendStorageRefusalV3::ForeignRequestRoot)
    );
    assert_eq!(
        storage.validate_minimum_reservation_v3(&first_root, source_id, source, 0),
        Ok(())
    );
    drop(storage);
    assert_eq!(first_ledger.load(Ordering::SeqCst), 0);
    assert_eq!(second_ledger.load(Ordering::SeqCst), 0);
}
