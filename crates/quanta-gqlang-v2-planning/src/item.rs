//! The sealed semantic planning input and the checked NON-PLAN vocabulary
//! (ticket 07, RR-07B item 2).
//!
//! The V1 caller-selected input model (`PlanRequestV1`, `PublicPlanV1`,
//! `produce_plan`, `item::classify_*`) is DELETED. Every classification now keys
//! off one of the sealed carriers below, so a caller can no longer select the
//! result kind, effect, or truth lane: those are derived from verified upstream
//! contracts only.
//!
//! [`NoPlanReasonV2`] is the checked spelling for a document item that
//! legitimately has no executable plan. It is a TYPED, non-error outcome: the
//! pipeline stores it in the compiled document's nonplan item variants, so
//! absence is never represented by a skipped stage.

use quanta_gqlang_v2_algebra::LoweredRelationItemV2;
use quanta_gqlang_v2_logic::{GraphQueryContractV2, RuleProgramContractV2};
use quanta_gqlang_v2_sema::{ResolvedMetaV2, ResolvedRenameCommandV3, SemanticItemIdV2};

use crate::check::CheckContractV2;
use crate::composition::CheckedCompositionPlanInputV2;
use crate::presentation::PresentationContractV2;

/// One sealed semantic planning input.
///
/// There is no raw-HIR or caller-assembled variant: the relational arm is
/// ticket 05's sealed lowering product, the graph and rule arms are ticket 06's
/// sealed logic contracts, the CHECK/presentation/composition arms are this
/// crate's validated wrappers over Sema contracts, and the meta arm is Sema's
/// resolved directive.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SemanticPlanInputV2 {
    /// Closed, exact-target rename intent; effect remains subject to host grant.
    Command(ResolvedRenameCommandV3),
    /// A fully lowered, checked relational item (ticket 05).
    Relational(Box<LoweredRelationItemV2>),
    /// A sealed FIND graph contract (ticket 06).
    Graph(GraphQueryContractV2),
    /// A validated CHECK contract wrapping Sema's resolved CHECK.
    Check(CheckContractV2),
    /// A validated presentation contract wrapping Sema's resolved presentation.
    Presentation(PresentationContractV2),
    /// The document-wide sealed rule program (ticket 06).
    Rule(RuleProgramContractV2),
    /// A `REQUIRE CERTIFIED` document directive. A `USE PACK` directive reaching
    /// this arm is a checked [`NoPlanReasonV2::MetaActivation`], never a plan.
    MetaRequireCertified(ResolvedMetaV2),
    /// A checked composition plan input (root, ordered producers, WITH identity,
    /// one sealed body input).
    Composition(Box<CheckedCompositionPlanInputV2>),
}

/// Why a document item produces NO host-agnostic plan.
///
/// This is a TYPED marker, not an error: the item is well-formed, it simply does
/// not lower to a servable plan. The pipeline stores it verbatim in the compiled
/// document's nonplan item variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NoPlanReasonV2 {
    /// A catalog declaration (VIEW / MODEL / PACK / PROFILE / GATE / RULE
    /// binding): it registers in the catalog and produces no per-declaration
    /// serving plan. An inline VIEW body is still lowered and checked for its
    /// typed relation proof, but its Catalog declaration keeps this reason.
    CatalogDeclaration {
        /// Which catalog payload the declaration carries.
        kind: CatalogDeclarationKindV2,
    },
    /// A typed value declaration: it binds a checked value, not a servable
    /// relation.
    ValueDeclaration,
    /// A `USE PACK` metadata activation: effect-style pack activation with no
    /// relational plan.
    MetaActivation,
    /// The document carries no Rule item, so there is no document rule program
    /// to plan.
    EmptyRuleProgram,
}

/// Closed catalog payload vocabulary, retained so a catalog declaration's
/// nonplan outcome names the exact payload.
///
/// A MODEL declaration is NOT inferred to be an advisory serving plan from its
/// name or provider bytes: the frontend has no advisory lane, and this variant
/// is what the planner reports instead.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CatalogDeclarationKindV2 {
    /// VIEW alias or typed inline body.
    View,
    /// MODEL with opaque provider metadata.
    Model,
    /// PACK source and activation.
    Pack,
    /// PROFILE capabilities and admission.
    Profile,
    /// GATE checks and binding policy.
    Gate,
    /// RULE binding to an exact CHECK reference.
    RuleBinding,
}

impl CatalogDeclarationKindV2 {
    /// Stable code suffix for this catalog payload.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Model => "model",
            Self::Pack => "pack",
            Self::Profile => "profile",
            Self::Gate => "gate",
            Self::RuleBinding => "rule_binding",
        }
    }

    /// Stable human name for this catalog payload.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::View => "VIEW",
            Self::Model => "MODEL",
            Self::Pack => "PACK",
            Self::Profile => "PROFILE",
            Self::Gate => "GATE",
            Self::RuleBinding => "RULE",
        }
    }
}

impl NoPlanReasonV2 {
    /// Full stable diagnostic code, including the exact catalog payload.
    #[must_use]
    pub const fn full_code(self) -> &'static str {
        match self {
            Self::CatalogDeclaration { kind } => match kind {
                CatalogDeclarationKindV2::View => "planning.no_plan.catalog_declaration.view",
                CatalogDeclarationKindV2::Model => "planning.no_plan.catalog_declaration.model",
                CatalogDeclarationKindV2::Pack => "planning.no_plan.catalog_declaration.pack",
                CatalogDeclarationKindV2::Profile => "planning.no_plan.catalog_declaration.profile",
                CatalogDeclarationKindV2::Gate => "planning.no_plan.catalog_declaration.gate",
                CatalogDeclarationKindV2::RuleBinding => {
                    "planning.no_plan.catalog_declaration.rule_binding"
                }
            },
            Self::ValueDeclaration => "planning.no_plan.value_declaration",
            Self::MetaActivation => "planning.no_plan.meta_activation",
            Self::EmptyRuleProgram => "planning.no_plan.empty_rule_program",
        }
    }

    /// The stable reason spelling (without the `planning.no_plan.` prefix).
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::CatalogDeclaration { kind: _ } => "catalog_declaration",
            Self::ValueDeclaration => "value_declaration",
            Self::MetaActivation => "meta_activation",
            Self::EmptyRuleProgram => "empty_rule_program",
        }
    }

    /// The catalog payload this reason names, when it is a catalog declaration.
    #[must_use]
    pub const fn catalog_kind(self) -> Option<CatalogDeclarationKindV2> {
        match self {
            Self::CatalogDeclaration { kind } => Some(kind),
            Self::ValueDeclaration | Self::MetaActivation | Self::EmptyRuleProgram => None,
        }
    }

    /// A stable human-readable message. Text is part of the contract: tests
    /// assert it verbatim.
    #[must_use]
    pub fn message(self) -> String {
        match self {
            Self::CatalogDeclaration { kind } => format!(
                "{} declaration registers in the catalog and has no per-declaration serving plan",
                kind.name()
            ),
            Self::ValueDeclaration => {
                "value declaration binds a checked value and has no serving plan".to_owned()
            }
            Self::MetaActivation => {
                "USE PACK activates a pack and has no relational serving plan".to_owned()
            }
            Self::EmptyRuleProgram => {
                "the document has no Rule item, so it has no rule program plan".to_owned()
            }
        }
    }
}

/// The canonical item identity an input arm plans for, when the arm retains one.
///
/// Used by the entry points to anchor an input/document mismatch diagnostic; the
/// per-arm membership validation itself lives in [`crate::plan`].
#[must_use]
pub(crate) const fn input_item_id(input: &SemanticPlanInputV2) -> Option<SemanticItemIdV2> {
    match input {
        SemanticPlanInputV2::Command(command) => Some(command.item_id()),
        SemanticPlanInputV2::Relational(item) => Some(item.item_id()),
        SemanticPlanInputV2::Graph(contract) => Some(contract.item_id()),
        SemanticPlanInputV2::Check(contract) => Some(contract.item_id()),
        SemanticPlanInputV2::Presentation(contract) => Some(contract.item_id()),
        SemanticPlanInputV2::Rule(_) => None,
        SemanticPlanInputV2::MetaRequireCertified(meta) => Some(meta.item_id()),
        SemanticPlanInputV2::Composition(composition) => Some(composition.root_item_id()),
    }
}
