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
    CheckedLocatedSpanV2, ConfirmModeV1, EmitModeV1, ExportFormatV1, ResolvedCheckV2,
    ResolvedMetaKindV2, ResolvedMetaV2, ResolvedPresentationKindV2, ResolvedRuleV2,
    SemanticDocumentV2, SemanticItemIdV2, SemanticItemV2,
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
        AlgebraFamilyV2::SemiJoin | AlgebraFamilyV2::AntiJoin => {
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
        PlanRootV2::Carrier => (arm.root_location(document), facts.policy),
        PlanRootV2::Composition(composition) => (
            composition.location().clone(),
            PlanPolicyV2::Composition(composition.policy(facts.policy)?),
        ),
    };
    let result = ResultContractV2::new(
        facts.result_kind,
        facts.render,
        facts.witness_selection,
        MultiplicityOrderContractV2::from_cardinality(facts.cardinality),
    );
    let semantic_cost = SemanticCostV2 {
        estimated_cardinality_class: facts.cardinality_class,
        recursion_risk_class: facts.recursion_risk,
        witness_explosion_risk_class: facts.witness_risk,
        ordering_contract: ordering_contract(facts.cardinality),
    };
    Ok(PlanPartsV2 {
        authority_lane: facts.authority_lane,
        evaluation_determinism: determinism_of(walk.effects),
        evidence: facts.evidence,
        effect: facts.effect,
        result,
        capabilities,
        semantic_cost,
        explain: ExplainNoteV2::new(root_location, walk.families.clone(), facts.recursive),
        policy,
    })
}

/// The relational arm: an ordinary checked relation renders a read-only table.
fn relational_facts(root: &CheckedAlgebraNodeV2, walk: &WalkV2) -> ArmFactsV2 {
    ArmFactsV2 {
        authority_lane: AuthorityLaneV2::FrontendChecked,
        evidence: EvidenceRequirementV2::None,
        effect: EffectClassV2::ReadOnly,
        result_kind: ResultKindV2::Table,
        render: RenderRequirementV2::RowsOnly,
        witness_selection: None,
        cardinality: root.output().cardinality(),
        cardinality_class: walk.cardinality_class,
        recursion_risk: RecursionRiskClassV2::None,
        witness_risk: WitnessExplosionRiskClassV2::None,
        recursive: false,
        policy: PlanPolicyV2::Relational {
            nodes: walk.algebra_nodes,
            effects: walk.effects,
        },
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
mod tests {
    #![expect(
        clippy::panic_in_result_fn,
        reason = "these rails assert exact plan fields and return a typed fixture error, so every assertion is a deliberate crash on drift"
    )]

    use quanta_gqlang_v2_algebra::lower_relation_item;
    use quanta_gqlang_v2_sema::test_support::{
        deep_relation_chain_document_v2, forged_cardinality_downgrade_document_v2,
        forged_small_receipt_relation_chain_document_v2, reference_graph_document_v2,
        reference_relation_document_v2, reference_rule_document_v2,
        second_source_relation_document_v2, ReferenceBudgetV2, ReferenceGraphDocumentV2,
        ReferenceGraphReferenceV2, ReferenceGraphRequestV2, ReferenceRelationDocumentV2,
        ReferenceRelationFixtureV2, ReferenceRuleFixtureV2, SMALL_BUDGET_MAX_SEMANTIC_STEPS,
    };
    use quanta_gqlang_v2_sema::{GraphQueryKindV1, GraphReferenceRoleV2};

    use super::*;

    /// The checked location a document-level outcome is attributed to.
    fn expected_anchor(
        document: &SemanticDocumentV2,
        id: SemanticItemIdV2,
    ) -> Option<CheckedLocatedSpanV2> {
        document.item(id).and_then(item_location)
    }

    /// The visited checked algebra nodes and folded effects of a plan that must
    /// carry the relational policy. Every other arm is a named failure, so the
    /// match stays exhaustive over the closed policy vocabulary.
    fn relational_policy(plan: &SemanticPlanV2) -> Result<(u32, EffectSummaryV2), String> {
        match plan.policy() {
            PlanPolicyV2::Relational { nodes, effects } => Ok((*nodes, *effects)),
            PlanPolicyV2::Graph(_)
            | PlanPolicyV2::Check(_)
            | PlanPolicyV2::Presentation(_)
            | PlanPolicyV2::Rule(_)
            | PlanPolicyV2::MetaRequireCertified
            | PlanPolicyV2::Composition(_) => Err("expected a relational policy".to_owned()),
        }
    }

    /// One reference relation document plus the sealed planning input lowered
    /// from it.
    fn relational_fixture(
        fixture: ReferenceRelationFixtureV2,
    ) -> Result<(ReferenceRelationDocumentV2, SemanticPlanInputV2), String> {
        let document = reference_relation_document_v2(fixture)?;
        let input = lower_relation_item(document.document(), document.item_id())
            .map(|item| SemanticPlanInputV2::Relational(Box::new(item)))
            .map_err(|error| format!("{error:?}"))?;
        Ok((document, input))
    }

    /// A taint FIND document plus the sealed graph planning input.
    fn graph_fixture() -> Result<(ReferenceGraphDocumentV2, SemanticPlanInputV2), String> {
        let document = reference_graph_document_v2(ReferenceGraphRequestV2 {
            kind: GraphQueryKindV1::Taint,
            references: vec![
                ReferenceGraphReferenceV2 {
                    role: GraphReferenceRoleV2::Source,
                    selector: "input_a",
                    span: (20, 27),
                },
                ReferenceGraphReferenceV2 {
                    role: GraphReferenceRoleV2::Sink,
                    selector: "exec_b",
                    span: (30, 36),
                },
            ],
            depth: Some(4),
            direction: None,
            summary_facets: Vec::new(),
            item_span: (0, 8),
        })?;
        let contract = GraphQueryContractV2::from_resolved(document.graph())
            .map_err(|error| format!("{error:?}"))?;
        Ok((document, SemanticPlanInputV2::Graph(contract)))
    }

    /// An ordinary relation plans a read-only table whose explain note carries
    /// the checked algebra families in root-first order.
    #[test]
    fn a_relational_scan_plans_a_read_only_table() -> Result<(), String> {
        let (fixture, input) = relational_fixture(ReferenceRelationFixtureV2::Scan)?;
        let document = fixture.document();
        let plan = plan(&input, document).map_err(|error| format!("{error:?}"))?;

        assert_eq!(plan.authority_lane(), AuthorityLaneV2::FrontendChecked);
        assert_eq!(
            plan.evaluation_determinism(),
            EvaluationDeterminismV2::Deterministic
        );
        assert_eq!(plan.evidence(), EvidenceRequirementV2::None);
        assert_eq!(plan.effect(), EffectClassV2::ReadOnly);
        assert_eq!(plan.result().result_kind(), ResultKindV2::Table);
        assert_eq!(
            plan.result().render_requirement(),
            RenderRequirementV2::RowsOnly
        );
        assert_eq!(plan.result().witness_selection(), None);
        assert!(!plan.result().multiplicity_order().is_ordered());
        assert!(plan.capabilities().contains(&CapabilityV2::RelationScan));
        assert_eq!(
            plan.semantic_cost().estimated_cardinality_class,
            CardinalityClassV2::LinearInInput
        );
        assert_eq!(
            plan.semantic_cost().recursion_risk_class,
            RecursionRiskClassV2::None
        );
        assert_eq!(
            plan.semantic_cost().witness_explosion_risk_class,
            WitnessExplosionRiskClassV2::None
        );
        assert_eq!(
            plan.semantic_cost().ordering_contract,
            OrderingContractV2::Unordered
        );
        assert_eq!(plan.explain().families(), &[AlgebraFamilyV2::Scan]);
        assert!(!plan.explain().recursive());
        assert_eq!(
            Some(plan.explain().root_location()),
            expected_anchor(document, fixture.item_id()).as_ref()
        );
        let (nodes, effects) = relational_policy(&plan)?;
        assert_eq!(nodes, 1);
        assert_eq!(effects.determinism, DeterminismV1::Deterministic);
        Ok(())
    }

    /// The exact sealed semantic-step limit plans; one more visited stage is
    /// refused at that stage, charging before the visit.
    #[test]
    fn the_exact_step_limit_plans_and_one_more_is_refused() -> Result<(), String> {
        let exact_depth = SMALL_BUDGET_MAX_SEMANTIC_STEPS - 1;
        let exact_carrier =
            deep_relation_chain_document_v2(exact_depth, ReferenceBudgetV2::Standard)?;
        let exact_input = lower_relation_item(exact_carrier.document(), exact_carrier.item_id())
            .map(|item| SemanticPlanInputV2::Relational(Box::new(item)))
            .map_err(|error| format!("{error:?}"))?;
        let exact_document = forged_small_receipt_relation_chain_document_v2(exact_depth)?;
        let mut session = PlanningSessionV2::new(exact_document.document());
        session
            .plan_outcome(&exact_input)
            .map_err(|error| format!("{error:?}"))?;
        assert!(matches!(
            session.plan_outcome(&exact_input),
            Err(PlanningDiagnosticV1::SemanticStepBudgetExceeded { .. })
        ));

        let over_carrier = deep_relation_chain_document_v2(
            SMALL_BUDGET_MAX_SEMANTIC_STEPS,
            ReferenceBudgetV2::Standard,
        )?;
        let over_input = lower_relation_item(over_carrier.document(), over_carrier.item_id())
            .map(|item| SemanticPlanInputV2::Relational(Box::new(item)))
            .map_err(|error| format!("{error:?}"))?;
        let over_document =
            forged_small_receipt_relation_chain_document_v2(SMALL_BUDGET_MAX_SEMANTIC_STEPS)?;
        match plan(&over_input, over_document.document()) {
            Err(PlanningDiagnosticV1::SemanticStepBudgetExceeded { limit, .. }) => {
                assert_eq!(limit, SMALL_BUDGET_MAX_SEMANTIC_STEPS);
            }
            other => return Err(format!("expected the exact budget refusal, got {other:?}")),
        }
        Ok(())
    }

    /// A carrier from another document is rejected before any classification, and
    /// a same-identity carrier with different sealed facts is an identity
    /// mismatch.
    #[test]
    fn a_carrier_must_belong_to_the_document() -> Result<(), String> {
        let (_foreign, foreign_input) = relational_fixture(ReferenceRelationFixtureV2::Scan)?;
        let other = second_source_relation_document_v2()?;
        match plan(&foreign_input, other.document()) {
            Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, .. }) => {
                assert_eq!(reason, InputDocumentMismatchReasonV2::Absent);
            }
            error => {
                return Err(format!(
                    "expected an absent-identity rejection, got {error:?}"
                ))
            }
        }

        let (pristine, forged_input) =
            relational_fixture(ReferenceRelationFixtureV2::GroupedAggregateSum)?;
        let forged = forged_cardinality_downgrade_document_v2()?;
        match plan(&forged_input, forged.document()) {
            Err(PlanningDiagnosticV1::InputDocumentMismatch { reason, .. }) => {
                assert_eq!(reason, InputDocumentMismatchReasonV2::IdentityMismatch);
            }
            error => {
                return Err(format!(
                    "expected a sealed-identity rejection, got {error:?}"
                ))
            }
        }
        // The untouched carrier still plans against its own document.
        plan(&forged_input, pristine.document()).map_err(|error| format!("{error:?}"))?;
        Ok(())
    }

    /// A document-wide rule program with no member is a typed non-plan outcome,
    /// not an error and not a silent skip.
    #[test]
    fn an_empty_rule_program_is_a_typed_non_plan() -> Result<(), String> {
        let (fixture, _input) = relational_fixture(ReferenceRelationFixtureV2::Scan)?;
        let document = fixture.document();
        let contract =
            RuleProgramContractV2::from_document(document).map_err(|error| format!("{error:?}"))?;
        let input = SemanticPlanInputV2::Rule(contract);

        match plan_outcome(&input, document).map_err(|error| format!("{error:?}"))? {
            PlanOutcomeV2::NoPlan(reason) => assert_eq!(reason, NoPlanReasonV2::EmptyRuleProgram),
            PlanOutcomeV2::Plan(plan) => {
                return Err(format!("an empty rule program must not plan: {plan:?}"))
            }
        }
        match plan(&input, document) {
            Err(PlanningDiagnosticV1::NoPlan { reason, span }) => {
                assert_eq!(reason, NoPlanReasonV2::EmptyRuleProgram);
                assert_eq!(
                    Some(&span),
                    expected_anchor(document, fixture.item_id()).as_ref()
                );
            }
            other => {
                return Err(format!(
                    "expected the typed non-plan refusal, got {other:?}"
                ))
            }
        }
        Ok(())
    }

    /// A FIND plans a proof/evidence-only path result that retains the exact
    /// sealed witness provenance.
    #[test]
    fn a_find_plan_retains_the_sealed_witness_provenance() -> Result<(), String> {
        let (document, input) = graph_fixture()?;
        let plan = plan(&input, document.document()).map_err(|error| format!("{error:?}"))?;

        assert_eq!(plan.authority_lane(), AuthorityLaneV2::FrontendChecked);
        assert_eq!(plan.evidence(), EvidenceRequirementV2::WitnessPayload);
        assert_eq!(plan.effect(), EffectClassV2::ProofEvidenceOnly);
        assert_eq!(plan.result().result_kind(), ResultKindV2::PathWitness);
        assert_eq!(
            plan.result().render_requirement(),
            RenderRequirementV2::WitnessPayload
        );
        assert_eq!(
            plan.result().witness_selection(),
            Some(WitnessSelectionPolicyV2::AllWitness)
        );
        assert_eq!(
            plan.semantic_cost().estimated_cardinality_class,
            CardinalityClassV2::Closure
        );
        assert_eq!(
            plan.semantic_cost().recursion_risk_class,
            RecursionRiskClassV2::BoundDependent
        );
        assert_eq!(
            plan.semantic_cost().witness_explosion_risk_class,
            WitnessExplosionRiskClassV2::CapBounded
        );
        assert!(plan
            .capabilities()
            .contains(&CapabilityV2::WitnessMaterialization));
        assert!(plan
            .capabilities()
            .contains(&CapabilityV2::RecursiveClosure));
        assert!(plan.explain().recursive());
        assert!(plan.explain().families().is_empty());

        let SemanticPlanInputV2::Graph(contract) = &input else {
            return Err("the fixture seals a graph input".to_owned());
        };
        match plan.policy() {
            PlanPolicyV2::Graph(retention) => {
                assert_eq!(retention.item_id(), contract.item_id());
                assert_eq!(retention.depth(), 4);
                assert_eq!(retention.max_witnesses(), contract.max_witnesses());
                assert_eq!(retention.dedup(), contract.dedup());
                assert_eq!(retention.tie_break(), contract.tie_break());
                assert_eq!(
                    retention.max_path_length(),
                    contract.witness().max_path_length()
                );
            }
            PlanPolicyV2::Relational { .. }
            | PlanPolicyV2::Check(_)
            | PlanPolicyV2::Presentation(_)
            | PlanPolicyV2::Rule(_)
            | PlanPolicyV2::MetaRequireCertified
            | PlanPolicyV2::Composition(_) => {
                return Err("expected a graph policy".to_owned());
            }
        }
        Ok(())
    }

    /// A recursive document rule program plans a bounded fixpoint whose policy
    /// retains the stratified member summary.
    #[test]
    fn a_recursive_rule_program_plans_a_bounded_fixpoint() -> Result<(), String> {
        let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::PositiveSelfRecursion)?;
        let document = fixture.document();
        let contract =
            RuleProgramContractV2::from_document(document).map_err(|error| format!("{error:?}"))?;
        let input = SemanticPlanInputV2::Rule(contract);
        let plan = plan(&input, document).map_err(|error| format!("{error:?}"))?;

        assert_eq!(plan.authority_lane(), AuthorityLaneV2::FrontendChecked);
        assert_eq!(plan.effect(), EffectClassV2::ReadOnly);
        assert_eq!(
            plan.semantic_cost().estimated_cardinality_class,
            CardinalityClassV2::Closure
        );
        assert_eq!(
            plan.semantic_cost().recursion_risk_class,
            RecursionRiskClassV2::BoundedFixpoint
        );
        assert!(plan
            .capabilities()
            .contains(&CapabilityV2::RecursiveClosure));
        assert!(plan.explain().recursive());
        match plan.policy() {
            PlanPolicyV2::Rule(policy) => {
                assert_eq!(policy.item_ids().len(), fixture.item_ids().len());
                assert_eq!(policy.recursive_members().len(), 1);
                assert!(policy.stratum_count() >= 1);
            }
            PlanPolicyV2::Relational { .. }
            | PlanPolicyV2::Graph(_)
            | PlanPolicyV2::Check(_)
            | PlanPolicyV2::Presentation(_)
            | PlanPolicyV2::MetaRequireCertified
            | PlanPolicyV2::Composition(_) => {
                return Err("expected a rule-program policy".to_owned());
            }
        }
        Ok(())
    }

    /// The traversal is root-first and charges exactly one step per visited
    /// node, and a checked sort makes the result order semantic (`D26`).
    #[test]
    fn the_traversal_is_root_first_and_prices_one_step_per_node() -> Result<(), String> {
        let (fixture, input) = relational_fixture(ReferenceRelationFixtureV2::SortAsc)?;
        let document = fixture.document();
        let plan = plan(&input, document).map_err(|error| format!("{error:?}"))?;

        assert_eq!(
            plan.explain().families(),
            &[AlgebraFamilyV2::Sort, AlgebraFamilyV2::Scan]
        );
        assert_eq!(
            plan.semantic_cost().ordering_contract,
            OrderingContractV2::Ordered
        );
        assert!(plan.result().multiplicity_order().is_ordered());
        assert_eq!(
            plan.semantic_cost().estimated_cardinality_class,
            CardinalityClassV2::LinearInInput
        );
        let (nodes, _effects) = relational_policy(&plan)?;
        assert_eq!(nodes, 2);
        Ok(())
    }

    /// The composition policy survives plan assembly unchanged: the assembled
    /// plan's `policy()` IS the retained composition policy, so no stage can
    /// substitute or drop the composition identity, its ordered producers, its
    /// `WITH` binding, its body identity, or the body's own policy.
    ///
    /// This private unit rail isolates the exact assembly hop
    /// ([`SemanticPlanV2::from_parts`]) and checked policy landing
    /// ([`crate::CompositionPolicyV2`]). Resolved composition fixtures exercise
    /// public construction and [`classify`] in the owner integration suite.
    #[test]
    fn a_composition_policy_survives_plan_assembly_exactly() -> Result<(), String> {
        let policy = crate::composition::tests::fixture_policy()?;
        let parts = PlanPartsV2 {
            authority_lane: AuthorityLaneV2::FrontendChecked,
            evaluation_determinism: EvaluationDeterminismV2::Deterministic,
            evidence: EvidenceRequirementV2::None,
            effect: EffectClassV2::ReadOnly,
            result: ResultContractV2::new(
                ResultKindV2::Table,
                RenderRequirementV2::RowsOnly,
                None,
                MultiplicityOrderContractV2::from_cardinality(CardinalityV1::Set),
            ),
            capabilities: BTreeSet::new(),
            semantic_cost: SemanticCostV2 {
                estimated_cardinality_class: CardinalityClassV2::LinearInInput,
                recursion_risk_class: RecursionRiskClassV2::None,
                witness_explosion_risk_class: WitnessExplosionRiskClassV2::None,
                ordering_contract: OrderingContractV2::Unordered,
            },
            explain: ExplainNoteV2::new(policy.location().clone(), Vec::new(), false),
            policy: PlanPolicyV2::Composition(policy.clone()),
        };
        let plan = SemanticPlanV2::from_parts(parts);

        match plan.policy() {
            PlanPolicyV2::Composition(retained) => {
                assert_eq!(retained, &policy);
                assert_eq!(retained.root_item_id(), policy.root_item_id());
                assert_eq!(retained.location(), policy.location());
                assert_eq!(retained.producers(), policy.producers());
                assert_eq!(retained.with_binding(), policy.with_binding());
                assert_eq!(retained.body_item_id(), policy.body_item_id());
                assert_eq!(retained.body_family(), policy.body_family());
                assert_eq!(retained.body(), policy.body());
            }
            PlanPolicyV2::Relational { .. }
            | PlanPolicyV2::Graph(_)
            | PlanPolicyV2::Check(_)
            | PlanPolicyV2::Presentation(_)
            | PlanPolicyV2::Rule(_)
            | PlanPolicyV2::MetaRequireCertified => {
                return Err("expected a composition policy".to_owned());
            }
        }
        assert_eq!(plan.policy().keyword(), "composition");
        Ok(())
    }
}
