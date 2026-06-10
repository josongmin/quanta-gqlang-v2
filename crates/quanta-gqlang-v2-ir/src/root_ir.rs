use crate::check_ir::CheckIrV1;
use crate::composition_ir::CompositionIrV1;
use crate::explain_ir::ExplainIrV1;
use crate::find_ir::FindIrV1;
use crate::planning::PublicPlanRootKindV1;
use crate::retrieval_ir::RetrievalIrV1;
use crate::select_ir::SelectIrV1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicIrRootV1 {
    Select(SelectIrV1),
    Find(FindIrV1),
    Check(CheckIrV1),
    Explain(ExplainIrV1),
    Retrieval(RetrievalIrV1),
    Interactive(InteractiveIrV1),
    Declaration(DeclarationIrV1),
    Meta(MetaIrV1),
    Composition(CompositionIrV1),
}

impl PublicIrRootV1 {
    /// The public plan root-kind this lowered root collapses to. This is the
    /// single authority for the 17-family → 8-kind mapping; every consumer
    /// (lowering-ready validation, the conformance registry) routes through it.
    #[must_use]
    pub fn plan_root_kind_v1(&self) -> PublicPlanRootKindV1 {
        match self {
            Self::Select(_) => PublicPlanRootKindV1::Select,
            Self::Find(_) => PublicPlanRootKindV1::Find,
            Self::Check(_) => PublicPlanRootKindV1::Check,
            Self::Explain(_) => PublicPlanRootKindV1::Explain,
            Self::Retrieval(_) => PublicPlanRootKindV1::Retrieval,
            Self::Interactive(_) => PublicPlanRootKindV1::Interactive,
            Self::Declaration(_) => PublicPlanRootKindV1::Declaration,
            Self::Meta(_) => PublicPlanRootKindV1::Meta,
            Self::Composition(_) => PublicPlanRootKindV1::Composition,
        }
    }
}

/// The read-only shell's single "why no result" label, surfaced by every
/// front-door producer (CLI headless/doctor/explain, LSP explain) so the wording
/// cannot desync between them. Presentation-only — it asserts no runtime truth.
/// This is the single source of truth; the CLI and LSP re-export it through
/// `ir::frontend` rather than copy-pasting the literal.
pub const NO_RESULT_REASON_V1: &str = "read-only-shell-has-no-execution-backend";

/// The read-only-shell execution lane a lowered root cannot materialize. This is
/// the single source of truth for the degraded-reason codes — previously the same
/// five strings were copy-pasted across the CLI doctor/headless producers (and the
/// explain-view consumers), with `_ => None` fallthrough arms that silently
/// dropped new roots.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DegradedReasonV1 {
    RuntimeHostRoutingUnavailable,
    SemanticConfirmLaneUnavailable,
    TruthEvidenceMaterializationUnavailable,
    RetrievalResultMaterializationUnavailable,
    CatalogAndInteractiveHostTruthUnavailable,
    CompositionMaterializationUnavailable,
}

impl DegradedReasonV1 {
    #[must_use]
    pub const fn code_v1(self) -> &'static str {
        match self {
            Self::RuntimeHostRoutingUnavailable => "runtime-host-routing-unavailable",
            Self::SemanticConfirmLaneUnavailable => "semantic-confirm-lane-unavailable",
            Self::TruthEvidenceMaterializationUnavailable => {
                "truth-evidence-materialization-unavailable"
            }
            Self::RetrievalResultMaterializationUnavailable => {
                "retrieval-result-materialization-unavailable"
            }
            Self::CatalogAndInteractiveHostTruthUnavailable => {
                "catalog-and-interactive-host-truth-unavailable"
            }
            Self::CompositionMaterializationUnavailable => {
                "composition-materialization-unavailable"
            }
        }
    }

    /// Exhaustive over `PublicIrRootV1` (no catch-all) so a new IR root forces a
    /// deliberate degraded-lane decision here instead of silently degrading to
    /// "no reason".
    #[must_use]
    pub fn for_ir_root_v1(root: &PublicIrRootV1) -> Option<Self> {
        match root {
            PublicIrRootV1::Find(_) => Some(Self::RuntimeHostRoutingUnavailable),
            PublicIrRootV1::Check(_) => Some(Self::SemanticConfirmLaneUnavailable),
            PublicIrRootV1::Explain(_) => Some(Self::TruthEvidenceMaterializationUnavailable),
            PublicIrRootV1::Retrieval(_) => Some(Self::RetrievalResultMaterializationUnavailable),
            PublicIrRootV1::Interactive(_) => Some(Self::CatalogAndInteractiveHostTruthUnavailable),
            PublicIrRootV1::Composition(_) => Some(Self::CompositionMaterializationUnavailable),
            PublicIrRootV1::Select(_)
            | PublicIrRootV1::Declaration(_)
            | PublicIrRootV1::Meta(_) => None,
        }
    }

    #[must_use]
    pub fn from_code_v1(code: &str) -> Option<Self> {
        match code {
            "runtime-host-routing-unavailable" => Some(Self::RuntimeHostRoutingUnavailable),
            "semantic-confirm-lane-unavailable" => Some(Self::SemanticConfirmLaneUnavailable),
            "truth-evidence-materialization-unavailable" => {
                Some(Self::TruthEvidenceMaterializationUnavailable)
            }
            "retrieval-result-materialization-unavailable" => {
                Some(Self::RetrievalResultMaterializationUnavailable)
            }
            "catalog-and-interactive-host-truth-unavailable" => {
                Some(Self::CatalogAndInteractiveHostTruthUnavailable)
            }
            "composition-materialization-unavailable" => {
                Some(Self::CompositionMaterializationUnavailable)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InteractiveIrV1 {
    Show {
        category: CatalogCategoryIrV1,
    },
    Describe {
        target: InteractiveTargetIrV1,
    },
    Compare {
        baseline: InteractiveRefIrV1,
        candidate: InteractiveRefIrV1,
        metric: CompareMetricIrV1,
    },
    Export {
        source: InteractiveRefIrV1,
        kind: ExportKindIrV1,
        profile: Option<String>,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CatalogCategoryIrV1 {
    Capabilities,
    Models,
    Views,
    Rules,
    Packs,
    Profiles,
    Gates,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InteractiveTargetIrV1 {
    pub kind: InteractiveTargetKindIrV1,
    pub path: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum InteractiveTargetKindIrV1 {
    Capability,
    Model,
    View,
    Rule,
    Pack,
    Profile,
    Gate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InteractiveRefIrV1 {
    pub kind: String,
    pub locator: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CompareMetricIrV1 {
    Exact,
    Structural,
    Semantic,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExportKindIrV1 {
    Json,
    Csv,
    Sarif,
    Markdown,
    GraphMl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeclarationIrV1 {
    View {
        name: String,
        query: ViewQueryIrV1,
    },
    Rule {
        name: String,
        check_ref: String,
    },
    Model {
        name: String,
        provider: String,
        source: String,
    },
    Pack {
        name: String,
        source: String,
        activation: PackActivationIrV1,
    },
    Profile {
        name: String,
        capabilities: Vec<String>,
        admission: ProfileAdmissionIrV1,
    },
    Gate {
        name: String,
        checks: Vec<String>,
        binding: GateBindingIrV1,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewQueryIrV1 {
    QueryRef(String),
    InlineSelect(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PackActivationIrV1 {
    Manual,
    Session,
    Workspace,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ProfileAdmissionIrV1 {
    Allowlist,
    ReviewRequired,
    CertifiedOnly,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GateBindingIrV1 {
    All,
    Any,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MetaIrV1 {
    UsePack {
        pack_id: String,
        activation_mode: PackActivationIrV1,
        lockfile_path: String,
    },
    RequireCertified {
        truth_requirement: CertifiedTruthRequirementIrV1,
        admission_policy: CertifiedAdmissionPolicyIrV1,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CertifiedTruthRequirementIrV1 {
    CertifiedOnly,
}

impl CertifiedTruthRequirementIrV1 {
    pub const fn as_str_v1(self) -> &'static str {
        match self {
            Self::CertifiedOnly => "certified-only",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CertifiedAdmissionPolicyIrV1 {
    FailClosed,
}

impl CertifiedAdmissionPolicyIrV1 {
    pub const fn as_str_v1(self) -> &'static str {
        match self {
            Self::FailClosed => "fail-closed",
        }
    }
}
