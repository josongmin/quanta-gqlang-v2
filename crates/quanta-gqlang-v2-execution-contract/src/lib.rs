#![forbid(unsafe_code)]

//! Host-agnostic, immutable execution contract projected from sealed V2 owners.
//!
//! This module copies only public semantic facts into a closed transport DTO.
//! Closed vocabularies are execution-local enums. The only reused owner types
//! are the inert identity closure (`SourceIdV2`, `SemanticItemIdV2`,
//! `SourceQualifiedNodeIdV2`) and [`AlgebraFamilyV2`], all already part of the
//! curated facade closure. Every other HIR, Sema, Algebra, Planning, and Types
//! owner type stays unreachable.

#[cfg(test)]
use encoding::{
    algebra_family_tag, authority_tag, cardinality_tag, determinism_tag, effect_tag, evidence_tag,
    exists_polarity_tag, family_tag, finish_sha256, graph_dedup_identity_tag, graph_direction_tag,
    graph_kind_tag, graph_reference_kind_tag, graph_reference_role_tag, graph_relation_shape_tag,
    graph_witness_tie_break_tag, join_mode_tag, logical_quantifier_tag, null_order_tag,
    policy_family_tag, put_graph_query, put_literal, put_node, scalar_tag, sha256,
    sort_direction_tag, source_kind_tag, write_contract, CanonicalByteSink,
};
use encoding::{
    digest_contract, digest_qualification, digest_source_bytes, CONTRACT_DIGEST_SCHEMA_VERSION,
    SOURCE_DIGEST_SCHEMA_VERSION,
};
use quanta_gqlang_v2_algebra::{AlgebraFamilyV2, CheckedAlgebraNodeV2, LoweredRelationItemV2};
use quanta_gqlang_v2_logic::{
    DedupIdentityV1, GraphQueryContractV2, RelationShapeV1, WitnessTieBreakV1,
};
use quanta_gqlang_v2_planning::{
    AuthorityLaneV2, CheckContractV2, EffectClassV2, EvaluationDeterminismV2,
    EvidenceRequirementV2, PlanPolicyV2, PresentationContractV2, SemanticPlanV2,
};
use quanta_gqlang_v2_sema::{
    CardinalityV1, CompareMetricV1, CompositionGraphCapabilityV2, ConfirmModeV1, DetectionModeV1,
    EmitModeV1, EvidenceModeV1, ExportFormatV1, ExpressionTypeStateV2, GraphDirectionV2,
    GraphQueryKindV1, GraphReferenceKindV2, GraphReferenceRoleV2, JoinModeV2,
    LogicalQuantifierKindV2, NullOrderV2, RelationAuthorityV2, RelationExistsPolarityV2,
    RelationOutputContractV2, RenamePhaseV3, ResolvedCallTargetV2, ResolvedCompositionBodyV2,
    ResolvedCompositionProducerV2, ResolvedCompositionV2, ResolvedExplainTargetV2,
    ResolvedGraphTargetV2, ResolvedGraphUseV2, ResolvedPresentationKindV2,
    ResolvedRelationSourceKindV2, ResolvedRenameCommandV3, SampleModeV1, ScalarTyV1,
    SemanticDocumentV2, SemanticItemIdV2, ShowCategoryV1, SortDirectionV2, SourceIdV2,
    SourceQualifiedNodeIdV2, TruthModeV1, TypedCallArgumentV2, TypedExprShapeV2, TypedExprV2,
    TypedLiteralFactV2, TypedLiteralValueV2, ValueTypeV1,
};
#[cfg(test)]
use sha2::{Digest as _, Sha256};
use std::sync::Arc;

pub use quanta_gqlang_v2_sema::OperatorSemanticKindV2 as ExecutionOperatorKindV3;
pub use vocabulary::{
    execution_operator_descriptor_v3, ExecutionOperatorDescriptorErrorV3,
    ExecutionOperatorDescriptorV3,
};

/// SHA-256 digest of the canonical execution-contract encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExecutionDigestV3([u8; 32]);

impl ExecutionDigestV3 {
    /// Raw digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Lower-case hexadecimal form.
    #[must_use]
    pub fn to_hex(self) -> String {
        hex_lower(&self.0)
    }
}

/// Lower-case hex encoding shared by digest display and predicate identity:
/// total, allocation-once, and free of per-byte `format!`.
fn hex_lower(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes.iter() {
        for shift in [4, 0] {
            let nibble = (byte >> shift) & 0x0f;
            hex.push(char::from_digit(u32::from(nibble), 16).unwrap_or('?'));
        }
    }
    hex
}

/// One immutable, source-ordered execution document.
///
/// Three digests separate three concerns: [`ExecutionDocumentV3::source_digest`]
/// covers the raw UTF-8 source bytes only, [`ExecutionDocumentV3::contract_digest`]
/// covers the canonical DTO encoding only, and
/// [`ExecutionDocumentV3::qualification_digest`] binds the two together under the
/// contract domain and schema version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionDocumentV3 {
    source_id: SourceIdV2,
    items: Box<[ExecutionItemV3]>,
    source_digest: ExecutionDigestV3,
    contract_digest: ExecutionDigestV3,
    qualification_digest: ExecutionDigestV3,
}

impl ExecutionDocumentV3 {
    pub(crate) fn new(source_id: SourceIdV2, source: &[u8], items: Vec<ExecutionItemV3>) -> Self {
        let contract_digest = digest_contract(source_id, &items);
        let source_digest = digest_source_bytes(source);
        let qualification_digest = digest_qualification(source_id, source_digest, contract_digest);
        Self {
            source_id,
            items: items.into_boxed_slice(),
            source_digest,
            contract_digest,
            qualification_digest,
        }
    }

    /// Source document identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.source_id
    }

    /// Source-ordered plan-required items. Non-plan declarations are absent;
    /// downstream admission must reject any family without an executable payload.
    #[must_use]
    pub fn items(&self) -> &[ExecutionItemV3] {
        &self.items
    }

    /// Document-local exact-key lookup; the ID alone is not provenance proof.
    #[must_use]
    pub fn item(&self, item_id: SemanticItemIdV2) -> Option<&ExecutionItemV3> {
        self.items.iter().find(|item| item.item_id == item_id)
    }

    /// Digest of the raw UTF-8 source bytes only. DTO changes never move it.
    #[must_use]
    pub const fn source_digest(&self) -> ExecutionDigestV3 {
        self.source_digest
    }

    /// Compare exact source bytes using the canonical source-digest encoder.
    /// This creates no frontend document and grants no runtime authority.
    #[must_use]
    pub fn matches_source_bytes_v3(&self, source: &[u8]) -> bool {
        self.source_digest == digest_source_bytes(source)
    }

    /// Canonical schema used by [`Self::source_digest`].
    #[must_use]
    pub const fn source_digest_schema_version(&self) -> u32 {
        SOURCE_DIGEST_SCHEMA_VERSION
    }

    /// Digest of the canonical DTO encoding only. Source-byte changes that do
    /// not alter the projected contract never move it.
    #[must_use]
    pub const fn contract_digest(&self) -> ExecutionDigestV3 {
        self.contract_digest
    }

    /// Canonical schema used by [`Self::contract_digest`].
    #[must_use]
    pub const fn contract_digest_schema_version(&self) -> u32 {
        CONTRACT_DIGEST_SCHEMA_VERSION
    }

    /// Digest binding the contract domain, the schema version, and both legs
    /// above. Its inputs are the projected execution contract and original
    /// source bytes; compiled dispositions and rule-program data outside that
    /// projection are not separately encoded. Source-byte changes remain bound
    /// even when the execution projection is unchanged.
    #[must_use]
    pub const fn qualification_digest(&self) -> ExecutionDigestV3 {
        self.qualification_digest
    }

    /// Canonical schema used by [`Self::qualification_digest`].
    #[must_use]
    pub const fn qualification_digest_schema_version(&self) -> u32 {
        CONTRACT_DIGEST_SCHEMA_VERSION
    }
}

/// Closed top-level execution family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionFamilyV3 {
    Command,
    Relation,
    Graph,
    Check,
    Presentation,
    Composition,
    MetaRequireCertified,
    /// Reserved wire vocabulary. Current compilation emits neither rule
    /// members nor the document rule program as V3 execution items.
    RuleProgram,
}

/// Exact host-issued code symbol carried by a compiled rename intent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionSymbolTargetV3 {
    selector: Arc<str>,
    repo_path: Arc<str>,
    symbol_id: Arc<str>,
    start_byte: u64,
    end_byte: u64,
}
impl ExecutionSymbolTargetV3 {
    #[must_use]
    pub fn selector(&self) -> &str {
        &self.selector
    }
    #[must_use]
    pub fn repo_path(&self) -> &str {
        &self.repo_path
    }
    #[must_use]
    pub fn symbol_id(&self) -> &str {
        &self.symbol_id
    }
    #[must_use]
    pub const fn start_byte(&self) -> u64 {
        self.start_byte
    }
    #[must_use]
    pub const fn end_byte(&self) -> u64 {
        self.end_byte
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionCommandPhaseV3 {
    Preview,
    Apply,
}

/// Source-bound rename intent. `expected_snapshot` and `preview_token` are
/// untrusted preconditions; this DTO never conveys a mutation grant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRenameCommandV3 {
    target: ExecutionSymbolTargetV3,
    new_name: String,
    phase: ExecutionCommandPhaseV3,
    expected_snapshot: String,
    preview_token: Option<String>,
}
impl ExecutionRenameCommandV3 {
    #[must_use]
    pub const fn target(&self) -> &ExecutionSymbolTargetV3 {
        &self.target
    }
    #[must_use]
    pub fn new_name(&self) -> &str {
        &self.new_name
    }
    #[must_use]
    pub const fn phase(&self) -> ExecutionCommandPhaseV3 {
        self.phase
    }
    #[must_use]
    pub fn expected_snapshot(&self) -> &str {
        &self.expected_snapshot
    }
    #[must_use]
    pub fn preview_token(&self) -> Option<&str> {
        self.preview_token.as_deref()
    }
}

fn project_command(command: &ResolvedRenameCommandV3) -> ExecutionRenameCommandV3 {
    let target = command.target();
    ExecutionRenameCommandV3 {
        target: ExecutionSymbolTargetV3 {
            selector: target.clone_selector_storage_v3(),
            repo_path: target.clone_repo_path_storage_v3(),
            symbol_id: target.clone_symbol_id_storage_v3(),
            start_byte: target.start_byte(),
            end_byte: target.end_byte(),
        },
        new_name: command.new_name().to_owned(),
        phase: match command.phase() {
            RenamePhaseV3::Preview => ExecutionCommandPhaseV3::Preview,
            RenamePhaseV3::Apply => ExecutionCommandPhaseV3::Apply,
        },
        expected_snapshot: command.expected_snapshot().to_owned(),
        preview_token: command.preview_token().map(str::to_owned),
    }
}

/// A checked catalog identity. The variant supplies the namespace; a host
/// must still resolve this identity against its admitted catalog generation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionCatalogIdentityV3 {
    module: String,
    name: String,
}
impl ExecutionCatalogIdentityV3 {
    #[must_use]
    pub fn module(&self) -> &str {
        &self.module
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionConfirmModeV3 {
    Skip,
    Prompt,
    Require,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionEmitModeV3 {
    Findings,
    Sarif,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionDetectionModeV3 {
    PatternOnly,
    SemanticOnly,
    Hybrid,
}

/// A nested CHECK/FIND retains its distinct source item identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionNestedGraphV3 {
    item_id: SemanticItemIdV2,
    query: ExecutionGraphQueryV3,
}
impl ExecutionNestedGraphV3 {
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    #[must_use]
    pub const fn query(&self) -> &ExecutionGraphQueryV3 {
        &self.query
    }
}

/// The coherent CHECK input projected from Planning's sealed contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionCheckV3 {
    rule: ExecutionCatalogIdentityV3,
    scope: Option<ExecutionGraphTargetV3>,
    pattern: Option<ExecutionGraphTargetV3>,
    nested_graph: Option<ExecutionNestedGraphV3>,
    confirm: ExecutionConfirmModeV3,
    emit: ExecutionEmitModeV3,
    mode: ExecutionDetectionModeV3,
}
impl ExecutionCheckV3 {
    #[must_use]
    pub const fn rule(&self) -> &ExecutionCatalogIdentityV3 {
        &self.rule
    }
    #[must_use]
    pub const fn scope(&self) -> Option<&ExecutionGraphTargetV3> {
        self.scope.as_ref()
    }
    #[must_use]
    pub const fn pattern(&self) -> Option<&ExecutionGraphTargetV3> {
        self.pattern.as_ref()
    }
    #[must_use]
    pub const fn nested_graph(&self) -> Option<&ExecutionNestedGraphV3> {
        self.nested_graph.as_ref()
    }
    #[must_use]
    pub const fn confirm(&self) -> ExecutionConfirmModeV3 {
        self.confirm
    }
    #[must_use]
    pub const fn emit(&self) -> ExecutionEmitModeV3 {
        self.emit
    }
    #[must_use]
    pub const fn mode(&self) -> ExecutionDetectionModeV3 {
        self.mode
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionEvidenceModeV3 {
    Auto,
    Trace,
    Citation,
    Counterexample,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionTruthModeV3 {
    Inspect,
    Confirm,
    Refute,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionShowCategoryV3 {
    Capabilities,
    Models,
    Views,
    Rules,
    Packs,
    Profiles,
    Gates,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionCompareMetricV3 {
    Exact,
    Structural,
    Semantic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionExportFormatV3 {
    Json,
    Csv,
    Sarif,
    Markdown,
    GraphMl,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionExplainTargetV3 {
    Rule(ExecutionCatalogIdentityV3),
    Entity(ExecutionGraphTargetV3),
}

/// Disjoint verb payload projected only after Presentation policy validation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionPresentationKindV3 {
    Explain {
        target: ExecutionExplainTargetV3,
        evidence: ExecutionEvidenceModeV3,
        truth: ExecutionTruthModeV3,
    },
    Show {
        category: ExecutionShowCategoryV3,
    },
    Describe {
        target: ExecutionGraphTargetV3,
    },
    Sample {
        query_text: String,
        mode: ExecutionSampleModeV3,
        limit: u32,
        scope: Option<ExecutionGraphTargetV3>,
    },
    Compare {
        baseline: ExecutionGraphTargetV3,
        candidate: ExecutionGraphTargetV3,
        metric: ExecutionCompareMetricV3,
    },
    Export {
        source: ExecutionGraphTargetV3,
        format: ExecutionExportFormatV3,
        profile: Option<ExecutionCatalogIdentityV3>,
    },
}

impl ExecutionFamilyV3 {
    /// Stable wire keyword.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Relation => "relation",
            Self::Graph => "graph",
            Self::Check => "check",
            Self::Presentation => "presentation",
            Self::Composition => "composition",
            Self::MetaRequireCertified => "meta_require_certified",
            Self::RuleProgram => "rule_program",
        }
    }
}

/// One plan-required item with its immutable planning authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionItemV3 {
    item_id: SemanticItemIdV2,
    family: ExecutionFamilyV3,
    relation: Option<ExecutionRelationV3>,
    graph: Option<ExecutionGraphQueryV3>,
    check: Option<ExecutionCheckV3>,
    presentation: Option<ExecutionPresentationKindV3>,
    composition: Option<ExecutionCompositionV3>,
    command: Option<ExecutionRenameCommandV3>,
    plan: ExecutionPlanV3,
}

impl ExecutionItemV3 {
    pub(crate) fn new(
        item_id: SemanticItemIdV2,
        family: ExecutionFamilyV3,
        relation: Option<ExecutionRelationV3>,
        graph: Option<ExecutionGraphQueryV3>,
        check: Option<ExecutionCheckV3>,
        presentation: Option<ExecutionPresentationKindV3>,
        composition: Option<ExecutionCompositionV3>,
        plan: &SemanticPlanV2,
    ) -> Self {
        Self {
            item_id,
            family,
            relation,
            graph,
            check,
            presentation,
            composition,
            command: match plan.policy() {
                PlanPolicyV2::Command(command) => Some(project_command(command)),
                PlanPolicyV2::Relational { .. }
                | PlanPolicyV2::Graph(_)
                | PlanPolicyV2::Check(_)
                | PlanPolicyV2::Presentation(_)
                | PlanPolicyV2::Rule(_)
                | PlanPolicyV2::MetaRequireCertified
                | PlanPolicyV2::Composition(_) => None,
            },
            plan: project_plan(plan),
        }
    }

    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    #[must_use]
    pub const fn family(&self) -> ExecutionFamilyV3 {
        self.family
    }
    #[must_use]
    pub const fn relation(&self) -> Option<&ExecutionRelationV3> {
        self.relation.as_ref()
    }
    #[must_use]
    pub const fn graph(&self) -> Option<&ExecutionGraphQueryV3> {
        self.graph.as_ref()
    }
    #[must_use]
    pub const fn check(&self) -> Option<&ExecutionCheckV3> {
        self.check.as_ref()
    }
    #[must_use]
    pub const fn presentation(&self) -> Option<&ExecutionPresentationKindV3> {
        self.presentation.as_ref()
    }
    #[must_use]
    pub const fn composition(&self) -> Option<&ExecutionCompositionV3> {
        self.composition.as_ref()
    }
    #[must_use]
    pub const fn command(&self) -> Option<&ExecutionRenameCommandV3> {
        self.command.as_ref()
    }
    /// Producer-derived plan summary and authority; callers cannot replace it.
    /// Detailed semantic policy and the rule program are outside this DTO.
    #[must_use]
    pub const fn plan(&self) -> &ExecutionPlanV3 {
        &self.plan
    }
}

/// Summary tags projected from the sealed semantic plan, not its full policy.
/// Rule-program plans and detailed capability, cost, explain and policy payloads
/// are outside this DTO and its contract digest. This is frontend vocabulary;
/// it carries no runtime, provider or backend routing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionPlanV3 {
    authority: ExecutionAuthorityV3,
    determinism: ExecutionDeterminismV3,
    evidence: ExecutionEvidenceV3,
    effect: ExecutionEffectV3,
    policy_family: ExecutionPolicyFamilyV3,
}
impl ExecutionPlanV3 {
    #[must_use]
    pub const fn authority(&self) -> ExecutionAuthorityV3 {
        self.authority
    }
    #[must_use]
    pub const fn determinism(&self) -> ExecutionDeterminismV3 {
        self.determinism
    }
    #[must_use]
    pub const fn evidence(&self) -> ExecutionEvidenceV3 {
        self.evidence
    }
    #[must_use]
    pub const fn effect(&self) -> ExecutionEffectV3 {
        self.effect
    }
    #[must_use]
    pub const fn policy_family(&self) -> ExecutionPolicyFamilyV3 {
        self.policy_family
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionAuthorityV3 {
    FrontendChecked,
    CertifiedRequirementValidated,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionDeterminismV3 {
    Deterministic,
    NonDeterministic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionEvidenceV3 {
    None,
    SourceLocation,
    WitnessPayload,
    ProofTree,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionEffectV3 {
    ReadOnly,
    ProofEvidenceOnly,
    ExportOnly,
    MutationForbidden,
    MutationRequiresGrant,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionPolicyFamilyV3 {
    Relational,
    Graph,
    Check,
    Presentation,
    RuleProgram,
    MetaRequireCertified,
    Composition,
    Command,
}
impl ExecutionPolicyFamilyV3 {
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
            Self::Relational => "relational",
            Self::Graph => "graph",
            Self::Check => "check",
            Self::Presentation => "presentation",
            Self::RuleProgram => "rule_program",
            Self::MetaRequireCertified => "meta_require_certified",
            Self::Composition => "composition",
            Self::Command => "command",
        }
    }
}

/// A sealed relational operator tree and its typed expression table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRelationV3 {
    item_id: SemanticItemIdV2,
    root: ExecutionRelationalOperatorV3,
    expressions: Box<[ExecutionExpressionV3]>,
}

impl ExecutionRelationV3 {
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    #[must_use]
    pub const fn root(&self) -> &ExecutionRelationalOperatorV3 {
        &self.root
    }
    #[must_use]
    pub fn expressions(&self) -> &[ExecutionExpressionV3] {
        &self.expressions
    }
}

/// One closed relational operator node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRelationalOperatorV3 {
    family: AlgebraFamilyV2,
    join_mode: Option<ExecutionJoinModeV3>,
    stage: SourceQualifiedNodeIdV2,
    source: Option<ExecutionSourceV3>,
    schema: ExecutionSchemaV3,
    predicates: Box<[SourceQualifiedNodeIdV2]>,
    expressions: Box<[SourceQualifiedNodeIdV2]>,
    correlations: Box<[ExecutionCorrelationV3]>,
    existential_terms: Box<[ExecutionExistentialTermV3]>,
    partition_input: Box<[ExecutionBindingV3]>,
    children: Box<[ExecutionRelationalOperatorV3]>,
}

impl ExecutionRelationalOperatorV3 {
    #[must_use]
    pub const fn family(&self) -> AlgebraFamilyV2 {
        self.family
    }
    /// Join mode for `Join` operators; `None` for every other family.
    #[must_use]
    pub const fn join_mode(&self) -> Option<ExecutionJoinModeV3> {
        self.join_mode
    }
    #[must_use]
    pub const fn stage(&self) -> SourceQualifiedNodeIdV2 {
        self.stage
    }
    #[must_use]
    pub const fn source(&self) -> Option<&ExecutionSourceV3> {
        self.source.as_ref()
    }
    #[must_use]
    pub const fn schema(&self) -> &ExecutionSchemaV3 {
        &self.schema
    }
    #[must_use]
    pub fn predicates(&self) -> &[SourceQualifiedNodeIdV2] {
        &self.predicates
    }
    #[must_use]
    pub fn expressions(&self) -> &[SourceQualifiedNodeIdV2] {
        &self.expressions
    }
    #[must_use]
    pub fn correlations(&self) -> &[ExecutionCorrelationV3] {
        &self.correlations
    }
    /// Flat EXISTS conditions; child zero is the input and each following
    /// child corresponds to one condition in authored order.
    pub fn existential_terms(&self) -> &[ExecutionExistentialTermV3] {
        &self.existential_terms
    }
    #[must_use]
    pub fn partition_input(&self) -> &[ExecutionBindingV3] {
        &self.partition_input
    }
    #[must_use]
    pub fn children(&self) -> &[ExecutionRelationalOperatorV3] {
        &self.children
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionExistentialTermV3 {
    expression: SourceQualifiedNodeIdV2,
    polarity: ExecutionExistsPolarityV3,
    correlations: Box<[ExecutionCorrelationV3]>,
}

impl ExecutionExistentialTermV3 {
    #[must_use]
    pub const fn expression(&self) -> SourceQualifiedNodeIdV2 {
        self.expression
    }
    #[must_use]
    pub const fn polarity(&self) -> ExecutionExistsPolarityV3 {
        self.polarity
    }
    #[must_use]
    pub fn correlations(&self) -> &[ExecutionCorrelationV3] {
        &self.correlations
    }
}

/// Typed relation output contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionSchemaV3 {
    fields: Box<[ExecutionFieldV3]>,
    cardinality: ExecutionCardinalityV3,
    partition_keys: Box<[SourceQualifiedNodeIdV2]>,
    order: Box<[ExecutionOrderKeyV3]>,
}
impl ExecutionSchemaV3 {
    #[must_use]
    pub fn fields(&self) -> &[ExecutionFieldV3] {
        &self.fields
    }
    #[must_use]
    pub const fn cardinality(&self) -> ExecutionCardinalityV3 {
        self.cardinality
    }
    #[must_use]
    pub fn partition_keys(&self) -> &[SourceQualifiedNodeIdV2] {
        &self.partition_keys
    }
    #[must_use]
    pub fn order(&self) -> &[ExecutionOrderKeyV3] {
        &self.order
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionFieldV3 {
    name: String,
    value_type: ExecutionValueTypeV3,
}
impl ExecutionFieldV3 {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn value_type(&self) -> &ExecutionValueTypeV3 {
        &self.value_type
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionOrderKeyV3 {
    expression: SourceQualifiedNodeIdV2,
    direction: ExecutionSortDirectionV3,
    null_order: ExecutionNullOrderV3,
    value_type: ExecutionValueTypeV3,
}
impl ExecutionOrderKeyV3 {
    #[must_use]
    pub const fn expression(&self) -> SourceQualifiedNodeIdV2 {
        self.expression
    }
    #[must_use]
    pub const fn direction(&self) -> ExecutionSortDirectionV3 {
        self.direction
    }
    #[must_use]
    pub const fn null_order(&self) -> ExecutionNullOrderV3 {
        self.null_order
    }
    #[must_use]
    pub const fn value_type(&self) -> &ExecutionValueTypeV3 {
        &self.value_type
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionExpressionV3 {
    id: SourceQualifiedNodeIdV2,
    value_type: ExecutionValueTypeV3,
    shape: ExecutionExpressionShapeV3,
}
impl ExecutionExpressionV3 {
    #[must_use]
    pub const fn id(&self) -> SourceQualifiedNodeIdV2 {
        self.id
    }
    #[must_use]
    pub const fn value_type(&self) -> &ExecutionValueTypeV3 {
        &self.value_type
    }
    #[must_use]
    pub const fn shape(&self) -> &ExecutionExpressionShapeV3 {
        &self.shape
    }
}

/// Closed executable expression vocabulary. Child identities address entries
/// in the document expression table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionExpressionShapeV3 {
    Literal(ExecutionLiteralV3),
    Binding(ExecutionBindingV3),
    Path {
        base: SourceQualifiedNodeIdV2,
        segments: Box<[String]>,
    },
    Unary {
        operator: ExecutionOperatorV3,
        operand: SourceQualifiedNodeIdV2,
    },
    Binary {
        operator: ExecutionOperatorV3,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    Call {
        target: ExecutionCallTargetV3,
        arguments: Box<[Option<SourceQualifiedNodeIdV2>]>,
    },
    Paren(SourceQualifiedNodeIdV2),
    Record(Box<[(String, SourceQualifiedNodeIdV2)]>),
    List(Box<[SourceQualifiedNodeIdV2]>),
    BoolValue(SourceQualifiedNodeIdV2),
    Compare {
        operator: ExecutionOperatorV3,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    And {
        operator: ExecutionOperatorV3,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    Or {
        operator: ExecutionOperatorV3,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    Not {
        operator: ExecutionOperatorV3,
        operand: SourceQualifiedNodeIdV2,
    },
    LogicalQuantifier {
        kind: ExecutionLogicalQuantifierV3,
        binders: Box<[ExecutionBindingV3]>,
        body: SourceQualifiedNodeIdV2,
    },
    RelationExists {
        polarity: ExecutionExistsPolarityV3,
        relation_root: SourceQualifiedNodeIdV2,
        correlations: Box<[ExecutionCorrelationV3]>,
    },
}

/// Typed literal projected from the checked type owner, never parsed from a
/// display witness. The containing execution document has no public mint path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionLiteralV3 {
    Int(i128),
    Float(ExecutionFiniteFloatV3),
    Bool(bool),
    String(String),
    Null,
}

/// Canonical finite IEEE-754 bits. The type owner normalizes negative zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExecutionFiniteFloatV3(u64);

impl ExecutionFiniteFloatV3 {
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    #[must_use]
    pub fn get(self) -> f64 {
        f64::from_bits(self.0)
    }
}

/// Public semantic operator: the canonical overload identity from the closed
/// registry plus its stable spelling witness. The `overload_id` is the stable
/// identity; `symbol` renders it and is never parsed back.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionOperatorV3 {
    symbol: String,
    overload_id: u32,
}
impl ExecutionOperatorV3 {
    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    #[must_use]
    pub const fn overload_id(&self) -> u32 {
        self.overload_id
    }
}

fn project_operator(
    operator: quanta_gqlang_v2_sema::OperatorIdV2,
) -> Result<ExecutionOperatorV3, ExecutionClosureFailureV3> {
    Ok(ExecutionOperatorV3 {
        symbol: registered_execution_symbol(operator.symbol())?,
        overload_id: operator.raw(),
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionRelationAuthorityV3 {
    Host,
    CanonicalWorkspaceFiles,
    CatalogView,
    CompositionBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionSourceV3 {
    authority: ExecutionRelationAuthorityV3,
    kind: ExecutionSourceKindV3,
    namespace: Option<String>,
    name: String,
    composition_binding: Option<ExecutionBindingV3>,
}
impl ExecutionSourceV3 {
    #[must_use]
    pub const fn authority(&self) -> ExecutionRelationAuthorityV3 {
        self.authority
    }
    #[must_use]
    pub const fn kind(&self) -> ExecutionSourceKindV3 {
        self.kind
    }
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Sealed Binder slot for a Composition source. Its name is also encoded in
    /// the scan and its exact slot is encoded by the enclosing composition.
    #[must_use]
    pub const fn composition_binding(&self) -> Option<&ExecutionBindingV3> {
        self.composition_binding.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionBindingV3 {
    source_id: SourceIdV2,
    owner_node: u32,
    ordinal: usize,
    name: String,
}
impl ExecutionBindingV3 {
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.source_id
    }
    #[must_use]
    pub const fn owner_node(&self) -> u32 {
        self.owner_node
    }
    #[must_use]
    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionCorrelationV3 {
    binding: ExecutionBindingV3,
    reference: SourceQualifiedNodeIdV2,
}
impl ExecutionCorrelationV3 {
    #[must_use]
    pub const fn binding(&self) -> &ExecutionBindingV3 {
        &self.binding
    }
    #[must_use]
    pub const fn reference(&self) -> SourceQualifiedNodeIdV2 {
        self.reference
    }
}

/// Ordered composition producer/binding contract plus exact WITH and body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionCompositionV3 {
    bindings: Box<[ExecutionCompositionBindingV3]>,
    with_binding: ExecutionBindingV3,
    body_item_id: SemanticItemIdV2,
    body_family: ExecutionFamilyV3,
    body_relation: Option<ExecutionRelationV3>,
    body: ExecutionCompositionBodyV3,
}
impl ExecutionCompositionV3 {
    #[must_use]
    pub fn bindings(&self) -> &[ExecutionCompositionBindingV3] {
        &self.bindings
    }
    #[must_use]
    pub const fn with_binding(&self) -> &ExecutionBindingV3 {
        &self.with_binding
    }
    #[must_use]
    pub const fn body_item_id(&self) -> SemanticItemIdV2 {
        self.body_item_id
    }
    #[must_use]
    pub const fn body_family(&self) -> ExecutionFamilyV3 {
        self.body_family
    }
    #[must_use]
    pub const fn body_relation(&self) -> Option<&ExecutionRelationV3> {
        self.body_relation.as_ref()
    }
    #[must_use]
    pub const fn body(&self) -> &ExecutionCompositionBodyV3 {
        &self.body
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionCompositionBodyV3 {
    Relation,
    Graph(ExecutionGraphQueryV3),
    Check,
    Explain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionCompositionBindingV3 {
    binding: ExecutionBindingV3,
    producer_item_id: SemanticItemIdV2,
    producer_family: ExecutionFamilyV3,
    schema: Option<ExecutionSchemaV3>,
    relation: Option<ExecutionRelationV3>,
    producer: ExecutionCompositionProducerV3,
}
impl ExecutionCompositionBindingV3 {
    #[must_use]
    pub const fn binding(&self) -> &ExecutionBindingV3 {
        &self.binding
    }
    #[must_use]
    pub const fn producer_item_id(&self) -> SemanticItemIdV2 {
        self.producer_item_id
    }
    #[must_use]
    pub const fn producer_family(&self) -> ExecutionFamilyV3 {
        self.producer_family
    }
    #[must_use]
    pub const fn schema(&self) -> Option<&ExecutionSchemaV3> {
        self.schema.as_ref()
    }
    #[must_use]
    pub const fn relation(&self) -> Option<&ExecutionRelationV3> {
        self.relation.as_ref()
    }
    #[must_use]
    pub const fn producer(&self) -> &ExecutionCompositionProducerV3 {
        &self.producer
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionCompositionProducerV3 {
    Relation,
    Sample {
        query_text: String,
        mode: ExecutionSampleModeV3,
        limit: u32,
        scope: Option<ExecutionGraphTargetV3>,
    },
    Graph {
        query: ExecutionGraphQueryV3,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionSampleModeV3 {
    Lexical,
    Semantic,
    Hybrid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionGraphKindV3 {
    Reachability,
    Flow,
    Slice,
    Taint,
    Path,
    PointsTo,
    Summary,
}

/// Closed traversal directions carried to runtime admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionGraphDirectionV3 {
    Forward,
    Backward,
    Both,
    Undirected,
}

/// Semantic role of one projected FIND selector.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionGraphReferenceRoleV3 {
    Source,
    Sink,
    Anchor,
    Reference,
    Pointee,
    Callable,
    Via,
    Through,
    Sanitizer,
    Scope,
    Window,
    Frontier,
}

/// Host selector kinds reachable from a FIND or scoped SAMPLE payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionGraphReferenceKindV3 {
    Entity,
    Callable,
    Scope,
    Window,
    Frontier,
    Sanitizer,
    Pattern,
    Snapshot,
    ExportSource,
}

/// Capability proved for a prior Composition binding used as a graph target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionGraphCapabilityV3 {
    Entity,
    Scope,
}

/// Whether the sealed witness relation can revisit nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionGraphRelationShapeV3 {
    Acyclic,
    Cyclic,
}

/// Exact identity used to deduplicate bounded graph witnesses.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionGraphDedupIdentityV3 {
    Endpoints,
    NodeSequence,
    NodeSet,
}

/// Deterministic ordering used when a witness cap truncates candidates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionGraphWitnessTieBreakV3 {
    ShortestThenLexical,
    Lexical,
}

/// Exact host or prior-Composition target used by a graph-bearing payload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionGraphTargetV3 {
    Host {
        kind: ExecutionGraphReferenceKindV3,
        spelling: String,
        entity_subtype: Option<String>,
    },
    Composition {
        binding: ExecutionBindingV3,
        capability: ExecutionGraphCapabilityV3,
    },
}

/// One role-qualified target in a projected FIND query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionGraphReferenceV3 {
    role: ExecutionGraphReferenceRoleV3,
    target: ExecutionGraphTargetV3,
}

impl ExecutionGraphReferenceV3 {
    #[must_use]
    pub const fn role(&self) -> ExecutionGraphReferenceRoleV3 {
        self.role
    }

    #[must_use]
    pub const fn target(&self) -> &ExecutionGraphTargetV3 {
        &self.target
    }
}

/// Complete checked FIND payload needed for downstream admission or rejection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionGraphQueryV3 {
    kind: ExecutionGraphKindV3,
    references: Box<[ExecutionGraphReferenceV3]>,
    depth: u32,
    direction: Option<ExecutionGraphDirectionV3>,
    summary_facets: Box<[String]>,
    relation_shape: ExecutionGraphRelationShapeV3,
    dedup_identity: ExecutionGraphDedupIdentityV3,
    max_witnesses: u32,
    max_path_length: Option<u32>,
    witness_tie_break: ExecutionGraphWitnessTieBreakV3,
}

impl ExecutionGraphQueryV3 {
    #[must_use]
    pub const fn kind(&self) -> ExecutionGraphKindV3 {
        self.kind
    }

    #[must_use]
    pub fn references(&self) -> &[ExecutionGraphReferenceV3] {
        &self.references
    }

    #[must_use]
    pub const fn depth(&self) -> u32 {
        self.depth
    }

    #[must_use]
    pub const fn direction(&self) -> Option<ExecutionGraphDirectionV3> {
        self.direction
    }

    #[must_use]
    pub fn summary_facets(&self) -> &[String] {
        &self.summary_facets
    }

    #[must_use]
    pub const fn relation_shape(&self) -> ExecutionGraphRelationShapeV3 {
        self.relation_shape
    }

    #[must_use]
    pub const fn dedup_identity(&self) -> ExecutionGraphDedupIdentityV3 {
        self.dedup_identity
    }

    #[must_use]
    pub const fn max_witnesses(&self) -> u32 {
        self.max_witnesses
    }

    #[must_use]
    pub const fn max_path_length(&self) -> Option<u32> {
        self.max_path_length
    }

    #[must_use]
    pub const fn witness_tie_break(&self) -> ExecutionGraphWitnessTieBreakV3 {
        self.witness_tie_break
    }
}

/// Execution projection cannot close an expression's source or symbol identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionClosureFailureV3 {
    /// A referenced expression identity is absent from the document table.
    Dangling,
    /// A referenced expression identity belongs to another source.
    Foreign,
    /// The closed owner ID has no registered builtin name or operator symbol.
    UnregisteredSymbol,
}

mod encoding;
mod projection;
mod scalar_request;

#[doc(hidden)]
pub mod producer;
mod vocabulary;

use vocabulary::{
    project_call_target, project_cardinality, project_exists_polarity, project_join_mode,
    project_logical_quantifier, project_null_order, project_plan, project_scalar_plan,
    project_sort_direction, project_value_type, registered_execution_symbol,
};
pub use vocabulary::{
    ExecutionCallTargetV3, ExecutionCardinalityV3, ExecutionExistsPolarityV3, ExecutionJoinModeV3,
    ExecutionLogicalQuantifierV3, ExecutionNullOrderV3, ExecutionRecordFieldV3,
    ExecutionScalarTypeV3, ExecutionSortDirectionV3, ExecutionSourceKindV3,
    ExecutionUnionVariantV3, ExecutionValueTypeV3,
};

#[cfg(test)]
use projection::{close_expression_ids, project_expression_shape};
#[cfg(test)]
use projection::{
    project_composition, ExecutionCompositionBodyProjectionV3,
    ExecutionCompositionProducerProjectionV3, ExecutionCompositionProjectionFailureV3,
};
#[cfg(test)]
#[path = "digest_oracle.rs"]
mod digest_oracle;

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    reason = "contract digest mutation rails assert exact fixture shapes"
)]
#[path = "tests.rs"]
mod tests;
