//! Typed, fail-closed diagnostics for the logic / recursion layer (ticket 05).
//!
//! Per packet decisions `D10` (no silent downgrade), `D27` (positive recursion
//! uses explicit least-fixpoint; recursive aggregates / unbounded witness reject
//! unless explicitly admitted) and `D31` (recursive witness/path materialization
//! requires explicit boundedness, cycle-dedup identity, and tie-break rules,
//! otherwise the shape rejects), every illegal logic shape emits one of these
//! typed diagnostics rather than degrading silently.
//!
//! Each variant renders a STABLE `code()` (dot-namespaced under `logic.`) and a
//! stable human `message()` so CLI/LSP/tests assert exact text (`D22`). The
//! `span()` points back at the originating HIR construct (`D20`).

use quanta_gqlang_v2_hir::{GraphQueryKindV1, SpanV1};

use crate::graph::GraphEndpointRoleV1;

/// A typed logic-semantics diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogicDiagnosticV1 {
    /// A source-level `exists` / `not exists` quantifier carried an unknown
    /// keyword that has no logic lowering (only `exists` / `not exists` /
    /// `forall` are recognized source quantifiers here).
    UnknownQuantifier {
        /// The quantifier keyword as written (e.g. `"some"`).
        keyword: String,
        /// Span of the offending quantifier.
        span: SpanV1,
    },
    /// A source horn rule head was not expressible as a predicate-rooted head
    /// under the current frontend contract. The public lowering accepts only a
    /// bare symbol or a path rooted at a symbol, so arbitrary scalar heads
    /// reject explicitly rather than pretending to be recursive predicates.
    UnsupportedRuleHead {
        /// Span of the offending rule head.
        span: SpanV1,
    },
    /// A recursion went through negation in a way that cannot be stratified:
    /// a predicate depends, transitively and through at least one negated
    /// edge, on itself. Stratified negation requires every negated dependency
    /// to sit in a strictly lower stratum, which a negation cycle makes
    /// impossible (`D27`).
    NegationCycle {
        /// The predicate names participating in the unstratifiable cycle, in
        /// deterministic (sorted) order for stable diagnostics.
        cycle: Vec<String>,
        /// Span of the rule whose body closed the negation cycle.
        span: SpanV1,
    },
    /// An aggregate was applied over a predicate that recurses (transitively)
    /// through itself, and the aggregate is NOT on the explicit admit list.
    /// Recursive aggregates have no monotone least-fixpoint meaning in general,
    /// so they reject unless this ticket's matrix admits the specific shape
    /// (`D27`).
    RecursiveAggregate {
        /// The aggregate function name as written (e.g. `sum`, `count`).
        function: String,
        /// The recursive predicate the aggregate folded over.
        predicate: String,
        /// Span of the offending aggregate call.
        span: SpanV1,
    },
    /// A witness / path materialization over a recursive relation was requested
    /// without a complete boundedness contract (a finite max-witness cap is
    /// mandatory; over a relation that can cycle a path-length bound is also
    /// mandatory). Unbounded witness families reject rather than streaming
    /// infinitely (`D31`).
    UnboundedWitness {
        /// Why the witness contract is not bounded.
        reason: UnboundedWitnessReasonV1,
        /// Span of the offending witness/path request.
        span: SpanV1,
    },
    /// A FIND graph query of a given subkind was missing an endpoint that the
    /// subkind REQUIRES (e.g. `taint` without a `sink`). Endpoint presence is a
    /// frontend legality this layer owns; a missing required endpoint fails
    /// closed rather than silently traversing an open graph (`D10`).
    GraphEndpointMissing {
        /// The FIND subkind that imposes the requirement.
        kind: GraphQueryKindV1,
        /// The missing endpoint role.
        role: GraphEndpointRoleV1,
        /// Span of the offending graph query.
        span: SpanV1,
    },
    /// A FIND graph query supplied an endpoint its subkind does NOT accept
    /// (e.g. a `slice` carrying a `source`/`sink` when it only anchors). A
    /// conflicting endpoint fails closed rather than silently ignoring it
    /// (`D10`), so the seed cannot mean two different traversals at once.
    GraphEndpointUnexpected {
        /// The FIND subkind that forbids the endpoint.
        kind: GraphQueryKindV1,
        /// The unexpected endpoint role that was present.
        role: GraphEndpointRoleV1,
        /// Span of the offending graph query.
        span: SpanV1,
    },
    /// A FIND graph query over a CYCLIC relation supplied BOTH an explicit
    /// `depth` bound AND bounding waypoints (`via` / `through`). These are two
    /// competing path-length binding strategies; silently letting `depth` win
    /// would drop the waypoint constraint (a silent downgrade, `D10`) and accept
    /// a seed that means two different things at once (`D26`). The query fails
    /// closed so the user explicitly picks ONE binding strategy.
    GraphConflictingPathBound {
        /// The FIND subkind that carried the conflicting bounds.
        kind: GraphQueryKindV1,
        /// Span of the offending graph query.
        span: SpanV1,
    },
    /// A least-fixpoint was requested over a rule body that is not monotone
    /// (it negates or aggregates the recursive predicate inside its own
    /// recursion), so the fixpoint is not guaranteed to exist / be unique.
    NonMonotoneRecursion {
        /// The recursive predicate whose defining rule is non-monotone.
        predicate: String,
        /// Why monotonicity fails.
        reason: NonMonotoneReasonV1,
        /// Span of the offending rule.
        span: SpanV1,
    },
}

/// Why a witness/path contract is not bounded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnboundedWitnessReasonV1 {
    /// No finite maximum-witness cap was supplied.
    MissingMaxWitnesses,
    /// The underlying relation can cycle but no path-length bound was supplied,
    /// so enumeration could revisit nodes forever.
    MissingPathLengthBound,
    /// A max-witness cap of zero was supplied, which can never materialize a
    /// witness and is therefore an ill-formed (degenerate) bound.
    ZeroMaxWitnesses,
    /// A path-length bound of zero was supplied, which can never traverse an
    /// edge (so it cannot connect two distinct endpoints) and is therefore an
    /// ill-formed (degenerate) bound, just like a zero max-witness cap.
    ZeroPathLengthBound,
}

impl UnboundedWitnessReasonV1 {
    /// Stable code suffix for this reason.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingMaxWitnesses => "missing_max_witnesses",
            Self::MissingPathLengthBound => "missing_path_length_bound",
            Self::ZeroMaxWitnesses => "zero_max_witnesses",
            Self::ZeroPathLengthBound => "zero_path_length_bound",
        }
    }
}

/// Why a recursive rule fails monotonicity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NonMonotoneReasonV1 {
    /// The rule negates the recursive predicate inside its own recursion.
    NegatedRecursion,
    /// The rule aggregates the recursive predicate inside its own recursion.
    AggregatedRecursion,
}

impl NonMonotoneReasonV1 {
    /// Stable code suffix for this reason.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NegatedRecursion => "negated_recursion",
            Self::AggregatedRecursion => "aggregated_recursion",
        }
    }
}

impl LogicDiagnosticV1 {
    /// The stable diagnostic code, shared by CLI/LSP/tests. Codes are
    /// dot-namespaced under `logic.` and never change spelling.
    #[must_use]
    pub fn code(&self) -> String {
        match self {
            Self::UnknownQuantifier { .. } => "logic.unknown_quantifier".to_owned(),
            Self::UnsupportedRuleHead { .. } => "logic.unsupported_rule_head".to_owned(),
            Self::NegationCycle { .. } => "logic.negation_cycle".to_owned(),
            Self::RecursiveAggregate { .. } => "logic.recursive_aggregate".to_owned(),
            Self::UnboundedWitness { reason, .. } => {
                format!("logic.unbounded_witness.{}", reason.code())
            }
            Self::GraphEndpointMissing { .. } => "logic.graph_endpoint_missing".to_owned(),
            Self::GraphEndpointUnexpected { .. } => "logic.graph_endpoint_unexpected".to_owned(),
            Self::GraphConflictingPathBound { .. } => {
                "logic.graph_conflicting_path_bound".to_owned()
            }
            Self::NonMonotoneRecursion { reason, .. } => {
                format!("logic.non_monotone_recursion.{}", reason.code())
            }
        }
    }

    /// The span the diagnostic points at.
    #[must_use]
    pub fn span(&self) -> SpanV1 {
        match self {
            Self::UnknownQuantifier { span, .. }
            | Self::UnsupportedRuleHead { span }
            | Self::NegationCycle { span, .. }
            | Self::RecursiveAggregate { span, .. }
            | Self::UnboundedWitness { span, .. }
            | Self::GraphEndpointMissing { span, .. }
            | Self::GraphEndpointUnexpected { span, .. }
            | Self::GraphConflictingPathBound { span, .. }
            | Self::NonMonotoneRecursion { span, .. } => *span,
        }
    }

    /// A stable human-readable message. Text is part of the contract: tests
    /// assert it verbatim.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::UnknownQuantifier { keyword, .. } => {
                format!("unknown source quantifier `{keyword}`; expected `exists` or `not exists`")
            }
            Self::UnsupportedRuleHead { .. } => {
                "source horn rule head must be a symbol or a path rooted at a symbol".to_owned()
            }
            Self::NegationCycle { cycle, .. } => {
                format!(
                    "recursion through negation is not stratifiable: cycle [{}]",
                    cycle.join(" -> ")
                )
            }
            Self::RecursiveAggregate {
                function,
                predicate,
                ..
            } => format!(
                "recursive aggregate `{function}` over recursive predicate `{predicate}` is not \
                 admitted"
            ),
            Self::UnboundedWitness { reason, .. } => match reason {
                UnboundedWitnessReasonV1::MissingMaxWitnesses => {
                    "unbounded witness materialization: a finite max-witness cap is required"
                        .to_owned()
                }
                UnboundedWitnessReasonV1::MissingPathLengthBound => {
                    "unbounded witness materialization: a cyclic relation requires a path-length \
                     bound"
                        .to_owned()
                }
                UnboundedWitnessReasonV1::ZeroMaxWitnesses => {
                    "ill-formed witness bound: max-witness cap of zero can never materialize a \
                     witness"
                        .to_owned()
                }
                UnboundedWitnessReasonV1::ZeroPathLengthBound => {
                    "ill-formed witness bound: a path-length bound of zero can never traverse an \
                     edge"
                        .to_owned()
                }
            },
            Self::GraphEndpointMissing { kind, role, .. } => {
                format!(
                    "FIND `{}` requires a `{}` endpoint",
                    crate::graph::graph_query_keyword(*kind),
                    role.keyword()
                )
            }
            Self::GraphEndpointUnexpected { kind, role, .. } => {
                format!(
                    "FIND `{}` does not accept a `{}` endpoint",
                    crate::graph::graph_query_keyword(*kind),
                    role.keyword()
                )
            }
            Self::GraphConflictingPathBound { kind, .. } => {
                format!(
                    "FIND `{}` supplies both an explicit `depth` bound and bounding waypoints \
                     (`via`/`through`); choose one path-length binding strategy",
                    crate::graph::graph_query_keyword(*kind)
                )
            }
            Self::NonMonotoneRecursion {
                predicate, reason, ..
            } => match reason {
                NonMonotoneReasonV1::NegatedRecursion => format!(
                    "non-monotone recursion: predicate `{predicate}` negates itself inside its own \
                     recursion"
                ),
                NonMonotoneReasonV1::AggregatedRecursion => format!(
                    "non-monotone recursion: predicate `{predicate}` aggregates itself inside its \
                     own recursion"
                ),
            },
        }
    }
}
