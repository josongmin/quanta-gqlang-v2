//! QGLang v3 logic semantics, recursion, and stratification (ticket 05).
//!
//! This crate owns the LOGIC / PREDICATE layer OVER algebra (ticket 04). It is
//! the source-level meaning of CodeQL-class existence, negation, recursion, and
//! witness/path reasoning; the algebra TARGET shapes those lower into are owned
//! by ticket 04, and planner truth/effect policy is downstream in ticket 07.
//! Cites packet decisions:
//!
//! - `D12`: ticket ownership is strict — `04` owns the algebra nodes
//!   (`SemiJoin`/`AntiJoin`), `05` (here) owns the source-level meaning and
//!   legality that lowers into them.
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
//! - [`ExistenceCheckV1`] / [`ExistencePolarityV1`]: source `exists` /
//!   `not exists` semantics that lower to algebra `SemiJoin` / `AntiJoin`.
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
pub use diagnostic::{LogicDiagnosticV1, NonMonotoneReasonV1, UnboundedWitnessReasonV1};
pub use graph::{GraphEndpointRoleV1, GraphQueryContractV1, GraphQuerySemanticsV1};
pub use recursion::{
    BodyLiteralV1, BodyPolarityV1, RecursiveProgramV1, RuleDefV1, StratificationProofV1,
};
pub use source::{ExistenceCheckV1, ExistencePolarityV1};
pub use witness::{
    DedupIdentityV1, RelationShapeV1, WitnessContractV1, WitnessRequestV1, WitnessTieBreakV1,
};
