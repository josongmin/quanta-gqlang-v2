#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration-test assertions intentionally use expect/unwrap/indexing, exercise fail-closed panics, and bind-all match arms that panic on unexpected variants"
)]

//! E03C ticket 03 — composition lowering + `WorkingSetV1` wiring + degraded
//! authority. Pins the working-set ref descriptor, the new degraded reason
//! (and its `-unavailable` suffix), basis mapping, and that the body's own
//! degraded lane is preserved (not swallowed) inside the composition.

use quanta_gqlang_v2_ir::composition_ir::CompositionIrV1;
use quanta_gqlang_v2_ir::root_ir::DegradedReasonV1;
use quanta_gqlang_v2_ir::working_set::{WorkingSetBasisV1, WorkingSetKindV1};
use quanta_gqlang_v2_ir::{lower_document_to_ir_v1, PublicIrRootV1};
use quanta_gqlang_v2_sema::{analyze_document_v1, parse_document_v1};

fn lower_one_v1(input: &str) -> PublicIrRootV1 {
    let doc = parse_document_v1(input).expect("parse");
    let typed = analyze_document_v1(&doc).expect("analyze");
    let roots = lower_document_to_ir_v1(&typed).expect("lower");
    assert_eq!(roots.len(), 1, "expected one IR root: {input}");
    roots.into_iter().next().expect("root")
}

fn composition_v1(input: &str) -> CompositionIrV1 {
    match lower_one_v1(input) {
        PublicIrRootV1::Composition(spec) => spec,
        other => panic!("expected composition IR, got {other:?}"),
    }
}

#[test]
fn composition_lowers_to_a_working_set_ref_and_lowered_body_v1() {
    let spec = composition_v1(concat!(
        "LET suspects = SELECT id FROM catalog.users LIMIT 5\n",
        "FIND taint api.entry -> db.write WITH suspects",
    ));
    assert_eq!(spec.bindings.len(), 1);
    let (reference, producer) = &spec.bindings[0];
    assert_eq!(reference.working_set_id, "suspects");
    assert_eq!(reference.kind, WorkingSetKindV1::CandidateSet);
    assert_eq!(reference.basis, WorkingSetBasisV1::SearchQuery);
    assert!(matches!(producer, PublicIrRootV1::Select(_)));
    assert_eq!(spec.narrowed_by, 0);
    assert!(matches!(*spec.body, PublicIrRootV1::Find(_)));
}

#[test]
fn find_producer_maps_to_ai_discovery_basis_v1() {
    let spec = composition_v1(concat!(
        "LET seed = FIND taint api.entry -> db.write\n",
        "SELECT id FROM catalog.users LIMIT 5 WITH seed",
    ));
    assert_eq!(spec.bindings[0].0.basis, WorkingSetBasisV1::AiDiscovery);
}

#[test]
fn composition_degraded_reason_is_unavailable_and_round_trips_v1() {
    let root = lower_one_v1(concat!(
        "LET x = SELECT id FROM catalog.users LIMIT 5\n",
        "FIND taint api.entry -> db.write WITH x",
    ));
    let reason = DegradedReasonV1::for_ir_root_v1(&root).expect("composition has a degraded lane");
    assert_eq!(
        reason,
        DegradedReasonV1::CompositionMaterializationUnavailable
    );
    assert_eq!(reason.code_v1(), "composition-materialization-unavailable");
    assert!(
        reason.code_v1().ends_with("-unavailable"),
        "authority suffix rule"
    );
    assert_eq!(
        DegradedReasonV1::from_code_v1("composition-materialization-unavailable"),
        Some(DegradedReasonV1::CompositionMaterializationUnavailable),
        "code round-trips through the single authority"
    );
}

#[test]
fn body_degraded_lane_is_preserved_inside_the_composition_v1() {
    // The FIND body has its own runtime-host-routing-unavailable lane. The
    // composition must keep the body's lowered IR so that lane is recoverable —
    // proof the body is not swallowed.
    let spec = composition_v1(concat!(
        "LET x = SELECT id FROM catalog.users LIMIT 5\n",
        "FIND taint api.entry -> db.write WITH x",
    ));
    assert_eq!(
        DegradedReasonV1::for_ir_root_v1(&spec.body),
        Some(DegradedReasonV1::RuntimeHostRoutingUnavailable),
        "the FIND body's own degraded lane must survive lowering"
    );
}
