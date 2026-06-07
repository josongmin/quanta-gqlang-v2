#![forbid(unsafe_code)]

pub mod admissibility;
pub mod ai;
pub mod declarations;
pub mod interactive;
pub mod profile;
pub mod roots;
pub mod scope;

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
pub use quanta_gqlang_v2_syntax::roots::select::SelectRootAstV1;
#[cfg(test)]
use quanta_gqlang_v2_syntax::typed_ast::TypedDocumentAstV1;
pub use quanta_gqlang_v2_syntax::CRATE_ID_V1 as SYNTAX_CRATE_ID_V1;
pub use quanta_gqlang_v2_syntax::{
    format_document_v1, parse_document_v1, DocumentAstV1, SyntaxDiagnosticPageV1,
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

#[cfg(test)]
pub fn bind_typed_document_v1(doc: &TypedDocumentV1) -> TypedDocumentAstV1 {
    TypedDocumentAstV1 {
        roots: doc
            .roots
            .iter()
            .map(|root| quanta_gqlang_v2_syntax::typed_ast::TypedRootAstV1 {
                root: match root {
                    TypedRootV1::Select(_) => {
                        AstRootV1::Select(quanta_gqlang_v2_syntax::roots::select::SelectRootAstV1 {
                            from: match root {
                                TypedRootV1::Select(spec) => spec.from_entity.clone(),
                                _ => unreachable!("root variant already matched"),
                            },
                        })
                    }
                    TypedRootV1::Find(_) => {
                        AstRootV1::Find(quanta_gqlang_v2_syntax::roots::find::FindRootAstV1 {
                            subkind: match root {
                                TypedRootV1::Find(spec) => bind_find_subkind_v1(&spec.subkind),
                                _ => unreachable!("root variant already matched"),
                            },
                            seed: match root {
                                TypedRootV1::Find(spec) => spec.seed.clone(),
                                _ => unreachable!("root variant already matched"),
                            },
                        })
                    }
                    TypedRootV1::Check(_) => AstRootV1::Check(match root {
                        TypedRootV1::Check(spec) => spec.to_ast().unwrap_or_else(|_| {
                            quanta_gqlang_v2_syntax::roots::check::CheckRootAstV1 {
                                rule_id: spec.rule_id.clone(),
                                scope: spec.scope.clone(),
                            }
                        }),
                        _ => unreachable!("root variant already matched"),
                    }),
                    TypedRootV1::Explain(_) => AstRootV1::Explain(
                        quanta_gqlang_v2_syntax::roots::explain::ExplainRootAstV1 {
                            target: match root {
                                TypedRootV1::Explain(spec) => spec.target.clone(),
                                _ => unreachable!("root variant already matched"),
                            },
                        },
                    ),
                    TypedRootV1::ShowDescribe(spec) => bind_show_describe_root_v1(spec),
                    TypedRootV1::SampleCompareExport(spec) => {
                        bind_sample_compare_export_root_v1(spec)
                    }
                    TypedRootV1::Declaration(spec) => bind_declaration_root_v1(spec),
                    TypedRootV1::Meta(spec) => AstRootV1::Meta(bind_meta_command_v1(spec)),
                },
                profile: None,
            })
            .collect(),
    }
}

fn sema_diagnostic_page_v1(message: impl Into<String>) -> SemaDiagnosticPageV1 {
    SemaDiagnosticPageV1 {
        diagnostics: vec![message.into()],
    }
}

#[cfg(test)]
fn bind_find_subkind_v1(raw: &str) -> quanta_gqlang_v2_syntax::roots::find::FindSubkindV1 {
    match raw.trim().to_ascii_lowercase().replace('_', "").as_str() {
        "reachability" => quanta_gqlang_v2_syntax::roots::find::FindSubkindV1::Reachability,
        "flow" => quanta_gqlang_v2_syntax::roots::find::FindSubkindV1::Flow,
        "slice" => quanta_gqlang_v2_syntax::roots::find::FindSubkindV1::Slice,
        "taint" => quanta_gqlang_v2_syntax::roots::find::FindSubkindV1::Taint,
        "path" => quanta_gqlang_v2_syntax::roots::find::FindSubkindV1::Path,
        "pointsto" => quanta_gqlang_v2_syntax::roots::find::FindSubkindV1::PointsTo,
        "summary" => quanta_gqlang_v2_syntax::roots::find::FindSubkindV1::Summary,
        other => panic!("unexpected validated FIND subkind `{other}`"),
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

#[cfg(test)]
fn bind_show_describe_root_v1(
    spec: &interactive::show_describe::TypedShowDescribeRootV1,
) -> AstRootV1 {
    match spec.typed_spec().expect("validated SHOW/DESCRIBE root") {
        interactive::show_describe::TypedShowDescribeSpecV1::Show(spec) => {
            AstRootV1::Show(quanta_gqlang_v2_syntax::interactive::show::ShowRootAstV1::from_category_spec_v1(
                match spec.category {
                    interactive::show_describe::TypedShowCategoryV1::Capabilities => {
                        quanta_gqlang_v2_syntax::interactive::show::ShowCategoryAstV1::Capabilities
                    }
                    interactive::show_describe::TypedShowCategoryV1::Models => {
                        quanta_gqlang_v2_syntax::interactive::show::ShowCategoryAstV1::Models
                    }
                    interactive::show_describe::TypedShowCategoryV1::Views => {
                        quanta_gqlang_v2_syntax::interactive::show::ShowCategoryAstV1::Views
                    }
                    interactive::show_describe::TypedShowCategoryV1::Rules => {
                        quanta_gqlang_v2_syntax::interactive::show::ShowCategoryAstV1::Rules
                    }
                    interactive::show_describe::TypedShowCategoryV1::Packs => {
                        quanta_gqlang_v2_syntax::interactive::show::ShowCategoryAstV1::Packs
                    }
                    interactive::show_describe::TypedShowCategoryV1::Profiles => {
                        quanta_gqlang_v2_syntax::interactive::show::ShowCategoryAstV1::Profiles
                    }
                    interactive::show_describe::TypedShowCategoryV1::Gates => {
                        quanta_gqlang_v2_syntax::interactive::show::ShowCategoryAstV1::Gates
                    }
                },
            ))
        }
        interactive::show_describe::TypedShowDescribeSpecV1::Describe(spec) => {
            AstRootV1::Describe(quanta_gqlang_v2_syntax::interactive::show::DescribeRootAstV1::from_target_spec_v1(
                quanta_gqlang_v2_syntax::interactive::show::DescribeTargetAstV1 {
                    kind: match spec.target.kind {
                        interactive::show_describe::TypedDescribeTargetKindV1::Capability => {
                            quanta_gqlang_v2_syntax::interactive::show::DescribeTargetKindAstV1::Capability
                        }
                        interactive::show_describe::TypedDescribeTargetKindV1::Model => {
                            quanta_gqlang_v2_syntax::interactive::show::DescribeTargetKindAstV1::Model
                        }
                        interactive::show_describe::TypedDescribeTargetKindV1::View => {
                            quanta_gqlang_v2_syntax::interactive::show::DescribeTargetKindAstV1::View
                        }
                        interactive::show_describe::TypedDescribeTargetKindV1::Rule => {
                            quanta_gqlang_v2_syntax::interactive::show::DescribeTargetKindAstV1::Rule
                        }
                        interactive::show_describe::TypedDescribeTargetKindV1::Pack => {
                            quanta_gqlang_v2_syntax::interactive::show::DescribeTargetKindAstV1::Pack
                        }
                        interactive::show_describe::TypedDescribeTargetKindV1::Profile => {
                            quanta_gqlang_v2_syntax::interactive::show::DescribeTargetKindAstV1::Profile
                        }
                        interactive::show_describe::TypedDescribeTargetKindV1::Gate => {
                            quanta_gqlang_v2_syntax::interactive::show::DescribeTargetKindAstV1::Gate
                        }
                    },
                    path: spec.target.path,
                },
            ))
        }
    }
}

#[cfg(test)]
fn bind_sample_compare_export_root_v1(
    spec: &interactive::sample_compare_export::TypedSampleCompareExportRootV1,
) -> AstRootV1 {
    match spec.shape().expect("validated SAMPLE/COMPARE/EXPORT root") {
        interactive::sample_compare_export::TypedSampleCompareExportShapeV1::Sample(intent) => {
            AstRootV1::Sample(quanta_gqlang_v2_syntax::interactive::sample::SampleRootAstV1::from_intent(
                quanta_gqlang_v2_syntax::interactive::sample::SampleRetrievalIntentAstV1 {
                    query: intent.query,
                    scope: intent.scope.map(|scope| {
                        quanta_gqlang_v2_syntax::interactive::sample::InteractiveRefAstV1 {
                            kind: scope.kind,
                            locator: scope.locator,
                        }
                    }),
                    mode: match intent.mode {
                        interactive::sample_compare_export::TypedSampleRetrievalModeV1::Lexical => {
                            quanta_gqlang_v2_syntax::interactive::sample::SampleRetrievalModeV1::Lexical
                        }
                        interactive::sample_compare_export::TypedSampleRetrievalModeV1::Semantic => {
                            quanta_gqlang_v2_syntax::interactive::sample::SampleRetrievalModeV1::Semantic
                        }
                        interactive::sample_compare_export::TypedSampleRetrievalModeV1::Hybrid => {
                            quanta_gqlang_v2_syntax::interactive::sample::SampleRetrievalModeV1::Hybrid
                        }
                    },
                    limit: intent.limit,
                },
            ))
        }
        interactive::sample_compare_export::TypedSampleCompareExportShapeV1::Compare(intent) => {
            AstRootV1::Compare(quanta_gqlang_v2_syntax::interactive::sample::CompareRootAstV1::from_intent(
                quanta_gqlang_v2_syntax::interactive::sample::CompareIntentAstV1 {
                    baseline: quanta_gqlang_v2_syntax::interactive::sample::InteractiveRefAstV1 {
                        kind: intent.baseline.kind,
                        locator: intent.baseline.locator,
                    },
                    candidate: quanta_gqlang_v2_syntax::interactive::sample::InteractiveRefAstV1 {
                        kind: intent.candidate.kind,
                        locator: intent.candidate.locator,
                    },
                    metric: match intent.metric {
                        interactive::sample_compare_export::TypedCompareMetricV1::Exact => {
                            quanta_gqlang_v2_syntax::interactive::sample::CompareMetricV1::Exact
                        }
                        interactive::sample_compare_export::TypedCompareMetricV1::Structural => {
                            quanta_gqlang_v2_syntax::interactive::sample::CompareMetricV1::Structural
                        }
                        interactive::sample_compare_export::TypedCompareMetricV1::Semantic => {
                            quanta_gqlang_v2_syntax::interactive::sample::CompareMetricV1::Semantic
                        }
                    },
                },
            ))
        }
        interactive::sample_compare_export::TypedSampleCompareExportShapeV1::Export(intent) => {
            AstRootV1::Export(quanta_gqlang_v2_syntax::interactive::sample::ExportRootAstV1::from_intent(
                quanta_gqlang_v2_syntax::interactive::sample::ExportIntentAstV1 {
                    source: quanta_gqlang_v2_syntax::interactive::sample::InteractiveRefAstV1 {
                        kind: intent.source.kind,
                        locator: intent.source.locator,
                    },
                    kind: match intent.kind {
                        interactive::sample_compare_export::TypedExportKindV1::Json => {
                            quanta_gqlang_v2_syntax::interactive::sample::ExportKindV1::Json
                        }
                        interactive::sample_compare_export::TypedExportKindV1::Csv => {
                            quanta_gqlang_v2_syntax::interactive::sample::ExportKindV1::Csv
                        }
                        interactive::sample_compare_export::TypedExportKindV1::Sarif => {
                            quanta_gqlang_v2_syntax::interactive::sample::ExportKindV1::Sarif
                        }
                        interactive::sample_compare_export::TypedExportKindV1::Markdown => {
                            quanta_gqlang_v2_syntax::interactive::sample::ExportKindV1::Markdown
                        }
                        interactive::sample_compare_export::TypedExportKindV1::GraphMl => {
                            quanta_gqlang_v2_syntax::interactive::sample::ExportKindV1::GraphMl
                        }
                    },
                    profile: intent.profile,
                },
            ))
        }
    }
}

#[cfg(test)]
fn bind_declaration_root_v1(spec: &declarations::model_pack::TypedDeclarationRootV1) -> AstRootV1 {
    match spec {
        declarations::model_pack::TypedDeclarationRootV1::View(spec) => {
            AstRootV1::View(quanta_gqlang_v2_syntax::declarations::view::ViewDeclAstV1 {
                name: match &spec.query {
                    declarations::model_pack::TypedViewQueryV1::QueryRef(query_ref) => {
                        format!("{} ;; query-ref={query_ref}", spec.name)
                    }
                    declarations::model_pack::TypedViewQueryV1::InlineSelect(select) => {
                        format!("{} ;; body={select}", spec.name)
                    }
                },
            })
        }
        declarations::model_pack::TypedDeclarationRootV1::Rule(spec) => {
            AstRootV1::Rule(quanta_gqlang_v2_syntax::declarations::view::RuleDeclAstV1 {
                name: format!("{} ;; check-ref={}", spec.name, spec.check_ref),
            })
        }
        declarations::model_pack::TypedDeclarationRootV1::Model(spec) => AstRootV1::Model(
            quanta_gqlang_v2_syntax::declarations::view::ModelDeclAstV1 {
                name: format!(
                    "{} ;; provider={} ;; source={}",
                    spec.name, spec.provider, spec.source
                ),
            },
        ),
        declarations::model_pack::TypedDeclarationRootV1::Pack(spec) => {
            AstRootV1::Pack(quanta_gqlang_v2_syntax::declarations::pack::PackDeclAstV1 {
                name: format!(
                    "{} ;; source={} ;; activation={}",
                    spec.name,
                    spec.source,
                    match spec.activation {
                        declarations::model_pack::TypedPackActivationModeV1::Manual => "manual",
                        declarations::model_pack::TypedPackActivationModeV1::Session => "session",
                        declarations::model_pack::TypedPackActivationModeV1::Workspace =>
                            "workspace",
                    }
                ),
            })
        }
        declarations::model_pack::TypedDeclarationRootV1::Profile(spec) => AstRootV1::Profile(
            quanta_gqlang_v2_syntax::declarations::pack::ProfileDeclAstV1 {
                name: format!(
                    "{} ;; capabilities={} ;; admission={}",
                    spec.name,
                    spec.capabilities.join(","),
                    match spec.admission {
                        declarations::model_pack::TypedProfileAdmissionPolicyV1::Allowlist =>
                            "allowlist",
                        declarations::model_pack::TypedProfileAdmissionPolicyV1::ReviewRequired => {
                            "review-required"
                        }
                        declarations::model_pack::TypedProfileAdmissionPolicyV1::CertifiedOnly => {
                            "certified-only"
                        }
                    }
                ),
            },
        ),
        declarations::model_pack::TypedDeclarationRootV1::Gate(spec) => {
            AstRootV1::Gate(quanta_gqlang_v2_syntax::declarations::pack::GateDeclAstV1 {
                name: format!(
                    "{} ;; checks={} ;; binding={}",
                    spec.name,
                    spec.checks.join(","),
                    match spec.binding {
                        declarations::model_pack::TypedGateCheckBindingModeV1::All => "all",
                        declarations::model_pack::TypedGateCheckBindingModeV1::Any => "any",
                    }
                ),
            })
        }
    }
}

#[cfg(test)]
fn bind_meta_command_v1(spec: &TypedMetaCommandV1) -> MetaCommandAstV1 {
    match spec {
        TypedMetaCommandV1::UsePack {
            pack_id,
            activation_mode,
            lockfile_path,
        } => MetaCommandAstV1::UsePack {
            pack_id: pack_id.clone(),
            activation_mode: Some(activation_mode.as_str_v1().to_string()),
            lockfile_path: Some(lockfile_path.clone()),
        },
        TypedMetaCommandV1::RequireCertified { .. } => MetaCommandAstV1::RequireCertified,
    }
}

#[cfg(test)]
mod tests {
    use super::{analyze_document_v1, bind_typed_document_v1, CRATE_ID_V1, SYNTAX_CRATE_ID_V1};
    use crate::TypedRootV1;

    #[test]
    fn sema_keeps_syntax_boundary_visible() {
        assert_eq!(CRATE_ID_V1, "quanta-gqlang-v2-sema");
        assert_eq!(SYNTAX_CRATE_ID_V1, "quanta-gqlang-v2-syntax");
    }

    #[test]
    fn sema_surface_is_wired() {
        let doc = quanta_gqlang_v2_syntax::parse_document_v1("SELECT id FROM catalog.users")
            .expect("parse");
        let typed = analyze_document_v1(&doc).expect("sema");
        assert_eq!(typed.roots.len(), 1);
    }

    #[test]
    fn sema_preserves_structured_interactive_declaration_and_meta_roots() {
        let show = quanta_gqlang_v2_syntax::parse_document_v1("SHOW profiles").expect("show");
        let sample = quanta_gqlang_v2_syntax::parse_document_v1(
            "SAMPLE query=payments latency ;; scope=repo::frontend ;; mode=hybrid ;; limit=5",
        )
        .expect("sample");
        let pack = quanta_gqlang_v2_syntax::parse_document_v1(
            "PACK secure-pack ;; source=registry::security ;; activation=workspace",
        )
        .expect("pack");
        let require =
            quanta_gqlang_v2_syntax::parse_document_v1("REQUIRE CERTIFIED").expect("require");

        let show_typed = analyze_document_v1(&show).expect("show sema");
        let sample_typed = analyze_document_v1(&sample).expect("sample sema");
        let pack_typed = analyze_document_v1(&pack).expect("pack sema");
        let require_typed = analyze_document_v1(&require).expect("require sema");

        assert!(matches!(show_typed.roots[0], TypedRootV1::ShowDescribe(_)));
        assert!(matches!(
            sample_typed.roots[0],
            TypedRootV1::SampleCompareExport(_)
        ));
        assert!(matches!(pack_typed.roots[0], TypedRootV1::Declaration(_)));
        assert!(matches!(require_typed.roots[0], TypedRootV1::Meta(_)));

        let rebound = bind_typed_document_v1(&pack_typed);
        assert_eq!(rebound.roots.len(), 1);
    }
}
