//! Per-family ITEM classification (ticket 07, FRONTEND-CONTRACT parity).
//!
//! [`classify_item`] is the single entry point that takes ONE parsed program
//! item ([`ProgramItemHirV1`]) and classifies it into the existing host-agnostic
//! planning vocabulary: a [`PublicPlanV1`] (result-kind / effect / capabilities /
//! truth lane / witness selection / fixed cost schema), a typed NO-PLAN marker
//! for items that register but do not plan, or a fail-closed
//! [`PlanningDiagnosticV1`].
//!
//! This is the SEAM that brings every non-relational v2 root family up to the
//! same one pipeline as SELECT, without executing anything (no taint traversal,
//! no SARIF bytes — that stays downstream, `D7`/`D14`):
//!
//! - FIND -> `path_witness` / `proof_evidence_only`, witness selection and
//!   witness-explosion risk taken from logic's [`GraphQueryContractV1`]; illegal
//!   FIND shapes fail closed with logic's diagnostic relayed unchanged.
//! - CHECK -> `alert` / `read_only`. EXPLAIN -> `proof_explain` / `read_only`.
//!   SHOW / SAMPLE -> `table` / `read_only`. DESCRIBE -> `alert`. COMPARE ->
//!   `path_witness` (a delta). EXPORT -> `export_only`.
//! - REQUIRE CERTIFIED -> a proof-confirmed truth gate. USE PACK -> no plan.
//! - DECLARATIONS (VIEW / MODEL / PACK / PROFILE / GATE / RULE) -> no plan
//!   (they register in the catalog; that is modules' concern, not planning's).
//! - A relational `Query` / `Decl` / `Rule` carrier is DEFERRED to the algebra
//!   pipeline ([`produce_plan`]): its fixed cost schema needs the NORMALIZED
//!   algebra shape (`D6`), which item classification deliberately does not invent.

use std::collections::BTreeSet;

use quanta_gqlang_v2_hir::{
    CompositionBodyV1, GraphQueryHirV1, LegalityHirV1, MetaHirV1, MetaKindV1, PresentationHirV1,
    PresentationKindV1, ProgramItemHirV1, SpanV1,
};
use quanta_gqlang_v2_logic::{GraphQueryContractV1, GraphQuerySemanticsV1, RelationShapeV1};
use quanta_gqlang_v2_types::CardinalityV1;

use crate::capability::CapabilityV1;
use crate::cost::{
    CardinalityClassV1, OrderingContractV1, RecursionRiskClassV1, SemanticCostV1,
    WitnessExplosionRiskClassV1,
};
use crate::diagnostic::PlanningDiagnosticV1;
use crate::effect::{EffectClassV1, EvidenceRequirementV1, TruthLaneV1};
use crate::explain::ExplainNoteV1;
use crate::plan::PublicPlanV1;
use crate::result_kind::{
    MultiplicityOrderContractV1, RenderRequirementV1, ResultKindV1, WitnessSelectionPolicyV1,
};

/// Why a program item produces NO host-agnostic plan. This is a TYPED marker,
/// not an error: the item is well-formed, it simply does not lower to a
/// servable plan at this seam.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoPlanReasonV1 {
    /// A catalog declaration (VIEW / MODEL / PACK / PROFILE / GATE / RULE): it
    /// registers in the catalog and produces no plan. Registration / duplicate
    /// rejection / admission gating are modules' concern.
    CatalogDeclaration,
    /// A meta activation directive (USE PACK): effect-style pack activation with
    /// no relational plan.
    MetaActivation,
    /// A relational carrier (SELECT `Query`, or a `Decl` / `Rule` body) whose
    /// plan must be produced through the algebra pipeline ([`crate::produce_plan`])
    /// from its NORMALIZED shape; item classification does not invent the algebra.
    RelationalDeferred,
}

impl NoPlanReasonV1 {
    /// Stable keyword for explain / diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::CatalogDeclaration => "catalog_declaration",
            Self::MetaActivation => "meta_activation",
            Self::RelationalDeferred => "relational_deferred",
        }
    }
}

/// The outcome of classifying a single program item.
#[derive(Clone, Debug, PartialEq)]
pub enum ItemClassificationV1 {
    /// The item lowers to a host-agnostic public plan.
    Plan(PublicPlanV1),
    /// The item is well-formed but produces no plan at this seam.
    NoPlan(NoPlanReasonV1),
    /// The item is rejected fail-closed with a typed planning diagnostic.
    Reject(PlanningDiagnosticV1),
}

/// Classify one program item into the host-agnostic planning vocabulary.
///
/// Never panics and never executes; FIND legality is delegated to logic and its
/// rejection relayed unchanged (`D10`/`D12`).
#[must_use]
pub fn classify_item(item: &ProgramItemHirV1) -> ItemClassificationV1 {
    match item {
        // Relational carriers go through the algebra pipeline.
        ProgramItemHirV1::Query(_) | ProgramItemHirV1::Decl(_) | ProgramItemHirV1::Rule(_) => {
            ItemClassificationV1::NoPlan(NoPlanReasonV1::RelationalDeferred)
        }
        ProgramItemHirV1::GraphQuery(find) => classify_find(find),
        ProgramItemHirV1::Legality(check) => classify_check(check),
        ProgramItemHirV1::Presentation(p) => classify_presentation(p),
        // Declarations register in the catalog and produce no plan.
        ProgramItemHirV1::CatalogDecl(_) => {
            ItemClassificationV1::NoPlan(NoPlanReasonV1::CatalogDeclaration)
        }
        // A composition's servability is its BODY's: classify the body. Binding /
        // producer legality is the binder's / logic's concern, not planning's.
        ProgramItemHirV1::Composition(c) => classify_composition_body(&c.body),
        ProgramItemHirV1::Meta(m) => classify_meta(m),
    }
}

fn classify_composition_body(body: &CompositionBodyV1) -> ItemClassificationV1 {
    match body {
        CompositionBodyV1::Query(_) => {
            ItemClassificationV1::NoPlan(NoPlanReasonV1::RelationalDeferred)
        }
        CompositionBodyV1::GraphQuery(find) => classify_find(find),
        CompositionBodyV1::Legality(check) => classify_check(check),
        CompositionBodyV1::Explain(explain) => {
            ItemClassificationV1::Plan(proof_explain_plan(explain.span))
        }
    }
}

/// FIND -> `path_witness` / `proof_evidence_only`, witness selection + explosion
/// risk from logic's bounded contract. An illegal FIND fails closed with logic's
/// diagnostic relayed unchanged.
fn classify_find(find: &GraphQueryHirV1) -> ItemClassificationV1 {
    let contract = match GraphQuerySemanticsV1::classify(find) {
        Ok(contract) => contract,
        Err(logic) => {
            return ItemClassificationV1::Reject(PlanningDiagnosticV1::IllegalGraphQuery { logic });
        }
    };

    // FIND always materializes the full bounded witness set under the contract's
    // tie-break; the selection is EXPLICIT rather than backend-accidental.
    let selection = WitnessSelectionPolicyV1::AllWitness;
    let witness_explosion = witness_risk_from_contract(&contract, selection);
    // Reachability/flow/slice/taint/path/summary traverse a relation that can
    // cycle, i.e. a bounded recursive closure; points_to is acyclic.
    let recursive = contract.witness.shape == RelationShapeV1::Cyclic;

    let mut capabilities = BTreeSet::new();
    capabilities.insert(CapabilityV1::WitnessMaterialization);
    if recursive {
        capabilities.insert(CapabilityV1::RecursiveClosure);
    }

    let recursion_risk = if recursive {
        // Termination depends on the explicit path-length bound in the contract.
        RecursionRiskClassV1::BoundDependent
    } else {
        RecursionRiskClassV1::None
    };
    let cardinality_class = if recursive {
        CardinalityClassV1::Closure
    } else {
        CardinalityClassV1::LinearInInput
    };

    let cost = SemanticCostV1 {
        estimated_cardinality_class: cardinality_class,
        recursion_risk_class: recursion_risk,
        witness_explosion_risk_class: witness_explosion,
        // A witness/path result is a bag of evidence, not an ordered table.
        ordering_contract: OrderingContractV1::Unordered,
        unsupported_reason_code: None,
    };

    let plan = PublicPlanV1 {
        capabilities,
        truth_lane: TruthLaneV1::Deterministic,
        evidence: EvidenceRequirementV1::WitnessPayload,
        effect: EffectClassV1::ProofEvidenceOnly,
        result_kind: ResultKindV1::PathWitness,
        render_requirement: RenderRequirementV1::WitnessPayload,
        witness_selection: Some(selection),
        multiplicity_order: MultiplicityOrderContractV1::from_cardinality(CardinalityV1::Set),
        cost,
        explain: ExplainNoteV1::for_item(find.span, recursive),
    };
    ItemClassificationV1::Plan(plan)
}

/// Map a bounded witness contract + selection policy onto the witness-explosion
/// risk class, mirroring [`crate::plan`]'s relational mapping so FIND and a
/// witness-bearing SELECT classify a cyclic relation identically.
fn witness_risk_from_contract(
    contract: &GraphQueryContractV1,
    selection: WitnessSelectionPolicyV1,
) -> WitnessExplosionRiskClassV1 {
    if contract.witness.shape == RelationShapeV1::Cyclic {
        WitnessExplosionRiskClassV1::CapBounded
    } else {
        match selection {
            WitnessSelectionPolicyV1::CanonicalOne => WitnessExplosionRiskClassV1::SingleWitness,
            WitnessSelectionPolicyV1::TopK { .. } | WitnessSelectionPolicyV1::AllWitness => {
                WitnessExplosionRiskClassV1::BoundedSet
            }
        }
    }
}

/// CHECK -> `alert` / `read_only`. The check evaluates a rule reference and emits
/// findings/SARIF downstream; here it is read-only alert classification.
fn classify_check(check: &LegalityHirV1) -> ItemClassificationV1 {
    ItemClassificationV1::Plan(alert_plan(check.span))
}

/// Presentation verbs map onto distinct result-kinds / effects.
fn classify_presentation(p: &PresentationHirV1) -> ItemClassificationV1 {
    let plan = match &p.kind {
        // EXPLAIN: rule/path provenance without execution.
        PresentationKindV1::Explain { .. } => proof_explain_plan(p.span),
        // SHOW / SAMPLE: catalog introspection / retrieval -> a read-only table.
        PresentationKindV1::Show { .. } | PresentationKindV1::Sample { .. } => {
            table_plan(p.span, EffectClassV1::ReadOnly)
        }
        // DESCRIBE: entity metadata -> alert.
        PresentationKindV1::Describe { .. } => alert_plan(p.span),
        // COMPARE: a baseline/candidate delta -> a path/witness result.
        PresentationKindV1::Compare { .. } => compare_plan(p.span),
        // EXPORT: a bridge receipt -> export-only effect, no mutation.
        PresentationKindV1::Export { .. } => table_plan(p.span, EffectClassV1::ExportOnly),
    };
    ItemClassificationV1::Plan(plan)
}

/// META: REQUIRE CERTIFIED is a proof-confirmed truth gate; USE PACK is an
/// effect-style activation with no plan.
fn classify_meta(meta: &MetaHirV1) -> ItemClassificationV1 {
    match meta.kind {
        MetaKindV1::UsePack { .. } => ItemClassificationV1::NoPlan(NoPlanReasonV1::MetaActivation),
        MetaKindV1::RequireCertified => {
            ItemClassificationV1::Plan(require_certified_plan(meta.span))
        }
    }
}

// ---- shared plan shapes ----------------------------------------------------

/// A read-only `alert` plan (CHECK / DESCRIBE): rows must carry a source
/// location.
fn alert_plan(span: SpanV1) -> PublicPlanV1 {
    PublicPlanV1 {
        capabilities: BTreeSet::new(),
        truth_lane: TruthLaneV1::Deterministic,
        evidence: EvidenceRequirementV1::SourceLocation,
        effect: EffectClassV1::ReadOnly,
        result_kind: ResultKindV1::Alert,
        render_requirement: RenderRequirementV1::SourceLocation,
        witness_selection: None,
        multiplicity_order: MultiplicityOrderContractV1::from_cardinality(CardinalityV1::Set),
        cost: read_only_cost(),
        explain: ExplainNoteV1::for_item(span, false),
    }
}

/// A `proof_explain` plan (EXPLAIN): provenance, not rows.
fn proof_explain_plan(span: SpanV1) -> PublicPlanV1 {
    PublicPlanV1 {
        capabilities: BTreeSet::new(),
        truth_lane: TruthLaneV1::Deterministic,
        evidence: EvidenceRequirementV1::ProofTree,
        effect: EffectClassV1::ReadOnly,
        result_kind: ResultKindV1::ProofExplain,
        render_requirement: RenderRequirementV1::ProvenanceTree,
        witness_selection: None,
        multiplicity_order: MultiplicityOrderContractV1::from_cardinality(CardinalityV1::Set),
        cost: read_only_cost(),
        explain: ExplainNoteV1::for_item(span, false),
    }
}

/// A `table` plan (SHOW / SAMPLE / EXPORT) with a caller-chosen effect.
fn table_plan(span: SpanV1, effect: EffectClassV1) -> PublicPlanV1 {
    PublicPlanV1 {
        capabilities: BTreeSet::new(),
        truth_lane: TruthLaneV1::Deterministic,
        evidence: EvidenceRequirementV1::None,
        effect,
        result_kind: ResultKindV1::Table,
        render_requirement: RenderRequirementV1::RowsOnly,
        witness_selection: None,
        multiplicity_order: MultiplicityOrderContractV1::from_cardinality(CardinalityV1::Set),
        cost: read_only_cost(),
        explain: ExplainNoteV1::for_item(span, false),
    }
}

/// A `path_witness` plan for COMPARE: a baseline/candidate delta carries
/// witness-style evidence but enumerates no recursive closure on its own.
fn compare_plan(span: SpanV1) -> PublicPlanV1 {
    PublicPlanV1 {
        capabilities: BTreeSet::new(),
        truth_lane: TruthLaneV1::Deterministic,
        evidence: EvidenceRequirementV1::WitnessPayload,
        effect: EffectClassV1::ReadOnly,
        result_kind: ResultKindV1::PathWitness,
        render_requirement: RenderRequirementV1::WitnessPayload,
        witness_selection: None,
        multiplicity_order: MultiplicityOrderContractV1::from_cardinality(CardinalityV1::Set),
        cost: read_only_cost(),
        explain: ExplainNoteV1::for_item(span, false),
    }
}

/// REQUIRE CERTIFIED -> a proof-confirmed truth gate: the strongest lane, which
/// forces a proof-tree evidence requirement (only this lane carries final policy
/// truth). It is evidence-only — a gate certifies, it does not export.
fn require_certified_plan(span: SpanV1) -> PublicPlanV1 {
    PublicPlanV1 {
        capabilities: BTreeSet::new(),
        truth_lane: TruthLaneV1::ProofConfirmed,
        evidence: EvidenceRequirementV1::ProofTree,
        effect: EffectClassV1::ProofEvidenceOnly,
        result_kind: ResultKindV1::ProofExplain,
        render_requirement: RenderRequirementV1::ProvenanceTree,
        witness_selection: None,
        multiplicity_order: MultiplicityOrderContractV1::from_cardinality(CardinalityV1::Set),
        cost: read_only_cost(),
        explain: ExplainNoteV1::for_item(span, false),
    }
}

/// The fixed cost schema for a non-relational, non-recursive, set-shaped item:
/// linear-in-input, no recursion, no witness enumeration, unordered, servable.
fn read_only_cost() -> SemanticCostV1 {
    SemanticCostV1 {
        estimated_cardinality_class: CardinalityClassV1::LinearInInput,
        recursion_risk_class: RecursionRiskClassV1::None,
        witness_explosion_risk_class: WitnessExplosionRiskClassV1::None,
        ordering_contract: OrderingContractV1::Unordered,
        unsupported_reason_code: None,
    }
}
