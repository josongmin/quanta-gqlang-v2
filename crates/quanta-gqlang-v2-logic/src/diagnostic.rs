//! Typed, fail-closed diagnostics for the logic / recursion layer.
//!
//! Per packet decisions `D10` (no silent downgrade), `D27` (positive recursion
//! uses explicit least-fixpoint; recursive aggregates / unbounded witness reject
//! unless explicitly admitted) and `D31` (recursive witness/path materialization
//! requires explicit boundedness, cycle-dedup identity, and tie-break rules,
//! otherwise the shape rejects), every illegal logic shape emits one of these
//! typed diagnostics rather than degrading silently.
//!
//! Each variant renders a STABLE `code()` (dot-namespaced under `logic.`) and a
//! stable human `message()` so CLI/LSP/tests assert exact text (`D22`). Every
//! span is a registry-proven [`CheckedLocatedSpanV2`] pointing back at the
//! originating document construct (`D20`); predicate identity is the sealed
//! [`ResolvedPredicateIdV2`], never a caller-supplied string.
//!
//! W3 pre-cut (integrator-owned): the dead `UnsupportedRuleHead` variant and the
//! `GraphConflictingPathBound` variant died with their policies. Depth plus
//! waypoints is now valid (waypoints never substitute for depth), and rule-head
//! legality is owned by Sema/Types before Logic ever sees a rule.

use quanta_gqlang_v2_sema::{CheckedLocatedSpanV2, GraphQueryKindV1};
use quanta_gqlang_v2_types::ResolvedPredicateIdV2;

use crate::graph::GraphEndpointRoleV1;

/// A typed logic-semantics diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogicDiagnosticV1 {
    /// A recursion went through negation in a way that cannot be stratified:
    /// a predicate depends, transitively and through at least one negated
    /// edge, on itself. Stratified negation requires every negated dependency
    /// to sit in a strictly lower stratum, which a negation cycle makes
    /// impossible (`D27`).
    NegationCycle {
        /// The canonical offending cycle as stable predicate identities, in
        /// deterministic order invariant under rule/edge insertion
        /// permutation.
        cycle: Vec<ResolvedPredicateIdV2>,
        /// The canonical offending edge's checked location (the negated call
        /// that closes the cycle).
        offending_edge: CheckedLocatedSpanV2,
        /// Checked location of the rule whose body closed the cycle.
        span: CheckedLocatedSpanV2,
    },
    /// An aggregate was applied over a predicate that recurses (transitively)
    /// through itself. Recursive aggregates have no monotone least-fixpoint
    /// meaning in general, so every spelling rejects (`D27`); the policy is
    /// reject-all and Sema owns nonrecursive aggregate legality.
    RecursiveAggregate {
        /// The aggregate function name as written (e.g. `sum`, `count`).
        function: String,
        /// The recursive predicate the aggregate folded over.
        predicate: ResolvedPredicateIdV2,
        /// Checked location of the offending aggregate call.
        span: CheckedLocatedSpanV2,
    },
    /// A witness / path materialization over a recursive relation was requested
    /// without a complete boundedness contract (a finite max-witness cap is
    /// mandatory; over a relation that can cycle a path-length bound is also
    /// mandatory). Unbounded witness families reject rather than streaming
    /// infinitely (`D31`).
    UnboundedWitness {
        /// Why the witness contract is not bounded.
        reason: UnboundedWitnessReasonV1,
        /// Checked location of the offending witness/path request.
        span: CheckedLocatedSpanV2,
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
        /// Checked location of the offending graph query.
        span: CheckedLocatedSpanV2,
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
        /// Checked location of the offending graph query.
        span: CheckedLocatedSpanV2,
    },
    /// The document-owned rule program could not be projected into a faithful
    /// contract: a member, call edge, target, polarity, source identity, or
    /// checked location disagreed with the sealed document. Logic validates
    /// before allocating any graph and fails closed rather than inventing a
    /// second resolver (`D10`); the reason names the exact disagreement.
    RuleProgramInvalid {
        /// Which part of the projected contract was not faithful.
        reason: RuleProgramInvalidReasonV1,
        /// Checked location of the offending fact.
        span: CheckedLocatedSpanV2,
    },
    /// A rule-graph traversal refused to exceed one immutable dimension of the
    /// document's downstream budget receipt. Charging happens before the work;
    /// exact limit succeeds and limit+1 rejects here at the refused step's
    /// checked location.
    RuleGraphBudgetExceeded {
        /// Which sealed dimension was exhausted.
        dimension: RuleGraphBudgetDimensionV1,
        /// The exact sealed limit that would have been exceeded.
        limit: u32,
        /// Checked location of the step whose charge was refused.
        span: CheckedLocatedSpanV2,
    },
}

/// Which immutable rule-graph budget dimension was exhausted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleGraphBudgetDimensionV1 {
    /// `max_rule_program_rules`, before graph projection.
    Rules,
    /// `max_rule_graph_nodes`.
    Nodes,
    /// `max_rule_graph_edges`.
    Edges,
    /// `max_rule_steps` (frame pushes, SCC emission, witness search,
    /// relaxation).
    Steps,
}

impl RuleGraphBudgetDimensionV1 {
    /// Stable code suffix for this dimension.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Rules => "rules",
            Self::Nodes => "nodes",
            Self::Edges => "edges",
            Self::Steps => "steps",
        }
    }

    /// Stable human name for this dimension.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Rules => "raw rule",
            Self::Nodes => "rule graph node",
            Self::Edges => "rule graph edge",
            Self::Steps => "rule graph step",
        }
    }
}

/// Why a projected rule program was not a faithful contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleProgramInvalidReasonV1 {
    /// A rule member present in the document was not enumerated.
    MissingMember,
    /// The projection enumerated one member more than once.
    DuplicateMember,
    /// A call edge did not resolve to a member predicate of this document.
    NonLocalTarget,
    /// A call edge disagreed with the document's resolved call identity.
    WrongCallId,
    /// A call edge's target disagreed with the document's resolved target.
    TargetTransplanted,
    /// A call edge's polarity disagreed with the document's resolved polarity.
    PolarityFlip,
    /// A checked location or registry identity belonged to another document.
    ForeignLocation,
    /// The document's rule set was inconsistent with its own item set.
    MembershipMismatch,
}

impl RuleProgramInvalidReasonV1 {
    /// Stable code suffix for this reason.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingMember => "missing_member",
            Self::DuplicateMember => "duplicate_member",
            Self::NonLocalTarget => "nonlocal_target",
            Self::WrongCallId => "wrong_call_id",
            Self::TargetTransplanted => "target_transplanted",
            Self::PolarityFlip => "polarity_flip",
            Self::ForeignLocation => "foreign_location",
            Self::MembershipMismatch => "membership_mismatch",
        }
    }
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
    /// The requested max-witness cap exceeds the document's sealed
    /// `max_witnesses` budget dimension.
    MaxWitnessesOverBudget,
    /// The requested path-length bound exceeds the document's sealed
    /// `max_witness_depth` budget dimension.
    PathLengthBoundOverBudget,
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
            Self::MaxWitnessesOverBudget => "max_witnesses_over_budget",
            Self::PathLengthBoundOverBudget => "path_length_bound_over_budget",
        }
    }
}

impl LogicDiagnosticV1 {
    /// Full stable code without allocation, shared with pipeline projections.
    #[must_use]
    pub const fn code_str(&self) -> &'static str {
        match self {
            Self::NegationCycle { .. } => "logic.negation_cycle",
            Self::RecursiveAggregate { .. } => "logic.recursive_aggregate",
            Self::UnboundedWitness { reason, .. } => match reason {
                UnboundedWitnessReasonV1::MissingMaxWitnesses => {
                    "logic.unbounded_witness.missing_max_witnesses"
                }
                UnboundedWitnessReasonV1::MissingPathLengthBound => {
                    "logic.unbounded_witness.missing_path_length_bound"
                }
                UnboundedWitnessReasonV1::ZeroMaxWitnesses => {
                    "logic.unbounded_witness.zero_max_witnesses"
                }
                UnboundedWitnessReasonV1::ZeroPathLengthBound => {
                    "logic.unbounded_witness.zero_path_length_bound"
                }
                UnboundedWitnessReasonV1::MaxWitnessesOverBudget => {
                    "logic.unbounded_witness.max_witnesses_over_budget"
                }
                UnboundedWitnessReasonV1::PathLengthBoundOverBudget => {
                    "logic.unbounded_witness.path_length_bound_over_budget"
                }
            },
            Self::GraphEndpointMissing { .. } => "logic.graph_endpoint_missing",
            Self::GraphEndpointUnexpected { .. } => "logic.graph_endpoint_unexpected",
            Self::RuleProgramInvalid { reason, .. } => match reason {
                RuleProgramInvalidReasonV1::MissingMember => "logic.rule_program.missing_member",
                RuleProgramInvalidReasonV1::DuplicateMember => {
                    "logic.rule_program.duplicate_member"
                }
                RuleProgramInvalidReasonV1::NonLocalTarget => "logic.rule_program.nonlocal_target",
                RuleProgramInvalidReasonV1::WrongCallId => "logic.rule_program.wrong_call_id",
                RuleProgramInvalidReasonV1::TargetTransplanted => {
                    "logic.rule_program.target_transplanted"
                }
                RuleProgramInvalidReasonV1::PolarityFlip => "logic.rule_program.polarity_flip",
                RuleProgramInvalidReasonV1::ForeignLocation => {
                    "logic.rule_program.foreign_location"
                }
                RuleProgramInvalidReasonV1::MembershipMismatch => {
                    "logic.rule_program.membership_mismatch"
                }
            },
            Self::RuleGraphBudgetExceeded { dimension, .. } => match dimension {
                RuleGraphBudgetDimensionV1::Rules => "logic.rule_graph_budget.rules",
                RuleGraphBudgetDimensionV1::Nodes => "logic.rule_graph_budget.nodes",
                RuleGraphBudgetDimensionV1::Edges => "logic.rule_graph_budget.edges",
                RuleGraphBudgetDimensionV1::Steps => "logic.rule_graph_budget.steps",
            },
        }
    }

    /// The stable diagnostic code, shared by CLI/LSP/tests. Codes are
    /// dot-namespaced under `logic.` and never change spelling.
    #[must_use]
    pub fn code(&self) -> String {
        self.code_str().to_owned()
    }

    /// The registry-proven location the diagnostic points at.
    #[must_use]
    pub const fn span(&self) -> &CheckedLocatedSpanV2 {
        match self {
            Self::NegationCycle { span, .. }
            | Self::RecursiveAggregate { span, .. }
            | Self::UnboundedWitness { span, .. }
            | Self::GraphEndpointMissing { span, .. }
            | Self::GraphEndpointUnexpected { span, .. }
            | Self::RuleProgramInvalid { span, .. }
            | Self::RuleGraphBudgetExceeded { span, .. } => span,
        }
    }

    /// A stable human-readable message. Text is part of the contract: tests
    /// assert it verbatim.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::NegationCycle { cycle, .. } => {
                let names: Vec<&str> = cycle.iter().map(ResolvedPredicateIdV2::as_str).collect();
                format!(
                    "recursion through negation is not stratifiable: cycle [{}]",
                    names.join(" -> ")
                )
            }
            Self::RecursiveAggregate {
                function,
                predicate,
                ..
            } => format!(
                "recursive aggregate `{function}` over recursive predicate `{}` is not admitted",
                predicate.as_str()
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
                UnboundedWitnessReasonV1::MaxWitnessesOverBudget => {
                    "witness bound exceeds the document's sealed max-witnesses budget".to_owned()
                }
                UnboundedWitnessReasonV1::PathLengthBoundOverBudget => {
                    "path-length bound exceeds the document's sealed witness-depth budget"
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
            Self::RuleProgramInvalid { reason, .. } => match reason {
                RuleProgramInvalidReasonV1::MissingMember => {
                    "rule program contract is incomplete: a document rule member was not enumerated"
                        .to_owned()
                }
                RuleProgramInvalidReasonV1::DuplicateMember => {
                    "rule program contract enumerated the same member more than once".to_owned()
                }
                RuleProgramInvalidReasonV1::NonLocalTarget => {
                    "rule call target is not a predicate of this document".to_owned()
                }
                RuleProgramInvalidReasonV1::WrongCallId => {
                    "rule call identity disagrees with the document's resolved call".to_owned()
                }
                RuleProgramInvalidReasonV1::TargetTransplanted => {
                    "rule call target disagrees with the document's resolved target".to_owned()
                }
                RuleProgramInvalidReasonV1::PolarityFlip => {
                    "rule call polarity disagrees with the document's resolved polarity".to_owned()
                }
                RuleProgramInvalidReasonV1::ForeignLocation => {
                    "rule fact location or registry identity belongs to another document".to_owned()
                }
                RuleProgramInvalidReasonV1::MembershipMismatch => {
                    "rule membership disagrees with the document's own item set".to_owned()
                }
            },
            Self::RuleGraphBudgetExceeded {
                dimension, limit, ..
            } => format!("{} budget of {limit} exceeded", dimension.name()),
        }
    }
}
