use crate::check_ir::CheckIrV1;
use crate::explain_ir::ExplainIrV1;
use crate::find_ir::FindIrV1;
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
