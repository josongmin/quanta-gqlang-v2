//! The one fixed policy for recursive aggregates: every spelling rejects.
//!
//! `SS-06B` step 6 replaces the V1 name-keyed admit/reject matrix
//! (`AggregateLegalityV1`, `RecursiveAggregateV1`, `RecursiveAggregateMatrixV1`,
//! `with_admitted`, `admits`, `classify`) with a single policy carrier. This
//! packet has no configurable recursive-aggregate exception: a spelling is never
//! a legality proof, and the V1 default that admitted `min`/`max` over
//! recursion is dead.
//!
//! Nonrecursive aggregate legality remains the semantic contract's decision.
//! Logic only ever consults this policy for an aggregate reference whose target
//! sits in the same recursive SCC, so a nonrecursive aggregate never gains a
//! second Logic-side legality decision.

use quanta_gqlang_v2_sema::CheckedLocatedSpanV2;
use quanta_gqlang_v2_types::ResolvedPredicateIdV2;

use crate::diagnostic::LogicDiagnosticV1;

/// The fixed reject-all policy for recursive aggregates.
///
/// The carrier is deliberately not configurable: there is no admit list, no
/// extension point, and no spelling-dependent branch. [`Self::check`] rejects
/// every recursive aggregate with [`LogicDiagnosticV1::RecursiveAggregate`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RejectAllRecursiveAggregatesV2;

impl RejectAllRecursiveAggregatesV2 {
    /// Stable rendering used when an aggregated reference was projected without
    /// its aggregate-function spelling. The exact code, predicate identity, and
    /// checked location remain the contract; only the human spelling is
    /// unavailable, and it is never inferred from a name.
    pub const UNSPECIFIED_FUNCTION: &'static str = "<aggregate>";

    /// The one recursive-aggregate policy.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Reject one recursive aggregate at its exact checked location.
    ///
    /// Always returns [`Err`]: there is no admitted spelling of a recursive
    /// aggregate in this packet.
    ///
    /// # Errors
    ///
    /// Always returns [`LogicDiagnosticV1::RecursiveAggregate`] carrying the
    /// aggregate function spelling, the folded recursive predicate identity, and
    /// the offending aggregate call's checked location.
    pub fn check(
        &self,
        function: &str,
        predicate: &ResolvedPredicateIdV2,
        span: &CheckedLocatedSpanV2,
    ) -> Result<(), LogicDiagnosticV1> {
        Err(LogicDiagnosticV1::RecursiveAggregate {
            function: function.to_owned(),
            predicate: predicate.clone(),
            span: span.clone(),
        })
    }
}
