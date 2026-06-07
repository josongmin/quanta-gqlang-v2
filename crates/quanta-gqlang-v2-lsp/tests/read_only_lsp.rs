use quanta_gqlang_v2_lsp::handlers::completion::handle_completion;
use quanta_gqlang_v2_lsp::handlers::diagnostics::{handle_diagnostics, DiagnosticItemV1};
use quanta_gqlang_v2_lsp::handlers::hover::{handle_hover, HoverCardV1};
use quanta_gqlang_v2_lsp::handlers::symbols::handle_symbols;

#[test]
fn hover_exposes_root_category_and_typed_target_summary() {
    let hover =
        handle_hover("SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 7")
            .expect("SELECT hover should succeed");

    assert_eq!(
        hover,
        HoverCardV1 {
            root_keyword: "SELECT".to_string(),
            category: "query".to_string(),
            typed_target: "catalog.users".to_string(),
            summary: vec![
                "projection=id, profile.name".to_string(),
                "filter=active = true".to_string(),
                "limit=7".to_string(),
            ],
        }
    );

    let describe =
        handle_hover("DESCRIBE rule::owasp.sql-injection").expect("DESCRIBE hover should succeed");
    assert_eq!(describe.root_keyword, "DESCRIBE");
    assert_eq!(describe.category, "interactive");
    assert_eq!(describe.typed_target, "rule::owasp.sql-injection");

    let use_pack = handle_hover(
        "USE PACK secure-pack ;; activation=workspace ;; lockfile=packs/secure-pack.lock",
    )
    .expect("USE PACK hover should succeed");
    assert_eq!(
        use_pack,
        HoverCardV1 {
            root_keyword: "USE PACK".to_string(),
            category: "meta".to_string(),
            typed_target: "secure-pack".to_string(),
            summary: vec![
                "activation=workspace".to_string(),
                "lockfile=packs/secure-pack.lock".to_string(),
            ],
        }
    );

    let require = handle_hover("REQUIRE CERTIFIED").expect("REQUIRE hover should succeed");
    assert_eq!(
        require,
        HoverCardV1 {
            root_keyword: "REQUIRE CERTIFIED".to_string(),
            category: "meta".to_string(),
            typed_target: "certified-only".to_string(),
            summary: vec![
                "truth=certified-only".to_string(),
                "admission=fail-closed".to_string(),
            ],
        }
    );
}

#[test]
fn diagnostics_separate_syntax_and_sema_classes() {
    let syntax = handle_diagnostics("SELECT id catalog.users")
        .expect("syntax diagnostics should be returned");
    assert_eq!(
        syntax,
        vec![DiagnosticItemV1 {
            class: "syntax".to_string(),
            code: "missing_from_clause".to_string(),
            message: "SELECT root requires a FROM clause".to_string(),
        }]
    );

    let sema = handle_diagnostics("SHOW catalog").expect("sema diagnostics should be returned");
    assert_eq!(
        sema,
        vec![DiagnosticItemV1 {
            class: "sema".to_string(),
            code: "unsupported_show_category".to_string(),
            message: "unsupported SHOW category `catalog`".to_string(),
        }]
    );
}

#[test]
fn completion_uses_roots_and_known_language_nouns() {
    let root_items = handle_completion("SE").expect("root completion");
    assert!(root_items.iter().any(|item| item.label == "SELECT"));
    assert!(root_items.iter().all(|item| item.label.starts_with("SE")));

    let noun_items = handle_completion("pro").expect("noun completion");
    assert!(noun_items.iter().any(|item| item.label == "PROFILE"));
    assert!(noun_items.iter().any(|item| item.label == "profiles"));
    assert!(noun_items.iter().any(|item| item.label == "profile::"));
}

#[test]
fn symbols_return_document_and_root_summary_not_echo() {
    let symbols =
        handle_symbols("PACK secure-pack ;; source=registry::security ;; activation=session")
            .expect("PACK symbols should succeed");

    assert_eq!(symbols[0].name, "document");
    assert_eq!(symbols[0].kind, "document");
    assert_eq!(symbols[1].name, "PACK");
    assert_eq!(symbols[1].kind, "root");
    assert_eq!(symbols[2].name, "secure-pack");
    assert_eq!(symbols[2].kind, "name");
}

#[test]
fn malformed_input_fails_closed() {
    let hover_error = handle_hover("FIND unknown src -> sink").expect_err("invalid FIND");
    assert_eq!(
        hover_error,
        "class=sema code=unknown_find_subkind message=unknown FIND subkind `unknown`"
    );

    let symbols_error =
        handle_symbols("DESCRIBE model::").expect_err("invalid DESCRIBE should fail closed");
    assert_eq!(
        symbols_error,
        "class=syntax code=invalid_describe_path message=DESCRIBE target path cannot be empty"
    );

    let use_pack_diag = handle_diagnostics(
        "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.txt",
    )
    .expect("USE PACK diagnostics should be returned");
    assert_eq!(
        use_pack_diag,
        vec![DiagnosticItemV1 {
            class: "syntax".to_string(),
            code: "use_pack_invalid_lockfile".to_string(),
            message: "USE PACK lockfile `packs/secure-pack.txt` must end with `.lock` and include a non-empty file stem".to_string(),
        }]
    );

    let require_error = handle_hover("REQUIRE CERTIFIED ;; truth=maybe")
        .expect_err("REQUIRE with extra fields should fail closed");
    assert_eq!(
        require_error,
        "class=syntax code=unsupported_require_meta message=unsupported REQUIRE target `CERTIFIED ;; truth=maybe`"
    );
}
