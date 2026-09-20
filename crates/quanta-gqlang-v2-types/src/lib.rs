//! Checked frontend types and the node-identity type kernel.

mod builtin;
mod check;
mod diag;
mod env;
mod op;
mod ty;

pub use builtin::{
    AggregationClassV1, BuiltinEntryV2, BuiltinFamilyIdV2, BuiltinIdV2, BuiltinMetaV1, BuiltinRegistryV2,
    CallableSignatureV2, CostClassV1, DeterminismV1, ExpressionEffectsV2, MonotonicityV1,
    PurityV1,
};
pub use check::{
    ContextualExpectedTypeInputV2, ContextualTypeReasonV2, ExpressionTypeKernelV2,
    ExpressionTypeStateV2, PredicateCallableSignatureV2, PredicateIdV2,
    ResolvedCallableInputV2, ResolvedCallTargetV2, ResolvedTypeInputEntryV2,
    ResolvedTypeInputValueV2, ResolvedTypeInputsV2, SourceQualifiedNodeIdV2,
    TypedExpressionFactV2, TypedExpressionFactsV2, TypedLiteralFactV2, UnknownTypeCauseV2,
};
pub use diag::{TypeAuthorityDiagnosticV2, TypeDiagnosticV2};
pub use env::{
    ProvenanceNamespaceV2, RelationCatalogV2, RelationSourceIdV2, RelationSourceInputV2,
    RelationSourceV2,
};
pub use op::{BinaryOverloadV1, OperatorIdV2, OperatorRegistryV2, UnaryOverloadV1};
pub use ty::{
    CardinalityV1, CollectionTypeV1, FiniteF64V2, OptionalTypeV1, RecordFieldTypeV1,
    RecordFieldV1, RecordTypeV1, RelationTypeV1, ScalarTyV1, TupleTypeV1, TypeV1,
    TypedLiteralValueV2, UnionTypeV1, UnionVariantTypeV1, UnionVariantV1, ValueTypeV1,
    MAX_TYPE_DEPTH, MAX_TYPE_NODES,
};
