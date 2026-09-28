#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "the adapter suite pins exact projected rule payloads and exact fail-closed rejection payloads"
)]
//! Adapter suite for [`RuleProgramContractV2`] (SS-06B, lane W3-C).
//!
//! The retired V1 `source.rs` lowered the source quantifiers `exists` /
//! `not exists` into Algebra `SemiJoin` / `AntiJoin`. Relational existence is
//! owned by semantic analysis and lowered by algebra, so this suite first proves
//! that no logic rule program materializes out of any relational-EXISTS document
//! and that the rule-graph budget is always the document's own immutable receipt.
//!
//! The rule-bearing half runs against the lane W3-D reference fixtures: a real
//! rule document resolved through the sealed semantic transaction must project
//! member-for-member into lane W3-A's program carrier with exact identities,
//! call targets, polarities, and checked locations, and every forged membership
//! or call fact must fail closed with its named `logic.rule_program.*` reason.
//!
//! Rule-document fixtures are minted by `quanta_gqlang_v2_sema::test_support`
//! (feature `test`); this file never builds a semantic document by hand and
//! never introduces a logic-side document DTO.

use quanta_gqlang_v2_logic::{
    stratify, BodyLiteralV1, BodyPolarityV1, LogicDiagnosticV1, RecursiveProgramV1, RuleDefV1,
    RuleProgramContractV2, RuleProgramInvalidReasonV1, StratificationProofV1,
};
use quanta_gqlang_v2_sema::test_support::{
    deep_relation_chain_document_v2, forged_non_source_rule_call_document_v2,
    forged_rule_call_shape_target_document_v2, forged_rule_call_wrapper_document_v2,
    forged_rule_fact_document_v2, forged_rule_membership_document_v2,
    forged_small_receipt_relation_chain_document_v2, reference_absent_item_id_v2,
    reference_relation_document_v2, reference_rule_document_v2, second_source_relation_document_v2,
    ForgedRuleCallWrapperV2, ForgedRuleFactMutationV2, ForgedRuleMembershipMutationV2,
    ReferenceBudgetV2, ReferenceRelationFixtureV2, ReferenceRuleFixtureV2,
    SECONDARY_REFERENCE_SOURCE_V2, SMALL_BUDGET_MAX_SEMANTIC_STEPS,
};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, FrontendBudgetV2, SemanticDocumentV2, SpanV1, TypedExprV2,
};
use quanta_gqlang_v2_types::ResolvedCallTargetV2;

/// `max_rule_graph_nodes` of `FrontendBudgetV2::standard_v2()`.
const STANDARD_RULE_GRAPH_NODES: u32 = 50_000;
/// `max_rule_graph_edges` of `FrontendBudgetV2::standard_v2()`.
const STANDARD_RULE_GRAPH_EDGES: u32 = 200_000;
/// `max_rule_steps` of `FrontendBudgetV2::standard_v2()`.
const STANDARD_RULE_STEPS: u32 = 1_000_000;
/// `max_semantic_steps` of `FrontendBudgetV2::standard_v2()`.
const STANDARD_SEMANTIC_STEPS: u32 = 1_000_000;

fn predicate(name: &str) -> quanta_gqlang_v2_types::ResolvedPredicateIdV2 {
    quanta_gqlang_v2_types::ResolvedPredicateIdV2::try_new(name)
        .expect("fixture predicate name is a valid predicate identity")
}

/// Project one document, expecting a faithful (accepted) contract.
fn accepted(document: &SemanticDocumentV2) -> RuleProgramContractV2 {
    RuleProgramContractV2::from_document(document)
        .expect("a real sealed document is a faithful rule-program contract")
}

/// Project one forged document, expecting exactly one typed rejection.
fn rejection(document: &SemanticDocumentV2) -> LogicDiagnosticV1 {
    match RuleProgramContractV2::from_document(document) {
        Ok(contract) => panic!(
            "forged rule program must reject, got {} accepted member(s)",
            contract.rules().len()
        ),
        Err(error) => error,
    }
}

/// The INDEPENDENT code suffix of one rule-program reason.
///
/// `assert_invalid` must not derive its expectation from `reason.code()`: the
/// diagnostic under test is itself built from `reason.code()`, so calling the
/// same production accessor on both sides makes the code assertion unable to
/// fail however the spellings drift.
fn expected_invalid_suffix(reason: RuleProgramInvalidReasonV1) -> &'static str {
    match reason {
        RuleProgramInvalidReasonV1::MissingMember => "missing_member",
        RuleProgramInvalidReasonV1::DuplicateMember => "duplicate_member",
        RuleProgramInvalidReasonV1::NonLocalTarget => "nonlocal_target",
        RuleProgramInvalidReasonV1::WrongCallId => "wrong_call_id",
        RuleProgramInvalidReasonV1::TargetTransplanted => "target_transplanted",
        RuleProgramInvalidReasonV1::PolarityFlip => "polarity_flip",
        RuleProgramInvalidReasonV1::ForeignLocation => "foreign_location",
        RuleProgramInvalidReasonV1::MembershipMismatch => "membership_mismatch",
    }
}

/// Assert the exact fail-closed payload of one rule-program rejection.
fn assert_invalid(
    error: &LogicDiagnosticV1,
    reason: RuleProgramInvalidReasonV1,
    message: &str,
    span: &CheckedLocatedSpanV2,
) {
    let suffix = expected_invalid_suffix(reason);
    assert_eq!(error.code(), format!("logic.rule_program.{suffix}"));
    assert_eq!(error.message(), message);
    assert_eq!(error.span(), span);
}

/// The exact rejection message of one rule-program reason.
fn message_of(reason: RuleProgramInvalidReasonV1) -> &'static str {
    match reason {
        RuleProgramInvalidReasonV1::MissingMember => {
            "rule program contract is incomplete: a document rule member was not enumerated"
        }
        RuleProgramInvalidReasonV1::DuplicateMember => {
            "rule program contract enumerated the same member more than once"
        }
        RuleProgramInvalidReasonV1::NonLocalTarget => {
            "rule call target is not a predicate of this document"
        }
        RuleProgramInvalidReasonV1::WrongCallId => {
            "rule call identity disagrees with the document's resolved call"
        }
        RuleProgramInvalidReasonV1::TargetTransplanted => {
            "rule call target disagrees with the document's resolved target"
        }
        RuleProgramInvalidReasonV1::PolarityFlip => {
            "rule call polarity disagrees with the document's resolved polarity"
        }
        RuleProgramInvalidReasonV1::ForeignLocation => {
            "rule fact location or registry identity belongs to another document"
        }
        RuleProgramInvalidReasonV1::MembershipMismatch => {
            "rule membership disagrees with the document's own item set"
        }
    }
}

/// The one projected rule whose head is `head`.
fn program_rule<'a>(program: &'a [RuleDefV1], head: &str) -> &'a RuleDefV1 {
    program
        .iter()
        .find(|rule| rule.head().as_str() == head)
        .expect("projected program carries the expected member predicate")
}

/// The one body reference of `rule` targeting `target`.
fn body_literal<'a>(rule: &'a RuleDefV1, target: &str) -> &'a BodyLiteralV1 {
    let mut matches = rule
        .body()
        .iter()
        .filter(|literal| literal.predicate().as_str() == target);
    let literal = matches
        .next()
        .expect("projected rule body carries the expected reference");
    assert!(
        matches.next().is_none(),
        "projected rule body carries exactly one reference to `{target}`"
    );
    literal
}

/// The registry-proven call fact of a rule's only resolved call.
fn only_call(
    rule: &quanta_gqlang_v2_sema::ResolvedRuleV2,
) -> &quanta_gqlang_v2_sema::ResolvedRuleCallV2 {
    assert_eq!(rule.calls().len(), 1, "fixture rule carries one call edge");
    rule.calls()
        .first()
        .expect("a one-call rule has a first call edge")
}

// ---- reachable without rule fixtures: the document receipt and EXISTS ownership

#[test]
fn relation_document_projects_an_empty_program_with_the_documents_own_receipt() {
    let reference =
        reference_relation_document_v2(ReferenceRelationFixtureV2::Scan).expect("scan resolves");
    let document = reference.document();
    let contract = accepted(document);

    assert!(contract.rules().is_empty());
    assert!(contract.to_program().is_empty());
    assert_eq!(contract.budget(), document.budget_receipt());
    assert_eq!(
        contract.budget().max_rule_graph_nodes(),
        STANDARD_RULE_GRAPH_NODES
    );
    assert_eq!(
        contract.budget().max_rule_graph_edges(),
        STANDARD_RULE_GRAPH_EDGES
    );
    assert_eq!(contract.budget().max_rule_steps(), STANDARD_RULE_STEPS);
    assert_eq!(
        contract.budget().max_semantic_steps(),
        STANDARD_SEMANTIC_STEPS
    );
}

#[test]
fn relational_exists_documents_never_project_logic_rule_members() {
    // Relational EXISTS / NOT EXISTS is owned by semantic analysis and lowered by
    // algebra (SS-05C). No EXISTS shape may produce a logic rule, a rule member,
    // or a rule-body edge — the retired V1 lowerer has no successor here.
    //
    // The table must carry EVERY relational-EXISTS fixture variant; dropping one
    // silently narrows the "no EXISTS shape" claim to the shapes still listed.
    for fixture in [
        ReferenceRelationFixtureV2::Exists,
        ReferenceRelationFixtureV2::NotExists,
        ReferenceRelationFixtureV2::NegatedExists,
        ReferenceRelationFixtureV2::ExistsWithBoolConjunct,
        ReferenceRelationFixtureV2::ConjunctiveExistsPair,
        ReferenceRelationFixtureV2::DisjunctiveExistsPair,
        ReferenceRelationFixtureV2::CorrelatedExists,
        ReferenceRelationFixtureV2::CorrelatedNotExists,
        ReferenceRelationFixtureV2::NestedExistsTwoLevels,
        ReferenceRelationFixtureV2::ExistsWithMultiplicativeRight,
    ] {
        let reference = reference_relation_document_v2(fixture).expect("EXISTS fixture resolves");
        let document = reference.document();
        let contract = accepted(document);
        assert!(
            contract.rules().is_empty(),
            "relational EXISTS {fixture:?} must not become a logic rule member"
        );
        assert!(
            contract.to_program().is_empty(),
            "relational EXISTS {fixture:?} must not become a logic rule body"
        );
        assert_eq!(contract.budget(), document.budget_receipt());
        assert!(
            document.expressions().any(|(_key, expression)| matches!(
                expression.shape(),
                quanta_gqlang_v2_sema::TypedExprShapeV2::RelationExists { .. }
            )),
            "the fixture really carries a relational EXISTS fact"
        );
    }
}

#[test]
fn small_receipt_document_budget_is_read_from_the_document_not_a_default() {
    // A forged document whose immutable receipt carries the small semantic-step
    // limit proves the contract copies THIS document's receipt: a default or
    // caller-selected budget would report the standard limit instead.
    let reference =
        forged_small_receipt_relation_chain_document_v2(2).expect("small-receipt chain resolves");
    let document = reference.document();
    let contract = accepted(document);

    assert_eq!(contract.budget(), document.budget_receipt());
    assert_eq!(
        contract.budget().max_semantic_steps(),
        SMALL_BUDGET_MAX_SEMANTIC_STEPS
    );
    assert_ne!(
        contract.budget().max_semantic_steps(),
        STANDARD_SEMANTIC_STEPS
    );
    // The rule-graph dimensions are untouched by that downgrade: they are this
    // document's exact receipt values, not a projection of the relation walk.
    assert_eq!(
        contract.budget().max_rule_graph_nodes(),
        STANDARD_RULE_GRAPH_NODES
    );
    assert_eq!(
        contract.budget().max_rule_graph_edges(),
        STANDARD_RULE_GRAPH_EDGES
    );
    assert_eq!(contract.budget().max_rule_steps(), STANDARD_RULE_STEPS);
    assert!(contract.rules().is_empty());
}

#[test]
fn deep_chain_document_receipt_is_reproduced_exactly() {
    let reference = deep_relation_chain_document_v2(3, ReferenceBudgetV2::Standard)
        .expect("deep chain resolves");
    let document = reference.document();
    let contract = accepted(document);

    assert_eq!(contract.budget(), document.budget_receipt());
    let standard = FrontendBudgetV2::standard_v2().expect("standard budget is constructible");
    assert_eq!(
        contract.budget().max_semantic_steps(),
        standard.max_semantic_steps()
    );
    assert!(contract.to_program().is_empty());

    // No document-local default: the receipt equality above is the whole budget,
    // including every dimension this adapter does not itself consume.
    assert_eq!(contract.budget().max_witnesses(), standard.max_witnesses());
    assert_eq!(
        contract.budget().max_diagnostics(),
        standard.max_diagnostics()
    );
}

#[test]
fn foreign_registry_document_projects_an_empty_program() {
    // A second source registry identity proves the adapter does not compare
    // document facts against an invented local registry. With no Rule item there
    // is no rule fact to validate, so the empty program is the faithful one.
    let reference = second_source_relation_document_v2().expect("secondary scan resolves");
    let document = reference.document();
    assert_eq!(document.source_id(), SECONDARY_REFERENCE_SOURCE_V2);

    let contract = accepted(document);
    assert!(contract.rules().is_empty());
    assert!(contract.to_program().is_empty());
    assert_eq!(contract.budget(), document.budget_receipt());
}

// ---- rule-bearing fixtures (lane W3-D) -------------------------------------
//
// The tests below are the exact projection and rejection rails of SS-06B. They
// compile and run once the lane W3-D `test_support` fixture surface named here
// lands; until then this file is the executable specification of that surface.

#[test]
fn single_rule_document_projects_its_exact_member_and_locations() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::SingleRule)
        .expect("single-rule fixture resolves");
    let document = fixture.document();
    let contract = accepted(document);

    assert_eq!(
        contract
            .rules()
            .iter()
            .map(quanta_gqlang_v2_sema::ResolvedRuleV2::item_id)
            .collect::<Vec<_>>(),
        fixture.item_ids()
    );
    assert_eq!(
        contract.rules().len(),
        1,
        "the fixture projects exactly one member, so `.first()` is that member"
    );
    let rule = contract.rules().first().expect("one projected member");
    assert_eq!(rule.predicate_id(), &predicate("r"));
    assert!(
        rule.calls().is_empty(),
        "a rule with no predicate call retains no graph edge"
    );

    let program = contract.to_program();
    assert_eq!(program.len(), 1);
    assert_eq!(
        program.first().map(RuleDefV1::head),
        Some(&predicate("r")),
        "the single projected program member IS the contract's single member"
    );
    let projected = program_rule(&program, "r");
    assert!(projected.body().is_empty());
    assert_eq!(projected.location(), rule.body().expression().location());
    assert_eq!(contract.budget(), document.budget_receipt());

    // The contract retains the ONE stratification proof of exactly this member
    // set under the document's own receipt: recomputing it over the contract's
    // own projection reproduces the retained proof, and a rule with no predicate
    // call admits with no recursive member. This is the F-06 seam: the ingress
    // charges the rule-graph dimensions once and downstream consumers read THIS
    // proof instead of re-stratifying.
    let recomputed: StratificationProofV1 = stratify(
        &RecursiveProgramV1::from_rules(contract.to_program()),
        &contract.budget(),
    )
    .expect("the admitted member set re-stratifies identically");
    assert_eq!(contract.stratification(), &recomputed);
    assert!(contract.stratification().recursive().is_empty());
    assert_eq!(
        contract.stratification().stratum_of(&predicate("r")),
        Some(0),
        "every member of the admitted program carries an explicit stratum"
    );
}

#[test]
fn positive_self_recursion_projects_one_positive_edge() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::PositiveSelfRecursion)
        .expect("positive self-recursion fixture resolves");
    let document = fixture.document();
    let contract = accepted(document);

    assert_eq!(
        contract.rules().len(),
        1,
        "the fixture projects exactly one member, so `.first()` is that member"
    );
    let rule = contract.rules().first().expect("one projected member");
    let call = only_call(rule);
    assert_eq!(
        call.target(),
        &ResolvedCallTargetV2::Predicate(predicate("r"))
    );
    assert_eq!(
        call.polarity(),
        quanta_gqlang_v2_sema::RuleCallPolarityV2::Positive
    );
    // The call node is a registered document expression fact, and its location
    // is the same registry-proven location the contract retains.
    let fact = document
        .expression(call.call())
        .expect("the resolved call node is a document expression fact");
    assert_eq!(fact.resolved_call(), Some(call.target()));
    assert_eq!(call.location(), fact.location());

    let program = contract.to_program();
    assert_eq!(program.len(), 1, "one member projects one program rule");
    let projected = program_rule(&program, "r");
    assert_eq!(
        projected.body().len(),
        1,
        "the self-recursion fixture projects exactly one edge"
    );
    let literal = body_literal(projected, "r");
    assert_eq!(literal.polarity(), BodyPolarityV1::Positive);
    assert_eq!(literal.location(), call.location());
    assert_eq!(literal.aggregate_function(), None);

    // The retained proof admits THIS program: the recomputation over the
    // contract's own projection is identical, `r` is its one recursive member,
    // and the positive self-edge lands in a single stratum whose least-fixpoint
    // evaluation is the admitted semantics.
    let recomputed = stratify(
        &RecursiveProgramV1::from_rules(contract.to_program()),
        &contract.budget(),
    )
    .expect("positive recursion admits under the document's own receipt");
    assert_eq!(contract.stratification(), &recomputed);
    assert!(contract.stratification().is_recursive(&predicate("r")));
    assert_eq!(contract.stratification().stratum_count(), 1);
    assert_eq!(contract.stratification().components().len(), 1);
}

#[test]
fn negated_self_recursion_is_refused_at_the_sole_ingress() {
    // A negated self-recursion is stratifiable by no proof, so the SOLE ingress
    // refuses it before any contract — and therefore any projected program —
    // exists. The `BodyPolarityV1::Negated` projection of an admitted document is
    // covered by `negated_nonrecursive_dependency_projects_one_negated_edge`; the
    // negation-cycle rejection is logic's own, reported at the rule that closed
    // the cycle.
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::NegatedSelfRecursion)
        .expect("negated self-recursion fixture resolves");
    let document = fixture.document();
    assert_eq!(
        document.rules().count(),
        1,
        "the fixture document has exactly one member, so `.next()` is that member"
    );
    let rule = document.rules().next().expect("one document member");
    let call = only_call(rule);
    assert_eq!(
        call.polarity(),
        quanta_gqlang_v2_sema::RuleCallPolarityV2::Negated,
        "the fixture document really retains a negated call edge"
    );

    let error = match RuleProgramContractV2::from_document(document) {
        Ok(contract) => panic!("a negated cycle has no stratification proof: {contract:?}"),
        Err(error) => error,
    };
    assert_eq!(error.code(), "logic.negation_cycle");
    assert_eq!(
        error.message(),
        "recursion through negation is not stratifiable: cycle [r]"
    );
    assert_eq!(error.span(), rule.body().expression().location());
}

#[test]
fn aggregate_wrapped_source_call_rejects_before_logic_ingress() {
    for fixture in [
        ReferenceRuleFixtureV2::AggregatedSelfRecursion,
        ReferenceRuleFixtureV2::AggregatedNonRecursiveDependency,
    ] {
        let error = reference_rule_document_v2(fixture)
            .expect_err("aggregate argument cannot prove a direct source literal");
        assert!(
            error.contains("sema.invalid_item_shape"),
            "{fixture:?}: {error}"
        );
        assert!(
            error.contains("source predicate call is not a direct rule truth literal"),
            "{fixture:?}: {error}"
        );
    }
}

#[test]
fn mutual_recursion_pair_projects_document_order_members_and_exact_edges() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)
        .expect("mutual-recursion fixture resolves");
    let document = fixture.document();
    let contract = accepted(document);

    // The complete member set, in document order, is exactly the document's Rule
    // item set — nothing dropped, duplicated, or invented.
    let member_ids: Vec<_> = contract
        .rules()
        .iter()
        .map(quanta_gqlang_v2_sema::ResolvedRuleV2::item_id)
        .collect();
    assert_eq!(member_ids, fixture.item_ids());
    assert_eq!(fixture.item_ids().len(), 2);

    let program = contract.to_program();
    assert_eq!(program.len(), 2);
    for (head, target) in [("r", "s"), ("s", "r")] {
        let rule = contract
            .rules()
            .iter()
            .find(|rule| rule.predicate_id().as_str() == head)
            .expect("both members are projected");
        let call = only_call(rule);
        assert_eq!(
            call.target(),
            &ResolvedCallTargetV2::Predicate(predicate(target))
        );
        assert_eq!(
            call.polarity(),
            quanta_gqlang_v2_sema::RuleCallPolarityV2::Positive
        );
        assert_eq!(
            program_rule(&program, head).body().len(),
            1,
            "each member of the mutual pair projects exactly one edge"
        );
        let literal = body_literal(program_rule(&program, head), target);
        assert_eq!(literal.polarity(), BodyPolarityV1::Positive);
        assert_eq!(literal.location(), call.location());
    }
}

#[test]
fn authored_rule_order_is_independent_of_identity_order() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::MutualRecursionPair)
        .expect("mutual-recursion fixture resolves");
    for (reverse, expected) in [(false, ["r", "s"]), (true, ["s", "r"])] {
        let document = fixture
            .document()
            .clone()
            .forged_membership(|order, _items| {
                if reverse {
                    order.reverse();
                }
            });
        assert_eq!(document.order().len(), 2);
        assert_eq!(
            document.order().windows(2).all(|pair| {
                let [left, right] = pair else {
                    return false;
                };
                left < right
            }),
            !reverse,
            "the control must actually differ from canonical identity order",
        );
        let contract = accepted(&document);
        assert_eq!(
            contract
                .rules()
                .iter()
                .map(|rule| rule.predicate_id().as_str())
                .collect::<Vec<_>>(),
            expected,
        );
        let program = contract.to_program();
        assert_eq!(
            program
                .iter()
                .map(|rule| rule.head().as_str())
                .collect::<Vec<_>>(),
            expected,
        );
        let proof = contract.stratification();
        assert_eq!(proof.components(), &[vec![predicate("r"), predicate("s")]]);
        assert_eq!(proof.stratum_of(&predicate("r")), Some(0));
        assert_eq!(proof.stratum_of(&predicate("s")), Some(0));
        assert!(proof.is_recursive(&predicate("r")));
        assert!(proof.is_recursive(&predicate("s")));
        assert_eq!(contract.budget(), document.budget_receipt());
    }
}

#[test]
fn nonrecursive_dependency_projects_exactly_one_edge_on_one_member() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::NonRecursiveDependency)
        .expect("nonrecursive dependency fixture resolves");
    let contract = accepted(fixture.document());
    let program = contract.to_program();

    assert_eq!(program.len(), 2);
    assert!(program_rule(&program, "s").body().is_empty());
    assert_eq!(
        program_rule(&program, "r").body().len(),
        1,
        "`r` projects exactly one edge on `s`"
    );
    let literal = body_literal(program_rule(&program, "r"), "s");
    assert_eq!(literal.polarity(), BodyPolarityV1::Positive);
    assert_eq!(
        contract
            .rules()
            .iter()
            .map(quanta_gqlang_v2_sema::ResolvedRuleV2::item_id)
            .collect::<Vec<_>>(),
        fixture.item_ids()
    );
}

#[test]
fn negated_nonrecursive_dependency_projects_one_negated_edge() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::NegatedNonRecursiveDependency)
        .expect("negated nonrecursive dependency fixture resolves");
    let contract = accepted(fixture.document());
    let program = contract.to_program();

    assert_eq!(
        program.len(),
        2,
        "the negated-dependency fixture projects two members"
    );
    assert_eq!(
        program_rule(&program, "r").body().len(),
        1,
        "`r` projects exactly one negated edge"
    );
    let literal = body_literal(program_rule(&program, "r"), "s");
    assert_eq!(literal.polarity(), BodyPolarityV1::Negated);
    assert!(program_rule(&program, "s").body().is_empty());
}

#[test]
fn builtin_call_body_never_becomes_a_local_graph_node() {
    // SS-06B step 4: a builtin/host call stays a typed body fact. The document
    // retains no resolved source-predicate edge for it, so the projected program
    // has no vertex and no edge for the builtin spelling, while the member itself
    // still projects.
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::BuiltinCallOnly)
        .expect("builtin-call fixture resolves");
    let document = fixture.document();
    let contract = accepted(document);

    assert_eq!(
        contract.rules().len(),
        1,
        "the fixture projects exactly one member, so `.first()` is that member"
    );
    let rule = contract.rules().first().expect("one projected member");
    assert_eq!(rule.predicate_id(), &predicate("r"));
    assert!(rule.calls().is_empty());
    assert!(
        document.expressions().any(|(_key, expression)| expression
            .resolved_call()
            .is_some_and(|target| !matches!(target, ResolvedCallTargetV2::Predicate(_)))),
        "the fixture really carries a builtin/host call fact"
    );

    let program = contract.to_program();
    assert_eq!(program.len(), 1);
    assert!(program_rule(&program, "r").body().is_empty());
}

#[test]
fn forged_builtin_call_edge_is_wrong_call_id() {
    let forged = forged_non_source_rule_call_document_v2()
        .expect("the real builtin call fact can be forged into the source edge list");
    let document = forged.document();
    let rule = document.rules().next().expect("one forged member");
    let call = only_call(rule);
    assert!(!matches!(call.target(), ResolvedCallTargetV2::Predicate(_)));
    assert_eq!(
        document
            .expression(call.call())
            .expect("the builtin call keeps its typed fact")
            .resolved_call(),
        Some(call.target())
    );

    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::WrongCallId,
        message_of(RuleProgramInvalidReasonV1::WrongCallId),
        call.location(),
    );
}

#[test]
fn forged_scalar_wrappers_cannot_smuggle_source_literal_edges() {
    let wrappers = [
        ForgedRuleCallWrapperV2::Unary,
        ForgedRuleCallWrapperV2::Binary,
        ForgedRuleCallWrapperV2::Compare,
        ForgedRuleCallWrapperV2::Or,
        ForgedRuleCallWrapperV2::Paren,
        ForgedRuleCallWrapperV2::Path,
        ForgedRuleCallWrapperV2::Record,
        ForgedRuleCallWrapperV2::List,
        ForgedRuleCallWrapperV2::LogicalQuantifier,
    ];
    for wrapper in wrappers {
        let fixture = forged_rule_call_wrapper_document_v2(wrapper)
            .expect("wrapper fixture retains the original resolved edge");
        let call_location = fixture
            .document()
            .rules()
            .next()
            .expect("forged caller remains present")
            .calls()
            .first()
            .expect("forged caller retains one source edge")
            .location();
        let error = rejection(fixture.document());
        assert_eq!(
            error.code(),
            "logic.rule_program.wrong_call_id",
            "{wrapper:?}"
        );
        assert_eq!(error.span(), call_location, "{wrapper:?}");
    }
}

// ---- forged document membership --------------------------------------------

#[test]
fn missing_member_without_surviving_items_rejects_at_original_source_start() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::SingleRule)
        .expect("sealed single-rule fixture");
    let original = fixture.document();
    let forged = original
        .clone()
        .forged_membership(|_order, items| items.clear());
    assert!(!forged.order().is_empty());
    assert_eq!(forged.items().count(), 0);
    let error = rejection(&forged);
    let LogicDiagnosticV1::RuleProgramInvalid { reason, span } = &error else {
        panic!("missing membership must have a typed rule-program rejection");
    };
    assert_eq!(*reason, RuleProgramInvalidReasonV1::MissingMember);
    assert_eq!(error.code(), "logic.rule_program.missing_member");
    assert_eq!(span.location().source_id, original.source_id());
    assert_eq!(span.location().span, SpanV1::point(0));
    assert_eq!(
        span.registry_identity(),
        original.catalog().source_registry_identity()
    );

    let empty = original.clone().forged_membership(|order, items| {
        order.clear();
        items.clear();
    });
    assert!(accepted(&empty).rules().is_empty());
}

#[test]
fn dropped_rule_item_is_missing_member() {
    let forged = forged_rule_membership_document_v2(ForgedRuleMembershipMutationV2::DroppedItem)
        .expect("forged membership document seals");
    let document = forged.document();
    // The forged document lost a Rule item the authored order still names; the
    // surviving member is the document-owned anchor of the rejection.
    let survivor = document
        .rules()
        .next()
        .expect("the surviving rule anchors the rejection");
    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::MissingMember,
        message_of(RuleProgramInvalidReasonV1::MissingMember),
        survivor.body().expression().location(),
    );
}

#[test]
fn rule_item_outside_the_document_order_is_membership_mismatch() {
    let forged = forged_rule_membership_document_v2(ForgedRuleMembershipMutationV2::OrphanItem)
        .expect("forged membership document seals");
    let document = forged.document();
    let orphan = document
        .rules()
        .find(|rule| !document.order().contains(&rule.item_id()))
        .expect("the forged document carries a Rule item outside its authored order");
    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::MembershipMismatch,
        message_of(RuleProgramInvalidReasonV1::MembershipMismatch),
        orphan.body().expression().location(),
    );
}

#[test]
fn rule_item_key_must_match_its_sealed_identity() {
    let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::SingleRule)
        .expect("reference rule document seals");
    let original = fixture.document();
    let rule = original.rules().next().expect("fixture retains one rule");
    let expected_location = rule.body().expression().location().clone();
    let forged = original.clone().forged_membership(|order, items| {
        let [old_id] = order.as_mut_slice() else {
            panic!("fixture must retain exactly one authored rule");
        };
        let item = items.remove(old_id).expect("fixture item exists");
        let alias = reference_absent_item_id_v2();
        *old_id = alias;
        assert!(items.insert(alias, item).is_none());
    });
    let error = rejection(&forged);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::MembershipMismatch,
        message_of(RuleProgramInvalidReasonV1::MembershipMismatch),
        &expected_location,
    );
}

#[test]
fn duplicate_authored_rule_identity_is_duplicate_member() {
    let forged =
        forged_rule_membership_document_v2(ForgedRuleMembershipMutationV2::DuplicateOrderId)
            .expect("forged membership document seals");
    let document = forged.document();
    let duplicated = document
        .rules()
        .find(|rule| {
            document
                .order()
                .iter()
                .filter(|id| **id == rule.item_id())
                .count()
                == 2
        })
        .expect("the forged document names one Rule identity twice");
    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::DuplicateMember,
        message_of(RuleProgramInvalidReasonV1::DuplicateMember),
        duplicated.body().expression().location(),
    );
}

// ---- forged rule facts ------------------------------------------------------

#[test]
fn call_target_outside_the_document_members_is_nonlocal_target() {
    let forged = forged_rule_fact_document_v2(
        ReferenceRuleFixtureV2::PositiveSelfRecursion,
        ForgedRuleFactMutationV2::NonLocalTarget,
    )
    .expect("forged rule-fact document seals");
    let document = forged.document();
    assert_eq!(
        document.rules().count(),
        1,
        "the forged document retains exactly one member, so `.next()` is that member"
    );
    let rule = document.rules().next().expect("forged member retained");
    let call = only_call(rule);
    assert!(
        !document
            .rules()
            .any(|member| member.predicate_id() == &predicate("ghost")),
        "the forged target is no member predicate"
    );
    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::NonLocalTarget,
        message_of(RuleProgramInvalidReasonV1::NonLocalTarget),
        call.location(),
    );
}

#[test]
fn transplanted_call_target_is_target_transplanted() {
    let forged = forged_rule_fact_document_v2(
        ReferenceRuleFixtureV2::MutualRecursionPair,
        ForgedRuleFactMutationV2::TargetTransplanted,
    )
    .expect("forged rule-fact document seals");
    let document = forged.document();
    let transplanted = document
        .rules()
        .flat_map(quanta_gqlang_v2_sema::ResolvedRuleV2::calls)
        .find(|call| {
            document
                .expression(call.call())
                .is_some_and(|fact| fact.resolved_call() != Some(call.target()))
        })
        .expect("the forged document carries a call whose target disagrees with its fact");
    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::TargetTransplanted,
        message_of(RuleProgramInvalidReasonV1::TargetTransplanted),
        transplanted.location(),
    );
}

#[test]
fn typed_call_shape_target_disagreement_is_target_transplanted() {
    let forged = forged_rule_call_shape_target_document_v2()
        .expect("the source call shape can be forged independently from its fact");
    let document = forged.document();
    let rule = document.rules().next().expect("the caller remains present");
    let call = only_call(rule);
    let fact = document
        .expression(call.call())
        .expect("the source call keeps its typed expression fact");
    let quanta_gqlang_v2_sema::TypedExprShapeV2::Call { target, .. } = fact.shape() else {
        panic!("the forged source call must retain a call shape");
    };
    assert_ne!(target, call.target());
    assert_eq!(fact.resolved_call(), Some(call.target()));

    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::TargetTransplanted,
        message_of(RuleProgramInvalidReasonV1::TargetTransplanted),
        call.location(),
    );
}

#[test]
fn flipped_call_polarity_is_polarity_flip() {
    let forged = forged_rule_fact_document_v2(
        ReferenceRuleFixtureV2::PositiveSelfRecursion,
        ForgedRuleFactMutationV2::PolarityFlipped,
    )
    .expect("forged rule-fact document seals");
    let document = forged.document();
    assert_eq!(
        document.rules().count(),
        1,
        "the forged document retains exactly one member, so `.next()` is that member"
    );
    let rule = document.rules().next().expect("forged member retained");
    let call = only_call(rule);
    assert_eq!(
        call.polarity(),
        quanta_gqlang_v2_sema::RuleCallPolarityV2::Negated
    );
    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::PolarityFlip,
        message_of(RuleProgramInvalidReasonV1::PolarityFlip),
        call.location(),
    );
}

#[test]
fn unknown_call_identity_is_wrong_call_id() {
    let forged = forged_rule_fact_document_v2(
        ReferenceRuleFixtureV2::PositiveSelfRecursion,
        ForgedRuleFactMutationV2::UnknownCallId,
    )
    .expect("forged rule-fact document seals");
    let document = forged.document();
    let unknown = document
        .rules()
        .flat_map(quanta_gqlang_v2_sema::ResolvedRuleV2::calls)
        .find(|call| document.expression(call.call()).is_none())
        .expect("the forged call node is absent from the document expression facts");
    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::WrongCallId,
        message_of(RuleProgramInvalidReasonV1::WrongCallId),
        unknown.location(),
    );
}

#[test]
fn foreign_call_location_is_foreign_location() {
    let forged = forged_rule_fact_document_v2(
        ReferenceRuleFixtureV2::PositiveSelfRecursion,
        ForgedRuleFactMutationV2::ForeignCallLocation,
    )
    .expect("forged rule-fact document seals");
    let document = forged.document();
    let foreign = document
        .rules()
        .flat_map(quanta_gqlang_v2_sema::ResolvedRuleV2::calls)
        .find(|call| {
            call.location().registry_identity() != document.catalog().source_registry_identity()
        })
        .expect("the forged call location belongs to another registry identity");
    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::ForeignLocation,
        message_of(RuleProgramInvalidReasonV1::ForeignLocation),
        foreign.location(),
    );
}

#[test]
fn same_registry_call_location_mismatch_is_wrong_call_id() {
    let forged = forged_rule_fact_document_v2(
        ReferenceRuleFixtureV2::PositiveSelfRecursion,
        ForgedRuleFactMutationV2::MismatchedCallLocation,
    )
    .expect("forged rule-fact document seals");
    let document = forged.document();
    let rule = document.rules().next().expect("one forged member");
    let call = only_call(rule);
    let fact = document
        .expression(call.call())
        .expect("the call retains its typed expression fact");
    assert_eq!(
        call.location().registry_identity(),
        fact.location().registry_identity()
    );
    assert_ne!(call.location(), fact.location());

    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::WrongCallId,
        message_of(RuleProgramInvalidReasonV1::WrongCallId),
        call.location(),
    );
}

#[test]
fn dropped_call_edge_is_wrong_call_id() {
    let forged = forged_rule_fact_document_v2(
        ReferenceRuleFixtureV2::PositiveSelfRecursion,
        ForgedRuleFactMutationV2::DroppedCall,
    )
    .expect("forged rule-fact document seals");
    let document = forged.document();
    assert_eq!(
        document.rules().count(),
        1,
        "the forged document retains exactly one member, so `.next()` is that member"
    );
    let rule = document.rules().next().expect("forged member retained");
    assert!(
        rule.calls().is_empty(),
        "the forged rule lost its resolved call edge"
    );
    let (_key, fact) = only_resolved_call_fact(document);
    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::WrongCallId,
        message_of(RuleProgramInvalidReasonV1::WrongCallId),
        fact.location(),
    );
}

#[test]
fn extra_call_edge_is_wrong_call_id() {
    let forged = forged_rule_fact_document_v2(
        ReferenceRuleFixtureV2::PositiveSelfRecursion,
        ForgedRuleFactMutationV2::ExtraCall,
    )
    .expect("forged rule-fact document seals");
    let document = forged.document();
    assert_eq!(
        document.rules().count(),
        1,
        "the forged document retains exactly one member, so `.next()` is that member"
    );
    let rule = document.rules().next().expect("forged member retained");
    assert_eq!(rule.calls().len(), 2, "the forged rule duplicates its edge");
    let duplicated = rule
        .calls()
        .last()
        .expect("the duplicated edge is retained");
    assert_eq!(
        rule.calls()
            .first()
            .expect("the original edge is retained")
            .call(),
        duplicated.call()
    );
    let error = rejection(document);
    assert_invalid(
        &error,
        RuleProgramInvalidReasonV1::WrongCallId,
        message_of(RuleProgramInvalidReasonV1::WrongCallId),
        duplicated.location(),
    );
}

/// The document's only expression fact that resolved to a source predicate.
fn only_resolved_call_fact(
    document: &SemanticDocumentV2,
) -> (
    quanta_gqlang_v2_types::SourceQualifiedNodeIdV2,
    &TypedExprV2,
) {
    let mut matches = document.expressions().filter(|(_key, expression)| {
        matches!(
            expression.resolved_call(),
            Some(ResolvedCallTargetV2::Predicate(_))
        )
    });
    let first = matches
        .next()
        .expect("the document retains one resolved source-predicate call fact");
    assert!(
        matches.next().is_none(),
        "the fixture retains exactly one resolved source-predicate call fact"
    );
    (*first.0, first.1)
}
