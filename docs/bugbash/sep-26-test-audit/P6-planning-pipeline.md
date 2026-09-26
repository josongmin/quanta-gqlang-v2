# P6 — planning + pipeline test audit (fail-open survey)

Audit basis: working tree plus concurrent owner edits; static review only, no `just ci`
/ `just qualify-local` (integrator-only). P6-owned scope only.

Scope (audited):

- `crates/quanta-gqlang-v2-planning/tests/` — 7 files + shared `tests/support/mod.rs`
- `crates/quanta-gqlang-v2-pipeline/tests/` — 3 files
- `crates/quanta-gqlang-v2-planning/src/plan.rs` — `#[cfg(test)]` only
- `crates/quanta-gqlang-v2-pipeline/src/compile.rs`, `src/execution.rs` — `#[cfg(test)]` only

Method: round 1 static grep-class census; round 2 parallel deep audit by five
independent reviewers (one per file group), every reported site re-verified against
the current source; round 3 supplemented the held items that could be grounded in
independent owner tests. Only fail-open → fail-closed conversions applied; no oracle
subject was substituted. Every edit was re-run against the owner tests.

## 1) Round 1 findings (static pass) — all fixed

| file:line | class | severity | one-line |
|---|---|---|---|
| `pipeline/tests/v3_compiled_document_assembly.rs:2171` | A + F | high | `for item … if let Some(relation)` asserted closure only when a relation existed; now counts closed relations and requires exactly one. |
| `pipeline/tests/v3_compiled_document_assembly.rs:2219` | A | high | `if let Some(body) = composition.body_relation()` skipped the closure proof for SAMPLE/Query bodies; now `.expect(..)`. |
| `pipeline/tests/v3_analysis_document_assembly.rs:241` | F | high | partial-mask `items().all(BlockedByDiagnostics)` was vacuously true on an empty item list; added `items().len() == 3`. |
| `pipeline/tests/v3_analysis_document_assembly.rs:342` | F | medium | `len==len` + per-item loop could pass `0==0`; non-empty guard added. |
| `pipeline/tests/v3_analysis_document_assembly.rs:384` | F | medium | `0..items().len()` loop could run zero times; non-empty guard added. |
| `pipeline/tests/v3_compiled_document_assembly.rs:1226` | F | medium | same `0..len` comparison shape; non-empty guard added. |
| `planning/tests/v3_planning_hellgate.rs:996` | F | low | `producers().all(ordinal==i)` vacuously true for `[]`; non-empty guard added. |
| `pipeline/src/execution.rs:2981` (`#[cfg(test)]`) | F | low | shared `assert_all_digests_distinct(&[])` passed; empty-set rejection added. |

## 2) Round 2 findings (parallel deep audit)

### Fixed

| file:line | class | severity | one-line |
|---|---|---|---|
| `planning/tests/v3_planning_hellgate.rs:1024` | D'/H | medium | composition producers compared by `item_id` only; now also `name_location` + `relation_output` against the sealed bindings. |
| `planning/tests/v3_planning_hellgate.rs:659` | H | low | CHECK option-matrix `Ok` arm checked presence only; now full `rule`/`scope`/`pattern`/`nested_graph` retention. |
| `planning/tests/v3_item_hellgate.rs:130` | H | medium | FIND retention compared 9 of 12 fields; added `direction`/`summary_facets`/`references` vs the sealed contract. |
| `planning/tests/v3_item_hellgate.rs:234` | H | low | CHECK policy compared `rule()` by location only; now full `rule`/`scope`/`pattern`. |
| `planning/tests/v3_cost_classification.rs:420` | H | medium | CHECK alert arm asserted only `!is_ordered()`; added `eliminates_duplicates()` + `cardinality == Set`. |
| `planning/tests/v3_planning_truth.rs:470` | H | low | “every accessor” rail sampled 3 fields; added whole-`SemanticPlanV2` equality. |
| `planning/tests/v3_explain_provenance.rs:262` | E | low | `families().len() == nodes` could hold as `0==0`; added `nodes == 5`. |
| `pipeline/tests/v3_compiled_document_assembly.rs:594` | F | medium | `for item in compiled.items()` had no count assert; added `len == fixture.item_ids().len()`. |
| `pipeline/tests/v3_compiled_document_assembly.rs:2195` | D' | low | schema-gated loop: schema-bearing counter **and** per-binding `schema().is_none() ⇒ relation().is_none()` invariant. |
| `pipeline/tests/v3_compiled_document_assembly.rs:2306` | H | low | formatter rail used two `contains` probes; now the exact canonical text. |
| `pipeline/tests/v3_analysis_document_assembly.rs:213` | H | low | “blocks every item” inspected `items()[0]` only; now loops every item. |
| `pipeline/tests/v3_analysis_document_assembly.rs:379` | G | low | analysis result was `expect`-discarded; now bound and its blocked diagnostics asserted. |
| `pipeline/tests/v3_composition_source_query.rs:177` | E | low | `"query"` arm only matched the shared `Query` variant; now asserts a `Query` producer. |
| `pipeline/src/execution.rs:3678` (`#[cfg(test)]`) | E | low | scalar projection asserted only `Int`; now a full 13-arm `ScalarTyV1` table. |

### Round 3 — held items supplemented (grounded in independent owner tests)

| file:line | class | severity | supplement + grounding |
|---|---|---|---|
| `planning/tests/v3_item_hellgate.rs:275` | H | medium | rule-fixture policy matrix: added `AggregatedNonRecursiveDependency` (strata 2, no recursive members). Grounding: `logic/tests/v3_logic_source.rs` `aggregated_nonrecursive_dependency_projects_exact_edge_and_stratification` pins `stratum_count() == 2` and `recursive().is_empty()`. The matrix now covers **all 7 admitted** `ReferenceRuleFixtureV2` fixtures; the 2 self-recursion fixtures are refused at the sole ingress and are correctly absent. |
| `planning/tests/v3_capability_derivation.rs:197` + `v3_item_hellgate.rs:491` | H | medium | reanchored-EXISTS matrix: added `ExistsWithMultiplicativeRight` (capability `{RelationScan, Join, ExistenceCheck}`) and `ConjunctiveExistsPair` (`{RelationScan, ExistenceCheck}`). Grounding: the explain rail pins `[SemiJoin, Scan, Join, Scan, Scan]`; `algebra/src/normalize.rs` pins the conjunctive pair to `[SemiJoin, SemiJoin, Scan, Scan, Scan]` with a SemiJoin root. `NegatedExists`/`DisjunctiveExistsPair` are pinned as algebra rejections (`algebra.invalid_stage_contract`) and are correctly absent; `ExistsWithBoolConjunct` stays covered as a Filter-root case. All plannable EXISTS families are now in the matrix. |
| `planning/tests/v3_item_hellgate.rs:459` | H | low | plan-policy vocabulary: added the `Composition` arm via `composition_input(Relational)`, closing the “closed per arm” claim over all 7 `PlanPolicyV2` variants. |
| `pipeline/tests/v3_compiled_document_assembly.rs:1199` | E | low | inline-View rail: added a negative control (unknown relation in `select=` must reject with `sema.unresolved_reference`), proving the body traversal is not skipped. |
| `pipeline/tests/v3_compiled_document_assembly.rs:402` | E | low | source-order rail: added a grounded three-item document (`MINIMAL_SOURCE`) whose compiled order must equal the authored document order, so ordering is discriminated, not just membership. |
| `pipeline/tests/v3_compiled_document_assembly.rs:1506` | H | low | graph-body references: replaced the first-target shape probe with a full role+target comparison (composition source binding + host `db.write` sink). |

### Still held (2)

| file:line | class | severity | reason |
|---|---|---|---|
| `planning/tests/v3_planning_hellgate.rs:995` | H | medium | composition body policy is compared by `keyword()` only. A full compare must reconstruct each family's body policy (the Relational arm needs a lowered traversal), i.e. re-implement classification inside the test; holding to avoid substituting a hand-built oracle. Recommend an owner-approved helper that classifies a sealed composition body. |
| `planning/tests/v3_explain_provenance.rs:172` | redundant | low | `assert_eq!(planned.explain().families(), families.as_slice())` is tautological (`families` came from the same plan). The real oracle is the exact literal family vector earlier in the test, so this is dead code, not fail-open. |

### Verified fail-closed (representative)

- Shared `planning/tests/support/mod.rs` audited: every builder propagates `Result`
  via `?`/`map_err`; `graph_references` and every `*_policy`/`item_location` match is
  exhaustive with no `_` arm; `deepest_node` is an independent terminating walk.
- Golden TSV gate: 15 data rows, per-row `fields.len()==7` + exact 3-digest equality + trailing `assert_eq!(selected, 15)`.
- 8-mask authority matrix: `0..=0b111` is statically 8 iterations, every branch asserts; the empty-document rail's emptiness assertions are the intended oracle.
- `assert_relation_closed`: `expect_references = true` at all call sites, so the non-empty control always runs.
- `shape_keys`/`operator_keys`/`schema_keys`: exhaustive matches (no `_` arm).
- `rule_program_receives_one_plan_and_exact_members`: `fixture.item_ids()[0]` already fails on empty.
- Planning vocabulary/matrix loops iterate literal arrays; keyword tables complete against current enums (Family 7/7, scalar 13/13, roles 12/12, ref-kinds 9/9, AlgebraFamily 9/9, shapes 16/16 + 3/3 call targets).
- `plan.rs` cfg(test): no early `return Ok(())`; every `?` / `other => Err(..)` fails the test.
- `compile.rs` cfg(test): no findings. No `#[ignore]` tests in scope.

## 3) Modified files (10)

1. `crates/quanta-gqlang-v2-planning/tests/v3_planning_hellgate.rs`
2. `crates/quanta-gqlang-v2-planning/tests/v3_item_hellgate.rs`
3. `crates/quanta-gqlang-v2-planning/tests/v3_cost_classification.rs`
4. `crates/quanta-gqlang-v2-planning/tests/v3_planning_truth.rs`
5. `crates/quanta-gqlang-v2-planning/tests/v3_explain_provenance.rs`
6. `crates/quanta-gqlang-v2-planning/tests/v3_capability_derivation.rs`
7. `crates/quanta-gqlang-v2-pipeline/tests/v3_analysis_document_assembly.rs`
8. `crates/quanta-gqlang-v2-pipeline/tests/v3_compiled_document_assembly.rs`
9. `crates/quanta-gqlang-v2-pipeline/tests/v3_composition_source_query.rs`
10. `crates/quanta-gqlang-v2-pipeline/src/execution.rs` (`#[cfg(test)]` only)

## 4) Test results

Baseline: `-p quanta-gqlang-v2-planning` green (148 tests); `-p quanta-gqlang-v2-pipeline` green (93 tests).

Final (after rounds 1–3):

- `cargo test -p quanta-gqlang-v2-planning` — **green**, 148 tests, 0 failed.
- `cargo test -p quanta-gqlang-v2-pipeline` — **green**, 93 tests, 0 failed (incl. 42 in `v3_compiled_document_assembly`).
- `cargo fmt --check` — all touched files clean (remaining diffs belong to other owners' files).

Note: during round 2 the shared `target/` directory was transiently corrupted by
concurrent worker builds (missing rlibs and an unrelated `quanta-gqlang-v2-logic`
unused-import break). Those runs were discarded; owner tests were re-run serially to
green and were not caused by P6 edits.
