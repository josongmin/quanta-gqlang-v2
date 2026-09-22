#![expect(
    clippy::expect_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]
#![expect(
    dead_code,
    reason = "shared test helpers are used by a subset of the integration-test crates that include this module"
)]

//! Shared logic fixture builders (ticket 06, lane W3-B).
//!
//! Every helper builds or compares SEALED Sema values through the `sema` `test`
//! feature; no helper names a transitional `quanta_gqlang_v2_hir` type, because
//! this crate's HIR edge dies at the W3 barrier. Byte spans are expressed as
//! `(start, end)` pairs and asserted through
//! [`quanta_gqlang_v2_sema::CheckedLocatedSpanV2`], never through a raw span
//! constructor.
//!
//! If the W3-D fixture names shift, only this module changes: the test crates
//! call the wrappers below.

use quanta_gqlang_v2_sema::test_support::{
    reference_checked_span_v2, reference_graph_document_v2, reference_nested_check_document_v2,
};
pub(crate) use quanta_gqlang_v2_sema::test_support::{
    ReferenceGraphDocumentV2, ReferenceGraphReferenceV2, ReferenceGraphRequestV2,
    ReferenceNestedCheckDocumentV2,
};
pub(crate) use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, GraphQueryKindV1, GraphReferenceRoleV2,
};

/// Build one sealed resolved FIND document.
///
/// # Errors
///
/// Returns a fixture-construction error when the requested shape no longer
/// satisfies the semantic contract.
pub(crate) fn graph_document(
    request: ReferenceGraphRequestV2,
) -> Result<ReferenceGraphDocumentV2, String> {
    reference_graph_document_v2(request)
}

/// Build one sealed nested CHECK/FIND document.
///
/// # Errors
///
/// Returns a fixture-construction error when the requested shape no longer
/// satisfies the semantic contract.
pub(crate) fn nested_check_document(
    request: ReferenceGraphRequestV2,
) -> Result<ReferenceNestedCheckDocumentV2, String> {
    reference_nested_check_document_v2(request)
}

/// One requested resolved FIND reference at its exact authored byte span.
#[must_use]
pub(crate) fn reference(
    role: GraphReferenceRoleV2,
    selector: &'static str,
    start: usize,
    end: usize,
) -> ReferenceGraphReferenceV2 {
    ReferenceGraphReferenceV2 {
        role,
        selector,
        span: (start, end),
    }
}

/// One requested resolved FIND document with an explicit depth.
#[must_use]
pub(crate) fn bounded_query(
    kind: GraphQueryKindV1,
    references: Vec<ReferenceGraphReferenceV2>,
    depth: u32,
    start: usize,
    end: usize,
) -> ReferenceGraphRequestV2 {
    ReferenceGraphRequestV2 {
        kind,
        references,
        depth: Some(depth),
        direction: None,
        summary_facets: Vec::new(),
        item_span: (start, end),
    }
}

/// One requested resolved FIND document with NO depth (an unbounded family).
#[must_use]
pub(crate) fn unbounded_query(
    kind: GraphQueryKindV1,
    references: Vec<ReferenceGraphReferenceV2>,
    start: usize,
    end: usize,
) -> ReferenceGraphRequestV2 {
    ReferenceGraphRequestV2 {
        kind,
        references,
        depth: None,
        direction: None,
        summary_facets: Vec::new(),
        item_span: (start, end),
    }
}

/// Mint one registry-proven checked location for owner-unit diagnostics.
#[must_use]
pub(crate) fn checked_span(start: usize, end: usize) -> CheckedLocatedSpanV2 {
    reference_checked_span_v2(start, end).expect("fixture checked location is in bounds")
}
