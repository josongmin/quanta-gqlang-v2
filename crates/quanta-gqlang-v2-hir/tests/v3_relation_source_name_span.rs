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
    assert!(matches!(
        check_tree(HirRootRefV1::Relation(&relation), 10),
        Err(HirIntegrityErrorV1::AuxiliarySpanOutOfBounds { span, source_len: 10 })
            if span == SpanV1::new(2, 13)
    ));
}

#[test]
fn source_name_auxiliary_span_inverted_rejects() {
    let builder = HirBuilderV1::new();
    let relation = builder.relation_source("users", SpanV1::new(7, 2), None, SpanV1::new(2, 7));
    assert!(matches!(
        check_tree(HirRootRefV1::Relation(&relation), 10),
        Err(HirIntegrityErrorV1::InvertedAuxiliarySpan { span }) if span == SpanV1::new(7, 2)
    ));
}
