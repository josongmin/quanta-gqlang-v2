#![expect(clippy::unwrap_used, reason = "owner assertions inspect exact closed semantic facts")]

use quanta_gqlang_v2_hir::{
    HirBuilderV1, HirDocumentV1, LocatedSpanV2, ProgramItemHirV1,
    SourceIdV2, SourceRegistrationV2, SourceRegistryViewV2, SpanV1,
};
use quanta_gqlang_v2_modules::{
    CurrentModuleSelectionInputV2, ModuleAuthorityBuilderV2, ModuleIdV2,
    ModuleNodeInputV2, VersionV2,
};
use quanta_gqlang_v2_sema::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    FrontendBudgetV2, GraphReferenceCatalogV2, SemanticItemV2, TypedBoolExprV2,
};
use quanta_gqlang_v2_types::RelationCatalogV2;

fn authority() -> ClosedFrontendAuthorityV2 {
    let host = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
        source_id: SourceIdV2::from_raw(1), source_len: 4,
    }]).unwrap();
    let checked = host.try_check_location(LocatedSpanV2 {
        source_id: SourceIdV2::from_raw(1), span: SpanV1::new(0, 1),
    }).unwrap();
    let module = ModuleIdV2::try_new("app", checked.clone()).unwrap();
    let budget = FrontendBudgetV2::standard_v2().unwrap();
    let profile = budget.authority_profile().unwrap();
    let mut builder = ModuleAuthorityBuilderV2::try_new(profile.clone(), host.clone()).unwrap();
    builder.try_add_module(ModuleNodeInputV2 { module_id: module.clone(), location: checked.clone() }).unwrap();
    let modules = builder.finish(CurrentModuleSelectionInputV2 { module_id: module, location: checked }, VersionV2::new(1, 0, 0)).unwrap();
    let relations = RelationCatalogV2::try_new(Vec::new(), &host, profile.clone()).unwrap();
    let graph = GraphReferenceCatalogV2::try_new(Vec::new(), &host, profile).unwrap();
    ClosedFrontendAuthorityV2::try_new(modules, relations, graph, budget).unwrap()
}

#[test]
fn closed_document_type_fact_is_the_only_boolean_proof_source() {
    let authority = authority();
    for (source_raw, is_bool) in [(41, true), (42, false)] {
        let builder = HirBuilderV1::new();
        let body = if is_bool { builder.bool_lit(true, SpanV1::new(8, 12)) } else { builder.int("1", SpanV1::new(8, 9)) };
        let location = LocatedSpanV2 { source_id: SourceIdV2::from_raw(source_raw), span: body.span };
        let decl = builder.decl("answer", SpanV1::new(0, 6), body, SpanV1::new(0, 12));
        let semantic = resolve_checked_document_v2(CheckedSyntaxDocumentV2::try_from_hir(HirDocumentV1 {
            source_id: location.source_id, source_len: 12,
            items: vec![ProgramItemHirV1::Decl(Box::new(decl))],
        }).unwrap(), &authority).unwrap();
        let SemanticItemV2::Declaration(typed) = semantic.items().next().unwrap().1 else { panic!("typed declaration expected") };
        assert_eq!(typed.body().location().location(), location);
        if is_bool {
            assert!(TypedBoolExprV2::try_new(typed.body().clone(), location).is_ok());
        } else {
            let error = TypedBoolExprV2::try_new(typed.body().clone(), location).unwrap_err();
            assert_eq!(error.diagnostics()[0].code(), "sema.non_boolean_predicate");
        }
    }
}
