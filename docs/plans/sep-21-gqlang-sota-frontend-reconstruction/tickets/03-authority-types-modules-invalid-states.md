# 03 — Authority, Types, Modules, And Invalid States

Status: `ACTIVE`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Closed

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

Receipt:

1. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/03.md`

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
                           source_registry: &SourceRegistryViewV2)
    -> Result<RelationCatalogV2, TypeAuthorityDiagnosticV2>
ExternalCatalogV2::try_new(
    assets,
    source_registry: &SourceRegistryViewV2,
    session: &mut ModuleBudgetSessionV2,
)
    -> Result<ExternalCatalogV2, ModuleDiagnosticV2>
SignatureV2::try_new(parameters: Vec<LocatedParameterInputV2>,
                     result: ValueTypeV1, callable_kind: CallableKindV2,
                     effects: ExpressionEffectsV2)
    -> Result<SignatureV2, ModuleDiagnosticV2>
SignatureV2::try_as_resolved_predicate(ResolvedPredicateIdV2)
    -> Result<ResolvedPredicateCallableV2, ModuleDiagnosticV2>
PackManifestV2::try_new(input: PackManifestInputV2,
                        source_registry: &SourceRegistryViewV2,
                        session: &mut ModuleBudgetSessionV2)
    -> Result<PackManifestV2, ModuleDiagnosticV2>
ExpressionTypeKernelV2::check_document(inputs: &ResolvedTypeInputsV2)
    -> Result<TypedExpressionFactsV2, TypeDiagnosticV2>
ModuleBudgetV2::begin_session_v2(&self) -> ModuleBudgetSessionV2
ModuleAuthorityBuilderV2::try_new(ModuleBudgetV2, SourceRegistryViewV2)
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
DocumentCatalogValidatorV2::try_new(ModuleBudgetV2,
                                     ExtendedSourceRegistryViewV2,
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
ModuleGraphBuilderV2::resolve(current_module: ModuleIdV2,
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
ModuleImportInputV2 { owner_module_id, local_name, target_asset_id,
                      location }
ModuleReexportInputV2 { owner_module_id, exported_name, target_asset_id,
                        location }
ExternalCatalogAssetInputV2 { module_name, local_name, kind_token,
                              typed_payload, provenance, location,
                              dependencies }
ExternalCatalogAssetV2 { id, kind, typed_payload, provenance, location,
                         dependencies } // validated output only
ExternalCatalogDependencyInputV2 { target_asset_id, expected_kind, location }
LockPinV2 { pack_id, version, content_digest, location }
RelationSourceInputV2 { provenance_namespace, qualified_name,
                        lookup_name, row, cardinality, provenance, location }
LocatedParameterInputV2 { name, value_type, location }
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
ProfileAdmissionInputV2 { profile_asset_id, admission, location }
DocumentCatalogDependencyInputV2 { source_asset_id, target_asset_id,
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
Both `RelationCatalogV2` and `ExternalCatalogV2` require this proof; ticket 04
constructs one combined external-plus-document registry and rejects collisions.
Each retains the opaque deterministic `SourceRegistryIdentityV2` it was checked
against. `ModuleAuthorityBuilderV2` seals the external identity with its host
registry; `ClosedFrontendAuthorityV2::try_new` compares the separately built
relation catalog identity with the sealed registry. A same-ID/different-length
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
`target_asset_id`; it is never separately caller supplied. The same spelling in
two different namespace kinds is valid. External catalog lookup is by the full
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

`ModuleBudgetV2` has private positive node/edge/step,
`max_authority_bytes`, and `max_authority_string_bytes` limits and a fallible
constructor. Builders charge UTF-8/raw lock input bytes before validation,
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
not accept a caller-created session.

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

Registry, resolution, cycle, version, lock, signature, and catalog positives
retain their existing owner. Update these files instead of cloning the suites:

1. `[UPDATE]` `crates/quanta-gqlang-v2-types/tests/v3_types_check_edges.rs` — numeric boundaries and typed facts
2. `[UPDATE]` `crates/quanta-gqlang-v2-types/tests/v3_types_hellgate.rs` — checked constructor/duplicate negatives
3. `[UPDATE]` `crates/quanta-gqlang-v2-types/tests/v3_types_universe.rs` — relation-only authority and metadata retention
4. `[UPDATE]` `crates/quanta-gqlang-v2-modules/tests/v3_modules_mutation_rails.rs` — duplicate mutation matrix
5. `[UPDATE]` `crates/quanta-gqlang-v2-modules/tests/v3_modules_graph.rs` — permutation and typed visibility
6. `[UPDATE]` `crates/quanta-gqlang-v2-modules/tests/v3_catalog_hellgate.rs` — kind/external-catalog/span matrix
7. `[UPDATE]` `crates/quanta-gqlang-v2-modules/tests/v3_modules_packs.rs` and `crates/quanta-gqlang-v2-modules/tests/v3_modules_version_lock_edges.rs` — proof wrappers and duplicate pins
8. `[UPDATE]` `crates/quanta-gqlang-v2-modules/tests/v3_modules_hellgate.rs` — exact primary/secondary/edge spans
9. `[ADD]` `crates/quanta-gqlang-v2-modules/tests/v3_module_budget.rs` — no existing resource-owner file

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
cargo test -p quanta-gqlang-v2-types
cargo test -p quanta-gqlang-v2-modules
cargo clippy -p quanta-gqlang-v2-types -p quanta-gqlang-v2-modules --all-targets -- -D warnings
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

## Dependencies

Requires 01 and ticket 02's HIR/location carrier. The types sub-lane may prepare
RED tests while 02 runs; modules implementation starts after 02. Tickets 02-08
remain one unreleasable atomic cutover stack, and crate manifests remain
integrator-owned.

## 2026-09-21 Final-Audit Work Log

Audit base was re-frozen from `e896a11024bd5a1e1ef03c094a70032f4e6c765f`
to externally integrated `96eab779bfce0a5178dbbd48e86bd457158db228`.
This section records contract repairs, not completion evidence:

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

Ticket status stays `ACTIVE` until implementation, owner-local tests, RR repair,
and receipt 03 all satisfy these amended contracts.
