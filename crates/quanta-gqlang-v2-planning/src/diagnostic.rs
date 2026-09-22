//! Typed, fail-closed planning diagnostics for the sealed V2 input model
//! (ticket 07, decisions `D10` / `D16`).
//!
//! The V2 planner consumes ALREADY-SEALED semantic contracts: the relational arm
//! is ticket 05's checked lowering product, the graph/rule arms are ticket 06's
//! validated logic contracts, and the CHECK/presentation/composition arms are
//! this crate's validated wrappers. There is therefore no caller-selected
//! result/effect/truth mix left to reject, and the V1 `UnsupportedMixReasonV1`
//! vocabulary is deleted with it: every rejection below is a property of the
//! sealed inputs, of their pairing with the document, or of the document's
//! immutable semantic-step budget.
//!
//! Each variant renders a STABLE `code()` (dot-namespaced under `planning.`,
//! except the two relay variants, which surface the originating logic code
//! verbatim) and a stable human `message()` so CLI/LSP/tests assert exact text
//! (`D22`). Every location is a registry-proven
//! [`CheckedLocatedSpanV2`].

use quanta_gqlang_v2_logic::LogicDiagnosticV1;
use quanta_gqlang_v2_sema::CheckedLocatedSpanV2;

use crate::item::NoPlanReasonV2;

/// Why a sealed input carrier does not belong to the supplied document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputDocumentMismatchReasonV2 {
    /// The document holds no item with the carrier's identity.
    Absent,
    /// The document holds that identity, but as a different item kind.
    WrongKind,
    /// The document holds a same-kind item whose sealed facts differ from the
    /// carrier's (a transplanted or forged identity, including a rule program
    /// whose retained budget is not the document's).
    IdentityMismatch,
}

impl InputDocumentMismatchReasonV2 {
    /// Stable code suffix for this reason.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::WrongKind => "wrong_kind",
            Self::IdentityMismatch => "identity_mismatch",
        }
    }

    /// Stable human name for this reason.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Absent => "absent from the document",
            Self::WrongKind => "a different item kind in the document",
            Self::IdentityMismatch => "a contradictory sealed identity in the document",
        }
    }
}

/// Why a sealed CHECK contract has no coherent planning policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CheckInvalidReasonV2 {
    /// The supplied nested graph contract names an item the resolved CHECK does
    /// not.
    NestedGraphIdentityMismatch,
    /// A nested graph contract was supplied, but the resolved CHECK carries no
    /// semantic confirm input.
    UnexpectedNestedGraph,
    /// The resolved CHECK carries a semantic confirm input, but no nested graph
    /// contract was supplied.
    MissingNestedGraph,
    /// `pattern` detection mode forbids a semantic confirm input.
    SemanticConfirmInPatternMode,
    /// `semantic` detection mode requires a semantic confirm input.
    MissingSemanticConfirm,
    /// `semantic` detection mode forbids a pattern input.
    PatternInputInSemanticMode,
    /// `hybrid` detection mode requires the authored pattern input.
    MissingHybridPatternInput,
    /// `hybrid` detection mode requires the semantic confirm input.
    MissingHybridSemanticInput,
}

impl CheckInvalidReasonV2 {
    /// Stable code suffix for this reason.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::NestedGraphIdentityMismatch => "nested_graph_identity_mismatch",
            Self::UnexpectedNestedGraph => "unexpected_nested_graph",
            Self::MissingNestedGraph => "missing_nested_graph",
            Self::SemanticConfirmInPatternMode => "semantic_confirm_in_pattern_mode",
            Self::MissingSemanticConfirm => "missing_semantic_confirm",
            Self::PatternInputInSemanticMode => "pattern_input_in_semantic_mode",
            Self::MissingHybridPatternInput => "missing_hybrid_pattern_input",
            Self::MissingHybridSemanticInput => "missing_hybrid_semantic_input",
        }
    }
}

/// Why a sealed presentation contract has no coherent planning policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentationInvalidReasonV2 {
    /// The EXPLAIN evidence/truth pair cannot hold: a counterexample cannot
    /// confirm a claim, and a citation cannot refute one.
    UnsupportedExplainEvidenceTruth,
}

impl PresentationInvalidReasonV2 {
    /// Stable code suffix for this reason.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnsupportedExplainEvidenceTruth => "unsupported_explain_evidence_truth",
        }
    }
}

/// Why a checked composition plan input is internally inconsistent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositionInvalidReasonV2 {
    /// A composition always has at least one ordered producer.
    EmptyProducers,
    /// A producer's ordinal is not its authored position.
    ProducerOrdinalMismatch,
    /// A producer's identity or relation output disagrees with the sealed
    /// binding at that ordinal.
    ProducerIdentityMismatch,
    /// Two producers claim the same identity.
    DuplicateProducer,
    /// The `WITH` binding is not exactly one of the ordered producers.
    WithTargetNotAProducer,
    /// The body input's identity is not the sealed composition body's identity.
    BodyIdentityMismatch,
    /// The body input's kind disagrees with the sealed composition body kind.
    BodyKindMismatch,
}

impl CompositionInvalidReasonV2 {
    /// Stable code suffix for this reason.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyProducers => "empty_producers",
            Self::ProducerOrdinalMismatch => "producer_ordinal_mismatch",
            Self::ProducerIdentityMismatch => "producer_identity_mismatch",
            Self::DuplicateProducer => "duplicate_producer",
            Self::WithTargetNotAProducer => "with_target_not_a_producer",
            Self::BodyIdentityMismatch => "body_identity_mismatch",
            Self::BodyKindMismatch => "body_kind_mismatch",
        }
    }
}

/// A typed planning diagnostic.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanningDiagnosticV1 {
    /// A FIND graph contract failed logic's legality/boundedness validation.
    /// The exact logic diagnostic is carried so the planner surfaces logic's
    /// code/message/location verbatim rather than inventing a parallel reason.
    IllegalGraphQuery {
        /// The originating logic diagnostic.
        logic: LogicDiagnosticV1,
    },
    /// The document rule program could not be stratified or its graph budget was
    /// exhausted. The exact logic diagnostic is relayed unchanged.
    IllegalRuleProgram {
        /// The originating logic diagnostic.
        logic: LogicDiagnosticV1,
    },
    /// The semantic-step traversal refused a charge that would exceed the
    /// document's sealed `max_semantic_steps`. Charging happens BEFORE the work,
    /// so the exact sealed limit succeeds and limit+1 rejects here at the
    /// refused node's checked location.
    SemanticStepBudgetExceeded {
        /// The exact sealed limit that would have been exceeded.
        limit: u32,
        /// Checked location of the node whose charge was refused.
        span: CheckedLocatedSpanV2,
    },
    /// A sealed input carrier does not belong to the supplied document.
    InputDocumentMismatch {
        /// Which mismatch was observed.
        reason: InputDocumentMismatchReasonV2,
        /// Checked location of the carrier the mismatch is attributed to.
        span: CheckedLocatedSpanV2,
    },
    /// A sealed CHECK contract has no coherent planning policy.
    InvalidCheck {
        /// Why the CHECK has no coherent policy.
        reason: CheckInvalidReasonV2,
        /// Checked location of the CHECK item.
        span: CheckedLocatedSpanV2,
    },
    /// A sealed presentation contract has no coherent planning policy.
    InvalidPresentation {
        /// Why the presentation has no coherent policy.
        reason: PresentationInvalidReasonV2,
        /// Checked location of the presentation item.
        span: CheckedLocatedSpanV2,
    },
    /// A checked composition plan input is internally inconsistent.
    InvalidComposition {
        /// Why the composition input is inconsistent.
        reason: CompositionInvalidReasonV2,
        /// Checked location of the composition item.
        span: CheckedLocatedSpanV2,
    },
    /// A non-plan input reached the plan entry. The checked
    /// [`NoPlanReasonV2`] is carried so a non-plan outcome is never silently
    /// skipped and never confused with a plan.
    NoPlan {
        /// Why the input has no plan.
        reason: NoPlanReasonV2,
        /// Checked location of the item.
        span: CheckedLocatedSpanV2,
    },
}

impl PlanningDiagnosticV1 {
    /// The stable diagnostic code, shared by CLI/LSP/tests.
    ///
    /// Codes are dot-namespaced under `planning.` and never change spelling,
    /// except the two relay variants, which return the originating logic code
    /// verbatim (logic owns graph-query and rule-program legality).
    #[must_use]
    pub fn code(&self) -> String {
        match self {
            // Legality is logic's; relay its stable code unchanged.
            Self::IllegalGraphQuery { logic } | Self::IllegalRuleProgram { logic } => logic.code(),
            Self::SemanticStepBudgetExceeded { .. } => {
                "planning.budget.semantic_steps_exceeded".to_owned()
            }
            Self::InputDocumentMismatch { reason, .. } => {
                format!("planning.input_document_mismatch.{}", reason.code())
            }
            Self::InvalidCheck { reason, .. } => {
                format!("planning.invalid_check.{}", reason.code())
            }
            Self::InvalidPresentation { reason, .. } => {
                format!("planning.invalid_presentation.{}", reason.code())
            }
            Self::InvalidComposition { reason, .. } => {
                format!("planning.invalid_composition.{}", reason.code())
            }
            Self::NoPlan { reason, .. } => match reason.catalog_kind() {
                Some(kind) => format!("planning.no_plan.{}.{}", reason.code(), kind.code()),
                None => format!("planning.no_plan.{}", reason.code()),
            },
        }
    }

    /// The checked location the diagnostic points at.
    #[must_use]
    pub const fn span(&self) -> &CheckedLocatedSpanV2 {
        match self {
            Self::IllegalGraphQuery { logic } | Self::IllegalRuleProgram { logic } => logic.span(),
            Self::SemanticStepBudgetExceeded { span, .. }
            | Self::InputDocumentMismatch { span, .. }
            | Self::InvalidCheck { span, .. }
            | Self::InvalidPresentation { span, .. }
            | Self::InvalidComposition { span, .. }
            | Self::NoPlan { span, .. } => span,
        }
    }

    /// A stable human-readable message. Text is part of the contract: tests
    /// assert it verbatim.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            // Relay logic's verbatim message.
            Self::IllegalGraphQuery { logic } | Self::IllegalRuleProgram { logic } => {
                logic.message()
            }
            Self::SemanticStepBudgetExceeded { limit, .. } => format!(
                "semantic planning exceeded the document's sealed max_semantic_steps budget of \
                 {limit}"
            ),
            Self::InputDocumentMismatch { reason, .. } => format!(
                "planning input is {} : the sealed carrier cannot be planned against this \
                 document",
                reason.name()
            ),
            Self::InvalidCheck { reason, .. } => match reason {
                CheckInvalidReasonV2::NestedGraphIdentityMismatch => {
                    "CHECK nested graph contract names an item the resolved CHECK does not"
                        .to_owned()
                }
                CheckInvalidReasonV2::UnexpectedNestedGraph => {
                    "CHECK carries no semantic confirm input, so no nested graph contract is \
                     admissible"
                        .to_owned()
                }
                CheckInvalidReasonV2::MissingNestedGraph => {
                    "CHECK semantic confirm input has no sealed nested graph contract".to_owned()
                }
                CheckInvalidReasonV2::SemanticConfirmInPatternMode => {
                    "CHECK pattern detection mode forbids a semantic confirm input".to_owned()
                }
                CheckInvalidReasonV2::MissingSemanticConfirm => {
                    "CHECK semantic detection mode requires a semantic confirm input".to_owned()
                }
                CheckInvalidReasonV2::PatternInputInSemanticMode => {
                    "CHECK semantic detection mode does not accept a pattern input".to_owned()
                }
                CheckInvalidReasonV2::MissingHybridPatternInput => {
                    "CHECK hybrid detection mode requires the authored pattern input".to_owned()
                }
                CheckInvalidReasonV2::MissingHybridSemanticInput => {
                    "CHECK hybrid detection mode requires the semantic confirm input".to_owned()
                }
            },
            Self::InvalidPresentation { reason, .. } => match reason {
                PresentationInvalidReasonV2::UnsupportedExplainEvidenceTruth => {
                    "EXPLAIN evidence and truth modes are incompatible: a counterexample cannot \
                     confirm a claim and a citation cannot refute one"
                        .to_owned()
                }
            },
            Self::InvalidComposition { reason, .. } => match reason {
                CompositionInvalidReasonV2::EmptyProducers => {
                    "composition plan input has no ordered producer".to_owned()
                }
                CompositionInvalidReasonV2::ProducerOrdinalMismatch => {
                    "composition producer ordinal is not its authored position".to_owned()
                }
                CompositionInvalidReasonV2::ProducerIdentityMismatch => {
                    "composition producer identity or relation output disagrees with its sealed \
                     binding"
                        .to_owned()
                }
                CompositionInvalidReasonV2::DuplicateProducer => {
                    "composition plan input repeats a producer identity".to_owned()
                }
                CompositionInvalidReasonV2::WithTargetNotAProducer => {
                    "composition WITH target is not exactly one of the ordered producers".to_owned()
                }
                CompositionInvalidReasonV2::BodyIdentityMismatch => {
                    "composition body input identity is not the sealed body identity".to_owned()
                }
                CompositionInvalidReasonV2::BodyKindMismatch => {
                    "composition body input kind disagrees with the sealed body kind".to_owned()
                }
            },
            Self::NoPlan { reason, .. } => format!("no plan: {}", reason.message()),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn catalog_declaration_codes_name_the_exact_payload() {
        for (kind, code) in [
            (
                crate::item::CatalogDeclarationKindV2::View,
                "planning.no_plan.catalog_declaration.view",
            ),
            (
                crate::item::CatalogDeclarationKindV2::Model,
                "planning.no_plan.catalog_declaration.model",
            ),
            (
                crate::item::CatalogDeclarationKindV2::Pack,
                "planning.no_plan.catalog_declaration.pack",
            ),
            (
                crate::item::CatalogDeclarationKindV2::Profile,
                "planning.no_plan.catalog_declaration.profile",
            ),
            (
                crate::item::CatalogDeclarationKindV2::Gate,
                "planning.no_plan.catalog_declaration.gate",
            ),
            (
                crate::item::CatalogDeclarationKindV2::RuleBinding,
                "planning.no_plan.catalog_declaration.rule_binding",
            ),
        ] {
            let span = span();
            let diagnostic = PlanningDiagnosticV1::NoPlan {
                reason: NoPlanReasonV2::CatalogDeclaration { kind },
                span,
            };
            assert_eq!(diagnostic.code(), code);
        }
    }

    #[test]
    fn every_reason_code_is_stable() {
        for (reason, code) in [
            (
                InputDocumentMismatchReasonV2::Absent,
                "planning.input_document_mismatch.absent",
            ),
            (
                InputDocumentMismatchReasonV2::WrongKind,
                "planning.input_document_mismatch.wrong_kind",
            ),
            (
                InputDocumentMismatchReasonV2::IdentityMismatch,
                "planning.input_document_mismatch.identity_mismatch",
            ),
        ] {
            assert_eq!(
                PlanningDiagnosticV1::InputDocumentMismatch {
                    reason,
                    span: span()
                }
                .code(),
                code
            );
        }
        for (reason, suffix) in [
            (
                CheckInvalidReasonV2::NestedGraphIdentityMismatch,
                "nested_graph_identity_mismatch",
            ),
            (
                CheckInvalidReasonV2::UnexpectedNestedGraph,
                "unexpected_nested_graph",
            ),
            (
                CheckInvalidReasonV2::MissingNestedGraph,
                "missing_nested_graph",
            ),
            (
                CheckInvalidReasonV2::SemanticConfirmInPatternMode,
                "semantic_confirm_in_pattern_mode",
            ),
            (
                CheckInvalidReasonV2::MissingSemanticConfirm,
                "missing_semantic_confirm",
            ),
            (
                CheckInvalidReasonV2::PatternInputInSemanticMode,
                "pattern_input_in_semantic_mode",
            ),
            (
                CheckInvalidReasonV2::MissingHybridPatternInput,
                "missing_hybrid_pattern_input",
            ),
            (
                CheckInvalidReasonV2::MissingHybridSemanticInput,
                "missing_hybrid_semantic_input",
            ),
        ] {
            assert_eq!(
                PlanningDiagnosticV1::InvalidCheck {
                    reason,
                    span: span()
                }
                .code(),
                format!("planning.invalid_check.{suffix}")
            );
        }
        for (reason, suffix) in [
            (
                CompositionInvalidReasonV2::EmptyProducers,
                "empty_producers",
            ),
            (
                CompositionInvalidReasonV2::ProducerOrdinalMismatch,
                "producer_ordinal_mismatch",
            ),
            (
                CompositionInvalidReasonV2::ProducerIdentityMismatch,
                "producer_identity_mismatch",
            ),
            (
                CompositionInvalidReasonV2::DuplicateProducer,
                "duplicate_producer",
            ),
            (
                CompositionInvalidReasonV2::WithTargetNotAProducer,
                "with_target_not_a_producer",
            ),
            (
                CompositionInvalidReasonV2::BodyIdentityMismatch,
                "body_identity_mismatch",
            ),
            (
                CompositionInvalidReasonV2::BodyKindMismatch,
                "body_kind_mismatch",
            ),
        ] {
            assert_eq!(
                PlanningDiagnosticV1::InvalidComposition {
                    reason,
                    span: span()
                }
                .code(),
                format!("planning.invalid_composition.{suffix}")
            );
        }
        assert_eq!(
            PlanningDiagnosticV1::SemanticStepBudgetExceeded {
                limit: 8,
                span: span()
            }
            .code(),
            "planning.budget.semantic_steps_exceeded"
        );
        assert_eq!(
            PlanningDiagnosticV1::NoPlan {
                reason: NoPlanReasonV2::ValueDeclaration,
                span: span()
            }
            .code(),
            "planning.no_plan.value_declaration"
        );
        assert_eq!(
            PlanningDiagnosticV1::NoPlan {
                reason: NoPlanReasonV2::MetaActivation,
                span: span()
            }
            .code(),
            "planning.no_plan.meta_activation"
        );
    }

    #[test]
    fn budget_message_and_span_are_exact() {
        let span = span();
        let diagnostic = PlanningDiagnosticV1::SemanticStepBudgetExceeded {
            limit: 8,
            span: span.clone(),
        };
        assert_eq!(
            diagnostic.message(),
            "semantic planning exceeded the document's sealed max_semantic_steps budget of 8"
        );
        assert_eq!(diagnostic.span(), &span);
    }

    #[test]
    fn reason_name_codes_are_stable() {
        assert_eq!(
            InputDocumentMismatchReasonV2::Absent.name(),
            "absent from the document"
        );
        assert_eq!(
            InputDocumentMismatchReasonV2::WrongKind.name(),
            "a different item kind in the document"
        );
        assert_eq!(
            InputDocumentMismatchReasonV2::IdentityMismatch.name(),
            "a contradictory sealed identity in the document"
        );
        assert_eq!(
            NoPlanReasonV2::EmptyRuleProgram.code(),
            "empty_rule_program"
        );
        assert_eq!(
            NoPlanReasonV2::CatalogDeclaration {
                kind: crate::item::CatalogDeclarationKindV2::Model
            }
            .catalog_kind(),
            Some(crate::item::CatalogDeclarationKindV2::Model)
        );
        assert_eq!(NoPlanReasonV2::ValueDeclaration.catalog_kind(), None);
    }

    #[expect(
        clippy::expect_used,
        reason = "owner rail asserts on one fixture span it minted in bounds"
    )]
    fn span() -> CheckedLocatedSpanV2 {
        quanta_gqlang_v2_sema::test_support::reference_checked_span_v2(3, 11)
            .expect("fixture checked location is in bounds")
    }
}
