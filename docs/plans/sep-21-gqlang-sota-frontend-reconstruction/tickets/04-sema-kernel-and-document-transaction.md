# 04 — Semantic Kernel And Document Transaction

Status: `RED_CAPTURED — IMPLEMENTING`

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
is still RED; Ticket 05 may not yet consume Group as a complete sealed stage.
