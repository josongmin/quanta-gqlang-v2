use quanta_gqlang_v2_syntax::declarations::pack::{
    GateCheckBindingModeV1 as SyntaxGateCheckBindingModeV1,
    GateDeclShapeAstV1 as SyntaxGateDeclShapeAstV1,
    PackActivationModeV1 as SyntaxPackActivationModeV1,
    PackDeclShapeAstV1 as SyntaxPackDeclShapeAstV1,
    ProfileAdmissionPolicyV1 as SyntaxProfileAdmissionPolicyV1,
    ProfileDeclShapeAstV1 as SyntaxProfileDeclShapeAstV1,
};
use quanta_gqlang_v2_syntax::declarations::view::{
    ModelDeclShapeAstV1 as SyntaxModelDeclShapeAstV1,
    RuleDeclShapeAstV1 as SyntaxRuleDeclShapeAstV1, ViewDeclShapeAstV1 as SyntaxViewDeclShapeAstV1,
    ViewQueryShapeAstV1 as SyntaxViewQueryShapeAstV1,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedDeclarationRootV1 {
    View(TypedViewDeclarationV1),
    Rule(TypedRuleDeclarationV1),
    Model(TypedModelDeclarationV1),
    Pack(TypedPackDeclarationV1),
    Profile(TypedProfileDeclarationV1),
    Gate(TypedGateDeclarationV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedViewDeclarationV1 {
    pub name: String,
    pub query: TypedViewQueryV1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedViewQueryV1 {
    QueryRef(String),
    InlineSelect(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedRuleDeclarationV1 {
    pub name: String,
    pub check_ref: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedModelDeclarationV1 {
    pub name: String,
    pub provider: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedPackDeclarationV1 {
    pub name: String,
    pub source: String,
    pub activation: TypedPackActivationModeV1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedPackActivationModeV1 {
    Manual,
    Session,
    Workspace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedProfileDeclarationV1 {
    pub name: String,
    pub capabilities: Vec<String>,
    pub admission: TypedProfileAdmissionPolicyV1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedProfileAdmissionPolicyV1 {
    Allowlist,
    ReviewRequired,
    CertifiedOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedGateDeclarationV1 {
    pub name: String,
    pub checks: Vec<String>,
    pub binding: TypedGateCheckBindingModeV1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedGateCheckBindingModeV1 {
    All,
    Any,
}

impl TypedDeclarationRootV1 {
    pub fn from_view_shape_v1(shape: SyntaxViewDeclShapeAstV1) -> Self {
        Self::View(TypedViewDeclarationV1::from(shape))
    }

    pub fn from_rule_shape_v1(shape: SyntaxRuleDeclShapeAstV1) -> Self {
        Self::Rule(TypedRuleDeclarationV1::from(shape))
    }

    pub fn from_model_shape_v1(shape: SyntaxModelDeclShapeAstV1) -> Self {
        Self::Model(TypedModelDeclarationV1::from(shape))
    }

    pub fn from_pack_shape_v1(shape: SyntaxPackDeclShapeAstV1) -> Self {
        Self::Pack(TypedPackDeclarationV1::from(shape))
    }

    pub fn from_profile_shape_v1(shape: SyntaxProfileDeclShapeAstV1) -> Self {
        Self::Profile(TypedProfileDeclarationV1::from(shape))
    }

    pub fn from_gate_shape_v1(shape: SyntaxGateDeclShapeAstV1) -> Self {
        Self::Gate(TypedGateDeclarationV1::from(shape))
    }
}

impl From<SyntaxViewDeclShapeAstV1> for TypedViewDeclarationV1 {
    fn from(value: SyntaxViewDeclShapeAstV1) -> Self {
        Self {
            name: value.name,
            query: match value.query {
                SyntaxViewQueryShapeAstV1::QueryRef(query_ref) => {
                    TypedViewQueryV1::QueryRef(query_ref)
                }
                SyntaxViewQueryShapeAstV1::InlineSelect(select) => {
                    TypedViewQueryV1::InlineSelect(select.raw_query_v1().to_string())
                }
            },
        }
    }
}

impl From<SyntaxRuleDeclShapeAstV1> for TypedRuleDeclarationV1 {
    fn from(value: SyntaxRuleDeclShapeAstV1) -> Self {
        Self {
            name: value.name,
            check_ref: value.linkage.check_ref,
        }
    }
}

impl From<SyntaxModelDeclShapeAstV1> for TypedModelDeclarationV1 {
    fn from(value: SyntaxModelDeclShapeAstV1) -> Self {
        Self {
            name: value.name,
            provider: value.binding.provider,
            source: value.binding.source,
        }
    }
}

impl From<SyntaxPackDeclShapeAstV1> for TypedPackDeclarationV1 {
    fn from(value: SyntaxPackDeclShapeAstV1) -> Self {
        Self {
            name: value.name,
            source: value.source,
            activation: match value.activation {
                SyntaxPackActivationModeV1::Manual => TypedPackActivationModeV1::Manual,
                SyntaxPackActivationModeV1::Session => TypedPackActivationModeV1::Session,
                SyntaxPackActivationModeV1::Workspace => TypedPackActivationModeV1::Workspace,
            },
        }
    }
}

impl From<SyntaxProfileDeclShapeAstV1> for TypedProfileDeclarationV1 {
    fn from(value: SyntaxProfileDeclShapeAstV1) -> Self {
        Self {
            name: value.name,
            capabilities: value.capabilities,
            admission: match value.admission {
                SyntaxProfileAdmissionPolicyV1::Allowlist => {
                    TypedProfileAdmissionPolicyV1::Allowlist
                }
                SyntaxProfileAdmissionPolicyV1::ReviewRequired => {
                    TypedProfileAdmissionPolicyV1::ReviewRequired
                }
                SyntaxProfileAdmissionPolicyV1::CertifiedOnly => {
                    TypedProfileAdmissionPolicyV1::CertifiedOnly
                }
            },
        }
    }
}

impl From<SyntaxGateDeclShapeAstV1> for TypedGateDeclarationV1 {
    fn from(value: SyntaxGateDeclShapeAstV1) -> Self {
        Self {
            name: value.name,
            checks: value.checks,
            binding: match value.binding {
                SyntaxGateCheckBindingModeV1::All => TypedGateCheckBindingModeV1::All,
                SyntaxGateCheckBindingModeV1::Any => TypedGateCheckBindingModeV1::Any,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        reason = "declaration sema tests assert validated shape decoding success"
    )]
    #![expect(
        clippy::panic,
        reason = "unexpected typed declaration variants must fail closed in unit tests"
    )]

    use super::{
        TypedDeclarationRootV1, TypedGateCheckBindingModeV1, TypedProfileAdmissionPolicyV1,
        TypedViewQueryV1,
    };
    use quanta_gqlang_v2_syntax::declarations::pack::{
        decode_gate_decl_shape_v1, decode_pack_decl_shape_v1, decode_profile_decl_shape_v1,
    };
    use quanta_gqlang_v2_syntax::declarations::view::{
        decode_model_decl_shape_v1, decode_rule_decl_shape_v1, decode_view_decl_shape_v1,
    };

    #[test]
    fn typed_view_declaration_preserves_query_ref_shape() {
        let shape = decode_view_decl_shape_v1("catalog.users ;; query-ref=queries/catalog/users")
            .expect("shape");
        let TypedDeclarationRootV1::View(spec) = TypedDeclarationRootV1::from_view_shape_v1(shape)
        else {
            panic!("expected VIEW typed declaration");
        };
        assert_eq!(spec.name, "catalog.users");
        assert_eq!(
            spec.query,
            TypedViewQueryV1::QueryRef("queries/catalog/users".to_string())
        );
    }

    #[test]
    fn typed_rule_model_pack_profile_gate_preserve_structured_fields() {
        let rule =
            decode_rule_decl_shape_v1("owasp.sql ;; check-ref=checks/owasp/sql").expect("rule");
        let model = decode_model_decl_shape_v1(
            "github.repo ;; provider=github ;; source=repositories/default",
        )
        .expect("model");
        let pack = decode_pack_decl_shape_v1(
            "secure-pack ;; source=registry::security ;; activation=session",
        )
        .expect("pack");
        let profile = decode_profile_decl_shape_v1(
            "strict-review ;; capabilities=graph.read,graph.trace ;; admission=certified-only",
        )
        .expect("profile");
        let gate =
            decode_gate_decl_shape_v1("release-gate ;; checks=owasp.sql,authz.path ;; binding=any")
                .expect("gate");

        let TypedDeclarationRootV1::Rule(rule) = TypedDeclarationRootV1::from_rule_shape_v1(rule)
        else {
            panic!("expected RULE");
        };
        let TypedDeclarationRootV1::Model(model) =
            TypedDeclarationRootV1::from_model_shape_v1(model)
        else {
            panic!("expected MODEL");
        };
        let TypedDeclarationRootV1::Pack(pack) = TypedDeclarationRootV1::from_pack_shape_v1(pack)
        else {
            panic!("expected PACK");
        };
        let TypedDeclarationRootV1::Profile(profile) =
            TypedDeclarationRootV1::from_profile_shape_v1(profile)
        else {
            panic!("expected PROFILE");
        };
        let TypedDeclarationRootV1::Gate(gate) = TypedDeclarationRootV1::from_gate_shape_v1(gate)
        else {
            panic!("expected GATE");
        };

        assert_eq!(rule.check_ref, "checks/owasp/sql");
        assert_eq!(model.provider, "github");
        assert_eq!(model.source, "repositories/default");
        assert_eq!(pack.source, "registry::security");
        assert_eq!(
            profile.admission,
            TypedProfileAdmissionPolicyV1::CertifiedOnly
        );
        assert_eq!(gate.binding, TypedGateCheckBindingModeV1::Any);
    }
}
