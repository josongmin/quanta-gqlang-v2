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
| `DELETE` | public invalid type constructors, silent overwrite mutation, raw pack/lock authority admission | delete once all current callers use checked values |
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

## Implementation

### Types

1. Migrate the canonical internal `TypeV1` universe in place: its record,
   relation, and union variants contain private-field checked `RecordTypeV1`,
   `RelationTypeV1`, and `UnionTypeV1` values. Raw/internal types are not
   exported by the curated facade, so a parallel `TypeV2` universe or
   conversion layer is forbidden.
2. Reject duplicate record fields and union tags in fallible constructors.
3. Make relation rows accept only `RecordTypeV1`.
4. Split scalar/value bindings from `RelationCatalogV2`.
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

## Required APIs

These API names and result shapes are pinned. A rename requires a packet
amendment before implementation; raw-map escape hatches are forbidden:

```rust
RecordFieldTypeV1::try_new(name, type, location) -> Result<RecordFieldTypeV1, TypeDiagnosticV2>
UnionVariantTypeV1::try_new(tag, type, location) -> Result<UnionVariantTypeV1, TypeDiagnosticV2>
RecordTypeV1::try_new(fields) -> Result<RecordTypeV1, TypeDiagnosticV2>
RelationTypeV1::new(row: RecordTypeV1, cardinality) -> RelationTypeV1
RelationCatalogV2::try_new(sources) -> Result<RelationCatalogV2, TypeAuthorityDiagnosticV2>
ExternalCatalogV2::try_new(
    assets,
    external_sources: &ExternalSourceRegistryViewV2,
    budget: &ModuleBudgetV2,
)
    -> Result<ExternalCatalogV2, ModuleDiagnosticV2>
ExpressionTypeKernelV2::check_document(inputs: &ResolvedTypeInputsV2)
    -> Result<TypedExpressionFactsV2, TypeDiagnosticV2>
ModuleGraphBuilderV2::new(budget: ModuleBudgetV2) -> ModuleGraphBuilderV2
ModuleGraphBuilderV2::try_declare(&mut self, ModuleDeclarationInputV2)
ModuleGraphBuilderV2::try_import(&mut self, ModuleImportInputV2)
ModuleGraphBuilderV2::try_reexport(&mut self, ModuleReexportInputV2)
ResolvedGraphV2::visible_bindings() -> &ResolvedModuleBindingsV2
PackGraphV2::try_new(manifests, budget: &ModuleBudgetV2)
    -> Result<PackGraphV2, ModuleDiagnosticV2>
PackGraphV2::validate(language_version: VersionV2, budget: &ModuleBudgetV2)
    -> Result<ValidatedPackGraphV2, ModuleDiagnosticV2>
LockfileV2::verify_against(&ValidatedPackGraphV2, budget: &ModuleBudgetV2)
    -> Result<VerifiedLockSetV2, ModuleDiagnosticV2>
ValidatedPackAuthorityV2::try_new(ValidatedPackGraphV2, VerifiedLockSetV2)
    -> Result<ValidatedPackAuthorityV2, ModuleDiagnosticV2>
ModuleGraphBuilderV2::resolve(current_module: ModuleIdV2)
    -> Result<ResolvedGraphV2, ModuleDiagnosticV2>
LockfileV2::parse(source_id: SourceIdV2, text: &str, budget: &ModuleBudgetV2)
    -> Result<LockfileV2, ModuleDiagnosticV2>
```

The builder ellipses above are typed DTOs, not raw maps or string tuples:

```text
ModuleDeclarationInputV2 { owner_module_id, asset_id, local_name,
                           namespace_kind, visibility, location }
ModuleImportInputV2 { owner_module_id, local_name, target_asset_id,
                      visibility, location }
ModuleReexportInputV2 { owner_module_id, exported_name, target_asset_id,
                        origin_asset_id, location }
ExternalCatalogAssetInputV2 { id, kind_token, typed_signature, provenance,
                              location, dependency_ids }
ExternalCatalogAssetV2 { id, kind, typed_signature, provenance, location,
                         dependency_ids } // validated output only
LockPinV2 { pack_id, version, content_digest, location }
```

Duplicate diagnostics use distinct stable codes for declaration, import,
re-export, lock pin, suite declaration, signature parameter, catalog ID, and
catalog `(kind,name)` collisions. Each carries primary conflict location,
secondary original location, and the relevant import/re-export edge location.
Unknown current module uses `modules.unknown_module`; budget exhaustion uses
`modules.resource_exhausted`.

There must be no `Default` implementation for closed semantic authority.
Ticket 04 owns `ClosedFrontendAuthorityV2::try_new`; it accepts one
`ResolvedGraphV2` and one `ValidatedPackAuthorityV2`, not separate current-module,
pack-graph, and lock-proof values.

`ResolvedTypeInputsV2` is fallibly constructed from checked HIR document roots
plus an entry list keyed by
`SourceQualifiedNodeIdV2 { source_id: SourceIdV2, node_id: HirNodeIdV1 }`.
It builds its private maps itself so duplicate input entries remain observable;
a caller-supplied pre-collapsed map is forbidden. Inputs contain the already resolved
binding type or callable signature for every symbol/call node; construction
rejects missing, duplicate, foreign-document, and extra facts before the
kernel walks those roots. `TypedExpressionFactsV2` contains the selected type,
`OperatorIdV2`/`BuiltinIdV2` when applicable, and the existing purity,
determinism, monotonicity, aggregation, and cost metadata. Missing or duplicate
node facts reject; there is no name-based fallback. Open analysis represents an
unresolved input with an explicit `UnknownTypeCauseV2`, not by omitting its node
entry. Unknown is represented by
`ExpressionTypeStateV2::{Known(TypeV1), Unknown(UnknownTypeCauseV2)}` and is
never a closed type variant. The kernel is mode-neutral; sema rejects every
Unknown state before compiled success.

The sole `i128::MIN` lexical exception attaches
`TypedLiteralValueV2::Int(i128::MIN)` to the unary-minus node. Its positive
`i128::MAX + 1` child retains
`TypedLiteralFactV2::DeferredI128MinMagnitude`, a closed marker legal only as
that unary node's direct operand; construction rejects the marker everywhere
else. Thus every expression node has a fact and downstream consumers never
reparse literal text.

Registry creation is fallible and production-owned:
`OperatorRegistryV2::try_closed(entries)` and
`BuiltinRegistryV2::try_closed(entries)` reject duplicate explicit IDs and
duplicate semantic signatures. No `unwrap`, vector index, or debug-only check
establishes registry identity. `CallArgHirV1::Star` matches only an explicit
aggregate-Star signature (initially `count(*)`); scalar, predicate, mixed, and
unsupported aggregate calls reject without assigning a fabricated Star type.

`ResolvedModuleBindingsV2` is a modules-owned typed visibility projection, not
a binder type and not `BTreeMap<String, Vec<String>>`. Ticket 04 converts it to
Binder inputs in sema. The precedence/ambiguity algorithm currently exercised
through `binder_view` is retained exactly once behind this API.

Each binding entry retains local name, resolved module/asset ID, namespace kind,
visibility, declaration origin, and import/re-export edge `LocatedSpanV2`.
`resolve(current_module, budget)` rejects an absent current module; neither
`visible_bindings()` nor its proof can silently return an empty fallback.

Direct-import behavior is exact: a second `try_import` for the same
`(owner_module_id, local_name)` always rejects as `duplicate_import`, whether
its target is identical or different. Deferred candidate ambiguity applies to
multiple re-export origins and other distinct candidate-producing edges, not to
a duplicate direct-import alias that the builder has already rejected.

`ModuleBudgetV2` has private positive node/edge/step limits and a fallible
constructor. Builders charge before inserting nodes/edges; resolution charges
each visited export/import/re-export edge, including iterative replacements for
recursive export-origin and catalog-cycle walks. Exact limit succeeds and the
next charge emits `modules.resource_exhausted { dimension, limit, observed }`.
The already-iterative module cycle DFS is retained, not rewritten.

One non-resetting internal counter exists per public bounded operation.
`ModuleGraphBuilderV2` owns its counter from construction through `resolve`.
Pack construction/validation, lock parsing/verification, and external-catalog
construction each create exactly one counter from the caller's immutable
budget and thread it through every nested walk. No helper creates or resets a
standard/default counter.

Stable IDs are collision-free checked canonical values, not truncated hashes:
`ModuleIdV2` owns a normalized qualified module name; `CatalogAssetIdV2` owns
the structured `(ModuleIdV2, NamespaceKindV2, local_name)` tuple; and
`RelationSourceIdV2` owns `(ProvenanceNamespaceV2, qualified_name)`. Canonical
name validation rejects empty segments, empty local names, separator aliases,
and non-normalized spellings before an ID exists.

The pack graph fingerprint is SHA-256 over domain
`qglang.pack-graph.v2\0` plus length-prefixed canonical fields, with sorted
packs/dependencies/exports/suites/members. It covers pack name, version,
content digest, language range, every dependency ID/range, export, suite name,
and member. Permutations yield the same fingerprint; changing any covered
semantic field changes it. The implementation uses pinned `sha2 =0.10.9`, not
a repository-local cryptographic implementation.

`SourceIdV2` remains transaction-local. `ExternalSourceRegistryViewV2` lists
every source ID, byte length, and immutable provenance admitted to one external
catalog and rejects duplicate IDs before catalog construction. Every external
`LocatedSpanV2` is validated against that view. Ticket 04's frontend transaction
then combines the external view with its document source and rejects any source
ID collision before building closed authority; raw `SourceIdV2::from_raw`
construction alone is never authority evidence.

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
    duplicate `(kind, name)` keys, unknown kinds, and unresolved external view
    dependencies before closed authority construction.
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

Numeric diagnostics are pinned to `types.integer_out_of_range`,
`types.float_non_finite`, and `types.float_underflow`. Typed literals retain the
parsed `i128` or finite `f64`; downstream stages never parse literal text again.
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
