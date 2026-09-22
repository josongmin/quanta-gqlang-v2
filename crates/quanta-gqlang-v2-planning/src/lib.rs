//! QGLang v3 planning, truth, evidence, and effects (ticket 07).
//!
//! Owns the host-agnostic planning IR over normalized algebra/logic: capability
//! derivation, truth lanes, evidence requirements, the effect model, result-kind
//! derivation (table/alert/path-witness/proof-explain), the fixed host-agnostic
//! semantic-cost schema, witness-selection policy, and multiplicity/order
//! contracts. Cites packet decisions:
//!
//! - `D6`: planning attaches to the NORMALIZED query/program shape, not the root
//!   kind. [`produce_plan`] keys every classification off the algebra/logic
//!   shape supplied in a [`PlanRequestV1`]; it never re-inspects syntax/root
//!   labels (ticket 07 non-goal 3).
//! - `D14`/`D7`: outputs are host-agnostic. No runtime/provider/backend noun
//!   appears in any public type; mutation is not a frontend effect — the
//!   strongest effect is [`EffectClassV1::ExportOnly`] and
//!   [`EffectClassV1::MutationForbidden`] is the explicit reject marker.
//! - `D20`: result kinds, source locations, and witness/path payloads are
//!   SEMANTIC outputs ([`ResultKindV1`] / [`RenderRequirementV1`] /
//!   [`EvidenceRequirementV1`]), not CLI-only presentation.
//! - `D24`: logical explain/provenance ([`ExplainNoteV1`]) and the semantic-cost
//!   signals are frontend-owned; runtime physical plans stay downstream.
//! - `D26`: multiplicity/duplicate/order is explicit
//!   ([`MultiplicityOrderContractV1`]) and an unordered result stays unordered.
//! - `D32`: the semantic-cost output uses a FIXED schema with stable field names
//!   ([`SemanticCostV1`] / [`SemanticCostV1::FIELD_NAMES`]).
//! - `D10`/`D16`: an unsupported construct MIX fails closed with a typed
//!   [`PlanningDiagnosticV1`] carrying a stable `unsupported_reason_code`; there
//!   is no lossy downgrade.
//!
//! # Entry point
//!
//! [`produce_plan`] consumes a [`PlanRequestV1`] (the normalized algebra root,
//! optional recursion/witness shapes, and the requested result-kind/effect/truth
//! lane) and returns a [`PublicPlanV1`] or a [`PlanningDiagnosticV1`].

mod capability;
mod check;
mod composition;
mod cost;
mod diagnostic;
mod effect;
mod explain;
mod item;
mod plan;
mod policy;
mod presentation;
mod result_kind;

pub use capability::CapabilityV2;
pub use check::{CheckContractV2, CheckPolicyV2};
pub use composition::{
    CheckedCompositionPlanInputV2, CheckedCompositionProducerV2, CompositionBodyPlanInputV2,
};
pub use composition::{CompositionBodyPolicyV2, CompositionPolicyV2};
pub use cost::{
    CardinalityClassV2, OrderingContractV2, RecursionRiskClassV2, SemanticCostV2,
    WitnessExplosionRiskClassV2,
};
pub use diagnostic::{
    CheckInvalidReasonV2, CompositionInvalidReasonV2, InputDocumentMismatchReasonV2,
    PlanningDiagnosticV1, PresentationInvalidReasonV2,
};
pub use effect::{AuthorityLaneV2, EffectClassV2, EvaluationDeterminismV2, EvidenceRequirementV2};
pub use explain::ExplainNoteV2;
pub use item::{CatalogDeclarationKindV2, NoPlanReasonV2, SemanticPlanInputV2};
pub use plan::{plan, plan_outcome, PlanOutcomeV2, SemanticPlanV2};
pub use policy::{EffectSummaryV2, GraphWitnessRetentionV2, PlanPolicyV2, RuleProgramPolicyV2};
pub use presentation::PresentationContractV2;
pub use result_kind::{
    MultiplicityOrderContractV2, RenderRequirementV2, ResultContractV2, ResultKindV2,
    WitnessSelectionPolicyV2,
};
