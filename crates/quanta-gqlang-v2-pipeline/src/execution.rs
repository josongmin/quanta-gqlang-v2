//! Host-agnostic, immutable execution contract projected from sealed V2 owners.
//!
//! This module copies only public semantic facts into a closed transport DTO.
//! It does not expose HIR, Sema, or Algebra owner types.

use quanta_gqlang_v2_algebra::{CheckedAlgebraNodeV2, LoweredRelationItemV2};
use quanta_gqlang_v2_planning::{
    AuthorityLaneV2, EffectClassV2, EvaluationDeterminismV2, EvidenceRequirementV2, PlanPolicyV2,
    SemanticPlanV2,
};
use quanta_gqlang_v2_sema::{
    CardinalityV1, ExpressionTypeStateV2, GraphQueryKindV1, JoinModeV2, LogicalQuantifierKindV2,
    NullOrderV2, RelationAuthorityV2, RelationExistsPolarityV2, RelationOutputContractV2,
    ResolvedCallTargetV2, ResolvedCompositionBodyV2, ResolvedCompositionProducerV2,
    ResolvedCompositionV2, ResolvedPresentationKindV2, ResolvedRelationSourceKindV2, SampleModeV1,
    SemanticDocumentV2, SemanticItemIdV2, SortDirectionV2, SourceIdV2, SourceQualifiedNodeIdV2,
    TypedCallArgumentV2, TypedExprShapeV2, TypedExprV2, TypedLiteralFactV2, TypedLiteralValueV2,
};

/// SHA-256 digest of the canonical execution-contract encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ExecutionDigestV2([u8; 32]);

impl ExecutionDigestV2 {
    /// Raw digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Lower-case hexadecimal form.
    #[must_use]
    pub fn to_hex(self) -> String {
        let mut result = String::with_capacity(64);
        for byte in self.0 {
            use std::fmt::Write as _;
            write!(&mut result, "{byte:02x}").expect("writing to String cannot fail");
        }
        result
    }
}

/// One immutable, source-ordered execution document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionDocumentV2 {
    source_id: SourceIdV2,
    items: Box<[ExecutionItemV2]>,
    source_digest: ExecutionDigestV2,
    digest: ExecutionDigestV2,
}

impl ExecutionDocumentV2 {
    pub(crate) fn new(source_id: SourceIdV2, items: Vec<ExecutionItemV2>) -> Self {
        let source_digest = digest_source(source_id, &items);
        let digest = digest_document(source_id, source_digest, &items);
        Self {
            source_id,
            items: items.into_boxed_slice(),
            source_digest,
            digest,
        }
    }

    /// Source document identity.
    #[must_use]
    pub const fn source_id(&self) -> SourceIdV2 {
        self.source_id
    }

    /// Source-ordered executable items. Non-executable declarations are absent.
    #[must_use]
    pub fn items(&self) -> &[ExecutionItemV2] {
        &self.items
    }

    /// Exact executable item by semantic identity.
    #[must_use]
    pub fn item(&self, item_id: SemanticItemIdV2) -> Option<&ExecutionItemV2> {
        self.items.iter().find(|item| item.item_id == item_id)
    }

    /// Digest of the complete canonical DTO encoding.
    #[must_use]
    pub const fn digest(&self) -> ExecutionDigestV2 {
        self.digest
    }

    /// Digest of source-derived identities, expression trees, and relation trees.
    #[must_use]
    pub const fn source_digest(&self) -> ExecutionDigestV2 {
        self.source_digest
    }

    pub(crate) fn bind_source_bytes(&mut self, source: &[u8]) {
        let mut bytes = Vec::new();
        put_str(&mut bytes, "qglang.execution.source-bytes.v2");
        put_usize(&mut bytes, source.len());
        bytes.extend_from_slice(source);
        self.source_digest = ExecutionDigestV2(sha256(&bytes));
        self.digest = digest_document(self.source_id, self.source_digest, &self.items);
    }
}

/// Closed top-level execution family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionFamilyV2 {
    Relation,
    Graph,
    Check,
    Presentation,
    Composition,
    MetaRequireCertified,
    RuleProgram,
}

impl ExecutionFamilyV2 {
    /// Stable wire keyword.
    #[must_use]
    pub const fn keyword(self) -> &'static str {
        match self {
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

/// One executable item with its immutable planning authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionItemV2 {
    item_id: SemanticItemIdV2,
    family: ExecutionFamilyV2,
    relation: Option<ExecutionRelationV2>,
    composition: Option<ExecutionCompositionV2>,
    plan: ExecutionPlanV2,
}

impl ExecutionItemV2 {
    pub(crate) fn new(
        item_id: SemanticItemIdV2,
        family: ExecutionFamilyV2,
        relation: Option<ExecutionRelationV2>,
        composition: Option<ExecutionCompositionV2>,
        plan: &SemanticPlanV2,
    ) -> Self {
        Self {
            item_id,
            family,
            relation,
            composition,
            plan: project_plan(plan),
        }
    }

    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    #[must_use]
    pub const fn family(&self) -> ExecutionFamilyV2 {
        self.family
    }
    #[must_use]
    pub const fn relation(&self) -> Option<&ExecutionRelationV2> {
        self.relation.as_ref()
    }
    #[must_use]
    pub const fn composition(&self) -> Option<&ExecutionCompositionV2> {
        self.composition.as_ref()
    }
    /// Producer-derived policy and authority; callers cannot replace it.
    #[must_use]
    pub const fn plan(&self) -> &ExecutionPlanV2 {
        &self.plan
    }
}

/// Runtime-relevant policy copied from the sealed semantic plan.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionPlanV2 {
    authority: ExecutionAuthorityV2,
    determinism: ExecutionDeterminismV2,
    evidence: ExecutionEvidenceV2,
    effect: ExecutionEffectV2,
    policy_family: ExecutionPolicyFamilyV2,
}
impl ExecutionPlanV2 {
    #[must_use]
    pub const fn authority(&self) -> ExecutionAuthorityV2 {
        self.authority
    }
    #[must_use]
    pub const fn determinism(&self) -> ExecutionDeterminismV2 {
        self.determinism
    }
    #[must_use]
    pub const fn evidence(&self) -> ExecutionEvidenceV2 {
        self.evidence
    }
    #[must_use]
    pub const fn effect(&self) -> ExecutionEffectV2 {
        self.effect
    }
    #[must_use]
    pub const fn policy_family(&self) -> ExecutionPolicyFamilyV2 {
        self.policy_family
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionAuthorityV2 {
    FrontendChecked,
    CertifiedRequirementValidated,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionDeterminismV2 {
    Deterministic,
    NonDeterministic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionEvidenceV2 {
    None,
    SourceLocation,
    WitnessPayload,
    ProofTree,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionEffectV2 {
    ReadOnly,
    ProofEvidenceOnly,
    ExportOnly,
    MutationForbidden,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionPolicyFamilyV2 {
    Relational,
    Graph,
    Check,
    Presentation,
    RuleProgram,
    MetaRequireCertified,
    Composition,
}
impl ExecutionPolicyFamilyV2 {
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
        }
    }
}

fn project_plan(plan: &SemanticPlanV2) -> ExecutionPlanV2 {
    let authority = match plan.authority_lane() {
        AuthorityLaneV2::FrontendChecked => ExecutionAuthorityV2::FrontendChecked,
        AuthorityLaneV2::CertifiedRequirementValidated => {
            ExecutionAuthorityV2::CertifiedRequirementValidated
        }
    };
    let determinism = match plan.evaluation_determinism() {
        EvaluationDeterminismV2::Deterministic => ExecutionDeterminismV2::Deterministic,
        EvaluationDeterminismV2::NonDeterministic => ExecutionDeterminismV2::NonDeterministic,
    };
    let evidence = match plan.evidence() {
        EvidenceRequirementV2::None => ExecutionEvidenceV2::None,
        EvidenceRequirementV2::SourceLocation => ExecutionEvidenceV2::SourceLocation,
        EvidenceRequirementV2::WitnessPayload => ExecutionEvidenceV2::WitnessPayload,
        EvidenceRequirementV2::ProofTree => ExecutionEvidenceV2::ProofTree,
    };
    let effect = match plan.effect() {
        EffectClassV2::ReadOnly => ExecutionEffectV2::ReadOnly,
        EffectClassV2::ProofEvidenceOnly => ExecutionEffectV2::ProofEvidenceOnly,
        EffectClassV2::ExportOnly => ExecutionEffectV2::ExportOnly,
        EffectClassV2::MutationForbidden => ExecutionEffectV2::MutationForbidden,
    };
    let policy_family = match plan.policy() {
        PlanPolicyV2::Relational { .. } => ExecutionPolicyFamilyV2::Relational,
        PlanPolicyV2::Graph(_) => ExecutionPolicyFamilyV2::Graph,
        PlanPolicyV2::Check(_) => ExecutionPolicyFamilyV2::Check,
        PlanPolicyV2::Presentation(_) => ExecutionPolicyFamilyV2::Presentation,
        PlanPolicyV2::Rule(_) => ExecutionPolicyFamilyV2::RuleProgram,
        PlanPolicyV2::MetaRequireCertified => ExecutionPolicyFamilyV2::MetaRequireCertified,
        PlanPolicyV2::Composition(_) => ExecutionPolicyFamilyV2::Composition,
    };
    ExecutionPlanV2 {
        authority,
        determinism,
        evidence,
        effect,
        policy_family,
    }
}

const fn authority_keyword(value: ExecutionAuthorityV2) -> &'static str {
    match value {
        ExecutionAuthorityV2::FrontendChecked => "frontend_checked",
        ExecutionAuthorityV2::CertifiedRequirementValidated => "certified_requirement_validated",
    }
}
const fn determinism_keyword(value: ExecutionDeterminismV2) -> &'static str {
    match value {
        ExecutionDeterminismV2::Deterministic => "deterministic",
        ExecutionDeterminismV2::NonDeterministic => "non_deterministic",
    }
}
const fn evidence_keyword(value: ExecutionEvidenceV2) -> &'static str {
    match value {
        ExecutionEvidenceV2::None => "none",
        ExecutionEvidenceV2::SourceLocation => "source_location",
        ExecutionEvidenceV2::WitnessPayload => "witness_payload",
        ExecutionEvidenceV2::ProofTree => "proof_tree",
    }
}
const fn effect_keyword(value: ExecutionEffectV2) -> &'static str {
    match value {
        ExecutionEffectV2::ReadOnly => "read_only",
        ExecutionEffectV2::ProofEvidenceOnly => "proof_evidence_only",
        ExecutionEffectV2::ExportOnly => "export_only",
        ExecutionEffectV2::MutationForbidden => "mutation_forbidden",
    }
}

/// A sealed relational operator tree and its typed expression table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRelationV2 {
    item_id: SemanticItemIdV2,
    root: ExecutionRelationalOperatorV2,
    expressions: Box<[ExecutionExpressionV2]>,
}

impl ExecutionRelationV2 {
    #[must_use]
    pub const fn item_id(&self) -> SemanticItemIdV2 {
        self.item_id
    }
    #[must_use]
    pub const fn root(&self) -> &ExecutionRelationalOperatorV2 {
        &self.root
    }
    #[must_use]
    pub fn expressions(&self) -> &[ExecutionExpressionV2] {
        &self.expressions
    }
}

/// One closed relational operator node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRelationalOperatorV2 {
    family: String,
    stage: SourceQualifiedNodeIdV2,
    source: Option<ExecutionSourceV2>,
    schema: ExecutionSchemaV2,
    predicates: Box<[SourceQualifiedNodeIdV2]>,
    expressions: Box<[SourceQualifiedNodeIdV2]>,
    correlations: Box<[ExecutionCorrelationV2]>,
    partition_input: Box<[ExecutionBindingV2]>,
    children: Box<[ExecutionRelationalOperatorV2]>,
}

impl ExecutionRelationalOperatorV2 {
    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }
    #[must_use]
    pub const fn stage(&self) -> SourceQualifiedNodeIdV2 {
        self.stage
    }
    #[must_use]
    pub const fn source(&self) -> Option<&ExecutionSourceV2> {
        self.source.as_ref()
    }
    #[must_use]
    pub const fn schema(&self) -> &ExecutionSchemaV2 {
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
    pub fn correlations(&self) -> &[ExecutionCorrelationV2] {
        &self.correlations
    }
    #[must_use]
    pub fn partition_input(&self) -> &[ExecutionBindingV2] {
        &self.partition_input
    }
    #[must_use]
    pub fn children(&self) -> &[ExecutionRelationalOperatorV2] {
        &self.children
    }
}

/// Typed relation output contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionSchemaV2 {
    fields: Box<[ExecutionFieldV2]>,
    cardinality: String,
    partition_keys: Box<[SourceQualifiedNodeIdV2]>,
    order: Box<[ExecutionOrderKeyV2]>,
}
impl ExecutionSchemaV2 {
    #[must_use]
    pub fn fields(&self) -> &[ExecutionFieldV2] {
        &self.fields
    }
    #[must_use]
    pub fn cardinality(&self) -> &str {
        &self.cardinality
    }
    #[must_use]
    pub fn partition_keys(&self) -> &[SourceQualifiedNodeIdV2] {
        &self.partition_keys
    }
    #[must_use]
    pub fn order(&self) -> &[ExecutionOrderKeyV2] {
        &self.order
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionFieldV2 {
    name: String,
    value_type: String,
}
impl ExecutionFieldV2 {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub fn value_type(&self) -> &str {
        &self.value_type
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionOrderKeyV2 {
    expression: SourceQualifiedNodeIdV2,
    direction: String,
    null_order: String,
    value_type: String,
}
impl ExecutionOrderKeyV2 {
    #[must_use]
    pub const fn expression(&self) -> SourceQualifiedNodeIdV2 {
        self.expression
    }
    #[must_use]
    pub fn direction(&self) -> &str {
        &self.direction
    }
    #[must_use]
    pub fn null_order(&self) -> &str {
        &self.null_order
    }
    #[must_use]
    pub fn value_type(&self) -> &str {
        &self.value_type
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionExpressionV2 {
    id: SourceQualifiedNodeIdV2,
    value_type: String,
    shape: ExecutionExpressionShapeV2,
}
impl ExecutionExpressionV2 {
    #[must_use]
    pub const fn id(&self) -> SourceQualifiedNodeIdV2 {
        self.id
    }
    #[must_use]
    pub fn value_type(&self) -> &str {
        &self.value_type
    }
    #[must_use]
    pub const fn shape(&self) -> &ExecutionExpressionShapeV2 {
        &self.shape
    }
}

/// Closed executable expression vocabulary. Child identities address entries
/// in the document expression table.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionExpressionShapeV2 {
    Literal(String),
    Binding(ExecutionBindingV2),
    Path {
        base: SourceQualifiedNodeIdV2,
        segments: Box<[String]>,
    },
    Unary {
        operator: ExecutionOperatorV2,
        operand: SourceQualifiedNodeIdV2,
    },
    Binary {
        operator: ExecutionOperatorV2,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    Call {
        target: String,
        arguments: Box<[Option<SourceQualifiedNodeIdV2>]>,
    },
    Paren(SourceQualifiedNodeIdV2),
    Record(Box<[(String, SourceQualifiedNodeIdV2)]>),
    List(Box<[SourceQualifiedNodeIdV2]>),
    BoolValue(SourceQualifiedNodeIdV2),
    Compare {
        operator: ExecutionOperatorV2,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    And {
        operator: ExecutionOperatorV2,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    Or {
        operator: ExecutionOperatorV2,
        lhs: SourceQualifiedNodeIdV2,
        rhs: SourceQualifiedNodeIdV2,
    },
    Not {
        operator: ExecutionOperatorV2,
        operand: SourceQualifiedNodeIdV2,
    },
    LogicalQuantifier {
        kind: String,
        binders: Box<[ExecutionBindingV2]>,
        body: SourceQualifiedNodeIdV2,
    },
    RelationExists {
        polarity: String,
        relation_root: SourceQualifiedNodeIdV2,
        correlations: Box<[ExecutionCorrelationV2]>,
    },
}

/// Public semantic operator plus the exact checked overload identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionOperatorV2 {
    symbol: String,
    overload_id: u32,
}
impl ExecutionOperatorV2 {
    #[must_use]
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    #[must_use]
    pub const fn overload_id(&self) -> u32 {
        self.overload_id
    }
}

fn project_operator(operator: quanta_gqlang_v2_sema::OperatorIdV2) -> ExecutionOperatorV2 {
    ExecutionOperatorV2 {
        symbol: operator
            .symbol()
            .unwrap_or("invalid_closed_operator")
            .to_owned(),
        overload_id: operator.raw(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionRelationAuthorityV2 {
    Host,
    CanonicalWorkspaceFiles,
    CatalogView,
    CompositionBinding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionSourceV2 {
    authority: ExecutionRelationAuthorityV2,
    kind: String,
    namespace: Option<String>,
    name: String,
}
impl ExecutionSourceV2 {
    #[must_use]
    pub const fn authority(&self) -> ExecutionRelationAuthorityV2 {
        self.authority
    }
    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }
    #[must_use]
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionBindingV2 {
    source_id: SourceIdV2,
    owner_node: u32,
    ordinal: usize,
    name: String,
}
impl ExecutionBindingV2 {
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
pub struct ExecutionCorrelationV2 {
    binding: ExecutionBindingV2,
    reference: SourceQualifiedNodeIdV2,
}
impl ExecutionCorrelationV2 {
    #[must_use]
    pub const fn binding(&self) -> &ExecutionBindingV2 {
        &self.binding
    }
    #[must_use]
    pub const fn reference(&self) -> SourceQualifiedNodeIdV2 {
        self.reference
    }
}

/// Ordered composition producer/binding contract plus exact WITH and body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionCompositionV2 {
    bindings: Box<[ExecutionCompositionBindingV2]>,
    with_binding: ExecutionBindingV2,
    body_item_id: SemanticItemIdV2,
    body_family: ExecutionFamilyV2,
    body_relation: Option<ExecutionRelationV2>,
    body: ExecutionCompositionBodyV2,
}
impl ExecutionCompositionV2 {
    #[must_use]
    pub fn bindings(&self) -> &[ExecutionCompositionBindingV2] {
        &self.bindings
    }
    #[must_use]
    pub const fn with_binding(&self) -> &ExecutionBindingV2 {
        &self.with_binding
    }
    #[must_use]
    pub const fn body_item_id(&self) -> SemanticItemIdV2 {
        self.body_item_id
    }
    #[must_use]
    pub const fn body_family(&self) -> ExecutionFamilyV2 {
        self.body_family
    }
    #[must_use]
    pub const fn body_relation(&self) -> Option<&ExecutionRelationV2> {
        self.body_relation.as_ref()
    }
    #[must_use]
    pub const fn body(&self) -> &ExecutionCompositionBodyV2 {
        &self.body
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionCompositionBodyV2 {
    Relation,
    Graph(ExecutionGraphKindV2),
    Check,
    Explain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionCompositionBindingV2 {
    binding: ExecutionBindingV2,
    producer_item_id: SemanticItemIdV2,
    producer_family: ExecutionFamilyV2,
    schema: Option<ExecutionSchemaV2>,
    relation: Option<ExecutionRelationV2>,
    producer: ExecutionCompositionProducerV2,
}
impl ExecutionCompositionBindingV2 {
    #[must_use]
    pub const fn binding(&self) -> &ExecutionBindingV2 {
        &self.binding
    }
    #[must_use]
    pub const fn producer_item_id(&self) -> SemanticItemIdV2 {
        self.producer_item_id
    }
    #[must_use]
    pub const fn producer_family(&self) -> ExecutionFamilyV2 {
        self.producer_family
    }
    #[must_use]
    pub const fn schema(&self) -> Option<&ExecutionSchemaV2> {
        self.schema.as_ref()
    }
    #[must_use]
    pub const fn relation(&self) -> Option<&ExecutionRelationV2> {
        self.relation.as_ref()
    }
    #[must_use]
    pub const fn producer(&self) -> &ExecutionCompositionProducerV2 {
        &self.producer
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionCompositionProducerV2 {
    Relation,
    Sample {
        query_text: String,
        mode: ExecutionSampleModeV2,
        limit: u32,
    },
    Graph {
        kind: ExecutionGraphKindV2,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionSampleModeV2 {
    Lexical,
    Semantic,
    Hybrid,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionGraphKindV2 {
    Reachability,
    Flow,
    Slice,
    Taint,
    Path,
    PointsTo,
    Summary,
}

pub(crate) fn project_relation(
    document: &SemanticDocumentV2,
    lowered: &LoweredRelationItemV2,
) -> ExecutionRelationV2 {
    project_relation_root(document, lowered.item_id(), lowered.root())
}

pub(crate) fn project_relation_root(
    document: &SemanticDocumentV2,
    item_id: SemanticItemIdV2,
    root: &CheckedAlgebraNodeV2,
) -> ExecutionRelationV2 {
    ExecutionRelationV2 {
        item_id,
        root: project_node(root),
        expressions: document
            .expressions()
            .map(|(_, expr)| project_expression(expr))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    }
}

pub(crate) fn project_composition(
    document: &SemanticDocumentV2,
    composition: &ResolvedCompositionV2,
    body_relation: Option<ExecutionRelationV2>,
    producer_relations: Vec<Option<ExecutionRelationV2>>,
) -> ExecutionCompositionV2 {
    let bindings = composition
        .bindings()
        .iter()
        .zip(producer_relations)
        .map(|(entry, relation)| {
            let (family, producer) = match entry.producer() {
                ResolvedCompositionProducerV2::Query(_) => (
                    ExecutionFamilyV2::Relation,
                    ExecutionCompositionProducerV2::Relation,
                ),
                ResolvedCompositionProducerV2::Sample { presentation, .. } => {
                    let ResolvedPresentationKindV2::Sample {
                        query_text,
                        mode,
                        limit,
                        ..
                    } = presentation.kind()
                    else {
                        unreachable!("sealed composition SAMPLE producer")
                    };
                    (
                        ExecutionFamilyV2::Presentation,
                        ExecutionCompositionProducerV2::Sample {
                            query_text: query_text.clone(),
                            mode: project_sample_mode(*mode),
                            limit: *limit,
                        },
                    )
                }
                ResolvedCompositionProducerV2::Graph { graph, .. } => (
                    ExecutionFamilyV2::Graph,
                    ExecutionCompositionProducerV2::Graph {
                        kind: project_graph_kind(graph.kind()),
                    },
                ),
            };
            ExecutionCompositionBindingV2 {
                binding: project_binding(entry.binding()),
                producer_item_id: entry.producer().item_id(),
                producer_family: family,
                schema: entry.relation_output().map(project_schema),
                relation,
                producer,
            }
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let (body_item_id, body_family, body) = match composition.body() {
        ResolvedCompositionBodyV2::Query(body) => (
            body.item_id(),
            ExecutionFamilyV2::Relation,
            ExecutionCompositionBodyV2::Relation,
        ),
        ResolvedCompositionBodyV2::Graph(body) => (
            body.item_id(),
            ExecutionFamilyV2::Graph,
            ExecutionCompositionBodyV2::Graph(project_graph_kind(body.kind())),
        ),
        ResolvedCompositionBodyV2::Check(body) => (
            body.item_id(),
            ExecutionFamilyV2::Check,
            ExecutionCompositionBodyV2::Check,
        ),
        ResolvedCompositionBodyV2::Explain(body) => (
            body.item_id(),
            ExecutionFamilyV2::Presentation,
            ExecutionCompositionBodyV2::Explain,
        ),
    };
    let _ = document;
    ExecutionCompositionV2 {
        bindings,
        with_binding: project_binding(composition.with_target()),
        body_item_id,
        body_family,
        body_relation,
        body,
    }
}

const fn project_sample_mode(mode: SampleModeV1) -> ExecutionSampleModeV2 {
    match mode {
        SampleModeV1::Lexical => ExecutionSampleModeV2::Lexical,
        SampleModeV1::Semantic => ExecutionSampleModeV2::Semantic,
        SampleModeV1::Hybrid => ExecutionSampleModeV2::Hybrid,
    }
}
const fn project_graph_kind(kind: GraphQueryKindV1) -> ExecutionGraphKindV2 {
    match kind {
        GraphQueryKindV1::Reachability => ExecutionGraphKindV2::Reachability,
        GraphQueryKindV1::Flow => ExecutionGraphKindV2::Flow,
        GraphQueryKindV1::Slice => ExecutionGraphKindV2::Slice,
        GraphQueryKindV1::Taint => ExecutionGraphKindV2::Taint,
        GraphQueryKindV1::Path => ExecutionGraphKindV2::Path,
        GraphQueryKindV1::PointsTo => ExecutionGraphKindV2::PointsTo,
        GraphQueryKindV1::Summary => ExecutionGraphKindV2::Summary,
    }
}

fn project_node(node: &CheckedAlgebraNodeV2) -> ExecutionRelationalOperatorV2 {
    let source = node.scan_source().map(project_source);
    let predicates = node
        .predicate()
        .into_iter()
        .map(|p| p.expression().key())
        .chain(node.filter_conjuncts().iter().map(|p| p.expression().key()))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let expressions = node
        .expressions()
        .iter()
        .map(TypedExprV2::key)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let correlations = node
        .correlations()
        .into_iter()
        .flat_map(|set| set.iter())
        .map(|edge| ExecutionCorrelationV2 {
            binding: project_binding(edge.binding()),
            reference: edge.reference(),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let partition_input = node
        .partition_input()
        .iter()
        .map(|entry| project_binding(entry.binding()))
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let children = node
        .children()
        .into_iter()
        .map(project_node)
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let family = match node.join_mode() {
        Some(JoinModeV2::Inner) => "join_inner",
        Some(JoinModeV2::Cross) => "join_cross",
        None => node.family().keyword(),
    }
    .to_owned();
    ExecutionRelationalOperatorV2 {
        family,
        stage: node.stage_key(),
        source,
        schema: project_schema(node.output()),
        predicates,
        expressions,
        correlations,
        partition_input,
        children,
    }
}

fn project_expression(expr: &TypedExprV2) -> ExecutionExpressionV2 {
    let value_type = match expr.type_state() {
        ExpressionTypeStateV2::Known(value) => value.to_string(),
        ExpressionTypeStateV2::Unknown(_) => "unknown".to_owned(),
    };
    ExecutionExpressionV2 {
        id: expr.key(),
        value_type,
        shape: project_expression_shape(expr.shape()),
    }
}

fn project_expression_shape(shape: &TypedExprShapeV2) -> ExecutionExpressionShapeV2 {
    match shape {
        TypedExprShapeV2::Literal(fact) => ExecutionExpressionShapeV2::Literal(match fact {
            TypedLiteralFactV2::DeferredI128MinMagnitude => {
                "int:-170141183460469231731687303715884105728".to_owned()
            }
            TypedLiteralFactV2::Value(value) => match value {
                TypedLiteralValueV2::Int(value) => format!("int:{value}"),
                TypedLiteralValueV2::Float(value) => {
                    format!("float:{:016x}", value.get().to_bits())
                }
                TypedLiteralValueV2::Bool(value) => format!("bool:{value}"),
                TypedLiteralValueV2::String(value) => format!("string:{value}"),
                TypedLiteralValueV2::Null => "null".to_owned(),
            },
        }),
        TypedExprShapeV2::Binding(binding) => {
            ExecutionExpressionShapeV2::Binding(project_binding(binding))
        }
        TypedExprShapeV2::Path { base, segments } => ExecutionExpressionShapeV2::Path {
            base: *base,
            segments: segments
                .iter()
                .map(|s| s.name().to_owned())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        },
        TypedExprShapeV2::Unary { operator, operand } => ExecutionExpressionShapeV2::Unary {
            operator: project_operator(*operator),
            operand: *operand,
        },
        TypedExprShapeV2::Binary { operator, lhs, rhs } => ExecutionExpressionShapeV2::Binary {
            operator: project_operator(*operator),
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::Call { target, arguments } => ExecutionExpressionShapeV2::Call {
            target: match target {
                ResolvedCallTargetV2::Builtin(id) => {
                    format!("builtin:{}", id.name().unwrap_or("invalid_closed_builtin"))
                }
                ResolvedCallTargetV2::Predicate(id) => format!("predicate:{}", id.as_str()),
                ResolvedCallTargetV2::HostPredicate(id) => id
                    .canonical_bytes()
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>(),
            },
            arguments: arguments
                .iter()
                .map(|arg| match arg {
                    TypedCallArgumentV2::Expression(id) => Some(*id),
                    TypedCallArgumentV2::Star => None,
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        },
        TypedExprShapeV2::Paren { inner } => ExecutionExpressionShapeV2::Paren(*inner),
        TypedExprShapeV2::Record(fields) => ExecutionExpressionShapeV2::Record(
            fields
                .iter()
                .map(|field| (field.name().to_owned(), field.value()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        TypedExprShapeV2::List(items) => {
            ExecutionExpressionShapeV2::List(items.clone().into_boxed_slice())
        }
        TypedExprShapeV2::BoolValue { value } => ExecutionExpressionShapeV2::BoolValue(*value),
        TypedExprShapeV2::Compare { operator, lhs, rhs } => ExecutionExpressionShapeV2::Compare {
            operator: project_operator(*operator),
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::And { operator, lhs, rhs } => ExecutionExpressionShapeV2::And {
            operator: project_operator(*operator),
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::Or { operator, lhs, rhs } => ExecutionExpressionShapeV2::Or {
            operator: project_operator(*operator),
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::Not { operator, operand } => ExecutionExpressionShapeV2::Not {
            operator: project_operator(*operator),
            operand: *operand,
        },
        TypedExprShapeV2::LogicalQuantifier {
            kind,
            binders,
            body,
        } => ExecutionExpressionShapeV2::LogicalQuantifier {
            kind: match kind {
                LogicalQuantifierKindV2::Exists => "exists",
                LogicalQuantifierKindV2::Forall => "forall",
            }
            .to_owned(),
            binders: binders
                .iter()
                .map(project_binding)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            body: *body,
        },
        TypedExprShapeV2::RelationExists {
            polarity,
            relation_root,
            correlations,
        } => ExecutionExpressionShapeV2::RelationExists {
            polarity: match polarity {
                RelationExistsPolarityV2::Exists => "exists",
                RelationExistsPolarityV2::NotExists => "not_exists",
            }
            .to_owned(),
            relation_root: *relation_root,
            correlations: correlations
                .iter()
                .map(|edge| ExecutionCorrelationV2 {
                    binding: project_binding(edge.binding()),
                    reference: edge.reference(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        },
    }
}

fn project_schema(output: &RelationOutputContractV2) -> ExecutionSchemaV2 {
    let fields = output
        .row()
        .fields()
        .iter()
        .map(|field| ExecutionFieldV2 {
            name: field.name().to_owned(),
            value_type: field.value_type().to_string(),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let cardinality = match output.cardinality() {
        CardinalityV1::Set => "set",
        CardinalityV1::Bag => "bag",
        CardinalityV1::Sequence => "sequence",
    }
    .to_owned();
    let order = output
        .ordering()
        .iter()
        .map(|key| ExecutionOrderKeyV2 {
            expression: key.expression(),
            direction: match key.direction() {
                SortDirectionV2::Asc => "asc",
                SortDirectionV2::Desc => "desc",
            }
            .to_owned(),
            null_order: match key.null_order() {
                NullOrderV2::First => "first",
                NullOrderV2::Last => "last",
            }
            .to_owned(),
            value_type: key.value_type().to_string(),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    ExecutionSchemaV2 {
        fields,
        cardinality,
        partition_keys: output.partition_keys().to_vec().into_boxed_slice(),
        order,
    }
}

fn project_source(source: &ResolvedRelationSourceKindV2) -> ExecutionSourceV2 {
    match source {
        ResolvedRelationSourceKindV2::Relation(id) => ExecutionSourceV2 {
            authority: match id.authority() {
                RelationAuthorityV2::Host => ExecutionRelationAuthorityV2::Host,
                RelationAuthorityV2::CanonicalWorkspaceFiles => {
                    ExecutionRelationAuthorityV2::CanonicalWorkspaceFiles
                }
            },
            kind: "relation".to_owned(),
            namespace: Some(id.namespace().as_str().to_owned()),
            name: id.qualified_name().to_owned(),
        },
        ResolvedRelationSourceKindV2::View(id) => ExecutionSourceV2 {
            authority: ExecutionRelationAuthorityV2::CatalogView,
            kind: "view".to_owned(),
            namespace: Some(id.module_id().as_str().to_owned()),
            name: id.local_name().to_owned(),
        },
        ResolvedRelationSourceKindV2::Composition(id) => ExecutionSourceV2 {
            authority: ExecutionRelationAuthorityV2::CompositionBinding,
            kind: "composition".to_owned(),
            namespace: None,
            name: id.binding().canonical_name().to_owned(),
        },
    }
}

fn project_binding(binding: &quanta_gqlang_v2_sema::ResolvedValueBindingV2) -> ExecutionBindingV2 {
    ExecutionBindingV2 {
        source_id: binding.source_id(),
        owner_node: binding.target_node().raw(),
        ordinal: binding.ordinal(),
        name: binding.canonical_name().to_owned(),
    }
}

fn digest_source(source_id: SourceIdV2, items: &[ExecutionItemV2]) -> ExecutionDigestV2 {
    let mut bytes = Vec::new();
    put_str(&mut bytes, "qglang.execution.source.v2");
    put_u32(&mut bytes, source_id.raw());
    for item in items {
        put_item_id(&mut bytes, item.item_id);
        put_str(&mut bytes, item.family.keyword());
        if let Some(relation) = &item.relation {
            put_relation(&mut bytes, relation);
        } else {
            put_u8(&mut bytes, 0);
        }
        if let Some(composition) = &item.composition {
            put_composition(&mut bytes, composition);
        } else {
            put_u8(&mut bytes, 0);
        }
    }
    ExecutionDigestV2(sha256(&bytes))
}

fn digest_document(
    source_id: SourceIdV2,
    source_digest: ExecutionDigestV2,
    items: &[ExecutionItemV2],
) -> ExecutionDigestV2 {
    let mut bytes = Vec::new();
    put_str(&mut bytes, "qglang.execution.v2");
    put_u32(&mut bytes, source_id.raw());
    bytes.extend_from_slice(source_digest.as_bytes());
    put_usize(&mut bytes, items.len());
    for item in items {
        put_item_id(&mut bytes, item.item_id);
        put_str(&mut bytes, item.family.keyword());
        put_str(&mut bytes, authority_keyword(item.plan.authority));
        put_str(&mut bytes, determinism_keyword(item.plan.determinism));
        put_str(&mut bytes, evidence_keyword(item.plan.evidence));
        put_str(&mut bytes, effect_keyword(item.plan.effect));
        put_str(&mut bytes, item.plan.policy_family.keyword());
        if let Some(relation) = &item.relation {
            put_relation(&mut bytes, relation);
        } else {
            put_u8(&mut bytes, 0);
        }
        if let Some(composition) = &item.composition {
            put_composition(&mut bytes, composition);
        } else {
            put_u8(&mut bytes, 0);
        }
    }
    ExecutionDigestV2(sha256(&bytes))
}

fn put_composition(out: &mut Vec<u8>, composition: &ExecutionCompositionV2) {
    put_u8(out, 1);
    put_usize(out, composition.bindings.len());
    for binding in &composition.bindings {
        put_binding(out, &binding.binding);
        put_item_id(out, binding.producer_item_id);
        put_str(out, binding.producer_family.keyword());
        if let Some(schema) = &binding.schema {
            put_u8(out, 1);
            put_schema(out, schema);
        } else {
            put_u8(out, 0);
        }
        if let Some(relation) = &binding.relation {
            put_relation(out, relation);
        } else {
            put_u8(out, 0);
        }
        put_composition_producer(out, &binding.producer);
    }
    put_binding(out, &composition.with_binding);
    put_item_id(out, composition.body_item_id);
    put_str(out, composition.body_family.keyword());
    if let Some(relation) = &composition.body_relation {
        put_relation(out, relation);
    } else {
        put_u8(out, 0);
    }
    put_composition_body(out, &composition.body);
}

fn put_composition_producer(out: &mut Vec<u8>, producer: &ExecutionCompositionProducerV2) {
    match producer {
        ExecutionCompositionProducerV2::Relation => put_u8(out, 0),
        ExecutionCompositionProducerV2::Sample {
            query_text,
            mode,
            limit,
        } => {
            put_u8(out, 1);
            put_str(out, query_text);
            put_u8(
                out,
                match mode {
                    ExecutionSampleModeV2::Lexical => 0,
                    ExecutionSampleModeV2::Semantic => 1,
                    ExecutionSampleModeV2::Hybrid => 2,
                },
            );
            put_u32(out, *limit);
        }
        ExecutionCompositionProducerV2::Graph { kind } => {
            put_u8(out, 2);
            put_u8(out, graph_kind_tag(*kind));
        }
    }
}
fn put_composition_body(out: &mut Vec<u8>, body: &ExecutionCompositionBodyV2) {
    match body {
        ExecutionCompositionBodyV2::Relation => put_u8(out, 0),
        ExecutionCompositionBodyV2::Graph(kind) => {
            put_u8(out, 1);
            put_u8(out, graph_kind_tag(*kind));
        }
        ExecutionCompositionBodyV2::Check => put_u8(out, 2),
        ExecutionCompositionBodyV2::Explain => put_u8(out, 3),
    }
}
const fn graph_kind_tag(kind: ExecutionGraphKindV2) -> u8 {
    match kind {
        ExecutionGraphKindV2::Reachability => 0,
        ExecutionGraphKindV2::Flow => 1,
        ExecutionGraphKindV2::Slice => 2,
        ExecutionGraphKindV2::Taint => 3,
        ExecutionGraphKindV2::Path => 4,
        ExecutionGraphKindV2::PointsTo => 5,
        ExecutionGraphKindV2::Summary => 6,
    }
}

fn put_relation(out: &mut Vec<u8>, relation: &ExecutionRelationV2) {
    put_u8(out, 1);
    put_item_id(out, relation.item_id);
    put_node(out, &relation.root);
    put_usize(out, relation.expressions.len());
    for expr in &relation.expressions {
        put_key(out, expr.id);
        put_str(out, &expr.value_type);
        put_expression_shape(out, &expr.shape);
    }
}
fn put_node(out: &mut Vec<u8>, node: &ExecutionRelationalOperatorV2) {
    put_str(out, &node.family);
    put_key(out, node.stage);
    if let Some(source) = &node.source {
        put_u8(out, 1);
        put_u8(out, relation_authority_tag(source.authority));
        put_str(out, &source.kind);
        put_str(out, source.namespace.as_deref().unwrap_or(""));
        put_str(out, &source.name);
    } else {
        put_u8(out, 0);
    }
    put_schema(out, &node.schema);
    put_usize(out, node.predicates.len());
    for predicate in &node.predicates {
        put_key(out, *predicate);
    }
    put_usize(out, node.expressions.len());
    for key in &node.expressions {
        put_key(out, *key);
    }
    put_usize(out, node.correlations.len());
    for edge in &node.correlations {
        put_binding(out, &edge.binding);
        put_key(out, edge.reference);
    }
    put_usize(out, node.partition_input.len());
    for binding in &node.partition_input {
        put_binding(out, binding);
    }
    put_usize(out, node.children.len());
    for child in &node.children {
        put_node(out, child);
    }
}

const fn relation_authority_tag(authority: ExecutionRelationAuthorityV2) -> u8 {
    match authority {
        ExecutionRelationAuthorityV2::Host => 0,
        ExecutionRelationAuthorityV2::CanonicalWorkspaceFiles => 1,
        ExecutionRelationAuthorityV2::CatalogView => 2,
        ExecutionRelationAuthorityV2::CompositionBinding => 3,
    }
}

fn put_expression_shape(out: &mut Vec<u8>, shape: &ExecutionExpressionShapeV2) {
    match shape {
        ExecutionExpressionShapeV2::Literal(value) => {
            put_u8(out, 0);
            put_str(out, value);
        }
        ExecutionExpressionShapeV2::Binding(binding) => {
            put_u8(out, 1);
            put_binding(out, binding);
        }
        ExecutionExpressionShapeV2::Path { base, segments } => {
            put_u8(out, 2);
            put_key(out, *base);
            put_usize(out, segments.len());
            for segment in segments {
                put_str(out, segment);
            }
        }
        ExecutionExpressionShapeV2::Unary { operator, operand } => {
            put_u8(out, 3);
            put_operator(out, operator);
            put_key(out, *operand);
        }
        ExecutionExpressionShapeV2::Binary { operator, lhs, rhs } => {
            put_u8(out, 4);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV2::Call { target, arguments } => {
            put_u8(out, 5);
            put_str(out, target);
            put_usize(out, arguments.len());
            for argument in arguments {
                if let Some(id) = argument {
                    put_u8(out, 1);
                    put_key(out, *id);
                } else {
                    put_u8(out, 0);
                }
            }
        }
        ExecutionExpressionShapeV2::Paren(id) => {
            put_u8(out, 6);
            put_key(out, *id);
        }
        ExecutionExpressionShapeV2::Record(fields) => {
            put_u8(out, 7);
            put_usize(out, fields.len());
            for (name, id) in fields {
                put_str(out, name);
                put_key(out, *id);
            }
        }
        ExecutionExpressionShapeV2::List(items) => {
            put_u8(out, 8);
            put_usize(out, items.len());
            for id in items {
                put_key(out, *id);
            }
        }
        ExecutionExpressionShapeV2::BoolValue(id) => {
            put_u8(out, 9);
            put_key(out, *id);
        }
        ExecutionExpressionShapeV2::Compare { operator, lhs, rhs } => {
            put_u8(out, 10);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV2::And { operator, lhs, rhs } => {
            put_u8(out, 11);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV2::Or { operator, lhs, rhs } => {
            put_u8(out, 12);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV2::Not { operator, operand } => {
            put_u8(out, 13);
            put_operator(out, operator);
            put_key(out, *operand);
        }
        ExecutionExpressionShapeV2::LogicalQuantifier {
            kind,
            binders,
            body,
        } => {
            put_u8(out, 14);
            put_str(out, kind);
            put_usize(out, binders.len());
            for binding in binders {
                put_binding(out, binding);
            }
            put_key(out, *body);
        }
        ExecutionExpressionShapeV2::RelationExists {
            polarity,
            relation_root,
            correlations,
        } => {
            put_u8(out, 15);
            put_str(out, polarity);
            put_key(out, *relation_root);
            put_usize(out, correlations.len());
            for edge in correlations {
                put_binding(out, &edge.binding);
                put_key(out, edge.reference);
            }
        }
    }
}
fn put_schema(out: &mut Vec<u8>, schema: &ExecutionSchemaV2) {
    put_usize(out, schema.fields.len());
    for field in &schema.fields {
        put_str(out, &field.name);
        put_str(out, &field.value_type);
    }
    put_str(out, &schema.cardinality);
    put_usize(out, schema.partition_keys.len());
    for key in &schema.partition_keys {
        put_key(out, *key);
    }
    put_usize(out, schema.order.len());
    for key in &schema.order {
        put_key(out, key.expression);
        put_str(out, &key.direction);
        put_str(out, &key.null_order);
        put_str(out, &key.value_type);
    }
}
fn put_binding(out: &mut Vec<u8>, binding: &ExecutionBindingV2) {
    put_u32(out, binding.source_id.raw());
    put_u32(out, binding.owner_node);
    put_usize(out, binding.ordinal);
    put_str(out, &binding.name);
}
fn put_operator(out: &mut Vec<u8>, operator: &ExecutionOperatorV2) {
    put_str(out, &operator.symbol);
    put_u32(out, operator.overload_id);
}
fn put_item_id(out: &mut Vec<u8>, id: SemanticItemIdV2) {
    put_u32(out, id.source_id().raw());
    put_u32(out, id.hir_item_id().raw());
}
fn put_key(out: &mut Vec<u8>, key: SourceQualifiedNodeIdV2) {
    put_u32(out, key.source_id.raw());
    put_u32(out, key.node_id.raw());
}
fn put_str(out: &mut Vec<u8>, value: &str) {
    put_usize(out, value.len());
    out.extend_from_slice(value.as_bytes());
}
fn put_usize(out: &mut Vec<u8>, value: usize) {
    out.extend_from_slice(&(value as u64).to_be_bytes());
}
fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}
fn put_u8(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}

// Small dependency-free SHA-256 keeps this DTO owner from adding a workspace or lockfile edge.
fn sha256(input: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut data = input.to_vec();
    let bit_len = (data.len() as u64) * 8;
    data.push(0x80);
    while data.len() % 64 != 56 {
        data.push(0);
    }
    data.extend_from_slice(&bit_len.to_be_bytes());
    let mut h = [
        0x6a09e667u32,
        0xbb67ae85,
        0x3c6ef372,
        0xa54ff53a,
        0x510e527f,
        0x9b05688c,
        0x1f83d9ab,
        0x5be0cd19,
    ];
    for chunk in data.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in w[..16].iter_mut().enumerate() {
            *word = u32::from_be_bytes(chunk[i * 4..i * 4 + 4].try_into().expect("four bytes"));
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (state, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *state = state.wrapping_add(value);
        }
    }
    let mut out = [0u8; 32];
    for (i, value) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&value.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sha256_matches_published_empty_vector() {
        assert_eq!(
            ExecutionDigestV2(sha256(b"")).to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
