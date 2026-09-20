# Current-Head Implementation Reuse Audit

Status: `AUDITED — REQUIRED INPUT TO TICKETS 01-09`

Base: `main@e896a11024bd5a1e1ef03c094a70032f4e6c765f`

This is the anti-duplication ledger. The base has 822 passing Rust tests and
substantial owner algorithms. The reconstruction changes authority, carriers,
and wiring; it does not authorize parallel implementations of working kernels.

## Classification Rules

| Class | Required action |
|---|---|
| `KEEP` | retain implementation and strengthen tests around it |
| `MIGRATE` | change the canonical implementation's carrier/API in place; one active algorithm remains |
| `DELETE` | remove obsolete authority, representation, or test after its replacement is wired |
| `NEW` | add only when no current owner implementation satisfies the contract |

Every ticket receipt must list touched assets under these four classes. A new
file with the same responsibility as a `KEEP` or `MIGRATE` asset is a stop
condition.

## Current Algorithm Census

| Owner | Existing current-head asset | Class | Required disposition |
|---|---|---|---|
| HIR | `HirBuilderV1`, monotonic IDs/spans, visitors, `CheckedHirV1::check_tree` | `MIGRATE` | extend the one tree/builder/visitor/integrity stack for typed relation variants; do not add another syntax tree or checker |
| Surface | precedence parser, document parser, formatter framework, property generators | `MIGRATE` | preserve parser kernels; replace lossy relation/string/format paths and add budgets |
| Types | `TypeCheckerV1`, closed operator/builtin registries and builtin effect metadata | `MIGRATE` | retain the recursive checker and registries as the only type/effect algorithms; emit resolved typed results and checked value carriers |
| Modules | module resolution, ambiguity/cycle detection, visibility projection behind `ResolvedGraphV1::binder_view`, catalog resolution, pack validation, lock verification | `MIGRATE` | preserve algorithms; make mutation fallible, add typed IDs/spans/proof wrappers, and expose a modules-owned typed visibility view that sema adapts for Binder |
| Binder | frame stack, capture barrier, shadow reporting, rule/query/composition walkers | `MIGRATE` | preserve lexical algorithms; make relation frames incremental and outputs ID-based |
| Pipeline | stage diagnostics and orchestration shell | `MIGRATE` | retain orchestration/diagnostic projection only; delete local semantic walkers and item-at-a-time authority mutation |
| Algebra | 12 operators, provenance/source identity, normalization helpers, aggregate-star carrier | `MIGRATE` | keep one logical algebra; accept sealed semantic input and enforce constructors in release builds |
| Logic | rule literal model, Tarjan SCC, stratification, graph/witness contracts, recursive aggregate matrix | `MIGRATE` | preserve algorithms; replace name/one-rule producers with document-wide resolved-ID inputs and bounded traversal |
| Planning | capability/cost/effect/result vocabularies, capability traversal, `produce_plan` validation | `MIGRATE` | preserve valid vocabulary/traversals; replace raw-HIR classification and hardcoded metadata |
| Transport | stdio loop, deterministic response rendering, byte-fragment tests | `MIGRATE` | keep the loop/testing techniques; replace object-concatenation framing and handwritten JSON request parser with bounded NDJSON |
| Facade | none suitable: current crate is a glob-export surface | `DELETE` + `NEW` | delete globs/V1 seams and add the curated document-only facade |
| Sema | no document-scoped typed semantic transaction exists | `NEW` | add typestate carriers and orchestration only; delegate lexical/type/module algorithms to their owners |

The Ticket 04 owner/consumer RR sharpened the `NEW` boundary: sema also owns
only the checked, immutable host graph-selector catalog carrier because no
existing module asset ID can represent the parser's dotted/slash/`::` graph
paths. It does **not** add a graph runtime or duplicate modules catalog
resolution. Types keeps the one expression/overload implementation and adds
bounded rule-variable constraints plus declaration-only forward dependency;
modules keeps the one catalog algorithm and adds same-session predeclaration
and all-family typed references. A `TypedExprV2` with no closed structure or a
sema relation enum copied from all algebra operators is not reuse: it forces
downstream raw-HIR reconstruction or fabricates unreachable source syntax.

## Confirmed Deletions

1. `CompositionOpV1` and synthetic `_keys` relation sources.
2. outer-Filter encoding of JOIN `on`.
3. pipeline `env_closed`, `type_check_bool`, item-local catalog mutation,
   `lower_source_rule`, callee-name dependency heuristics, and raw-HIR output
   reconstruction.
4. raw-HIR planning classifiers and facade glob exports.
5. object-concatenation framing, whole-buffer-clear recovery, handwritten
   request JSON scanner, and every LSP identity claim.
6. all frozen V1 public compile seams at the ticket 08 atomic cut.

## Source-Reachability Correction

The original plan draft accidentally proposed a source `UnionHirV2` without a
grammar production and mentioned explicit CROSS only in algebra tests. The
correct contract is:

1. add explicit `cross join` syntax in ticket 02;
2. require `on` for ordinary `join`;
3. do not add source UNION/LIMIT/HAVING/semi/anti syntax;
4. retain checked programmatic-only algebra operators without advertising
   source reachability.

Relational existence had the same producer gap. The pinned source forms are
`exists (from ...)` and `not exists (from ...)`; logical binder quantifiers are
rule-only and require at least one binder plus `in`. This makes the current
semi/anti lowering reachable without treating the old placeholder quantifier as
a subquery.

## Existing-Test Disposition

The current suite is input material, not a second suite to leave beside V2.
After the final test-delta audit, tickets mention 91 unique test/support paths;
that routing universe includes current paths, both sides of tracked moves,
optional owner-local support modules, temporary `v3_legacy_api_ratchet.rs`, and
genuine new boundary files. It is not a net-new-file count. The earlier
73-path/47-update/8-move/18-new estimate is superseded.

Ticket 09 generates the exact final path inventory from the landed diff and
classifies every row as retained, moved, added, deleted, support-only, or
temporary-deleted. A support module is not a test target. No worker may infer
permission for a new test file merely from this aggregate count; each new path
must still appear in its owner ticket.

| Lane | Existing tests to retain/migrate | Required consolidation |
|---|---|---|
| 02 | HIR identity/visitor/hellgate; surface query/format/expr/proptest | update four declared existing files; move overlapping escape/precedence assertions from `v3_format_edges.rs`, `v3_format_hellgate.rs`, and `v3_mutation_rails.rs` into the new semantic oracles, then delete obsolete V1-only assertions |
| 03 | all three types suites; six module/catalog/pack suites | preserve registry, resolution, cycle, version, lock, and signature positives; add only missing duplicate/kind/span/proof/budget negatives |
| 04 | four binder suites; pipeline alias/context/diagnostic/document tests | keep binder capture/shadow tests; rewrite pipeline tests against the document seam and delete tests whose only contract is a removed V1 helper |
| 05 | four algebra suites plus group/order pipeline tests | migrate normalization/operator/provenance tests to checked constructors; replace `_keys` and raw-HIR expectations rather than duplicating them |
| 06 | eight logic suites | keep Tarjan/stratification/witness tests; add source-document reachability, exact edge polarity/span, and budget tests only |
| 07 | seven planning suites and facade `downstream_contract.rs` | migrate vocabulary/cost/capability tests; replace raw-HIR item and glob-export assertions with curated positive tests plus `compile_fail` facade doctests, do not retain both public contracts |
| 08 | nine old `lsp` transport tests and CLI parity/front-door tests | tracked-move and rewrite the byte cases for NDJSON; delete object-concatenation expectations and old names in the same cut |
| 09 | three conformance tests and pipeline `audit_18_families.rs` | upgrade/rename into one semantic registry; routing-only checks may remain only under an explicit smoke name and cannot satisfy conformance rows |

## Cross-Ticket Omissions Closed By This Audit

1. `AggregateContextV2` and aggregate placement are produced by ticket 04
   sema, then consumed read-only by ticket 05. Ticket 05 must not edit sema.
2. Predicate IDs and predeclaration are produced by ticket 04; ticket 06 only
   validates and lowers the complete program.
3. CLI and diagnostics stdio both use open document analysis. Only embedding
   hosts with closed authority call compilation.
4. No source HIR variant is added merely because an algebra variant exists.
5. Ticket 08 must migrate old tests by tracked move/update; copying them into a
   renamed crate and retaining old protocol assertions is forbidden.
6. Ticket 04 stops at `SemanticDocumentV2`. Ticket 07 is the first owner of
   final `CompiledDocumentV2`/`AnalysisDocumentV2` assembly, after algebra and
   logic exist; no partial compiled stub is permitted in the cutover stack.

## Receipt Addition

Add these fields to every ticket 01-09 receipt:

```text
kept_assets=
migrated_assets=
deleted_assets=
new_assets=
existing_tests_migrated=
obsolete_tests_deleted=
duplicate_owner_scan=
```

`duplicate_owner_scan` records the exact `rg`/dependency/API inventory used to
show that one canonical implementation remains.
