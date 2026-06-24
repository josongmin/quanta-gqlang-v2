//! v3 HIR fail-closed integrity rails (ticket 01 scenario 5).
//!
//! Each test forces one integrity violation by assembling a deliberately
//! unsound tree BY HAND (bypassing the id-allocating builder) and asserts the
//! EXACT typed diagnostic code + Display text from `check_tree`, never just
//! "an error occurred".

#![allow(clippy::unwrap_used, reason = "tests assert on Result directly")]
#![allow(clippy::panic, reason = "tests panic on unexpected Ok")]

use quanta_gqlang_v2_hir::{
    check_tree, BinaryExprV1, BinaryOpV1, HirIntegrityErrorV1, HirNodeIdV1, HirRootRefV1,
    LiteralV1, SpanV1, ValueExprHirV1, ValueExprKindV1,
};

fn lit_int(id: u32, span: SpanV1) -> ValueExprHirV1 {
    ValueExprHirV1::new(
        HirNodeIdV1::from_raw(id),
        span,
        ValueExprKindV1::Literal(LiteralV1::Int("1".into())),
    )
}

fn binary(id: u32, span: SpanV1, lhs: ValueExprHirV1, rhs: ValueExprHirV1) -> ValueExprHirV1 {
    ValueExprHirV1::new(
        HirNodeIdV1::from_raw(id),
        span,
        ValueExprKindV1::Binary(BinaryExprV1 {
            op: BinaryOpV1 { symbol: "+".into() },
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        }),
    )
}

/// Duplicate node id across two distinct nodes must fail closed.
#[test]
fn reject_duplicate_node_id() {
    // lhs and rhs deliberately share raw id 7.
    let lhs = lit_int(7, SpanV1::new(0, 1));
    let rhs = lit_int(7, SpanV1::new(4, 5));
    let root = binary(1, SpanV1::new(0, 5), lhs, rhs);

    let err = check_tree(HirRootRefV1::Value(&root), 5).unwrap_err();
    assert_eq!(err.code(), "hir.integrity.duplicate_node_id");
    let HirIntegrityErrorV1::DuplicateNodeId {
        id,
        first_span,
        second_span,
    } = &err
    else {
        panic!("expected DuplicateNodeId, got {err:?}");
    };
    assert_eq!(id.raw(), 7);
    assert_eq!(*first_span, SpanV1::new(0, 1));
    assert_eq!(*second_span, SpanV1::new(4, 5));
    assert_eq!(
        err.to_string(),
        "hir.integrity.duplicate_node_id: duplicate HIR node id 7 (first at 0..1, again at 4..5)"
    );
}

/// Inverted span (start > end) must fail closed.
#[test]
fn reject_inverted_span() {
    // raw id 3 carries an inverted span 9..2.
    let bad = lit_int(3, SpanV1::new(9, 2));
    let ok = lit_int(4, SpanV1::new(0, 1));
    let root = binary(1, SpanV1::new(0, 10), bad, ok);

    let err = check_tree(HirRootRefV1::Value(&root), 10).unwrap_err();
    assert_eq!(err.code(), "hir.integrity.inverted_span");
    let HirIntegrityErrorV1::InvertedSpan { id, span } = &err else {
        panic!("expected InvertedSpan, got {err:?}");
    };
    assert_eq!(id.raw(), 3);
    assert_eq!(*span, SpanV1::new(9, 2));
    assert_eq!(
        err.to_string(),
        "hir.integrity.inverted_span: HIR node id 3 has inverted span 9..2 (start > end)"
    );
}

/// Span ending past the declared source length must fail closed.
#[test]
fn reject_span_out_of_bounds() {
    // source length is 5, but node id 2 spans up to 12.
    let bad = lit_int(2, SpanV1::new(3, 12));
    let ok = lit_int(4, SpanV1::new(0, 1));
    let root = binary(1, SpanV1::new(0, 4), ok, bad);

    let err = check_tree(HirRootRefV1::Value(&root), 5).unwrap_err();
    assert_eq!(err.code(), "hir.integrity.span_out_of_bounds");
    let HirIntegrityErrorV1::SpanOutOfBounds {
        id,
        span,
        source_len,
    } = &err
    else {
        panic!("expected SpanOutOfBounds, got {err:?}");
    };
    assert_eq!(id.raw(), 2);
    assert_eq!(*span, SpanV1::new(3, 12));
    assert_eq!(*source_len, 5);
    assert_eq!(
        err.to_string(),
        "hir.integrity.span_out_of_bounds: HIR node id 2 span 3..12 exceeds source length 5"
    );
}

/// A sound hand-built tree passes (positive control for the gate).
#[test]
fn accept_sound_hand_built_tree() {
    let lhs = lit_int(10, SpanV1::new(0, 1));
    let rhs = lit_int(11, SpanV1::new(4, 5));
    let root = binary(12, SpanV1::new(0, 5), lhs, rhs);
    let checked = check_tree(HirRootRefV1::Value(&root), 5).unwrap();
    assert_eq!(checked.node_count(), 3);
    assert_eq!(checked.source_len(), 5);
}
