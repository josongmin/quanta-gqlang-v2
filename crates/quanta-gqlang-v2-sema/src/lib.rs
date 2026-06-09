#![forbid(unsafe_code)]

pub mod admissibility;
pub mod ai;
pub mod declarations;
pub mod interactive;
pub mod profile;
pub mod roots;

// Wiring unit tests live in their own module so this root stays production-only.
#[cfg(test)]
mod lib_tests;

pub const CRATE_ID_V1: &str = "quanta-gqlang-v2-sema";

pub use quanta_gqlang_v2_syntax::ast::AstRootV1;
pub use quanta_gqlang_v2_syntax::declarations::pack::{
    GateDeclAstV1, PackDeclAstV1, ProfileDeclAstV1,
};
pub use quanta_gqlang_v2_syntax::declarations::view::{
    ModelDeclAstV1, RuleDeclAstV1, ViewDeclAstV1,
};
pub use quanta_gqlang_v2_syntax::interactive::meta::{
    CertifiedAdmissionPolicyAstV1, CertifiedTruthRequirementAstV1, MetaCommandAstV1,
    MetaCommandShapeErrorV1, MetaCommandShapeV1, PackActivationModeAstV1,
};
pub use quanta_gqlang_v2_syntax::interactive::sample::{
    CompareRootAstV1, ExportRootAstV1, SampleRootAstV1,
};
pub use quanta_gqlang_v2_syntax::interactive::show::{DescribeRootAstV1, ShowRootAstV1};
pub use quanta_gqlang_v2_syntax::roots::check::{CheckRootAstV1, CheckRootIntentV1};
pub use quanta_gqlang_v2_syntax::roots::explain::{ExplainRootAstV1, ExplainShapeAstV1};
pub use quanta_gqlang_v2_syntax::roots::find::FindRootAstV1;
pub use quanta_gqlang_v2_syntax::roots::select::{
    parse_select_shape_v1, SelectFieldRefAstV1, SelectFilterAstV1, SelectFilterOpV1,
    SelectLiteralAstV1, SelectProjectionAstV1, SelectQueryShapeV1, SelectRootAstV1,
    SelectRootErrorV1, SelectTableRefAstV1,
};
pub use quanta_gqlang_v2_syntax::CRATE_ID_V1 as SYNTAX_CRATE_ID_V1;
pub use quanta_gqlang_v2_syntax::{
    document_statements_v1, format_document_v1, parse_document_v1, DocumentAstV1,
    SyntaxDiagnosticPageV1,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SemaDiagnosticPageV1 {
    pub diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TypedDocumentV1 {
    pub roots: Vec<TypedRootV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedRootV1 {
    Select(roots::select::TypedSelectRootV1),
    Find(roots::find::TypedFindRootV1),
    Check(roots::check::TypedCheckRootV1),
    Explain(roots::explain::TypedExplainRootV1),
    ShowDescribe(interactive::show_describe::TypedShowDescribeRootV1),
    SampleCompareExport(interactive::sample_compare_export::TypedSampleCompareExportRootV1),
    Declaration(declarations::model_pack::TypedDeclarationRootV1),
    Meta(TypedMetaCommandV1),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypedMetaCommandV1 {
    UsePack {
        pack_id: String,
        activation_mode: TypedPackActivationModeV1,
        lockfile_path: String,
    },
    RequireCertified {
        truth_requirement: TypedCertifiedTruthRequirementV1,
        admission_policy: TypedCertifiedAdmissionPolicyV1,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedPackActivationModeV1 {
    Session,
    Workspace,
}

impl TypedPackActivationModeV1 {
    pub const fn as_str_v1(self) -> &'static str {
        match self {
            Self::Session => "session",
            Self::Workspace => "workspace",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedCertifiedTruthRequirementV1 {
    CertifiedOnly,
}

impl TypedCertifiedTruthRequirementV1 {
    pub const fn as_str_v1(self) -> &'static str {
        match self {
            Self::CertifiedOnly => "certified-only",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TypedCertifiedAdmissionPolicyV1 {
    FailClosed,
}

impl TypedCertifiedAdmissionPolicyV1 {
    pub const fn as_str_v1(self) -> &'static str {
        match self {
            Self::FailClosed => "fail-closed",
        }
    }
}

pub fn analyze_document_v1(doc: &DocumentAstV1) -> Result<TypedDocumentV1, SemaDiagnosticPageV1> {
    let roots = doc
        .roots
        .iter()
        .map(analyze_root_v1)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TypedDocumentV1 { roots })
}

pub fn analyze_root_v1(root: &AstRootV1) -> Result<TypedRootV1, SemaDiagnosticPageV1> {
    match root {
        AstRootV1::Select(spec) => roots::select::analyze_select_root_v1(spec)
            .map(TypedRootV1::Select)
            .map_err(|err| sema_diagnostic_page_v1(format!("{}: {}", err.code, err.message))),
        AstRootV1::Find(spec) => {
            let typed = roots::find::TypedFindRootV1 {
                subkind: spec.subkind.as_str().to_string(),
                seed: spec.seed.clone(),
            };
            typed
                .typed_spec()
                .map_err(|err| sema_diagnostic_page_v1(err.message.clone()))?;
            Ok(TypedRootV1::Find(typed))
        }
        AstRootV1::Check(spec) => {
            let typed = roots::check::TypedCheckRootV1::from_ast(spec);
            typed
                .intent()
                .map_err(|err| sema_diagnostic_page_v1(err.message.clone()))?;
            Ok(TypedRootV1::Check(typed))
        }
        AstRootV1::Explain(spec) => {
            let typed = roots::explain::TypedExplainRootV1 {
                target: spec.target.clone(),
            };
            typed
                .shape()
                .map_err(|err| sema_diagnostic_page_v1(err.message.clone()))?;
            Ok(TypedRootV1::Explain(typed))
        }
        AstRootV1::Show(spec) => {
            interactive::show_describe::TypedShowDescribeRootV1::from_show_ast_v1(spec)
                .map(TypedRootV1::ShowDescribe)
                .map_err(|err| sema_diagnostic_page_v1(format!("{}: {}", err.code, err.message)))
        }
        AstRootV1::Describe(spec) => {
            interactive::show_describe::TypedShowDescribeRootV1::from_describe_ast_v1(spec)
                .map(TypedRootV1::ShowDescribe)
                .map_err(|err| sema_diagnostic_page_v1(format!("{}: {}", err.code, err.message)))
        }
        AstRootV1::Sample(spec) => interactive::sample_compare_export::analyze_sample_root_v1(spec)
            .map(TypedRootV1::SampleCompareExport)
            .map_err(|err| sema_diagnostic_page_v1(err.message)),
        AstRootV1::Compare(spec) => {
            interactive::sample_compare_export::analyze_compare_root_v1(spec)
                .map(TypedRootV1::SampleCompareExport)
                .map_err(|err| sema_diagnostic_page_v1(err.message))
        }
        AstRootV1::Export(spec) => interactive::sample_compare_export::analyze_export_root_v1(spec)
            .map(TypedRootV1::SampleCompareExport)
            .map_err(|err| sema_diagnostic_page_v1(err.message)),
        AstRootV1::View(spec) => spec
            .shape_v1()
            .map(declarations::model_pack::TypedDeclarationRootV1::from_view_shape_v1)
            .map(TypedRootV1::Declaration)
            .map_err(|err| sema_diagnostic_page_v1(err.message)),
        AstRootV1::Rule(spec) => spec
            .shape_v1()
            .map(declarations::model_pack::TypedDeclarationRootV1::from_rule_shape_v1)
            .map(TypedRootV1::Declaration)
            .map_err(|err| sema_diagnostic_page_v1(err.message)),
        AstRootV1::Model(spec) => spec
            .shape_v1()
            .map(declarations::model_pack::TypedDeclarationRootV1::from_model_shape_v1)
            .map(TypedRootV1::Declaration)
            .map_err(|err| sema_diagnostic_page_v1(err.message)),
        AstRootV1::Pack(spec) => spec
            .shape()
            .map(declarations::model_pack::TypedDeclarationRootV1::from_pack_shape_v1)
            .map(TypedRootV1::Declaration)
            .map_err(|err| sema_diagnostic_page_v1(err.message)),
        AstRootV1::Profile(spec) => spec
            .shape()
            .map(declarations::model_pack::TypedDeclarationRootV1::from_profile_shape_v1)
            .map(TypedRootV1::Declaration)
            .map_err(|err| sema_diagnostic_page_v1(err.message)),
        AstRootV1::Gate(spec) => spec
            .shape()
            .map(declarations::model_pack::TypedDeclarationRootV1::from_gate_shape_v1)
            .map(TypedRootV1::Declaration)
            .map_err(|err| sema_diagnostic_page_v1(err.message)),
        AstRootV1::Meta(command) => analyze_meta_command_v1(command).map(TypedRootV1::Meta),
    }
}

fn sema_diagnostic_page_v1(message: impl Into<String>) -> SemaDiagnosticPageV1 {
    SemaDiagnosticPageV1 {
        diagnostics: vec![message.into()],
    }
}

fn analyze_meta_command_v1(
    command: &MetaCommandAstV1,
) -> Result<TypedMetaCommandV1, SemaDiagnosticPageV1> {
    match command
        .shape()
        .map_err(|err| sema_diagnostic_page_v1(format!("{}: {}", err.code, err.message)))?
    {
        MetaCommandShapeV1::UsePack(spec) => Ok(TypedMetaCommandV1::UsePack {
            pack_id: spec.pack_id.to_string(),
            activation_mode: match spec.activation_mode {
                PackActivationModeAstV1::Session => TypedPackActivationModeV1::Session,
                PackActivationModeAstV1::Workspace => TypedPackActivationModeV1::Workspace,
            },
            lockfile_path: spec.lockfile_path.to_string(),
        }),
        MetaCommandShapeV1::RequireCertified(spec) => Ok(TypedMetaCommandV1::RequireCertified {
            truth_requirement: match spec.truth_requirement {
                CertifiedTruthRequirementAstV1::CertifiedOnly => {
                    TypedCertifiedTruthRequirementV1::CertifiedOnly
                }
            },
            admission_policy: match spec.admission_policy {
                CertifiedAdmissionPolicyAstV1::FailClosed => {
                    TypedCertifiedAdmissionPolicyV1::FailClosed
                }
            },
        }),
    }
}
