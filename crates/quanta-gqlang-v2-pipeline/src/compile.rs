//! The single shared frontend compiler service (ticket 07, lane W4-B).
//!
//! This module orchestrates exactly five owners and nothing else:
//!
//! ```text
//! Surface parse -> Sema sealed document -> Algebra lowering
//!   -> Logic rule/graph contracts -> Planning -> exact-key assembly
//! ```
//!
//! It owns NO parser, binder, type kernel, algebra node, logic rule, or planner
//! rule. Every semantic fact is read from the sealed [`SemanticDocumentV2`]; the
//! only values this module mints are the compiled/analysis DTOs, the exact-key
//! assembly maps, and the pipeline-owned diagnostics in [`crate::diagnostic`].
//!
//! ## Exact-key assembly
//!
//! [`SemanticDocumentV2::order`] is the authored source order and
//! [`SemanticDocumentV2::items`] is the produced item map. Assembly never zips
//! positionally, never matches by display name, and never represents a missing
//! plan as `None`:
//!
//! - every authored id must resolve to exactly one item whose own
//!   [`SemanticItemV2::item_id`] equals the key and whose source is this
//!   document's source (the [`DocumentAssemblyCodeV2`] vocabulary rejects
//!   missing, extra, duplicate, foreign, and wrong-kind results);
//! - plan-required families (relation, graph, CHECK, presentation, composition,
//!   and REQUIRE CERTIFIED) carry a [`SemanticPlanV2`];
//! - catalog declarations, value declarations, and `USE PACK` carry a checked
//!   [`NoPlanReasonV2`];
//! - source rules carry one [`RuleProgramMemberV2`] and the document-wide
//!   program carries the single plan.
//!
//! ## Budget
//!
//! No entry accepts a caller-selected traversal budget. `compile_*` and
//! `analyze_*` read every cap from the closed/open authority; lowering, rule
//! stratification, and planning then read the sealed document's immutable
//! downstream budget receipt. `format_source_document_v2` is the one parse-only
//! entry and takes the caller's checked [`FrontendBudgetV2`] because it performs
//! no semantic traversal.

use std::collections::{BTreeMap, BTreeSet};

use quanta_gqlang_v2_algebra::AlgebraSessionV2;
use quanta_gqlang_v2_logic::{GraphQueryContractV2, RuleProgramContractV2, StratificationProofV1};
use quanta_gqlang_v2_planning::{
    CatalogDeclarationKindV2, CheckContractV2, CheckedCompositionPlanInputV2,
    CompositionBodyPlanInputV2, CompositionRelationalBodyPlanInputV2, NoPlanReasonV2,
    PlanOutcomeV2, PlanningDiagnosticV1, PlanningSessionV2, PresentationContractV2,
    SemanticPlanInputV2, SemanticPlanV2, ShowPlanningErrorV2, ShowPlanningSessionV2,
};
use quanta_gqlang_v2_sema::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    DeclaredDocumentV2, DeclaredShowSourceV2, FrontendBudgetV2, FrontendDiagnosticV2,
    FrontendFailureV2, FrontendSeverityV2, FrontendStageV2, FrontendTransactionErrorV2,
    LocatedSpanV2, OpenAnalysisAuthorityV2, ResolvedCatalogItemV2, ResolvedCatalogPayloadV2,
    ResolvedCheckV2, ResolvedCompositionBodyV2, ResolvedCompositionProducerV2,
    ResolvedCompositionV2, ResolvedMetaKindV2, SemanticDocumentV2, SemanticItemIdV2,
    SemanticItemV2, SourceIdV2, SourceRegistrationV2, SpanV1,
};
use quanta_gqlang_v2_surface::{
    census_source_lexical_materialization_v2, classify_parsed_root_v3,
    format_source_document_with_parse_budget_v2, parse_source_document_v2,
    parse_source_document_with_lexical_owner_v1, AdmittedLexicalParsedDocumentV1,
    AdmittedLexicalValueV1, FrontendLexicalMaterializationRootV1, FrontendLexicalPermitV1,
    LexicalMaterializationCensusV1, ParseBudgetDimensionV2, ParseBudgetV2,
};

pub use quanta_gqlang_v2_surface::SourceRootFamilyV3 as FrontendSourceRootFamilyV3;

use crate::diagnostic::{
    algebra_transaction_error, assembly_transaction_error, canonicalize_diagnostics,
    document_transaction_error, logic_transaction_error, planning_transaction_error,
    surface_failure, surface_transaction_error, DocumentAssemblyCodeV2,
};
use crate::execution::{
    project_check, project_composition, project_graph_query, project_presentation,
    project_relation, project_relation_root, ExecutionCheckV3, ExecutionClosureFailureV3,
    ExecutionCompositionBodyProjectionV3, ExecutionCompositionProducerProjectionV3,
    ExecutionCompositionProjectionFailureV3, ExecutionCompositionV3, ExecutionDocumentV3,
    ExecutionFamilyV3, ExecutionGraphQueryV3, ExecutionItemV3, ExecutionPresentationKindV3,
    ExecutionProjectionV3, ExecutionRelationV3,
};

mod request_owner;
pub use request_owner::{
    begin_bind_source_document_with_request_owner_v3,
    begin_prepare_source_body_with_request_owner_v3,
    begin_resolve_source_document_with_request_owner_v3,
    check_source_document_with_request_owner_v3, classify_single_source_root_with_request_owner_v3,
    compile_scalar_parsed_source_with_request_owner_v3,
    compile_scalar_source_with_request_owner_v3, declare_source_document_with_request_owner_v3,
    lower_source_body_with_request_owner_v3, materialize_source_body_with_request_owner_v3,
    parse_source_document_with_request_owner_v3, plan_source_body_with_request_owner_v3,
    predeclare_source_catalog_with_request_owner_v3, project_source_body_with_request_owner_v3,
    resolve_source_bindings_with_request_owner_v3,
    seal_scalar_execution_document_with_request_owner_v3,
    seal_source_catalog_with_request_owner_v3, topologize_source_body_with_request_owner_v3,
    type_source_body_with_request_owner_v3, AdmittedCanonicalFilesScalarReadV3,
    AdmittedScalarExecutionDocumentV3, CatalogSealedSourceBodyRequestSourceV3,
    CheckedRequestSourceV3, DeclaredRequestSourceV3, DeclaringBinderRequestSourceV3,
    FrontendRequestCheckErrorV3, FrontendRequestPredeclareErrorV3,
    LoweredSourceBodyRequestSourceV3, MaterializedSourceBodyRequestSourceV3, ParsedRequestSourceV3,
    PlannedSourceBodyRequestSourceV3, PredeclaredRequestSourceV3, PreparingBodyRequestSourceV3,
    ProjectedSourceBodyRequestSourceV3, ResolvedSourceBinderRequestSourceV3,
    ResolvingRequestSourceV3, ScalarExecutionRequestErrorV3, ScalarSourceCompileRequestErrorV3,
    SharedAdmittedScalarExecutionDocumentV3, SourceBodyTopologyRequestErrorV3,
    SourceCatalogSealErrorV3, TopologizedSourceBodyRequestSourceV3, TypedSourceBodyRequestSourceV3,
};

#[cfg(test)]
use quanta_gqlang_v2_surface::{FrontendRequestMaterializationRootV3, FrontendStorageRefusalV3};

/// Pipeline-owned code for an open-analysis source whose authority is partial.
pub const OPEN_AUTHORITY_INCOMPLETE_CODE: &str = "pipeline.analysis.open_authority_incomplete";

/// Pipeline-owned code for a failed frontend-budget projection.
///
/// Every dimension of a checked [`FrontendBudgetV2`] is a `NonZeroU32`, and the
/// parse budget rejects only zero, so no checked budget can reach this code.
/// The vocabulary is retained for the wire-code inventory; its one construction
/// site anchors on the caller's own source and exact span, never on a fabricated
/// source identity.
pub const PARSE_BUDGET_PROJECTION_FAILED_CODE: &str = "pipeline.parse_budget_projection_failed";
/// Pipeline-owned code for an open-analysis diagnostic count over its sealed cap.
pub const ANALYSIS_DIAGNOSTIC_LIMIT_CODE: &str = "pipeline.analysis.diagnostic_limit_exceeded";

/// Failure to classify exactly one parsed source root.
#[derive(Debug)]
pub enum FrontendSourceRootClassificationErrorV3 {
    Frontend(FrontendTransactionErrorV2),
    NoItem,
    MultipleItems { observed: usize },
}

// ---------------------------------------------------------------------------
// Public DTO roots
// ---------------------------------------------------------------------------

/// One fully compiled frontend document.
///
/// Fields are private: a caller can read the source identity, the canonical
/// (non-error) diagnostics, the source-ordered item list, and the optional
/// document rule program, but cannot construct a compiled document from
/// unverified parts. There is no `Option<SemanticPlanV2>` representation: a
/// plan-required item always carries [`SemanticPlanV2`], and a non-executable
/// declaration always carries [`NoPlanReasonV2`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledDocumentV2 {
    source_id: SourceIdV2,
    diagnostics: Box<[FrontendDiagnosticV2]>,
    items: Box<[CompiledItemV2]>,
    rule_program: Option<CompiledRuleProgramV2>,
}

impl CompiledDocumentV2 {
    /// Seal one compiled document.
    ///
    /// A closed compiled success carries no `Error` diagnostic; only
    /// producer-sorted/deduplicated `Warning`/`Note` diagnostics are admitted.
    fn try_new(
        source_id: SourceIdV2,
        diagnostics: Vec<FrontendDiagnosticV2>,
        items: Vec<CompiledItemV2>,
        rule_program: Option<CompiledRuleProgramV2>,
    ) -> Result<Self, FrontendTransactionErrorV2> {
        let diagnostics = canonicalize_diagnostics(diagnostics);
        if diagnostics
            .iter()
            .any(|diagnostic| diagnostic.severity() == FrontendSeverityV2::Error)
        {
            return Err(document_transaction_error(FrontendDiagnosticV2::new(
                FrontendStageV2::Validation,
                DocumentAssemblyCodeV2::ItemMembershipMismatch.as_str(),
                FrontendSeverityV2::Error,
                zero_location(source_id),
                Vec::new(),
                "a compiled document cannot carry an error diagnostic",
            )));
        }
        Ok(Self {
            source_id,
            diagnostics: diagnostics.into_boxed_slice(),
            items: items.into_boxed_slice(),
            rule_program,
        })
    }

    /// Source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.source_id
    }

    /// Canonical, producer-sorted, deduplicated non-error diagnostics.
    #[must_use]
    pub fn diagnostics(&self) -> &[FrontendDiagnosticV2] {
        &self.diagnostics
    }

    /// Source-ordered compiled items.
    #[must_use]
    pub fn items(&self) -> &[CompiledItemV2] {
        &self.items
    }

    /// Exact compiled item by semantic identity.
    #[must_use]
    pub fn item(&self, id: SemanticItemIdV2) -> Option<&CompiledItemV2> {
        self.items.iter().find(|item| item.item_id() == id)
    }

    /// The single document-wide rule program, absent when the document has no
    /// rules.
    #[must_use]
    pub const fn rule_program(&self) -> Option<&CompiledRuleProgramV2> {
        self.rule_program.as_ref()
    }
}

/// Source-bound closed compile result. Only the source entry can construct it;
/// resolved-document compilation lacks original bytes and returns
/// [`CompiledDocumentV2`] without an execution document instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledSourceDocumentV3 {
    compiled: CompiledDocumentV2,
    execution: ExecutionDocumentV3,
}

/// Closed result retaining lexer and top-level parser item-vector debits.
/// HIR conversion, semantic and plan allocations remain open.
pub type AdmittedLexicalCompiledSourceDocumentV1<'root, 'source> =
    AdmittedLexicalValueV1<'root, 'source, CompiledSourceDocumentV3>;

/// SHOW-only syntax proof. No public method exposes owned or cloneable HIR or
/// the generic V1 map; all parser reservations remain attached.
///
/// ```compile_fail
/// use quanta_gqlang_v2_pipeline::AdmittedShowDocumentV2;
/// fn assert_clone<T: Clone>() {}
/// assert_clone::<AdmittedShowDocumentV2<'static, 'static>>();
/// ```
///
/// ```compile_fail
/// use quanta_gqlang_v2_pipeline::AdmittedShowDocumentV2;
/// fn detach(owner: &AdmittedShowDocumentV2<'_, '_>) {
///     let _ = owner.value_v1();
/// }
/// ```
pub struct AdmittedShowDocumentV2<'root, 'source> {
    checked: AdmittedLexicalValueV1<'root, 'source, CheckedSyntaxDocumentV2>,
    budget: FrontendBudgetV2,
}

/// SHOW source declaration retaining the syntax root and the exact host
/// registry extension. This is not a resolved semantic document or a plan.
///
/// ```compile_fail
/// use quanta_gqlang_v2_pipeline::AdmittedShowSourceDeclarationV2;
/// fn detach(owner: &AdmittedShowSourceDeclarationV2<'_, '_>) {
///     let _ = owner.value_v1();
/// }
/// ```
pub struct AdmittedShowSourceDeclarationV2<'root, 'source> {
    declared: AdmittedLexicalValueV1<'root, 'source, DeclaredShowSourceV2>,
}

/// SHOW semantic plans in authored order, retaining both the parser and plan
/// output debits. No public accessor exposes a cloneable plan or bare HIR.
/// Generic compilation, execution projection and host issuer remain open.
pub struct AdmittedShowPlannedDocumentV2<'root, 'source> {
    plans: Vec<SemanticPlanV2>,
    declared: AdmittedShowSourceDeclarationV2<'root, 'source>,
    _plan_permit: FrontendLexicalPermitV1<'root, 'source>,
}

/// Selected SHOW execution DTO retaining syntax, semantic-plan, and execution
/// vector debits on the same source root. It exposes only copied identity and
/// digest projections; a cloneable bare execution document cannot escape.
pub struct AdmittedShowExecutionDocumentV2<'root, 'source> {
    execution: ExecutionDocumentV3,
    planned: AdmittedShowPlannedDocumentV2<'root, 'source>,
    _execution_permit: FrontendLexicalPermitV1<'root, 'source>,
}

impl AdmittedShowDocumentV2<'_, '_> {
    #[must_use]
    pub fn source_id_v2(&self) -> SourceIdV2 {
        self.checked.value_v1().source_id()
    }

    #[must_use]
    pub fn item_count_v2(&self) -> usize {
        self.checked.value_v1().checked().document().items.len()
    }

    #[must_use]
    pub fn checked_node_count_v2(&self) -> usize {
        self.checked.value_v1().node_count()
    }
}

impl AdmittedShowSourceDeclarationV2<'_, '_> {
    #[must_use]
    pub fn source_id_v2(&self) -> SourceIdV2 {
        self.declared.value_v1().source_id()
    }

    #[must_use]
    pub fn item_count_v2(&self) -> usize {
        self.declared.value_v1().item_count()
    }
}

impl AdmittedShowPlannedDocumentV2<'_, '_> {
    #[must_use]
    pub fn source_id_v2(&self) -> SourceIdV2 {
        self.declared.source_id_v2()
    }

    #[must_use]
    pub fn plan_count_v2(&self) -> usize {
        self.plans.len()
    }
}

impl AdmittedShowExecutionDocumentV2<'_, '_> {
    #[must_use]
    pub fn source_id_v2(&self) -> SourceIdV2 {
        self.planned.source_id_v2()
    }

    #[must_use]
    pub fn item_count_v2(&self) -> usize {
        self.execution.items().len()
    }

    #[must_use]
    pub fn qualification_digest_v2(&self) -> crate::execution::ExecutionDigestV3 {
        self.execution.qualification_digest()
    }
}

/// A selected SHOW-only stage has an explicit incomplete result for adjacent
/// grammar. Existing V1/legacy frontend entrypoints remain unqualified.
#[derive(Debug)]
pub enum ShowSyntaxAdmissionErrorV2 {
    Frontend(FrontendTransactionErrorV2),
    IncompleteShowGrammar,
    IncompleteAuthorityBudget,
    IncompleteShowRoot,
    IncompleteShowOutputDemand,
    IncompleteShowOutputStorage,
    IncompleteShowPlanning(ShowPlanningErrorV2),
    IncompleteShowSourceLimit,
    IncompleteSourceDiagnostics,
    IncompleteShowExecution(&'static str),
}

impl ShowSyntaxAdmissionErrorV2 {
    #[must_use]
    pub const fn reason_code(&self) -> &'static str {
        match self {
            Self::Frontend(_) => "pipeline.show_syntax_frontend_rejected",
            Self::IncompleteShowGrammar => "pipeline.show_syntax_incomplete_grammar",
            Self::IncompleteAuthorityBudget => "pipeline.show_source_incomplete_authority_budget",
            Self::IncompleteShowRoot => "pipeline.show_plan_incomplete_root",
            Self::IncompleteShowOutputDemand => "pipeline.show_plan_incomplete_output_demand",
            Self::IncompleteShowOutputStorage => "pipeline.show_plan_incomplete_output_storage",
            Self::IncompleteShowPlanning(_) => "pipeline.show_plan_incomplete_planning",
            Self::IncompleteShowSourceLimit => "pipeline.show_source_incomplete_source_limit",
            Self::IncompleteSourceDiagnostics => "pipeline.show_source_incomplete_diagnostic_owner",
            Self::IncompleteShowExecution(_) => "pipeline.show_execution_incomplete",
        }
    }
}

impl CompiledSourceDocumentV3 {
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.compiled.source_id()
    }

    #[must_use]
    pub fn diagnostics(&self) -> &[FrontendDiagnosticV2] {
        self.compiled.diagnostics()
    }

    #[must_use]
    pub fn items(&self) -> &[CompiledItemV2] {
        self.compiled.items()
    }

    #[must_use]
    pub fn item(&self, id: SemanticItemIdV2) -> Option<&CompiledItemV2> {
        self.compiled.item(id)
    }

    #[must_use]
    pub const fn rule_program(&self) -> Option<&CompiledRuleProgramV2> {
        self.compiled.rule_program()
    }

    /// Immutable, source-qualified host-agnostic execution document.
    #[must_use]
    pub const fn execution(&self) -> &ExecutionDocumentV3 {
        &self.execution
    }
}

/// One compiled top-level item. The closed enum encodes plan applicability
/// structurally: a plan-required item cannot exist without a plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompiledItemV2 {
    /// The item ran every applicable stage and carries its semantic plan.
    Planned(Box<PlannedItemV2>),
    /// The item is a checked non-plan declaration.
    NoPlan {
        /// Canonical item identity.
        item_id: SemanticItemIdV2,
        /// Checked non-plan reason.
        reason: NoPlanReasonV2,
    },
}

impl CompiledItemV2 {
    /// Canonical item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        match self {
            Self::Planned(item) => item.item_id(),
            Self::NoPlan { item_id, .. } => *item_id,
        }
    }

    /// The semantic plan, present only for plan-required families.
    #[must_use]
    pub const fn plan(&self) -> Option<&SemanticPlanV2> {
        match self {
            Self::Planned(item) => item.plan(),
            Self::NoPlan { .. } => None,
        }
    }

    /// The checked non-plan reason, present only for non-executable
    /// declarations.
    #[must_use]
    pub const fn no_plan_reason(&self) -> Option<&NoPlanReasonV2> {
        match self {
            Self::Planned(_) => None,
            Self::NoPlan { reason, .. } => Some(reason),
        }
    }
}

/// A plan-required compiled item. Every variant except the rule member carries
/// exactly one [`SemanticPlanV2`]; the rule member is a checked reference into
/// the document rule program and never duplicates its plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannedItemV2 {
    /// Exact-target rename command with a host-grant-dependent effect.
    Command {
        item_id: SemanticItemIdV2,
        plan: SemanticPlanV2,
    },
    /// Relational query/relation item.
    Relation {
        /// Canonical item identity.
        item_id: SemanticItemIdV2,
        /// Derived semantic plan.
        plan: SemanticPlanV2,
    },
    /// FIND / graph-query item.
    Graph {
        /// Canonical item identity.
        item_id: SemanticItemIdV2,
        /// Derived semantic plan.
        plan: SemanticPlanV2,
    },
    /// CHECK item, including its identity-checked nested graph contract.
    Check {
        /// Canonical item identity.
        item_id: SemanticItemIdV2,
        /// Derived semantic plan.
        plan: SemanticPlanV2,
    },
    /// Presentation item (EXPLAIN / SHOW / DESCRIBE / SAMPLE / COMPARE /
    /// EXPORT).
    Presentation {
        /// Canonical item identity.
        item_id: SemanticItemIdV2,
        /// Derived semantic plan.
        plan: SemanticPlanV2,
    },
    /// Sequential `LET .. WITH` composition item.
    Composition {
        /// Canonical item identity.
        item_id: SemanticItemIdV2,
        /// Derived semantic plan.
        plan: SemanticPlanV2,
    },
    /// Exact `REQUIRE CERTIFIED` directive with its validated proof.
    MetaRequireCertified {
        /// Canonical item identity.
        item_id: SemanticItemIdV2,
        /// Derived semantic plan.
        plan: SemanticPlanV2,
    },
    /// One source rule, stored as a checked program member reference.
    RuleMember(RuleProgramMemberV2),
}

impl PlannedItemV2 {
    /// Canonical item identity.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        match self {
            Self::Command { item_id, .. }
            | Self::Relation { item_id, .. }
            | Self::Graph { item_id, .. }
            | Self::Check { item_id, .. }
            | Self::Presentation { item_id, .. }
            | Self::Composition { item_id, .. }
            | Self::MetaRequireCertified { item_id, .. } => *item_id,
            Self::RuleMember(member) => member.member_item_id(),
        }
    }

    /// The semantic plan; a rule member's plan lives once in the document rule
    /// program.
    #[must_use]
    pub const fn plan(&self) -> Option<&SemanticPlanV2> {
        match self {
            Self::Command { plan, .. }
            | Self::Relation { plan, .. }
            | Self::Graph { plan, .. }
            | Self::Check { plan, .. }
            | Self::Presentation { plan, .. }
            | Self::Composition { plan, .. }
            | Self::MetaRequireCertified { plan, .. } => Some(plan),
            Self::RuleMember(_) => None,
        }
    }

    /// The checked rule program member reference.
    #[must_use]
    pub const fn rule_member(&self) -> Option<&RuleProgramMemberV2> {
        match self {
            Self::RuleMember(member) => Some(member),
            Self::Command { .. }
            | Self::Relation { .. }
            | Self::Graph { .. }
            | Self::Check { .. }
            | Self::Presentation { .. }
            | Self::Composition { .. }
            | Self::MetaRequireCertified { .. } => None,
        }
    }
}

/// One checked source rule reference into the document-wide rule program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuleProgramMemberV2 {
    program_id: SemanticItemIdV2,
    member_item_id: SemanticItemIdV2,
}

impl RuleProgramMemberV2 {
    const fn new(program_id: SemanticItemIdV2, member_item_id: SemanticItemIdV2) -> Self {
        Self {
            program_id,
            member_item_id,
        }
    }

    /// Identity of the rule program this member belongs to: the first source
    /// rule in document order, which anchors the program's stable identity.
    #[must_use]
    pub const fn program_id(self) -> SemanticItemIdV2 {
        self.program_id
    }

    /// This member's source rule item.
    #[must_use]
    pub const fn member_item_id(self) -> SemanticItemIdV2 {
        self.member_item_id
    }
}

/// The single plan of one document-wide rule program, plus the logic
/// stratification proof that admitted it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledRuleProgramV2 {
    program_id: SemanticItemIdV2,
    members: Box<[RuleProgramMemberV2]>,
    plan: SemanticPlanV2,
    stratification: StratificationProofV1,
}

impl CompiledRuleProgramV2 {
    /// Stable program identity (the first member in document order).
    #[must_use]
    pub const fn program_id(&self) -> SemanticItemIdV2 {
        self.program_id
    }

    /// Exact member references in document order.
    #[must_use]
    pub fn members(&self) -> &[RuleProgramMemberV2] {
        &self.members
    }

    /// The one document-wide semantic plan.
    #[must_use]
    pub const fn plan(&self) -> &SemanticPlanV2 {
        &self.plan
    }

    /// The logic stratification proof that admitted this program.
    #[must_use]
    pub const fn stratification(&self) -> &StratificationProofV1 {
        &self.stratification
    }
}

/// One open-analysis document. Every item stage is an explicit, non-optional
/// status; the document may retain unresolved facts and can never be converted
/// into a [`CompiledDocumentV2`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisDocumentV2 {
    source_id: SourceIdV2,
    order: Box<[SemanticItemIdV2]>,
    items: Box<[AnalysisItemV2]>,
    rule_program: Option<CompiledRuleProgramV2>,
    unresolved: Box<[UnresolvedAnalysisFactV2]>,
}

/// Canonical status of one open-analysis transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnalysisStatusV2 {
    /// Every item reached its applicable terminal stage.
    Complete,
    /// Missing host authority prevented a closed answer.
    Incomplete,
    /// At least one item was rejected by a frontend stage.
    Error,
}

impl AnalysisStatusV2 {
    /// Stable wire/CLI spelling.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Incomplete => "incomplete",
            Self::Error => "error",
        }
    }
}

impl AnalysisDocumentV2 {
    /// Source identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.source_id
    }

    /// Source-ordered analysis item identities.
    #[must_use]
    pub fn order(&self) -> &[SemanticItemIdV2] {
        &self.order
    }

    /// Source-ordered analysis items; `order()` names the identity of each.
    #[must_use]
    pub fn items(&self) -> &[AnalysisItemV2] {
        &self.items
    }

    /// The analysis stage status of one authored identity.
    #[must_use]
    pub fn item(&self, id: SemanticItemIdV2) -> Option<&AnalysisItemV2> {
        self.order
            .iter()
            .position(|candidate| *candidate == id)
            .and_then(|index| self.items.get(index))
    }

    /// The document-wide rule program when the rule stage completed.
    #[must_use]
    pub const fn rule_program(&self) -> Option<&CompiledRuleProgramV2> {
        self.rule_program.as_ref()
    }

    /// Unresolved analysis facts, ordered by canonical location.
    #[must_use]
    pub fn unresolved(&self) -> &[UnresolvedAnalysisFactV2] {
        &self.unresolved
    }

    /// Canonical transaction status. Unresolved authority takes precedence
    /// over item-local warning carriers; otherwise any blocked item is an
    /// error. Front doors consume this classification instead of re-deriving
    /// their own success policy.
    #[must_use]
    pub fn status(&self) -> AnalysisStatusV2 {
        if !self.unresolved.is_empty() {
            AnalysisStatusV2::Incomplete
        } else if self
            .items
            .iter()
            .any(|item| matches!(item, AnalysisItemV2::BlockedByDiagnostics(_)))
        {
            AnalysisStatusV2::Error
        } else {
            AnalysisStatusV2::Complete
        }
    }

    /// Canonical sorted and deduplicated diagnostics across all blocked items.
    #[must_use]
    pub fn diagnostics(&self) -> Box<[FrontendDiagnosticV2]> {
        let diagnostics = self
            .items
            .iter()
            .filter_map(|item| match item {
                AnalysisItemV2::BlockedByDiagnostics(diagnostics) => Some(diagnostics.as_ref()),
                AnalysisItemV2::Complete(_) | AnalysisItemV2::NotApplicable(_) => None,
            })
            .flatten()
            .cloned()
            .collect::<Vec<_>>();
        canonicalize_diagnostics(diagnostics).into_boxed_slice()
    }
}

/// One item-stage status inside an [`AnalysisDocumentV2`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AnalysisItemV2 {
    /// Every applicable stage completed for this item.
    Complete(Box<PlannedItemV2>),
    /// The item is a checked non-plan declaration.
    NotApplicable(NoPlanReasonV2),
    /// A stage blocked this item; the ordered diagnostics are the reason.
    BlockedByDiagnostics(Box<[FrontendDiagnosticV2]>),
}

/// One unresolved open-analysis fact.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnresolvedAnalysisFactV2 {
    code: &'static str,
    location: LocatedSpanV2,
}

impl UnresolvedAnalysisFactV2 {
    const fn new(code: &'static str, location: LocatedSpanV2) -> Self {
        Self { code, location }
    }

    /// Stable unresolved code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        self.code
    }

    /// Exact unresolved location.
    #[must_use]
    pub const fn location(self) -> LocatedSpanV2 {
        self.location
    }
}

// ---------------------------------------------------------------------------
// Public entries
// ---------------------------------------------------------------------------

/// Parse one source with the canonical frontend and classify exactly one root
/// before host catalogs are consulted.
///
/// The result is suitable for fail-closed route selection only. It does not
/// prove that referenced relations, graph entities, rules, or providers exist.
pub fn classify_single_source_root_v3(
    source_id: SourceIdV2,
    source: &str,
    budget: &FrontendBudgetV2,
) -> Result<FrontendSourceRootFamilyV3, FrontendSourceRootClassificationErrorV3> {
    let parsed = parse_source(source_id, source, budget)
        .map_err(FrontendSourceRootClassificationErrorV3::Frontend)?;
    let [item] = parsed.hir.items.as_slice() else {
        return Err(if parsed.hir.items.is_empty() {
            FrontendSourceRootClassificationErrorV3::NoItem
        } else {
            FrontendSourceRootClassificationErrorV3::MultipleItems {
                observed: parsed.hir.items.len(),
            }
        });
    };
    Ok(classify_parsed_root_v3(item))
}

/// Classify with a source-bound lexical owner. The downstream admitted host
/// must authenticate the host ceiling before calling this public API.
pub fn classify_single_source_root_with_lexical_owner_v1(
    source_id: SourceIdV2,
    source: &str,
    budget: &FrontendBudgetV2,
    root: &FrontendLexicalMaterializationRootV1<'_>,
) -> Result<FrontendSourceRootFamilyV3, FrontendSourceRootClassificationErrorV3> {
    let parsed = parse_source_with_lexical_owner(source_id, source, budget, root)
        .map_err(FrontendSourceRootClassificationErrorV3::Frontend)?;
    let [item] = parsed.value_v1().items.as_slice() else {
        return Err(if parsed.value_v1().items.is_empty() {
            FrontendSourceRootClassificationErrorV3::NoItem
        } else {
            FrontendSourceRootClassificationErrorV3::MultipleItems {
                observed: parsed.value_v1().items.len(),
            }
        });
    };
    Ok(classify_parsed_root_v3(item))
}

/// Validate source and parse limits, then count only lexer-owned storage.
/// A runtime must admit HIR, semantic, and plan storage separately before it
/// may claim a complete frontend materialization owner.
pub fn census_source_lexical_materialization_v3(
    source_id: SourceIdV2,
    source: &str,
    budget: &FrontendBudgetV2,
) -> Result<LexicalMaterializationCensusV1, FrontendTransactionErrorV2> {
    enforce_source_budget(source_id, source, budget).map_err(FrontendTransactionErrorV2::from)?;
    let projected = project_parse_budget(source_id, source, budget)
        .map_err(FrontendTransactionErrorV2::from)?;
    census_source_lexical_materialization_v2(source_id, source, &projected)
        .map_err(|error| surface_transaction_error(&error))
}

/// Compile one source document against complete immutable authority.
///
/// # Errors
///
/// Returns the first failing stage: a source-anchored document failure
/// (surface parse, HIR integrity, semantic transaction, algebra, logic,
/// planning, or pipeline assembly) or a source-less authority failure.
pub fn compile_source_document_v3(
    source_id: SourceIdV2,
    source: &str,
    authority: &ClosedFrontendAuthorityV2,
) -> Result<CompiledSourceDocumentV3, FrontendTransactionErrorV2> {
    let result = (|| {
        let parsed = parse_source(source_id, source, authority.budget())?;
        let checked = CheckedSyntaxDocumentV2::try_from_hir(parsed.hir)
            .map_err(FrontendTransactionErrorV2::from)?;
        let document = resolve_checked_document_v2(checked, authority)?;
        let (compiled, execution_items) = compile_resolved_parts(&document)?;
        Ok(CompiledSourceDocumentV3 {
            compiled,
            execution: execution_items.finish(source_id, source.as_bytes()),
        })
    })();
    result.map_err(|error| {
        cap_transaction_error(error, source_id, authority.budget().max_diagnostics())
    })
}

/// Compile under the same source-bound root used for route classification.
/// The returned wrapper keeps lexer and parser item-vector debits; subsequent
/// stage demands remain unqualified until their producer owners issue permits.
pub fn compile_source_document_with_lexical_owner_v1<'root, 'source>(
    source_id: SourceIdV2,
    source: &'source str,
    authority: &ClosedFrontendAuthorityV2,
    root: &'root FrontendLexicalMaterializationRootV1<'source>,
) -> Result<AdmittedLexicalCompiledSourceDocumentV1<'root, 'source>, FrontendTransactionErrorV2> {
    let result = (|| {
        let parsed = parse_source_with_lexical_owner(source_id, source, authority.budget(), root)?;
        parsed.try_map_v1(|hir| {
            let checked = CheckedSyntaxDocumentV2::try_from_hir(hir)
                .map_err(FrontendTransactionErrorV2::from)?;
            let document = resolve_checked_document_v2(checked, authority)?;
            let (compiled, execution_items) = compile_resolved_parts(&document)?;
            Ok(CompiledSourceDocumentV3 {
                compiled,
                execution: execution_items.finish(source_id, source.as_bytes()),
            })
        })
    })();
    result.map_err(|error| {
        cap_transaction_error(error, source_id, authority.budget().max_diagnostics())
    })
}

/// Admit the parsed and integrity-checked SHOW syntax under one source root.
/// This is a partial frontend stage: semantic, planning and execution storage
/// still require producer-owned reservations before a positive compile claim.
pub fn admit_show_syntax_v2<'root, 'source>(
    source_id: SourceIdV2,
    source: &'source str,
    budget: &FrontendBudgetV2,
    root: &'root FrontendLexicalMaterializationRootV1<'source>,
) -> Result<AdmittedShowDocumentV2<'root, 'source>, ShowSyntaxAdmissionErrorV2> {
    let parsed = parse_source_with_lexical_owner(source_id, source, budget, root)
        .map_err(ShowSyntaxAdmissionErrorV2::Frontend)?;
    if !parsed.is_show_only_v2() {
        return Err(ShowSyntaxAdmissionErrorV2::IncompleteShowGrammar);
    }
    let checked = parsed.try_map_v1(|hir| {
        CheckedSyntaxDocumentV2::try_from_show_hir_monotonic_v2(hir)
            .map_err(FrontendTransactionErrorV2::from)
            .map_err(ShowSyntaxAdmissionErrorV2::Frontend)
    })?;
    Ok(AdmittedShowDocumentV2 {
        checked,
        budget: budget.clone(),
    })
}

/// Consume SHOW syntax into the allocation-free source declaration stage.
/// Generic semantic item maps and planning remain outside this contract.
pub fn admit_show_source_declaration_v2<'root, 'source>(
    syntax: AdmittedShowDocumentV2<'root, 'source>,
    authority: &ClosedFrontendAuthorityV2,
) -> Result<AdmittedShowSourceDeclarationV2<'root, 'source>, ShowSyntaxAdmissionErrorV2> {
    if syntax.budget != *authority.budget() {
        return Err(ShowSyntaxAdmissionErrorV2::IncompleteAuthorityBudget);
    }
    let declared = syntax.checked.try_map_v1(|checked| {
        DeclaredShowSourceV2::try_new(checked, authority).map_err(|error| {
            ShowSyntaxAdmissionErrorV2::Frontend(FrontendTransactionErrorV2::from(error))
        })
    })?;
    Ok(AdmittedShowSourceDeclarationV2 { declared })
}

/// Plan the selected SHOW document under the same live root as its parser
/// owner. The checked authored item count, not a caller estimate, determines
/// the logical plan-vector debit before its first allocation. This entry is a
/// producer stage only; an installed host must authenticate `root` separately.
pub fn admit_show_plans_v2<'root, 'source>(
    declared: AdmittedShowSourceDeclarationV2<'root, 'source>,
    source: &'source str,
    root: &'root FrontendLexicalMaterializationRootV1<'source>,
) -> Result<AdmittedShowPlannedDocumentV2<'root, 'source>, ShowSyntaxAdmissionErrorV2> {
    if !declared.declared.shares_root_v2(root) {
        return Err(ShowSyntaxAdmissionErrorV2::IncompleteShowRoot);
    }
    let item_count = declared.declared.value_v1().item_count();
    let bytes = item_count
        .checked_mul(std::mem::size_of::<SemanticPlanV2>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<SemanticPlanV2>>()))
        .ok_or(ShowSyntaxAdmissionErrorV2::IncompleteShowOutputDemand)?;
    let permit = root
        .reserve_stage_v2(declared.source_id_v2(), source, bytes)
        .map_err(|_error| ShowSyntaxAdmissionErrorV2::IncompleteShowOutputStorage)?;
    let mut plans = Vec::new();
    plans
        .try_reserve_exact(item_count)
        .map_err(|_error| ShowSyntaxAdmissionErrorV2::IncompleteShowOutputStorage)?;
    if plans.capacity() > item_count {
        return Err(ShowSyntaxAdmissionErrorV2::IncompleteShowOutputStorage);
    }
    let mut planning = ShowPlanningSessionV2::new(declared.declared.value_v1());
    for index in 0..item_count {
        plans.push(
            planning
                .plan_next_v2(index)
                .map_err(ShowSyntaxAdmissionErrorV2::IncompleteShowPlanning)?,
        );
    }
    Ok(AdmittedShowPlannedDocumentV2 {
        plans,
        declared,
        _plan_permit: permit,
    })
}

/// Move the selected plan owner into a source-bound execution DTO. SHOW's
/// per-item projection and canonical digest stream are allocation-free; the
/// authored-count execution vector is debited before it is created.
pub fn admit_show_execution_v2<'root, 'source>(
    planned: AdmittedShowPlannedDocumentV2<'root, 'source>,
    source: &'source str,
    root: &'root FrontendLexicalMaterializationRootV1<'source>,
) -> Result<AdmittedShowExecutionDocumentV2<'root, 'source>, ShowSyntaxAdmissionErrorV2> {
    if !planned.declared.declared.shares_root_v2(root) {
        return Err(ShowSyntaxAdmissionErrorV2::IncompleteShowRoot);
    }
    let count = planned.plans.len();
    let bytes = count
        .checked_mul(std::mem::size_of::<ExecutionItemV3>())
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<ExecutionItemV3>>()))
        .ok_or(ShowSyntaxAdmissionErrorV2::IncompleteShowOutputDemand)?;
    let permit = root
        .reserve_stage_v2(planned.source_id_v2(), source, bytes)
        .map_err(|_error| ShowSyntaxAdmissionErrorV2::IncompleteShowOutputStorage)?;
    let mut items = ExecutionProjectionV3::try_with_capacity(count)
        .map_err(|_error| ShowSyntaxAdmissionErrorV2::IncompleteShowOutputStorage)?;
    for (index, plan) in planned.plans.iter().enumerate() {
        let resolved = planned
            .declared
            .declared
            .value_v1()
            .resolve_show_item_v2(index)
            .map_err(ShowSyntaxAdmissionErrorV2::IncompleteShowExecution)?;
        items
            .push_show(&resolved, plan)
            .map_err(ShowSyntaxAdmissionErrorV2::IncompleteShowExecution)?;
    }
    let execution = items.finish(planned.source_id_v2(), source.as_bytes());
    Ok(AdmittedShowExecutionDocumentV2 {
        execution,
        planned,
        _execution_permit: permit,
    })
}

/// Raw source cannot enter the selected owned path until the parser's error
/// message and diagnostic Vec births have a preissued request-root debit.
/// Refuse without invoking the lexer: both overlong and malformed source
/// otherwise allocate diagnostics before the first surface reservation.
/// Hosts may still consume an already-admitted syntax owner with the staged
/// APIs, but that partial path is not qualified as public source admission.
pub fn admit_show_planned_source_v2<'root, 'source>(
    _source_id: SourceIdV2,
    source: &'source str,
    authority: &ClosedFrontendAuthorityV2,
    _root: &'root FrontendLexicalMaterializationRootV1<'source>,
) -> Result<AdmittedShowPlannedDocumentV2<'root, 'source>, ShowSyntaxAdmissionErrorV2> {
    if source.len() > usize::try_from(authority.budget().max_source_bytes()).unwrap_or(usize::MAX) {
        return Err(ShowSyntaxAdmissionErrorV2::IncompleteShowSourceLimit);
    }
    Err(ShowSyntaxAdmissionErrorV2::IncompleteSourceDiagnostics)
}

/// Compile one already-sealed semantic document.
///
/// This is the checked-document pipeline entry point (ticket 07 step 9). The
/// facade exports only the source entry points; tests and the integration gate
/// use this one to drive forged documents through the exact-key assembler.
///
/// # Errors
///
/// Returns a typed failure when the document's own membership is inconsistent
/// (missing/extra/duplicate/foreign/wrong-kind), when any applicable owner stage
/// rejects, or when the assembled key sets disagree.
pub fn compile_resolved_document_v2(
    document: &SemanticDocumentV2,
) -> Result<CompiledDocumentV2, FrontendTransactionErrorV2> {
    let (compiled, _execution_items) = compile_resolved_parts(document).map_err(|error| {
        cap_transaction_error(
            error,
            document.source_id(),
            document.budget_receipt().max_diagnostics(),
        )
    })?;
    Ok(compiled)
}

fn compile_resolved_parts(
    document: &SemanticDocumentV2,
) -> Result<(CompiledDocumentV2, ExecutionProjectionV3), FrontendTransactionErrorV2> {
    let (items, rule_program, execution_items) = run_document(document)?.into_compiled()?;
    let compiled =
        CompiledDocumentV2::try_new(document.source_id(), Vec::new(), items, rule_program)?;
    enforce_diagnostic_limit(
        document.source_id(),
        compiled.diagnostics().len(),
        document.budget_receipt().max_diagnostics(),
    )?;
    Ok((compiled, execution_items))
}

/// Analyze one source document against explicit open editor authority.
///
/// A fully resolved open analysis carries the same stage payloads as a closed
/// compile but remains an [`AnalysisDocumentV2`]. Partial authority maps every
/// item to `BlockedByDiagnostics` with explicit unresolved facts; absence of
/// authority is never filled with a fabricated default.
///
/// # Errors
///
/// Returns a source-anchored hard semantic error or a source-less authority
/// error. Unresolved (non-erroneous) analysis is not an error.
pub fn analyze_source_document_v2(
    source_id: SourceIdV2,
    source: &str,
    authority: &OpenAnalysisAuthorityV2,
) -> Result<AnalysisDocumentV2, FrontendTransactionErrorV2> {
    let result = (|| {
        let parsed = parse_source(source_id, source, authority.budget())?;
        let source_len = parsed.hir.source_len;
        let missing_symbol_target_location = parsed.hir.items.iter().find_map(|item| {
            (classify_parsed_root_v3(item) == FrontendSourceRootFamilyV3::Command).then_some(
                LocatedSpanV2 {
                    source_id,
                    span: item.span(),
                },
            )
        });
        let checked = CheckedSyntaxDocumentV2::try_from_hir(parsed.hir)
            .map_err(FrontendTransactionErrorV2::from)?;
        let (Some(modules), Some(relations), Some(graph_references)) = (
            authority.modules(),
            authority.relations(),
            authority.graph_references(),
        ) else {
            return incomplete_open_analysis(
                source_id,
                source_len,
                checked,
                authority,
                missing_symbol_target_location,
            );
        };
        // Open authority currently carries no symbol-target catalog. Preserve
        // that absence as an incomplete editor result; the closed resolver's
        // `unknown_symbol_target` means a complete catalog rejected a name.
        if missing_symbol_target_location.is_some() {
            return incomplete_open_analysis(
                source_id,
                source_len,
                checked,
                authority,
                missing_symbol_target_location,
            );
        }
        let closed = ClosedFrontendAuthorityV2::try_new(
            modules.clone(),
            relations.clone(),
            graph_references.clone(),
            authority.budget().clone(),
        )?;
        let document = resolve_checked_document_v2(checked, &closed)?;
        analyze_resolved_document_v2(&document)
    })();
    result.map_err(|error| {
        cap_transaction_error(error, source_id, authority.budget().max_diagnostics())
    })
}

/// Analyze one already-sealed semantic document.
///
/// # Errors
///
/// Returns a typed failure only when the sealed document's own membership is
/// inconsistent; blocked item stages stay inside the analysis document.
pub fn analyze_resolved_document_v2(
    document: &SemanticDocumentV2,
) -> Result<AnalysisDocumentV2, FrontendTransactionErrorV2> {
    run_document(document)
        .map_err(|error| {
            cap_transaction_error(
                error,
                document.source_id(),
                document.budget_receipt().max_diagnostics(),
            )
        })?
        .into_analysis(
            document.source_id(),
            document.budget_receipt().max_diagnostics(),
        )
}

/// Format one source document from its checked parse, using only the caller's
/// checked frontend budget. This entry performs no semantic traversal.
///
/// # Errors
///
/// Returns a bounded source-anchored failure when the source exceeds the
/// budget's `max_source_bytes`, when the surface parser rejects the source, or
/// when canonical output exceeds the same source/parse budget. Output failures
/// anchor on the original document, never on generated-text byte offsets.
pub fn format_source_document_v2(
    source_id: SourceIdV2,
    source: &str,
    budget: &FrontendBudgetV2,
) -> Result<String, FrontendFailureV2> {
    enforce_source_budget(source_id, source, budget)?;
    let projected = project_parse_budget(source_id, source, budget)?;
    let rendered = format_source_document_with_parse_budget_v2(source_id, source, &projected)
        .map_err(|error| surface_failure(&error))?;
    if rendered.len() > usize::try_from(budget.max_source_bytes()).unwrap_or(usize::MAX) {
        return Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
            FrontendStageV2::Resource,
            "frontend.resource_limit_exceeded",
            FrontendSeverityV2::Error,
            LocatedSpanV2 {
                source_id,
                span: SpanV1::new(0, source.len()),
            },
            Vec::new(),
            "canonical output exceeds the frontend budget's max_source_bytes",
        )));
    }
    Ok(rendered)
}

// ---------------------------------------------------------------------------
// Stage orchestration
// ---------------------------------------------------------------------------

/// Project the parse-stage budget from one checked frontend budget.
///
/// Every dimension of a checked [`FrontendBudgetV2`] is a `NonZeroU32` widened
/// to `usize`, and [`ParseBudgetV2::try_new`] applies the same HIR safety caps, so this
/// projection has no reachable rejection. The rejection is therefore surfaced
/// with the CALLER's real source identity and the source's own exact span — the
/// same anchor the sibling resource-limit failure uses — instead of a fabricated
/// source or a byte-0 point that belongs to no document.
fn project_parse_budget(
    source_id: SourceIdV2,
    source: &str,
    budget: &FrontendBudgetV2,
) -> Result<ParseBudgetV2, FrontendFailureV2> {
    ParseBudgetV2::try_from_dimensions(|dimension| {
        let limit = match dimension {
            ParseBudgetDimensionV2::Tokens => budget.max_tokens(),
            ParseBudgetDimensionV2::DocumentItems => budget.max_document_items(),
            ParseBudgetDimensionV2::ExpressionNodes => budget.max_expression_nodes(),
            ParseBudgetDimensionV2::ExpressionDepth => budget.max_expression_depth(),
            ParseBudgetDimensionV2::RelationDepth => budget.max_relation_depth(),
            ParseBudgetDimensionV2::CollectionWidth => budget.max_collection_width(),
            ParseBudgetDimensionV2::ProjectionWidth => budget.max_projection_width(),
        };
        usize::try_from(limit).unwrap_or(usize::MAX)
    })
    .map_err(|message| {
        FrontendFailureV2::one(FrontendDiagnosticV2::new(
            FrontendStageV2::Resource,
            PARSE_BUDGET_PROJECTION_FAILED_CODE,
            FrontendSeverityV2::Error,
            LocatedSpanV2 {
                source_id,
                span: SpanV1::new(0, source.len()),
            },
            Vec::new(),
            message,
        ))
    })
}

/// Parse one source string with the authority's own parse budget.
fn parse_source(
    source_id: SourceIdV2,
    source: &str,
    budget: &FrontendBudgetV2,
) -> Result<quanta_gqlang_v2_surface::ParsedDocumentV2, FrontendTransactionErrorV2> {
    enforce_source_budget(source_id, source, budget).map_err(FrontendTransactionErrorV2::from)?;
    let projected = project_parse_budget(source_id, source, budget)
        .map_err(FrontendTransactionErrorV2::from)?;
    parse_source_document_v2(source_id, source, &projected)
        .map_err(|error| surface_transaction_error(&error))
}

fn parse_source_with_lexical_owner<'root, 'source>(
    source_id: SourceIdV2,
    source: &'source str,
    budget: &FrontendBudgetV2,
    root: &'root FrontendLexicalMaterializationRootV1<'source>,
) -> Result<AdmittedLexicalParsedDocumentV1<'root, 'source>, FrontendTransactionErrorV2> {
    enforce_source_budget(source_id, source, budget).map_err(FrontendTransactionErrorV2::from)?;
    let projected = project_parse_budget(source_id, source, budget)
        .map_err(FrontendTransactionErrorV2::from)?;
    parse_source_document_with_lexical_owner_v1(source_id, source, &projected, root)
        .map_err(|error| surface_transaction_error(&error))
}

fn enforce_source_budget(
    source_id: SourceIdV2,
    source: &str,
    budget: &FrontendBudgetV2,
) -> Result<(), FrontendFailureV2> {
    if source.len() <= usize::try_from(budget.max_source_bytes()).unwrap_or(usize::MAX) {
        return Ok(());
    }
    Err(FrontendFailureV2::one(FrontendDiagnosticV2::new(
        FrontendStageV2::Resource,
        "frontend.resource_limit_exceeded",
        FrontendSeverityV2::Error,
        LocatedSpanV2 {
            source_id,
            span: SpanV1::new(0, source.len()),
        },
        Vec::new(),
        "source exceeds the frontend budget's max_source_bytes",
    )))
}

fn enforce_diagnostic_limit(
    source_id: SourceIdV2,
    count: usize,
    limit: u32,
) -> Result<(), FrontendTransactionErrorV2> {
    if count <= usize::try_from(limit).unwrap_or(usize::MAX) {
        return Ok(());
    }
    Err(diagnostic_limit_error(source_id, count, limit))
}

fn diagnostic_limit_error(
    source_id: SourceIdV2,
    count: usize,
    limit: u32,
) -> FrontendTransactionErrorV2 {
    document_transaction_error(FrontendDiagnosticV2::new(
        FrontendStageV2::Resource,
        ANALYSIS_DIAGNOSTIC_LIMIT_CODE,
        FrontendSeverityV2::Error,
        zero_location(source_id),
        Vec::new(),
        format!("diagnostic count {count} exceeds max_diagnostics {limit}"),
    ))
}

fn cap_transaction_error(
    error: FrontendTransactionErrorV2,
    source_id: SourceIdV2,
    limit: u32,
) -> FrontendTransactionErrorV2 {
    match &error {
        FrontendTransactionErrorV2::Document(failure)
            if failure.diagnostics().len() > usize::try_from(limit).unwrap_or(usize::MAX) =>
        {
            diagnostic_limit_error(source_id, failure.diagnostics().len(), limit)
        }
        FrontendTransactionErrorV2::Authority(_) | FrontendTransactionErrorV2::Document(_) => error,
    }
}

/// Build the explicit incomplete-analysis document for partial open authority.
fn incomplete_open_analysis(
    source_id: SourceIdV2,
    source_len: usize,
    checked: CheckedSyntaxDocumentV2,
    authority: &OpenAnalysisAuthorityV2,
    missing_symbol_target_location: Option<LocatedSpanV2>,
) -> Result<AnalysisDocumentV2, FrontendTransactionErrorV2> {
    let extended = authority
        .host_sources()
        .try_extend_document(SourceRegistrationV2 {
            source_id,
            source_len,
        })
        .map_err(|error| {
            document_transaction_error(FrontendDiagnosticV2::new(
                FrontendStageV2::Integrity,
                error.code(),
                FrontendSeverityV2::Error,
                zero_location(source_id),
                Vec::new(),
                error.to_string(),
            ))
        })?;
    let declared = DeclaredDocumentV2::try_new(checked, authority.budget(), &extended)
        .map_err(FrontendTransactionErrorV2::from)?;
    let spans: BTreeMap<SemanticItemIdV2, LocatedSpanV2> = declared
        .expected()
        .map(|(id, span)| {
            (
                id,
                LocatedSpanV2 {
                    source_id: id.source_id(),
                    span,
                },
            )
        })
        .collect();
    // Authored source order, not identity order: a blocked analysis must list
    // items exactly where the closed compile lists them.
    let known: Vec<(SemanticItemIdV2, LocatedSpanV2)> = declared
        .order()
        .iter()
        .filter_map(|id| spans.get(id).map(|location| (*id, *location)))
        .collect();
    let mut missing_authority: Vec<FrontendDiagnosticV2> = Vec::new();
    for (absent, code, message) in [
        (
            authority.modules().is_none(),
            "pipeline.analysis.missing_module_authority",
            "open analysis has no validated module authority",
        ),
        (
            authority.relations().is_none(),
            "pipeline.analysis.missing_relation_authority",
            "open analysis has no relation catalog authority",
        ),
        (
            authority.graph_references().is_none(),
            "pipeline.analysis.missing_graph_authority",
            "open analysis has no graph selector authority",
        ),
    ] {
        if !absent {
            continue;
        }
        let Some((_, location)) = known.first() else {
            continue;
        };
        missing_authority.push(FrontendDiagnosticV2::new(
            FrontendStageV2::Declaration,
            code,
            FrontendSeverityV2::Warning,
            *location,
            Vec::new(),
            message,
        ));
    }
    if let Some(location) = missing_symbol_target_location {
        missing_authority.push(FrontendDiagnosticV2::new(
            FrontendStageV2::Declaration,
            "pipeline.analysis.missing_symbol_target_authority",
            FrontendSeverityV2::Warning,
            location,
            Vec::new(),
            "open analysis has no closed symbol target catalog for rename commands",
        ));
    }
    let diagnostics = canonicalize_diagnostics(missing_authority);
    enforce_diagnostic_limit(
        source_id,
        diagnostics.len(),
        authority.budget().max_diagnostics(),
    )?;
    let items = known
        .iter()
        .map(|_item| AnalysisItemV2::BlockedByDiagnostics(diagnostics.clone().into_boxed_slice()))
        .collect::<Vec<_>>();
    let unresolved = known
        .iter()
        .map(|(_, location)| {
            UnresolvedAnalysisFactV2::new(OPEN_AUTHORITY_INCOMPLETE_CODE, *location)
        })
        .collect::<Vec<_>>();
    Ok(AnalysisDocumentV2 {
        source_id,
        order: known
            .iter()
            .map(|(id, _location)| *id)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        items: items.into_boxed_slice(),
        rule_program: None,
        unresolved: unresolved.into_boxed_slice(),
    })
}

mod assembly;

use assembly::{run_document, zero_location};

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    reason = "owner-local assembly tests inspect exact sealed fixture results"
)]
#[path = "compile/tests.rs"]
mod tests;
