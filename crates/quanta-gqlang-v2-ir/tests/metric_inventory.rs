#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

#[path = "../src/catalog/metric_inventory.rs"]
pub mod metric_inventory;

#[path = "../src/catalog/external_findings.rs"]
pub mod external_findings;

use external_findings::{
    external_finding_descriptor_v1, external_finding_rows_v1,
    validate_external_finding_inventory_v1, ExternalFindingDescriptorV1, ExternalFindingIdV1,
    ExternalFindingProviderSeverityV1,
};
use metric_inventory::{
    metric_descriptor_v1, metric_inventory_rows_v1, validate_metric_inventory_v1, MetricClassV1,
    MetricDescriptorV1, MetricIdV1, MetricUnitV1,
};

const MISMATCHED_PROVIDER_MAPPINGS_V1: [external_findings::ExternalFindingSeverityMappingV1; 1] =
    [external_findings::ExternalFindingSeverityMappingV1 {
        provider_severity: ExternalFindingProviderSeverityV1::CodeQlError,
        normalized_severity: external_findings::ExternalFindingSeverityV1::High,
    }];

#[test]
fn metric_inventory_names_are_disjoint_from_compare_metric_vocabulary_v1() {
    // The host-ingested metric inventory and the COMPARE `metric=` clause are two
    // unrelated closed vocabularies (the clause is the 3-value `CompareMetricV1`
    // in the syntax crate). Pin that they never share a token, so the two can
    // never be conflated into a single "metric" surface.
    const COMPARE_METRIC_VOCABULARY_V1: [&str; 3] = ["exact", "structural", "semantic"];
    for descriptor in metric_inventory_rows_v1() {
        let name = descriptor.id.as_str_v1();
        assert!(
            !COMPARE_METRIC_VOCABULARY_V1.contains(&name),
            "metric inventory id `{name}` collides with the COMPARE metric vocabulary"
        );
    }
}

#[test]
fn metric_inventory_exposes_named_external_signal_rows_v1() {
    validate_metric_inventory_v1(metric_inventory_rows_v1()).expect("inventory should validate");

    let row = metric_descriptor_v1(MetricIdV1::ExternalFindingCount)
        .expect("external finding count row should exist");
    assert_eq!(row.id.as_str_v1(), "external_finding_count");
    assert_eq!(row.class, MetricClassV1::ExternalSignal);
    assert_eq!(row.unit, MetricUnitV1::Items);
}

#[test]
fn external_findings_expose_provider_kind_and_metric_mappings_v1() {
    validate_external_finding_inventory_v1(external_finding_rows_v1())
        .expect("external finding inventory should validate");

    let row = external_finding_descriptor_v1(ExternalFindingIdV1::CodeQlPathProblem)
        .expect("codeql path problem row should exist");
    assert_eq!(row.id.as_str_v1(), "codeql_path_problem");
    assert_eq!(row.count_metric_id, MetricIdV1::ExternalFindingCount);
    assert_eq!(
        row.elevated_count_metric_id,
        Some(MetricIdV1::ExternalHighSeverityCount)
    );
    assert_eq!(
        row.suppressed_count_metric_id,
        Some(MetricIdV1::SuppressedFindingCount)
    );
}

#[test]
fn metric_inventory_fails_closed_on_duplicate_or_mismatched_rows_v1() {
    let mut rows = metric_inventory_rows_v1().to_vec();
    rows[1] = MetricDescriptorV1 {
        id: MetricIdV1::QueryResultCount,
        ..rows[1]
    };
    let duplicate_err =
        validate_metric_inventory_v1(&rows).expect_err("duplicate ids must fail closed");
    assert_eq!(duplicate_err.code, "metric_inventory_duplicate_id");

    let mismatched = [MetricDescriptorV1 {
        unit: MetricUnitV1::Percent,
        ..metric_inventory_rows_v1()[0]
    }];
    let mismatch_err = validate_metric_inventory_v1(&mismatched)
        .expect_err("count metrics cannot use percent units");
    assert_eq!(mismatch_err.code, "metric_unit_scalar_mismatch");
}

#[test]
fn external_findings_fail_closed_on_non_external_metrics_and_provider_mismatches_v1() {
    let mut rows = external_finding_rows_v1().to_vec();
    rows[0] = ExternalFindingDescriptorV1 {
        count_metric_id: MetricIdV1::QueryResultCount,
        ..rows[0]
    };
    let metric_err = validate_external_finding_inventory_v1(&rows)
        .expect_err("non-external metrics must fail closed");
    assert_eq!(metric_err.code, "external_finding_non_external_metric");

    let provider_mismatch = [ExternalFindingDescriptorV1 {
        id: ExternalFindingIdV1::SemgrepRuleMatch,
        provider: external_findings::ExternalFindingProviderV1::Semgrep,
        kind: external_findings::ExternalFindingKindV1::RuleMatch,
        provenance: external_findings::ExternalFindingProvenanceV1::SarifImport,
        default_severity: external_findings::ExternalFindingSeverityV1::High,
        count_metric_id: MetricIdV1::ExternalFindingCount,
        elevated_count_metric_id: Some(MetricIdV1::ExternalHighSeverityCount),
        suppressed_count_metric_id: Some(MetricIdV1::SuppressedFindingCount),
        severity_mappings: &MISMATCHED_PROVIDER_MAPPINGS_V1,
    }];
    let provider_err = validate_external_finding_inventory_v1(&provider_mismatch)
        .expect_err("provider severity mismatches must fail closed");
    assert_eq!(
        provider_err.code,
        "external_finding_provider_severity_mismatch"
    );
}
