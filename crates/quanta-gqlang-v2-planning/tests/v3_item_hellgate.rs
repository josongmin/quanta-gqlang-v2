#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::wildcard_enum_match_arm,
    reason = "integration tests assert with expect/unwrap/indexing/panic and match exhaustively; allowed for test targets only"
)]

//! v3 ITEM-classification hellgate: every sealed v2 carrier family is CLASSIFIED
//! into the host-agnostic planning vocabulary, fail-closed (ticket 07
//! FRONTEND-CONTRACT parity), plus the exact sealed payload each arm retains.
//!
//! V2 cut: `classify_item` over a raw `ProgramItemHirV1` is DELETED with the
//! whole HIR ingress (the `quanta-gqlang-v2-hir` crate is not a dependency of
//! this crate). Classification now keys off the SEALED carriers only, so these
//! rails drive the real entry points and pin:
//!
//! - FIND -> `path_witness` / `proof_evidence_only`, an explicit witness
//!   selection, the exact sealed depth/cap/dedup/tie-break/path-bound retention,
//!   and the witness + recursive-closure capability set;
//! - CHECK -> `alert` / `read_only` with its exact detection/confirm/emit policy
//!   and nested confirm provenance;
//! - a plain relation -> `table` / `read_only` (the V1 "relational deferred to
//!   the algebra pipeline" placeholder is replaced by the real relational arm:
//!   in V2 the relational carrier IS ticket 05's lowered product);
//! - a document rule program -> `table` / `read_only` with its exact stratified
//!   policy (stratum count + recursive member identities);
//! - declarations / `USE PACK` / an empty program -> the TYPED no-plan marker,
//!   never an error.
//!
//! The reference matrix now drives all six presentation verbs, both meta
//! directives, and re-anchored Semi/Anti joins through the real owner entry.
//! Catalog/declaration non-plan carriers remain covered by their closed typed
//! vocabulary until Sema exposes resolved declaration fixtures.

use std::collections::BTreeSet;

use quanta_gqlang_v2_logic::DedupIdentityV1;
use quanta_gqlang_v2_planning::{
    plan, AuthorityLaneV2, CapabilityV2, CardinalityClassV2, EffectClassV2,
    EvaluationDeterminismV2, EvidenceRequirementV2, GraphWitnessRetentionV2, OrderingContractV2,
    PlanPolicyV2, RecursionRiskClassV2, RenderRequirementV2, ResultKindV2, SemanticPlanInputV2,
    WitnessExplosionRiskClassV2, WitnessSelectionPolicyV2,
};
use quanta_gqlang_v2_sema::test_support::ReferencePresentationFixtureV2;
use quanta_gqlang_v2_sema::test_support::{ReferenceRelationFixtureV2, ReferenceRuleFixtureV2};
use quanta_gqlang_v2_sema::{EvidenceModeV1, GraphQueryKindV1, SemanticItemV2, TruthModeV1};
use quanta_gqlang_v2_types::ResolvedPredicateIdV2;

mod support;

use support::{check_input, graph_input, planned_relation, relational_policy, rule_input};

/// A stable predicate identity for the recursive-member assertions.
fn predicate(name: &str) -> ResolvedPredicateIdV2 {
    ResolvedPredicateIdV2::try_new(name).expect("the fixture predicate identity is valid")
}

/// FIND (taint, source -> sink, bounded by depth) -> path_witness /
/// proof_evidence_only, explicit witness selection, populated cost schema, and
/// the EXACT sealed witness provenance retained as the plan policy.
#[test]
fn a_find_is_a_path_witness_evidence_only_result() {
    let (document, input) =
        graph_input(GraphQueryKindV1::Taint, 4).expect("the reference taint FIND");
    let planned = plan(&input, document.document()).expect("a bounded FIND plans");

    assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(
        planned.evaluation_determinism(),
        EvaluationDeterminismV2::Deterministic
    );
    assert_eq!(planned.evidence(), EvidenceRequirementV2::WitnessPayload);
    assert_eq!(planned.effect(), EffectClassV2::ProofEvidenceOnly);
    assert_eq!(planned.result().result_kind(), ResultKindV2::PathWitness);
    assert_eq!(
        planned.result().render_requirement(),
        RenderRequirementV2::WitnessPayload
    );
    // Witness selection is EXPLICIT, derived from the sealed cap.
    assert_eq!(
        planned.result().witness_selection(),
        Some(WitnessSelectionPolicyV2::AllWitness)
    );
    assert_eq!(
        planned.semantic_cost().estimated_cardinality_class,
        CardinalityClassV2::Closure
    );
    assert_eq!(
        planned.semantic_cost().recursion_risk_class,
        RecursionRiskClassV2::BoundDependent
    );
    assert_eq!(
        planned.semantic_cost().witness_explosion_risk_class,
        WitnessExplosionRiskClassV2::CapBounded
    );
    assert_eq!(
        planned.semantic_cost().ordering_contract,
        OrderingContractV2::Unordered
    );
    // FIND requires witness + recursive-closure capability.
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([
            CapabilityV2::RecursiveClosure,
            CapabilityV2::WitnessMaterialization,
        ])
    );
    assert!(planned.explain().recursive());

    let SemanticPlanInputV2::Graph(contract) = &input else {
        panic!("the fixture seals a graph carrier");
    };
    match planned.policy() {
        PlanPolicyV2::Graph(retention) => {
            assert_eq!(retention.item_id(), contract.item_id());
            assert_eq!(retention.kind(), GraphQueryKindV1::Taint);
            assert_eq!(retention.location(), contract.location());
            assert_eq!(retention.depth(), 4);
            assert_eq!(
                retention.shape(),
                quanta_gqlang_v2_logic::RelationShapeV1::Cyclic
            );
            assert_eq!(retention.max_witnesses(), contract.max_witnesses());
            assert_eq!(retention.dedup(), contract.dedup());
            assert_eq!(retention.tie_break(), contract.tie_break());
            assert_eq!(
                retention.max_path_length(),
                contract.witness().max_path_length()
            );
        }
        other => panic!("expected a graph policy, got {other:?}"),
    }
}

/// The sealed dedup identity follows the FIND subkind exactly, and every family
/// binds as a CYCLIC relation whose path bound is the authored depth with the
/// document's sealed cap.
#[test]
fn the_sealed_find_provenance_follows_the_subkind() {
    for (kind, dedup) in [
        (GraphQueryKindV1::Reachability, DedupIdentityV1::Endpoints),
        (GraphQueryKindV1::Flow, DedupIdentityV1::NodeSequence),
        (GraphQueryKindV1::Taint, DedupIdentityV1::NodeSequence),
        (GraphQueryKindV1::Path, DedupIdentityV1::NodeSequence),
        (GraphQueryKindV1::Slice, DedupIdentityV1::NodeSet),
        (GraphQueryKindV1::Summary, DedupIdentityV1::NodeSet),
        (GraphQueryKindV1::PointsTo, DedupIdentityV1::NodeSet),
    ] {
        let (document, input) = graph_input(kind, 6).expect("the reference FIND authors");
        let planned = plan(&input, document.document()).expect("a bounded FIND plans");
        let retention: &GraphWitnessRetentionV2 =
            support::graph_policy(&planned).expect("a graph policy");
        assert_eq!(retention.kind(), kind);
        assert_eq!(retention.depth(), 6, "{kind:?}");
        assert_eq!(
            retention.shape(),
            quanta_gqlang_v2_logic::RelationShapeV1::Cyclic,
            "{kind:?}"
        );
        assert_eq!(retention.dedup(), dedup, "{kind:?}");
        assert_eq!(
            retention.tie_break(),
            quanta_gqlang_v2_logic::WitnessTieBreakV1::ShortestThenLexical,
            "{kind:?}"
        );
        assert_eq!(retention.max_witnesses(), 10_000, "{kind:?}");
        assert_eq!(retention.max_path_length(), Some(6), "{kind:?}");
    }
}

/// A FIND carrier whose sealed depth was mutated is REFUSED as an identity
/// mismatch, while the pristine carrier still plans: the retained bound is a
/// sealed fact, never a re-readable knob.
#[test]
fn a_mutated_find_bound_is_refused_and_the_pristine_carrier_plans() {
    let (_document, input) = graph_input(GraphQueryKindV1::Taint, 4).expect("the taint FIND");
    let mutated = support::graph_document(GraphQueryKindV1::Taint, 5).expect("the deeper FIND");
    match plan(&input, mutated.document()) {
        Err(quanta_gqlang_v2_planning::PlanningDiagnosticV1::InputDocumentMismatch {
            reason,
            ..
        }) => {
            assert_eq!(
                reason,
                quanta_gqlang_v2_planning::InputDocumentMismatchReasonV2::IdentityMismatch
            );
        }
        other => panic!("expected a sealed-identity rejection, got {other:?}"),
    }
    let (own, input) = graph_input(GraphQueryKindV1::Taint, 4).expect("the taint FIND");
    let planned = plan(&input, own.document()).expect("the pristine carrier plans");
    assert_eq!(
        support::graph_policy(&planned)
            .expect("a graph policy")
            .depth(),
        4
    );
}

/// CHECK -> alert / read-only, with the exact sealed policy retained (V1
/// replacement for the HIR-built CHECK rail).
#[test]
fn a_check_is_an_alert_read_only_result() {
    let (document, input) = check_input().expect("the nested CHECK fixture");
    let planned = plan(&input, document.document()).expect("the CHECK plans");

    assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(planned.result().result_kind(), ResultKindV2::Alert);
    assert_eq!(planned.effect(), EffectClassV2::ReadOnly);
    assert_eq!(
        planned.result().render_requirement(),
        RenderRequirementV2::SourceLocation
    );
    assert_eq!(planned.evidence(), EvidenceRequirementV2::SourceLocation);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([
            CapabilityV2::RecursiveClosure,
            CapabilityV2::WitnessMaterialization,
        ])
    );

    let policy = support::check_policy(&planned).expect("a CHECK policy");
    assert_eq!(
        policy.mode(),
        quanta_gqlang_v2_sema::DetectionModeV1::SemanticOnly
    );
    assert_eq!(policy.confirm(), quanta_gqlang_v2_sema::ConfirmModeV1::Skip);
    assert_eq!(policy.emit(), quanta_gqlang_v2_sema::EmitModeV1::Findings);
    assert_eq!(policy.rule().location(), {
        let check = support::resolved_check(&document).expect("the resolved CHECK");
        check.rule().location()
    });
    let nested = policy
        .nested_graph()
        .expect("the nested confirm provenance");
    assert_eq!(nested.kind(), GraphQueryKindV1::Taint);
    assert_eq!(nested.depth(), 4);
    assert_eq!(nested.dedup(), DedupIdentityV1::NodeSequence);
    assert_eq!(nested.max_path_length(), Some(4));
}

/// A relational Query plans a table / read-only result directly: the V1
/// "deferred to the algebra pipeline" placeholder is replaced by the REAL
/// relational arm, because in V2 the relational carrier already IS the checked
/// lowering product of the pipeline.
#[test]
fn a_plain_relation_is_a_table_read_only_result() {
    let (document, input) =
        support::relation_input(ReferenceRelationFixtureV2::Scan).expect("the Scan fixture");
    let planned = plan(&input, document.document()).expect("a Scan plans");
    assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);
    assert_eq!(planned.result().result_kind(), ResultKindV2::Table);
    assert_eq!(planned.effect(), EffectClassV2::ReadOnly);
    assert_eq!(
        planned.result().render_requirement(),
        RenderRequirementV2::RowsOnly
    );
    assert_eq!(planned.evidence(), EvidenceRequirementV2::None);
    assert_eq!(
        planned.capabilities(),
        &BTreeSet::from([CapabilityV2::RelationScan])
    );
    let (nodes, _effects) = relational_policy(&planned).expect("a relational policy");
    assert_eq!(nodes, 1);
}

/// Every rule-program fixture plans a table whose policy retains the program's
/// exact member identities in document order, its exact stratum count, and its
/// exact recursive members.
#[test]
fn a_rule_program_retains_its_exact_stratified_policy() {
    for (fixture, strata, recursive) in [
        (ReferenceRuleFixtureV2::SingleRule, 1, Vec::new()),
        (
            ReferenceRuleFixtureV2::PositiveSelfRecursion,
            1,
            vec![predicate("r")],
        ),
        (
            ReferenceRuleFixtureV2::MutualRecursionPair,
            1,
            vec![predicate("r"), predicate("s")],
        ),
        (ReferenceRuleFixtureV2::BuiltinCallOnly, 1, Vec::new()),
        (
            ReferenceRuleFixtureV2::NonRecursiveDependency,
            1,
            Vec::new(),
        ),
        (
            ReferenceRuleFixtureV2::NegatedNonRecursiveDependency,
            2,
            Vec::new(),
        ),
    ] {
        let (document, input) = rule_input(fixture).expect("the rule fixture seals");
        let planned = plan(&input, document.document()).expect("the program plans");
        assert_eq!(
            planned.result().result_kind(),
            ResultKindV2::Table,
            "{fixture:?}"
        );
        assert_eq!(planned.effect(), EffectClassV2::ReadOnly, "{fixture:?}");
        assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);

        let policy = support::rule_policy(&planned).expect("a rule-program policy");
        assert_eq!(policy.item_ids(), document.item_ids(), "{fixture:?}");
        assert_eq!(policy.stratum_count(), strata, "{fixture:?}");
        assert_eq!(
            policy.recursive_members(),
            recursive.as_slice(),
            "{fixture:?}"
        );
        assert_eq!(
            planned.explain().recursive(),
            !recursive.is_empty(),
            "{fixture:?}"
        );
        assert_eq!(
            planned
                .capabilities()
                .contains(&CapabilityV2::RecursiveClosure),
            !recursive.is_empty(),
            "{fixture:?}"
        );
        assert_eq!(
            planned.semantic_cost().recursion_risk_class,
            if recursive.is_empty() {
                RecursionRiskClassV2::None
            } else {
                RecursionRiskClassV2::BoundedFixpoint
            },
            "{fixture:?}"
        );
    }
}

/// Every presentation verb reaches the real planner and retains its complete
/// sealed payload while deriving the verb-specific result/evidence/effect.
#[test]
fn every_presentation_verb_retains_its_sealed_payload_and_plan_shape() {
    for (fixture, result, render, evidence, effect) in [
        (
            ReferencePresentationFixtureV2::Explain {
                evidence: EvidenceModeV1::Trace,
                truth: TruthModeV1::Confirm,
            },
            ResultKindV2::ProofExplain,
            RenderRequirementV2::ProvenanceTree,
            EvidenceRequirementV2::ProofTree,
            EffectClassV2::ReadOnly,
        ),
        (
            ReferencePresentationFixtureV2::Show,
            ResultKindV2::Table,
            RenderRequirementV2::RowsOnly,
            EvidenceRequirementV2::None,
            EffectClassV2::ReadOnly,
        ),
        (
            ReferencePresentationFixtureV2::Describe,
            ResultKindV2::Alert,
            RenderRequirementV2::SourceLocation,
            EvidenceRequirementV2::SourceLocation,
            EffectClassV2::ReadOnly,
        ),
        (
            ReferencePresentationFixtureV2::Sample,
            ResultKindV2::Table,
            RenderRequirementV2::RowsOnly,
            EvidenceRequirementV2::None,
            EffectClassV2::ReadOnly,
        ),
        (
            ReferencePresentationFixtureV2::Compare,
            ResultKindV2::PathWitness,
            RenderRequirementV2::WitnessPayload,
            EvidenceRequirementV2::WitnessPayload,
            EffectClassV2::ReadOnly,
        ),
        (
            ReferencePresentationFixtureV2::Export,
            ResultKindV2::Table,
            RenderRequirementV2::SarifFindings,
            EvidenceRequirementV2::None,
            EffectClassV2::ExportOnly,
        ),
    ] {
        let (document, input) =
            support::presentation_input(fixture).expect("the presentation fixture seals");
        let planned = plan(&input, document.document()).expect("the presentation plans");
        assert_eq!(planned.result().result_kind(), result, "{fixture:?}");
        assert_eq!(planned.result().render_requirement(), render, "{fixture:?}");
        assert_eq!(planned.evidence(), evidence, "{fixture:?}");
        assert_eq!(planned.effect(), effect, "{fixture:?}");
        assert_eq!(planned.authority_lane(), AuthorityLaneV2::FrontendChecked);
        match planned.policy() {
            PlanPolicyV2::Presentation(kind) => {
                assert_eq!(kind, document.presentation().kind(), "{fixture:?}");
            }
            other => panic!("expected presentation policy for {fixture:?}, got {other:?}"),
        }
    }
}

/// The plan policy vocabulary is closed per arm: every carrier family reports
/// its own stable keyword.
#[test]
fn the_plan_policy_vocabulary_is_closed_per_arm() {
    let (_document, relation) =
        planned_relation(ReferenceRelationFixtureV2::Scan).expect("the Scan fixture");
    assert_eq!(relation.policy().keyword(), "relational");

    let (document, input) =
        graph_input(GraphQueryKindV1::Taint, 4).expect("the reference taint FIND");
    assert_eq!(
        plan(&input, document.document())
            .expect("a bounded FIND plans")
            .policy()
            .keyword(),
        "graph"
    );

    let (document, input) = check_input().expect("the nested CHECK fixture");
    assert_eq!(
        plan(&input, document.document())
            .expect("the CHECK plans")
            .policy()
            .keyword(),
        "check"
    );

    let (document, input) =
        rule_input(ReferenceRuleFixtureV2::SingleRule).expect("the single-rule fixture");
    assert_eq!(
        plan(&input, document.document())
            .expect("the program plans")
            .policy()
            .keyword(),
        "rule_program"
    );

    assert_eq!(
        PlanPolicyV2::Presentation(quanta_gqlang_v2_sema::ResolvedPresentationKindV2::Show {
            category: quanta_gqlang_v2_sema::ShowCategoryV1::Capabilities,
        })
        .keyword(),
        "presentation"
    );
    assert_eq!(
        PlanPolicyV2::MetaRequireCertified.keyword(),
        "meta_require_certified"
    );
}

/// A relational carrier is exactly ticket 05's sealed lowering product: the
/// carrier's item identity, root stage key, and output contract all agree with
/// the document item's own sealed facts.
#[test]
fn a_relational_carrier_is_the_sealed_lowering_product() {
    let (document, input) =
        support::relation_input(ReferenceRelationFixtureV2::SelfJoinInner).expect("self join");
    let SemanticPlanInputV2::Relational(carrier) = &input else {
        panic!("the fixture seals a relational carrier");
    };
    let Some(SemanticItemV2::Relation(relation)) = document.document().item(document.item_id())
    else {
        panic!("the fixture item is a resolved relation");
    };
    assert_eq!(carrier.item_id(), relation.item_id());
    assert_eq!(carrier.root().item_id(), relation.item_id());
    assert_eq!(carrier.root().stage_key(), relation.root().key());
    assert_eq!(carrier.output(), relation.output());
    assert_eq!(carrier.root().location(), relation.root().location());
}

/// Canonical lowering may promote a pure EXISTS/NOT EXISTS stage to the root.
/// Planning accepts that re-anchoring because item identity and the complete
/// output contract remain sealed, while the plan stays anchored on the authored
/// relation item location.
#[test]
fn every_reanchored_exists_family_plans_from_its_sealed_owner() {
    for fixture in [
        ReferenceRelationFixtureV2::Exists,
        ReferenceRelationFixtureV2::NotExists,
        ReferenceRelationFixtureV2::CorrelatedExists,
        ReferenceRelationFixtureV2::CorrelatedNotExists,
        ReferenceRelationFixtureV2::NestedExistsTwoLevels,
    ] {
        let (document, input) = support::relation_input(fixture).expect("the fixture lowers");
        let SemanticPlanInputV2::Relational(carrier) = &input else {
            panic!("the fixture seals a relational carrier");
        };
        let Some(SemanticItemV2::Relation(relation)) = document.document().item(document.item_id())
        else {
            panic!("the fixture item is a resolved relation");
        };
        assert_eq!(carrier.item_id(), relation.item_id(), "{fixture:?}");
        assert_eq!(carrier.root().item_id(), relation.item_id(), "{fixture:?}");
        assert_eq!(carrier.output(), relation.output(), "{fixture:?}");
        assert_ne!(
            carrier.root().stage_key(),
            relation.root().key(),
            "{fixture:?}: this fixture must exercise root re-anchoring"
        );
        assert_ne!(
            carrier.root().location(),
            relation.root().location(),
            "{fixture:?}"
        );
        let planned = plan(&input, document.document()).expect("the re-anchored relation plans");
        assert_eq!(planned.result().result_kind(), ResultKindV2::Table);
        assert_eq!(
            planned.explain().root_location(),
            relation.location(),
            "{fixture:?}"
        );
        assert_eq!(planned.policy().keyword(), "relational");
    }
}

/// The relational arms that are NOT re-anchored by lowering keep full identity
/// agreement, so the re-anchored rail above does not weaken ordinary root
/// identity.
#[test]
fn an_unreanchored_relational_carrier_keeps_stage_key_identity() {
    for fixture in [
        ReferenceRelationFixtureV2::Scan,
        ReferenceRelationFixtureV2::Filter,
        ReferenceRelationFixtureV2::CrossJoin,
        ReferenceRelationFixtureV2::InnerJoinOn,
        ReferenceRelationFixtureV2::SelfJoinInner,
        ReferenceRelationFixtureV2::ProjectMultipleExpressions,
        ReferenceRelationFixtureV2::GroupedAggregateSum,
        ReferenceRelationFixtureV2::ScalarCountStar,
        ReferenceRelationFixtureV2::SortAsc,
        ReferenceRelationFixtureV2::ExistsWithBoolConjunct,
    ] {
        let (document, input) = support::relation_input(fixture).expect("the fixture lowers");
        let SemanticPlanInputV2::Relational(carrier) = &input else {
            panic!("the fixture seals a relational carrier");
        };
        let Some(SemanticItemV2::Relation(relation)) = document.document().item(document.item_id())
        else {
            panic!("the fixture item is a resolved relation");
        };
        assert_eq!(
            carrier.root().stage_key(),
            relation.root().key(),
            "{fixture:?}"
        );
        let planned = plan(&input, document.document()).expect("the fixture plans");
        assert_eq!(
            planned.result().result_kind(),
            ResultKindV2::Table,
            "{fixture:?}"
        );
    }
}
