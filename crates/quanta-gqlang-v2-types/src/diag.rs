use crate::ty::TypeV1;
use quanta_gqlang_v2_hir::{HirNodeIdV1, LocatedSpanV2};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeDiagnosticV2 {
    InvalidTypeName { location: LocatedSpanV2 },
    DuplicateRecordField { name: String, primary: LocatedSpanV2, secondary: LocatedSpanV2 },
    DuplicateUnionTag { tag: String, primary: LocatedSpanV2, secondary: LocatedSpanV2 },
    IntegerOutOfRange { node_id: HirNodeIdV1, location: LocatedSpanV2, text: String },
    FloatNonFinite { location: LocatedSpanV2 },
    FloatUnderflow { node_id: HirNodeIdV1, location: LocatedSpanV2, text: String },
    MissingNodeFact { node_id: HirNodeIdV1, location: LocatedSpanV2 },
    DuplicateNodeFact { node_id: HirNodeIdV1, location: LocatedSpanV2 },
    ForeignDocumentFact { node_id: HirNodeIdV1, location: LocatedSpanV2 },
    ExtraNodeFact { node_id: HirNodeIdV1 },
    UnknownOperator { node_id: HirNodeIdV1, symbol: String, location: LocatedSpanV2 },
    UnknownBuiltin { node_id: HirNodeIdV1, name: String, location: LocatedSpanV2 },
    BadOverload { node_id: HirNodeIdV1, name: String, operands: Vec<TypeV1>, location: LocatedSpanV2 },
    BadProjection { node_id: HirNodeIdV1, base: TypeV1, field: String, location: LocatedSpanV2 },
    HeterogeneousCollection { node_id: HirNodeIdV1, expected: TypeV1, actual: TypeV1, location: LocatedSpanV2 },
    InvalidStarArgument { node_id: HirNodeIdV1, location: LocatedSpanV2 },
    RegistryDuplicateId { registry: &'static str, id: u32 },
}

impl TypeDiagnosticV2 {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidTypeName { .. } => "types.invalid_name",
            Self::DuplicateRecordField { .. } => "types.duplicate_record_field",
            Self::DuplicateUnionTag { .. } => "types.duplicate_union_tag",
            Self::IntegerOutOfRange { .. } => "types.integer_out_of_range",
            Self::FloatNonFinite { .. } => "types.float_non_finite",
            Self::FloatUnderflow { .. } => "types.float_underflow",
            Self::MissingNodeFact { .. } => "types.missing_node_fact",
            Self::DuplicateNodeFact { .. } => "types.duplicate_node_fact",
            Self::ForeignDocumentFact { .. } => "types.foreign_document_fact",
            Self::ExtraNodeFact { .. } => "types.extra_node_fact",
            Self::UnknownOperator { .. } => "types.unknown_operator",
            Self::UnknownBuiltin { .. } => "types.unknown_builtin",
            Self::BadOverload { .. } => "types.bad_overload",
            Self::BadProjection { .. } => "types.bad_projection",
            Self::HeterogeneousCollection { .. } => "types.heterogeneous_collection",
            Self::InvalidStarArgument { .. } => "types.invalid_star_argument",
            Self::RegistryDuplicateId { .. } => "types.registry_duplicate_id",
        }
    }
}

impl fmt::Display for TypeDiagnosticV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}: {self:?}", self.code()) }
}
impl std::error::Error for TypeDiagnosticV2 {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeAuthorityDiagnosticV2 {
    DuplicateRelationSource { name: String, primary: LocatedSpanV2, secondary: LocatedSpanV2 },
    DuplicateRelationSourceId { id: u64, primary: LocatedSpanV2, secondary: LocatedSpanV2 },
}

impl TypeAuthorityDiagnosticV2 {
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::DuplicateRelationSource { .. } => "types.duplicate_relation_source",
            Self::DuplicateRelationSourceId { .. } => "types.duplicate_relation_source_id",
        }
    }
}

impl fmt::Display for TypeAuthorityDiagnosticV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}: {self:?}", self.code()) }
}
impl std::error::Error for TypeAuthorityDiagnosticV2 {}
