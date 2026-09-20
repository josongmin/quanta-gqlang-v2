#![allow(clippy::unwrap_used, reason = "owner assertions")]

use quanta_gqlang_v2_hir::{check_document_v2, CallArgHirV1, CallArgsHirV1, DeclHirV1, HirBuilderV1, HirDocumentV1, HirNodeIdV1, ProgramItemHirV1, SourceIdV2, SpanV1};
use quanta_gqlang_v2_types::*;

fn checked_document(body: quanta_gqlang_v2_hir::ValueExprHirV1) -> HirDocumentV1 {
    HirDocumentV1 { source_id: SourceIdV2::from_raw(9), source_len: 100, items: vec![ProgramItemHirV1::Decl(Box::new(DeclHirV1::new(HirNodeIdV1::from_raw(10_000), SpanV1::new(0, 90), "x".into(), SpanV1::new(0, 1), body)))] }
}

#[test]
fn resolved_inputs_reject_missing_duplicate_foreign_and_extra_entries() {
    let builder = HirBuilderV1::new();
    let symbol = builder.symbol("name", SpanV1::new(1, 5));
    let key = SourceQualifiedNodeIdV2 { source_id: SourceIdV2::from_raw(9), node_id: symbol.id };
    let doc = checked_document(symbol);
    let checked = check_document_v2(&doc).unwrap();
    assert_eq!(ResolvedTypeInputsV2::try_new(checked, vec![], vec![]).unwrap_err().code(), "types.missing_resolved_input");
    let entry = ResolvedTypeInputEntryV2 { key, value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(TypeV1::string())) };
    assert_eq!(ResolvedTypeInputsV2::try_new(checked, vec![entry.clone(), entry], vec![]).unwrap_err().code(), "types.duplicate_resolved_input");
    let foreign = ResolvedTypeInputEntryV2 { key: SourceQualifiedNodeIdV2 { source_id: SourceIdV2::from_raw(10), node_id: key.node_id }, value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(TypeV1::string())) };
    assert_eq!(ResolvedTypeInputsV2::try_new(checked, vec![foreign], vec![]).unwrap_err().code(), "types.foreign_document_input");
}

#[test]
fn kernel_selects_stable_operator_and_builtin_ids() {
    let builder = HirBuilderV1::new();
    let lhs = builder.symbol("a", SpanV1::new(0, 1));
    let lhs_id = lhs.id;
    let rhs = builder.int("1", SpanV1::new(4, 5));
    let sum = builder.binary("+", lhs, rhs, SpanV1::new(0, 5));
    let sum_key = SourceQualifiedNodeIdV2 { source_id: SourceIdV2::from_raw(9), node_id: sum.id };
    let doc = checked_document(sum);
    let entry = ResolvedTypeInputEntryV2 { key: SourceQualifiedNodeIdV2 { source_id: SourceIdV2::from_raw(9), node_id: lhs_id }, value: ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(TypeV1::int())) };
    let inputs = ResolvedTypeInputsV2::try_new(check_document_v2(&doc).unwrap(), vec![entry], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs).unwrap();
    assert!(facts.get(sum_key).unwrap().operator_id.is_some());

    let builder = HirBuilderV1::new();
    let arg = builder.int("1", SpanV1::new(6, 7));
    let call = builder.call("count", SpanV1::new(0, 5), CallArgsHirV1::try_new(vec![CallArgHirV1::Expr(arg)]).unwrap(), SpanV1::new(0, 8));
    let key = SourceQualifiedNodeIdV2 { source_id: SourceIdV2::from_raw(9), node_id: call.id };
    let doc = checked_document(call);
    let registry = BuiltinRegistryV2::try_closed().unwrap();
    let input = ResolvedTypeInputEntryV2 { key, value: ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(registry.family_id("count").unwrap())) };
    let inputs = ResolvedTypeInputsV2::try_new(check_document_v2(&doc).unwrap(), vec![input], vec![]).unwrap();
    let facts = ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs).unwrap();
    assert!(matches!(facts.get(key).unwrap().resolved_call(), Some(ResolvedCallTargetV2::Builtin(_))));
}

#[test]
fn resolved_inputs_reject_extra_and_wrong_kind_before_kernel_walk() {
    let builder=HirBuilderV1::new();let symbol=builder.symbol("x",SpanV1::new(0,1));let symbol_id=symbol.id;let doc=checked_document(symbol);let checked=check_document_v2(&doc).unwrap();
    let registry=BuiltinRegistryV2::try_closed().unwrap();
    let wrong=ResolvedTypeInputEntryV2{key:SourceQualifiedNodeIdV2{source_id:SourceIdV2::from_raw(9),node_id:symbol_id},value:ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(registry.family_id("count").unwrap()))};
    assert_eq!(ResolvedTypeInputsV2::try_new(checked,vec![wrong],vec![]).unwrap_err().code(),"types.wrong_resolved_input_kind");
    let extra=ResolvedTypeInputEntryV2{key:SourceQualifiedNodeIdV2{source_id:SourceIdV2::from_raw(9),node_id:HirNodeIdV1::from_raw(777)},value:ResolvedTypeInputValueV2::Binding(ExpressionTypeStateV2::Known(TypeV1::int()))};
    assert_eq!(ResolvedTypeInputsV2::try_new(checked,vec![extra],vec![]).unwrap_err().code(),"types.extra_resolved_input");
}

#[test]
fn star_is_accepted_only_for_explicit_count_star_signature() {
    let registry=BuiltinRegistryV2::try_closed().unwrap();
    for (name,expected_ok) in [("count",true),("starts_with",false)] {
        let builder=HirBuilderV1::new();let call=builder.call(name,SpanV1::new(0,name.len()),CallArgsHirV1::try_new(vec![CallArgHirV1::star(SpanV1::new(name.len()+1,name.len()+2))]).unwrap(),SpanV1::new(0,name.len()+3));let key=SourceQualifiedNodeIdV2{source_id:SourceIdV2::from_raw(9),node_id:call.id};let doc=checked_document(call);
        let input=ResolvedTypeInputEntryV2{key,value:ResolvedTypeInputValueV2::Callable(ResolvedCallableInputV2::Builtin(registry.family_id(name).unwrap()))};let inputs=ResolvedTypeInputsV2::try_new(check_document_v2(&doc).unwrap(),vec![input],vec![]).unwrap();let result=ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs);
        if expected_ok { assert!(matches!(result.unwrap().get(key).unwrap().resolved_call(),Some(ResolvedCallTargetV2::Builtin(_)))); } else { assert_eq!(result.unwrap_err().code(),"types.invalid_star_argument"); }
    }
}

#[test]
fn unknown_and_bad_operator_reject_distinctly() {
    for (operator,code) in [("~","types.unknown_operator"),("not","types.bad_overload")] {
        let builder=HirBuilderV1::new();let operand=builder.string("x",SpanV1::new(1,4));let unary=builder.unary(operator,operand,SpanV1::new(0,4));let doc=checked_document(unary);let inputs=ResolvedTypeInputsV2::try_new(check_document_v2(&doc).unwrap(),vec![],vec![]).unwrap();assert_eq!(ExpressionTypeKernelV2::try_closed().unwrap().check_document(&inputs).unwrap_err().code(),code);
    }
}
