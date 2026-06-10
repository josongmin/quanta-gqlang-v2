#![forbid(unsafe_code)]

pub mod ai_plan;
pub mod catalog;
pub mod check_ir;
pub mod composition_ir;
pub mod diagnostic_class;
#[path = "facts/entity_schema.rs"]
pub mod entity_schema;
pub mod explain_ir;
pub mod explain_presentation;
pub mod find_ir;
pub mod frontend;
pub mod lowering_ready;
pub mod planning;
#[path = "facts/relation_schema.rs"]
pub mod relation_schema;
pub mod retrieval_ir;
pub mod root_ir;
pub mod select_ir;
pub mod working_set;

// Inline wiring unit tests live in their own module so this root stays production-only.
#[cfg(test)]
mod lib_tests;

pub const CRATE_ID_V1: &str = "quanta-gqlang-v2-ir";

use quanta_gqlang_v2_sema::declarations::model_pack::{
    TypedDeclarationRootV1, TypedGateCheckBindingModeV1, TypedPackActivationModeV1,
    TypedProfileAdmissionPolicyV1, TypedViewQueryV1,
};
use quanta_gqlang_v2_sema::interactive::composition::{TypedCompositionV1, TypedWorkingSetBasisV1};
use quanta_gqlang_v2_sema::interactive::sample_compare_export::TypedSampleCompareExportRootV1;
use quanta_gqlang_v2_sema::interactive::sample_compare_export::{
    TypedCompareMetricV1, TypedSampleCompareExportShapeV1, TypedSampleRetrievalModeV1,
};
use quanta_gqlang_v2_sema::interactive::show_describe::TypedShowDescribeRootV1;
use quanta_gqlang_v2_sema::interactive::show_describe::{
    TypedDescribeTargetKindV1, TypedShowCategoryV1, TypedShowDescribeSpecV1,
};
pub use quanta_gqlang_v2_sema::CRATE_ID_V1 as SEMA_CRATE_ID_V1;
use quanta_gqlang_v2_sema::{
    TypedCertifiedAdmissionPolicyV1, TypedCertifiedTruthRequirementV1, TypedDocumentV1,
    TypedMetaCommandV1, TypedRootV1,
};
pub use root_ir::PublicIrRootV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrLoweringErrorV1 {
    pub message: String,
}

pub fn lower_root_to_ir_v1(root: &TypedRootV1) -> Result<PublicIrRootV1, IrLoweringErrorV1> {
    let ir = match root {
        TypedRootV1::Select(spec) => {
            PublicIrRootV1::Select(select_ir::lower_select_root_to_ir_v1(spec).map_err(|err| {
                IrLoweringErrorV1 {
                    message: format!("{}: {}", err.code, err.message),
                }
            })?)
        }
        TypedRootV1::Find(spec) => {
            let ir = find_ir::FindIrV1 {
                subkind: spec.subkind.clone(),
                seed: spec.seed.clone(),
            };
            ir.execution_plan().map_err(|err| IrLoweringErrorV1 {
                message: err.message,
            })?;
            PublicIrRootV1::Find(ir)
        }
        TypedRootV1::Check(spec) => {
            let ir = check_ir::CheckIrV1::from_typed(spec);
            ir.intent().map_err(|err| IrLoweringErrorV1 {
                message: err.message,
            })?;
            PublicIrRootV1::Check(ir)
        }
        TypedRootV1::Explain(spec) => {
            let ir = explain_ir::ExplainIrV1 {
                target: spec.target.clone(),
            };
            ir.shape().map_err(|err| IrLoweringErrorV1 {
                message: err.message,
            })?;
            PublicIrRootV1::Explain(ir)
        }
        TypedRootV1::ShowDescribe(spec) => lower_show_describe_root_to_ir_v1(spec)?,
        TypedRootV1::SampleCompareExport(spec) => lower_sample_compare_export_root_to_ir_v1(spec)?,
        TypedRootV1::Declaration(spec) => lower_declaration_root_to_ir_v1(spec),
        TypedRootV1::Meta(command) => lower_meta_command_to_ir_v1(command),
        TypedRootV1::Composition(spec) => lower_composition_root_to_ir_v1(spec)?,
    };
    Ok(ir)
}

fn lower_composition_root_to_ir_v1(
    spec: &TypedCompositionV1,
) -> Result<PublicIrRootV1, IrLoweringErrorV1> {
    let mut bindings = Vec::with_capacity(spec.bindings.len());
    for binding in &spec.bindings {
        // Reuse the per-root lowering verbatim; the producer keeps its own IR
        // (and its own degraded lane) inside the composition — nothing dropped.
        let producer = lower_root_to_ir_v1(&binding.producer)?;
        let reference = composition_ir::WorkingSetRefIrV1 {
            working_set_id: binding.name.clone(),
            kind: working_set::WorkingSetKindV1::CandidateSet,
            basis: match binding.basis {
                TypedWorkingSetBasisV1::SearchQuery => working_set::WorkingSetBasisV1::SearchQuery,
                TypedWorkingSetBasisV1::AiDiscovery => working_set::WorkingSetBasisV1::AiDiscovery,
            },
        };
        bindings.push((reference, producer));
    }
    let body = lower_root_to_ir_v1(&spec.body)?;
    Ok(PublicIrRootV1::Composition(
        composition_ir::CompositionIrV1 {
            bindings,
            narrowed_by: spec.with_index,
            body: Box::new(body),
        },
    ))
}

pub fn lower_document_to_ir_v1(
    doc: &TypedDocumentV1,
) -> Result<Vec<PublicIrRootV1>, IrLoweringErrorV1> {
    doc.roots.iter().map(lower_root_to_ir_v1).collect()
}

fn lower_show_describe_root_to_ir_v1(
    spec: &TypedShowDescribeRootV1,
) -> Result<PublicIrRootV1, IrLoweringErrorV1> {
    match spec.typed_spec().map_err(|err| IrLoweringErrorV1 {
        message: format!("{}: {}", err.code, err.message),
    })? {
        TypedShowDescribeSpecV1::Show(spec) => Ok(PublicIrRootV1::Interactive(
            root_ir::InteractiveIrV1::Show {
                category: match spec.category {
                    TypedShowCategoryV1::Capabilities => root_ir::CatalogCategoryIrV1::Capabilities,
                    TypedShowCategoryV1::Models => root_ir::CatalogCategoryIrV1::Models,
                    TypedShowCategoryV1::Views => root_ir::CatalogCategoryIrV1::Views,
                    TypedShowCategoryV1::Rules => root_ir::CatalogCategoryIrV1::Rules,
                    TypedShowCategoryV1::Packs => root_ir::CatalogCategoryIrV1::Packs,
                    TypedShowCategoryV1::Profiles => root_ir::CatalogCategoryIrV1::Profiles,
                    TypedShowCategoryV1::Gates => root_ir::CatalogCategoryIrV1::Gates,
                },
            },
        )),
        TypedShowDescribeSpecV1::Describe(spec) => Ok(PublicIrRootV1::Interactive(
            root_ir::InteractiveIrV1::Describe {
                target: root_ir::InteractiveTargetIrV1 {
                    kind: match spec.target.kind {
                        TypedDescribeTargetKindV1::Capability => {
                            root_ir::InteractiveTargetKindIrV1::Capability
                        }
                        TypedDescribeTargetKindV1::Model => {
                            root_ir::InteractiveTargetKindIrV1::Model
                        }
                        TypedDescribeTargetKindV1::View => root_ir::InteractiveTargetKindIrV1::View,
                        TypedDescribeTargetKindV1::Rule => root_ir::InteractiveTargetKindIrV1::Rule,
                        TypedDescribeTargetKindV1::Pack => root_ir::InteractiveTargetKindIrV1::Pack,
                        TypedDescribeTargetKindV1::Profile => {
                            root_ir::InteractiveTargetKindIrV1::Profile
                        }
                        TypedDescribeTargetKindV1::Gate => root_ir::InteractiveTargetKindIrV1::Gate,
                    },
                    path: spec.target.path,
                },
            },
        )),
    }
}

fn lower_sample_compare_export_root_to_ir_v1(
    spec: &TypedSampleCompareExportRootV1,
) -> Result<PublicIrRootV1, IrLoweringErrorV1> {
    match spec.shape().map_err(|err| IrLoweringErrorV1 { message: err.message })? {
        TypedSampleCompareExportShapeV1::Sample(intent) => Ok(PublicIrRootV1::Retrieval(
            retrieval_ir::RetrievalIrV1 {
                query: intent.query,
                scope: intent.scope.map(|scope| retrieval_ir::InteractiveScopeIrV1 {
                    kind: scope.kind,
                    locator: scope.locator,
                }),
                mode: match intent.mode {
                    TypedSampleRetrievalModeV1::Lexical => retrieval_ir::RetrievalModeIrV1::Lexical,
                    TypedSampleRetrievalModeV1::Semantic => retrieval_ir::RetrievalModeIrV1::Semantic,
                    TypedSampleRetrievalModeV1::Hybrid => retrieval_ir::RetrievalModeIrV1::Hybrid,
                },
                limit: intent.limit,
            },
        )),
        TypedSampleCompareExportShapeV1::Compare(intent) => Ok(PublicIrRootV1::Interactive(
            root_ir::InteractiveIrV1::Compare {
                baseline: root_ir::InteractiveRefIrV1 {
                    kind: intent.baseline.kind,
                    locator: intent.baseline.locator,
                },
                candidate: root_ir::InteractiveRefIrV1 {
                    kind: intent.candidate.kind,
                    locator: intent.candidate.locator,
                },
                metric: match intent.metric {
                    TypedCompareMetricV1::Exact => root_ir::CompareMetricIrV1::Exact,
                    TypedCompareMetricV1::Structural => root_ir::CompareMetricIrV1::Structural,
                    TypedCompareMetricV1::Semantic => root_ir::CompareMetricIrV1::Semantic,
                },
            },
        )),
        TypedSampleCompareExportShapeV1::Export(intent) => Ok(PublicIrRootV1::Interactive(
            root_ir::InteractiveIrV1::Export {
                source: root_ir::InteractiveRefIrV1 {
                    kind: intent.source.kind,
                    locator: intent.source.locator,
                },
                kind: match intent.kind {
                    quanta_gqlang_v2_sema::interactive::sample_compare_export::TypedExportKindV1::Json => {
                        root_ir::ExportKindIrV1::Json
                    }
                    quanta_gqlang_v2_sema::interactive::sample_compare_export::TypedExportKindV1::Csv => {
                        root_ir::ExportKindIrV1::Csv
                    }
                    quanta_gqlang_v2_sema::interactive::sample_compare_export::TypedExportKindV1::Sarif => {
                        root_ir::ExportKindIrV1::Sarif
                    }
                    quanta_gqlang_v2_sema::interactive::sample_compare_export::TypedExportKindV1::Markdown => {
                        root_ir::ExportKindIrV1::Markdown
                    }
                    quanta_gqlang_v2_sema::interactive::sample_compare_export::TypedExportKindV1::GraphMl => {
                        root_ir::ExportKindIrV1::GraphMl
                    }
                },
                profile: intent.profile,
            },
        )),
    }
}

fn lower_declaration_root_to_ir_v1(spec: &TypedDeclarationRootV1) -> PublicIrRootV1 {
    let ir = match spec {
        TypedDeclarationRootV1::View(spec) => root_ir::DeclarationIrV1::View {
            name: spec.name.clone(),
            query: match &spec.query {
                TypedViewQueryV1::QueryRef(query_ref) => {
                    root_ir::ViewQueryIrV1::QueryRef(query_ref.clone())
                }
                TypedViewQueryV1::InlineSelect(query_text) => {
                    root_ir::ViewQueryIrV1::InlineSelect(query_text.clone())
                }
            },
        },
        TypedDeclarationRootV1::Rule(spec) => root_ir::DeclarationIrV1::Rule {
            name: spec.name.clone(),
            check_ref: spec.check_ref.clone(),
        },
        TypedDeclarationRootV1::Model(spec) => root_ir::DeclarationIrV1::Model {
            name: spec.name.clone(),
            provider: spec.provider.clone(),
            source: spec.source.clone(),
        },
        TypedDeclarationRootV1::Pack(spec) => root_ir::DeclarationIrV1::Pack {
            name: spec.name.clone(),
            source: spec.source.clone(),
            activation: match spec.activation {
                TypedPackActivationModeV1::Manual => root_ir::PackActivationIrV1::Manual,
                TypedPackActivationModeV1::Session => root_ir::PackActivationIrV1::Session,
                TypedPackActivationModeV1::Workspace => root_ir::PackActivationIrV1::Workspace,
            },
        },
        TypedDeclarationRootV1::Profile(spec) => root_ir::DeclarationIrV1::Profile {
            name: spec.name.clone(),
            capabilities: spec.capabilities.clone(),
            admission: match spec.admission {
                TypedProfileAdmissionPolicyV1::Allowlist => {
                    root_ir::ProfileAdmissionIrV1::Allowlist
                }
                TypedProfileAdmissionPolicyV1::ReviewRequired => {
                    root_ir::ProfileAdmissionIrV1::ReviewRequired
                }
                TypedProfileAdmissionPolicyV1::CertifiedOnly => {
                    root_ir::ProfileAdmissionIrV1::CertifiedOnly
                }
            },
        },
        TypedDeclarationRootV1::Gate(spec) => root_ir::DeclarationIrV1::Gate {
            name: spec.name.clone(),
            checks: spec.checks.clone(),
            binding: match spec.binding {
                TypedGateCheckBindingModeV1::All => root_ir::GateBindingIrV1::All,
                TypedGateCheckBindingModeV1::Any => root_ir::GateBindingIrV1::Any,
            },
        },
    };
    PublicIrRootV1::Declaration(ir)
}

fn lower_meta_command_to_ir_v1(command: &TypedMetaCommandV1) -> PublicIrRootV1 {
    PublicIrRootV1::Meta(match command {
        TypedMetaCommandV1::UsePack {
            pack_id,
            activation_mode,
            lockfile_path,
        } => root_ir::MetaIrV1::UsePack {
            pack_id: pack_id.clone(),
            activation_mode: match activation_mode {
                quanta_gqlang_v2_sema::TypedPackActivationModeV1::Session => {
                    root_ir::PackActivationIrV1::Session
                }
                quanta_gqlang_v2_sema::TypedPackActivationModeV1::Workspace => {
                    root_ir::PackActivationIrV1::Workspace
                }
            },
            lockfile_path: lockfile_path.clone(),
        },
        TypedMetaCommandV1::RequireCertified {
            truth_requirement,
            admission_policy,
        } => root_ir::MetaIrV1::RequireCertified {
            truth_requirement: match truth_requirement {
                TypedCertifiedTruthRequirementV1::CertifiedOnly => {
                    root_ir::CertifiedTruthRequirementIrV1::CertifiedOnly
                }
            },
            admission_policy: match admission_policy {
                TypedCertifiedAdmissionPolicyV1::FailClosed => {
                    root_ir::CertifiedAdmissionPolicyIrV1::FailClosed
                }
            },
        },
    })
}
