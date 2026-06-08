#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

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
fn select_with_limit_before_where_is_a_clean_error_not_a_panic() {
    // Regression: the hand-rolled WHERE/LIMIT splitter computed `remainder[14..0]`
    // for `LIMIT` appearing before `WHERE` and panicked the LSP front door.
    let result = handle_hover("SELECT a FROM t LIMIT 5 WHERE x = 1");
    assert!(
        result.is_err(),
        "LIMIT-before-WHERE must be a clean diagnostic, got {result:?}"
    );
}

#[test]
fn select_where_value_containing_limit_keyword_is_not_misparsed() {
    // Regression: `LIMIT ` inside a quoted WHERE value used to be mistaken for the
    // LIMIT clause and produced a spurious numeric diagnostic on a valid query.
    let hover = handle_hover("SELECT id FROM catalog.users WHERE name = 'LIMIT x'")
        .expect("quoted LIMIT inside a WHERE value must parse");
    assert_eq!(hover.typed_target, "catalog.users");
    assert!(
        hover
            .summary
            .contains(&"filter=name = 'LIMIT x'".to_string()),
        "got {:?}",
        hover.summary
    );
}

#[test]
fn multi_statement_document_does_not_produce_garbage() {
    // Regression: the parser became multi-statement; the LSP read-only surface must
    // split on the same boundaries instead of mangling two statements into one root
    // (which used to embed a newline in the SHOW category / SELECT table).
    let hover = handle_hover("SHOW models\nSHOW views").expect("first statement hovers cleanly");
    assert_eq!(hover.root_keyword, "SHOW");
    assert_eq!(hover.typed_target, "models");

    let clean = handle_diagnostics("SHOW models\nSHOW views").expect("diagnostics");
    assert!(
        clean.is_empty(),
        "valid 2-statement document must have no diagnostics, got {clean:?}"
    );

    // Per-statement error recovery: a malformed second statement is still reported.
    let mixed = handle_diagnostics("SHOW capabilities\nSHOW bogus").expect("diagnostics");
    assert!(
        mixed
            .iter()
            .any(|item| item.code == "unsupported_show_category"),
        "the malformed second statement must be diagnosed, got {mixed:?}"
    );
}

#[test]
fn view_body_clause_matches_the_canonical_parser() {
    // Regression: hover/diagnostics read `query=` while the canonical grammar uses
    // `body=`, so a valid `body=` VIEW drew a false diagnostic and an invalid
    // `query=` VIEW drew none. Align the read-only surface with the parser.
    let valid = "VIEW catalog.users ;; body=SELECT id FROM catalog.users LIMIT 25";
    assert!(
        handle_diagnostics(valid).expect("diagnostics").is_empty(),
        "a valid body= VIEW must not be flagged"
    );
    assert_eq!(
        handle_hover(valid).expect("body= VIEW hovers").root_keyword,
        "VIEW"
    );
    assert!(
        !handle_diagnostics("VIEW catalog.users ;; query=SELECT id FROM catalog.users")
            .expect("diagnostics")
            .is_empty(),
        "an unsupported query= VIEW clause must be flagged"
    );
    // A malformed body= value (not a valid SELECT) must also draw a diagnostic,
    // matching the canonical decoder rather than a false-clean editor advisory.
    assert!(
        !handle_diagnostics("VIEW catalog.users ;; body=SELECT id catalog.users")
            .expect("diagnostics")
            .is_empty(),
        "a body= that is not a valid SELECT must be flagged"
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
            code: "expected_from_keyword".to_string(),
            message: "SELECT projection must be followed by FROM".to_string(),
        }]
    );

    let sema = handle_diagnostics("SHOW catalog").expect("sema diagnostics should be returned");
    assert_eq!(
        sema,
        vec![DiagnosticItemV1 {
            class: "sema".to_string(),
            code: "unsupported_show_category".to_string(),
            message: "SHOW category `catalog` is unsupported; expected one of capabilities, models, views, rules, packs, profiles, gates".to_string(),
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
        "class=sema code=unsupported_find_subkind message=unsupported FIND subkind `unknown`"
    );

    let symbols_error =
        handle_symbols("DESCRIBE model::").expect_err("invalid DESCRIBE should fail closed");
    assert_eq!(
        symbols_error,
        "class=syntax code=describe_target_invalid_shape message=DESCRIBE target must use `<kind>::<identifier>`"
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
