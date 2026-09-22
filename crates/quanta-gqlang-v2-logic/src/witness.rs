//! Bounded witness / path materialization contract (ticket 06, lane W3-B).
//!
//! Cites `D31`: recursive witness/path materialization requires explicit
//! boundedness, cycle-dedup identity, and tie-break rules; otherwise the shape
//! REJECTS. This layer performs STATIC contract validation only: it never
//! traverses a graph or materializes a runtime witness.
//!
//! A well-formed [`WitnessContractV2`] carries:
//!
//! 1. a DEDUP IDENTITY ([`DedupIdentityV1`]) — what makes two enumerated
//!    witnesses "the same" so cycles do not re-emit equivalents forever;
//! 2. a finite MAX WITNESS COUNT — the hard cap on emitted witnesses, taken from
//!    the document's sealed witness bound; there is no caller-selected or
//!    independently defaulted cap;
//! 3. a PATH-LENGTH BOUND — mandatory over a relation that can cycle, so
//!    enumeration cannot revisit nodes without end;
//! 4. a canonical TIE-BREAK ([`WitnessTieBreakV1`]) — deterministic ordering for
//!    equivalent-cost witnesses so the kept set is reproducible.
//!
//! Every bound is checked twice: it must be present and non-degenerate, and it
//! must not exceed the document's sealed witness budget (`max_witness_depth`,
//! `max_witnesses`). Rejection order is deterministic (missing cap, zero cap,
//! over-budget cap, missing path bound over a cyclic relation, zero path bound,
//! over-budget path bound), so a shape with several defects always reports the
//! same first one.
//!
//! The binding input [`WitnessBindingV2`] and the validation entry points are
//! crate-private: the only production producer is the FIND graph classifier in
//! [`crate::graph`], which derives every field from the sealed
//! [`quanta_gqlang_v2_sema::ResolvedGraphQueryV2`]. External callers receive the
//! validated contract through read-only accessors.

use quanta_gqlang_v2_sema::CheckedLocatedSpanV2;

use crate::diagnostic::{LogicDiagnosticV1, UnboundedWitnessReasonV1};

/// Whether witness enumeration over the underlying relation can revisit nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationShapeV1 {
    /// The relation is acyclic (a DAG): every path is finite by structure, so a
    /// path-length bound is optional.
    ///
    /// Retained vocabulary only. No FIND family in this packet is admitted as
    /// `Acyclic`: this packet accepts no unowned acyclic-proof shortcut, so
    /// [`crate::graph::GraphQueryContractV2`] binds every subkind as
    /// [`RelationShapeV1::Cyclic`] and requires an explicit positive depth.
    Acyclic,
    /// The relation can cycle: a path-length bound is MANDATORY to keep witness
    /// enumeration finite.
    Cyclic,
}

/// The identity under which two enumerated witnesses are considered equal for
/// cycle-dedup. Materialization keeps at most one witness per dedup-identity
/// class.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DedupIdentityV1 {
    /// Two witnesses are the same iff their endpoint pair (source, sink) is the
    /// same. Collapses all paths between a pair to one witness.
    Endpoints,
    /// Two witnesses are the same iff their full ordered node sequence is the
    /// same. Distinct routes between the same endpoints are distinct witnesses.
    NodeSequence,
    /// Two witnesses are the same iff their unordered node set is the same.
    /// Distinguishes which nodes were visited but not their order.
    NodeSet,
}

/// The canonical tie-break for equivalent witnesses, so the kept set under a cap
/// is deterministic and reproducible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WitnessTieBreakV1 {
    /// Prefer the shortest path; break further ties by lexical node order.
    ShortestThenLexical,
    /// Prefer the lexically-smallest node sequence outright.
    Lexical,
}

/// The boundedness facts a contract is sealed from, before validation.
///
/// Crate-private on purpose: no downstream caller may select, default, extend,
/// or replace a witness bound. The only production producer is the FIND graph
/// classifier, which reads every field from the sealed resolved query and the
/// document's immutable budget projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WitnessBindingV2 {
    /// Whether the underlying relation can cycle.
    pub(crate) shape: RelationShapeV1,
    /// Dedup identity for equivalent witnesses.
    pub(crate) dedup: DedupIdentityV1,
    /// Maximum number of witnesses to materialize. `None` is UNBOUNDED and
    /// rejects; `Some(0)` is degenerate and rejects.
    pub(crate) max_witnesses: Option<u32>,
    /// Path-length bound (max edges in an enumerated path). Mandatory when
    /// `shape` is [`RelationShapeV1::Cyclic`]. `Some(0)` is degenerate (zero
    /// edges traverse nothing) and rejects.
    pub(crate) max_path_length: Option<u32>,
    /// Canonical tie-break for equivalent witnesses.
    pub(crate) tie_break: WitnessTieBreakV1,
    /// The document's sealed `max_witnesses` budget dimension.
    pub(crate) sealed_max_witnesses: u32,
    /// The document's sealed `max_witness_depth` budget dimension.
    pub(crate) sealed_max_depth: u32,
}

/// A validated, BOUNDED witness contract.
///
/// Only constructable through the crate-private binding entry points, which
/// enforce the boundedness rules and the sealed budget; the fields are therefore
/// guaranteed finite. The presence of this value is itself the proof that the
/// witness shape is admissible.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WitnessContractV2 {
    shape: RelationShapeV1,
    dedup: DedupIdentityV1,
    max_witnesses: u32,
    max_path_length: Option<u32>,
    tie_break: WitnessTieBreakV1,
}

impl WitnessContractV2 {
    /// Validate a binding into a bounded contract, reporting the FIRST
    /// deterministic reason it is not admissible. Span-free so owner-unit rails
    /// can prove the reason matrix without minting a checked location.
    pub(crate) fn validate(binding: &WitnessBindingV2) -> Result<Self, UnboundedWitnessReasonV1> {
        let max_witnesses = match binding.max_witnesses {
            None => return Err(UnboundedWitnessReasonV1::MissingMaxWitnesses),
            Some(0) => return Err(UnboundedWitnessReasonV1::ZeroMaxWitnesses),
            Some(cap) if cap > binding.sealed_max_witnesses => {
                return Err(UnboundedWitnessReasonV1::MaxWitnessesOverBudget);
            }
            Some(cap) => cap,
        };

        if binding.shape == RelationShapeV1::Cyclic && binding.max_path_length.is_none() {
            return Err(UnboundedWitnessReasonV1::MissingPathLengthBound);
        }

        if binding.max_path_length == Some(0) {
            return Err(UnboundedWitnessReasonV1::ZeroPathLengthBound);
        }

        if binding
            .max_path_length
            .is_some_and(|bound| bound > binding.sealed_max_depth)
        {
            return Err(UnboundedWitnessReasonV1::PathLengthBoundOverBudget);
        }

        Ok(Self {
            shape: binding.shape,
            dedup: binding.dedup,
            max_witnesses,
            max_path_length: binding.max_path_length,
            tie_break: binding.tie_break,
        })
    }

    /// Bind a checked location into the validated contract, anchoring any
    /// rejection at the offending FIND item.
    pub(crate) fn bind(
        binding: &WitnessBindingV2,
        span: &CheckedLocatedSpanV2,
    ) -> Result<Self, LogicDiagnosticV1> {
        Self::validate(binding).map_err(|reason| LogicDiagnosticV1::UnboundedWitness {
            reason,
            span: span.clone(),
        })
    }

    /// The relation shape this contract was bound against.
    #[must_use]
    pub const fn shape(&self) -> RelationShapeV1 {
        self.shape
    }

    /// Dedup identity for equivalent witnesses.
    #[must_use]
    pub const fn dedup(&self) -> DedupIdentityV1 {
        self.dedup
    }

    /// The finite, non-zero max-witness cap.
    #[must_use]
    pub const fn max_witnesses(&self) -> u32 {
        self.max_witnesses
    }

    /// The path-length bound (always present for cyclic relations; may be
    /// present for acyclic ones too).
    #[must_use]
    pub const fn max_path_length(&self) -> Option<u32> {
        self.max_path_length
    }

    /// Canonical tie-break.
    #[must_use]
    pub const fn tie_break(&self) -> WitnessTieBreakV1 {
        self.tie_break
    }
}

#[cfg(test)]
mod tests {
    #![expect(
        clippy::expect_used,
        clippy::panic,
        clippy::wildcard_enum_match_arm,
        reason = "owner contract rails assert exact rejects on fixed bindings and fail closed on an unexpected diagnostic variant"
    )]

    use super::*;
    use crate::diagnostic::LogicDiagnosticV1;

    const SEALED_MAX_WITNESSES: u32 = 32;
    const SEALED_MAX_DEPTH: u32 = 16;

    fn binding(
        shape: RelationShapeV1,
        max_witnesses: Option<u32>,
        max_path_length: Option<u32>,
    ) -> WitnessBindingV2 {
        WitnessBindingV2 {
            shape,
            dedup: DedupIdentityV1::NodeSequence,
            max_witnesses,
            max_path_length,
            tie_break: WitnessTieBreakV1::ShortestThenLexical,
            sealed_max_witnesses: SEALED_MAX_WITNESSES,
            sealed_max_depth: SEALED_MAX_DEPTH,
        }
    }

    // Exact sealed limit on BOTH dimensions binds, and the contract retains the
    // exact sealed values rather than any invented default.
    #[test]
    fn exact_sealed_limit_binds_and_retains_exact_bounds() {
        let contract = WitnessContractV2::validate(&binding(
            RelationShapeV1::Cyclic,
            Some(SEALED_MAX_WITNESSES),
            Some(SEALED_MAX_DEPTH),
        ))
        .expect("exact sealed limit is admissible");
        assert_eq!(contract.max_witnesses(), SEALED_MAX_WITNESSES);
        assert_eq!(contract.max_path_length(), Some(SEALED_MAX_DEPTH));
        assert_eq!(contract.shape(), RelationShapeV1::Cyclic);
        assert_eq!(contract.dedup(), DedupIdentityV1::NodeSequence);
        assert_eq!(contract.tie_break(), WitnessTieBreakV1::ShortestThenLexical);
    }

    // One over the sealed witness cap rejects with the over-budget reason: the
    // document receipt is the only cap authority (no independent default).
    #[test]
    fn one_over_sealed_witness_cap_rejects_over_budget() {
        let err = WitnessContractV2::validate(&binding(
            RelationShapeV1::Cyclic,
            Some(SEALED_MAX_WITNESSES + 1),
            Some(1),
        ))
        .expect_err("cap above the sealed budget rejects");
        assert_eq!(err, UnboundedWitnessReasonV1::MaxWitnessesOverBudget);
        assert_eq!(err.code(), "max_witnesses_over_budget");
    }

    // One over the sealed witness depth rejects with the over-budget reason.
    #[test]
    fn one_over_sealed_witness_depth_rejects_over_budget() {
        let err = WitnessContractV2::validate(&binding(
            RelationShapeV1::Cyclic,
            Some(1),
            Some(SEALED_MAX_DEPTH + 1),
        ))
        .expect_err("depth above the sealed budget rejects");
        assert_eq!(err, UnboundedWitnessReasonV1::PathLengthBoundOverBudget);
        assert_eq!(err.code(), "path_length_bound_over_budget");
    }

    // A missing or zero cap is degenerate before any over-budget consideration.
    #[test]
    fn missing_and_zero_witness_cap_reject() {
        assert_eq!(
            WitnessContractV2::validate(&binding(RelationShapeV1::Cyclic, None, Some(1)))
                .expect_err("missing cap rejects"),
            UnboundedWitnessReasonV1::MissingMaxWitnesses
        );
        assert_eq!(
            WitnessContractV2::validate(&binding(RelationShapeV1::Cyclic, Some(0), Some(1)))
                .expect_err("zero cap rejects"),
            UnboundedWitnessReasonV1::ZeroMaxWitnesses
        );
    }

    // A cyclic relation without a path-length bound rejects; a zero bound is
    // degenerate even though it is present.
    #[test]
    fn missing_and_zero_path_length_reject_over_a_cyclic_relation() {
        assert_eq!(
            WitnessContractV2::validate(&binding(RelationShapeV1::Cyclic, Some(4), None))
                .expect_err("cyclic relation without a path bound rejects"),
            UnboundedWitnessReasonV1::MissingPathLengthBound
        );
        assert_eq!(
            WitnessContractV2::validate(&binding(RelationShapeV1::Cyclic, Some(4), Some(0)))
                .expect_err("zero path bound rejects"),
            UnboundedWitnessReasonV1::ZeroPathLengthBound
        );
    }

    // Rejection order is part of the contract: the first defect in
    // (cap presence, cap degeneracy, cap budget, path presence, path degeneracy,
    // path budget) always wins, whatever else is also wrong.
    #[test]
    fn rejection_order_is_deterministic() {
        let cyclic = RelationShapeV1::Cyclic;
        assert_eq!(
            WitnessContractV2::validate(&binding(cyclic, None, None)).expect_err("first defect"),
            UnboundedWitnessReasonV1::MissingMaxWitnesses
        );
        assert_eq!(
            WitnessContractV2::validate(&binding(cyclic, Some(0), None))
                .expect_err("second defect"),
            UnboundedWitnessReasonV1::ZeroMaxWitnesses
        );
        assert_eq!(
            WitnessContractV2::validate(&binding(cyclic, Some(SEALED_MAX_WITNESSES + 1), None))
                .expect_err("third defect"),
            UnboundedWitnessReasonV1::MaxWitnessesOverBudget
        );
        assert_eq!(
            WitnessContractV2::validate(&binding(cyclic, Some(4), None))
                .expect_err("fourth defect"),
            UnboundedWitnessReasonV1::MissingPathLengthBound
        );
        assert_eq!(
            WitnessContractV2::validate(&binding(cyclic, Some(4), Some(0)))
                .expect_err("fifth defect"),
            UnboundedWitnessReasonV1::ZeroPathLengthBound
        );
        assert_eq!(
            WitnessContractV2::validate(&binding(cyclic, Some(4), Some(SEALED_MAX_DEPTH + 1)))
                .expect_err("sixth defect"),
            UnboundedWitnessReasonV1::PathLengthBoundOverBudget
        );
    }

    // The acyclic arm of the retained vocabulary still admits a structurally
    // finite relation without a path bound. No production path constructs it in
    // this packet; the graph classifier binds every FIND family as Cyclic.
    #[test]
    fn acyclic_relation_may_omit_the_path_bound() {
        let contract =
            WitnessContractV2::validate(&binding(RelationShapeV1::Acyclic, Some(4), None))
                .expect("acyclic relation needs no path bound");
        assert_eq!(contract.shape(), RelationShapeV1::Acyclic);
        assert_eq!(contract.max_path_length(), None);
    }

    // The transaction-UNREACHABLE rails (a sealed document never carries a
    // missing/zero cap, `depth = 0`, or an over-budget depth) still report the
    // exact full diagnostic code, message, and checked location when bound.
    #[test]
    fn degenerate_and_over_budget_rails_report_exact_codes_and_location() {
        let span = quanta_gqlang_v2_sema::test_support::reference_checked_span_v2(3, 11)
            .expect("fixture checked location is in bounds");
        let cyclic = RelationShapeV1::Cyclic;
        let cases = [
            (
                binding(cyclic, None, Some(1)),
                UnboundedWitnessReasonV1::MissingMaxWitnesses,
                "logic.unbounded_witness.missing_max_witnesses",
                "unbounded witness materialization: a finite max-witness cap is required",
            ),
            (
                binding(cyclic, Some(0), Some(1)),
                UnboundedWitnessReasonV1::ZeroMaxWitnesses,
                "logic.unbounded_witness.zero_max_witnesses",
                "ill-formed witness bound: max-witness cap of zero can never materialize a witness",
            ),
            (
                binding(cyclic, Some(SEALED_MAX_WITNESSES + 1), Some(1)),
                UnboundedWitnessReasonV1::MaxWitnessesOverBudget,
                "logic.unbounded_witness.max_witnesses_over_budget",
                "witness bound exceeds the document's sealed max-witnesses budget",
            ),
            (
                binding(cyclic, Some(4), None),
                UnboundedWitnessReasonV1::MissingPathLengthBound,
                "logic.unbounded_witness.missing_path_length_bound",
                "unbounded witness materialization: a cyclic relation requires a path-length bound",
            ),
            (
                binding(cyclic, Some(4), Some(0)),
                UnboundedWitnessReasonV1::ZeroPathLengthBound,
                "logic.unbounded_witness.zero_path_length_bound",
                "ill-formed witness bound: a path-length bound of zero can never traverse an edge",
            ),
            (
                binding(cyclic, Some(4), Some(SEALED_MAX_DEPTH + 1)),
                UnboundedWitnessReasonV1::PathLengthBoundOverBudget,
                "logic.unbounded_witness.path_length_bound_over_budget",
                "path-length bound exceeds the document's sealed witness-depth budget",
            ),
        ];

        for (binding, reason, code, message) in cases {
            let err = WitnessContractV2::bind(&binding, &span).expect_err("rail rejects");
            assert_eq!(err.code(), code);
            assert_eq!(err.message(), message);
            assert_eq!(err.span(), &span);
            match err {
                LogicDiagnosticV1::UnboundedWitness {
                    reason: actual,
                    span: actual_span,
                } => {
                    assert_eq!(actual, reason);
                    assert_eq!(&actual_span, &span);
                }
                other => panic!("expected UnboundedWitness, got {other:?}"),
            }
        }
    }

    // Each reject reason keeps its stable code suffix (the `logic.` prefix is
    // owned by `LogicDiagnosticV1::code`).
    #[test]
    fn reason_code_suffixes_are_stable() {
        let codes = [
            (
                UnboundedWitnessReasonV1::MissingMaxWitnesses,
                "missing_max_witnesses",
            ),
            (
                UnboundedWitnessReasonV1::MissingPathLengthBound,
                "missing_path_length_bound",
            ),
            (
                UnboundedWitnessReasonV1::ZeroMaxWitnesses,
                "zero_max_witnesses",
            ),
            (
                UnboundedWitnessReasonV1::ZeroPathLengthBound,
                "zero_path_length_bound",
            ),
            (
                UnboundedWitnessReasonV1::MaxWitnessesOverBudget,
                "max_witnesses_over_budget",
            ),
            (
                UnboundedWitnessReasonV1::PathLengthBoundOverBudget,
                "path_length_bound_over_budget",
            ),
        ];
        for (reason, code) in codes {
            assert_eq!(reason.code(), code);
        }
    }
}
