#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! FIND endpoint-ROLE conflict rails (ticket 06, lane W3-B).
//!
//! A FIND that carries an endpoint role its subkind does not accept must fail
//! closed (`D10`), and the rejection must be fully pinned: the exact diagnostic
//! code, the exact offending ROLE, the exact subkind, and the originating FIND
//! item's checked location. A rail that only asserted "some error" would let a
//! mutant that reports the wrong role, the wrong subkind, or a generic code
//! survive.
//!
//! Endpoint roles come from the RESOLVED selectors of the sealed query, not from
//! raw HIR endpoint slots.

use quanta_gqlang_v2_logic::{GraphEndpointRoleV1, GraphQueryContractV2, LogicDiagnosticV1};
use quanta_gqlang_v2_sema::{CheckedLocatedSpanV2, GraphQueryKindV1};

mod support;

use support::{bounded_query, graph_document, reference, GraphReferenceRoleV2};

/// The exact byte range of one checked location, without naming a HIR type.
fn span_of(location: &CheckedLocatedSpanV2) -> (usize, usize) {
    let span = location.location().span;
    (span.start, span.end)
}

// `slice` anchors only. Supplying a `source`/`sink` as well must reject with the
// FIRST offending role in the deterministic scan order (`source` before `sink`)
// at the slice item's checked location.
#[test]
fn slice_with_extra_endpoints_should_reject() {
    let request = bounded_query(
        GraphQueryKindV1::Slice,
        vec![
            reference(GraphReferenceRoleV2::Anchor, "p.foo", 20, 25),
            reference(GraphReferenceRoleV2::Source, "a", 5, 6),
            reference(GraphReferenceRoleV2::Sink, "b", 10, 11),
        ],
        5,
        0,
        40,
    );
    let document = graph_document(request).expect("slice fixture resolves");
    let location = document.graph().location().clone();

    let err = GraphQueryContractV2::from_resolved(document.graph())
        .expect_err("slice with source/sink must be rejected");
    assert_eq!(err.code(), "logic.graph_endpoint_unexpected");
    assert_eq!(
        err.message(),
        "FIND `slice` does not accept a `source` endpoint"
    );
    assert_eq!(err.span(), &location);
    assert_eq!(span_of(err.span()), (0, 40));
    match err {
        LogicDiagnosticV1::GraphEndpointUnexpected { kind, role, span } => {
            assert_eq!(kind, GraphQueryKindV1::Slice);
            assert_eq!(role, GraphEndpointRoleV1::Source);
            assert_eq!(&span, &location);
            assert_eq!(span_of(&span), (0, 40));
        }
        other => panic!("expected GraphEndpointUnexpected, got {other:?}"),
    }
}

// `points_to` relates a reference to a pointee; a `callable` endpoint is not
// part of that contract and must reject with the exact role.
#[test]
fn points_to_with_a_callable_endpoint_rejects() {
    let request = bounded_query(
        GraphQueryKindV1::PointsTo,
        vec![
            reference(GraphReferenceRoleV2::Reference, "p", 5, 6),
            reference(GraphReferenceRoleV2::Pointee, "q", 10, 11),
            reference(GraphReferenceRoleV2::Callable, "f", 14, 15),
        ],
        3,
        0,
        20,
    );
    let document = graph_document(request).expect("points_to fixture resolves");

    let err = GraphQueryContractV2::from_resolved(document.graph())
        .expect_err("callable endpoint on points_to must be rejected");
    assert_eq!(err.code(), "logic.graph_endpoint_unexpected");
    assert_eq!(
        err.message(),
        "FIND `points_to` does not accept a `callable` endpoint"
    );
    assert_eq!(err.span(), document.graph().location());
    match err {
        LogicDiagnosticV1::GraphEndpointUnexpected { kind, role, .. } => {
            assert_eq!(kind, GraphQueryKindV1::PointsTo);
            assert_eq!(role, GraphEndpointRoleV1::Callable);
        }
        other => panic!("expected GraphEndpointUnexpected, got {other:?}"),
    }
}
