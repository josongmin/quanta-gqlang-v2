#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::wildcard_enum_match_arm,
    reason = "integration test rails assert on fixed shapes and panic on unexpected variants"
)]
//! v3 ITEM-classification hellgate: every non-relational v2 root family is
//! CLASSIFIED into the host-agnostic planning vocabulary, fail-closed (ticket 07
//! FRONTEND-CONTRACT parity).
//!
//! These rails pin the per-family classification of [`classify_item`]:
//!
//! - FIND -> a [`PublicPlanV1`] with `result_kind = path_witness`,
//!   `effect = proof_evidence_only`, an explicit witness selection, and a
//!   populated cost schema (witness explosion risk from logic's contract).
//! - CHECK -> alert / read-only. EXPLAIN -> proof_explain / read-only.
//!   SHOW / SAMPLE -> table / read-only. DESCRIBE -> alert. COMPARE ->
//!   path_witness. EXPORT -> export_only.
//! - DECLARATIONS (VIEW / MODEL / PACK / PROFILE / GATE / RULE) -> the typed
//!   no-plan marker, NOT an error. USE PACK -> no-plan. REQUIRE CERTIFIED -> a
//!   proof-confirmed truth gate.
//! - An illegal FIND (missing required endpoint) fails closed with the exact
//!   logic diagnostic code / message surfaced through planning.
//!
//! Each test asserts the ACTUAL field values / codes, never merely that a value
//! was produced.

use quanta_gqlang_v2_hir::{
    CatalogDeclHirV1, CatalogPayloadV1, CompareMetricV1, ConfirmModeV1, DetectionModeV1,
    EmitModeV1, EvidenceModeV1, ExportFormatV1, GateBindingV1, GraphConstraintsV1,
    GraphEndpointsV1, GraphQueryHirV1, GraphQueryKindV1, GraphRefV1, HirNodeIdV1, LegalityHirV1,
    MetaHirV1, MetaKindV1, PresentationHirV1, PresentationKindV1, ProgramItemHirV1, SampleModeV1,
    ShowCategoryV1, SpanV1, TruthModeV1, UsePackActivationV1, UsePackHirV1, ViewBodyV1,
};
use quanta_gqlang_v2_planning::{
    classify_item, CapabilityV1, EffectClassV1, EvidenceRequirementV1, ItemClassificationV1,
    NoPlanReasonV1, PlanningDiagnosticV1, RenderRequirementV1, ResultKindV1, TruthLaneV1,
    WitnessExplosionRiskClassV1, WitnessSelectionPolicyV1,
};

fn id(n: u32) -> HirNodeIdV1 {
    HirNodeIdV1::from_raw(n)
}

fn span() -> SpanV1 {
    SpanV1::new(0, 10)
}

fn gref(name: &str) -> GraphRefV1 {
    GraphRefV1 {
        name: name.to_owned(),
        span: span(),
    }
}

fn expect_plan(item: &ProgramItemHirV1) -> quanta_gqlang_v2_planning::PublicPlanV1 {
    match classify_item(item) {
        ItemClassificationV1::Plan(plan) => plan,
        other => panic!("expected a plan, got {other:?}"),
    }
}

/// FIND (taint, source -> sink, bounded by depth) -> path_witness /
/// proof_evidence_only, explicit witness selection, populated cost schema.
#[test]
fn find_taint_classifies_to_path_witness_evidence_only() {
    let find = GraphQueryHirV1::new(
        id(1),
        span(),
        GraphQueryKindV1::Taint,
        GraphEndpointsV1 {
            source: Some(gref("user_input")),
            sink: Some(gref("sql_exec")),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1 {
            depth: Some(8),
            ..GraphConstraintsV1::default()
        },
    );
    let plan = expect_plan(&ProgramItemHirV1::from(find));

    assert_eq!(plan.result_kind, ResultKindV1::PathWitness);
    assert_eq!(plan.effect, EffectClassV1::ProofEvidenceOnly);
    assert_eq!(plan.render_requirement, RenderRequirementV1::WitnessPayload);
    assert_eq!(plan.evidence, EvidenceRequirementV1::WitnessPayload);
    // Witness selection is EXPLICIT, not backend-accidental.
    assert_eq!(
        plan.witness_selection,
        Some(WitnessSelectionPolicyV1::AllWitness)
    );
    // A cyclic taint relation, capped by an explicit depth, is cap-bounded.
    assert_eq!(
        plan.cost.witness_explosion_risk_class,
        WitnessExplosionRiskClassV1::CapBounded
    );
    assert_eq!(plan.cost.unsupported_reason_code, None);
    // FIND requires witness + recursive-closure capability.
    assert!(plan
        .capabilities
        .contains(&CapabilityV1::WitnessMaterialization));
    assert!(plan.capabilities.contains(&CapabilityV1::RecursiveClosure));
}

/// FIND missing a required endpoint fails closed with the EXACT logic
/// diagnostic surfaced through planning.
#[test]
fn find_missing_endpoint_rejects_fail_closed() {
    let find = GraphQueryHirV1::new(
        id(1),
        span(),
        GraphQueryKindV1::Taint,
        // No sink: taint requires source AND sink.
        GraphEndpointsV1 {
            source: Some(gref("user_input")),
            ..GraphEndpointsV1::default()
        },
        GraphConstraintsV1 {
            depth: Some(8),
            ..GraphConstraintsV1::default()
        },
    );
    let err = match classify_item(&ProgramItemHirV1::from(find)) {
        ItemClassificationV1::Reject(err) => err,
        other => panic!("expected a rejection, got {other:?}"),
    };
    assert!(matches!(
        err,
        PlanningDiagnosticV1::IllegalGraphQuery { .. }
    ));
    assert_eq!(err.code(), "logic.graph_endpoint_missing");
    assert_eq!(err.message(), "FIND `taint` requires a `sink` endpoint");
    assert_eq!(err.span(), span());
}

/// CHECK -> alert / read-only.
#[test]
fn check_classifies_to_alert_read_only() {
    let check = LegalityHirV1::new(
        id(1),
        span(),
        gref("no_eval"),
        None,
        None,
        ConfirmModeV1::Skip,
        EmitModeV1::Findings,
        DetectionModeV1::Hybrid,
    );
    let plan = expect_plan(&ProgramItemHirV1::from(check));
    assert_eq!(plan.result_kind, ResultKindV1::Alert);
    assert_eq!(plan.effect, EffectClassV1::ReadOnly);
    assert_eq!(plan.render_requirement, RenderRequirementV1::SourceLocation);
}

/// EXPLAIN -> proof_explain / read-only.
#[test]
fn explain_classifies_to_proof_explain_read_only() {
    let explain = PresentationHirV1::new(
        id(1),
        span(),
        PresentationKindV1::Explain {
            target_ref: gref("rule_x"),
            evidence: EvidenceModeV1::Trace,
            truth: TruthModeV1::Inspect,
        },
    );
    let plan = expect_plan(&ProgramItemHirV1::from(explain));
    assert_eq!(plan.result_kind, ResultKindV1::ProofExplain);
    assert_eq!(plan.effect, EffectClassV1::ReadOnly);
    assert_eq!(plan.evidence, EvidenceRequirementV1::ProofTree);
}

/// SHOW -> table / read-only.
#[test]
fn show_classifies_to_table_read_only() {
    let show = PresentationHirV1::new(
        id(1),
        span(),
        PresentationKindV1::Show {
            category: ShowCategoryV1::Capabilities,
        },
    );
    let plan = expect_plan(&ProgramItemHirV1::from(show));
    assert_eq!(plan.result_kind, ResultKindV1::Table);
    assert_eq!(plan.effect, EffectClassV1::ReadOnly);
}

/// DESCRIBE -> alert / read-only.
#[test]
fn describe_classifies_to_alert() {
    let describe = PresentationHirV1::new(
        id(1),
        span(),
        PresentationKindV1::Describe {
            kind: "function".to_owned(),
            path: gref("a.b.c"),
        },
    );
    let plan = expect_plan(&ProgramItemHirV1::from(describe));
    assert_eq!(plan.result_kind, ResultKindV1::Alert);
    assert_eq!(plan.effect, EffectClassV1::ReadOnly);
}

/// SAMPLE -> table / read-only.
#[test]
fn sample_classifies_to_table() {
    let sample = PresentationHirV1::new(
        id(1),
        span(),
        PresentationKindV1::Sample {
            query_text: "find sql".to_owned(),
            mode: SampleModeV1::Hybrid,
            limit: 10,
            scope: None,
        },
    );
    let plan = expect_plan(&ProgramItemHirV1::from(sample));
    assert_eq!(plan.result_kind, ResultKindV1::Table);
    assert_eq!(plan.effect, EffectClassV1::ReadOnly);
}

/// COMPARE -> path_witness (delta) / read-only.
#[test]
fn compare_classifies_to_path_witness() {
    let compare = PresentationHirV1::new(
        id(1),
        span(),
        PresentationKindV1::Compare {
            baseline_ref: gref("base"),
            candidate_ref: gref("cand"),
            metric: CompareMetricV1::Structural,
        },
    );
    let plan = expect_plan(&ProgramItemHirV1::from(compare));
    assert_eq!(plan.result_kind, ResultKindV1::PathWitness);
    assert_eq!(plan.effect, EffectClassV1::ReadOnly);
}

/// EXPORT -> export_only effect.
#[test]
fn export_classifies_to_export_only() {
    let export = PresentationHirV1::new(
        id(1),
        span(),
        PresentationKindV1::Export {
            source_ref: gref("findings"),
            format: ExportFormatV1::Sarif,
            profile: None,
        },
    );
    let plan = expect_plan(&ProgramItemHirV1::from(export));
    assert_eq!(plan.effect, EffectClassV1::ExportOnly);
    assert_eq!(plan.result_kind, ResultKindV1::Table);
}

/// A VIEW declaration -> the typed no-plan marker (declaration), NOT an error.
#[test]
fn view_declaration_is_no_plan_marker() {
    let view = CatalogDeclHirV1::new(
        id(1),
        span(),
        "active_users",
        span(),
        CatalogPayloadV1::View(ViewBodyV1::QueryRef(gref("users_q"))),
    );
    match classify_item(&ProgramItemHirV1::from(view)) {
        ItemClassificationV1::NoPlan(reason) => {
            assert_eq!(reason, NoPlanReasonV1::CatalogDeclaration);
        }
        other => panic!("expected a no-plan marker, got {other:?}"),
    }
}

/// A GATE declaration -> the no-plan marker too (all declarations register, none plan).
#[test]
fn gate_declaration_is_no_plan_marker() {
    let gate = CatalogDeclHirV1::new(
        id(1),
        span(),
        "release_gate",
        span(),
        CatalogPayloadV1::Gate {
            checks: vec![gref("c1"), gref("c2")],
            binding: GateBindingV1::All,
        },
    );
    match classify_item(&ProgramItemHirV1::from(gate)) {
        ItemClassificationV1::NoPlan(reason) => {
            assert_eq!(reason, NoPlanReasonV1::CatalogDeclaration);
        }
        other => panic!("expected a no-plan marker, got {other:?}"),
    }
}

/// USE PACK -> no-plan (effect-style activation, no relational plan).
#[test]
fn use_pack_is_no_plan_marker() {
    let use_pack = MetaHirV1::new(
        id(1),
        span(),
        MetaKindV1::UsePack(UsePackHirV1 {
            pack_id: "owasp".to_owned(),
            activation: UsePackActivationV1::Session,
            lockfile: "owasp.lock".to_owned(),
        }),
    );
    match classify_item(&ProgramItemHirV1::from(use_pack)) {
        ItemClassificationV1::NoPlan(reason) => {
            assert_eq!(reason, NoPlanReasonV1::MetaActivation);
        }
        other => panic!("expected a no-plan marker, got {other:?}"),
    }
}

/// REQUIRE CERTIFIED -> a proof-confirmed truth gate (a plan whose lane demands
/// the strongest proof semantics).
#[test]
fn require_certified_is_proof_confirmed_truth_gate() {
    let require = MetaHirV1::new(id(1), span(), MetaKindV1::RequireCertified);
    let plan = expect_plan(&ProgramItemHirV1::from(require));
    assert_eq!(plan.truth_lane, TruthLaneV1::ProofConfirmed);
    assert!(plan.truth_lane.is_final_policy_capable());
    assert_eq!(plan.evidence, EvidenceRequirementV1::ProofTree);
    assert_eq!(plan.effect, EffectClassV1::ProofEvidenceOnly);
}

/// A relational Query (SELECT) is DEFERRED to the algebra pipeline rather than
/// classified item-only: its cost schema needs the normalized algebra shape.
#[test]
fn select_query_defers_to_relational_pipeline() {
    use quanta_gqlang_v2_hir::{
        QueryHirV1, RelationExprHirV1, RelationExprKindV1, RelationSourceV1,
    };
    let source = RelationExprHirV1::new(
        id(2),
        span(),
        RelationExprKindV1::Source(RelationSourceV1 {
            name: "users".to_owned(),
            alias: None,
        }),
    );
    let query = QueryHirV1::new(id(1), span(), source, None);
    match classify_item(&ProgramItemHirV1::from(query)) {
        ItemClassificationV1::NoPlan(reason) => {
            assert_eq!(reason, NoPlanReasonV1::RelationalDeferred);
        }
        other => panic!("expected a relational-deferred marker, got {other:?}"),
    }
}

/// Each `NoPlanReasonV1` renders its stable keyword verbatim. These keywords
/// are a closed vocabulary surfaced through explain / diagnostics and CLI/LSP
/// rendering, so any drift (or an empty / placeholder string) must fail.
#[test]
fn no_plan_reason_keywords_are_stable_verbatim() {
    assert_eq!(
        NoPlanReasonV1::CatalogDeclaration.keyword(),
        "catalog_declaration"
    );
    assert_eq!(NoPlanReasonV1::MetaActivation.keyword(), "meta_activation");
    assert_eq!(
        NoPlanReasonV1::RelationalDeferred.keyword(),
        "relational_deferred"
    );
}
