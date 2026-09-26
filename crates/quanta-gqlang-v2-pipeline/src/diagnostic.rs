//! Pipeline diagnostic surface (ticket 07, lane W4-B).
//!
//! The shared compiler service defines no private diagnostic wrapper: every
//! stage failure is carried by the Sema-owned
//! [`FrontendTransactionErrorV2`], which separates a source-less authority
//! session failure from a source-anchored document failure. This module:
//!
//! 1. uses the Sema-owned transaction vocabulary the facade and front doors match on
//!    ([`FrontendTransactionErrorV2`], [`FrontendFailureV2`],
//!    [`FrontendDiagnosticV2`], [`FrontendSeverityV2`], [`FrontendStageV2`],
//!    [`quanta_gqlang_v2_sema::FrontendAuthorityErrorV2`]);
//! 2. owns the projection of each owner diagnostic (surface, algebra, logic,
//!    planning) into a [`FrontendDiagnosticV2`] that preserves the owner's
//!    stable code, exact checked span, and message;
//! 3. owns exact-key assembly codes ([`DocumentAssemblyCodeV2`]);
//!    open-analysis codes are emitted by the sibling compile module.
//!
//! ## Owner code authority
//!
//! [`FrontendDiagnosticV2::new`] requires a `&'static str` code. Each owner
//! exposes that full stable code directly; pipeline bridge functions delegate
//! to it. The independent literal tests pin the external vocabulary.

use quanta_gqlang_v2_algebra::{AlgebraDiagnosticV2, AlgebraFailureKindV2};
use quanta_gqlang_v2_logic::LogicDiagnosticV1;
use quanta_gqlang_v2_planning::{NoPlanReasonV2, PlanningDiagnosticV1};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, FrontendDiagnosticV2, FrontendFailureV2, FrontendSeverityV2,
    FrontendStageV2, FrontendTransactionErrorV2, LocatedSpanV2,
};
use quanta_gqlang_v2_surface::{ParseErrorCodeV1, SurfaceDiagnosticV2};

/// Pipeline-owned stable exact-key assembly codes.
///
/// Each code is minted only by this crate's document assembler; the
/// vocabulary is closed so a caller can match every pipeline failure without
/// parsing diagnostic text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DocumentAssemblyCodeV2 {
    /// An authored item that requires one produced result has none.
    MissingItemResult,
    /// A produced result has no authored item identity.
    ExtraItemResult,
    /// The same authored identity was produced more than once.
    DuplicateItemResult,
    /// A produced result (or authored identity) belongs to another source.
    ForeignItemResult,
    /// A produced result does not match the authored item's closed kind.
    WrongKindItemResult,
    /// The authored order and the produced item map disagree.
    ItemMembershipMismatch,
    /// The document rule program's member set differs from the authored rules.
    RuleMembershipMismatch,
    /// A plan-required input returned a checked non-plan outcome.
    PlanOutcomeMismatch,
    /// A producer's ordinal, identity, or checked output was transplanted.
    CompositionProducerMismatch,
    /// A composition body identity or contract was transplanted.
    CompositionBodyMismatch,
    /// A CHECK's nested graph contract does not carry the named graph identity.
    CheckGraphMismatch,
    /// A catalog or declaration item was planned without an applicable input.
    InapplicableItemResult,
    /// An executable item references an expression identity the sealed
    /// document table does not carry.
    DanglingExecutionReference,
    /// An executable expression's closed ID has no registered spelling.
    UnregisteredExecutionSymbol,
}

impl DocumentAssemblyCodeV2 {
    /// The stable, machine-readable code.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingItemResult => "pipeline.document.missing_item_result",
            Self::ExtraItemResult => "pipeline.document.extra_item_result",
            Self::DuplicateItemResult => "pipeline.document.duplicate_item_result",
            Self::ForeignItemResult => "pipeline.document.foreign_item_result",
            Self::WrongKindItemResult => "pipeline.document.wrong_kind_item_result",
            Self::ItemMembershipMismatch => "pipeline.document.item_membership_mismatch",
            Self::RuleMembershipMismatch => "pipeline.document.rule_membership_mismatch",
            Self::PlanOutcomeMismatch => "pipeline.document.plan_outcome_mismatch",
            Self::CompositionProducerMismatch => "pipeline.document.composition_producer_mismatch",
            Self::CompositionBodyMismatch => "pipeline.document.composition_body_mismatch",
            Self::CheckGraphMismatch => "pipeline.document.check_graph_mismatch",
            Self::InapplicableItemResult => "pipeline.document.inapplicable_item_result",
            Self::DanglingExecutionReference => "pipeline.document.dangling_execution_reference",
            Self::UnregisteredExecutionSymbol => "pipeline.document.unregistered_execution_symbol",
        }
    }
}

/// The pipeline stage a diagnostic originated from.
///
/// Display vocabulary for owner stages. Code prefixes identify code authority,
/// not necessarily the stage reporting it: planning may relay a logic code.
/// Production transaction diagnostics carry the Sema-owned `FrontendStageV2`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PipelineStageV2 {
    /// Surface parse / lex.
    Parse,
    /// Checked HIR document integrity and the semantic transaction.
    Sema,
    /// Semantic algebra lowering.
    Algebra,
    /// Rule/graph logic contracts.
    Logic,
    /// Semantic planning.
    Planning,
    /// Pipeline-owned exact-key document assembly.
    Assembly,
}

impl PipelineStageV2 {
    /// The stable stage tag.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Sema => "sema",
            Self::Algebra => "algebra",
            Self::Logic => "logic",
            Self::Planning => "planning",
            Self::Assembly => "assembly",
        }
    }
}

/// Order two diagnostics by the Sema-owned canonical key.
///
/// [`FrontendDiagnosticV2`] keeps its `sort_key` private, so the pipeline
/// reproduces the same ordering from public accessors: primary location, then
/// stage, severity, code, and message.
#[must_use]
pub fn diagnostic_sort_key(
    diagnostic: &FrontendDiagnosticV2,
) -> (
    (u32, usize, usize),
    FrontendStageV2,
    FrontendSeverityV2,
    &str,
    &str,
) {
    let primary = diagnostic.primary();
    (
        (
            primary.source_id.raw(),
            primary.span.start,
            primary.span.end,
        ),
        diagnostic.stage(),
        diagnostic.severity(),
        diagnostic.code(),
        diagnostic.message(),
    )
}

/// Stable-sort by the Sema key, then remove adjacent full-struct duplicates.
///
/// The adjacent deduplication is pipeline policy, not a Sema sort guarantee.
/// Equal-key diagnostics retain input order; secondary spans are outside the
/// sort key, so distinct tied diagnostics can separate identical entries.
pub(crate) fn canonicalize_diagnostics(
    mut diagnostics: Vec<FrontendDiagnosticV2>,
) -> Vec<FrontendDiagnosticV2> {
    diagnostics.sort_by(|left, right| diagnostic_sort_key(left).cmp(&diagnostic_sort_key(right)));
    diagnostics.dedup();
    diagnostics
}

// ---------------------------------------------------------------------------
// Owner-code bridge
// ---------------------------------------------------------------------------

/// The full stable code of one algebra failure kind.
#[must_use]
pub fn algebra_code(kind: AlgebraFailureKindV2) -> &'static str {
    kind.full_code()
}

/// The full stable code of one logic diagnostic.
#[must_use]
pub fn logic_code(error: &LogicDiagnosticV1) -> &'static str {
    error.code_str()
}

/// The full stable code of one planning diagnostic.
#[must_use]
pub fn planning_code(error: &PlanningDiagnosticV1) -> &'static str {
    error.code_str()
}

/// The full stable code of one non-plan reason, including its catalog kind.
#[must_use]
pub fn no_plan_code(reason: NoPlanReasonV2) -> &'static str {
    reason.full_code()
}

// ---------------------------------------------------------------------------
// Owner projections
// ---------------------------------------------------------------------------

/// Project one surface parse diagnostic.
///
/// Field mapping: `code_str()` -> `code`, `source_id`+`span` -> `primary`,
/// `message` -> `message`, always `FrontendSeverityV2::Error`. The surface
/// diagnostic carries no secondary/edge spans, so the pipeline never
/// synthesizes one, and it never rewrites the surface code.
#[must_use]
pub(crate) fn surface_failure(error: &SurfaceDiagnosticV2) -> FrontendFailureV2 {
    let stage = if error.code == ParseErrorCodeV1::ResourceExhausted {
        FrontendStageV2::Resource
    } else {
        FrontendStageV2::Integrity
    };
    FrontendFailureV2::one(FrontendDiagnosticV2::new(
        stage,
        error.code_str(),
        FrontendSeverityV2::Error,
        LocatedSpanV2 {
            source_id: error.source_id,
            span: error.span,
        },
        Vec::new(),
        error.message.clone(),
    ))
}

/// Project one surface parse diagnostic as a transaction failure.
#[must_use]
pub(crate) fn surface_transaction_error(error: &SurfaceDiagnosticV2) -> FrontendTransactionErrorV2 {
    FrontendTransactionErrorV2::Document(surface_failure(error))
}

/// Project one algebra lowering diagnostic.
///
/// The algebra diagnostic's client anchor is the failing stage's checked
/// location; when it carries none, the pipeline anchors on the owning item.
#[must_use]
pub(crate) fn algebra_transaction_error(
    error: &AlgebraDiagnosticV2,
    fallback: LocatedSpanV2,
) -> FrontendTransactionErrorV2 {
    document_transaction_error(FrontendDiagnosticV2::new(
        FrontendStageV2::Validation,
        algebra_code(error.kind()),
        FrontendSeverityV2::Error,
        error
            .location()
            .map_or(fallback, CheckedLocatedSpanV2::location),
        Vec::new(),
        error.message(),
    ))
}

/// Project one logic contract diagnostic.
#[must_use]
pub(crate) fn logic_transaction_error(error: &LogicDiagnosticV1) -> FrontendTransactionErrorV2 {
    document_transaction_error(FrontendDiagnosticV2::new(
        FrontendStageV2::Validation,
        logic_code(error),
        FrontendSeverityV2::Error,
        error.span().location(),
        Vec::new(),
        error.message(),
    ))
}

/// Project one planning diagnostic.
#[must_use]
pub(crate) fn planning_transaction_error(
    error: &PlanningDiagnosticV1,
) -> FrontendTransactionErrorV2 {
    document_transaction_error(FrontendDiagnosticV2::new(
        FrontendStageV2::Validation,
        planning_code(error),
        FrontendSeverityV2::Error,
        error.span().location(),
        Vec::new(),
        error.message(),
    ))
}

/// Mint one pipeline-owned assembly failure.
#[must_use]
pub(crate) fn assembly_transaction_error(
    location: LocatedSpanV2,
    code: DocumentAssemblyCodeV2,
    message: &'static str,
) -> FrontendTransactionErrorV2 {
    document_transaction_error(FrontendDiagnosticV2::new(
        FrontendStageV2::Validation,
        code.as_str(),
        FrontendSeverityV2::Error,
        location,
        Vec::new(),
        message,
    ))
}

/// Wrap one diagnostic as a document transaction failure.
#[must_use]
pub(crate) fn document_transaction_error(
    diagnostic: FrontendDiagnosticV2,
) -> FrontendTransactionErrorV2 {
    FrontendTransactionErrorV2::Document(FrontendFailureV2::one(diagnostic))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "owner-local bridge tests mint checked fixture locations and predicate ids"
)]
mod tests {
    use super::*;
    use quanta_gqlang_v2_logic::{
        GraphEndpointRoleV1, RuleGraphBudgetDimensionV1, RuleProgramInvalidReasonV1,
        UnboundedWitnessReasonV1,
    };
    use quanta_gqlang_v2_planning::{
        CatalogDeclarationKindV2, CheckInvalidReasonV2, CompositionInvalidReasonV2,
        InputDocumentMismatchReasonV2, PresentationInvalidReasonV2,
    };
    use quanta_gqlang_v2_sema::GraphQueryKindV1;

    fn span() -> CheckedLocatedSpanV2 {
        quanta_gqlang_v2_sema::test_support::reference_checked_span_v2(3, 11)
            .expect("checked fixture span")
    }

    #[test]
    fn every_pipeline_owned_code_is_an_independent_literal() {
        for (code, expected) in [
            (
                DocumentAssemblyCodeV2::MissingItemResult,
                "pipeline.document.missing_item_result",
            ),
            (
                DocumentAssemblyCodeV2::ExtraItemResult,
                "pipeline.document.extra_item_result",
            ),
            (
                DocumentAssemblyCodeV2::DuplicateItemResult,
                "pipeline.document.duplicate_item_result",
            ),
            (
                DocumentAssemblyCodeV2::ForeignItemResult,
                "pipeline.document.foreign_item_result",
            ),
            (
                DocumentAssemblyCodeV2::WrongKindItemResult,
                "pipeline.document.wrong_kind_item_result",
            ),
            (
                DocumentAssemblyCodeV2::ItemMembershipMismatch,
                "pipeline.document.item_membership_mismatch",
            ),
            (
                DocumentAssemblyCodeV2::RuleMembershipMismatch,
                "pipeline.document.rule_membership_mismatch",
            ),
            (
                DocumentAssemblyCodeV2::PlanOutcomeMismatch,
                "pipeline.document.plan_outcome_mismatch",
            ),
            (
                DocumentAssemblyCodeV2::CompositionProducerMismatch,
                "pipeline.document.composition_producer_mismatch",
            ),
            (
                DocumentAssemblyCodeV2::CompositionBodyMismatch,
                "pipeline.document.composition_body_mismatch",
            ),
            (
                DocumentAssemblyCodeV2::CheckGraphMismatch,
                "pipeline.document.check_graph_mismatch",
            ),
            (
                DocumentAssemblyCodeV2::InapplicableItemResult,
                "pipeline.document.inapplicable_item_result",
            ),
            (
                DocumentAssemblyCodeV2::DanglingExecutionReference,
                "pipeline.document.dangling_execution_reference",
            ),
            (
                DocumentAssemblyCodeV2::UnregisteredExecutionSymbol,
                "pipeline.document.unregistered_execution_symbol",
            ),
        ] {
            assert_eq!(code.as_str(), expected);
        }
        for (stage, expected) in [
            (PipelineStageV2::Parse, "parse"),
            (PipelineStageV2::Sema, "sema"),
            (PipelineStageV2::Algebra, "algebra"),
            (PipelineStageV2::Logic, "logic"),
            (PipelineStageV2::Planning, "planning"),
            (PipelineStageV2::Assembly, "assembly"),
        ] {
            assert_eq!(stage.as_str(), expected);
        }
    }

    #[test]
    fn every_algebra_bridge_code_is_an_independent_literal() {
        for (kind, expected) in [
            (
                AlgebraFailureKindV2::NonRelationItem,
                "algebra.non_relation_item",
            ),
            (
                AlgebraFailureKindV2::MissingExpression,
                "algebra.missing_expression",
            ),
            (
                AlgebraFailureKindV2::ForeignSemanticFact,
                "algebra.foreign_semantic_fact",
            ),
            (
                AlgebraFailureKindV2::InvalidStageContract,
                "algebra.invalid_stage_contract",
            ),
            (
                AlgebraFailureKindV2::AggregateWithoutGroup,
                "algebra.aggregate_without_group",
            ),
            (
                AlgebraFailureKindV2::NestedAggregate,
                "algebra.nested_aggregate",
            ),
            (
                AlgebraFailureKindV2::MissingNestedRelation,
                "algebra.missing_nested_relation",
            ),
            (
                AlgebraFailureKindV2::DuplicateCorrelationReference,
                "algebra.duplicate_correlation_reference",
            ),
            (
                AlgebraFailureKindV2::TraversalBudgetExceeded,
                "algebra.traversal_budget_exceeded",
            ),
        ] {
            assert_eq!(algebra_code(kind), expected);
            assert_eq!(kind.full_code(), expected);
        }
    }

    #[test]
    fn every_logic_bridge_code_matches_an_independent_literal() {
        let base = [
            (
                LogicDiagnosticV1::GraphEndpointMissing {
                    kind: GraphQueryKindV1::Taint,
                    role: GraphEndpointRoleV1::Sink,
                    span: span(),
                },
                "logic.graph_endpoint_missing",
            ),
            (
                LogicDiagnosticV1::GraphEndpointUnexpected {
                    kind: GraphQueryKindV1::Slice,
                    role: GraphEndpointRoleV1::Source,
                    span: span(),
                },
                "logic.graph_endpoint_unexpected",
            ),
        ];
        for (diagnostic, expected) in base {
            assert_eq!(logic_code(&diagnostic), expected);
            assert_eq!(diagnostic.code(), expected);
        }

        for (reason, expected) in [
            (
                UnboundedWitnessReasonV1::MissingMaxWitnesses,
                "logic.unbounded_witness.missing_max_witnesses",
            ),
            (
                UnboundedWitnessReasonV1::MissingPathLengthBound,
                "logic.unbounded_witness.missing_path_length_bound",
            ),
            (
                UnboundedWitnessReasonV1::ZeroMaxWitnesses,
                "logic.unbounded_witness.zero_max_witnesses",
            ),
            (
                UnboundedWitnessReasonV1::ZeroPathLengthBound,
                "logic.unbounded_witness.zero_path_length_bound",
            ),
            (
                UnboundedWitnessReasonV1::MaxWitnessesOverBudget,
                "logic.unbounded_witness.max_witnesses_over_budget",
            ),
            (
                UnboundedWitnessReasonV1::PathLengthBoundOverBudget,
                "logic.unbounded_witness.path_length_bound_over_budget",
            ),
        ] {
            let diagnostic = LogicDiagnosticV1::UnboundedWitness {
                reason,
                span: span(),
            };
            assert_eq!(logic_code(&diagnostic), expected);
            assert_eq!(diagnostic.code(), expected);
        }

        for (reason, expected) in [
            (
                RuleProgramInvalidReasonV1::MissingMember,
                "logic.rule_program.missing_member",
            ),
            (
                RuleProgramInvalidReasonV1::DuplicateMember,
                "logic.rule_program.duplicate_member",
            ),
            (
                RuleProgramInvalidReasonV1::NonLocalTarget,
                "logic.rule_program.nonlocal_target",
            ),
            (
                RuleProgramInvalidReasonV1::WrongCallId,
                "logic.rule_program.wrong_call_id",
            ),
            (
                RuleProgramInvalidReasonV1::TargetTransplanted,
                "logic.rule_program.target_transplanted",
            ),
            (
                RuleProgramInvalidReasonV1::PolarityFlip,
                "logic.rule_program.polarity_flip",
            ),
            (
                RuleProgramInvalidReasonV1::ForeignLocation,
                "logic.rule_program.foreign_location",
            ),
            (
                RuleProgramInvalidReasonV1::MembershipMismatch,
                "logic.rule_program.membership_mismatch",
            ),
        ] {
            let diagnostic = LogicDiagnosticV1::RuleProgramInvalid {
                reason,
                span: span(),
            };
            assert_eq!(logic_code(&diagnostic), expected);
            assert_eq!(diagnostic.code(), expected);
        }

        for (dimension, expected) in [
            (
                RuleGraphBudgetDimensionV1::Rules,
                "logic.rule_graph_budget.rules",
            ),
            (
                RuleGraphBudgetDimensionV1::Nodes,
                "logic.rule_graph_budget.nodes",
            ),
            (
                RuleGraphBudgetDimensionV1::Edges,
                "logic.rule_graph_budget.edges",
            ),
            (
                RuleGraphBudgetDimensionV1::Steps,
                "logic.rule_graph_budget.steps",
            ),
        ] {
            let diagnostic = LogicDiagnosticV1::RuleGraphBudgetExceeded {
                dimension,
                limit: 7,
                span: span(),
            };
            assert_eq!(logic_code(&diagnostic), expected);
            assert_eq!(diagnostic.code(), expected);
        }
    }

    #[test]
    fn every_planning_bridge_code_matches_an_independent_literal() {
        for (reason, expected) in [
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
            let diagnostic = PlanningDiagnosticV1::InputDocumentMismatch {
                reason,
                span: span(),
            };
            assert_eq!(planning_code(&diagnostic), expected);
            assert_eq!(diagnostic.code(), expected);
        }
        for (reason, expected) in [
            (
                CheckInvalidReasonV2::NestedGraphIdentityMismatch,
                "planning.invalid_check.nested_graph_identity_mismatch",
            ),
            (
                CheckInvalidReasonV2::UnexpectedNestedGraph,
                "planning.invalid_check.unexpected_nested_graph",
            ),
            (
                CheckInvalidReasonV2::MissingNestedGraph,
                "planning.invalid_check.missing_nested_graph",
            ),
            (
                CheckInvalidReasonV2::SemanticConfirmInPatternMode,
                "planning.invalid_check.semantic_confirm_in_pattern_mode",
            ),
            (
                CheckInvalidReasonV2::MissingSemanticConfirm,
                "planning.invalid_check.missing_semantic_confirm",
            ),
            (
                CheckInvalidReasonV2::PatternInputInSemanticMode,
                "planning.invalid_check.pattern_input_in_semantic_mode",
            ),
            (
                CheckInvalidReasonV2::MissingHybridPatternInput,
                "planning.invalid_check.missing_hybrid_pattern_input",
            ),
            (
                CheckInvalidReasonV2::MissingHybridSemanticInput,
                "planning.invalid_check.missing_hybrid_semantic_input",
            ),
        ] {
            let diagnostic = PlanningDiagnosticV1::InvalidCheck {
                reason,
                span: span(),
            };
            assert_eq!(planning_code(&diagnostic), expected);
            assert_eq!(diagnostic.code(), expected);
        }
        for (reason, expected) in [
            (
                CompositionInvalidReasonV2::EmptyProducers,
                "planning.invalid_composition.empty_producers",
            ),
            (
                CompositionInvalidReasonV2::ProducerOrdinalMismatch,
                "planning.invalid_composition.producer_ordinal_mismatch",
            ),
            (
                CompositionInvalidReasonV2::ProducerIdentityMismatch,
                "planning.invalid_composition.producer_identity_mismatch",
            ),
            (
                CompositionInvalidReasonV2::DuplicateProducer,
                "planning.invalid_composition.duplicate_producer",
            ),
            (
                CompositionInvalidReasonV2::WithTargetNotAProducer,
                "planning.invalid_composition.with_target_not_a_producer",
            ),
            (
                CompositionInvalidReasonV2::BodyIdentityMismatch,
                "planning.invalid_composition.body_identity_mismatch",
            ),
            (
                CompositionInvalidReasonV2::BodyKindMismatch,
                "planning.invalid_composition.body_kind_mismatch",
            ),
        ] {
            let diagnostic = PlanningDiagnosticV1::InvalidComposition {
                reason,
                span: span(),
            };
            assert_eq!(planning_code(&diagnostic), expected);
            assert_eq!(diagnostic.code(), expected);
        }
        let presentation = PlanningDiagnosticV1::InvalidPresentation {
            reason: PresentationInvalidReasonV2::UnsupportedExplainEvidenceTruth,
            span: span(),
        };
        assert_eq!(
            planning_code(&presentation),
            "planning.invalid_presentation.unsupported_explain_evidence_truth"
        );
        assert_eq!(planning_code(&presentation), presentation.code());
        let budget = PlanningDiagnosticV1::SemanticStepBudgetExceeded {
            limit: 7,
            span: span(),
        };
        assert_eq!(
            planning_code(&budget),
            "planning.budget.semantic_steps_exceeded"
        );
        assert_eq!(planning_code(&budget), budget.code());

        for (reason, expected) in [
            (
                NoPlanReasonV2::ValueDeclaration,
                "planning.no_plan.value_declaration",
            ),
            (
                NoPlanReasonV2::MetaActivation,
                "planning.no_plan.meta_activation",
            ),
            (
                NoPlanReasonV2::EmptyRuleProgram,
                "planning.no_plan.empty_rule_program",
            ),
        ] {
            assert_eq!(no_plan_code(reason), expected);
            let diagnostic = PlanningDiagnosticV1::NoPlan {
                reason,
                span: span(),
            };
            assert_eq!(planning_code(&diagnostic), expected);
            assert_eq!(diagnostic.code(), expected);
        }
        for (kind, expected) in [
            (
                CatalogDeclarationKindV2::View,
                "planning.no_plan.catalog_declaration.view",
            ),
            (
                CatalogDeclarationKindV2::Model,
                "planning.no_plan.catalog_declaration.model",
            ),
            (
                CatalogDeclarationKindV2::Pack,
                "planning.no_plan.catalog_declaration.pack",
            ),
            (
                CatalogDeclarationKindV2::Profile,
                "planning.no_plan.catalog_declaration.profile",
            ),
            (
                CatalogDeclarationKindV2::Gate,
                "planning.no_plan.catalog_declaration.gate",
            ),
            (
                CatalogDeclarationKindV2::RuleBinding,
                "planning.no_plan.catalog_declaration.rule_binding",
            ),
        ] {
            let reason = NoPlanReasonV2::CatalogDeclaration { kind };
            assert_eq!(no_plan_code(reason), expected);
            let diagnostic = PlanningDiagnosticV1::NoPlan {
                reason,
                span: span(),
            };
            assert_eq!(planning_code(&diagnostic), expected);
            assert_eq!(diagnostic.code(), expected);
        }
    }
}
