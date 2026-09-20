use crate::ty::ValueTypeV1;
use quanta_gqlang_v2_hir::{HirNodeIdV1, LocatedSpanV2};
use std::fmt;

const MAX_DIAGNOSTIC_EXCERPT_BYTES: usize = 128;

fn diagnostic_excerpt(value: &str) -> &str {
    if value.len() <= MAX_DIAGNOSTIC_EXCERPT_BYTES {
        return value;
    }
    let mut end = MAX_DIAGNOSTIC_EXCERPT_BYTES;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeDiagnosticV2 {
    InvalidTypeName {
        location: LocatedSpanV2,
    },
    DuplicateRecordField {
        name: String,
        primary: LocatedSpanV2,
        secondary: LocatedSpanV2,
    },
    DuplicateUnionTag {
        tag: String,
        primary: LocatedSpanV2,
        secondary: LocatedSpanV2,
    },
    IntegerOutOfRange {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
        text: String,
    },
    FloatNonFinite {
        location: LocatedSpanV2,
    },
    FloatUnderflow {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
        text: String,
    },
    MalformedLiteral {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    MissingNodeFact {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    DuplicateNodeFact {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    ForeignDocumentFact {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    ExtraNodeFact {
        node_id: HirNodeIdV1,
    },
    UnusedContextualInput {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    WrongNodeFactKind {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    InvalidResolvedIdentity,
    UnknownOperator {
        node_id: HirNodeIdV1,
        symbol: String,
        location: LocatedSpanV2,
    },
    UnknownBuiltin {
        node_id: HirNodeIdV1,
        name: String,
        location: LocatedSpanV2,
    },
    BadOverload {
        node_id: HirNodeIdV1,
        name: String,
        operands: Vec<ValueTypeV1>,
        location: LocatedSpanV2,
    },
    BadProjection {
        node_id: HirNodeIdV1,
        base: ValueTypeV1,
        field: String,
        location: LocatedSpanV2,
    },
    HeterogeneousCollection {
        node_id: HirNodeIdV1,
        expected: ValueTypeV1,
        actual: ValueTypeV1,
        location: LocatedSpanV2,
    },
    InvalidStarArgument {
        node_id: HirNodeIdV1,
        location: LocatedSpanV2,
    },
    RegistryDuplicateId {
        registry: &'static str,
        id: u32,
    },
    RelationUsedAsValue,
    TypeShapeLimit,
    RedundantOptional,
}

impl TypeDiagnosticV2 {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidTypeName { .. } => "types.invalid_type_name",
            Self::DuplicateRecordField { .. } => "types.duplicate_record_field",
            Self::DuplicateUnionTag { .. } => "types.duplicate_union_tag",
            Self::IntegerOutOfRange { .. } => "types.integer_out_of_range",
            Self::FloatNonFinite { .. } => "types.float_non_finite",
            Self::FloatUnderflow { .. } => "types.float_underflow",
            Self::MalformedLiteral { .. } => "types.malformed_literal",
            Self::MissingNodeFact { .. } => "types.missing_resolved_input",
            Self::DuplicateNodeFact { .. } => "types.duplicate_resolved_input",
            Self::ForeignDocumentFact { .. } => "types.foreign_document_input",
            Self::ExtraNodeFact { .. } => "types.extra_resolved_input",
            Self::UnusedContextualInput { .. } => "types.unused_contextual_input",
            Self::WrongNodeFactKind { .. } => "types.wrong_resolved_input_kind",
            Self::InvalidResolvedIdentity => "types.invalid_resolved_identity",
            Self::UnknownOperator { .. } => "types.unknown_operator",
            Self::UnknownBuiltin { .. } => "types.unknown_builtin",
            Self::BadOverload { .. } => "types.bad_overload",
            Self::BadProjection { .. } => "types.bad_projection",
            Self::HeterogeneousCollection { .. } => "types.heterogeneous_collection",
            Self::InvalidStarArgument { .. } => "types.invalid_star_argument",
            Self::RegistryDuplicateId { .. } => "types.registry_duplicate_id",
            Self::RelationUsedAsValue => "types.relation_used_as_value",
            Self::TypeShapeLimit => "types.type_shape_limit",
            Self::RedundantOptional => "types.redundant_optional",
        }
    }
}

impl fmt::Display for TypeDiagnosticV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateRecordField { name, .. } => {
                write!(
                    f,
                    "{}: duplicate record field `{}`",
                    self.code(),
                    diagnostic_excerpt(name)
                )
            }
            Self::DuplicateUnionTag { tag, .. } => {
                write!(
                    f,
                    "{}: duplicate union tag `{}`",
                    self.code(),
                    diagnostic_excerpt(tag)
                )
            }
            Self::IntegerOutOfRange { .. } => {
                write!(f, "{}: integer literal is outside i128", self.code())
            }
            Self::FloatNonFinite { .. } => {
                write!(f, "{}: float literal is non-finite", self.code())
            }
            Self::FloatUnderflow { .. } => write!(
                f,
                "{}: nonzero float literal underflowed to zero",
                self.code()
            ),
            Self::MalformedLiteral { .. } => {
                write!(f, "{}: malformed numeric literal", self.code())
            }
            Self::BadOverload { name, .. } => {
                write!(
                    f,
                    "{}: no overload for `{}`",
                    self.code(),
                    diagnostic_excerpt(name)
                )
            }
            Self::BadProjection { field, .. } => {
                write!(
                    f,
                    "{}: cannot project field `{}`",
                    self.code(),
                    diagnostic_excerpt(field)
                )
            }
            Self::InvalidTypeName { .. }
            | Self::MissingNodeFact { .. }
            | Self::DuplicateNodeFact { .. }
            | Self::ForeignDocumentFact { .. }
            | Self::ExtraNodeFact { .. }
            | Self::UnusedContextualInput { .. }
            | Self::WrongNodeFactKind { .. }
            | Self::InvalidResolvedIdentity
            | Self::UnknownOperator { .. }
            | Self::UnknownBuiltin { .. }
            | Self::HeterogeneousCollection { .. }
            | Self::InvalidStarArgument { .. }
            | Self::RegistryDuplicateId { .. } => f.write_str(self.code()),
            Self::RelationUsedAsValue | Self::TypeShapeLimit | Self::RedundantOptional => {
                f.write_str(self.code())
            }
        }
    }
}
impl std::error::Error for TypeDiagnosticV2 {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeAuthorityDiagnosticV2 {
    InvalidAuthorityBudget,
    ResourceExhausted {
        dimension: &'static str,
        limit: usize,
        observed: usize,
    },
    DuplicateRelationSource {
        name: String,
        primary: LocatedSpanV2,
        secondary: LocatedSpanV2,
    },
    DuplicateRelationSourceId {
        id: String,
        primary: LocatedSpanV2,
        secondary: LocatedSpanV2,
    },
    InvalidSourceLocation {
        location: LocatedSpanV2,
    },
    SourceRegistryMismatch {
        location: LocatedSpanV2,
    },
    InvalidCanonicalName {
        name: String,
        location: LocatedSpanV2,
    },
    InvalidRelationType(TypeDiagnosticV2),
}

impl TypeAuthorityDiagnosticV2 {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidAuthorityBudget => "types.invalid_authority_budget",
            Self::ResourceExhausted { .. } => "types.resource_exhausted",
            Self::DuplicateRelationSource { .. } => "types.duplicate_relation_source",
            Self::DuplicateRelationSourceId { .. } => "types.duplicate_relation_source_id",
            Self::InvalidSourceLocation { .. } => "types.invalid_source_location",
            Self::SourceRegistryMismatch { .. } => "types.source_registry_mismatch",
            Self::InvalidCanonicalName { .. } => "types.invalid_canonical_name",
            Self::InvalidRelationType(error) => error.code(),
        }
    }
}

impl fmt::Display for TypeAuthorityDiagnosticV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidAuthorityBudget => f.write_str(self.code()),
            Self::ResourceExhausted {
                dimension,
                limit,
                observed,
            } => write!(
                f,
                "{}: {dimension} limit {limit}, observed {observed}",
                self.code()
            ),
            Self::DuplicateRelationSource { name, .. } => write!(
                f,
                "{}: duplicate relation lookup name `{}`",
                self.code(),
                diagnostic_excerpt(name)
            ),
            Self::DuplicateRelationSourceId { id, .. } => {
                write!(
                    f,
                    "{}: duplicate relation source id `{}`",
                    self.code(),
                    diagnostic_excerpt(id)
                )
            }
            Self::InvalidCanonicalName { name, .. } => {
                write!(
                    f,
                    "{}: invalid canonical name `{}`",
                    self.code(),
                    diagnostic_excerpt(name)
                )
            }
            Self::InvalidSourceLocation { .. } | Self::SourceRegistryMismatch { .. } => {
                f.write_str(self.code())
            }
            Self::InvalidRelationType(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for TypeAuthorityDiagnosticV2 {}
