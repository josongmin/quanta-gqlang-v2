#![allow(clippy::unwrap_used, reason = "owner assertions")]

use quanta_gqlang_v2_hir::{check_document_v2, DeclHirV1, HirBuilderV1, HirDocumentV1, HirNodeIdV1, ProgramItemHirV1, SourceIdV2, SpanV1};
use quanta_gqlang_v2_types::*;

fn document(body: quanta_gqlang_v2_hir::ValueExprHirV1) -> HirDocumentV1 {
    HirDocumentV1 { source_id: SourceIdV2::from_raw(7), source_len: 512, items: vec![ProgramItemHirV1::Decl(Box::new(DeclHirV1::new(HirNodeIdV1::from_raw(10_000), SpanV1::new(0, 500), "x".into(), SpanV1::new(0, 1), body)))] }
}

#[test]
fn typed_literals_keep_values_and_numeric_boundaries() {
    for (text, expected) in [(i128::MAX.to_string(), i128::MAX)] {
        let builder = HirBuilderV1::new();
        let value = builder.int(text, SpanV1::new(0, 40));
        let key = SourceQualifiedNodeIdV2 { source_id: SourceIdV2::from_raw(7), node_id: value.id };
        let doc = document(value);
        let inputs = ResolvedTypeInputsV2::try_new(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
        let facts = ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs).unwrap();
        assert_eq!(facts.get(key).unwrap().literal, Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Int(expected))));
    }
}

#[test]
fn unary_minimum_uses_closed_deferred_child_fact() {
    let builder = HirBuilderV1::new();
    let child = builder.int("170141183460469231731687303715884105728", SpanV1::new(1, 40));
    let child_key = SourceQualifiedNodeIdV2 { source_id: SourceIdV2::from_raw(7), node_id: child.id };
    let unary = builder.unary("-", child, SpanV1::new(0, 40));
    let unary_key = SourceQualifiedNodeIdV2 { source_id: SourceIdV2::from_raw(7), node_id: unary.id };
    let doc = document(unary);
    let inputs = ResolvedTypeInputsV2::try_new(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs).unwrap();
    assert_eq!(facts.get(child_key).unwrap().literal, Some(TypedLiteralFactV2::DeferredI128MinMagnitude));
    assert_eq!(facts.get(unary_key).unwrap().literal, Some(TypedLiteralFactV2::Value(TypedLiteralValueV2::Int(i128::MIN))));
}

#[test]
fn non_finite_and_nonzero_underflow_reject_with_exact_codes() {
    let overflow = format!("{}.0", "9".repeat(400));
    let underflow = format!("0.{}1", "0".repeat(400));
    for (text, code) in [(&overflow, "types.float_non_finite"), (&underflow, "types.float_underflow")] {
        let builder = HirBuilderV1::new();
        let doc = document(builder.float(text, SpanV1::new(0, text.len())));
        let inputs = ResolvedTypeInputsV2::try_new(check_document_v2(&doc).unwrap(), vec![], vec![]).unwrap();
        assert_eq!(ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs).unwrap_err().code(), code);
    }
}

#[test]
fn signed_exponent_whitespace_and_separator_numeric_payloads_reject_as_malformed() {
    for text in ["+1", "-1", " 1", "1_0"] {
        let builder=HirBuilderV1::new(); let doc=document(builder.int(text,SpanV1::new(0,text.len())));
        let inputs=ResolvedTypeInputsV2::try_new(check_document_v2(&doc).unwrap(),vec![],vec![]).unwrap();
        assert_eq!(ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs).unwrap_err().code(),"types.malformed_literal");
    }
    for text in ["1e3","1.",".1","+1.0","1.0 "] {
        let builder=HirBuilderV1::new(); let doc=document(builder.float(text,SpanV1::new(0,text.len())));
        let inputs=ResolvedTypeInputsV2::try_new(check_document_v2(&doc).unwrap(),vec![],vec![]).unwrap();
        assert_eq!(ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs).unwrap_err().code(),"types.malformed_literal");
    }
}

#[test]
fn empty_list_is_unknown_without_context_and_exact_with_checked_context() {
    let builder=HirBuilderV1::new(); let list=builder.list(vec![],SpanV1::new(0,2));
    let key=SourceQualifiedNodeIdV2{source_id:SourceIdV2::from_raw(7),node_id:list.id}; let doc=document(list);
    let checked=check_document_v2(&doc).unwrap();
    let open=ResolvedTypeInputsV2::try_new(checked,vec![],vec![]).unwrap();
    assert_eq!(ExpressionTypeKernelV2::try_closed().unwrap().check_document(&open).unwrap().get(key).unwrap().state,ExpressionTypeStateV2::Unknown(UnknownTypeCauseV2::EmptyCollectionNeedsContext));
    let context=ContextualExpectedTypeInputV2{key,expected_type:TypeV1::list(TypeV1::string()),reason:ContextualTypeReasonV2::ExpectedBinding};
    let contextual=ResolvedTypeInputsV2::try_new(checked,vec![],vec![context]).unwrap();
    assert_eq!(ExpressionTypeKernelV2::try_closed().unwrap().check_document(&contextual).unwrap().get(key).unwrap().state,ExpressionTypeStateV2::Known(TypeV1::list(TypeV1::string())));
}

#[test]
fn positive_plus_and_larger_than_minimum_magnitude_reject() {
    for (operator,magnitude) in [("+","170141183460469231731687303715884105728"),("-","170141183460469231731687303715884105729")] {
        let builder=HirBuilderV1::new(); let child=builder.int(magnitude,SpanV1::new(1,40)); let unary=builder.unary(operator,child,SpanV1::new(0,40)); let doc=document(unary);
        let inputs=ResolvedTypeInputsV2::try_new(check_document_v2(&doc).unwrap(),vec![],vec![]).unwrap();
        assert_eq!(ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs).unwrap_err().code(),"types.integer_out_of_range");
    }
}
