use super::*;

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn grouped_aggregate_project_uses_sealed_partition_input_not_public_key_frame() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let scan_id = scan.id;
    let group_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .unwrap();
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(42, 43)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(44, 49),
            }],
            SpanV1::new(42, 49),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(38, 41),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(38, 50),
    );
    let sum_id = sum.id;
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value: sum,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(55, 60),
                }),
            }],
            SpanV1::new(5, 60),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 60));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 60,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed grouped aggregate expected")
    };
    let group = relation.root().inputs().next().unwrap();
    assert_eq!(group.partition_input().len(), 1);
    assert_eq!(
        (
            group
                .partition_input()
                .first()
                .unwrap()
                .binding()
                .target_node(),
            group.partition_input().first().unwrap().binding().ordinal()
        ),
        (scan_id, 0)
    );
    assert_eq!(
        group
            .output()
            .columns()
            .iter()
            .map(|column| column.name())
            .collect::<Vec<_>>(),
        vec!["id"]
    );
    assert_eq!(
        relation
            .output()
            .columns()
            .iter()
            .map(|column| column.name())
            .collect::<Vec<_>>(),
        vec!["total"]
    );
    assert_eq!(
        relation.root().expressions().first().unwrap().node_id,
        sum_id
    );
}

#[test]
fn partition_candidate_cannot_cross_an_intervening_sort_stage() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let group_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .unwrap();
    let sorted = builder
        .order_by(
            group,
            vec![OrderKeyHirV1::nulls_last(
                builder.symbol("id", SpanV1::new(35, 37)),
                SortDirectionHirV1::Asc,
            )],
            SpanV1::new(5, 37),
        )
        .unwrap();
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(49, 50)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(51, 56),
            }],
            SpanV1::new(49, 56),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(45, 48),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(45, 57),
    );
    let project = builder
        .try_project(
            sorted,
            vec![ProjectionItemV1 {
                value: sum,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(61, 66),
                }),
            }],
            SpanV1::new(5, 66),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 66));
    let result = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 66,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    );
    assert!(
        matches!(result, Err(FrontendTransactionErrorV2::Document(ref error))
        if error.diagnostics().first().is_some_and(|diagnostic|
            diagnostic.code() == "sema.document_validation_failed"
            && diagnostic.primary().span == SpanV1::new(49, 50))),
        "{result:?}"
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn scalar_over_aggregate_cannot_smuggle_group_partition_input() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let group_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .unwrap();
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(44, 45)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(46, 48),
            }],
            SpanV1::new(44, 48),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(40, 43),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(40, 49),
    );
    let outer = builder.call(
        "is_null",
        SpanV1::new(32, 39),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(sum)]).unwrap(),
        SpanV1::new(32, 50),
    );
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value: outer,
                alias: Some(AliasV1 {
                    name: "bad".into(),
                    span: SpanV1::new(55, 58),
                }),
            }],
            SpanV1::new(5, 58),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 58));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 58,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("composed aggregate effect cannot authorize scalar outer call")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.illegal_aggregate_context"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(44, 45)
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn scalar_call_cannot_authorize_a_group_partition_candidate() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![key], SpanV1::new(5, 30))
        .unwrap();
    let arg = builder
        .try_path(
            builder.symbol("r", SpanV1::new(42, 43)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(44, 49),
            }],
            SpanV1::new(42, 49),
        )
        .unwrap();
    let value = builder.call(
        "is_null",
        SpanV1::new(34, 41),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
        SpanV1::new(34, 50),
    );
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value,
                alias: Some(AliasV1 {
                    name: "bad".into(),
                    span: SpanV1::new(50, 53),
                }),
            }],
            SpanV1::new(5, 53),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 53));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 53,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("scalar call alone must not authorize partition-input reference")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.illegal_aggregate_context"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(42, 43)
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn plain_group_partition_field_cannot_be_projected_without_aggregate() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![key], SpanV1::new(5, 30))
        .unwrap();
    let value = builder
        .try_path(
            builder.symbol("r", SpanV1::new(42, 43)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(44, 49),
            }],
            SpanV1::new(42, 49),
        )
        .unwrap();
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value,
                alias: Some(AliasV1 {
                    name: "bad".into(),
                    span: SpanV1::new(52, 55),
                }),
            }],
            SpanV1::new(5, 55),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 55));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 55,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("plain group partition field is unavailable")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(42, 43)
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn nested_aggregate_cannot_authorize_group_partition_candidate() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![key], SpanV1::new(5, 30))
        .unwrap();
    let arg = builder
        .try_path(
            builder.symbol("r", SpanV1::new(44, 45)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(46, 48),
            }],
            SpanV1::new(44, 48),
        )
        .unwrap();
    let inner = builder.call(
        "sum",
        SpanV1::new(40, 43),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
        SpanV1::new(40, 49),
    );
    let outer = builder.call(
        "sum",
        SpanV1::new(32, 35),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(inner)]).unwrap(),
        SpanV1::new(32, 50),
    );
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value: outer,
                alias: Some(AliasV1 {
                    name: "bad".into(),
                    span: SpanV1::new(55, 58),
                }),
            }],
            SpanV1::new(5, 58),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 58));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 58,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("nested aggregates cannot consume partition input")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().code(),
        "sema.illegal_aggregate_context"
    );
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(44, 45)
    );
}

#[test]
fn standalone_declaration_cannot_introduce_an_aggregate() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let argument = builder.int("1", SpanV1::new(14, 15));
    let aggregate = builder.call(
        "sum",
        SpanV1::new(10, 13),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(10, 16),
    );
    let decl = builder.decl("total", SpanV1::new(4, 9), aggregate, SpanV1::new(0, 16));
    let result = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 16,
            items: vec![ProgramItemHirV1::Decl(Box::new(decl))],
        })
        .unwrap(),
        &authority,
    );
    assert!(
        matches!(result, Err(FrontendTransactionErrorV2::Document(ref error))
        if error.diagnostics().first().is_some_and(|diagnostic|
            diagnostic.code() == "sema.illegal_aggregate_context"
            && diagnostic.primary().span == SpanV1::new(10, 16))),
        "{result:?}"
    );
}

#[test]
fn order_key_cannot_introduce_an_aggregate_inside_a_nested_expression() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let source = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let argument = builder.int("1", SpanV1::new(28, 29));
    let aggregate = builder.call(
        "sum",
        SpanV1::new(24, 27),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(24, 30),
    );
    let wrapped = builder.paren_value(aggregate, SpanV1::new(23, 31));
    let sorted = builder
        .order_by(
            source,
            vec![OrderKeyHirV1::nulls_last(wrapped, SortDirectionHirV1::Asc)],
            SpanV1::new(5, 31),
        )
        .unwrap();
    let query = builder.query(sorted, None, SpanV1::new(0, 31));
    let result = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 31,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    );
    assert!(
        matches!(result, Err(FrontendTransactionErrorV2::Document(ref error))
        if error.diagnostics().first().is_some_and(|diagnostic|
            diagnostic.code() == "sema.illegal_aggregate_context"
            && diagnostic.primary().span == SpanV1::new(24, 30))),
        "{result:?}"
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn identical_host_relation_scans_preserve_distinct_authored_occurrence_ids() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let first = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "u".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let second = builder.relation_source(
        "records",
        SpanV1::new(25, 32),
        Some(AliasV1 {
            name: "v".into(),
            span: SpanV1::new(33, 34),
        }),
        SpanV1::new(25, 34),
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 34,
            items: vec![
                ProgramItemHirV1::Query(Box::new(quanta_gqlang_v2_hir::QueryHirV1::new(
                    HirNodeIdV1::from_raw(20),
                    SpanV1::new(0, 14),
                    first,
                    None,
                ))),
                ProgramItemHirV1::Query(Box::new(quanta_gqlang_v2_hir::QueryHirV1::new(
                    HirNodeIdV1::from_raw(21),
                    SpanV1::new(20, 34),
                    second,
                    None,
                ))),
            ],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let relation = |node| {
        let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document
            .item(SemanticItemIdV2::new(
                source_id,
                HirNodeIdV1::from_raw(node),
            ))
            .unwrap()
        else {
            panic!("typed scan expected");
        };
        relation
    };
    let first = relation(20);
    let second = relation(21);
    assert_eq!(first.root().source(), second.root().source());
    assert_ne!(first.root().key(), second.root().key());
    assert_ne!(
        first.output().columns().first().unwrap().origin(),
        second.output().columns().first().unwrap().origin()
    );
}

#[test]
fn diagnostics_sort_stage_severity_code_and_secondary_edges() {
    use quanta_gqlang_v2_sema::{
        FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2,
    };
    let source = SourceIdV2::from_raw(9);
    let a = LocatedSpanV2 {
        source_id: source,
        span: SpanV1::new(3, 4),
    };
    let b = LocatedSpanV2 {
        source_id: source,
        span: SpanV1::new(8, 9),
    };
    let later = FrontendDiagnosticV2::new(
        FrontendStageV2::Typing,
        "z",
        FrontendSeverityV2::Error,
        b,
        vec![b, a, a],
        "Ω".repeat(600),
    );
    assert_eq!(later.secondary(), &[a, b]);
    assert!(later.message().len() <= 1_024);
    assert!(later.message().is_char_boundary(later.message().len()));
    let first = FrontendDiagnosticV2::new(
        FrontendStageV2::Binding,
        "a",
        FrontendSeverityV2::Error,
        a,
        Vec::new(),
        "first",
    );
    let failure = FrontendFailureV2::try_new(vec![later, first]).unwrap();
    assert_eq!(
        failure
            .diagnostics()
            .iter()
            .map(FrontendDiagnosticV2::code)
            .collect::<Vec<_>>(),
        vec!["a", "z"]
    );
    assert!(FrontendFailureV2::try_new(Vec::new()).is_err());
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic shape must fail this owner test"
)]
fn scalar_count_star_projects_through_exactly_one_implicit_empty_group() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let count = builder.call(
        "count",
        SpanV1::new(24, 29),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Star {
            span: SpanV1::new(30, 31),
        }])
        .unwrap(),
        SpanV1::new(24, 31),
    );
    let count_id = count.id;
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: count,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(36, 41),
                }),
            }],
            SpanV1::new(16, 41),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 41));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("scalar aggregate relation expected");
    };
    // Exact Group(empty) -> Project(aggregate) shape with Set cardinality,
    // empty partition keys, retained star, and no caller option.
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Project(_) = relation.root() else {
        panic!("aggregate projection root expected");
    };
    let group = relation.root().inputs().next().unwrap();
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_) = group else {
        panic!("implicit empty-partition Group expected");
    };
    assert!(group.expressions().is_empty());
    assert!(group.partition_input().is_empty());
    assert_eq!(group.output().cardinality(), CardinalityV1::Set);
    assert!(group.output().partition_keys().is_empty());
    assert!(group.output().columns().is_empty());
    assert!(group.output().row().fields().is_empty());
    assert_eq!(relation.output().cardinality(), CardinalityV1::Set);
    assert!(relation.output().partition_keys().is_empty());
    assert!(relation.output().ordering().is_empty());
    let count = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: count_id,
        })
        .unwrap();
    assert_eq!(
        count.shape(),
        &quanta_gqlang_v2_sema::TypedExprShapeV2::Call {
            target: count.resolved_call().cloned().unwrap(),
            arguments: vec![quanta_gqlang_v2_sema::TypedCallArgumentV2::Star],
        }
    );
    assert!(count.aggregate_context().uses_star());
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic shape must fail this owner test"
)]
fn grouped_sum_output_retains_the_authored_group_partition_identity() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let scan_id = scan.id;
    let group_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(22, 23)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(24, 26),
            }],
            SpanV1::new(22, 26),
        )
        .unwrap();
    let group_key_id = group_key.id;
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .unwrap();
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(42, 43)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(44, 49),
            }],
            SpanV1::new(42, 49),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(38, 41),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(38, 50),
    );
    let project = builder
        .try_project(
            group,
            vec![ProjectionItemV1 {
                value: sum,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(55, 60),
                }),
            }],
            SpanV1::new(34, 60),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 60));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("grouped aggregate relation expected");
    };
    // Exactly one authored Group; the aggregate Project consumes it directly
    // and no second Group is synthesized.
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Project(_) = relation.root() else {
        panic!("aggregate projection root expected");
    };
    let group = relation.root().inputs().next().unwrap();
    let quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_) = group else {
        panic!("the authored Group must be preserved");
    };
    let mut groups = 0usize;
    fn count_groups(stage: &quanta_gqlang_v2_sema::TypedRelationExprV2, groups: &mut usize) {
        if matches!(stage, quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_)) {
            *groups += 1;
        }
        for child in stage.inputs() {
            count_groups(child, groups);
        }
    }
    count_groups(relation.root(), &mut groups);
    assert_eq!(groups, 1, "exactly one authored Group may exist");
    // Exact Binder partition input is retained on the Group.
    assert_eq!(group.partition_input().len(), 1);
    let partition = group.partition_input().first().unwrap();
    assert_eq!(
        (
            partition.binding().target_node(),
            partition.binding().ordinal()
        ),
        (scan_id, 0)
    );
    // Aggregate output retains the exact authored partition identity.
    assert_eq!(
        relation.output().partition_keys(),
        &[quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: group_key_id,
        }]
    );
    assert_eq!(relation.output().cardinality(), CardinalityV1::Set);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic shape must fail this owner test"
)]
fn multiple_aggregates_and_key_plus_aggregate_keep_authored_order_and_aligned_origins() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let group_key = builder
        .try_path(
            builder.symbol("r", SpanV1::new(22, 23)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(24, 26),
            }],
            SpanV1::new(22, 26),
        )
        .unwrap();
    let group = builder
        .group_by(scan, vec![group_key], SpanV1::new(5, 30))
        .unwrap();
    // Authored order: sum(r.value), then the key, then count(r.value).
    let sum_argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(44, 45)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(46, 51),
            }],
            SpanV1::new(44, 51),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(40, 43),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(sum_argument)]).unwrap(),
        SpanV1::new(40, 52),
    );
    let sum_id = sum.id;
    let key_ref = builder.symbol("id", SpanV1::new(56, 58));
    let key_ref_id = key_ref.id;
    let count_argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(68, 69)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(70, 75),
            }],
            SpanV1::new(68, 75),
        )
        .unwrap();
    let count = builder.call(
        "count",
        SpanV1::new(62, 67),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(count_argument)]).unwrap(),
        SpanV1::new(62, 76),
    );
    let count_id = count.id;
    let project = builder
        .try_project(
            group,
            vec![
                ProjectionItemV1 {
                    value: sum,
                    alias: Some(AliasV1 {
                        name: "total".into(),
                        span: SpanV1::new(80, 85),
                    }),
                },
                ProjectionItemV1 {
                    value: key_ref,
                    alias: None,
                },
                ProjectionItemV1 {
                    value: count,
                    alias: Some(AliasV1 {
                        name: "n".into(),
                        span: SpanV1::new(96, 97),
                    }),
                },
            ],
            SpanV1::new(36, 97),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 97));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 128,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("grouped aggregate relation expected");
    };
    // Stable authored expression order on the aggregate Project.
    assert_eq!(
        relation
            .root()
            .expressions()
            .iter()
            .map(|key| key.node_id)
            .collect::<Vec<_>>(),
        vec![sum_id, key_ref_id, count_id]
    );
    // Exact row-aligned expression origins, one per output column, in order.
    for (expression, column) in relation
        .root()
        .expressions()
        .iter()
        .zip(relation.output().columns())
    {
        assert_eq!(
            column.origin(),
            &quanta_gqlang_v2_sema::OutputColumnOriginV2::Expression(*expression)
        );
    }
    assert_eq!(relation.output().columns().len(), 3);
    assert_eq!(relation.output().row().fields().len(), 3);
    assert_eq!(relation.output().cardinality(), CardinalityV1::Set);
    assert_eq!(relation.output().partition_keys().len(), 1);
}

#[test]
fn mixed_star_argument_is_unrepresentable_in_checked_hir() {
    // `count(*, x)`: the HIR owner rejects a star mixed with an expression
    // argument at construction, so the illegal shape can never reach the
    // semantic transaction, let alone algebra.
    let error = CallArgsHirV1::try_new(vec![
        CallArgHirV1::Star {
            span: SpanV1::new(30, 31),
        },
        CallArgHirV1::Expr(HirBuilderV1::new().int("1", SpanV1::new(33, 34))),
    ])
    .unwrap_err();
    assert_eq!(error, "'*' must be the sole call argument");
}

#[test]
fn illegal_star_matrix_fails_before_algebra_at_the_authored_call_span() {
    for (name, args, call_span) in [
        (
            "sum",
            vec![CallArgHirV1::Star {
                span: SpanV1::new(30, 31),
            }],
            SpanV1::new(24, 31),
        ),
        (
            "min",
            vec![CallArgHirV1::Star {
                span: SpanV1::new(30, 31),
            }],
            SpanV1::new(24, 31),
        ),
        (
            "max",
            vec![CallArgHirV1::Star {
                span: SpanV1::new(30, 31),
            }],
            SpanV1::new(24, 31),
        ),
    ] {
        let (_host, authority) = host();
        let builder = HirBuilderV1::new();
        let scan = builder.relation_source(
            "records",
            SpanV1::new(5, 12),
            Some(AliasV1 {
                name: "r".into(),
                span: SpanV1::new(13, 14),
            }),
            SpanV1::new(5, 14),
        );
        let call = builder.call(
            name,
            SpanV1::new(24, 29),
            CallArgsHirV1::try_new(args).unwrap(),
            call_span,
        );
        let project = builder
            .try_project(
                scan,
                vec![ProjectionItemV1 {
                    value: call,
                    alias: None,
                }],
                SpanV1::new(16, call_span.end),
            )
            .unwrap();
        let query = builder.query(project, None, SpanV1::new(0, call_span.end));
        let result = resolve_checked_document_v2(
            CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
                source_id: SourceIdV2::from_raw(9),
                source_len: 64,
                items: vec![ProgramItemHirV1::Query(Box::new(query))],
            })
            .unwrap(),
            &authority,
        );
        let FrontendTransactionErrorV2::Document(error) = result.unwrap_err() else {
            panic!("{name} star must fail inside the document transaction");
        };
        let diagnostic = error.diagnostics().first().unwrap();
        assert_eq!(diagnostic.code(), "types.invalid_star_argument", "{name}");
        assert_eq!(
            diagnostic.primary(),
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(9),
                span: call_span,
            },
            "{name} star must fail at the exact authored call span"
        );
    }
}

#[test]
fn global_aggregate_projection_cannot_retain_a_plain_column() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let plain = builder
        .try_path(
            builder.symbol("r", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let count = builder.call(
        "count",
        SpanV1::new(27, 32),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Star {
            span: SpanV1::new(33, 34),
        }])
        .unwrap(),
        SpanV1::new(27, 34),
    );
    let project = builder
        .try_project(
            scan,
            vec![
                ProjectionItemV1 {
                    value: plain,
                    alias: None,
                },
                ProjectionItemV1 {
                    value: count,
                    alias: None,
                },
            ],
            SpanV1::new(16, 34),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 34));
    let result = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    );
    assert!(
        matches!(result, Err(FrontendTransactionErrorV2::Document(ref error))
        if error.diagnostics().first().is_some_and(|diagnostic|
            diagnostic.code() == "sema.mixed_aggregate_projection"
                && diagnostic.primary().span == SpanV1::new(20, 24))),
        "{result:?}"
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn global_nested_aggregate_is_rejected_at_the_inner_call() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(32, 33)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(34, 39),
            }],
            SpanV1::new(32, 39),
        )
        .unwrap();
    let inner_span = SpanV1::new(28, 40);
    let inner = builder.call(
        "sum",
        SpanV1::new(28, 31),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        inner_span,
    );
    let outer = builder.call(
        "sum",
        SpanV1::new(24, 27),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(inner)]).unwrap(),
        SpanV1::new(24, 41),
    );
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: outer,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(45, 50),
                }),
            }],
            SpanV1::new(16, 50),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 50));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("nested global aggregate must fail in sema")
    };
    let diagnostic = error.diagnostics().first().unwrap();
    assert_eq!(diagnostic.code(), "sema.illegal_aggregate_context");
    assert_eq!(diagnostic.primary().span, inner_span);
}

#[test]
fn global_single_sum_remains_admitted() {
    let (_host, authority) = host();
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let argument = builder
        .try_path(
            builder.symbol("r", SpanV1::new(28, 29)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(30, 35),
            }],
            SpanV1::new(28, 35),
        )
        .unwrap();
    let sum = builder.call(
        "sum",
        SpanV1::new(24, 27),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
        SpanV1::new(24, 36),
    );
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: sum,
                alias: Some(AliasV1 {
                    name: "total".into(),
                    span: SpanV1::new(40, 45),
                }),
            }],
            SpanV1::new(16, 45),
        )
        .unwrap();
    let query = builder.query(project, None, SpanV1::new(0, 45));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 64,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("global SUM must remain a relation")
    };
    assert!(matches!(
        relation.root(),
        quanta_gqlang_v2_sema::TypedRelationExprV2::Project(_)
    ));
    assert!(matches!(
        relation.root().inputs().next(),
        Some(quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_))
    ));
}

#[test]
fn same_primary_diagnostics_pin_stage_severity_code_and_message_ties() {
    use quanta_gqlang_v2_sema::{
        FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2, FrontendStageV2,
    };
    let primary = LocatedSpanV2 {
        source_id: SourceIdV2::from_raw(9),
        span: SpanV1::new(3, 4),
    };
    let cases = [
        (
            FrontendStageV2::Binding,
            FrontendSeverityV2::Note,
            "z",
            "binding-note",
        ),
        (
            FrontendStageV2::Typing,
            FrontendSeverityV2::Error,
            "a",
            "typing-error-a-a",
        ),
        (
            FrontendStageV2::Typing,
            FrontendSeverityV2::Error,
            "a",
            "typing-error-a-b",
        ),
        (
            FrontendStageV2::Typing,
            FrontendSeverityV2::Error,
            "z",
            "typing-error-z",
        ),
        (
            FrontendStageV2::Typing,
            FrontendSeverityV2::Warning,
            "a",
            "typing-warning",
        ),
        (
            FrontendStageV2::Typing,
            FrontendSeverityV2::Note,
            "a",
            "typing-note",
        ),
    ];
    for order in [[5, 4, 3, 2, 1, 0], [2, 5, 0, 4, 1, 3]] {
        let diagnostics = order
            .into_iter()
            .map(|index| {
                let (stage, severity, code, message) = *cases
                    .get(index)
                    .expect("registered diagnostic permutation index");
                FrontendDiagnosticV2::new(stage, code, severity, primary, Vec::new(), message)
            })
            .collect();
        let failure = FrontendFailureV2::try_new(diagnostics).unwrap();
        assert_eq!(
            failure
                .diagnostics()
                .iter()
                .map(FrontendDiagnosticV2::message)
                .collect::<Vec<_>>(),
            [
                "binding-note",
                "typing-error-a-a",
                "typing-error-a-b",
                "typing-error-z",
                "typing-warning",
                "typing-note"
            ]
        );
    }
}
