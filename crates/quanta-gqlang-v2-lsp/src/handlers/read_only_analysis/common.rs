use super::super::{ReadOnlyRootSummaryV1, SymbolItemV1};
use quanta_gqlang_v2_ir::frontend::{
    CertifiedAdmissionPolicyAstV1, CertifiedTruthRequirementAstV1, PackActivationModeAstV1,
};

pub(super) fn root_summary_v1(
    root_keyword: &'static str,
    category: &'static str,
    typed_target: String,
    summary: Vec<String>,
) -> ReadOnlyRootSummaryV1 {
    let symbols = vec![
        document_symbol_v1(),
        root_symbol_v1(root_keyword, category),
        SymbolItemV1 {
            name: typed_target.clone(),
            kind: "target".to_string(),
            detail: Some(format!("{}.target", category)),
        },
    ];
    ReadOnlyRootSummaryV1 {
        root_keyword,
        category,
        typed_target,
        summary,
        symbols,
    }
}

pub(super) fn document_symbol_v1() -> SymbolItemV1 {
    SymbolItemV1 {
        name: "document".to_string(),
        kind: "document".to_string(),
        detail: Some("roots=1".to_string()),
    }
}

pub(super) fn root_symbol_v1(keyword: &str, category: &str) -> SymbolItemV1 {
    SymbolItemV1 {
        name: keyword.to_string(),
        kind: "root".to_string(),
        detail: Some(format!("category={category}")),
    }
}

pub(super) fn declaration_summary_v1(
    keyword: &'static str,
    name: &str,
    summary: Vec<String>,
) -> ReadOnlyRootSummaryV1 {
    ReadOnlyRootSummaryV1 {
        root_keyword: keyword,
        category: "declaration",
        typed_target: name.to_string(),
        symbols: vec![
            SymbolItemV1 {
                name: "document".to_string(),
                kind: "document".to_string(),
                detail: Some("roots=1".to_string()),
            },
            SymbolItemV1 {
                name: keyword.to_string(),
                kind: "root".to_string(),
                detail: Some("category=declaration".to_string()),
            },
            SymbolItemV1 {
                name: name.to_string(),
                kind: "name".to_string(),
                detail: Some(keyword.to_ascii_lowercase()),
            },
        ],
        summary,
    }
}

pub(super) fn pack_activation_mode_summary_v1(mode: PackActivationModeAstV1) -> &'static str {
    match mode {
        PackActivationModeAstV1::Session => "session",
        PackActivationModeAstV1::Workspace => "workspace",
    }
}

pub(super) fn certified_truth_summary_v1(value: CertifiedTruthRequirementAstV1) -> &'static str {
    match value {
        CertifiedTruthRequirementAstV1::CertifiedOnly => "certified-only",
    }
}

pub(super) fn certified_admission_summary_v1(value: CertifiedAdmissionPolicyAstV1) -> &'static str {
    match value {
        CertifiedAdmissionPolicyAstV1::FailClosed => "fail-closed",
    }
}
