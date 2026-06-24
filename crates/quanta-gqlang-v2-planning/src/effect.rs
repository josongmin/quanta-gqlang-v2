//! Truth-lane, evidence, and effect models (ticket 07).
//!
//! These are the host-agnostic policy classifications the planner derives from
//! the normalized algebra/logic shape. They are SEMANTIC requirements, not
//! runtime DTOs (`D14`): they say what TRUTH lane a result lives in, what
//! EVIDENCE it requires, and what EFFECT it is permitted to have — never how a
//! runtime would execute it.
//!
//! Pinned-semantics 2/3 require that AI/candidate/proof lanes stay explicit and
//! typed, and that final policy truth remains impossible without the right proof
//! semantics. The [`TruthLaneV1`] / [`EvidenceRequirementV1`] split encodes that
//! directly.

/// The TRUTH lane a result lives in: how authoritative its truth value is.
///
/// `D7`/`D14`: these are host-agnostic policy lanes, not runtime/provider nouns.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TruthLaneV1 {
    /// Deterministic relational/logic truth: the result is true by the closed
    /// semantics of the normalized algebra/logic (a plain query/closure result).
    Deterministic,
    /// A candidate lane: the result is a proposed candidate set that has NOT yet
    /// been reconfirmed; its truth is provisional pending proof.
    Candidate,
    /// A proof-confirmed lane: the result's truth is backed by an explicit
    /// proof/witness, so it may carry final policy weight.
    ProofConfirmed,
}

impl TruthLaneV1 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::Candidate => "candidate",
            Self::ProofConfirmed => "proof_confirmed",
        }
    }

    /// Whether a result in this lane may carry FINAL policy truth.
    ///
    /// Pinned-semantics 3: final policy truth remains impossible without the
    /// right proof semantics; only [`Self::ProofConfirmed`] qualifies.
    #[must_use]
    pub const fn is_final_policy_capable(self) -> bool {
        matches!(self, Self::ProofConfirmed)
    }
}

/// What evidence a result must carry to be admissible in its truth lane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EvidenceRequirementV1 {
    /// No standalone evidence beyond the relational/logic result itself.
    None,
    /// A source location (file/span) must accompany each result row, e.g. for an
    /// alert that must point at code (`D20`).
    SourceLocation,
    /// A concrete witness/path payload must accompany the result (e.g. a
    /// reachability path), per the witness contract.
    WitnessPayload,
    /// A full proof/explain tree must back the result (the strongest evidence;
    /// required for the proof-confirmed lane).
    ProofTree,
}

impl EvidenceRequirementV1 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::SourceLocation => "source_location",
            Self::WitnessPayload => "witness_payload",
            Self::ProofTree => "proof_tree",
        }
    }
}

/// The EFFECT class a plan is permitted to have.
///
/// `D7`/`D14`: mutation is NOT a frontend effect; the strongest permitted class
/// here is `ExportOnly`, and `MutationForbidden` is the explicit guard that
/// mutation may never be requested through this host-agnostic frontend.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffectClassV1 {
    /// Read-only: the plan reads relations and produces a result; no side
    /// effects of any kind.
    ReadOnly,
    /// Proof/evidence-only: the plan produces proof/evidence artifacts and must
    /// NOT export or materialize candidates beyond that evidence.
    ProofEvidenceOnly,
    /// Export-only: the plan may export a candidate set (e.g. for downstream
    /// reconfirmation), but performs no mutation.
    ExportOnly,
    /// Mutation-forbidden: an explicit marker that mutation was requested and is
    /// rejected by this frontend. Carried so the rejection is visible rather
    /// than silent.
    MutationForbidden,
}

impl EffectClassV1 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::ProofEvidenceOnly => "proof_evidence_only",
            Self::ExportOnly => "export_only",
            Self::MutationForbidden => "mutation_forbidden",
        }
    }
}
