#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "grammar acceptance asserts validated parses and exercises fail-closed rejects"
)]

//! F10 proof rail (jun-10-gqlnag-v2-followup) — E03B `SAMPLE` / `COMPARE` /
//! `EXPORT` grammar contract (`--test sample_compare_export_grammar`).
//!
//! NOTE TO OTHER AGENTS: this file pins the E03B DoD nouns to live typed
//! surface. Invariants that must never regress:
//!   1. `SAMPLE` routes through typed retrieval clauses (query/mode/limit/scope).
//!   2. `COMPARE` requires an EXPLICIT `compare_target` (the candidate ref). A
//!      payload with no candidate, or a candidate that is not a `<kind>::<loc>`
//!      reference, fails closed — there is no implicit/self compare.
//!   3. `EXPORT` stays bridge-only: its typed `bridge_receipt` carries the
//!      `export_target`, kind, and profile — and NOTHING that could mutate or
//!      promote a verdict. An unknown export kind fails closed.
//!
//! Each of the three roots has at least one acceptance AND one fail-closed
//! reject below. Do not relax these into positive-only checks.

use quanta_gqlang_v2_syntax::ast::AstRootV1;
use quanta_gqlang_v2_syntax::interactive::sample::{
    CompareMetricV1, ExportKindV1, SampleRetrievalModeV1,
};
use quanta_gqlang_v2_syntax::parse_document_v1;

fn single_root_v1(input: &str) -> AstRootV1 {
    let doc = parse_document_v1(input).expect("parse");
    assert_eq!(doc.roots.len(), 1, "expected one root: {input}");
    doc.roots.into_iter().next().expect("root")
}

fn reject_codes_v1(input: &str) -> Vec<&'static str> {
    let page = parse_document_v1(input).expect_err("expected fail-closed rejection");
    page.diagnostics.iter().map(|d| d.code).collect()
}

#[test]
fn sample_acceptance_routes_typed_retrieval_clauses_v1() {
    let AstRootV1::Sample(spec) = single_root_v1(
        "SAMPLE query=authorization middleware ;; scope=repo::frontend ;; mode=semantic ;; limit=20",
    ) else {
        panic!("expected SAMPLE root");
    };
    let intent = spec.intent().expect("sample intent");
    assert_eq!(intent.query, "authorization middleware");
    assert_eq!(intent.mode, SampleRetrievalModeV1::Semantic);
    assert_eq!(intent.limit, 20);
    let scope = intent.scope.expect("scope present");
    assert_eq!(scope.kind, "repo");
    assert_eq!(scope.locator, "frontend");
}

#[test]
fn sample_rejects_zero_limit_fail_closed_v1() {
    // A non-positive limit must fail closed, not silently clamp to a default.
    let codes = reject_codes_v1("SAMPLE query=auth ;; mode=hybrid ;; limit=0");
    assert!(
        codes.contains(&"invalid_sample_payload"),
        "expected invalid_sample_payload, got {codes:?}"
    );
}

#[test]
fn compare_acceptance_exposes_explicit_compare_target_v1() {
    let AstRootV1::Compare(spec) = single_root_v1(
        "COMPARE baseline=baseline::nightly ;; candidate=result::run-123 ;; metric=structural",
    ) else {
        panic!("expected COMPARE root");
    };
    let intent = spec.intent().expect("compare intent");
    assert_eq!(intent.metric, CompareMetricV1::Structural);
    // DoD noun: the explicit compare target is the candidate ref.
    let target = intent.compare_target();
    assert_eq!(target.kind, "result");
    assert_eq!(target.locator, "run-123");
    assert_eq!(target, &intent.candidate);
}

#[test]
fn compare_rejects_missing_compare_target_fail_closed_v1() {
    // No `candidate` clause -> no explicit compare target -> fail closed.
    let codes = reject_codes_v1("COMPARE baseline=baseline::nightly ;; metric=exact");
    assert!(
        codes.contains(&"invalid_compare_payload"),
        "expected invalid_compare_payload, got {codes:?}"
    );
}

#[test]
fn export_acceptance_yields_bridge_only_receipt_v1() {
    let AstRootV1::Export(spec) = single_root_v1(
        "EXPORT source=result::hotspots/latest ;; kind=sarif ;; profile=security-audit",
    ) else {
        panic!("expected EXPORT root");
    };
    let intent = spec.intent().expect("export intent");
    // DoD nouns: export_target + bridge_receipt.
    let target = intent.export_target();
    assert_eq!(target.kind, "result");
    assert_eq!(target.locator, "hotspots/latest");

    let receipt = intent.bridge_receipt();
    assert_eq!(receipt.export_target, *target);
    assert_eq!(receipt.kind, ExportKindV1::Sarif);
    assert_eq!(receipt.profile.as_deref(), Some("security-audit"));
}

#[test]
fn export_rejects_unknown_kind_fail_closed_v1() {
    // An unsupported serialization kind must fail closed; the bridge never
    // invents a default format.
    let codes = reject_codes_v1("EXPORT source=result::r ;; kind=mutate-and-apply");
    assert!(
        codes.contains(&"invalid_export_payload"),
        "expected invalid_export_payload, got {codes:?}"
    );
}
