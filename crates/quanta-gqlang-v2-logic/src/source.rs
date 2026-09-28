//! Sole ingress from a sealed semantic document into the rule graph (SS-06B).
//!
//! Relational `EXISTS` / `NOT EXISTS` are owned by semantic analysis and lowered
//! by algebra; logic must not manufacture algebra nodes or reparse source
//! quantifier spelling, so the retired V1 existence producer is gone.
//!
//! [`RuleProgramContractV2::from_document`] enumerates the document's complete
//! Rule item set itself — it never accepts a caller-selected `Vec` or a separate
//! budget — validates every projected member, call identity, target, polarity,
//! source/registry identity, and checked location against the document's own
//! resolved facts, and fails closed with
//! [`LogicDiagnosticV1::RuleProgramInvalid`] before any graph is allocated.
//!
//! The SAME ingress then stratifies the validated member set exactly ONCE under
//! the document's own rule-graph budget and retains the resulting
//! [`StratificationProofV1`]; [`RuleProgramContractV2::stratification`] is the
//! one read surface for it. Downstream consumers (planning, the pipeline
//! assembly) read that retained proof, so one document owns one rule-graph
//! charge and one proof instead of re-running the stratification per stage with
//! a fresh full allowance each time.

use std::collections::{BTreeMap, BTreeSet};

use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, DownstreamBudgetReceiptV2, ResolvedRuleCallV2, ResolvedRuleV2,
    RuleCallPolarityV2, SemanticDocumentV2, SemanticItemIdV2, SemanticItemV2,
};
use quanta_gqlang_v2_types::{
    ResolvedCallTargetV2, ResolvedPredicateIdV2, SourceQualifiedNodeIdV2,
};

use crate::diagnostic::{LogicDiagnosticV1, RuleProgramInvalidReasonV1};
use crate::recursion::{
    stratify, BodyLiteralV1, BodyPolarityV1, RecursiveProgramV1, RuleDefV1, StratificationProofV1,
};

/// The validated, stratified rule program of one sealed semantic document.
///
/// Construction is the fallible owner operation [`Self::from_document`]; every
/// member and every call edge retained here already agreed with the document's
/// resolved facts, and the retained [`StratificationProofV1`] is the one
/// admission proof of the whole member set under the document's own rule-graph
/// budget. The budget is the document's immutable receipt and is never supplied
/// by a caller.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleProgramContractV2 {
    rules: Vec<ResolvedRuleV2>,
    budget: DownstreamBudgetReceiptV2,
    stratification: StratificationProofV1,
}

impl RuleProgramContractV2 {
    /// Sole ingress: enumerate the complete document Rule item set, validate
    /// every projected fact, then stratify the validated member set once under
    /// the document's own rule-graph budget.
    ///
    /// The returned contract retains that one proof; no consumer recomputes it,
    /// so `max_rule_graph_nodes`, `max_rule_graph_edges`, and `max_rule_steps`
    /// are charged exactly once per document.
    ///
    /// # Errors
    ///
    /// Returns [`LogicDiagnosticV1::RuleProgramInvalid`] naming the exact
    /// disagreement when the document's Rule membership, a call identity, a call
    /// target, a call polarity, a source/registry identity, or a checked location
    /// is not a faithful, self-consistent document fact, and otherwise relays the
    /// [`stratify`] rejection verbatim ([`LogicDiagnosticV1::NegationCycle`],
    /// the fixed recursive-aggregate policy, a non-local target, or an exhausted
    /// rule-graph dimension) for a validated member set that is not stratifiable
    /// under this document's own receipt.
    pub fn from_document(document: &SemanticDocumentV2) -> Result<Self, LogicDiagnosticV1> {
        // The document's complete Rule item set, by canonical identity. This is
        // the only membership authority; no caller enumerates it for us.
        let mut members: BTreeMap<SemanticItemIdV2, &ResolvedRuleV2> = BTreeMap::new();
        for (id, item) in document.items() {
            if let SemanticItemV2::Rule(rule) = item {
                if *id != rule.item_id() {
                    return Err(invalid(
                        RuleProgramInvalidReasonV1::MembershipMismatch,
                        rule_location(rule),
                    ));
                }
                members.insert(*id, rule);
            }
        }

        // Cross-check that set against the document's own authored item order: a
        // rule member is exactly one authored item identity that resolves to one
        // Rule item, and nothing else may claim that identity.
        let mut authored_members = BTreeSet::new();
        let mut ordered = Vec::new();
        for id in document.order() {
            if let Some(item) = document.item(*id) {
                if let SemanticItemV2::Rule(rule) = item {
                    if !authored_members.insert(*id) {
                        return Err(invalid(
                            RuleProgramInvalidReasonV1::DuplicateMember,
                            rule_location(rule),
                        ));
                    }
                    ordered.push(rule);
                }
            } else {
                // The document's authored item set names an identity its item map
                // cannot produce, so the rule program cannot prove that it
                // enumerated every member. The document retains its original
                // checked source anchor even if no item survives.
                return Err(invalid(
                    RuleProgramInvalidReasonV1::MissingMember,
                    membership_anchor(&members, document),
                ));
            }
        }
        for (id, rule) in &members {
            if !authored_members.contains(id) {
                // A Rule item outside the document's own item order is an extra
                // membership: the rule set disagrees with the document item set.
                return Err(invalid(
                    RuleProgramInvalidReasonV1::MembershipMismatch,
                    rule_location(rule),
                ));
            }
        }

        // Validate every member and every call edge before any graph allocation.
        let predicates: BTreeSet<&ResolvedPredicateIdV2> =
            members.values().map(|rule| rule.predicate_id()).collect();
        for rule in members.values() {
            validate_rule(document, &predicates, rule)?;
        }
        // The authored sequence borrows the document directly. Release the
        // validation indexes before allocating the projected rule program.
        drop(predicates);
        drop(members);
        drop(authored_members);

        // The ONE stratification of this document's member set, under this
        // document's own rule-graph budget. Charging happens here and nowhere
        // else: the proof below is the single allowance spend for this document.
        let budget = document.budget_receipt();
        let program = RecursiveProgramV1::from_rules(project_rules(ordered.iter().copied()));
        let stratification = stratify(&program, &budget)?;
        drop(program);

        // The output owns its rules only after admission. Rejected documents
        // never deep-clone the typed heads, bodies, or retained call vectors.
        let rules = ordered.into_iter().cloned().collect();
        Ok(Self {
            rules,
            budget,
            stratification,
        })
    }

    /// Validated members in document order.
    #[must_use]
    pub fn rules(&self) -> &[ResolvedRuleV2] {
        &self.rules
    }

    /// Immutable rule-graph budget read from the document's receipt.
    #[must_use]
    pub const fn budget(&self) -> DownstreamBudgetReceiptV2 {
        self.budget
    }

    /// The admitted stratification proof of this exact member set under the
    /// document's own rule-graph budget, computed once at ingress.
    ///
    /// Planning and pipeline assembly read this proof instead of re-deriving it,
    /// so the rule-graph charge is taken exactly once per document and every
    /// consumer observes the same admission decision.
    #[must_use]
    pub const fn stratification(&self) -> &StratificationProofV1 {
        &self.stratification
    }

    /// Project into lane W3-A's program carrier (frozen C->A junction).
    ///
    /// Builtin/host calls are typed body facts and never become local graph
    /// nodes, so only resolved source-predicate calls become
    /// [`BodyLiteralV1`] edges; every projected target was already proven to be a
    /// member predicate of this document.
    ///
    /// This is the exact program [`Self::stratification`] admitted: both are
    /// projections of the same validated member set.
    #[must_use]
    pub fn to_program(&self) -> Vec<RuleDefV1> {
        project_rules(self.rules.iter())
    }
}

/// Project one validated member set into lane W3-A's program carrier.
///
/// The sole projection: [`RuleProgramContractV2::to_program`] exposes it and the
/// ingress stratifies it, so the admitted proof and the projected program can
/// never describe different member sets.
fn project_rules<'a>(rules: impl Iterator<Item = &'a ResolvedRuleV2>) -> Vec<RuleDefV1> {
    rules
        .map(|rule| {
            let body = rule
                .calls()
                .iter()
                .filter_map(|call| {
                    let ResolvedCallTargetV2::Predicate(predicate) = call.target() else {
                        return None;
                    };
                    Some(BodyLiteralV1::new(
                        predicate.clone(),
                        body_polarity(call.polarity()),
                        call.location().clone(),
                    ))
                })
                .collect();
            RuleDefV1::new(rule.predicate_id().clone(), body, rule_location(rule))
        })
        .collect()
}

/// Validate one member rule and every resolved call it retained.
fn validate_rule(
    document: &SemanticDocumentV2,
    predicates: &BTreeSet<&ResolvedPredicateIdV2>,
    rule: &ResolvedRuleV2,
) -> Result<(), LogicDiagnosticV1> {
    let source_id = document.source_id();
    let registry = document.catalog().source_registry_identity();
    if rule.item_id().source_id() != source_id {
        return Err(invalid(
            RuleProgramInvalidReasonV1::ForeignLocation,
            rule_location(rule),
        ));
    }
    for location in [
        rule.head_term().location(),
        rule.body().expression().location(),
    ] {
        if location.location().source_id != source_id || location.registry_identity() != registry {
            return Err(invalid(
                RuleProgramInvalidReasonV1::ForeignLocation,
                location.clone(),
            ));
        }
    }

    // Independently re-derive the call edges the rule body proves, so a forged
    // `calls()` vector cannot smuggle in an unknown call identity, a transplanted
    // target, a flipped polarity, or an edge the body never resolved.
    let derived = derive_body_calls(document, rule)?;

    let mut seen: BTreeSet<SourceQualifiedNodeIdV2> = BTreeSet::new();
    for call in rule.calls() {
        let location = call.location();
        if call.call().source_id != source_id
            || location.location().source_id != source_id
            || location.registry_identity() != registry
        {
            return Err(invalid(
                RuleProgramInvalidReasonV1::ForeignLocation,
                location.clone(),
            ));
        }
        if !seen.insert(call.call()) {
            return Err(invalid(
                RuleProgramInvalidReasonV1::WrongCallId,
                location.clone(),
            ));
        }
        let Some(fact) = document.expression(call.call()) else {
            return Err(invalid(
                RuleProgramInvalidReasonV1::WrongCallId,
                location.clone(),
            ));
        };
        // The target must first be a predicate this document defines; only then
        // can a disagreement with the retained expression fact be classified as a
        // transplant. Checking the fact first would mask every nonlocal target
        // (a ghost target can never agree with a real document fact), collapsing
        // the two distinct rejection families into one.
        let ResolvedCallTargetV2::Predicate(predicate) = call.target() else {
            // Builtin/host calls remain typed expression facts, never members
            // of the rule's source-dependency edge list.
            return Err(invalid(
                RuleProgramInvalidReasonV1::WrongCallId,
                location.clone(),
            ));
        };
        if !predicates.contains(&predicate) {
            return Err(invalid(
                RuleProgramInvalidReasonV1::NonLocalTarget,
                location.clone(),
            ));
        }
        if fact.resolved_call() != Some(call.target()) {
            return Err(invalid(
                RuleProgramInvalidReasonV1::TargetTransplanted,
                location.clone(),
            ));
        }
        let Some(derived_call) = derived.get(&call.call()) else {
            return Err(invalid(
                RuleProgramInvalidReasonV1::WrongCallId,
                location.clone(),
            ));
        };
        if derived_call.target() != call.target() {
            return Err(invalid(
                RuleProgramInvalidReasonV1::TargetTransplanted,
                location.clone(),
            ));
        }
        if derived_call.polarity() != call.polarity() {
            return Err(invalid(
                RuleProgramInvalidReasonV1::PolarityFlip,
                location.clone(),
            ));
        }
        if derived_call.location() != location {
            return Err(invalid(
                RuleProgramInvalidReasonV1::WrongCallId,
                location.clone(),
            ));
        }
    }

    // Every source-predicate call the body proves must be a retained edge.
    for (key, call) in &derived {
        if !seen.contains(key) {
            return Err(invalid(
                RuleProgramInvalidReasonV1::WrongCallId,
                call.location().clone(),
            ));
        }
    }
    Ok(())
}

/// Re-derive edges through the semantic owner's exact typed-shape classifier.
/// Any unproven scalar occurrence or missing body node fails closed before graph
/// construction.
fn derive_body_calls(
    document: &SemanticDocumentV2,
    rule: &ResolvedRuleV2,
) -> Result<BTreeMap<SourceQualifiedNodeIdV2, ResolvedRuleCallV2>, LogicDiagnosticV1> {
    document
        .classify_rule_body_calls(rule.body().expression().key())
        .map(|calls| calls.into_iter().map(|call| (call.call(), call)).collect())
        .map_err(|error| {
            invalid(
                RuleProgramInvalidReasonV1::WrongCallId,
                error
                    .location()
                    .cloned()
                    .unwrap_or_else(|| rule_location(rule)),
            )
        })
}

/// The location a rule-program rejection points at when the rule itself is the
/// offending fact.
fn rule_location(rule: &ResolvedRuleV2) -> CheckedLocatedSpanV2 {
    rule.body().expression().location().clone()
}

/// A document-owned anchor for a membership rejection that has no rule fact of
/// its own. A surviving Rule member is preferred; then any located item; then
/// the document's retained source anchor from its original registry.
fn membership_anchor(
    members: &BTreeMap<SemanticItemIdV2, &ResolvedRuleV2>,
    document: &SemanticDocumentV2,
) -> CheckedLocatedSpanV2 {
    if let Some(rule) = members.values().next() {
        return rule_location(rule);
    }
    document
        .items()
        .find_map(|(_id, item)| item_location(item).cloned())
        .unwrap_or_else(|| document.source_anchor().clone())
}

/// The checked location of an item kind that retains one.
fn item_location(item: &SemanticItemV2) -> Option<&CheckedLocatedSpanV2> {
    match item {
        SemanticItemV2::Relation(item) => Some(item.location()),
        SemanticItemV2::Graph(item) => Some(item.location()),
        SemanticItemV2::Composition(item) => Some(item.location()),
        SemanticItemV2::Meta(item) => Some(item.location()),
        SemanticItemV2::Rule(_)
        | SemanticItemV2::Check(_)
        | SemanticItemV2::Presentation(_)
        | SemanticItemV2::Catalog(_)
        | SemanticItemV2::Declaration(_) => None,
    }
}

/// Project the document's resolved call polarity into the graph carrier.
fn body_polarity(polarity: RuleCallPolarityV2) -> BodyPolarityV1 {
    match polarity {
        RuleCallPolarityV2::Positive => BodyPolarityV1::Positive,
        RuleCallPolarityV2::Negated => BodyPolarityV1::Negated,
        RuleCallPolarityV2::Aggregated => BodyPolarityV1::Aggregated,
    }
}

/// One fail-closed rule-program rejection at a document-owned location.
fn invalid(reason: RuleProgramInvalidReasonV1, span: CheckedLocatedSpanV2) -> LogicDiagnosticV1 {
    LogicDiagnosticV1::RuleProgramInvalid { reason, span }
}
