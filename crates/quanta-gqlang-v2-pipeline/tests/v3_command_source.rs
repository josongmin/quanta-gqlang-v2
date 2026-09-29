#![expect(
    clippy::expect_used,
    reason = "source-to-contract regression assertions"
)]

use quanta_gqlang_v2_pipeline::{
    classify_single_source_root_v3, compile_source_document_v3, format_source_document_v2,
    ClosedFrontendAuthorityBuilderV2, ExecutionCommandPhaseV3, ExecutionEffectV3,
    ExecutionFamilyV3, FrontendAuthorityLocationV2, FrontendAuthoritySourceV2, FrontendBudgetV2,
    FrontendLanguageVersionV2, FrontendSourceRootFamilyV3, FrontendSymbolTargetInputV3,
    FrontendTransactionErrorV2, SourceIdV2,
};

const HOST: SourceIdV2 = SourceIdV2::from_raw(1);
const DOC: SourceIdV2 = SourceIdV2::from_raw(9);

fn authority() -> quanta_gqlang_v2_pipeline::ClosedFrontendAuthorityV2 {
    authority_with_identity("def:17")
}

fn authority_with_identity(
    symbol_id: &str,
) -> quanta_gqlang_v2_pipeline::ClosedFrontendAuthorityV2 {
    authority_with_target("src/a.py".to_owned(), symbol_id.to_owned())
}

fn authority_with_target(
    repo_path: String,
    symbol_id: String,
) -> quanta_gqlang_v2_pipeline::ClosedFrontendAuthorityV2 {
    let location = FrontendAuthorityLocationV2 {
        source_id: HOST,
        start: 0,
        end: 1,
    };
    let mut builder = ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST,
            source_len: 64,
        }],
        "app".to_owned(),
        location,
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("builder");
    builder.add_symbol_target(FrontendSymbolTargetInputV3 {
        selector: "symbol.one".to_owned(),
        repo_path,
        symbol_id,
        start_byte: 17,
        end_byte: 20,
        location,
    });
    builder.finish().expect("closed symbol authority")
}

#[test]
fn target_authority_changes_contract_but_not_source_and_duplicate_rejects() {
    let source = "rename symbol.one to \"next\" | phase=preview | snapshot=\"snap-7\"";
    let first =
        compile_source_document_v3(DOC, source, &authority_with_identity("def:17")).expect("first");
    let second = compile_source_document_v3(DOC, source, &authority_with_identity("def:18"))
        .expect("second");
    assert_eq!(
        first.execution().source_digest(),
        second.execution().source_digest()
    );
    assert_ne!(
        first.execution().contract_digest(),
        second.execution().contract_digest()
    );
    assert_ne!(
        first.execution().qualification_digest(),
        second.execution().qualification_digest()
    );

    let location = FrontendAuthorityLocationV2 {
        source_id: HOST,
        start: 0,
        end: 1,
    };
    let mut builder = ClosedFrontendAuthorityBuilderV2::try_new(
        FrontendBudgetV2::standard_v2().expect("budget"),
        vec![FrontendAuthoritySourceV2 {
            source_id: HOST,
            source_len: 64,
        }],
        "app".to_owned(),
        location,
        FrontendLanguageVersionV2 {
            major: 1,
            minor: 0,
            patch: 0,
        },
    )
    .expect("builder");
    let input = FrontendSymbolTargetInputV3 {
        selector: "symbol.one".to_owned(),
        repo_path: "src/a.py".to_owned(),
        symbol_id: "def:17".to_owned(),
        start_byte: 17,
        end_byte: 20,
        location,
    };
    builder.add_symbol_target(input.clone());
    builder.add_symbol_target(input);
    assert_eq!(
        builder.finish().expect_err("duplicate selector").code(),
        "sema.duplicate_symbol_target"
    );
}

#[test]
fn repeated_rename_targets_share_large_immutable_identity_storage() {
    let repeated = "rename symbol.one to \"new_name\" | phase=preview | snapshot=\"snap-7\"";
    let source = format!("{repeated}; {repeated}");
    let large_identity = "x".repeat(32 * 1024);
    let authority = authority_with_target(
        format!("src/{large_identity}"),
        format!("def:{large_identity}"),
    );
    let document = compile_source_document_v3(DOC, &source, &authority)
        .expect("repeated exact target compiles");
    let items = document.execution().items();
    assert_eq!(items.len(), 2);
    let mut items = items.iter();
    let first = items
        .next()
        .expect("first command item")
        .command()
        .expect("first command payload")
        .target();
    let second = items
        .next()
        .expect("second command item")
        .command()
        .expect("second command payload")
        .target();
    assert!(std::ptr::eq(
        first.repo_path().as_ptr(),
        second.repo_path().as_ptr()
    ));
    assert!(std::ptr::eq(
        first.symbol_id().as_ptr(),
        second.symbol_id().as_ptr()
    ));
    assert!(std::ptr::eq(
        first.selector().as_ptr(),
        second.selector().as_ptr()
    ));
}

#[test]
fn canonical_rename_format_is_idempotent_and_reordered_clauses_reject() {
    let budget = FrontendBudgetV2::standard_v2().expect("budget");
    let source = "rename symbol.one to \"name\\\"x\" | phase=apply | snapshot=\"snap-7\" | preview_token=\"issued-1\"";
    let formatted = format_source_document_v2(DOC, source, &budget).expect("format");
    assert_eq!(formatted, source);
    assert_eq!(
        format_source_document_v2(DOC, &formatted, &budget).expect("reformat"),
        formatted
    );
    assert!(compile_source_document_v3(
        DOC,
        "rename symbol.one to \"next\" | snapshot=\"snap-7\" | phase=preview",
        &authority()
    )
    .is_err());
}

#[test]
fn preview_and_apply_are_exact_target_source_bound_and_phase_separated() {
    let authority = authority();
    let preview = "rename symbol.one to \"new_name\" | phase=preview | snapshot=\"snap-7\"";
    assert_eq!(
        classify_single_source_root_v3(
            DOC,
            preview,
            &FrontendBudgetV2::standard_v2().expect("budget")
        )
        .expect("classify"),
        FrontendSourceRootFamilyV3::Command
    );
    let preview_doc =
        compile_source_document_v3(DOC, preview, &authority).expect("preview compiles");
    let item = preview_doc
        .execution()
        .items()
        .first()
        .expect("one preview item");
    assert_eq!(item.family(), ExecutionFamilyV3::Command);
    assert_eq!(item.plan().effect(), ExecutionEffectV3::ReadOnly);
    let command = item.command().expect("typed command projection");
    assert_eq!(command.phase(), ExecutionCommandPhaseV3::Preview);
    assert_eq!(command.target().repo_path(), "src/a.py");
    assert_eq!(command.target().symbol_id(), "def:17");
    assert_eq!(
        (command.target().start_byte(), command.target().end_byte()),
        (17, 20)
    );
    assert_eq!(command.new_name(), "new_name");
    assert_eq!(command.expected_snapshot(), "snap-7");
    assert_eq!(command.preview_token(), None);

    let apply = "rename symbol.one to \"new_name\" | phase=apply | snapshot=\"snap-7\" | preview_token=\"issued-1\"";
    let apply_doc =
        compile_source_document_v3(DOC, apply, &authority).expect("apply intent compiles");
    let apply_item = apply_doc
        .execution()
        .items()
        .first()
        .expect("one apply item");
    assert_eq!(
        apply_item.plan().effect(),
        ExecutionEffectV3::MutationRequiresGrant
    );
    assert_eq!(
        apply_item.command().expect("command").phase(),
        ExecutionCommandPhaseV3::Apply
    );
    assert_eq!(
        apply_item.command().expect("command").preview_token(),
        Some("issued-1")
    );
    assert_ne!(
        preview_doc.execution().qualification_digest(),
        apply_doc.execution().qualification_digest()
    );
    assert_eq!(apply_doc.execution().contract_digest_schema_version(), 8);
}

#[test]
fn absent_symbol_and_invalid_phase_token_combinations_reject() {
    let authority = authority();
    let missing = compile_source_document_v3(
        DOC,
        "rename other to \"bar\" | phase=preview | snapshot=\"s\"",
        &authority,
    )
    .expect_err("unknown target cannot compile");
    assert!(matches!(missing, FrontendTransactionErrorV2::Document(_)));
    for source in [
        "rename symbol.one to \"bar\" | phase=apply | snapshot=\"s\"",
        "rename symbol.one to \"bar\" | phase=preview | snapshot=\"s\" | preview_token=\"x\"",
        "rename symbol.one to \"bar\" | phase=preview",
        "rename symbol.one to \"bar\" | phase=preview | snapshot=\"s\" | operation=delete",
    ] {
        assert!(
            compile_source_document_v3(DOC, source, &authority).is_err(),
            "must reject {source}"
        );
    }
}
