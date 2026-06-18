//! E11C — the public metric inventory vocabulary and the `metric.<name>` typed
//! expression surface for SELECT.
//!
//! These metrics are host-ingested (MetricsHost), not analyzer-computed here;
//! this module owns only the named vocabulary (grouped by family) and the
//! fail-closed resolution of `metric.<name>` projection targets. It lives in
//! sema (not ir) so SELECT analysis can validate metric projections; ir
//! re-exports it for the catalog surface.

use crate::roots::select::{TypedSelectProjectionV1, TypedSelectRootV1};
use crate::SemaDiagnosticV1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetricFamilyV1 {
    Coverage,
    Duplication,
    Complexity,
    Size,
    TestExecution,
    IssueCounts,
    Ratings,
    Remediation,
    HotspotReview,
}

/// One named metric and its family. The vocabulary is the single source of
/// truth; `metric.<name>` resolves iff `name` appears here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MetricDescriptorV1 {
    pub name: &'static str,
    pub family: MetricFamilyV1,
}

/// The full E11C named metric inventory (nine families).
pub const EXTERNAL_METRIC_VOCABULARY_V1: &[MetricDescriptorV1] = &[
    // Coverage
    MetricDescriptorV1 {
        name: "coverage",
        family: MetricFamilyV1::Coverage,
    },
    MetricDescriptorV1 {
        name: "new_coverage",
        family: MetricFamilyV1::Coverage,
    },
    MetricDescriptorV1 {
        name: "branch_coverage",
        family: MetricFamilyV1::Coverage,
    },
    MetricDescriptorV1 {
        name: "new_branch_coverage",
        family: MetricFamilyV1::Coverage,
    },
    MetricDescriptorV1 {
        name: "lines_to_cover",
        family: MetricFamilyV1::Coverage,
    },
    MetricDescriptorV1 {
        name: "uncovered_lines",
        family: MetricFamilyV1::Coverage,
    },
    MetricDescriptorV1 {
        name: "conditions_to_cover",
        family: MetricFamilyV1::Coverage,
    },
    MetricDescriptorV1 {
        name: "uncovered_conditions",
        family: MetricFamilyV1::Coverage,
    },
    // Duplication
    MetricDescriptorV1 {
        name: "duplicated_lines_density",
        family: MetricFamilyV1::Duplication,
    },
    MetricDescriptorV1 {
        name: "new_duplicated_lines_density",
        family: MetricFamilyV1::Duplication,
    },
    MetricDescriptorV1 {
        name: "duplicated_lines",
        family: MetricFamilyV1::Duplication,
    },
    MetricDescriptorV1 {
        name: "duplicated_blocks",
        family: MetricFamilyV1::Duplication,
    },
    MetricDescriptorV1 {
        name: "duplicated_files",
        family: MetricFamilyV1::Duplication,
    },
    // Complexity
    MetricDescriptorV1 {
        name: "complexity",
        family: MetricFamilyV1::Complexity,
    },
    MetricDescriptorV1 {
        name: "cognitive_complexity",
        family: MetricFamilyV1::Complexity,
    },
    // Size
    MetricDescriptorV1 {
        name: "ncloc",
        family: MetricFamilyV1::Size,
    },
    MetricDescriptorV1 {
        name: "lines",
        family: MetricFamilyV1::Size,
    },
    MetricDescriptorV1 {
        name: "functions",
        family: MetricFamilyV1::Size,
    },
    MetricDescriptorV1 {
        name: "statements",
        family: MetricFamilyV1::Size,
    },
    // TestExecution
    MetricDescriptorV1 {
        name: "tests",
        family: MetricFamilyV1::TestExecution,
    },
    MetricDescriptorV1 {
        name: "test_failures",
        family: MetricFamilyV1::TestExecution,
    },
    MetricDescriptorV1 {
        name: "test_errors",
        family: MetricFamilyV1::TestExecution,
    },
    MetricDescriptorV1 {
        name: "skipped_tests",
        family: MetricFamilyV1::TestExecution,
    },
    MetricDescriptorV1 {
        name: "test_execution_time",
        family: MetricFamilyV1::TestExecution,
    },
    // IssueCounts
    MetricDescriptorV1 {
        name: "bugs",
        family: MetricFamilyV1::IssueCounts,
    },
    MetricDescriptorV1 {
        name: "new_bugs",
        family: MetricFamilyV1::IssueCounts,
    },
    MetricDescriptorV1 {
        name: "vulnerabilities",
        family: MetricFamilyV1::IssueCounts,
    },
    MetricDescriptorV1 {
        name: "new_vulnerabilities",
        family: MetricFamilyV1::IssueCounts,
    },
    MetricDescriptorV1 {
        name: "code_smells",
        family: MetricFamilyV1::IssueCounts,
    },
    MetricDescriptorV1 {
        name: "new_code_smells",
        family: MetricFamilyV1::IssueCounts,
    },
    // Ratings
    MetricDescriptorV1 {
        name: "reliability_rating",
        family: MetricFamilyV1::Ratings,
    },
    MetricDescriptorV1 {
        name: "security_rating",
        family: MetricFamilyV1::Ratings,
    },
    MetricDescriptorV1 {
        name: "maintainability_rating",
        family: MetricFamilyV1::Ratings,
    },
    // Remediation
    MetricDescriptorV1 {
        name: "security_remediation_effort",
        family: MetricFamilyV1::Remediation,
    },
    MetricDescriptorV1 {
        name: "reliability_remediation_effort",
        family: MetricFamilyV1::Remediation,
    },
    MetricDescriptorV1 {
        name: "maintainability_remediation_effort",
        family: MetricFamilyV1::Remediation,
    },
    // HotspotReview
    MetricDescriptorV1 {
        name: "security_hotspots",
        family: MetricFamilyV1::HotspotReview,
    },
    MetricDescriptorV1 {
        name: "new_security_hotspots",
        family: MetricFamilyV1::HotspotReview,
    },
    MetricDescriptorV1 {
        name: "security_hotspots_reviewed",
        family: MetricFamilyV1::HotspotReview,
    },
    MetricDescriptorV1 {
        name: "new_security_hotspots_reviewed",
        family: MetricFamilyV1::HotspotReview,
    },
    MetricDescriptorV1 {
        name: "security_review_rating",
        family: MetricFamilyV1::HotspotReview,
    },
    MetricDescriptorV1 {
        name: "new_security_review_rating",
        family: MetricFamilyV1::HotspotReview,
    },
];

/// Resolve a `metric.<name>` target.
#[must_use]
pub fn metric_descriptor_v1(name: &str) -> Option<&'static MetricDescriptorV1> {
    EXTERNAL_METRIC_VOCABULARY_V1
        .iter()
        .find(|descriptor| descriptor.name == name)
}

/// Whether a SELECT field path is a `metric.<name>` target.
#[must_use]
pub fn metric_projection_name_v1(field_path: &[String]) -> Option<&str> {
    match field_path {
        [head, name] if head == "metric" => Some(name.as_str()),
        _ => None,
    }
}

/// Fail-closed admission of the `metric.<name>` projections in a SELECT — an
/// unknown metric name is rejected so a typo never silently selects nothing.
pub fn admit_select_metric_projections_v1(
    select: &TypedSelectRootV1,
) -> Result<(), SemaDiagnosticV1> {
    let shape = select.shape_v1().map_err(|err| SemaDiagnosticV1 {
        code: err.code,
        message: err.message,
    })?;
    if let TypedSelectProjectionV1::Fields(fields) = &shape.projection {
        for field in fields {
            if let Some(metric_name) = metric_projection_name_v1(&field.field_path) {
                if metric_descriptor_v1(metric_name).is_none() {
                    return Err(SemaDiagnosticV1 {
                        code: "select_unknown_metric",
                        message: format!(
                            "`metric.{metric_name}` is not a known metric; \
                             see the metric inventory vocabulary"
                        ),
                    });
                }
            }
        }
    }
    Ok(())
}
