//! Host-agnostic, immutable execution contract projected from sealed V2 owners.
//!
//! This module copies only public semantic facts into a closed transport DTO.
//! Closed vocabularies are execution-local enums. The only reused owner types
//! are the inert identity closure (`SourceIdV2`, `SemanticItemIdV2`,
//! `SourceQualifiedNodeIdV2`) and [`AlgebraFamilyV2`], all already part of the
//! curated facade closure. Every other HIR, Sema, Algebra, Planning, and Types
//! owner type stays unreachable.

use quanta_gqlang_v2_algebra::{AlgebraFamilyV2, CheckedAlgebraNodeV2, LoweredRelationItemV2};
use quanta_gqlang_v2_planning::{
    AuthorityLaneV2, EffectClassV2, EvaluationDeterminismV2, EvidenceRequirementV2, PlanPolicyV2,
    SemanticPlanV2,
};
use quanta_gqlang_v2_sema::{
    CardinalityV1, ExpressionTypeStateV2, GraphQueryKindV1, JoinModeV2, LogicalQuantifierKindV2,
    NullOrderV2, RelationAuthorityV2, RelationExistsPolarityV2, RelationOutputContractV2,
    ResolvedCallTargetV2, ResolvedCompositionBodyV2, ResolvedCompositionProducerV2,
    ResolvedCompositionV2, ResolvedPresentationKindV2, ResolvedRelationSourceKindV2, SampleModeV1,
    ScalarTyV1, SemanticDocumentV2, SemanticItemIdV2, SortDirectionV2, SourceIdV2,
    SourceQualifiedNodeIdV2, TypedCallArgumentV2, TypedExprShapeV2, TypedExprV2,
    TypedLiteralFactV2, TypedLiteralValueV2, ValueTypeV1,
};
use sha2::{Digest as _, Sha256};

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
/// Three digests separate three concerns: [`ExecutionDocumentV2::source_digest`]
/// covers the raw UTF-8 source bytes only, [`ExecutionDocumentV2::contract_digest`]
/// covers the canonical DTO encoding only, and
/// [`ExecutionDocumentV2::qualification_digest`] binds the two together under the
/// contract domain and schema version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionDocumentV2 {
    source_id: SourceIdV2,
    items: Box<[ExecutionItemV2]>,
    source_digest: ExecutionDigestV2,
    contract_digest: ExecutionDigestV2,
    qualification_digest: ExecutionDigestV2,
}

impl ExecutionDocumentV2 {
    pub(crate) fn new(source_id: SourceIdV2, items: Vec<ExecutionItemV2>) -> Self {
        let contract_digest = digest_contract(source_id, &items);
        // Source bytes arrive later through `bind_source_bytes` (the resolved
        // entry point has no bytes at all); until then the source leg binds the
        // empty input so every digest is always defined.
        let source_digest = digest_source_bytes(&[]);
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

    /// Digest of the raw UTF-8 source bytes only. DTO changes never move it.
    #[must_use]
    pub const fn source_digest(&self) -> ExecutionDigestV2 {
        self.source_digest
    }

    /// Digest of the canonical DTO encoding only. Source-byte changes that do
    /// not alter the projected contract never move it.
    #[must_use]
    pub const fn contract_digest(&self) -> ExecutionDigestV2 {
        self.contract_digest
    }

    /// Digest binding the contract domain, the schema version, and both legs
    /// above. Any DTO or source-byte change moves it.
    #[must_use]
    pub const fn qualification_digest(&self) -> ExecutionDigestV2 {
        self.qualification_digest
    }

    pub(crate) fn bind_source_bytes(&mut self, source: &[u8]) {
        self.source_digest = digest_source_bytes(source);
        self.qualification_digest =
            digest_qualification(self.source_id, self.source_digest, self.contract_digest);
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

/// Frontend-checked policy copied from the sealed semantic plan. This is pure
/// frontend vocabulary: no runtime, provider, or backend routing appears here.
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

/// Closed scalar vocabulary projected from the sealed type owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionScalarTypeV2 {
    Bool,
    Int,
    Float,
    String,
    Duration,
    Location,
    Path,
    RepoPath,
    EntityRef,
    SymbolRef,
    RuleRef,
    SourceScope,
    ProofTree,
}

const fn project_scalar(scalar: &ScalarTyV1) -> ExecutionScalarTypeV2 {
    match scalar {
        ScalarTyV1::Bool => ExecutionScalarTypeV2::Bool,
        ScalarTyV1::Int => ExecutionScalarTypeV2::Int,
        ScalarTyV1::Float => ExecutionScalarTypeV2::Float,
        ScalarTyV1::String => ExecutionScalarTypeV2::String,
        ScalarTyV1::Duration => ExecutionScalarTypeV2::Duration,
        ScalarTyV1::Location => ExecutionScalarTypeV2::Location,
        ScalarTyV1::Path => ExecutionScalarTypeV2::Path,
        ScalarTyV1::RepoPath => ExecutionScalarTypeV2::RepoPath,
        ScalarTyV1::EntityRef => ExecutionScalarTypeV2::EntityRef,
        ScalarTyV1::SymbolRef => ExecutionScalarTypeV2::SymbolRef,
        ScalarTyV1::RuleRef => ExecutionScalarTypeV2::RuleRef,
        ScalarTyV1::SourceScope => ExecutionScalarTypeV2::SourceScope,
        ScalarTyV1::ProofTree => ExecutionScalarTypeV2::ProofTree,
    }
}

/// Closed value-type vocabulary. Only record field names and union tags are
/// open strings; every structural kind is an enum variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionValueTypeV2 {
    Scalar(ExecutionScalarTypeV2),
    Null,
    Tuple(Box<[ExecutionValueTypeV2]>),
    Record(Box<[ExecutionRecordFieldV2]>),
    Optional(Box<ExecutionValueTypeV2>),
    Collection {
        element: Box<ExecutionValueTypeV2>,
        cardinality: ExecutionCardinalityV2,
    },
    Union(Box<[ExecutionUnionVariantV2]>),
    /// The checked expression carries no known type. The unknown cause is
    /// editor-specific, so it is excluded from the contract by design: two
    /// unknowns are indistinguishable here however they arose. Schemas never
    /// project this variant: row and order types are always known.
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRecordFieldV2 {
    name: String,
    value_type: ExecutionValueTypeV2,
}
impl ExecutionRecordFieldV2 {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn value_type(&self) -> &ExecutionValueTypeV2 {
        &self.value_type
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionUnionVariantV2 {
    tag: String,
    value_type: ExecutionValueTypeV2,
}
impl ExecutionUnionVariantV2 {
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }
    #[must_use]
    pub const fn value_type(&self) -> &ExecutionValueTypeV2 {
        &self.value_type
    }
}

fn project_value_type(value: &ValueTypeV1) -> ExecutionValueTypeV2 {
    if let Some(scalar) = value.as_type().as_scalar() {
        return ExecutionValueTypeV2::Scalar(project_scalar(scalar));
    }
    if value.is_null() {
        return ExecutionValueTypeV2::Null;
    }
    if let Some(tuple) = value.as_tuple() {
        return ExecutionValueTypeV2::Tuple(
            tuple
                .items()
                .iter()
                .map(project_value_type)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
    }
    if let Some(record) = value.as_record() {
        return ExecutionValueTypeV2::Record(
            record
                .fields()
                .iter()
                .map(|field| ExecutionRecordFieldV2 {
                    name: field.name().to_owned(),
                    value_type: project_value_type(field.value_type()),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
    }
    if let Some(optional) = value.as_optional() {
        return ExecutionValueTypeV2::Optional(Box::new(project_value_type(optional.inner())));
    }
    if let Some(collection) = value.as_collection() {
        return ExecutionValueTypeV2::Collection {
            element: Box::new(project_value_type(collection.element())),
            cardinality: project_cardinality(collection.cardinality()),
        };
    }
    if let Some(union) = value.as_union() {
        return ExecutionValueTypeV2::Union(
            union
                .variants()
                .iter()
                .map(|variant| ExecutionUnionVariantV2 {
                    tag: variant.tag().to_owned(),
                    value_type: project_value_type(variant.value_type()),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
    }
    // `ValueTypeV1` construction rejects relation kinds, and every other kind
    // is matched above; the unreachable remainder fails closed as Unknown so
    // the projection stays total even if the owner grows a kind.
    ExecutionValueTypeV2::Unknown
}

/// Closed row/collection cardinality vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionCardinalityV2 {
    Set,
    Bag,
    Sequence,
}

const fn project_cardinality(value: CardinalityV1) -> ExecutionCardinalityV2 {
    match value {
        CardinalityV1::Set => ExecutionCardinalityV2::Set,
        CardinalityV1::Bag => ExecutionCardinalityV2::Bag,
        CardinalityV1::Sequence => ExecutionCardinalityV2::Sequence,
    }
}

/// Closed sort-direction vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionSortDirectionV2 {
    Asc,
    Desc,
}

const fn project_sort_direction(value: SortDirectionV2) -> ExecutionSortDirectionV2 {
    match value {
        SortDirectionV2::Asc => ExecutionSortDirectionV2::Asc,
        SortDirectionV2::Desc => ExecutionSortDirectionV2::Desc,
    }
}

/// Closed null-ordering vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionNullOrderV2 {
    First,
    Last,
}

const fn project_null_order(value: NullOrderV2) -> ExecutionNullOrderV2 {
    match value {
        NullOrderV2::First => ExecutionNullOrderV2::First,
        NullOrderV2::Last => ExecutionNullOrderV2::Last,
    }
}

/// Closed join-mode vocabulary. Only `Join` operators carry a mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionJoinModeV2 {
    Inner,
    Cross,
}

const fn project_join_mode(value: JoinModeV2) -> ExecutionJoinModeV2 {
    match value {
        JoinModeV2::Inner => ExecutionJoinModeV2::Inner,
        JoinModeV2::Cross => ExecutionJoinModeV2::Cross,
    }
}

/// Closed scan-source-kind vocabulary. Names and namespaces stay open strings
/// on [`ExecutionSourceV2`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionSourceKindV2 {
    Relation,
    View,
    Composition,
}

/// Closed call-target vocabulary. Only the callee identity is an open string;
/// the target class is an enum variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionCallTargetV2 {
    Builtin { name: String },
    Predicate { name: String },
    HostPredicate { fingerprint: String },
}

fn project_call_target(target: &ResolvedCallTargetV2) -> ExecutionCallTargetV2 {
    match target {
        ResolvedCallTargetV2::Builtin(id) => ExecutionCallTargetV2::Builtin {
            name: id.name().unwrap_or("invalid_closed_builtin").to_owned(),
        },
        ResolvedCallTargetV2::Predicate(id) => ExecutionCallTargetV2::Predicate {
            name: id.as_str().to_owned(),
        },
        ResolvedCallTargetV2::HostPredicate(id) => ExecutionCallTargetV2::HostPredicate {
            fingerprint: hex_lower(id.canonical_bytes()),
        },
    }
}

/// Closed logical-quantifier vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionLogicalQuantifierV2 {
    Exists,
    Forall,
}

const fn project_logical_quantifier(
    value: LogicalQuantifierKindV2,
) -> ExecutionLogicalQuantifierV2 {
    match value {
        LogicalQuantifierKindV2::Exists => ExecutionLogicalQuantifierV2::Exists,
        LogicalQuantifierKindV2::Forall => ExecutionLogicalQuantifierV2::Forall,
    }
}

/// Closed relational-existence polarity vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionExistsPolarityV2 {
    Exists,
    NotExists,
}

const fn project_exists_polarity(value: RelationExistsPolarityV2) -> ExecutionExistsPolarityV2 {
    match value {
        RelationExistsPolarityV2::Exists => ExecutionExistsPolarityV2::Exists,
        RelationExistsPolarityV2::NotExists => ExecutionExistsPolarityV2::NotExists,
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
    family: AlgebraFamilyV2,
    join_mode: Option<ExecutionJoinModeV2>,
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
    pub const fn family(&self) -> AlgebraFamilyV2 {
        self.family
    }
    /// Join mode for `Join` operators; `None` for every other family.
    #[must_use]
    pub const fn join_mode(&self) -> Option<ExecutionJoinModeV2> {
        self.join_mode
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
    cardinality: ExecutionCardinalityV2,
    partition_keys: Box<[SourceQualifiedNodeIdV2]>,
    order: Box<[ExecutionOrderKeyV2]>,
}
impl ExecutionSchemaV2 {
    #[must_use]
    pub fn fields(&self) -> &[ExecutionFieldV2] {
        &self.fields
    }
    #[must_use]
    pub const fn cardinality(&self) -> ExecutionCardinalityV2 {
        self.cardinality
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
    value_type: ExecutionValueTypeV2,
}
impl ExecutionFieldV2 {
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn value_type(&self) -> &ExecutionValueTypeV2 {
        &self.value_type
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionOrderKeyV2 {
    expression: SourceQualifiedNodeIdV2,
    direction: ExecutionSortDirectionV2,
    null_order: ExecutionNullOrderV2,
    value_type: ExecutionValueTypeV2,
}
impl ExecutionOrderKeyV2 {
    #[must_use]
    pub const fn expression(&self) -> SourceQualifiedNodeIdV2 {
        self.expression
    }
    #[must_use]
    pub const fn direction(&self) -> ExecutionSortDirectionV2 {
        self.direction
    }
    #[must_use]
    pub const fn null_order(&self) -> ExecutionNullOrderV2 {
        self.null_order
    }
    #[must_use]
    pub const fn value_type(&self) -> &ExecutionValueTypeV2 {
        &self.value_type
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionExpressionV2 {
    id: SourceQualifiedNodeIdV2,
    value_type: ExecutionValueTypeV2,
    shape: ExecutionExpressionShapeV2,
}
impl ExecutionExpressionV2 {
    #[must_use]
    pub const fn id(&self) -> SourceQualifiedNodeIdV2 {
        self.id
    }
    #[must_use]
    pub const fn value_type(&self) -> &ExecutionValueTypeV2 {
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
        target: ExecutionCallTargetV2,
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
        kind: ExecutionLogicalQuantifierV2,
        binders: Box<[ExecutionBindingV2]>,
        body: SourceQualifiedNodeIdV2,
    },
    RelationExists {
        polarity: ExecutionExistsPolarityV2,
        relation_root: SourceQualifiedNodeIdV2,
        correlations: Box<[ExecutionCorrelationV2]>,
    },
}

/// Public semantic operator: the canonical overload identity from the closed
/// registry plus its stable spelling witness. The `overload_id` is the stable
/// identity; `symbol` renders it and is never parsed back.
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
    kind: ExecutionSourceKindV2,
    namespace: Option<String>,
    name: String,
}
impl ExecutionSourceV2 {
    #[must_use]
    pub const fn authority(&self) -> ExecutionRelationAuthorityV2 {
        self.authority
    }
    #[must_use]
    pub const fn kind(&self) -> ExecutionSourceKindV2 {
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

/// Expression-closure failure: the item root references an expression the
/// document table cannot supply from the item's own source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExecutionClosureFailureV2 {
    /// A referenced expression identity is absent from the document table.
    Dangling,
    /// A referenced expression identity belongs to another source.
    Foreign,
}

pub(crate) fn project_relation(
    document: &SemanticDocumentV2,
    lowered: &LoweredRelationItemV2,
) -> Result<ExecutionRelationV2, ExecutionClosureFailureV2> {
    project_relation_root(document, lowered.item_id(), lowered.root())
}

pub(crate) fn project_relation_root(
    document: &SemanticDocumentV2,
    item_id: SemanticItemIdV2,
    root: &CheckedAlgebraNodeV2,
) -> Result<ExecutionRelationV2, ExecutionClosureFailureV2> {
    let projected = project_node(root);
    let mut roots = Vec::new();
    collect_operator_roots(&projected, &mut roots);
    let closed = close_expression_ids(item_id.source_id(), &roots, &|key| {
        document
            .expression(key)
            .map(|expr| shape_children(expr.shape()))
    })?;
    let mut table = Vec::with_capacity(closed.len());
    for key in closed {
        // `close_expression_ids` resolved every key through this same table.
        let Some(expr) = document.expression(key) else {
            return Err(ExecutionClosureFailureV2::Dangling);
        };
        table.push(project_expression(expr));
    }
    Ok(ExecutionRelationV2 {
        item_id,
        root: projected,
        expressions: table.into_boxed_slice(),
    })
}

/// Every expression identity the operator tree references directly: predicate
/// and projection keys, correlation references, schema partition keys, and
/// order-key expressions, recursively through children.
fn collect_operator_roots(
    node: &ExecutionRelationalOperatorV2,
    roots: &mut Vec<SourceQualifiedNodeIdV2>,
) {
    roots.extend_from_slice(&node.predicates);
    roots.extend_from_slice(&node.expressions);
    for edge in &node.correlations {
        roots.push(edge.reference);
    }
    roots.extend_from_slice(&node.schema.partition_keys);
    for key in &node.schema.order {
        roots.push(key.expression);
    }
    for child in &node.children {
        collect_operator_roots(child, roots);
    }
}

/// Child expression identities of one checked shape. Binding shapes carry no
/// expression children; the `RelationExists` relation root is a relation key,
/// not an expression identity, so only its correlation references close.
fn shape_children(shape: &TypedExprShapeV2) -> Vec<SourceQualifiedNodeIdV2> {
    match shape {
        TypedExprShapeV2::Literal(_) | TypedExprShapeV2::Binding(_) => Vec::new(),
        TypedExprShapeV2::Path { base, .. } => vec![*base],
        TypedExprShapeV2::Unary { operand, .. } | TypedExprShapeV2::Not { operand, .. } => {
            vec![*operand]
        }
        TypedExprShapeV2::Binary { lhs, rhs, .. }
        | TypedExprShapeV2::Compare { lhs, rhs, .. }
        | TypedExprShapeV2::And { lhs, rhs, .. }
        | TypedExprShapeV2::Or { lhs, rhs, .. } => vec![*lhs, *rhs],
        TypedExprShapeV2::Call { arguments, .. } => arguments
            .iter()
            .filter_map(|argument| match argument {
                TypedCallArgumentV2::Expression(id) => Some(*id),
                TypedCallArgumentV2::Star => None,
            })
            .collect(),
        TypedExprShapeV2::Paren { inner } | TypedExprShapeV2::BoolValue { value: inner } => {
            vec![*inner]
        }
        TypedExprShapeV2::Record(fields) => fields.iter().map(|field| field.value()).collect(),
        TypedExprShapeV2::List(items) => items.to_vec(),
        TypedExprShapeV2::LogicalQuantifier { body, .. } => vec![*body],
        TypedExprShapeV2::RelationExists { correlations, .. } => {
            correlations.iter().map(|edge| edge.reference()).collect()
        }
    }
}

/// Transitively close expression identities over a caller-supplied child
/// lookup. The result is deduplicated and sorted in stable identity order, so
/// unrelated table rows never affect an item's contract. A dangling identity
/// or an identity from another source fails closed.
fn close_expression_ids(
    source_id: SourceIdV2,
    roots: &[SourceQualifiedNodeIdV2],
    children_of: &dyn Fn(SourceQualifiedNodeIdV2) -> Option<Vec<SourceQualifiedNodeIdV2>>,
) -> Result<Vec<SourceQualifiedNodeIdV2>, ExecutionClosureFailureV2> {
    let mut closed = std::collections::BTreeSet::new();
    let mut stack: Vec<SourceQualifiedNodeIdV2> = roots.to_vec();
    while let Some(key) = stack.pop() {
        if key.source_id != source_id {
            return Err(ExecutionClosureFailureV2::Foreign);
        }
        if !closed.insert(key) {
            continue;
        }
        let Some(children) = children_of(key) else {
            return Err(ExecutionClosureFailureV2::Dangling);
        };
        stack.extend(children);
    }
    Ok(closed.into_iter().collect())
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
    ExecutionRelationalOperatorV2 {
        family: node.family(),
        join_mode: node.join_mode().map(project_join_mode),
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
        ExpressionTypeStateV2::Known(value) => project_value_type(value),
        ExpressionTypeStateV2::Unknown(_) => ExecutionValueTypeV2::Unknown,
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
            target: project_call_target(target),
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
            kind: project_logical_quantifier(*kind),
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
            polarity: project_exists_polarity(*polarity),
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
            value_type: project_value_type(field.value_type()),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let cardinality = project_cardinality(output.cardinality());
    let order = output
        .ordering()
        .iter()
        .map(|key| ExecutionOrderKeyV2 {
            expression: key.expression(),
            direction: project_sort_direction(key.direction()),
            null_order: project_null_order(key.null_order()),
            value_type: project_value_type(key.value_type()),
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
            kind: ExecutionSourceKindV2::Relation,
            namespace: Some(id.namespace().as_str().to_owned()),
            name: id.qualified_name().to_owned(),
        },
        ResolvedRelationSourceKindV2::View(id) => ExecutionSourceV2 {
            authority: ExecutionRelationAuthorityV2::CatalogView,
            kind: ExecutionSourceKindV2::View,
            namespace: Some(id.module_id().as_str().to_owned()),
            name: id.local_name().to_owned(),
        },
        ResolvedRelationSourceKindV2::Composition(id) => ExecutionSourceV2 {
            authority: ExecutionRelationAuthorityV2::CompositionBinding,
            kind: ExecutionSourceKindV2::Composition,
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

/// Canonical encoding rules (all three legs): domain-separated schema
/// version first, big-endian integers, length-prefixed strings and sequences,
/// one fixed tag byte per enum variant, every DTO field exactly once, and
/// explicitly ordered collections (source order, identity order, or
/// owner-stable order — never hash order). Derived display witnesses (operator
/// spellings) are encoded deliberately so the audit stays syntactic — one
/// `put_*` call per field, no semantic filtering; cross-version drift is gated
/// by the schema version instead.
const CONTRACT_SCHEMA_VERSION_V2: u32 = 2;

/// Digest of the raw UTF-8 source bytes only.
fn digest_source_bytes(source: &[u8]) -> ExecutionDigestV2 {
    let mut bytes = Vec::new();
    put_str(&mut bytes, "qglang.execution.source-bytes.v2");
    put_usize(&mut bytes, source.len());
    bytes.extend_from_slice(source);
    ExecutionDigestV2(sha256(&bytes))
}

/// Digest of the canonical DTO encoding only.
fn digest_contract(source_id: SourceIdV2, items: &[ExecutionItemV2]) -> ExecutionDigestV2 {
    let mut bytes = Vec::new();
    put_str(&mut bytes, "qglang.execution.contract.v2");
    put_u32(&mut bytes, CONTRACT_SCHEMA_VERSION_V2);
    put_u32(&mut bytes, source_id.raw());
    put_usize(&mut bytes, items.len());
    for item in items {
        put_item_id(&mut bytes, item.item_id);
        put_u8(&mut bytes, family_tag(item.family));
        put_u8(&mut bytes, authority_tag(item.plan.authority));
        put_u8(&mut bytes, determinism_tag(item.plan.determinism));
        put_u8(&mut bytes, evidence_tag(item.plan.evidence));
        put_u8(&mut bytes, effect_tag(item.plan.effect));
        put_u8(&mut bytes, policy_family_tag(item.plan.policy_family));
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

/// Digest binding the contract domain, the schema version, and both legs.
fn digest_qualification(
    source_id: SourceIdV2,
    source_digest: ExecutionDigestV2,
    contract_digest: ExecutionDigestV2,
) -> ExecutionDigestV2 {
    let mut bytes = Vec::new();
    put_str(&mut bytes, "qglang.execution.qualification.v2");
    put_u32(&mut bytes, CONTRACT_SCHEMA_VERSION_V2);
    put_u32(&mut bytes, source_id.raw());
    bytes.extend_from_slice(source_digest.as_bytes());
    bytes.extend_from_slice(contract_digest.as_bytes());
    ExecutionDigestV2(sha256(&bytes))
}

const fn family_tag(value: ExecutionFamilyV2) -> u8 {
    match value {
        ExecutionFamilyV2::Relation => 0,
        ExecutionFamilyV2::Graph => 1,
        ExecutionFamilyV2::Check => 2,
        ExecutionFamilyV2::Presentation => 3,
        ExecutionFamilyV2::Composition => 4,
        ExecutionFamilyV2::MetaRequireCertified => 5,
        ExecutionFamilyV2::RuleProgram => 6,
    }
}

const fn authority_tag(value: ExecutionAuthorityV2) -> u8 {
    match value {
        ExecutionAuthorityV2::FrontendChecked => 0,
        ExecutionAuthorityV2::CertifiedRequirementValidated => 1,
    }
}

const fn determinism_tag(value: ExecutionDeterminismV2) -> u8 {
    match value {
        ExecutionDeterminismV2::Deterministic => 0,
        ExecutionDeterminismV2::NonDeterministic => 1,
    }
}

const fn evidence_tag(value: ExecutionEvidenceV2) -> u8 {
    match value {
        ExecutionEvidenceV2::None => 0,
        ExecutionEvidenceV2::SourceLocation => 1,
        ExecutionEvidenceV2::WitnessPayload => 2,
        ExecutionEvidenceV2::ProofTree => 3,
    }
}

const fn effect_tag(value: ExecutionEffectV2) -> u8 {
    match value {
        ExecutionEffectV2::ReadOnly => 0,
        ExecutionEffectV2::ProofEvidenceOnly => 1,
        ExecutionEffectV2::ExportOnly => 2,
        ExecutionEffectV2::MutationForbidden => 3,
    }
}

const fn policy_family_tag(value: ExecutionPolicyFamilyV2) -> u8 {
    match value {
        ExecutionPolicyFamilyV2::Relational => 0,
        ExecutionPolicyFamilyV2::Graph => 1,
        ExecutionPolicyFamilyV2::Check => 2,
        ExecutionPolicyFamilyV2::Presentation => 3,
        ExecutionPolicyFamilyV2::RuleProgram => 4,
        ExecutionPolicyFamilyV2::MetaRequireCertified => 5,
        ExecutionPolicyFamilyV2::Composition => 6,
    }
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
        put_value_type(out, &expr.value_type);
        put_expression_shape(out, &expr.shape);
    }
}
fn put_node(out: &mut Vec<u8>, node: &ExecutionRelationalOperatorV2) {
    put_u8(out, algebra_family_tag(node.family));
    match node.join_mode {
        Some(mode) => {
            put_u8(out, 1);
            put_u8(out, join_mode_tag(mode));
        }
        None => put_u8(out, 0),
    }
    put_key(out, node.stage);
    if let Some(source) = &node.source {
        put_u8(out, 1);
        put_u8(out, relation_authority_tag(source.authority));
        put_u8(out, source_kind_tag(source.kind));
        match &source.namespace {
            Some(namespace) => {
                put_u8(out, 1);
                put_str(out, namespace);
            }
            None => put_u8(out, 0),
        }
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
            put_call_target(out, target);
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
            put_u8(out, logical_quantifier_tag(*kind));
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
            put_u8(out, exists_polarity_tag(*polarity));
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
        put_value_type(out, &field.value_type);
    }
    put_u8(out, cardinality_tag(schema.cardinality));
    put_usize(out, schema.partition_keys.len());
    for key in &schema.partition_keys {
        put_key(out, *key);
    }
    put_usize(out, schema.order.len());
    for key in &schema.order {
        put_key(out, key.expression);
        put_u8(out, sort_direction_tag(key.direction));
        put_u8(out, null_order_tag(key.null_order));
        put_value_type(out, &key.value_type);
    }
}

fn put_value_type(out: &mut Vec<u8>, value: &ExecutionValueTypeV2) {
    match value {
        ExecutionValueTypeV2::Scalar(scalar) => {
            put_u8(out, 0);
            put_u8(out, scalar_tag(*scalar));
        }
        ExecutionValueTypeV2::Null => put_u8(out, 1),
        ExecutionValueTypeV2::Tuple(items) => {
            put_u8(out, 2);
            put_usize(out, items.len());
            for item in items {
                put_value_type(out, item);
            }
        }
        ExecutionValueTypeV2::Record(fields) => {
            put_u8(out, 3);
            put_usize(out, fields.len());
            for field in fields {
                put_str(out, &field.name);
                put_value_type(out, &field.value_type);
            }
        }
        ExecutionValueTypeV2::Optional(inner) => {
            put_u8(out, 4);
            put_value_type(out, inner);
        }
        ExecutionValueTypeV2::Collection {
            element,
            cardinality,
        } => {
            put_u8(out, 5);
            put_value_type(out, element);
            put_u8(out, cardinality_tag(*cardinality));
        }
        ExecutionValueTypeV2::Union(variants) => {
            put_u8(out, 6);
            put_usize(out, variants.len());
            for variant in variants {
                put_str(out, &variant.tag);
                put_value_type(out, &variant.value_type);
            }
        }
        ExecutionValueTypeV2::Unknown => put_u8(out, 7),
    }
}

const fn scalar_tag(value: ExecutionScalarTypeV2) -> u8 {
    match value {
        ExecutionScalarTypeV2::Bool => 0,
        ExecutionScalarTypeV2::Int => 1,
        ExecutionScalarTypeV2::Float => 2,
        ExecutionScalarTypeV2::String => 3,
        ExecutionScalarTypeV2::Duration => 4,
        ExecutionScalarTypeV2::Location => 5,
        ExecutionScalarTypeV2::Path => 6,
        ExecutionScalarTypeV2::RepoPath => 7,
        ExecutionScalarTypeV2::EntityRef => 8,
        ExecutionScalarTypeV2::SymbolRef => 9,
        ExecutionScalarTypeV2::RuleRef => 10,
        ExecutionScalarTypeV2::SourceScope => 11,
        ExecutionScalarTypeV2::ProofTree => 12,
    }
}

const fn cardinality_tag(value: ExecutionCardinalityV2) -> u8 {
    match value {
        ExecutionCardinalityV2::Set => 0,
        ExecutionCardinalityV2::Bag => 1,
        ExecutionCardinalityV2::Sequence => 2,
    }
}

const fn sort_direction_tag(value: ExecutionSortDirectionV2) -> u8 {
    match value {
        ExecutionSortDirectionV2::Asc => 0,
        ExecutionSortDirectionV2::Desc => 1,
    }
}

const fn null_order_tag(value: ExecutionNullOrderV2) -> u8 {
    match value {
        ExecutionNullOrderV2::First => 0,
        ExecutionNullOrderV2::Last => 1,
    }
}

const fn algebra_family_tag(value: AlgebraFamilyV2) -> u8 {
    match value {
        AlgebraFamilyV2::Scan => 0,
        AlgebraFamilyV2::Filter => 1,
        AlgebraFamilyV2::Project => 2,
        AlgebraFamilyV2::Join => 3,
        AlgebraFamilyV2::Group => 4,
        AlgebraFamilyV2::Aggregate => 5,
        AlgebraFamilyV2::Sort => 6,
        AlgebraFamilyV2::SemiJoin => 7,
        AlgebraFamilyV2::AntiJoin => 8,
    }
}

const fn join_mode_tag(value: ExecutionJoinModeV2) -> u8 {
    match value {
        ExecutionJoinModeV2::Inner => 0,
        ExecutionJoinModeV2::Cross => 1,
    }
}

const fn source_kind_tag(value: ExecutionSourceKindV2) -> u8 {
    match value {
        ExecutionSourceKindV2::Relation => 0,
        ExecutionSourceKindV2::View => 1,
        ExecutionSourceKindV2::Composition => 2,
    }
}

fn put_call_target(out: &mut Vec<u8>, target: &ExecutionCallTargetV2) {
    match target {
        ExecutionCallTargetV2::Builtin { name } => {
            put_u8(out, 0);
            put_str(out, name);
        }
        ExecutionCallTargetV2::Predicate { name } => {
            put_u8(out, 1);
            put_str(out, name);
        }
        ExecutionCallTargetV2::HostPredicate { fingerprint } => {
            put_u8(out, 2);
            put_str(out, fingerprint);
        }
    }
}

const fn logical_quantifier_tag(value: ExecutionLogicalQuantifierV2) -> u8 {
    match value {
        ExecutionLogicalQuantifierV2::Exists => 0,
        ExecutionLogicalQuantifierV2::Forall => 1,
    }
}

const fn exists_polarity_tag(value: ExecutionExistsPolarityV2) -> u8 {
    match value {
        ExecutionExistsPolarityV2::Exists => 0,
        ExecutionExistsPolarityV2::NotExists => 1,
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
    // Total conversion: lengths beyond `u64::MAX` saturate (unreachable on any
    // real target; the digest writer must stay total regardless of platform).
    out.extend_from_slice(&u64::try_from(value).unwrap_or(u64::MAX).to_be_bytes());
}
fn put_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}
fn put_u8(out: &mut Vec<u8>, value: u8) {
    out.push(value);
}

/// SHA-256 over the canonical encoding via the audited `sha2` crate. No
/// hand-rolled cryptography lives in this module.
fn sha256(input: &[u8]) -> [u8; 32] {
    let digest = Sha256::digest(input);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "contract digest mutation rails assert exact fixture shapes"
)]
mod tests {
    use super::*;
    use quanta_gqlang_v2_sema::HirNodeIdV1;

    #[test]
    fn sha256_matches_published_empty_vector() {
        assert_eq!(
            ExecutionDigestV2(sha256(b"")).to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn canonical_digests_match_the_pinned_golden_vectors() {
        let mut document = ExecutionDocumentV2::new(SourceIdV2::from_raw(0), vec![relation_item()]);
        document.bind_source_bytes(b"show models");
        // Golden pins: any canonical-encoding change (tag swap, field reorder,
        // domain edit) must update these deliberately, never silently.
        assert_eq!(
            document.source_digest().to_hex(),
            "c8b5afd3b89c8fed8d82c41761232bbbd3929a8f9b590ba18c5dea1633f82dac"
        );
        assert_eq!(
            document.contract_digest().to_hex(),
            "4306ff6c24b462c8b4a06152f1441cf5531cbb0c8c9268d383d4011b2d2058a3"
        );
        assert_eq!(
            document.qualification_digest().to_hex(),
            "f5fc6b4fb8223ea087c0cc2c5182cc3faea7225c49a00a97bd6152ca30ce20f2"
        );
    }

    fn assert_distinct_tags<const N: usize>(tags: [u8; N]) {
        let mut sorted = tags.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), N, "duplicate canonical tag in {tags:?}");
    }

    #[test]
    fn every_canonical_tag_set_is_duplicate_free() {
        use ExecutionAuthorityV2 as Authority;
        use ExecutionDeterminismV2 as Determinism;
        use ExecutionEffectV2 as Effect;
        use ExecutionEvidenceV2 as Evidence;
        assert_distinct_tags([
            family_tag(ExecutionFamilyV2::Relation),
            family_tag(ExecutionFamilyV2::Graph),
            family_tag(ExecutionFamilyV2::Check),
            family_tag(ExecutionFamilyV2::Presentation),
            family_tag(ExecutionFamilyV2::Composition),
            family_tag(ExecutionFamilyV2::MetaRequireCertified),
            family_tag(ExecutionFamilyV2::RuleProgram),
        ]);
        assert_distinct_tags([
            authority_tag(Authority::FrontendChecked),
            authority_tag(Authority::CertifiedRequirementValidated),
        ]);
        assert_distinct_tags([
            determinism_tag(Determinism::Deterministic),
            determinism_tag(Determinism::NonDeterministic),
        ]);
        assert_distinct_tags([
            evidence_tag(Evidence::None),
            evidence_tag(Evidence::SourceLocation),
            evidence_tag(Evidence::WitnessPayload),
            evidence_tag(Evidence::ProofTree),
        ]);
        assert_distinct_tags([
            effect_tag(Effect::ReadOnly),
            effect_tag(Effect::ProofEvidenceOnly),
            effect_tag(Effect::ExportOnly),
            effect_tag(Effect::MutationForbidden),
        ]);
        assert_distinct_tags([
            policy_family_tag(ExecutionPolicyFamilyV2::Relational),
            policy_family_tag(ExecutionPolicyFamilyV2::Graph),
            policy_family_tag(ExecutionPolicyFamilyV2::Check),
            policy_family_tag(ExecutionPolicyFamilyV2::Presentation),
            policy_family_tag(ExecutionPolicyFamilyV2::RuleProgram),
            policy_family_tag(ExecutionPolicyFamilyV2::MetaRequireCertified),
            policy_family_tag(ExecutionPolicyFamilyV2::Composition),
        ]);
        assert_distinct_tags([
            cardinality_tag(ExecutionCardinalityV2::Set),
            cardinality_tag(ExecutionCardinalityV2::Bag),
            cardinality_tag(ExecutionCardinalityV2::Sequence),
        ]);
        assert_distinct_tags([
            sort_direction_tag(ExecutionSortDirectionV2::Asc),
            sort_direction_tag(ExecutionSortDirectionV2::Desc),
        ]);
        assert_distinct_tags([
            null_order_tag(ExecutionNullOrderV2::First),
            null_order_tag(ExecutionNullOrderV2::Last),
        ]);
        assert_distinct_tags([
            algebra_family_tag(AlgebraFamilyV2::Scan),
            algebra_family_tag(AlgebraFamilyV2::Filter),
            algebra_family_tag(AlgebraFamilyV2::Project),
            algebra_family_tag(AlgebraFamilyV2::Join),
            algebra_family_tag(AlgebraFamilyV2::Group),
            algebra_family_tag(AlgebraFamilyV2::Aggregate),
            algebra_family_tag(AlgebraFamilyV2::Sort),
            algebra_family_tag(AlgebraFamilyV2::SemiJoin),
            algebra_family_tag(AlgebraFamilyV2::AntiJoin),
        ]);
        assert_distinct_tags([
            join_mode_tag(ExecutionJoinModeV2::Inner),
            join_mode_tag(ExecutionJoinModeV2::Cross),
        ]);
        assert_distinct_tags([
            source_kind_tag(ExecutionSourceKindV2::Relation),
            source_kind_tag(ExecutionSourceKindV2::View),
            source_kind_tag(ExecutionSourceKindV2::Composition),
        ]);
        assert_distinct_tags([
            scalar_tag(ExecutionScalarTypeV2::Bool),
            scalar_tag(ExecutionScalarTypeV2::Int),
            scalar_tag(ExecutionScalarTypeV2::Float),
            scalar_tag(ExecutionScalarTypeV2::String),
            scalar_tag(ExecutionScalarTypeV2::Duration),
            scalar_tag(ExecutionScalarTypeV2::Location),
            scalar_tag(ExecutionScalarTypeV2::Path),
            scalar_tag(ExecutionScalarTypeV2::RepoPath),
            scalar_tag(ExecutionScalarTypeV2::EntityRef),
            scalar_tag(ExecutionScalarTypeV2::SymbolRef),
            scalar_tag(ExecutionScalarTypeV2::RuleRef),
            scalar_tag(ExecutionScalarTypeV2::SourceScope),
            scalar_tag(ExecutionScalarTypeV2::ProofTree),
        ]);
        assert_distinct_tags([
            logical_quantifier_tag(ExecutionLogicalQuantifierV2::Exists),
            logical_quantifier_tag(ExecutionLogicalQuantifierV2::Forall),
        ]);
        assert_distinct_tags([
            exists_polarity_tag(ExecutionExistsPolarityV2::Exists),
            exists_polarity_tag(ExecutionExistsPolarityV2::NotExists),
        ]);
        assert_distinct_tags([
            graph_kind_tag(ExecutionGraphKindV2::Reachability),
            graph_kind_tag(ExecutionGraphKindV2::Flow),
            graph_kind_tag(ExecutionGraphKindV2::Slice),
            graph_kind_tag(ExecutionGraphKindV2::Taint),
            graph_kind_tag(ExecutionGraphKindV2::Path),
            graph_kind_tag(ExecutionGraphKindV2::PointsTo),
            graph_kind_tag(ExecutionGraphKindV2::Summary),
        ]);
    }

    fn key(source: u32, node: u32) -> SourceQualifiedNodeIdV2 {
        SourceQualifiedNodeIdV2 {
            source_id: SourceIdV2::from_raw(source),
            node_id: HirNodeIdV1::from_raw(node),
        }
    }

    fn item(source: u32, node: u32) -> SemanticItemIdV2 {
        SemanticItemIdV2::new(SourceIdV2::from_raw(source), HirNodeIdV1::from_raw(node))
    }

    fn binding(name: &str) -> ExecutionBindingV2 {
        ExecutionBindingV2 {
            source_id: SourceIdV2::from_raw(0),
            owner_node: 7,
            ordinal: 0,
            name: name.to_owned(),
        }
    }

    fn schema() -> ExecutionSchemaV2 {
        ExecutionSchemaV2 {
            fields: Box::new([ExecutionFieldV2 {
                name: "a".to_owned(),
                value_type: ExecutionValueTypeV2::Scalar(ExecutionScalarTypeV2::Int),
            }]),
            cardinality: ExecutionCardinalityV2::Set,
            partition_keys: Box::new([]),
            order: Box::new([]),
        }
    }

    fn root() -> ExecutionRelationalOperatorV2 {
        ExecutionRelationalOperatorV2 {
            family: AlgebraFamilyV2::Scan,
            join_mode: None,
            stage: key(0, 1),
            source: None,
            schema: schema(),
            predicates: Box::new([]),
            expressions: Box::new([]),
            correlations: Box::new([]),
            partition_input: Box::new([]),
            children: Box::new([]),
        }
    }

    fn literal_expression(id: SourceQualifiedNodeIdV2, body: &str) -> ExecutionExpressionV2 {
        ExecutionExpressionV2 {
            id,
            value_type: ExecutionValueTypeV2::Scalar(ExecutionScalarTypeV2::Int),
            shape: ExecutionExpressionShapeV2::Literal(body.to_owned()),
        }
    }

    fn relation() -> ExecutionRelationV2 {
        ExecutionRelationV2 {
            item_id: item(0, 1),
            root: root(),
            expressions: Box::new([literal_expression(key(0, 2), "int:1")]),
        }
    }

    fn plan() -> ExecutionPlanV2 {
        ExecutionPlanV2 {
            authority: ExecutionAuthorityV2::FrontendChecked,
            determinism: ExecutionDeterminismV2::Deterministic,
            evidence: ExecutionEvidenceV2::None,
            effect: ExecutionEffectV2::ReadOnly,
            policy_family: ExecutionPolicyFamilyV2::Relational,
        }
    }

    fn relation_item() -> ExecutionItemV2 {
        ExecutionItemV2 {
            item_id: item(0, 1),
            family: ExecutionFamilyV2::Relation,
            relation: Some(relation()),
            composition: None,
            plan: plan(),
        }
    }

    fn composition_item(binding_schema: Option<ExecutionSchemaV2>) -> ExecutionItemV2 {
        ExecutionItemV2 {
            item_id: item(0, 9),
            family: ExecutionFamilyV2::Composition,
            relation: None,
            composition: Some(ExecutionCompositionV2 {
                bindings: Box::new([ExecutionCompositionBindingV2 {
                    binding: binding("q"),
                    producer_item_id: item(0, 1),
                    producer_family: ExecutionFamilyV2::Relation,
                    schema: binding_schema,
                    relation: None,
                    producer: ExecutionCompositionProducerV2::Relation,
                }]),
                with_binding: binding("q"),
                body_item_id: item(0, 1),
                body_family: ExecutionFamilyV2::Relation,
                body_relation: None,
                body: ExecutionCompositionBodyV2::Relation,
            }),
            plan: plan(),
        }
    }

    fn contract_of(items: Vec<ExecutionItemV2>) -> ExecutionDigestV2 {
        ExecutionDocumentV2::new(SourceIdV2::from_raw(0), items).contract_digest()
    }

    fn qualification_of(items: Vec<ExecutionItemV2>) -> ExecutionDigestV2 {
        ExecutionDocumentV2::new(SourceIdV2::from_raw(0), items).qualification_digest()
    }

    #[test]
    fn predicate_mutation_moves_contract_and_qualification() {
        let base = contract_of(vec![relation_item()]);
        let mut mutated = relation_item();
        mutated.relation.as_mut().expect("relation").root.predicates = Box::new([key(0, 2)]);
        assert_ne!(contract_of(vec![mutated.clone()]), base);
        assert_ne!(
            qualification_of(vec![mutated]),
            qualification_of(vec![relation_item()])
        );
    }

    #[test]
    fn partition_input_mutation_moves_contract_and_qualification() {
        let base = contract_of(vec![relation_item()]);
        let mut mutated = relation_item();
        mutated
            .relation
            .as_mut()
            .expect("relation")
            .root
            .partition_input = Box::new([binding("p")]);
        assert_ne!(contract_of(vec![mutated.clone()]), base);
        assert_ne!(
            qualification_of(vec![mutated]),
            qualification_of(vec![relation_item()])
        );
    }

    #[test]
    fn correlation_mutation_moves_contract_and_qualification() {
        let base = contract_of(vec![relation_item()]);
        let mut mutated = relation_item();
        mutated
            .relation
            .as_mut()
            .expect("relation")
            .root
            .correlations = Box::new([ExecutionCorrelationV2 {
            binding: binding("c"),
            reference: key(0, 2),
        }]);
        assert_ne!(contract_of(vec![mutated.clone()]), base);
        assert_ne!(
            qualification_of(vec![mutated]),
            qualification_of(vec![relation_item()])
        );
    }

    #[test]
    fn schema_mutation_moves_contract_and_qualification() {
        let base = contract_of(vec![relation_item()]);
        let mut mutated = relation_item();
        mutated.relation.as_mut().expect("relation").root.schema = ExecutionSchemaV2 {
            fields: Box::new([ExecutionFieldV2 {
                name: "b".to_owned(),
                value_type: ExecutionValueTypeV2::Scalar(ExecutionScalarTypeV2::String),
            }]),
            cardinality: ExecutionCardinalityV2::Bag,
            partition_keys: Box::new([key(0, 2)]),
            order: Box::new([ExecutionOrderKeyV2 {
                expression: key(0, 2),
                direction: ExecutionSortDirectionV2::Desc,
                null_order: ExecutionNullOrderV2::Last,
                value_type: ExecutionValueTypeV2::Scalar(ExecutionScalarTypeV2::String),
            }]),
        };
        assert_ne!(contract_of(vec![mutated.clone()]), base);
        assert_ne!(
            qualification_of(vec![mutated]),
            qualification_of(vec![relation_item()])
        );
    }

    #[test]
    fn expression_shape_mutation_moves_contract_and_qualification() {
        let base = contract_of(vec![relation_item()]);
        let mut mutated = relation_item();
        mutated.relation.as_mut().expect("relation").expressions =
            Box::new([literal_expression(key(0, 2), "int:2")]);
        assert_ne!(contract_of(vec![mutated.clone()]), base);
        assert_ne!(
            qualification_of(vec![mutated]),
            qualification_of(vec![relation_item()])
        );
    }

    #[test]
    fn composition_binding_schema_mutation_moves_contract_and_qualification() {
        let base = contract_of(vec![composition_item(None)]);
        assert_ne!(contract_of(vec![composition_item(Some(schema()))]), base);
        assert_ne!(
            qualification_of(vec![composition_item(Some(schema()))]),
            qualification_of(vec![composition_item(None)])
        );
    }

    #[test]
    fn body_relation_mutation_moves_contract_and_qualification() {
        let base = contract_of(vec![composition_item(None)]);
        let mut mutated = composition_item(None);
        mutated
            .composition
            .as_mut()
            .expect("composition")
            .body_relation = Some(relation());
        assert_ne!(contract_of(vec![mutated.clone()]), base);
        assert_ne!(
            qualification_of(vec![mutated]),
            qualification_of(vec![composition_item(None)])
        );
    }

    #[test]
    fn plan_authority_effect_evidence_mutations_move_contract_and_qualification() {
        let base_contract = contract_of(vec![relation_item()]);
        let base_qualification = qualification_of(vec![relation_item()]);
        let mut authority = relation_item();
        authority.plan.authority = ExecutionAuthorityV2::CertifiedRequirementValidated;
        assert_ne!(contract_of(vec![authority.clone()]), base_contract);
        assert_ne!(qualification_of(vec![authority]), base_qualification);
        let mut effect = relation_item();
        effect.plan.effect = ExecutionEffectV2::ExportOnly;
        assert_ne!(contract_of(vec![effect.clone()]), base_contract);
        assert_ne!(qualification_of(vec![effect]), base_qualification);
        let mut evidence = relation_item();
        evidence.plan.evidence = ExecutionEvidenceV2::ProofTree;
        assert_ne!(contract_of(vec![evidence.clone()]), base_contract);
        assert_ne!(qualification_of(vec![evidence]), base_qualification);
    }

    #[test]
    fn source_byte_mutation_moves_qualification_but_not_contract() {
        let mut base = ExecutionDocumentV2::new(SourceIdV2::from_raw(0), vec![relation_item()]);
        base.bind_source_bytes(b"show models");
        let mut mutated = ExecutionDocumentV2::new(SourceIdV2::from_raw(0), vec![relation_item()]);
        mutated.bind_source_bytes(b"show models ");
        assert_eq!(base.contract_digest(), mutated.contract_digest());
        assert_ne!(base.source_digest(), mutated.source_digest());
        assert_ne!(base.qualification_digest(), mutated.qualification_digest());
    }

    #[test]
    fn closure_collects_transitively_in_stable_identity_order() {
        let root_key = key(0, 9);
        let left = key(0, 3);
        let right = key(0, 5);
        let leaf = key(0, 1);
        let children = |id: SourceQualifiedNodeIdV2| {
            if id == root_key {
                Some(vec![right, left])
            } else if id == left || id == right {
                Some(vec![leaf])
            } else if id == leaf {
                Some(Vec::new())
            } else {
                None
            }
        };
        let closed =
            close_expression_ids(SourceIdV2::from_raw(0), &[root_key], &children).expect("closed");
        assert_eq!(closed, vec![leaf, left, right, root_key]);
    }

    #[test]
    fn closure_dangling_reference_fails_closed() {
        let children =
            |_: SourceQualifiedNodeIdV2| -> Option<Vec<SourceQualifiedNodeIdV2>> { None };
        assert_eq!(
            close_expression_ids(SourceIdV2::from_raw(0), &[key(0, 1)], &children),
            Err(ExecutionClosureFailureV2::Dangling)
        );
    }

    #[test]
    fn closure_cross_source_reference_fails_closed() {
        let children = |_: SourceQualifiedNodeIdV2| Some(Vec::new());
        assert_eq!(
            close_expression_ids(SourceIdV2::from_raw(0), &[key(1, 1)], &children),
            Err(ExecutionClosureFailureV2::Foreign)
        );
    }

    #[test]
    fn value_type_projection_covers_every_constructible_arm() {
        use quanta_gqlang_v2_sema::{
            LocatedSpanV2, RecordFieldTypeV1, RecordTypeV1, SourceRegistrationV2,
            SourceRegistryViewV2, SpanV1,
        };
        assert_eq!(
            project_value_type(&ValueTypeV1::int()),
            ExecutionValueTypeV2::Scalar(ExecutionScalarTypeV2::Int)
        );
        assert_eq!(
            project_value_type(&ValueTypeV1::null()),
            ExecutionValueTypeV2::Null
        );
        let tuple = ValueTypeV1::try_tuple(vec![ValueTypeV1::int(), ValueTypeV1::string()])
            .expect("tuple of scalars");
        assert_eq!(
            project_value_type(&tuple),
            ExecutionValueTypeV2::Tuple(Box::new([
                ExecutionValueTypeV2::Scalar(ExecutionScalarTypeV2::Int),
                ExecutionValueTypeV2::Scalar(ExecutionScalarTypeV2::String),
            ]))
        );
        let optional = ValueTypeV1::try_optional(ValueTypeV1::bool()).expect("optional scalar");
        assert_eq!(
            project_value_type(&optional),
            ExecutionValueTypeV2::Optional(Box::new(ExecutionValueTypeV2::Scalar(
                ExecutionScalarTypeV2::Bool
            )))
        );
        let collection = ValueTypeV1::try_collection(ValueTypeV1::int(), CardinalityV1::Sequence)
            .expect("sequence collection");
        assert_eq!(
            project_value_type(&collection),
            ExecutionValueTypeV2::Collection {
                element: Box::new(ExecutionValueTypeV2::Scalar(ExecutionScalarTypeV2::Int)),
                cardinality: ExecutionCardinalityV2::Sequence,
            }
        );
        let registry = SourceRegistryViewV2::try_new(vec![SourceRegistrationV2 {
            source_id: SourceIdV2::from_raw(0),
            source_len: 64,
        }])
        .expect("one registered source");
        let location = registry
            .try_check_location(LocatedSpanV2 {
                source_id: SourceIdV2::from_raw(0),
                span: SpanV1::new(0, 1),
            })
            .expect("checked location");
        let field =
            RecordFieldTypeV1::try_new("id", ValueTypeV1::int(), location).expect("record field");
        let record =
            ValueTypeV1::try_record(RecordTypeV1::try_new(vec![field]).expect("record row"))
                .expect("record value");
        assert_eq!(
            project_value_type(&record),
            ExecutionValueTypeV2::Record(Box::new([ExecutionRecordFieldV2 {
                name: "id".to_owned(),
                value_type: ExecutionValueTypeV2::Scalar(ExecutionScalarTypeV2::Int),
            }]))
        );
        // Union projection mirrors record arm-for-arm; its constructors are not
        // re-exported for downstream (no producer path builds unions), so the
        // union ENCODING is pinned by `composite_value_type_arms_encode_distinctly`
        // instead of round-tripping through the owner here.
    }

    #[test]
    fn composite_value_type_arms_encode_distinctly() {
        let scalar = ExecutionValueTypeV2::Scalar(ExecutionScalarTypeV2::Int);
        let variants = [
            ExecutionValueTypeV2::Null,
            ExecutionValueTypeV2::Tuple(Box::new([scalar.clone()])),
            ExecutionValueTypeV2::Record(Box::new([ExecutionRecordFieldV2 {
                name: "id".to_owned(),
                value_type: scalar.clone(),
            }])),
            ExecutionValueTypeV2::Optional(Box::new(scalar.clone())),
            ExecutionValueTypeV2::Collection {
                element: Box::new(scalar.clone()),
                cardinality: ExecutionCardinalityV2::Sequence,
            },
            ExecutionValueTypeV2::Union(Box::new([ExecutionUnionVariantV2 {
                tag: "ok".to_owned(),
                value_type: scalar.clone(),
            }])),
            ExecutionValueTypeV2::Unknown,
        ];
        let mut digests = Vec::with_capacity(variants.len());
        for value_type in &variants {
            let mut item = relation_item();
            item.relation.as_mut().expect("relation").root.schema.fields =
                Box::new([ExecutionFieldV2 {
                    name: "a".to_owned(),
                    value_type: value_type.clone(),
                }]);
            digests.push(contract_of(vec![item]));
        }
        let mut deduped = digests.clone();
        deduped.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        deduped.dedup();
        assert_eq!(
            deduped.len(),
            variants.len(),
            "two value-type arms share one encoding"
        );
    }

    #[test]
    fn plan_determinism_and_policy_family_mutations_move_contract() {
        let base = contract_of(vec![relation_item()]);
        let mut determinism = relation_item();
        determinism.plan.determinism = ExecutionDeterminismV2::NonDeterministic;
        assert_ne!(contract_of(vec![determinism]), base);
        let mut policy = relation_item();
        policy.plan.policy_family = ExecutionPolicyFamilyV2::Graph;
        assert_ne!(contract_of(vec![policy]), base);
    }
}
