# P5 — logic + algebra fail-open test audit

Scope (only these may be edited):

- `crates/quanta-gqlang-v2-logic/tests/` (10 files + `tests/support/mod.rs`)
- `crates/quanta-gqlang-v2-logic/src/{graph.rs,witness.rs}` `#[cfg(test)]` blocks
- `crates/quanta-gqlang-v2-algebra/tests/` (4 files)
- `crates/quanta-gqlang-v2-algebra/src/normalize.rs` `#[cfg(test)]` block

Method. Three passes: (1) static review of every test body; (2) a five-way
parallel independent audit (four scoped reviewers plus one adversarial
cross-checker that had to name the concrete production mutation each finding
lets survive); (3) consolidation, where every proposed fix was re-verified by
reading production source and every applied fix was mutation-tested. No oracle
was changed; only fail-open → fail-closed conversions were applied.

## 1. Findings

Severity: high = a real verification hole on a claimed contract; medium = weak
oracle; low = coverage bookkeeping / style.

| file:line | class | severity | action | one-line |
|---|---|---|---|---|
| `logic/src/graph.rs` (endpoint matrix) | E self-oracle | **high** | **fixed** | `every_subkind_admits_its_required_endpoints` / `every_required_endpoint_is_enforced_by_exact_role` / `every_subkind_rejects_every_unexpected_endpoint` built their expectation from production `required_endpoints(kind)` itself. A narrowed set made the first two vacuous, so `Reachability`/`Flow`/`Path`/`Summary` required-roles had zero independent coverage. Added an authored `required_expectations()` table plus `required_endpoint_table_matches_the_authored_expectation`, and drove all three loops off the table. |
| `logic/tests/v3_logic_source.rs:80` (`assert_invalid`) | E self-oracle | **high** | **fixed** | `suffix = reason.code()` compared against a diagnostic also built from `reason.code()` — a tautology; a spelling mutant in `RuleProgramInvalidReasonV1::code()` moved both sides. Replaced with a test-local `expected_invalid_suffix()` literal table. |
| `logic/src/graph.rs` (keyword table) | E/H under-test | **high** | **fixed** | `subkind_keywords_are_stable` covered only 3 of 7 arms (`flow`/`slice`/`path`/`summary` unverified). Now a full table with a width assert against `ALL_KINDS`. |
| `logic/tests/v3_logic_source.rs` (5 sites) | H cardinality | **high** | **fixed** | "one projected edge"/"exact edges" claims never asserted `body().len()`; duplicate projected edges passed. Guards added for the self-recursion, mutual-pair, nonrecursive, negated, and aggregate rails. |
| `logic/tests/v3_logic_source.rs` (9 sites) | H/D' cardinality | **high** | **fixed** | `.first()`/`.next()` chosen for a "one member"/"one document member"/"forged member retained" claim without asserting the count, and `to_program()` compared by name lookup only. Length/membership guards added. |
| `logic/tests/v3_rule_scc_hellgate.rs:385` | B silent skip | medium | **fixed** | `if let Some(rule) = rules.get(idx)` dropped an out-of-range permutation index, shrinking the program silently. |
| `logic/src/witness.rs:313` (`rejection_order_is_deterministic`) | E weak oracle | medium | **fixed** | Only single-defect cases: a mutant reordering the cap check after the path check survived. Added four cross-cases (cap presence/degeneracy/budget each paired with a later path defect). |
| `logic/tests/v3_logic_source.rs:129` (`body_literal`) | H cardinality | medium | **fixed** | `.find(..).expect(..)` returned the first match, so a duplicated body literal for the same target passed. Added an exactly-one check. |
| `logic/tests/v3_logic_source.rs:174` | H table gap | medium | **fixed** | The "no EXISTS shape projects a logic rule" table omitted `ExistsWithMultiplicativeRight`, a real relational-EXISTS variant. Added. |
| `algebra/tests/v3_algebra_normalization.rs` (`assert_single_key_sort`) | E weak oracle | medium | **fixed** | The key's expression identity was never pinned (only direction / null-order / value type), so retargeting the key at a same-typed column passed. Now resolves the key to its authored field path. |
| `algebra/tests/v3_algebra_normalization.rs` (`two_key_group…`) | E weak oracle | medium | **fixed** | "r.id then r.value" authored order was never pinned — any internally consistent order (including reversed) passed. Now asserts the resolved segment names. |
| `algebra/tests/v3_algebra_normalization.rs` (4 `zip` loops) | H zip truncation | medium | **fixed** | `.zip()` truncates to the shorter side; expression↔column loops had no width guard, and the shared helper was only guard-by-caller. Width asserts added, helper made self-guarding. |
| `algebra/tests/v3_algebra_probe.rs:128` | E weak oracle | medium | **fixed** | The NOT-EXISTS rail claimed the "same exact captures" but checked only binding name/kind. Now also pins ordinal, source identity, the resolved Binding fact, and the reference's checked location. |
| `algebra/tests/v3_algebra_mutants.rs:28` | E weak oracle | low | **fixed** | `scan_source().is_some()` accepted a wrong/foreign source. Now matches the exact sealed relation identity. |
| `algebra/tests/v3_algebra_normalization.rs:647,726` | H cardinality | medium | **fixed** | Single-key sort rails never asserted `order_keys().len() == 1`. |
| `algebra/tests/v3_algebra_normalization.rs:575` | H cardinality | low | **fixed** | count(*) asserted the first column origin but not the column count. |
| `logic/tests/v3_logic_recursion.rs` (50k node budget) | E weak oracle | medium | **fixed** | The rejected `span` was discarded (`..`), leaving "refused at the last vertex" unverified. Now pinned to the defining rule's location. |
| `logic/tests/v3_rule_scc_hellgate.rs:123` | E weak oracle | low | **fixed** | The raw body-literal cap checked only code/span, not the `Edges` dimension or limit. |
| `logic/tests/v3_logic_witness.rs:124` | H table gap | low | **fixed** | The 7-row dedup table had no width guard. |
| `logic/tests/v3_graph_query_hellgate.rs:283` | H table gap | low | **fixed** | The sealed-limit `cases` matrix had no width guard. |
| `logic/src/witness.rs:395,456` | H table gap | low | **fixed** | The 6-row reason and code tables had no width guards. |
| `logic/tests/v3_rule_aggregate_policy.rs:177` | H cardinality | low | **fixed** | `.first().and_then(..)` claimed "one body reference" while taking only the first. |
| `logic/tests/v3_logic_mutation_rails.rs:288` | E vacuous match | low | **held** | `matches!(contract.dedup(), Endpoints \| NodeSequence \| NodeSet)` names every variant of a 3-variant enum: it cannot fail. Tightening it here would duplicate (and change) the per-kind oracle owned by `v3_logic_witness.rs`. Reported, not silently deleted. |
| `logic/tests/v3_logic_source.rs:335,391,598` | E self-recompute | low | **held** | `contract.stratification() == stratify(to_program(), budget)` compares two results of the same production functions, so the equality itself cannot fail. Removing it is fail-open→weaker, not a hardening; replacing it needs an independently authored program literal. Reported. |
| `algebra/src/normalize.rs:1315` | E weak oracle | low | **held** | `root.predicate().is_some()` does not pin the ON identity; the integration rail at `v3_algebra_normalization.rs:304` already pins the key. |
| `algebra/src/normalize.rs:1061,1475` | E self-comparison | low | **held** | `root.output() == lowered.output()` compares the root against a root-output clone. |
| `algebra/tests/v3_algebra_probe.rs:250` | E wrong-oracle risk | — | **reverted** | A reviewer proposed asserting the nested capture set is *disjoint* from the outer set. Instrumenting the fixture showed the nested edge legitimately shares the inner reference site with an outer edge, so the disjointness assert is a **wrong oracle**. Left as a documented note instead. |
| `logic/tests/support/mod.rs:85,103` | H coverage gap | medium | **held** | `direction`/`summary_facets` are hard-coded `None`/empty, so `GraphQueryContractV2::direction()`/`summary_facets()` retention is untested anywhere in the crate. Closing it needs new fixtures plus a newly authored expectation (an oracle addition). |
| 4 `ALL_KINDS` arrays + `cases` | H unanchored table | low | **held** | `GraphQueryKindV1` has no production `ALL`/count, so a duplicated 7-element array is not compiler-coupled to the enum. A test-local exhaustive `match` anchor is possible but is a convention decision, not a mechanical repair; the touched matrices now carry explicit width asserts. |
| `logic/src/graph.rs` (`from_role` non-endpoint arms) | H coverage gap | medium | **held** | `Sanitizer`/`Scope`/`Window`/`Frontier` are never built into a FIND document, so "these are not endpoint roles" has no rail. Needs new fixtures with a newly authored expectation. |
| `logic/src/witness.rs:228` (`WitnessTieBreakV1::Lexical`) | H coverage gap | low | **held** | Production never constructs `Lexical`; adding a rail would assert a value production does not emit. |
| `ReferenceCompositionBodyV2::{Graph,Check,Explain}` | H coverage gap | low | **held** | No algebra/logic test lowers these bodies; `Relational` is the only one exercised. Needs new tests. |
| `v3_rule_scc_hellgate.rs:414`, `v3_logic_recursion.rs:318` | H permutation claim | medium | **held** | "every permutation"/"every authored order" claims check 5 of 120 and 3 of 48 combinations. Enumerating them is straightforward but expands the oracle surface well beyond fail-closed conversion; deferred. |

Non-findings verified clean (no action):

- `logic/tests/v3_logic_recursion.rs:352-476` — the 343-graph independent
  reachability/strata oracle: full 7³ space, `members[0]` provably non-empty
  (`reachable[n][n]` seeded true), `.any()` only on the true branch, both
  admit/reject branches fail closed.
- `logic/tests/v3_logic_source.rs` — `rejection()` panics on `Ok`; `only_call`
  asserts `len()==1`; `only_resolved_call_fact` asserts exactly one match.
- `algebra/tests/v3_algebra_mutants.rs:132-141` — guarded by explicit lengths,
  so the `Option`-vs-`Option` comparison cannot go vacuous.
- `logic/tests/v3_rule_scc_hellgate.rs` — literal codes/messages/spans plus the
  permutation-completeness guard; `algebra/src/normalize.rs` test module has no
  `if let`-skip, `.ok()`, or `unwrap_or*` oracle and every loop is length- or
  literal-anchored.
- All 9 `ReferenceRuleFixtureV2`, 3 `ForgedRuleMembershipMutationV2`, 7
  `ForgedRuleFactMutationV2`, 4 `ForgedCaptureEdgeMutationV2`, and 9
  `ForgedRuleCallWrapperV2` variants are exercised in scope.
- No `#[ignore]` in scope; no fixture/`.expect`/`.tsv`/`docs/` edits.

## 2. Files changed

1. `crates/quanta-gqlang-v2-logic/src/graph.rs` (`#[cfg(test)]` only) —
   authored `required_expectations()` table + new
   `required_endpoint_table_matches_the_authored_expectation` test; the three
   matrix loops now consume the authored table; full subkind keyword table with
   an `ALL_KINDS` width assert; endpoint-role table width assert.
2. `crates/quanta-gqlang-v2-logic/src/witness.rs` (`#[cfg(test)]` only) —
   cross-case rejection-order cases; width guards on the reason and code tables.
3. `crates/quanta-gqlang-v2-logic/tests/v3_logic_source.rs` — test-local
   `expected_invalid_suffix()` literal table; cardinality guards at 14 sites;
   body-literal uniqueness; `ExistsWithMultiplicativeRight` added; wrapper table
   width assert. A documented note that the nested/outer capture sets are not
   disjoint by site.
4. `crates/quanta-gqlang-v2-logic/tests/v3_logic_witness.rs` — dedup table width.
5. `crates/quanta-gqlang-v2-logic/tests/v3_graph_query_hellgate.rs` — matrix
   width assert.
6. `crates/quanta-gqlang-v2-logic/tests/v3_rule_scc_hellgate.rs` — `.expect` +
   completeness assert in the permutation builder; `Edges`/limit pinning.
7. `crates/quanta-gqlang-v2-logic/tests/v3_rule_aggregate_policy.rs` — exact
   rule/body cardinality.
8. `crates/quanta-gqlang-v2-logic/tests/v3_logic_recursion.rs` — rejected-span
   pinning in the 50k node-budget rail.
9. `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_normalization.rs` —
   width guards (4 `zip` loops + count(*) columns), self-guarding helper,
   authored key order, key field-path identity, `order_keys` cardinality.
10. `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_probe.rs` — full NOT-EXISTS
    capture payload; documented non-disjointness note.
11. `crates/quanta-gqlang-v2-algebra/tests/v3_algebra_mutants.rs` — exact scan
    source identity.

The oracle in every applied case is unchanged: the same diagnostics, fields, and
expected values are compared. Only the "silently shorter/absent/other-identity
value still satisfies the claim" holes were closed.

## 3. Held items and why

1. **Vacuous `matches!` union (`v3_logic_mutation_rails.rs:288`)** — the union
   names all three variants. Deleting it would be a weakening, and pinning the
   per-kind value here would duplicate the oracle already owned by
   `v3_logic_witness.rs`; reported rather than silently removed.
2. **Self-recompute equalities (`v3_logic_source.rs:335,391,598`, `normalize.rs:1061,1475`)**
   — these compare production output to production output. Replacing them needs
   an authored expected literal (an oracle addition), so they are reported.
3. **Coverage gaps needing new fixtures/expectations** — `direction`/
   `summary_facets` retention, non-endpoint `from_role` arms, three
   `ReferenceCompositionBodyV2` bodies, `WitnessTieBreakV1::Lexical`. Each
   requires authoring a new oracle, which the audit mandate excludes.
4. **Permutation enumeration (120 / 48)** — straightforward to write but a large
   oracle expansion; deferred rather than folded into a hardening pass.
5. **`ALL_KINDS` anchoring** — no production `ALL`/count exists; adding a
   test-local exhaustive-match anchor is a repository-convention decision.
6. **Wrong-oracle proposal (capture disjointness)** — instrumented and disproved;
   kept as documentation so the next auditor does not re-add it.

## 4. Test results

| stage | command | result |
|---|---|---|
| baseline | `cargo test -p quanta-gqlang-v2-logic` | green — 110 passed, 0 failed, 0 ignored |
| baseline | `cargo test -p quanta-gqlang-v2-algebra` | green — 62 passed, 0 failed, 0 ignored |
| final | `cargo test -p quanta-gqlang-v2-logic` | green — **111** passed, 0 failed, 0 ignored (+1 new owner test) |
| final | `cargo test -p quanta-gqlang-v2-algebra` | green — 62 passed, 0 failed, 0 ignored |

No test was deleted, renamed, ignored, or weakened. One transient
`error[E0463]: can't find crate for quanta_gqlang_v2_sema` in `--doc` during a
concurrent multi-worker build was target-artifact eviction, not a defect;
`--doc` alone is green.

Note: a parallel worker committed the working tree mid-audit (`9040a8a Close Sep
26 test audit proof gaps`). All changes above are present in `HEAD` and verified
there by inspection; the suite results quoted are from the committed state.

## 5. Mutation verification

Each high-value fix was checked by injecting the mutant it must kill.

| mutant | rail | result |
|---|---|---|
| `GraphQueryKindV1::Flow => "flow_mutant"` | `subkind_keywords_are_stable` | FAILED (invisible before) |
| `to_program()` duplicates its first member | `v3_logic_source.rs` rails | 5 tests FAILED (all passed before) |
| `required_endpoints(PointsTo)` widened with `Source` | `required_endpoint_table_matches_the_authored_expectation` + 2 matrix rails | 3 tests FAILED (invisible before) |
| Summary case dropped from the sealed-limit matrix | `every_subkind_admits_at_the_sealed_depth_limit` | FAILED (caught by the new width guard) |

The Summary-drop and widened-PointsTo probes also surfaced (and forced me to
revert) two of my own temporary production mutations, which is why the final
tree has no production residue (`git grep` confirms the original
`PointsTo => &[Reference, Pointee]` and `Summary => &[Callable]` arms).
