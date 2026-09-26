#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! v3 planning hellgate: the COMPLETE V2 fail-closed surface (ticket 07,
//! decisions `D10` / `D16`).
//!
//! V2 cut: the V1 hellgate rejected caller-selected construct MIXES
//! (`PlanRequestV1` carrying a contradictory result/effect/truth combination).
//! That whole concept is DELETED with `PlanRequestV1` / `UnsupportedMixReasonV1`:
//! a caller can no longer select any of those lanes, so there is no mix left to
//! reject. The equivalent V2 fail-closed surface is a property of the SEALED
//! carriers and of their pairing with the document:
//!
//! - `planning.input_document_mismatch.{absent,wrong_kind,identity_mismatch}` —
//!   a carrier that does not belong to the supplied document;
//! - `planning.budget.semantic_steps_exceeded` — a refused charge against the
//!   document's immutable `max_semantic_steps`;
//! - `planning.invalid_check.*` / `planning.invalid_presentation.*` /
//!   `planning.invalid_composition.*` — a validated wrapper whose sealed payload
//!   has no coherent policy;
//! - `planning.no_plan.*` — the typed, non-error non-plan vocabulary;
//! - the two `logic.*` RELAY variants, which surface the originating logic
//!   code/message/span verbatim.
//!
//! Every rail asserts the ACTUAL typed variant, code, message, and checked span
//! — never merely that an error occurred.
//!
//! Resolved meta and composition fixtures drive their real public constructors
//! and owner entry points; direct vocabulary assertions remain separate because
//! stable codes/messages are a public diagnostic contract.

use quanta_gqlang_v2_logic::{GraphEndpointRoleV1, GraphQueryContractV2, LogicDiagnosticV1};
use quanta_gqlang_v2_planning::{
    plan, plan_outcome, CheckContractV2, CheckInvalidReasonV2, CheckedCompositionPlanInputV2,
    CompositionBodyFamilyV2, CompositionBodyPlanInputV2, CompositionInvalidReasonV2, EffectClassV2,
    EffectSummaryV2, InputDocumentMismatchReasonV2, NoPlanReasonV2, PlanOutcomeV2, PlanPolicyV2,
    PlanningDiagnosticV1, PresentationContractV2, PresentationInvalidReasonV2, ResultKindV2,
    SemanticPlanInputV2,
};
use quanta_gqlang_v2_sema::test_support::{
    forged_cardinality_downgrade_document_v2, forged_small_receipt_relation_chain_document_v2,
    reference_alternate_graph_document_v2, reference_check_document_v2, reference_checked_span_v2,
    reference_presentation_document_v2, reference_relation_document_v2, reference_rule_document_v2,
    second_source_graph_document_v2, ReferenceBudgetV2, ReferenceCheckFixtureV2,
    ReferenceCompositionBodyV2, ReferenceGraphRequestV2, ReferenceMetaFixtureV2,
    ReferencePresentationFixtureV2, ReferenceRelationFixtureV2, ReferenceRuleFixtureV2,
    SMALL_BUDGET_MAX_SEMANTIC_STEPS,
};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, DetectionModeV1, EvidenceModeV1, GraphQueryKindV1, SemanticItemV2,
    TruthModeV1,
};
use quanta_gqlang_v2_types::{AggregationClassV1, DeterminismV1, MonotonicityV1, PurityV1};

mod support;

use support::{
    check_document, check_input, composition_input, deepest_node, document_item_location,
    graph_document, graph_input, graph_references, meta_input, relation_input, resolved_check,
    rule_input, FIXTURE_GRAPH_DEPTH_V2, FIXTURE_GRAPH_ITEM_SPAN_V2,
};

/// One checked fixture location, for the diagnostic rails that construct a
/// diagnostic directly instead of routing it through a carrier.
fn fixture_span() -> CheckedLocatedSpanV2 {
    reference_checked_span_v2(3, 11).expect("the fixture checked location is in bounds")
}

// ---- identity: absent / wrong kind / transplanted --------------------------

/// A carrier from another document is ABSENT: its identity is not in the
/// supplied document at all, and the rejection is anchored on the carrier's own
/// checked location.
#[test]
fn a_foreign_carrier_is_absent() {
    let (_primary, input) =
        relation_input(ReferenceRelationFixtureV2::Scan).expect("the reference Scan fixture");
    let other = quanta_gqlang_v2_sema::test_support::second_source_relation_document_v2()
        .expect("the secondary source document");
    let SemanticPlanInputV2::Relational(carrier) = &input else {
        panic!("the fixture seals a relational carrier");
    };

    match plan(&input, other.document()) {
        Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, span }) => {
            assert_eq!(reason, InputDocumentMismatchReasonV2::Absent);
            assert_eq!(span, *carrier.root().location());
        }
        other => panic!("expected an absent-identity rejection, got {other:?}"),
    }
}

/// The exact diagnostic code and message of the absent-identity rejection.
#[test]
fn the_absent_identity_rejection_renders_its_stable_code() {
    let diagnostic = PlanningDiagnosticV1::InputDocumentMismatch {
        reason: InputDocumentMismatchReasonV2::Absent,
        span: fixture_span(),
    };
    assert_eq!(diagnostic.code(), "planning.input_document_mismatch.absent");
    assert_eq!(
        diagnostic.message(),
        "planning input is absent from the document : the sealed carrier cannot be planned \
         against this document"
    );
    assert_eq!(diagnostic.span(), &fixture_span());
}

/// A carrier whose identity the document holds as ANOTHER item kind is
/// `WrongKind`, anchored on the offending member's own checked location: the
/// single-rule fixture's member shares its identity with the reference Filter
/// item, which the document holds as a relation.
#[test]
fn a_same_identity_other_kind_carrier_is_wrong_kind() {
    let (rule_document, input) =
        rule_input(ReferenceRuleFixtureV2::SingleRule).expect("the single-rule fixture");
    let relation = reference_relation_document_v2(ReferenceRelationFixtureV2::Filter)
        .expect("the reference Filter fixture");
    let SemanticPlanInputV2::Rule(contract) = &input else {
        panic!("the fixture seals a rule program carrier");
    };
    let member = contract.rules().first().expect("one member");
    assert_eq!(member.item_id(), relation.item_id());

    match plan(&input, relation.document()) {
        Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, span }) => {
            assert_eq!(reason, InputDocumentMismatchReasonV2::WrongKind);
            assert_eq!(span, *member.body().expression().location());
        }
        other => panic!("expected a wrong-kind rejection, got {other:?}"),
    }
    // The untouched carrier still plans against its OWN document.
    plan(&input, rule_document.document()).expect("the same carrier plans on its own document");
}

/// A same-identity carrier whose sealed facts were downgraded is an identity
/// mismatch, while the pristine carrier plans against the pristine document.
#[test]
fn a_transplanted_relational_carrier_is_an_identity_mismatch() {
    let (pristine, input) = relation_input(ReferenceRelationFixtureV2::GroupedAggregateSum)
        .expect("the reference grouped-aggregate fixture");
    let forged = forged_cardinality_downgrade_document_v2().expect("the forged downgrade document");
    let SemanticPlanInputV2::Relational(carrier) = &input else {
        panic!("the fixture seals a relational carrier");
    };
    assert_eq!(forged.item_id(), pristine.item_id());

    match plan(&input, forged.document()) {
        Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, span }) => {
            assert_eq!(reason, InputDocumentMismatchReasonV2::IdentityMismatch);
            assert_eq!(span, *carrier.root().location());
        }
        other => panic!("expected a sealed-identity rejection, got {other:?}"),
    }
    plan(&input, pristine.document()).expect("the pristine carrier still plans");
}

/// A FIND carrier whose sealed witness bound was mutated (same identity, a
/// different depth) is an identity mismatch — the plan can never silently
/// reclassify a bound the document did not seal.
#[test]
fn a_graph_carrier_with_a_mutated_bound_is_an_identity_mismatch() {
    let (_document, input) = graph_input(GraphQueryKindV1::Taint, 4).expect("the taint FIND");
    let mutated = graph_document(GraphQueryKindV1::Taint, 5).expect("the deeper taint FIND");
    let SemanticPlanInputV2::Graph(carrier) = &input else {
        panic!("the fixture seals a graph carrier");
    };
    assert_eq!(mutated.graph_item_id(), carrier.item_id());

    match plan(&input, mutated.document()) {
        Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, span }) => {
            assert_eq!(reason, InputDocumentMismatchReasonV2::IdentityMismatch);
            assert_eq!(span, *carrier.location());
        }
        other => panic!("expected a sealed-identity rejection, got {other:?}"),
    }
    let (own, input) = graph_input(GraphQueryKindV1::Taint, 4).expect("the taint FIND");
    plan(&input, own.document()).expect("the pristine carrier still plans");
}

/// A rule carrier whose member identity exists in the document but whose sealed
/// body facts disagree is an identity mismatch at that member's location: a
/// same-identity rule transplant can never plan.
#[test]
fn a_rule_carrier_with_transplanted_body_facts_is_an_identity_mismatch() {
    let (recursive_document, input) =
        rule_input(ReferenceRuleFixtureV2::PositiveSelfRecursion).expect("the recursive fixture");
    let other = quanta_gqlang_v2_sema::test_support::reference_rule_document_v2(
        ReferenceRuleFixtureV2::NonRecursiveDependency,
    )
    .expect("the nonrecursive fixture");
    let SemanticPlanInputV2::Rule(contract) = &input else {
        panic!("the fixture seals a rule program carrier");
    };
    let member = contract.rules().first().expect("one member");
    assert!(other.item_ids().contains(&member.item_id()));

    match plan(&input, other.document()) {
        Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, span }) => {
            assert_eq!(reason, InputDocumentMismatchReasonV2::IdentityMismatch);
            assert_eq!(span, *member.body().expression().location());
        }
        other => panic!("expected a sealed-identity rejection, got {other:?}"),
    }
    plan(&input, recursive_document.document()).expect("the pristine carrier still plans");
}

/// A CHECK carrier is absent from a document that does not hold its identity,
/// anchored on the CHECK's own checked location.
#[test]
fn a_check_carrier_is_absent_from_an_unrelated_document() {
    let (_check_document, input) = check_input().expect("the nested CHECK fixture");
    let unrelated = graph_document(GraphQueryKindV1::Taint, 4).expect("the taint FIND");
    let SemanticPlanInputV2::Check(carrier) = &input else {
        panic!("the fixture seals a CHECK carrier");
    };

    match plan(&input, unrelated.document()) {
        Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, span }) => {
            assert_eq!(reason, InputDocumentMismatchReasonV2::Absent);
            assert_eq!(span, *carrier.location());
        }
        other => panic!("expected an absent-identity rejection, got {other:?}"),
    }
}

// ---- budget ----------------------------------------------------------------

/// The exact sealed step limit plans; limit + 1 is refused at the refused node's
/// checked location, with the exact limit in the payload.
#[test]
fn the_exact_step_limit_plans_and_limit_plus_one_is_refused_at_the_refused_node() {
    let exact_depth = SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;
    let carrier = quanta_gqlang_v2_sema::test_support::deep_relation_chain_document_v2(
        exact_depth,
        ReferenceBudgetV2::Standard,
    )
    .expect("the exact-depth chain authors");
    let input =
        quanta_gqlang_v2_algebra::lower_relation_item(carrier.document(), carrier.item_id())
            .map(|item| SemanticPlanInputV2::Relational(Box::new(item)))
            .expect("the chain lowers");
    let document = forged_small_receipt_relation_chain_document_v2(exact_depth)
        .expect("the exact-depth small-receipt document");
    assert_eq!(
        document.document().budget_receipt().max_semantic_steps(),
        SMALL_BUDGET_MAX_SEMANTIC_STEPS
    );
    let planned = plan(&input, document.document()).expect("the exact limit plans");
    let (nodes, _effects) = support::relational_policy(&planned).expect("relational policy");
    assert_eq!(nodes, SMALL_BUDGET_MAX_SEMANTIC_STEPS);

    let over_depth = SMALL_BUDGET_MAX_SEMANTIC_STEPS;
    let carrier = quanta_gqlang_v2_sema::test_support::deep_relation_chain_document_v2(
        over_depth,
        ReferenceBudgetV2::Standard,
    )
    .expect("the over-depth chain authors");
    let over_input =
        quanta_gqlang_v2_algebra::lower_relation_item(carrier.document(), carrier.item_id())
            .map(|item| SemanticPlanInputV2::Relational(Box::new(item)))
            .expect("the chain lowers");
    let over_document = forged_small_receipt_relation_chain_document_v2(over_depth)
        .expect("the over-depth small-receipt document");
    let SemanticPlanInputV2::Relational(over_carrier) = &over_input else {
        panic!("the fixture seals a relational carrier");
    };
    let refused = deepest_node(over_carrier.root());

    match plan(&over_input, over_document.document()) {
        Err(PlanningDiagnosticV1::SemanticStepBudgetExceeded { limit, span }) => {
            assert_eq!(limit, SMALL_BUDGET_MAX_SEMANTIC_STEPS);
            assert_eq!(span, *refused.location());
            assert_eq!(
                PlanningDiagnosticV1::SemanticStepBudgetExceeded { limit, span }.code(),
                "planning.budget.semantic_steps_exceeded"
            );
        }
        other => panic!("expected the exact budget refusal, got {other:?}"),
    }
}

/// The budget refusal message names the exact sealed limit.
#[test]
fn the_budget_refusal_names_the_exact_sealed_limit() {
    let diagnostic = PlanningDiagnosticV1::SemanticStepBudgetExceeded {
        limit: SMALL_BUDGET_MAX_SEMANTIC_STEPS,
        span: fixture_span(),
    };
    assert_eq!(
        diagnostic.message(),
        "semantic planning exceeded the document's sealed max_semantic_steps budget of 8"
    );
    assert_eq!(diagnostic.span(), &fixture_span());
}

/// A one-stage relation charges exactly ONE semantic step under a small receipt
/// and plans: the budget rail is an exact charge account, not a heuristic.
#[test]
fn a_single_stage_plan_charges_exactly_one_step() {
    let document = forged_small_receipt_relation_chain_document_v2(0)
        .expect("the single-stage small-receipt document");
    let carrier = quanta_gqlang_v2_sema::test_support::deep_relation_chain_document_v2(
        0,
        ReferenceBudgetV2::Standard,
    )
    .expect("the single-stage chain authors");
    let input =
        quanta_gqlang_v2_algebra::lower_relation_item(carrier.document(), carrier.item_id())
            .map(|item| SemanticPlanInputV2::Relational(Box::new(item)))
            .expect("the stage lowers");
    let planned = plan(&input, document.document()).expect("one charge fits the limit");
    let (nodes, _effects) = support::relational_policy(&planned).expect("relational policy");
    assert_eq!(nodes, 1);
    assert_eq!(
        planned.explain().families(),
        &[quanta_gqlang_v2_algebra::AlgebraFamilyV2::Scan]
    );
    assert_eq!(
        Some(planned.explain().root_location()),
        document_item_location(document.document(), document.item_id()).as_ref()
    );
}

// ---- typed non-plan outcomes -----------------------------------------------

/// A document-wide rule program with no member is a typed non-plan outcome
/// through BOTH entry points — never an error, never a skipped stage.
#[test]
fn an_empty_rule_program_is_a_typed_non_plan_from_both_entries() {
    let (document, _input) =
        relation_input(ReferenceRelationFixtureV2::Scan).expect("the reference Scan fixture");
    let contract =
        quanta_gqlang_v2_logic::RuleProgramContractV2::from_document(document.document())
            .expect("a document without a Rule item seals an empty program");
    let input = SemanticPlanInputV2::Rule(contract);
    let anchor = document_item_location(document.document(), document.item_id());

    match plan_outcome(&input, document.document()).expect("an empty program is not an error") {
        PlanOutcomeV2::NoPlan(reason) => assert_eq!(reason, NoPlanReasonV2::EmptyRuleProgram),
        PlanOutcomeV2::Plan(planned) => {
            panic!("an empty rule program must not plan: {planned:?}")
        }
    }

    match plan(&input, document.document()) {
        Err(PlanningDiagnosticV1::NoPlan { reason, span }) => {
            assert_eq!(reason, NoPlanReasonV2::EmptyRuleProgram);
            assert_eq!(Some(&span), anchor.as_ref());
            assert_eq!(
                span,
                *anchor.as_ref().expect("the Scan item has a location")
            );
            assert_eq!(
                reason.message(),
                "the document has no Rule item, so it has no rule program plan"
            );
        }
        other => panic!("expected the typed non-plan refusal, got {other:?}"),
    }
}

#[test]
fn itemless_document_diagnostic_retains_the_original_registry() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::SingleRule)
        .expect("sealed rule fixture");
    let original = fixture.document();
    let empty = original.clone().forged_membership(|order, items| {
        order.clear();
        items.clear();
    });
    let contract = quanta_gqlang_v2_logic::RuleProgramContractV2::from_document(&empty)
        .expect("empty membership is consistent");
    let input = SemanticPlanInputV2::Rule(contract);
    assert_eq!(
        plan_outcome(&input, &empty).expect("empty program outcome"),
        PlanOutcomeV2::NoPlan(NoPlanReasonV2::EmptyRuleProgram),
    );
    let error = plan(&input, &empty).expect_err("compatibility entry reports typed no-plan");
    let PlanningDiagnosticV1::NoPlan { reason, span } = &error else {
        panic!("empty program must retain the typed no-plan reason");
    };
    assert_eq!(*reason, NoPlanReasonV2::EmptyRuleProgram);
    assert_eq!(error.code(), "planning.no_plan.empty_rule_program");
    assert_eq!(span.location().source_id, original.source_id());
    assert_eq!(
        span.location().span,
        quanta_gqlang_v2_sema::SpanV1::point(0)
    );
    assert_eq!(
        span.registry_identity(),
        original.catalog().source_registry_identity()
    );
}

/// A resolved `USE PACK` is a typed non-plan through both public entries, with
/// the authored directive span preserved in the refusal diagnostic.
#[test]
fn a_use_pack_is_a_typed_meta_activation_non_plan() {
    let (document, input) =
        meta_input(ReferenceMetaFixtureV2::UsePack).expect("the USE PACK fixture seals");
    assert_eq!(
        plan_outcome(&input, document.document()).expect("USE PACK is not an owner error"),
        PlanOutcomeV2::NoPlan(NoPlanReasonV2::MetaActivation)
    );
    let Some(SemanticItemV2::Meta(meta)) = document.document().item(document.meta_item_id()) else {
        panic!("the fixture item is a resolved meta directive");
    };
    let diagnostic = plan(&input, document.document()).expect_err("USE PACK has no serving plan");
    assert_eq!(
        diagnostic,
        PlanningDiagnosticV1::NoPlan {
            reason: NoPlanReasonV2::MetaActivation,
            span: meta.location().clone(),
        }
    );
    assert_eq!(diagnostic.code(), "planning.no_plan.meta_activation");
    assert_eq!(
        diagnostic.message(),
        "no plan: USE PACK activates a pack and has no relational serving plan"
    );
    assert_eq!(NoPlanReasonV2::MetaActivation.catalog_kind(), None);
}

/// Every `NoPlanReasonV2` spelling and every catalog payload code is stable, and
/// each reason carries its exact human message.
#[test]
fn the_non_plan_vocabulary_names_every_reason_and_catalog_payload() {
    let span = fixture_span();
    for (reason, code, message) in [
        (
            NoPlanReasonV2::ValueDeclaration,
            "planning.no_plan.value_declaration",
            "no plan: value declaration binds a checked value and has no serving plan",
        ),
        (
            NoPlanReasonV2::MetaActivation,
            "planning.no_plan.meta_activation",
            "no plan: USE PACK activates a pack and has no relational serving plan",
        ),
        (
            NoPlanReasonV2::EmptyRuleProgram,
            "planning.no_plan.empty_rule_program",
            "no plan: the document has no Rule item, so it has no rule program plan",
        ),
    ] {
        let diagnostic = PlanningDiagnosticV1::NoPlan {
            reason,
            span: span.clone(),
        };
        assert_eq!(diagnostic.code(), code);
        assert_eq!(diagnostic.message(), message);
        assert_eq!(diagnostic.span(), &span);
    }

    for (kind, code, name) in [
        (
            quanta_gqlang_v2_planning::CatalogDeclarationKindV2::View,
            "view",
            "VIEW",
        ),
        (
            quanta_gqlang_v2_planning::CatalogDeclarationKindV2::Model,
            "model",
            "MODEL",
        ),
        (
            quanta_gqlang_v2_planning::CatalogDeclarationKindV2::Pack,
            "pack",
            "PACK",
        ),
        (
            quanta_gqlang_v2_planning::CatalogDeclarationKindV2::Profile,
            "profile",
            "PROFILE",
        ),
        (
            quanta_gqlang_v2_planning::CatalogDeclarationKindV2::Gate,
            "gate",
            "GATE",
        ),
        (
            quanta_gqlang_v2_planning::CatalogDeclarationKindV2::RuleBinding,
            "rule_binding",
            "RULE",
        ),
    ] {
        assert_eq!(kind.code(), code);
        assert_eq!(kind.name(), name);
        let reason = NoPlanReasonV2::CatalogDeclaration { kind };
        let diagnostic = PlanningDiagnosticV1::NoPlan {
            reason,
            span: span.clone(),
        };
        assert_eq!(
            diagnostic.code(),
            format!("planning.no_plan.catalog_declaration.{code}")
        );
        assert_eq!(
            diagnostic.message(),
            format!(
                "no plan: {name} declaration registers in the catalog and has no per-declaration \
                 serving plan"
            )
        );
        assert_eq!(reason.code(), "catalog_declaration");
        assert_eq!(reason.catalog_kind(), Some(kind));
    }
}

// ---- rule-program legality: owned by the sole ingress ----------------------

/// An unstratifiable rule program is refused at the program's SOLE INGRESS with
/// logic's typed diagnostic verbatim — code, message, and span. No contract (and
/// therefore no `SemanticPlanInputV2::Rule`) can exist for it, so planning can no
/// longer be reached with one: `PlanningDiagnosticV1::IllegalRuleProgram` is
/// structurally unreachable from the pipeline path, while its spelling stays
/// pinned by `no_deleted_mix_code_survives_in_the_diagnostic_surface`.
#[test]
fn an_unstratifiable_rule_program_is_refused_at_the_sole_ingress() {
    let document = reference_rule_document_v2(ReferenceRuleFixtureV2::NegatedSelfRecursion)
        .expect("the negated self-recursion fixture");
    let member = document
        .document()
        .rules()
        .next()
        .expect("the fixture carries one member");

    let logic = quanta_gqlang_v2_logic::RuleProgramContractV2::from_document(document.document())
        .expect_err("a negated recursion cycle is not stratifiable");
    assert_eq!(logic.code(), "logic.negation_cycle");
    assert_eq!(
        logic.message(),
        "recursion through negation is not stratifiable: cycle [r]"
    );
    assert_eq!(logic.span(), member.body().expression().location());
}

/// The recursive-aggregate policy is reject-all, and the ingress relays logic's
/// exact reason for it: the diagnostic points at the AGGREGATE CALL that folds
/// the recursive predicate, not at the rule body.
#[test]
fn a_recursive_aggregate_rule_program_is_refused_at_the_sole_ingress() {
    let document = reference_rule_document_v2(ReferenceRuleFixtureV2::AggregatedSelfRecursion)
        .expect("the aggregated self-recursion fixture");
    let member = document
        .document()
        .rules()
        .next()
        .expect("the fixture carries one member");
    let call = member.calls().first().expect("one aggregated call");
    assert_ne!(call.location(), member.body().expression().location());

    let logic = quanta_gqlang_v2_logic::RuleProgramContractV2::from_document(document.document())
        .expect_err("aggregated recursion is not stratifiable");
    assert_eq!(logic.code(), "logic.recursive_aggregate");
    assert_eq!(
        logic.message(),
        "recursive aggregate `<aggregate>` over recursive predicate `r` is not admitted"
    );
    assert_eq!(logic.span(), call.location());
}

/// The graph-legality relay surfaces logic's missing-endpoint diagnostic
/// verbatim through planning: the V1 rail that drove this through a raw HIR item
/// is replaced by the relay payload itself, because planning can only receive an
/// already-sealed logic contract.
#[test]
fn the_graph_legality_relay_surfaces_logic_verbatim() {
    let logic = LogicDiagnosticV1::GraphEndpointMissing {
        kind: GraphQueryKindV1::Taint,
        role: GraphEndpointRoleV1::Sink,
        span: fixture_span(),
    };
    let relayed = PlanningDiagnosticV1::IllegalGraphQuery {
        logic: logic.clone(),
    };
    assert_eq!(relayed.code(), "logic.graph_endpoint_missing");
    assert_eq!(relayed.message(), "FIND `taint` requires a `sink` endpoint");
    assert_eq!(relayed.span(), &fixture_span());
    assert_eq!(relayed.message(), logic.message());
}

// ---- validated-wrapper coherence -------------------------------------------

/// A CHECK carrying a semantic confirm FIND must receive its sealed nested graph
/// contract: a missing one rejects at the CHECK's own checked location.
#[test]
fn a_check_contract_rejects_a_missing_nested_confirm_graph() {
    let document = check_document().expect("the nested CHECK fixture");
    let check = resolved_check(&document).expect("the resolved CHECK item");
    assert_eq!(
        check
            .nested_graph()
            .map(quanta_gqlang_v2_sema::ResolvedGraphQueryV2::item_id),
        Some(document.nested_graph_item_id())
    );

    match CheckContractV2::from_resolved(check, None) {
        Err(PlanningDiagnosticV1::InvalidCheck { reason, span }) => {
            assert_eq!(reason, CheckInvalidReasonV2::MissingNestedGraph);
            assert_eq!(span, *check.location());
            assert_eq!(
                PlanningDiagnosticV1::InvalidCheck { reason, span }.code(),
                "planning.invalid_check.missing_nested_graph"
            );
        }
        other => panic!("expected a missing-nested-graph rejection, got {other:?}"),
    }
}

fn check_request() -> ReferenceGraphRequestV2 {
    ReferenceGraphRequestV2 {
        kind: GraphQueryKindV1::Taint,
        references: graph_references(GraphQueryKindV1::Taint),
        depth: Some(FIXTURE_GRAPH_DEPTH_V2),
        direction: None,
        summary_facets: Vec::new(),
        item_span: FIXTURE_GRAPH_ITEM_SPAN_V2,
    }
}

/// Every constructible mode/pattern/semantic-confirm combination is driven
/// through the real Sema transaction and then through the planning wrapper.
/// This pins both first-defect precedence and the CHECK's exact checked span.
#[test]
fn resolved_check_contract_enforces_the_complete_option_matrix() {
    for mode in [
        DetectionModeV1::PatternOnly,
        DetectionModeV1::SemanticOnly,
        DetectionModeV1::Hybrid,
    ] {
        for with_pattern in [false, true] {
            for with_nested_graph in [false, true] {
                let fixture = reference_check_document_v2(
                    check_request(),
                    ReferenceCheckFixtureV2 {
                        mode,
                        with_pattern,
                        with_nested_graph,
                    },
                )
                .expect("the configurable CHECK fixture resolves");
                let check = fixture.check();
                let nested = check
                    .nested_graph()
                    .map(GraphQueryContractV2::from_resolved)
                    .transpose()
                    .expect("the nested graph contract seals");
                let expected = match mode {
                    DetectionModeV1::PatternOnly if with_nested_graph => {
                        Some(CheckInvalidReasonV2::SemanticConfirmInPatternMode)
                    }
                    DetectionModeV1::SemanticOnly if !with_nested_graph => {
                        Some(CheckInvalidReasonV2::MissingSemanticConfirm)
                    }
                    DetectionModeV1::SemanticOnly if with_pattern => {
                        Some(CheckInvalidReasonV2::PatternInputInSemanticMode)
                    }
                    DetectionModeV1::Hybrid if !with_pattern => {
                        Some(CheckInvalidReasonV2::MissingHybridPatternInput)
                    }
                    DetectionModeV1::Hybrid if !with_nested_graph => {
                        Some(CheckInvalidReasonV2::MissingHybridSemanticInput)
                    }
                    DetectionModeV1::PatternOnly
                    | DetectionModeV1::SemanticOnly
                    | DetectionModeV1::Hybrid => None,
                };

                match (
                    CheckContractV2::from_resolved(check, nested.clone()),
                    expected,
                ) {
                    (
                        Err(PlanningDiagnosticV1::InvalidCheck { reason, span }),
                        Some(expected_reason),
                    ) => {
                        assert_eq!(reason, expected_reason);
                        assert_eq!(span, *check.location());
                        assert_eq!(
                            PlanningDiagnosticV1::InvalidCheck { reason, span }.code(),
                            format!("planning.invalid_check.{}", expected_reason.code())
                        );
                    }
                    (Ok(contract), None) => {
                        assert_eq!(contract.item_id(), check.item_id());
                        assert_eq!(contract.location(), check.location());
                        assert_eq!(contract.mode(), mode);
                        assert_eq!(contract.rule(), check.rule());
                        assert_eq!(contract.scope(), check.scope());
                        assert_eq!(contract.pattern(), check.pattern());
                        assert_eq!(contract.nested_graph(), nested.as_ref());
                        assert_eq!(contract.pattern().is_some(), with_pattern);
                        assert_eq!(contract.nested_graph().is_some(), with_nested_graph);
                        let planned =
                            plan(&SemanticPlanInputV2::Check(contract), fixture.document())
                                .expect("valid CHECK plans");
                        let result = planned.result().multiplicity_order();
                        assert_eq!(
                            result.cardinality,
                            quanta_gqlang_v2_types::CardinalityV1::Set
                        );
                        assert!(result.eliminates_duplicates());
                        assert!(!result.is_ordered());
                        assert_eq!(
                            planned.semantic_cost().ordering_contract,
                            quanta_gqlang_v2_planning::OrderingContractV2::Unordered
                        );
                        assert_eq!(
                            planned.result().witness_selection(),
                            if with_nested_graph {
                                Some(
                                    quanta_gqlang_v2_planning::WitnessSelectionPolicyV2::AllWitness,
                                )
                            } else {
                                None
                            }
                        );
                    }
                    (actual, expected) => panic!(
                        "CHECK option matrix mismatch: mode={mode:?} pattern={with_pattern} \
                         nested={with_nested_graph} expected={expected:?} actual={actual:?}"
                    ),
                }
            }
        }
    }
}

#[test]
fn a_check_without_semantic_confirm_rejects_an_unexpected_nested_contract() {
    let fixture = reference_check_document_v2(
        check_request(),
        ReferenceCheckFixtureV2 {
            mode: DetectionModeV1::PatternOnly,
            with_pattern: true,
            with_nested_graph: false,
        },
    )
    .expect("pattern-only CHECK resolves");
    let standalone = graph_document(GraphQueryKindV1::Taint, FIXTURE_GRAPH_DEPTH_V2)
        .expect("standalone graph resolves");
    let nested =
        GraphQueryContractV2::from_resolved(standalone.graph()).expect("standalone graph contract");
    match CheckContractV2::from_resolved(fixture.check(), Some(nested)) {
        Err(PlanningDiagnosticV1::InvalidCheck { reason, span }) => {
            assert_eq!(reason, CheckInvalidReasonV2::UnexpectedNestedGraph);
            assert_eq!(span, *fixture.check().location());
        }
        other => panic!("expected unexpected-nested-graph rejection, got {other:?}"),
    }
}

#[test]
fn a_check_rejects_a_nested_contract_from_another_source() {
    let fixture = reference_check_document_v2(
        check_request(),
        ReferenceCheckFixtureV2 {
            mode: DetectionModeV1::SemanticOnly,
            with_pattern: false,
            with_nested_graph: true,
        },
    )
    .expect("semantic CHECK resolves");
    let foreign = second_source_graph_document_v2().expect("second-source graph resolves");
    let nested =
        GraphQueryContractV2::from_resolved(foreign.graph()).expect("second-source graph contract");
    assert_ne!(
        fixture
            .check()
            .nested_graph()
            .expect("nested graph")
            .item_id(),
        nested.item_id()
    );
    match CheckContractV2::from_resolved(fixture.check(), Some(nested)) {
        Err(PlanningDiagnosticV1::InvalidCheck { reason, span }) => {
            assert_eq!(reason, CheckInvalidReasonV2::NestedGraphIdentityMismatch);
            assert_eq!(span, *fixture.check().location());
        }
        other => panic!("expected nested-identity rejection, got {other:?}"),
    }
}

/// A same-source graph with a distinct semantic item identity cannot be
/// transplanted into CHECK even when its complete payload otherwise matches.
#[test]
fn a_check_rejects_a_same_source_different_item_identity() {
    let fixture = reference_check_document_v2(
        check_request(),
        ReferenceCheckFixtureV2 {
            mode: DetectionModeV1::SemanticOnly,
            with_pattern: false,
            with_nested_graph: true,
        },
    )
    .expect("semantic CHECK resolves");
    let alternate = reference_alternate_graph_document_v2(check_request())
        .expect("same-source alternate graph resolves");
    let nested = GraphQueryContractV2::from_resolved(alternate.graph())
        .expect("alternate graph contract seals");
    let sealed = fixture.check().nested_graph().expect("nested graph");
    assert_eq!(sealed.item_id().source_id(), nested.item_id().source_id());
    assert_ne!(sealed.item_id(), nested.item_id());

    match CheckContractV2::from_resolved(fixture.check(), Some(nested)) {
        Err(PlanningDiagnosticV1::InvalidCheck { reason, span }) => {
            assert_eq!(reason, CheckInvalidReasonV2::NestedGraphIdentityMismatch);
            assert_eq!(span, *fixture.check().location());
        }
        other => panic!("expected same-source identity rejection, got {other:?}"),
    }
}

#[test]
fn a_check_rejects_a_same_identity_nested_payload_transplant() {
    let fixture = reference_check_document_v2(
        check_request(),
        ReferenceCheckFixtureV2 {
            mode: DetectionModeV1::SemanticOnly,
            with_pattern: false,
            with_nested_graph: true,
        },
    )
    .expect("semantic CHECK resolves");
    let transplanted = graph_document(GraphQueryKindV1::Taint, FIXTURE_GRAPH_DEPTH_V2 + 1)
        .expect("same-identity deeper graph resolves");
    let nested = GraphQueryContractV2::from_resolved(transplanted.graph())
        .expect("transplanted graph contract");
    assert_eq!(
        fixture
            .check()
            .nested_graph()
            .expect("nested graph")
            .item_id(),
        nested.item_id(),
        "the adversarial contract keeps the same source-qualified item identity"
    );
    assert_ne!(
        fixture
            .check()
            .nested_graph()
            .expect("nested graph")
            .depth(),
        Some(nested.depth()),
        "only the sealed payload is transplanted"
    );

    match CheckContractV2::from_resolved(fixture.check(), Some(nested)) {
        Err(PlanningDiagnosticV1::InvalidCheck { reason, span }) => {
            assert_eq!(reason, CheckInvalidReasonV2::NestedGraphIdentityMismatch);
            assert_eq!(span, *fixture.check().location());
            let diagnostic = PlanningDiagnosticV1::InvalidCheck { reason, span };
            assert_eq!(
                diagnostic.code(),
                "planning.invalid_check.nested_graph_identity_mismatch"
            );
            assert_eq!(
                diagnostic.message(),
                "CHECK nested graph contract differs from the resolved CHECK's sealed nested \
                 graph identity or payload"
            );
        }
        other => panic!("expected nested-payload rejection, got {other:?}"),
    }
}

/// Every `CheckInvalidReasonV2` renders its stable code and exact message.
#[test]
fn invalid_check_reasons_all_render_stable_codes() {
    let span = fixture_span();
    for (reason, suffix, message) in [
        (
            CheckInvalidReasonV2::NestedGraphIdentityMismatch,
            "nested_graph_identity_mismatch",
            "CHECK nested graph contract differs from the resolved CHECK's sealed nested graph \
             identity or payload",
        ),
        (
            CheckInvalidReasonV2::UnexpectedNestedGraph,
            "unexpected_nested_graph",
            "CHECK carries no semantic confirm input, so no nested graph contract is admissible",
        ),
        (
            CheckInvalidReasonV2::MissingNestedGraph,
            "missing_nested_graph",
            "CHECK semantic confirm input has no sealed nested graph contract",
        ),
        (
            CheckInvalidReasonV2::SemanticConfirmInPatternMode,
            "semantic_confirm_in_pattern_mode",
            "CHECK pattern detection mode forbids a semantic confirm input",
        ),
        (
            CheckInvalidReasonV2::MissingSemanticConfirm,
            "missing_semantic_confirm",
            "CHECK semantic detection mode requires a semantic confirm input",
        ),
        (
            CheckInvalidReasonV2::PatternInputInSemanticMode,
            "pattern_input_in_semantic_mode",
            "CHECK semantic detection mode does not accept a pattern input",
        ),
        (
            CheckInvalidReasonV2::MissingHybridPatternInput,
            "missing_hybrid_pattern_input",
            "CHECK hybrid detection mode requires the authored pattern input",
        ),
        (
            CheckInvalidReasonV2::MissingHybridSemanticInput,
            "missing_hybrid_semantic_input",
            "CHECK hybrid detection mode requires the semantic confirm input",
        ),
    ] {
        let diagnostic = PlanningDiagnosticV1::InvalidCheck {
            reason,
            span: span.clone(),
        };
        assert_eq!(
            diagnostic.code(),
            format!("planning.invalid_check.{suffix}")
        );
        assert_eq!(diagnostic.message(), message);
        assert_eq!(reason.code(), suffix);
    }
}

/// Every evidence/truth pair is routed through the real resolved-presentation
/// carrier. The two incoherent pairs reject at the authored presentation span;
/// all ten other pairs retain their exact sealed payload.
#[test]
fn resolved_explain_contract_enforces_the_complete_evidence_truth_matrix() {
    const EVIDENCE: [EvidenceModeV1; 4] = [
        EvidenceModeV1::Auto,
        EvidenceModeV1::Trace,
        EvidenceModeV1::Citation,
        EvidenceModeV1::Counterexample,
    ];
    const TRUTH: [TruthModeV1; 3] = [
        TruthModeV1::Inspect,
        TruthModeV1::Confirm,
        TruthModeV1::Refute,
    ];

    for evidence in EVIDENCE {
        for truth in TRUTH {
            let document =
                reference_presentation_document_v2(ReferencePresentationFixtureV2::Explain {
                    evidence,
                    truth,
                })
                .expect("the reference EXPLAIN fixture resolves");
            let presentation = document.presentation();
            let incoherent = matches!(
                (evidence, truth),
                (EvidenceModeV1::Counterexample, TruthModeV1::Confirm)
                    | (EvidenceModeV1::Citation, TruthModeV1::Refute)
            );

            match PresentationContractV2::from_resolved(presentation) {
                Err(PlanningDiagnosticV1::InvalidPresentation { reason, span }) if incoherent => {
                    assert_eq!(
                        reason,
                        PresentationInvalidReasonV2::UnsupportedExplainEvidenceTruth
                    );
                    assert_eq!(span, *presentation.location());
                    assert_eq!(
                        PlanningDiagnosticV1::InvalidPresentation { reason, span }.code(),
                        "planning.invalid_presentation.unsupported_explain_evidence_truth"
                    );
                }
                Ok(contract) if !incoherent => {
                    assert_eq!(contract.item_id(), presentation.item_id());
                    assert_eq!(contract.location(), presentation.location());
                    assert_eq!(contract.kind(), presentation.kind());
                }
                other => panic!(
                    "unexpected EXPLAIN result for evidence={evidence:?} truth={truth:?}: {other:?}"
                ),
            }
        }
    }
}

/// The closed presentation reason vocabulary remains byte-stable independently
/// of the entry-point matrix above.
#[test]
fn invalid_presentation_reasons_render_a_stable_code() {
    let diagnostic = PlanningDiagnosticV1::InvalidPresentation {
        reason: PresentationInvalidReasonV2::UnsupportedExplainEvidenceTruth,
        span: fixture_span(),
    };
    assert_eq!(
        diagnostic.code(),
        "planning.invalid_presentation.unsupported_explain_evidence_truth"
    );
    assert_eq!(
        diagnostic.message(),
        "EXPLAIN evidence and truth modes are incompatible: a counterexample cannot confirm a \
         claim and a citation cannot refute one"
    );
}

/// All four closed composition body families cross the public constructor and
/// real planner. The composition root, ordered producers, WITH identity, body
/// identity/family, and body-derived result/effect are retained together.
#[test]
fn every_composition_body_family_plans_with_exact_retention() {
    for (body, family, result, effect, body_policy) in [
        (
            ReferenceCompositionBodyV2::Relational,
            CompositionBodyFamilyV2::Query,
            ResultKindV2::Table,
            EffectClassV2::ReadOnly,
            "relational",
        ),
        (
            ReferenceCompositionBodyV2::Graph,
            CompositionBodyFamilyV2::Graph,
            ResultKindV2::PathWitness,
            EffectClassV2::ProofEvidenceOnly,
            "graph",
        ),
        (
            ReferenceCompositionBodyV2::Check,
            CompositionBodyFamilyV2::Check,
            ResultKindV2::Alert,
            EffectClassV2::ReadOnly,
            "check",
        ),
        (
            ReferenceCompositionBodyV2::Explain,
            CompositionBodyFamilyV2::Explain,
            ResultKindV2::ProofExplain,
            EffectClassV2::ReadOnly,
            "presentation",
        ),
    ] {
        let (document, input) = composition_input(body).expect("the composition carrier seals");
        let planned = plan(&input, document.document()).expect("the composition plans");
        assert_eq!(planned.result().result_kind(), result, "{body:?}");
        assert_eq!(planned.effect(), effect, "{body:?}");
        assert_eq!(
            planned.explain().root_location(),
            document.composition().location(),
            "{body:?}"
        );
        let PlanPolicyV2::Composition(policy) = planned.policy() else {
            panic!("expected composition policy for {body:?}");
        };
        assert_eq!(policy.root_item_id(), document.composition_item_id());
        assert_eq!(policy.location(), document.composition().location());
        assert_eq!(policy.body_item_id(), document.body_item_id());
        assert_eq!(policy.body_family(), family);
        assert_eq!(policy.with_binding(), document.with_binding());
        assert_eq!(policy.body().policy().keyword(), body_policy);
        let SemanticPlanInputV2::Composition(input) = &input else {
            panic!("expected checked composition input");
        };
        // Compare the complete retained payload to the sealed input. The
        // relational fixture is exactly Project -> Scan with one pure path;
        // its authored fold is pinned rather than recomputed by the planner.
        match (input.body(), policy.body().policy()) {
            (CompositionBodyPlanInputV2::Relational(_), retained) => assert_eq!(
                retained,
                &PlanPolicyV2::Relational {
                    nodes: 2,
                    effects: EffectSummaryV2::new(
                        PurityV1::Pure,
                        DeterminismV1::Deterministic,
                        MonotonicityV1::Monotone,
                        AggregationClassV1::Scalar,
                    ),
                },
            ),
            (CompositionBodyPlanInputV2::Graph(contract), PlanPolicyV2::Graph(retained)) => {
                assert_eq!(retained.item_id(), contract.item_id());
                assert_eq!(retained.kind(), contract.kind());
                assert_eq!(retained.location(), contract.location());
                assert_eq!(retained.depth(), contract.depth());
                assert_eq!(retained.direction(), contract.direction());
                assert_eq!(retained.summary_facets(), contract.summary_facets());
                assert_eq!(retained.references(), contract.references());
                assert_eq!(retained.shape(), contract.shape());
                assert_eq!(retained.max_witnesses(), contract.max_witnesses());
                assert_eq!(
                    retained.max_path_length(),
                    contract.witness().max_path_length()
                );
                assert_eq!(retained.dedup(), contract.dedup());
                assert_eq!(retained.tie_break(), contract.tie_break());
            }
            (CompositionBodyPlanInputV2::Check(contract), PlanPolicyV2::Check(retained)) => {
                assert_eq!(retained, contract.policy());
                let quanta_gqlang_v2_sema::ResolvedCompositionBodyV2::Check(sealed) =
                    document.composition().body()
                else {
                    panic!("expected sealed CHECK body");
                };
                assert_eq!(retained.mode(), sealed.mode());
                assert_eq!(retained.confirm(), sealed.confirm());
                assert_eq!(retained.emit(), sealed.emit());
                assert_eq!(retained.rule(), sealed.rule());
                assert_eq!(retained.scope(), sealed.scope());
                assert_eq!(retained.pattern(), sealed.pattern());
                assert!(
                    retained.nested_graph().is_none(),
                    "pattern-only fixture has no nested graph"
                );
            }
            (
                CompositionBodyPlanInputV2::Explain(contract),
                PlanPolicyV2::Presentation(retained),
            ) => {
                assert_eq!(retained, contract.kind());
                let quanta_gqlang_v2_sema::ResolvedCompositionBodyV2::Explain(sealed) =
                    document.composition().body()
                else {
                    panic!("expected sealed EXPLAIN body");
                };
                assert_eq!(retained, sealed.kind());
            }
            (expected, actual) => {
                panic!("composition body policy mismatch: {expected:?} / {actual:?}")
            }
        }
        assert!(
            !policy.producers().is_empty(),
            "{body:?}: a composition plan retains its ordered producers"
        );
        assert_eq!(
            policy
                .producers()
                .iter()
                .map(quanta_gqlang_v2_planning::CheckedCompositionProducerV2::item_id)
                .collect::<Vec<_>>(),
            document.producer_item_ids(),
            "{body:?}"
        );
        assert!(
            policy
                .producers()
                .iter()
                .enumerate()
                .all(|(ordinal, producer)| producer.ordinal() == ordinal),
            "{body:?}"
        );
        // Retain more than the identity: the checked name location and the
        // optional relation output are copied from the sealed binding, so a
        // dropped/relocated producer payload must fail here.
        let sealed_bindings = document.composition().bindings();
        assert_eq!(policy.producers().len(), sealed_bindings.len(), "{body:?}");
        for (producer, binding) in policy.producers().iter().zip(sealed_bindings) {
            assert_eq!(producer.item_id(), binding.producer().item_id(), "{body:?}");
            assert_eq!(
                producer.name_location(),
                binding.name_location(),
                "{body:?}"
            );
            assert_eq!(
                producer.relation_output(),
                binding.relation_output(),
                "{body:?}"
            );
        }
    }
}

/// Public construction rejects both body-family and same-family identity
/// transplants before a composition can reach `plan`.
#[test]
fn composition_body_transplants_fail_closed_at_the_public_constructor() {
    let (composition_document, graph_input) =
        composition_input(ReferenceCompositionBodyV2::Graph).expect("graph composition seals");
    let SemanticPlanInputV2::Composition(graph_input) = graph_input else {
        panic!("the fixture seals a composition carrier");
    };
    let graph_body = graph_input.body().clone();

    let (explain_document, _input) =
        composition_input(ReferenceCompositionBodyV2::Explain).expect("explain composition seals");
    let rejection = CheckedCompositionPlanInputV2::new(explain_document.composition(), graph_body)
        .expect_err("a graph body cannot replace an EXPLAIN body");
    assert_eq!(
        rejection,
        PlanningDiagnosticV1::InvalidComposition {
            reason: CompositionInvalidReasonV2::BodyKindMismatch,
            span: explain_document.composition().location().clone(),
        }
    );

    let standalone =
        graph_document(GraphQueryKindV1::Path, 1).expect("same-source standalone graph resolves");
    let standalone_body = CompositionBodyPlanInputV2::Graph(
        GraphQueryContractV2::from_resolved(standalone.graph())
            .expect("standalone graph contract seals"),
    );
    assert_eq!(
        standalone.graph().item_id().source_id(),
        composition_document.body_item_id().source_id()
    );
    assert_ne!(
        standalone.graph().item_id(),
        composition_document.body_item_id()
    );
    let rejection =
        CheckedCompositionPlanInputV2::new(composition_document.composition(), standalone_body)
            .expect_err("a same-family body identity transplant must reject");
    assert_eq!(
        rejection,
        PlanningDiagnosticV1::InvalidComposition {
            reason: CompositionInvalidReasonV2::BodyIdentityMismatch,
            span: composition_document.composition().location().clone(),
        }
    );
}

#[test]
fn composition_body_rejects_a_different_same_output_nested_relation() {
    let (document, canonical_input) =
        composition_input(ReferenceCompositionBodyV2::RelationalFilter)
            .expect("true-filter composition seals");
    let (other_document, other_input) =
        composition_input(ReferenceCompositionBodyV2::RelationalFilterFalse)
            .expect("false-filter composition seals");
    let SemanticPlanInputV2::Composition(canonical) = canonical_input else {
        panic!("fixture must produce composition input");
    };
    let SemanticPlanInputV2::Composition(other) = other_input else {
        panic!("other fixture must produce composition input");
    };
    let CompositionBodyPlanInputV2::Relational(expected) = canonical.body() else {
        panic!("canonical body is relational");
    };
    let CompositionBodyPlanInputV2::Relational(substituted) = other.body() else {
        panic!("other body is relational");
    };
    assert_eq!(
        document.composition_item_id(),
        other_document.composition_item_id()
    );
    assert_eq!(document.body_item_id(), other_document.body_item_id());
    assert_ne!(substituted.root(), expected.root(), "predicates differ");
    assert_eq!(substituted.root().output(), expected.root().output());
    assert_eq!(
        CheckedCompositionPlanInputV2::new(document.composition(), other.body().clone()),
        Err(PlanningDiagnosticV1::InvalidComposition {
            reason: CompositionInvalidReasonV2::BodyIdentityMismatch,
            span: document.composition().location().clone(),
        })
    );
}

/// Every closed composition reason code and message remains byte-stable,
/// independently of the behavioral entry rails above.
#[test]
fn invalid_composition_reasons_all_render_stable_codes() {
    let span = fixture_span();
    for (reason, suffix, message) in [
        (
            CompositionInvalidReasonV2::EmptyProducers,
            "empty_producers",
            "composition plan input has no ordered producer",
        ),
        (
            CompositionInvalidReasonV2::ProducerOrdinalMismatch,
            "producer_ordinal_mismatch",
            "composition producer ordinal is not its authored position",
        ),
        (
            CompositionInvalidReasonV2::ProducerIdentityMismatch,
            "producer_identity_mismatch",
            "composition producer identity or relation output disagrees with its sealed binding",
        ),
        (
            CompositionInvalidReasonV2::DuplicateProducer,
            "duplicate_producer",
            "composition plan input repeats a producer identity",
        ),
        (
            CompositionInvalidReasonV2::WithTargetNotAProducer,
            "with_target_not_a_producer",
            "composition WITH target is not exactly one of the ordered producers",
        ),
        (
            CompositionInvalidReasonV2::BodyIdentityMismatch,
            "body_identity_mismatch",
            "composition body input identity is not the sealed body identity",
        ),
        (
            CompositionInvalidReasonV2::BodyKindMismatch,
            "body_kind_mismatch",
            "composition body input kind disagrees with the sealed body kind",
        ),
    ] {
        let diagnostic = PlanningDiagnosticV1::InvalidComposition {
            reason,
            span: span.clone(),
        };
        assert_eq!(
            diagnostic.code(),
            format!("planning.invalid_composition.{suffix}")
        );
        assert_eq!(diagnostic.message(), message);
        assert_eq!(reason.code(), suffix);
    }
}

/// The identity-mismatch reason names and codes are stable: they are part of the
/// human message and the dot-namespaced code.
#[test]
fn input_document_mismatch_reason_names_are_stable() {
    for (reason, code, name) in [
        (
            InputDocumentMismatchReasonV2::Absent,
            "absent",
            "absent from the document",
        ),
        (
            InputDocumentMismatchReasonV2::WrongKind,
            "wrong_kind",
            "a different item kind in the document",
        ),
        (
            InputDocumentMismatchReasonV2::IdentityMismatch,
            "identity_mismatch",
            "a contradictory sealed identity in the document",
        ),
    ] {
        assert_eq!(reason.code(), code);
        assert_eq!(reason.name(), name);
        assert_eq!(
            PlanningDiagnosticV1::InputDocumentMismatch {
                reason,
                span: fixture_span()
            }
            .code(),
            format!("planning.input_document_mismatch.{code}")
        );
    }
}

/// The deleted V1 MIX vocabulary has no surviving code, and every live
/// planning-owned code keeps its documented namespace.
#[test]
fn no_deleted_mix_code_survives_in_the_diagnostic_surface() {
    let span = fixture_span();
    let mut codes = vec![
        PlanningDiagnosticV1::SemanticStepBudgetExceeded {
            limit: 8,
            span: span.clone(),
        }
        .code(),
        PlanningDiagnosticV1::InvalidPresentation {
            reason: PresentationInvalidReasonV2::UnsupportedExplainEvidenceTruth,
            span: span.clone(),
        }
        .code(),
        PlanningDiagnosticV1::NoPlan {
            reason: NoPlanReasonV2::EmptyRuleProgram,
            span: span.clone(),
        }
        .code(),
    ];
    for reason in [
        InputDocumentMismatchReasonV2::Absent,
        InputDocumentMismatchReasonV2::WrongKind,
        InputDocumentMismatchReasonV2::IdentityMismatch,
    ] {
        codes.push(
            PlanningDiagnosticV1::InputDocumentMismatch {
                reason,
                span: span.clone(),
            }
            .code(),
        );
    }
    for reason in [
        CheckInvalidReasonV2::NestedGraphIdentityMismatch,
        CheckInvalidReasonV2::UnexpectedNestedGraph,
        CheckInvalidReasonV2::MissingNestedGraph,
        CheckInvalidReasonV2::SemanticConfirmInPatternMode,
        CheckInvalidReasonV2::MissingSemanticConfirm,
        CheckInvalidReasonV2::PatternInputInSemanticMode,
        CheckInvalidReasonV2::MissingHybridPatternInput,
        CheckInvalidReasonV2::MissingHybridSemanticInput,
    ] {
        codes.push(
            PlanningDiagnosticV1::InvalidCheck {
                reason,
                span: span.clone(),
            }
            .code(),
        );
    }
    for reason in [
        CompositionInvalidReasonV2::EmptyProducers,
        CompositionInvalidReasonV2::ProducerOrdinalMismatch,
        CompositionInvalidReasonV2::ProducerIdentityMismatch,
        CompositionInvalidReasonV2::DuplicateProducer,
        CompositionInvalidReasonV2::WithTargetNotAProducer,
        CompositionInvalidReasonV2::BodyIdentityMismatch,
        CompositionInvalidReasonV2::BodyKindMismatch,
    ] {
        codes.push(
            PlanningDiagnosticV1::InvalidComposition {
                reason,
                span: span.clone(),
            }
            .code(),
        );
    }

    for code in &codes {
        assert!(
            code.starts_with("planning."),
            "`{code}` is dot-namespaced under planning."
        );
        assert!(
            !code.contains("unsupported_mix"),
            "the deleted V1 mix vocabulary has no surviving code: `{code}`"
        );
        for deleted in [
            "explain_with_export",
            "export_with_proof_only_lane",
            "mutation_with_witness",
            "ordered_recursive_closure",
            "path_witness_without_witness",
            "candidate_with_proof_evidence",
        ] {
            assert!(
                !code.contains(deleted),
                "`{deleted}` is no longer a planning code: `{code}`"
            );
        }
    }
    // The relay variants are the two codes that are NOT planning-namespaced.
    let relayed = PlanningDiagnosticV1::IllegalRuleProgram {
        logic: LogicDiagnosticV1::NegationCycle {
            cycle: vec![quanta_gqlang_v2_types::ResolvedPredicateIdV2::try_new("r")
                .expect("a stable predicate identity")],
            offending_edge: span.clone(),
            span: span.clone(),
        },
    };
    assert_eq!(relayed.code(), "logic.negation_cycle");
    assert_eq!(relayed.span(), &span);
}
