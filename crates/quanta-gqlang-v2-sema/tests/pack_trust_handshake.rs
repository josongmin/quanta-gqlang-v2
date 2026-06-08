#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

use quanta_gqlang_v2_sema::{
    analyze_document_v1, TypedCertifiedAdmissionPolicyV1, TypedCertifiedTruthRequirementV1,
    TypedMetaCommandV1, TypedPackActivationModeV1, TypedRootV1,
};

fn analyze_single_root_v1(query: &str) -> TypedRootV1 {
    let doc = quanta_gqlang_v2_syntax::parse_document_v1(query).expect("parse");
    let typed = analyze_document_v1(&doc).expect("sema");
    typed.roots.into_iter().next().expect("single root")
}

#[test]
fn use_pack_keeps_activation_and_lockfile_explicit_v1() {
    let root = analyze_single_root_v1(
        "USE PACK secure-pack ;; activation=workspace ;; lockfile=packs/secure-pack.lock",
    );

    assert_eq!(
        root,
        TypedRootV1::Meta(TypedMetaCommandV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: TypedPackActivationModeV1::Workspace,
            lockfile_path: "packs/secure-pack.lock".to_string(),
        })
    );
}

#[test]
fn require_certified_stays_fail_closed_and_explicit_v1() {
    let root = analyze_single_root_v1("REQUIRE CERTIFIED");

    assert_eq!(
        root,
        TypedRootV1::Meta(TypedMetaCommandV1::RequireCertified {
            truth_requirement: TypedCertifiedTruthRequirementV1::CertifiedOnly,
            admission_policy: TypedCertifiedAdmissionPolicyV1::FailClosed,
        })
    );
}

#[test]
fn pack_trust_queries_fail_closed_when_trust_surface_is_not_explicit_v1() {
    let missing_lockfile =
        quanta_gqlang_v2_syntax::parse_document_v1("USE PACK secure-pack ;; activation=workspace")
            .expect_err("USE PACK without lockfile must fail closed");
    assert_eq!(
        missing_lockfile.diagnostics[0].code,
        "use_pack_missing_lockfile"
    );

    let ambient_clause = quanta_gqlang_v2_syntax::parse_document_v1(
        "USE PACK secure-pack ;; activation=workspace ;; lockfile=packs/secure-pack.lock ;; source=ambient",
    )
    .expect_err("ambient USE PACK clause must fail closed");
    assert_eq!(
        ambient_clause.diagnostics[0].code,
        "unsupported_use_pack_clause"
    );

    let require_with_tail =
        quanta_gqlang_v2_syntax::parse_document_v1("REQUIRE CERTIFIED ;; truth=maybe")
            .expect_err("REQUIRE CERTIFIED must reject non-explicit tail clauses");
    assert_eq!(
        require_with_tail.diagnostics[0].code,
        "unsupported_require_meta"
    );
}
