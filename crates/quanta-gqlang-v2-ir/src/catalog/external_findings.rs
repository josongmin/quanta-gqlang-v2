use super::metric_inventory::{
    metric_descriptor_v1, MetricCategoryV1, MetricDescriptorV1, MetricIdV1,
    MetricInventoryValidationErrorV1, MetricScalarKindV1,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExternalFindingIdV1 {
    CodeQlPathProblem,
    SemgrepRuleMatch,
    ClippyLint,
    CargoAuditAdvisory,
}

impl ExternalFindingIdV1 {
    #[must_use]
    pub const fn as_str_v1(self) -> &'static str {
        match self {
            Self::CodeQlPathProblem => "codeql_path_problem",
            Self::SemgrepRuleMatch => "semgrep_rule_match",
            Self::ClippyLint => "clippy_lint",
            Self::CargoAuditAdvisory => "cargo_audit_advisory",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExternalFindingProviderV1 {
    CodeQl,
    Semgrep,
    Clippy,
    CargoAudit,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExternalFindingKindV1 {
    PathProblem,
    RuleMatch,
    Lint,
    Advisory,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExternalFindingSeverityV1 {
    Informational,
    Low,
    Medium,
    High,
    Critical,
}

impl ExternalFindingSeverityV1 {
    #[must_use]
    pub const fn is_high_signal_v1(self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExternalFindingProvenanceV1 {
    SarifImport,
    NativeDiagnostic,
    AdvisoryFeed,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExternalFindingProviderSeverityV1 {
    CodeQlError,
    CodeQlWarning,
    CodeQlRecommendation,
    SemgrepInfo,
    SemgrepWarning,
    SemgrepError,
    ClippyWarning,
    ClippyError,
    CargoAuditLow,
    CargoAuditModerate,
    CargoAuditHigh,
    CargoAuditCritical,
}

impl ExternalFindingProviderSeverityV1 {
    #[must_use]
    pub const fn provider_v1(self) -> ExternalFindingProviderV1 {
        match self {
            Self::CodeQlError | Self::CodeQlWarning | Self::CodeQlRecommendation => {
                ExternalFindingProviderV1::CodeQl
            }
            Self::SemgrepInfo | Self::SemgrepWarning | Self::SemgrepError => {
                ExternalFindingProviderV1::Semgrep
            }
            Self::ClippyWarning | Self::ClippyError => ExternalFindingProviderV1::Clippy,
            Self::CargoAuditLow
            | Self::CargoAuditModerate
            | Self::CargoAuditHigh
            | Self::CargoAuditCritical => ExternalFindingProviderV1::CargoAudit,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ExternalFindingSeverityMappingV1 {
    pub provider_severity: ExternalFindingProviderSeverityV1,
    pub normalized_severity: ExternalFindingSeverityV1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ExternalFindingDescriptorV1 {
    pub id: ExternalFindingIdV1,
    pub provider: ExternalFindingProviderV1,
    pub kind: ExternalFindingKindV1,
    pub provenance: ExternalFindingProvenanceV1,
    pub default_severity: ExternalFindingSeverityV1,
    pub count_metric_id: MetricIdV1,
    pub elevated_count_metric_id: Option<MetricIdV1>,
    pub suppressed_count_metric_id: Option<MetricIdV1>,
    pub severity_mappings: &'static [ExternalFindingSeverityMappingV1],
}

impl ExternalFindingDescriptorV1 {
    fn validate_v1(&self) -> Result<(), ExternalFindingValidationErrorV1> {
        validate_external_metric_v1(self.count_metric_id)?;

        if let Some(metric_id) = self.elevated_count_metric_id {
            validate_external_metric_v1(metric_id)?;
        }

        if let Some(metric_id) = self.suppressed_count_metric_id {
            validate_external_metric_v1(metric_id)?;
        }

        if self.severity_mappings.is_empty() {
            return Err(ExternalFindingValidationErrorV1::new_v1(
                "external_finding_missing_severity_mappings",
                "External finding rows must declare provider-to-normalized severity mappings",
            ));
        }

        let mut has_default_severity = false;
        let mut has_high_signal = false;
        let mut left = 0;
        while let Some(mapping) = self.severity_mappings.get(left).copied() {
            if mapping.provider_severity.provider_v1() != self.provider {
                return Err(ExternalFindingValidationErrorV1::new_v1(
                    "external_finding_provider_severity_mismatch",
                    "Provider severity mapping must belong to the row provider",
                ));
            }
            if mapping.normalized_severity == self.default_severity {
                has_default_severity = true;
            }
            if mapping.normalized_severity.is_high_signal_v1() {
                has_high_signal = true;
            }

            let mut right = left + 1;
            while let Some(other_mapping) = self.severity_mappings.get(right).copied() {
                if mapping.provider_severity == other_mapping.provider_severity {
                    return Err(ExternalFindingValidationErrorV1::new_v1(
                        "external_finding_duplicate_provider_severity",
                        "Provider severity mappings must stay unique within a row",
                    ));
                }
                right += 1;
            }

            left += 1;
        }

        if !has_default_severity {
            return Err(ExternalFindingValidationErrorV1::new_v1(
                "external_finding_default_severity_unmapped",
                "Default severity must be represented by at least one provider severity mapping",
            ));
        }

        if has_high_signal && self.elevated_count_metric_id.is_none() {
            return Err(ExternalFindingValidationErrorV1::new_v1(
                "external_finding_missing_elevated_metric",
                "Rows with high-signal severities must name an elevated-count metric",
            ));
        }

        Ok(())
    }
}

const CODEQL_SEVERITY_MAPPINGS_V1: [ExternalFindingSeverityMappingV1; 3] = [
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::CodeQlError,
        normalized_severity: ExternalFindingSeverityV1::High,
    },
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::CodeQlWarning,
        normalized_severity: ExternalFindingSeverityV1::Medium,
    },
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::CodeQlRecommendation,
        normalized_severity: ExternalFindingSeverityV1::Low,
    },
];

const SEMGREP_SEVERITY_MAPPINGS_V1: [ExternalFindingSeverityMappingV1; 3] = [
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::SemgrepInfo,
        normalized_severity: ExternalFindingSeverityV1::Informational,
    },
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::SemgrepWarning,
        normalized_severity: ExternalFindingSeverityV1::Medium,
    },
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::SemgrepError,
        normalized_severity: ExternalFindingSeverityV1::High,
    },
];

const CLIPPY_SEVERITY_MAPPINGS_V1: [ExternalFindingSeverityMappingV1; 2] = [
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::ClippyWarning,
        normalized_severity: ExternalFindingSeverityV1::Medium,
    },
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::ClippyError,
        normalized_severity: ExternalFindingSeverityV1::High,
    },
];

const CARGO_AUDIT_SEVERITY_MAPPINGS_V1: [ExternalFindingSeverityMappingV1; 4] = [
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::CargoAuditLow,
        normalized_severity: ExternalFindingSeverityV1::Low,
    },
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::CargoAuditModerate,
        normalized_severity: ExternalFindingSeverityV1::Medium,
    },
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::CargoAuditHigh,
        normalized_severity: ExternalFindingSeverityV1::High,
    },
    ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::CargoAuditCritical,
        normalized_severity: ExternalFindingSeverityV1::Critical,
    },
];

const EXTERNAL_FINDING_ROWS_V1: [ExternalFindingDescriptorV1; 4] = [
    ExternalFindingDescriptorV1 {
        id: ExternalFindingIdV1::CodeQlPathProblem,
        provider: ExternalFindingProviderV1::CodeQl,
        kind: ExternalFindingKindV1::PathProblem,
        provenance: ExternalFindingProvenanceV1::SarifImport,
        default_severity: ExternalFindingSeverityV1::High,
        count_metric_id: MetricIdV1::ExternalFindingCount,
        elevated_count_metric_id: Some(MetricIdV1::ExternalHighSeverityCount),
        suppressed_count_metric_id: Some(MetricIdV1::SuppressedFindingCount),
        severity_mappings: &CODEQL_SEVERITY_MAPPINGS_V1,
    },
    ExternalFindingDescriptorV1 {
        id: ExternalFindingIdV1::SemgrepRuleMatch,
        provider: ExternalFindingProviderV1::Semgrep,
        kind: ExternalFindingKindV1::RuleMatch,
        provenance: ExternalFindingProvenanceV1::SarifImport,
        default_severity: ExternalFindingSeverityV1::Medium,
        count_metric_id: MetricIdV1::ExternalFindingCount,
        elevated_count_metric_id: Some(MetricIdV1::ExternalHighSeverityCount),
        suppressed_count_metric_id: Some(MetricIdV1::SuppressedFindingCount),
        severity_mappings: &SEMGREP_SEVERITY_MAPPINGS_V1,
    },
    ExternalFindingDescriptorV1 {
        id: ExternalFindingIdV1::ClippyLint,
        provider: ExternalFindingProviderV1::Clippy,
        kind: ExternalFindingKindV1::Lint,
        provenance: ExternalFindingProvenanceV1::NativeDiagnostic,
        default_severity: ExternalFindingSeverityV1::Medium,
        count_metric_id: MetricIdV1::ExternalFindingCount,
        elevated_count_metric_id: Some(MetricIdV1::ExternalHighSeverityCount),
        suppressed_count_metric_id: Some(MetricIdV1::SuppressedFindingCount),
        severity_mappings: &CLIPPY_SEVERITY_MAPPINGS_V1,
    },
    ExternalFindingDescriptorV1 {
        id: ExternalFindingIdV1::CargoAuditAdvisory,
        provider: ExternalFindingProviderV1::CargoAudit,
        kind: ExternalFindingKindV1::Advisory,
        provenance: ExternalFindingProvenanceV1::AdvisoryFeed,
        default_severity: ExternalFindingSeverityV1::High,
        count_metric_id: MetricIdV1::ExternalFindingCount,
        elevated_count_metric_id: Some(MetricIdV1::ExternalHighSeverityCount),
        suppressed_count_metric_id: None,
        severity_mappings: &CARGO_AUDIT_SEVERITY_MAPPINGS_V1,
    },
];

pub const EXTERNAL_FINDING_INVENTORY_V1: &[ExternalFindingDescriptorV1] = &EXTERNAL_FINDING_ROWS_V1;

#[must_use]
pub const fn external_finding_rows_v1() -> &'static [ExternalFindingDescriptorV1] {
    EXTERNAL_FINDING_INVENTORY_V1
}

#[must_use]
pub fn external_finding_descriptor_v1(
    id: ExternalFindingIdV1,
) -> Option<&'static ExternalFindingDescriptorV1> {
    let mut index = 0;
    while let Some(row) = EXTERNAL_FINDING_ROWS_V1.get(index) {
        if row.id == id {
            return Some(row);
        }
        index += 1;
    }
    None
}

pub fn validate_external_finding_inventory_v1(
    rows: &[ExternalFindingDescriptorV1],
) -> Result<(), ExternalFindingValidationErrorV1> {
    validate_metric_inventory_dependency_v1()?;

    if rows.is_empty() {
        return Err(ExternalFindingValidationErrorV1::new_v1(
            "external_finding_inventory_empty",
            "External finding inventory must expose at least one public row",
        ));
    }

    let mut left = 0;
    while let Some(left_row) = rows.get(left) {
        left_row.validate_v1()?;
        let mut right = left + 1;
        while let Some(right_row) = rows.get(right) {
            if left_row.id == right_row.id {
                return Err(ExternalFindingValidationErrorV1::new_v1(
                    "external_finding_duplicate_id",
                    "External finding ids must remain unique",
                ));
            }
            right += 1;
        }
        left += 1;
    }

    Ok(())
}

fn validate_metric_inventory_dependency_v1() -> Result<(), ExternalFindingValidationErrorV1> {
    super::metric_inventory::validate_metric_inventory_v1(
        super::metric_inventory::metric_inventory_rows_v1(),
    )
    .map_err(ExternalFindingValidationErrorV1::from)
}

fn validate_external_metric_v1(
    metric_id: MetricIdV1,
) -> Result<(), ExternalFindingValidationErrorV1> {
    let descriptor = metric_descriptor_v1(metric_id).ok_or_else(|| {
        ExternalFindingValidationErrorV1::new_v1(
            "external_finding_unknown_metric_id",
            "External finding rows must reference known metric ids",
        )
    })?;
    ensure_external_metric_shape_v1(descriptor)
}

fn ensure_external_metric_shape_v1(
    descriptor: &MetricDescriptorV1,
) -> Result<(), ExternalFindingValidationErrorV1> {
    if !matches!(descriptor.category, MetricCategoryV1::ExternalFindings) {
        return Err(ExternalFindingValidationErrorV1::new_v1(
            "external_finding_non_external_metric",
            "External finding rows may only reference external-findings metrics",
        ));
    }
    if !matches!(descriptor.scalar_kind, MetricScalarKindV1::Count) {
        return Err(ExternalFindingValidationErrorV1::new_v1(
            "external_finding_non_count_metric",
            "External finding rows may only reference count metrics",
        ));
    }
    Ok(())
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ExternalFindingValidationErrorV1 {
    pub code: &'static str,
    pub message: &'static str,
}

impl ExternalFindingValidationErrorV1 {
    const fn new_v1(code: &'static str, message: &'static str) -> Self {
        Self { code, message }
    }
}

impl From<MetricInventoryValidationErrorV1> for ExternalFindingValidationErrorV1 {
    fn from(value: MetricInventoryValidationErrorV1) -> Self {
        Self {
            code: value.code,
            message: value.message,
        }
    }
}
