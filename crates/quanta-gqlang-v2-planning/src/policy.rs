//! Retained family policy of one semantic plan (ticket 07, junction R1).
//!
//! The eight pinned plan fields describe the plan's derived classification. They
//! cannot retain the family-specific sealed payload that ticket 07's item
//! contract matrix requires to survive planning ("lost CHECK emit mode",
//! "lost FIND witness cap/tie-break", SAMPLE/COMPARE/EXPORT payload retention),
//! so [`PlanPolicyV2`] carries exactly that retention:
//!
//! - FIND keeps its exact direction, facets, resolved selectors, and graph
//!   witness depth/cap/dedup/tie-break provenance from
//!   [`quanta_gqlang_v2_logic::GraphQueryContractV2`] — never reclassified.
//! - CHECK keeps the exact detection/confirm/emit modes and its resolved rule,
//!   scope, pattern, and nested graph provenance.
//! - Presentation keeps the exact sealed presentation payload.
//! - A rule program keeps its ordered member identities and stratified proof
//!   summary.
//! - A composition keeps its root, ordered producers, `WITH` binding identity,
//!   body identity, and the body's own policy.
//!
//! Every value here is copied from a sealed carrier; none of it is re-derived
//! from syntax, names, or display text.

use quanta_gqlang_v2_logic::{
    DedupIdentityV1, GraphQueryContractV2, RelationShapeV1, RuleProgramContractV2,
    WitnessTieBreakV1,
};
use quanta_gqlang_v2_sema::{
    CheckedLocatedSpanV2, GraphDirectionV2, GraphQueryKindV1, ResolvedGraphReferenceV2,
    ResolvedPresentationKindV2, SemanticItemIdV2,
};
use quanta_gqlang_v2_types::{
    AggregationClassV1, DeterminismV1, MonotonicityV1, PurityV1, ResolvedPredicateIdV2,
};

use crate::check::CheckPolicyV2;
use crate::composition::CompositionPolicyV2;

/// The folded expression-effect projection a plan retains.
///
/// It is the join of every sealed `ExpressionEffectsV2` the budgeted traversal
/// visited, without the physical-cost class: purity, determinism, monotonicity,
/// and aggregation placement are semantic facts, and the determinism lane of
/// [`crate::SemanticPlanV2`] is derived from the `determinism` field here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EffectSummaryV2 {
    /// Folded purity of every retained expression effect.
    pub purity: PurityV1,
    /// Folded determinism of every retained expression effect.
    pub determinism: DeterminismV1,
    /// Folded monotonicity of every retained expression effect.
    pub monotonicity: MonotonicityV1,
    /// Folded aggregation placement of every retained expression effect.
    pub aggregation: AggregationClassV1,
}

impl EffectSummaryV2 {
    /// A projection of a sealed effect set.
    #[must_use]
    pub const fn new(
        purity: PurityV1,
        determinism: DeterminismV1,
        monotonicity: MonotonicityV1,
        aggregation: AggregationClassV1,
    ) -> Self {
        Self {
            purity,
            determinism,
            monotonicity,
            aggregation,
        }
    }
}

/// Exact query and witness provenance retained from a sealed graph contract.
///
/// Every field is read from [`GraphQueryContractV2`]; planning never
/// reclassifies a FIND from raw source, display names, or risk class.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphWitnessRetentionV2 {
    item_id: SemanticItemIdV2,
    kind: GraphQueryKindV1,
    location: CheckedLocatedSpanV2,
    depth: u32,
    direction: Option<GraphDirectionV2>,
    summary_facets: Vec<String>,
    references: Vec<ResolvedGraphReferenceV2>,
    shape: RelationShapeV1,
    max_witnesses: u32,
    max_path_length: Option<u32>,
    dedup: DedupIdentityV1,
    tie_break: WitnessTieBreakV1,
}

impl GraphWitnessRetentionV2 {
    /// Project the exact provenance of one sealed graph contract.
    #[must_use]
    pub fn from_contract(contract: &GraphQueryContractV2) -> Self {
        Self {
            item_id: contract.item_id(),
            kind: contract.kind(),
            location: contract.location().clone(),
            depth: contract.depth(),
            direction: contract.direction(),
            summary_facets: contract.summary_facets().to_vec(),
            references: contract.references().to_vec(),
            shape: contract.shape(),
            max_witnesses: contract.max_witnesses(),
            max_path_length: contract.witness().max_path_length(),
            dedup: contract.dedup(),
            tie_break: contract.tie_break(),
        }
    }

    /// Semantic item identity of the FIND this provenance came from.
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }

    /// The FIND subkind.
    #[must_use]
    pub const fn kind(&self) -> GraphQueryKindV1 {
        self.kind
    }

    /// Checked location of the originating FIND item.
    #[must_use]
    pub const fn location(&self) -> &CheckedLocatedSpanV2 {
        &self.location
    }

    /// The explicit positive traversal depth, exactly as resolved.
    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }

    /// Authored traversal direction, when present.
    #[must_use]
    pub const fn direction(&self) -> Option<GraphDirectionV2> {
        self.direction
    }

    /// Ordered summary facets retained from the sealed graph contract.
    #[must_use]
    pub fn summary_facets(&self) -> &[String] {
        &self.summary_facets
    }

    /// Every resolved endpoint and selector in Sema-owned deterministic order.
    #[must_use]
    pub fn references(&self) -> &[ResolvedGraphReferenceV2] {
        &self.references
    }

    /// The relation shape the witness contract was bound against.
    #[must_use]
    pub const fn shape(&self) -> RelationShapeV1 {
        self.shape
    }

    /// The finite, non-zero max-witness cap.
    #[must_use]
    pub const fn max_witnesses(&self) -> u32 {
        self.max_witnesses
    }

    /// The path-length bound, exactly as sealed.
    #[must_use]
    pub const fn max_path_length(&self) -> Option<u32> {
        self.max_path_length
    }

    /// The dedup identity of enumerated witnesses.
    #[must_use]
    pub const fn dedup(&self) -> DedupIdentityV1 {
        self.dedup
    }

    /// The canonical tie-break of equivalent witnesses.
    #[must_use]
    pub const fn tie_break(&self) -> WitnessTieBreakV1 {
        self.tie_break
    }
}

/// The retained summary of one document rule program.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleProgramPolicyV2 {
    item_ids: Vec<SemanticItemIdV2>,
    stratum_count: usize,
    recursive_members: Vec<ResolvedPredicateIdV2>,
}

impl RuleProgramPolicyV2 {
    /// Retain the program's ordered member identities and stratified proof
    /// summary.
    #[must_use]
    pub fn new(
        contract: &RuleProgramContractV2,
        stratum_count: usize,
        recursive_members: Vec<ResolvedPredicateIdV2>,
    ) -> Self {
        Self {
            item_ids: contract
                .rules()
                .iter()
                .map(quanta_gqlang_v2_sema::ResolvedRuleV2::item_id)
                .collect(),
            stratum_count,
            recursive_members,
        }
    }

    /// The program's member item identities, in document order.
    #[must_use]
    pub fn item_ids(&self) -> &[SemanticItemIdV2] {
        &self.item_ids
    }

    /// The number of strata the admitted proof assigned.
    #[must_use]
    pub const fn stratum_count(&self) -> usize {
        self.stratum_count
    }

    /// The recursive member predicates, in ascending sealed identity order.
    #[must_use]
    pub fn recursive_members(&self) -> &[ResolvedPredicateIdV2] {
        &self.recursive_members
    }
}

/// Retained family policy of one semantic plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlanPolicyV2 {
    /// Exact sealed rename intent. Runtime snapshot/grant/preview checks remain required.
    Command(quanta_gqlang_v2_sema::ResolvedRenameCommandV3),
    /// An ordinary relational plan: the checked algebra root is the whole
    /// policy, and the retained effect summary is its folded expression effects.
    Relational {
        /// How many checked algebra nodes the budgeted traversal visited.
        nodes: u32,
        /// The folded expression-effect projection.
        effects: EffectSummaryV2,
    },
    /// A FIND graph plan: exact witness provenance, never reclassified.
    Graph(GraphWitnessRetentionV2),
    /// A CHECK plan: exact detection/confirm/emit and nested graph provenance.
    Check(CheckPolicyV2),
    /// A presentation plan: the exact sealed presentation payload.
    Presentation(ResolvedPresentationKindV2),
    /// The document rule program plan.
    Rule(RuleProgramPolicyV2),
    /// A `REQUIRE CERTIFIED` directive matched to the modules admission proof.
    MetaRequireCertified,
    /// A composition plan: the retained composition identity plus the body's own
    /// policy. It is NOT a second classifier and creates no per-producer plan.
    Composition(CompositionPolicyV2),
}

impl PlanPolicyV2 {
    /// Stable family keyword for explain/diagnostics and CLI/LSP rendering.
    #[must_use]
    pub const fn keyword(&self) -> &'static str {
        match self {
            Self::Command(_) => "command",
            Self::Relational { .. } => "relational",
            Self::Graph(_) => "graph",
            Self::Check(_) => "check",
            Self::Presentation(_) => "presentation",
            Self::Rule(_) => "rule_program",
            Self::MetaRequireCertified => "meta_require_certified",
            Self::Composition(_) => "composition",
        }
    }
}
