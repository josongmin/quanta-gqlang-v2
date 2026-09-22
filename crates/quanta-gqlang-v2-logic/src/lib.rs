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
//! - [`RuleProgramContractV2`]: the sole document ingress; it enumerates and
//!   validates the complete rule set of a sealed semantic document.
//! - [`RuleDefV1`] / [`BodyLiteralV1`] / [`RecursiveProgramV1`]: stable-ID rule
//!   carriers with the free function [`stratify`] producing a
//!   [`StratificationProofV1`] for admitted positive recursion or a typed
//!   rejection for negation cycles.
//! - [`RejectAllRecursiveAggregatesV2`]: the fixed reject-all policy for
//!   recursive aggregates (Sema owns nonrecursive aggregate legality).
//! - [`WitnessContractV2`]: the bounded witness/path
//!   materialization contract (dedup identity, max count, path-length bound,
//!   canonical tie-break).
//! - [`GraphQueryContractV2`]: FIND graph-query semantics over the resolved
//!   document query — per-subkind endpoint-presence legality plus the
//!   bounded-witness contract, classifying (not executing) the FIND family.
//! - [`LogicDiagnosticV1`]: the closed, typed diagnostic family.

mod aggregate;
mod diagnostic;
mod graph;
mod recursion;
mod source;
mod witness;

pub use aggregate::RejectAllRecursiveAggregatesV2;
pub use diagnostic::{
    LogicDiagnosticV1, RuleGraphBudgetDimensionV1, RuleProgramInvalidReasonV1,
    UnboundedWitnessReasonV1,
};
pub use graph::{GraphEndpointRoleV1, GraphQueryContractV2};
pub use recursion::{
    stratify, BodyLiteralV1, BodyPolarityV1, RecursiveProgramV1, RuleDefV1, StratificationProofV1,
};
pub use source::RuleProgramContractV2;
pub use witness::{DedupIdentityV1, RelationShapeV1, WitnessContractV2, WitnessTieBreakV1};
