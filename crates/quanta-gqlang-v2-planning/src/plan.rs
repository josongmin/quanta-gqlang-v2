//! The host-agnostic public plan and its sealed-input producer (ticket 07).
//!
//! [`SemanticPlanV2`] is the ONE assembled planning output. It is keyed off the
//! sealed [`SemanticPlanInputV2`] carriers only: the relational arm is ticket 05's
//! checked lowering product, the graph/rule arms are ticket 06's validated logic
//! contracts, the CHECK/presentation/composition arms are this crate's validated
//! wrappers, and the meta arm is Sema's resolved directive. A caller can no
//! longer select the truth lane, result kind, effect, or cost class: every field
//! is derived from a verified upstream contract (`D6`/`D24`).
//!
//! The V1 caller-selected surface (`PlanRequestV1`, `PublicPlanV1`,
//! `produce_plan`, the recursive `classify_node` walk) is DELETED with no alias.
//!
//! # Entry points
//!
//! - [`plan`] requires a plan and fails closed with a typed
//!   [`PlanningDiagnosticV1`], including the checked non-plan outcomes
//!   ([`PlanningDiagnosticV1::NoPlan`]).
//! - [`plan_outcome`] distinguishes the typed, non-error non-plan outcome
//!   ([`PlanOutcomeV2::NoPlan`]) from a rejection, so absence is never a skipped
//!   stage.
//!
//! Both run the SAME four phases, in this fixed order:
//!
//! 1. **Identity validation first.** Every carrier identity must belong to the
//!    supplied document, or the input fails closed with
//!    [`PlanningDiagnosticV1::InputDocumentMismatch`] (`Absent`, `WrongKind`, or
//!    `IdentityMismatch`) anchored on the carrier's own checked location.
//! 2. **One iterative, deterministic, budgeted traversal** over the visited
//!    sealed inputs ([`walk`]): an explicit frame stack (no recursion), stable
//!    root-first / left-before-right order, charging the document's sealed
//!    `max_semantic_steps` exactly once per visited node BEFORE the visit. The
//!    exact limit succeeds; limit + 1 rejects at the refused node's checked
//!    location with [`PlanningDiagnosticV1::SemanticStepBudgetExceeded`]. There
//!    is no planner-local budget and no default.
//! 3. **Per-arm classification** from the sealed carriers alone: capabilities,
//!    folded expression effects, the result contract, effect class, evidence
//!    requirement, the fixed semantic-cost schema, the logical explain note, and
//!    the retained family policy.
//! 4. **Typed non-plan outcomes, never silent skips.** A
//!    `REQUIRE CERTIFIED` directive whose sealed kind is `USE PACK` is
//!    [`NoPlanReasonV2::MetaActivation`]; a document-wide rule program with no
//!    member is [`NoPlanReasonV2::EmptyRuleProgram`].
//!
//! # Per-arm classification
//!
//! | Sealed arm | Result | Effect | Evidence | Policy |
//! |---|---|---|---|---|
//! | relational | `table` | `read_only` | none | visited algebra nodes + folded effects |
//! | graph | `path_witness` | `proof_evidence_only` | witness payload | exact witness provenance |
//! | CHECK | `alert` | `read_only` (`export_only` when `emit=sarif`) | source location (`proof_tree` when `confirm=require`) | exact mode/confirm/emit |
//! | EXPLAIN | `proof_explain` | `read_only` | proof tree | exact sealed payload |
//! | SHOW | `table` | `read_only` | none | exact sealed payload |
//! | DESCRIBE | `alert` | `read_only` | source location | exact sealed payload |
//! | SAMPLE | `table` (bounded) | `read_only` | none | exact sealed payload |
//! | COMPARE | `path_witness` | `read_only` | witness payload | exact sealed payload |
//! | EXPORT | `table` | `export_only` | none | exact sealed payload |
//! | rule program | `table` | `read_only` | none | members + stratified proof |
//! | `REQUIRE CERTIFIED` | `proof_explain` | `proof_evidence_only` | proof tree | certified directive |
//! | composition | its body's plan, rooted on the composition item | its body's plan | its body's plan | the composition identity + its body's policy |
//!
//! The composition arm dispatches to the body's existing classification through
//! the SAME [`classify`] function: it is not a second classifier and it never
//! synthesizes a plan per `LET` producer. The classification runs under the
//! composition's OWN root ([`PlanRootV2::Composition`]), so the assembled plan is
//! attributed to the composition item and retains
//! [`crate::CompositionPolicyV2`] — root identity, ordered producers, exact `WITH`
//! binding, body identity, and the body's own policy — around the body policy
//! that same call derived.
//!
//! # The nine retained fields
//!
//! `authority_lane`, `evaluation_determinism`, `evidence`, `effect`, `result`,
//! `capabilities`, `semantic_cost`, `explain`, and `policy`. Every one is private
//! and read through one accessor, so the plan cannot be mutated after assembly.

use std::collections::BTreeSet;

use quanta_gqlang_v2_algebra::{AlgebraFamilyV2, CheckedAlgebraNodeV2};
use quanta_gqlang_v2_logic::{GraphQueryContractV2, RelationShapeV1, RuleProgramContractV2};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, ConfirmModeV1, DeclaredShowSourceV2, EmitModeV1, ExportFormatV1,
    ResolvedCheckV2, ResolvedMetaKindV2, ResolvedMetaV2, ResolvedPresentationKindV2,
    ResolvedRenameCommandV3, ResolvedRuleV2, SemanticDocumentV2, SemanticItemIdV2, SemanticItemV2,
};
use quanta_gqlang_v2_types::{
    CardinalityV1, DeterminismV1, ExpressionEffectsV2, ResolvedPredicateIdV2,
};

use crate::capability::{capability_for_family, CapabilityV2};
use crate::check::CheckContractV2;
use crate::composition::{CheckedCompositionPlanInputV2, CompositionBodyPlanInputV2};
use crate::cost::{
    CardinalityClassV2, OrderingContractV2, RecursionRiskClassV2, SemanticCostV2,
    WitnessExplosionRiskClassV2,
};
use crate::diagnostic::{InputDocumentMismatchReasonV2, PlanningDiagnosticV1};
use crate::effect::{
    AuthorityLaneV2, EffectClassV2, EvaluationDeterminismV2, EvidenceRequirementV2,
};
use crate::explain::ExplainNoteV2;
use crate::item::{input_item_id, NoPlanReasonV2, SemanticPlanInputV2};
use crate::policy::{EffectSummaryV2, GraphWitnessRetentionV2, PlanPolicyV2, RuleProgramPolicyV2};
use crate::presentation::PresentationContractV2;
use crate::result_kind::{
    MultiplicityOrderContractV2, RenderRequirementV2, ResultContractV2, ResultKindV2,
    WitnessSelectionPolicyV2,
};

mod scalar_request;
pub use scalar_request::{
    plan_scalar_source_project_with_request_owner_v3, AdmittedScalarPlanV3,
    ScalarPlanningRequestErrorV3,
};

/// The checked outcome of planning one sealed input.
///
/// [`Self::NoPlan`] is a TYPED, non-error outcome: the input is well-formed, it
/// simply does not lower to a servable host-agnostic plan. [`plan`] surfaces it
/// as a [`PlanningDiagnosticV1::NoPlan`] rejection so a plan-required caller can
/// never mistake it for success; [`plan_outcome`] surfaces it verbatim so an
/// assembly stage can store the exact reason.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanOutcomeV2 {
    /// The input plans.
    Plan(Box<SemanticPlanV2>),
    /// The input has a checked, non-error non-plan outcome.
    NoPlan(NoPlanReasonV2),
}

/// The assembled, host-agnostic semantic plan.
///
/// Every field is private and derived from sealed carriers; the accessors below
/// are the whole read surface.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemanticPlanV2 {
    authority_lane: AuthorityLaneV2,
    evaluation_determinism: EvaluationDeterminismV2,
    evidence: EvidenceRequirementV2,
    effect: EffectClassV2,
    result: ResultContractV2,
    capabilities: BTreeSet<CapabilityV2>,
    semantic_cost: SemanticCostV2,
    explain: ExplainNoteV2,
    policy: PlanPolicyV2,
}

impl SemanticPlanV2 {
    /// Assemble one plan from its nine fully derived fields.
    fn from_parts(parts: PlanPartsV2) -> Self {
        Self {
            authority_lane: parts.authority_lane,
            evaluation_determinism: parts.evaluation_determinism,
            evidence: parts.evidence,
            effect: parts.effect,
            result: parts.result,
            capabilities: parts.capabilities,
            semantic_cost: parts.semantic_cost,
            explain: parts.explain,
            policy: parts.policy,
        }
    }

    /// WHO checked the result (ordinary frontend semantics or the certified
    /// requirement the document's exact directive validated).
    #[must_use]
    pub const fn authority_lane(&self) -> AuthorityLaneV2 {
        self.authority_lane
    }

    /// HOW deterministic the retained expression effects make evaluation.
    #[must_use]
    pub const fn evaluation_determinism(&self) -> EvaluationDeterminismV2 {
        self.evaluation_determinism
    }

    /// The evidence the result must carry for its authority lane.
    #[must_use]
    pub const fn evidence(&self) -> EvidenceRequirementV2 {
        self.evidence
    }

    /// The permitted effect class.
    #[must_use]
    pub const fn effect(&self) -> EffectClassV2 {
        self.effect
    }

    /// The assembled result contract (kind, render payload, witness selection,
    /// multiplicity/order).
    #[must_use]
    pub const fn result(&self) -> &ResultContractV2 {
        &self.result
    }

    /// The host-agnostic capability requirements of the visited sealed inputs.
    #[must_use]
    pub const fn capabilities(&self) -> &BTreeSet<CapabilityV2> {
        &self.capabilities
    }

    /// The FIXED semantic-cost schema (`D32`).
    #[must_use]
    pub const fn semantic_cost(&self) -> &SemanticCostV2 {
        &self.semantic_cost
    }

    /// The logical explain / provenance note.
    #[must_use]
    pub const fn explain(&self) -> &ExplainNoteV2 {
        &self.explain
    }

    /// The retained family policy: the exact sealed payload planning must not
    /// lose (graph witness provenance, CHECK modes, presentation payload, rule
    /// members, certified directive, or the body's own policy).
    #[must_use]
    pub const fn policy(&self) -> &PlanPolicyV2 {
        &self.policy
    }
}

/// Produce the host-agnostic [`SemanticPlanV2`] of one sealed input, or reject.
///
/// A checked non-plan input ([`PlanOutcomeV2::NoPlan`]) is a rejection here,
/// because a plan-required caller must never receive an absence it can mistake
/// for a plan.
///
/// # Errors
///
/// Returns [`PlanningDiagnosticV1`] for an input that does not belong to the
/// document, for a refused semantic-step charge, and for a typed non-plan
/// outcome. An unstratifiable document rule program is rejected earlier, at the
/// program's sole ingress (`RuleProgramContractV2::from_document`), so planning
/// never sees one and never relays [`PlanningDiagnosticV1::IllegalRuleProgram`].
pub fn plan(
    input: &SemanticPlanInputV2,
    document: &SemanticDocumentV2,
) -> Result<SemanticPlanV2, PlanningDiagnosticV1> {
    match plan_outcome(input, document)? {
        PlanOutcomeV2::Plan(plan) => Ok(*plan),
        PlanOutcomeV2::NoPlan(reason) => Err(PlanningDiagnosticV1::NoPlan {
            reason,
            span: attribution_anchor(input, document),
        }),
    }
}

/// Produce the checked planning outcome of one sealed input.
///
/// # Errors
///
/// Returns [`PlanningDiagnosticV1`] for an input that does not belong to the
/// document and for a refused semantic-step charge. Rule-program legality is
/// owned by the program's sole ingress, whose retained stratification proof this
/// entry reads.
pub fn plan_outcome(
    input: &SemanticPlanInputV2,
    document: &SemanticDocumentV2,
) -> Result<PlanOutcomeV2, PlanningDiagnosticV1> {
    PlanningSessionV2::new(document).plan_outcome(input)
}

/// One document-scoped planning transaction.
///
/// All plan-required items in a document share this counter. Constructing a
/// session borrows the sealed document, so a caller cannot transplant the
/// remaining allowance to another document or reset it between items without
/// starting a distinct planning transaction.
pub struct PlanningSessionV2<'a> {
    document: &'a SemanticDocumentV2,
    charge: ChargeV2,
}

impl<'a> PlanningSessionV2<'a> {
    /// Start the sole planning traversal session for `document`.
    #[must_use]
    pub fn new(document: &'a SemanticDocumentV2) -> Self {
        Self {
            document,
            charge: ChargeV2::new(document.budget_receipt().max_semantic_steps()),
        }
    }

    /// Plan one sealed input while charging the shared document allowance.
    ///
    /// # Errors
    ///
    /// Returns the same typed identity, policy, and budget diagnostics as
    /// [`plan_outcome`]. A refused charge does not reset the session.
    pub fn plan_outcome(
        &mut self,
        input: &SemanticPlanInputV2,
    ) -> Result<PlanOutcomeV2, PlanningDiagnosticV1> {
        let document = self.document;
        // 1. Identity first: an input that does not belong to this document is
        //    rejected before any classification or budget work.
        validate_identity(input, document)?;

        // 2. Typed non-plan outcomes. They consume no semantic step: nothing is
        //    walked, and nothing is silently skipped.
        if let Some(reason) = non_plan_reason(input) {
            return Ok(PlanOutcomeV2::NoPlan(reason));
        }

        // 3. One budgeted traversal over the visited sealed inputs.
        let walk = walk(input, &mut self.charge)?;

        // 4. Per-arm classification from the sealed carriers alone. A composition
        //    input is classified under its own root, so the plan retains the
        //    composition identity instead of the body's.
        let parts = classify(ArmV2::of(input), document, &walk, PlanRootV2::of(input))?;
        Ok(PlanOutcomeV2::Plan(Box::new(SemanticPlanV2::from_parts(
            parts,
        ))))
    }
}

/// SHOW-only planner over the parser-issued authored order. It reads the
/// declaration's closed-authority receipt and does not construct the generic
/// semantic map, walk stack, or boxed plan. The host issuer and output storage
/// remain the responsibility of the consuming pipeline owner.
pub struct ShowPlanningSessionV2<'a> {
    declared: &'a DeclaredShowSourceV2,
    next_index: usize,
    charge: ChargeV2,
}

/// Failures of the selected SHOW planning stage. These variants contain no
/// dynamically formatted diagnostic payload before the plan output debit.
#[derive(Debug)]
pub enum ShowPlanningErrorV2 {
    /// The checked source declaration cannot produce the requested SHOW item.
    Semantic(&'static str),
    /// A checked planning rule, including the exact semantic-step cap, refused.
    Planning(PlanningDiagnosticV1),
    /// The caller attempted to skip or repeat an authored item.
    NonSequentialItem,
}

impl<'a> ShowPlanningSessionV2<'a> {
    #[must_use]
    pub fn new(declared: &'a DeclaredShowSourceV2) -> Self {
        Self {
            declared,
            next_index: 0,
            charge: ChargeV2::new(declared.budget_receipt_v2().max_semantic_steps()),
        }
    }

    /// Plan exactly the next authored SHOW. The semantic item and all nine
    /// plan fields are derived from the checked declaration; this path has no
    /// heap birth before its caller's admitted output vector.
    pub fn plan_next_v2(&mut self, index: usize) -> Result<SemanticPlanV2, ShowPlanningErrorV2> {
        if index != self.next_index {
            return Err(ShowPlanningErrorV2::NonSequentialItem);
        }
        let resolved = self
            .declared
            .resolve_show_item_v2(index)
            .map_err(ShowPlanningErrorV2::Semantic)?;
        let contract = PresentationContractV2::from_resolved(&resolved)
            .map_err(ShowPlanningErrorV2::Planning)?;
        self.charge
            .charge(contract.location())
            .map_err(ShowPlanningErrorV2::Planning)?;
        let effects = ExpressionEffectsV2::pure_scalar();
        let walk = WalkV2 {
            capabilities: BTreeSet::new(),
            families: Vec::new(),
            cardinality_class: presentation_class(contract.kind()),
            effects: EffectSummaryV2::new(
                effects.purity,
                effects.determinism,
                effects.monotonicity,
                effects.aggregation,
            ),
            algebra_nodes: 0,
        };
        let facts = presentation_facts(&contract);
        let location = contract.location().clone();
        let policy = facts.policy.clone();
        let parts = assemble_plan_parts_v2(facts, &walk, BTreeSet::new(), location, policy);
        self.next_index += 1;
        Ok(SemanticPlanV2::from_parts(parts))
    }
}

/// The checked non-plan reason of an input, if it has one.
///
/// Only two sealed shapes have no plan at this layer: a meta directive whose
/// RESOLVED kind is `USE PACK` (an effect-style pack activation with no
/// relational plan), and a document-wide rule program with no member. Both are
/// typed outcomes, not errors and not skips. Every other directive, and every
/// other arm, plans normally.
fn non_plan_reason(input: &SemanticPlanInputV2) -> Option<NoPlanReasonV2> {
    match input {
        SemanticPlanInputV2::MetaRequireCertified(meta) => match meta.kind() {
            ResolvedMetaKindV2::UsePack { .. } => Some(NoPlanReasonV2::MetaActivation),
            ResolvedMetaKindV2::RequireCertified => None,
        },
        SemanticPlanInputV2::Rule(contract) if contract.rules().is_empty() => {
            Some(NoPlanReasonV2::EmptyRuleProgram)
        }
        SemanticPlanInputV2::Relational(_)
        | SemanticPlanInputV2::Command(_)
        | SemanticPlanInputV2::Graph(_)
        | SemanticPlanInputV2::Check(_)
        | SemanticPlanInputV2::Presentation(_)
        | SemanticPlanInputV2::Rule(_)
        | SemanticPlanInputV2::Composition(_) => None,
    }
}

// ---------------------------------------------------------------------------
// Phase 1: identity validation
// ---------------------------------------------------------------------------

/// Require every carrier identity to belong to `document`.
///
/// `Absent` means the document holds no item with the carrier's identity;
/// `WrongKind` means it holds that identity as another semantic item kind;
/// `IdentityMismatch` means it holds a same-kind item whose sealed facts differ
/// from the carrier's (a transplanted or forged identity). Every rejection is
/// anchored on the carrier's own checked location where it retains one, else on
/// the document's own anchor.
fn validate_identity(
    input: &SemanticPlanInputV2,
    document: &SemanticDocumentV2,
) -> Result<(), PlanningDiagnosticV1> {
    // The canonical identity the arm plans for; the document-wide rule program
    // retains none, so it validates its members instead (see below).
    let anchor = attribution_anchor(input, document);
    let resolved = match input_item_id(input) {
        Some(id) => Some(document.item(id).ok_or_else(|| absent(&anchor))?),
        None => None,
    };
    match input {
        SemanticPlanInputV2::Command(command) => {
            let Some(SemanticItemV2::Command(resolved)) = resolved else {
                return Err(wrong_kind(&anchor));
            };
            if resolved != command {
                return Err(identity_mismatch(&anchor));
            }
            Ok(())
        }
        SemanticPlanInputV2::Relational(item) => {
            let Some(SemanticItemV2::Relation(relation)) = resolved else {
                return Err(wrong_kind(&anchor));
            };
            // The carrier must be the sealed lowering of THIS document item and
            // retain that item's exact output contract. The root stage KEY is
            // deliberately NOT compared: canonical lowering legitimately
            // re-anchors the root (a pure relational EXISTS conjunct is folded
            // into a root Semi/Anti join, asserted by
            // algebra::pure_exists_conjunct_lowers_to_semijoin_with_no_residual_filter),
            // so the promoted root carries the EXISTS stage key while the sealed
            // relation keeps the authored filter key. Comparing keys would make
            // every EXISTS/AntiJoin relation unplannable.
            if relation.item_id() != item.root().item_id() || item.output() != relation.output() {
                return Err(identity_mismatch(&anchor));
            }
            Ok(())
        }
        SemanticPlanInputV2::Graph(contract) => {
            let Some(SemanticItemV2::Graph(resolved)) = resolved else {
                return Err(wrong_kind(&anchor));
            };
            // Re-seal the document's own resolved FIND with logic: the carrier is
            // identical only when every legality/boundedness fact it retained is
            // the document's. Re-sealing reads sealed facts only and charges no
            // semantic step.
            match GraphQueryContractV2::from_resolved(resolved) {
                Ok(sealed) if sealed == *contract => Ok(()),
                Ok(_) => Err(identity_mismatch(&anchor)),
                Err(_reseal_error) => Err(identity_mismatch(&anchor)),
            }
        }
        SemanticPlanInputV2::Check(contract) => {
            let Some(SemanticItemV2::Check(resolved)) = resolved else {
                return Err(wrong_kind(&anchor));
            };
            if resolved.item_id() != contract.item_id()
                || resolved.location() != contract.location()
                || resolved.mode() != contract.mode()
                || resolved.confirm() != contract.confirm()
                || resolved.emit() != contract.emit()
                || resolved.rule() != contract.rule()
                || resolved.scope() != contract.scope()
                || resolved.pattern() != contract.pattern()
                || !nested_graph_matches(resolved, contract)
            {
                return Err(identity_mismatch(&anchor));
            }
            Ok(())
        }
        SemanticPlanInputV2::Presentation(contract) => {
            let Some(SemanticItemV2::Presentation(resolved)) = resolved else {
                return Err(wrong_kind(&anchor));
            };
            if resolved.item_id() != contract.item_id()
                || resolved.location() != contract.location()
                || resolved.kind() != contract.kind()
            {
                return Err(identity_mismatch(&anchor));
            }
            Ok(())
        }
        SemanticPlanInputV2::Rule(contract) => validate_rule_membership(contract, document),
        SemanticPlanInputV2::MetaRequireCertified(meta) => {
            let Some(SemanticItemV2::Meta(resolved)) = resolved else {
                return Err(wrong_kind(&anchor));
            };
            if resolved != meta {
                return Err(identity_mismatch(&anchor));
            }
            Ok(())
        }
        SemanticPlanInputV2::Composition(composition) => {
            let Some(SemanticItemV2::Composition(resolved)) = resolved else {
                return Err(wrong_kind(&anchor));
            };
            // The sealed composition retains the ordered producers, the exact
            // WITH binding, and the body identity, so exact equality proves all
            // three belong to this document.
            if resolved != composition.sealed_composition() {
                return Err(identity_mismatch(&anchor));
            }
            Ok(())
        }
    }
}

/// Require the document-wide rule program to be exactly the document's rules.
///
/// The program has no carrier location of its own, so every rejection is
/// anchored on the member it is attributed to (or, for the document-wide
/// membership itself, on the document's anchor).
fn validate_rule_membership(
    contract: &RuleProgramContractV2,
    document: &SemanticDocumentV2,
) -> Result<(), PlanningDiagnosticV1> {
    let mut member_ids = Vec::with_capacity(contract.rules().len());
    for member in contract.rules() {
        let anchor = rule_location(member);
        let Some(resolved) = document.item(member.item_id()) else {
            return Err(absent(anchor));
        };
        let SemanticItemV2::Rule(resolved) = resolved else {
            return Err(wrong_kind(anchor));
        };
        if resolved != member {
            return Err(identity_mismatch(anchor));
        }
        member_ids.push(member.item_id());
    }

    // Completeness and authored order: the document's own Rule items must be
    // exactly this member sequence, and no Rule item may sit outside it.
    let document_members: Vec<SemanticItemIdV2> = document
        .order()
        .iter()
        .filter(|id| matches!(document.item(**id), Some(SemanticItemV2::Rule(_))))
        .copied()
        .collect();
    if document_members != member_ids {
        return Err(identity_mismatch(&document_anchor(document)));
    }
    let members: BTreeSet<SemanticItemIdV2> = member_ids.iter().copied().collect();
    if document
        .items()
        .any(|(id, item)| matches!(item, SemanticItemV2::Rule(_)) && !members.contains(id))
    {
        return Err(identity_mismatch(&document_anchor(document)));
    }
    // The rule-graph budget is the document's immutable receipt, never a
    // caller-supplied replacement.
    if contract.budget() != document.budget_receipt() {
        return Err(identity_mismatch(&document_anchor(document)));
    }
    Ok(())
}

/// Whether the carrier's nested confirm graph is exactly the document's.
fn nested_graph_matches(resolved: &ResolvedCheckV2, contract: &CheckContractV2) -> bool {
    match (resolved.nested_graph(), contract.nested_graph()) {
        (None, None) => true,
        (Some(resolved_nested), Some(sealed)) => {
            GraphQueryContractV2::from_resolved(resolved_nested)
                .is_ok_and(|rebuilt| rebuilt == *sealed)
        }
        (Some(_), None) | (None, Some(_)) => false,
    }
}

/// The checked location an input-level outcome is attributed to.
///
/// A relational carrier is anchored on its own lowered root and a composition on
/// the composition item itself; every other arm shares the one arm-to-location
/// mapping of [`ArmV2::root_location`], so the anchor vocabulary cannot drift
/// from the attribution the assembled plan uses.
fn attribution_anchor(
    input: &SemanticPlanInputV2,
    document: &SemanticDocumentV2,
) -> CheckedLocatedSpanV2 {
    match input {
        SemanticPlanInputV2::Command(command) => command.location().clone(),
        SemanticPlanInputV2::Relational(item) => item.root().location().clone(),
        SemanticPlanInputV2::Composition(composition) => composition.location().clone(),
        SemanticPlanInputV2::Graph(_)
        | SemanticPlanInputV2::Check(_)
        | SemanticPlanInputV2::Presentation(_)
        | SemanticPlanInputV2::Rule(_)
        | SemanticPlanInputV2::MetaRequireCertified(_) => ArmV2::of(input).root_location(document),
    }
}

/// The document's own anchor: the checked location of its first item.
///
/// A document that carries no item at all retains no checked item location, so
/// the fallback is the source-start anchor retained by semantic admission from
/// the transaction's original extended registry. Planning never creates a
/// replacement registry to manufacture a diagnostic location.
fn document_anchor(document: &SemanticDocumentV2) -> CheckedLocatedSpanV2 {
    document
        .order()
        .iter()
        .find_map(|id| document.item(*id))
        .and_then(item_location)
        .unwrap_or_else(|| document.source_anchor().clone())
}

/// The checked location of one semantic item, where the item retains one.
fn item_location(item: &SemanticItemV2) -> Option<CheckedLocatedSpanV2> {
    match item {
        SemanticItemV2::Command(command) => Some(command.location().clone()),
        SemanticItemV2::Relation(relation) => Some(relation.location().clone()),
        SemanticItemV2::Rule(rule) => Some(rule_location(rule).clone()),
        SemanticItemV2::Graph(graph) => Some(graph.location().clone()),
        SemanticItemV2::Check(check) => Some(check.location().clone()),
        SemanticItemV2::Presentation(presentation) => Some(presentation.location().clone()),
        SemanticItemV2::Catalog(catalog) => {
            catalog.inline_view().map(|view| view.location().clone())
        }
        SemanticItemV2::Declaration(declaration) => Some(declaration.body().location().clone()),
        SemanticItemV2::Composition(composition) => Some(composition.location().clone()),
        SemanticItemV2::Meta(meta) => Some(meta.location().clone()),
    }
}

/// The rule's own checked body location.
fn rule_location(rule: &ResolvedRuleV2) -> &CheckedLocatedSpanV2 {
    rule.body().expression().location()
}

fn absent(span: &CheckedLocatedSpanV2) -> PlanningDiagnosticV1 {
    PlanningDiagnosticV1::InputDocumentMismatch {
        reason: InputDocumentMismatchReasonV2::Absent,
        span: span.clone(),
    }
}

fn wrong_kind(span: &CheckedLocatedSpanV2) -> PlanningDiagnosticV1 {
    PlanningDiagnosticV1::InputDocumentMismatch {
        reason: InputDocumentMismatchReasonV2::WrongKind,
        span: span.clone(),
    }
}

fn identity_mismatch(span: &CheckedLocatedSpanV2) -> PlanningDiagnosticV1 {
    PlanningDiagnosticV1::InputDocumentMismatch {
        reason: InputDocumentMismatchReasonV2::IdentityMismatch,
        span: span.clone(),
    }
}

// ---------------------------------------------------------------------------
// Phase 2: the one iterative, deterministic, budgeted traversal
// ---------------------------------------------------------------------------

/// Everything the one budgeted traversal derives from the visited sealed inputs.
struct WalkV2 {
    /// Capability requirements implied by the visited sealed constructs.
    capabilities: BTreeSet<CapabilityV2>,
    /// Checked algebra families, root-first in stable pre-order. Empty for the
    /// non-relational arms, which have no algebra tree.
    families: Vec<AlgebraFamilyV2>,
    /// The coarse cardinality class of the walked result.
    cardinality_class: CardinalityClassV2,
    /// The folded expression-effect projection of every retained effect visited.
    effects: EffectSummaryV2,
    /// How many checked algebra nodes were charged and visited.
    algebra_nodes: u32,
}

/// One pending traversal unit. `Enter`/`Exit` pairs implement the post-order
/// cardinality fold without recursion.
enum WalkFrameV2<'a> {
    /// One sealed input arm.
    Input(&'a SemanticPlanInputV2),
    /// One document rule member.
    RuleMember(&'a ResolvedRuleV2),
    /// A checked algebra node about to be visited.
    AlgebraEnter(&'a CheckedAlgebraNodeV2),
    /// A checked algebra node whose children are all visited. `value_base`
    /// isolates this node's ordered child results from sibling/ancestor folds.
    AlgebraExit {
        node: &'a CheckedAlgebraNodeV2,
        value_base: usize,
    },
}

/// Charge-before-visit semantic-step accounting against the document receipt.
struct ChargeV2 {
    limit: u32,
    used: u32,
}

impl ChargeV2 {
    const fn new(limit: u32) -> Self {
        Self { limit, used: 0 }
    }

    /// Charge one step for a node, or refuse at the exact sealed limit.
    fn charge(&mut self, span: &CheckedLocatedSpanV2) -> Result<(), PlanningDiagnosticV1> {
        if self.used == self.limit {
            return Err(PlanningDiagnosticV1::SemanticStepBudgetExceeded {
                limit: self.limit,
                span: span.clone(),
            });
        }
        self.used = self.used.saturating_add(1);
        Ok(())
    }
}

/// Walk one sealed planning input, charging the document's `max_semantic_steps`
/// once per visited node BEFORE the visit.
///
/// Frames are pushed in reverse so nodes are visited root-first and
/// left-before-right, and a composition's ordered producers are visited before
/// its body. The exact sealed limit succeeds; the next charge rejects at the
/// refused node's checked location.
///
/// # Errors
///
/// Returns [`PlanningDiagnosticV1::SemanticStepBudgetExceeded`] at the first
/// node whose charge would exceed the sealed limit.
fn walk(
    input: &SemanticPlanInputV2,
    charge: &mut ChargeV2,
) -> Result<WalkV2, PlanningDiagnosticV1> {
    let mut capabilities = BTreeSet::new();
    let mut families = Vec::new();
    let mut effects = ExpressionEffectsV2::pure_scalar();
    let mut algebra_nodes = 0_u32;
    let mut algebra_depth = 0_u32;
    let mut plan_class: Option<CardinalityClassV2> = None;
    let mut values: Vec<CardinalityClassV2> = Vec::new();
    let mut stack: Vec<WalkFrameV2<'_>> = vec![WalkFrameV2::Input(input)];

    while let Some(frame) = stack.pop() {
        match frame {
            WalkFrameV2::Input(arm) => match arm {
                SemanticPlanInputV2::Command(command) => {
                    charge.charge(command.location())?;
                    capabilities.insert(CapabilityV2::SymbolRenamePreview);
                    if command.phase() == quanta_gqlang_v2_sema::RenamePhaseV3::Apply {
                        capabilities.insert(CapabilityV2::SymbolRenameApply);
                    }
                    plan_class = Some(CardinalityClassV2::ScalarOrSingleton);
                }
                SemanticPlanInputV2::Relational(item) => {
                    stack.push(WalkFrameV2::AlgebraEnter(item.root()));
                }
                SemanticPlanInputV2::Graph(contract) => {
                    plan_class = Some(visit_leaf(
                        LeafCarrierV2::Graph(contract),
                        charge,
                        &mut capabilities,
                    )?);
                }
                SemanticPlanInputV2::Check(contract) => {
                    plan_class = Some(visit_leaf(
                        LeafCarrierV2::Check(contract),
                        charge,
                        &mut capabilities,
                    )?);
                }
                SemanticPlanInputV2::Presentation(contract) => {
                    plan_class = Some(visit_leaf(
                        LeafCarrierV2::Presentation(contract),
                        charge,
                        &mut capabilities,
                    )?);
                }
                SemanticPlanInputV2::Rule(contract) => {
                    // The program node itself is anchored on its first member;
                    // each member is then visited in authored order.
                    if let Some(first) = contract.rules().first() {
                        charge.charge(rule_location(first))?;
                    }
                    for rule in contract.rules().iter().rev() {
                        stack.push(WalkFrameV2::RuleMember(rule));
                    }
                }
                SemanticPlanInputV2::MetaRequireCertified(meta) => {
                    charge.charge(meta.location())?;
                    plan_class = Some(CardinalityClassV2::LinearInInput);
                }
                SemanticPlanInputV2::Composition(composition) => {
                    charge.charge(composition.location())?;
                    // Ordered producers are visited in authored LET order before
                    // the body; none of them becomes a plan of its own.
                    for producer in composition.producers() {
                        charge.charge(producer.name_location())?;
                    }
                    match composition.body() {
                        CompositionBodyPlanInputV2::Relational(body) => {
                            stack.push(WalkFrameV2::AlgebraEnter(body.root()));
                        }
                        CompositionBodyPlanInputV2::Graph(contract) => {
                            plan_class = Some(visit_leaf(
                                LeafCarrierV2::Graph(contract),
                                charge,
                                &mut capabilities,
                            )?);
                        }
                        CompositionBodyPlanInputV2::Check(contract) => {
                            plan_class = Some(visit_leaf(
                                LeafCarrierV2::Check(contract),
                                charge,
                                &mut capabilities,
                            )?);
                        }
                        CompositionBodyPlanInputV2::Explain(contract) => {
                            plan_class = Some(visit_leaf(
                                LeafCarrierV2::Presentation(contract),
                                charge,
                                &mut capabilities,
                            )?);
                        }
                    }
                }
            },
            WalkFrameV2::RuleMember(rule) => {
                charge.charge(rule_location(rule))?;
                effects = effects.join(rule.head_term().effects());
                effects = effects.join(rule.body().expression().effects());
            }
            WalkFrameV2::AlgebraEnter(node) => {
                charge.charge(node.location())?;
                algebra_nodes = algebra_nodes.saturating_add(1);
                algebra_depth = algebra_depth.saturating_add(1);
                let family = node.family();
                families.push(family);
                if let Some(capability) = capability_for_family(family) {
                    capabilities.insert(capability);
                }
                for term in node.existential_terms() {
                    capabilities.insert(match term.polarity() {
                        quanta_gqlang_v2_sema::RelationExistsPolarityV2::Exists => {
                            CapabilityV2::ExistenceCheck
                        }
                        quanta_gqlang_v2_sema::RelationExistsPolarityV2::NotExists => {
                            CapabilityV2::NegationCheck
                        }
                    });
                }
                effects = effects.join(node.output().effects());
                for expression in node.expressions() {
                    effects = effects.join(expression.effects());
                }
                stack.push(WalkFrameV2::AlgebraExit {
                    node,
                    value_base: values.len(),
                });
                let children = node.children();
                for child in children.iter().rev() {
                    stack.push(WalkFrameV2::AlgebraEnter(child));
                }
            }
            WalkFrameV2::AlgebraExit { node, value_base } => {
                let class = exit_class(node, values.iter().skip(value_base).copied());
                values.truncate(value_base);
                algebra_depth = algebra_depth.saturating_sub(1);
                if algebra_depth == 0 {
                    plan_class = Some(class);
                }
                values.push(class);
            }
        }
    }

    Ok(WalkV2 {
        capabilities,
        families,
        cardinality_class: plan_class.unwrap_or(CardinalityClassV2::LinearInInput),
        effects: EffectSummaryV2::new(
            effects.purity,
            effects.determinism,
            effects.monotonicity,
            effects.aggregation,
        ),
        algebra_nodes,
    })
}

/// One sealed leaf carrier the traversal charges a visit for: a FIND, a CHECK,
/// or a presentation verb.
///
/// A top-level input arm and a composition body of the same family are the SAME
/// carrier, so they are charged, capability-collected, and cardinality-classified
/// in exactly one place.
#[derive(Clone, Copy)]
enum LeafCarrierV2<'a> {
    /// A sealed FIND contract.
    Graph(&'a GraphQueryContractV2),
    /// A validated CHECK contract.
    Check(&'a CheckContractV2),
    /// A validated presentation contract.
    Presentation(&'a PresentationContractV2),
}

/// Visit one sealed leaf carrier: charge every step it owns, record the
/// capabilities its sealed facts imply, and derive its cardinality class.
///
/// # Errors
///
/// Returns [`PlanningDiagnosticV1::SemanticStepBudgetExceeded`] at the first
/// charge the sealed limit refuses.
fn visit_leaf(
    carrier: LeafCarrierV2<'_>,
    charge: &mut ChargeV2,
    capabilities: &mut BTreeSet<CapabilityV2>,
) -> Result<CardinalityClassV2, PlanningDiagnosticV1> {
    match carrier {
        LeafCarrierV2::Graph(contract) => {
            charge.charge(contract.location())?;
            capabilities.insert(CapabilityV2::WitnessMaterialization);
            if contract.shape() == RelationShapeV1::Cyclic {
                capabilities.insert(CapabilityV2::RecursiveClosure);
            }
            Ok(graph_class(contract))
        }
        LeafCarrierV2::Check(contract) => {
            charge.charge(contract.location())?;
            if let Some(nested) = contract.nested_graph() {
                charge.charge(nested.location())?;
                capabilities.insert(CapabilityV2::WitnessMaterialization);
                if nested.shape() == RelationShapeV1::Cyclic {
                    capabilities.insert(CapabilityV2::RecursiveClosure);
                }
            }
            Ok(check_class(contract))
        }
        LeafCarrierV2::Presentation(contract) => {
            charge.charge(contract.location())?;
            Ok(presentation_class(contract.kind()))
        }
    }
}

/// The coarse cardinality class of one algebra node, folded from its children's
/// already-computed classes.
fn exit_class(
    node: &CheckedAlgebraNodeV2,
    mut children: impl Iterator<Item = CardinalityClassV2>,
) -> CardinalityClassV2 {
    match node.family() {
        AlgebraFamilyV2::Scan => CardinalityClassV2::LinearInInput,
        AlgebraFamilyV2::Limit => {
            if node.limit_count().is_some_and(|count| count <= 1) {
                CardinalityClassV2::ScalarOrSingleton
            } else {
                CardinalityClassV2::Bounded
            }
        }
        AlgebraFamilyV2::Join => CardinalityClassV2::Multiplicative,
        AlgebraFamilyV2::Aggregate => {
            // An aggregate's output cardinality is governed by its GROUPING
            // partition, not by the size of the input it folds: a GROUPED
            // aggregate emits one row per distinct key, while a SCALAR aggregate
            // has no partition keys and folds the whole input into one row. The
            // fold reads the ABSENCE of a non-empty partition from the sealed
            // operator rather than asserting the empty-keyed `Group` structure.
            let grouped = matches!(
                node.children().first(),
                Some(input)
                    if input.family() == AlgebraFamilyV2::Group && !input.expressions().is_empty()
            );
            let child = children.next().unwrap_or(CardinalityClassV2::LinearInInput);
            if grouped {
                child
            } else {
                CardinalityClassV2::ScalarOrSingleton
            }
        }
        AlgebraFamilyV2::Group
        | AlgebraFamilyV2::Project
        | AlgebraFamilyV2::Filter
        | AlgebraFamilyV2::Sort => children.next().unwrap_or(CardinalityClassV2::LinearInInput),
        AlgebraFamilyV2::SemiJoin | AlgebraFamilyV2::AntiJoin | AlgebraFamilyV2::ExistsFilter => {
            // The result is bounded by the LEFT input; the right input only
            // decides existence. Child results are source ordered, so do not
            // recover them by popping a shared stack: nested joins can leave
            // unrelated intermediate values adjacent to this node's results.
            children.next().unwrap_or(CardinalityClassV2::LinearInInput)
        }
    }
}

/// The cardinality class of a sealed FIND result.
fn graph_class(contract: &GraphQueryContractV2) -> CardinalityClassV2 {
    if contract.shape() == RelationShapeV1::Cyclic {
        CardinalityClassV2::Closure
    } else {
        CardinalityClassV2::LinearInInput
    }
}

/// The cardinality class of a CHECK result: an alert row is bounded by the
/// nested confirm closure only when that closure recurses.
fn check_class(contract: &CheckContractV2) -> CardinalityClassV2 {
    let cyclic = contract
        .nested_graph()
        .is_some_and(|nested| nested.shape() == RelationShapeV1::Cyclic);
    if cyclic {
        CardinalityClassV2::Closure
    } else {
        CardinalityClassV2::LinearInInput
    }
}

/// The cardinality class of a sealed presentation result.
fn presentation_class(kind: &ResolvedPresentationKindV2) -> CardinalityClassV2 {
    match kind {
        ResolvedPresentationKindV2::Sample { .. } => CardinalityClassV2::Bounded,
        ResolvedPresentationKindV2::Explain { .. }
        | ResolvedPresentationKindV2::Show { .. }
        | ResolvedPresentationKindV2::Describe { .. }
        | ResolvedPresentationKindV2::Compare { .. }
        | ResolvedPresentationKindV2::Export { .. } => CardinalityClassV2::LinearInInput,
    }
}

// ---------------------------------------------------------------------------
// Phase 3: per-arm classification
// ---------------------------------------------------------------------------

/// One sealed carrier, viewed as the arm it classifies.
///
/// The composition arm views its sealed BODY through this same type, so the body
/// is classified by the one classifier rather than by a parallel one.
#[derive(Clone, Copy)]
enum ArmV2<'a> {
    Command(&'a ResolvedRenameCommandV3),
    /// One relational plan root: a top-level lowered item's root, or a
    /// composition body's lowered nested root.
    Relational(&'a CheckedAlgebraNodeV2),
    Graph(&'a GraphQueryContractV2),
    Check(&'a CheckContractV2),
    Presentation(&'a PresentationContractV2),
    Rule(&'a RuleProgramContractV2),
    MetaRequireCertified(&'a ResolvedMetaV2),
}

impl<'a> ArmV2<'a> {
    /// The arm one sealed planning input classifies.
    ///
    /// A composition input is represented by its sealed body, which is the
    /// carrier whose plan the composition's plan is.
    fn of(input: &'a SemanticPlanInputV2) -> Self {
        match input {
            SemanticPlanInputV2::Command(command) => Self::Command(command),
            SemanticPlanInputV2::Relational(item) => Self::Relational(item.root()),
            SemanticPlanInputV2::Graph(contract) => Self::Graph(contract),
            SemanticPlanInputV2::Check(contract) => Self::Check(contract),
            SemanticPlanInputV2::Presentation(contract) => Self::Presentation(contract),
            SemanticPlanInputV2::Rule(contract) => Self::Rule(contract),
            SemanticPlanInputV2::MetaRequireCertified(meta) => Self::MetaRequireCertified(meta),
            SemanticPlanInputV2::Composition(composition) => Self::of_body(composition.body()),
        }
    }

    /// The arm one sealed composition body classifies.
    fn of_body(body: &'a CompositionBodyPlanInputV2) -> Self {
        match body {
            CompositionBodyPlanInputV2::Relational(body) => Self::Relational(body.root()),
            CompositionBodyPlanInputV2::Graph(contract) => Self::Graph(contract),
            CompositionBodyPlanInputV2::Check(contract) => Self::Check(contract),
            CompositionBodyPlanInputV2::Explain(contract) => Self::Presentation(contract),
        }
    }

    /// The checked location this arm's plan is attributed to: the planned item's
    /// own checked location where the document holds one, else the location the
    /// carrier itself retains.
    ///
    /// The document-wide rule program retains no location of its own; it is
    /// attributed to its first member, else to the document's own anchor.
    fn root_location(self, document: &SemanticDocumentV2) -> CheckedLocatedSpanV2 {
        match self {
            Self::Command(command) => command.location().clone(),
            Self::Relational(root) => document
                .item(root.item_id())
                .and_then(item_location)
                .unwrap_or_else(|| root.location().clone()),
            Self::Graph(contract) => contract.location().clone(),
            Self::Check(contract) => contract.location().clone(),
            Self::Presentation(contract) => contract.location().clone(),
            Self::Rule(contract) => contract.rules().first().map_or_else(
                || document_anchor(document),
                |rule| rule_location(rule).clone(),
            ),
            Self::MetaRequireCertified(meta) => meta.location().clone(),
        }
    }
}

/// The root one classification is produced under.
///
/// It decides both the location the plan is attributed to and the policy the
/// plan retains, so an arm can never be classified under one root and retain
/// another's policy.
#[derive(Clone, Copy)]
enum PlanRootV2<'a> {
    /// The input's own carrier is the plan's root.
    Carrier,
    /// One checked composition is the plan's root: the classified carrier is its
    /// sealed body, and the plan retains the composition identity plus the body's
    /// own policy.
    Composition(&'a CheckedCompositionPlanInputV2),
}

impl<'a> PlanRootV2<'a> {
    /// The root one sealed planning input is classified under.
    fn of(input: &'a SemanticPlanInputV2) -> Self {
        match input {
            SemanticPlanInputV2::Composition(composition) => Self::Composition(composition),
            SemanticPlanInputV2::Relational(_)
            | SemanticPlanInputV2::Command(_)
            | SemanticPlanInputV2::Graph(_)
            | SemanticPlanInputV2::Check(_)
            | SemanticPlanInputV2::Presentation(_)
            | SemanticPlanInputV2::Rule(_)
            | SemanticPlanInputV2::MetaRequireCertified(_) => Self::Carrier,
        }
    }
}

/// The nine derived fields of one plan, before assembly.
struct PlanPartsV2 {
    authority_lane: AuthorityLaneV2,
    evaluation_determinism: EvaluationDeterminismV2,
    evidence: EvidenceRequirementV2,
    effect: EffectClassV2,
    result: ResultContractV2,
    capabilities: BTreeSet<CapabilityV2>,
    semantic_cost: SemanticCostV2,
    explain: ExplainNoteV2,
    policy: PlanPolicyV2,
}

/// The arm-specific classification, before the shared folds run.
struct ArmFactsV2 {
    authority_lane: AuthorityLaneV2,
    evidence: EvidenceRequirementV2,
    effect: EffectClassV2,
    result_kind: ResultKindV2,
    render: RenderRequirementV2,
    witness_selection: Option<WitnessSelectionPolicyV2>,
    cardinality: CardinalityV1,
    cardinality_class: CardinalityClassV2,
    recursion_risk: RecursionRiskClassV2,
    witness_risk: WitnessExplosionRiskClassV2,
    recursive: bool,
    policy: PlanPolicyV2,
}

/// Classify one sealed arm from its carrier plus the one traversal's folds.
///
/// `root` is the one root the classification is produced under: a composition
/// passes its OWN checked root, so the assembled plan is attributed to the
/// composition item and retains the composition's policy around the body policy
/// this same call derived.
///
/// # Errors
///
/// Returns [`PlanningDiagnosticV1::InvalidComposition`] when a composition
/// root's body policy cannot be retained by the composition it was classified
/// for.
fn classify(
    arm: ArmV2<'_>,
    document: &SemanticDocumentV2,
    walk: &WalkV2,
    root: PlanRootV2<'_>,
) -> Result<PlanPartsV2, PlanningDiagnosticV1> {
    let mut capabilities = walk.capabilities.clone();
    let facts = match arm {
        ArmV2::Command(command) => command_facts(command),
        ArmV2::Relational(root) => relational_facts(root, walk),
        ArmV2::Graph(contract) => graph_facts(contract),
        ArmV2::Check(contract) => check_facts(contract),
        ArmV2::Presentation(contract) => presentation_facts(contract),
        ArmV2::Rule(contract) => {
            let facts = rule_facts(contract);
            if facts.recursive {
                capabilities.insert(CapabilityV2::RecursiveClosure);
            }
            facts
        }
        ArmV2::MetaRequireCertified(_meta) => meta_facts(),
    };
    let (root_location, policy) = match root {
        PlanRootV2::Carrier => (arm.root_location(document), facts.policy.clone()),
        PlanRootV2::Composition(composition) => (
            composition.location().clone(),
            PlanPolicyV2::Composition(composition.policy(facts.policy.clone())?),
        ),
    };
    Ok(assemble_plan_parts_v2(
        facts,
        walk,
        capabilities,
        root_location,
        policy,
    ))
}

fn assemble_plan_parts_v2(
    facts: ArmFactsV2,
    walk: &WalkV2,
    capabilities: BTreeSet<CapabilityV2>,
    root_location: CheckedLocatedSpanV2,
    policy: PlanPolicyV2,
) -> PlanPartsV2 {
    let (result, semantic_cost) = classified_result_and_cost(&facts);
    PlanPartsV2 {
        authority_lane: facts.authority_lane,
        evaluation_determinism: determinism_of(walk.effects),
        evidence: facts.evidence,
        effect: facts.effect,
        result,
        capabilities,
        semantic_cost,
        explain: ExplainNoteV2::new(root_location, walk.families.clone(), facts.recursive),
        policy,
    }
}

fn classified_result_and_cost(facts: &ArmFactsV2) -> (ResultContractV2, SemanticCostV2) {
    let result = ResultContractV2::new(
        facts.result_kind,
        facts.render,
        facts.witness_selection,
        MultiplicityOrderContractV2::from_cardinality(facts.cardinality),
    );
    let cost = SemanticCostV2 {
        estimated_cardinality_class: facts.cardinality_class,
        recursion_risk_class: facts.recursion_risk,
        witness_explosion_risk_class: facts.witness_risk,
        ordering_contract: ordering_contract(facts.cardinality),
    };
    (result, cost)
}

fn command_facts(command: &ResolvedRenameCommandV3) -> ArmFactsV2 {
    ArmFactsV2 {
        authority_lane: AuthorityLaneV2::FrontendChecked,
        evidence: EvidenceRequirementV2::None,
        effect: if command.phase() == quanta_gqlang_v2_sema::RenamePhaseV3::Apply {
            EffectClassV2::MutationRequiresGrant
        } else {
            EffectClassV2::ReadOnly
        },
        result_kind: ResultKindV2::CommandReceipt,
        render: RenderRequirementV2::CommandReceipt,
        witness_selection: None,
        cardinality: CardinalityV1::Set,
        cardinality_class: CardinalityClassV2::ScalarOrSingleton,
        recursion_risk: RecursionRiskClassV2::None,
        witness_risk: WitnessExplosionRiskClassV2::None,
        recursive: false,
        policy: PlanPolicyV2::Command(command.clone()),
    }
}

/// The relational arm: an ordinary checked relation renders a read-only table.
fn relational_facts(root: &CheckedAlgebraNodeV2, walk: &WalkV2) -> ArmFactsV2 {
    relational_facts_from_shape(
        root.output().cardinality(),
        walk.cardinality_class,
        walk.algebra_nodes,
        walk.effects,
    )
}

fn relational_facts_from_shape(
    cardinality: CardinalityV1,
    cardinality_class: CardinalityClassV2,
    nodes: u32,
    effects: EffectSummaryV2,
) -> ArmFactsV2 {
    ArmFactsV2 {
        authority_lane: AuthorityLaneV2::FrontendChecked,
        evidence: EvidenceRequirementV2::None,
        effect: EffectClassV2::ReadOnly,
        result_kind: ResultKindV2::Table,
        render: RenderRequirementV2::RowsOnly,
        witness_selection: None,
        cardinality,
        cardinality_class,
        recursion_risk: RecursionRiskClassV2::None,
        witness_risk: WitnessExplosionRiskClassV2::None,
        recursive: false,
        policy: PlanPolicyV2::Relational { nodes, effects },
    }
}

/// The graph arm: a sealed FIND materializes bounded witnesses, so its result is
/// a path/witness result backed by the exact sealed provenance.
fn graph_facts(contract: &GraphQueryContractV2) -> ArmFactsV2 {
    let cyclic = contract.shape() == RelationShapeV1::Cyclic;
    let selection = witness_selection(contract.max_witnesses());
    ArmFactsV2 {
        authority_lane: AuthorityLaneV2::FrontendChecked,
        evidence: EvidenceRequirementV2::WitnessPayload,
        effect: EffectClassV2::ProofEvidenceOnly,
        result_kind: ResultKindV2::PathWitness,
        render: RenderRequirementV2::WitnessPayload,
        witness_selection: Some(selection),
        // Witnesses are deduplicated by the sealed dedup identity and carry no
        // semantic order.
        cardinality: CardinalityV1::Set,
        cardinality_class: if cyclic {
            CardinalityClassV2::Closure
        } else {
            CardinalityClassV2::LinearInInput
        },
        recursion_risk: if cyclic {
            RecursionRiskClassV2::BoundDependent
        } else {
            RecursionRiskClassV2::None
        },
        witness_risk: witness_risk(cyclic, selection),
        recursive: cyclic,
        policy: PlanPolicyV2::Graph(GraphWitnessRetentionV2::from_contract(contract)),
    }
}

/// The CHECK arm: an alert whose evidence follows `confirm`, whose render and
/// effect follow `emit`, and whose nested witness provenance is retained.
fn check_facts(contract: &CheckContractV2) -> ArmFactsV2 {
    let nested = contract.policy().nested_graph();
    let cyclic = nested.is_some_and(|retention| retention.shape() == RelationShapeV1::Cyclic);
    let selection = nested.map(|retention| witness_selection(retention.max_witnesses()));
    let sarif = contract.emit() == EmitModeV1::Sarif;
    ArmFactsV2 {
        authority_lane: AuthorityLaneV2::FrontendChecked,
        evidence: if contract.confirm() == ConfirmModeV1::Require {
            EvidenceRequirementV2::ProofTree
        } else {
            EvidenceRequirementV2::SourceLocation
        },
        effect: if sarif {
            EffectClassV2::ExportOnly
        } else {
            EffectClassV2::ReadOnly
        },
        result_kind: ResultKindV2::Alert,
        render: if sarif {
            RenderRequirementV2::SarifFindings
        } else {
            RenderRequirementV2::SourceLocation
        },
        witness_selection: selection,
        cardinality: CardinalityV1::Set,
        cardinality_class: if cyclic {
            CardinalityClassV2::Closure
        } else {
            CardinalityClassV2::LinearInInput
        },
        recursion_risk: if cyclic {
            RecursionRiskClassV2::BoundDependent
        } else {
            RecursionRiskClassV2::None
        },
        witness_risk: selection.map_or(WitnessExplosionRiskClassV2::None, |selection| {
            witness_risk(cyclic, selection)
        }),
        recursive: cyclic,
        policy: PlanPolicyV2::Check(contract.policy().clone()),
    }
}

/// The presentation arms: each sealed verb keeps its exact payload and derives
/// its own result kind, render payload, and effect.
fn presentation_facts(contract: &PresentationContractV2) -> ArmFactsV2 {
    let kind = contract.kind();
    let (result_kind, render, evidence, effect, cardinality_class) = match kind {
        ResolvedPresentationKindV2::Explain { .. } => (
            ResultKindV2::ProofExplain,
            RenderRequirementV2::ProvenanceTree,
            EvidenceRequirementV2::ProofTree,
            EffectClassV2::ReadOnly,
            CardinalityClassV2::LinearInInput,
        ),
        ResolvedPresentationKindV2::Show { .. } => (
            ResultKindV2::Table,
            RenderRequirementV2::RowsOnly,
            EvidenceRequirementV2::None,
            EffectClassV2::ReadOnly,
            CardinalityClassV2::LinearInInput,
        ),
        ResolvedPresentationKindV2::Describe { .. } => (
            ResultKindV2::Alert,
            RenderRequirementV2::SourceLocation,
            EvidenceRequirementV2::SourceLocation,
            EffectClassV2::ReadOnly,
            CardinalityClassV2::LinearInInput,
        ),
        ResolvedPresentationKindV2::Sample { .. } => (
            ResultKindV2::Table,
            RenderRequirementV2::RowsOnly,
            EvidenceRequirementV2::None,
            EffectClassV2::ReadOnly,
            CardinalityClassV2::Bounded,
        ),
        ResolvedPresentationKindV2::Compare { .. } => (
            ResultKindV2::PathWitness,
            RenderRequirementV2::WitnessPayload,
            EvidenceRequirementV2::WitnessPayload,
            EffectClassV2::ReadOnly,
            CardinalityClassV2::LinearInInput,
        ),
        ResolvedPresentationKindV2::Export { format, .. } => (
            ResultKindV2::Table,
            if *format == ExportFormatV1::Sarif {
                RenderRequirementV2::SarifFindings
            } else {
                RenderRequirementV2::RowsOnly
            },
            EvidenceRequirementV2::None,
            EffectClassV2::ExportOnly,
            CardinalityClassV2::LinearInInput,
        ),
    };
    ArmFactsV2 {
        authority_lane: AuthorityLaneV2::FrontendChecked,
        evidence,
        effect,
        result_kind,
        render,
        witness_selection: None,
        cardinality: CardinalityV1::Set,
        cardinality_class,
        recursion_risk: RecursionRiskClassV2::None,
        witness_risk: WitnessExplosionRiskClassV2::None,
        recursive: false,
        policy: PlanPolicyV2::Presentation(kind.clone()),
    }
}

/// The document rule program arm: its policy is the stratification proof the
/// program's SOLE ingress ([`RuleProgramContractV2::from_document`]) already
/// computed for these exact members under the document's own rule-graph budget.
///
/// This arm never re-stratifies: the contract carries the one proof, so the
/// rule-graph dimensions are charged exactly once per document and planning can
/// not reach a different admission decision than the ingress did.
fn rule_facts(contract: &RuleProgramContractV2) -> ArmFactsV2 {
    let proof = contract.stratification();
    let recursive_members: Vec<ResolvedPredicateIdV2> = proof.recursive().iter().cloned().collect();
    let recursive = !recursive_members.is_empty();
    ArmFactsV2 {
        authority_lane: AuthorityLaneV2::FrontendChecked,
        evidence: EvidenceRequirementV2::None,
        effect: EffectClassV2::ReadOnly,
        result_kind: ResultKindV2::Table,
        render: RenderRequirementV2::RowsOnly,
        witness_selection: None,
        // A least-fixpoint closure is deduplicated and carries no semantic order.
        cardinality: CardinalityV1::Set,
        cardinality_class: if recursive {
            CardinalityClassV2::Closure
        } else {
            CardinalityClassV2::LinearInInput
        },
        recursion_risk: if recursive {
            RecursionRiskClassV2::BoundedFixpoint
        } else {
            RecursionRiskClassV2::None
        },
        witness_risk: WitnessExplosionRiskClassV2::None,
        recursive,
        policy: PlanPolicyV2::Rule(RuleProgramPolicyV2::new(
            contract,
            proof.stratum_count(),
            recursive_members,
        )),
    }
}

/// The `REQUIRE CERTIFIED` arm: the one lane whose authority is the certified
/// requirement the document's exact directive validated.
///
/// A `USE PACK` carrier never reaches this classifier: [`plan_outcome`] returns
/// the checked [`NoPlanReasonV2::MetaActivation`] outcome for it first.
fn meta_facts() -> ArmFactsV2 {
    ArmFactsV2 {
        authority_lane: AuthorityLaneV2::CertifiedRequirementValidated,
        evidence: EvidenceRequirementV2::ProofTree,
        effect: EffectClassV2::ProofEvidenceOnly,
        result_kind: ResultKindV2::ProofExplain,
        render: RenderRequirementV2::ProvenanceTree,
        witness_selection: None,
        cardinality: CardinalityV1::Set,
        cardinality_class: CardinalityClassV2::LinearInInput,
        recursion_risk: RecursionRiskClassV2::None,
        witness_risk: WitnessExplosionRiskClassV2::None,
        recursive: false,
        policy: PlanPolicyV2::MetaRequireCertified,
    }
}

/// The determinism lane of the folded sealed expression effects.
const fn determinism_of(effects: EffectSummaryV2) -> EvaluationDeterminismV2 {
    match effects.determinism {
        DeterminismV1::Deterministic => EvaluationDeterminismV2::Deterministic,
        DeterminismV1::NonDeterministic => EvaluationDeterminismV2::NonDeterministic,
    }
}

/// The order contract implied by the result's sealed cardinality (`D26`).
const fn ordering_contract(cardinality: CardinalityV1) -> OrderingContractV2 {
    match cardinality {
        CardinalityV1::Sequence => OrderingContractV2::Ordered,
        CardinalityV1::Set | CardinalityV1::Bag => OrderingContractV2::Unordered,
    }
}

/// The witness-selection policy the SEALED cap implies: exactly one admissible
/// witness selects the canonical single witness, a wider cap materializes every
/// witness up to it.
const fn witness_selection(max_witnesses: u32) -> WitnessSelectionPolicyV2 {
    if max_witnesses == 1 {
        WitnessSelectionPolicyV2::CanonicalOne
    } else {
        WitnessSelectionPolicyV2::AllWitness
    }
}

/// The witness/path explosion risk of a bounded witness family.
const fn witness_risk(
    cyclic: bool,
    selection: WitnessSelectionPolicyV2,
) -> WitnessExplosionRiskClassV2 {
    if cyclic {
        WitnessExplosionRiskClassV2::CapBounded
    } else {
        match selection {
            WitnessSelectionPolicyV2::CanonicalOne => WitnessExplosionRiskClassV2::SingleWitness,
            WitnessSelectionPolicyV2::TopK { .. } | WitnessSelectionPolicyV2::AllWitness => {
                WitnessExplosionRiskClassV2::BoundedSet
            }
        }
    }
}

#[cfg(test)]
#[path = "plan/tests.rs"]
mod tests;
