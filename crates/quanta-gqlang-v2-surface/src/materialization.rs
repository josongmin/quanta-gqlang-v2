//! Source-bound surface storage admission. The request-root producer covers
//! lexer storage and parser-authored HIR containers and copies, including
//! nested expression and composition producers. Semantic, planning, and
//! execution storage require later producer admission.

use std::sync::atomic::{AtomicU64, Ordering};

use quanta_gqlang_v2_hir::{PresentationHirV1, SourceIdV2, SpanV1};

use crate::diag::{ParseDiagnosticV1, ParseErrorCodeV1};

// One host-issued storage primitive is shared by all frontend stages.
pub use quanta_gqlang_v2_hir::{
    FrontendRequestMaterializationRootV3, FrontendRequestStoragePermitV3,
    FrontendRequestStorageSinkV3, FrontendRetainedStoragePermitV3, FrontendStorageRefusalV3,
};

/// A produced value and every request-root reservation that was live while it
/// was built. Moving the value through the frontend stages moves custody too.
pub struct FrontendRequestOwnedValueV3<T> {
    value: T,
    permit: FrontendRetainedStoragePermitV3,
    source_issued_node_slots: usize,
}

impl<T> FrontendRequestOwnedValueV3<T> {
    pub(crate) fn new_v3(
        value: T,
        permit: FrontendRetainedStoragePermitV3,
        source_issued_node_slots: usize,
    ) -> Self {
        Self {
            value,
            permit,
            source_issued_node_slots,
        }
    }

    #[must_use]
    pub fn value_v3(&self) -> &T {
        &self.value
    }

    /// Source parser's builder census retained across trusted frontend stages.
    #[must_use]
    pub fn source_issued_node_slots_v3(&self) -> usize {
        self.source_issued_node_slots
    }

    /// Authenticate the retained parser/stage custody without exposing it.
    pub fn validate_owner_v3(
        &self,
        root: &FrontendRequestMaterializationRootV3<'_>,
        source_id: SourceIdV2,
        source: &str,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.permit.validate_owner_v3(root, source_id, source)
    }

    /// Observe that every retained stage belongs to the issued host ledger.
    pub fn validate_issued_sink_v3(
        &self,
        sink: &dyn FrontendRequestStorageSinkV3,
    ) -> Result<(), FrontendStorageRefusalV3> {
        self.permit.validate_issued_sink_v3(sink)
    }

    /// The producer receives the retained permit before constructing its next
    /// value. It must extend the same root for each allocation it will make.
    pub fn try_map_v3<U, E>(
        self,
        produce: impl FnOnce(T, &mut FrontendRetainedStoragePermitV3) -> Result<U, E>,
    ) -> Result<FrontendRequestOwnedValueV3<U>, E> {
        let mut permit = self.permit;
        let value = produce(self.value, &mut permit)?;
        Ok(FrontendRequestOwnedValueV3 {
            value,
            permit,
            source_issued_node_slots: self.source_issued_node_slots,
        })
    }
}

/// A finite ceiling supplied separately from a request's own limit. This
/// public type does not authenticate its issuer; the downstream admitted host
/// must install it from its own policy, never from public request bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontendLexicalHostCeilingV1 {
    max_live_bytes: u64,
}

impl FrontendLexicalHostCeilingV1 {
    pub fn try_new(max_live_bytes: u64) -> Result<Self, &'static str> {
        if max_live_bytes == 0 {
            return Err("lexical host ceiling must be positive");
        }
        Ok(Self { max_live_bytes })
    }
}

/// One invocation's lexical ledger. Exact source pointer and transaction-local
/// ID give local consistency while the borrow is live. They do not prove a
/// downstream session generation or host authorization.
pub struct FrontendLexicalMaterializationRootV1<'source> {
    source_id: SourceIdV2,
    source: &'source str,
    max_live_bytes: u64,
    live_bytes: AtomicU64,
}

impl<'source> FrontendLexicalMaterializationRootV1<'source> {
    pub fn issue_v1(
        source_id: SourceIdV2,
        source: &'source str,
        requested_max_live_bytes: u64,
        installed_host: FrontendLexicalHostCeilingV1,
    ) -> Result<Self, &'static str> {
        if requested_max_live_bytes == 0 || requested_max_live_bytes > installed_host.max_live_bytes
        {
            return Err("lexical request limit exceeds installed host ceiling");
        }
        Ok(Self {
            source_id,
            source,
            max_live_bytes: requested_max_live_bytes,
            live_bytes: AtomicU64::new(0),
        })
    }

    pub(crate) fn reserve_v1<'root>(
        &'root self,
        source_id: SourceIdV2,
        source: &str,
        bytes: usize,
    ) -> Result<FrontendLexicalPermitV1<'root, 'source>, ParseDiagnosticV1> {
        self.reserve_at_v1(source_id, source, bytes, SpanV1::new(0, source.len()))
    }

    /// Debit a later producer's checked logical storage before its first
    /// allocation. The producer must derive `bytes` from its source-owned
    /// object count; this low-level debit does not certify that census itself.
    pub fn reserve_stage_v2<'root>(
        &'root self,
        source_id: SourceIdV2,
        source: &str,
        bytes: usize,
    ) -> Result<FrontendLexicalPermitV1<'root, 'source>, ParseDiagnosticV1> {
        self.reserve_v1(source_id, source, bytes)
    }

    fn reserve_at_v1<'root>(
        &'root self,
        source_id: SourceIdV2,
        source: &str,
        bytes: usize,
        span: SpanV1,
    ) -> Result<FrontendLexicalPermitV1<'root, 'source>, ParseDiagnosticV1> {
        self.validate_source_v1(source_id, source, span)?;
        let bytes = u64::try_from(bytes).map_err(|_error| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::LexicalMaterializationOverflow,
                span,
                "lexical materialization size is not representable",
            )
        })?;
        self.charge_v1(bytes, span)?;
        Ok(FrontendLexicalPermitV1 { root: self, bytes })
    }

    fn validate_source_v1(
        &self,
        source_id: SourceIdV2,
        source: &str,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        if source_id != self.source_id
            || !std::ptr::eq(source.as_ptr(), self.source.as_ptr())
            || source.len() != self.source.len()
        {
            return Err(ParseDiagnosticV1::new(
                ParseErrorCodeV1::ResourceExhausted,
                span,
                "lexical materialization owner belongs to another source invocation",
            ));
        }
        Ok(())
    }

    fn charge_v1(&self, bytes: u64, span: SpanV1) -> Result<(), ParseDiagnosticV1> {
        let mut live = self.live_bytes.load(Ordering::Acquire);
        loop {
            let next = live.checked_add(bytes).ok_or_else(|| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::LexicalMaterializationOverflow,
                    span,
                    "lexical materialization live size overflow",
                )
            })?;
            if next > self.max_live_bytes {
                return Err(ParseDiagnosticV1::new(
                    ParseErrorCodeV1::ResourceExhausted,
                    span,
                    "lexical materialization exceeds the admitted live byte limit",
                ));
            }
            match self.live_bytes.compare_exchange_weak(
                live,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(()),
                Err(observed) => live = observed,
            }
        }
    }

    #[must_use]
    pub fn live_bytes_v1(&self) -> u64 {
        self.live_bytes.load(Ordering::Acquire)
    }
}

/// Move-only stage permit retained with the parsed or compiled source owner.
pub struct FrontendLexicalPermitV1<'root, 'source> {
    root: &'root FrontendLexicalMaterializationRootV1<'source>,
    bytes: u64,
}

impl FrontendLexicalPermitV1<'_, '_> {
    pub(crate) fn belongs_to_root_v2(
        &self,
        root: &FrontendLexicalMaterializationRootV1<'_>,
    ) -> bool {
        std::ptr::eq(self.root, root)
    }

    fn try_extend_v1(
        &mut self,
        source_id: SourceIdV2,
        source: &str,
        bytes: usize,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        self.root.validate_source_v1(source_id, source, span)?;
        let bytes = u64::try_from(bytes).map_err(|_error| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::LexicalMaterializationOverflow,
                span,
                "reference materialization size is not representable",
            )
        })?;
        let next = self.bytes.checked_add(bytes).ok_or_else(|| {
            ParseDiagnosticV1::new(
                ParseErrorCodeV1::LexicalMaterializationOverflow,
                span,
                "reference materialization aggregate size overflow",
            )
        })?;
        self.root.charge_v1(bytes, span)?;
        self.bytes = next;
        Ok(())
    }
}

/// Exact prebirth debits at the common reference and SHOW box producers.
pub(crate) trait ReferenceCloneAdmissionV1 {
    fn reserve_reference_v1(&mut self, source: &str, span: SpanV1)
        -> Result<(), ParseDiagnosticV1>;
    fn reserve_show_box_v1(&mut self, source: &str, span: SpanV1) -> Result<(), ParseDiagnosticV1>;

    fn is_request_owner_v3(&self) -> bool {
        false
    }

    /// The existing lexical-only owner does not qualify general nested HIR.
    /// Its default keeps that partial contract explicit while request-root
    /// producers can reserve each concrete birth at the parser call site.
    fn reserve_nested_bytes_v3(
        &mut self,
        _bytes: usize,
        _span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        Ok(())
    }

    /// Scratch has its own permit and is released when the check returns.
    fn reserve_temporary_scratch_v3(
        &mut self,
        _bytes: usize,
        _span: SpanV1,
    ) -> Result<Option<FrontendRetainedStoragePermitV3>, ParseDiagnosticV1> {
        Ok(None)
    }
}

/// One aggregate permit avoids allocating permit metadata for each parser node.
/// It is moved into the parsed owner after all parser branches complete.
pub(crate) struct ReferenceCloneAggregateV1<'root, 'source> {
    root: &'root FrontendLexicalMaterializationRootV1<'source>,
    source_id: SourceIdV2,
    permit: Option<FrontendLexicalPermitV1<'root, 'source>>,
}

impl<'root, 'source> ReferenceCloneAggregateV1<'root, 'source> {
    pub(crate) fn new(
        root: &'root FrontendLexicalMaterializationRootV1<'source>,
        source_id: SourceIdV2,
    ) -> Self {
        Self {
            root,
            source_id,
            permit: None,
        }
    }

    pub(crate) fn into_permit_v1(self) -> Option<FrontendLexicalPermitV1<'root, 'source>> {
        self.permit
    }

    fn reserve_bytes_v1(
        &mut self,
        source: &str,
        span: SpanV1,
        bytes: usize,
    ) -> Result<(), ParseDiagnosticV1> {
        self.root.validate_source_v1(self.source_id, source, span)?;
        match self.permit.as_mut() {
            Some(permit) => permit.try_extend_v1(self.source_id, source, bytes, span),
            None => {
                self.permit = Some(
                    self.root
                        .reserve_at_v1(self.source_id, source, bytes, span)?,
                );
                Ok(())
            }
        }
    }
}

impl ReferenceCloneAdmissionV1 for ReferenceCloneAggregateV1<'_, '_> {
    fn reserve_reference_v1(
        &mut self,
        source: &str,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        let bytes = source
            .get(span.start..span.end)
            .ok_or_else(|| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::UnexpectedToken,
                    span,
                    "reference span is outside source",
                )
            })?
            .len();
        self.reserve_bytes_v1(source, span, bytes)
    }

    fn reserve_show_box_v1(&mut self, source: &str, span: SpanV1) -> Result<(), ParseDiagnosticV1> {
        self.reserve_bytes_v1(source, span, std::mem::size_of::<PresentationHirV1>())
    }
}

/// Reuses the parser's exact reference and SHOW-box producers with the host
/// request root. The first parser reservation already owns the permit; each
/// nested allocation extends that same owner before construction.
pub(crate) struct RequestReferenceAggregateV3<'a, 'source, 'permit> {
    root: &'a FrontendRequestMaterializationRootV3<'source>,
    source_id: SourceIdV2,
    permit: &'permit mut FrontendRetainedStoragePermitV3,
}

impl<'a, 'source, 'permit> RequestReferenceAggregateV3<'a, 'source, 'permit> {
    pub(crate) fn new_v3(
        root: &'a FrontendRequestMaterializationRootV3<'source>,
        source_id: SourceIdV2,
        permit: &'permit mut FrontendRetainedStoragePermitV3,
    ) -> Self {
        Self {
            root,
            source_id,
            permit,
        }
    }

    fn reserve_bytes_v3(
        &mut self,
        source: &str,
        span: SpanV1,
        bytes: usize,
    ) -> Result<(), ParseDiagnosticV1> {
        self.permit
            .try_extend_v3(self.root, self.source_id, source, bytes)
            .map_err(|refusal| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::ResourceExhausted,
                    span,
                    match refusal {
                        FrontendStorageRefusalV3::ForeignSource => {
                            "frontend request source mismatch"
                        }
                        FrontendStorageRefusalV3::ForeignRequestRoot => {
                            "frontend request root mismatch"
                        }
                        FrontendStorageRefusalV3::DemandOverflow => {
                            "frontend request storage demand overflow"
                        }
                        FrontendStorageRefusalV3::BudgetRejected => {
                            "frontend request storage budget rejected"
                        }
                    },
                )
            })
    }
}

impl ReferenceCloneAdmissionV1 for RequestReferenceAggregateV3<'_, '_, '_> {
    fn is_request_owner_v3(&self) -> bool {
        true
    }
    fn reserve_reference_v1(
        &mut self,
        source: &str,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        let bytes = source
            .get(span.start..span.end)
            .ok_or_else(|| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::UnexpectedToken,
                    span,
                    "reference span is outside source",
                )
            })?
            .len();
        self.reserve_bytes_v3(source, span, bytes)
    }

    fn reserve_show_box_v1(&mut self, source: &str, span: SpanV1) -> Result<(), ParseDiagnosticV1> {
        self.reserve_bytes_v3(source, span, std::mem::size_of::<PresentationHirV1>())
    }

    fn reserve_nested_bytes_v3(
        &mut self,
        bytes: usize,
        span: SpanV1,
    ) -> Result<(), ParseDiagnosticV1> {
        self.reserve_bytes_v3(self.root.source_v3(), span, bytes)
    }

    fn reserve_temporary_scratch_v3(
        &mut self,
        bytes: usize,
        span: SpanV1,
    ) -> Result<Option<FrontendRetainedStoragePermitV3>, ParseDiagnosticV1> {
        self.root
            .reserve_v3(self.source_id, self.root.source_v3(), bytes)
            .map(Some)
            .map_err(|refusal| {
                ParseDiagnosticV1::new(
                    ParseErrorCodeV1::ResourceExhausted,
                    span,
                    match refusal {
                        FrontendStorageRefusalV3::ForeignSource => {
                            "frontend request source mismatch"
                        }
                        FrontendStorageRefusalV3::ForeignRequestRoot => {
                            "frontend request root mismatch"
                        }
                        FrontendStorageRefusalV3::DemandOverflow => {
                            "frontend request storage demand overflow"
                        }
                        FrontendStorageRefusalV3::BudgetRejected => {
                            "frontend request storage budget rejected"
                        }
                    },
                )
            })
    }
}

impl Drop for FrontendLexicalPermitV1<'_, '_> {
    fn drop(&mut self) {
        self.root.live_bytes.fetch_sub(self.bytes, Ordering::AcqRel);
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "unit tests assert exact budget and ownership outcomes"
    )]

    use super::*;
    use std::sync::Arc;

    struct CountingRequestSinkV3 {
        limit: u64,
        live: Arc<AtomicU64>,
    }

    struct CountingRequestPermitV3 {
        live: Arc<AtomicU64>,
        bytes: u64,
    }

    impl Drop for CountingRequestPermitV3 {
        fn drop(&mut self) {
            self.live.fetch_sub(self.bytes, Ordering::AcqRel);
        }
    }

    impl FrontendRequestStorageSinkV3 for CountingRequestSinkV3 {
        fn reserve_v3(
            &self,
            logical_bytes: u64,
        ) -> Result<Box<dyn FrontendRequestStoragePermitV3>, FrontendStorageRefusalV3> {
            let bytes = logical_bytes
                .checked_add(
                    u64::try_from(std::mem::size_of::<CountingRequestPermitV3>())
                        .expect("test storage fits u64"),
                )
                .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
            let mut live = self.live.load(Ordering::Acquire);
            loop {
                let next = live
                    .checked_add(bytes)
                    .ok_or(FrontendStorageRefusalV3::DemandOverflow)?;
                if next > self.limit {
                    return Err(FrontendStorageRefusalV3::BudgetRejected);
                }
                match self.live.compare_exchange_weak(
                    live,
                    next,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                ) {
                    Ok(_) => break,
                    Err(observed) => live = observed,
                }
            }
            Ok(Box::new(CountingRequestPermitV3 {
                live: Arc::clone(&self.live),
                bytes,
            }))
        }

        fn owns_permit_v3(&self, permit: &dyn FrontendRequestStoragePermitV3) -> bool {
            permit
                .as_any_v3()
                .downcast_ref::<CountingRequestPermitV3>()
                .is_some_and(|permit| Arc::ptr_eq(&self.live, &permit.live))
        }
    }

    #[test]
    fn request_permit_chain_releases_on_a_small_stack_v3() {
        std::thread::Builder::new()
            .stack_size(64 * 1024)
            .spawn(|| {
                let links = 16_384_u64;
                let carrier = u64::try_from(std::mem::size_of::<CountingRequestPermitV3>())
                    .expect("test storage fits u64");
                let link = u64::try_from(std::mem::size_of::<FrontendRetainedStoragePermitV3>())
                    .expect("test storage fits u64");
                let exact = carrier + links * (carrier + link);
                let source = String::from("show capabilities");
                let source_id = SourceIdV2::from_raw(72);
                let live = Arc::new(AtomicU64::new(0));
                let sink = CountingRequestSinkV3 {
                    limit: exact,
                    live: Arc::clone(&live),
                };
                let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
                    source_id, &source, &sink,
                );
                let mut owner = root.reserve_v3(source_id, &source, 0).unwrap();
                for _ in 0..links {
                    owner.try_extend_v3(&root, source_id, &source, 0).unwrap();
                }
                assert_eq!(live.load(Ordering::Acquire), exact);
                drop(owner);
                assert_eq!(live.load(Ordering::Acquire), 0);
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn request_sink_retains_exact_stage_chain_and_refuses_foreign_or_short_v3() {
        let source = String::from("show capabilities");
        let foreign_source = String::from("show capabilities");
        let source_id = SourceIdV2::from_raw(71);
        let exact = 3_u64
            + 5
            + u64::try_from(std::mem::size_of::<FrontendRetainedStoragePermitV3>())
                .expect("test storage fits u64")
            + 2 * u64::try_from(std::mem::size_of::<CountingRequestPermitV3>())
                .expect("test storage fits u64");
        let live = Arc::new(AtomicU64::new(0));
        let sink = CountingRequestSinkV3 {
            limit: exact,
            live: Arc::clone(&live),
        };
        let foreign_sink = CountingRequestSinkV3 {
            limit: exact,
            live: Arc::new(AtomicU64::new(0)),
        };
        let owner = {
            let root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
                source_id, &source, &sink,
            );
            assert!(matches!(
                root.reserve_v3(source_id, &foreign_source, 3),
                Err(FrontendStorageRefusalV3::ForeignSource)
            ));
            let mut owner = root.reserve_v3(source_id, &source, 3).unwrap();
            let foreign_root = FrontendRequestMaterializationRootV3::from_issued_sink_v3(
                source_id,
                &source,
                &foreign_sink,
            );
            assert_eq!(
                owner.try_extend_v3(&foreign_root, source_id, &source, 5),
                Err(FrontendStorageRefusalV3::ForeignRequestRoot)
            );
            assert_eq!(
                live.load(Ordering::Acquire),
                3 + u64::try_from(std::mem::size_of::<CountingRequestPermitV3>())
                    .expect("test storage fits u64")
            );
            owner.try_extend_v3(&root, source_id, &source, 5).unwrap();
            assert_eq!(live.load(Ordering::Acquire), exact);
            assert_eq!(
                owner.try_extend_v3(&root, source_id, &source, 1),
                Err(FrontendStorageRefusalV3::BudgetRejected)
            );
            assert_eq!(live.load(Ordering::Acquire), exact);
            owner
        };
        assert_eq!(live.load(Ordering::Acquire), exact);
        drop(owner);
        assert_eq!(live.load(Ordering::Acquire), 0);
    }

    #[test]
    fn exact_one_short_and_foreign_source_are_distinct() {
        let source = String::from("show models");
        let id = SourceIdV2::from_raw(7);
        let host = FrontendLexicalHostCeilingV1::try_new(100).unwrap();
        let root = FrontendLexicalMaterializationRootV1::issue_v1(id, &source, 8, host).unwrap();
        assert!(root.reserve_v1(id, &source, 9).is_err());
        let foreign = String::from("show models");
        assert!(root.reserve_v1(id, &foreign, 8).is_err());
        assert!(root
            .reserve_v1(SourceIdV2::from_raw(8), &source, 8)
            .is_err());
        let permit = root.reserve_v1(id, &source, 8).unwrap();
        assert_eq!(root.live_bytes_v1(), 8);
        assert!(root.reserve_v1(id, &source, 1).is_err());
        drop(permit);
        assert_eq!(root.live_bytes_v1(), 0);
    }

    #[test]
    fn requested_limit_cannot_install_its_own_host_ceiling() {
        let source = "show models";
        let host = FrontendLexicalHostCeilingV1::try_new(8).unwrap();
        assert!(FrontendLexicalMaterializationRootV1::issue_v1(
            SourceIdV2::from_raw(1),
            source,
            9,
            host,
        )
        .is_err());
    }

    #[test]
    fn reference_aggregate_extends_exactly_and_releases_after_short_failure() {
        let source = String::from("foo/bar");
        let foreign = String::from("foo/bar");
        let id = SourceIdV2::from_raw(9);
        let host = FrontendLexicalHostCeilingV1::try_new(6).unwrap();
        let root = FrontendLexicalMaterializationRootV1::issue_v1(id, &source, 6, host).unwrap();
        let mut aggregate = ReferenceCloneAggregateV1::new(&root, id);
        assert!(aggregate
            .reserve_reference_v1(&foreign, SpanV1::new(0, 3))
            .is_err());
        assert_eq!(root.live_bytes_v1(), 0);
        aggregate
            .reserve_reference_v1(&source, SpanV1::new(0, 3))
            .unwrap();
        aggregate
            .reserve_reference_v1(&source, SpanV1::new(4, 7))
            .unwrap();
        assert_eq!(root.live_bytes_v1(), 6);
        assert!(aggregate
            .reserve_reference_v1(&source, SpanV1::new(0, 1))
            .is_err());
        assert_eq!(root.live_bytes_v1(), 6);
        drop(aggregate);
        assert_eq!(root.live_bytes_v1(), 0);
    }

    #[test]
    fn show_box_debit_is_source_bound_exact_and_retained_until_owner_drop() {
        let source = String::from("show models");
        let foreign = String::from("show models");
        let id = SourceIdV2::from_raw(10);
        let box_bytes = std::mem::size_of::<PresentationHirV1>();
        let exact = u64::try_from(box_bytes).expect("box size fits u64");
        let host = FrontendLexicalHostCeilingV1::try_new(exact).unwrap();
        let root =
            FrontendLexicalMaterializationRootV1::issue_v1(id, &source, exact, host).unwrap();
        let mut aggregate = ReferenceCloneAggregateV1::new(&root, id);
        let span = SpanV1::new(0, source.len());
        assert!(aggregate.reserve_show_box_v1(&foreign, span).is_err());
        assert_eq!(root.live_bytes_v1(), 0);
        aggregate.reserve_show_box_v1(&source, span).unwrap();
        assert_eq!(root.live_bytes_v1(), exact);
        let permit = aggregate.into_permit_v1().expect("retained box debit");
        assert_eq!(root.live_bytes_v1(), exact);
        drop(permit);
        assert_eq!(root.live_bytes_v1(), 0);

        let short = FrontendLexicalMaterializationRootV1::issue_v1(id, &source, exact - 1, host)
            .expect("one-byte-short root");
        let mut denied = ReferenceCloneAggregateV1::new(&short, id);
        assert!(denied.reserve_show_box_v1(&source, span).is_err());
        assert_eq!(short.live_bytes_v1(), 0);
    }
}
