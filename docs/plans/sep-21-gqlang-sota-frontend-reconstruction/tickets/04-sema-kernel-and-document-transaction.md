# 04 — Semantic Kernel And Document Transaction

Status: `PLANNED`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Closed

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
ResolvedGraphV2::visible_bindings/current module/external catalog
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
7. `ResolvedCallV2`
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
  -> Result<SemanticDocumentV2, FrontendFailureV2>
```

These are sema-owned APIs and are never exported by the frontend facade. The
constructor consumes the HIR-owned document containing its sole source-length
authority, invokes the one integrity checker over every root with one
document-wide ID set/source bound, and yields an owned proof carrier; callers
cannot construct it by fields or supply a second source length.

Source entry points are pipeline-owned because pipeline may depend on both
surface and sema:

```text
resolve_source_to_semantics_v2(source: &str,
  authority: &ClosedFrontendAuthorityV2)
  -> Result<SemanticDocumentV2, FrontendFailureV2>
analyze_source_to_semantics_v2(source: &str,
  authority: &OpenAnalysisAuthorityV2) -> SemanticAnalysisV2
```

Pipeline parses to `ParsedDocumentV2`, extracts its owned `HirDocumentV1`, and
immediately delegates to the checked-document transaction. Sema never depends
on surface. Architecture fixtures and compile checks pin this direction.

`SemanticAnalysisV2` is a closed state enum:

```text
Complete(SemanticDocumentV2)
Incomplete(IncompleteSemanticDocumentV2)
Error(FrontendFailureV2)
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
ClosedFrontendAuthorityV2::try_new(resolved_imports, relation_catalog,
  external_catalog, validated_pack_authority, budget)
OpenAnalysisAuthorityV2::standard_v2()
OpenAnalysisAuthorityV2::try_new(partial_authority, budget)
```

`standard_v2()` contains the builtin/operator registry and
`FrontendBudgetV2::standard_v2()` but no invented external relation, module,
catalog, pack, or lock authority.

`resolved_imports` owns current-module identity. The constructor rejects no
cross-field identity by convention because no duplicate current-module field
exists. `validated_pack_authority` is the ticket 03 bundle that proves its lock
set was verified against its exact validated pack graph.

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
4. Implement typed reference resolution with expected/actual kind diagnostics.
5. Convert Binder output to stable resolved IDs and explicit capture edges.
6. Build node-ID-keyed type inputs from Binder/module resolution, invoke the
   types-owned kernel once, and assemble typed expressions/resolved calls/effect
   summaries from its facts.
7. Implement incremental relation frames and output contracts.
8. Implement item semantic contracts for FIND/CHECK/presentation/catalog/meta.
9. Implement the document typestate phases in the fixed order.
10. Compile inline VIEW bodies during the same body-compilation phase as queries.
11. Assemble all source rules before global logic validation.
12. Run catalog cycle/certification checks once after all bodies and metadata.
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
    rg -n 'compile_(program_structural|program_with_schema|with_context|item_structural|item_with_schema|item_with_context|document_structural|document_with_schema|document_with_context)' crates README.md docs --glob '*.rs' --glob '*.md' | sort
    ```

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
max_rule_graph_nodes
max_rule_graph_edges
max_rule_steps
max_witness_depth
max_witnesses
```

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
cargo test -p quanta-gqlang-v2-pipeline --test v3_document_semantics_v2
cargo test -p quanta-gqlang-v2-sema --test v3_document_transaction
cargo test -p quanta-gqlang-v2-sema --test v3_semantic_budget
cargo test -p quanta-gqlang-v2-pipeline --test v3_legacy_api_ratchet
cargo clippy -p quanta-gqlang-v2-sema -p quanta-gqlang-v2-binder -p quanta-gqlang-v2-pipeline --all-targets -- -D warnings
python3 tools/architecture/check_architecture.py
git diff --check
```

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
6. forward, unknown, wrong-kind, and ambiguous reference tests
7. REQUIRE/declaration order permutation test
8. document atomicity test
9. source and prechecked-syntax entrypoint parity
10. exact-limit and one-over-limit semantic document/expression budget tests
11. aggregate placement/nesting/star context matrix and rule
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

Requires 02 and 03. Blocks 05-09.
