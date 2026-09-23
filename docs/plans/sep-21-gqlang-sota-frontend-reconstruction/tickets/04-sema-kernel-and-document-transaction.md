# 04 — Semantic Kernel And Document Transaction

Status: `CODE INTEGRATED AT d9d0f28; CURRENT-SHA QUALIFICATION OPEN`

This header supersedes older RED/implementing checkpoints. Checked document
resolution, source rule membership, composition, and aggregate legality are
reachable. The committed F04 nested-aggregate fix and F08 authored-order
certification oracle passed the pre-commit 1,168/1,168 workspace gate; a
clean `d9d0f28` qualification receipt is still pending.

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Owned / To Close

F02, F05, F06, F08, F09, F10's document assembly, F15, F16, F19, F24's
pipeline half, F28, F30's resolution half, F31's metadata retention, and F33.

## Goal

Introduce the single canonical producer of bound and typed document semantics,
then replace item-at-a-time public compilation with one atomic document
transaction.

## Current-Head Reuse/Delete/Build Audit

| Class | Assets | Concrete action |
|---|---|---|
| `KEEP` | Binder frame/capture/shadow algorithms; type checker and closed registries; module/catalog resolution algorithms; pipeline diagnostic projection | call these owners from sema and preserve their focused tests |
| `MIGRATE` | Binder outputs to stable IDs/incremental row frames; pipeline to document orchestration | change carriers and call graph without reimplementing lexical/type/module kernels |
| `DELETE` | pipeline `type_check_bool`, `env_closed`, per-item resolution/catalog mutation, raw-HIR output reconstruction, `lower_source_rule`, dependency-name heuristics | remove when the semantic document feeds all downstream stages |
| `NEW` | authority types, typestate document transaction, typed semantic nodes/effects, aggregate context, atomic failure/result | sema owns these missing contracts only |

`quanta-gqlang-v2-sema` must not contain a second Binder, overload resolver,
module resolver, catalog resolver, Tarjan pass, or algebra normalizer. Its
expression module coordinates and records results from the canonical owner
registries; its document module owns ordering and atomicity.

The exact wiring is:

```text
ResolvedGraphV2::visible_bindings/current module + ExternalCatalogV2
  -> sema-owned binder authority adapter
  -> Binder resolved binding IDs + capture edges by HirNodeId
  -> ResolvedTypeInputsV2 with binding types/call signatures by HirNodeId
  -> types::ExpressionTypeKernelV2
  -> TypedExpressionFactsV2
  -> sema TypedExprV2/TypedRelationV2/ResolvedRuleV2 assembly
```

Sema may diagnose a missing/duplicate fact or illegal cross-node context, but it
may not perform a second name lookup or overload search after these owners run.

## Write Set

1. `crates/quanta-gqlang-v2-sema/src/authority.rs`
2. `crates/quanta-gqlang-v2-sema/src/document.rs`
3. `crates/quanta-gqlang-v2-sema/src/expr.rs`
4. `crates/quanta-gqlang-v2-sema/src/relation.rs`
5. `crates/quanta-gqlang-v2-sema/src/rule.rs`
6. `crates/quanta-gqlang-v2-sema/src/diagnostic.rs`
7. `crates/quanta-gqlang-v2-sema/src/names.rs`
8. `crates/quanta-gqlang-v2-sema/src/item.rs`
9. `crates/quanta-gqlang-v2-sema/src/lib.rs`
10. `crates/quanta-gqlang-v2-sema/Cargo.toml` — integrator only
11. `crates/quanta-gqlang-v2-binder/src/resolve.rs`
12. `crates/quanta-gqlang-v2-binder/src/scope.rs`
13. `crates/quanta-gqlang-v2-binder/src/diagnostic.rs`
14. `crates/quanta-gqlang-v2-pipeline/src/compile.rs` — integrator only
15. `crates/quanta-gqlang-v2-pipeline/src/diagnostic.rs` — integrator only
16. `crates/quanta-gqlang-v2-pipeline/Cargo.toml` — integrator only
17. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/04.md`
18. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/LEGACY_API_INVENTORY.txt`
19. all four Binder suites named below and owner-local Binder support module
20. `crates/quanta-gqlang-v2-pipeline/tests/v3_alias_type_closure_gap.rs`
21. `crates/quanta-gqlang-v2-pipeline/tests/v3_frontend_context.rs`
22. `crates/quanta-gqlang-v2-pipeline/tests/v3_diagnostic_surface.rs`
23. owner-local pipeline test support module when consumed by at least two
    touched tests

## Core Types

1. `ClosedFrontendAuthorityV2`
2. `OpenAnalysisAuthorityV2`
3. `CheckedSyntaxDocumentV2`
4. `DeclaredDocumentV2`
5. `ResolvedDocumentV2`
6. `TypedExprV2`
7. `ResolvedCallTargetV2` from types; no sema alias
8. `ExpressionEffectsV2`
9. `TypedRelationV2`
10. `RelationOutputContractV2`
11. `ResolvedRuleV2`
12. `SemanticDocumentV2`
13. `FrontendBudgetV2`
14. `AggregateContextV2`

## Required Module Layout

```text
sema/src/
  authority.rs   closed/open authority construction and proof access
  document.rs    typestate transaction and global validation order
  names.rs       typed namespaces and resolved IDs
  expr.rs        bound/typed expressions and effect summaries
  relation.rs    incremental row frames and relation output contracts
  rule.rs        rule variables, predicate calls, program assembly inputs
  item.rs        CHECK/FIND/presentation/catalog semantic contracts
  diagnostic.rs  stable semantic codes and source locations
  lib.rs         curated crate-local exports
```

Do not put all semantic behavior back into
`crates/quanta-gqlang-v2-pipeline/src/compile.rs`.

## Implementation

### Expression semantics

1. Resolve every call to builtin/overload ID or predicate ID.
2. Retain type, purity, evaluation determinism, monotonicity, aggregate class,
   and cost on typed expression nodes.
3. Implement `type_bool`; enforce Bool at every predicate root.
4. Route compare/and/or/not through the closed operator registry.
5. Remove whole-expression `env_closed` skip behavior.
6. Open analysis may use Unknown, but known contradictions still reject.
7. A closed semantic transaction rejects any remaining Unknown.
8. Compute `AggregateContextV2` while typing every expression position and
   reject illegal/nested/star combinations before algebra. Ticket 05 consumes
   this result read-only.
9. Resolve relational EXISTS/NOT EXISTS through a capturing child frame whose
   capture set is explicit; reject rule quantifiers in query predicates and
   relational subqueries in rule bodies.

### Incremental relation binding

1. Bind each input before its stage.
2. Expose JOIN RHS alias only to ON and later stages.
3. Replace the row frame after projection.
4. Reject duplicate projection aliases before creating output schema.
5. Carry relation output contract forward without HIR reconstruction.

### Rule semantics

1. Predeclare every source rule as a stable predicate ID before body resolution.
2. Treat current grammar as unary; declare the head variable and quantifier
   variables in explicit scopes.
3. Resolve every body call as builtin or predicate, enforce arity, and retain
   positive/negated/aggregated polarity plus exact call span.
4. Reject duplicate quantifier binders, free rule variables, bare predicate
   symbols, and relation subqueries in rule bodies.
5. Emit `ResolvedRuleV2` for every source rule. Ticket 06 alone assembles the
   complete rule program and owns SCC/stratification.

### Document transaction

```text
parse
-> integrity
-> predeclare assets/predicates/views/meta
-> resolve typed refs/imports
-> bind/type bodies
-> emit relation/rule/graph/item contracts
-> global validation
-> atomic semantic document
```

1. Resolve forward references after predeclaration.
2. Compile inline VIEW bodies through the same typed relation path.
3. Collect all META before final certified-admission validation.
4. Resolve catalog reference existence and kind for every family.
5. Emit resolved view dependency edges for cycle validation.
6. Return no partial semantic document on transaction failure.

## Staged Semantic API

Add only the internal semantic seam in this ticket:

```text
CheckedSyntaxDocumentV2::try_from_hir(document: HirDocumentV1)
  -> Result<CheckedSyntaxDocumentV2, FrontendFailureV2>
resolve_checked_document_v2(checked: CheckedSyntaxDocumentV2,
  authority: &ClosedFrontendAuthorityV2)
  -> Result<SemanticDocumentV2, FrontendTransactionErrorV2>
```

These are sema-owned APIs and are never exported by the frontend facade. The
constructor consumes the HIR-owned document containing its sole source-length
authority, invokes the one integrity checker over every root with one
document-wide ID set/source bound, and yields an owned proof carrier; callers
cannot construct it by fields or supply a second source length.

Source entry points are pipeline-owned because pipeline may depend on both
surface and sema:

```text
resolve_source_to_semantics_v2(source_id: SourceIdV2, source: &str,
  authority: &ClosedFrontendAuthorityV2)
  -> Result<SemanticDocumentV2, FrontendTransactionErrorV2>
analyze_source_to_semantics_v2(source_id: SourceIdV2, source: &str,
  authority: &OpenAnalysisAuthorityV2) -> SemanticAnalysisV2
```

Pipeline parses to `ParsedDocumentV2`, extracts its owned `HirDocumentV1`, and
immediately delegates to the checked-document transaction. Sema never depends
on surface. Architecture fixtures and compile checks pin this direction.

Every source entry point requires an explicit caller-owned `SourceIdV2`; no
layer invents, hashes, increments, or silently renumbers one. The same ID is
passed into ticket 02's parser and the resulting HIR document. A collision with
host authority rejects deterministically before semantic work. Repeated or
concurrent calls with the same `(authority, source_id, bytes)` produce identical
identity and diagnostics because the authority is immutable.

`SemanticAnalysisV2` is a closed state enum:

```text
Complete(SemanticDocumentV2)
Incomplete(IncompleteSemanticDocumentV2)
Error(FrontendTransactionErrorV2)
```

The incomplete carrier contains explicit unresolved facts and ordered
diagnostics but cannot expose or convert to `SemanticDocumentV2`. Only
`Complete` may enter algebra, logic, or planning. A fully resolved open analysis
still remains analysis and is never implicitly promoted to compiled output.

Ticket 04 must not define `compile_source_document_v2`, `CompiledDocumentV2`,
or final `AnalysisDocumentV2`: algebra, logic, and planning do not exist on the
V2 path until tickets 05-07. Ticket 07 owns final assembly and activates those
public signatures. A stub, empty plan, V1 fallback, or "semantic-only compiled"
result is forbidden.

Authority constructors are pinned:

```text
ClosedFrontendAuthorityV2::try_new(validated_module_authority_parts,
  relation_catalog, graph_reference_catalog, budget)
OpenAnalysisAuthorityInputV2::try_new(optional_modules, optional_relations,
  optional_graph_references, host_sources, budget)
OpenAnalysisAuthorityV2::try_new(input)
OpenAnalysisAuthorityV2::try_standard_v2()
```

`try_standard_v2()` contains the builtin/operator registry and
`FrontendBudgetV2::standard_v2()` but no invented external relation, module,
catalog, pack, or lock authority.

`ValidatedModuleAuthorityPartsV2` owns one resolved graph (and therefore its
current-module identity), one external catalog, one validated pack authority,
their exact host `SourceRegistryViewV2`, and the exact
`AuthorityBudgetProfileV2` used for admission; its private fields prevent
mixing proofs from different registry or budget sessions. `RelationCatalogV2`
retains both its registry identity and its authority budget profile. The
constructor compares the relation catalog's source-registry identity with that
host registry and rejects a mismatch. It also projects
`AuthorityBudgetProfileV2` canonically from `FrontendBudgetV2` and rejects
unless both retained profiles equal that projection. A larger-budget cached
proof cannot be paired with a smaller frontend budget. For each checked
document, sema builds a new registry from the retained host registry with
`SourceRegistryViewV2::try_extend_document` and
`HirDocumentV1::{source_id,source_len}`, rejecting duplicate source IDs before
any binding/type/catalog work. It does not silently renumber either host or
document sources.

The resulting `ExtendedSourceRegistryViewV2` proves append-only ancestry. Host
authority proofs must match its host-parent identity, while all newly minted
document DTO locations must match its combined identity. Sema passes the sealed
`ExtendedSourceRegistryViewV2` and `ValidatedModuleAuthorityPartsV2` into
`DocumentCatalogValidatorV2::try_new`; it does not compare opaque IDs itself or
revalidate cached host catalogs.

The atomic 08 cut removes these nine exact V1 seams after all callers migrate:

1. `compile_program_structural`
2. `compile_program_with_schema`
3. `compile_with_context`
4. `compile_item_structural`
5. `compile_item_with_schema`
6. `compile_item_with_context`
7. `compile_document_structural`
8. `compile_document_with_schema`
9. `compile_document_with_context`

The same inventory records final disposition for `CompiledItemV1`,
`CompiledProgramV1`, `FrontendContextV1`, and `NoServeReasonV1`; re-exports,
callers, tests, and docs count as inventory rows, not just definitions.

Tickets 02-08 run as one stacked cutover. Ticket 04 does not add an adapter and
does not route V1 through manufactured empty authority; it freezes the existing
V1 public compile-seam definitions and inventory while the new transaction is
built in the same unreleasable stack. “Alongside” applies only to the temporary
presence of those named public entrypoint definitions; it does not permit a
parallel `TypeV2` universe, V1/V2 type conversions, compatibility adapters, or
a compiling legacy path after ticket 03 migrates canonical `TypeV1` in place.
Those frozen seams may remain intentionally uncompilable until their callers
are migrated and ticket 08 deletes them. No 04-07 revision is a releasable or
workspace-green public contract. Ticket 08 deletes the frozen inventory before
the stack becomes `INTEGRATED`.

## Step-By-Step Procedure

1. Implement closed/open authority types and prevent default closed authority.
2. Implement the owned checked-syntax constructor over ticket 02's HIR
   integrity proof and make both source paths delegate to checked-document
   paths.
3. Implement namespace predeclaration for document catalog assets, source rules,
   composition locals, module imports, relation sources, and builtins.
   Mint every document modules/types authority DTO location through
   `ExtendedSourceRegistryViewV2::try_check_location`; never forward a bare
   `LocatedSpanV2` as authority evidence. Host-only cached inputs retain checks
   against the extension's opaque parent registry identity.
4. Implement typed reference resolution with expected/actual kind diagnostics.
5. Convert Binder output to stable resolved IDs and explicit capture edges.
6. Build node-ID-keyed type inputs from Binder/module resolution, including one
   opaque relation-source/capture/effect input for each `RelationExists`; invoke the
   types-owned kernel once, and assemble typed expressions/resolved calls/effect
   summaries from its facts.
7. Implement incremental relation frames and output contracts.
8. Implement item semantic contracts for FIND/CHECK/presentation/catalog/meta.
9. Implement the document typestate phases in the fixed order.
10. Compile inline VIEW bodies during the same body-compilation phase as queries.
11. Assemble all source rules before global logic validation.
12. Build ticket 03's neutral `DocumentCatalogInputV2` from resolved typed
    assets, dependency edges, profile admission, activations, and locations;
    construct `DocumentCatalogValidatorV2` exactly once from the sealed module
    authority and `ExtendedSourceRegistryViewV2`, then call its public
    `validate` method exactly once after all bodies and metadata. Sema never
    calls crate-private `DocumentCatalogAuthorityV2::try_new`, does not
    reimplement view-cycle, certification, unknown-pack, or activation
    validation, and never passes raw HIR bodies to modules.
13. Add pipeline orchestration over `SemanticDocumentV2`.
14. Add `FrontendBudgetV2` as the one public limit configuration carried by
    closed/open authority. Project it into ticket 02's `ParseBudgetV2`, ticket
    03's `ModuleBudgetV2`, ticket 06's logic budget, and one private semantic
    step counter. Surface alone charges document items and expression work;
    sema charges only semantic transaction steps.
15. Migrate the tests owned by this ticket to source-document V2 APIs.
16. Freeze all remaining V1 compile-seam references with this command and
    commit its sorted output as `LEGACY_API_INVENTORY.txt`:

    ```bash
    {
      rg -n 'compile_(program_structural|program_with_schema|with_context|item_structural|item_with_schema|item_with_context|document_structural|document_with_schema|document_with_context)' crates --glob '*.rs'
      rg -n 'compile_(program_structural|program_with_schema|with_context|item_structural|item_with_schema|item_with_context|document_structural|document_with_schema|document_with_context)' README.md docs/language --glob '*.md'
    } | sort
    ```

    This inventory covers active source, tests, and current public docs only;
    `docs/plans/**` is historical/audit evidence and is classified by ticket
    09's truth-claim inventory instead of rewritten by ticket 08.

17. Do not add, wrap, or reinterpret a V1 compile seam. Ticket 08 deletes every
    inventory entry after caller migration.
18. Update the existing binder capture/shadow suites in place and rewrite
    pipeline alias/context/diagnostic tests against the document seam. Delete
    any test whose only asserted API is a removed V1 helper; do not clone it
    under a V2 filename.

## Diagnostic Codes To Add

At minimum:

```text
sema.unresolved_reference
sema.wrong_reference_kind
sema.ambiguous_reference
sema.unknown_type_in_closed_compile
sema.non_boolean_predicate
sema.duplicate_output_name
sema.illegal_capture
sema.invalid_item_shape
sema.document_validation_failed
frontend.resource_limit_exceeded
```

Existing owner diagnostics may be wrapped but their stable code and primary/
secondary spans must remain available.

`FrontendFailureV2` contains a nonempty diagnostic vector and never a partial
compiled artifact. `FrontendDiagnosticV2` contains stage, stable code, severity,
primary span, sorted secondary/edge spans, and bounded message. User-derived
identifier excerpts are UTF-8-boundary truncated to 128 bytes; the rendered
message is capped at 1,024 bytes without changing code or spans. Sort the final
vector by `(source_id, primary_start, primary_end, stage_rank, severity_rank,
code, message)`. Stage rank follows the state machine; severity is error before
warning before note. CLI and diagnostics stdio use this exact order.

## Budget Contract To Add

`FrontendBudgetV2` has private `NonZeroU32` fields, a fallible constructor,
read-only `u32` accessors, and no zero or unbounded default:

```text
max_source_bytes
max_frame_bytes
max_tokens
max_document_items
max_expression_depth
max_expression_nodes
max_collection_width
max_projection_width
max_semantic_steps
max_module_graph_nodes
max_module_graph_edges
max_module_steps
max_authority_bytes
max_authority_string_bytes
max_rule_graph_nodes
max_rule_graph_edges
max_rule_steps
max_witness_depth
max_witnesses
```

`FrontendBudgetV2::authority_profile()` is the only projection into ticket
03 authority admission. It returns the checked `AuthorityBudgetProfileV2`
containing `max_module_graph_nodes`, `max_module_graph_edges`,
`max_module_steps`, `max_authority_bytes`, and
`max_authority_string_bytes`. Closed-authority construction compares this value
with the profiles sealed into both `ValidatedModuleAuthorityPartsV2` and
`RelationCatalogV2`; inequality is a typed construction error. It never
revalidates a large proof under a smaller limit and never accepts an unrelated
`ModuleBudgetV2` argument.

The named CLI/service profile `FrontendBudgetV2::standard_v2()` is pinned to:

| Dimension | Limit |
|---|---:|
| source bytes | 1,048,576 |
| frame bytes | 4,194,304 |
| tokens | 131,072 |
| document items | 4,096 |
| expression depth | 64 |
| expression nodes | 262,144 |
| collection width | 4,096 |
| projection width | 4,096 |
| semantic steps | 1,000,000 |
| module graph nodes | 50,000 |
| module graph edges | 200,000 |
| module steps | 1,000,000 |
| authority bytes | 16,777,216 |
| authority string bytes | 65,536 |
| rule graph nodes | 50,000 |
| rule graph edges | 200,000 |
| rule steps | 1,000,000 |
| witness depth | 1,024 |
| witnesses | 10,000 |

Custom authority may select smaller positive values. Larger values require an
explicit constructor call and remain bounded by `u32`; values are not silently
clamped. Validate `max_frame_bytes >= max_source_bytes`.

Ticket 04 validates positivity and carries this immutable configuration in both
authority types. Pipeline checks `max_source_bytes`, then creates exactly one
owner-local counter for surface, sema, modules, and logic when applicable.
Ticket 08 adds bounded CLI/service byte prechecks and proves the same
configuration reaches pipeline. No remaining-count carrier crosses stages;
every cumulative dimension is incremented by exactly one canonical owner.

## Test Files

Retain and update all four Binder suites (`v3_binder_scope.rs`,
`v3_binder_hellgate.rs`, `v3_binder_mutants.rs`, and
`v3_composition_hellgate.rs`). Rewrite the existing pipeline alias/type/context,
diagnostic, relation-authority, group/order, pack-authority, and uncovered-branch
tests onto the document API before adding the cross-stage files below. The
receipt lists every removed V1-helper test function by name.

1. `[ADD]` `crates/quanta-gqlang-v2-sema/tests/v3_semantic_kernel.rs` — expression typing, aggregate context, and rule resolution
2. `[ADD]` `crates/quanta-gqlang-v2-sema/tests/v3_incremental_relation_frames.rs`
3. `[ADD]` `crates/quanta-gqlang-v2-sema/tests/v3_reference_resolution.rs`
4. `[ADD]` `crates/quanta-gqlang-v2-sema/tests/v3_document_transaction.rs` — order, inline-view parity, open/closed distinction, and atomicity
5. `[ADD]` `crates/quanta-gqlang-v2-sema/tests/v3_semantic_budget.rs`
6. `[ADD]` `crates/quanta-gqlang-v2-pipeline/tests/v3_document_semantics_v2.rs` — source/checked path and stage-envelope parity
7. `[ADD-THEN-DELETE-08]` `crates/quanta-gqlang-v2-pipeline/tests/v3_legacy_api_ratchet.rs`

Audited existing-test disposition:

1. Delete the redundant declaration-body type error rail after retaining the
   stronger diagnostic code/display and exact-span tests.
2. In `v3_alias_type_closure_gap`, delete the two proven restatements and retain
   the one distinct context-closure case.
3. Replace `assert_eq!(diag.span(), diag.span())` with the exact expected span.
4. Delete the three proven redundant Binder mutant tests only after their
   hellgate/scope owners are green; retain the other ten distinct Binder mutant
   rails.
5. Delete `LexicalScopeV1::depth` and its sole-coverage test if no final sema
   production consumer exists. This packet does not retain it as public API.
6. Normalize touched test lint headers and extract only repeated owner-local
   helpers; do not create a cross-crate test-support dependency.

Add one Binder fixture that emits at least two diagnostics with distinct
locations/codes. Pin the sema sort key and pipeline's first-diagnostic
projection against that ordered vector. This is the owner proof for diagnostic
ordering; single-element transport determinism tests may not claim it.

## Exact RED/Expected Matrix

| Input | Expected repaired result |
|---|---|
| `where 1` | `sema.non_boolean_predicate` |
| `where 1 = "x"` | closed operator mismatch |
| `where "x" < true` | closed operator mismatch |
| `where 1 and 2` | Bool operand mismatch |
| earlier stage references later JOIN alias | unresolved at exact reference |
| projection then `order by` its alias | success |
| correlated `exists (from ...)` using an earlier alias | success with explicit capture set |
| correlated subquery using a later JOIN alias | unresolved at exact reference |
| rule quantifier in query / relation subquery in rule | `sema.invalid_item_shape` |
| duplicate rule quantifier / free rule variable / bare predicate | stable sema reject at exact symbol span |
| duplicate projection alias | `sema.duplicate_output_name` |
| inline VIEW containing `where 1` | same semantic code as direct query |
| unknown catalog ref | `sema.unresolved_reference` |
| rule ref used where profile expected | `sema.wrong_reference_kind` |
| `profile lax; require certified` and reverse | same certification reject |
| closed semantic transaction with Unknown | hard reject |
| open semantic analysis with Unknown | analysis result, never compiled artifact |

## Exact Verification

```bash
cargo test -p quanta-gqlang-v2-sema
cargo test -p quanta-gqlang-v2-binder
cargo test -p quanta-gqlang-v2-sema --test v3_document_transaction
cargo test -p quanta-gqlang-v2-sema --test v3_semantic_budget
cargo clippy -p quanta-gqlang-v2-sema -p quanta-gqlang-v2-binder --all-targets -- -D warnings
python3 tools/architecture/check_architecture.py
git diff --check
```

The pipeline semantic-document and legacy-ratchet commands are deferred to
ticket 07 and listed, with their current failures, in receipt 04.

## Forbidden Shortcuts

1. leaving `env_closed` or an equivalent all-or-nothing skip
2. resolving refs in each item compiler independently
3. compiling VIEW bodies through a special reduced path
4. resolving catalog after every declaration
5. returning partially compiled items after document failure
6. adding a V1-to-V2 adapter that manufactures missing authority
7. changing a frozen V1 compile-seam implementation or adding a new V1
   compile-seam call site during the stacked cutover
8. returning `CompiledDocumentV2` before tickets 05-07 have produced and planned
   every applicable semantic contract

## RED

1. invalid predicates compile
2. later aliases are visible too early while projection aliases are invisible
3. inline VIEW skips the canonical binder/type semantic transaction
4. unknown/wrong-kind refs compile
5. REQUIRE CERTIFIED result depends on declaration order
6. each catalog declaration triggers full resolve
7. sibling rule calls are unresolved or omit their resolved dependency edge

## Tests

1. invalid Bool/comparison/logical operator matrix
2. partial-Unknown tests that still catch literal contradictions
3. JOIN/projection frame visibility tests
4. duplicate projection alias rejection
5. inline VIEW/direct query diagnostic parity
6. two-diagnostic Binder order plus pipeline first-diagnostic selection
7. `SemanticAnalysisV2` complete/incomplete/error state separation and absence
   of any conversion from incomplete/open analysis to compiled semantics
8. forward, unknown, wrong-kind, and ambiguous reference tests
9. REQUIRE/declaration order permutation test
10. document atomicity test
11. source and prechecked-syntax entrypoint parity
12. exact-limit and one-over-limit semantic document/expression budget tests
13. aggregate placement/nesting/star context matrix and rule
    binder/call/free-variable matrix

## Definition Of Done

1. the V2 pipeline owns orchestration only; local type/reference walkers are gone
2. every V2 lowerable item originates from one `SemanticDocumentV2`; this ticket
   does not claim compiled output
3. open analysis cannot masquerade as compiled success
4. document ordering affects sequential composition only, not global declaration
   legality
5. the V1 inventory is frozen and no new V1 use exists; final removal remains a
   mandatory ticket 08 gate, so this ticket reaches `FOCUSED_GREEN`, not
   `INTEGRATED`, by itself

## Dependencies

## 2026-09-21 Final Adversarial Audit Corrections

This section is normative and supersedes any earlier API, ownership, or
verification wording in this ticket that conflicts with it.

### Executable stage boundary

- Ticket 04 may reach only `FOCUSED_GREEN`. Its owner proof is `sema` plus
  `binder`; it must not require `algebra`, `logic`, `planning`, or `pipeline` to
  compile before their consumer migrations in tickets 05-07.
- Record the currently deferred consumer failures in receipt 04 by exact path,
  symbol, and compiler code. The known migration set is
  `binder/src/resolve.rs` and `pipeline/src/compile.rs` for private
  `ProjectionV1`, `PathAccessV1`, `CallExprV1`, composition producer/body, and
  removed `Compose`/`Quantifier` variants.
- Cross-stage pipeline proof first becomes required by ticket 07. Ticket 04
  must not claim `INTEGRATED` from an owner-only pass.

### Exact authority constructors

Add a private-field `OpenAnalysisAuthorityInputV2` with these explicit inputs:

1. optional `ValidatedModuleAuthorityPartsV2`;
2. optional `RelationCatalogV2`;
3. optional `GraphReferenceCatalogV2`;
4. one checked host `SourceRegistryViewV2`;
5. one `FrontendBudgetV2`.

`OpenAnalysisAuthorityV2::try_new(input)` checks source-registry compatibility
and retained authority-profile equality for every supplied proof. Missing
module, relation, or graph-selector authority means unresolved editor facts,
never an empty closed proof. `try_standard_v2() ->
Result<Self, FrontendAuthorityErrorV2>` is the only
standard constructor because builtin/operator registry creation is fallible.
Ticket 08 must propagate this failure at startup/request handling; no unwrap,
default, or error erasure is allowed.

Ticket 04 owns private-field `ClosedFrontendAuthorityV2` and its internal
`try_new` constructor for semantic owner tests. Ticket 07 owns the public
facade builder and host-facing DTO after all lowerers exist; it delegates
validation/sealing through pipeline and sema.

### Canonical semantic DTO chain

This ticket owns the following private-field, fallibly constructed results:

- `SemanticItemIdV2 { source_id, hir_item_id }`, the sole join key for all
  downstream per-item outputs;
- `ResolvedCallTargetV2` from ticket 03, reused directly; do not introduce a
  second `ResolvedCallV2` identity;
- `TypedExprV2 { key: SourceQualifiedNodeIdV2, shape, value_type, effects,
  resolved_call, aggregate_context }`; `SourceQualifiedNodeIdV2` is the only
  expression ID. The closed shape retains literal/operator/path/call-star
  identity and ordered child keys or typed children, so algebra and logic do
  not reconstruct expression structure from raw HIR;
- `TypedBoolExprV2`, constructible only from a `TypedExprV2` whose checked type
  is Bool;
- `TypedRelationV2 { item_id, root: TypedRelationExprV2, output }`; its closed
  source expression enum covers only the six HIR variants: Source, Filter,
  Project, Join (inner/cross), GroupBy, and OrderBy. It retains typed expression
  references, resolved source IDs, checked correlations, and producer-sealed
  output contracts. Apply, Aggregate, Limit, Union, SemiJoin, and AntiJoin
  are the six existing non-stage algebra operators handled by ticket 05, not
  fabricated source-reachable sema variants;
- `ResolvedGraphQueryV2`, `ResolvedCheckV2`, and
  `ResolvedPresentationV2`, each retaining `SemanticItemIdV2`;
- `AggregateContextV2` and `RelationOutputContractV2`, retained rather than
  recomputed by algebra;
- `SemanticDocumentV2`, a private exact-key map plus original source order.

`ExpressionEffectsV2` stays types-owned. `OutputProvenanceV2` is a field of the
sema-owned `RelationOutputContractV2`; `OrderKeyContractV2` is a sema-owned
checked descriptor containing the source-qualified expression key, direction,
null policy, and checked type. There is no `TypedRelationNodeV2` or
`TypedExprIdV2` parallel universe.

FIND produces `ResolvedGraphQueryV2`; CHECK produces `ResolvedCheckV2` with the
exact nested graph item identity; presentation clauses produce
`ResolvedPresentationV2`. Ticket 06 converts the graph value into
`GraphQueryContractV2`. Ticket 07 assembles `CheckContractV2` only when the
nested graph contract has the same document and item identity. Planning may
not revisit raw HIR or redo semantic validation.

### Ownership clarifications

- "global validation" here means document catalog/reference/type/aggregate
  validation only. SCC, stratification, graph boundedness, and witness policy
  belong to ticket 06; sema must not depend on logic.
- Aggregate placement, nesting, star legality, and expression effects are
  decided here exactly once. Ticket 05 consumes the sealed result.
- `CatalogPayloadV1::Model.provider` is bounded opaque frontend metadata. No
  provider lookup, capability routing, SDK selection, or backend registry may
  be added.
- The authority-byte limit is a per sealed admission session limit, not a
  cross-domain total. Module authority, relation authority, and per-document
  catalog validation each have one counter and the same projected ceiling;
  receipt 04 must state this explicitly. No proof may reset its own counter.

### Additional required tests

- every optional-authority combination: absent, modules only, relations only,
  both, and mismatched profile/source registry;
- `try_standard_v2` failure propagation with a deliberately invalid registry
  fixture;
- duplicate/missing/extra/foreign `SemanticItemIdV2` rejection;
- nested CHECK/FIND identity mismatch rejection;
- Bool-only `TypedBoolExprV2` construction;
- provider text preserved byte-for-byte as opaque metadata with no lookup;
- old private-HIR references are zero-match in binder after its migration.

### Inherited external delta audit (`fcf106b..f9ec954`)

Commit `ceb4142` is an **UNQUALIFIED PARTIAL CONSUMER MIGRATION**, not Ticket 04
progress. Its 29 binder tests pass, but the production walk still pre-registers
the complete relation tree before resolution. In a Join this makes the RHS
alias visible while resolving the LHS, projection does not replace the active
row frame, and relational EXISTS records no explicit correlation set. The
implementation still returns V1 binder carriers rather than the semantic V2
contracts above. Three deleted nested-composition tests are accepted only as a
transfer to Ticket 02's closed-HIR invalid-state proof; they are not closure
evidence here.

Ticket 04 must add exact rails for:

1. RHS aliases are unavailable in a Join LHS but available to `ON` and later
   stages;
2. projection replaces the row frame, so dropped fields cannot resolve later;
3. correlated EXISTS retains the exact nonempty capture set and uncorrelated
   EXISTS retains an explicit empty set;
4. competing binder failures select the pinned diagnostic independent of
   traversal/insertion order;
5. stage-order fixtures include both `group by ... | project ... aggregate`
   and the existing `project ... aggregate | group by ...`, so Ticket 05 never
   guesses whether an existing partition must be retained.

At the current inherited start (`f9ec954`), `cargo test -p
quanta-gqlang-v2-binder` is focused V1 green only. `cargo check -p
quanta-gqlang-v2-pipeline` is red with 25 compiler errors. Receipt 04 must use
the actual ticket-start HEAD plus dirty fingerprint and keep those result
classes separate.

Requires 02 and 03. Blocks 05-09.

## 2026-09-21 Owner/Consumer RR Reopen — Exact Work Remaining

This section is normative over every earlier contradictory signature or DTO
sketch. The live-source audit distinguished fixes already in the dirty tree
from gaps that still prevent `FOCUSED_GREEN`. Do not mark this ticket green from
successful compilation of an authority/DTO skeleton or from Binder's V1 tests.

### Producer seams that must close before the sema transaction

1. **One integrity pass, owned proof.** HIR owns
   `CheckedHirDocumentOwnedV2::try_new(HirDocumentV1)` and a borrowing
   `as_checked()` method. Sema's `CheckedSyntaxDocumentV2` contains that opaque
   owned proof; it never re-runs `check_document_v2` after moving the document.
   The current dirty HIR implementation and positive/invalid owner tests are
   the candidate; include them in the final HIR proof and receipt 03 delta.
2. **Exact combined source registry.** Types' `ResolvedTypeInputsV2::try_new`
   takes the same `&ExtendedSourceRegistryViewV2` that sema constructed from
   sealed host authority and the checked document. Types must not mint a
   document-only registry: record-field checked locations must inherit the
   combined identity. Reject source ID/length mismatch before a kernel walk.
   The dirty types implementation and exact/wrong-length owner test are the
   candidate; add a host-parent mismatch integration test at the sema authority
   boundary. The `BindingFromExpression` dependency target is a types-owned
   node ID, never a sema-inferred type; foreign, missing, self, forward, and
   cyclic dependencies reject deterministically.
3. **Modules predeclaration and all-family reference validation.** The current
   `DocumentCatalogValidatorV2::validate` creates local IDs only at final
   validation, while sema needs those IDs for forward references and typed
   item DTOs. Add one modules-owned predeclaration/resolution API on the
   existing validator and its single non-resettable budget session. It accepts
   all document assets once, seals checked local IDs/kinds, resolves typed
   local/visible-host/external references with expected-kind and ambiguity
   diagnostics, then final validation consumes that exact predeclaration. The
   old `validate(input)` may delegate internally for existing clients but may
   not run a second ID derivation or reset the counter. Extend the current
   View-only dependency rule to cover Model source, Pack source, Profile
   capabilities, Gate checks, and RuleBinding check references with explicit
   source/target-kind policy. Keep view-cycle detection limited to actual View
   edges; do not classify every family dependency as a view-cycle edge.
   Semantically unknown source/target kinds are a hard design failure, not a
   reason to silently omit an edge. Negative owner tests: missing/wrong-kind
   target for each family, forward local target, imported/external target,
   duplicate ID, ambiguous visible import, cycle only on View edges, and
   exact-limit/one-over across predeclaration plus final validation. Sema
   cannot call `CatalogAssetIdV2::derive` or duplicate module lookup logic.

   The source-backed family kind matrix is fixed here; the new kinds are
   frontend-only inert catalog metadata, not runtime/provider handles:

   | Authored reference | Required namespace kind |
   |---|---|
   | `VIEW query-ref` | `Query` |
   | `MODEL source` | `ModelSource` |
   | `PACK source` | `PackSource` |
   | `PROFILE capabilities[]` | `Capability` |
   | `GATE checks[]` | `Check` |
   | `RULEDEF check-ref` | `Check` |

   `CHECK rule-ref` resolves a `Rule` catalog asset or the exact source-rule
   predicate namespace; neither path treats it as a bare value. Authored
   catalog names preserve their full dotted/slash/`::`/hyphen spelling as one
   checked local path under the selected module. Do not split `catalog.users`
   heuristically into a different module or strip punctuation. Module ID
   syntax remains separate; local exact, visible current-module, and external
   candidates have deterministic kind-directed resolution and reject distinct
   same-name IDs as ambiguous. A wrong-kind candidate is not silently ignored
   to pick a different kind.
4. **Graph selector authority.** `GraphRefV1` accepts dotted, slash, hyphen,
   and `::` names; these cannot be reinterpreted as modules'
   `CatalogAssetIdV2` segments. Add an immutable, host-agnostic
   `GraphReferenceCatalogV2` to closed authority and an optional one to open
   authority. It seals typed canonical selector ID, kind, checked host
   location, and exact host registry/budget profile; it contains no graph
   runtime, provider lookup, traversal, SDK, or existence-at-runtime promise.
   FIND endpoint/waypoint/sanitizer/scope/window/frontier references and
   graph-valued presentation references use context-required selector kinds.
   CHECK rule references and catalog-family references use the rule/modules
   namespace, not this selector catalog. The source spellings above must have
   positive exact-ID tests; unknown, ambiguous, wrong-kind, foreign registry,
   and missing optional open authority must each have distinct negative or
   `Incomplete` tests. No raw string is labeled `ResolvedGraphReferenceV2`.

   The selector-kind matrix is:

   | Authored use | Required graph selector kind |
   |---|---|
   | FIND source/sink/anchor/reference/pointee/via/through | `Entity` |
   | FIND callable | `Callable` |
   | FIND/CHECK/SAMPLE scope | `Scope` |
   | FIND window/frontier/sanitizer | `Window` / `Frontier` / `Sanitizer` respectively |
   | CHECK pattern | `Pattern` |
   | COMPARE baseline/candidate | `Snapshot` |
   | EXPORT source | `ExportSource` |
   | DESCRIBE path | `Entity` with exact host-admitted entity subtype matching authored `kind` |

   EXPLAIN target may resolve to a catalog `Rule` or graph `Entity`, but two
   distinct candidates are ambiguous; neither namespace silently wins.
   EXPORT profile is modules `Profile`; CHECK rule is source Predicate or
   modules `Rule`. SHOW has no selector. The host catalog proves frontend
   reference identity/kind only, never runtime existence or provider routing.
5. **Source-less construction errors.** `FrontendDiagnosticV2.primary` remains
   mandatory for source-backed transaction failures. Authority/budget/standard
   constructor failures before a source exists return a distinct
   `FrontendAuthorityErrorV2` with stable code and bounded message, not a
   fabricated `SourceIdV2::from_raw(0)`/`SpanV1::point(0)`. Therefore
   `FrontendBudgetV2::{try_new,standard_v2,authority_profile}` and
   `ClosedFrontendAuthorityV2::try_new` /
   `OpenAnalysisAuthorityV2::{try_new,try_standard_v2}` return this type.
   Source entrypoints convert only genuinely document-anchored failures into
   `FrontendFailureV2`. `FrontendTransactionErrorV2` is the closed sum
   `Authority(FrontendAuthorityErrorV2) | Document(FrontendFailureV2)` and is
   the error type for semantic source/checked-document entrypoints and the
   `SemanticAnalysisV2::Error` payload. Modules source-less budget/registry
   errors remain source-less; a missing primary may not be silently anchored
   at byte zero.
   Test constructor failures without any source ID and deterministic ordering
   of multiple source-backed diagnostics separately.

### Sema-owned complete carriers and algorithm gates

1. `TypedExprV2` is a closed, private-field typed expression tree or shape
   table keyed solely by `SourceQualifiedNodeIdV2`. It retains every HIR value
   and Boolean variant; literal value, operator ID, ordered children,
   path-segment names/spans, resolved binding ID, resolved call target, and
   `CallArg::Star` are never recovered by a downstream HIR walk. The node's
   type/effects come only from one types-kernel result. Tests compare every
   shape, resolved target, effect, and child key with the input plus kernel
   facts; missing/extra/duplicate child facts reject.
   In the live draft, `Path { segments: Vec<String> }` and record fields omit
   authored segment/name spans, and `RelationExists` stores only a set of outer
   bindings, losing each inner reference location. Replace them with checked
   path/field/correlation carriers that retain `(outer binding identity,
   inner reference node and location)` per edge, including multiple references
   to the same binding. `TypedExprV2::try_from_fact` and
   `AggregateContextV2::try_new` cannot remain public caller-selected proofs:
   the producer must tie each shape and placement to the exact checked HIR
   node. Add same-key wrong-shape, wrong-position, dropped-correlation, and
   duplicate-reference negative tests. Store the checked expression location
   in `TypedExprV2`; passing it only to a constructor error loses downstream
   diagnostic provenance. Preserve operator/callee subspans where the authored
   HIR carries them.
2. `TypedRelationExprV2` has exactly the six source-reachable HIR stage kinds
   above. A `Join` carries inner/cross as a checked subkind, not two redundant
   source variants. Algebra-only operators are introduced by ticket 05.
   `ResolvedRelationSourceIdV2` is a closed sema enum over types-owned
   `RelationSourceIdV2` and modules-owned `CatalogAssetIdV2`; provenance uses
   that enum, not a string or `RelationSourceIdV2` alone. Source stages retain
   exact source ID, row type, alias/frame and provenance. Stage constructors
   are crate-private or validate output equality against producer-derived
   inputs: caller-supplied `RelationOutputContractV2` cannot forge source
   provenance, row, cardinality, partition/order, or effects. Negative tests
   explicitly attempt forged scan/filter/project/join output contracts.
3. `ResolvedValueBindingV2` retains Binder's exact `(owner HirNodeIdV1,
   ordinal)` identity. The display name is not the ID. Use the Binder
   relation-output and capture side tables; don't duplicate its scope walk.
   Pin omitted projection names (Symbol name, final Path segment, otherwise
   canonical `col`), duplicate defaults, dropped-field rejection after
   projection, independently bound JOIN RHS, and nested EXISTS capture
   ownership. An uncorrelated EXISTS has an explicit empty capture set.
4. `SemanticDocumentV2` covers all nine `ProgramItemHirV1` variants in exact
   source order: Query, Decl, Rule, GraphQuery, Legality, Presentation,
   CatalogDecl, Composition, and Meta. Each item has one
   `SemanticItemIdV2`; duplicate/missing/extra/foreign IDs reject before any
   partial result escapes. Exact ID alone is insufficient: assembly also
   compares each produced semantic variant with the authored HIR item kind
   and proves the catalog authority came from this exact document input and
   extended registry, not another same-source-ID/length transaction.
   Negative tests inject a same-ID wrong variant and a foreign catalog proof.
   Nested CHECK/FIND and inline VIEW identities are
   checked by an explicit parent-child relation, not merely equal source ID.
   Graph DTO retains direction and all endpoint/constraint references; CHECK
   retains rule, scope, pattern, policy, and nested graph; Catalog DTO retains
   every payload-specific ref/activation/admission/binding/provider field;
   Composition retains LET producer/WITH consumer and nested body; Meta keeps
   exact directive and checked refs. Only genuinely illegal grammar shapes may
   return `sema.invalid_item_shape`; dropping a supported family is not a
   successful Ticket 04 implementation.
5. Namespace lookup is deterministic and kind-directed. Local lexical values
   shadow only value bindings; they do not shadow predicates, relations, or
   catalog assets of another kind. Source rules predeclare in the predicate
   namespace, never as bare values. Module visibility is the sole import
   precedence authority. Within one namespace, two distinct visible IDs for
   the same spelling are ambiguous; no declaration-order/last-wins fallback.
   Builtins and user predicates require call syntax; an exact collision in a
   callable context rejects unless a documented qualified form disambiguates
   it. Record precedence and collision tests for builtin/source-rule/module
   Predicate or Rule, relation catalog/module Relation or View/document View,
   and composition-local bindings.
6. Rule grammar remains `rule name : <value> if <bool>`, but closed unary rule
   semantics admit only a single Symbol head. Arbitrary expression heads
   reject at the exact head span. No concrete type can be guessed for a head
   or quantifier binder: ticket 03 types owns a bounded finite-domain
   rule-variable constraint solver over exact Binder identities, all source
   rule IDs, authoritative host signatures, literal/operator/builtin
   constraints, and checked document HIR. Its sealed result supplies unique
   concrete binding types and unary predicate signatures to the existing
   `ResolvedTypeInputsV2` builder; the existing expression kernel then runs
   exactly once. Empty candidate intersection rejects contradiction; multiple
   candidates or an unanchored recursive SCC reject closed compilation and
   remain explicit incomplete facts in open analysis. `rule r : x if r(x)` is
   unanchored and therefore *not* a closed positive fixture. Use an anchored
   recursive fixture such as `rule r : x if x = 1 and r(x)` after checking the
   operator overload selects a unique `int`. Tests: arbitrary head,
   self/mutual unanchored recursion, conflicting anchors, ambiguous overload,
   unanchored quantifier, free variable, predicate arity, unknown callable,
   exact solver budget, and one-over. Sema does not implement a second
   overload/constraint engine, and logic does not infer types.
7. Charge `max_semantic_steps` in one document-wide, non-resettable sema
   session: one charge at entry to each authored item, each resolved reference,
   each Binder binding/capture fact imported, each node-ID type input emitted,
   each relation stage assembled, and each semantic item/catalog DTO emitted.
   Charge immediately before work/allocation and attach the actual source
   span of that work to an over-limit failure. Surface owns parse tokens and
   expression-node count; types' rule solver charges its own bounded
   node/edge/iteration dimensions; modules owns its existing admission
   session. Do not charge the same event in two sema phases. Owner tests set a
   custom limit to exactly the sum of a documented tiny fixture's charges,
   assert success at that limit, then assert the next charge fails at its
   expected source span and code.

### Checkpoint evidence required

- Re-run HIR/types/modules/Binder owner suites and strict Clippy after their
  producer changes. Preserve Ticket 03's earlier receipt as frozen evidence,
  append the changed API/test counts and a fresh scoped fingerprint rather
  than pretending the original proof covers new code.
- The sema receipt records a source-family acceptance matrix, every typed DTO
  field's producer/consumer, one-kernel-call proof, one-catalog-validation
  proof, authority source identity, exact budget schedule, and all RED-to-GREEN
  commands. A skeleton with unsupported valid families is `REPAIR`, not
  `FOCUSED_GREEN`.
- Keep the preexisting pipeline compiler failures classified as deferred
  ticket 07 consumer migration. A focused Ticket 04 pass never implies a
  releasable workspace or final V2 facade.

## 2026-09-21 Live-Tree Final Reconciliation (HEAD `f9ec954` + dirty overlay)

The sema crate now contains authority, diagnostics, document, expression,
relation, rule, name, and item modules plus owner tests. This is an in-flight
producer, not an implemented document transaction: at this checkpoint
`resolve_document_bodies` rejected every nonempty document. The later live
delta below has Decl/RequireCertified source paths, but no DTO-constructor or
two-family test may satisfy the nine-family acceptance matrix.

The next checkpoint must execute this order without substituting constructor
tests for source-document tests:

1. Freeze the Ticket 03 producer APIs and rerun their owner suites. Import the
   exact checked HIR, Binder side tables, types results, modules predeclared
   IDs, and graph-selector proof; do not parse or resolve any of them twice.
2. The initial seven-variant sema relation draft (`Scan`, `Filter`,
   `Project`, `Join`, `Cross`, `Group`, `Sort`) was found by RR and has been
   collapsed in the dirty overlay to six variants with checked inner/cross
   `JoinModeV2`; stage constructors were narrowed to crate scope. This is
   only a shape repair. Tie each producer-only output to owner-derived row,
   source-sum provenance, cardinality, partition, ordering, and effects;
   restrict `TypedRelationV2::try_new` to the producer or verify its authored
   item ID at construction. Negative owner tests must forge each output field
   independently and try a same-root/foreign-item-ID transplant.
3. Materialize all nine authored item variants in one source-order transaction,
   with exact item ID and variant equality. Resolve nested CHECK/FIND and
   inline VIEW child identities explicitly; compare the exact set of semantic
   catalog references with the validated `DocumentCatalogAuthorityV2` proof.
   Reject a foreign catalog proof even if source ID and length happen to match.
   Test one positive source case
   per variant plus missing/extra/duplicate/foreign/wrong-kind mutations.
   `GraphReferenceCatalogV2::resolve` must not return a detached spelling/kind
   tuple: the resolved selector retains its checked host declaration location
   and opaque catalog/registry/profile provenance (or an equivalent sealed
   reference back to the exact catalog). A same-spelling/kind ID transplanted
   from another host registry or catalog must fail even when it compares equal
   as text. No downstream graph DTO may reconstruct that proof by relookup.
4. Invoke the types kernel once on the complete resolved input. Rule-variable
   singleton proof comes from types; Binder identities/captures come from
   Binder; module references from modules. Document-level source rules must
   retain exact target ID, polarity, and span for Ticket 06. Graph and item
   payloads must retain every authored option for Ticket 07. Before constructing
   solver inputs, compare each rule-symbol mapping to Binder's exact resolved
   target and each call target to the checked callee spelling plus sealed
   builtin/host/source-predicate authority. Same-spelling cross-rule and
   cross-call transplants reject; a caller-supplied signature is not proof.
5. Charge one non-resettable semantic budget with an enumerated tiny fixture's
   event ledger. Run limit and limit+1, ordered multi-diagnostic, open
   incomplete vs closed failure, wrong authority/profile/registry, and
   source-less construction-error tests. Then run the entire sema/Binder owner
   suites, strict Clippy, architecture guard, and postimplementation RR.

`cargo check --lib` on the new crate or a small authority/DTO test subset is
only a compilation checkpoint. Ticket 04 remains `RED_CAPTURED — IMPLEMENTING`
until the above source-document matrix and fresh receipt are green.

## 2026-09-21 Second Live Consumer Audit — Source Reachability Gate

`sema/src/document.rs` now has a checked-HIR → predeclared modules → body →
final catalog validation → private exact-item-set assembly path. The current
body resolver's last all-target-proven source families were value `Decl` and
`Meta(RequireCertified)`; the other seven root DTO variants existed as types
only on that fingerprint. A `cargo check --tests` checkpoint is not an executed
test or strict Clippy receipt. The serialized
`v3_document_transaction` run passed 8/8, including forward declarations and
an exact-span rule-body relational EXISTS rejection; it does not cover the
seven unproduced root variants or qualify the full owner suite. A later dirty
GraphQuery source-positive test passed in isolation and Presentation resolution
is being edited; neither is included in that 8/8 or 19-test all-target
checkpoint. Recompute the exact produced-family set only after the next
stable sema full run.

Next compilation-only checkpoint: `cargo check -p quanta-gqlang-v2-sema
--tests` passed with 12 dead-code warnings. Focused source positives now exist
for 4/9 roots: Decl, GraphQuery/FIND, Presentation/SHOW, and
Meta/RequireCertified. Only SHOW has an executed Presentation positive; other
Presentation verbs, EXPLAIN dual authority, and Meta/UsePack lack positive
source proof. Query, Rule, CHECK, CatalogDecl, and Composition remain source
RED. The latest full sema owner suite and strict Clippy have not run after
these changes; 4/9 is a reachability checkpoint, not semantic closure.

The subsequent dirty checkpoint advanced to 6/9 root variants: authored
CatalogDecl/RuleBinding is matched to modules-issued exact name/kind/checked
span, CHECK resolves its local Rule and optional typed Scope/Pattern with exact
nested FIND identity, and private assembly compares catalog asset and family
dependency sets to the modules proof. One focused catalog+CHECK source test and
`cargo check -p sema --tests` passed (10 dead-code warnings). Query, Rule,
Composition remain without source producers; most Catalog/Presentation/Meta
subvariants and the full owner suite/Clippy remain unproved. The 4/9 statement
above is an earlier checkpoint, not the latest reachability count.

Next isolated Query/Source positive passed: Binder `resolve_query` produced
the checked relation frame, the sealed host `RelationCatalogV2` supplied exact
row/cardinality/provenance, and the typed stage retained its source-qualified
HIR key. Sema `cargo check --tests` passed with seven dead-code warnings.
Reachability is now 7/9 roots at the named subvariants; Rule and Composition
remain absent. Filter/Project/Join/GroupBy/OrderBy source stages and the
remaining family subvariants still need source positives and negatives.
At that checkpoint the post-Query full sema suite and strict Clippy were unrun.

A later full sema owner run passed 23/23 before a new typed-child semantic
budget charge; seven dead-code warnings remain. Rerun after the budget edit
and eliminate warnings by completing/using the missing source branches, not
by suppressing them. This is not a 04 `FOCUSED_GREEN` receipt.

The immediate post-budget rerun passed 24/24, including a typed-child
exact-step/one-over diagnostic. Seven dead-code warnings still prevent strict
Clippy; Rule/Composition and non-Scan relation stages remain RED. Ticket 05
subsequently reopened relation provenance/location/nested-EXISTS producer
gaps, so the 24-test fingerprint is a checkpoint, not a frozen Ticket 04
handoff.

Implementation order and hard acceptance for the remaining root work:

1. For every authored root variant, add a minimal source-positive and a
   wrong-kind/foreign-ID negative in the sema owner suite. Query must traverse
   all six HIR relation stages; Rule must synthesize the unary head parameter
   for Binder and submit exact symbol/call targets to the types solver; FIND
   and CHECK must retain every checked graph selector, role, option, nested
   identity, and host provenance; Presentation, Catalog, Composition, and
   `Meta(UsePack)` must retain all authored payloads. Merely constructing the
   nine DTO variants is not evidence.
2. Build one complete `ResolvedTypeInputsV2` for the whole document and call
   `ExpressionTypeKernelV2::check_document` once. Retain exact typed value and
   Boolean nodes, subspans, captures, effects, and aggregate context. Replace
   current generic item-span type errors with the owner diagnostic location;
   a source-backed type error must point to the violating operand/callee, not
   the first document item.
3. The graph catalog now preserves checked host declaration location and full
   catalog identity; expression DTOs preserve path/record subspans and each
   correlation edge. Verify that every constructor remains producer-private
   and every downstream DTO carries the proof unchanged. Negative tests must
   transplant a same-spelling graph ID across registries/catalogs, a relation
   output with one forged field, an item ID with wrong variant, and a nested
   CHECK/FIND child ID with the right source but wrong node.
4. Collect every authored catalog declaration before body resolution, then
   supply exact six-family located references and resolved View-only edges to
   the single modules validator. Compare the exact set of sema catalog refs
   against the validator's sealed authority, not only source registry and
   count. Prove forward View/reference and both orders of
   `REQUIRE CERTIFIED` against one final validation.
5. `exists (from ...)` in a rule body is parser-reachable. Preserve the
   Ticket 00/04 semantic prohibition: exact-span
   `sema.invalid_item_shape` at source ingress and a direct types-solver
   negative. Query EXISTS remains a supported typed/capturing relation path;
   rejecting the source-rule form must not disable query correlation.
6. Execute the full sema/Binder/types/modules owner suites, strict Clippy,
   exact semantic budget limit and limit+1, ordered multi-diagnostic/open
   incomplete cases, architecture transition guard, and a failure-injection
   mutation. Record selected/executed counts and every still-red consumer in
   `receipts/04.md`. No physical algebra/planning or final facade claim belongs
   to Ticket 04.
7. An earlier `sema/src/rule.rs` draft exposed public constructors for a
   caller-selected rule call list and checked only source ID. The dirty draft
   now narrows constructors and retains call-node IDs. Complete the producer
   comparison against the exact checked-HIR call-node set, types-resolved
   target, authored polarity, and checked location before sealing. The current
   expected-call tuple still omits the target, so same-key target transplant
   must reject. Add missing/duplicate/extra edges, a same-source
   different-rule head/body transplant, and a negated edge relabeled positive.
   Ticket 06 may trust only this sealed rule graph, never reconstruct it from
   names or raw HIR.
8. EXPLAIN target can be a modules catalog Rule or a graph Entity. Resolve
   both through their existing owners, using modules' same-session
   nonterminal absence probe and an equivalent typed nonterminal lookup on the
   immutable graph catalog; reject two present candidates as ambiguous.
   A modules wrong-kind or ambiguity error is not an absence fallback. Add
   both probe orders, catalog-only, graph-only, dual-present, wrong-kind plus
   graph-present, and neither-present source fixtures. CHECK's source-predicate
   versus catalog Rule choice needs the analogous exact-ID matrix.
9. `SemanticDocumentV2::try_assemble` currently checks expression map keys
   against contained fact keys and source ID, but not that the map is the
   complete checked-HIR/types expression set. Require exact equality of
   checked expression IDs, types-owned fact IDs, and materialized semantic
   expression IDs; reject omitted/extra child nodes and same-key wrong-shape
   transplants. Apply the graph-catalog `admits` and checked-use-location
   validation through nested CHECK/FIND and composition graph payloads as well
   as standalone Graph roots.
10. After charge-before-work repair, the types solver's constructed
    Record/List fixture measures 15 nodes, 20 edges, and 6,884 steps over
    74 candidate types. In the source rule path, project the
    one canonical `FrontendBudgetV2` into `RuleTypeBudgetV2`, retain exact
    solver usage in the receipt, and test standard-budget positive plus exact
    limit/limit+1. Do not enlarge or reset the budget just to make one fixture
    pass. Ticket 03 charged the major candidate prepass/tuple allocation
    paths; the remaining input-map/fixed-allocation and host-signature-driven
    universe bound still need source stress proof before resource closure.
11. Ticket 05 lowering currently cannot consume the producer without falling
    back to HIR: relation stages retain expression IDs but no direct typed
    projection/group/sort payload, `OutputProvenanceV2` is only a source set,
    and `RelationExists` retains a subquery root ID without the typed child
    relation. Freeze one sealed `SemanticDocumentV2` lookup contract for every
    referenced expression and nested relation; require exact ID, membership,
    type/effect, and capture proof. Add per-output-column origin aligned
    one-to-one with the checked row schema (direct source field versus derived
    typed expression; algebra-only programmatic variants remain algebra-owned).
    The sema producer proves these, including column reorder/alias/group and
    nested EXISTS transplants; Ticket 05 must not infer them from raw HIR or
    caller-provided optional maps.
12. Each `TypedRelationExprV2` stage and its root item must retain a checked
    source-qualified location, not only a node ID. The final DAG removes
    algebra's direct HIR dependency while current algebra diagnostics still
    store `SpanV1`/`HirNodeIdV1`; absent sema locations would force a HIR
    rewalk or fabricated error position. Expose a read-only host-agnostic
    checked location through sema and test foreign-source and wrong-stage
    location transplants. Algebra may import semantic/types primitives via
    final allowed edges, not direct HIR syntax.
13. In row-aligned `OutputColumnOriginV2::SourceField`, the host/View asset
    ID alone is insufficient: self-joining the same relation under two
    aliases gives two scan occurrences with one asset ID. Retain the exact
    source-qualified scan-stage or Binder binding ID in each origin and
    validate its mapping to the sealed source. Test self-join same asset,
    swapped alias/scan ID, and duplicate field names. Projection-derived
    columns still point to typed expression IDs. Re-export the existing
    host-agnostic checked location and sort-direction primitives through
    sema, or replace with one sema-owned closed carrier, so Ticket 05 can
    drop the direct HIR Cargo edge without inventing duplicate enums.
14. Seal complete rule membership at the document producer: enumerate every
    authored Rule item from the checked HIR in source order, require exactly
    one `ResolvedRuleV2` per item with the same source-qualified ID, and
    expose the immutable enumeration through `SemanticDocumentV2`. For each
    rule, derive the expected call-node/target/polarity/location tuples from
    the checked body plus types result independently of the supplied call
    vector; `ResolvedRuleV2::try_new` comparing two caller-built vectors is
    not a proof. Reject omitted, duplicate, foreign, target-swapped, and
    polarity-swapped calls in sema owner tests. Logic consumes this sealed
    document enumeration and must not rebuild it from names. Distinguish
    source-local predicate calls from host predicate calls by the exact
    resolved modules/catalog origin before emitting SCC edges:
    `ResolvedCallTargetV2::Predicate(id)` alone does not encode origin.
    Builtin/host calls stay in typed expression facts but not local
    `ResolvedRuleV2::calls()`. Test a host predicate with the same display name
    as a source rule and a forged local/host origin transplant.
15. Preserve the authored FIND kind, every ordered/optional selector,
    direction, depth, summary facet, and one checked whole-query location in
    `ResolvedGraphQueryV2`. Project immutable `FrontendBudgetV2` depth and
    witness caps, but do not duplicate logic's subkind-to-dedup/tie-break
    policy: Ticket 06 owns that derivation. HIR `GraphConstraintsV1` has no
    authored cap/dedup/tie-break field, so none may be invented as source
    metadata. Re-export the canonical closed `GraphQueryKindV1` through sema
    so logic can remove its direct HIR dependency without cloning the enum.
    Test wrong-registry query span and selector/span transplants.
16. Resolve host predicate calls via the exact modules catalog asset:
    `resolve_reference` -> `external_catalog().asset(reference.id())` ->
    `ExternalCatalogPayloadV2::Predicate(signature)`. Do not synthesize a
    `ResolvedPredicateIdV2` from its display name: types' current ID grammar
    permits only ASCII `::` segments, while modules permits local names with
    `.` and `/`. Ticket 03 must add a modules-owned injective typed host
    identity bridge. Explicit rejection is acceptable only as a temporary
    fail-closed cutover state, not final coverage; never normalize distinct
    host names to one ID. Test punctuation names, same-spelling
    local/host predicates, and invalid payload kind.
17. Do not manufacture planning authority from free-form relation
    `provenance: String`. For `REQUIRE CERTIFIED`, compare the authored Meta
    item checked location and exact count to
    `DocumentCatalogAuthorityV2::require_certified_location()` after modules
    validates profile admissions; expose the validated directive as a closed
    sema fact. No other item becomes "certified" by inheriting that display
    string. Catalog Model remains non-serving metadata in this packet. Test a
    forged/missing certified directive and a renamed host provenance string
    that cannot change plan authority.
18. Re-export the canonical HIR-defined `SourceIdV2` through sema alongside
    checked locations and graph-kind/direction carriers. Ticket 07's pipeline
    and facade need to name source identity after their direct HIR Cargo edge
    is removed; a second ID type or a hidden syntax import is forbidden.

## Latest Rule Source Producer Checkpoint

The sema owner subsequently added one partial Rule source path: a unary
`rule r : x if x = 1` is prechecked, Binder supplies the exact head parameter
and `(owner, ordinal)` symbol identities, types' rule solver anchors `x`,
the one expression kernel proves the Bool body, and a closed
`ResolvedRuleV2` is emitted. The focused document transaction suite passed
16/16, including unanchored `x = x` rejection, rule-body relation EXISTS
rejection, exact child type-error location, and FIND depth-over-budget.
Call-bearing rules still reject, so no source call graph, negation SCC, or
complete rule-program proof exists. This raises named root reachability to
8/9; Composition, five non-Scan relation stages, and subvariant coverage
remain RED. The focused 16/16 result is not a post-delta full sema suite or
strict Clippy receipt. Steps 14-18 above are still mandatory.

Later focused source work completed builtin-only Rule calls and all six
Catalog reference-form declaration branches in the same modules validator
session. The owner reported 18/18 then 19/19 focused document tests;
`SemanticDocumentV2::try_assemble` compares six-family dependency edges and
profile admissions to the modules proof. Builtin calls remain typed facts,
not SCC edges. Source-predicate and host calls, Inline VIEW, USE PACK,
Composition, and the non-Scan relation matrix remain RED. The prior full
30/30 sema run predates these edits; strict Clippy exited 101 on an earlier
source (20 library, 41 library-test errors), so no current owner closure is
claimed. Steps 14-18 and full post-delta owner proof remain required.

The next full owner checkpoint passed 34/34 after source-local anchored and
negated Rule calls, six Catalog reference forms, and exact validated REQUIRE
CERTIFIED location/count. A later focused two-rule mutual-dependency test
passed 1/1, but the 34/34 full run predates it. Source-local Rule calls are
therefore no longer universally RED; host calls, full aggregate/negation SCC
outcomes, Composition, Inline VIEW, five non-Scan stages, and strict Clippy
remain open. Ticket 06, not sema, classifies the accepted mutual/negative
dependency graph. The host ID producer split is in Ticket 03.

## Final Live Gap Audit — Group, Sort, Host Calls, Composition

The earlier 36/36 sema all-target result is historical. The later 43/43
all-target owner run includes Query Scan/Filter/Project/inner Join/Sort,
host Predicate calls, and a self-Join proof with two scan occurrences,
distinct column-expression origins, checked `ON`, and Bag output. Strict
Clippy still fails with 45 lib-test errors. Group has a later focused
key-only frame/row milestone, but no aggregate partition-input proof; the
existing Sort/host positives do not satisfy their full negative matrix or
the public pipeline. Exact three-file hashes are in `receipts/04.md`.

### Group output and aggregate-only input — Binder/Sema owner

At the pre-repair audit snapshot, `binder/src/resolve.rs:593-600` resolved
Group keys in the input frame and returned that entire frame, while
`algebra/src/normalize.rs:389-418` emitted key-only Group columns. The
subsequent focused Binder/sema milestone implements step 1 and the key-only
part of step 3 below; steps 2, 4, and full owner proof remain open:

1. Binder resolves every authored key against the input frame, then replaces
   the public frame with ordered key-only `Alias` bindings at exact identity
   `(group_relation.id, authored_key_ordinal)`. Factor Project's output naming
   rule for both stages: Symbol name, final Path segment, otherwise `col`.
   Group has no authored alias. Use the existing checked relation-frame
   constructor to reject duplicate implicit names at the second key span.
2. Keep the input frame as an opaque, keyed `PartitionInputV2` side authority
   attached to that Group stage, never as public output. Binder may retain a
   deferred candidate for a post-group reference inside a call argument, but
   must not accept it merely because it is syntactically inside a call. Sema
   first resolves the exact types-owned callable and its *intrinsic* effects
   by the chosen `BuiltinIdV2`; only an aggregate call may consume these
   candidates, and only for its argument
   subtree. A scalar/builtin/host nonaggregate call, nested aggregate, plain
   Project value, Filter, Join ON, or OrderBy cannot consume them. Missing,
   ambiguous, transplanted, or dropped partition identities reject.
3. `QueryTypeInputsV2::visit_relation` seeds `(group.id, ordinal) ->
   BindingFromExpression(key.id)` before downstream expressions are typed.
   Materialization checks Binder frame count/order/identities against the
   authored keys, retains key-only row schema and exact typed key origins,
   marks Set cardinality, and seals partition-input identities separately.
   Algebra receives that sealed partition, not the Binder frame or raw HIR.
4. Owner tests: `group by u.id | order by id` succeeds with `(group.id,0)`;
   `order by u.id` and post-group `where u.id` reject; two keys preserve order
   and ordinals; duplicate final Path names and two computed `col` names
   reject at the second span; Project→Group uses the prior Project binding;
   Group→aggregate Project accepts `sum(u.value)` only through partition
   authority, while `project u.value`, `project abs(u.value)`, and a forged
   aggregate-candidate identity reject. Keep `project ... aggregate | group
   by ...` as a separate stage-order fixture, with no accidental second Group.
   Do not use composed `TypedExpressionFact.effects` as the classifier:
   `abs(sum(x))` has aggregate-composed effects although `abs` is scalar.
   Pin a negative where a scalar wrapper cannot gain partition access merely
   from an aggregate child, and a same-family different-overload fixture if
   the builtin registry supports one.
5. If an aggregate argument after Group names a private pre-group alias and
   the same spelling also resolves to a declaration or import, Binder rejects
   `binder.partition_authority_collision` at that reference span. It must
   neither silently prefer the declaration/import nor promote the private
   alias into the public lexical frame. Test both collision sources and a
   no-collision non-key aggregate positive.

### Sort and host-call decisions

- HIR `OrderKeyHirV1` has only Asc/Desc; no source null-placement token exists.
  Apply Ticket 00's `NULLS LAST` default to both directions in sema's
  `OrderKeyContractV2`. Retain checked key type and exact expression ID.
  Test ascending/descending nullable keys, order after Project alias, and
  rejection of an unavailable pre-Project/pre-Group field. Ticket 05 may not
  rederive this default.
- For every Rule call, compare exact source Rule ID and builtin family
  candidates. If either exists, run one modules Predicate probe at the call's
  checked location; a host hit or source+builtin overlap rejects with
  `sema.ambiguous_callable`. With neither candidate, call modules
  `resolve_host_predicate_call` directly, consume its sealed same-session
  `ResolvedHostPredicateProofV2`, and feed its distinct host key/signature to
  types. Revalidate key, target, and location at semantic assembly. Host and
  builtin calls remain typed facts but never source-local SCC edges. Tests
  cover dotted/slashed names, imported alias preserving underlying ID,
  source/host and builtin/host collisions, wrong payload kind, foreign
  session/location/key transplants, and exact/one-under admission budget.

### Composition working-set authority

Binder already resolves sequential `LET`, no self-reference, `WITH`, and an
outer-composition capture barrier in `binder/src/resolve.rs:316-375`; do not
reimplement lexical scope in sema. The current sema scans only root Queries
and resolves relation/graph sources by host text, so no Composition producer
is source-positive. Implement one typed local working-set table keyed by
`(composition.id, ordinal)` and the exact Binder `CompositionLet` target:

1. Visit each LET producer in source order before inserting its binding.
   Seal its actual semantic output and kind (`Query` relation, `GraphQuery`
   graph result, or `Sample` result) with source-qualified producer node ID,
   output contract, and checked location. Migrate the existing exact
   composition-row contract from `pipeline/src/compile.rs:859-934` into this
   one producer: Sample has `Set<{path: RepoPath}>`; bounded PATH and
   REACHABILITY FIND have `Set<{function: EntityRef}>`. Query uses its sealed
   typed output row; other FIND kinds have no relational row contract. The
   Sample/FIND row is a frontend type contract, not runtime materialization.
   Do not turn any local result into a host catalog asset or synthesize a
   relation/graph selector ID.
2. On a relation Source or graph/scope reference, consult the exact Binder
   target first. A `CompositionLet` target must consume the matching sealed
   local producer with a kind-compatible typed source-use carrier; a mismatch
   rejects at the use span. Only an actually unbound local reference may go
   through the existing host modules/graph catalog resolver. A local binding
   cannot fall back to a same-spelled host name when its producer is wrong.
3. Resolve the body and `WITH` against the same table. Require `WITH` to name
   one prior LET and retain the exact consumed producer/body identity and
   source order in the Composition semantic item; Ticket 07 selects the body
   result without replanning each LET as an independent root item.
4. Migrate `pipeline/tests/v3_composition_type_env.rs`: Query projection
   `id` then `from p` succeeds, bad overload/missing field reject; Sample
   `from p` exposes only `path: RepoPath`, not `function`; PATH/REACHABILITY
   FIND exposes only `function: EntityRef`, not `path`; TAINT FIND cannot be
   consumed as a relation. The V1
   `composition_empty_projection_does_not_overbind` positive is intentionally
   inverted: closed V2 must reject an untyped/missing producer row or field,
   not pass via schema-open env. Add sequential LET→LET, wrong-kind use,
   forward/self reference, duplicate LET, missing WITH, same-spelling host
   shadow, and outer-composition capture negatives with exact use spans.

The post-43/43 focused Group milestone changed Binder public output to
`(group.id, key ordinal)` and retained the pre-group frame under
`partition_input(group.id)`. Sema maps the keys to their typed expressions,
checks Binder output/partition identity, and emits a key-only Set row.
Binder 2/2 and sema 2/2 focused tests passed, including `group by r.id |
order by id` and rejection of post-group `r`. This is not a post-edit
all-target or Clippy result. Aggregate-only partition candidate authorization
was still RED at that 2+2 snapshot; Ticket 05 may not consume Group merely
from that focused evidence.

A later full owner-local run on `ecc38f5`+dirty passed Binder 46/46 and sema
51/51, including `group by r.id | project sum(r.value)` and plain/scalar,
nested, scalar-over-aggregate, post-group Filter/Order negatives. Combined
strict Clippy remains RED (21 sema lib and 43 sema lib-test errors);
candidate forgery/outer-call transplant and the declaration/import collision
rails in step 5 remain open. Exact hashes and the scoped result are in
`receipts/04.md`; this does not authorize Ticket 05 integration yet.

### Rule edge polarity must use intrinsic ancestor effects

The same composed/intrinsic distinction affects
`RuleDependencyCollectorV2::visit_value` (`sema/src/document.rs:674-721`):
the earlier 43/43 snapshot set its `aggregated` traversal state from
`fact.effects.aggregation`, which includes descendants. A scalar call with
an aggregate child can therefore mark a source Rule call in a sibling
argument as an aggregated dependency although the scalar ancestor is not an
aggregate operator. For each ancestor Call, compare its exact types-resolved
target to the source/modules proof, then use the chosen builtin overload's
intrinsic effects (or sealed host callable's intrinsic effects); source
predicates are intrinsically Scalar. Push/pop that intrinsic context through
the actual call tree. The call's own SCC edge polarity reflects ancestors,
not the composed effects of its arguments. Add exact call-node/edge/polarity
oracles for scalar-over-aggregate, aggregate-over-source-call,
aggregate-sibling/source-call under the *same scalar Call ancestor*, nested
aggregate reject, and negation plus aggregation reject. A test with two
separate `is_null(...)` siblings does not kill the old defect: the source
call needs to be a sibling descendant of an aggregate under the same
scalar wrapper, e.g. a record argument `{a: count(1), b: r(x)}` if the
closed grammar/type proof admits it. Re-enable the old composed-effect line
as a mutation and require the expected Positive edge oracle to fail.
Ticket 06 must not repair wrong polarity after the fact. The 51/51 source
changed this to exact builtin/host intrinsic effects, but its first focused
source oracle used separate scalar-call siblings and is not yet a
mutation-killing proof.

Latest bounded owner snapshot supersedes that first oracle: Binder 47/47
all-target tests and strict Clippy pass; sema 52/52 all-target tests pass but
strict Clippy fails with 21 library and 43 library-test errors. The source
polarity fixture now uses one scalar `is_null({aggregate: count(1),
recursive: r(x)})` wrapper, so the Positive `r(x)` edge discriminates the
old composed-effect collector; `count(r(x))` remains Aggregated. The
declaration/import partition collision negatives also pass at the exact
reference span. `receipts/04.md` holds checked file hashes. This closes
those *owner-local source oracles only*: candidate forgery/outer-call
transplant, Composition, Inline VIEW, relational EXISTS, broad subvariant
matrix, sema strict Clippy, and integrated document pipeline remain RED.

### Final adversarial delta — exact Group witness, Composition uses, placement

This is a work specification, not an additional GREEN claim. The current
`BindResultV1::partition_candidates` side table and
`sema/src/document.rs:1536-1566` check exact partition binding, Binder target,
nonempty call stack, and types-resolved intrinsic builtin effects. They do
**not yet independently prove** that each recorded call-stack ID is an
ancestor of the exact symbol in the authored HIR or that the candidate came
from the Project immediately consuming the named Group. A private Binder
field makes arbitrary public forgery impossible, but does not substitute for
the cross-owner invariant or its mutation oracle. Complete this before
Ticket 05 consumes `PartitionInputBindingV2`:

1. Sema walks the already checked six-stage HIR once to index each source
   symbol by `(source ID, reference node ID)` with exact span, containing
   relation stage, predecessor Group identity, and outer-to-inner **Call
   argument** ancestor IDs. This is structural containment only; Binder
   still exclusively owns lexical resolution. Compare that witness to each
   Binder candidate's reference, group, span, and entire ordered call stack,
   then to the exact Binder partition/target identity and types-owned
   resolved overload effects. Reject missing, extra, duplicated, stale,
   sibling-call, wrong-Group, and non-Project candidates before materializing
   any semantic item. Do not infer containment from overlapping spans.
2. Use an owner-internal candidate-tamper test hook (not a public constructor)
   to replace the group ID, binding ordinal, reference ID, and outer-call ID
   independently on an otherwise valid Group→aggregate Project. Each must
   fail atomically at the authored reference span. A second Group and two
   same-family Call nodes make transplant/same-name substitutions observable.
   Remove each comparison in a mutation run and require at least one test to
   fail. Positive `sum(scalar(path))` must retain only the one outer aggregate
   and scalar inner call; `scalar(sum(path))`, nested aggregate, Filter/Join
   predicate, and OrderBy remain negative.
3. `AggregateContextV2::try_new` currently rejects aggregate effects only in
   `GroupKey` and `Predicate` (`sema/src/expr.rs:153-177`). A root Decl is
   materialized at `Scalar` (`document.rs:1656`), so a standalone aggregate
   declaration has no placement reject in that function. Add an exact-source
   `Decl = sum(1)` negative before changing policy; if types admits it, reject
   at the aggregate call through the central position contract. Keep RuleBody
   aggregate polarity legal only under its separately specified rule semantics.
   OrderBy's root-level aggregate guard exists, but add a nested-expression
   oracle so no child/path bypass is mistaken for a central guarantee.

Composition requires an additional Binder/HIR ingress before the local table
described above can work. The parser/HIR already preserve the closed
Query/Sample/FIND producer and Query/FIND/CHECK/EXPLAIN body kinds; do not
create a second Composition syntax tree. Execute this owner sequence:

1. Ticket 02 threads the existing parsed source-name token span into
   `RelationSourceV1` (see its producer delta). Binder's Source branch
   (`binder/src/resolve.rs:587-600`) must then emit one exact source-use
   resolution at relation ID/name span before it creates the alias frame.
   A same-spelled host is fallback **only** for genuinely unbound local uses;
   a wrong-kind local LET rejects and cannot fall through to host lookup.
2. Binder graph references currently record multiple uses under one enclosing
   node ID (`resolve.rs:430-463`), while `target_of` returns the first match
   by node (`:130-137`). Add an exact per-use accessor keyed by node, checked
   span, and spelling (or a deliberate HIR-wide unique-use-ID migration),
   reject duplicate keys, and make Sema consume that proof for every FIND,
   CHECK, EXPLAIN, and graph/scope reference. Test two differently bound refs
   with the same enclosing node ID, including one wrong-kind/local-vs-host
   collision. Do not use `target_of` for these multi-use nodes.
3. In the one document transaction, visit LET producers in order, seal each
   output under `(composition.id, ordinal)`, then type/materialize the body and
   exact WITH use. Extend the existing type-input traversal to nested producer
   and body Queries; keep the root item set as one Composition, but include
   *all* nested expression IDs in assembly equality. Query uses its existing
   typed output; Sample and bounded PATH/REACHABILITY FIND use the exact V1
   frontend row contracts already listed above; other FIND kinds have no
   relation row. A missing row/field rejects closed typing, not schema-open
   acceptance.
4. Extend `ResolvedScanSourceV2` with a sealed local Composition producer ID
   and checked use location, not a fake host/View asset. Graph uses need a
   typed local-vs-host target retaining role/kind and provenance. Strengthen
   `ResolvedCompositionBindingV2`/`ResolvedCompositionV2` from strings and
   boxed items to exact producer kind/output, Binder owner+ordinal, ordered
   body identity, and checked WITH target. The document assembler recursively
   verifies nested item kind/ID/source/session/catalog authority; root-only
   equality is insufficient.
5. Migrate `pipeline/tests/v3_composition_type_env.rs` and
   `v3_composition_producer_cardinality.rs` to owner-level and later facade
   oracles. Replace the old schema-open empty-projection success with V2
   rejection. Test sequential LET→LET, self/forward, duplicate/missing WITH,
   same-spelling host shadow, wrong kind, Query/Sample/FIND row and field
   typing, body four-kind dispatch, exact graph multi-ref, foreign session,
   and one-under/over budget. The old
   `v3_composition_catalog_scope.rs` invalid-HIR fixture uses the retired
   unrestricted constructor; replace it with a compile-fail closed-enum
   assertion and a valid same-document catalog case. Outer-Composition
   capture is Binder-frame-only because nested Composition is not in the
   current closed source grammar; do not expand language scope for the test.

After source wiring, remove only the now-fulfilled narrow `expect(dead_code)`
annotations on private Composition/EXISTS constructors; strict Clippy passing
with future-only suppressions is not source-family coverage. Record selected
and executed counts, exact owner-file hashes, mutation outcome, and the
still-red pipeline consumer compile separately in receipt 04.

The subsequent post-recovery owner checkpoint passed sema 52/52 all-target
tests and strict all-target Clippy on the exact seven-file fingerprints in
`receipts/04.md`. This supersedes the earlier 21+43 lint-error statement as
current owner status, not the semantic RED inventory. The three private
future-only `expect(dead_code)` sites and function-local test panic expects
are enumerated in that receipt. Composition/EXISTS reachability and the
Group candidate/Scalar placement oracles above remain unimplemented, so
Ticket 04 is still `IMPLEMENTING`, not `FOCUSED_GREEN`.

The next scoped owner delta **does** close the Scalar placement item above:
`Decl total = sum(1)` was first observed to succeed with
`AggregateContext(position=Scalar, aggregation=Aggregate)`, then the central
constructor was changed to reject Scalar and OrderKey aggregate effects.
The exact-call-span declaration and nested OrderBy negatives pass; sema
all-target 54/54 and strict Clippy pass on the receipt 04 fingerprints.
Binder now records a Composition `FROM` use at the new Ticket 02 name span,
rejects a wrong-kind local without host fallback, and exposes an exact
`target_of_use(node, span, spelling)` for multi-use graph references with
duplicate-key rejects. Five new Binder oracles pass within 52/52 all-target
plus strict Clippy. This is **not** a Composition source semantic positive:
Sema still must consume the Binder target and sealed local producer, and
the Group candidate HIR-ancestry/tamper work remains open.

The checked-HIR candidate work is now owner-green as the next checkpoint:
Sema independently reconstructs direct Group→Project, exact reference/span,
and outer-to-inner Call-argument ancestry before accepting any Binder
candidate. Primitive tamper oracles cover wrong reference, span, Group,
outer/inner Call and dropped path; a source negative rejects an aggregate
Project separated from Group by Sort. Sema all-target tests pass 56/56 and
strict Clippy passes on the latest `receipts/04.md` hashes. This closes the
forgery/outer-call transplant and direct-stage items at owner scope, without
claiming a separate deliberate mutation build. Composition, Inline VIEW,
relational EXISTS, broad family coverage, pipeline integration, and removal
of the three future-only constructor expects remain RED.

## Final Execution Audit — `main@ecc38f59` + Dirty Overlay

Verdict: `IMPLEMENTING`. Binder 52/52 and Sema 56/56 all-target suites pass;
the combined owner strict Clippy lane exits 0. These are owner-local results,
not a complete nine-family semantic transaction.

`cargo fmt --all --check` is RED on Binder and Sema production/tests (and three
HIR/Surface test files), so the current behavior/lint checkpoint is not a
format-complete receipt. Reconcile the shared write sets, run rustfmt on the
owned files, and rerun tests/Clippy before recording the next hashes. Do not
bulk-format unrelated dirty owners merely to clear this gate.

Implemented; reuse without duplication:

- Binder sequential frames, capture barriers, key-only Group output, private
  partition input, exact Composition source-use resolution, exact per-use graph
  resolution, and duplicate-use rejection;
- Sema's one checked document transaction, closed authority, nine-variant item
  vocabulary, six-stage typed relation DTO, host-call proof consumption,
  central aggregate-placement rejection, Rule polarity, and independent
  checked-HIR Group candidate ancestry/tamper oracle;
- direct Query paths for Scan/Filter/Project/Join/Group/Sort, declaration and
  Rule typing, graph/check/presentation/catalog/meta paths already covered by
  current owner fixtures.

Open work must be executed in this order:

1. **Composition Binder/type ingress.** In `sema/src/document.rs`, call
   `Binder::resolve_composition`; traverse every producer and body in query
   type-input and expected-expression collection. Inline VIEW query bodies need
   the same Binder/type traversal. Key working-set outputs by exact composition
   item ID plus LET ordinal, not display name.
2. **Composition source materialization.** In the Source branch, inspect
   `bound.target_of_use(relation.id, source.name_span, &source.name)` before
   host authority. A `CompositionLet` target must load its prior sealed row;
   a wrong local kind rejects and an unbound name alone may try host lookup.
3. **Producer rows.** Query producers use their sealed relation output; Sample
   yields only `path: RepoPath`; PATH/REACHABILITY FIND yields only
   `function: EntityRef`; other FIND kinds have no row and reject relational
   use. Materialize Query/FIND/CHECK/EXPLAIN bodies without a second item tree.
4. **Inline VIEW.** Route `VIEW ... AS <query>` through the same query
   materializer and populate `ResolvedCatalogItemV2::inline_view`. Keep the
   query-ref XOR inline invariant and add wrong-session/foreign-output tests.
5. **Query relational EXISTS.** Extend query boolean type-input traversal and
   materialization to recurse into the nested relation, retain exact Binder
   captures, and emit `TypedExprShapeV2::RelationExists`. Preserve the existing
   rule-body `sema.invalid_item_shape` negative.
6. **Full family transaction.** Add one multi-item source document covering all
   nine root variants plus Composition subforms and Inline VIEW. Inject missing,
   duplicate, foreign-source, wrong-kind, and same-name/wrong-identity facts.
7. Remove the future-only constructor expects only after their positive source
   path executes; rerun Binder/Sema all-target tests and strict Clippy, then
   update `receipts/04.md` with current hashes and selected/executed counts.

Composition carrier correction found by the final audit:

- The current `ResolvedCompositionBindingV2 { item: SemanticItemV2 }` is not a
  valid closed producer contract. SAMPLE has no top-level `SemanticItemV2`
  variant, while unrelated Rule/Catalog/Meta items can be stored. Replace it
  with `ResolvedCompositionProducerV2::{Query, Sample, Graph}` matching the HIR
  enum exactly. Do not add SAMPLE as a new document-root item.
- Likewise replace the arbitrary `SemanticItemV2` body with
  `ResolvedCompositionBodyV2::{Query, Graph, Check, Explain}`. This preserves
  the grammar's legal subfamilies without a second syntax representation.
- Each binding retains the exact Binder-owned Composition identity
  `(source_id, composition_node_id, ordinal)`, checked name location, producer,
  and optional sealed relational output. `WITH` retains the resolved binding
  identity and use location, not only `with_name: String`; constructor checks
  exact membership and same document.
- Extend `ResolvedScanSourceV2` with a Composition-local variant keyed by that
  identity. Host Relation, catalog View, and Composition-local sources remain
  disjoint. Provenance and scan occurrence must retain the local identity.
- `resolve_graph_item` must accept the Binder result and use exact
  `(node, span, spelling)` resolution before graph authority. A local
  Composition target uses its producer contract; an unbound reference alone
  may use host graph authority. Wrong local producer kind must not fall back.
- Replace the host-only `ResolvedGraphReferenceV2.target: GraphReferenceIdV2`
  assumption with a closed target such as
  `ResolvedGraphTargetV2::{Host, Composition}`. Use the same disjoint target in
  CHECK/EXPLAIN/presentation references that are legal inside a Composition.
  Validate each local producer's capability for the authored role before
  construction; never mint a fake `GraphReferenceIdV2` from a LET name.

Required carrier negatives: Rule inserted as producer, Sample inserted as body,
same name/wrong ordinal, foreign composition ID, foreign source, WITH name with
wrong resolved target, producer/output kind mismatch, and local graph target
falling back to a host selector. These reject in Sema construction/assembly,
not later in Planning.

Required source regressions: forward/self Composition LET reject; local name
must not fall back to host; two graph refs in one node retain distinct targets;
duplicate exact refs reject; direct Group aggregate succeeds but a Sort between
Group and Project rejects; Scalar/GroupKey/Predicate/OrderKey aggregate calls
reject at the exact inner call span.

Stop condition: do not expose public unchecked constructors, remint Binder IDs,
or add a Sema-local resolver/type checker. Any missing owner fact is a hard
diagnostic, not a fallback walk.

## Final Independent RR — Carrier Checkpoint And Remaining Source Work

Implemented in the current dirty overlay, but not yet source-reachable:

- closed Composition producer/body families replaced arbitrary
  `SemanticItemV2` payloads;
- binding/body constructors compare expected nested HIR IDs, Binder
  owner/ordinal/kind, one exact WITH target, and same-source identity;
- non-row FIND cannot carry any output; PATH/REACHABILITY require the canonical
  field shape;
- Inline VIEW compares against its distinct expected nested Query ID, not the
  CatalogDecl ID;
- Sema all-target tests pass 59/59 (8+42+1+4+3+1) and strict all-target Clippy
  passes on the current dirty files; receipt 04 still records the prior
  56-test fingerprint.

This is a carrier checkpoint only. Execute these blockers in order:

Checkpoint hashes (dirty overlay, not receipt):

```text
sema/src/item.rs=cc1e53fa67cee10ffccd526e022d851924d7fbe4a1370e1c5778a3b436cfe71d
sema/src/names.rs=6e10412aaf4699c8e720cf81f03fa01feeddc6ae7d933e3a26ce6ce7b8e82a2d
sema/src/relation.rs=8c73d7616f7955030cc228a185ef4e91b744270c1a50d67c214c6aa96683ae7a
sema/src/lib.rs=3b6aa61d9af21eb50cb4c76e8a199a5f834403f329ba1eae3bb807f8c02736bd
sema/src/document.rs=7be4276861b5272d171a31202ea888bb0c8a60667339f146ede650dd52b82c25
```

Observed commands: `cargo test -p quanta-gqlang-v2-sema --all-targets`
selected/executed 59/59, exit 0; strict all-target Clippy exit 0;
`git diff --check` exit 0. `cargo fmt --all --check` remains exit 1 on 23
shared dirty files.

1. Add a private-field `ResolvedCompositionBindingIdV2` minted only from the
   exact Binder proof. Reuse it in Composition bindings, scan provenance,
   generated outputs, and graph targets. Delete public raw
   `Composition { source_id, owner, ordinal }` reminting.
2. Add a sealed generated-column origin for Composition producer output.
   SAMPLE `path` and PATH/REACHABILITY FIND `function` are not authored
   expressions and must not use fabricated expression keys or pretend to be a
   scan. Producer constructors mint canonical rows internally; callers do not
   supply arbitrary `RelationOutputContractV2` values.
3. Collapse the bare `ResolvedScanSourceV2` alias and unused raw-location
   `ResolvedRelationSourceV2` into one checked scan-use carrier: disjoint
   host Relation/catalog View/Composition identity plus exact checked authored
   source-name location. Preserve it in provenance and output origins.
4. Replace host-only graph references/uses with
   `ResolvedGraphTargetV2::{Host,Composition}`. Validate capability for FIND,
   SAMPLE scope, CHECK scope/pattern/nested FIND, and EXPLAIN. A bound local of
   the wrong capability fails without host fallback.
5. Add a checked item location to `ResolvedPresentationV2`; retain checked
   registry/session proof in catalog references instead of raw locations.
   Composition constructors compare the registry identity of every nested
   producer/body/use. Wrap public Sample/Graph/Explain payloads in private-field
   checked structs so public enum construction cannot bypass invariants.
6. Consume Ticket 03's deferred row/capture carriers in the single whole-
   document Types call. Wire Binder `target_of_use(node,name_span,spelling)`
   before host authority and seal each producer frame only after success.
7. Materialize Inline VIEW through the canonical relation path with the exact
   nested ID after Modules cycle/topological proof.
8. Materialize Query EXISTS once, preserving its exact relation root and all
   Binder capture edges. Keep Rule-body EXISTS rejected.
9. Tighten `IncompleteSemanticDocumentV2`: nonempty reasons where required,
   deterministic ordering, and duplicate rejection happen at the producer;
   Ticket 07 does not repair them later.

Required new tests:

- local graph target with same-named host selector, two local ordinals in one
  FIND, and each SAMPLE/CHECK/EXPLAIN local-target capability;
- same-source/different-registry producer/body/catalog proof transplants;
- wrong generated origin/provenance/effects/session, duplicate producer IDs,
  producer=root/body collisions, and exact source-name span;
- full `ResolvedCatalogItemV2::try_new` Inline VIEW positive/foreign-session
  negative, not only a helper comparison;
- public downstream compile-fail for raw Composition identity and unchecked
  Sample/Graph/Explain construction;
- Query EXISTS uncorrelated, correlated, multi-capture, multi-source, and every
  dropped/swapped/duplicate/foreign capture mutation;
- incomplete-analysis empty/duplicate/permutation mutations.

Only after source positives execute may future-only dead-code expectations be
removed. Acceptance remains full Binder/Sema all-target tests, strict Clippy,
owned-file format, deliberate mutation, nine-root source matrix, and refreshed
receipt 04 hashes/counts.

Resource-bound addendum: Ticket 04 authority and semantic producers add and
charge canonical `max_diagnostics` and `max_response_bytes` (or the equivalent
proved streaming-output bound) before accumulating output. Exact-limit/+1 and
diagnostic-amplification tests belong here and in Ticket 08; neither owner may
invent a private default.
## 2026-09-21 Final Current-Tree Audit — Partial Integration, No Closure

Status remains `RED_CAPTURED — IMPLEMENTING`. Current owner evidence is Binder
53/53 and Sema 59/59 all-target tests, both strict Clippy GREEN. This supersedes
the counts in receipt 04 only as a dirty-overlay checkpoint; it is not a receipt.

Implemented and reusable:

- Binder exposes exact Composition bindings by `(owner, ordinal)`, records
  exact source uses, and rejects self/forward LET names before host fallback.
- Sema's query type-input traversal consumes Binder targets and Types deferred
  rows for Composition Query producers.
- Query EXISTS type-input traversal consumes the exact Binder capture edges and
  passes the Types-neutral capture carrier into the one document kernel call.
- closed Composition producer/body DTOs, Inline View identity checks, and the
  prior Group/aggregate/host-predicate work remain owner-green.

The implemented carriers are not yet source-reachable end to end:

1. `resolve_document_bodies` still rejects every root Composition at
   `sema/src/document.rs:2069`; add a single canonical Composition materializer.
2. `materialize_relation` still has no Composition output environment and its
   Source branch remains host-oriented. Resolve exact Binder use first, then
   load the already-sealed prior producer row; only truly unbound names may use
   host authority.
3. Query EXISTS is typed but `materialize_bool` still rejects the variant at
   `sema/src/document.rs:1261`. Materialize its nested relation once and retain
   an owned nested-relation registry in `SemanticDocumentV2`; a root ID without
   the typed subtree is not sufficient.
4. Inline View still hard-rejects at `sema/src/document.rs:1984`. Consume the
   Ticket 03 topology proof and the same query materializer before Modules
   finalization.
5. Replace the public raw
   `ResolvedRelationSourceKindV2::Composition { source_id, owner, ordinal }`
   with a private-field ID minted from Binder proof. Collapse the unused raw
   `ResolvedRelationSourceV2` plus `ResolvedScanSourceV2` alias into one checked
   scan-use carrier with exact authored name location.
6. Add generated-column origins for SAMPLE/FIND; do not fabricate expression
   keys or scan fields.
7. Graph/CHECK/EXPLAIN local targets require a closed Host/Composition target
   and per-role capability validation. Presentation needs an item location;
   catalog references must retain validator/session authority.
8. After source positives exist, add one nine-root document, all Composition
   subforms, Inline View, correlated/uncorrelated/multi-source EXISTS, and every
   identity/session/capture mutation. Then remove only the now-obsolete
   `dead_code` expectations.

Stop conditions: no name-keyed working set, no per-LET type-kernel call, no
second resolver, no Sema-built `RecordTypeV1`, and no host fallback for a bound
wrong-kind local.

## SS-04A Query-Only Composition Checkpoint — 2026-09-21

Status remains `RED_CAPTURED — IMPLEMENTING`. The root Composition path is no
longer universally rejected: the Query-producer/Query-body slice is now
source-reachable through one canonical query materializer.

Implemented and independently rerun:

- producer LETs are visited in source order;
- exact Binder `composition_binding(owner, ordinal)` identities are consumed;
- sealed outputs are stored by `(composition item ID, ordinal)`, never name;
- `target_of_use(node, name_span, spelling)` is checked before host authority;
- a bound self/forward/wrong-kind local cannot fall back to a same-named host;
- each prior Types row fact is compared with the producer's sealed row;
- Query body and exact WITH target are retained in
  `ResolvedCompositionV2`;
- Sample/Graph producers and Graph/CHECK/EXPLAIN bodies fail explicitly until
  their authority carriers exist.

Evidence on the same dirty overlay:

```text
focused Composition tests: 4/4, exit 0
Sema all-target: 63/63 (8+4+42+1+4+3+1), exit 0
Sema strict all-target Clippy: exit 0
Binder all-target: 53/53, exit 0
Binder strict all-target Clippy: exit 0
owned package rustfmt check: exit 0
git diff --check: exit 0
forced-local-ordinal-zero mutation: sequential chain oracle failed, then recovery GREEN
```

RR disposition: this is a Query slice, not Composition closure. Execute the
remaining work in this order:

1. Replace publicly remintable raw Composition tuples with one private-field
   Binder-derived identity shared by binding, checked scan use, provenance,
   generated output and graph target. Collapse the duplicate scan-source APIs.
2. Add honest generated-column origins for Sample `path: RepoPath` and bounded
   PATH/REACHABILITY FIND `function: EntityRef`; producer constructors mint
   the complete row/provenance/effects contract. Do not fabricate expression
   IDs or accept arbitrary caller-supplied rows.
3. Add `Host | Composition` graph targets with per-role capability checks for
   FIND, Sample scope, CHECK scope/pattern/nested FIND and EXPLAIN. Bound local
   wrong-kind targets reject before host lookup.
4. Preserve checked registry/session identity on every nested producer/body,
   presentation location and catalog reference. Wrap publicly constructible
   Sample/Graph/Explain payloads in private-field checked carriers.
5. Integrate Inline View correctly: collect exact direct View edges, obtain the
   Modules topology proof, materialize inline queries in
   `dependency_first_order()`, then finalize the same validator/proof. The
   current `validate_view_topology(Vec::new())` plus hard reject is a stub.
6. Materialize Query relational EXISTS exactly once, retain nested typed
   relations in `SemanticDocumentV2`, emit correlation edges from the exact
   Binder capture set, and keep Rule-body relational EXISTS rejected.
7. Tighten incomplete-document diagnostics at the producer: nonempty reasons,
   duplicate rejection and canonical ordering. Ticket 07 must not repair this.
8. Add Surface -> HIR -> Sema fixtures for all Composition subforms, Inline
   View chain/diamond/cycle, uncorrelated/correlated/multi-source EXISTS, one
   mixed nine-root document, and every identity/session/capture mutation.

Do not remove the remaining correlation dead-code gate until the EXISTS source
positive executes. Query carrier reachability justified removal only of the two
Composition constructor gates.

## SS-04B Incomplete Analysis And Inline View Execution Boundary — 2026-09-21

The incomplete-analysis producer now rejects empty codes, foreign facts,
duplicate unresolved facts, foreign/Error diagnostics, and duplicate
diagnostics; accepted facts and diagnostics are canonically sorted. This is
Sema owner work, not a facade repair. Its unit oracle is included in the
current 64/64 Sema all-target run.

Inline View remains a P0 integration boundary. RR established that replacing
the existing reject branch alone is invalid. Implement one coherent slice:

1. While the Modules validator is `Predeclared`, Binder resolves every inline
   query and Sema creates a private View pre-resolution plan. It resolves every
   direct `FROM` use through the same validator and retains canonical direct
   dependency inputs; a local inline View is the source asset, while its target
   may be local or external.
2. Pass those inputs once to `validate_view_topology`, then materialize local
   inline query bodies only in `dependency_first_order()`. Cache output under
   the exact `CatalogAssetIdV2`, never a name. A local query-ref-only View has
   no relation row and rejects rather than falling back to a host relation.
3. Extend one document Types traversal and one relation materializer to consume
   the preplan/cache. Local scan rows are `PriorRelation`; external View rows
   come only from a Modules-owned typed View payload. Sema does not create a
   row type.
4. Retain canonical direct View use facts (including `family=None` edges and
   exact source-name location) in the sealed catalog semantic item so final
   semantic-edge equality agrees with the Modules topology proof.
5. Finalize the same validator with the same opaque topology proof. Test
   inline-host positive, reversed local chain, diamond plus independent View,
   cycle, duplicate direct use, local query-ref reject/no host fallback,
   foreign cache/Types-row mismatch, and Surface-to-HIR-to-Sema path.

Do not introduce another resolver, topology graph, type-kernel pass, or
name-keyed view output map.

## SS-04C Binder Inline-View Reachability Checkpoint — 2026-09-21

`Binder::resolve_catalog_decl` is now the only Binder entry point for catalog
payloads. It resolves only `VIEW ... Inline(Query)` through the canonical query
resolver; QueryRef and every non-View catalog payload remain outside Binder
authority. Sema and Pipeline now dispatch `CatalogDecl` through that entry
point, so an inline query's aliases and unresolved symbols are no longer
silently skipped before the Modules/Sema ownership boundary.

This is deliberately not Inline View semantic support: Sema still has the
empty topology proof and the explicit inline materialization reject. Do not
turn either into a compatibility fallback. SS-04B's predeclare -> direct-edge
plan -> topology proof -> dependency-first cache -> same-session finalization
sequence remains required.

Binder owner evidence on the shared dirty overlay:

```text
Binder all-target: 54/54 (0+12+10+19+3+4+6), exit 0
Binder focused catalog-inline-view: 3/3, exit 0
Binder strict all-target Clippy: exit 0
cargo fmt --all --check: exit 0
git diff --check: exit 0
```

The focused oracle covers query alias binding, unresolved projection symbol
with exact authored span, and non-inline no-op authority. Required next source
tests remain: full Sema inline-host positive, local chain/diamond/cycle,
query-ref no-host-fallback, validator-session/cache transplant, and complete
Surface -> HIR -> Sema path.

## SS-04D Inline View Dependency-Order RCA — 2026-09-21

Inline View remains RED for a second structural reason: Modules correctly
returns dependency-first order independently of authored item order, but Types
currently validates and materializes `PriorRelation` rows in authored HIR
traversal order. A reverse-authored acyclic View chain would therefore fail
before Sema can consume its inline cache. Removing the Types order guard is
not a repair; it would admit forged and cyclic deferred rows.

The coherent implementation order is now mandatory:

1. Sema collects direct View source uses through the single Modules session and
   validates topology once.
2. It derives a private, exact inline query-root pre-evaluation sequence from
   the opaque dependency-first proof.
3. Types validates source, root identity, uniqueness, and that every
   `PriorRelation` dependency is earlier in that supplied sequence; it prewalks
   those roots before normal document traversal.
4. Sema materializes/caches the same roots in that proof order, retains their
   canonical `family=None` dependency facts on the catalog semantic item, then
   finalizes that same Modules proof.

No Sema-side DFS, second type pass, relaxed raw ordering rule, or name-keyed
view cache is permitted. The required mutations are dropped/swapped/foreign
pre-evaluation roots, cache asset/root substitution, omitted `family=None`
semantic edge, and reverse-authored chain/diamond stability.

## SS-04E Query Relational EXISTS Semantic Registry — 2026-09-21

Query relational EXISTS is now source-reachable semantic IR. One
document-local registry maps the exact nested relation root to its sealed
`TypedRelationExprV2`; it is threaded through top-level queries and
Query-only Composition materialization. `SemanticDocumentV2::try_assemble`
requires exact equality between that registry's keys and every
`TypedExprShapeV2::RelationExists.relation_root`, plus source/registry/key
identity of each stored relation. A root-only placeholder cannot pass.

The semantic materializer consumes Binder's explicit capture set exactly as
recorded. Each edge must retain a matching resolved local reference, owner,
ordinal, canonical binding kind/name, and exact reference span before it is
sealed as `TypedCorrelationEdgeV2`. An empty Binder set is retained as an
explicit empty correlation vector. Rule-body relational EXISTS remains
rejected before semantic materialization.

Source-level owner oracles cover:

- correlated `EXISTS`: outer alias owner/ordinal, inner reference node/span,
  exact nested Filter -> Scan -> predicate tree, and exactly one registry root;
- uncorrelated `NOT EXISTS`: explicit empty capture vector and exact nested
  source root;
- existing Rule-body exact-span reject regression.

```text
Sema all-target: 66/66 (9+4+44+1+4+3+1), exit 0
Sema strict all-target Clippy: exit 0
cargo fmt --all --check: exit 0
git diff --check: exit 0
sema/src/document.rs=5b8d5234af13d3809dd0c7e715c277b77107324a754eab4c40a068c9574b5926
sema/src/expr.rs=36e961b2f12956e0398555a29b1f10aa18fd51a42f2a54776992a350f24e8952
sema/tests/v3_document_transaction.rs=adaf686416014dc4fc21afd893a8f4804c1303731bac9fea48df129471aa73d8
```

RR remains open for nested/multi-source/multi-capture EXISTS and private
registry wrong-key/extra/missing mutation oracles. This completes the Query
EXISTS materialization slice, not Ticket 04 or Inline View.

## SS-04F Inline View Same-Session Materialization — 2026-09-21

Inline View is no longer an empty-topology stub. The only implemented path is:

```text
predeclare -> exact View source probe -> Modules topology proof
-> Types validated pre-evaluation -> proof-order cache -> same-session finalization
```

Sema's private plan contains only exact source-use node IDs, Modules-owned
asset IDs, HIR relation roots, and `DocumentCatalogDependencyInputV2` facts. It
does not derive a second graph. `resolve_reference_if_present` is a Modules
owner API: an absent catalog name remains available for host-relation lookup;
wrong-kind and ambiguous catalog candidates still terminally reject. A source
that resolves to a local QueryRef-only View or an external View has no
Modules-owned typed row payload and rejects as `sema.unmaterialized_view_source`;
it cannot fall back to a same-named host relation.

After Modules returns its opaque `dependency_first_order`, Sema projects only
inline query body roots into `RelationPreEvaluationInputV2`, materializes each
View under exact `CatalogAssetIdV2`, and gives normal queries the fully sealed
cache. A View scan compares the Types row with the cached output row before
constructing `ResolvedScanSourceV2::View`. `ResolvedCatalogItemV2` retains
canonical direct View references; assembly includes them as `family=None`
edges and requires exact equality with the Modules-finalized dependency set.
Repeated direct uses are canonicalized by `(target asset, earliest use span)`
to match Modules deduplication.

Owner tests execute inline-host materialization, a reverse-authored local
chain, no-family direct edge retention, cycle rejection, and the optional
Modules probe's absent/local/wrong-kind matrix. This is an Inline View slice,
not Ticket 04 closure. Still open: diamond/independent-order and repeated-use
source tests, private cache/registry mutation tests, Surface-to-HIR-to-Sema
e2e, external typed View payloads, non-Query Composition, and the remaining
EXISTS mutation matrix.

## SS-04G Composition Source Identity Sealing — 2026-09-21

The public raw Composition source variant
`Composition { source_id, owner, ordinal }` is removed. It allowed any
downstream caller to re-create a relation-source identity from coordinates
that are only meaningful after Binder resolution. The replacement is one
private-field carrier:

```text
ResolvedValueBindingV2 (Binder-derived) -> ResolvedCompositionSourceIdV2
-> ResolvedRelationSourceKindV2::Composition(carrier)
```

`ResolvedCompositionSourceIdV2` exposes only inspect-only retained binding
data. Its field and sealing constructor are not public. The document
transaction constructs it only after the existing exact Binder local target
and `CompositionLet` kind checks; it retains the resolved name, owner node,
ordinal, and source identity. There is no tuple constructor, no compatibility
variant, and no name lookup/fallback added by this repair.

Negative rails are compile-time private-field construction and runtime
non-Composition-binding rejection. The external Composition query oracle now
checks the exposed retained Binder binding rather than manufacturing raw
coordinates. Source scans confirm zero raw struct-variant matches.

This closes only the Composition relation-source identity P0. It does not
materialize Sample/FIND producers, accept graph/CHECK/EXPLAIN Composition
bodies, synthesize their rows, or complete the Ticket 04 transaction.

## SS-04H Generated Composition Row Origin — 2026-09-21

SAMPLE and row-capable FIND must not claim an authored typed expression as the
origin of their closed frontend rows. `OutputColumnOriginV2` now has
`Generated(SemanticItemIdV2)`, which retains the exact nested producer item
instead of inventing a `SourceQualifiedNodeIdV2` expression fact. The shared
output constructor rejects a generated origin from another source.

`ResolvedCompositionProducerV2::try_sample` and `try_graph` now require all
of the following in addition to the closed one-field schema: empty source
provenance, `pure_scalar` effects, empty partition/order state, one aligned
column, and that column's exact generated producer identity. Cross-producer
schema transplants therefore reject before a later relation scan can consume
them.

This is the producer contract prerequisite, not non-Query reachability.
`materialize_query_composition` still rejects non-Query HIR forms until the
single Binder-first `Host | Composition` graph-target carrier and Types input
path land. Do not activate Sample/FIND with an expression-origin workaround.

## SS-04I Graph Target Domain Migration — 2026-09-21

Graph IR now distinguishes `ResolvedGraphTargetV2::Host` from opaque
`Composition`. Composition retains the existing Binder-derived source plus a
closed `Entity` or `Scope` capability; it is never coerced into a host catalog
selector. Existing host FIND, presentation, and CHECK callers construct the
Host branch through this domain. Owner mutation rejects Entity-as-Scope.

This is domain migration only: the mandatory Binder-first resolver is not
connected, so no local graph use or non-Query Composition form is enabled.
The next patch must use `target_of_use(owner, span, spelling)` before any host
lookup, sharing that resolver across FIND, SAMPLE scope, CHECK, and EXPLAIN.

## SS-04J Binder-First FIND Resolver — 2026-09-21

`resolve_graph_target_use` is now the single FIND/nested-CHECK-FIND resolver.
It charges the exact use once, checks the source registry once, consults
`BindResultV1::target_of_use(owner, span, spelling)` before host authority,
and treats any exact local target as terminal: it must be a prior sealed
Composition binding with the required capability or rejects. Only no Binder
target may consult the host catalog. Top-level callers supply an empty local
environment; Composition materialization must supply a prior-only map before
it enables any graph body or producer.

## SS-04K Graph Body Source Reachability — 2026-09-21

Composition now materializes an authored Graph body after all Query LETs have
been sealed. It derives a read-only map keyed by the exact Binder
owner/ordinal, passes it to the same Binder-first resolver, and retains the
exact Graph item ID in `ResolvedCompositionBodyV2::Graph`. A Query LET has no
graph capability, so a local graph use of it remains terminally rejected.

The source oracle constructs Composition HIR with a Query LET and a host-only
Graph body, then asserts the retained Graph body ID and exact WITH binding.
This is only Graph-body reachability. Graph/Sample producers, local Entity or
Scope positives, CHECK, and EXPLAIN remain separate work.

## SS-04L Canonical SAMPLE and FIND Producer Reachability — 2026-09-21

`materialize_query_composition` now admits all three closed producer families
through one ordered, Binder-proven assembly loop. Query and SAMPLE always
provide a relation output; FIND provides one only for `PATH` and
`REACHABILITY`. A non-row FIND stays sealed as `Graph { output: None }`, so a
later `FROM` use deterministically emits `sema.non_relation_composition_source`
instead of getting a fabricated row or falling back to a same-named host
relation.

The producer owns the generated row at the only materialization seam:

```text
SAMPLE        -> Set<{path: RepoPath}>       -> Generated(sample item ID)
FIND PATH/REACHABILITY
              -> Set<{function: EntityRef}>  -> Generated(graph item ID)
```

Both rows have empty provenance, pure scalar effects, and no partition/order
keys; `try_sample` / `try_graph` independently seal those conditions. Types
receives the same producer row under the actual consuming relation root, not a
synthetic HIR key. Consequently a Composition Query body can project `path`
from SAMPLE or `function` from PATH, but SAMPLE's `function` rejects at the
exact field span as `types.bad_projection`.

The prior-only local graph environment now has a source-positive Entity case:
a PATH LET is usable as the source of a later Graph body, resolves to the
opaque Composition target, and cannot mint a host selector. The negative
Flow-as-relation test remains: non-row FIND bindings never become relation
sources.

Owner evidence: `v3_composition_query` 8/8, Sema all-target 75/75, strict
all-target Clippy, package fmt, and scoped diff check green on the shared
overlay. This does **not** close local SAMPLE Scope, CHECK, EXPLAIN, same-name
host collision for local graph capability mismatch, Surface-to-HIR-to-Sema e2e,
or Ticket 04's full transaction/frozen-candidate gates.

## SS-04M Composition Scope and EXPLAIN Binder-First Closure — 2026-09-21

SAMPLE scope and Composition EXPLAIN now consume the same prior-only
Binder-derived target environment as FIND. A later SAMPLE can scope to a prior
SAMPLE (the only local `Scope` capability), and EXPLAIN can resolve a prior
PATH/REACHABILITY LET as an opaque local `Entity`. A local Binder target is
terminal: EXPLAIN does not probe Rule or host Entity namespaces in that case.
No host selector is minted from a Composition target.

Composition CHECK now passes its scope, pattern, and nested semantic FIND
through `resolve_check_item`, whose optional graph references were converted
to the shared resolver. Its rule reference remains a Modules validator
responsibility. To preserve the distinction between source-anchored document
diagnostics and source-less authority failures, the Composition materializer
now returns `FrontendTransactionErrorV2` rather than coercing validator
failures into fabricated document locations.

The current source oracle is 10/10 and Sema all-target is 77/77. It proves
prior SAMPLE Scope and local-Entity EXPLAIN with an empty host graph registry.
Required before CHECK may be marked closed: a source-positive local
scope/pattern/nested-FIND case with a Modules-proven Rule and the corresponding
capability-mismatch/same-name-host mutations. Surface-to-HIR integration and
Ticket 04 transaction closure remain open.

## SS-04N Composition CHECK Rule/Scope/Nested-FIND Oracle — 2026-09-21

CHECK is now source-proven across its three separate authorities without a
second resolver or a name-only test fixture:

```text
Catalog RuleBinding(rule1 -> external Check check1)  [Modules]
Composition LET scoped = SAMPLE                      [local Scope]
Composition LET paths  = FIND PATH                   [local Entity]
CHECK rule1 SCOPE scoped SEMANTIC_CONFIRM FIND source=paths
```

The oracle checks the sealed CHECK item ID, exact Modules `Rule` asset identity
instead of its spelling, the local-only Scope target, and the nested FIND's
local-only Entity target. A sibling negative changes only `SCOPE paths` and
must reject as `sema.invalid_composition_graph_target` at that authored use.
The authoritative host graph fixture now admits a same-named `scope` selector
with the correct Scope kind; a local PATH LET named `scope` remains terminally
wrong and rejects at its authored CHECK use. This proves local capability
failure cannot fall through to a matching host spelling.

`Pattern` is deliberately not given a synthetic positive: no Composition
producer owns the `Pattern` capability. A future positive requires a real
producer contract; a PATH result in CHECK pattern remains a mismatch case.
Owner evidence: composition source oracle 13/13; Sema all-target 80/80;
strict all-target Clippy, fmt, and scoped diff check green. Still open:
Surface→HIR→Sema Composition e2e, remaining transaction-wide mutations, and
the immutable-candidate/whole-workspace gates.

## SS-04P Surface → HIR → Sema Composition Front-Door Matrix — 2026-09-21

Sema now has an owner test dependency on Surface only in `dev-dependencies`;
the production semantic crate still depends on Binder/HIR/Modules/Types only.
The front-door matrix parses real public source through
`parse_source_document_v2`, passes the produced one-document HIR through the
sole integrity checker, then invokes the same semantic transaction used by
hosts. It covers:

```text
Query producer -> Query body
SAMPLE producer -> Query body
PATH FIND producer -> FIND body
SAMPLE producer -> CHECK body (with a parsed RuleBinding document item)
PATH FIND producer -> EXPLAIN body
```

Each vector asserts the retained closed semantic body variant; the dedicated
SAMPLE e2e additionally asserts the body row is the exact producer row. The
CHECK vector includes public `ruledef ... check-ref=...` source, proving the
catalog declaration and Composition share one parsed HIR document/session.
The fixture admits only the required host Entity/Scope selectors, so local
Composition targets remain observable rather than being hidden by broad host
fixtures.

Owner evidence: source suite 15/15, Sema all-target 82/82, strict all-target
Clippy, package fmt, and scoped diff check green. This closes the Composition
front-door gap, not mixed-nine-root, Inline View diamond/external-payload, or
whole-workspace qualification.

## SS-04Q Mixed All-Variant Public Transaction — 2026-09-21

One public source document now crosses Surface parse, checked HIR, Modules,
Binder, Types, and Sema with every `SemanticItemV2` family present:

```text
Declaration, Rule, Relation, Graph, Check, Presentation,
Catalog RuleBinding, Catalog certified Profile, Composition, Meta
```

The source has ten authored items because `CHECK` requires a `RuleBinding`
Catalog item while `require certified` needs the certified `Profile` catalog
contract to be exercised. These are two distinct Catalog payloads, not a
duplicated semantic variant. The Rule is type-anchored as `x = 1`; an initial
literal rule head was rejected by the real `sema.invalid_item_shape` rail and
an unanchored symbol head was rejected as `types.ambiguous_rule_type`. The
final fixture keeps both constraints visible and proves the positive only with
the valid public form.

The host fixture grants exactly `records.id`, graph Entities `a`/`b`, external
`check1`, and external `capability1`. The assertion counts every final semantic
variant and requires `(1, 1, 1, 1, 1, 1, 2, 1, 1)` in that order; it cannot
pass by parsing an item that fails to enter the sealed document.

Owner evidence: composition source suite 16/16, Sema all-target 83/83,
strict all-target Clippy, package fmt, and scoped diff check green. This closes
the mixed-root front-door oracle only. Inline View external-payload/diamond,
cross-session/cache/capture mutation matrix, frozen-candidate, and
whole-workspace qualification remain Ticket 04 gates.

## SS-04R Typed External View And Inline Diamond Closure — 2026-09-21

`ExternalCatalogPayloadV2::View(RelationTypeV1)` is now a usable closed
relation source. This is not a host-relation fallback: the source must first
resolve through the Modules View namespace, retain that exact external asset
ID, and consume its immutable row/cardinality contract. Types receives that
row as a known source; Sema rejects any fact that disagrees with the payload
row, then creates `ResolvedScanSourceV2::View` with source-field origins tied
to the exact scan stage. External Views remain frontend authority data; no
runtime/provider path is introduced.

Local inline Views still use only the Modules-owned topology proof. An
authored-order-reversed four-View diamond (`top -> {left,right} -> base`) now
proves that Sema materializes in `dependency_first_order()` without rebuilding
the graph. The oracle asserts direct dependency cardinalities (2, 1, 1), exact
base/left View asset targets in sealed relation stages, four final no-family
catalog edges, and the retained top Filter body. The pre-existing cycle and
repeat-use oracles remain separate rails.

Owner evidence: document transaction suite 49/49, Sema all-target 85/85,
strict all-target Clippy, package fmt, and scoped diff check green. This closes
the externally typed View and diamond source gaps. Remaining Ticket 04 gates:
cross-session/cache/capture mutation matrix, frozen candidate, and
whole-workspace qualification.

## SS-04S Multi-Reference Correlation Capture Oracle — 2026-09-21

The relational EXISTS owner suite now has a true multi-capture case. Its inner
filter compares both `f.id = r.id` and `f.value = r.value`; the same outer
alias is captured at two distinct authored reference nodes. The final
`TypedExprShapeV2::RelationExists` must retain two edges, each with the exact
outer owner/ordinal and its own inner reference node/span. It cannot collapse
captures to one binding-level summary or deduplicate by binding identity.

The nested relation registry is also checked by its exact inner root and scan
identity. Existing correlated-single, uncorrelated-empty, and Rule-body reject
tests retain their distinct contracts. Owner evidence: document transaction
50/50, Sema all-target 86/86, strict all-target Clippy, package fmt, and
scoped diff check green. Nested multi-EXISTS and private cache/session mutation
tests remain before Ticket 04 qualification.

## SS-04T Nested Relational EXISTS Boundary Oracle — 2026-09-21

A two-level correlated EXISTS now proves capture propagation across every
crossed boundary. The outer EXISTS sees its direct outer reference and the
deeper reference; the nested EXISTS sees only the deeper reference. Every edge
retains the same outer alias owner/ordinal/name, while the semantic document
retains two distinct nested relation roots. Root iteration is asserted in its
actual stable BTree identity order rather than an incidental authored order.

This prevents both failure directions: losing a transitive outer capture, and
incorrectly copying a direct outer capture into an inner EXISTS. Owner
evidence: document transaction 51/51, Sema all-target 87/87, strict all-target
Clippy, package fmt, and scoped diff check green. Private cache/session and
forged-input mutation rails, frozen candidate, and whole-workspace
qualification remain.

## SS-04U Sealed Downstream Tags — 2026-09-21

Ticket 05 audit found that an otherwise sealed Sema relation contract still
published three HIR-owned tags: `SortDirectionHirV1`,
`LogicalQuantifierKindV1`, and `RelationExistsPolarityV1`. That made removal
of Algebra's direct HIR dependency impossible without a consumer-side shim.

The Sema producer now converts those values at materialization into
`SortDirectionV2`, `LogicalQuantifierKindV2`, and
`RelationExistsPolarityV2`. `OrderKeyContractV2` and `TypedExprShapeV2` expose
only those semantic carriers. The conversion remains at the HIR-to-Sema
boundary; no downstream crate needs to import HIR to inspect a sealed relation
or EXISTS expression. Existing transaction assertions now consume the public
Sema types for order and EXISTS semantics.

Owner evidence: Sema all-target 87/87, strict all-target Clippy, package fmt,
and scoped diff check green. This is a Ticket 04 producer repair enabling the
Ticket 05 algebra cutover; cache/session/forged-input mutations, frozen
candidate, and whole-workspace qualification remain open.

## SS-04V Catalog Reference Session Retention — 2026-09-21

`ResolvedCatalogReferenceV2` previously copied the raw `LocatedSpanV2` out of
the Modules-owned reference proof. That lost the source-registry identity at
the Sema boundary, allowing later consumers to compare catalog references only
by source ID/span. The carrier now retains `CheckedLocatedSpanV2` directly and
exposes it unchanged. Document dependency comparison explicitly projects the
raw span only where it compares against Modules' final dependency ledger.

This is a producer repair, not a consumer-side session check. Inline View,
CHECK, EXPLAIN, and every catalog-family reference retain one checked location
whose registry identity can be compared against the document transaction.

Owner evidence: Sema all-target 87/87, strict all-target Clippy, package fmt,
and scoped diff check green. Frozen candidate and whole-workspace
qualification remain open.

## SS-04W Final Current-State Supersession — 2026-09-21

This section supersedes every older current-state/count statement in this
ticket. Older checkpoints remain historical evidence only. The audited tree is
`main@ecc38f59e9fdcd62b5e030445188ffa8b01e100a` plus a 178-entry shared dirty
overlay; no clean-candidate or workspace qualification is implied.

### Current disposition

| Area | Disposition | Current evidence / gap |
|---|---|---|
| Atomic Sema document transaction | `IMPLEMENTED, OWNER-GREEN` | one checked document owns item order, exact item set, typed expressions, nested relations, catalog proof, and stable diagnostics |
| Source item reachability | `IMPLEMENTED, OWNER-GREEN` | mixed transaction and 16-case Composition suite reach all current semantic item/body variants |
| Six relation stages | `IMPLEMENTED, OWNER-GREEN` | Scan/Filter/Project/Join/Group/Sort are private-constructor typed stages with checked output contracts |
| Group authority | `IMPLEMENTED, OWNER-GREEN` | key-only public Group row, Binder-owned partition input, Set cardinality, exact key IDs, and intervening-Sort rejection |
| Query relational EXISTS | `IMPLEMENTED, OWNER-GREEN` | nested relation registry, EXISTS/NOT EXISTS polarity, empty/single/multiple/nested capture boundaries, and Rule-body rejection |
| Composition/View/catalog session identity | `IMPLEMENTED, OWNER-GREEN` | Binder-first Composition source identity, external typed View, dependency-first inline View diamond, and checked catalog locations |
| Aggregate-to-relation contract | `PARTIAL — BLOCKING` | explicit Group legality exists, but aggregate Project materialization does not seal the final Group/Aggregate contract required by Algebra |
| Downstream resource-budget receipt | `MISSING — BLOCKING 05/06` | the transaction consumes `FrontendBudgetV2`, but `SemanticDocumentV2` retains no immutable algebra/rule-graph budget projection |
| Package qualification | `GREEN FOR CURRENT OVERLAY` | fresh 87/87 all-target tests and strict all-target Clippy passed |
| Workspace/product qualification | `NOT PROVEN` | Logic/Pipeline consumers are RED; architecture and clean-SHA/hosted-CI gates remain open |

### Aggregate producer RCA

The remaining Ticket 04 code defect is localized but semantic:

1. `materialize_relation` materializes every Project the same way. The output
   copies `previous.cardinality()` and resets partition and ordering to empty.
   It does not branch on the sealed aggregate effect.
2. A grouped aggregate therefore has a real Group child, but its Project output
   drops the Group partition keys. Ticket 05 cannot reconstruct them from the
   aggregate spelling or the prior row.
3. A scalar aggregate Project over Scan/Filter has no empty-partition Group at
   all and inherits Bag/Sequence cardinality. Ticket 05 correctly rejects that
   shape with `algebra.aggregate_without_group`.
4. `TypedRelationExprV2::try_group` currently requires nonempty key
   expressions, so scalar aggregation cannot be repaired by calling the
   existing explicit-Group constructor with an empty vector. The semantic
   carrier must distinguish an authored nonempty Group from a producer-sealed
   scalar aggregate partition without exposing a forgeable public constructor.

Do not repair this in Algebra. The producer cut is:

1. Inspect the complete projection expression vector once using each
   expression's `AggregateContextV2`; reject a mixed/illegal shape before
   constructing any relation stage.
2. For an explicit Group child, preserve its exact partition-key IDs and
   partition-input proof into the aggregate-ready output. Do not synthesize a
   second Group.
3. For a legal scalar aggregate, materialize one private empty-partition Group
   owned by Sema, with Set cardinality, no public keys, and the exact pre-fold
   input contract. The aggregate Project must directly consume it.
4. Preserve the resolved call target and `TypedCallArgumentV2::Star`; only the
   Sema/types legality owner may admit `count(*)`. `sum(*)`, `min(*)`,
   `max(*)`, mixed star/argument calls, and bare star must fail before Algebra.
5. Seal the aggregate output cardinality, partition identity, row-aligned
   origins, effects, and checked locations in one constructor. Algebra may only
   verify and copy this contract.

The same producer cut must retain a read-only downstream budget receipt on the
sealed document. Today `FrontendBudgetV2` is consumed while declaring and
materializing the document, but the final `SemanticDocumentV2` stores only
source/items/expressions/nested relations/catalog. Ticket 05 and Ticket 06
therefore cannot implement exact-limit/+1 traversal without accepting a new
caller-selected budget or inventing a default. Retain one immutable projection
covering expression/relation walk limits; rule nodes/edges/steps; graph witness
depth/count; Planning traversal; source/frame bytes; diagnostic count; and
response bytes. The canonical positive caps include `max_diagnostics` and
`max_response_bytes`, so Ticket 08 consumes rather than amends the authority.
Consumers read the receipt from the document; no consumer accepts an
independent budget argument.

### Exact remaining owner work

Add or strengthen these Sema-owner oracles before SS-04W can advance:

- scalar `count(*)`: exact `Group(empty) -> Project(aggregate)` shape, Set
  cardinality, empty partition keys, star retained, and no caller option;
- grouped `sum(field)`: exactly one authored Group, exact Binder partition
  input, aggregate output retains the Group partition identity;
- multiple aggregates and group-key-plus-aggregate projection: stable authored
  expression order and exact row-aligned expression origins;
- negative star matrix: `sum(*)`, `min(*)`, `max(*)`, `count(*, x)`, and bare
  `*`, each at the exact authored call/star span;
- forged aggregate mutation matrix: missing/extra/swapped partition keys,
  wrong Group child, wrong scan occurrence, transplanted expression ID,
  cardinality downgrade, and foreign registry identity;
- exact-limit/+1 semantic-step cases covering the implicit Group path;
- budget-receipt identity/value mutation: dropped projection, zero dimension,
  value transplant from another authority profile, and consumer exact/+1,
  including diagnostic-count and response-byte boundaries;
- finalizer mutations for dropped/extra nested root and dropped/duplicate/
  swapped correlation reference, including a cross-registry checked location.

Existing Composition, View, catalog, and nested-EXISTS positives are complete
for owner-local scope; do not recreate them under new test files. Only add a
case when it kills one of the mutations above.

### Next SS/RR acceptance

SS-04W is accepted only when the aggregate producer change and its owner
mutations are present and all commands below pass against one unchanged source
fingerprint:

```bash
cargo test -p quanta-gqlang-v2-sema --all-targets
cargo clippy -p quanta-gqlang-v2-sema --all-targets -- -D warnings
cargo fmt --check -p quanta-gqlang-v2-sema
git diff --check -- crates/quanta-gqlang-v2-sema \
  docs/plans/sep-21-gqlang-sota-frontend-reconstruction/tickets/04-sema-kernel-and-document-transaction.md \
  docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/04.md
```

RR-04W then inspects the concrete relation tree and output contract; a passing
test count alone is insufficient. Workspace, architecture, hosted CI, and
clean-candidate closure remain Ticket 09 evidence, not Ticket 04 GREEN.
