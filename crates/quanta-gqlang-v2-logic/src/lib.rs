//! QGLang v3 logic semantics, recursion, and stratification (ticket 05).
//!
//! This crate owns rule, recursion, graph-query, and witness semantics over the
//! sealed frontend document. Relational existence is owned by semantic analysis
//! and lowered by algebra; planner truth/effect policy is downstream.
//! Cites packet decisions:
//!
//! - `D12`: ticket ownership is strict; this crate does not manufacture
//!   relational algebra nodes.
//! - `D27`: positive recursion uses EXPLICIT least-fixpoint semantics; recursive
//!   aggregates and unbounded witness/path materialization REJECT unless a
//!   legality rule explicitly admits them.
//! - `D31`: recursive witness/path materialization requires explicit
//!   boundedness, cycle-dedup identity, and tie-break rules; otherwise the shape
//!   rejects.
//! - `D10`: illegal recursion / negation cycles / unbounded witnesses reject
//!   with typed [`LogicDiagnosticV1`]; there is no silent downgrade.
//!
//! # What lives here
//!
//! - [`RecursiveProgramV1`] / [`RuleDefV1`] / [`BodyLiteralV1`]: recursive
//!   closure over the shared [`quanta_gqlang_v2_hir::RuleHirV1`] carrier, with
//!   [`RecursiveProgramV1::stratify`] producing a [`StratificationProofV1`] for
//!   admitted positive recursion or a typed rejection for negation cycles.
//! - [`RecursiveAggregateMatrixV1`]: the explicit admit/reject matrix for
//!   recursive aggregates.
//! - [`WitnessContractV1`] / [`WitnessRequestV1`]: the bounded witness/path
//!   materialization contract (dedup identity, max count, path-length bound,
//!   canonical tie-break).
//! - [`GraphQuerySemanticsV1`] / [`GraphQueryContractV1`]: FIND graph-query
//!   source semantics over [`quanta_gqlang_v2_hir::GraphQueryHirV1`] —
//!   per-subkind endpoint-presence legality plus the bounded-witness contract,
//!   classifying (not executing) the FIND family.
//! - [`LogicDiagnosticV1`]: the closed, typed diagnostic family.

mod aggregate;
mod diagnostic;
mod graph;
mod recursion;
mod source;
mod witness;

pub use aggregate::{AggregateLegalityV1, RecursiveAggregateMatrixV1, RecursiveAggregateV1};
pub use diagnostic::{
    LogicDiagnosticV1, RuleGraphBudgetDimensionV1, RuleProgramInvalidReasonV1,
    UnboundedWitnessReasonV1,
};
pub use graph::{GraphEndpointRoleV1, GraphQueryContractV1, GraphQuerySemanticsV1};
pub use recursion::{
    BodyLiteralV1, BodyPolarityV1, RecursiveProgramV1, RuleDefV1, StratificationProofV1,
};
pub use witness::{
    DedupIdentityV1, RelationShapeV1, WitnessContractV1, WitnessRequestV1, WitnessTieBreakV1,
};
