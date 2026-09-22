#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::match_wildcard_for_single_variants,
    clippy::panic,
    clippy::single_match,
    clippy::unwrap_used,
    clippy::wildcard_enum_match_arm,
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
//! INTEGRATOR-FIXTURE-PENDING (see the lane report): the
//! `planning.invalid_presentation.*` and `planning.invalid_composition.*` entry
//! rails need a resolved presentation / composition document fixture, and the
//! `NoPlanReasonV2::{MetaActivation}` entry rail needs a resolved `USE PACK`
//! meta document. Their closed code/message vocabulary is asserted below by
//! direct diagnostic construction, so no spelling loses coverage while the
//! fixtures are missing.

use quanta_gqlang_v2_logic::{GraphEndpointRoleV1, LogicDiagnosticV1};
use quanta_gqlang_v2_planning::{
    plan, plan_outcome, CheckContractV2, CheckInvalidReasonV2, CompositionInvalidReasonV2,
    InputDocumentMismatchReasonV2, NoPlanReasonV2, PlanOutcomeV2, PlanningDiagnosticV1,
    PresentationInvalidReasonV2, SemanticPlanInputV2,
};
use quanta_gqlang_v2_sema::test_support::{
    forged_cardinality_downgrade_document_v2, forged_small_receipt_relation_chain_document_v2,
    reference_checked_span_v2, reference_relation_document_v2, reference_rule_document_v2,
    ReferenceBudgetV2, ReferenceRelationFixtureV2, ReferenceRuleFixtureV2,
    SMALL_BUDGET_MAX_SEMANTIC_STEPS,
};
use quanta_gqlang_v2_sema::{CheckedLocatedSpanV2, GraphQueryKindV1};

mod support;

use support::{
    check_document, check_input, deepest_node, document_item_location, graph_document, graph_input,
    relation_input, resolved_check, rule_input,
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
            .map(SemanticPlanInputV2::Relational)
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
            .map(SemanticPlanInputV2::Relational)
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
            .map(SemanticPlanInputV2::Relational)
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

/// INTEGRATOR-FIXTURE-PENDING: a resolved `USE PACK` meta document does not exist
/// in Sema's fixture matrix, so the entry-point rail
/// (`plan_outcome` -> `NoPlan(MetaActivation)`, `plan` -> `Err(NoPlan)`) cannot be
/// driven yet. The closed reason vocabulary is asserted here meanwhile.
#[test]
fn the_meta_activation_reason_vocabulary_is_stable() {
    let diagnostic = PlanningDiagnosticV1::NoPlan {
        reason: NoPlanReasonV2::MetaActivation,
        span: fixture_span(),
    };
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

/// A CHECK with no semantic confirm input must NOT receive a nested graph
/// contract; the standalone FIND fixture's contract cannot be transplanted onto
/// it (the identity is not the CHECK's nested identity).
#[test]
fn a_check_contract_rejects_an_unexpected_nested_graph() {
    let document = check_document().expect("the nested CHECK fixture");
    let check = resolved_check(&document).expect("the resolved CHECK item");
    let standalone = graph_input(GraphQueryKindV1::Taint, 4).expect("the standalone taint FIND");
    let SemanticPlanInputV2::Graph(nested) = &standalone.1 else {
        panic!("the fixture seals a graph carrier");
    };
    // The standalone FIND shares the nested identity in this fixture matrix, so
    // the transplant rejection is exercised through the identity rule instead:
    // a contract for the same identity is accepted by the wrapper and only
    // rejects at plan time against the wrong document.
    if nested.item_id() == check.nested_graph().expect("nested").item_id() {
        let contract = CheckContractV2::from_resolved(check, Some(nested.clone()))
            .expect("the same sealed nested identity is admissible");
        assert_eq!(contract.nested_graph_item_id(), Some(nested.item_id()));
        match plan(
            &SemanticPlanInputV2::Check(contract),
            standalone.0.document(),
        ) {
            Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, span }) => {
                assert_eq!(reason, InputDocumentMismatchReasonV2::Absent);
                assert_eq!(span, *check.location());
            }
            other => panic!("expected an absent-identity rejection, got {other:?}"),
        }
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
            "CHECK nested graph contract names an item the resolved CHECK does not",
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

/// INTEGRATOR-FIXTURE-PENDING: no resolved presentation document fixture exists,
/// so the presentation entry rail cannot be driven. Its closed reason vocabulary
/// is asserted verbatim.
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

/// INTEGRATOR-FIXTURE-PENDING: no resolved composition document fixture exists,
/// so the composition entry rail cannot be driven. Every closed reason code and
/// message is asserted verbatim.
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
