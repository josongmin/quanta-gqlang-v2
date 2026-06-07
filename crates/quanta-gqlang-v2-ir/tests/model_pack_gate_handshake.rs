use quanta_gqlang_v2_ir::{
    lower_document_to_ir_v1, root_ir::DeclarationIrV1, root_ir::GateBindingIrV1, root_ir::MetaIrV1,
    root_ir::PackActivationIrV1, root_ir::ProfileAdmissionIrV1, root_ir::PublicIrRootV1,
};

fn lower_single_root_v1(query: &str) -> PublicIrRootV1 {
    let doc = quanta_gqlang_v2_syntax::parse_document_v1(query).expect("parse");
    let typed = quanta_gqlang_v2_sema::analyze_document_v1(&doc).expect("sema");
    let mut ir = lower_document_to_ir_v1(&typed).expect("ir");
    assert_eq!(ir.len(), 1, "expected single-root IR");
    ir.remove(0)
}

#[test]
fn pack_profile_gate_lower_to_typed_product_nodes_v1() {
    let pack = lower_single_root_v1(
        "PACK secure-pack ;; source=registry::security ;; activation=workspace",
    );
    assert_eq!(
        pack,
        PublicIrRootV1::Declaration(DeclarationIrV1::Pack {
            name: "secure-pack".to_string(),
            source: "registry::security".to_string(),
            activation: PackActivationIrV1::Workspace,
        })
    );

    let profile = lower_single_root_v1(
        "PROFILE strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only",
    );
    assert_eq!(
        profile,
        PublicIrRootV1::Declaration(DeclarationIrV1::Profile {
            name: "strict-review".to_string(),
            capabilities: vec!["graph.read".to_string(), "graph.trace".to_string()],
            admission: ProfileAdmissionIrV1::CertifiedOnly,
        })
    );

    let gate =
        lower_single_root_v1("GATE release-gate ;; checks=owasp.sql,authz.path ;; binding=all");
    assert_eq!(
        gate,
        PublicIrRootV1::Declaration(DeclarationIrV1::Gate {
            name: "release-gate".to_string(),
            checks: vec!["owasp.sql".to_string(), "authz.path".to_string()],
            binding: GateBindingIrV1::All,
        })
    );
}

#[test]
fn use_pack_and_require_certified_lower_to_explicit_meta_nodes_v1() {
    let use_pack = lower_single_root_v1(
        "USE PACK secure-pack ;; activation=session ;; lockfile=packs/secure-pack.lock",
    );
    assert_eq!(
        use_pack,
        PublicIrRootV1::Meta(MetaIrV1::UsePack {
            pack_id: "secure-pack".to_string(),
            activation_mode: quanta_gqlang_v2_ir::root_ir::PackActivationIrV1::Session,
            lockfile_path: "packs/secure-pack.lock".to_string(),
        })
    );

    let require = lower_single_root_v1("REQUIRE CERTIFIED");
    assert_eq!(
        require,
        PublicIrRootV1::Meta(MetaIrV1::RequireCertified {
            truth_requirement:
                quanta_gqlang_v2_ir::root_ir::CertifiedTruthRequirementIrV1::CertifiedOnly,
            admission_policy:
                quanta_gqlang_v2_ir::root_ir::CertifiedAdmissionPolicyIrV1::FailClosed,
        })
    );
}
