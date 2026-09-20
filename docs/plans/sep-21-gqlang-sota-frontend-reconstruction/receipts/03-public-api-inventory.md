# Ticket 03 Public API Inventory

Captured from `main@f9ec9543888954106bb7c8bf1dc6500a5e6d34f7` plus the
Ticket 03 dirty repair described by `03.md`.

Commands:

```text
cargo public-api -p quanta-gqlang-v2-types --simplified
cargo public-api -p quanta-gqlang-v2-modules --simplified
```

The complete simplified output is 1,320 lines for types and 1,560 lines for
modules. SHA-256 of those complete outputs is respectively
`f205b7f6ef4baf9144df4c07bad7b217c9d9b0b0a7dd1990f8f0d79169f05b40`
and `5985ffaf4775575429a60b811fea7998582d56e3572a379d3abb20a63638ea5f`.
The exact authority ingress signatures are pinned in Ticket 03's public API
block; this appendix inventories every exported type so a later diff cannot
hide a sibling authority universe behind aggregate counts.

## Types owner — 52 exported types

Enums (17):

```text
AggregationClassV1
CardinalityV1
ContextualTypeReasonV2
CostClassV1
DeterminismV1
ExpressionTypeStateV2
MonotonicityV1
PurityV1
ResolvedCallTargetV2
ResolvedCallableInputV2
ResolvedTypeInputValueV2
ScalarTyV1
TypeAuthorityDiagnosticV2
TypeDiagnosticV2
TypedLiteralFactV2
TypedLiteralValueV2
UnknownTypeCauseV2
```

Structs (35):

```text
AuthorityBudgetProfileV2
BuiltinFamilyIdV2
BuiltinIdV2
BuiltinRegistryV2
CollectionTypeV1
ContextualExpectedTypeInputV2
ExpressionEffectsV2
ExpressionTypeKernelV2
FiniteF64V2
OperatorIdV2
OperatorRegistryV2
OptionalTypeV1
ProvenanceNamespaceV2
RecordFieldTypeV1
RecordFieldV1
RecordTypeV1
RelationCatalogV2
RelationSourceIdV2
RelationSourceInputV2
RelationSourceV2
RelationTypeV1
ResolvedPredicateCallableV2
ResolvedPredicateIdV2
ResolvedRelationPredicateInputV2
ResolvedTypeInputEntryV2
ResolvedTypeInputsV2
SourceQualifiedNodeIdV2
TupleTypeV1
TypeV1
TypedExpressionFactV2
TypedExpressionFactsV2
UnionTypeV1
UnionVariantTypeV1
UnionVariantV1
ValueTypeV1
```

## Modules owner — 76 exported types

Enums (9):

```text
CallableKindV2
CatalogPackActivationV2
ExternalCatalogPayloadInputV2
ExternalCatalogPayloadV2
GateBindingV2
NamespaceKindV2
ProfileAdmissionV2
UsePackActivationV2
VisibilityV2
```

Structs (67):

```text
AmbiguousCandidateV2
CatalogAssetIdV2
CatalogAssetRefInputV2
ComposedModelFactV2
ComposedModelV2
ContentDigestV2
CurrentModuleSelectionInputV2
DocumentCatalogAssetInputV2
DocumentCatalogAuthorityV2
DocumentCatalogDependencyInputV2
DocumentCatalogInputV2
DocumentCatalogValidatorV2
ExternalCatalogAssetInputV2
ExternalCatalogAssetV2
ExternalCatalogDependencyInputV2
ExternalCatalogV2
ExternalPackContractV2
GateContractV2
LocatedDependencyInputV2
LocatedEdgeV2
LocatedExportInputV2
LocatedModelValueInputV2
LocatedParameterInputV2
LocatedSuiteInputV2
LocatedSuiteMemberInputV2
LockPinV2
LockReferenceInputV2
LockReferenceV2
LockfileV2
ModelAttributeInputV2
ModelFactInputV2
ModelPackInputV2
ModelSchemaInputV2
ModelSchemaV2
ModuleAuthorityBuilderV2
ModuleBudgetV2
ModuleDeclarationInputV2
ModuleDiagnosticV2
ModuleGraphBuilderV2
ModuleIdV2
ModuleImportInputV2
ModuleNodeInputV2
ModuleReexportInputV2
PackActivationInputV2
PackCoordinateV2
PackGraphV2
PackIdV2
PackManifestInputV2
PackManifestV2
ProfileAdmissionInputV2
RequireCertifiedInputV2
ResolvedBindingV2
ResolvedGraphV2
ResolvedModuleBindingsV2
ResourceExhaustionV2
SignatureInputV2
SignatureParameterV2
SignatureV2
ValidatedModuleAuthorityPartsV2
ValidatedPackActivationV2
ValidatedPackActivationsV2
ValidatedPackAuthorityV2
ValidatedPackGraphV2
ValidatedPackManifestV2
VerifiedLockSetV2
VersionRangeV2
VersionV2
```

## Negative inventory

The source scan has zero definitions for `SignatureInstanceV1`, public
`TypeV2`, a parallel module type universe, or public `Default` implementations
for `ValidatedPackActivationsV2` and `ResolvedModuleBindingsV2`. The one
`#[derive(Default)]` in the types source is private traversal state, not an
authority proof.
