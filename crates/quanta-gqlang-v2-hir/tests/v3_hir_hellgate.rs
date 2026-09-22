#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

use quanta_gqlang_v2_hir::{
    check_tree, BinaryExprV1, BinaryOpV1, CallArgsHirV1, HirBuilderV1, HirIntegrityErrorV1,
    HirNodeIdV1, HirRootRefV1, LiteralV1, PathSegmentV1, RecordFieldV1, SpanV1, ValueExprHirV1,
    ValueExprKindV1,
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

#[test]
fn reject_out_of_bounds_auxiliary_spans_across_value_payloads() {
    let builder = HirBuilderV1::new();
    let path = builder
        .try_path(
            builder.symbol("x", SpanV1::new(0, 1)),
            vec![PathSegmentV1 {
                name: "field".into(),
                span: SpanV1::new(9, 10),
            }],
            SpanV1::new(0, 4),
        )
        .unwrap();
    let call = builder.call(
        "f",
        SpanV1::new(9, 10),
        CallArgsHirV1::try_new(vec![]).unwrap(),
        SpanV1::new(0, 4),
    );
    let record = builder.record(
        vec![RecordFieldV1 {
            name: "field".into(),
            name_span: SpanV1::new(9, 10),
            value: builder.int("1", SpanV1::new(2, 3)),
        }],
        SpanV1::new(0, 4),
    );
    for root in [&path, &call, &record] {
        assert!(matches!(
            check_tree(HirRootRefV1::Value(root), 4),
            Err(HirIntegrityErrorV1::AuxiliarySpanOutOfBounds { .. })
        ));
    }
}

#[test]
fn reject_empty_path_before_hir_construction() {
    let builder = HirBuilderV1::new();
    let base = builder.symbol("x", SpanV1::new(0, 1));

    let error = builder
        .try_path(base, Vec::new(), SpanV1::new(0, 1))
        .unwrap_err();

    assert_eq!(error, "path access requires at least one segment");
}
