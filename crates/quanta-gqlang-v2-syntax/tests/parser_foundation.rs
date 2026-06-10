#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "foundation acceptance asserts validated parses and exercises fail-closed rejects"
)]

//! F10 proof rail (jun-10-gqlnag-v2-followup) — E01 parser-foundation named
//! seams (`--test parser_foundation`).
//!
//! NOTE TO OTHER AGENTS: this file pins the E01 foundation contract. Invariants
//! that must never regress:
//!   1. `RootKindV1` is the typed root-discriminant of the parser. Every
//!      `AstRootV1` variant maps to exactly one `RootKindV1` — the parser is
//!      root-discriminated from day one, never a single-query/JSON fallback.
//!   2. `ParseErrorV1` is a TYPED view over the existing diagnostic channel
//!      (`SyntaxDiagnosticV1{code,message}`); it does NOT invent a second error
//!      channel. A parse reject must carry a stable `code`, not just prose.
//!   3. Rejects fail closed: an unsupported root keyword and an empty document
//!      both produce a typed `ParseErrorV1`, never a silent empty/default doc.
//!
//! Do not relax these into positive-only checks.

use quanta_gqlang_v2_syntax::error::ParseErrorV1;
use quanta_gqlang_v2_syntax::parse_document_v1;
use quanta_gqlang_v2_syntax::root::RootKindV1;

fn root_kinds_v1(input: &str) -> Vec<RootKindV1> {
    let doc = parse_document_v1(input).expect("parse");
    doc.roots.iter().map(RootKindV1::of_root_v1).collect()
}

/// Assert the first typed parse error of a fail-closed rejection. The
/// diagnostic page owns the storage and `ParseErrorV1` borrows it, so the page
/// must stay alive for the assertion — hence a closure rather than a returned
/// borrow (which would dangle).
fn assert_first_parse_error_v1(input: &str, assert: impl FnOnce(ParseErrorV1<'_>)) {
    let page = parse_document_v1(input).expect_err("expected a fail-closed parse rejection");
    let error = ParseErrorV1::from_page_v1(&page)
        .into_iter()
        .next()
        .expect("a rejection must carry at least one typed ParseErrorV1");
    assert(error);
}

#[test]
fn parser_is_root_discriminated_across_families_v1() {
    assert_eq!(
        root_kinds_v1("SELECT name FROM catalog.users"),
        vec![RootKindV1::Select]
    );
    assert_eq!(
        root_kinds_v1("FIND taint api.entry -> db.write"),
        vec![RootKindV1::Find]
    );
    assert_eq!(
        root_kinds_v1("USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure.lock"),
        vec![RootKindV1::Meta]
    );
}

#[test]
fn root_kind_mirrors_every_ast_root_family_v1() {
    // Non-vacuous coverage guard: the 17 canonical root families plus the two
    // composed/meta variants must each round-trip keyword -> kind. If a new
    // `AstRootV1` variant is added without a `RootKindV1` mapping, `of_root_v1`
    // stops compiling, so this list cannot silently drift.
    let keyword_to_kind = [
        ("SELECT name FROM catalog.users", RootKindV1::Select),
        ("FIND taint api.entry -> db.write", RootKindV1::Find),
        (
            "CHECK rule.x check:v1|class=a|scope=repo%3A%2F%2Fx|pattern=~|confirm=skip",
            RootKindV1::Check,
        ),
        ("EXPLAIN rule::owasp.sql ;; truth=confirm", RootKindV1::Explain),
        ("SHOW capabilities", RootKindV1::Show),
        ("DESCRIBE pack::secure", RootKindV1::Describe),
        (
            "SAMPLE query=auth ;; mode=hybrid ;; limit=5",
            RootKindV1::Sample,
        ),
        (
            "COMPARE baseline=run::a ;; candidate=run::b ;; metric=exact",
            RootKindV1::Compare,
        ),
        (
            "EXPORT source=result::r ;; kind=json",
            RootKindV1::Export,
        ),
        (
            "VIEW catalog.users ;; query-ref=queries/catalog/users",
            RootKindV1::View,
        ),
        ("RULE shell.exec ;; check-ref=checks/shell/exec", RootKindV1::Rule),
        (
            "MODEL embeddings.code ;; provider=openai ;; source=models/code-search",
            RootKindV1::Model,
        ),
        (
            "PACK security-default ;; source=registry://packs/security-default ;; activation=workspace",
            RootKindV1::Pack,
        ),
        (
            "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only",
            RootKindV1::Profile,
        ),
        (
            "GATE release-gate ;; checks=owasp.sql,authz.path ;; binding=all",
            RootKindV1::Gate,
        ),
        ("REQUIRE CERTIFIED", RootKindV1::Meta),
    ];

    for (source, expected) in keyword_to_kind {
        let kinds = root_kinds_v1(source);
        assert_eq!(kinds, vec![expected], "root kind mismatch for `{source}`");
    }

    // Composition is the one root with no leading keyword of its own.
    let composed =
        root_kinds_v1("LET base = SELECT name FROM catalog.users\nSHOW capabilities WITH base");
    assert_eq!(composed, vec![RootKindV1::Composition]);
}

#[test]
fn unsupported_root_keyword_fails_closed_with_typed_error_v1() {
    assert_first_parse_error_v1("FROBNICATE everything", |error| {
        assert_eq!(error.code(), "unsupported_root");
        assert!(
            error.message().contains("unsupported root keyword"),
            "typed error must name the unsupported keyword, got: {}",
            error.message()
        );
    });
}

#[test]
fn empty_document_fails_closed_with_typed_error_v1() {
    // An empty document must not decode to an empty/default DocumentAstV1; it is
    // a typed reject so callers cannot silently proceed on nothing.
    assert_first_parse_error_v1("   \n  \n", |error| {
        assert_eq!(error.code(), "empty_document");
    });
}

#[test]
fn parse_error_view_preserves_the_diagnostic_channel_codes_v1() {
    // ParseErrorV1 must be a faithful view of the underlying diagnostic page —
    // no swallowed diagnostics, no invented second channel.
    let page = parse_document_v1("LET a = SELECT x FROM c.t").expect_err("dangling LET");
    let typed = ParseErrorV1::from_page_v1(&page);
    assert_eq!(typed.len(), page.diagnostics.len());
    for (error, diagnostic) in typed.iter().zip(&page.diagnostics) {
        assert_eq!(error.code(), diagnostic.code);
        assert_eq!(error.message(), diagnostic.message);
    }
    assert!(typed
        .iter()
        .any(|error| error.code() == "composition_unconsumed_let"));
}
