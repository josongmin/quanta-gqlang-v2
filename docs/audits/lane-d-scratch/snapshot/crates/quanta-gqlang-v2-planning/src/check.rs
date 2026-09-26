//! The validated CHECK planning input (ticket 07, item contract matrix 1).
//!
//! Sema resolves a CHECK into [`ResolvedCheckV2`]: the exact rule reference, the
//! optional scope and pattern reference, the optional inline semantic confirm
//! FIND, and the authored detection/confirm/emit modes. Planning owns the
//! COHERENCE of that option set, because it is a planning policy question, not a
//! name-resolution question:
//!
//! 1. `pattern` detection mode forbids a semantic confirm input;
//! 2. `semantic` detection mode requires the semantic confirm input and does not
//!    accept a pattern input;
//! 3. `hybrid` detection mode requires BOTH the authored pattern input and the
//!    semantic confirm input;
//! 4. the nested graph contract supplied by Logic must name the exact identity
//!    the resolved CHECK carries — a same-document wrong-body transplant rejects.
//!
//! Rejection order inside one mode is fixed, so a CHECK with several defects
//! always reports the same first one. The sealed payload (`mode`/`confirm`/
//! `emit`, the resolved references, and the nested witness provenance) is
//! retained verbatim: the plan derives its evidence/render/effect policy from it
//! and never re-reads source options.

use quanta_gqlang_v2_logic::GraphQueryContractV2;
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, ConfirmModeV1, DetectionModeV1, EmitModeV1, ResolvedCatalogReferenceV2,
    ResolvedCheckV2, ResolvedGraphUseV2, SemanticItemIdV2,
};

use crate::diagnostic::{CheckInvalidReasonV2, PlanningDiagnosticV1};
use crate::policy::GraphWitnessRetentionV2;

/// The exact CHECK policy a plan retains.
///
/// Every field is copied from the sealed CHECK contract; nothing is re-derived
/// from source spelling or display text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckPolicyV2 {
    mode: DetectionModeV1,
    confirm: ConfirmModeV1,
    emit: EmitModeV1,
    rule: ResolvedCatalogReferenceV2,
    scope: Option<ResolvedGraphUseV2>,
    pattern: Option<ResolvedGraphUseV2>,
    nested_graph: Option<GraphWitnessRetentionV2>,
}

impl CheckPolicyV2 {
    /// The authored detection mode.
    #[must_use]
    pub const fn mode(&self) -> DetectionModeV1 {
        self.mode
    }

    /// The authored confirmation policy.
    #[must_use]
    pub const fn confirm(&self) -> ConfirmModeV1 {
        self.confirm
    }

    /// The authored emission target.
    #[must_use]
    pub const fn emit(&self) -> EmitModeV1 {
        self.emit
    }

    /// The resolved rule reference.
    #[must_use]
    pub const fn rule(&self) -> &ResolvedCatalogReferenceV2 {
        &self.rule
    }

    /// The optional resolved scope reference.
    #[must_use]
    pub const fn scope(&self) -> Option<&ResolvedGraphUseV2> {
        self.scope.as_ref()
    }

    /// The optional resolved pattern reference.
    #[must_use]
    pub const fn pattern(&self) -> Option<&ResolvedGraphUseV2> {
        self.pattern.as_ref()
    }

    /// The exact nested semantic-confirm witness provenance, when present.
    #[must_use]
    pub const fn nested_graph(&self) -> Option<&GraphWitnessRetentionV2> {
        self.nested_graph.as_ref()
    }
}

/// A sealed, policy-coherent CHECK planning input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckContractV2 {
    item_id: SemanticItemIdV2,
    location: CheckedLocatedSpanV2,
    nested_graph: Option<GraphQueryContractV2>,
    policy: CheckPolicyV2,
}

impl CheckContractV2 {
    /// Validate the sealed CHECK's option set and retain its exact policy.
    ///
    /// `nested` is Logic's sealed contract for the resolved inline FIND the
    /// CHECK carries as its semantic confirm input; the caller builds it with
    /// [`GraphQueryContractV2::from_resolved`]. A nested contract must equal the
    /// complete contract derived from that resolved FIND (identity, kind,
    /// location, references, depth, and witness policy), so a same-identity
    /// payload transplant cannot silently plan.
    ///
    /// # Errors
    ///
    /// Returns [`PlanningDiagnosticV1::InvalidCheck`] with the first defect in
    /// the fixed order documented on this module.
    pub fn from_resolved(
        check: &ResolvedCheckV2,
        nested: Option<GraphQueryContractV2>,
    ) -> Result<Self, PlanningDiagnosticV1> {
        let location = check.location().clone();
        let reject = |reason: CheckInvalidReasonV2| PlanningDiagnosticV1::InvalidCheck {
            reason,
            span: location.clone(),
        };

        // 1. The nested contract must be present exactly when the resolved CHECK
        //    carries a semantic confirm input, and must equal the exact sealed
        //    contract derived from that item rather than merely sharing its ID.
        match (check.nested_graph(), &nested) {
            (Some(resolved), Some(contract)) => {
                let expected = GraphQueryContractV2::from_resolved(resolved);
                if expected.as_ref().ok() != Some(contract) {
                    return Err(reject(CheckInvalidReasonV2::NestedGraphIdentityMismatch));
                }
            }
            (Some(_), None) => return Err(reject(CheckInvalidReasonV2::MissingNestedGraph)),
            (None, Some(_)) => return Err(reject(CheckInvalidReasonV2::UnexpectedNestedGraph)),
            (None, None) => {}
        }

        // 2. The detection mode's option matrix, in a fixed order per mode.
        let semantic_confirm_present = nested.is_some();
        match check.mode() {
            DetectionModeV1::PatternOnly => {
                if semantic_confirm_present {
                    return Err(reject(CheckInvalidReasonV2::SemanticConfirmInPatternMode));
                }
            }
            DetectionModeV1::SemanticOnly => {
                if !semantic_confirm_present {
                    return Err(reject(CheckInvalidReasonV2::MissingSemanticConfirm));
                }
                if check.pattern().is_some() {
                    return Err(reject(CheckInvalidReasonV2::PatternInputInSemanticMode));
                }
            }
            DetectionModeV1::Hybrid => {
                if check.pattern().is_none() {
                    return Err(reject(CheckInvalidReasonV2::MissingHybridPatternInput));
                }
                if !semantic_confirm_present {
                    return Err(reject(CheckInvalidReasonV2::MissingHybridSemanticInput));
                }
            }
        }

        let policy = CheckPolicyV2 {
            mode: check.mode(),
            confirm: check.confirm(),
            emit: check.emit(),
            rule: check.rule().clone(),
            scope: check.scope().cloned(),
            pattern: check.pattern().cloned(),
            nested_graph: nested.as_ref().map(GraphWitnessRetentionV2::from_contract),
        };
        Ok(Self {
            item_id: check.item_id(),
            location,
            nested_graph: nested,
            policy,
        })
    }

    /// Semantic item identity of the CHECK this contract seals.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// Checked location of the originating CHECK item.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }

    /// The sealed nested semantic-confirm graph contract, when present.
    #[must_use]
    pub const fn nested_graph(&self) -> Option<&GraphQueryContractV2> {
        self.nested_graph.as_ref()
    }

    /// Identity of the nested semantic-confirm graph, when present.
    #[must_use]
    pub fn nested_graph_item_id(&self) -> Option<SemanticItemIdV2> {
        self.nested_graph
            .as_ref()
            .map(GraphQueryContractV2::item_id)
    }

    /// The exact retained CHECK policy.
    #[must_use]
    pub const fn policy(&self) -> &CheckPolicyV2 {
        &self.policy
    }

    /// The resolved rule reference.
    #[must_use]
    pub const fn rule(&self) -> &ResolvedCatalogReferenceV2 {
        self.policy.rule()
    }

    /// The optional resolved scope reference.
    #[must_use]
    pub const fn scope(&self) -> Option<&ResolvedGraphUseV2> {
        self.policy.scope()
    }

    /// The optional resolved pattern reference.
    #[must_use]
    pub const fn pattern(&self) -> Option<&ResolvedGraphUseV2> {
        self.policy.pattern()
    }

    /// The authored detection mode.
    #[must_use]
    pub const fn mode(&self) -> DetectionModeV1 {
        self.policy.mode()
    }

    /// The authored confirmation policy.
    #[must_use]
    pub const fn confirm(&self) -> ConfirmModeV1 {
        self.policy.confirm()
    }

    /// The authored emission target.
    #[must_use]
    pub const fn emit(&self) -> EmitModeV1 {
        self.policy.emit()
    }
}
