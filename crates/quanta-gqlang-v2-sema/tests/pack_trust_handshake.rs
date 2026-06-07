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
