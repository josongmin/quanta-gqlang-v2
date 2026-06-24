//! QGLang v3 relational algebra and normalization (ticket 04).
//!
//! Owns the explicit algebra node family (Scan/Project/Filter/Join/Apply/Group/
//! Aggregate/Sort/Limit/Union/SemiJoin/AntiJoin), normalization from shared HIR
//! query shapes into algebra, set/bag/cardinality semantics, column provenance,
//! and correlated/lateral subquery normalization. Cites packet decisions:
//!
//! - `D3`: normalization runs AFTER binding; it consumes binder-derived
//!   alias/correlation visibility rather than re-resolving names.
//! - `D5`: query normalization targets relational algebra, not a root-local IR.
//!   [`AlgebraNodeV1`] is the normalized target shape.
//! - `D23`: correlated subqueries are EXPLICIT first-class algebra. A correlated
//!   subquery normalizes into an [`AlgebraOpV1::Apply`] with VISIBLE
//!   [`CorrelationColumnV1`]s; no root-local side channel, no planner heuristic.
//! - `D26`: multiplicity / duplicate-elimination / output order are explicit and
//!   testable. Every node carries an explicit [`quanta_gqlang_v2_types::CardinalityV1`];
//!   union/aggregate duplicate behaviour is an explicit flag, never a default.
//! - `D10`: unsupported / ill-formed algebra states reject with a typed
//!   [`AlgebraDiagnosticV1`]; there is no silent downgrade.
//!
//! # Out of lane
//!
//! Source-level `exists` / recursion legality is ticket 05; planner
//! capability/truth/effect derivation is ticket 07. This crate only owns the
//! algebra TARGET shape those lower into / consume.
//!
//! # Entry points
//!
//! [`normalize_query`] lowers a [`quanta_gqlang_v2_hir::QueryHirV1`] body;
//! [`normalize_relation`] lowers any [`quanta_gqlang_v2_hir::RelationExprHirV1`].
//! Both take a [`NormalizeOptionsV1`] carrying the aggregate-callee catalog
//! (which cannot be recovered from structure-only HIR) and return a
//! [`NormalizeResultV1`].

mod diagnostic;
mod node;
mod normalize;

pub use diagnostic::{AlgebraDiagnosticV1, IllFormedAggregateReasonV1};
pub use node::{
    AggregateExprV1, AggregateV1, AlgebraFamilyV1, AlgebraNodeV1, AlgebraOpV1, AntiJoinV1, ApplyV1,
    ColumnProvenanceV1, ColumnV1, CorrelationColumnV1, FilterV1, GroupV1, JoinV1, LimitV1,
    ProjectV1, ScanV1, SemiJoinV1, SortDirectionV1, SortKeyV1, SortV1, UnionV1,
};
pub use normalize::{normalize_query, normalize_relation, NormalizeOptionsV1, NormalizeResultV1};
