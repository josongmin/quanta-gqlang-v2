#![allow(clippy::unwrap_used, reason = "owner assertions")]

use quanta_gqlang_v2_hir::{
    check_document_v2, CallArgHirV1, CallArgsHirV1, CheckedHirDocumentV2, DeclHirV1, HirBuilderV1,
    HirDocumentV1, HirNodeIdV1, PathSegmentV1, ProgramItemHirV1, RecordFieldV1, SourceIdV2,
    SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_types::*;

fn resolved_type_inputs(
    checked: CheckedHirDocumentV2<'_>,
    entries: Vec<ResolvedTypeInputEntryV2>,
    contexts: Vec<ContextualExpectedTypeInputV2>,
) -> Result<ResolvedTypeInputsV2<'_>, TypeDiagnosticV2> {
    let registry = SourceRegistryViewV2::try_new(Vec::new())
        .unwrap()
        .try_extend_document(SourceRegistrationV2 {
            source_id: checked.document().source_id,
            source_len: checked.document().source_len,
        })
        .unwrap();
    ResolvedTypeInputsV2::try_new(checked, &registry, entries, contexts)
}

fn document(body: quanta_gqlang_v2_hir::ValueExprHirV1) -> HirDocumentV1 {
    HirDocumentV1 {
        source_id: SourceIdV2::from_raw(7),
        source_len: 512,
        items: vec![ProgramItemHirV1::Decl(Box::new(DeclHirV1::new(
            HirNodeIdV1::from_raw(10_000),
            SpanV1::new(0, 500),
            "x".into(),
            SpanV1::new(0, 1),
            body,
        )))],
    }
}

fn check_value(
    body: quanta_gqlang_v2_hir::ValueExprHirV1,
    entries: Vec<ResolvedTypeInputEntryV2>,
) -> TypedExpressionFactsV2 {
    let doc = document(body);
    let inputs = resolved_type_inputs(check_document_v2(&doc).unwrap(), entries, vec![]).unwrap();
    ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap()
}

fn assert_bad_projection(base_type: ValueTypeV1, field: &str) {
    let builder = HirBuilderV1::new();
    let base = builder.symbol("base", SpanV1::new(0, 4));
    let base_id = base.id;
    let path = builder
        .try_path(
            base,
            vec![PathSegmentV1 {
                name: field.to_owned(),
                span: SpanV1::new(5, 5 + field.len()),
            }],
            SpanV1::new(0, 5 + field.len()),
        )
        .unwrap();
    let path_id = path.id;
    let input = ResolvedTypeInputEntryV2 {
        key: SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(7),
            node_id: base_id,
        },
        value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(base_type.clone())),
    };
    let doc = document(path);
    let inputs =
        resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    let error = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap_err();
    assert_eq!(error.code(), "types.bad_projection");
    assert!(error.to_string().contains(field));
    assert!(matches!(&error, TypeDiagnosticV2::BadProjection { .. }));
    if let TypeDiagnosticV2::BadProjection {
        node_id,
        base,
        field: actual_field,
        location,
    } = &error
    {
        assert_eq!(*node_id, path_id);
        assert_eq!(base, &base_type);
        assert_eq!(actual_field, field);
        assert_eq!(location.source_id, SourceIdV2::from_raw(7));
        assert_eq!(location.span, SpanV1::new(5, 5 + field.len()));
    }
}

#[test]
fn type_inputs_require_the_exact_extended_document_registry() {
    let builder = HirBuilderV1::new();
    let value = builder.int("7", SpanV1::new(0, 1));
    let doc = document(value);
    let checked = check_document_v2(&doc).unwrap();
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(99),
        source_len: 12,
    }])
    .unwrap();
    let exact = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: doc.source_id,
            source_len: doc.source_len,
        })
        .unwrap();
    ResolvedTypeInputsV2::try_new(checked, &exact, vec![], vec![]).unwrap();

    let wrong_length = host
        .try_extend_document(SourceRegistrationV2 {
            source_id: doc.source_id,
            source_len: doc.source_len - 1,
        })
        .unwrap();
    let error = ResolvedTypeInputsV2::try_new(checked, &wrong_length, vec![], vec![]).unwrap_err();
    assert_eq!(error.code(), "types.source_registry_mismatch");
}

#[test]
fn binding_from_prior_expression_reuses_the_single_kernel_fact() {
    let builder = HirBuilderV1::new();
    let producer = builder.int("7", SpanV1::new(0, 1));
    let producer_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: producer.id,
    };
    let reference = builder.symbol("projected", SpanV1::new(3, 12));
    let reference_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: reference.id,
    };
    let root = builder.list(vec![producer, reference], SpanV1::new(0, 13));
    let doc = document(root);
    let inputs = resolved_type_inputs(
        check_document_v2(&doc).unwrap(),
        vec![ResolvedTypeInputEntryV2 {
            key: reference_key,
            value: ResolvedTypeInputValueV2::BindingFromExpression(producer_key),
        }],
        vec![],
    )
    .unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(
        facts.get(reference_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::int())
    );
}

#[test]
fn binding_expression_dependencies_reject_foreign_missing_self_and_forward_edges() {
    for target in [
        SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(99),
            node_id: HirNodeIdV1::from_raw(1),
        },
        SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(7),
            node_id: HirNodeIdV1::from_raw(99_999),
        },
    ] {
        let builder = HirBuilderV1::new();
        let reference = builder.symbol("x", SpanV1::new(0, 1));
        let key = SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(7),
            node_id: reference.id,
        };
        let doc = document(reference);
        let error = resolved_type_inputs(
            check_document_v2(&doc).unwrap(),
            vec![ResolvedTypeInputEntryV2 {
                key,
                value: ResolvedTypeInputValueV2::BindingFromExpression(target),
            }],
            vec![],
        )
        .unwrap_err();
        assert_eq!(error.code(), "types.invalid_binding_dependency");
    }

    let builder = HirBuilderV1::new();
    let reference = builder.symbol("self_ref", SpanV1::new(0, 8));
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: reference.id,
    };
    let doc = document(reference);
    let error = resolved_type_inputs(
        check_document_v2(&doc).unwrap(),
        vec![ResolvedTypeInputEntryV2 {
            key,
            value: ResolvedTypeInputValueV2::BindingFromExpression(key),
        }],
        vec![],
    )
    .unwrap_err();
    assert_eq!(error.code(), "types.invalid_binding_dependency");

    let builder = HirBuilderV1::new();
    let reference = builder.symbol("future", SpanV1::new(0, 6));
    let reference_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: reference.id,
    };
    let producer = builder.int("7", SpanV1::new(8, 9));
    let producer_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: producer.id,
    };
    let root = builder.list(vec![reference, producer], SpanV1::new(0, 10));
    let doc = document(root);
    let inputs = resolved_type_inputs(
        check_document_v2(&doc).unwrap(),
        vec![ResolvedTypeInputEntryV2 {
            key: reference_key,
            value: ResolvedTypeInputValueV2::BindingFromExpression(producer_key),
        }],
        vec![],
    )
    .unwrap();
    let error = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap_err();
    assert_eq!(error.code(), "types.unavailable_binding_dependency");

    let builder = HirBuilderV1::new();
    let left = builder.symbol("left", SpanV1::new(0, 4));
    let right = builder.symbol("right", SpanV1::new(6, 11));
    let left_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: left.id,
    };
    let right_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: right.id,
    };
    let root = builder.list(vec![left, right], SpanV1::new(0, 12));
    let doc = document(root);
    let inputs = resolved_type_inputs(
        check_document_v2(&doc).unwrap(),
        vec![
            ResolvedTypeInputEntryV2 {
                key: left_key,
                value: ResolvedTypeInputValueV2::BindingFromExpression(right_key),
            },
            ResolvedTypeInputEntryV2 {
                key: right_key,
                value: ResolvedTypeInputValueV2::BindingFromExpression(left_key),
            },
        ],
        vec![],
    )
    .unwrap();
    let error = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap_err();
    assert_eq!(error.code(), "types.unavailable_binding_dependency");
}

#[test]
fn predeclared_forward_declaration_uses_one_kernel_pass() {
    let builder = HirBuilderV1::new();
    let reference = builder.symbol("later", SpanV1::new(8, 13));
    let reference_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: reference.id,
    };
    let later = builder.int("1", SpanV1::new(32, 33));
    let later_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: later.id,
    };
    let doc = HirDocumentV1 {
        source_id: SourceIdV2::from_raw(7),
        source_len: 64,
        items: vec![
            ProgramItemHirV1::Decl(Box::new(DeclHirV1::new(
                HirNodeIdV1::from_raw(10_000),
                SpanV1::new(0, 16),
                "first".into(),
                SpanV1::new(0, 5),
                reference,
            ))),
            ProgramItemHirV1::Decl(Box::new(DeclHirV1::new(
                HirNodeIdV1::from_raw(10_001),
                SpanV1::new(20, 40),
                "later".into(),
                SpanV1::new(20, 25),
                later,
            ))),
        ],
    };
    let inputs = resolved_type_inputs(
        check_document_v2(&doc).unwrap(),
        vec![ResolvedTypeInputEntryV2 {
            key: reference_key,
            value: ResolvedTypeInputValueV2::BindingFromDeclaration(later_key),
        }],
        vec![],
    )
    .unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(
        facts.get(reference_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::int())
    );
}

#[test]
fn cyclic_declarations_fail_without_guessing_a_type() {
    let builder = HirBuilderV1::new();
    let left = builder.symbol("right", SpanV1::new(8, 13));
    let right = builder.symbol("left", SpanV1::new(30, 34));
    let left_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: left.id,
    };
    let right_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: right.id,
    };
    let doc = HirDocumentV1 {
        source_id: SourceIdV2::from_raw(7),
        source_len: 64,
        items: vec![
            ProgramItemHirV1::Decl(Box::new(DeclHirV1::new(
                HirNodeIdV1::from_raw(10_000),
                SpanV1::new(0, 16),
                "left".into(),
                SpanV1::new(0, 4),
                left,
            ))),
            ProgramItemHirV1::Decl(Box::new(DeclHirV1::new(
                HirNodeIdV1::from_raw(10_001),
                SpanV1::new(20, 40),
                "right".into(),
                SpanV1::new(20, 25),
                right,
            ))),
        ],
    };
    let inputs = resolved_type_inputs(
        check_document_v2(&doc).unwrap(),
        vec![
            ResolvedTypeInputEntryV2 {
                key: left_key,
                value: ResolvedTypeInputValueV2::BindingFromDeclaration(right_key),
            },
            ResolvedTypeInputEntryV2 {
                key: right_key,
                value: ResolvedTypeInputValueV2::BindingFromDeclaration(left_key),
            },
        ],
        vec![],
    )
    .unwrap();
    assert_eq!(
        ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs)
            .unwrap_err()
            .code(),
        "types.cyclic_declaration_dependency"
    );
}

#[test]
fn declaration_lazy_dependency_rejects_non_declaration_target() {
    let builder = HirBuilderV1::new();
    let reference = builder.symbol("x", SpanV1::new(0, 1));
    let reference_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: reference.id,
    };
    let literal = builder.int("1", SpanV1::new(3, 4));
    let literal_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: literal.id,
    };
    let doc = document(builder.list(vec![reference, literal], SpanV1::new(0, 5)));
    let error = resolved_type_inputs(
        check_document_v2(&doc).unwrap(),
        vec![ResolvedTypeInputEntryV2 {
            key: reference_key,
            value: ResolvedTypeInputValueV2::BindingFromDeclaration(literal_key),
        }],
        vec![],
    )
    .unwrap_err();
    assert_eq!(error.code(), "types.invalid_binding_dependency");
}

#[test]
fn typed_literals_keep_values_and_numeric_boundaries() {
    let builder = HirBuilderV1::new();
    let value = builder.int(i128::MAX.to_string(), SpanV1::new(0, 40));
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: value.id,
    };
    let doc = document(value);
    let inputs = resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(
        facts.get(key).unwrap().literal,
        Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Int(
            i128::MAX
        )))
    );
}

#[test]
fn unary_minimum_uses_closed_deferred_child_fact() {
    let builder = HirBuilderV1::new();
    let child = builder.int(
        "170141183460469231731687303715884105728",
        SpanV1::new(1, 40),
    );
    let child_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: child.id,
    };
    let unary = builder.unary("-", child, SpanV1::new(0, 40));
    let unary_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: unary.id,
    };
    let doc = document(unary);
    let inputs = resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed()
        .unwrap()
        .check_document(&inputs)
        .unwrap();
    assert_eq!(
        facts.get(child_key).unwrap().literal,
        Some(TypedLiteralFactV2::DeferredI128MinMagnitude)
    );
    assert_eq!(
        facts.get(unary_key).unwrap().literal,
        Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Int(
            i128::MIN
        )))
    );
}

#[test]
fn non_finite_and_nonzero_underflow_reject_with_exact_codes() {
    let overflow = format!("{}.0", "9".repeat(400));
    let underflow = format!("0.{}1", "0".repeat(400));
    for (text, code) in [
        (&overflow, "types.float_non_finite"),
        (&underflow, "types.float_underflow"),
    ] {
        let builder = HirBuilderV1::new();
        let doc = document(builder.float(text, SpanV1::new(0, text.len())));
        let inputs =
            resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
        assert_eq!(
            ExpressionTypeKernelV2::try_closed()
                .unwrap()
                .check_document(&inputs)
                .unwrap_err()
                .code(),
            code
        );
    }
}

#[test]
fn signed_exponent_whitespace_and_separator_numeric_payloads_reject_as_malformed() {
    for text in ["+1", "-1", " 1", "1_0"] {
        let builder = HirBuilderV1::new();
        let doc = document(builder.int(text, SpanV1::new(0, text.len())));
        let inputs =
            resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
        assert_eq!(
            ExpressionTypeKernelV2::try_closed()
                .unwrap()
                .check_document(&inputs)
                .unwrap_err()
                .code(),
            "types.malformed_literal"
        );
    }
    for text in ["1e3", "1.", ".1", "+1.0", "1.0 "] {
        let builder = HirBuilderV1::new();
        let doc = document(builder.float(text, SpanV1::new(0, text.len())));
        let inputs =
            resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
        assert_eq!(
            ExpressionTypeKernelV2::try_closed()
                .unwrap()
                .check_document(&inputs)
                .unwrap_err()
                .code(),
            "types.malformed_literal"
        );
    }
}

#[test]
fn empty_list_is_unknown_without_context_and_exact_with_checked_context() {
    let builder = HirBuilderV1::new();
    let list = builder.list(vec![], SpanV1::new(0, 2));
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: list.id,
    };
    let doc = document(list);
    let checked = check_document_v2(&doc).unwrap();
    let open = resolved_type_inputs(checked, vec![], vec![]).unwrap();
    assert_eq!(
        ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&open)
            .unwrap()
            .get(key)
            .unwrap()
            .state,
        ExpressionTypeStateV2::Unknown(UnknownTypeCauseV2::EmptyCollectionElement)
    );
    let expected = ValueTypeV1::try_list(ValueTypeV1::string()).unwrap();
    let context = ContextualExpectedTypeInputV2 {
        key,
        expected_type: expected.clone(),
        reason: ContextualTypeReasonV2::ExpectedBinding,
    };
    let contextual = resolved_type_inputs(checked, vec![], vec![context]).unwrap();
    assert_eq!(
        ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&contextual)
            .unwrap()
            .get(key)
            .unwrap()
            .state,
        ExpressionTypeStateV2::Known(expected)
    );
}

#[test]
fn positive_plus_and_larger_than_minimum_magnitude_reject() {
    for (operator, magnitude) in [
        ("+", "170141183460469231731687303715884105728"),
        ("-", "170141183460469231731687303715884105729"),
    ] {
        let builder = HirBuilderV1::new();
        let child = builder.int(magnitude, SpanV1::new(1, 40));
        let unary = builder.unary(operator, child, SpanV1::new(0, 40));
        let doc = document(unary);
        let inputs =
            resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
        assert_eq!(
            ExpressionTypeKernelV2::try_closed()
                .unwrap()
                .check_document(&inputs)
                .unwrap_err()
                .code(),
            "types.integer_out_of_range"
        );
    }
}

#[test]
fn integer_and_float_addition_select_exact_result_types() {
    for (left, right, expected) in [
        ("1", "2", ValueTypeV1::int()),
        ("1.0", "2.0", ValueTypeV1::float()),
    ] {
        let builder = HirBuilderV1::new();
        let lhs = if left.contains('.') {
            builder.float(left, SpanV1::new(0, left.len()))
        } else {
            builder.int(left, SpanV1::new(0, left.len()))
        };
        let rhs = if right.contains('.') {
            builder.float(right, SpanV1::new(4, 4 + right.len()))
        } else {
            builder.int(right, SpanV1::new(4, 4 + right.len()))
        };
        let value = builder.binary("+", lhs, rhs, SpanV1::new(0, 8));
        let key = SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(7),
            node_id: value.id,
        };
        let facts = check_value(value, vec![]);
        let fact = facts.get(key).unwrap();
        assert_eq!(fact.state, ExpressionTypeStateV2::Known(expected));
        assert!(fact.operator_id.is_some());
    }
}

#[test]
fn mixed_string_and_integer_addition_rejects_bad_overload() {
    let builder = HirBuilderV1::new();
    let value = builder.binary(
        "+",
        builder.string("x", SpanV1::new(0, 3)),
        builder.int("1", SpanV1::new(6, 7)),
        SpanV1::new(0, 7),
    );
    let doc = document(value);
    let inputs = resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
    assert_eq!(
        ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs)
            .unwrap_err()
            .code(),
        "types.bad_overload"
    );
}

#[test]
fn parenthesized_value_preserves_type_and_child_fact() {
    let builder = HirBuilderV1::new();
    let child = builder.int("7", SpanV1::new(1, 2));
    let child_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: child.id,
    };
    let parent = builder.paren_value(child, SpanV1::new(0, 3));
    let parent_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: parent.id,
    };
    let facts = check_value(parent, vec![]);
    assert_eq!(
        facts.get(child_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::int())
    );
    assert_eq!(
        facts.get(parent_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::int())
    );
}

#[test]
fn nested_record_and_list_type_through_the_shared_kernel() {
    let builder = HirBuilderV1::new();
    let list = builder.list(
        vec![
            builder.string("a", SpanV1::new(8, 11)),
            builder.string("b", SpanV1::new(13, 16)),
        ],
        SpanV1::new(7, 17),
    );
    let record = builder.record(
        vec![RecordFieldV1 {
            name: "tags".into(),
            name_span: SpanV1::new(2, 6),
            value: list,
        }],
        SpanV1::new(0, 19),
    );
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: record.id,
    };
    let facts = check_value(record, vec![]);
    let state = &facts.get(key).unwrap().state;
    assert!(matches!(state, ExpressionTypeStateV2::Known(_)));
    if let ExpressionTypeStateV2::Known(value_type) = state {
        assert_eq!(value_type.to_string(), "{ tags: list<string> }")
    }
}

#[test]
fn path_projection_reads_checked_record_fields() {
    let builder = HirBuilderV1::new();
    let base = builder.record(
        vec![RecordFieldV1 {
            name: "id".into(),
            name_span: SpanV1::new(1, 3),
            value: builder.int("1", SpanV1::new(5, 6)),
        }],
        SpanV1::new(0, 7),
    );
    let path = builder
        .try_path(
            base,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(8, 10),
            }],
            SpanV1::new(0, 10),
        )
        .unwrap();
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: path.id,
    };
    assert_eq!(
        check_value(path, vec![]).get(key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::int())
    );
}

#[test]
fn invalid_projection_branches_preserve_base_field_and_segment_location() {
    assert_bad_projection(ValueTypeV1::int(), "field");
    assert_bad_projection(
        ValueTypeV1::try_optional(ValueTypeV1::int()).unwrap(),
        "field",
    );

    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(7),
        source_len: 32,
    }])
    .unwrap();
    let record = RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
        "id",
        ValueTypeV1::int(),
        registry
            .try_check_location(quanta_gqlang_v2_hir::LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(7),
                span: SpanV1::new(0, 1),
            })
            .unwrap(),
    )
    .unwrap()])
    .unwrap();
    assert_bad_projection(ValueTypeV1::try_record(record).unwrap(), "missing");
}

#[test]
fn optional_record_assignability_and_null_are_explicit() {
    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(7),
        source_len: 32,
    }])
    .unwrap();
    let row = RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
        "id",
        ValueTypeV1::int(),
        registry
            .try_check_location(quanta_gqlang_v2_hir::LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(7),
                span: SpanV1::new(0, 1),
            })
            .unwrap(),
    )
    .unwrap()])
    .unwrap();
    let record = ValueTypeV1::try_record(row).unwrap();
    let optional = ValueTypeV1::try_optional(record.clone()).unwrap();
    assert!(record.is_assignable_to(&optional));
    assert!(ValueTypeV1::null().is_assignable_to(&optional));
    assert!(
        !ValueTypeV1::null().is_assignable_to(&ValueTypeV1::try_list(ValueTypeV1::int()).unwrap())
    );
}

#[test]
fn size_builtin_accepts_only_its_exact_collection_domain() {
    let registry = BuiltinRegistryV2::try_closed().unwrap();
    for (element, ok) in [("1", true), ("x", false)] {
        let builder = HirBuilderV1::new();
        let item = if ok {
            builder.int(element, SpanV1::new(6, 7))
        } else {
            builder.string(element, SpanV1::new(6, 9))
        };
        let list = builder.list(vec![item], SpanV1::new(5, 10));
        let call = builder.call(
            "size",
            SpanV1::new(0, 4),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(list)]).unwrap(),
            SpanV1::new(0, 11),
        );
        let key = SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(7),
            node_id: call.id,
        };
        let entry = ResolvedTypeInputEntryV2 {
            key,
            value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
                registry.family_id("size").unwrap(),
            )),
        };
        let doc = document(call);
        let inputs =
            resolved_type_inputs(check_document_v2(&doc).unwrap(), vec![entry], vec![]).unwrap();
        let result = ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs);
        if ok {
            assert_eq!(
                result.unwrap().get(key).unwrap().state,
                ExpressionTypeStateV2::Known(ValueTypeV1::int())
            );
        } else {
            assert_eq!(result.unwrap_err().code(), "types.bad_overload");
        }
    }
}

#[test]
fn aggregate_overload_retains_selected_result_and_effect_class() {
    let builder = HirBuilderV1::new();
    let arg = builder.float("1.0", SpanV1::new(4, 7));
    let call = builder.call(
        "sum",
        SpanV1::new(0, 3),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(),
        SpanV1::new(0, 8),
    );
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: call.id,
    };
    let registry = BuiltinRegistryV2::try_closed().unwrap();
    let entry = ResolvedTypeInputEntryV2 {
        key,
        value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
            registry.family_id("sum").unwrap(),
        )),
    };
    let facts = check_value(call, vec![entry]);
    let fact = facts.get(key).unwrap();
    assert_eq!(
        fact.state,
        ExpressionTypeStateV2::Known(ValueTypeV1::float())
    );
    assert_eq!(fact.effects.aggregation, AggregationClassV1::Aggregate);
    assert_eq!(fact.effects.cost, CostClassV1::Linear);
    assert!(matches!(
        fact.resolved_call(),
        Some(ResolvedCallTargetV2::Builtin(_))
    ));
}

#[test]
fn path_and_proof_builtin_families_type_through_the_shared_kernel() {
    let registry = BuiltinRegistryV2::try_closed().unwrap();
    for (name, input_type, expected) in [
        (
            "path_length",
            ValueTypeV1::scalar(ScalarTyV1::Path),
            ValueTypeV1::int(),
        ),
        (
            "path_locations",
            ValueTypeV1::scalar(ScalarTyV1::Path),
            ValueTypeV1::try_collection(
                ValueTypeV1::scalar(ScalarTyV1::Location),
                CardinalityV1::Sequence,
            )
            .unwrap(),
        ),
    ] {
        let builder = HirBuilderV1::new();
        let argument = builder.symbol("path", SpanV1::new(20, 24));
        let argument_id = argument.id;
        let call = builder.call(
            name,
            SpanV1::new(0, name.len()),
            CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(argument)]).unwrap(),
            SpanV1::new(0, 25),
        );
        let call_key = SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(7),
            node_id: call.id,
        };
        let facts = check_value(
            call,
            vec![
                ResolvedTypeInputEntryV2 {
                    key: SourceQualifiedNodeIdV2 {
                        source_id: SourceIdV2::from_raw(7),
                        node_id: argument_id,
                    },
                    value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
                        input_type,
                    )),
                },
                ResolvedTypeInputEntryV2 {
                    key: call_key,
                    value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
                        registry.family_id(name).unwrap(),
                    )),
                },
            ],
        );
        assert_eq!(
            facts.get(call_key).unwrap().state,
            ExpressionTypeStateV2::Known(expected)
        );
    }

    let builder = HirBuilderV1::new();
    let flag = builder.symbol("flag", SpanV1::new(20, 24));
    let flag_id = flag.id;
    let explain = builder.call(
        "explain",
        SpanV1::new(11, 18),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(flag)]).unwrap(),
        SpanV1::new(11, 25),
    );
    let explain_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: explain.id,
    };
    let proof_size = builder.call(
        "proof_size",
        SpanV1::new(0, 10),
        CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(explain)]).unwrap(),
        SpanV1::new(0, 26),
    );
    let proof_size_key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: proof_size.id,
    };
    let facts = check_value(
        proof_size,
        vec![
            ResolvedTypeInputEntryV2 {
                key: SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(7),
                    node_id: flag_id,
                },
                value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
                    ValueTypeV1::bool(),
                )),
            },
            ResolvedTypeInputEntryV2 {
                key: explain_key,
                value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
                    registry.family_id("explain").unwrap(),
                )),
            },
            ResolvedTypeInputEntryV2 {
                key: proof_size_key,
                value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(
                    registry.family_id("proof_size").unwrap(),
                )),
            },
        ],
    );
    assert_eq!(
        facts.get(explain_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::scalar(ScalarTyV1::ProofTree))
    );
    assert_eq!(
        facts.get(proof_size_key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::int())
    );
}

#[test]
fn repo_path_equality_is_exact_and_does_not_coerce_to_path() {
    for (right_type, should_succeed) in [
        (ValueTypeV1::scalar(ScalarTyV1::RepoPath), true),
        (ValueTypeV1::scalar(ScalarTyV1::Path), false),
    ] {
        let builder = HirBuilderV1::new();
        let left = builder.symbol("left", SpanV1::new(0, 4));
        let left_id = left.id;
        let right = builder.symbol("right", SpanV1::new(7, 12));
        let right_id = right.id;
        let equality = builder.binary("=", left, right, SpanV1::new(0, 12));
        let key = SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(7),
            node_id: equality.id,
        };
        let entries = vec![
            ResolvedTypeInputEntryV2 {
                key: SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(7),
                    node_id: left_id,
                },
                value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
                    ValueTypeV1::scalar(ScalarTyV1::RepoPath),
                )),
            },
            ResolvedTypeInputEntryV2 {
                key: SourceQualifiedNodeIdV2 {
                    source_id: SourceIdV2::from_raw(7),
                    node_id: right_id,
                },
                value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(right_type)),
            },
        ];
        let doc = document(equality);
        let inputs =
            resolved_type_inputs(check_document_v2(&doc).unwrap(), entries, vec![]).unwrap();
        let result = ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs);
        if should_succeed {
            assert_eq!(
                result.unwrap().get(key).unwrap().state,
                ExpressionTypeStateV2::Known(ValueTypeV1::bool())
            );
        } else {
            assert_eq!(result.unwrap_err().code(), "types.bad_overload");
        }
    }
}

#[test]
fn proof_tree_arithmetic_is_not_admitted_by_value_overloads() {
    let builder = HirBuilderV1::new();
    let left = builder.symbol("left", SpanV1::new(0, 4));
    let left_id = left.id;
    let right = builder.symbol("right", SpanV1::new(7, 12));
    let right_id = right.id;
    let sum = builder.binary("+", left, right, SpanV1::new(0, 12));
    let entries = [left_id, right_id]
        .into_iter()
        .map(|node_id| ResolvedTypeInputEntryV2 {
            key: SourceQualifiedNodeIdV2 {
                source_id: SourceIdV2::from_raw(7),
                node_id,
            },
            value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(
                ValueTypeV1::scalar(ScalarTyV1::ProofTree),
            )),
        })
        .collect();
    let doc = document(sum);
    let inputs = resolved_type_inputs(check_document_v2(&doc).unwrap(), entries, vec![]).unwrap();
    assert_eq!(
        ExpressionTypeKernelV2::try_closed()
            .unwrap()
            .check_document(&inputs)
            .unwrap_err()
            .code(),
        "types.bad_overload"
    );
}

#[test]
fn optional_record_projection_preserves_nullability() {
    let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(7),
        source_len: 64,
    }])
    .unwrap();
    let record = RecordTypeV1::try_new(vec![RecordFieldTypeV1::try_new(
        "id",
        ValueTypeV1::int(),
        registry
            .try_check_location(quanta_gqlang_v2_hir::LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(7),
                span: SpanV1::new(0, 1),
            })
            .unwrap(),
    )
    .unwrap()])
    .unwrap();
    let optional_record =
        ValueTypeV1::try_optional(ValueTypeV1::try_record(record).unwrap()).unwrap();
    let builder = HirBuilderV1::new();
    let base = builder.symbol("maybe", SpanV1::new(0, 5));
    let base_id = base.id;
    let path = builder
        .try_path(
            base,
            vec![PathSegmentV1 {
                name: "id".into(),
                span: SpanV1::new(6, 8),
            }],
            SpanV1::new(0, 8),
        )
        .unwrap();
    let key = SourceQualifiedNodeIdV2 {
        source_id: SourceIdV2::from_raw(7),
        node_id: path.id,
    };
    let input = ResolvedTypeInputEntryV2 {
        key: SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(7),
            node_id: base_id,
        },
        value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(optional_record)),
    };
    assert_eq!(
        check_value(path, vec![input]).get(key).unwrap().state,
        ExpressionTypeStateV2::Known(ValueTypeV1::try_optional(ValueTypeV1::int()).unwrap())
    );
}
