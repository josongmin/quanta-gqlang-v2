#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "explain-path handshake asserts the front-door view contract end-to-end"
)]
//! F19/J09C handshake — the EXPLAIN/FIND front door (`explain::run`) end-to-end.
//! Proof rail: `cli --test explain_path_handshake`. Unlike `explain_path_ux`
//! (which unit-tests the view builders), this exercises the real seam
//! parse→analyze→lower→explain and pins the path-witness + why-no-result UX
//! contract the J09C ExplainHost/PathHost handshake depends on.

use quanta_gqlang_v2_cli::commands::explain;
use quanta_gqlang_v2_cli::NO_RESULT_REASON_V1;

#[test]
fn explain_root_front_door_surfaces_subject_and_why_no_result_v1() {
    let view = explain::run("EXPLAIN rule::owasp.sql-injection ;; evidence=trace ;; truth=confirm")
        .expect("EXPLAIN front door");
    assert_eq!(view.subject, "rule::owasp.sql-injection");
    assert_eq!(
        view.why_no_result.as_deref(),
        Some(NO_RESULT_REASON_V1),
        "presentation-only: the front door always carries the shared no-result reason"
    );
    assert!(
        view.path_witness.is_none(),
        "an EXPLAIN root carries no path witness"
    );
}

#[test]
fn find_front_door_preserves_source_via_sink_order_v1() {
    let view = explain::run("FIND taint api.entry -> db.write").expect("FIND explain front door");
    let witness = view
        .path_witness
        .as_ref()
        .expect("a FIND reachability/flow root must surface a path witness");
    let labels: Vec<&str> = witness
        .steps
        .iter()
        .map(|step| step.label.as_str())
        .collect();
    let source_idx = labels.iter().position(|label| *label == "source");
    let sink_idx = labels.iter().position(|label| *label == "sink");
    assert!(
        matches!((source_idx, sink_idx), (Some(s), Some(k)) if s < k),
        "source must precede sink in the path witness, got {labels:?}"
    );
}
