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
    SemanticPlanInputV2, SemanticPlanV2,
};
use quanta_gqlang_v2_sema::{
    resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityV2,
    DeclaredDocumentV2, FrontendBudgetV2, FrontendDiagnosticV2, FrontendFailureV2,
    FrontendSeverityV2, FrontendStageV2, FrontendTransactionErrorV2, LocatedSpanV2,
    OpenAnalysisAuthorityV2, ResolvedCatalogItemV2, ResolvedCatalogPayloadV2, ResolvedCheckV2,
    ResolvedCompositionBodyV2, ResolvedCompositionProducerV2, ResolvedCompositionV2,
    ResolvedMetaKindV2, SemanticDocumentV2, SemanticItemIdV2, SemanticItemV2, SourceIdV2,
    SourceRegistrationV2, SpanV1,
};
use quanta_gqlang_v2_surface::{
    classify_parsed_root_v3, format_source_document_with_parse_budget_v2, parse_source_document_v2,
    ParseBudgetDimensionV2, ParseBudgetV2,
};

pub use quanta_gqlang_v2_surface::SourceRootFamilyV3 as FrontendSourceRootFamilyV3;

use crate::diagnostic::{
    algebra_transaction_error, assembly_transaction_error, canonicalize_diagnostics,
    document_transaction_error, logic_transaction_error, planning_transaction_error,
    surface_failure, surface_transaction_error, DocumentAssemblyCodeV2,
};
use crate::execution::{
    project_composition, project_graph_query, project_relation, project_relation_root,
    ExecutionClosureFailureV3, ExecutionCompositionBodyProjectionV3,
    ExecutionCompositionProducerProjectionV3, ExecutionCompositionProjectionFailureV3,
    ExecutionCompositionV3, ExecutionDocumentV3, ExecutionFamilyV3, ExecutionItemV3,
    ExecutionRelationV3,
};

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
            Self::Relation { item_id, .. }
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
            Self::Relation { plan, .. }
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
            Self::Relation { .. }
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
            execution: ExecutionDocumentV3::new(source_id, source.as_bytes(), execution_items),
        })
    })();
    result.map_err(|error| {
        cap_transaction_error(error, source_id, authority.budget().max_diagnostics())
    })
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
) -> Result<(CompiledDocumentV2, Vec<ExecutionItemV3>), FrontendTransactionErrorV2> {
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
        let checked = CheckedSyntaxDocumentV2::try_from_hir(parsed.hir)
            .map_err(FrontendTransactionErrorV2::from)?;
        let (Some(modules), Some(relations), Some(graph_references)) = (
            authority.modules(),
            authority.relations(),
            authority.graph_references(),
        ) else {
            return incomplete_open_analysis(source_id, source_len, checked, authority);
        };
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

/// The completed run of every applicable stage over one sealed document.
struct DocumentRunV2 {
    items: Vec<ItemRunV2>,
    rule_program: Option<CompiledRuleProgramV2>,
    rule_blocked: Option<FrontendFailureV2>,
}

type CompiledPartsV2 = (
    Vec<CompiledItemV2>,
    Option<CompiledRuleProgramV2>,
    Vec<ExecutionItemV3>,
);

/// Boxed planned payload: the plan plus its two execution projections dwarf the
/// other outcomes, so the box keeps every `ItemRunV2` variant pointer-sized.
#[derive(Clone, Debug, PartialEq, Eq)]
struct PlannedRunV2 {
    planned: Box<PlannedItemV2>,
    relation: Option<ExecutionRelationV3>,
    composition: Option<ExecutionCompositionV3>,
}

/// One item's stage outcome.
#[derive(Clone, Debug, PartialEq, Eq)]
enum ItemRunV2 {
    /// Every applicable stage completed.
    Planned(Box<PlannedRunV2>),
    /// The item is a checked non-plan declaration.
    NoPlan {
        item_id: SemanticItemIdV2,
        reason: NoPlanReasonV2,
    },
    /// A stage failed; the owner failure guarantees non-empty ordered diagnostics.
    Blocked {
        item_id: SemanticItemIdV2,
        failure: FrontendFailureV2,
    },
}

impl DocumentRunV2 {
    /// The exact compiled item list, failing closed on the first blocked stage.
    fn into_compiled(self) -> Result<CompiledPartsV2, FrontendTransactionErrorV2> {
        if let Some(failure) = self.rule_blocked {
            return Err(FrontendTransactionErrorV2::Document(failure.deduplicated()));
        }
        let mut items = Vec::with_capacity(self.items.len());
        let mut execution_items = Vec::new();
        for item in self.items {
            match item {
                ItemRunV2::Planned(run) => {
                    if let Some((family, plan)) = execution_identity(run.planned.as_ref()) {
                        execution_items.push(ExecutionItemV3::new(
                            run.planned.item_id(),
                            family,
                            run.relation,
                            run.composition,
                            plan,
                        ));
                    }
                    items.push(CompiledItemV2::Planned(run.planned));
                }
                ItemRunV2::NoPlan { item_id, reason } => {
                    items.push(CompiledItemV2::NoPlan { item_id, reason });
                }
                ItemRunV2::Blocked { failure, .. } => {
                    return Err(FrontendTransactionErrorV2::Document(failure.deduplicated()));
                }
            }
        }
        Ok((items, self.rule_program, execution_items))
    }

    /// The exact analysis item list; blocked stages stay explicit.
    fn into_analysis(
        self,
        source_id: SourceIdV2,
        max_diagnostics: u32,
    ) -> Result<AnalysisDocumentV2, FrontendTransactionErrorV2> {
        // Count canonical diagnostics before a blocked rule-program diagnostic
        // is cloned onto every authored rule member.
        let mut diagnostics = self
            .rule_blocked
            .as_ref()
            .map(|failure| failure.diagnostics().to_vec())
            .unwrap_or_default();
        for item in &self.items {
            if let ItemRunV2::Blocked { failure, .. } = item {
                diagnostics.extend(failure.diagnostics().iter().cloned());
            }
        }
        enforce_diagnostic_limit(
            source_id,
            canonicalize_diagnostics(diagnostics).len(),
            max_diagnostics,
        )?;
        let rule_blocked = self.rule_blocked;
        let mut order = Vec::with_capacity(self.items.len());
        let items = self
            .items
            .into_iter()
            .map(|item| match item {
                ItemRunV2::Planned(run) => {
                    let planned = run.planned;
                    if let PlannedItemV2::RuleMember(member) = planned.as_ref() {
                        order.push(member.member_item_id());
                        match &rule_blocked {
                            Some(failure) => {
                                AnalysisItemV2::BlockedByDiagnostics(failure.diagnostics().into())
                            }
                            None => AnalysisItemV2::Complete(planned),
                        }
                    } else {
                        order.push(planned.item_id());
                        AnalysisItemV2::Complete(planned)
                    }
                }
                ItemRunV2::NoPlan { item_id, reason } => {
                    order.push(item_id);
                    AnalysisItemV2::NotApplicable(reason)
                }
                ItemRunV2::Blocked { item_id, failure } => {
                    order.push(item_id);
                    AnalysisItemV2::BlockedByDiagnostics(failure.diagnostics().into())
                }
            })
            .collect::<Vec<_>>();
        Ok(AnalysisDocumentV2 {
            source_id,
            order: order.into_boxed_slice(),
            items: items.into_boxed_slice(),
            rule_program: self.rule_program,
            unresolved: Box::new([]),
        })
    }
}

/// Run every applicable stage of one sealed document.
///
/// Only membership inconsistency is a hard error here; every owner stage
/// failure is collected so the analysis path can report it per item while the
/// compile path fails closed on the same collection.
fn run_document(
    document: &SemanticDocumentV2,
) -> Result<DocumentRunV2, FrontendTransactionErrorV2> {
    let membership = DocumentMembershipV2::of(document)?;
    let rule_members = membership.rule_member_ids();
    let rule_program_id = membership.rule_program_id();
    let mut algebra = AlgebraSessionV2::new(document);
    let mut planning = PlanningSessionV2::new(document);

    let (rule_program, rule_blocked) = match rule_program_id {
        Some(program_id) => {
            match compile_rule_program(document, &mut planning, program_id, &rule_members) {
                Ok(program) => (Some(program), None),
                Err(FrontendTransactionErrorV2::Document(failure)) => (None, Some(failure)),
                Err(error @ FrontendTransactionErrorV2::Authority(_)) => return Err(error),
            }
        }
        None => (None, None),
    };

    let mut items = Vec::with_capacity(membership.order().len());
    let mut produced_plan: BTreeSet<SemanticItemIdV2> = BTreeSet::new();
    let mut produced_member: BTreeSet<SemanticItemIdV2> = BTreeSet::new();
    let mut produced_no_plan: BTreeSet<SemanticItemIdV2> = BTreeSet::new();
    let mut blocked_stage = false;
    for id in membership.order() {
        let location = membership.location(*id);
        match run_item(document, &membership, &mut algebra, &mut planning, *id) {
            Ok(ItemRunV2::Planned(run)) => {
                if matches!(run.planned.as_ref(), PlannedItemV2::RuleMember(_)) {
                    record_produced_identity(
                        &mut produced_member,
                        *id,
                        location,
                        "one authored identity produced two results",
                    )?;
                } else {
                    record_produced_identity(
                        &mut produced_plan,
                        *id,
                        location,
                        "one authored identity produced two results",
                    )?;
                }
                items.push(ItemRunV2::Planned(run));
            }
            Ok(ItemRunV2::NoPlan { item_id, reason }) => {
                record_produced_identity(
                    &mut produced_no_plan,
                    *id,
                    location,
                    "one authored identity produced two non-plan results",
                )?;
                items.push(ItemRunV2::NoPlan { item_id, reason });
            }
            Ok(blocked @ ItemRunV2::Blocked { .. }) => {
                blocked_stage = true;
                items.push(blocked);
            }
            Err(FrontendTransactionErrorV2::Document(failure)) => {
                blocked_stage = true;
                items.push(ItemRunV2::Blocked {
                    item_id: *id,
                    failure,
                });
            }
            Err(error @ FrontendTransactionErrorV2::Authority(_)) => return Err(error),
        }
    }

    // A blocked owner stage is reported first: the produced key sets are
    // incomplete by construction, so comparing them would mask the real stage
    // failure with a pipeline membership code.
    if blocked_stage || rule_blocked.is_some() {
        return Ok(DocumentRunV2 {
            items,
            rule_program,
            rule_blocked,
        });
    }

    validate_produced_membership(
        document.source_id(),
        &membership,
        &produced_plan,
        &produced_member,
        &produced_no_plan,
    )?;

    Ok(DocumentRunV2 {
        items,
        rule_program,
        rule_blocked,
    })
}

fn record_produced_identity(
    produced: &mut BTreeSet<SemanticItemIdV2>,
    id: SemanticItemIdV2,
    location: LocatedSpanV2,
    message: &'static str,
) -> Result<(), FrontendTransactionErrorV2> {
    if produced.insert(id) {
        Ok(())
    } else {
        Err(assembly(
            location,
            DocumentAssemblyCodeV2::DuplicateItemResult,
            message,
        ))
    }
}

fn validate_produced_membership(
    source_id: SourceIdV2,
    membership: &DocumentMembershipV2<'_>,
    produced_plan: &BTreeSet<SemanticItemIdV2>,
    produced_member: &BTreeSet<SemanticItemIdV2>,
    produced_no_plan: &BTreeSet<SemanticItemIdV2>,
) -> Result<(), FrontendTransactionErrorV2> {
    let expected_plan = membership.plan_required_ids();
    let expected_member: BTreeSet<SemanticItemIdV2> =
        membership.rule_member_ids().into_iter().collect();
    let expected_no_plan = membership.no_plan_ids();
    if produced_member != &expected_member {
        let anchor = expected_member
            .iter()
            .chain(produced_member.iter())
            .find(|id| expected_member.contains(id) != produced_member.contains(id))
            .copied();
        let location =
            anchor.map_or_else(|| zero_location(source_id), |id| membership.location(id));
        return Err(assembly(
            location,
            DocumentAssemblyCodeV2::RuleMembershipMismatch,
            "the produced rule members disagree with the authored rule set",
        ));
    }
    for id in &expected_plan {
        if !produced_plan.contains(id) {
            return Err(assembly(
                membership.location(*id),
                DocumentAssemblyCodeV2::MissingItemResult,
                "an applicable authored item produced no plan result",
            ));
        }
    }
    for id in produced_plan {
        if !expected_plan.contains(id) {
            return Err(assembly(
                membership.location(*id),
                DocumentAssemblyCodeV2::ExtraItemResult,
                "a produced plan has no applicable authored identity",
            ));
        }
    }
    for id in &expected_no_plan {
        if !produced_no_plan.contains(id) {
            return Err(assembly(
                membership.location(*id),
                DocumentAssemblyCodeV2::MissingItemResult,
                "a non-executable declaration produced no checked non-plan reason",
            ));
        }
    }
    for id in produced_no_plan {
        if !expected_no_plan.contains(id) {
            return Err(assembly(
                membership.location(*id),
                DocumentAssemblyCodeV2::ExtraItemResult,
                "a produced non-plan result has no authored declaration identity",
            ));
        }
    }
    Ok(())
}

/// Run the applicable stages for one authored item.
fn run_item(
    document: &SemanticDocumentV2,
    membership: &DocumentMembershipV2<'_>,
    algebra: &mut AlgebraSessionV2<'_>,
    planning: &mut PlanningSessionV2<'_>,
    id: SemanticItemIdV2,
) -> Result<ItemRunV2, FrontendTransactionErrorV2> {
    let Some(item) = membership.item(id) else {
        return Err(assembly(
            membership.location(id),
            DocumentAssemblyCodeV2::MissingItemResult,
            "an authored item has no produced semantic result",
        ));
    };
    match item {
        SemanticItemV2::Relation(_) => {
            let lowered = algebra
                .lower_relation_item(id)
                .map_err(|error| algebra_error(document, id, &error))?;
            let relation = project_relation(document, &lowered)
                .map_err(|failure| closure_error(document, id, failure))?;
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Relational(Box::new(lowered)),
            )?;
            Ok(ItemRunV2::Planned(Box::new(PlannedRunV2 {
                planned: Box::new(PlannedItemV2::Relation { item_id: id, plan }),
                relation: Some(relation),
                composition: None,
            })))
        }
        SemanticItemV2::Graph(graph) => {
            let contract = GraphQueryContractV2::from_resolved(graph)
                .map_err(|error| logic_transaction_error(&error))?;
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Graph(contract),
            )?;
            Ok(planned_run(PlannedItemV2::Graph { item_id: id, plan }))
        }
        SemanticItemV2::Check(check) => {
            let contract = check_contract(check)?;
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Check(contract),
            )?;
            Ok(planned_run(PlannedItemV2::Check { item_id: id, plan }))
        }
        SemanticItemV2::Presentation(presentation) => {
            let contract = PresentationContractV2::from_resolved(presentation)
                .map_err(|error| planning_transaction_error(&error))?;
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Presentation(contract),
            )?;
            Ok(planned_run(PlannedItemV2::Presentation {
                item_id: id,
                plan,
            }))
        }
        SemanticItemV2::Composition(composition) => {
            let mut producer_projections = Vec::with_capacity(composition.bindings().len());
            for binding in composition.bindings() {
                let projection = match binding.producer() {
                    ResolvedCompositionProducerV2::Query(query) => {
                        let root = algebra
                            .lower_nested_relation(composition.item_id(), query.root())
                            .map_err(|error| algebra_error(document, id, &error))?;
                        ExecutionCompositionProducerProjectionV3::Relation(
                            project_relation_root(document, query.item_id(), &root)
                                .map_err(|failure| closure_error(document, id, failure))?,
                        )
                    }
                    ResolvedCompositionProducerV2::Sample { .. } => {
                        ExecutionCompositionProducerProjectionV3::Sample
                    }
                    ResolvedCompositionProducerV2::Graph { graph, .. } => {
                        let contract = GraphQueryContractV2::from_resolved(graph)
                            .map_err(|error| logic_transaction_error(&error))?;
                        ExecutionCompositionProducerProjectionV3::Graph {
                            item_id: contract.item_id(),
                            query: project_graph_query(&contract),
                        }
                    }
                };
                producer_projections.push(projection);
            }
            let (body, body_projection) = composition_body_input(document, algebra, composition)?;
            let input = CheckedCompositionPlanInputV2::new(composition, body)
                .map_err(|error| planning_transaction_error(&error))?;
            let plan = planning_plan(
                document,
                planning,
                id,
                &SemanticPlanInputV2::Composition(Box::new(input)),
            )?;
            let execution = project_composition(composition, producer_projections, body_projection)
                .map_err(|failure| composition_projection_error(document, id, failure))?;
            Ok(ItemRunV2::Planned(Box::new(PlannedRunV2 {
                planned: Box::new(PlannedItemV2::Composition { item_id: id, plan }),
                relation: None,
                composition: Some(execution),
            })))
        }
        SemanticItemV2::Meta(meta) => match meta.kind() {
            ResolvedMetaKindV2::RequireCertified => {
                let plan = planning_plan(
                    document,
                    planning,
                    id,
                    &SemanticPlanInputV2::MetaRequireCertified(meta.clone()),
                )?;
                Ok(planned_run(PlannedItemV2::MetaRequireCertified {
                    item_id: id,
                    plan,
                }))
            }
            ResolvedMetaKindV2::UsePack { .. } => Ok(ItemRunV2::NoPlan {
                item_id: id,
                reason: NoPlanReasonV2::MetaActivation,
            }),
        },
        SemanticItemV2::Rule(_) => match membership.rule_program_id() {
            Some(program_id) => Ok(planned_run(PlannedItemV2::RuleMember(
                RuleProgramMemberV2::new(program_id, id),
            ))),
            None => Err(assembly(
                membership.location(id),
                DocumentAssemblyCodeV2::RuleMembershipMismatch,
                "a source rule has no document rule program",
            )),
        },
        SemanticItemV2::Catalog(catalog) => {
            if let Some(view) = catalog.inline_view() {
                algebra
                    .lower_nested_relation(id, view.root())
                    .map_err(|error| algebra_error(document, id, &error))?;
            }
            Ok(ItemRunV2::NoPlan {
                item_id: id,
                reason: NoPlanReasonV2::CatalogDeclaration {
                    kind: catalog_declaration_kind(catalog),
                },
            })
        }
        SemanticItemV2::Declaration(_) => Ok(ItemRunV2::NoPlan {
            item_id: id,
            reason: NoPlanReasonV2::ValueDeclaration,
        }),
    }
}

/// Build and plan the one document-wide rule program.
///
/// Stratification is NOT repeated here: the program's sole ingress
/// ([`RuleProgramContractV2::from_document`]) already charged the document's
/// rule-graph allowance once and retained the proof, and this DTO carries that
/// exact proof.
fn compile_rule_program(
    document: &SemanticDocumentV2,
    planning: &mut PlanningSessionV2<'_>,
    program_id: SemanticItemIdV2,
    members: &[SemanticItemIdV2],
) -> Result<CompiledRuleProgramV2, FrontendTransactionErrorV2> {
    let contract = RuleProgramContractV2::from_document(document)
        .map_err(|error| logic_transaction_error(&error))?;
    let stratification = contract.stratification().clone();
    let plan = planning_plan(
        document,
        planning,
        program_id,
        &SemanticPlanInputV2::Rule(contract),
    )?;
    Ok(CompiledRuleProgramV2 {
        program_id,
        members: members
            .iter()
            .map(|member| RuleProgramMemberV2::new(program_id, *member))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        plan,
        stratification,
    })
}

/// Build one CHECK contract with its identity-checked nested graph contract.
fn check_contract(check: &ResolvedCheckV2) -> Result<CheckContractV2, FrontendTransactionErrorV2> {
    let graph = match check.nested_graph() {
        Some(nested) => Some(
            GraphQueryContractV2::from_resolved(nested)
                .map_err(|error| logic_transaction_error(&error))?,
        ),
        None => None,
    };
    CheckContractV2::from_resolved(check, graph).map_err(|error| planning_transaction_error(&error))
}

/// Build one composition body plan input.
fn composition_body_input(
    document: &SemanticDocumentV2,
    algebra: &mut AlgebraSessionV2<'_>,
    composition: &ResolvedCompositionV2,
) -> Result<
    (
        CompositionBodyPlanInputV2,
        ExecutionCompositionBodyProjectionV3,
    ),
    FrontendTransactionErrorV2,
> {
    let owner = composition.item_id();
    match composition.body() {
        ResolvedCompositionBodyV2::Query(body) => {
            let root = algebra
                .lower_nested_relation(owner, body.root())
                .map_err(|error| algebra_error(document, owner, &error))?;
            let execution = project_relation_root(document, body.item_id(), &root)
                .map_err(|failure| closure_error(document, owner, failure))?;
            let body = CompositionRelationalBodyPlanInputV2::try_new(body.item_id(), root)
                .map_err(|reason| {
                    planning_transaction_error(&PlanningDiagnosticV1::InvalidComposition {
                        reason,
                        span: composition.location().clone(),
                    })
                })?;
            Ok((
                CompositionBodyPlanInputV2::Relational(body),
                ExecutionCompositionBodyProjectionV3::Relation(execution),
            ))
        }
        ResolvedCompositionBodyV2::Graph(graph) => {
            let contract = GraphQueryContractV2::from_resolved(graph)
                .map_err(|error| logic_transaction_error(&error))?;
            let execution = project_graph_query(&contract);
            let item_id = contract.item_id();
            Ok((
                CompositionBodyPlanInputV2::Graph(contract),
                ExecutionCompositionBodyProjectionV3::Graph {
                    item_id,
                    query: execution,
                },
            ))
        }
        ResolvedCompositionBodyV2::Check(check) => Ok((
            CompositionBodyPlanInputV2::Check(check_contract(check)?),
            ExecutionCompositionBodyProjectionV3::Check,
        )),
        ResolvedCompositionBodyV2::Explain(explain) => Ok((
            CompositionBodyPlanInputV2::Explain(
                PresentationContractV2::from_resolved(explain)
                    .map_err(|error| planning_transaction_error(&error))?,
            ),
            ExecutionCompositionBodyProjectionV3::Explain,
        )),
    }
}

fn planned_run(planned: PlannedItemV2) -> ItemRunV2 {
    ItemRunV2::Planned(Box::new(PlannedRunV2 {
        planned: Box::new(planned),
        relation: None,
        composition: None,
    }))
}

fn execution_identity(planned: &PlannedItemV2) -> Option<(ExecutionFamilyV3, &SemanticPlanV2)> {
    match planned {
        PlannedItemV2::Relation { plan, .. } => Some((ExecutionFamilyV3::Relation, plan)),
        PlannedItemV2::Graph { plan, .. } => Some((ExecutionFamilyV3::Graph, plan)),
        PlannedItemV2::Check { plan, .. } => Some((ExecutionFamilyV3::Check, plan)),
        PlannedItemV2::Presentation { plan, .. } => Some((ExecutionFamilyV3::Presentation, plan)),
        PlannedItemV2::Composition { plan, .. } => Some((ExecutionFamilyV3::Composition, plan)),
        PlannedItemV2::MetaRequireCertified { plan, .. } => {
            Some((ExecutionFamilyV3::MetaRequireCertified, plan))
        }
        PlannedItemV2::RuleMember(_) => None,
    }
}

/// Map one resolved catalog payload to its closed non-plan reason kind.
fn catalog_declaration_kind(catalog: &ResolvedCatalogItemV2) -> CatalogDeclarationKindV2 {
    match catalog.payload() {
        ResolvedCatalogPayloadV2::View { .. } => CatalogDeclarationKindV2::View,
        ResolvedCatalogPayloadV2::Model { .. } => CatalogDeclarationKindV2::Model,
        ResolvedCatalogPayloadV2::Pack { .. } => CatalogDeclarationKindV2::Pack,
        ResolvedCatalogPayloadV2::Profile { .. } => CatalogDeclarationKindV2::Profile,
        ResolvedCatalogPayloadV2::Gate { .. } => CatalogDeclarationKindV2::Gate,
        ResolvedCatalogPayloadV2::RuleBinding { .. } => CatalogDeclarationKindV2::RuleBinding,
    }
}

/// Plan one input and require the plan-required outcome.
fn planning_plan(
    document: &SemanticDocumentV2,
    planning: &mut PlanningSessionV2<'_>,
    id: SemanticItemIdV2,
    input: &SemanticPlanInputV2,
) -> Result<SemanticPlanV2, FrontendTransactionErrorV2> {
    match planning.plan_outcome(input) {
        Ok(PlanOutcomeV2::Plan(plan)) => Ok(*plan),
        Ok(PlanOutcomeV2::NoPlan(_reason)) => Err(assembly(
            item_location_of(document, id),
            DocumentAssemblyCodeV2::PlanOutcomeMismatch,
            "a plan-required item returned a checked non-plan outcome",
        )),
        Err(error) => Err(planning_transaction_error(&error)),
    }
}

/// Project one algebra diagnostic with an explicit same-document fallback.
fn algebra_error(
    document: &SemanticDocumentV2,
    id: SemanticItemIdV2,
    error: &quanta_gqlang_v2_algebra::AlgebraDiagnosticV2,
) -> FrontendTransactionErrorV2 {
    algebra_transaction_error(error, item_location_of(document, id))
}

/// Map an execution expression-closure failure to a stable assembly error.
fn closure_error(
    document: &SemanticDocumentV2,
    id: SemanticItemIdV2,
    failure: ExecutionClosureFailureV3,
) -> FrontendTransactionErrorV2 {
    let location = item_location_of(document, id);
    match failure {
        ExecutionClosureFailureV3::Dangling => assembly(
            location,
            DocumentAssemblyCodeV2::DanglingExecutionReference,
            "an executable item references an expression the sealed document does not carry",
        ),
        ExecutionClosureFailureV3::UnregisteredSymbol => assembly(
            location,
            DocumentAssemblyCodeV2::UnregisteredExecutionSymbol,
            "an executable expression's closed identity has no registered symbol",
        ),
        ExecutionClosureFailureV3::Foreign => assembly(
            location,
            DocumentAssemblyCodeV2::ForeignItemResult,
            "an executable item references an expression from another source",
        ),
    }
}

/// Map an impossible producer/body projection drift to a typed pipeline
/// diagnostic instead of truncating a binding or aborting the process.
fn composition_projection_error(
    document: &SemanticDocumentV2,
    id: SemanticItemIdV2,
    failure: ExecutionCompositionProjectionFailureV3,
) -> FrontendTransactionErrorV2 {
    let (code, message) = match failure {
        ExecutionCompositionProjectionFailureV3::Producer => (
            DocumentAssemblyCodeV2::CompositionProducerMismatch,
            "a composition producer and its checked execution projection disagree",
        ),
        ExecutionCompositionProjectionFailureV3::Body => (
            DocumentAssemblyCodeV2::CompositionBodyMismatch,
            "a composition body and its checked execution projection disagree",
        ),
    };
    assembly(item_location_of(document, id), code, message)
}

/// Mint one pipeline-owned assembly failure.
fn assembly(
    location: LocatedSpanV2,
    code: DocumentAssemblyCodeV2,
    message: &'static str,
) -> FrontendTransactionErrorV2 {
    assembly_transaction_error(location, code, message)
}

/// The only location a fully empty document can offer.
fn zero_location(source_id: SourceIdV2) -> LocatedSpanV2 {
    LocatedSpanV2 {
        source_id,
        span: SpanV1::point(0),
    }
}

/// The authored location of one semantic item.
///
/// A catalog declaration retains its checked location through its inline VIEW
/// where it has one, and otherwise through the checked
/// [`quanta_gqlang_v2_sema::ResolvedCatalogReferenceV2`] its payload resolved;
/// both are document-owned facts, so a declaration is never anchored on a
/// fabricated byte 0.
fn item_location(item: &SemanticItemV2) -> Option<LocatedSpanV2> {
    match item {
        SemanticItemV2::Relation(relation) => Some(relation.location().location()),
        SemanticItemV2::Graph(graph) => Some(graph.location().location()),
        SemanticItemV2::Check(check) => Some(check.rule().location().location()),
        SemanticItemV2::Presentation(presentation) => Some(presentation.location().location()),
        SemanticItemV2::Composition(composition) => Some(composition.location().location()),
        SemanticItemV2::Meta(meta) => Some(meta.location().location()),
        SemanticItemV2::Catalog(catalog) => catalog_location(catalog),
        SemanticItemV2::Declaration(declaration) => Some(declaration.body().location().location()),
        SemanticItemV2::Rule(rule) => Some(rule.body().expression().location().location()),
    }
}

/// The checked location of one catalog declaration, from the facts it retains.
///
/// A View alias/inline body anchors on the inline VIEW's own typed relation; a
/// declaration without one anchors on its payload's exact checked reference (the
/// queried/source/check reference the declaration resolved), falling back to the
/// first canonical inline-View dependency. Only a catalog item that resolves no
/// reference at all has no location of its own.
fn catalog_location(catalog: &ResolvedCatalogItemV2) -> Option<LocatedSpanV2> {
    if let Some(view) = catalog.inline_view() {
        return Some(view.location().location());
    }
    let payload_reference = match catalog.payload() {
        ResolvedCatalogPayloadV2::View { query_ref } => query_ref.as_ref(),
        ResolvedCatalogPayloadV2::Model { source } => Some(source),
        ResolvedCatalogPayloadV2::Pack { source, .. } => Some(source),
        ResolvedCatalogPayloadV2::Profile { capabilities, .. } => capabilities.first(),
        ResolvedCatalogPayloadV2::Gate { checks, .. } => checks.first(),
        ResolvedCatalogPayloadV2::RuleBinding { check_ref } => Some(check_ref),
    };
    payload_reference
        .or_else(|| catalog.inline_view_dependencies().first())
        .map(|reference| reference.location().location())
}

/// A located anchor for one authored identity.
fn item_location_of(document: &SemanticDocumentV2, id: SemanticItemIdV2) -> LocatedSpanV2 {
    document
        .item(id)
        .and_then(item_location)
        .unwrap_or_else(|| zero_location(document.source_id()))
}

// ---------------------------------------------------------------------------
// Exact-key membership
// ---------------------------------------------------------------------------

/// The authored membership of one sealed document, verified exactly once.
struct DocumentMembershipV2<'a> {
    items: BTreeMap<SemanticItemIdV2, &'a SemanticItemV2>,
    order: Vec<SemanticItemIdV2>,
    rule_program_id: Option<SemanticItemIdV2>,
}

impl<'a> DocumentMembershipV2<'a> {
    /// Verify duplicate, missing, extra, foreign, and wrong-kind membership.
    fn of(document: &'a SemanticDocumentV2) -> Result<Self, FrontendTransactionErrorV2> {
        let mut items: BTreeMap<SemanticItemIdV2, &'a SemanticItemV2> = BTreeMap::new();
        for (key, item) in document.items() {
            if key.source_id() != document.source_id() || item.item_id() != *key {
                return Err(assembly(
                    item_location(item).unwrap_or_else(|| zero_location(document.source_id())),
                    DocumentAssemblyCodeV2::WrongKindItemResult,
                    "a produced item is registered under a foreign or mismatched identity",
                ));
            }
            if items.insert(*key, item).is_some() {
                return Err(assembly(
                    item_location(item).unwrap_or_else(|| zero_location(document.source_id())),
                    DocumentAssemblyCodeV2::DuplicateItemResult,
                    "a produced item identity was registered twice",
                ));
            }
        }
        let mut order = Vec::with_capacity(document.order().len());
        let mut seen = BTreeSet::new();
        for id in document.order() {
            if id.source_id() != document.source_id() {
                return Err(assembly(
                    zero_location(document.source_id()),
                    DocumentAssemblyCodeV2::ForeignItemResult,
                    "the authored order names an item of another source",
                ));
            }
            if !seen.insert(*id) {
                return Err(assembly(
                    items
                        .get(id)
                        .and_then(|item| item_location(item))
                        .unwrap_or_else(|| zero_location(document.source_id())),
                    DocumentAssemblyCodeV2::DuplicateItemResult,
                    "the authored order names one item twice",
                ));
            }
            if !items.contains_key(id) {
                return Err(assembly(
                    zero_location(document.source_id()),
                    DocumentAssemblyCodeV2::MissingItemResult,
                    "an authored item has no produced semantic result",
                ));
            }
            order.push(*id);
        }
        if let Some(extra) = items.keys().find(|id| !seen.contains(id)) {
            let location = items
                .get(extra)
                .and_then(|item| item_location(item))
                .unwrap_or_else(|| zero_location(document.source_id()));
            return Err(assembly(
                location,
                DocumentAssemblyCodeV2::ExtraItemResult,
                "a produced semantic item was not authored in document order",
            ));
        }
        let rule_program_id = order.iter().copied().find(|id| {
            items
                .get(id)
                .is_some_and(|item| matches!(item, SemanticItemV2::Rule(_)))
        });
        Ok(Self {
            items,
            order,
            rule_program_id,
        })
    }

    /// Authored source order.
    fn order(&self) -> &[SemanticItemIdV2] {
        &self.order
    }

    /// Exact authored item by identity, present for every verified order id.
    fn item(&self, id: SemanticItemIdV2) -> Option<&'a SemanticItemV2> {
        self.items.get(&id).copied()
    }

    /// A located anchor for one authored identity.
    fn location(&self, id: SemanticItemIdV2) -> LocatedSpanV2 {
        self.item(id)
            .and_then(item_location)
            .unwrap_or_else(|| zero_location(id.source_id()))
    }

    /// The stable document rule program identity (first rule in order).
    fn rule_program_id(&self) -> Option<SemanticItemIdV2> {
        self.rule_program_id
    }

    /// Authored rule identities in source order.
    fn rule_member_ids(&self) -> Vec<SemanticItemIdV2> {
        self.order
            .iter()
            .copied()
            .filter(|id| {
                self.item(*id)
                    .is_some_and(|item| matches!(item, SemanticItemV2::Rule(_)))
            })
            .collect()
    }

    /// Identities that must carry one semantic plan.
    fn plan_required_ids(&self) -> BTreeSet<SemanticItemIdV2> {
        self.order
            .iter()
            .copied()
            .filter(|id| {
                self.item(*id).is_some_and(|item| {
                    matches!(
                        item,
                        SemanticItemV2::Relation(_)
                            | SemanticItemV2::Graph(_)
                            | SemanticItemV2::Check(_)
                            | SemanticItemV2::Presentation(_)
                            | SemanticItemV2::Composition(_)
                    ) || matches!(
                        item,
                        SemanticItemV2::Meta(meta)
                            if matches!(meta.kind(), ResolvedMetaKindV2::RequireCertified)
                    )
                })
            })
            .collect()
    }

    /// Identities that must carry one checked non-plan reason.
    fn no_plan_ids(&self) -> BTreeSet<SemanticItemIdV2> {
        self.order
            .iter()
            .copied()
            .filter(|id| {
                self.item(*id).is_some_and(|item| {
                    matches!(
                        item,
                        SemanticItemV2::Catalog(_) | SemanticItemV2::Declaration(_)
                    ) || matches!(
                        item,
                        SemanticItemV2::Meta(meta)
                            if matches!(meta.kind(), ResolvedMetaKindV2::UsePack { .. })
                    )
                })
            })
            .collect()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "owner-local assembly tests inspect exact sealed fixture results"
)]
mod tests {
    use super::*;
    use quanta_gqlang_v2_sema::test_support::{
        reference_meta_document_v2, reference_relation_document_v2, ReferenceMetaFixtureV2,
        ReferenceRelationFixtureV2,
    };
    use quanta_gqlang_v2_sema::HirNodeIdV1;

    fn failure_code(error: &FrontendTransactionErrorV2) -> &str {
        match error {
            FrontendTransactionErrorV2::Document(failure) => failure
                .diagnostics()
                .first()
                .expect("one assembly diagnostic")
                .code(),
            FrontendTransactionErrorV2::Authority(_) => {
                panic!("assembly test unexpectedly returned an authority error")
            }
        }
    }

    fn standard_budget_v2() -> FrontendBudgetV2 {
        FrontendBudgetV2::standard_v2().expect("standard frontend budget")
    }

    #[test]
    fn parse_budget_projection_preserves_every_distinct_frontend_cap() {
        let budget = FrontendBudgetV2::try_new(
            151, 157, 31, 37, 11, 13, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97, 101, 103,
            107,
        )
        .expect("distinct positive frontend caps");
        let projected = project_parse_budget(SourceIdV2::from_raw(9), "", &budget)
            .expect("validated owner profile projects");
        assert_eq!(
            projected,
            ParseBudgetV2::try_new(31, 37, 41, 11, 13, 43, 47)
                .expect("independently specified parse limits")
        );
    }

    fn blocked_fixture_failure(code: &'static str) -> FrontendFailureV2 {
        let diagnostic = FrontendDiagnosticV2::new(
            FrontendStageV2::Validation,
            code,
            FrontendSeverityV2::Error,
            LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(73),
                span: SpanV1::new(17, 31),
            },
            vec![LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(74),
                span: SpanV1::new(44, 49),
            }],
            "owner stage failed",
        );
        FrontendFailureV2::try_new(vec![diagnostic.clone(), diagnostic])
            .expect("nonempty owner failure")
    }

    #[test]
    fn blocked_compile_retains_exact_owner_failure_for_rule_and_item_paths() {
        let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)
            .expect("reference relation");
        let failure = blocked_fixture_failure("planning.test.item_blocked");
        let rule_failure = blocked_fixture_failure("planning.test.rule_blocked");
        for rule_path in [false, true] {
            let expected = if rule_path { &rule_failure } else { &failure }
                .diagnostics()
                .first()
                .expect("nonempty owner failure")
                .clone();
            let run = DocumentRunV2 {
                items: vec![ItemRunV2::Blocked {
                    item_id: fixture.item_id(),
                    failure: failure.clone(),
                }],
                rule_program: None,
                rule_blocked: rule_path.then(|| rule_failure.clone()),
            };
            assert_eq!(
                run.into_compiled()
                    .expect_err("blocked stage must reject compilation"),
                FrontendTransactionErrorV2::Document(FrontendFailureV2::one(expected.clone())),
            );
        }
    }

    #[test]
    fn blocked_analysis_retains_item_failure_and_counts_canonical_diagnostics() {
        let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)
            .expect("reference relation");
        let failure = blocked_fixture_failure("planning.test.item_blocked");
        let make_run = || DocumentRunV2 {
            items: vec![ItemRunV2::Blocked {
                item_id: fixture.item_id(),
                failure: failure.clone(),
            }],
            rule_program: None,
            rule_blocked: None,
        };
        let analysis = make_run()
            .into_analysis(SourceIdV2::from_raw(73), 1)
            .expect("two identical diagnostics count once");
        assert_eq!(analysis.order.as_ref(), &[fixture.item_id()]);
        assert_eq!(
            analysis.items(),
            &[AnalysisItemV2::BlockedByDiagnostics(
                failure.diagnostics().into()
            )]
        );
        assert_eq!(
            analysis.diagnostics().as_ref(),
            &[failure
                .diagnostics()
                .first()
                .expect("nonempty owner failure")
                .clone()]
        );
        let error = make_run()
            .into_analysis(SourceIdV2::from_raw(73), 0)
            .expect_err("nonempty failure exceeds zero diagnostic budget");
        assert_eq!(
            failure_code(&error),
            "pipeline.analysis.diagnostic_limit_exceeded"
        );
    }

    #[test]
    fn blocked_rule_analysis_projects_owner_failure_to_every_member_after_counting() {
        use quanta_gqlang_v2_sema::test_support::{
            reference_rule_document_v2, ReferenceRuleFixtureV2,
        };
        let fixture = reference_rule_document_v2(ReferenceRuleFixtureV2::NonRecursiveDependency)
            .expect("reference rules");
        let membership = DocumentMembershipV2::of(fixture.document()).expect("membership");
        let program_id = membership.rule_program_id().expect("rule program");
        let members = membership.rule_member_ids();
        assert_eq!(members.len(), 2);
        let failure = blocked_fixture_failure("planning.test.item_blocked");
        let run = DocumentRunV2 {
            items: members
                .iter()
                .map(|id| {
                    ItemRunV2::Planned(Box::new(PlannedRunV2 {
                        planned: Box::new(PlannedItemV2::RuleMember(RuleProgramMemberV2::new(
                            program_id, *id,
                        ))),
                        relation: None,
                        composition: None,
                    }))
                })
                .collect(),
            rule_program: None,
            rule_blocked: Some(failure.clone()),
        };
        let analysis = run
            .into_analysis(SourceIdV2::from_raw(73), 1)
            .expect("one canonical failure is counted before member fanout");
        assert_eq!(analysis.order.as_ref(), members.as_slice());
        assert_eq!(
            analysis.items(),
            &[
                AnalysisItemV2::BlockedByDiagnostics(failure.diagnostics().into()),
                AnalysisItemV2::BlockedByDiagnostics(failure.diagnostics().into()),
            ]
        );
        assert_eq!(
            analysis.diagnostics().as_ref(),
            &[failure
                .diagnostics()
                .first()
                .expect("nonempty owner failure")
                .clone()]
        );
    }

    #[test]
    fn source_root_classifier_maps_supported_and_refused_families_without_authority() {
        for (source_v1, expected_v1) in [
            (
                "find path src.marker -> src.marker | depth=8",
                FrontendSourceRootFamilyV3::Graph,
            ),
            (
                "show capabilities",
                FrontendSourceRootFamilyV3::Presentation,
            ),
            (
                "let q = sample query=\"x\" | mode=lexical | limit=1 from users u with q",
                FrontendSourceRootFamilyV3::Composition,
            ),
        ] {
            assert_eq!(
                classify_single_source_root_v3(
                    SourceIdV2::from_raw(1),
                    source_v1,
                    &standard_budget_v2(),
                )
                .expect("valid root must classify"),
                expected_v1,
            );
        }
    }

    #[test]
    fn source_root_classifier_rejects_zero_multiple_and_malformed_documents() {
        assert!(matches!(
            classify_single_source_root_v3(SourceIdV2::from_raw(1), "", &standard_budget_v2(),),
            Err(FrontendSourceRootClassificationErrorV3::NoItem)
        ));
        assert!(matches!(
            classify_single_source_root_v3(
                SourceIdV2::from_raw(1),
                "show capabilities; show models",
                &standard_budget_v2(),
            ),
            Err(FrontendSourceRootClassificationErrorV3::MultipleItems { observed: 2 })
        ));
        assert!(matches!(
            classify_single_source_root_v3(
                SourceIdV2::from_raw(1),
                "find path src.marker src.marker",
                &standard_budget_v2(),
            ),
            Err(FrontendSourceRootClassificationErrorV3::Frontend(_))
        ));
    }

    #[test]
    fn duplicate_producer_identity_fails_closed_at_the_second_insert() {
        let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)
            .expect("reference relation");
        let id = fixture.item_id();
        let location = item_location_of(fixture.document(), id);
        let mut produced = BTreeSet::new();
        record_produced_identity(&mut produced, id, location, "duplicate")
            .expect("first identity is admitted");
        let error = record_produced_identity(&mut produced, id, location, "duplicate")
            .expect_err("second identity rejects");
        assert_eq!(
            failure_code(&error),
            "pipeline.document.duplicate_item_result"
        );
    }

    #[test]
    fn plan_membership_missing_extra_and_rule_mismatch_are_exact() {
        let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)
            .expect("reference relation");
        let membership = DocumentMembershipV2::of(fixture.document()).expect("membership");
        let id = fixture.item_id();
        let expected_plan = BTreeSet::from([id]);
        validate_produced_membership(
            fixture.document().source_id(),
            &membership,
            &expected_plan,
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .expect("exact plan membership");

        let missing = validate_produced_membership(
            fixture.document().source_id(),
            &membership,
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .expect_err("missing plan rejects");
        assert_eq!(
            failure_code(&missing),
            "pipeline.document.missing_item_result"
        );

        let extra_id = SemanticItemIdV2::new(
            fixture.document().source_id(),
            HirNodeIdV1::from_raw(id.hir_item_id().raw() + 100),
        );
        let extra = validate_produced_membership(
            fixture.document().source_id(),
            &membership,
            &BTreeSet::from([id, extra_id]),
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .expect_err("extra plan rejects");
        assert_eq!(failure_code(&extra), "pipeline.document.extra_item_result");

        let rule_mismatch = validate_produced_membership(
            fixture.document().source_id(),
            &membership,
            &expected_plan,
            &BTreeSet::from([extra_id]),
            &BTreeSet::new(),
        )
        .expect_err("unexpected rule member rejects first");
        assert_eq!(
            failure_code(&rule_mismatch),
            "pipeline.document.rule_membership_mismatch"
        );
    }

    #[test]
    fn checked_non_plan_membership_missing_and_extra_are_exact() {
        let fixture = reference_meta_document_v2(ReferenceMetaFixtureV2::UsePack)
            .expect("reference USE PACK");
        let membership = DocumentMembershipV2::of(fixture.document()).expect("membership");
        let id = fixture.meta_item_id();
        validate_produced_membership(
            fixture.document().source_id(),
            &membership,
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::from([id]),
        )
        .expect("exact non-plan membership");

        let missing = validate_produced_membership(
            fixture.document().source_id(),
            &membership,
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::new(),
        )
        .expect_err("missing non-plan rejects");
        assert_eq!(
            failure_code(&missing),
            "pipeline.document.missing_item_result"
        );

        let extra_id = SemanticItemIdV2::new(
            fixture.document().source_id(),
            HirNodeIdV1::from_raw(id.hir_item_id().raw() + 100),
        );
        let extra = validate_produced_membership(
            fixture.document().source_id(),
            &membership,
            &BTreeSet::new(),
            &BTreeSet::new(),
            &BTreeSet::from([id, extra_id]),
        )
        .expect_err("extra non-plan rejects");
        assert_eq!(failure_code(&extra), "pipeline.document.extra_item_result");
    }

    #[test]
    fn execution_closure_failures_project_to_independent_pipeline_codes() {
        let fixture = reference_relation_document_v2(ReferenceRelationFixtureV2::Scan)
            .expect("reference relation");
        for (failure, expected) in [
            (
                ExecutionClosureFailureV3::Dangling,
                "pipeline.document.dangling_execution_reference",
            ),
            (
                ExecutionClosureFailureV3::Foreign,
                "pipeline.document.foreign_item_result",
            ),
            (
                ExecutionClosureFailureV3::UnregisteredSymbol,
                "pipeline.document.unregistered_execution_symbol",
            ),
        ] {
            let error = closure_error(fixture.document(), fixture.item_id(), failure);
            assert_eq!(failure_code(&error), expected);
        }
    }
}
