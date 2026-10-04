//! Source-bound surface storage admission. This ledger currently covers lexer
//! logical storage, top-level parser/HIR vectors, `parse_ref` string copies,
//! and the SHOW presentation box.
//! Other nested HIR, semantic, planning and execution storage need separate
//! producer admission.

use std::sync::atomic::{AtomicU64, Ordering};

use quanta_gqlang_v2_hir::{PresentationHirV1, SourceIdV2, SpanV1};

use crate::diag::{ParseDiagnosticV1, ParseErrorCodeV1};

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
