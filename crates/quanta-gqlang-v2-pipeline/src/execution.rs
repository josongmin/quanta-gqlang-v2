//! Host-agnostic, immutable execution contract projected from sealed V2 owners.
//!
//! This module copies only public semantic facts into a closed transport DTO.
//! Closed vocabularies are execution-local enums. The only reused owner types
//! are the inert identity closure (`SourceIdV2`, `SemanticItemIdV2`,
//! `SourceQualifiedNodeIdV2`) and [`AlgebraFamilyV2`], all already part of the
//! curated facade closure. Every other HIR, Sema, Algebra, Planning, and Types
//! owner type stays unreachable.

use quanta_gqlang_v2_algebra::{AlgebraFamilyV2, CheckedAlgebraNodeV2, LoweredRelationItemV2};
use quanta_gqlang_v2_logic::{
    DedupIdentityV1, GraphQueryContractV2, RelationShapeV1, WitnessTieBreakV1,
};
use quanta_gqlang_v2_planning::{
    AuthorityLaneV2, EffectClassV2, EvaluationDeterminismV2, EvidenceRequirementV2, PlanPolicyV2,
    SemanticPlanV2,
};
use quanta_gqlang_v2_sema::{
    CardinalityV1, CompositionGraphCapabilityV2, ExpressionTypeStateV2, GraphDirectionV2,
    GraphQueryKindV1, GraphReferenceKindV2, GraphReferenceRoleV2, JoinModeV2,
    LogicalQuantifierKindV2, NullOrderV2, RelationAuthorityV2, RelationExistsPolarityV2,
    RelationOutputContractV2, ResolvedCallTargetV2, ResolvedCompositionBodyV2,
    ResolvedCompositionProducerV2, ResolvedCompositionV2, ResolvedGraphTargetV2,
    ResolvedGraphUseV2, ResolvedPresentationKindV2, ResolvedRelationSourceKindV2, SampleModeV1,
    ScalarTyV1, SemanticDocumentV2, SemanticItemIdV2, SortDirectionV2, SourceIdV2,
    SourceQualifiedNodeIdV2, TypedCallArgumentV2, TypedExprShapeV2, TypedExprV2,
    TypedLiteralFactV2, TypedLiteralValueV2, ValueTypeV1,
};
use sha2::{Digest as _, Sha256};

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
    /// only payload-bearing families are candidates for downstream execution.
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
    /// above. Any DTO or source-byte change moves it.
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
    Relation,
    Graph,
    Check,
    Presentation,
    Composition,
    MetaRequireCertified,
    RuleProgram,
}

impl ExecutionFamilyV3 {
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

/// One plan-required item with its immutable planning authority.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionItemV3 {
    item_id: SemanticItemIdV2,
    family: ExecutionFamilyV3,
    relation: Option<ExecutionRelationV3>,
    composition: Option<ExecutionCompositionV3>,
    plan: ExecutionPlanV3,
}

impl ExecutionItemV3 {
    pub(crate) fn new(
        item_id: SemanticItemIdV2,
        family: ExecutionFamilyV3,
        relation: Option<ExecutionRelationV3>,
        composition: Option<ExecutionCompositionV3>,
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
    pub const fn family(&self) -> ExecutionFamilyV3 {
        self.family
    }
    #[must_use]
    pub const fn relation(&self) -> Option<&ExecutionRelationV3> {
        self.relation.as_ref()
    }
    #[must_use]
    pub const fn composition(&self) -> Option<&ExecutionCompositionV3> {
        self.composition.as_ref()
    }
    /// Producer-derived policy and authority; callers cannot replace it.
    #[must_use]
    pub const fn plan(&self) -> &ExecutionPlanV3 {
        &self.plan
    }
}

/// Frontend-checked policy copied from the sealed semantic plan. This is pure
/// frontend vocabulary: no runtime, provider, or backend routing appears here.
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
        }
    }
}

/// Closed scalar vocabulary projected from the sealed type owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionScalarTypeV3 {
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

const fn project_scalar(scalar: &ScalarTyV1) -> ExecutionScalarTypeV3 {
    match scalar {
        ScalarTyV1::Bool => ExecutionScalarTypeV3::Bool,
        ScalarTyV1::Int => ExecutionScalarTypeV3::Int,
        ScalarTyV1::Float => ExecutionScalarTypeV3::Float,
        ScalarTyV1::String => ExecutionScalarTypeV3::String,
        ScalarTyV1::Duration => ExecutionScalarTypeV3::Duration,
        ScalarTyV1::Location => ExecutionScalarTypeV3::Location,
        ScalarTyV1::Path => ExecutionScalarTypeV3::Path,
        ScalarTyV1::RepoPath => ExecutionScalarTypeV3::RepoPath,
        ScalarTyV1::EntityRef => ExecutionScalarTypeV3::EntityRef,
        ScalarTyV1::SymbolRef => ExecutionScalarTypeV3::SymbolRef,
        ScalarTyV1::RuleRef => ExecutionScalarTypeV3::RuleRef,
        ScalarTyV1::SourceScope => ExecutionScalarTypeV3::SourceScope,
        ScalarTyV1::ProofTree => ExecutionScalarTypeV3::ProofTree,
    }
}

/// Closed value-type vocabulary. Only record field names and union tags are
/// open strings; every structural kind is an enum variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionValueTypeV3 {
    Scalar(ExecutionScalarTypeV3),
    Null,
    Tuple(Box<[ExecutionValueTypeV3]>),
    Record(Box<[ExecutionRecordFieldV3]>),
    Optional(Box<ExecutionValueTypeV3>),
    Collection {
        element: Box<ExecutionValueTypeV3>,
        cardinality: ExecutionCardinalityV3,
    },
    Union(Box<[ExecutionUnionVariantV3]>),
    /// The checked expression carries no known type. The unknown cause is
    /// editor-specific, so it is excluded from the contract by design: two
    /// unknowns are indistinguishable here however they arose. Schemas never
    /// project this variant: row and order types are always known.
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionRecordFieldV3 {
    name: String,
    value_type: ExecutionValueTypeV3,
}
impl ExecutionRecordFieldV3 {
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
pub struct ExecutionUnionVariantV3 {
    tag: String,
    value_type: ExecutionValueTypeV3,
}
impl ExecutionUnionVariantV3 {
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }
    #[must_use]
    pub const fn value_type(&self) -> &ExecutionValueTypeV3 {
        &self.value_type
    }
}

fn project_value_type(value: &ValueTypeV1) -> ExecutionValueTypeV3 {
    if let Some(scalar) = value.as_type().as_scalar() {
        return ExecutionValueTypeV3::Scalar(project_scalar(scalar));
    }
    if value.is_null() {
        return ExecutionValueTypeV3::Null;
    }
    if let Some(tuple) = value.as_tuple() {
        return ExecutionValueTypeV3::Tuple(
            tuple
                .items()
                .iter()
                .map(project_value_type)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
    }
    if let Some(record) = value.as_record() {
        return ExecutionValueTypeV3::Record(
            record
                .fields()
                .iter()
                .map(|field| ExecutionRecordFieldV3 {
                    name: field.name().to_owned(),
                    value_type: project_value_type(field.value_type()),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
    }
    if let Some(optional) = value.as_optional() {
        return ExecutionValueTypeV3::Optional(Box::new(project_value_type(optional.inner())));
    }
    if let Some(collection) = value.as_collection() {
        return ExecutionValueTypeV3::Collection {
            element: Box::new(project_value_type(collection.element())),
            cardinality: project_cardinality(collection.cardinality()),
        };
    }
    if let Some(union) = value.as_union() {
        return ExecutionValueTypeV3::Union(
            union
                .variants()
                .iter()
                .map(|variant| ExecutionUnionVariantV3 {
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
    ExecutionValueTypeV3::Unknown
}

/// Closed row/collection cardinality vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionCardinalityV3 {
    Set,
    Bag,
    Sequence,
}

const fn project_cardinality(value: CardinalityV1) -> ExecutionCardinalityV3 {
    match value {
        CardinalityV1::Set => ExecutionCardinalityV3::Set,
        CardinalityV1::Bag => ExecutionCardinalityV3::Bag,
        CardinalityV1::Sequence => ExecutionCardinalityV3::Sequence,
    }
}

/// Closed sort-direction vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionSortDirectionV3 {
    Asc,
    Desc,
}

const fn project_sort_direction(value: SortDirectionV2) -> ExecutionSortDirectionV3 {
    match value {
        SortDirectionV2::Asc => ExecutionSortDirectionV3::Asc,
        SortDirectionV2::Desc => ExecutionSortDirectionV3::Desc,
    }
}

/// Closed null-ordering vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionNullOrderV3 {
    First,
    Last,
}

const fn project_null_order(value: NullOrderV2) -> ExecutionNullOrderV3 {
    match value {
        NullOrderV2::First => ExecutionNullOrderV3::First,
        NullOrderV2::Last => ExecutionNullOrderV3::Last,
    }
}

/// Closed join-mode vocabulary. Only `Join` operators carry a mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionJoinModeV3 {
    Inner,
    Cross,
}

const fn project_join_mode(value: JoinModeV2) -> ExecutionJoinModeV3 {
    match value {
        JoinModeV2::Inner => ExecutionJoinModeV3::Inner,
        JoinModeV2::Cross => ExecutionJoinModeV3::Cross,
    }
}

/// Closed scan-source-kind vocabulary. Names and namespaces stay open strings
/// on [`ExecutionSourceV3`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionSourceKindV3 {
    Relation,
    View,
    Composition,
}

/// Closed call-target vocabulary. Only the callee identity is an open string;
/// the target class is an enum variant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionCallTargetV3 {
    Builtin { name: String },
    Predicate { name: String },
    HostPredicate { fingerprint: String },
}

fn project_call_target(target: &ResolvedCallTargetV2) -> ExecutionCallTargetV3 {
    match target {
        ResolvedCallTargetV2::Builtin(id) => ExecutionCallTargetV3::Builtin {
            name: id.name().unwrap_or("invalid_closed_builtin").to_owned(),
        },
        ResolvedCallTargetV2::Predicate(id) => ExecutionCallTargetV3::Predicate {
            name: id.as_str().to_owned(),
        },
        ResolvedCallTargetV2::HostPredicate(id) => ExecutionCallTargetV3::HostPredicate {
            fingerprint: hex_lower(id.canonical_bytes()),
        },
    }
}

/// Closed logical-quantifier vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionLogicalQuantifierV3 {
    Exists,
    Forall,
}

const fn project_logical_quantifier(
    value: LogicalQuantifierKindV2,
) -> ExecutionLogicalQuantifierV3 {
    match value {
        LogicalQuantifierKindV2::Exists => ExecutionLogicalQuantifierV3::Exists,
        LogicalQuantifierKindV2::Forall => ExecutionLogicalQuantifierV3::Forall,
    }
}

/// Closed relational-existence polarity vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionExistsPolarityV3 {
    Exists,
    NotExists,
}

const fn project_exists_polarity(value: RelationExistsPolarityV2) -> ExecutionExistsPolarityV3 {
    match value {
        RelationExistsPolarityV2::Exists => ExecutionExistsPolarityV3::Exists,
        RelationExistsPolarityV2::NotExists => ExecutionExistsPolarityV3::NotExists,
    }
}

fn project_plan(plan: &SemanticPlanV2) -> ExecutionPlanV3 {
    let authority = match plan.authority_lane() {
        AuthorityLaneV2::FrontendChecked => ExecutionAuthorityV3::FrontendChecked,
        AuthorityLaneV2::CertifiedRequirementValidated => {
            ExecutionAuthorityV3::CertifiedRequirementValidated
        }
    };
    let determinism = match plan.evaluation_determinism() {
        EvaluationDeterminismV2::Deterministic => ExecutionDeterminismV3::Deterministic,
        EvaluationDeterminismV2::NonDeterministic => ExecutionDeterminismV3::NonDeterministic,
    };
    let evidence = match plan.evidence() {
        EvidenceRequirementV2::None => ExecutionEvidenceV3::None,
        EvidenceRequirementV2::SourceLocation => ExecutionEvidenceV3::SourceLocation,
        EvidenceRequirementV2::WitnessPayload => ExecutionEvidenceV3::WitnessPayload,
        EvidenceRequirementV2::ProofTree => ExecutionEvidenceV3::ProofTree,
    };
    let effect = match plan.effect() {
        EffectClassV2::ReadOnly => ExecutionEffectV3::ReadOnly,
        EffectClassV2::ProofEvidenceOnly => ExecutionEffectV3::ProofEvidenceOnly,
        EffectClassV2::ExportOnly => ExecutionEffectV3::ExportOnly,
        EffectClassV2::MutationForbidden => ExecutionEffectV3::MutationForbidden,
    };
    let policy_family = match plan.policy() {
        PlanPolicyV2::Relational { .. } => ExecutionPolicyFamilyV3::Relational,
        PlanPolicyV2::Graph(_) => ExecutionPolicyFamilyV3::Graph,
        PlanPolicyV2::Check(_) => ExecutionPolicyFamilyV3::Check,
        PlanPolicyV2::Presentation(_) => ExecutionPolicyFamilyV3::Presentation,
        PlanPolicyV2::Rule(_) => ExecutionPolicyFamilyV3::RuleProgram,
        PlanPolicyV2::MetaRequireCertified => ExecutionPolicyFamilyV3::MetaRequireCertified,
        PlanPolicyV2::Composition(_) => ExecutionPolicyFamilyV3::Composition,
    };
    ExecutionPlanV3 {
        authority,
        determinism,
        evidence,
        effect,
        policy_family,
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
    #[must_use]
    pub fn partition_input(&self) -> &[ExecutionBindingV3] {
        &self.partition_input
    }
    #[must_use]
    pub fn children(&self) -> &[ExecutionRelationalOperatorV3] {
        &self.children
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

fn project_operator(operator: quanta_gqlang_v2_sema::OperatorIdV2) -> ExecutionOperatorV3 {
    ExecutionOperatorV3 {
        symbol: operator
            .symbol()
            .unwrap_or("invalid_closed_operator")
            .to_owned(),
        overload_id: operator.raw(),
    }
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

/// Expression-closure failure: the item root references an expression the
/// document table cannot supply from the item's own source.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExecutionClosureFailureV3 {
    /// A referenced expression identity is absent from the document table.
    Dangling,
    /// A referenced expression identity belongs to another source.
    Foreign,
}

pub(crate) fn project_relation(
    document: &SemanticDocumentV2,
    lowered: &LoweredRelationItemV2,
) -> Result<ExecutionRelationV3, ExecutionClosureFailureV3> {
    project_relation_root(document, lowered.item_id(), lowered.root())
}

pub(crate) fn project_relation_root(
    document: &SemanticDocumentV2,
    item_id: SemanticItemIdV2,
    root: &CheckedAlgebraNodeV2,
) -> Result<ExecutionRelationV3, ExecutionClosureFailureV3> {
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
            return Err(ExecutionClosureFailureV3::Dangling);
        };
        table.push(project_expression(expr));
    }
    Ok(ExecutionRelationV3 {
        item_id,
        root: projected,
        expressions: table.into_boxed_slice(),
    })
}

/// Every expression identity the operator tree references directly: predicate
/// and projection keys, correlation references, schema partition keys, and
/// order-key expressions, recursively through children.
fn collect_operator_roots(
    node: &ExecutionRelationalOperatorV3,
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
) -> Result<Vec<SourceQualifiedNodeIdV2>, ExecutionClosureFailureV3> {
    let mut closed = std::collections::BTreeSet::new();
    let mut stack: Vec<SourceQualifiedNodeIdV2> = roots.to_vec();
    while let Some(key) = stack.pop() {
        if key.source_id != source_id {
            return Err(ExecutionClosureFailureV3::Foreign);
        }
        if !closed.insert(key) {
            continue;
        }
        let Some(children) = children_of(key) else {
            return Err(ExecutionClosureFailureV3::Dangling);
        };
        stack.extend(children);
    }
    Ok(closed.into_iter().collect())
}

/// Exactly one checked execution payload for one Composition producer.
///
/// This tagged projection prevents relation/graph payload arrays from drifting
/// out of alignment with the sealed binding order.
pub(crate) enum ExecutionCompositionProducerProjectionV3 {
    Relation(ExecutionRelationV3),
    Sample,
    Graph {
        item_id: SemanticItemIdV2,
        query: ExecutionGraphQueryV3,
    },
}

/// Exactly one checked execution payload for the Composition body.
pub(crate) enum ExecutionCompositionBodyProjectionV3 {
    Relation(ExecutionRelationV3),
    Graph {
        item_id: SemanticItemIdV2,
        query: ExecutionGraphQueryV3,
    },
    Check,
    Explain,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExecutionCompositionProjectionFailureV3 {
    Producer,
    Body,
}

pub(crate) fn project_composition(
    composition: &ResolvedCompositionV2,
    producer_projections: Vec<ExecutionCompositionProducerProjectionV3>,
    body_projection: ExecutionCompositionBodyProjectionV3,
) -> Result<ExecutionCompositionV3, ExecutionCompositionProjectionFailureV3> {
    if producer_projections.len() != composition.bindings().len() {
        return Err(ExecutionCompositionProjectionFailureV3::Producer);
    }
    let bindings = composition
        .bindings()
        .iter()
        .zip(producer_projections)
        .map(|(entry, projection)| {
            let (family, relation, producer) = match (entry.producer(), projection) {
                (
                    ResolvedCompositionProducerV2::Query(_),
                    ExecutionCompositionProducerProjectionV3::Relation(relation),
                ) if relation.item_id() == entry.producer().item_id() => (
                    ExecutionFamilyV3::Relation,
                    Some(relation),
                    ExecutionCompositionProducerV3::Relation,
                ),
                (
                    ResolvedCompositionProducerV2::Sample { presentation, .. },
                    ExecutionCompositionProducerProjectionV3::Sample,
                ) => {
                    let ResolvedPresentationKindV2::Sample {
                        query_text,
                        mode,
                        limit,
                        scope,
                    } = presentation.kind()
                    else {
                        return Err(ExecutionCompositionProjectionFailureV3::Producer);
                    };
                    (
                        ExecutionFamilyV3::Presentation,
                        None,
                        ExecutionCompositionProducerV3::Sample {
                            query_text: query_text.clone(),
                            mode: project_sample_mode(*mode),
                            limit: *limit,
                            scope: scope.as_ref().map(project_graph_use),
                        },
                    )
                }
                (
                    ResolvedCompositionProducerV2::Graph { .. },
                    ExecutionCompositionProducerProjectionV3::Graph { item_id, query },
                ) if item_id == entry.producer().item_id() => (
                    ExecutionFamilyV3::Graph,
                    None,
                    ExecutionCompositionProducerV3::Graph { query },
                ),
                _ => return Err(ExecutionCompositionProjectionFailureV3::Producer),
            };
            Ok(ExecutionCompositionBindingV3 {
                binding: project_binding(entry.binding()),
                producer_item_id: entry.producer().item_id(),
                producer_family: family,
                schema: entry.relation_output().map(project_schema),
                relation,
                producer,
            })
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_boxed_slice();
    let (body_item_id, body_family, body_relation, body) =
        match (composition.body(), body_projection) {
            (
                ResolvedCompositionBodyV2::Query(body),
                ExecutionCompositionBodyProjectionV3::Relation(relation),
            ) if relation.item_id() == body.item_id() => (
                body.item_id(),
                ExecutionFamilyV3::Relation,
                Some(relation),
                ExecutionCompositionBodyV3::Relation,
            ),
            (
                ResolvedCompositionBodyV2::Graph(body),
                ExecutionCompositionBodyProjectionV3::Graph { item_id, query },
            ) if item_id == body.item_id() => (
                body.item_id(),
                ExecutionFamilyV3::Graph,
                None,
                ExecutionCompositionBodyV3::Graph(query),
            ),
            (
                ResolvedCompositionBodyV2::Check(body),
                ExecutionCompositionBodyProjectionV3::Check,
            ) => (
                body.item_id(),
                ExecutionFamilyV3::Check,
                None,
                ExecutionCompositionBodyV3::Check,
            ),
            (
                ResolvedCompositionBodyV2::Explain(body),
                ExecutionCompositionBodyProjectionV3::Explain,
            ) => (
                body.item_id(),
                ExecutionFamilyV3::Presentation,
                None,
                ExecutionCompositionBodyV3::Explain,
            ),
            _ => return Err(ExecutionCompositionProjectionFailureV3::Body),
        };
    Ok(ExecutionCompositionV3 {
        bindings,
        with_binding: project_binding(composition.with_target()),
        body_item_id,
        body_family,
        body_relation,
        body,
    })
}

pub(crate) fn project_graph_query(contract: &GraphQueryContractV2) -> ExecutionGraphQueryV3 {
    ExecutionGraphQueryV3 {
        kind: project_graph_kind(contract.kind()),
        references: contract
            .references()
            .iter()
            .map(|reference| ExecutionGraphReferenceV3 {
                role: project_graph_reference_role(reference.role()),
                target: project_graph_target(reference.target()),
            })
            .collect::<Vec<_>>()
            .into_boxed_slice(),
        depth: contract.depth(),
        direction: contract.direction().map(project_graph_direction),
        summary_facets: contract.summary_facets().to_vec().into_boxed_slice(),
        relation_shape: project_graph_relation_shape(contract.shape()),
        dedup_identity: project_graph_dedup_identity(contract.dedup()),
        max_witnesses: contract.max_witnesses(),
        max_path_length: contract.witness().max_path_length(),
        witness_tie_break: project_graph_witness_tie_break(contract.tie_break()),
    }
}

const fn project_graph_relation_shape(shape: RelationShapeV1) -> ExecutionGraphRelationShapeV3 {
    match shape {
        RelationShapeV1::Acyclic => ExecutionGraphRelationShapeV3::Acyclic,
        RelationShapeV1::Cyclic => ExecutionGraphRelationShapeV3::Cyclic,
    }
}

const fn project_graph_dedup_identity(identity: DedupIdentityV1) -> ExecutionGraphDedupIdentityV3 {
    match identity {
        DedupIdentityV1::Endpoints => ExecutionGraphDedupIdentityV3::Endpoints,
        DedupIdentityV1::NodeSequence => ExecutionGraphDedupIdentityV3::NodeSequence,
        DedupIdentityV1::NodeSet => ExecutionGraphDedupIdentityV3::NodeSet,
    }
}

const fn project_graph_witness_tie_break(
    tie_break: WitnessTieBreakV1,
) -> ExecutionGraphWitnessTieBreakV3 {
    match tie_break {
        WitnessTieBreakV1::ShortestThenLexical => {
            ExecutionGraphWitnessTieBreakV3::ShortestThenLexical
        }
        WitnessTieBreakV1::Lexical => ExecutionGraphWitnessTieBreakV3::Lexical,
    }
}

fn project_graph_use(graph_use: &ResolvedGraphUseV2) -> ExecutionGraphTargetV3 {
    project_graph_target(graph_use.target())
}

fn project_graph_target(target: &ResolvedGraphTargetV2) -> ExecutionGraphTargetV3 {
    match target {
        ResolvedGraphTargetV2::Host(selector) => ExecutionGraphTargetV3::Host {
            kind: project_graph_reference_kind(selector.kind()),
            spelling: selector.spelling().to_owned(),
            entity_subtype: selector.entity_subtype().map(str::to_owned),
        },
        ResolvedGraphTargetV2::Composition(target) => ExecutionGraphTargetV3::Composition {
            binding: project_binding(target.source().binding()),
            capability: match target.capability() {
                CompositionGraphCapabilityV2::Entity => ExecutionGraphCapabilityV3::Entity,
                CompositionGraphCapabilityV2::Scope => ExecutionGraphCapabilityV3::Scope,
            },
        },
    }
}

const fn project_graph_reference_role(role: GraphReferenceRoleV2) -> ExecutionGraphReferenceRoleV3 {
    match role {
        GraphReferenceRoleV2::Source => ExecutionGraphReferenceRoleV3::Source,
        GraphReferenceRoleV2::Sink => ExecutionGraphReferenceRoleV3::Sink,
        GraphReferenceRoleV2::Anchor => ExecutionGraphReferenceRoleV3::Anchor,
        GraphReferenceRoleV2::Reference => ExecutionGraphReferenceRoleV3::Reference,
        GraphReferenceRoleV2::Pointee => ExecutionGraphReferenceRoleV3::Pointee,
        GraphReferenceRoleV2::Callable => ExecutionGraphReferenceRoleV3::Callable,
        GraphReferenceRoleV2::Via => ExecutionGraphReferenceRoleV3::Via,
        GraphReferenceRoleV2::Through => ExecutionGraphReferenceRoleV3::Through,
        GraphReferenceRoleV2::Sanitizer => ExecutionGraphReferenceRoleV3::Sanitizer,
        GraphReferenceRoleV2::Scope => ExecutionGraphReferenceRoleV3::Scope,
        GraphReferenceRoleV2::Window => ExecutionGraphReferenceRoleV3::Window,
        GraphReferenceRoleV2::Frontier => ExecutionGraphReferenceRoleV3::Frontier,
    }
}

const fn project_graph_reference_kind(kind: GraphReferenceKindV2) -> ExecutionGraphReferenceKindV3 {
    match kind {
        GraphReferenceKindV2::Entity => ExecutionGraphReferenceKindV3::Entity,
        GraphReferenceKindV2::Callable => ExecutionGraphReferenceKindV3::Callable,
        GraphReferenceKindV2::Scope => ExecutionGraphReferenceKindV3::Scope,
        GraphReferenceKindV2::Window => ExecutionGraphReferenceKindV3::Window,
        GraphReferenceKindV2::Frontier => ExecutionGraphReferenceKindV3::Frontier,
        GraphReferenceKindV2::Sanitizer => ExecutionGraphReferenceKindV3::Sanitizer,
        GraphReferenceKindV2::Pattern => ExecutionGraphReferenceKindV3::Pattern,
        GraphReferenceKindV2::Snapshot => ExecutionGraphReferenceKindV3::Snapshot,
        GraphReferenceKindV2::ExportSource => ExecutionGraphReferenceKindV3::ExportSource,
    }
}

const fn project_graph_direction(direction: GraphDirectionV2) -> ExecutionGraphDirectionV3 {
    match direction {
        GraphDirectionV2::Forward => ExecutionGraphDirectionV3::Forward,
        GraphDirectionV2::Backward => ExecutionGraphDirectionV3::Backward,
        GraphDirectionV2::Both => ExecutionGraphDirectionV3::Both,
        GraphDirectionV2::Undirected => ExecutionGraphDirectionV3::Undirected,
    }
}

const fn project_sample_mode(mode: SampleModeV1) -> ExecutionSampleModeV3 {
    match mode {
        SampleModeV1::Lexical => ExecutionSampleModeV3::Lexical,
        SampleModeV1::Semantic => ExecutionSampleModeV3::Semantic,
        SampleModeV1::Hybrid => ExecutionSampleModeV3::Hybrid,
    }
}
const fn project_graph_kind(kind: GraphQueryKindV1) -> ExecutionGraphKindV3 {
    match kind {
        GraphQueryKindV1::Reachability => ExecutionGraphKindV3::Reachability,
        GraphQueryKindV1::Flow => ExecutionGraphKindV3::Flow,
        GraphQueryKindV1::Slice => ExecutionGraphKindV3::Slice,
        GraphQueryKindV1::Taint => ExecutionGraphKindV3::Taint,
        GraphQueryKindV1::Path => ExecutionGraphKindV3::Path,
        GraphQueryKindV1::PointsTo => ExecutionGraphKindV3::PointsTo,
        GraphQueryKindV1::Summary => ExecutionGraphKindV3::Summary,
    }
}

fn project_node(node: &CheckedAlgebraNodeV2) -> ExecutionRelationalOperatorV3 {
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
        .map(|edge| ExecutionCorrelationV3 {
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
    ExecutionRelationalOperatorV3 {
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

fn project_expression(expr: &TypedExprV2) -> ExecutionExpressionV3 {
    let value_type = match expr.type_state() {
        ExpressionTypeStateV2::Known(value) => project_value_type(value),
        ExpressionTypeStateV2::Unknown(_) => ExecutionValueTypeV3::Unknown,
    };
    ExecutionExpressionV3 {
        id: expr.key(),
        value_type,
        shape: project_expression_shape(expr.shape()),
    }
}

fn project_expression_shape(shape: &TypedExprShapeV2) -> ExecutionExpressionShapeV3 {
    match shape {
        TypedExprShapeV2::Literal(fact) => ExecutionExpressionShapeV3::Literal(match fact {
            TypedLiteralFactV2::DeferredI128MinMagnitude => ExecutionLiteralV3::Int(i128::MIN),
            TypedLiteralFactV2::Value(value) => match value {
                TypedLiteralValueV2::Int(value) => ExecutionLiteralV3::Int(*value),
                TypedLiteralValueV2::Float(value) => {
                    ExecutionLiteralV3::Float(ExecutionFiniteFloatV3(value.get().to_bits()))
                }
                TypedLiteralValueV2::Bool(value) => ExecutionLiteralV3::Bool(*value),
                TypedLiteralValueV2::String(value) => ExecutionLiteralV3::String(value.clone()),
                TypedLiteralValueV2::Null => ExecutionLiteralV3::Null,
            },
        }),
        TypedExprShapeV2::Binding(binding) => {
            ExecutionExpressionShapeV3::Binding(project_binding(binding))
        }
        TypedExprShapeV2::Path { base, segments } => ExecutionExpressionShapeV3::Path {
            base: *base,
            segments: segments
                .iter()
                .map(|s| s.name().to_owned())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        },
        TypedExprShapeV2::Unary { operator, operand } => ExecutionExpressionShapeV3::Unary {
            operator: project_operator(*operator),
            operand: *operand,
        },
        TypedExprShapeV2::Binary { operator, lhs, rhs } => ExecutionExpressionShapeV3::Binary {
            operator: project_operator(*operator),
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::Call { target, arguments } => ExecutionExpressionShapeV3::Call {
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
        TypedExprShapeV2::Paren { inner } => ExecutionExpressionShapeV3::Paren(*inner),
        TypedExprShapeV2::Record(fields) => ExecutionExpressionShapeV3::Record(
            fields
                .iter()
                .map(|field| (field.name().to_owned(), field.value()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        TypedExprShapeV2::List(items) => {
            ExecutionExpressionShapeV3::List(items.clone().into_boxed_slice())
        }
        TypedExprShapeV2::BoolValue { value } => ExecutionExpressionShapeV3::BoolValue(*value),
        TypedExprShapeV2::Compare { operator, lhs, rhs } => ExecutionExpressionShapeV3::Compare {
            operator: project_operator(*operator),
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::And { operator, lhs, rhs } => ExecutionExpressionShapeV3::And {
            operator: project_operator(*operator),
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::Or { operator, lhs, rhs } => ExecutionExpressionShapeV3::Or {
            operator: project_operator(*operator),
            lhs: *lhs,
            rhs: *rhs,
        },
        TypedExprShapeV2::Not { operator, operand } => ExecutionExpressionShapeV3::Not {
            operator: project_operator(*operator),
            operand: *operand,
        },
        TypedExprShapeV2::LogicalQuantifier {
            kind,
            binders,
            body,
        } => ExecutionExpressionShapeV3::LogicalQuantifier {
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
        } => ExecutionExpressionShapeV3::RelationExists {
            polarity: project_exists_polarity(*polarity),
            relation_root: *relation_root,
            correlations: correlations
                .iter()
                .map(|edge| ExecutionCorrelationV3 {
                    binding: project_binding(edge.binding()),
                    reference: edge.reference(),
                })
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        },
    }
}

fn project_schema(output: &RelationOutputContractV2) -> ExecutionSchemaV3 {
    let fields = output
        .row()
        .fields()
        .iter()
        .map(|field| ExecutionFieldV3 {
            name: field.name().to_owned(),
            value_type: project_value_type(field.value_type()),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    let cardinality = project_cardinality(output.cardinality());
    let order = output
        .ordering()
        .iter()
        .map(|key| ExecutionOrderKeyV3 {
            expression: key.expression(),
            direction: project_sort_direction(key.direction()),
            null_order: project_null_order(key.null_order()),
            value_type: project_value_type(key.value_type()),
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();
    ExecutionSchemaV3 {
        fields,
        cardinality,
        partition_keys: output.partition_keys().to_vec().into_boxed_slice(),
        order,
    }
}

fn project_source(source: &ResolvedRelationSourceKindV2) -> ExecutionSourceV3 {
    match source {
        ResolvedRelationSourceKindV2::Relation(id) => ExecutionSourceV3 {
            authority: match id.authority() {
                RelationAuthorityV2::Host => ExecutionRelationAuthorityV3::Host,
                RelationAuthorityV2::CanonicalWorkspaceFiles => {
                    ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles
                }
            },
            kind: ExecutionSourceKindV3::Relation,
            namespace: Some(id.namespace().as_str().to_owned()),
            name: id.qualified_name().to_owned(),
        },
        ResolvedRelationSourceKindV2::View(id) => ExecutionSourceV3 {
            authority: ExecutionRelationAuthorityV3::CatalogView,
            kind: ExecutionSourceKindV3::View,
            namespace: Some(id.module_id().as_str().to_owned()),
            name: id.local_name().to_owned(),
        },
        ResolvedRelationSourceKindV2::Composition(id) => ExecutionSourceV3 {
            authority: ExecutionRelationAuthorityV3::CompositionBinding,
            kind: ExecutionSourceKindV3::Composition,
            namespace: None,
            name: id.binding().canonical_name().to_owned(),
        },
    }
}

fn project_binding(binding: &quanta_gqlang_v2_sema::ResolvedValueBindingV2) -> ExecutionBindingV3 {
    ExecutionBindingV3 {
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
const SOURCE_DIGEST_SCHEMA_VERSION: u32 = 3;
// The Rust DTO is still Execution V3. Its contract/qualification encoding has
// an independent revision because adding scoped SAMPLE, complete graph-query
// payloads, and the sealed witness policy is byte-incompatible with the
// original schema-3 stream. The source byte stream itself did not change, so
// its stable schema remains 3.
const CONTRACT_DIGEST_SCHEMA_VERSION: u32 = 5;

/// Digest of the raw UTF-8 source bytes only.
fn digest_source_bytes(source: &[u8]) -> ExecutionDigestV3 {
    let mut bytes = Vec::new();
    put_str(&mut bytes, "qglang.execution.source-bytes.v3");
    put_u32(&mut bytes, SOURCE_DIGEST_SCHEMA_VERSION);
    put_usize(&mut bytes, source.len());
    bytes.extend_from_slice(source);
    ExecutionDigestV3(sha256(&bytes))
}

/// Digest of the canonical DTO encoding only.
fn digest_contract(source_id: SourceIdV2, items: &[ExecutionItemV3]) -> ExecutionDigestV3 {
    let mut bytes = Vec::new();
    put_str(&mut bytes, "qglang.execution.contract.v5");
    put_u32(&mut bytes, CONTRACT_DIGEST_SCHEMA_VERSION);
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
    ExecutionDigestV3(sha256(&bytes))
}

/// Digest binding the contract domain, the schema version, and both legs.
fn digest_qualification(
    source_id: SourceIdV2,
    source_digest: ExecutionDigestV3,
    contract_digest: ExecutionDigestV3,
) -> ExecutionDigestV3 {
    let mut bytes = Vec::new();
    put_str(&mut bytes, "qglang.execution.qualification.v5");
    put_u32(&mut bytes, CONTRACT_DIGEST_SCHEMA_VERSION);
    put_u32(&mut bytes, source_id.raw());
    bytes.extend_from_slice(source_digest.as_bytes());
    bytes.extend_from_slice(contract_digest.as_bytes());
    ExecutionDigestV3(sha256(&bytes))
}

const fn family_tag(value: ExecutionFamilyV3) -> u8 {
    match value {
        ExecutionFamilyV3::Relation => 0,
        ExecutionFamilyV3::Graph => 1,
        ExecutionFamilyV3::Check => 2,
        ExecutionFamilyV3::Presentation => 3,
        ExecutionFamilyV3::Composition => 4,
        ExecutionFamilyV3::MetaRequireCertified => 5,
        ExecutionFamilyV3::RuleProgram => 6,
    }
}

const fn authority_tag(value: ExecutionAuthorityV3) -> u8 {
    match value {
        ExecutionAuthorityV3::FrontendChecked => 0,
        ExecutionAuthorityV3::CertifiedRequirementValidated => 1,
    }
}

const fn determinism_tag(value: ExecutionDeterminismV3) -> u8 {
    match value {
        ExecutionDeterminismV3::Deterministic => 0,
        ExecutionDeterminismV3::NonDeterministic => 1,
    }
}

const fn evidence_tag(value: ExecutionEvidenceV3) -> u8 {
    match value {
        ExecutionEvidenceV3::None => 0,
        ExecutionEvidenceV3::SourceLocation => 1,
        ExecutionEvidenceV3::WitnessPayload => 2,
        ExecutionEvidenceV3::ProofTree => 3,
    }
}

const fn effect_tag(value: ExecutionEffectV3) -> u8 {
    match value {
        ExecutionEffectV3::ReadOnly => 0,
        ExecutionEffectV3::ProofEvidenceOnly => 1,
        ExecutionEffectV3::ExportOnly => 2,
        ExecutionEffectV3::MutationForbidden => 3,
    }
}

const fn policy_family_tag(value: ExecutionPolicyFamilyV3) -> u8 {
    match value {
        ExecutionPolicyFamilyV3::Relational => 0,
        ExecutionPolicyFamilyV3::Graph => 1,
        ExecutionPolicyFamilyV3::Check => 2,
        ExecutionPolicyFamilyV3::Presentation => 3,
        ExecutionPolicyFamilyV3::RuleProgram => 4,
        ExecutionPolicyFamilyV3::MetaRequireCertified => 5,
        ExecutionPolicyFamilyV3::Composition => 6,
    }
}

fn put_composition(out: &mut Vec<u8>, composition: &ExecutionCompositionV3) {
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

fn put_composition_producer(out: &mut Vec<u8>, producer: &ExecutionCompositionProducerV3) {
    match producer {
        ExecutionCompositionProducerV3::Relation => put_u8(out, 0),
        ExecutionCompositionProducerV3::Sample {
            query_text,
            mode,
            limit,
            scope,
        } => {
            put_u8(out, 1);
            put_str(out, query_text);
            put_u8(
                out,
                match mode {
                    ExecutionSampleModeV3::Lexical => 0,
                    ExecutionSampleModeV3::Semantic => 1,
                    ExecutionSampleModeV3::Hybrid => 2,
                },
            );
            put_u32(out, *limit);
            put_optional_graph_target(out, scope.as_ref());
        }
        ExecutionCompositionProducerV3::Graph { query } => {
            put_u8(out, 2);
            put_graph_query(out, query);
        }
    }
}
fn put_composition_body(out: &mut Vec<u8>, body: &ExecutionCompositionBodyV3) {
    match body {
        ExecutionCompositionBodyV3::Relation => put_u8(out, 0),
        ExecutionCompositionBodyV3::Graph(query) => {
            put_u8(out, 1);
            put_graph_query(out, query);
        }
        ExecutionCompositionBodyV3::Check => put_u8(out, 2),
        ExecutionCompositionBodyV3::Explain => put_u8(out, 3),
    }
}

fn put_graph_query(out: &mut Vec<u8>, query: &ExecutionGraphQueryV3) {
    put_u8(out, graph_kind_tag(query.kind));
    put_usize(out, query.references.len());
    for reference in &query.references {
        put_u8(out, graph_reference_role_tag(reference.role));
        put_graph_target(out, &reference.target);
    }
    put_u32(out, query.depth);
    match query.direction {
        Some(direction) => {
            put_u8(out, 1);
            put_u8(out, graph_direction_tag(direction));
        }
        None => put_u8(out, 0),
    }
    put_usize(out, query.summary_facets.len());
    for facet in &query.summary_facets {
        put_str(out, facet);
    }
    put_u8(out, graph_relation_shape_tag(query.relation_shape));
    put_u8(out, graph_dedup_identity_tag(query.dedup_identity));
    put_u32(out, query.max_witnesses);
    match query.max_path_length {
        Some(max_path_length) => {
            put_u8(out, 1);
            put_u32(out, max_path_length);
        }
        None => put_u8(out, 0),
    }
    put_u8(out, graph_witness_tie_break_tag(query.witness_tie_break));
}

fn put_optional_graph_target(out: &mut Vec<u8>, target: Option<&ExecutionGraphTargetV3>) {
    match target {
        Some(target) => {
            put_u8(out, 1);
            put_graph_target(out, target);
        }
        None => put_u8(out, 0),
    }
}

fn put_graph_target(out: &mut Vec<u8>, target: &ExecutionGraphTargetV3) {
    match target {
        ExecutionGraphTargetV3::Host {
            kind,
            spelling,
            entity_subtype,
        } => {
            put_u8(out, 0);
            put_u8(out, graph_reference_kind_tag(*kind));
            put_str(out, spelling);
            match entity_subtype {
                Some(subtype) => {
                    put_u8(out, 1);
                    put_str(out, subtype);
                }
                None => put_u8(out, 0),
            }
        }
        ExecutionGraphTargetV3::Composition {
            binding,
            capability,
        } => {
            put_u8(out, 1);
            put_binding(out, binding);
            put_u8(
                out,
                match capability {
                    ExecutionGraphCapabilityV3::Entity => 0,
                    ExecutionGraphCapabilityV3::Scope => 1,
                },
            );
        }
    }
}

const fn graph_direction_tag(direction: ExecutionGraphDirectionV3) -> u8 {
    match direction {
        ExecutionGraphDirectionV3::Forward => 0,
        ExecutionGraphDirectionV3::Backward => 1,
        ExecutionGraphDirectionV3::Both => 2,
        ExecutionGraphDirectionV3::Undirected => 3,
    }
}

const fn graph_relation_shape_tag(shape: ExecutionGraphRelationShapeV3) -> u8 {
    match shape {
        ExecutionGraphRelationShapeV3::Acyclic => 0,
        ExecutionGraphRelationShapeV3::Cyclic => 1,
    }
}

const fn graph_dedup_identity_tag(identity: ExecutionGraphDedupIdentityV3) -> u8 {
    match identity {
        ExecutionGraphDedupIdentityV3::Endpoints => 0,
        ExecutionGraphDedupIdentityV3::NodeSequence => 1,
        ExecutionGraphDedupIdentityV3::NodeSet => 2,
    }
}

const fn graph_witness_tie_break_tag(tie_break: ExecutionGraphWitnessTieBreakV3) -> u8 {
    match tie_break {
        ExecutionGraphWitnessTieBreakV3::ShortestThenLexical => 0,
        ExecutionGraphWitnessTieBreakV3::Lexical => 1,
    }
}

const fn graph_reference_role_tag(role: ExecutionGraphReferenceRoleV3) -> u8 {
    match role {
        ExecutionGraphReferenceRoleV3::Source => 0,
        ExecutionGraphReferenceRoleV3::Sink => 1,
        ExecutionGraphReferenceRoleV3::Anchor => 2,
        ExecutionGraphReferenceRoleV3::Reference => 3,
        ExecutionGraphReferenceRoleV3::Pointee => 4,
        ExecutionGraphReferenceRoleV3::Callable => 5,
        ExecutionGraphReferenceRoleV3::Via => 6,
        ExecutionGraphReferenceRoleV3::Through => 7,
        ExecutionGraphReferenceRoleV3::Sanitizer => 8,
        ExecutionGraphReferenceRoleV3::Scope => 9,
        ExecutionGraphReferenceRoleV3::Window => 10,
        ExecutionGraphReferenceRoleV3::Frontier => 11,
    }
}

const fn graph_reference_kind_tag(kind: ExecutionGraphReferenceKindV3) -> u8 {
    match kind {
        ExecutionGraphReferenceKindV3::Entity => 0,
        ExecutionGraphReferenceKindV3::Callable => 1,
        ExecutionGraphReferenceKindV3::Scope => 2,
        ExecutionGraphReferenceKindV3::Window => 3,
        ExecutionGraphReferenceKindV3::Frontier => 4,
        ExecutionGraphReferenceKindV3::Sanitizer => 5,
        ExecutionGraphReferenceKindV3::Pattern => 6,
        ExecutionGraphReferenceKindV3::Snapshot => 7,
        ExecutionGraphReferenceKindV3::ExportSource => 8,
    }
}
const fn graph_kind_tag(kind: ExecutionGraphKindV3) -> u8 {
    match kind {
        ExecutionGraphKindV3::Reachability => 0,
        ExecutionGraphKindV3::Flow => 1,
        ExecutionGraphKindV3::Slice => 2,
        ExecutionGraphKindV3::Taint => 3,
        ExecutionGraphKindV3::Path => 4,
        ExecutionGraphKindV3::PointsTo => 5,
        ExecutionGraphKindV3::Summary => 6,
    }
}

fn put_relation(out: &mut Vec<u8>, relation: &ExecutionRelationV3) {
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
fn put_node(out: &mut Vec<u8>, node: &ExecutionRelationalOperatorV3) {
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

const fn relation_authority_tag(authority: ExecutionRelationAuthorityV3) -> u8 {
    match authority {
        ExecutionRelationAuthorityV3::Host => 0,
        ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles => 1,
        ExecutionRelationAuthorityV3::CatalogView => 2,
        ExecutionRelationAuthorityV3::CompositionBinding => 3,
    }
}

fn put_expression_shape(out: &mut Vec<u8>, shape: &ExecutionExpressionShapeV3) {
    match shape {
        ExecutionExpressionShapeV3::Literal(value) => {
            put_u8(out, 0);
            put_literal(out, value);
        }
        ExecutionExpressionShapeV3::Binding(binding) => {
            put_u8(out, 1);
            put_binding(out, binding);
        }
        ExecutionExpressionShapeV3::Path { base, segments } => {
            put_u8(out, 2);
            put_key(out, *base);
            put_usize(out, segments.len());
            for segment in segments {
                put_str(out, segment);
            }
        }
        ExecutionExpressionShapeV3::Unary { operator, operand } => {
            put_u8(out, 3);
            put_operator(out, operator);
            put_key(out, *operand);
        }
        ExecutionExpressionShapeV3::Binary { operator, lhs, rhs } => {
            put_u8(out, 4);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV3::Call { target, arguments } => {
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
        ExecutionExpressionShapeV3::Paren(id) => {
            put_u8(out, 6);
            put_key(out, *id);
        }
        ExecutionExpressionShapeV3::Record(fields) => {
            put_u8(out, 7);
            put_usize(out, fields.len());
            for (name, id) in fields {
                put_str(out, name);
                put_key(out, *id);
            }
        }
        ExecutionExpressionShapeV3::List(items) => {
            put_u8(out, 8);
            put_usize(out, items.len());
            for id in items {
                put_key(out, *id);
            }
        }
        ExecutionExpressionShapeV3::BoolValue(id) => {
            put_u8(out, 9);
            put_key(out, *id);
        }
        ExecutionExpressionShapeV3::Compare { operator, lhs, rhs } => {
            put_u8(out, 10);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV3::And { operator, lhs, rhs } => {
            put_u8(out, 11);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV3::Or { operator, lhs, rhs } => {
            put_u8(out, 12);
            put_operator(out, operator);
            put_key(out, *lhs);
            put_key(out, *rhs);
        }
        ExecutionExpressionShapeV3::Not { operator, operand } => {
            put_u8(out, 13);
            put_operator(out, operator);
            put_key(out, *operand);
        }
        ExecutionExpressionShapeV3::LogicalQuantifier {
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
        ExecutionExpressionShapeV3::RelationExists {
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
fn put_literal(out: &mut Vec<u8>, literal: &ExecutionLiteralV3) {
    match literal {
        ExecutionLiteralV3::Int(value) => {
            put_u8(out, 0);
            out.extend_from_slice(&value.to_be_bytes());
        }
        ExecutionLiteralV3::Float(value) => {
            put_u8(out, 1);
            out.extend_from_slice(&value.bits().to_be_bytes());
        }
        ExecutionLiteralV3::Bool(value) => {
            put_u8(out, 2);
            put_u8(out, u8::from(*value));
        }
        ExecutionLiteralV3::String(value) => {
            put_u8(out, 3);
            put_str(out, value);
        }
        ExecutionLiteralV3::Null => put_u8(out, 4),
    }
}
fn put_schema(out: &mut Vec<u8>, schema: &ExecutionSchemaV3) {
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

fn put_value_type(out: &mut Vec<u8>, value: &ExecutionValueTypeV3) {
    match value {
        ExecutionValueTypeV3::Scalar(scalar) => {
            put_u8(out, 0);
            put_u8(out, scalar_tag(*scalar));
        }
        ExecutionValueTypeV3::Null => put_u8(out, 1),
        ExecutionValueTypeV3::Tuple(items) => {
            put_u8(out, 2);
            put_usize(out, items.len());
            for item in items {
                put_value_type(out, item);
            }
        }
        ExecutionValueTypeV3::Record(fields) => {
            put_u8(out, 3);
            put_usize(out, fields.len());
            for field in fields {
                put_str(out, &field.name);
                put_value_type(out, &field.value_type);
            }
        }
        ExecutionValueTypeV3::Optional(inner) => {
            put_u8(out, 4);
            put_value_type(out, inner);
        }
        ExecutionValueTypeV3::Collection {
            element,
            cardinality,
        } => {
            put_u8(out, 5);
            put_value_type(out, element);
            put_u8(out, cardinality_tag(*cardinality));
        }
        ExecutionValueTypeV3::Union(variants) => {
            put_u8(out, 6);
            put_usize(out, variants.len());
            for variant in variants {
                put_str(out, &variant.tag);
                put_value_type(out, &variant.value_type);
            }
        }
        ExecutionValueTypeV3::Unknown => put_u8(out, 7),
    }
}

const fn scalar_tag(value: ExecutionScalarTypeV3) -> u8 {
    match value {
        ExecutionScalarTypeV3::Bool => 0,
        ExecutionScalarTypeV3::Int => 1,
        ExecutionScalarTypeV3::Float => 2,
        ExecutionScalarTypeV3::String => 3,
        ExecutionScalarTypeV3::Duration => 4,
        ExecutionScalarTypeV3::Location => 5,
        ExecutionScalarTypeV3::Path => 6,
        ExecutionScalarTypeV3::RepoPath => 7,
        ExecutionScalarTypeV3::EntityRef => 8,
        ExecutionScalarTypeV3::SymbolRef => 9,
        ExecutionScalarTypeV3::RuleRef => 10,
        ExecutionScalarTypeV3::SourceScope => 11,
        ExecutionScalarTypeV3::ProofTree => 12,
    }
}

const fn cardinality_tag(value: ExecutionCardinalityV3) -> u8 {
    match value {
        ExecutionCardinalityV3::Set => 0,
        ExecutionCardinalityV3::Bag => 1,
        ExecutionCardinalityV3::Sequence => 2,
    }
}

const fn sort_direction_tag(value: ExecutionSortDirectionV3) -> u8 {
    match value {
        ExecutionSortDirectionV3::Asc => 0,
        ExecutionSortDirectionV3::Desc => 1,
    }
}

const fn null_order_tag(value: ExecutionNullOrderV3) -> u8 {
    match value {
        ExecutionNullOrderV3::First => 0,
        ExecutionNullOrderV3::Last => 1,
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

const fn join_mode_tag(value: ExecutionJoinModeV3) -> u8 {
    match value {
        ExecutionJoinModeV3::Inner => 0,
        ExecutionJoinModeV3::Cross => 1,
    }
}

const fn source_kind_tag(value: ExecutionSourceKindV3) -> u8 {
    match value {
        ExecutionSourceKindV3::Relation => 0,
        ExecutionSourceKindV3::View => 1,
        ExecutionSourceKindV3::Composition => 2,
    }
}

fn put_call_target(out: &mut Vec<u8>, target: &ExecutionCallTargetV3) {
    match target {
        ExecutionCallTargetV3::Builtin { name } => {
            put_u8(out, 0);
            put_str(out, name);
        }
        ExecutionCallTargetV3::Predicate { name } => {
            put_u8(out, 1);
            put_str(out, name);
        }
        ExecutionCallTargetV3::HostPredicate { fingerprint } => {
            put_u8(out, 2);
            put_str(out, fingerprint);
        }
    }
}

const fn logical_quantifier_tag(value: ExecutionLogicalQuantifierV3) -> u8 {
    match value {
        ExecutionLogicalQuantifierV3::Exists => 0,
        ExecutionLogicalQuantifierV3::Forall => 1,
    }
}

const fn exists_polarity_tag(value: ExecutionExistsPolarityV3) -> u8 {
    match value {
        ExecutionExistsPolarityV3::Exists => 0,
        ExecutionExistsPolarityV3::NotExists => 1,
    }
}
fn put_binding(out: &mut Vec<u8>, binding: &ExecutionBindingV3) {
    put_u32(out, binding.source_id.raw());
    put_u32(out, binding.owner_node);
    put_usize(out, binding.ordinal);
    put_str(out, &binding.name);
}
fn put_operator(out: &mut Vec<u8>, operator: &ExecutionOperatorV3) {
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
    clippy::indexing_slicing,
    clippy::panic,
    reason = "contract digest mutation rails assert exact fixture shapes"
)]
mod tests {
    use super::*;
    use quanta_gqlang_v2_sema::{
        test_support::{reference_composition_document_v2, ReferenceCompositionBodyV2},
        HirNodeIdV1,
    };

    #[test]
    fn sha256_matches_published_empty_vector() {
        assert_eq!(
            ExecutionDigestV3(sha256(b"")).to_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn typed_literal_bytes_match_the_v3_contract() {
        let mut encoded = Vec::new();
        put_literal(&mut encoded, &ExecutionLiteralV3::Int(i128::MIN));
        assert_eq!(encoded, [vec![0, 0x80], vec![0; 15]].concat());

        encoded.clear();
        put_literal(
            &mut encoded,
            &ExecutionLiteralV3::Float(ExecutionFiniteFloatV3(1.5f64.to_bits())),
        );
        assert_eq!(
            encoded,
            [vec![1], 1.5f64.to_bits().to_be_bytes().to_vec()].concat()
        );

        encoded.clear();
        put_literal(&mut encoded, &ExecutionLiteralV3::Bool(false));
        assert_eq!(encoded, [2, 0]);
        encoded.clear();
        put_literal(&mut encoded, &ExecutionLiteralV3::Bool(true));
        assert_eq!(encoded, [2, 1]);

        encoded.clear();
        put_literal(&mut encoded, &ExecutionLiteralV3::String("é".to_owned()));
        assert_eq!(
            encoded,
            [vec![3], 2u64.to_be_bytes().to_vec(), vec![0xc3, 0xa9]].concat()
        );

        encoded.clear();
        put_literal(&mut encoded, &ExecutionLiteralV3::Null);
        assert_eq!(encoded, [4]);
    }

    #[test]
    fn canonical_digests_match_the_pinned_golden_vectors() {
        let document = ExecutionDocumentV3::new(
            SourceIdV2::from_raw(0),
            b"show models",
            vec![relation_item()],
        );
        assert_eq!(document.source_digest_schema_version(), 3);
        assert_eq!(document.contract_digest_schema_version(), 5);
        assert_eq!(document.qualification_digest_schema_version(), 5);
        // Golden pins: any canonical-encoding change (tag swap, field reorder,
        // domain edit) must update these deliberately, never silently.
        assert_eq!(
            document.source_digest().to_hex(),
            "a8b69361d9c0d24e3022914bf1fba6386643164dc53911d34f1115dbc6102480"
        );
        assert_eq!(
            document.contract_digest().to_hex(),
            "b429b5f8aaa3abc18bb81b47b56b86ceb3f2dbce6a61122182d5e16eacd78ebd"
        );
        assert_eq!(
            document.qualification_digest().to_hex(),
            "aa642e6b13a3bc34ca613536cdc31d84b587632c47221001b928b8f9a217ba78"
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
        use ExecutionAuthorityV3 as Authority;
        use ExecutionDeterminismV3 as Determinism;
        use ExecutionEffectV3 as Effect;
        use ExecutionEvidenceV3 as Evidence;
        assert_distinct_tags([
            family_tag(ExecutionFamilyV3::Relation),
            family_tag(ExecutionFamilyV3::Graph),
            family_tag(ExecutionFamilyV3::Check),
            family_tag(ExecutionFamilyV3::Presentation),
            family_tag(ExecutionFamilyV3::Composition),
            family_tag(ExecutionFamilyV3::MetaRequireCertified),
            family_tag(ExecutionFamilyV3::RuleProgram),
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
            policy_family_tag(ExecutionPolicyFamilyV3::Relational),
            policy_family_tag(ExecutionPolicyFamilyV3::Graph),
            policy_family_tag(ExecutionPolicyFamilyV3::Check),
            policy_family_tag(ExecutionPolicyFamilyV3::Presentation),
            policy_family_tag(ExecutionPolicyFamilyV3::RuleProgram),
            policy_family_tag(ExecutionPolicyFamilyV3::MetaRequireCertified),
            policy_family_tag(ExecutionPolicyFamilyV3::Composition),
        ]);
        assert_distinct_tags([
            cardinality_tag(ExecutionCardinalityV3::Set),
            cardinality_tag(ExecutionCardinalityV3::Bag),
            cardinality_tag(ExecutionCardinalityV3::Sequence),
        ]);
        assert_distinct_tags([
            sort_direction_tag(ExecutionSortDirectionV3::Asc),
            sort_direction_tag(ExecutionSortDirectionV3::Desc),
        ]);
        assert_distinct_tags([
            null_order_tag(ExecutionNullOrderV3::First),
            null_order_tag(ExecutionNullOrderV3::Last),
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
            join_mode_tag(ExecutionJoinModeV3::Inner),
            join_mode_tag(ExecutionJoinModeV3::Cross),
        ]);
        assert_distinct_tags([
            source_kind_tag(ExecutionSourceKindV3::Relation),
            source_kind_tag(ExecutionSourceKindV3::View),
            source_kind_tag(ExecutionSourceKindV3::Composition),
        ]);
        assert_distinct_tags([
            scalar_tag(ExecutionScalarTypeV3::Bool),
            scalar_tag(ExecutionScalarTypeV3::Int),
            scalar_tag(ExecutionScalarTypeV3::Float),
            scalar_tag(ExecutionScalarTypeV3::String),
            scalar_tag(ExecutionScalarTypeV3::Duration),
            scalar_tag(ExecutionScalarTypeV3::Location),
            scalar_tag(ExecutionScalarTypeV3::Path),
            scalar_tag(ExecutionScalarTypeV3::RepoPath),
            scalar_tag(ExecutionScalarTypeV3::EntityRef),
            scalar_tag(ExecutionScalarTypeV3::SymbolRef),
            scalar_tag(ExecutionScalarTypeV3::RuleRef),
            scalar_tag(ExecutionScalarTypeV3::SourceScope),
            scalar_tag(ExecutionScalarTypeV3::ProofTree),
        ]);
        assert_distinct_tags([
            logical_quantifier_tag(ExecutionLogicalQuantifierV3::Exists),
            logical_quantifier_tag(ExecutionLogicalQuantifierV3::Forall),
        ]);
        assert_distinct_tags([
            exists_polarity_tag(ExecutionExistsPolarityV3::Exists),
            exists_polarity_tag(ExecutionExistsPolarityV3::NotExists),
        ]);
        assert_distinct_tags([
            graph_kind_tag(ExecutionGraphKindV3::Reachability),
            graph_kind_tag(ExecutionGraphKindV3::Flow),
            graph_kind_tag(ExecutionGraphKindV3::Slice),
            graph_kind_tag(ExecutionGraphKindV3::Taint),
            graph_kind_tag(ExecutionGraphKindV3::Path),
            graph_kind_tag(ExecutionGraphKindV3::PointsTo),
            graph_kind_tag(ExecutionGraphKindV3::Summary),
        ]);
        assert_distinct_tags([
            graph_direction_tag(ExecutionGraphDirectionV3::Forward),
            graph_direction_tag(ExecutionGraphDirectionV3::Backward),
            graph_direction_tag(ExecutionGraphDirectionV3::Both),
            graph_direction_tag(ExecutionGraphDirectionV3::Undirected),
        ]);
        assert_distinct_tags([
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Source),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Sink),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Anchor),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Reference),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Pointee),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Callable),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Via),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Through),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Sanitizer),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Scope),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Window),
            graph_reference_role_tag(ExecutionGraphReferenceRoleV3::Frontier),
        ]);
        assert_distinct_tags([
            graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Entity),
            graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Callable),
            graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Scope),
            graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Window),
            graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Frontier),
            graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Sanitizer),
            graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Pattern),
            graph_reference_kind_tag(ExecutionGraphReferenceKindV3::Snapshot),
            graph_reference_kind_tag(ExecutionGraphReferenceKindV3::ExportSource),
        ]);
        assert_distinct_tags([
            graph_relation_shape_tag(ExecutionGraphRelationShapeV3::Acyclic),
            graph_relation_shape_tag(ExecutionGraphRelationShapeV3::Cyclic),
        ]);
        assert_distinct_tags([
            graph_dedup_identity_tag(ExecutionGraphDedupIdentityV3::Endpoints),
            graph_dedup_identity_tag(ExecutionGraphDedupIdentityV3::NodeSequence),
            graph_dedup_identity_tag(ExecutionGraphDedupIdentityV3::NodeSet),
        ]);
        assert_distinct_tags([
            graph_witness_tie_break_tag(ExecutionGraphWitnessTieBreakV3::ShortestThenLexical),
            graph_witness_tie_break_tag(ExecutionGraphWitnessTieBreakV3::Lexical),
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

    fn binding(name: &str) -> ExecutionBindingV3 {
        ExecutionBindingV3 {
            source_id: SourceIdV2::from_raw(0),
            owner_node: 7,
            ordinal: 0,
            name: name.to_owned(),
        }
    }

    fn schema() -> ExecutionSchemaV3 {
        ExecutionSchemaV3 {
            fields: Box::new([ExecutionFieldV3 {
                name: "a".to_owned(),
                value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int),
            }]),
            cardinality: ExecutionCardinalityV3::Set,
            partition_keys: Box::new([]),
            order: Box::new([]),
        }
    }

    fn root() -> ExecutionRelationalOperatorV3 {
        ExecutionRelationalOperatorV3 {
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

    fn literal_expression(id: SourceQualifiedNodeIdV2, value: i128) -> ExecutionExpressionV3 {
        ExecutionExpressionV3 {
            id,
            value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int),
            shape: ExecutionExpressionShapeV3::Literal(ExecutionLiteralV3::Int(value)),
        }
    }

    fn operator(symbol: &str, overload_id: u32) -> ExecutionOperatorV3 {
        ExecutionOperatorV3 {
            symbol: symbol.to_owned(),
            overload_id,
        }
    }

    fn contract_with_shape(shape: ExecutionExpressionShapeV3) -> ExecutionDigestV3 {
        let mut item = relation_item();
        item.relation.as_mut().expect("relation").expressions = Box::new([ExecutionExpressionV3 {
            id: key(0, 2),
            value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int),
            shape,
        }]);
        contract_of(vec![item])
    }

    fn assert_all_digests_distinct(digests: &[ExecutionDigestV3]) {
        assert!(
            !digests.is_empty(),
            "an empty digest set cannot witness distinct encodings"
        );
        let mut deduped = digests.to_vec();
        deduped.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
        deduped.dedup();
        assert_eq!(
            deduped.len(),
            digests.len(),
            "two DTO arms share one encoding"
        );
    }

    fn relation() -> ExecutionRelationV3 {
        ExecutionRelationV3 {
            item_id: item(0, 1),
            root: root(),
            expressions: Box::new([literal_expression(key(0, 2), 1)]),
        }
    }

    fn plan() -> ExecutionPlanV3 {
        ExecutionPlanV3 {
            authority: ExecutionAuthorityV3::FrontendChecked,
            determinism: ExecutionDeterminismV3::Deterministic,
            evidence: ExecutionEvidenceV3::None,
            effect: ExecutionEffectV3::ReadOnly,
            policy_family: ExecutionPolicyFamilyV3::Relational,
        }
    }

    fn relation_item() -> ExecutionItemV3 {
        ExecutionItemV3 {
            item_id: item(0, 1),
            family: ExecutionFamilyV3::Relation,
            relation: Some(relation()),
            composition: None,
            plan: plan(),
        }
    }

    fn composition_item(binding_schema: Option<ExecutionSchemaV3>) -> ExecutionItemV3 {
        ExecutionItemV3 {
            item_id: item(0, 9),
            family: ExecutionFamilyV3::Composition,
            relation: None,
            composition: Some(ExecutionCompositionV3 {
                bindings: Box::new([ExecutionCompositionBindingV3 {
                    binding: binding("q"),
                    producer_item_id: item(0, 1),
                    producer_family: ExecutionFamilyV3::Relation,
                    schema: binding_schema,
                    relation: None,
                    producer: ExecutionCompositionProducerV3::Relation,
                }]),
                with_binding: binding("q"),
                body_item_id: item(0, 1),
                body_family: ExecutionFamilyV3::Relation,
                body_relation: None,
                body: ExecutionCompositionBodyV3::Relation,
            }),
            plan: plan(),
        }
    }

    fn graph_query(kind: ExecutionGraphKindV3) -> ExecutionGraphQueryV3 {
        ExecutionGraphQueryV3 {
            kind,
            references: Box::new([]),
            depth: 1,
            direction: None,
            summary_facets: Box::new([]),
            relation_shape: ExecutionGraphRelationShapeV3::Cyclic,
            dedup_identity: ExecutionGraphDedupIdentityV3::NodeSequence,
            max_witnesses: 1,
            max_path_length: Some(1),
            witness_tie_break: ExecutionGraphWitnessTieBreakV3::ShortestThenLexical,
        }
    }

    #[test]
    fn composition_rejects_a_same_variant_body_projection_from_another_item() {
        let fixture = reference_composition_document_v2(ReferenceCompositionBodyV2::Relational)
            .expect("reference composition resolves");
        let composition = fixture.composition();
        let producer_projections = composition
            .bindings()
            .iter()
            .map(|_| ExecutionCompositionProducerProjectionV3::Sample)
            .collect();

        assert_eq!(
            project_composition(
                composition,
                producer_projections,
                ExecutionCompositionBodyProjectionV3::Relation(relation()),
            ),
            Err(ExecutionCompositionProjectionFailureV3::Body),
        );
    }

    #[test]
    fn composition_rejects_same_variant_graph_projections_from_other_items() {
        let fixture = reference_composition_document_v2(ReferenceCompositionBodyV2::Graph)
            .expect("reference graph composition resolves");
        let composition = fixture.composition();

        assert_eq!(
            project_composition(
                composition,
                vec![ExecutionCompositionProducerProjectionV3::Graph {
                    item_id: fixture.body_item_id(),
                    query: graph_query(ExecutionGraphKindV3::Path),
                }],
                ExecutionCompositionBodyProjectionV3::Graph {
                    item_id: fixture.body_item_id(),
                    query: graph_query(ExecutionGraphKindV3::Path),
                },
            ),
            Err(ExecutionCompositionProjectionFailureV3::Producer),
        );

        assert_eq!(
            project_composition(
                composition,
                vec![ExecutionCompositionProducerProjectionV3::Graph {
                    item_id: fixture.producer_item_ids()[0],
                    query: graph_query(ExecutionGraphKindV3::Path),
                }],
                ExecutionCompositionBodyProjectionV3::Graph {
                    item_id: fixture.producer_item_ids()[0],
                    query: graph_query(ExecutionGraphKindV3::Path),
                },
            ),
            Err(ExecutionCompositionProjectionFailureV3::Body),
        );
    }

    #[test]
    fn independent_graph_encoder_matches_the_canonical_substream() {
        fn write_u32(out: &mut Vec<u8>, value: u32) {
            out.extend_from_slice(&value.to_be_bytes());
        }
        fn write_len(out: &mut Vec<u8>, value: usize) {
            out.extend_from_slice(
                &u64::try_from(value)
                    .expect("test length fits")
                    .to_be_bytes(),
            );
        }
        fn write_text(out: &mut Vec<u8>, value: &str) {
            write_len(out, value.len());
            out.extend_from_slice(value.as_bytes());
        }
        fn independent_target(out: &mut Vec<u8>, target: &ExecutionGraphTargetV3) {
            match target {
                ExecutionGraphTargetV3::Host {
                    kind,
                    spelling,
                    entity_subtype,
                } => {
                    out.push(0);
                    out.push(match kind {
                        ExecutionGraphReferenceKindV3::Entity => 0,
                        ExecutionGraphReferenceKindV3::Callable => 1,
                        ExecutionGraphReferenceKindV3::Scope => 2,
                        ExecutionGraphReferenceKindV3::Window => 3,
                        ExecutionGraphReferenceKindV3::Frontier => 4,
                        ExecutionGraphReferenceKindV3::Sanitizer => 5,
                        ExecutionGraphReferenceKindV3::Pattern => 6,
                        ExecutionGraphReferenceKindV3::Snapshot => 7,
                        ExecutionGraphReferenceKindV3::ExportSource => 8,
                    });
                    write_text(out, spelling);
                    if let Some(subtype) = entity_subtype {
                        out.push(1);
                        write_text(out, subtype);
                    } else {
                        out.push(0);
                    }
                }
                ExecutionGraphTargetV3::Composition {
                    binding,
                    capability,
                } => {
                    out.push(1);
                    write_u32(out, binding.source_id().raw());
                    write_u32(out, binding.owner_node());
                    write_len(out, binding.ordinal());
                    write_text(out, binding.name());
                    out.push(match capability {
                        ExecutionGraphCapabilityV3::Entity => 0,
                        ExecutionGraphCapabilityV3::Scope => 1,
                    });
                }
            }
        }
        fn independent_encode(query: &ExecutionGraphQueryV3) -> Vec<u8> {
            let mut out = Vec::new();
            out.push(match query.kind() {
                ExecutionGraphKindV3::Reachability => 0,
                ExecutionGraphKindV3::Flow => 1,
                ExecutionGraphKindV3::Slice => 2,
                ExecutionGraphKindV3::Taint => 3,
                ExecutionGraphKindV3::Path => 4,
                ExecutionGraphKindV3::PointsTo => 5,
                ExecutionGraphKindV3::Summary => 6,
            });
            write_len(&mut out, query.references().len());
            for reference in query.references() {
                out.push(match reference.role() {
                    ExecutionGraphReferenceRoleV3::Source => 0,
                    ExecutionGraphReferenceRoleV3::Sink => 1,
                    ExecutionGraphReferenceRoleV3::Anchor => 2,
                    ExecutionGraphReferenceRoleV3::Reference => 3,
                    ExecutionGraphReferenceRoleV3::Pointee => 4,
                    ExecutionGraphReferenceRoleV3::Callable => 5,
                    ExecutionGraphReferenceRoleV3::Via => 6,
                    ExecutionGraphReferenceRoleV3::Through => 7,
                    ExecutionGraphReferenceRoleV3::Sanitizer => 8,
                    ExecutionGraphReferenceRoleV3::Scope => 9,
                    ExecutionGraphReferenceRoleV3::Window => 10,
                    ExecutionGraphReferenceRoleV3::Frontier => 11,
                });
                independent_target(&mut out, reference.target());
            }
            write_u32(&mut out, query.depth());
            if let Some(direction) = query.direction() {
                out.push(1);
                out.push(match direction {
                    ExecutionGraphDirectionV3::Forward => 0,
                    ExecutionGraphDirectionV3::Backward => 1,
                    ExecutionGraphDirectionV3::Both => 2,
                    ExecutionGraphDirectionV3::Undirected => 3,
                });
            } else {
                out.push(0);
            }
            write_len(&mut out, query.summary_facets().len());
            for facet in query.summary_facets() {
                write_text(&mut out, facet);
            }
            out.push(match query.relation_shape() {
                ExecutionGraphRelationShapeV3::Acyclic => 0,
                ExecutionGraphRelationShapeV3::Cyclic => 1,
            });
            out.push(match query.dedup_identity() {
                ExecutionGraphDedupIdentityV3::Endpoints => 0,
                ExecutionGraphDedupIdentityV3::NodeSequence => 1,
                ExecutionGraphDedupIdentityV3::NodeSet => 2,
            });
            write_u32(&mut out, query.max_witnesses());
            if let Some(max_path_length) = query.max_path_length() {
                out.push(1);
                write_u32(&mut out, max_path_length);
            } else {
                out.push(0);
            }
            out.push(match query.witness_tie_break() {
                ExecutionGraphWitnessTieBreakV3::ShortestThenLexical => 0,
                ExecutionGraphWitnessTieBreakV3::Lexical => 1,
            });
            out
        }

        let query = ExecutionGraphQueryV3 {
            kind: ExecutionGraphKindV3::Summary,
            references: Box::new([
                ExecutionGraphReferenceV3 {
                    role: ExecutionGraphReferenceRoleV3::Source,
                    target: ExecutionGraphTargetV3::Host {
                        kind: ExecutionGraphReferenceKindV3::Snapshot,
                        spelling: "repo@head".to_owned(),
                        entity_subtype: Some("function".to_owned()),
                    },
                },
                ExecutionGraphReferenceV3 {
                    role: ExecutionGraphReferenceRoleV3::Scope,
                    target: ExecutionGraphTargetV3::Composition {
                        binding: binding("prior"),
                        capability: ExecutionGraphCapabilityV3::Scope,
                    },
                },
            ]),
            depth: 7,
            direction: Some(ExecutionGraphDirectionV3::Undirected),
            summary_facets: Box::new(["calls".to_owned(), "types".to_owned()]),
            relation_shape: ExecutionGraphRelationShapeV3::Acyclic,
            dedup_identity: ExecutionGraphDedupIdentityV3::NodeSet,
            max_witnesses: 11,
            max_path_length: Some(13),
            witness_tie_break: ExecutionGraphWitnessTieBreakV3::Lexical,
        };
        let mut canonical = Vec::new();
        put_graph_query(&mut canonical, &query);
        assert_eq!(canonical, independent_encode(&query));
    }

    fn host_graph_target(spelling: &str) -> ExecutionGraphTargetV3 {
        ExecutionGraphTargetV3::Host {
            kind: ExecutionGraphReferenceKindV3::Entity,
            spelling: spelling.to_owned(),
            entity_subtype: None,
        }
    }

    fn contract_of(items: Vec<ExecutionItemV3>) -> ExecutionDigestV3 {
        ExecutionDocumentV3::new(SourceIdV2::from_raw(0), b"show models", items).contract_digest()
    }

    fn qualification_of(items: Vec<ExecutionItemV3>) -> ExecutionDigestV3 {
        ExecutionDocumentV3::new(SourceIdV2::from_raw(0), b"show models", items)
            .qualification_digest()
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
            .correlations = Box::new([ExecutionCorrelationV3 {
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
        mutated.relation.as_mut().expect("relation").root.schema = ExecutionSchemaV3 {
            fields: Box::new([ExecutionFieldV3 {
                name: "b".to_owned(),
                value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::String),
            }]),
            cardinality: ExecutionCardinalityV3::Bag,
            partition_keys: Box::new([key(0, 2)]),
            order: Box::new([ExecutionOrderKeyV3 {
                expression: key(0, 2),
                direction: ExecutionSortDirectionV3::Desc,
                null_order: ExecutionNullOrderV3::Last,
                value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::String),
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
            Box::new([literal_expression(key(0, 2), 2)]);
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
    fn composition_sample_scope_and_every_graph_query_field_move_contract() {
        let mut unscoped = composition_item(None);
        unscoped.composition.as_mut().expect("composition").bindings[0].producer =
            ExecutionCompositionProducerV3::Sample {
                query_text: "auth".to_owned(),
                mode: ExecutionSampleModeV3::Lexical,
                limit: 1,
                scope: None,
            };
        let mut scoped = unscoped.clone();
        let ExecutionCompositionProducerV3::Sample { scope, .. } =
            &mut scoped.composition.as_mut().expect("composition").bindings[0].producer
        else {
            panic!("sample")
        };
        *scope = Some(host_graph_target("repo"));
        assert_ne!(contract_of(vec![unscoped]), contract_of(vec![scoped]));

        let mut base = composition_item(None);
        base.composition.as_mut().expect("composition").bindings[0].producer =
            ExecutionCompositionProducerV3::Graph {
                query: graph_query(ExecutionGraphKindV3::Path),
            };
        let base_digest = contract_of(vec![base.clone()]);
        let mut mutations = Vec::new();

        let mut kind = base.clone();
        let ExecutionCompositionProducerV3::Graph { query } =
            &mut kind.composition.as_mut().expect("composition").bindings[0].producer
        else {
            panic!("graph")
        };
        query.kind = ExecutionGraphKindV3::Flow;
        mutations.push(kind);

        let mut references = base.clone();
        let ExecutionCompositionProducerV3::Graph { query } = &mut references
            .composition
            .as_mut()
            .expect("composition")
            .bindings[0]
            .producer
        else {
            panic!("graph")
        };
        query.references = Box::new([ExecutionGraphReferenceV3 {
            role: ExecutionGraphReferenceRoleV3::Source,
            target: host_graph_target("a"),
        }]);
        mutations.push(references);

        let mut depth = base.clone();
        let ExecutionCompositionProducerV3::Graph { query } =
            &mut depth.composition.as_mut().expect("composition").bindings[0].producer
        else {
            panic!("graph")
        };
        query.depth = 2;
        mutations.push(depth);

        let mut direction = base.clone();
        let ExecutionCompositionProducerV3::Graph { query } = &mut direction
            .composition
            .as_mut()
            .expect("composition")
            .bindings[0]
            .producer
        else {
            panic!("graph")
        };
        query.direction = Some(ExecutionGraphDirectionV3::Backward);
        mutations.push(direction);

        let mut facets = base.clone();
        let ExecutionCompositionProducerV3::Graph { query } =
            &mut facets.composition.as_mut().expect("composition").bindings[0].producer
        else {
            panic!("graph")
        };
        query.summary_facets = Box::new(["calls".to_owned()]);
        mutations.push(facets);

        let mut relation_shape = base.clone();
        let ExecutionCompositionProducerV3::Graph { query } = &mut relation_shape
            .composition
            .as_mut()
            .expect("composition")
            .bindings[0]
            .producer
        else {
            panic!("graph")
        };
        query.relation_shape = ExecutionGraphRelationShapeV3::Acyclic;
        mutations.push(relation_shape);

        let mut dedup_identity = base.clone();
        let ExecutionCompositionProducerV3::Graph { query } = &mut dedup_identity
            .composition
            .as_mut()
            .expect("composition")
            .bindings[0]
            .producer
        else {
            panic!("graph")
        };
        query.dedup_identity = ExecutionGraphDedupIdentityV3::Endpoints;
        mutations.push(dedup_identity);

        let mut witnesses = base.clone();
        let ExecutionCompositionProducerV3::Graph { query } = &mut witnesses
            .composition
            .as_mut()
            .expect("composition")
            .bindings[0]
            .producer
        else {
            panic!("graph")
        };
        query.max_witnesses = 2;
        mutations.push(witnesses);

        let mut max_path_length = base.clone();
        let ExecutionCompositionProducerV3::Graph { query } = &mut max_path_length
            .composition
            .as_mut()
            .expect("composition")
            .bindings[0]
            .producer
        else {
            panic!("graph")
        };
        query.max_path_length = Some(2);
        mutations.push(max_path_length);

        let mut witness_tie_break = base;
        let ExecutionCompositionProducerV3::Graph { query } = &mut witness_tie_break
            .composition
            .as_mut()
            .expect("composition")
            .bindings[0]
            .producer
        else {
            panic!("graph")
        };
        query.witness_tie_break = ExecutionGraphWitnessTieBreakV3::Lexical;
        mutations.push(witness_tie_break);

        for mutation in mutations {
            assert_ne!(contract_of(vec![mutation]), base_digest);
        }
    }

    #[test]
    fn composition_binding_identity_and_order_move_contract() {
        let mut renamed = composition_item(None);
        let composition = renamed.composition.as_mut().expect("composition");
        composition
            .bindings
            .first_mut()
            .expect("one binding")
            .binding
            .name = "renamed".to_owned();
        composition.with_binding.name = "renamed".to_owned();
        assert_ne!(
            contract_of(vec![renamed]),
            contract_of(vec![composition_item(None)])
        );

        let mut ordered = composition_item(None);
        let composition = ordered.composition.as_mut().expect("composition");
        let first = composition.bindings.first().expect("one binding").clone();
        let mut second = first.clone();
        second.binding.ordinal = 1;
        second.binding.name = "second".to_owned();
        second.producer_item_id = item(0, 3);
        composition.bindings = vec![first, second].into_boxed_slice();
        let ordered_digest = contract_of(vec![ordered.clone()]);
        ordered
            .composition
            .as_mut()
            .expect("composition")
            .bindings
            .swap(0, 1);
        assert_ne!(ordered_digest, contract_of(vec![ordered]));
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
        authority.plan.authority = ExecutionAuthorityV3::CertifiedRequirementValidated;
        assert_ne!(contract_of(vec![authority.clone()]), base_contract);
        assert_ne!(qualification_of(vec![authority]), base_qualification);
        let mut effect = relation_item();
        effect.plan.effect = ExecutionEffectV3::ExportOnly;
        assert_ne!(contract_of(vec![effect.clone()]), base_contract);
        assert_ne!(qualification_of(vec![effect]), base_qualification);
        let mut evidence = relation_item();
        evidence.plan.evidence = ExecutionEvidenceV3::ProofTree;
        assert_ne!(contract_of(vec![evidence.clone()]), base_contract);
        assert_ne!(qualification_of(vec![evidence]), base_qualification);
    }

    #[test]
    fn source_byte_mutation_moves_qualification_but_not_contract() {
        let base = ExecutionDocumentV3::new(
            SourceIdV2::from_raw(0),
            b"show models",
            vec![relation_item()],
        );
        let mutated = ExecutionDocumentV3::new(
            SourceIdV2::from_raw(0),
            b"show models ",
            vec![relation_item()],
        );
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
            Err(ExecutionClosureFailureV3::Dangling)
        );
    }

    #[test]
    fn closure_cross_source_reference_fails_closed() {
        let children = |_: SourceQualifiedNodeIdV2| Some(Vec::new());
        assert_eq!(
            close_expression_ids(SourceIdV2::from_raw(0), &[key(1, 1)], &children),
            Err(ExecutionClosureFailureV3::Foreign)
        );
    }

    #[test]
    fn value_type_projection_covers_every_constructible_arm() {
        use quanta_gqlang_v2_sema::{
            LocatedSpanV2, RecordFieldTypeV1, RecordTypeV1, SourceRegistrationV2,
            SourceRegistryViewV2, SpanV1,
        };
        // Every constructible scalar arm projects one-to-one; a swapped
        // `project_scalar` pair must fail here rather than stay green.
        for (scalar, expected) in [
            (ScalarTyV1::Bool, ExecutionScalarTypeV3::Bool),
            (ScalarTyV1::Int, ExecutionScalarTypeV3::Int),
            (ScalarTyV1::Float, ExecutionScalarTypeV3::Float),
            (ScalarTyV1::String, ExecutionScalarTypeV3::String),
            (ScalarTyV1::Duration, ExecutionScalarTypeV3::Duration),
            (ScalarTyV1::Location, ExecutionScalarTypeV3::Location),
            (ScalarTyV1::Path, ExecutionScalarTypeV3::Path),
            (ScalarTyV1::RepoPath, ExecutionScalarTypeV3::RepoPath),
            (ScalarTyV1::EntityRef, ExecutionScalarTypeV3::EntityRef),
            (ScalarTyV1::SymbolRef, ExecutionScalarTypeV3::SymbolRef),
            (ScalarTyV1::RuleRef, ExecutionScalarTypeV3::RuleRef),
            (ScalarTyV1::SourceScope, ExecutionScalarTypeV3::SourceScope),
            (ScalarTyV1::ProofTree, ExecutionScalarTypeV3::ProofTree),
        ] {
            assert_eq!(
                project_value_type(&ValueTypeV1::scalar(scalar.clone())),
                ExecutionValueTypeV3::Scalar(expected),
                "{scalar:?}"
            );
        }
        assert_eq!(
            project_value_type(&ValueTypeV1::null()),
            ExecutionValueTypeV3::Null
        );
        let tuple = ValueTypeV1::try_tuple(vec![ValueTypeV1::int(), ValueTypeV1::string()])
            .expect("tuple of scalars");
        assert_eq!(
            project_value_type(&tuple),
            ExecutionValueTypeV3::Tuple(Box::new([
                ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int),
                ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::String),
            ]))
        );
        let optional = ValueTypeV1::try_optional(ValueTypeV1::bool()).expect("optional scalar");
        assert_eq!(
            project_value_type(&optional),
            ExecutionValueTypeV3::Optional(Box::new(ExecutionValueTypeV3::Scalar(
                ExecutionScalarTypeV3::Bool
            )))
        );
        let collection = ValueTypeV1::try_collection(ValueTypeV1::int(), CardinalityV1::Sequence)
            .expect("sequence collection");
        assert_eq!(
            project_value_type(&collection),
            ExecutionValueTypeV3::Collection {
                element: Box::new(ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int)),
                cardinality: ExecutionCardinalityV3::Sequence,
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
            ExecutionValueTypeV3::Record(Box::new([ExecutionRecordFieldV3 {
                name: "id".to_owned(),
                value_type: ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int),
            }]))
        );
        // Union projection mirrors record arm-for-arm; its constructors are not
        // re-exported for downstream (no producer path builds unions), so the
        // union ENCODING is pinned by `composite_value_type_arms_encode_distinctly`
        // instead of round-tripping through the owner here.
    }

    #[test]
    fn composite_value_type_arms_encode_distinctly() {
        let scalar = ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int);
        let variants = [
            ExecutionValueTypeV3::Null,
            ExecutionValueTypeV3::Tuple(Box::new([scalar.clone()])),
            ExecutionValueTypeV3::Record(Box::new([ExecutionRecordFieldV3 {
                name: "id".to_owned(),
                value_type: scalar.clone(),
            }])),
            ExecutionValueTypeV3::Optional(Box::new(scalar.clone())),
            ExecutionValueTypeV3::Collection {
                element: Box::new(scalar.clone()),
                cardinality: ExecutionCardinalityV3::Sequence,
            },
            ExecutionValueTypeV3::Union(Box::new([ExecutionUnionVariantV3 {
                tag: "ok".to_owned(),
                value_type: scalar.clone(),
            }])),
            ExecutionValueTypeV3::Unknown,
        ];
        let mut digests = Vec::with_capacity(variants.len());
        for value_type in &variants {
            let mut item = relation_item();
            item.relation.as_mut().expect("relation").root.schema.fields =
                Box::new([ExecutionFieldV3 {
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
        determinism.plan.determinism = ExecutionDeterminismV3::NonDeterministic;
        assert_ne!(contract_of(vec![determinism]), base);
        let mut policy = relation_item();
        policy.plan.policy_family = ExecutionPolicyFamilyV3::Graph;
        assert_ne!(contract_of(vec![policy]), base);
    }

    #[test]
    fn every_expression_shape_and_call_target_has_a_distinct_encoding() {
        let lhs = key(0, 3);
        let rhs = key(0, 4);
        let op = operator("+", 7);
        let shapes = vec![
            ExecutionExpressionShapeV3::Literal(ExecutionLiteralV3::Int(1)),
            ExecutionExpressionShapeV3::Binding(binding("value")),
            ExecutionExpressionShapeV3::Path {
                base: lhs,
                segments: Box::new(["field".to_owned()]),
            },
            ExecutionExpressionShapeV3::Unary {
                operator: op.clone(),
                operand: lhs,
            },
            ExecutionExpressionShapeV3::Binary {
                operator: op.clone(),
                lhs,
                rhs,
            },
            ExecutionExpressionShapeV3::Call {
                target: ExecutionCallTargetV3::Builtin {
                    name: "len".to_owned(),
                },
                arguments: Box::new([Some(lhs), None]),
            },
            ExecutionExpressionShapeV3::Call {
                target: ExecutionCallTargetV3::Predicate {
                    name: "p".to_owned(),
                },
                arguments: Box::new([Some(lhs)]),
            },
            ExecutionExpressionShapeV3::Call {
                target: ExecutionCallTargetV3::HostPredicate {
                    fingerprint: "00ff".to_owned(),
                },
                arguments: Box::new([Some(lhs)]),
            },
            ExecutionExpressionShapeV3::Paren(lhs),
            ExecutionExpressionShapeV3::Record(Box::new([("field".to_owned(), lhs)])),
            ExecutionExpressionShapeV3::List(Box::new([lhs, rhs])),
            ExecutionExpressionShapeV3::BoolValue(lhs),
            ExecutionExpressionShapeV3::Compare {
                operator: op.clone(),
                lhs,
                rhs,
            },
            ExecutionExpressionShapeV3::And {
                operator: op.clone(),
                lhs,
                rhs,
            },
            ExecutionExpressionShapeV3::Or {
                operator: op.clone(),
                lhs,
                rhs,
            },
            ExecutionExpressionShapeV3::Not {
                operator: op,
                operand: lhs,
            },
            ExecutionExpressionShapeV3::LogicalQuantifier {
                kind: ExecutionLogicalQuantifierV3::Exists,
                binders: Box::new([binding("x")]),
                body: lhs,
            },
            ExecutionExpressionShapeV3::RelationExists {
                polarity: ExecutionExistsPolarityV3::Exists,
                relation_root: lhs,
                correlations: Box::new([ExecutionCorrelationV3 {
                    binding: binding("outer"),
                    reference: rhs,
                }]),
            },
        ];
        let digests: Vec<_> = shapes.into_iter().map(contract_with_shape).collect();
        assert_all_digests_distinct(&digests);
    }

    #[test]
    fn expression_payload_fields_move_the_contract_digest() {
        let lhs = key(0, 3);
        let rhs = key(0, 4);
        let changed = key(0, 5);
        let base_call = contract_with_shape(ExecutionExpressionShapeV3::Call {
            target: ExecutionCallTargetV3::Predicate {
                name: "p".to_owned(),
            },
            arguments: Box::new([Some(lhs), None]),
        });
        for shape in [
            ExecutionExpressionShapeV3::Call {
                target: ExecutionCallTargetV3::Predicate {
                    name: "q".to_owned(),
                },
                arguments: Box::new([Some(lhs), None]),
            },
            ExecutionExpressionShapeV3::Call {
                target: ExecutionCallTargetV3::Predicate {
                    name: "p".to_owned(),
                },
                arguments: Box::new([Some(changed), None]),
            },
            ExecutionExpressionShapeV3::Call {
                target: ExecutionCallTargetV3::Predicate {
                    name: "p".to_owned(),
                },
                arguments: Box::new([Some(lhs), Some(rhs)]),
            },
        ] {
            assert_ne!(contract_with_shape(shape), base_call);
        }

        let base_quantifier = contract_with_shape(ExecutionExpressionShapeV3::LogicalQuantifier {
            kind: ExecutionLogicalQuantifierV3::Exists,
            binders: Box::new([binding("x")]),
            body: lhs,
        });
        for shape in [
            ExecutionExpressionShapeV3::LogicalQuantifier {
                kind: ExecutionLogicalQuantifierV3::Forall,
                binders: Box::new([binding("x")]),
                body: lhs,
            },
            ExecutionExpressionShapeV3::LogicalQuantifier {
                kind: ExecutionLogicalQuantifierV3::Exists,
                binders: Box::new([binding("y")]),
                body: lhs,
            },
            ExecutionExpressionShapeV3::LogicalQuantifier {
                kind: ExecutionLogicalQuantifierV3::Exists,
                binders: Box::new([binding("x")]),
                body: changed,
            },
        ] {
            assert_ne!(contract_with_shape(shape), base_quantifier);
        }

        let base_exists = contract_with_shape(ExecutionExpressionShapeV3::RelationExists {
            polarity: ExecutionExistsPolarityV3::Exists,
            relation_root: lhs,
            correlations: Box::new([ExecutionCorrelationV3 {
                binding: binding("outer"),
                reference: rhs,
            }]),
        });
        for shape in [
            ExecutionExpressionShapeV3::RelationExists {
                polarity: ExecutionExistsPolarityV3::NotExists,
                relation_root: lhs,
                correlations: Box::new([ExecutionCorrelationV3 {
                    binding: binding("outer"),
                    reference: rhs,
                }]),
            },
            ExecutionExpressionShapeV3::RelationExists {
                polarity: ExecutionExistsPolarityV3::Exists,
                relation_root: changed,
                correlations: Box::new([ExecutionCorrelationV3 {
                    binding: binding("outer"),
                    reference: rhs,
                }]),
            },
            ExecutionExpressionShapeV3::RelationExists {
                polarity: ExecutionExistsPolarityV3::Exists,
                relation_root: lhs,
                correlations: Box::new([ExecutionCorrelationV3 {
                    binding: binding("changed"),
                    reference: rhs,
                }]),
            },
        ] {
            assert_ne!(contract_with_shape(shape), base_exists);
        }
    }

    #[test]
    fn join_source_composition_body_and_sample_mode_arms_are_digest_bound() {
        let mut join_digests = Vec::new();
        for mode in [
            None,
            Some(ExecutionJoinModeV3::Inner),
            Some(ExecutionJoinModeV3::Cross),
        ] {
            let mut candidate = relation_item();
            let root = &mut candidate.relation.as_mut().expect("relation").root;
            root.family = AlgebraFamilyV2::Join;
            root.join_mode = mode;
            join_digests.push(contract_of(vec![candidate]));
        }
        assert_all_digests_distinct(&join_digests);

        let mut source_digests = Vec::new();
        for authority in [
            ExecutionRelationAuthorityV3::Host,
            ExecutionRelationAuthorityV3::CanonicalWorkspaceFiles,
            ExecutionRelationAuthorityV3::CatalogView,
            ExecutionRelationAuthorityV3::CompositionBinding,
        ] {
            let mut candidate = relation_item();
            candidate.relation.as_mut().expect("relation").root.source = Some(ExecutionSourceV3 {
                authority,
                kind: ExecutionSourceKindV3::Relation,
                namespace: Some("ns".to_owned()),
                name: "items".to_owned(),
            });
            source_digests.push(contract_of(vec![candidate]));
        }
        assert_all_digests_distinct(&source_digests);

        let mut body_digests = Vec::new();
        for body in [
            ExecutionCompositionBodyV3::Relation,
            ExecutionCompositionBodyV3::Graph(graph_query(ExecutionGraphKindV3::Taint)),
            ExecutionCompositionBodyV3::Check,
            ExecutionCompositionBodyV3::Explain,
        ] {
            let mut candidate = composition_item(None);
            candidate.composition.as_mut().expect("composition").body = body;
            body_digests.push(contract_of(vec![candidate]));
        }
        assert_all_digests_distinct(&body_digests);

        let mut sample_digests = Vec::new();
        for mode in [
            ExecutionSampleModeV3::Lexical,
            ExecutionSampleModeV3::Semantic,
            ExecutionSampleModeV3::Hybrid,
        ] {
            let mut candidate = composition_item(None);
            let binding = candidate
                .composition
                .as_mut()
                .expect("composition")
                .bindings
                .first_mut()
                .expect("one binding");
            binding.producer = ExecutionCompositionProducerV3::Sample {
                query_text: "auth".to_owned(),
                mode,
                limit: 7,
                scope: None,
            };
            sample_digests.push(contract_of(vec![candidate]));
        }
        assert_all_digests_distinct(&sample_digests);
    }
    fn authored_projector_facts(source: &str) -> SemanticDocumentV2 {
        use quanta_gqlang_v2_sema::{
            resolve_checked_document_v2, CheckedSyntaxDocumentV2, ClosedFrontendAuthorityBuilderV2,
            FrontendAuthorityLocationV2, FrontendAuthoritySourceV2, FrontendBudgetV2,
            FrontendCallableParameterInputV2, FrontendCatalogAssetInputV2,
            FrontendCatalogPayloadV2, FrontendLanguageVersionV2, FrontendRelationAuthorityV2,
            FrontendRelationCardinalityV2, FrontendRelationFieldInputV2, FrontendRelationInputV2,
            FrontendScalarTypeV2,
        };
        let location = FrontendAuthorityLocationV2 {
            source_id: SourceIdV2::from_raw(1),
            start: 0,
            end: 1,
        };
        let budget = FrontendBudgetV2::standard_v2().expect("standard budget");
        let mut builder = ClosedFrontendAuthorityBuilderV2::try_new(
            budget,
            vec![FrontendAuthoritySourceV2 {
                source_id: location.source_id,
                source_len: 64,
            }],
            "app".to_owned(),
            location,
            FrontendLanguageVersionV2 {
                major: 1,
                minor: 0,
                patch: 0,
            },
        )
        .expect("closed builder");
        builder.add_relation(FrontendRelationInputV2 {
            authority: FrontendRelationAuthorityV2::Host,
            provenance_namespace: "host".to_owned(),
            qualified_name: "app::records".to_owned(),
            lookup_name: "records".to_owned(),
            fields: vec![FrontendRelationFieldInputV2 {
                name: "id".to_owned(),
                scalar_type: FrontendScalarTypeV2::Int,
                optional: false,
                location,
            }],
            cardinality: FrontendRelationCardinalityV2::Bag,
            provenance: "projector test".to_owned(),
            location,
        });
        builder.add_catalog_asset(FrontendCatalogAssetInputV2 {
            module_name: "app".to_owned(),
            local_name: "host_pred".to_owned(),
            payload: FrontendCatalogPayloadV2::Rule {
                parameters: vec![FrontendCallableParameterInputV2 {
                    name: "value".to_owned(),
                    scalar_type: FrontendScalarTypeV2::Int,
                    optional: false,
                    location,
                }],
                result: FrontendScalarTypeV2::Bool,
            },
            provenance: "projector test".to_owned(),
            location,
        });
        let seed = builder.finish().expect("closed fixture seed");
        // A Rule catalog payload cannot satisfy a source predicate call. The
        // curated builder has no Predicate input, so seal that test authority
        // through the modules owner with the SAME registry and budget profile.
        let authority = {
            use quanta_gqlang_v2_modules::{
                CallableKindV2, CurrentModuleSelectionInputV2, ExternalCatalogAssetInputV2,
                ExternalCatalogPayloadInputV2, LocatedParameterInputV2, ModuleAuthorityBuilderV2,
                ModuleIdV2, ModuleNodeInputV2, SignatureInputV2, VersionV2,
            };
            use quanta_gqlang_v2_sema::{ClosedFrontendAuthorityV2, LocatedSpanV2, SpanV1};
            let host = seed.modules().source_registry().clone();
            let checked_location = host
                .try_check_location(LocatedSpanV2 {
                    source_id: SourceIdV2::from_raw(1),
                    span: SpanV1::new(0, 1),
                })
                .expect("checked fixture location");
            let effects = seed
                .modules()
                .external_catalog()
                .iter()
                .find_map(|asset| {
                    if let quanta_gqlang_v2_modules::ExternalCatalogPayloadV2::Rule(signature) =
                        asset.payload()
                    {
                        Some(signature.effects())
                    } else {
                        None
                    }
                })
                .expect("seed callable effects");
            let module = ModuleIdV2::try_new("app", checked_location.clone()).expect("app module");
            let mut modules = ModuleAuthorityBuilderV2::try_new(
                seed.budget()
                    .authority_profile()
                    .expect("authority profile"),
                host,
            )
            .expect("modules builder");
            modules
                .try_add_module(ModuleNodeInputV2 {
                    module_id: module.clone(),
                    location: checked_location.clone(),
                })
                .expect("app module added");
            modules
                .try_add_external(ExternalCatalogAssetInputV2 {
                    module_name: "app".to_owned(),
                    local_name: "host_pred".to_owned(),
                    kind_token: "predicate".to_owned(),
                    typed_payload: ExternalCatalogPayloadInputV2::Predicate(SignatureInputV2 {
                        parameters: vec![LocatedParameterInputV2 {
                            name: "value".to_owned(),
                            value_type: ValueTypeV1::int(),
                            location: checked_location.clone(),
                        }],
                        result: ValueTypeV1::bool(),
                        callable_kind: CallableKindV2::Predicate,
                        effects,
                        location: checked_location.clone(),
                    }),
                    provenance: "projector test".to_owned(),
                    location: checked_location.clone(),
                    dependencies: Vec::new(),
                })
                .expect("host predicate added");
            let modules = modules
                .finish(
                    CurrentModuleSelectionInputV2 {
                        module_id: module,
                        location: checked_location,
                    },
                    VersionV2::new(1, 0, 0),
                )
                .expect("predicate module authority seals");
            ClosedFrontendAuthorityV2::try_new(
                modules,
                seed.relations().clone(),
                seed.graph_references().clone(),
                seed.budget().clone(),
            )
            .expect("shared registry and profile authority seals")
        };
        let parsed = quanta_gqlang_v2_surface::parse_source_document_v2(
            SourceIdV2::from_raw(9),
            source,
            &quanta_gqlang_v2_surface::ParseBudgetV2::standard_v2(),
        )
        .expect("authored projector source parses");
        let checked = CheckedSyntaxDocumentV2::try_from_hir(parsed.hir).expect("checked HIR");
        resolve_checked_document_v2(checked, &authority).expect("authored source resolves")
    }

    #[test]
    fn quantified_shape_retains_kind_binder_identities_and_body() {
        use quanta_gqlang_v2_sema::{ResolvedBindingKindV2, ResolvedValueBindingV2};
        // Logical quantifiers are currently rule-only, and source rule members
        // issue no Execution item. Pin this private projector's full owner-fact
        // payload without claiming an active source execution path.
        let first = ResolvedValueBindingV2::try_new(
            SourceIdV2::from_raw(9),
            HirNodeIdV1::from_raw(71),
            2,
            ResolvedBindingKindV2::Quantified,
            "left",
        )
        .expect("first quantified identity");
        let second = ResolvedValueBindingV2::try_new(
            SourceIdV2::from_raw(9),
            HirNodeIdV1::from_raw(72),
            3,
            ResolvedBindingKindV2::Quantified,
            "right",
        )
        .expect("second quantified identity");
        for (kind, expected_kind) in [
            (
                LogicalQuantifierKindV2::Exists,
                ExecutionLogicalQuantifierV3::Exists,
            ),
            (
                LogicalQuantifierKindV2::Forall,
                ExecutionLogicalQuantifierV3::Forall,
            ),
        ] {
            assert_eq!(
                project_expression_shape(&TypedExprShapeV2::LogicalQuantifier {
                    kind,
                    binders: vec![first.clone(), second.clone()],
                    body: key(9, 83),
                }),
                ExecutionExpressionShapeV3::LogicalQuantifier {
                    kind: expected_kind,
                    binders: Box::new([
                        ExecutionBindingV3 {
                            source_id: SourceIdV2::from_raw(9),
                            owner_node: 71,
                            ordinal: 2,
                            name: "left".to_owned(),
                        },
                        ExecutionBindingV3 {
                            source_id: SourceIdV2::from_raw(9),
                            owner_node: 72,
                            ordinal: 3,
                            name: "right".to_owned(),
                        },
                    ]),
                    body: key(9, 83),
                },
            );
        }
    }

    #[test]
    fn authored_exists_shape_retains_polarity_nested_identity_and_capture() {
        for (source, expected_polarity) in [
            (
                "from records r | where exists (from records s | where s.id = r.id)",
                ExecutionExistsPolarityV3::Exists,
            ),
            (
                "from records r | where not exists (from records s | where s.id = r.id)",
                ExecutionExistsPolarityV3::NotExists,
            ),
        ] {
            let document = authored_projector_facts(source);
            let shapes = document
                .expressions()
                .filter_map(|(_, expression)| {
                    if let TypedExprShapeV2::RelationExists {
                        relation_root,
                        correlations,
                        ..
                    } = expression.shape()
                    {
                        Some((expression.shape(), *relation_root, correlations))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            let [(shape, relation_root, captures)] = shapes.as_slice() else {
                panic!("one authored EXISTS semantic fact expected");
            };
            let [capture] = captures.as_slice() else {
                panic!("one authored outer-r capture expected");
            };
            assert_eq!(capture.binding().canonical_name(), "r");
            assert_eq!(relation_root.source_id, SourceIdV2::from_raw(9));
            assert!(document.nested_relation(*relation_root).is_some());
            // EXISTS is lowered to SemiJoin/AntiJoin on the active source path;
            // its typed expression shape currently has no retained DTO root.
            // This checks the dormant shape projector against sealed source facts.
            assert_eq!(
                project_expression_shape(shape),
                ExecutionExpressionShapeV3::RelationExists {
                    polarity: expected_polarity,
                    relation_root: *relation_root,
                    correlations: Box::new([ExecutionCorrelationV3 {
                        binding: ExecutionBindingV3 {
                            source_id: SourceIdV2::from_raw(9),
                            owner_node: capture.binding().target_node().raw(),
                            ordinal: capture.binding().ordinal(),
                            name: "r".to_owned(),
                        },
                        reference: capture.reference(),
                    }]),
                },
            );
        }
    }

    #[test]
    fn authored_local_and_host_calls_retain_target_identity_and_argument_keys() {
        let source = "rule local_pred : x if x = 1; rule caller : x if x = 1 and local_pred(x) and host_pred(x)";
        let document = authored_projector_facts(source);
        let mut local_calls = 0;
        let mut host_calls = 0;
        for (_, expression) in document.expressions() {
            let TypedExprShapeV2::Call { target, arguments } = expression.shape() else {
                continue;
            };
            let [TypedCallArgumentV2::Expression(argument)] = arguments.as_slice() else {
                panic!("each authored predicate takes exactly one expression argument");
            };
            assert_eq!(argument.source_id, SourceIdV2::from_raw(9));
            let argument_fact = document
                .expression(*argument)
                .expect("authored argument fact");
            let TypedExprShapeV2::Binding(binding) = argument_fact.shape() else {
                panic!("the authored argument is the caller's x binding");
            };
            assert_eq!(binding.canonical_name(), "x");
            let expected_target = match target {
                ResolvedCallTargetV2::Predicate(_) => {
                    local_calls += 1;
                    ExecutionCallTargetV3::Predicate {
                        name: "local_pred".to_owned(),
                    }
                }
                ResolvedCallTargetV2::HostPredicate(_) => {
                    host_calls += 1;
                    // Independent literal for prefix + len(app) + app + Predicate(1)
                    // + len(host_pred) + host_pred. Do not derive via hex_lower
                    // or canonical_bytes: both are part of the production seam.
                    ExecutionCallTargetV3::HostPredicate {
                        fingerprint: "67716c616e672d686f73742d7072656469636174652d7632000000000000000003617070010000000000000009686f73745f70726564".to_owned(),
                    }
                }
                ResolvedCallTargetV2::Builtin(_) => panic!("no authored builtin"),
            };
            assert_eq!(
                project_expression_shape(expression.shape()),
                ExecutionExpressionShapeV3::Call {
                    target: expected_target,
                    arguments: Box::new([Some(*argument)]),
                },
            );
        }
        assert_eq!((local_calls, host_calls), (1, 1));
    }
}
