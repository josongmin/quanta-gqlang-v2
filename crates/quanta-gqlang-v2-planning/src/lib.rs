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
mod cost;
mod diagnostic;
mod effect;
mod explain;
mod item;
mod plan;
mod result_kind;

pub use capability::{derive_algebra_capabilities, CapabilityV1};
pub use cost::{
    CardinalityClassV1, OrderingContractV1, RecursionRiskClassV1, SemanticCostV1,
    WitnessExplosionRiskClassV1,
};
pub use diagnostic::{PlanningDiagnosticV1, UnsupportedMixReasonV1};
pub use effect::{EffectClassV1, EvidenceRequirementV1, TruthLaneV1};
pub use explain::ExplainNoteV1;
pub use item::{classify_item, ItemClassificationV1, NoPlanReasonV1};
pub use plan::{
    produce_plan, PlanRequestV1, PublicPlanV1, RecursionShapeV1, RequestedResultKindV1,
    WitnessShapeV1,
};
pub use result_kind::{
    MultiplicityOrderContractV1, RenderRequirementV1, ResultKindV1, WitnessSelectionPolicyV1,
};
