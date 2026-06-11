#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "AI-lane admissibility tests assert typed reject codes over the real analyzer"
)]
//! F03/E05E hellgate — the AI exploration lane admissibility. Proof rail:
//! `sema --test ai_profile`. The lane never asserts final truth: exploratory
//! roots admit (reconfirmation-required), and any root demanding final/certified
//! policy truth fails closed. The load-bearing guard is the over-block check — an
//! unconfirmed CHECK and a plain SELECT must stay admissible.

use quanta_gqlang_v2_sema::ai_profile::ai_lane_admissibility_v1;
use quanta_gqlang_v2_sema::{analyze_document_v1, parse_document_v1, TypedRootV1};

fn first_typed_root_v1(input: &str) -> TypedRootV1 {
    let doc = parse_document_v1(input).expect("parse");
    let typed = analyze_document_v1(&doc).expect("analyze");
    typed.roots.into_iter().next().expect("one root")
}

#[test]
fn exploratory_roots_admit_with_reconfirmation_required_v1() {
    for input in [
        "SELECT id FROM catalog.files LIMIT 3",
        "SAMPLE query=auth ;; mode=lexical ;; limit=5",
        "FIND taint api.entry -> db.write",
    ] {
        let profile = ai_lane_admissibility_v1(&first_typed_root_v1(input))
            .unwrap_or_else(|err| panic!("`{input}` must admit on the AI lane: {}", err.message));
        assert!(
            profile.reconfirmation_required,
            "every admitted AI-lane root is reconfirmation-required: `{input}`"
        );
    }
}

#[test]
fn require_certified_is_forbidden_on_the_ai_lane_v1() {
    let err = ai_lane_admissibility_v1(&first_typed_root_v1("REQUIRE CERTIFIED"))
        .expect_err("REQUIRE CERTIFIED demands certified final truth");
    assert_eq!(err.code, "ai_lane_final_truth_forbidden");
}

#[test]
fn require_confirmed_check_is_forbidden_on_the_ai_lane_v1() {
    let err = ai_lane_admissibility_v1(&first_typed_root_v1(
        "CHECK rule.x check:v1|class=security|scope=repo%3A%2F%2Fr|pattern=~|confirm=require",
    ))
    .expect_err("a require-confirmed CHECK asserts authoritative policy truth");
    assert_eq!(err.code, "ai_lane_final_truth_forbidden");
}

#[test]
fn unconfirmed_check_stays_admissible_on_the_ai_lane_v1() {
    // Over-block guard: only final-truth-demanding roots are fenced. A skip-confirm
    // CHECK is advisory exploration and must still ride the lane.
    let profile = ai_lane_admissibility_v1(&first_typed_root_v1(
        "CHECK rule.x check:v1|class=security|scope=repo%3A%2F%2Fr|pattern=~|confirm=skip",
    ))
    .expect("an advisory CHECK must admit on the AI lane");
    assert!(profile.reconfirmation_required);
}
