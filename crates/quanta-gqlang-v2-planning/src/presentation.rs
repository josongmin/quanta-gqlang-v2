//! The validated presentation planning input (ticket 07, item contract matrix 2
//! and 4-6).
//!
//! Sema resolves one presentation verb into
//! [`ResolvedPresentationKindV2`]: EXPLAIN with its disjoint resolved target and
//! evidence/truth modes, SHOW with a closed category, DESCRIBE with a resolved
//! entity, SAMPLE with mode/positive limit/optional scope, COMPARE with both
//! resolved targets and a metric, and EXPORT with a resolved source, format, and
//! optional profile. Planning owns the COHERENCE of the EXPLAIN evidence/truth
//! pair, because that is a policy question, and retains every other payload
//! verbatim so SAMPLE mode/limit, COMPARE metric/asset identities, and EXPORT
//! format/profile survive planning exactly.
//!
//! Evidence and truth are judged INDEPENDENTLY: a `Trace` or `Auto` evidence mode
//! is meaningful under every truth mode, while a counterexample cannot CONFIRM a
//! claim and a citation cannot REFUTE one.

use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, EvidenceModeV1, ResolvedPresentationKindV2, ResolvedPresentationV2,
    SemanticItemIdV2, TruthModeV1,
};

use crate::diagnostic::{PlanningDiagnosticV1, PresentationInvalidReasonV2};

/// A sealed, policy-coherent presentation planning input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresentationContractV2 {
    item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    kind: ResolvedPresentationKindV2,
}

impl PresentationContractV2 {
    /// Validate the sealed presentation's policy and retain its exact payload.
    ///
    /// # Errors
    ///
    /// Returns [`PlanningDiagnosticV1::InvalidPresentation`] when the EXPLAIN
    /// evidence and truth modes are incompatible.
    pub fn from_resolved(
        presentation: &ResolvedPresentationV2,
    ) -> Result<Self, PlanningDiagnosticV1> {
        if let ResolvedPresentationKindV2::Explain {
            evidence, truth, ..
        } = presentation.kind()
        {
            if !evidence_truth_is_coherent(*evidence, *truth) {
                return Err(PlanningDiagnosticV1::InvalidPresentation {
                    reason: PresentationInvalidReasonV2::UnsupportedExplainEvidenceTruth,
                    span: presentation.location().clone(),
                });
            }
        }
        Ok(Self {
            item_id: presentation.item_id(),
            location: presentation.location().clone(),
            kind: presentation.kind().clone(),
        })
    }

    /// Semantic item identity of the presentation this contract seals.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// Checked location of the originating presentation item.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }

    /// The exact sealed presentation payload, retained verbatim.
    #[must_use]
    pub const fn kind(&self) -> &ResolvedPresentationKindV2 {
        &self.kind
    }
}

/// Whether an EXPLAIN evidence mode can carry a truth mode.
///
/// Evidence is judged independently of truth: `Auto` lets the frontend select
/// the evidence the truth mode needs, and `Trace` is a concrete execution
/// artifact that can both support and contradict a claim. The two incoherent
/// pairs are a counterexample used to CONFIRM (falsifying evidence cannot
/// establish the claim) and a citation used to REFUTE (documentary evidence does
/// not establish falsity).
const fn evidence_truth_is_coherent(evidence: EvidenceModeV1, truth: TruthModeV1) -> bool {
    match truth {
        TruthModeV1::Inspect => true,
        TruthModeV1::Confirm => !matches!(evidence, EvidenceModeV1::Counterexample),
        TruthModeV1::Refute => !matches!(evidence, EvidenceModeV1::Citation),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_evidence_truth_pair_is_judged_independently() {
        const EVIDENCE: [EvidenceModeV1; 4] = [
            EvidenceModeV1::Auto,
            EvidenceModeV1::Trace,
            EvidenceModeV1::Citation,
            EvidenceModeV1::Counterexample,
        ];
        const TRUTH: [TruthModeV1; 3] = [
            TruthModeV1::Inspect,
            TruthModeV1::Confirm,
            TruthModeV1::Refute,
        ];
        let mut legal = 0_u8;
        let mut illegal = 0_u8;
        for evidence in EVIDENCE {
            for truth in TRUTH {
                if evidence_truth_is_coherent(evidence, truth) {
                    legal += 1;
                } else {
                    illegal += 1;
                }
            }
        }
        assert_eq!(legal, 10, "ten of the twelve pairs are coherent");
        assert_eq!(illegal, 2, "exactly the two documented pairs reject");
        assert!(!evidence_truth_is_coherent(
            EvidenceModeV1::Counterexample,
            TruthModeV1::Confirm
        ));
        assert!(!evidence_truth_is_coherent(
            EvidenceModeV1::Citation,
            TruthModeV1::Refute
        ));
        assert!(evidence_truth_is_coherent(
            EvidenceModeV1::Counterexample,
            TruthModeV1::Refute
        ));
        assert!(evidence_truth_is_coherent(
            EvidenceModeV1::Citation,
            TruthModeV1::Confirm
        ));
    }
}
