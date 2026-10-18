use super::*;

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn query_scan_uses_binder_frame_and_types_owned_host_relation_contract() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let stage_node = scan.id;
    let query = quanta_gqlang_v2_hir::QueryHirV1::new(
        HirNodeIdV1::from_raw(20),
        SpanV1::new(0, 12),
        scan,
        None,
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 12,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let id = SemanticItemIdV2::new(source_id, HirNodeIdV1::from_raw(20));
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) = document.item(id).unwrap()
    else {
        panic!("typed relation expected")
    };
    assert_eq!(relation.root().key().node_id, stage_node);
    assert_eq!(
        relation.root().location().location(),
        LocatedSpanV2 {
            source_id,
            span: SpanV1::new(5, 12)
        }
    );
    assert_eq!(
        relation.location().location(),
        LocatedSpanV2 {
            source_id,
            span: SpanV1::new(0, 12)
        }
    );
    assert_eq!(relation.output().columns().len(), 1);
    assert_eq!(relation.output().columns().first().unwrap().name(), "id");
    assert_eq!(
        relation
            .output()
            .columns()
            .first()
            .unwrap()
            .location()
            .location(),
        LocatedSpanV2 {
            source_id: SourceIdV2::from_raw(1),
            span: SpanV1::new(4, 5)
        }
    );
    let quanta_gqlang_v2_sema::OutputColumnOriginV2::SourceField {
        source,
        scan_stage,
        field_name,
    } = relation.output().columns().first().unwrap().origin()
    else {
        panic!("scan column must retain authoritative source field")
    };
    assert_eq!(source, relation.root().source().unwrap());
    assert_eq!(*scan_stage, relation.root().key());
    assert_eq!(field_name, "id");
    assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
    assert_eq!(relation.output().provenance().iter().count(), 1);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn query_filter_retains_sealed_predicate_and_inherited_scan_columns() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let scan_id = scan.id;
    let predicate = builder.compare(
        "=",
        builder.int("1", SpanV1::new(15, 16)),
        builder.int("1", SpanV1::new(19, 20)),
        SpanV1::new(15, 20),
    );
    let predicate_id = predicate.id;
    let filter = builder.relation_filter(scan, predicate, SpanV1::new(5, 20));
    let query = builder.query(filter, None, SpanV1::new(0, 20));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 20,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed filter relation expected");
    };
    assert!(matches!(
        relation.root(),
        quanta_gqlang_v2_sema::TypedRelationExprV2::Filter(_)
    ));
    assert_eq!(
        relation.root().inputs().next().unwrap().key().node_id,
        scan_id
    );
    assert_eq!(
        relation
            .root()
            .predicate()
            .unwrap()
            .expression()
            .key()
            .node_id,
        predicate_id
    );
    assert_eq!(
        relation.root().location().location().span,
        SpanV1::new(5, 20)
    );
    assert_eq!(
        relation.output().columns().first().unwrap().origin(),
        relation
            .root()
            .inputs()
            .next()
            .unwrap()
            .output()
            .columns()
            .first()
            .unwrap()
            .origin()
    );
    assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
    assert_eq!(document.expressions().count(), 3);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong nested relation or capture identity must fail this owner test"
)]
fn query_relation_exists_seals_nested_relation_and_exact_binder_capture() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let outer_scan = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let outer_scan_id = outer_scan.id;
    let inner_scan = builder.relation_source(
        "records2",
        SpanV1::new(25, 33),
        Some(AliasV1 {
            name: "f".into(),
            span: SpanV1::new(34, 35),
        }),
        SpanV1::new(25, 35),
    );
    let inner_scan_id = inner_scan.id;
    let inner_f = builder.symbol("f", SpanV1::new(38, 39));
    let inner_f_path = builder
        .try_path(
            inner_f,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(40, 42),
            }],
            SpanV1::new(38, 42),
        )
        .unwrap();
    let captured_r = builder.symbol("r", SpanV1::new(45, 46));
    let captured_r_id = captured_r.id;
    let captured_r_path = builder
        .try_path(
            captured_r,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(47, 49),
            }],
            SpanV1::new(45, 49),
        )
        .unwrap();
    let inner_compare = builder.compare("=", inner_f_path, captured_r_path, SpanV1::new(38, 49));
    let inner_compare_id = inner_compare.id;
    let inner_filter = builder.relation_filter(inner_scan, inner_compare, SpanV1::new(25, 49));
    let inner_filter_id = inner_filter.id;
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        inner_filter,
        SpanV1::new(20, 50),
    );
    let exists_id = exists.id;
    let outer_filter = builder.relation_filter(outer_scan, exists, SpanV1::new(5, 50));
    let query = builder.query(outer_filter, None, SpanV1::new(0, 50));
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
    let exists_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: exists_id,
    };
    let inner_filter_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: inner_filter_id,
    };
    let exists_expression = document.expression(exists_key).unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::RelationExists {
        polarity,
        relation_root,
        correlations,
    } = exists_expression.shape()
    else {
        panic!("sealed relational EXISTS shape expected")
    };
    assert_eq!(
        (*polarity, *relation_root),
        (
            quanta_gqlang_v2_sema::RelationExistsPolarityV2::Exists,
            inner_filter_key
        )
    );
    let [correlation] = correlations.as_slice() else {
        panic!("one exact outer-alias capture expected")
    };
    assert_eq!(
        (
            correlation.binding().target_node(),
            correlation.binding().ordinal(),
            correlation.binding().kind(),
            correlation.binding().canonical_name(),
            correlation.reference().node_id,
            correlation.location().location().span,
        ),
        (
            outer_scan_id,
            0,
            quanta_gqlang_v2_sema::ResolvedBindingKindV2::Alias,
            "r",
            captured_r_id,
            SpanV1::new(45, 46),
        )
    );
    let nested = document.nested_relation(inner_filter_key).unwrap();
    assert!(matches!(
        nested,
        quanta_gqlang_v2_sema::TypedRelationExprV2::Filter(_)
    ));
    assert_eq!(nested.key(), inner_filter_key);
    assert_eq!(nested.inputs().next().unwrap().key().node_id, inner_scan_id);
    assert_eq!(
        nested.predicate().unwrap().expression().key().node_id,
        inner_compare_id
    );
    assert_eq!(
        document
            .nested_relations()
            .map(|(root, _)| *root)
            .collect::<Vec<_>>(),
        vec![inner_filter_key]
    );
    assert_eq!(document.items().count(), 1);
}

#[test]
#[expect(
    clippy::panic,
    reason = "multi-capture EXISTS must retain each exact reference rather than one binding summary"
)]
fn query_relation_exists_retains_all_outer_reference_captures() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let outer = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let outer_id = outer.id;
    let inner = builder.relation_source(
        "records2",
        SpanV1::new(25, 33),
        Some(AliasV1 {
            name: "f".into(),
            span: SpanV1::new(34, 35),
        }),
        SpanV1::new(25, 35),
    );
    let inner_id = inner.id;
    let path = |name: &str, name_span: SpanV1, field: &str, field_span: SpanV1| {
        builder
            .try_path(
                builder.symbol(name, name_span),
                vec![PathSegmentV1 {
                    name: field.into(),
                    span: field_span,
                }],
                SpanV1::new(name_span.start, field_span.end),
            )
            .unwrap()
    };
    let inner_id_path = path("f", SpanV1::new(38, 39), "id", SpanV1::new(40, 42));
    let captured_id = builder.symbol("r", SpanV1::new(45, 46));
    let captured_id_ref = captured_id.id;
    let captured_id_path = builder
        .try_path(
            captured_id,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(47, 49),
            }],
            SpanV1::new(45, 49),
        )
        .unwrap();
    let inner_value_path = path("f", SpanV1::new(54, 55), "value", SpanV1::new(56, 61));
    let captured_value = builder.symbol("r", SpanV1::new(64, 65));
    let captured_value_ref = captured_value.id;
    let captured_value_path = builder
        .try_path(
            captured_value,
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(66, 71),
            }],
            SpanV1::new(64, 71),
        )
        .unwrap();
    let predicate = builder.and(
        builder.compare("=", inner_id_path, captured_id_path, SpanV1::new(38, 49)),
        builder.compare(
            "=",
            inner_value_path,
            captured_value_path,
            SpanV1::new(54, 71),
        ),
        SpanV1::new(38, 71),
    );
    let inner_filter = builder.relation_filter(inner, predicate, SpanV1::new(25, 71));
    let inner_root = inner_filter.id;
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        inner_filter,
        SpanV1::new(20, 72),
    );
    let exists_id = exists.id;
    let query = builder.query(
        builder.relation_filter(outer, exists, SpanV1::new(5, 72)),
        None,
        SpanV1::new(0, 72),
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 80,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let exists_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: exists_id,
    };
    let quanta_gqlang_v2_sema::TypedExprShapeV2::RelationExists {
        relation_root,
        correlations,
        ..
    } = document.expression(exists_key).unwrap().shape()
    else {
        panic!("multi-capture EXISTS shape expected")
    };
    assert_eq!(relation_root.node_id, inner_root);
    assert_eq!(
        correlations
            .iter()
            .map(|edge| (
                edge.binding().target_node(),
                edge.binding().ordinal(),
                edge.reference().node_id,
                edge.location().location().span,
            ))
            .collect::<Vec<_>>(),
        vec![
            (outer_id, 0, captured_id_ref, SpanV1::new(45, 46)),
            (outer_id, 0, captured_value_ref, SpanV1::new(64, 65)),
        ],
    );
    let root = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: inner_root,
    };
    let nested = document.nested_relation(root).unwrap();
    assert_eq!(nested.key(), root);
    assert_eq!(nested.inputs().next().unwrap().key().node_id, inner_id);
}

#[test]
#[expect(
    clippy::panic,
    reason = "nested EXISTS must retain captures for every crossed correlation boundary"
)]
fn nested_relation_exists_retains_outer_and_inner_capture_sets() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let outer = builder.relation_source(
        "records2",
        SpanV1::new(5, 13),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(14, 15),
        }),
        SpanV1::new(5, 15),
    );
    let outer_id = outer.id;
    let middle = builder.relation_source(
        "records2",
        SpanV1::new(25, 33),
        Some(AliasV1 {
            name: "f".into(),
            span: SpanV1::new(34, 35),
        }),
        SpanV1::new(25, 35),
    );
    let deepest = builder.relation_source(
        "records2",
        SpanV1::new(65, 73),
        Some(AliasV1 {
            name: "g".into(),
            span: SpanV1::new(74, 75),
        }),
        SpanV1::new(65, 75),
    );
    let path = |name: &str, name_span: SpanV1, field: &str, field_span: SpanV1| {
        builder
            .try_path(
                builder.symbol(name, name_span),
                vec![PathSegmentV1 {
                    name: field.into(),
                    span: field_span,
                }],
                SpanV1::new(name_span.start, field_span.end),
            )
            .unwrap()
    };
    let direct_outer = builder.symbol("r", SpanV1::new(45, 46));
    let direct_outer_ref = direct_outer.id;
    let direct = builder.compare(
        "=",
        path("f", SpanV1::new(38, 39), "id", SpanV1::new(40, 42)),
        builder
            .try_path(
                direct_outer,
                vec![PathSegmentV1 {
                    name: "id".into(),
                    span: SpanV1::new(47, 49),
                }],
                SpanV1::new(45, 49),
            )
            .unwrap(),
        SpanV1::new(38, 49),
    );
    let deep_outer = builder.symbol("r", SpanV1::new(84, 85));
    let deep_outer_ref = deep_outer.id;
    let deep = builder.compare(
        "=",
        path("g", SpanV1::new(78, 79), "value", SpanV1::new(80, 85)),
        builder
            .try_path(
                deep_outer,
                vec![PathSegmentV1 {
                    name: "value".into(),
                    span: SpanV1::new(86, 91),
                }],
                SpanV1::new(84, 91),
            )
            .unwrap(),
        SpanV1::new(78, 91),
    );
    let deepest_filter = builder.relation_filter(deepest, deep, SpanV1::new(65, 91));
    let deepest_root = deepest_filter.id;
    let nested_exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        deepest_filter,
        SpanV1::new(60, 92),
    );
    let nested_exists_id = nested_exists.id;
    let middle_filter = builder.relation_filter(
        middle,
        builder.and(direct, nested_exists, SpanV1::new(38, 92)),
        SpanV1::new(25, 92),
    );
    let middle_root = middle_filter.id;
    let outer_exists = builder.relation_exists(
        RelationExistsPolarityV1::Exists,
        middle_filter,
        SpanV1::new(20, 93),
    );
    let outer_exists_id = outer_exists.id;
    let query = builder.query(
        builder.relation_filter(outer, outer_exists, SpanV1::new(5, 93)),
        None,
        SpanV1::new(0, 93),
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 104,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let shape = |node_id| {
        let key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 { source_id, node_id };
        match document.expression(key).unwrap().shape() {
            quanta_gqlang_v2_sema::TypedExprShapeV2::RelationExists {
                relation_root,
                correlations,
                ..
            } => (*relation_root, correlations),
            quanta_gqlang_v2_sema::TypedExprShapeV2::Literal(_)
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(_)
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Path { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Unary { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Binary { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Call { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Record { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::List { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::BoolValue { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Compare { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::And { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Or { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Not { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::Paren { .. }
            | quanta_gqlang_v2_sema::TypedExprShapeV2::LogicalQuantifier { .. } => {
                panic!("relational EXISTS shape expected")
            }
        }
    };
    let (outer_root, outer_captures) = shape(outer_exists_id);
    let (nested_root, nested_captures) = shape(nested_exists_id);
    assert_eq!(outer_root.node_id, middle_root);
    assert_eq!(nested_root.node_id, deepest_root);
    let expected_outer_references = if direct_outer_ref < deep_outer_ref {
        vec![direct_outer_ref, deep_outer_ref]
    } else {
        vec![deep_outer_ref, direct_outer_ref]
    };
    assert_eq!(
        outer_captures
            .iter()
            .map(|edge| edge.reference().node_id)
            .collect::<Vec<_>>(),
        expected_outer_references,
    );
    assert_eq!(
        nested_captures
            .iter()
            .map(|edge| edge.reference().node_id)
            .collect::<Vec<_>>(),
        vec![deep_outer_ref],
    );
    for capture in outer_captures.iter().chain(nested_captures.iter()) {
        assert_eq!(capture.binding().target_node(), outer_id);
        assert_eq!(capture.binding().ordinal(), 0);
        assert_eq!(capture.binding().canonical_name(), "r");
    }
    let roots = document
        .nested_relations()
        .map(|(root, _)| root.node_id)
        .collect::<Vec<_>>();
    let expected_roots = if middle_root < deepest_root {
        vec![middle_root, deepest_root]
    } else {
        vec![deepest_root, middle_root]
    };
    assert_eq!(roots, expected_roots);
}

#[test]
#[expect(
    clippy::panic,
    reason = "uncorrelated EXISTS must retain an explicit empty Binder capture set"
)]
fn query_not_exists_seals_an_uncorrelated_nested_relation() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let outer = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let inner = builder.relation_source("records", SpanV1::new(25, 32), None, SpanV1::new(25, 32));
    let inner_id = inner.id;
    let exists = builder.relation_exists(
        RelationExistsPolarityV1::NotExists,
        inner,
        SpanV1::new(20, 33),
    );
    let exists_id = exists.id;
    let query = builder.query(
        builder.relation_filter(outer, exists, SpanV1::new(5, 33)),
        None,
        SpanV1::new(0, 33),
    );
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 48,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let exists_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: exists_id,
    };
    let inner_key = quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
        source_id,
        node_id: inner_id,
    };
    let quanta_gqlang_v2_sema::TypedExprShapeV2::RelationExists {
        polarity,
        relation_root,
        correlations,
    } = document.expression(exists_key).unwrap().shape()
    else {
        panic!("sealed relational NOT EXISTS shape expected")
    };
    assert_eq!(
        (*polarity, *relation_root),
        (
            quanta_gqlang_v2_sema::RelationExistsPolarityV2::NotExists,
            inner_key
        )
    );
    assert!(correlations.is_empty());
    assert_eq!(document.nested_relations().count(), 1);
    assert_eq!(
        document.nested_relation(inner_key).unwrap().key(),
        inner_key
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn query_filter_alias_field_type_comes_from_exact_host_row_and_binder_owner() {
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
    let scan_id = scan.id;
    let symbol = builder.symbol("r", SpanV1::new(17, 18));
    let symbol_id = symbol.id;
    let path = builder
        .try_path(
            symbol,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(19, 21),
            }],
            SpanV1::new(17, 21),
        )
        .unwrap();
    let predicate = builder.compare(
        "=",
        path,
        builder.int("1", SpanV1::new(24, 25)),
        SpanV1::new(17, 25),
    );
    let filter = builder.relation_filter(scan, predicate, SpanV1::new(5, 25));
    let query = builder.query(filter, None, SpanV1::new(0, 25));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 25,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let expression = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: symbol_id,
        })
        .unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = expression.shape() else {
        panic!("exact row alias binding expected")
    };
    assert_eq!(
        (
            binding.target_node(),
            binding.ordinal(),
            binding.canonical_name()
        ),
        (scan_id, 0, "r")
    );
    let quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(ty) = expression.type_state() else {
        panic!("host row type required")
    };
    let row = ty.as_record().unwrap();
    assert_eq!(row.fields().len(), 1);
    assert_eq!(row.fields().first().unwrap().name(), "id");
    assert_eq!(
        row.fields().first().unwrap().value_type(),
        &ValueTypeV1::int()
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn query_projection_alias_is_a_prior_typed_expression_not_a_host_field_guess() {
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
    let path = builder
        .try_path(
            builder.symbol("r", SpanV1::new(22, 23)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(24, 26),
            }],
            SpanV1::new(22, 26),
        )
        .unwrap();
    let path_id = path.id;
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: path,
                alias: Some(AliasV1 {
                    name: "score".into(),
                    span: SpanV1::new(30, 35),
                }),
            }],
            SpanV1::new(5, 35),
        )
        .unwrap();
    let project_id = project.id;
    let output_ref = builder.symbol("score", SpanV1::new(44, 49));
    let output_ref_id = output_ref.id;
    let predicate = builder.compare(
        "=",
        output_ref,
        builder.int("1", SpanV1::new(52, 53)),
        SpanV1::new(44, 53),
    );
    let filter = builder.relation_filter(project, predicate, SpanV1::new(5, 53));
    let query = builder.query(filter, None, SpanV1::new(0, 53));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 53,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed query expected")
    };
    let project = relation.root().inputs().next().unwrap();
    assert!(matches!(
        project,
        quanta_gqlang_v2_sema::TypedRelationExprV2::Project(_)
    ));
    assert_eq!(project.expressions().first().unwrap().node_id, path_id);
    assert_eq!(project.output().columns().first().unwrap().name(), "score");
    assert_eq!(
        project.output().columns().first().unwrap().origin(),
        &quanta_gqlang_v2_sema::OutputColumnOriginV2::Expression(
            quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
                source_id,
                node_id: path_id
            }
        )
    );
    assert_eq!(
        project
            .output()
            .columns()
            .first()
            .unwrap()
            .location()
            .location()
            .span,
        SpanV1::new(30, 35)
    );
    let reference = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: output_ref_id,
        })
        .unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = reference.shape() else {
        panic!("projection frame binding required")
    };
    assert_eq!(
        (
            binding.target_node(),
            binding.ordinal(),
            binding.canonical_name()
        ),
        (project_id, 0, "score")
    );
    assert_eq!(
        reference.type_state(),
        &quanta_gqlang_v2_types::ExpressionTypeStateV2::Known(ValueTypeV1::int())
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn self_join_projected_columns_retain_distinct_scan_occurrences_and_checked_on() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let left_scan = builder.relation_source(
        "records",
        SpanV1::new(5, 12),
        Some(AliasV1 {
            name: "l".into(),
            span: SpanV1::new(13, 14),
        }),
        SpanV1::new(5, 14),
    );
    let left_scan_id = left_scan.id;
    let left_value = builder
        .try_path(
            builder.symbol("l", SpanV1::new(20, 21)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(22, 24),
            }],
            SpanV1::new(20, 24),
        )
        .unwrap();
    let left = builder
        .try_project(
            left_scan,
            vec![ProjectionItemV1 {
                value: left_value,
                alias: Some(AliasV1 {
                    name: "left_id".into(),
                    span: SpanV1::new(28, 35),
                }),
            }],
            SpanV1::new(5, 35),
        )
        .unwrap();
    let right_scan = builder.relation_source(
        "records",
        SpanV1::new(40, 47),
        Some(AliasV1 {
            name: "r".into(),
            span: SpanV1::new(48, 49),
        }),
        SpanV1::new(40, 49),
    );
    let right_scan_id = right_scan.id;
    let right_value = builder
        .try_path(
            builder.symbol("r", SpanV1::new(55, 56)),
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(57, 59),
            }],
            SpanV1::new(55, 59),
        )
        .unwrap();
    let right = builder
        .try_project(
            right_scan,
            vec![ProjectionItemV1 {
                value: right_value,
                alias: Some(AliasV1 {
                    name: "right_id".into(),
                    span: SpanV1::new(63, 71),
                }),
            }],
            SpanV1::new(40, 71),
        )
        .unwrap();
    let on = builder.compare(
        "=",
        builder.symbol("left_id", SpanV1::new(75, 82)),
        builder.symbol("right_id", SpanV1::new(85, 93)),
        SpanV1::new(75, 93),
    );
    let on_id = on.id;
    let join = builder.inner_join(left, right, on, SpanV1::new(5, 93));
    let query = builder.query(join, None, SpanV1::new(0, 93));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 93,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed self-join required")
    };
    assert!(matches!(
        relation.root(),
        quanta_gqlang_v2_sema::TypedRelationExprV2::Join(_)
    ));
    assert_eq!(
        relation.root().join_mode(),
        Some(quanta_gqlang_v2_sema::JoinModeV2::Inner)
    );
    assert_eq!(
        relation
            .root()
            .predicate()
            .unwrap()
            .expression()
            .key()
            .node_id,
        on_id
    );
    assert_eq!(
        relation
            .output()
            .row()
            .fields()
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
        vec!["left_id", "right_id"]
    );
    let sources = relation
        .root()
        .inputs()
        .map(|project| project.inputs().next().unwrap().key().node_id)
        .collect::<Vec<_>>();
    assert_eq!(sources, vec![left_scan_id, right_scan_id]);
    assert_ne!(sources.first().unwrap(), sources.get(1).unwrap());
    assert_eq!(relation.output().cardinality(), CardinalityV1::Bag);
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn ordered_projection_alias_and_null_key_retain_direction_null_policy_and_sequence() {
    let (_host, authority) = host();
    let source_id = SourceIdV2::from_raw(9);
    let builder = HirBuilderV1::new();
    let scan = builder.relation_source("records", SpanV1::new(5, 12), None, SpanV1::new(5, 12));
    let project = builder
        .try_project(
            scan,
            vec![ProjectionItemV1 {
                value: builder.int("3", SpanV1::new(20, 21)),
                alias: Some(AliasV1 {
                    name: "score".into(),
                    span: SpanV1::new(24, 29),
                }),
            }],
            SpanV1::new(5, 29),
        )
        .unwrap();
    let asc = builder.symbol("score", SpanV1::new(39, 44));
    let asc_id = asc.id;
    let desc = builder.null(SpanV1::new(51, 55));
    let desc_id = desc.id;
    let order = builder
        .order_by(
            project,
            vec![
                OrderKeyHirV1::nulls_last(asc, SortDirectionHirV1::Asc),
                OrderKeyHirV1::nulls_last(desc, SortDirectionHirV1::Desc),
            ],
            SpanV1::new(5, 55),
        )
        .unwrap();
    let query = builder.query(order, None, SpanV1::new(0, 55));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 55,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed sort required")
    };
    assert!(matches!(
        relation.root(),
        quanta_gqlang_v2_sema::TypedRelationExprV2::Sort(_)
    ));
    assert_eq!(relation.output().cardinality(), CardinalityV1::Sequence);
    assert_eq!(
        relation
            .root()
            .expressions()
            .iter()
            .map(|key| key.node_id)
            .collect::<Vec<_>>(),
        vec![asc_id, desc_id]
    );
    assert_eq!(
        relation
            .output()
            .ordering()
            .iter()
            .map(|key| (key.direction(), key.null_order()))
            .collect::<Vec<_>>(),
        vec![
            (
                quanta_gqlang_v2_sema::SortDirectionV2::Asc,
                quanta_gqlang_v2_sema::NullOrderV2::Last
            ),
            (
                quanta_gqlang_v2_sema::SortDirectionV2::Desc,
                quanta_gqlang_v2_sema::NullOrderV2::Last
            ),
        ]
    );
    assert_eq!(relation.output().columns().first().unwrap().name(), "score");
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn grouped_key_only_row_is_visible_to_sort_by_binder_owned_ordinal() {
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
    let key_id = key.id;
    let group = builder
        .group_by(scan, vec![key], SpanV1::new(5, 30))
        .unwrap();
    let group_id = group.id;
    let order_ref = builder.symbol("id", SpanV1::new(39, 41));
    let order_ref_id = order_ref.id;
    let order = builder
        .order_by(
            group,
            vec![OrderKeyHirV1::nulls_last(
                order_ref,
                SortDirectionHirV1::Asc,
            )],
            SpanV1::new(5, 41),
        )
        .unwrap();
    let query = builder.query(order, None, SpanV1::new(0, 41));
    let document = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id,
            source_len: 41,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap();
    let quanta_gqlang_v2_sema::SemanticItemV2::Relation(relation) =
        document.items().next().unwrap().1
    else {
        panic!("typed group and sort required")
    };
    let group = relation.root().inputs().next().unwrap();
    assert!(matches!(
        group,
        quanta_gqlang_v2_sema::TypedRelationExprV2::Group(_)
    ));
    assert_eq!(group.output().cardinality(), CardinalityV1::Set);
    assert_eq!(
        group
            .output()
            .row()
            .fields()
            .iter()
            .map(|field| field.name())
            .collect::<Vec<_>>(),
        vec!["id"]
    );
    assert_eq!(
        group.output().partition_keys().first().unwrap().node_id,
        key_id
    );
    assert_eq!(relation.output().cardinality(), CardinalityV1::Sequence);
    let reference = document
        .expression(quanta_gqlang_v2_types::SourceQualifiedNodeIdV2 {
            source_id,
            node_id: order_ref_id,
        })
        .unwrap();
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Binding(binding) = reference.shape() else {
        panic!("grouped key binding expected")
    };
    assert_eq!(
        (
            binding.target_node(),
            binding.ordinal(),
            binding.canonical_name()
        ),
        (group_id, 0, "id")
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn source_alias_disappears_from_public_group_output_frame() {
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
    let unavailable = builder.symbol("r", SpanV1::new(39, 40));
    let order = builder
        .order_by(
            group,
            vec![OrderKeyHirV1::nulls_last(
                unavailable,
                SortDirectionHirV1::Asc,
            )],
            SpanV1::new(5, 40),
        )
        .unwrap();
    let query = builder.query(order, None, SpanV1::new(0, 40));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 40,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("pre-group alias cannot escape group frame")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(39, 40)
    );
}

#[test]
#[expect(
    clippy::panic,
    reason = "wrong semantic variant must fail this owner test"
)]
fn post_group_filter_cannot_recover_pre_group_field_without_aggregate_authority() {
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
            builder.symbol("r", SpanV1::new(39, 40)),
            vec![PathSegmentV1 {
                name: "value".into(),
                span: SpanV1::new(41, 46),
            }],
            SpanV1::new(39, 46),
        )
        .unwrap();
    let predicate = builder.compare(
        "=",
        value,
        builder.int("1", SpanV1::new(49, 50)),
        SpanV1::new(39, 50),
    );
    let filter = builder.relation_filter(group, predicate, SpanV1::new(5, 50));
    let query = builder.query(filter, None, SpanV1::new(0, 50));
    let FrontendTransactionErrorV2::Document(error) = resolve_checked_document_v2(
        CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: SourceIdV2::from_raw(9),
            source_len: 50,
            items: vec![ProgramItemHirV1::Query(Box::new(query))],
        })
        .unwrap(),
        &authority,
    )
    .unwrap_err() else {
        panic!("post-group WHERE must see only public key row")
    };
    assert_eq!(
        error.diagnostics().first().unwrap().primary().span,
        SpanV1::new(39, 40)
    );
}
