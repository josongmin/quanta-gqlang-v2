//! Test-only wiring checks for the syntax surface. Kept out of `lib.rs` so the
//! production root stays free of inline `#[cfg(test)]` content.

#![expect(
    clippy::expect_used,
    reason = "unit tests assert parser/tokenizer success on validated fixtures"
)]
#![expect(
    clippy::indexing_slicing,
    reason = "wiring tests assert the single dispatched root directly"
)]

use crate::interactive::meta::MetaCommandAstV1;
use crate::roots::find::FindSubkindV1;
use crate::{parse_document_v1, tokenize_v1, AstRootV1, CRATE_ID_V1};

#[test]
fn crate_id_is_stable() {
    assert_eq!(CRATE_ID_V1, "quanta-gqlang-v2-syntax");
}

#[test]
fn syntax_surface_is_wired() {
    let tokens = tokenize_v1("SELECT id FROM catalog.users").expect("tokenize");
    assert!(!tokens.is_empty());
    let doc = parse_document_v1("SELECT id FROM catalog.users").expect("parse");
    assert_eq!(doc.roots.len(), 1);
}

#[test]
fn parser_dispatches_core_roots_v1() {
    let select = parse_document_v1("SELECT id FROM catalog.users").expect("select");
    assert!(matches!(
        &select.roots[0],
        AstRootV1::Select(spec) if spec.from == "SELECT id FROM catalog.users"
    ));

    let find = parse_document_v1("FIND taint src -> sink").expect("find");
    assert!(matches!(
        &find.roots[0],
        AstRootV1::Find(spec) if spec.subkind == FindSubkindV1::Taint && spec.seed == "src -> sink"
    ));

    let check = parse_document_v1("CHECK rule.scope workspace").expect("check");
    assert!(matches!(
        &check.roots[0],
        AstRootV1::Check(spec) if spec.rule_id == "rule.scope" && spec.scope == "workspace"
    ));

    let explain = parse_document_v1("EXPLAIN result-123").expect("explain");
    assert!(matches!(
        &explain.roots[0],
        AstRootV1::Explain(spec) if spec.target.as_deref() == Some("result-123")
    ));
}

#[test]
fn parser_dispatches_meta_roots_v1() {
    let use_pack = parse_document_v1(
        "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock",
    )
    .expect("use pack");
    assert!(matches!(
        &use_pack.roots[0],
        AstRootV1::Meta(MetaCommandAstV1::UsePack {
            pack_id,
            activation_mode,
            lockfile_path,
        }) if pack_id == "secure-pack"
            && activation_mode.as_deref() == Some("session")
            && lockfile_path.as_deref() == Some("packs/secure-pack.lock")
    ));

    let require_certified = parse_document_v1("REQUIRE CERTIFIED").expect("require");
    assert!(matches!(
        &require_certified.roots[0],
        AstRootV1::Meta(MetaCommandAstV1::RequireCertified)
    ));
}

#[test]
fn parser_dispatches_interactive_and_declaration_roots_v1() {
    let show = parse_document_v1("SHOW capabilities").expect("show");
    assert!(matches!(
        &show.roots[0],
        AstRootV1::Show(spec) if spec.category == "capabilities"
    ));

    let describe = parse_document_v1("DESCRIBE rule::owasp.sql-injection").expect("describe");
    assert!(matches!(
        &describe.roots[0],
        AstRootV1::Describe(spec) if spec.target == "rule::owasp.sql-injection"
    ));

    let sample = parse_document_v1(
        "SAMPLE query=error propagation ;; scope=repo::frontend ;; mode=hybrid ;; limit=5",
    )
    .expect("sample");
    assert!(matches!(
        &sample.roots[0],
        AstRootV1::Sample(spec) if spec.query_text.contains("mode=hybrid")
    ));

    let view =
        parse_document_v1("VIEW catalog.users ;; query-ref=queries/catalog/users").expect("view");
    assert!(matches!(
        &view.roots[0],
        AstRootV1::View(spec) if spec.name.contains("query-ref=queries/catalog/users")
    ));

    let profile = parse_document_v1(
        "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=review_required",
    )
    .expect("profile");
    assert!(matches!(
        &profile.roots[0],
        AstRootV1::Profile(spec) if spec.name.contains("admission=review_required")
    ));
}
