# 03 — Authority, Types, Modules, And Invalid States

Status: `CODE INTEGRATED; EXACT-SHA REPO-LOCAL QUALIFICATION AT 08e5ce5 (RECEIPT 09)`

This header supersedes older owner-only checkpoints. Types, Modules, and
Binder participate in the compiled source path and clean `08e5ce5`
qualification (Receipt 09). Historical mutation receipts were not rerun;
their original source pins remain in the owner receipt.

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Owned / To Close

F06's type carrier, F15, F19, F22, F23, F25, F27, F32, and F37.

## Goal

Create validated authority inputs and valid-by-construction type/module values
before the semantic compiler depends on them.

## Current-Head Reuse/Delete/Build Audit

| Class | Assets | Concrete action |
|---|---|---|
| `KEEP` | closed operator/builtin registries and their overload/effect metadata; module precedence, ambiguity, cycle, catalog, pack, lock, signature, and extension algorithms | retain these algorithms and their positive tests |
| `MIGRATE` | `TypeCheckerV1`, `TypeV1`, `TypeEnvV1`, `ModuleGraphBuilderV1`, the visibility projection behind `ResolvedGraphV1::binder_view`, `CatalogV1`, `PackGraphV1::validate`, `LockfileV1::verify_against` | make carriers/updates checked and diagnostics source-rich around the one existing implementation |
| `DELETE` | public invalid type constructors, silent overwrite mutation, raw pack/lock authority admission, orphan query metadata carriers | delete once all current callers use checked values; query metadata has no production caller and is removed with owner-only tests |
| `NEW` | stable IDs, relation-only authority, proof wrappers, source-rich duplicate diagnostics, module budget | add only missing evidence/carriers; proof wrappers call the existing validators |

Do not write a second overload resolver, module resolver, cycle detector, pack
validator, or lock verifier. The V2 transition refactors shared internals first;
it does not copy V1 algorithms into parallel files.

## Write Set

Types lane:

1. `crates/quanta-gqlang-v2-types/src/lib.rs`
2. `crates/quanta-gqlang-v2-types/src/ty.rs`
3. `crates/quanta-gqlang-v2-types/src/env.rs`
4. `crates/quanta-gqlang-v2-types/src/check.rs`
5. `crates/quanta-gqlang-v2-types/src/builtin.rs`
6. `crates/quanta-gqlang-v2-types/src/op.rs`
7. `crates/quanta-gqlang-v2-types/src/diag.rs`
8. `crates/quanta-gqlang-v2-types/src/solver.rs`
9. `crates/quanta-gqlang-v2-types/tests/v3_rule_type_solver.rs`
10. `crates/quanta-gqlang-v2-types/tests/v3_types_check_edges.rs`

Modules lane:

1. `crates/quanta-gqlang-v2-modules/src/lib.rs`
2. `crates/quanta-gqlang-v2-modules/src/graph.rs`
3. `crates/quanta-gqlang-v2-modules/src/catalog.rs`
4. `crates/quanta-gqlang-v2-modules/src/manifest.rs`
5. `crates/quanta-gqlang-v2-modules/src/lockfile.rs`
6. `crates/quanta-gqlang-v2-modules/src/signature.rs`
7. `crates/quanta-gqlang-v2-modules/src/diagnostic.rs`
8. `crates/quanta-gqlang-v2-modules/src/budget.rs`
9. `crates/quanta-gqlang-v2-modules/Cargo.toml` — integrator only, adds the
   final `modules -> types` edge pinned by `TARGET_ARCHITECTURE.md`
10. `crates/quanta-gqlang-v2-modules/src/metadata.rs`
11. owner-local types/modules test support modules when consumed by at least
    two touched tests
12. `crates/quanta-gqlang-v2-modules/src/version.rs`
13. `crates/quanta-gqlang-v2-modules/src/visibility.rs`
14. `crates/quanta-gqlang-v2-modules/src/extension.rs` when typed external
    signatures replace opaque extension tags
15. `crates/quanta-gqlang-v2-modules/src/authority.rs`
16. `crates/quanta-gqlang-v2-modules/tests/v3_document_predeclaration.rs`

Receipt:

1. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/03.md`
2. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/03-public-api-inventory.md`
3. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/03-test-disposition.md`

Shared source-bounds carrier extension:

1. `crates/quanta-gqlang-v2-hir/src/span.rs`
2. `crates/quanta-gqlang-v2-hir/src/lib.rs`

## Implementation

### Types

1. Migrate the canonical internal `TypeV1` universe in place: its record,
   relation, union, tuple, optional, and collection branches contain
   private-field checked carriers. `TypeV1` itself becomes a public
   private-field struct over a private kind enum; callers construct it only
   through checked scalar/record/relation/union/tuple/optional/collection
   functions. Raw/internal types are not
   exported by the curated facade, so a parallel `TypeV2` universe or
   conversion layer is forbidden.
2. Reject duplicate record fields and union tags in fallible constructors.
3. Make relation rows accept only `RecordTypeV1`.
4. Split scalar/value bindings from `RelationCatalogV2`.
   `ValueTypeV1` is a checked private-field wrapper over the canonical type
   universe that recursively excludes `TypeV1::Relation`; expression bindings,
   operator/builtin/predicate parameters/results, record fields, collections,
   and expression facts use it. Only relation authority carries
   `RelationTypeV1`.
5. Give relation sources stable IDs, row type, cardinality, provenance, and span.
6. Replace overwrite-style inserts with duplicate diagnostics.
7. Parse integers as `i128` and floats as finite `f64`; reject non-finite values.
8. Refactor the existing recursive `TypeCheckerV1` into the sole
   `ExpressionTypeKernelV2`. It receives node-ID-keyed binding/call signatures
   from sema and returns node-ID-keyed type, resolved overload/builtin ID, and
   effect facts. It does not resolve lexical names and sema does not repeat
   overload selection.
9. Add `TypedLiteralValueV2::{Int(i128), Float(FiniteF64V2), Bool, String,
   Null}` to node facts. `FiniteF64V2` has private checked construction, rejects
   non-finite values, normalizes both signed zeros to positive zero, and uses
   normalized IEEE bits plus `total_cmp` for deterministic equality, hashing,
   and ordering. Detect nonzero lexical underflow before zero normalization.
10. Give builtin/operator overloads explicit stable IDs declared beside each
    registry entry. Registry construction rejects duplicate IDs; vector index
    is never identity. Resolved predicate signatures enter through an opaque
    callable-signature input so `types` does not depend on sema.

### Modules

1. Add stable `ModuleIdV2`, `CatalogAssetIdV2`, and typed catalog namespaces.
2. Couple `ResolvedGraphV2` with current-module identity.
3. Preserve distinct direct-import targets until ambiguity rejection.
4. Treat identical declarations as explicitly idempotent only where specified;
   reject other duplicates.
5. Add `ValidatedPackGraphV2` and `VerifiedLockSetV2` proof types, then seal
   them in `ValidatedPackAuthorityV2`; a verified lock set retains the exact
   graph fingerprint it was checked against.
6. Verify lock pack/version/digest against the validated graph.
7. Reject duplicate pins, suites, and signature parameters.
8. Add primary, secondary, and edge HIR-owned `LocatedSpanV2` values to module
   diagnostics. Bare byte offsets are forbidden for cross-document conflicts.
9. Separate host-provided `ExternalCatalogV2` from document-local declarations.
   External assets carry stable ID, kind, typed signature, provenance, and
   source span; they never store a document-local raw HIR body.
10. Reuse canonical `types` carriers in module/catalog signatures through the
    pinned `modules -> types` edge. Delete opaque string type tags as callers
    migrate; do not create a modules-local type universe.
11. Store current-module identity only in `ResolvedGraphV2`. Any authority
    accessor delegates to it; no second field or equality-by-convention is
    allowed.
12. Derive stable IDs from checked canonical names, never insertion order:
    module ID from qualified module name; catalog asset ID from module,
    namespace kind, and local name; relation source ID from provenance
    namespace plus qualified name.
13. Add `ContentDigestV2` as a checked lowercase SHA-256 digest. V2 pack
    manifests and lock pins require it; parsing is fallible, duplicate pins
    reject, and version plus digest must match.
14. Reject pack dependency cycles with the module budget and an iterative
    traversal. A validated pack graph is acyclic.
15. Duplicate policy is exact: identical re-export edges, suite members, and
    export members are idempotent sets; duplicate declarations, imports, lock
    pins, suite declarations, and signature parameters reject even when their
    payloads are identical. Different-payload repeats always reject and retain
    both locations.
16. Migrate declarative model extensions without widening into runtime/provider
    wiring. `ModelSchemaV2`, located model-pack/fact inputs, and
    `ComposedModelV2` preserve validation, permutation stability, idempotent
    agreement, and fail-closed target/attribute conflict semantics.
17. Delete orphan `QueryMetadataV1`, `QueryKindV1`, `SeverityClassV1`,
    `HelpDocV1`, their token methods/re-exports, and tests. They have no
    production consumer and are unrelated to F31's expression effect metadata.

## Required APIs

These API names and result shapes are pinned. A rename requires a packet
amendment before implementation; raw-map escape hatches are forbidden. The
low-level methods that mention `ModuleBudgetSessionV2` are `pub(crate)` owner
internals. Only the session-owning builders below are public, so callers cannot
reset a budget between phases:

```rust
ValueTypeV1::try_from_type(TypeV1) -> Result<ValueTypeV1, TypeDiagnosticV2>
RecordFieldTypeV1::try_new(name, value_type, checked_location) -> Result<RecordFieldTypeV1, TypeDiagnosticV2>
UnionVariantTypeV1::try_new(tag, value_type, checked_location) -> Result<UnionVariantTypeV1, TypeDiagnosticV2>
RecordTypeV1::try_new(fields) -> Result<RecordTypeV1, TypeDiagnosticV2>
RelationTypeV1::try_new(row: RecordTypeV1, cardinality) -> Result<RelationTypeV1, TypeDiagnosticV2>
ModuleIdV2::try_new(&str, CheckedLocatedSpanV2) -> Result<ModuleIdV2, ModuleDiagnosticV2>
PackIdV2::try_new(&str, CheckedLocatedSpanV2) -> Result<PackIdV2, ModuleDiagnosticV2>
ProvenanceNamespaceV2::try_new(&str, CheckedLocatedSpanV2)
    -> Result<ProvenanceNamespaceV2, TypeAuthorityDiagnosticV2>
ContentDigestV2::try_parse(&str, CheckedLocatedSpanV2)
    -> Result<ContentDigestV2, ModuleDiagnosticV2>
VersionV2::new(u64, u64, u64) -> VersionV2
VersionV2::try_parse(&str, CheckedLocatedSpanV2)
    -> Result<VersionV2, ModuleDiagnosticV2>
TypeV1::try_tuple(Vec<ValueTypeV1>) -> Result<TypeV1, TypeDiagnosticV2>
TypeV1::try_optional(ValueTypeV1) -> Result<TypeV1, TypeDiagnosticV2>
TypeV1::try_collection(ValueTypeV1, CardinalityV1) -> Result<TypeV1, TypeDiagnosticV2>
TupleTypeV1::try_new(Vec<ValueTypeV1>) -> Result<TupleTypeV1, TypeDiagnosticV2>
OptionalTypeV1::try_new(ValueTypeV1) -> Result<OptionalTypeV1, TypeDiagnosticV2>
CollectionTypeV1::try_new(ValueTypeV1, CardinalityV1) -> Result<CollectionTypeV1, TypeDiagnosticV2>
SourceRegistryViewV2::try_new(Vec<SourceRegistrationV2>)
    -> Result<SourceRegistryViewV2, SourceRegistryDiagnosticV2>
SourceRegistryViewV2::try_extend_document(&self, SourceRegistrationV2)
    -> Result<ExtendedSourceRegistryViewV2, SourceRegistryDiagnosticV2>
SourceRegistryViewV2::identity() -> SourceRegistryIdentityV2
SourceRegistryViewV2::try_check_location(LocatedSpanV2)
    -> Result<CheckedLocatedSpanV2, SourceRegistryDiagnosticV2>
ExtendedSourceRegistryViewV2::try_check_location(LocatedSpanV2)
    -> Result<CheckedLocatedSpanV2, SourceRegistryDiagnosticV2>
RelationCatalogV2::try_new(inputs: Vec<RelationSourceInputV2>,
                           source_registry: &SourceRegistryViewV2,
                           budget_profile: AuthorityBudgetProfileV2)
    -> Result<RelationCatalogV2, TypeAuthorityDiagnosticV2>
ExternalCatalogV2::try_new(
    assets,
    source_registry: &SourceRegistryViewV2,
    session: &mut ModuleBudgetSessionV2,
)
    -> Result<ExternalCatalogV2, ModuleDiagnosticV2>
SignatureV2::try_new(input: SignatureInputV2,
                     source_registry: &SourceRegistryViewV2,
                     session: &mut ModuleBudgetSessionV2)
    -> Result<SignatureV2, ModuleDiagnosticV2>
SignatureV2::try_as_resolved_host_predicate(ResolvedHostPredicateKeyV2,
                                            CheckedLocatedSpanV2)
    -> Result<ResolvedHostPredicateCallableV2, ModuleDiagnosticV2>
PackManifestV2::try_new(input: PackManifestInputV2,
                        source_registry: &SourceRegistryViewV2,
                        session: &mut ModuleBudgetSessionV2)
    -> Result<PackManifestV2, ModuleDiagnosticV2>
ExpressionTypeKernelV2::check_document(inputs: &ResolvedTypeInputsV2)
    -> Result<TypedExpressionFactsV2, TypeDiagnosticV2>
ResolvedTypeInputsV2 accepts checked deferred relation-row inputs whose ordered
fields are `Known(ValueTypeV1)` or exact prior expression dependencies; the
kernel returns owner-built row facts without a second traversal.
AuthorityBudgetProfileV2::try_new(module_nodes, module_edges, module_steps,
                                  authority_bytes, authority_string_bytes)
    -> Result<AuthorityBudgetProfileV2, TypeAuthorityDiagnosticV2>
ModuleBudgetV2::from_profile(AuthorityBudgetProfileV2) -> ModuleBudgetV2
ModuleBudgetV2::begin_session_v2(&self) -> ModuleBudgetSessionV2
ModuleAuthorityBuilderV2::try_new(AuthorityBudgetProfileV2, SourceRegistryViewV2)
    -> Result<ModuleAuthorityBuilderV2, ModuleDiagnosticV2>
ModuleAuthorityBuilderV2::try_add_module(ModuleNodeInputV2)
    -> Result<(), ModuleDiagnosticV2>
ModuleAuthorityBuilderV2::try_declare(ModuleDeclarationInputV2)
    -> Result<(), ModuleDiagnosticV2>
ModuleAuthorityBuilderV2::try_import(ModuleImportInputV2)
    -> Result<(), ModuleDiagnosticV2>
ModuleAuthorityBuilderV2::try_reexport(ModuleReexportInputV2)
    -> Result<(), ModuleDiagnosticV2>
ModuleAuthorityBuilderV2::try_add_manifest(PackManifestInputV2)
    -> Result<(), ModuleDiagnosticV2>
ModuleAuthorityBuilderV2::try_set_lockfile(LockReferenceInputV2, SourceIdV2, &str)
    -> Result<(), ModuleDiagnosticV2>
ModuleAuthorityBuilderV2::try_add_external(ExternalCatalogAssetInputV2)
    -> Result<(), ModuleDiagnosticV2>
ModuleAuthorityBuilderV2::try_set_model_schema(ModelSchemaInputV2)
    -> Result<(), ModuleDiagnosticV2>
ModuleAuthorityBuilderV2::try_add_model_pack(ModelPackInputV2)
    -> Result<(), ModuleDiagnosticV2>
ModuleAuthorityBuilderV2::finish(current_module: CurrentModuleSelectionInputV2,
                                 language_version: VersionV2)
    -> Result<ValidatedModuleAuthorityPartsV2, ModuleDiagnosticV2>
DocumentCatalogValidatorV2::try_new(ExtendedSourceRegistryViewV2,
                                     &ValidatedModuleAuthorityPartsV2)
    -> Result<DocumentCatalogValidatorV2, ModuleDiagnosticV2>
DocumentCatalogValidatorV2::validate(DocumentCatalogInputV2)
    -> Result<DocumentCatalogAuthorityV2, ModuleDiagnosticV2>
ModuleGraphBuilderV2::new() -> ModuleGraphBuilderV2
ModuleGraphBuilderV2::try_add_module(&mut self, ModuleNodeInputV2,
                                     &mut ModuleBudgetSessionV2)
ModuleGraphBuilderV2::try_declare(&mut self, ModuleDeclarationInputV2,
                                  &mut ModuleBudgetSessionV2)
ModuleGraphBuilderV2::try_import(&mut self, ModuleImportInputV2,
                                 &mut ModuleBudgetSessionV2)
ModuleGraphBuilderV2::try_reexport(&mut self, ModuleReexportInputV2,
                                   &mut ModuleBudgetSessionV2)
ResolvedGraphV2::visible_bindings() -> &ResolvedModuleBindingsV2
PackGraphV2::try_new(manifests, session: &mut ModuleBudgetSessionV2)
    -> Result<PackGraphV2, ModuleDiagnosticV2>
PackGraphV2::validate(language_version: VersionV2,
                      session: &mut ModuleBudgetSessionV2)
    -> Result<ValidatedPackGraphV2, ModuleDiagnosticV2>
LockfileV2::verify_against(&ValidatedPackGraphV2,
                           session: &mut ModuleBudgetSessionV2)
    -> Result<VerifiedLockSetV2, ModuleDiagnosticV2>
ValidatedPackAuthorityV2::try_new(ValidatedPackGraphV2, VerifiedLockSetV2)
    -> Result<ValidatedPackAuthorityV2, ModuleDiagnosticV2>
ValidatedPackAuthorityV2::manifest(pack_id: &PackIdV2)
    -> Option<&ValidatedPackManifestV2>
ValidatedPackAuthorityV2::validate_activations(
    inputs: Vec<PackActivationInputV2>,
    source_registry: &SourceRegistryViewV2,
    session: &mut ModuleBudgetSessionV2,
) -> Result<ValidatedPackActivationsV2, ModuleDiagnosticV2>
DocumentCatalogAuthorityV2::try_new(
    input: DocumentCatalogInputV2,
    external: &ExternalCatalogV2,
    packs: &ValidatedPackAuthorityV2,
    source_registry: &SourceRegistryViewV2,
    session: &mut ModuleBudgetSessionV2,
) -> Result<DocumentCatalogAuthorityV2, ModuleDiagnosticV2>
ModuleGraphBuilderV2::resolve(current: CurrentModuleSelectionInputV2,
                              session: &mut ModuleBudgetSessionV2)
    -> Result<ResolvedGraphV2, ModuleDiagnosticV2>
LockfileV2::parse(source_id: SourceIdV2, text: &str,
                  source_registry: &SourceRegistryViewV2,
                  session: &mut ModuleBudgetSessionV2)
    -> Result<LockfileV2, ModuleDiagnosticV2>
```

`SignatureV2::try_new`, `VersionRangeV2::try_new`,
`PackManifestV2::try_new`, `ExternalCatalogV2::try_new`, raw lock parsing, and
all graph/pack phase APIs are likewise crate-private. Public callers submit the
listed checked-location entry DTOs to `ModuleAuthorityBuilderV2` or
`DocumentCatalogValidatorV2`; they never construct a completed proof fragment
that could have used another registry or budget session.

The builder ellipses above are typed DTOs, not raw maps or string tuples:

```text
ModuleNodeInputV2 { module_id, location }
CurrentModuleSelectionInputV2 { module_id, location }
ModuleDeclarationInputV2 { owner_module_id, local_name,
                           namespace_kind, visibility, location }
CatalogAssetRefInputV2 { module_name, namespace_kind, local_name, location }
ModuleImportInputV2 { owner_module_id, local_name, target: CatalogAssetRefInputV2,
                      location }
ModuleReexportInputV2 { owner_module_id, exported_name,
                        target: CatalogAssetRefInputV2,
                        location }
ExternalCatalogAssetInputV2 { module_name, local_name, kind_token,
                              payload: ExternalCatalogPayloadInputV2,
                              provenance, location,
                              dependencies }
ExternalCatalogAssetV2 { id, kind, typed_payload, provenance, location,
                         dependencies } // validated output only
ExternalCatalogDependencyInputV2 { target: CatalogAssetRefInputV2,
                                   expected_kind, location }
LockPinV2 { pack_id, version, content_digest, location }
RelationSourceInputV2 { provenance_namespace, qualified_name,
                        lookup_name, row, cardinality, provenance, location }
LocatedParameterInputV2 { name, value_type, location }
SignatureInputV2 { parameters, result, callable_kind, effects, location }
PackManifestInputV2 { pack_id, version, content_digest, language_range,
                      dependencies, exports, suites, location }
LocatedDependencyInputV2 { pack_id, version_range, location }
LocatedExportInputV2 { name, location }
LocatedSuiteInputV2 { name, members, location }
LocatedSuiteMemberInputV2 { name, location }
PackActivationInputV2 { pack_id, activation, lock_reference, location }
LockReferenceInputV2 { path, location }
DocumentCatalogInputV2 { assets, profile_admissions, activations,
                         require_certified_directives, dependency_edges }
DocumentCatalogAssetInputV2 { owner_module_id, local_name, namespace_kind,
                              location }
ProfileAdmissionInputV2 { profile: CatalogAssetRefInputV2,
                          admission, location }
DocumentCatalogDependencyInputV2 { source: CatalogAssetRefInputV2,
                                   target: CatalogAssetRefInputV2,
                                   expected_target_kind, location }
RequireCertifiedInputV2 { location }
ModelSchemaInputV2 { schema_version, attributes, location }
ModelAttributeInputV2 { attribute, allowed_values, location }
LocatedModelValueInputV2 { value, location }
ModelPackInputV2 { pack_id, model_version, facts, location }
ModelFactInputV2 { target, attribute, value, location }
```

Every `location` in these public authority DTOs is
`CheckedLocatedSpanV2`, not raw `LocatedSpanV2`. The checked carrier has private
fields, retains its `SourceRegistryIdentityV2`, and exposes the copied raw
location read-only for diagnostics/sorting. Module/document builders reject a
checked location whose registry identity differs from the builder's owned
registry. Low-level parsers/constructors that initially receive raw spans are
crate-private and must mint this proof before mutation.

`CatalogAssetRefInputV2` and `SignatureInputV2` are public declarative ingress,
not completed authority values. An enclosing session-owning builder validates
their checked locations, canonical names, namespace/payload compatibility,
parameter uniqueness, result domain, callable kind, and effects atomically.
Only then does it derive private-field `CatalogAssetIdV2`/resolved references
and opaque `SignatureV2`. There is no public constructor for either completed
result, and callers cannot smuggle a precomputed ID or signature through a
payload enum.

`NamespaceKindV2` is closed and its exact stable tags are `value`, `relation`,
`predicate`, `type`, `view`, `model`, `pack`, `profile`, `gate`, and `rule`.
`CallableKindV2` is closed to `predicate` and `rule`. The validated external
payload enum is exact:

```text
ExternalCatalogPayloadV2::Value(ValueTypeV1)
ExternalCatalogPayloadV2::Relation(RelationTypeV1)
ExternalCatalogPayloadV2::Predicate(SignatureV2)
ExternalCatalogPayloadV2::Type(ValueTypeV1)
ExternalCatalogPayloadV2::View(RelationTypeV1)
ExternalCatalogPayloadV2::Model(PackIdV2)
ExternalCatalogPayloadV2::Pack(ExternalPackContractV2)
ExternalCatalogPayloadV2::Profile(ProfileAdmissionV2)
ExternalCatalogPayloadV2::Gate(GateContractV2)
ExternalCatalogPayloadV2::Rule(SignatureV2)
```

The parsed `kind_token` must match the payload variant exactly. Signatures are
required only for Predicate/Rule and are unrepresentable on every other kind;
there is no optional signature field in validated output. Each dependency edge
states its expected target kind. Construction resolves the target ID and rejects
an actual-kind mismatch with both edge and target locations. An exhaustive
ten-kind matrix tests accepted payload, mismatched payload, unknown token, and
dependency expected/actual kind.

The declarative `ExternalCatalogPayloadInputV2` has the same ten variants, but
its Predicate and Rule cases contain `SignatureInputV2`, not `SignatureV2`.
The enclosing catalog builder validates and seals the completed signature in
the same transaction as namespace/payload matching. Input and validated enums
are not aliases and no public conversion accepts a prebuilt signature.

Modules owns closed semantic enums rather than retaining syntax enums:
`ProfileAdmissionV2::{Allowlist, ReviewRequired, CertifiedOnly}` with stable
tags `allowlist`, `review-required`, `certified-only`;
`GateBindingV2::{All, Any}` with tags `all`, `any` and
`GateContractV2 { binding }`;
`CatalogPackActivationV2::{Manual, Session, Workspace}`; and the deliberately
narrower `UsePackActivationV2::{Session, Workspace}`.
`ExternalPackContractV2` contains a `PackIdV2` plus
`CatalogPackActivationV2`; `PackActivationInputV2.activation` is exactly
`UsePackActivationV2`, so Manual is unrepresentable for USE PACK. Ticket 04
owns exhaustive HIR-to-semantic conversion and tests every variant/tag plus the
Manual rejection; modules authority never reparses tokens.

Duplicate diagnostics use distinct stable codes for module, declaration,
import, re-export, lock pin, suite declaration, signature parameter, external
catalog asset ID, relation source ID, and relation lookup-name collisions.
Each carries primary conflict location,
secondary original location, and the relevant import/re-export edge location.
Unknown current module uses `modules.unknown_module`; budget exhaustion uses
`modules.resource_exhausted`.
Registry-identity substitution uses `modules.source_registry_mismatch` at the
modules boundary and `types.source_registry_mismatch` at the relation/types
boundary. Recursive relation admission to `ValueTypeV1` uses
`types.relation_used_as_value`.
`SourceRegistryDiagnosticV2` implements `code`, bounded `Display`, and
`std::error::Error`. Its exact codes are
`hir.source_registry.duplicate_source_id`,
`hir.source_registry.unknown_source_id`, and
`hir.source_registry.inverted_span` or
`hir.source_registry.location_out_of_bounds` according to the underlying span
failure; no `Debug` dump is its public message contract.

There must be no `Default` implementation for closed semantic authority.
Ticket 04 owns `ClosedFrontendAuthorityV2::try_new`; it accepts one
`ValidatedModuleAuthorityPartsV2` plus one relation catalog, not separate graph,
external, current-module, pack-graph, lock-proof, or host-registry values.

`ResolvedTypeInputsV2` is fallibly constructed from checked HIR document roots
plus an entry list keyed by
`SourceQualifiedNodeIdV2 { source_id: SourceIdV2, node_id: HirNodeIdV1 }`.
It builds its private maps itself so duplicate input entries remain observable;
a caller-supplied pre-collapsed map is forbidden. Inputs contain the already
resolved binding type or callable authority for every symbol/call node.
Each `RelationExists` Bool node additionally requires an opaque
`ResolvedRelationPredicateInputV2` carrying its resolved relation-source and
capture identity plus conservative relation base effects, or an explicit
open-analysis Unknown cause. It contains no relation plan and creates no
types-to-sema/algebra dependency. Missing, wrong-kind, duplicate, extra, and
foreign checks apply exactly as they do to binding/call inputs.
`TypedExpressionFactsV2` contains the selected type, `OperatorIdV2`, and
`ResolvedCallTargetV2::{Builtin(BuiltinIdV2),
Predicate(ResolvedPredicateIdV2)}` when applicable, plus purity, determinism,
monotonicity, aggregation, and cost. Its fallible entry-list constructor rejects
duplicate expression keys with `types.duplicate_resolved_input`, missing entries
with `types.missing_resolved_input`, extra entries with
`types.extra_resolved_input`, and foreign roots with
`types.foreign_document_input`. There is no name-based fallback. Open analysis represents an
unresolved input with an explicit `UnknownTypeCauseV2`, not by omitting its node
entry. Unknown is represented by
`ExpressionTypeStateV2::{Known(ValueTypeV1), Unknown(UnknownTypeCauseV2)}` and is
never a closed type variant. The kernel is mode-neutral; sema rejects every
Unknown state before compiled success.

Context is represented without overloading symbol/call entries.
`ResolvedTypeInputsV2::try_new` accepts a separate entry list of
`ContextualExpectedTypeInputV2 { key, expected_type, reason }`. It rejects
duplicate keys, foreign documents, and unsupported target node kinds. Such an
entry is optional overall but, when present, must be used by the matching
expression. Only the kernel knows actual traversal/context consumption, so
`check_document` rejects any unconsumed entry after traversal with
`types.unused_contextual_input`; the constructor cannot claim that proof. The kernel also propagates expected
argument types internally while testing operator/builtin/predicate candidates;
sema does not infer an empty-list element type after the kernel. An empty list
becomes a known list only when all viable checked contexts agree on one element
type; absent or conflicting context remains
`UnknownTypeCauseV2::EmptyCollectionElement`.

The sole `i128::MIN` lexical exception attaches
`TypedLiteralValueV2::Int(i128::MIN)` to the unary-minus node. Its positive
`i128::MAX + 1` child retains
`TypedLiteralFactV2::DeferredI128MinMagnitude`, a closed marker legal only as
that unary node's direct operand; construction rejects the marker everywhere
else. Thus every expression node has a fact and downstream consumers never
reparse literal text.

Located record-field and union-variant values are construction inputs, not
semantic type identity. Checked `RecordTypeV1`/`UnionTypeV1` outputs retain only
canonical names and nested `TypeV1`; source locations are kept in diagnostics
or separate provenance facts. `Eq`, `Hash`, and `Ord` for types and callable
signatures are therefore location-independent. The same schema declared in
different source files remains the same semantic type.

Every checked type stores owner-computed shape facts and enforces
`MAX_TYPE_DEPTH = 64` and `MAX_TYPE_NODES = 4096` with checked arithmetic before
allocation/insertion. Depth counts the root as one; node count includes every
scalar and container occurrence. Tuple, optional, collection, record, union,
and relation constructors compose already checked child facts and reject
`types.type_shape_limit` at exact one-over. `ValueTypeV1` reads the stored
`contains_relation` fact instead of recursively trusting arbitrary public enum
construction. Display/equality/hash/assignability therefore traverse only a
bounded tree. Exact-limit, one-over, and deeply nested relation-admission tests
cover collection, tuple, and optional shapes.
Nested optional construction rejects as `types.redundant_optional` rather than
creating two semantic representations of the same nullability. Empty tuples
remain valid unit-shaped values. Old infallible `TypeV1::{tuple, optional,
collection, list, set}` helpers and direct public enum variants are removed and
inventoried; fallible convenience constructors delegate to the checked carriers.
Record-field names and union tags use the canonical ASCII segment grammar and
`MAX_TYPE_NAME_BYTES = 256`; empty, noncanonical, or one-byte-over values reject
`types.invalid_type_name` before cloning. Type diagnostic rendering applies the
same bounded-excerpt policy as other frontend diagnostics.

An empty list does not become `list<Null>` and `Null` is never a wildcard type
variable. Without a supplied checked contextual element type, `[]` produces
`UnknownTypeCauseV2::EmptyCollectionElement`; open analysis may retain that
state, while closed compilation rejects it. With a contextual list type the
kernel emits that exact element type. `ValueTypeV1::is_assignable_to` has no
Null-element exception. Empty-list open, closed, and contextual cases are all
owner tests.

Registry creation is fallible and production-owned:
`OperatorRegistryV2::try_closed()` and `BuiltinRegistryV2::try_closed()`
validate private static canonical entries and reject duplicate explicit IDs or
semantic signatures. A private `try_from_entries` exists only for owner unit
non-vacuity tests; no public caller can inject or extend the closed core
registry. No `unwrap`, vector index, or debug-only check establishes registry
identity. `CallArgHirV1::Star` matches only an explicit
aggregate-Star signature (initially `count(*)`); scalar, predicate, mixed, and
unsupported aggregate calls reject without assigning a fabricated Star type.
The kernel detects `any_star` before collecting expression argument types. Only
the exact one-element `[Star]` shape may enter AggregateStar lookup; every mixed,
multiple, scalar, predicate, or unsupported Star call rejects
`types.invalid_star_argument` before ordinary overload search.

Registry uniqueness is based on accepted call domain, not result type. Builtin
keys include canonical name/family and argument shape/parameter types while
excluding result/effects; overlapping `AnySingle` and one-parameter `Fixed`
domains for the same name reject. Name-to-family is a bijection. Operator keys
likewise exclude result/effects, so two entries accepting the same symbol and
operand domain cannot become order-dependent. An absent operator symbol emits
`types.unknown_operator`; a present symbol with no matching domain emits
`types.bad_overload`, and both branches have direct tests.

`ExpressionTypeKernelV2` owns overload selection for every value Unary/Binary,
Bool Compare, Bool connective/Not, builtin call, and opaque predicate call. It
emits source-qualified facts for every `ValueExprHirV1` and `BoolExprHirV1`
node. Sema supplies resolved operands and
`ResolvedCallableInputV2::{Builtin(BuiltinFamilyIdV2),
Predicate(ResolvedPredicateCallableV2)}` per call node; it lexical-resolves the
callee category but never selects an overload. `ResolvedPredicateCallableV2`
has private fields and a fallible constructor for a canonical
`ResolvedPredicateIdV2`, ordered `ValueTypeV1` parameters, a `ValueTypeV1`
result, and all five
`ExpressionEffectsV2` fields. It cannot contain a caller-selected builtin or
operator ID. Types selects the explicit `BuiltinIdV2` for builtin families,
checks predicate arity/types, and copies the complete predicate identity and
effects into result facts;
ticket 04 must not repeat registry search in `type_bool` or call handling.

Logical quantifiers type their Bool body with sema-supplied binder facts and
inherit its composed effects. `RelationExists` recursively types every scalar
value/Bool expression in its relation subtree, then combines those child facts
with its required `ResolvedRelationPredicateInputV2`; it emits a Bool/Unknown
fact for the exists node without constructing algebra. Ticket 04 resolves the
relation source/capture identity before building type inputs, but it does not
type the exists node itself. This closes every-Bool coverage without a crate
cycle or a second semantic typer.

Every operator registry entry carries its own `ExpressionEffectsV2` base facts.
The canonical operator table is exhaustive: all entries are Pure,
Deterministic, Scalar, and Constant; unary `+` is Monotone, unary `-` and `not`
are Antitone; binary `+`, `and`, and `or` are Monotone; binary `-`, `*`, `/`,
`%`, `xor`, `++`, every comparison, and every equality/inequality are
NonMonotone. A table-completeness owner test enumerates every explicit operator
ID and asserts its exact tuple.
Parent facts conservatively compose the base with every child: `Impure`,
`NonDeterministic`, `NonMonotone`, `Aggregate`, and `SuperLinear` dominate their
respective dimensions; otherwise equal monotonicities remain equal and a
Monotone/Antitone mix becomes `NonMonotone`; cost uses
`Constant < Linear < SuperLinear`. Calls use the selected builtin/predicate base
effects and the same child join. Literal/symbol base effects are pure,
deterministic, monotone, scalar, and constant. These joins never erase a child
effect; aggregate-context legality remains ticket 04's consumer check.

Unknown operands filter overload candidates by known contradictions only. If a
single remaining domain determines the result independently of unresolved
operands, the result may be known; otherwise the expression remains Unknown and
has no selected operator/call ID. `AnySingle` is result-known for one non-Star
argument because its domain accepts every type. Unknown is not converted into a
false `bad_overload`, and production facts never claim an ID that was not
uniquely selected.

`ResolvedModuleBindingsV2` is a modules-owned typed visibility projection, not
a binder type and not `BTreeMap<String, Vec<String>>`. Ticket 04 converts it to
Binder inputs in sema. The precedence/ambiguity algorithm currently exercised
through `binder_view` is retained exactly once behind this API.

`SourceRegistryViewV2` is the HIR-owned checked source-bounds carrier built from
an entry list of `SourceRegistrationV2 { source_id, source_len }`. Its fallible
constructor rejects duplicate source IDs before any map collapse, and its only
location operation validates a `LocatedSpanV2` against the registered length.
Successful validation returns `CheckedLocatedSpanV2`; a bare location or a
proof minted against an incompatible registry cannot enter public authority
DTOs. `try_extend_document` adds exactly one document registration without
exposing a raw map and rejects a collision with cached host authority. It
returns a private-field `ExtendedSourceRegistryViewV2` that retains both the
exact structural host-parent identity and the combined identity.
Host `RelationCatalogV2` and `ExternalCatalogV2` validate locations against the
base `SourceRegistryViewV2` and retain its opaque deterministic identity.
Ticket 04 extends that exact host registry only for document-local DTOs;
`ExtendedSourceRegistryViewV2` checks those locations against the combined
identity while retaining the host-parent proof. `ModuleAuthorityBuilderV2`
seals the external identity with its host registry;
`ClosedFrontendAuthorityV2::try_new` compares the separately built relation
catalog identity with the sealed host registry. A same-ID/different-length
registry cannot be substituted by convention.

Host proof compatibility and document-location compatibility are distinct:
cached relation/external/pack/lock proofs must equal the extended view's exact
host-parent identity; new document DTO locations must equal its combined
identity. Structural registries with byte-for-byte equal sorted registrations
are semantically the same identity; callers cannot construct or forge the
opaque identity directly. A sibling/different host view, an extension from the
wrong parent, or a same-ID/different-length document extension rejects. This
append-only parent proof lets cached host authority participate in a document
transaction without revalidation or equality-by-convention.

`SourceRegistryIdentityV2` stores the sorted structural registration sequence
in an immutable shared canonical allocation (`Arc<[SourceRegistrationV2]>` or
equivalent). Cloning identity into any number of checked locations is O(1),
while equality/hash remain structural so independently constructed byte-equal
registries compare equal. Pointer identity and truncated hashes are forbidden.
Owner tests mint many locations, assert shared backing, and assert equal
independent registries plus unequal length/source variants.

Manifest, pack graph, parsed lockfile, verified lock set, and validated pack
authority also retain and compare the same registry identity. Crate-private
`LockfileV2::parse` requires the builder-owned registry and rejects an unknown
source or a registered length different from `text.len()` before tokenization.
Every parsed lock span is minted as `CheckedLocatedSpanV2`. The public module
builder is therefore the only raw lock-text ingress and seals all pack/lock
proofs to its host registry.

Each binding entry retains local name, resolved module/asset ID, namespace kind,
visibility, declaration origin, and import/re-export edge `LocatedSpanV2`.
`resolve(current_module, session)` rejects an absent current module; neither
`visible_bindings()` nor its proof can silently return an empty fallback.
The public finish call receives `CurrentModuleSelectionInputV2`; an unknown
selection uses that checked configuration location as the primary span rather
than fabricating source zero. Low-level resolution remains crate-private.

Namespace kinds are disjoint name spaces. Declaration, direct-import,
re-export, ambiguity, and visibility keys are exactly
`(owner_module_id, namespace_kind, local_name)`. Import kind is derived from
the internally resolved target asset ID; it is never separately caller
supplied. Public import/re-export DTOs carry `CatalogAssetRefInputV2`, and the
builder derives the ID only after validation. The same spelling in two
different namespace kinds is valid. External catalog lookup is by the full
derived `CatalogAssetIdV2`, so equal kind/local names in two modules are valid;
no module-dropping `(kind, local_name)` index exists.

Direct imports never accept caller-supplied visibility. Effective visibility is
derived only from the resolved declaration origin; importing a private target
from another module rejects at the import edge. Re-export is the sole explicit
edge that republishes a binding. `ResolvedBindingV2` stores this owner-derived
visibility, and an API scan proves no import-visibility field remains.

Direct-import behavior is exact: a second `try_import` for the same
`(owner_module_id, namespace_kind, local_name)` always rejects as
`duplicate_import`, whether
its target is identical or different. Deferred candidate ambiguity applies to
multiple re-export origins and other distinct candidate-producing edges, not to
a duplicate direct-import alias that the builder has already rejected.

Deferred ambiguity emits `modules.ambiguous_import` with owner module, typed
local name, and a nonempty sorted `Vec<AmbiguousCandidateV2>`. Each candidate
contains the distinct origin `CatalogAssetIdV2`, its declaration location, and
the full canonical `Vec<LocatedEdgeV2>` path that exposed it. Multiple paths to
the same origin collapse to the lexicographically smallest edge witness;
distinct origins are never truncated. Candidates sort by complete origin ID
then witness, and rendering uses candidate zero only as the primary while
retaining every other location as structured evidence. A three-origin input
permutation test and a two-path/same-origin non-ambiguity test pin this behavior.

`AuthorityBudgetProfileV2` is the canonical private-field checked value for
positive module node/edge/step, `max_authority_bytes`, and
`max_authority_string_bytes` limits. `ModuleBudgetV2` is derived from it and
cannot alter a field. Both `ValidatedModuleAuthorityPartsV2` and
`RelationCatalogV2` retain the complete profile and expose it read-only for the
ticket 04 equality gate.

`ModuleBudgetV2` is the private counter configuration derived only from a
checked `AuthorityBudgetProfileV2`; it has no independent public constructor or
mutable limit setter. Builders charge module-owned authored UTF-8/raw lock input bytes before validation,
cloning, sorting, deduplication, or hashing and reject any single field over the
string limit. Repeated inputs charge again before idempotent dedup. Registered
source lengths are bounds metadata and do not charge unless their text is
actually submitted (lock text does). Fingerprinting streams canonical fields
into SHA-256 instead of materializing a second encoded graph. Builders charge
before inserting nodes/edges; resolution charges
each visited export/import/re-export edge, including iterative replacements for
recursive export-origin and catalog-cycle walks. Exact limit succeeds and the
next charge emits `modules.resource_exhausted { dimension, limit, observed }`.
Diagnostic identifier excerpts are UTF-8-boundary truncated to 128 bytes and
rendered owner messages to 1,024 bytes without changing codes/locations.
The already-iterative module cycle DFS is retained, not rewritten.

The module counters intentionally do not re-walk or recharge opaque checked
`ValueTypeV1`/`RelationTypeV1` payload internals. Those values were admitted by
the types owner under `MAX_TYPE_DEPTH` and `MAX_TYPE_NODES`; modules charges the
enclosing asset/signature node, every authored parameter name, and every other
module-owned string. No module ingress accepts raw record fields, union tags,
or type nodes. If serialized host input later needs a total wire-byte ceiling,
that ceiling belongs at its serialization ingress; modules must not invent an
unstable `Display` encoding to charge a checked type a second time.

An empty module is declared explicitly with `try_add_module`; declarations and
edges require an already-added owner and never create it implicitly. Duplicate
module insertion rejects with `modules.duplicate_module` and both locations.
Module-import, re-export-origin, external-catalog, and pack-dependency cycle
diagnostics retain a closed ordered edge witness with every edge's
`LocatedSpanV2`. Witnesses are canonicalized by lexicographically minimal
rotation and stable direction; when several cycles exist, the lexicographically
smallest encoded witness wins. Input permutation cannot change the code,
primary edge, or ordered witness.

One opaque `ModuleBudgetSessionV2` counter exists per closed module-authority
build and is created once from `ModuleBudgetV2`. Graph mutation/resolution,
pack construction/validation, lock parsing/verification, and external-catalog
construction all receive the same mutable session and never reset it by phase.
Separately cached host authority is built under its own bounded session before
document compilation; per-document code consumes its proof instead of charging
it again. No helper creates or resets a standard/default counter.
`ModuleBudgetSessionV2` and every session-accepting phase API are crate-private;
the public `ModuleAuthorityBuilderV2` owns the sole session and exposes only
session-free mutation/finalization calls. `ValidatedModuleAuthorityPartsV2`
contains the resolved graph, external catalog, validated pack authority, and
the exact host `SourceRegistryViewV2` from that one build. Its fields are
private; consumers use typed accessors and cannot mix proofs from two builds.
Document-catalog validation uses a separate public
session-owning `DocumentCatalogValidatorV2` for its per-document build; it does
not accept a caller-created session or a second budget. It derives its one
session from the sealed module authority's retained profile.

Stable IDs are collision-free checked canonical values, not truncated hashes:
`ModuleIdV2` owns a normalized qualified module name; `CatalogAssetIdV2` owns
the structured `(ModuleIdV2, NamespaceKindV2, local_name)` tuple; and
`RelationSourceIdV2` owns `(ProvenanceNamespaceV2, qualified_name)`. Canonical
name validation is ASCII, case-sensitive, and accepts only `::`-separated
segments matching `[A-Za-z_][A-Za-z0-9_-]*`; hyphen is segment content, never a
separator. It rejects empty segments, empty local names, `.`, `/`, or lone `-`
separator aliases, whitespace, and non-normalized spellings before an ID
exists. Ticket 04 converts every admitted authored reference spelling to this
canonical form before constructing IDs.

The public primitive constructors above are the only ingress for authority IDs,
digests, and parsed versions. They return typed located diagnostics; no public
`from_raw`, unchecked string wrapper, or `&'static str` error remains.
`VersionV2::try_parse` accepts exactly three canonical decimal `u64` components
with no leading zero except `0`. `ContentDigestV2` accepts exactly 64 lowercase
hex bytes. `CatalogAssetIdV2`, `RelationSourceIdV2`, and `LockReferenceV2`
constructors remain owner-private because their inputs are derived/validated by
the enclosing authority builders.

`ModuleDeclarationInputV2` never accepts a caller-computed asset ID;
`CatalogAssetIdV2` is derived after owner module, namespace kind, and local name
validate. `ModuleReexportInputV2` never accepts a caller-asserted origin; the
resolver derives the declaration origin while retaining every edge location.
`ExternalCatalogV2::try_new` parses `kind_token`, validates module/local names,
derives the ID, and resolves dependency IDs. No authority boundary relies on
equality-by-convention between redundant ingress fields.

`PackIdV2` is a checked canonical pack name only. `PackCoordinateV2` is the
nonredundant `(PackIdV2, VersionV2)` pair used by manifests and graph nodes; a
`LockPinV2` carries the same two fields separately. V2 never embeds a version
inside `PackIdV2` while also accepting another version.
Pack names are ASCII, case-sensitive, and match
`[A-Za-z_][A-Za-z0-9_]*(?:[.-][A-Za-z0-9_]+)*`. Thus `secure-pack` and
`my.pack` are canonical distinct names; leading/trailing/repeated separators,
`::`, `/`, whitespace, Unicode, and case folding reject rather than normalize.
`VersionRangeV2::try_new(lower, upper, location)` requires `lower < upper` and
rejects equal or inverted bounds before a manifest/dependency can exist.
`PackGraphV2` permits exactly one coordinate per `PackIdV2`; a second manifest
with the same pack name rejects as `modules.duplicate_pack` even when its
version differs, with both locations. This makes name-only activation and
manifest lookup unambiguous.

`RelationCatalogV2` accepts only `RelationSourceInputV2`, validates its location
through `SourceRegistryViewV2`, derives `RelationSourceIdV2` internally, wraps
the checked row in `RelationTypeV1`, and rejects duplicate derived IDs and
duplicate `lookup_name` values with both locations. Public callers cannot
supply a precomputed relation ID or completed `RelationSourceV2`.

The pack graph fingerprint is SHA-256 over domain bytes
`qglang.pack-graph.v2\0`. Every sequence starts with a `u64` element count in
big-endian order. Every variable-length UTF-8 field starts with a `u64`
big-endian byte length. Version integers are fixed-width `u64` big-endian;
ranges encode lower then upper. Enum/record alternatives use explicit stable
ASCII domain tags, each encoded as a length-prefixed field. Content digests are
decoded to raw 32 bytes. Packs, dependencies, exports, suites, and members sort
by their complete canonical encoded key before hashing. The input covers pack
name, version, content digest, language range, every dependency ID/range,
export, suite name, and member. An exact golden digest fixture pins the byte
recipe; permutation tests yield the same fingerprint and changing any covered
semantic field changes it. The implementation uses pinned `sha2 =0.10.9`, not
a repository-local cryptographic implementation.

`SourceIdV2` remains transaction-local. Every external `LocatedSpanV2` and
relation-source span is validated against the shared registry view. Ticket 04's
frontend transaction combines external registrations with its document source
and rejects any source-ID collision before building closed authority; raw
`SourceIdV2::from_raw` construction alone is never authority evidence.

`SignatureV2` has private fields and is built from a located ordered parameter
entry list. It rejects duplicate names with
`modules.duplicate_signature_parameter` and both locations, retains parameter
order, result type, callable kind, and all effect fields, and performs fallible
structural construction. Locations do not participate in signature semantic
identity. It never checks invocation arity/arguments. The sole call-facing API
converts predicate/rule signatures into types-owned
`ResolvedPredicateCallableV2`; `ExpressionTypeKernelV2` alone validates calls.
Delete `SignatureInstanceV1`, `SignatureV1::instantiate`, their re-exports, and
owner assertions rather than preserving a second type-check path.

`PackManifestV2` also has private fields. Its entry-list constructor validates
the digest, checked language/dependency ranges, and source locations before
`PackGraphV2::try_new`. Duplicate dependency IDs and suite declarations reject
with original/conflicting locations. Exact duplicate export names and exact
suite members canonicalize as idempotent sets; different payloads cannot hide
behind those set rules. No caller supplies a pre-collapsed manifest map.
For every idempotent repeated semantic edge/member, the owner charges the input
occurrence before deduplication and retains the lexicographically minimum
`(source_id,start,end)` location. Thus input order cannot change diagnostics or
cycle witnesses. External `dependencies` and document `dependency_edges` use
the same exact-idempotent/minimum-location rule; different targets are distinct
edges, unresolved targets reject, and all occurrences consume budget.

V2 lock text has one canonical grammar: each nonempty line is exactly
`<pack-id> SP <major>.<minor>.<patch> SP <64-lowercase-hex-digest> LF`.
Version components are canonical unsigned decimal (`0` or no leading zero).
Tabs, CRLF, blank lines, leading/trailing/multiple spaces, uppercase hex,
comments, and a missing final LF reject with a located lock syntax diagnostic.
The empty graph serializes to the empty string. `LockfileV2::to_text` emits pins
sorted by `PackIdV2`; parse/to-text round trips are byte-canonical. Duplicate
pack IDs reject before map collapse.

An authority with a nonempty manifest graph owns exactly one
`LockReferenceV2` sealed with its
parsed lock bytes, verified set, graph fingerprint, and registry identity.
Canonical lock references are case-sensitive relative UTF-8 paths using `/`,
ending in `.lock`, with no empty, `.`, `..`, absolute, backslash, NUL, or
non-normalized segments; they obey the authority string limit.
`try_set_lockfile` accepts a located raw reference input and retains the checked
reference in every proof layer. Each USE PACK activation supplies its own
located reference input, which must canonicalize to exact equality with the
sealed reference. Same bytes under a different reference, malformed/unknown
reference, or reference substitution rejects with both activation and lock
locations; suffix-only validation is forbidden.

`ModuleAuthorityBuilderV2::finish` validates every module and every edge in
sorted order—including disconnected modules—before projecting current-module
bindings. A bad cycle, origin, visibility, unknown target, or private access in
an unrelated component still rejects the whole proof. A nonempty manifest graph
requires `try_set_lockfile` and otherwise emits `modules.missing_lockfile`.
An empty graph requires no submitted lock and produces the canonical empty
verified-lock proof internally; a submitted nonempty pin for an empty graph
rejects. Both paths retain the graph fingerprint and registry identity.

`DocumentCatalogAuthorityV2` is the sole modules-owned replacement for
`CatalogV1` validation. Ticket 04 supplies a neutral typed projection—asset
IDs/kinds, dependency IDs, profile admission, META activations, and locations,
never raw HIR bodies. The owner retains the existing deterministic view-cycle,
certification, unknown-pack, and activation algorithms and returns a private
validated proof. `ValidatedPackAuthorityV2::manifest` is a read-only typed
lookup; `validate_activations` validates USE PACK and lock references and
returns observable typed activation state. Ticket 04 consumes these APIs and
does not implement a second catalog resolver or inspect raw maps.

Document catalog assets never accept a caller-computed ID; the owner derives it
from module/kind/local name. Exactly one `ProfileAdmissionInputV2` is required
for every local Profile asset. Missing, duplicate, unknown-asset, non-Profile,
and disagreeing profile rows reject with exact locations. A document dependency
edge's source must be a declared local View; its target resolves by full ID
across local plus external assets and must match `expected_target_kind`.
`require_certified_directives` is an entry list, never a pre-collapsed bool or
Option. The owner charges every occurrence, treats identical directives as
idempotent, and retains the minimum location so violations point to a
deterministic META span. REQUIRE CERTIFIED checks every local
Profile plus every external Profile in the transitive dependency closure
reachable from document assets/activations; unrelated cached external profiles
are outside the transaction. Local-to-external dependency success and every
missing/extra/wrong-kind/duplicate profile case are owner tests.

A second activation for the same `PackIdV2` always rejects
`modules.duplicate_pack_activation` with both locations, even when mode and lock
reference are identical; no map overwrite is permitted. A second public
`try_set_lockfile` or `try_set_model_schema` call likewise rejects before
mutation as `modules.duplicate_lockfile_input` or
`modules.duplicate_model_schema`, preserving original/conflicting locations and
charging the attempted input first.

Declarative model-extension authority remains frontend-only and host-agnostic.
`ModelSchemaV2` has private fields, one checked schema version, and an entry-list
constructor that rejects duplicate attributes while exact duplicate allowed
values are idempotent with minimum-location retention. Model target names use
the canonical qualified-name grammar; attributes use the canonical segment
grammar; values are nonempty bounded UTF-8 strings. A `ModelPackV2` is keyed by
an existing `PackIdV2`, requires model-version equality with the schema, and is
built only through `ModuleAuthorityBuilderV2`. Every entry carries a
registry-bound checked location and charges before deduplication.

Within or across packs, identical `(target, attribute, value)` facts are
idempotent and retain all asserting pack IDs plus the minimum fact location.
Two values for the same `(target, attribute)` reject
`modules.model_conflict` with both pack IDs and both locations. Undeclared
attributes, disallowed values, unknown packs, and model-version mismatch reject
`modules.invalid_model_data` at the fact/pack location. Composition sorts by
pack ID then complete fact key and returns private `ComposedModelV2` coupled to
the builder's source-registry identity. It exposes deterministic read-only fact
iteration and provenance, never a runtime/provider/SDK registry.

## Step-By-Step Procedure

1. Introduce private-field record/union/relation carriers inside the canonical
   type universe on the atomic cutover stack, factoring existing checker logic
   into one shared internal path.
2. Migrate type-checker constructors and tests directly to the checked carriers;
   do not land public checked and unchecked type universes together.
3. Introduce scalar binding and relation catalog as different types.
4. Add stable source IDs and source provenance.
5. Change duplicate insert APIs to `Result`; migrate every caller explicitly.
6. Add finite numeric literal parsing and typed range diagnostics.
7. Refactor the current checker traversal to consume resolved node inputs and
   return typed facts; keep registry lookup in types and delete the name-env
   fallback after callers migrate.
8. Introduce stable module/catalog IDs and namespace-kind tables.
9. Change module builder mutation methods to fallible duplicate-preserving APIs.
10. Add `LocatedSpanV2` to declarations/imports/re-exports and propagate source
    identity plus byte span into diagnostics.
11. Add validated pack and verified lock proof types. Compute a deterministic
    graph fingerprint from the validated graph, retain it in the lock proof,
    and construct one `ValidatedPackAuthorityV2` only when both fingerprints
    match.
12. Export only the proof types needed by ticket 04's closed authority
    constructor.
13. Add `ModuleBudgetV2`; charge graph nodes, graph edges, and every traversal
    step before allocation. Keep the current iterative module-cycle DFS and
    convert only recursive export-origin and catalog-cycle walks to explicit
    stacks; a node/edge counter alone does not bound native call-stack use.
14. Remove direct raw graph/map constructors after all callers migrate.
15. Migrate existing types/module tests first: preserve registry overload,
    resolution precedence, cycle, version, lock, signature, and catalog
    positives; add only missing duplicate/kind/span/proof/budget cases. Delete
    V1-constructor assertions when their public constructor is removed.
16. Build `ExternalCatalogV2` once from host authority and keep document-local
   catalog declarations in ticket 04's transaction. Reject duplicate IDs,
   duplicate full `(module_id, namespace_kind, local_name)` keys, unknown kinds,
   and unresolved external view dependencies before closed authority
   construction. Equal kind/local spellings in different modules are positive.
17. Remove public invalid constructors, silent-overwrite methods, opaque string
    type tags, and raw pack/lock authority admission in this owner ticket. The
    02-08 stack may be workspace-red between cuts, but ticket 03 may not leave a
    dual public API for ticket 08 to clean up.
18. Record an exact public-owner API inventory in receipt 03 and prove removed
    symbols/constructors have zero definitions after downstream migration.
19. Type `CallArgHirV1::Star` only through explicit aggregate-Star signatures;
    accept `count(*)` and reject Star for every scalar builtin, predicate,
    mixed list, or unsupported aggregate with an exact typed diagnostic.
20. Reject duplicate pack dependency declarations by dependency pack ID even
    when ranges are identical. Located dependency inputs retain both original
    and conflicting `LocatedSpanV2` values.

## Test Files

The implementation uses one authoritative integration suite rather than six
overlapping module files. This disposition supersedes the earlier in-place
list; receipt 03 must map every deleted semantic case to a retained replacement
test by name, not infer preservation from aggregate counts:

1. `[UPDATE]` `crates/quanta-gqlang-v2-types/tests/v3_types_check_edges.rs` — numeric boundaries and typed facts
2. `[UPDATE]` `crates/quanta-gqlang-v2-types/tests/v3_types_hellgate.rs` — checked constructor/duplicate negatives
3. `[UPDATE]` `crates/quanta-gqlang-v2-types/tests/v3_types_universe.rs` — relation-only authority and metadata retention
4. `[DELETE AFTER CASE-BY-CASE MIGRATION]` `v3_modules_mutation_rails.rs`,
   `v3_modules_graph.rs`, `v3_catalog_hellgate.rs`, `v3_modules_packs.rs`,
   `v3_modules_version_lock_edges.rs`, and `v3_modules_hellgate.rs`.
5. `[ADD, SOLE MODULE TEST OWNER]`
   `crates/quanta-gqlang-v2-modules/tests/v3_modules_v2_authority.rs` — graph,
   visibility, ambiguity, catalog, signature, pack, lock, extension, budget,
   permutation, and mutation rails. Macro-generated namespace rows count as
   parameter cases, not authored semantic test bodies.

The consolidated suite is accepted only if it retains all unique old behaviors
and adds: exact node/edge/step/byte/string boundaries; proof mismatch; multiple
and permuted cycles; duplicate same/different matrices; external/activation/
lock/schema/profile duplicates; model agreement/conflict/invalid value;
certified success plus activated-profile closure; acyclic diamond; private and
not-exported failures; canonical lock roundtrip and malformed located errors;
suite idempotence; and ambiguity primary/secondary permutation stability.

Additional audited dispositions:

1. Rewrite the existing ambiguous re-export mutation fixture so the importer
   sorts before the ambiguous re-exporter; this reaches the inner origin
   ambiguity branch. Retain the hellgate case that reaches the outer candidate
   ambiguity branch.
2. Add an exact `modules.unknown_module` assertion and an absent-current-module
   resolution negative.
3. Delete `path_projection_unknown_field_on_record_rejects` only after the
   stronger hellgate Display/code/span assertion is retained.
4. Remove the unused scalar helper/test and make registry accessors private when
   their final production consumer disappears. Do not delete bugbash candidates
   that were marked pending rather than proven duplicates.
5. `active_packs` must either become an observable typed activation result with
   a real assertion or be deleted as write-only state; an unwrap-only test does
   not justify retaining it.
6. Delete `PackGraphV1::module_count`, `CatalogV1::{asset,is_empty,
   certified_required}`, and their sole-coverage tests if the final production
   graph has no caller. This packet resolves the API decision in favor of
   deletion, not intentional public exposure.
7. Delete unconsumed `QueryKindV1::token` and `SeverityClassV1::token`
   accessors and spelling pins. Retain accessors that are actually rendered by
   a final CLI/service consumer.
8. Delete the proven redundant exact-lock/version-drift/type projection rows
   only after their stronger pack/hellgate owners are migrated. Retain the
   remaining nine unique module mutation tests and every bugbash candidate not
   proven safe.
9. Delete signature-instantiation tests only with the removed
   `SignatureInstanceV1`/`instantiate` path; retain and migrate ordered
   parameter, duplicate-name, callable-kind/effect, and predicate-conversion
   assertions. Types owner tests are the only call-argument oracle.

## Exact Negative Matrix

1. duplicate record field with same and different type
2. duplicate union tag
3. non-record relation row
4. scalar/record/list name used as relation source
5. duplicate source identity
6. duplicate declaration with different visibility
7. duplicate import with same and different target
8. ambiguous re-export
9. duplicate lock pin with same and different version/digest
10. duplicate suite and signature parameter
11. unknown/wrong-kind catalog reference
12. integer outside `i128`
13. float parsed as infinity or NaN
14. nonzero float literal that underflows to signed or unsigned zero
15. duplicate or unresolved external catalog asset
16. missing current module
17. pack dependency cycle
18. pack/lock proofs from different validated graphs
19. aggregate Star accepted only for `count(*)`; Star on a scalar or
    unsupported builtin rejects
20. duplicate pack dependency with identical and different version ranges
21. duplicate module node and undeclared owner module
22. equal and inverted version ranges
23. source registry duplicate ID and external/relation out-of-bounds span
24. resolved-input wrong kind, duplicate, missing, extra, and foreign document
25. empty list in open, closed, and contextually typed positions
26. duplicate manifest suite/dependency entries with both locations
27. multiple/permuted cycles select the same ordered located witness
28. relation type used directly or nested as an expression value type
29. checked location and pack/relation proof substituted across registry identities
30. repeated REQUIRE CERTIFIED, pack activation, lockfile input, and model schema
    with identical and different payload where applicable

Numeric diagnostics are pinned to `types.integer_out_of_range`,
`types.float_non_finite`, and `types.float_underflow`. Typed literals retain the
parsed `i128` or finite `f64`; downstream stages never parse literal text again.
Integer HIR payloads must match ASCII `[0-9]+`; float payloads must match the
surface-reachable decimal grammar `[0-9]+\.[0-9]+`. Leading signs, exponent
notation, whitespace, separators, and other programmatic-only spellings reject
as `types.malformed_literal`; sign is represented only by unary HIR. Owner tests
use the source parser for reachability and may construct malformed HIR only as
an explicit fail-closed integrity rail.
Because the lexer keeps sign as a unary operator, the checker special-cases
unary minus applied directly to decimal magnitude `i128::MAX + 1` and emits the
typed literal `i128::MIN`. That magnitude rejects in positive or unary-plus
position, and larger magnitudes always reject. This is the only pre-typing
integer fold; it prevents an accidental language range of
`-(i128::MAX)..=i128::MAX`.

## Exact Verification

```bash
cargo test -p quanta-gqlang-v2-types --all-targets
cargo test -p quanta-gqlang-v2-modules --all-targets
cargo test -p quanta-gqlang-v2-hir --all-targets
cargo test -p quanta-gqlang-v2-sema --all-targets
cargo clippy -p quanta-gqlang-v2-types -p quanta-gqlang-v2-modules --all-targets -- -D warnings
cargo clippy -p quanta-gqlang-v2-hir -p quanta-gqlang-v2-sema --all-targets -- -D warnings
cargo fmt --all --check
git diff --check
```

## Forbidden Shortcuts

1. validating duplicates only in debug builds
2. keeping a public unchecked constructor beside the checked one
3. converting scalar environments into relation authority by filtering later
4. accepting raw PackGraph/Lockfile in closed context
5. dropping the original and conflicting spans from duplicate diagnostics
6. applying a graph limit only after constructing the full graph
7. retaining recursive module/catalog DFS merely because node/edge counters
   exist
8. adding a parallel `TypeV2` universe or V1/V2 type conversion layer
9. using registry vector position as a stable operator/builtin ID
10. treating a bare `SpanV1` as cross-document location evidence

## RED

1. `TypeEnvV1` overwrites duplicate names.
2. non-record relation rows can be constructed.
3. `users: int` becomes relation source authority.
4. direct import and declaration builders silently overwrite.
5. raw pack graph and filename-only lock checks reach compile context.
6. module diagnostics report only the outer item span.
7. a 400-digit float is accepted as a float type.

## Tests

1. duplicate field/tag/source/import/declaration/pin/suite/parameter negatives
2. scalar/record/collection used as relation source negatives
3. module/import input-permutation property test
4. wrong-kind catalog reference matrix
5. exact primary/secondary span assertions
6. manifest/lock version and digest mismatch tests
7. `i128::MIN`, `i128::MAX`, positive `MAX+1`, negative `MAX+1`, and larger
   integer boundaries plus finite/non-finite/underflow float tests
8. exact node/edge limit and one-over-limit module/catalog tests
9. pack/lock proof from different validated graphs rejects
10. `ResolvedGraphV2` current-module accessor and authority delegation agree by
    construction
11. inner and outer ambiguity branches with deterministic module-order fixtures
12. exact diagnostic assertion for `modules.unknown_module`
13. pack cycle and digest mismatch rejection
14. canonical type equality is location-independent
15. complete facts and composed effects for every value and Boolean node
16. full legacy operator/builtin registry parity with explicit stable IDs
17. pack graph fingerprint golden digest plus field-by-field mutation matrix
18. same local spelling in distinct namespaces and distinct modules succeeds
19. external catalog ten-kind payload/signature/dependency matrix
20. disconnected invalid module rejects whole-graph proof
21. canonical lock parse/to-text bytes, malformed whitespace, missing/nonempty
    lock matrix, and exact lock-reference equality/substitution
22. model schema/version/data validation and permutation-stable conflict spans
23. host registry proof plus document extension succeeds; wrong parent and
    same-ID/different-length reject
24. authority byte/string exact limit and one-over for name, model value, and lock
25. type shape exact depth/node limits, nested relation, and redundant optional
26. orphan query-metadata definitions/re-exports/tests have zero matches

## Definition Of Done

1. semantic authority cannot be constructed from raw maps or raw pack graphs
2. invalid record/relation/union values cannot reach sema
3. no authority builder uses silent last-write-wins
4. all module and catalog reference diagnostics carry exact source locations
5. one types-owned kernel selects every operator/builtin overload; sema only
   supplies resolved node inputs and consumes typed facts
6. one Types-owned bounded solver seals source Rule variables and host/source/
   builtin calls remain disjoint
7. deferred Composition Query rows and RelationExists captures are owner-built,
   exact, source-ordered, and proven in one whole-document kernel call
8. Modules host proofs and View topology are consumed by Sema without a second
   validator or reminted identity
9. final owner/consumer mutation, API inventory, format, strict Clippy, and
   frozen hashes are recorded in receipt 03

## Dependencies

Requires 01 and ticket 02's HIR/location carrier. The types sub-lane may prepare
RED tests while 02 runs; modules implementation starts after 02. Tickets 02-08
remain one unreleasable atomic cutover stack, and crate manifests remain
integrator-owned.

## 2026-09-21 Final-Audit Work Log

The original audit base is
`e896a11024bd5a1e1ef03c094a70032f4e6c765f`. External commit
`96eab779bfce0a5178dbbd48e86bd457158db228` is contaminated with Ticket
03-owned destructive changes in addition to the verified Ticket 01/02 stack.
External commit `fcf106b56e794eb38f74c52b3bd2c1cf34f77fdc` is an atomic-red partial
Ticket 03 snapshot: types did not compile, modules had no Ticket 03 source
delta, formatting failed, and receipt 03 was absent. Neither commit is promoted
to a verified Ticket 03 baseline. Receipt 03 must audit both
`e896a110..96eab779` and `96eab779..final`, with preservation compared to
`e896a110`. This section records contract and implementation checkpoints, not
completion evidence:

1. Replaced phase-local counters with one opaque mutable module-budget session;
   graph, pack, lock, catalog, and resolution work cannot reset limits.
2. Removed redundant caller-computed declaration/re-export/relation/catalog IDs;
   owner constructors derive IDs and resolver origins from checked components.
3. Added explicit empty-module creation and undeclared-owner rejection so an
   empty current module is not confused with an unknown module.
4. Made namespace kinds disjoint and included kind in every module binding,
   collision, ambiguity, and visibility key.
5. Pinned one case-sensitive canonical ID grammar and assigned authored spelling
   normalization to sema before ID construction.
6. Added HIR-owned source-registry construction, copy-on-extend, bounded-location
   validation, stable diagnostics, and opaque registry identity coupling.
7. Changed relation catalog ingress to located raw inputs; it derives stable IDs,
   checked relation rows, and duplicate ID/lookup diagnostics itself.
8. Expanded predicate-call authority to include stable identity, ordered
   parameter/result types, and all five effect dimensions; callers cannot select
   builtin overload IDs.
9. Required complete value and Boolean facts, operand/child effect composition,
   input-kind validation, and exact missing/duplicate/extra/foreign codes.
10. Removed `list<Null>` as an empty-list inference sentinel; open unknown and
    checked contextual typing are explicit, while closed unresolved lists fail.
11. Pinned location-independent equality/hash/order/display for semantic types
    and signatures so source positions cannot change overload identity.
12. Required closed production registries with private test-only injection,
    explicit IDs, duplicate semantic-signature checks, and full legacy registry
    parity instead of a reduced builtin/operator subset.
13. Added checked signature, version-range, pack-manifest, activation, and
    document-catalog constructors; raw/pre-collapsed maps cannot become proof.
14. Defined PackId as name-only and separated version into a nonredundant pack
    coordinate, preventing two disagreeing version fields.
15. Pinned fingerprint bytes (domain, counts, lengths, integer endian/width,
    enum tags, raw digest bytes, sort key) plus a golden digest fixture.
16. Required deterministic located cycle witnesses and deterministic cycle
    selection across input permutations.
17. Kept catalog validation in modules through a typed projection/proof API;
    sema cannot reimplement view-cycle, certification, or pack activation logic.
18. Added source-reachable large-decimal overflow/underflow proof and corrected
    the HIR test-builder ID-collision fixture requirement.
19. Replaced caller-owned budget sessions with session-owning public builders;
    low-level phase APIs are crate-private and fresh-session bypass is impossible.
20. Sealed graph, external catalog, pack proof, and source registry into one
    module-authority result so ticket 04 cannot mix artifacts from two builds.
21. Rejected multiple versions under one pack name to keep name-only activation
    lookup unambiguous.
22. Pinned deterministic minimum-location retention and charge-before-dedup for
    idempotent members/edges, including catalog dependency edges.
23. Added contextual expected-type entries and a deterministic five-dimension
    effect-join algebra; types, not sema, owns empty-list context and effects.
24. Rejected an intermediate wholesale modules rewrite that deleted mature
    resolver/cycle/catalog/pack algorithms. All modules files were restored
    exactly to re-frozen HEAD before continuing; the implementation must now
    refactor shared internals and preserve owner tests instead of replacing them
    with a reduced parallel path.
25. Added recursive `ValueTypeV1` admission so relation types cannot enter an
    expression binding, nested value, callable signature, or typed fact.
26. Replaced raw authority locations with registry-bound checked-location
    proofs and coupled relation/external/pack/lock results to the same registry
    identity, including exact lock-text length validation.
27. Closed Star/overload nondeterminism with pre-search Star rejection,
    argument-domain uniqueness, name/family bijection, and Unknown-aware
    candidate filtering.
28. Added an opaque relation-predicate input so types can cover quantifier and
    EXISTS Bool nodes without importing sema/algebra or delegating a second typer.
29. Closed the model-extension hole with checked schema/pack/fact inputs,
    version coupling, deterministic provenance, budget charges, and located
    conflict/data diagnostics while keeping it host-agnostic.
30. Chose explicit deletion for orphan query metadata carriers and owner-only
    tests; F31 refers to live expression effects, not those unused DTOs.
31. Moved contextual-input consumption proof from input construction to the
    post-traversal kernel finalizer and pinned an unused-context code.
32. Pinned the exact base effect tuple for every operator family and a registry
    table-completeness test.
33. Defined the ten catalog namespace variants, heterogeneous closed payload
    matrix, callable-only signatures, and typed dependency-kind edges.
34. Required whole-graph validation before current-module projection and made
    nonempty pack graphs require a lock while empty graphs receive one canonical
    empty proof.
35. Pinned distinct PackId grammar and byte-canonical digest-bearing lock text.
36. Replaced exact host/document registry equality with an opaque append-only
    parent proof: cached host proofs match the parent identity and document
    checked locations match the combined identity.
37. Made source IDs explicit across semantic/compile/analyze/format/transport
    APIs; no hidden source-ID allocation or renumbering remains.
38. Added aggregate authority byte/string limits, pre-allocation charging,
    streaming fingerprinting, and bounded diagnostic rendering.
39. Closed recursive type construction with private checked containers and
    exact depth/node limits; made relation construction fallible.
40. Bound verified lock bytes to one canonical relative `LockReferenceV2` and
    required USE PACK references to equal that sealed reference, removing the
    legacy suffix-only/file-name authority gap.
41. Expanded document-catalog field sketches into exact located asset,
    admission, and dependency DTOs; pinned REQUIRE CERTIFIED location and its
    local-plus-reachable-external scope.
42. Kept repeated directives observable: REQUIRE CERTIFIED is an entry list
    with charged deterministic idempotence, while duplicate activation,
    lockfile, and model-schema mutations reject before overwrite.
43. Recovered the inherited type/HIR atomic-red state without a compatibility
    layer. The latest recorded focused checkpoint passed
    `cargo test -p quanta-gqlang-v2-hir --all-targets` with 68 tests and
    `cargo test -p quanta-gqlang-v2-types --all-targets` with 46 tests, followed
    by strict all-target Clippy for both crates. This is owner-local evidence
    only; it is not Ticket 03 verification.
44. The type checkpoint includes private checked `TypeV1`/`ValueTypeV1`
    carriers, closed overload/effect facts, recursive relation-type exclusion,
    complete relation-existence typing, contextual-input consumption, stable
    IDs, and source-registry-bound authority names. The integration suite now
    has 40 tests versus 39 at `e896a110`; receipt 03 must still enumerate exact
    old-to-new test disposition rather than infer preservation from the count.
45. At that intermediate checkpoint, modules implementation remained active
    under `/root/ticket03_modules_impl`; no modules-green, full Ticket 03 RR,
    atomic cutover, receipt 03, hosted CI, or packet completion was claimed.
46. Closed the budget-provenance contract: types owns checked
    `AuthorityBudgetProfileV2`; module and relation proofs retain it; ticket 04
    compares both with the canonical `FrontendBudgetV2` projection. Document
    catalog validation derives its session from the sealed module proof instead
    of accepting a caller-selected second budget.
47. Closed the declarative-constructor hole: public
    `CatalogAssetRefInputV2` and `SignatureInputV2` are validated atomically by
    session-owning builders, while derived catalog IDs, resolved references,
    and completed signatures remain private/opaque.
48. Removed the public default-empty module-binding escape; bindings are
    produced only by successful graph resolution. Ambiguity primary location is
    derived from the canonical sorted candidate witness, not input edge order.
49. `SignatureInputV2` now includes a registry-checked location; nested
    signature parameters are validated and budgeted before admission. External
    and model inputs charge all submitted strings/nodes/edges before mutation,
    including rejected duplicate attempts.
50. `REQUIRE CERTIFIED` evaluates the exact local plus activation-reachable
    profile closure. It no longer validates only document dependency targets.
51. Expected builtin/predicate parameter types now contextualize empty list
    arguments inside the types kernel. This is owner logic, not sema inference;
    ambiguous overload families still provide no fabricated context.
52. Added direct same-type record duplicate, NaN, exact recursive type-node
    boundary, one-over, predicate-Star, and quantifier-effect rails. The latest
    types run is 56 tests (`7 + 19 + 16 + 14`) and strict Clippy is green after
    repairing unsafe indexing in the overload-context helper.
53. The authority-byte ceiling is explicitly per sealed admission session, not
    a global total across module, relation, and document-catalog proofs. Each
    session has one non-resettable owner and charges repeated input again.
54. Final adversarial RR found and closed five residual gaps: actual checked
    `SignatureV2`/module-resolution signatures now replace the stale API sketch;
    checked type payload byte ownership is explicit; unsupported `sum(*)` and
    absent declarations have distinct negatives; and fingerprint mutation
    coverage changes every dependency/suite field, not merely collection
    presence.
55. Consolidated module proof is 109 tests (1 unit + 108 integration); types is
    56 tests. The six deleted V1 module suites' 56 authored cases are mapped
    one-by-one in `receipts/03-test-disposition.md`, including three intentional
    contract deletions rather than compatibility assertions.
56. Captured the complete simplified public-API output hashes and all 52 types
    plus 76 modules exported type names in `receipts/03-public-api-inventory.md`.
    Duplicate scans found no `SignatureInstanceV1`, public sibling `TypeV2`, or
    default-constructible closed bindings/activations.
57. A deliberate aggregate-Star mutation was killed by the exact owner test,
    reverted, and followed by full owner tests and strict Clippy. Receipt 03
    records the RED and restored GREEN without leaving the mutation in source.
58. External commits `ceb4142`, `6583091`, and `f9ec954` were separately audited.
    They are unqualified consumer migrations, not Ticket 04/05/07 completion;
    their exact binder/algebra/planning/pipeline gaps are written into those
    tickets and the execution ledger.

Ticket 03 owner scope is verified by `receipts/03.md`. This does not claim a
clean ticket-only commit, hosted CI, workspace closure, or any Ticket 04-09
consumer completion.

Verification artifacts:

- [Ticket 03 receipt](../receipts/03.md)
- [exact public API inventory](../receipts/03-public-api-inventory.md)
- [56-case deleted-test disposition](../receipts/03-test-disposition.md)

## Ticket 04 Producer-Seam Reopen

Ticket 04 exposed one types-owner omission that its consumer may not patch
around: a later stage reference such as `project ... as x | order by x` needs
the type of an earlier expression, while `ResolvedTypeInputValueV2` accepted
only a caller-supplied completed type. Supplying that type from sema would make
sema a second type checker; invoking the kernel by stage would violate the
single document-kernel contract.

The canonical types owner therefore adds exactly one dependency carrier:
`BindingFromExpression(SourceQualifiedNodeIdV2)`. Admission rejects foreign,
missing, and self targets. The kernel accepts only a target fact already
produced in document traversal order; forward and cyclic dependencies reject
as `types.unavailable_binding_dependency`. The carrier copies the checked type
state, not effects or call identity, because reading a materialized row binding
does not re-evaluate its producer. Direct, foreign/missing/self, forward, and
cycle rails live in the existing types suite. Receipt 03 must append the final
full-suite/Clippy proof before returning to `VERIFIED`.

Forward document declarations are a distinct legal edge: `let a = b; let b =
1` is predeclared by Ticket 04, but `BindingFromExpression` intentionally
rejects a future stage value. Types must add a separately checked
declaration-reference input that proves both reference and target belong to
predeclared `DeclHirV1` bodies in the same checked document. The kernel may
resolve that target lazily through one node index during its one document pass,
with a visiting/done cycle guard. It must not relax the prior-fact rule for
relation stages. Tests: forward declaration succeeds, self/mutual declaration
cycles reject with stable witness, and a later relation alias remains rejected.

The subsequent Ticket 04 adversarial owner/consumer audit found two more
producer omissions. They are **not** covered by the earlier 56-types/109-modules
verification; Ticket 03 remains `REPAIR` until a new receipt delta and owner
tests prove them.

1. **Types rule-variable proof.** Add one bounded, deterministic finite-domain
   constraint solver to the existing types owner. It takes checked HIR plus
   exact Binder binding identities, resolved callable IDs/signatures, and the
   source-rule unary predicate IDs supplied by sema. Types, not sema, extracts
   literal/operator/builtin/call constraints and runs a monotone worklist.
   Candidate concrete types come only from sealed host signatures and closed
   operator/builtin/literal domains; no guessed default, unbounded type
   variable, or second overload engine. A sealed output maps each exact
   rule/quantifier binding and source predicate ID to one concrete type or
   reports contradiction/ambiguity/unanchored recursion. Empty intersection
   rejects; multiple candidates or unanchored self/mutual recursive component
   rejects closed compilation and is explicit Unknown only in open analysis.
   It must charge node/edge/iteration work against an immutable positive budget
   projected from `FrontendBudgetV2`, with exact-limit and one-over rails.
   The existing `ExpressionTypeKernelV2::check_document` then runs once using
   this proof; it does not infer those variables again. Required owner tests:
   anchored int self/mutual recursion, conflicting int/string anchors,
   ambiguous overload, unanchored self/mutual recursion, quantifier anchoring,
   unknown external signature, foreign/duplicate binding ID, arity mismatch,
   deterministic permutation, and budget boundary. This is needed because
   `rule r : x if r(x)` has no source of concrete type and cannot be a closed
   positive test merely by setting `x` to a convenient scalar.
2. **Modules document predeclaration and family references.** Extend the
   existing `DocumentCatalogValidatorV2` single session with predeclaration,
   typed local/host/external reference resolution, and final validation using
   the same IDs/counter. `CatalogAssetIdV2::derive` remains nonpublic; sema
   cannot replicate it. `DocumentCatalogDependencyInputV2` currently accepts
   only a local View source; add Model, Pack, Profile, Gate, and RuleBinding
   reference policy without widening View cycle detection to unrelated edges.
   Preserve exact checked registry identity and provenance on every minted
   location. The owner matrix needs positive forward/import/external refs,
   unknown/wrong-kind/ambiguous refs per family, duplicate ID, non-View edge
   without false View-cycle, and cumulative budget exact-limit/one-over.
   Receipt 03 must record changed public API, existing-test disposition, and
   zero second catalog-resolver definitions before `VERIFIED` resumes.

## 2026-09-21 Live-Tree Final Reconciliation (HEAD `f9ec954` + dirty overlay)

This is an execution checklist, not a fresh green receipt. The earlier 56-types/
109-modules counts cover the pre-reopen API and must not be reused for this
overlay.

| Owner | Already implemented in the overlay | Still required before `VERIFIED` |
|---|---|---|
| HIR | owned checked-document proof and invalid/valid owner tests | rerun full HIR owner suite and strict Clippy on the final producer fingerprint; prove sema borrows this proof without a second integrity pass |
| Types | exact extended-registry input; prior-expression dependency; declaration-only forward reference; initial finite-domain rule solver using the existing operator/builtin registries | complete the solver's mutual-recursion, quantifier, host-signature, arity, permutation, forged-ID, open-Unknown, and exact budget matrix; prove the single expression-kernel pass; full owner suite and strict Clippy |
| Modules | same-session `predeclare_assets`/`resolve_reference`/`validate_predeclared` path; inert catalog target kinds and full-path parser | finish every family positive/wrong-kind/unknown, local/import/external ambiguity, View-only cycle, foreign registry, validator-reuse, and cumulative exact-limit/+1 tests; full owner suite and strict Clippy |

The types solver is an extension of the canonical checker/registries, not a
second sema inference engine. The modules predeclaration is an extension of
`DocumentCatalogValidatorV2`, not a sema map or a second asset-ID derivation.
`validate(input)` may delegate to the same session, but an accepted reference
must resolve to the identical `CatalogAssetIdV2` before and after final
validation. Owner tests must assert that identity and that the counter cannot
reset through either entry point.

Rule-solver ingress must reject a same-spelling symbol mapped to a binding in
another rule or quantifier scope; matching text and an existing `(owner,
ordinal)` are not Binder proof. Likewise a caller-provided Builtin family or
host predicate signature must be tied to the checked call's callee spelling
and sealed callable authority, not merely to arity or a family ID that exists
somewhere in the registry. Add cross-rule same-name and cross-call target
transplant negatives. Types cannot depend on Binder under the final DAG; sema
imports Binder's exact output and seals the cross-owner mapping before
invoking the types solver.
The shared Boolean parser is also used for source rule bodies and currently
accepts `exists (from ...)` there (`surface/src/unit.rs::parse_rule` calls
`parse_bool`). The initial rule solver's generic unsupported result for
`RelationExists` was therefore a reachable language gap. Ticket 00 now pins
the explicit rule-body semantic prohibition; the dirty solver returns a
dedicated `RelationExistsInRule` diagnostic and Ticket 04 has a focused
exact-span source reject. Retain both in the final receipt. Query EXISTS
remains a supported typed/capturing path.

The live modules resolver currently filters same-spelling candidates to the
expected kind before checking the remainder. Under the pinned fail-closed
policy, a different-kind local/import/external candidate with the same
spelling cannot be silently discarded while another kind wins. Add an exact
cross-kind collision fixture and reject deterministically. Final validation
must also prove a `ResolvedDocumentCatalogReferenceV2` came from this
predeclaration/visibility session, not merely that a same-named ID exists in
some authority sharing its source registry.

The fresh receipt must list the exact changed public symbols, selected and
executed test counts, Clippy exit, failure-injection outcome, and scoped dirty
fingerprint. Until then Ticket 03 is `REPAIR`, even if `cargo check --lib` or a
five-case solver subset passes.

## 2026-09-21 Second Live Producer Audit — No Duplicate Solver/Resolver

The dirty implementation has moved beyond the first subset, but no combined
producer/consumer fingerprint has been verified:

1. Types extends the existing `ExpressionTypeKernelV2`, operator registry, and
   builtin registry. `types/src/check.rs` now has declaration-only lazy
   `BindingFromDeclaration` dependency and cycle rejection; relation-stage
   `BindingFromExpression` stays prior-fact-only. `types/src/solver.rs` owns a
   finite-domain rule constraint worklist, exact lexical binding-scope checks,
   closed/open results, and a positive node/edge/step budget. It is not a
   second general type checker. The latest types all-target suite passed
   78/78 (7 unit, 16 solver, 25 check-edge, 16 hellgate, 14 universe) after
   exact-budget and relational-EXISTS defense tests. Strict all-target Clippy
   passed on the same checkpoint. The earlier
   67-test suite is historical only.
2. Finish rule `Record` and `List` constraints in the same solver, including
   contextual empty-list typing, nested shapes, ambiguity, and precise
   diagnosis. The current `solver.rs` has Record/List constraint branches and
   a positive constructed-value rule test; contextual empty list and the
   negative/permutation matrix still need explicit proof. That positive needs
   a 100,000-step test allowance. After RR added charge-before-work to
   universe seeding, host-param cloning, list/record candidate expansion,
   tuple materialization, and propagation support, its measured usage is
   15 nodes, 20 edges, 6,884 steps over 74 candidate types. Exact 6,884
   passes; 6,883 steps, 14 nodes, 19 edges, and a one-step seed reject.
   Latest 78/78 tests and strict Clippy passed on this correction. This is
   not a byte-level bound for every fixed tuple/input map allocation; audit
   host-signature/type-shape universe growth and show Ticket 04
   projects the canonical frontend budget without silently increasing it.
   Keep the explicit
   direct-solver rule-body relational EXISTS reject, and prove the source-level
   `sema.invalid_item_shape` boundary pinned in Ticket 00. Add closed and open
   Unknown cases and exact budget/effect/overload permutation rails. Do not
   manufacture rule types in sema.
3. The caller-provided host predicate target is not authority by itself.
   Seal it to the checked host signature and authored callee at the
   sema/modules ingress, then test a signature transplanted between same-arity
   calls, a cross-rule same-spelling Binder target, duplicate/foreign node IDs,
   wrong arity, and unknown/ambiguous callables. The types solver cannot import
   Binder under the final DAG; sema must supply and verify exact Binder IDs.
4. Modules extends `DocumentCatalogValidatorV2` in one session instead of
   introducing a sema-side resolver. `modules/src/authority.rs` now
   predeclares, resolves, and validates with a validator-local session token;
   same-spelling cross-kind candidates reject before expected-kind filtering,
   and a local declaration colliding with a distinct host declaration of the
   same asset ID rejects. RR also closed distinct visible-host/external
   origin collisions, foreign-owner predeclarations, and same-session
   `probe_reference`: only true absence returns `Ok(None)`; wrong kind,
   ambiguity, foreign authority, and budget failure terminate with typed
   errors. The latest modules owner run passed 122/122 and strict Clippy on
   the hashes below. Sema still must supply every source family reference and
   compare exact validated identities; inert API targets do not prove source
   reachability. EXPLAIN/CHECK dual-namespace integration remains Ticket 04.
5. Update `receipts/03.md` with final owner hashes, changed public symbols,
   current full-suite/Clippy counts, one failing mutation and recovery, and
   cross-owner Ticket 04 integration. Do not restore `VERIFIED` from the older
   receipt or a compilation-only check.

Modules owner-local post-probe checkpoint: 122 tests passed (1 unit, 13 new
predeclaration/probe tests, 108 existing; zero doctests), strict all-target
Clippy passed, and scoped diff-check passed. `probe_reference` returns `None`
only for true absence; wrong kind/ambiguity/registry/budget errors terminate
the same session. The foreign-owner and distinct-host-origin cases are now
covered. This closes the modules *owner-local implementation* items above on
its recorded fingerprint, not the Ticket 03 cross-owner sema ingress, types
solver, mutation, or final receipt gates.

HIR's owned checked-document producer has also now passed its full 70-test
owner suite and strict Clippy on the hashes in `receipts/03.md`. Sema must
still consume that exact owned proof without rerunning integrity or admitting
an unchecked document.

## Latest Types Owner Supersession And Catalog Identity Gate

The 78-test/6,884-step result above is an earlier fingerprint. The latest
types owner-local frozen source passed 79/79 all-target tests and strict
Clippy; the Record/List candidate universe remains 74, with shape-aware
usage 15 nodes, 20 edges, 8,632 steps. `ValueTypeV1::node_count()` is charged
for registry/host signature shapes and set clones before allocation. Exact
and one-under usage and a nested-host-signature scaling test passed. The
source hashes and count breakdown are in `receipts/03.md`. Do not treat this
as a sema-source or byte-level total-memory proof.

Host predicate identity remains an explicit producer gate. Types
`ResolvedPredicateIdV2::try_new` accepts the canonical ASCII `::` grammar,
while modules `CatalogAssetIdV2` admits punctuation in local asset names;
`SignatureV2::try_as_resolved_predicate(id)` accepts an ID supplied by its
caller and does not itself prove an injective catalog-to-types mapping. Either
Implement an injective modules-owned typed identity projection with an
exact catalog asset ID round trip. A temporary sema rejection of unsupported
names is fail-closed during the cutover but is not a final Ticket 03/04
completion condition, because those names are legal catalog inputs. Prefer a
distinct typed host target/key carrying length-prefixed canonical asset
identity over squeezing arbitrary punctuation into the source-predicate
ASCII grammar or hashing into a collision-prone display token. Sema must
retrieve the signature from the exact
resolved catalog asset and match the payload kind; a display-name hash,
punctuation stripping, or caller-selected predicate ID is forbidden.
Owner/consumer tests: two punctuation-distinct legal names remain distinct
and source-callable, wrong payload kind and same-spelling local/host collision reject;
the accepted signature's parameter types and source catalog ID are unchanged.

### Newly audited rule-solver producer repairs

1. At the audited pre-repair checkpoint, `RuleTypeSolverV2::analyze_open` invoked recursive
   `Expected::visit_rule` and fills maps/sets/cloned names before creating
   `prepass_steps` or charging the supplied budget (`solver.rs:385-399` versus
   first charge near `:589`). Move budget admission to the front of this
   prepass: charge every visited rule/expression/call/binding and each cloned
   name/set before growth, check source-qualified node and step caps, and use
   an iterative work stack or a separately checked maximum depth. A checked
   HIR integrity proof counts nodes but does not cap them. A programmatic
   checked oversized/deep rule must fail with the exact resource diagnostic
   before unbounded allocation or stack growth. Test exact limit, one over,
   and a deep but otherwise valid quantifier/expression chain.
2. At the audited pre-repair checkpoint, `Expected` stored every active binding owner for a symbol and ingress
   accepted any same-name owner in that set (old `solver.rs:239-262,463-472`). This
   admits a forged outer binding for an inner same-name quantifier. Resolve
   the nearest lexical same-name `(owner, ordinal)` expected binding (or
   compare to an independently sealed Binder map) and require exact equality.
   Add `rule r : x if exists x in (x = 1)` with a valid inner target and a
   forged outer-head target; the latter must reject at the inner symbol span.
   A different-name quantifier test cannot detect this shadowing failure.
3. A source predicate call cannot be typed by sema from
   `RuleTypeProofV2::predicate_type` alone without fabricating the callable's
   Bool result/effects. Add a types-owned `source_callable(id)` projection (or
   equivalent sealed result) from the inferred unary predicate proof. It
   returns the exact ID, one inferred parameter type, Bool result, and the
   fixed source-rule effect policy; absent/foreign IDs reject. Sema uses this
   value in `ResolvedTypeInputsV2` and never calls
   `ResolvedPredicateCallableV2::try_new` with guessed semantics. Prove one
   anchored self-call, a mutual call, and negative/aggregate polarity through
   the source transaction before Ticket 04/06 can count recursion reachable.

Owner-local supersession: the three types-owned repairs above now pass an
81/81 full owner suite and strict Clippy on the hashes in `receipts/03.md`.
The `Expected` prepass is iterative and charged before growth, with depth
64 accepted/65 rejected; nearest same-name lexical binding rejects forged
outer shadow targets; `RuleTypeProofV2::try_source_predicate_callable` emits
the inferred unary/Bool/conservative-Scalar signature. Sema has separately
shown anchored source recursion and negation from source on a later 34/34
owner run. This does not prove mutual/aggregate source recursion, host
predicate mapping, full Ticket 04, or a combined Ticket 03 receipt.

### Host predicate identity implementation split

The read-only seam audit located `CatalogAssetIdV2` as the typed
`(ModuleIdV2, NamespaceKindV2, local_name)` tuple; legal local names include
`.`, `/`, `:`, and `::`, and can exceed the 256-byte source
`ResolvedPredicateIdV2` grammar. The final repair is disjoint:

1. Types owns a distinct `ResolvedHostPredicateKeyV2` encoded injectively
   from length-prefixed canonical module/kind/local components, with no
   truncation, hash-only identity, punctuation replacement, or source-ID
   grammar reuse. `ResolvedCallableInputV2`, `ResolvedCallTargetV2`, and
   `RuleCallableTargetV2` distinguish host from source predicate. Retain the
   host signature's exact parameter/result/effect facts in the typed call.
   Check component lengths and charge the encoded length before allocating
   the key; a public types constructor must not create an unbounded vector
   ahead of modules' authority budget. Test exact/+1 key bytes and long
   punctuation-distinct assets under the module session limit.
2. Modules owns a nonforgeable same-session
   `DocumentCatalogValidatorV2::resolve_host_predicate_call` proof: call its
   exact `resolve_reference(expected_kind=Predicate)`, look up that ID in
   `external_catalog().asset(id)`, require a Predicate payload, project the
   types host callable from the full catalog tuple, and retain checked use
   location plus asset ID. Sema cannot manufacture the proof from a string or
   select an unrelated signature.
3. Sema routes source-local and host predicate calls separately, verifies
   the exact authored callee/use location against the modules proof, passes
   its typed callable into the one expression kernel, and emits SCC edges
   only for source-local predicates. Tests cover punctuation-distinct and
   long legal names, same-spelling source/host ambiguity, wrong kind,
   wrong session, unproven alias/ID transplant under the existing resolver
   visibility policy, cross-call target transplant, and
   immutable signature effects. Ticket 06 verifies host calls do not become
   local SCC nodes.

Types/modules owners may implement steps 1-2 in parallel only after sharing
the exact type API; Ticket 04 consumes the frozen proof and never edits either
owner to make a source test pass. Both owner-local receipts and a sema source
integration positive are required before this gate closes.

### Latest host-proof owner checkpoint and remaining gate

The 81/81 types and 122/122 modules counts above are prior fingerprints.
Types' distinct host/source identity and injective length-prefixed key passed
83/83 all-target tests, then exact intrinsic builtin effects passed 85/85;
both owner-local runs passed strict Clippy. Modules' same-session exact
Predicate/signature proof passed 126/126 all-target tests and strict Clippy.
Full owner file hashes and source/test counts are in `receipts/03.md`.
These changes reuse the existing type kernel and catalog validator; they do
not add another resolver or checker.

Ticket 03 stays `REPAIR`. Ticket 04 has a source host-call positive and one
source/host same-spelling collision negative in a later 43/43 sema owner run,
but still needs builtin/host collision, legal punctuation names, imported
alias origin, proof-key/location transplants, and host exclusion from local
SCC edges. Then run a combined owner/consumer command set on one frozen dirty
fingerprint, plus the required
mutation and public API diff. An owner-local count must not be reported as a
cross-owner or pipeline result.

### Intrinsic aggregate effect lookup required by Group

Ticket 04's aggregate-only partition-input seam cannot authorize a call from
the composed `TypedExpressionFact.effects`: a scalar `abs(sum(x))` inherits
aggregate effects from its child. Types owns a read-only intrinsic-effects
lookup keyed by the exact chosen `BuiltinIdV2` from
`ResolvedCallTargetV2::Builtin`, with absent IDs rejecting. Do not expose a
family-name/string classifier or duplicate the builtin registry in Binder or
sema. That API and its owner tests now pass 85/85 all-target tests and strict
Clippy on the `builtin.rs` hash in `receipts/03.md`. The tests distinguish
scalar wrapper intrinsic effects from aggregate-composed effects and two
overload IDs in one family. Sema's later 52/52 owner suite uses this exact
ID for Group partition admission and Rule edge polarity; candidate
forgery/transplant mutation, sema strict Clippy, and combined frozen
producer/consumer proof remain open. The types result alone is not closure.

## Final Execution Audit — `main@ecc38f59` + Dirty Overlay

Verdict: canonical Types and Modules owners are implemented and current
owner-local suites are green, but Ticket 03 remains `REPAIR` because its
consumer seam and mutation receipt are incomplete. Do not add another solver,
builtin registry, catalog resolver, or host-key codec.

Reuse exactly:

- Types 85/85: closed overload resolution, iterative/budgeted rule constraint
  proof, distinct source/host callable identity, injective host key, and exact
  builtin-ID intrinsic-effects lookup;
- Modules 126/126: one admission/lock/catalog validator, same-session
  predeclaration/finalization, ten-kind external payload matrix, and checked
  host Predicate proof;
- strict all-target Clippy for both owners, included in the latest combined
  owner command, exits 0.

Required remaining work, in order:

1. Add Ticket 04 source integration fixtures for legal punctuation-heavy host
   Predicate names and imported alias origin; assert exact catalog asset ID,
   callable signature, use location, and host/source target kind.
2. Add negative transplants for foreign validator session, same key with wrong
   location, same display spelling with wrong asset ID, and swapped host-call
   proof. Each must fail before expression materialization.
3. Prove builtin/source/host same-spelling collision and that host/builtin calls
   never enter the local Rule dependency set consumed by Ticket 06.
4. Run one frozen producer/consumer command set covering Types, Modules, Sema,
   strict Clippy, and the exact mutation. Record hashes and counts in
   `receipts/03.md`; do not replace owner receipts with a Sema-only pass.
5. Remove no public API merely to obtain zero warnings. Temporary Sema
   `#[expect(dead_code)]` constructors disappear only when Ticket 04 makes
   their source paths reachable.

Stop condition: punctuation replacement, truncation, or hash-only catalog-name
identity is invalid and must not proceed.

## Final Independent RR — Composition Row/EXISTS Authority Corrections

Ticket 03 remains `REPAIR`. Do not duplicate the existing solver, registries,
host proof, validator, or key codec. The missing owner contract is a
Types-owned deferred relation-row dependency for Composition Query producers.

### One whole-document kernel call

`BindingFromExpression` copies one prior scalar fact and cannot by itself
represent an ordered multi-field Query output row. Add one closed deferred row
carrier keyed by exact producer/root identity whose ordered fields retain name,
expression key, and checked location. Types owns record construction during the
same authored-order document traversal. Admission rejects duplicate fields,
missing/non-expression targets, foreign source/registry, self/forward producer,
wrong field order, and a caller-precollapsed record. Sema supplies Binder/HIR
identity only; it does not construct `ValueTypeV1` for the row.

Required owner tests: one- and multi-field prior Query producers; missing,
duplicate, reordered, nonprojection, self, forward, cyclic, and foreign
dependencies; and an invocation counter proving exactly one
`check_document`. Per-LET kernel calls, a second Sema type checker, or
Sema-synthesized producer record types are stop conditions.

Concrete non-Query fields remain direct known bindings: host/View authority,
SAMPLE `path: RepoPath`, and PATH/REACHABILITY FIND `function: EntityRef`.
Filter/Sort pass the exact deferred frame; Join concatenates exact frames;
Project/Group create deferred fields from exact expression nodes.

### Relation EXISTS input

Current `ResolvedRelationPredicateInputV2` stores one relation source plus an
opaque capture string. It cannot represent join/multi-source input or prove
Binder capture identity. Replace it with a Types-neutral carrier containing the
exact relation root and an ordered, duplicate-free list of exact capture
reference plus target owner/ordinal, with conservative base effects. Validate
same-document membership, subtree membership, uniqueness, and wrong-kind facts
without importing Binder/Sema/Algebra/Modules. Mutations: dropped, duplicate,
swapped, foreign, wrong-root, same-name/wrong-owner, and multi-source captures.

### Modules/receipt reconciliation

Inline View order is: one validator session predeclares all assets, resolves
View edges, rejects cycles and seals deterministic topological order, then Sema
materializes inline queries and finalizes that same session. No second
validator or pre-cycle materialization.

Expand the authoritative write set for live `types/src/solver.rs`, rule-solver
tests, `modules/src/authority.rs`, document-predeclaration tests, API inventory,
and receipt appendices. Supersede stale Required APIs/prose with live
`BindingFromExpression`, `BindingFromDeclaration`, HostPredicate proof/key,
rule solver, and deferred-row API. Regenerate the public API inventory; the
`f9ec954` 52/76 list and two Types test hashes are stale.

Final receipt proof includes Types/Modules all-target tests, HIR producer and
Sema consumer/mutation tests, owned-file format, strict Clippy, public API diff,
deliberate failure injection, frozen hashes, and selected/executed counts. Mark
the old “Sema has not consumed host proof” receipt paragraph historical: live
Sema now consumes and revalidates that proof.
## 2026-09-21 Final Current-Tree Audit — Implemented Versus Still Required

Status remains `REPAIR`, but the final RR producer gaps are no longer all RED.
Reuse the current implementations; do not create parallel carriers or another
kernel/validator.

Implemented and owner-green on the current dirty overlay:

- Types owns deferred relation rows through
  `DeferredRelationRowInputV2`/`DeferredRelationRowKindV2`, including known
  fields, prior-expression fields, and an exact prior-relation dependency.
- Types owns exact relational-EXISTS inputs through relation root plus ordered
  `(reference,target owner,target ordinal)` captures; the opaque capture string
  is gone.
- `TypedExpressionFactsV2` retains owner-built relation rows; Sema invokes
  `ExpressionTypeKernelV2::check_document` once for the document.
- Types all-target tests are 86/86 GREEN and strict Clippy is GREEN.
- Modules all-target tests are 126/126 GREEN and strict Clippy is GREEN.

Remaining owner defects/gates:

1. `PriorRelation` currently ignores `expected_field_count`. Replace the
   meaningless caller-supplied `0` with a structurally count-free variant, or
   verify a count derived by Types after resolving the prior row. Do not make
   Sema precompute the record.
2. Add the missing deferred-row mutations: duplicate input root, wrong source,
   non-relation target, reordered projected fields, and exact self/forward
   boundary cases in one table-driven owner suite.
3. Add the full EXISTS mutation matrix: dropped, duplicate, reordered/swapped,
   foreign, wrong root, same-name/wrong-owner, and multi-source captures.
4. Modules needs a same-session phase between predeclaration and finalization:
   resolve direct View edges, reject cycles, and return an opaque deterministic
   topological-order proof. Current `validate_predeclared` performs final
   validation immediately; Sema cannot materialize inline Views in proved
   dependency order with that API.
5. Regenerate the public API inventory and append fresh hashes/counts to
   receipt 03. The old 85-test Types fingerprint is superseded but remains
   historical.

Consumer closure belongs to Ticket 04: source-reachable Composition, inline
View, and Query EXISTS. Their absence must not trigger another Types checker or
another Modules validator.

## SS-03C Owner Checkpoint And Final RR Corrections — 2026-09-21

Status remains `REPAIR`. The current dirty overlay has two additional owner
implementations that must be reused:

- Types replaced the overloaded deferred-row constructor with
  `known_source`, count-free `prior_relation`, and `expression_fields`.
  Duplicate roots, foreign roots, self/forward targets, non-relation targets,
  and field-count/order failures reject in the Types owner.
- Modules now enforces
  `Empty -> Predeclared -> TopologyValidated -> Finished`, terminalizes phase
  misuse as `Failed`, returns an opaque same-session
  `DocumentViewTopologyProofV2`, and finalizes only with the exact proof.
  The proof retains deterministic dependency-first View IDs.

Observed on one overlay:

```text
Types all-target: 86/86, exit 0
Types strict all-target Clippy: exit 0
Modules all-target: 129/129, exit 0
Modules strict all-target Clippy: exit 0
owned package rustfmt check: exit 0
git diff --check: exit 0
Modules deliberate order.reverse() mutation: order oracle failed, then recovery GREEN
```

This does not close Ticket 03. RR found these exact remaining obligations:

1. `ResolvedRelationPredicateInputV2` must not advertise more authority than
   it proves. Types currently validates source/root/reference order and global
   owner membership, but does not validate `target_ordinal`, exact
   owner/ordinal substitution, or capture-set completeness. Binder is the
   lexical authority: Sema must compare the complete ordered capture vector
   to Binder's exact set before constructing the Types-neutral carrier. Types
   then validates HIR membership, uniqueness, source and deterministic order.
   Do not add a Types-to-Binder dependency or infer captures from names.
2. Remove caller-controlled relation-predicate base effects. Types derives the
   base effect policy and composes nested expression effects; Sema does not
   pass `pure_scalar()` as an authority claim.
3. Align capture ordering. Binder's tuple sort and Types' nested-HIR order
   currently disagree for programmatic non-monotonic IDs. Pin one canonical
   authored traversal order and add a reversed-ID two-capture fixture.
4. Add a positive direct `PriorRelation` owner test, swapped expression
   targets with unchanged ordinals, foreign prior target, and a three-producer
   earlier-unrelated-row transplant. Add an invocation oracle that fails if a
   second document type-kernel call is introduced.
5. Add a diamond plus unrelated View under multiple asset/edge permutations.
   The current linear chain proves dependency direction but not stable
   tie-breaking among multiple ready nodes.
6. Refresh the public API inventory and record the combined producer/consumer
   mutation set on a frozen fingerprint. The hashes below are dirty-overlay
   checkpoints, not an immutable receipt.

Stop conditions remain: no second type checker, no second catalog validator,
no name-keyed dependency graph, and no Sema-created record type.

## SS-03D Relation EXISTS Effect Authority — 2026-09-21

Status remains `REPAIR`, but one RR P0 is now closed at its true owner.
`ResolvedRelationPredicateInputV2::try_resolved` accepts only `(relation root,
captures)`; it cannot receive a caller-supplied effect tuple. Types derives the
resolved EXISTS base effects with `conservative_source_predicate()` and composes
the nested relation effects. Sema no longer supplies `pure_scalar()`.

The Types input owner also rejects duplicate reference captures at construction,
collects relation-source HIR nodes for Composition-local captures, and checks
root/source/HIR membership/duplicate/authored traversal order. The two-capture
swapped-order oracle fails. This is not a claim that Types can prove Binder's
lexical owner/ordinal substitution: Sema must compare the full capture vector
with Binder proof before Types ingress.

```text
Types all-target: 87/87, exit 0
Types strict all-target Clippy: exit 0
Sema all-target: 64/64, exit 0
Sema strict all-target Clippy: exit 0
cargo fmt --all --check: exit 0
git diff --check: exit 0
```

Remaining Ticket 03 work is exact cross-owner capture-vector equality,
PriorRelation/topology mutation expansion, public API inventory, and a frozen
producer/consumer receipt. Do not restore a caller effect parameter to make a
consumer test convenient.

## SS-03E Prior-Relation Source Authority Oracle — 2026-09-21

Types now has a direct positive `PriorRelation` owner oracle. A nested EXISTS
relation source reuses the exact earlier source root's sealed typed row; a
foreign-source target rejects as `types.unavailable_relation_row_dependency`.
The fixture supplies the real resolved EXISTS input, so it proves the deferred
row path rather than bypassing relation-predicate validation.

```text
Types all-target: 88/88, exit 0
Types strict all-target Clippy: exit 0
cargo fmt --all --check: exit 0
git diff --check: exit 0
types/tests/v3_types_check_edges.rs=dc1441f237ed8957c18f053340335bc7c91ba5acd46d9df9506b9a9b49d8d7f0
```

This oracle intentionally covers the existing authored-order contract only.
Inline View requires Modules dependency-first evaluation order, which can be
reverse-authored; Ticket 04 must introduce a Types-validated pre-evaluation
order before relaxing any `PriorRelation` order guard. Deleting the guard would
admit forged/cyclic rows and is forbidden.

## SS-03F Validated Relation Pre-Evaluation — 2026-09-21

The former Ticket 04 prerequisite is now implemented at the Types owner.
`RelationPreEvaluationInputV2` is validated against the checked document before
the kernel walks any body: every root must have the document source identity,
be an exact relation root, be unique, and place every planned
`PriorRelation` target earlier in the supplied sequence. The prewalk happens
once before normal authored traversal. Existing `ResolvedTypeInputsV2::try_new`
remains the no-plan/authored-order constructor.

An RR repair closed the consumer boundary: an unplanned ordinary relation may
read a target in the validated plan because all plan roots have already been
completed. It may not read any other forward row. The owner fixture proves a
reverse-authored dependency, a normal consumer outside the plan, omitted and
swapped order rejection, foreign/non-relation/duplicate root rejection, and
exact copied-row equality.

This input is deliberately not a second topology graph. Modules remains the
only View graph/proof authority; Sema may project its opaque order into this
validated Types input. The remaining Ticket 03 work is unchanged: exact Binder
capture-vector equality, topology diamond permutation coverage, public API
inventory, and frozen cross-owner receipt.

## SS-03G Final Current-State Re-Audit — 2026-09-21

Audit snapshot: `main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a`,
174 short-status entries. The original receipt and SS-03C through SS-03F remain
historical checkpoints. This section is authoritative for current status.

### Implemented owner contracts

- Binder records exact relation-source use targets and explicit empty/nonempty
  relational-EXISTS capture sets. `BindResultV1::capture_edges` carries
  `(reference, owner, ordinal)` and `target_of(reference)` supplies the exact
  local binding proof.
- Types owns checked deferred relation rows, count-free `PriorRelation`, exact
  relation-predicate root/capture membership and authored HIR order, owner-built
  effects, and validated `RelationPreEvaluationInputV2`.
- Modules enforces `Empty -> Predeclared -> TopologyValidated -> Finished`,
  returns same-session opaque `DocumentViewTopologyProofV2`, rejects phase/proof
  substitution, and supplies deterministic dependency-first View IDs.
- Sema consumes these proofs. In `sema/src/document.rs`, relation EXISTS checks
  every Binder capture against its exact local `(owner, ordinal)` before Types
  input construction and before sealing `TypedCorrelationEdgeV2`. Inline Views
  consume Modules topology and project its order into Types pre-evaluation.
- The former prior-relation and linear-topology producer gaps are implemented.
  Do not reopen them or create parallel carriers.

### Current execution evidence

Executed on the audit overlay:

```text
cargo test -p quanta-gqlang-v2-binder --all-targets   # 54/54 GREEN
cargo test -p quanta-gqlang-v2-types --all-targets    # 89/89 GREEN
cargo test -p quanta-gqlang-v2-modules --all-targets  # 130/130 GREEN
cargo clippy -p quanta-gqlang-v2-binder --all-targets -- -D warnings   # GREEN
cargo clippy -p quanta-gqlang-v2-types --all-targets -- -D warnings    # GREEN
cargo clippy -p quanta-gqlang-v2-modules --all-targets -- -D warnings  # GREEN
cargo fmt --all --check                               # GREEN
```

The current Binder/Types/Modules/Sema binary diff hash was
`83bf674a0da29ddc767ae2d2ccb908169270b8e664725e97224cf286ffef41ac`;
the status-list hash was
`d75d5b14abd9dde08571d6fda7758ec828eba176098fc0e2ca4078c52c3000ec`.
These include concurrent Sema work and do not create a clean Ticket 03 receipt.
Sema's separately observed 87/87 belongs to Ticket 04 and is consumer evidence,
not part of this audit's executed owner command set.

### Remaining concrete work

1. Add a Sema negative capture-vector mutation table in
   `sema/tests/v3_document_transaction.rs`: dropped edge, duplicate reference,
   reversed two-capture order, correct reference with wrong owner, correct owner
   with wrong ordinal, foreign reference, same-name/wrong-owner, wrong nested
   relation root, and explicit empty-set omission. Assert exact code/location;
   positive shape assertions are insufficient. Because `BindResultV1` is
   intentionally opaque, factor the existing comparison into a private Sema
   validator and mutate its read-only witness inputs in a unit test; do not add
   a public Binder forgery constructor or expose mutable capture storage.
2. Add a Modules topology diamond plus unrelated ready View under multiple
   asset and edge permutations in
   `modules/tests/v3_document_predeclaration.rs`. Assert identical exact
   `dependency_first_order`, not only a linear reverse-authored chain. Retain
   the same-session dropped/reordered/foreign proof tests.
3. Strengthen Types pre-evaluation mutation in
   `types/tests/v3_types_check_edges.rs` with a three-producer transplant and
   swapped targets that retain plausible ordinals. Assert exact copied row and
   fail-closed code.
4. Add an observable one-document invocation oracle at the Types/Sema seam.
   `ExpressionTypeKernelV2::check_document` must be called exactly once; a
   second pass for Views, Composition, or EXISTS must fail the test. Keep any
   counter/injection private under `cfg(test)`; do not put execution counters in
   the public semantic document contract.
5. Regenerate `receipts/03-public-api-inventory.md` for Types and Modules and
   verify no public constructor can mint a topology proof, checked authority,
   typed fact, or pre-evaluation result.
6. Freeze Binder/Types/Modules plus exact Sema consumer tests on one fingerprint;
   rerun owner tests, strict Clippy, scoped format/diff, deliberate mutations,
   and record selected/executed counts and file hashes in receipt 03.

### Owner boundary and duplicate-work prohibition

- Binder alone determines lexical capture identity and order. Types validates
  source/HIR membership and effects; it must not depend on Binder.
- Modules alone computes the View dependency graph and topology proof. Sema may
  project opaque order, never rebuild or sort a name-keyed graph.
- Types alone builds relation rows and expression facts. Sema may not precompute
  a record or pass caller-controlled effects.
- Sema alone performs cross-owner equality. Do not add a second checker to
  Algebra, Logic, or Pipeline.

### Next SS / RR checklist

`SS-03H` is the frozen cross-owner mutation checkpoint. RR requires all six
remaining actions above, unchanged owner hashes across every command, proof
that deliberate mutations fail then recovery passes, public API diff review,
and no new Binder<->Types, Sema topology, or caller-effect authority edge.

## SS-03H Execution Routing

This section clarifies scheduling only; the final re-audit above remains the
semantic worklist. Under `WAVE_EXECUTION_PLAN.md`, SS-03H is not one unrestricted
writer:

- the Types/Modules lane owns only the named owner-test mutations and inventory
  drafts;
- the combined `SS-03H-S + SS-04W` lane is the sole Sema writer and owns the
  capture-vector validator, mutation table, and exactly-once Types-kernel
  invocation oracle;
- Binder production remains read-only unless a new owner defect is reproduced;
- Receipts 03, shared Sema manifests, packet indexes, and wave qualification
  are integrator-only.

Do not dispatch an independent Ticket 03 Sema agent beside Ticket 04. Both
require `sema/src/document.rs` and `sema/tests/v3_document_transaction.rs`; a
split would create two competing patches over the same authority boundary.
