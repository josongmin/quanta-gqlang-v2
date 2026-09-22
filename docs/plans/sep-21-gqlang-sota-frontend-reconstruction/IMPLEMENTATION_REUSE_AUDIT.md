# Current-Head Implementation Reuse Audit

Status: `AUDITED — REQUIRED INPUT TO TICKETS 01-09`

## Current Reuse Supersession — 2026-09-21

This section overrides stale `no Sema owner` and `raw-HIR Algebra normalizer`
statements below. They remain as historical justification only.

| Owner | Current disposition | Do not duplicate | Remaining migration |
|---|---|---|---|
| Sema | `KEEP + MIGRATE`; the document transaction and sealed item/relation/expression carriers now exist | no second semantic document, catalog/session wrapper, Composition identity, graph target, generated origin, nested relation registry, budget authority, or test builder authority | repair implicit global aggregate Group producer; seal the downstream budget receipt; close remaining discriminating transplant/public-boundary mutations |
| Algebra | SS-05B migration implemented | do not restore `normalize_query`, `normalize_relation`, `NormalizeOptionsV1`, public V1 nodes, caller budget/default, or HIR/Binder dependencies | remove consumed EXISTS conjunct after Semi/Anti lowering; make traversal iterative/budgeted; extend operator/mutation matrix |
| Logic | `MIGRATE + DELETE` | keep one Tarjan/stratifier/graph kernel; do not create another EXISTS-to-Algebra path | consume Sema document, use iterative budgeted SCC, delete raw/name/V1 adapters and `source.rs` lowerer |
| Pipeline | `MIGRATE + DELETE` | no V1 compatibility imports or local semantic walkers | assemble one compiled document from Sema + Algebra + Logic, then project diagnostics |
| Planning | `MIGRATE` | keep capability/cost/effect vocabulary; no physical optimizer or raw-HIR classifier | iterative traversals over sealed contracts and valid-by-construction result variants |
| Facade | `DELETE + NEW` remains | no glob or internal-crate mirror API | one explicit allowlist plus external compile-fail for forbidden symbols |
| Frontdoors | `MIGRATE` | no second compiler pipeline or fake LSP identity | bounded wire DTO/framing/diagnostics/argv over facade only |

The current legacy Algebra V1 scan reports a broad downstream blast radius
(173 references at the latest audit point). That number is routing evidence, not 173
independent fixes: remove the producer API once, then migrate Logic, Pipeline,
Planning, facade, and their tests in ticket order.

The feature-gated Sema reference transaction used by Algebra tests is the
canonical test fixture. It must remain incapable of minting unchecked
production authority and must not become a second public builder.

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
2. Types owns source/host Predicate ID forms and intrinsic builtin-overload
   effects; modules owns catalog predeclaration and same-session host proof
   (Ticket 03). Sema seals each authored call and the complete Rule member/
   dependency set (Ticket 04); ticket 06 only validates and lowers that
   complete program. No second lookup or polarity classifier belongs in
   logic.
3. CLI and diagnostics stdio both use open document analysis. Only embedding
   hosts with closed authority call compilation.
4. No source HIR variant is added merely because an algebra variant exists.
5. Ticket 08 must migrate old tests by tracked move/update; copying them into a
   renamed crate and retaining old protocol assertions is forbidden.
6. Ticket 04 stops at `SemanticDocumentV2`. Ticket 07 is the first owner of
   final `CompiledDocumentV2`/`AnalysisDocumentV2` assembly, after algebra and
   logic exist; no partial compiled stub is permitted in the cutover stack.
7. Composition's V1 pipeline already contains a useful exact producer-row
   contract in `compile.rs:859-934`: Sample is
   `Set<{path: RepoPath}>`, bounded PATH/REACHABILITY FIND is
   `Set<{function: EntityRef}>`, and other FIND kinds have no relation row.
   Migrate those semantics and `v3_composition_type_env.rs` cases to the one
   typed local-output producer in Ticket 04; do not keep the V1 raw-HIR
   `TypeCheckerV1` helper or create another row inference in planning.
   Its schema-open empty-projection success is an unsound V1 behavior and
   becomes a closed V2 reject, not a compatibility fixture.
8. Group's key-only public output, aggregate-only partition input, and Sort
   NULLS LAST are sema contracts. Algebra and planning consume them without
   resetting a default. `BuiltinRegistryV2::effects_of_resolved` is the one
   exact-ID intrinsic-effect source for both Group access and Rule edge
   polarity; composed expression effects cannot classify an ancestor call.
9. Composition's existing parser/HIR LET/body variants and Binder sequential
   frame are not rewritten. Ticket 02 adds only the missing `FROM` name-token
   span to the existing Source carrier. Ticket 04 adds one Binder source-use
   proof and an exact `(node, span, spelling)` accessor for multi-use graph
   references, then seals local-vs-host source/graph identity in one semantic
   document transaction. `target_of(node)` is not a valid graph multi-use
   oracle. No fake host asset, second lexical resolver, or nested Composition
   grammar branch is introduced to make the tests pass.
10. Group candidate authorization must compare the Binder side-table
    witness to independently walked checked-HIR call ancestry and the exact
    Project/Group stage route. Reusing types-owned intrinsic effects is
    necessary but does not prove the call stack was not transplanted. Ticket
    04 owns this cross-owner invariant and mutation; Ticket 05 consumes only
    the sealed partition result.

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

## Final Duplicate-Owner Audit — `main@ecc38f59` + Dirty Overlay

| Concern | Canonical owner to reuse | Legacy seam to remove, not copy |
|---|---|---|
| syntax and byte spans | Surface + HIR integrity | consumers must use `name_span` |
| lexical scope/capture | Binder | Sema/planning name re-resolution |
| expression/rule typing | Types | Pipeline V1 `TypeCheckerV1` path |
| module/catalog authority | Modules | Pipeline V1 `CatalogV1`/`PackGraphV1` path |
| document semantic truth | Sema | caller-fabricated planning facts |
| relational lowering | Algebra | Logic EXISTS lowering and raw-HIR normalizer |
| SCC/stratification/graph bounds | Logic | raw-name dependency rebuild |
| plan metadata | Planning | raw-HIR item classifiers |
| public assembly | Pipeline + curated frontend-contract | facade glob re-exports |
| executable transport | diagnostics-stdio in place | LSP names and cross-package parity test |

Concrete scan results:

- Algebra still exposes raw `QueryHirV1`/`RelationExprHirV1` normalization and
  its four suites fail on removed builders. Migrate the owner once; do not add
  adapters or restore builders.
- Logic still imports raw `RuleHirV1`/`GraphQueryHirV1` and owns
  `ExistenceCheckV1` over Algebra. Ticket 06 must leave one Sema/Types-facing
  owner and zero Logic-to-Algebra edge.
- Planning still exports `PlanRequestV1`, `PublicPlanV1`, and raw-HIR
  classifiers. Ticket 07 replaces that input; it does not wrap it.
- Frontend-contract glob-re-exports HIR, Algebra, Planning, and Types. Ticket 07
  replaces globs with curated document/facade exports.
- Diagnostics still uses `LspServerV1`/`StdioLspErrorV1` and Pipeline V1.
  Ticket 08 renames and rewires the same crate; no new transport crate.

Binder's Group candidate and Sema's independent checked-HIR witness are not
duplicate checkers: they form a producer/consumer invariant. Types intrinsic
effects and Sema placement policy are likewise distinct owner facts.

## Final RR Duplicate-Prevention Addendum

| Missing contract | Build once in | Consumers that must not recreate it |
|---|---|---|
| deferred multi-field Composition Query row | Types | Sema, Algebra, Pipeline |
| exact neutral RelationExists root/captures | Types input contract + Binder facts sealed by Sema | Algebra, Logic |
| Composition binding identity | Binder proof wrapped by Sema private-field ID | scan provenance, graph references, Planning |
| generated SAMPLE/FIND column origin | Sema producer constructor | Algebra, Planning |
| View cycle/topological proof | one Modules validator session | Sema, Pipeline |
| algebra traversal order | one iterative Planning fold | capability, explain, cardinality helpers |
| frontend public API allowlist | Ticket 07 external API gate | Ticket 09 only aggregates evidence |
| transport/request/response bounds | shared `FrontendBudgetV2` + Ticket 08 codec | CLI/stdio project, never redefine |
| 18-root semantic manifest | one conformance manifest | README/index/Rust tests load or generate |

Delete or avoid bare public Composition identity tuples, caller-supplied
canonical producer rows, opaque capture strings, Algebra/Logic capture
reconstruction, three recursive Planning walkers, hardcoded Rust example tables
beside TSV, and inventories derived from a clean tree's empty `git diff`.

## 2026-09-21 Final Symbol/Consumer Scan

The final scan used:

```text
rg -n "ExpressionTypeKernelV2|DeferredRelationRow(Input|Kind)V2|ResolvedRelationPredicateInputV2|DocumentCatalogValidatorV2|TypeCheckerV1|CatalogV1|PackGraphV1|materialize_relation|Tarjan" crates --glob '*.rs'
```

Result and disposition:

- exactly one live V2 document type kernel is defined in Types, with one Sema
  `check_document` call site;
- exactly one deferred-row carrier family and one exact relation-predicate
  carrier family are defined in Types;
- exactly one document catalog validator is defined in Modules;
- exactly one semantic relation materializer is defined in Sema;
- Logic contains the one Tarjan implementation to retain;
- Pipeline still contains the old `TypeCheckerV1`, `CatalogV1`, and
  `PackGraphV1` semantic path, but it is compiler-RED because those exports are
  gone. It is a stale consumer scheduled for deletion in Ticket 07, not a
  second implementation to repair;
- Algebra's raw-HIR normalizer is the current owner being replaced in place by
  sealed Sema-input lowering in Ticket 05; adding a parallel V2 normalizer file
  while retaining both public seams is prohibited;
- frontend-contract still glob-re-exports internal crates and therefore remains
  the duplicate public-authority surface to delete in Ticket 07.

This scan closes the ambiguity between implemented producer contracts and
missing consumers. Tickets 03/04 must finish owner proof and Sema reachability;
Tickets 05/07 migrate the existing Algebra/Pipeline owners in place. No ticket
is authorized to restore deleted V1 exports merely to make the stale consumers
compile.

## SS-03D / SS-04B Reuse Boundary

Types now owns the resolved relational-EXISTS base effect as well as its root
and capture carrier. Sema passes only exact root/capture facts; it must not
reintroduce a caller-selected effect enum. Binder remains the only lexical
owner for exact capture `(owner, ordinal)` identity, which Sema compares at
ingress rather than teaching Types a second resolver.

Inline View must reuse the single Modules validator phase/proof. The required
Sema pre-resolution plan, dependency-first materialization cache, and retained
canonical direct-view edge are consumer artifacts of that proof—not a second
topology implementation. A local `query-ref` View has no typed relation row;
it rejects as a source until a Modules/Sema owner contract provides one.

## SS-03E / SS-04D Order-Authority Boundary

The new Types `PriorRelation` positive reuses an earlier authored source row;
it is not an Inline View ordering mechanism. Modules alone owns View topology
and its opaque dependency-first proof. Sema may project that proof into one
private typed pre-evaluation input, but Types must validate and consume the
exact root order before it prewalks relation rows. This keeps topology out of
Types, type construction out of Sema, and prevents a reverse-authored View
chain from becoming an order-dependent compatibility bug.

## SS-04E EXISTS Consumer Boundary

Sema now consumes existing Binder capture proof and existing Types facts to
materialize the semantic EXISTS form. It does not add a resolver, a type pass,
or a synthetic relation item. `TypedRelationExprV2` is the nested registry
value; `TypedRelationV2` remains reserved for a real item and must not be
forged with a fake item ID. The registry is transaction-owned and exact-root
keyed, while all outer expression facts remain in the existing document map.

## SS-03F / SS-04F Reuse Boundary — 2026-09-21

The Inline View repair reuses, rather than replaces:

- Modules `DocumentCatalogValidatorV2` predeclaration, optional exact probe,
  topology proof, canonical dependency order, deduplication, and finalization.
- Types `ResolvedTypeInputsV2` and its single kernel pass; the new plan is only
  a validated execution order, not an alternate row/type solver.
- Sema's existing query relation materializer, typed scan/output constructors,
  and catalog edge equality check.

Forbidden duplication remains explicit: no Sema DFS/Tarjan, no name-keyed View
cache, no View row synthesis, no second Types traversal, no fallback from a
resolved View to a host relation, and no downstream HIR reconstruction. The
only new Modules API preserves its existing resolver semantics while allowing
an absent catalog asset to remain a disjoint host-namespace candidate.

## SS-04G Composition Identity Reuse Boundary — 2026-09-21

The source identity repair reuses Binder's existing resolved local identity
and Sema's `ResolvedValueBindingV2`; it adds only a private Composition-only
seal at the public relation-source boundary. It does not add a second binder,
coordinate registry, source-name resolver, or parallel Composition output
map. `ResolvedCompositionSourceIdV2` is an opaque consumer carrier, not a new
identity authority.

## SS-04H Generated-Row Reuse Boundary — 2026-09-21

Generated SAMPLE/FIND rows reuse `RelationOutputContractV2` and its one
row/column alignment constructor. The only added origin is the exact existing
semantic producer item ID; no expression fact, relation-source identity,
runtime row, or second output builder is synthesized. The producer variants
remain the sole schema/capability gate.
