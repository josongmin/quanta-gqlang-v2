#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::interactive::meta::MetaCommandAstV1;
use quanta_gqlang_v2_syntax::{parse_document_v1, SyntaxDiagnosticV1};

fn parse_root_v1(input: &str) -> AstRootV1 {
    parse_document_v1(input)
        .unwrap_or_else(|page| panic!("expected parse success for `{input}`: {page:?}"))
        .roots
        .into_iter()
        .next()
        .expect("one root")
}

fn parse_error_v1(input: &str) -> SyntaxDiagnosticV1 {
    parse_document_v1(input)
        .expect_err("expected parse failure")
        .diagnostics
        .into_iter()
        .next()
        .expect("one diagnostic")
}

#[test]
fn shipped_roots_parse_without_fallback_v1() {
    assert!(matches!(
        parse_root_v1("SELECT id, profile.name FROM catalog.users WHERE active = true LIMIT 7"),
        AstRootV1::Select(_)
    ));
    assert!(matches!(
        parse_root_v1("FIND taint api.entry -> db.write | sanitizers=authz.clean | depth=3"),
        AstRootV1::Find(_)
    ));
    assert!(matches!(
        parse_root_v1(
            "CHECK rule.hybrid check:v1|class=security|scope=repo%3A%2F%2Fruntime|pattern=call%20%3D%20%22std%3A%3Aprocess%3A%3ACommand%22|confirm=require"
        ),
        AstRootV1::Check(_)
    ));
    assert!(matches!(
        parse_root_v1("EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm"),
        AstRootV1::Explain(_)
    ));
    assert!(matches!(
        parse_root_v1("SHOW capabilities"),
        AstRootV1::Show(_)
    ));
    assert!(matches!(
        parse_root_v1("DESCRIBE rule::owasp.sql-injection"),
        AstRootV1::Describe(_)
    ));
    assert!(matches!(
        parse_root_v1(
            "SAMPLE query=callgraph entrypoints ;; scope=repo::frontend ;; mode=semantic ;; limit=5"
        ),
        AstRootV1::Sample(_)
    ));
    assert!(matches!(
        parse_root_v1(
            "COMPARE baseline=run::baseline-1 ;; candidate=run::candidate-2 ;; metric=semantic"
        ),
        AstRootV1::Compare(_)
    ));
    assert!(matches!(
        parse_root_v1("EXPORT source=result::latest ;; kind=sarif ;; profile=strict"),
        AstRootV1::Export(_)
    ));
    assert!(matches!(
        parse_root_v1("VIEW catalog.users ;; query-ref=queries/catalog/users"),
        AstRootV1::View(_)
    ));
    assert!(matches!(
        parse_root_v1("RULE shell.exec ;; check-ref=checks/security/shell-exec"),
        AstRootV1::Rule(_)
    ));
    assert!(matches!(
        parse_root_v1("MODEL embeddings.code ;; provider=openai ;; source=models/code-search"),
        AstRootV1::Model(_)
    ));
    assert!(matches!(
        parse_root_v1("PACK secure-pack ;; source=registry::security ;; activation=workspace"),
        AstRootV1::Pack(_)
    ));
    assert!(matches!(
        parse_root_v1(
            "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only"
        ),
        AstRootV1::Profile(_)
    ));
    assert!(matches!(
        parse_root_v1("GATE release-gate ;; checks=owasp.sql,authz.path ;; binding=all"),
        AstRootV1::Gate(_)
    ));
    assert!(matches!(
        parse_root_v1(
            "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock"
        ),
        AstRootV1::Meta(MetaCommandAstV1::UsePack { .. })
    ));
    assert!(matches!(
        parse_root_v1("REQUIRE CERTIFIED"),
        AstRootV1::Meta(MetaCommandAstV1::RequireCertified)
    ));
}

#[test]
fn malformed_inputs_fail_closed_v1() {
    let empty = parse_error_v1("");
    assert_eq!(empty.code, "empty_document");

    let check_missing_scope = parse_error_v1("CHECK rule.hybrid");
    assert_eq!(check_missing_scope.code, "check_missing_rule_or_scope");

    let describe_missing_target = parse_error_v1("DESCRIBE");
    assert_eq!(describe_missing_target.code, "describe_missing_target");

    let explain_invalid_truth = parse_error_v1("EXPLAIN rule::owasp.sql-injection ;; truth=maybe");
    assert_eq!(explain_invalid_truth.code, "invalid_explain_shape");
    assert!(explain_invalid_truth.message.contains("unsupported"));
}

#[test]
fn duplicate_and_unsupported_clauses_fail_closed_v1() {
    let duplicate_find = parse_error_v1("FIND taint api.entry -> db.write | depth=3 | depth=5");
    assert_eq!(duplicate_find.code, "invalid_find_seed");
    assert!(duplicate_find.message.contains("duplicated"));

    let duplicate_sample = parse_error_v1("SAMPLE query=callgraph ;; mode=semantic ;; mode=hybrid");
    assert_eq!(duplicate_sample.code, "invalid_sample_payload");
    assert!(duplicate_sample.message.contains("duplicated"));

    let unsupported_export =
        parse_error_v1("EXPORT source=result::latest ;; kind=sarif ;; format=pretty");
    assert_eq!(unsupported_export.code, "invalid_export_payload");
    assert!(unsupported_export.message.contains("unsupported"));
}

#[test]
fn parser_has_no_unknown_root_or_meta_fallback_v1() {
    let unknown_root = parse_error_v1("ANALYZE graph");
    assert_eq!(unknown_root.code, "unsupported_root");

    let unsupported_use = parse_error_v1("USE PROFILE strict-review");
    assert_eq!(unsupported_use.code, "unsupported_use_meta");

    let unsupported_require = parse_error_v1("REQUIRE TRUSTED");
    assert_eq!(unsupported_require.code, "unsupported_require_meta");

    let missing_lockfile = parse_error_v1("USE PACK secure-pack ;; activation=session");
    assert_eq!(missing_lockfile.code, "use_pack_missing_lockfile");

    let invalid_activation = parse_error_v1(
        "USE PACK secure-pack ;; activation=manual ;; lockfile=packs/secure-pack.lock",
    );
    assert_eq!(invalid_activation.code, "use_pack_invalid_activation");

    let unsupported_clause = parse_error_v1(
        "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock ;; source=ambient",
    );
    assert_eq!(unsupported_clause.code, "unsupported_use_pack_clause");

    let invalid_lockfile_suffix = parse_error_v1(
        "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.txt",
    );
    assert_eq!(invalid_lockfile_suffix.code, "use_pack_invalid_lockfile");
}
