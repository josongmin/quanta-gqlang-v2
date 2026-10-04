//! Host-issued source-bound request storage custody shared by frontend stages.

use crate::span::SourceIdV2;
use std::any::Any;

/// Host-owned request ledger. The host must reserve both `logical_bytes` and
/// its concrete boxed permit carrier before constructing the returned box.
/// The frontend neither issues a second budget nor accepts a numeric ceiling
/// supplied by source text.
pub trait FrontendRequestStorageSinkV3: Send + Sync {
    fn reserve_v3(
        &self,
        logical_bytes: u64,
    ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3>;

    /// The host compares the retained permit against its actual issued ledger.
    /// Source identity or an equal numeric policy does not establish custody.
    fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool;
}

/// Type-erased, move-only host reservation retained until the last payload
/// owner drops. Concrete hosts implement this through their issued root.
pub trait FrontendRequestStoragePermitV3: Any + Send + Sync {
    fn as_any_v3(&self) -> &dyn Any;
}

impl<T: Any + Send + Sync> FrontendRequestStoragePermitV3 for T {
    fn as_any_v3(&self) -> &dyn Any {
        self
    }
}

#[expect(
    clippy::as_conversions,
    reason = "MSRV-compatible live source address comparison; never dereferenced or used as a public identity"
)]
fn source_address_v3(source: &str) -> usize {
    source.as_ptr() as usize
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrontendStorageRefusalV3 {
    ForeignSource,
    ForeignRequestRoot,
    DemandOverflow,
    BudgetRejected,
}

/// Invocation-local source binding for the host's issued request ledger.
/// Its returned permits own the host reservation and may outlive this borrow;
/// source pointer equality here only checks continuity within this invocation.
pub struct FrontendRequestMaterializationRootV3<'a> {
    source_id: SourceIdV2,
    source: &'a str,
    sink: &'a dyn FrontendRequestStorageSinkV3,
}

impl<'a> FrontendRequestMaterializationRootV3<'a> {
    #[must_use]
    pub fn from_issued_sink_v3(
        source_id: SourceIdV2,
        source: &'a str,
        sink: &'a dyn FrontendRequestStorageSinkV3,
    ) -> Self {
        Self {
            source_id,
            source,
            sink,
        }
    }

    /// Borrow the invocation's original source for parser-owned copies.
    #[must_use]
    pub const fn source_v3(&self) -> &'a str {
        self.source
    }

    fn validate_source_v3(
        &self,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        if self.source_id != source_id
            || !std::ptr::eq(self.source.as_ptr(), source.as_ptr())
            || self.source.len() != source.len()
        {
            Err(FrontendStorageRefusalV3::ForeignSource)
        } else {
            Ok(())
        }
    }

    pub fn reserve_v3(
        &self,
        source_id: SourceIdV2,
        source: &str,
        logical_bytes: usize,
    ) -> Result<FrontendRetainedStoragePermitV3, FrontendStorageRefusalV3> {
        self.validate_source_v3(source_id, source)?;
        let bytes = u64::try_from(logical_bytes)
            .map_err(|_error| FrontendStorageRefusalV3::DemandOverflow)?;
        let permit = self.sink.reserve_v3(bytes)?;
        Ok(FrontendRetainedStoragePermitV3 {
            permit,
            previous: None,
            logical_bytes,
            source_id,
            source_address: source_address_v3(self.source),
            source_len: self.source.len(),
        })
    }
}

/// One inline permit plus a linked set of earlier stage permits. Each link's
/// Box storage is charged to the same host root before the Box is created.
/// No Vec growth or cloned permit can silently detach the previous stages.
pub struct FrontendRetainedStoragePermitV3 {
    permit: Box<dyn FrontendRequestStoragePermitV3>,
    previous: Option<Box<Self>>,
    logical_bytes: usize,
    source_id: SourceIdV2,
    source_address: usize,
    source_len: usize,
}

impl Drop for FrontendRetainedStoragePermitV3 {
    fn drop(&mut self) {
        // Reservation count follows allocation count, not HIR nesting depth.
        // Detach every next link before dropping its node to bound the stack.
        let mut previous = self.previous.take();
        while let Some(mut node) = previous {
            previous = node.previous.take();
        }
    }
}

impl FrontendRetainedStoragePermitV3 {
    /// Authenticate every retained stage against the supplied issued ledger.
    /// This observation does not reserve storage or transfer any permit.
    pub fn validate_issued_sink_v3(
        &self,
        sink: &dyn FrontendRequestStorageSinkV3,
    ) -> Result<(), FrontendStorageRefusalV3> {
        let mut current = Some(self);
        while let Some(permit) = current {
            if !sink.owns_permit_v3(permit.permit.as_ref()) {
                return Err(FrontendStorageRefusalV3::ForeignRequestRoot);
            }
            current = permit.previous.as_deref();
        }
        Ok(())
    }

    /// Check this issued stage reservation against demand derived by its
    /// producer. Earlier links and their carrier overhead do not count.
    pub fn validate_minimum_reservation_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        required: usize,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.validate_owner_v3(root, source_id, source)?;
        if self.logical_bytes < required {
            return Err(FrontendStorageRefusalV3::BudgetRejected);
        }
        Ok(())
    }

    /// Authenticate a later stage against the root that owns this live
    /// payload, without creating a second reservation.
    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        root.validate_source_v3(source_id, source)?;
        if self.source_id != source_id
            || self.source_address != source_address_v3(source)
            || self.source_len != source.len()
            || !root.sink.owns_permit_v3(self.permit.as_ref())
        {
            return Err(FrontendStorageRefusalV3::ForeignRequestRoot);
        }
        Ok(())
    }

    pub fn try_extend_v3(
        &mut self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
        logical_bytes: usize,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.validate_owner_v3(root, source_id, source)?;
        let bytes = logical_bytes
            .checked_add(std::mem::size_of::<Self>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
        let permit = root.sink.reserve_v3(bytes)?;
        let replacement = Self {
            permit,
            previous: None,
            logical_bytes,
            source_id,
            source_address: self.source_address,
            source_len: self.source_len,
        };
        let prior = std::mem::replace(self, replacement);
        self.previous = Some(Box::new(prior));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    struct Sink {
        issuer: Arc<()>,
        live: Arc<AtomicU64>,
        next: AtomicU64,
    }

    struct Permit {
        issuer: Arc<()>,
        live: Arc<AtomicU64>,
        ordinal: u64,
        bytes: u64,
    }

    impl Drop for Permit {
        fn drop(&mut self) {
            self.live.fetch_sub(self.bytes, Ordering::AcqRel);
        }
    }

    impl FrontendRequestStorageSinkV3 for Sink {
        fn reserve_v3(
            &self,
            logical_bytes: u64,
        ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
            let bytes = logical_bytes
                .checked_add(
                    u64::try_from(std::mem::size_of::<Permit>())
                        .map_err(|_error| FrontendStorageRefusalV3::DemandOverflow)?,
                )
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            self.live
                .fetch_update(Ordering::AcqRel, Ordering::Acquire, |live| {
                    live.checked_add(bytes)
                })
                .map_err(|_live| FrontendStorageRefusalV3::BudgetRejected)?;
            Ok(Box::new(Permit {
                issuer: Arc::clone(&self.issuer),
                live: Arc::clone(&self.live),
                ordinal: self.next.fetch_add(1, Ordering::AcqRel),
                bytes,
            }))
        }

        fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
            permit
                .as_any_v3()
                .downcast_ref::<Permit>()
                .is_some_and(|permit| Arc::ptr_eq(&self.issuer, &permit.issuer))
        }
    }

    struct HeadOnlySink<'a> {
        issued: &'a Sink,
        probes: AtomicU64,
    }

    impl FrontendRequestStorageSinkV3 for HeadOnlySink<'_> {
        fn reserve_v3(
            &self,
            _logical_bytes: u64,
        ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
            Err(FrontendStorageRefusalV3::BudgetRejected)
        }

        fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
            self.probes.fetch_add(1, Ordering::AcqRel);
            self.issued.owns_permit_v3(permit)
                && permit
                    .as_any_v3()
                    .downcast_ref::<Permit>()
                    .is_some_and(|permit| permit.ordinal == 1)
        }
    }

    #[test]
    fn sink_authentication_includes_previous_stage_and_releases_all_custody() {
        let live = Arc::new(AtomicU64::new(0));
        let sink = Sink {
            issuer: Arc::new(()),
            live: Arc::clone(&live),
            next: AtomicU64::new(0),
        };
        let foreign = Sink {
            issuer: Arc::new(()),
            live: Arc::new(AtomicU64::new(0)),
            next: AtomicU64::new(0),
        };
        let source_id = SourceIdV2::from_raw(7);
        let source = "from files f | project f.path as path";
        let root =
            FrontendRequestMaterializationRootV3::from_issued_sink_v3(source_id, source, &sink);
        let reserved = root.reserve_v3(source_id, source, 16);
        assert!(reserved.is_ok(), "the fixture must issue the first stage");
        let Ok(mut permit) = reserved else {
            return;
        };
        assert_eq!(permit.try_extend_v3(&root, source_id, source, 32), Ok(()));
        let retained = live.load(Ordering::Acquire);
        assert!(retained > 0);
        assert_eq!(permit.validate_issued_sink_v3(&sink), Ok(()));
        assert_eq!(
            permit.validate_issued_sink_v3(&foreign),
            Err(FrontendStorageRefusalV3::ForeignRequestRoot)
        );
        let head_only = HeadOnlySink {
            issued: &sink,
            probes: AtomicU64::new(0),
        };
        assert_eq!(
            permit.validate_issued_sink_v3(&head_only),
            Err(FrontendStorageRefusalV3::ForeignRequestRoot)
        );
        assert_eq!(
            head_only.probes.load(Ordering::Acquire),
            2,
            "authenticating only the last stage would admit foreign retained custody"
        );
        assert_eq!(live.load(Ordering::Acquire), retained);
        assert_eq!(foreign.live.load(Ordering::Acquire), 0);
        drop(permit);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }
}
