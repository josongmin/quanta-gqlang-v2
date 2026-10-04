//! Closed execution value and plan vocabulary.

use super::{
    hex_lower, AuthorityLaneV2, CardinalityV1, EffectClassV2, EvaluationDeterminismV2,
    EvidenceRequirementV2, ExecutionAuthorityV3, ExecutionClosureFailureV3, ExecutionDeterminismV3,
    ExecutionEffectV3, ExecutionEvidenceV3, ExecutionOperatorKindV3, ExecutionPlanV3,
    ExecutionPolicyFamilyV3, JoinModeV2, LogicalQuantifierKindV2, NullOrderV2, PlanPolicyV2,
    RelationExistsPolarityV2, ResolvedCallTargetV2, ScalarTyV1, SemanticPlanV2, SortDirectionV2,
    ValueTypeV1,
};

/// Canonical overload semantics for downstream execution. Not part of the encoded DTO.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExecutionOperatorDescriptorV3 {
    kind: ExecutionOperatorKindV3,
    operands: Box<[ExecutionValueTypeV3]>,
    result: ExecutionValueTypeV3,
}

impl ExecutionOperatorDescriptorV3 {
    #[must_use]
    pub const fn kind(&self) -> ExecutionOperatorKindV3 {
        self.kind
    }
    #[must_use]
    pub fn operand_types(&self) -> &[ExecutionValueTypeV3] {
        &self.operands
    }
    #[must_use]
    pub const fn result_type(&self) -> &ExecutionValueTypeV3 {
        &self.result
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExecutionOperatorDescriptorErrorV3 {
    InvalidCanonicalRegistry { reason: String },
}

/// Looks up an ID in the closed type-checker registry. Unknown IDs are explicit absence;
/// an invalid canonical registry is an error, never an invented overload.
pub fn execution_operator_descriptor_v3(
    raw_id: u32,
) -> Result<Option<ExecutionOperatorDescriptorV3>, ExecutionOperatorDescriptorErrorV3> {
    let descriptor = quanta_gqlang_v2_sema::operator_descriptor_v2(raw_id).map_err(|error| {
        ExecutionOperatorDescriptorErrorV3::InvalidCanonicalRegistry {
            reason: error.to_string(),
        }
    })?;
    Ok(descriptor.map(|descriptor| ExecutionOperatorDescriptorV3 {
        kind: descriptor.kind(),
        operands: descriptor
            .operand_types()
            .iter()
            .map(project_value_type)
            .collect(),
        result: project_value_type(descriptor.result_type()),
    }))
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

pub(super) const fn project_scalar(scalar: &ScalarTyV1) -> ExecutionScalarTypeV3 {
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
    pub(super) name: String,
    pub(super) value_type: ExecutionValueTypeV3,
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
    pub(super) tag: String,
    pub(super) value_type: ExecutionValueTypeV3,
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

pub(super) fn project_value_type(value: &ValueTypeV1) -> ExecutionValueTypeV3 {
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

pub(super) const fn project_cardinality(value: CardinalityV1) -> ExecutionCardinalityV3 {
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

pub(super) const fn project_sort_direction(value: SortDirectionV2) -> ExecutionSortDirectionV3 {
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

pub(super) const fn project_null_order(value: NullOrderV2) -> ExecutionNullOrderV3 {
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

pub(super) const fn project_join_mode(value: JoinModeV2) -> ExecutionJoinModeV3 {
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

pub(super) fn project_call_target(
    target: &ResolvedCallTargetV2,
) -> Result<ExecutionCallTargetV3, ExecutionClosureFailureV3> {
    Ok(match target {
        ResolvedCallTargetV2::Builtin(id) => ExecutionCallTargetV3::Builtin {
            name: registered_execution_symbol(id.name())?,
        },
        ResolvedCallTargetV2::Predicate(id) => ExecutionCallTargetV3::Predicate {
            name: id.as_str().to_owned(),
        },
        ResolvedCallTargetV2::HostPredicate(id) => ExecutionCallTargetV3::HostPredicate {
            fingerprint: hex_lower(id.canonical_bytes()),
        },
    })
}

/// Never turn a missing owner spelling into a success-shaped DTO identity.
pub(super) fn registered_execution_symbol(
    spelling: Option<&str>,
) -> Result<String, ExecutionClosureFailureV3> {
    spelling
        .map(str::to_owned)
        .ok_or(ExecutionClosureFailureV3::UnregisteredSymbol)
}

/// Closed logical-quantifier vocabulary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExecutionLogicalQuantifierV3 {
    Exists,
    Forall,
}

pub(super) const fn project_logical_quantifier(
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

pub(super) const fn project_exists_polarity(
    value: RelationExistsPolarityV2,
) -> ExecutionExistsPolarityV3 {
    match value {
        RelationExistsPolarityV2::Exists => ExecutionExistsPolarityV3::Exists,
        RelationExistsPolarityV2::NotExists => ExecutionExistsPolarityV3::NotExists,
    }
}

pub(super) fn project_plan(plan: &SemanticPlanV2) -> ExecutionPlanV3 {
    project_plan_parts(
        plan.authority_lane(),
        plan.evaluation_determinism(),
        plan.evidence(),
        plan.effect(),
        plan.policy(),
    )
}

pub(super) fn project_scalar_plan(
    plan: &quanta_gqlang_v2_planning::AdmittedScalarPlanV3<'_>,
) -> ExecutionPlanV3 {
    project_plan_parts(
        plan.authority_lane_v3(),
        plan.evaluation_determinism_v3(),
        plan.evidence_v3(),
        plan.effect_v3(),
        plan.policy_v3(),
    )
}

fn project_plan_parts(
    authority_lane: AuthorityLaneV2,
    evaluation_determinism: EvaluationDeterminismV2,
    evidence_requirement: EvidenceRequirementV2,
    effect_class: EffectClassV2,
    policy: &PlanPolicyV2,
) -> ExecutionPlanV3 {
    let authority = match authority_lane {
        AuthorityLaneV2::FrontendChecked => ExecutionAuthorityV3::FrontendChecked,
        AuthorityLaneV2::CertifiedRequirementValidated => {
            ExecutionAuthorityV3::CertifiedRequirementValidated
        }
    };
    let determinism = match evaluation_determinism {
        EvaluationDeterminismV2::Deterministic => ExecutionDeterminismV3::Deterministic,
        EvaluationDeterminismV2::NonDeterministic => ExecutionDeterminismV3::NonDeterministic,
    };
    let evidence = match evidence_requirement {
        EvidenceRequirementV2::None => ExecutionEvidenceV3::None,
        EvidenceRequirementV2::SourceLocation => ExecutionEvidenceV3::SourceLocation,
        EvidenceRequirementV2::WitnessPayload => ExecutionEvidenceV3::WitnessPayload,
        EvidenceRequirementV2::ProofTree => ExecutionEvidenceV3::ProofTree,
    };
    let effect = match effect_class {
        EffectClassV2::ReadOnly => ExecutionEffectV3::ReadOnly,
        EffectClassV2::ProofEvidenceOnly => ExecutionEffectV3::ProofEvidenceOnly,
        EffectClassV2::ExportOnly => ExecutionEffectV3::ExportOnly,
        EffectClassV2::MutationForbidden => ExecutionEffectV3::MutationForbidden,
        EffectClassV2::MutationRequiresGrant => ExecutionEffectV3::MutationRequiresGrant,
    };
    let policy_family = match policy {
        PlanPolicyV2::Relational { .. } => ExecutionPolicyFamilyV3::Relational,
        PlanPolicyV2::Graph(_) => ExecutionPolicyFamilyV3::Graph,
        PlanPolicyV2::Check(_) => ExecutionPolicyFamilyV3::Check,
        PlanPolicyV2::Presentation(_) => ExecutionPolicyFamilyV3::Presentation,
        PlanPolicyV2::Rule(_) => ExecutionPolicyFamilyV3::RuleProgram,
        PlanPolicyV2::MetaRequireCertified => ExecutionPolicyFamilyV3::MetaRequireCertified,
        PlanPolicyV2::Composition(_) => ExecutionPolicyFamilyV3::Composition,
        PlanPolicyV2::Command(_) => ExecutionPolicyFamilyV3::Command,
    };
    ExecutionPlanV3 {
        authority,
        determinism,
        evidence,
        effect,
        policy_family,
    }
}
