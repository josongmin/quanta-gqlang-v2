#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "metric target tests assert typed resolution + fail-closed unknown"
)]
//! F05/E11C hellgate — `metric.<name>` is a typed SELECT target resolved against
//! the metric inventory vocabulary; an unknown metric fails closed.
use quanta_gqlang_v2_sema::metric_vocabulary::{
    metric_descriptor_v1, MetricFamilyV1, EXTERNAL_METRIC_VOCABULARY_V1,
};
use quanta_gqlang_v2_sema::{analyze_document_v1, parse_document_v1};
use std::collections::BTreeSet;

fn reject_code_v1(input: &str) -> Vec<&'static str> {
    analyze_document_v1(&parse_document_v1(input).expect("parse"))
        .expect_err("must reject")
        .diagnostics
        .iter()
        .map(|d| d.code)
        .collect()
}

#[test]
fn vocabulary_covers_all_nine_families_with_unique_names_v1() {
    let families: BTreeSet<MetricFamilyV1> = EXTERNAL_METRIC_VOCABULARY_V1
        .iter()
        .map(|m| m.family)
        .collect();
    assert_eq!(families.len(), 9, "all nine metric families present");
    let names: BTreeSet<&str> = EXTERNAL_METRIC_VOCABULARY_V1
        .iter()
        .map(|m| m.name)
        .collect();
    assert_eq!(
        names.len(),
        EXTERNAL_METRIC_VOCABULARY_V1.len(),
        "metric names are unique"
    );
    assert!(metric_descriptor_v1("cognitive_complexity").is_some());
    assert_eq!(
        metric_descriptor_v1("new_coverage")
            .expect("coverage metric")
            .family,
        MetricFamilyV1::Coverage
    );
}

#[test]
fn known_metric_projection_admits_v1() {
    analyze_document_v1(
        &parse_document_v1("SELECT metric.new_coverage FROM project.current LIMIT 5")
            .expect("parse"),
    )
    .expect("known metric admits");
}

#[test]
fn unknown_metric_projection_fails_closed_v1() {
    let codes = reject_code_v1("SELECT metric.totally_made_up FROM project.current LIMIT 5");
    assert!(codes.contains(&"select_unknown_metric"), "got {codes:?}");
}
