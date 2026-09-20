# 06 — Rule And Graph Logic Contracts

Status: `PLANNED`

Decision authority: [00 — Packet Scope And Pinned Decisions](./00-packet-scope-and-pinned-decisions.md)

## Findings Closed

F10, F17, F20's logic producer, F26's logic recursion bounds, and F30.

## Goal

Connect source rules and graph queries to logic using resolved predicate IDs and
explicit boundedness proofs rather than HIR-shape heuristics.

## Current-Head Reuse/Delete/Build Audit

| Class | Assets | Concrete action |
|---|---|---|
| `KEEP` | `BodyPolarityV1`, rule/program carriers, Tarjan SCC, stratification, recursive aggregate matrix, graph/witness/existence contracts | retain these algorithms and migrate their existing focused tests |
| `MIGRATE` | recursion graph keys, graph boundedness inputs, witness traversal | switch names to resolved IDs/spans and add checked budgets without replacing algorithms |
| `DELETE` | one-rule-at-a-time producer, callee-argument heuristic, waypoint-as-depth inference | remove at the pipeline/source adapter |
| `NEW` | document-wide resolved rule-program adapter, exact dependency edges, budget counter/proof fields | no second SCC/DFS/stratifier is allowed |

Existing `v3_logic_recursion.rs`, graph, witness, hellgate, and punchlist suites
remain the owner rails. New tests cover source-document reachability and missing
payloads; they must not restate current Tarjan positives under new filenames.

## Write Set

1. `crates/quanta-gqlang-v2-logic/src/recursion.rs`
2. `crates/quanta-gqlang-v2-logic/src/graph.rs`
3. `crates/quanta-gqlang-v2-logic/src/witness.rs`
4. `crates/quanta-gqlang-v2-logic/src/source.rs`
5. `crates/quanta-gqlang-v2-logic/src/diagnostic.rs`
6. `crates/quanta-gqlang-v2-logic/src/aggregate.rs`
7. `crates/quanta-gqlang-v2-logic/src/lib.rs`
8. `crates/quanta-gqlang-v2-logic/Cargo.toml` — integrator only; remove direct
   HIR edge and add sema
9. `/Cargo.lock` — integrator only if the edge cut changes it
10. `crates/quanta-gqlang-v2-pipeline/src/compile.rs` — integrator only
11. the exact logic and pipeline rule/graph tests listed below
12. `docs/plans/sep-21-gqlang-sota-frontend-reconstruction/receipts/06.md`
13. `crates/quanta-gqlang-v2-logic/tests/support/mod.rs` when repeated `sp`,
    `id`, and graph-reference helpers from touched tests are consolidated

`crates/quanta-gqlang-v2-sema/src/rule.rs` is a read-only input owned and frozen
by ticket 04. If it lacks resolved IDs, polarity, spans, or budget projection,
stop and amend 04; do not patch sema from this ticket.

## Rule Implementation

1. Accept only the complete `Vec<ResolvedRuleV2>` emitted by ticket 04; logic
   performs no lexical name, variable, call-kind, or arity resolution.
2. Validate that every dependency target ID belongs to the frozen program and
   that every edge retains polarity/span.
3. Build one document rule program before Tarjan/stratification.
4. Change the existing dependency graph/Tarjan inputs from names to stable IDs.
5. Reject recursive aggregate and negation cycles from the retained edge
   polarity.
6. Preserve head/body/edge spans on deterministic cycle diagnostics.
7. Migrate the existing recursive-aggregate matrix carrier to fixed
   `RejectAllRecursiveAggregatesV2` policy. Remove default `min`/`max`
   admission and public `with_admitted` extension from the V2 path; this packet
   has no configurable recursive aggregate exception.

## Graph Implementation

1. Keep waypoint constraints as path predicates only.
2. Require explicit depth for cyclic relation traversal unless a distinct
   acyclic proof exists.
3. Preserve max path length, witness cap, dedup identity, tie-break, relation
   shape, and endpoint contract in `GraphQueryContractV2`.
4. Use iterative graph traversal or a checked graph-depth budget.
5. Preserve nested FIND contracts inside CHECK.

## Required Contracts

```rust
pub struct ResolvedRuleV2 {
    predicate: PredicateIdV2,
    head_terms: Vec<TypedExprV2>,
    body: TypedBoolExprV2,
    dependencies: Vec<ResolvedPredicateEdgeV2>,
    span: SpanV1,
}

pub struct ResolvedPredicateEdgeV2 {
    target: PredicateIdV2,
    polarity: BodyPolarityV1,
    span: SpanV1,
}

pub struct RuleProgramContractV2 {
    rules: Vec<ResolvedRuleV2>,
    proof: StratificationProofV2,
}
```

`GraphQueryContractV2` must expose exact witness bounds through read-only
accessors. A Boolean `bounded=true` is insufficient.

## Step-By-Step Procedure

1. Verify ticket 04 produced predicate IDs and predeclared every source rule
   name; reopen 04 if the frozen input contract is incomplete.
2. Validate and consume ticket 04's resolved dependency edges; do not rewalk
   expression HIR or re-resolve calls.
3. Construct one `RuleProgramContractV2` from all document rules.
4. Change Tarjan/stratification inputs from names to stable predicate IDs.
5. Preserve user-facing names only for deterministic diagnostics/rendering.
6. Reject recursive aggregate and negation edges before producing a success
   proof.
7. Remove `lower_source_rule`, `root_symbol_name`, and call-argument dependency
   heuristics.
8. Change graph boundedness so only explicit depth or an independent acyclic
    proof satisfies cyclic traversal.
9. Retain via/through as ordered path constraints.
10. Make DFS/Tarjan iterative or charge every traversal step to graph budget.
11. Return exact graph and rule contracts to pipeline/planning.

The private logic counter receives `max_rule_graph_nodes`,
`max_rule_graph_edges`, `max_rule_steps`, `max_witness_depth`, and
`max_witnesses` from the immutable frontend budget. It charges each rule node,
dependency edge, SCC/stratification visit, graph traversal step, and emitted
witness before work/allocation. Exact limit succeeds; the next charge rejects.
No surface, sema, algebra, or planning counter is reused or recharged here.

## Test Files

1. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_logic_recursion.rs` — document program, ID-based SCC, polarity, spans
2. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_logic_source.rs` — resolved-edge adapter and no raw-HIR/name fallback
3. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_logic_hellgate.rs` — negative/aggregate cycles and invalid targets
4. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_graph_query_depth_waypoint_conflict.rs` — counterexamples and explicit bounds
5. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_graph_query_hellgate.rs` — graph budget exhaustion
6. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_logic_witness.rs` and `crates/quanta-gqlang-v2-logic/tests/test_endpoint_conflict.rs` — exact retained contract fields
7. `[UPDATE]` `crates/quanta-gqlang-v2-logic/tests/v3_logic_punchlist.rs` — mutation cases without a second suite
8. `[ADD]` `crates/quanta-gqlang-v2-pipeline/tests/v3_source_rule_document.rs`
9. `[ADD]` `crates/quanta-gqlang-v2-pipeline/tests/v3_graph_contract_retention.rs`

Existing SCC/stratification/witness positives remain in their owner files. The
two new pipeline files prove public document reachability only.

Audited existing-test disposition:

1. Strengthen `test_endpoint_conflict.rs` to assert
   `logic.graph_endpoint_unexpected`, endpoint role, and exact span; never
   replace the assertion with printing.
2. Add an exact `logic.non_monotone_recursion.negated_recursion` fixture.
3. Delete only the proven redundant zero-path, wrapper quantifier,
   order-erased negation-chain, acyclic-composition, and duplicate bounded-taint
   tests after their stronger owner rails are green.
4. Delete unconsumed `DedupIdentityV1`, `WitnessTieBreakV1`, and
   `ExistencePolarityV1::keyword` spelling pins/accessors; retain the
   `from_keyword` behavioral round trip.
5. Normalize touched lint headers and extract owner-local helpers only after
   semantic migration.

## Exact Expected Outcomes

| Source | Outcome |
|---|---|
| `rule r : x if r(x)` | recursive positive SCC |
| `rule r : x if not r(x)` | `logic.negation_cycle` |
| `rule r : x if count(r(x)) > 0` | recursive aggregate reject |
| recursive `min(r(x))` / `max(r(x))` | same recursive aggregate reject |
| nonrecursive `count/min/max` | success when ordinary aggregate context allows it |
| `rule r : x if r` | bare predicate reject |
| `rule r : x if unknown + 1 > 0` | unresolved rule variable |
| two rules `r -> s`, `s -> r` | one SCC |
| cyclic FIND with `via=x`, no depth | unbounded witness reject |
| same FIND with explicit depth | bounded contract with exact depth |

## Exact Verification

```bash
cargo test -p quanta-gqlang-v2-logic
cargo test -p quanta-gqlang-v2-pipeline --test v3_source_rule_document
cargo test -p quanta-gqlang-v2-pipeline --test v3_graph_contract_retention
cargo clippy -p quanta-gqlang-v2-logic -p quanta-gqlang-v2-pipeline --all-targets -- -D warnings
python3 tools/architecture/check_architecture.py
git diff --check
```

## Forbidden Shortcuts

1. extracting dependencies from symbol names or callee text
2. stratifying one source rule at a time
3. treating predicate call arguments as predicate dependencies
4. downgrading unsupported calls to nonrecursive success
5. using waypoint count as any termination bound
6. recomputing graph contracts in planning

## RED

```text
rule r : x if r(x)             -> recursive=false
rule r : x if not r(x)         -> recursive=false
rule r : x if count(r) > 0     -> recursive aggregate accepted
```

`via`/`through` also incorrectly turns a cyclic traversal into a bounded one.
Duplicate quantifier, free-variable, bare-predicate, and call-kind REDs belong to
ticket 04 and must already be green before this ticket starts.

## Tests

1. positive self recursion
2. negative self recursion reject
3. aggregate recursion reject
4. mutual recursion SCC
5. stratified cross-SCC negation
6. resolved dependency target/polarity validation
7. cycle-before-waypoint and cycle-after-waypoint counterexamples
8. explicit depth positive
9. witness contract field retention
10. deterministic edge-span diagnostics
11. one public document test proving ticket 04's resolved predicate call reaches
    the document-wide SCC without logic re-resolution

## Mutation Cases

1. ignore call callee
2. downgrade Aggregated to Positive
3. stratify rules independently
4. treat waypoint count as depth
5. discard witness cap or tie-break

## Definition Of Done

1. no logic API consumes raw Rule/Graph HIR
2. source mutual recursion reaches Tarjan through the public document API
3. illegal recursion cannot silently become nonrecursive
4. graph boundedness is explicit and retained downstream

## Dependencies

Requires 04. May run in parallel with 05 after sema contracts are frozen.
