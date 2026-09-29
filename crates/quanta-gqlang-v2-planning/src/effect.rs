//! Authority-lane, determinism-lane, evidence, and effect vocabulary (ticket 07).
//!
//! The V2 cut separates two classifications the V1 `TruthLaneV1` conflated:
//!
//! - [`AuthorityLaneV2`]: WHO checked the result — ordinary frontend-checked
//!   semantics, or an exact `REQUIRE CERTIFIED` item matched to the modules
//!   admission proof. There is deliberately NO advisory/candidate lane: no
//!   accepted source producer carries a certified/advisory truth lane for
//!   arbitrary relations, so a planner guess cannot manufacture one.
//! - [`EvaluationDeterminismV2`]: HOW deterministic the retained expression
//!   effects make the result. It is folded from sealed effects only, never
//!   hardcoded.
//!
//! These are host-agnostic semantic requirements, not runtime DTOs (`D14`): they
//! say what evidence a result must carry and what effect it may have — never how
//! a runtime would execute it. Mutation is not a frontend effect, so the
//! strongest class below is [`EffectClassV2::ExportOnly`]; the
//! [`EffectClassV2::MutationForbidden`] guard marker is retained as the explicit
//! reject spelling and is unreachable from any sealed input.

/// WHO checked a plan's result.
///
/// Mapping (ticket 07 vocabulary table):
///
/// | Sealed input | `AuthorityLaneV2` |
/// |---|---|
/// | ordinary typed relation, graph, CHECK, or presentation | [`Self::FrontendChecked`] |
/// | exact `REQUIRE CERTIFIED` item matched to the modules admission proof | [`Self::CertifiedRequirementValidated`] |
/// | catalog declaration / unresolved open analysis | no plan at all |
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AuthorityLaneV2 {
    /// Ordinary frontend-checked semantics: the result is true by the closed
    /// semantics of the sealed document.
    FrontendChecked,
    /// The exact `REQUIRE CERTIFIED` directive validated against the modules
    /// admission proof. This certifies only that directive, never an unrelated
    /// output.
    CertifiedRequirementValidated,
}

impl AuthorityLaneV2 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::FrontendChecked => "frontend_checked",
            Self::CertifiedRequirementValidated => "certified_requirement_validated",
        }
    }
}

/// HOW deterministic the sealed expression effects make a plan's evaluation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EvaluationDeterminismV2 {
    /// Every retained expression effect says deterministic, so the same sealed
    /// document always evaluates to the same result.
    Deterministic,
    /// At least one retained expression effect is nondeterministic (e.g. a
    /// `now()` call), so evaluation is not reproducible from the document alone.
    NonDeterministic,
}

impl EvaluationDeterminismV2 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::NonDeterministic => "non_deterministic",
        }
    }
}

/// What evidence a result must carry to be admissible in its authority lane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EvidenceRequirementV2 {
    /// No standalone evidence beyond the relational/logic result itself.
    None,
    /// A source location must accompany each result row, e.g. for an alert that
    /// must point at code (`D20`).
    SourceLocation,
    /// A concrete witness/path payload must accompany the result, per the sealed
    /// witness contract.
    WitnessPayload,
    /// A full proof/explain tree must back the result (the strongest evidence;
    /// required by a required confirmation and by the certified lane).
    ProofTree,
}

impl EvidenceRequirementV2 {
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
/// `D7`/`D14`: mutation is NOT a frontend effect. No sealed input can request it,
/// so [`Self::MutationForbidden`] is a guard spelling rather than a reachable
/// outcome; the strongest reachable class is [`Self::ExportOnly`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffectClassV2 {
    /// Read-only: the plan reads relations and produces a result; no side
    /// effects of any kind.
    ReadOnly,
    /// Proof/evidence-only: the plan produces proof/evidence artifacts and must
    /// NOT export or materialize candidates beyond that evidence.
    ProofEvidenceOnly,
    /// Export-only: the plan may export a candidate set, but performs no
    /// mutation.
    ExportOnly,
    /// Mutation-forbidden guard marker. Unreachable from a sealed input; kept so
    /// the frontend's explicit mutation prohibition has one stable spelling.
    MutationForbidden,
    /// An apply intent requiring an independent runtime mutation grant.
    MutationRequiresGrant,
}

impl EffectClassV2 {
    /// Stable keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::ReadOnly => "read_only",
            Self::ProofEvidenceOnly => "proof_evidence_only",
            Self::ExportOnly => "export_only",
            Self::MutationForbidden => "mutation_forbidden",
            Self::MutationRequiresGrant => "mutation_requires_grant",
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn authority_lane_keywords_are_stable() {
        for (lane, keyword) in [
            (AuthorityLaneV2::FrontendChecked, "frontend_checked"),
            (
                AuthorityLaneV2::CertifiedRequirementValidated,
                "certified_requirement_validated",
            ),
        ] {
            assert_eq!(lane.keyword(), keyword);
        }
    }

    #[test]
    fn determinism_and_evidence_keywords_are_stable() {
        for (lane, keyword) in [
            (EvaluationDeterminismV2::Deterministic, "deterministic"),
            (
                EvaluationDeterminismV2::NonDeterministic,
                "non_deterministic",
            ),
        ] {
            assert_eq!(lane.keyword(), keyword);
        }
        for (evidence, keyword) in [
            (EvidenceRequirementV2::None, "none"),
            (EvidenceRequirementV2::SourceLocation, "source_location"),
            (EvidenceRequirementV2::WitnessPayload, "witness_payload"),
            (EvidenceRequirementV2::ProofTree, "proof_tree"),
        ] {
            assert_eq!(evidence.keyword(), keyword);
        }
    }

    #[test]
    fn effect_class_keywords_are_stable() {
        for (effect, keyword) in [
            (EffectClassV2::ReadOnly, "read_only"),
            (EffectClassV2::ProofEvidenceOnly, "proof_evidence_only"),
            (EffectClassV2::ExportOnly, "export_only"),
            (EffectClassV2::MutationForbidden, "mutation_forbidden"),
        ] {
            assert_eq!(effect.keyword(), keyword);
        }
    }
}
