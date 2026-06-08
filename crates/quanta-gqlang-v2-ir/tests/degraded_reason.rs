#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing and exercise fail-closed panics"
)]

//! `DegradedReasonV1` is the single source of truth for the read-only-shell
//! degraded-reason codes that were previously copy-pasted across the CLI
//! doctor/headless producers. The code mapping must round-trip and stay unique.

use quanta_gqlang_v2_ir::root_ir::DegradedReasonV1;

const ALL_REASONS_V1: [DegradedReasonV1; 5] = [
    DegradedReasonV1::RuntimeHostRoutingUnavailable,
    DegradedReasonV1::SemanticConfirmLaneUnavailable,
    DegradedReasonV1::TruthEvidenceMaterializationUnavailable,
    DegradedReasonV1::RetrievalResultMaterializationUnavailable,
    DegradedReasonV1::CatalogAndInteractiveHostTruthUnavailable,
];

#[test]
fn code_round_trips_through_from_code() {
    for reason in ALL_REASONS_V1 {
        assert_eq!(
            DegradedReasonV1::from_code_v1(reason.code_v1()),
            Some(reason)
        );
    }
}

#[test]
fn codes_are_distinct() {
    let mut codes = ALL_REASONS_V1
        .iter()
        .map(|reason| reason.code_v1())
        .collect::<Vec<_>>();
    codes.sort_unstable();
    codes.dedup();
    assert_eq!(codes.len(), ALL_REASONS_V1.len());
}

#[test]
fn unknown_code_has_no_reason() {
    assert_eq!(DegradedReasonV1::from_code_v1("not-a-real-reason"), None);
}
