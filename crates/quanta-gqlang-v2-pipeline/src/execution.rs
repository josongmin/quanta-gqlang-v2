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

    /// Source-ordered executable items. Non-executable declarations are absent.
    #[must_use]
    pub fn items(&self) -> &[ExecutionItemV3] {
        &self.items
    }

    /// Exact executable item by semantic identity.
    #[must_use]
    pub fn item(&self, item_id: SemanticItemIdV2) -> Option<&ExecutionItemV3> {
        self.items.iter().find(|item| item.item_id == item_id)
    }

    /// Digest of the raw UTF-8 source bytes only. DTO changes never move it.
    #[must_use]
    pub const fn source_digest(&self) -> ExecutionDigestV3 {
        self.source_digest
    }

    /// Digest of the canonical DTO encoding only. Source-byte changes that do
    /// not alter the projected contract never move it.
    #[must_use]
    pub const fn contract_digest(&self) -> ExecutionDigestV3 {
        self.contract_digest
    }

    /// Digest binding the contract domain, the schema version, and both legs
    /// above. Any DTO or source-byte change moves it.
    #[must_use]
    pub const fn qualification_digest(&self) -> ExecutionDigestV3 {
        self.qualification_digest
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

/// One executable item with its immutable planning authority.
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
    Graph(ExecutionGraphKindV3),
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
    },
    Graph {
        kind: ExecutionGraphKindV3,
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

pub(crate) fn project_composition(
    document: &SemanticDocumentV2,
    composition: &ResolvedCompositionV2,
    body_relation: Option<ExecutionRelationV3>,
    producer_relations: Vec<Option<ExecutionRelationV3>>,
) -> ExecutionCompositionV3 {
    let bindings = composition
        .bindings()
        .iter()
        .zip(producer_relations)
        .map(|(entry, relation)| {
            let (family, producer) = match entry.producer() {
                ResolvedCompositionProducerV2::Query(_) => (
                    ExecutionFamilyV3::Relation,
                    ExecutionCompositionProducerV3::Relation,
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
                        ExecutionFamilyV3::Presentation,
                        ExecutionCompositionProducerV3::Sample {
                            query_text: query_text.clone(),
                            mode: project_sample_mode(*mode),
                            limit: *limit,
                        },
                    )
                }
                ResolvedCompositionProducerV2::Graph { graph, .. } => (
                    ExecutionFamilyV3::Graph,
                    ExecutionCompositionProducerV3::Graph {
                        kind: project_graph_kind(graph.kind()),
                    },
                ),
            };
            ExecutionCompositionBindingV3 {
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
            ExecutionFamilyV3::Relation,
            ExecutionCompositionBodyV3::Relation,
        ),
        ResolvedCompositionBodyV2::Graph(body) => (
            body.item_id(),
            ExecutionFamilyV3::Graph,
            ExecutionCompositionBodyV3::Graph(project_graph_kind(body.kind())),
        ),
        ResolvedCompositionBodyV2::Check(body) => (
            body.item_id(),
            ExecutionFamilyV3::Check,
            ExecutionCompositionBodyV3::Check,
        ),
        ResolvedCompositionBodyV2::Explain(body) => (
            body.item_id(),
            ExecutionFamilyV3::Presentation,
            ExecutionCompositionBodyV3::Explain,
        ),
    };
    let _ = document;
    ExecutionCompositionV3 {
        bindings,
        with_binding: project_binding(composition.with_target()),
        body_item_id,
        body_family,
        body_relation,
        body,
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
const CONTRACT_SCHEMA_VERSION_V3: u32 = 3;

/// Digest of the raw UTF-8 source bytes only.
fn digest_source_bytes(source: &[u8]) -> ExecutionDigestV3 {
    let mut bytes = Vec::new();
    put_str(&mut bytes, "qglang.execution.source-bytes.v3");
    put_u32(&mut bytes, CONTRACT_SCHEMA_VERSION_V3);
    put_usize(&mut bytes, source.len());
    bytes.extend_from_slice(source);
    ExecutionDigestV3(sha256(&bytes))
}

/// Digest of the canonical DTO encoding only.
fn digest_contract(source_id: SourceIdV2, items: &[ExecutionItemV3]) -> ExecutionDigestV3 {
    let mut bytes = Vec::new();
    put_str(&mut bytes, "qglang.execution.contract.v3");
    put_u32(&mut bytes, CONTRACT_SCHEMA_VERSION_V3);
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
    put_str(&mut bytes, "qglang.execution.qualification.v3");
    put_u32(&mut bytes, CONTRACT_SCHEMA_VERSION_V3);
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
        }
        ExecutionCompositionProducerV3::Graph { kind } => {
            put_u8(out, 2);
            put_u8(out, graph_kind_tag(*kind));
        }
    }
}
fn put_composition_body(out: &mut Vec<u8>, body: &ExecutionCompositionBodyV3) {
    match body {
        ExecutionCompositionBodyV3::Relation => put_u8(out, 0),
        ExecutionCompositionBodyV3::Graph(kind) => {
            put_u8(out, 1);
            put_u8(out, graph_kind_tag(*kind));
        }
        ExecutionCompositionBodyV3::Check => put_u8(out, 2),
        ExecutionCompositionBodyV3::Explain => put_u8(out, 3),
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
    reason = "contract digest mutation rails assert exact fixture shapes"
)]
mod tests {
    use super::*;
    use quanta_gqlang_v2_sema::HirNodeIdV1;

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
        // Golden pins: any canonical-encoding change (tag swap, field reorder,
        // domain edit) must update these deliberately, never silently.
        assert_eq!(
            document.source_digest().to_hex(),
            "a8b69361d9c0d24e3022914bf1fba6386643164dc53911d34f1115dbc6102480"
        );
        assert_eq!(
            document.contract_digest().to_hex(),
            "572f6156cc71d5a1f344832a3ed3a16ddfac8541219d838d0129da82844bc1f1"
        );
        assert_eq!(
            document.qualification_digest().to_hex(),
            "2f012948c36a5a807d5ec5498e9e6498560c37c428446d6a5ebd3d89e7641a54"
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
        assert_eq!(
            project_value_type(&ValueTypeV1::int()),
            ExecutionValueTypeV3::Scalar(ExecutionScalarTypeV3::Int)
        );
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
            ExecutionCompositionBodyV3::Graph(ExecutionGraphKindV3::Taint),
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
            };
            sample_digests.push(contract_of(vec![candidate]));
        }
        assert_all_digests_distinct(&sample_digests);
    }
}
