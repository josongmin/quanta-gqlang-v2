//! Typed, fail-closed planning diagnostics for unsupported construct MIXES
//! (ticket 07, decisions `D10` / `D16`).
//!
//! The planner consumes ALREADY-NORMALIZED algebra/logic (the lower crates
//! already rejected ill-formed shapes). What the planner additionally rejects is
//! a combination of otherwise-legal constructs that has no coherent
//! truth/effect/result-kind classification — e.g. an export effect mixed with a
//! proof-only evidence lane, or an unbounded-risk witness shape combined with a
//! mutation effect. Per `D16` there is no lossy downgrade: such a mix stays
//! explicitly non-servable, carrying a stable `unsupported_reason_code`.
//!
//! Each variant renders a STABLE `code()` (dot-namespaced under `planning.`) and
//! a stable human `message()` so CLI/LSP/tests assert exact text (`D22`).

use quanta_gqlang_v2_hir::SpanV1;
use quanta_gqlang_v2_logic::LogicDiagnosticV1;

/// Why an otherwise-normalized plan shape has no coherent planning
/// classification. The `code()` of each reason is the `unsupported_reason_code`
/// that flows into [`crate::SemanticCostV1::unsupported_reason_code`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnsupportedMixReasonV1 {
    /// A `path_witness` result was requested without any witness contract /
    /// selection policy. Witness payload rendering is impossible without an
    /// admitted witness shape.
    PathWitnessWithoutWitness,
    /// An export effect was requested alongside a proof/evidence-only lane: the
    /// plan both forbids materialization (proof-only) and demands it (export),
    /// which cannot both hold.
    ExportWithProofOnlyLane,
    /// A mutation-bearing effect was combined with witness/path materialization;
    /// mutation is forbidden in this host-agnostic frontend (`D14`/`D7`), and
    /// pairing it with witness enumeration has no admissible classification.
    MutationWithWitness,
    /// A recursive closure result was combined with an explicit total-order
    /// (sort) rendering contract over the closure, which has no stable order
    /// semantics at the closure level.
    OrderedRecursiveClosure,
    /// A proof/explain result-kind was requested together with an export effect;
    /// explain is read-only provenance and cannot also export candidates.
    ExplainWithExport,
}

impl UnsupportedMixReasonV1 {
    /// The stable reason code suffix. This exact string is what surfaces as the
    /// `unsupported_reason_code` cost field.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::PathWitnessWithoutWitness => "path_witness_without_witness",
            Self::ExportWithProofOnlyLane => "export_with_proof_only_lane",
            Self::MutationWithWitness => "mutation_with_witness",
            Self::OrderedRecursiveClosure => "ordered_recursive_closure",
            Self::ExplainWithExport => "explain_with_export",
        }
    }
}

/// A typed planning diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanningDiagnosticV1 {
    /// A mix of normalized constructs has no coherent planning classification.
    UnsupportedMix {
        /// Why the mix is unsupported.
        reason: UnsupportedMixReasonV1,
        /// Span of the construct (or result root) the mix is attributed to.
        span: SpanV1,
    },
    /// A FIND graph query failed its source-level legality (endpoint presence or
    /// bounded-witness boundedness). The exact logic diagnostic is carried so the
    /// planner surfaces logic's code/message/span verbatim rather than inventing
    /// a parallel reason (`D10`/`D12`: logic owns FIND legality, planning relays
    /// it fail-closed).
    IllegalGraphQuery {
        /// The originating logic diagnostic from FIND classification.
        logic: LogicDiagnosticV1,
    },
}

impl PlanningDiagnosticV1 {
    /// The stable diagnostic code, shared by CLI/LSP/tests. Codes are
    /// dot-namespaced under `planning.` and never change spelling.
    #[must_use]
    pub fn code(&self) -> String {
        match self {
            Self::UnsupportedMix { reason, .. } => {
                format!("planning.unsupported_mix.{}", reason.code())
            }
            // FIND legality is logic's; relay its stable code unchanged.
            Self::IllegalGraphQuery { logic } => logic.code(),
        }
    }

    /// The bare `unsupported_reason_code` (the reason suffix only), which flows
    /// into the fixed cost schema's `unsupported_reason_code` field.
    #[must_use]
    pub fn unsupported_reason_code(&self) -> String {
        match self {
            Self::UnsupportedMix { reason, .. } => reason.code().to_owned(),
            Self::IllegalGraphQuery { logic } => logic.code(),
        }
    }

    /// The span the diagnostic points at.
    #[must_use]
    pub fn span(&self) -> SpanV1 {
        match self {
            Self::UnsupportedMix { span, .. } => *span,
            Self::IllegalGraphQuery { logic } => logic.span(),
        }
    }

    /// A stable human-readable message. Text is part of the contract: tests
    /// assert it verbatim.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::UnsupportedMix { reason, .. } => match reason {
                UnsupportedMixReasonV1::PathWitnessWithoutWitness => {
                    "unsupported construct mix: a path-witness result requires an explicit \
                     witness contract and selection policy"
                        .to_owned()
                }
                UnsupportedMixReasonV1::ExportWithProofOnlyLane => {
                    "unsupported construct mix: export effect cannot combine with a \
                     proof/evidence-only lane"
                        .to_owned()
                }
                UnsupportedMixReasonV1::MutationWithWitness => {
                    "unsupported construct mix: mutation effect cannot combine with witness/path \
                     materialization"
                        .to_owned()
                }
                UnsupportedMixReasonV1::OrderedRecursiveClosure => {
                    "unsupported construct mix: a recursive closure result has no total-order \
                     rendering contract"
                        .to_owned()
                }
                UnsupportedMixReasonV1::ExplainWithExport => {
                    "unsupported construct mix: a proof/explain result cannot also export \
                     candidates"
                        .to_owned()
                }
            },
            // Relay logic's verbatim FIND message.
            Self::IllegalGraphQuery { logic } => logic.message(),
        }
    }
}
