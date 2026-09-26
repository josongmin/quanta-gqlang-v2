#![expect(
    clippy::expect_used,
    reason = "the auxiliary-span rails assert the exact rejected error and its stable code"
)]

//! Exact source-name span is independent of the alias-inclusive relation span.

use quanta_gqlang_v2_hir::{
    check_tree, AliasV1, HirBuilderV1, HirIntegrityErrorV1, HirRootRefV1, SpanV1,
};

#[test]
fn source_name_auxiliary_span_out_of_bounds_rejects_even_if_relation_is_in_bounds() {
    let builder = HirBuilderV1::new();
    let relation = builder.relation_source(
        "users",
        SpanV1::new(2, 13),
        Some(AliasV1 {
            name: "u".into(),
            span: SpanV1::new(8, 9),
        }),
        SpanV1::new(2, 9),
    );
    let error = check_tree(HirRootRefV1::Relation(&relation), 10)
        .expect_err("out-of-bounds source-name auxiliary span must reject");
    assert_eq!(error.code(), "hir.integrity.auxiliary_span_out_of_bounds");
    assert_eq!(
        error.to_string(),
        "hir.integrity.auxiliary_span_out_of_bounds: auxiliary HIR span 2..13 \
         exceeds source length 10"
    );
    assert!(matches!(
        error,
        HirIntegrityErrorV1::AuxiliarySpanOutOfBounds { span, source_len: 10 }
            if span == SpanV1::new(2, 13)
    ));
}

#[test]
fn source_name_auxiliary_span_inverted_rejects() {
    let builder = HirBuilderV1::new();
    let relation = builder.relation_source("users", SpanV1::new(7, 2), None, SpanV1::new(2, 7));
    let error = check_tree(HirRootRefV1::Relation(&relation), 10)
        .expect_err("inverted source-name auxiliary span must reject");
    assert_eq!(error.code(), "hir.integrity.inverted_auxiliary_span");
    assert_eq!(
        error.to_string(),
        "hir.integrity.inverted_auxiliary_span: auxiliary HIR span 7..2 is inverted"
    );
    assert!(matches!(
        error,
        HirIntegrityErrorV1::InvertedAuxiliarySpan { span } if span == SpanV1::new(7, 2)
    ));
}
